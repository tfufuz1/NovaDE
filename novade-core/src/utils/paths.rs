//! Filesystem Path Utilities.
//!
//! This module provides helper functions for resolving standard XDG (X Desktop Group)
//! base directories and application-specific directories for configuration, data,
//! cache, and state files. It uses the `directories-next` crate.

use crate::error::{ConfigError, CoreError};
use directories_next::{BaseDirs, ProjectDirs};
use std::path::PathBuf;

/// The qualifier for the project, typically a reverse domain name.
const QUALIFIER: &str = "org";
/// The organization name associated with the project.
const ORGANIZATION: &str = "NovaDE";
/// The application name.
const APPLICATION: &str = "NovaDE";

/// Retrieves the XDG configuration base directory.
///
/// This is typically `~/.config` on Linux.
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the path to the config base directory.
/// * `Err(CoreError)` if the directory cannot be determined.
pub fn get_config_base_dir() -> Result<PathBuf, CoreError> {
    BaseDirs::new()
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or_else(|| {
            CoreError::Config(ConfigError::DirectoryUnavailable {
                dir_type: "Config Base".to_string(),
            })
        })
}

/// Retrieves the XDG data base directory.
///
/// This is typically `~/.local/share` on Linux.
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the path to the data base directory.
/// * `Err(CoreError)` if the directory cannot be determined.
pub fn get_data_base_dir() -> Result<PathBuf, CoreError> {
    BaseDirs::new()
        .map(|dirs| dirs.data_dir().to_path_buf())
        .ok_or_else(|| {
            CoreError::Config(ConfigError::DirectoryUnavailable {
                dir_type: "Data Base".to_string(),
            })
        })
}

/// Retrieves the XDG cache base directory.
///
/// This is typically `~/.cache` on Linux.
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the path to the cache base directory.
/// * `Err(CoreError)` if the directory cannot be determined.
pub fn get_cache_base_dir() -> Result<PathBuf, CoreError> {
    BaseDirs::new()
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .ok_or_else(|| {
            CoreError::Config(ConfigError::DirectoryUnavailable {
                dir_type: "Cache Base".to_string(),
            })
        })
}

/// Retrieves the XDG state base directory.
///
/// On Linux, this attempts to use `$XDG_STATE_HOME`, falling back to `~/.local/state`.
/// On other platforms, it uses the equivalent of a local data directory.
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the path to the state base directory.
/// * `Err(CoreError)` if the directory cannot be determined.
pub fn get_state_base_dir() -> Result<PathBuf, CoreError> {
    BaseDirs::new()
        .map(|dirs| {
            #[cfg(target_os = "linux")]
            {
                std::env::var("XDG_STATE_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| dirs.home_dir().join(".local/state"))
            }
            #[cfg(not(target_os = "linux"))]
            {
                // As per spec, using data_local_dir for non-Linux state.
                // Note: directories_next::BaseDirs doesn't have a dedicated `state_dir`.
                // `data_local_dir` is often `~/.local/share` on Linux,
                // `~/Library/Application Support` on macOS,
                // `%APPDATA%` (roaming) or `%LOCALAPPDATA%` on Windows.
                // The spec might need refinement here for true cross-platform state,
                // but adhering to current spec.
                dirs.data_local_dir().to_path_buf()
            }
        })
        .ok_or_else(|| {
            CoreError::Config(ConfigError::DirectoryUnavailable {
                dir_type: "State Base".to_string(),
            })
        })
}

/// Retrieves the application-specific configuration directory.
///
/// Path is typically like `~/.config/NovaDE/NovaDE` on Linux.
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the path to the application's config directory.
/// * `Err(CoreError)` if the directory cannot be determined.
pub fn get_app_config_dir() -> Result<PathBuf, CoreError> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or_else(|| {
            CoreError::Config(ConfigError::DirectoryUnavailable {
                dir_type: "App Config".to_string(),
            })
        })
}

/// Retrieves the application-specific data directory.
///
/// Path is typically like `~/.local/share/NovaDE/NovaDE` on Linux.
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the path to the application's data directory.
/// * `Err(CoreError)` if the directory cannot be determined.
pub fn get_app_data_dir() -> Result<PathBuf, CoreError> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.data_dir().to_path_buf())
        .ok_or_else(|| {
            CoreError::Config(ConfigError::DirectoryUnavailable {
                dir_type: "App Data".to_string(),
            })
        })
}

/// Retrieves the application-specific cache directory.
///
/// Path is typically like `~/.cache/NovaDE/NovaDE` on Linux.
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the path to the application's cache directory.
/// * `Err(CoreError)` if the directory cannot be determined.
pub fn get_app_cache_dir() -> Result<PathBuf, CoreError> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .ok_or_else(|| {
            CoreError::Config(ConfigError::DirectoryUnavailable {
                dir_type: "App Cache".to_string(),
            })
        })
}

/// Retrieves the application-specific state directory.
///
/// This combines the XDG state base directory with the project's path components.
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the path to the application's state directory.
/// * `Err(CoreError)` if the directory cannot be determined.
pub fn get_app_state_dir() -> Result<PathBuf, CoreError> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .and_then(|proj_dirs| {
            get_state_base_dir().ok().map(|base_state| base_state.join(proj_dirs.project_path()))
        })
        .ok_or_else(|| {
            // This error is returned if ProjectDirs::from was None OR if get_state_base_dir().ok() was None (meaning get_state_base_dir() returned Err).
            // Consider a more specific error message or error type if get_state_base_dir() itself fails.
            CoreError::Config(ConfigError::DirectoryUnavailable {
                dir_type: "App State".to_string(), 
            })
        })
}

/// Retrieves the system-wide configuration file path.
///
/// Path is typically `/etc/novade/config.toml`.
/// This can be overridden for testing using the `NOVADE_TEST_SYSTEM_CONFIG_PATH` environment variable.
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the path to the system configuration file.
/// * `Err(ConfigError::DirectoryUnavailable)` if the path cannot be determined (e.g. env var is invalid UTF-8).
pub fn get_system_config_path_with_override() -> Result<PathBuf, ConfigError> {
    match std::env::var("NOVADE_TEST_SYSTEM_CONFIG_PATH") {
        Ok(path_str) => {
            if path_str.is_empty() {
                 // Treat empty string as "use default" or error, this seems like an invalid override.
                 // For now, let's assume an empty path is not intended and fall back or error.
                 // Falling back to default:
                 Ok(PathBuf::from("/etc/novade/config.toml"))
            } else {
                Ok(PathBuf::from(path_str))
            }
        }
        Err(std::env::VarError::NotPresent) => {
            // Environment variable not set, use the default system path
            Ok(PathBuf::from("/etc/novade/config.toml"))
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            // Environment variable was set but contained invalid UTF-8
            Err(ConfigError::DirectoryUnavailable { // Reusing DirectoryUnavailable, consider a new variant if more fitting
                dir_type: "System Config Path (Invalid UTF-8 Env Var)".to_string(),
            })
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    // Mocking directories_next is complex. Tests will focus on:
    // 1. The constants used.
    // 2. The logic of get_app_state_dir if get_state_base_dir and ProjectDirs work.
    // Full path tests depend on the environment and `directories-next` behavior.
    // We assume `directories-next` works as intended.

    #[test]
    fn constants_are_set_correctly() {
        assert_eq!(QUALIFIER, "org");
        assert_eq!(ORGANIZATION, "NovaDE");
        assert_eq!(APPLICATION, "NovaDE");
    }

    // The following tests are highly dependent on the environment where they are run
    // and the behavior of `directories-next`. They might be flaky or fail in
    // constrained CI environments if HOME or other required env vars are not set.
    // Therefore, we'll mostly test that they *return something* or the correct error type
    // if `directories-next` can't find the dirs (which happens if HOME isn't set).

    #[test]
    fn get_config_base_dir_returns_ok_or_specific_error() {
        match get_config_base_dir() {
            Ok(path) => {
                println!("Config base dir: {:?}", path); // For debugging in test output
                assert!(path.is_absolute(), "Path should be absolute");
            }
            Err(CoreError::Config(ConfigError::DirectoryUnavailable { dir_type })) => {
                assert_eq!(dir_type, "Config Base");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn get_data_base_dir_returns_ok_or_specific_error() {
        match get_data_base_dir() {
            Ok(path) => {
                println!("Data base dir: {:?}", path);
                assert!(path.is_absolute());
            }
            Err(CoreError::Config(ConfigError::DirectoryUnavailable { dir_type })) => {
                assert_eq!(dir_type, "Data Base");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn get_cache_base_dir_returns_ok_or_specific_error() {
        match get_cache_base_dir() {
            Ok(path) => {
                println!("Cache base dir: {:?}", path);
                assert!(path.is_absolute());
            }
            Err(CoreError::Config(ConfigError::DirectoryUnavailable { dir_type })) => {
                assert_eq!(dir_type, "Cache Base");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
    
    #[test]
    fn get_state_base_dir_returns_ok_or_specific_error() {
        // This test is particularly environment-dependent for Linux XDG_STATE_HOME.
        // We're mostly checking if it resolves to *something* or the correct error.
        match get_state_base_dir() {
            Ok(path) => {
                println!("State base dir: {:?}", path);
                assert!(path.is_absolute());
                // On Linux, if XDG_STATE_HOME is not set, it should default to ~/.local/state
                #[cfg(target_os = "linux")]
                if std::env::var("XDG_STATE_HOME").is_err() {
                    if let Some(home) = std::env::var("HOME").ok().map(PathBuf::from) {
                         assert_eq!(path, home.join(".local/state"));
                    } else {
                        // If HOME isn't set, BaseDirs might fail, covered by the error case.
                    }
                }
            }
            Err(CoreError::Config(ConfigError::DirectoryUnavailable { dir_type })) => {
                assert_eq!(dir_type, "State Base");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }


    #[test]
    fn get_app_config_dir_returns_ok_or_specific_error() {
        match get_app_config_dir() {
            Ok(path) => {
                println!("App config dir: {:?}", path);
                assert!(path.is_absolute());
                // Check for application name, as path structure can vary by OS/env
                assert!(path.to_string_lossy().to_lowercase().contains(&APPLICATION.to_lowercase()));
            }
            Err(CoreError::Config(ConfigError::DirectoryUnavailable { dir_type })) => {
                assert_eq!(dir_type, "App Config");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn get_app_data_dir_returns_ok_or_specific_error() {
         match get_app_data_dir() {
            Ok(path) => {
                println!("App data dir: {:?}", path);
                assert!(path.is_absolute());
                assert!(path.to_string_lossy().to_lowercase().contains(&APPLICATION.to_lowercase()));
            }
            Err(CoreError::Config(ConfigError::DirectoryUnavailable { dir_type })) => {
                assert_eq!(dir_type, "App Data");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn get_app_cache_dir_returns_ok_or_specific_error() {
        match get_app_cache_dir() {
            Ok(path) => {
                println!("App cache dir: {:?}", path);
                assert!(path.is_absolute());
                assert!(path.to_string_lossy().to_lowercase().contains(&APPLICATION.to_lowercase()));
            }
            Err(CoreError::Config(ConfigError::DirectoryUnavailable { dir_type })) => {
                assert_eq!(dir_type, "App Cache");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
    
    #[test]
    fn get_app_state_dir_returns_ok_or_specific_error() {
        match get_app_state_dir() {
            Ok(path) => {
                println!("App state dir: {:?}", path);
                assert!(path.is_absolute());
                assert!(path.to_string_lossy().to_lowercase().contains(&APPLICATION.to_lowercase()));
                 if cfg!(target_os = "linux") && std::env::var("XDG_STATE_HOME").is_err() && std::env::var("HOME").is_ok() {
                    // Ensure the base state path component is also present if we are on Linux with default XDG structure
                    assert!(path.to_string_lossy().contains(".local/state"));
                }
            }
            Err(CoreError::Config(ConfigError::DirectoryUnavailable { dir_type })) => {
                assert_eq!(dir_type, "App State");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    // Example of how one might write more deterministic tests if `directories-next` was mockable
    // or if we controlled the environment variables fully.
    // For now, these tests rely on the system's environment.
    #[test]
    #[cfg(target_os = "linux")] // Example: Linux specific test for XDG_STATE_HOME
    fn test_get_state_base_dir_linux_xdg_state_home_respected() {
        let original_xdg_state_home = std::env::var("XDG_STATE_HOME").ok();
        let test_state_path = "/tmp/test-xdg-state-home";
        std::env::set_var("XDG_STATE_HOME", test_state_path);

        match get_state_base_dir() {
            Ok(path) => {
                assert_eq!(path, PathBuf::from(test_state_path));
            }
            Err(e) => {
                // Restore env var before panicking
                if let Some(val) = original_xdg_state_home {
                    std::env::set_var("XDG_STATE_HOME", val);
                } else {
                    std::env::remove_var("XDG_STATE_HOME");
                }
                panic!("get_state_base_dir failed when XDG_STATE_HOME was set: {:?}", e);
            }
        }

        // Restore original environment
        if let Some(val) = original_xdg_state_home {
            std::env::set_var("XDG_STATE_HOME", val);
        } else {
            std::env::remove_var("XDG_STATE_HOME");
        }
    }

    #[test]
    fn test_get_system_config_path_default() {
        let original_env = env::var("NOVADE_TEST_SYSTEM_CONFIG_PATH").ok();
        env::remove_var("NOVADE_TEST_SYSTEM_CONFIG_PATH");

        assert_eq!(
            get_system_config_path_with_override().unwrap(),
            PathBuf::from("/etc/novade/config.toml")
        );

        if let Some(val) = original_env {
            env::set_var("NOVADE_TEST_SYSTEM_CONFIG_PATH", val);
        }
    }

    #[test]
    fn test_get_system_config_path_override_valid() {
        let original_env = env::var("NOVADE_TEST_SYSTEM_CONFIG_PATH").ok();
        let test_path = "/tmp/custom_system_config.toml";
        env::set_var("NOVADE_TEST_SYSTEM_CONFIG_PATH", test_path);

        assert_eq!(
            get_system_config_path_with_override().unwrap(),
            PathBuf::from(test_path)
        );

        if let Some(val) = original_env {
            env::set_var("NOVADE_TEST_SYSTEM_CONFIG_PATH", val);
        } else {
            env::remove_var("NOVADE_TEST_SYSTEM_CONFIG_PATH");
        }
    }

    #[test]
    fn test_get_system_config_path_override_empty_falls_back_to_default() {
        let original_env = env::var("NOVADE_TEST_SYSTEM_CONFIG_PATH").ok();
        env::set_var("NOVADE_TEST_SYSTEM_CONFIG_PATH", "");

        assert_eq!(
            get_system_config_path_with_override().unwrap(),
            PathBuf::from("/etc/novade/config.toml") // Falls back to default
        );

        if let Some(val) = original_env {
            env::set_var("NOVADE_TEST_SYSTEM_CONFIG_PATH", val);
        } else {
            env::remove_var("NOVADE_TEST_SYSTEM_CONFIG_PATH");
        }
    }

    // Test for NotUnicode is harder as it requires setting non-UTF8 env var,
    // which is platform-specific and not straightforward with Rust's env::set_var.
    // This case is less common for paths but good to be aware of.
}
