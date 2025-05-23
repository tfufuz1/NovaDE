//! XDG Base Directory and Application-Specific Path Resolution.
//!
//! This module provides utility functions for resolving standard directory paths
//! according to the XDG Base Directory Specification and for constructing paths
//! specific to the NovaDE application. It relies on the `directories-next` crate.
//!
//! # Key Functions
//!
//! - **XDG Base Directories**:
//!   - [`get_config_base_dir()`]: Returns `$XDG_CONFIG_HOME` (e.g., `~/.config`).
//!   - [`get_data_base_dir()`]: Returns `$XDG_DATA_HOME` (e.g., `~/.local/share`).
//!   - [`get_cache_base_dir()`]: Returns `$XDG_CACHE_HOME` (e.g., `~/.cache`).
//!   - [`get_state_base_dir()`]: Returns `$XDG_STATE_HOME` (e.g., `~/.local/state` on Linux).
//!
//! - **Application-Specific Directories** (derived from XDG paths):
//!   - [`get_app_config_dir()`]: e.g., `~/.config/NovaDE/NovaDE`.
//!   - [`get_app_data_dir()`]: e.g., `~/.local/share/NovaDE/NovaDE`.
//!   - [`get_app_cache_dir()`]: e.g., `~/.cache/NovaDE/NovaDE`.
//!   - [`get_app_state_dir()`]: e.g., `~/.local/state/NovaDE/NovaDE`.
//!
//! All functions return `Result<PathBuf, CoreError>`, typically yielding
//! [`CoreError::Config(ConfigError::DirectoryUnavailable)`] if a required
//! directory cannot be determined (e.g., when the HOME directory is not found).
//!
//! # Constants
//!
//! The paths for application-specific directories are constructed using the following constants:
//! - `QUALIFIER`: "org"
//! - `ORGANIZATION`: "NovaDE"
//! - `APPLICATION`: "NovaDE"
//! These might be made configurable in the future if necessary.

use std::path::PathBuf;
use directories_next::{BaseDirs, ProjectDirs};
use crate::error::{CoreError, ConfigError}; // Ensure these are correctly pathed

// These constants need to be defined as per the spec for ProjectDirs.
// For novade-core, they might be generic or related to "NovaDE" itself.
// The spec uses "YourOrg", "YourApp". Let's use "NovaDE" for Application
// and a placeholder for QUALIFIER and ORGANIZATION or make them configurable.
// For now, hardcoding placeholders as per spec example, but this should be reviewed.
const QUALIFIER: &str = "org";
const ORGANIZATION: &str = "NovaDE"; // Using NovaDE instead of "YourOrg"
const APPLICATION: &str = "NovaDE";

/// Returns the primary base directory for user-specific configuration files.
///
/// This path typically corresponds to `$XDG_CONFIG_HOME` on Linux systems
/// (e.g., `~/.config`). On other platforms, it resolves to the conventional
/// user-specific configuration directory.
///
/// # Errors
/// Returns [`CoreError::Config(ConfigError::DirectoryUnavailable)`] if the base
/// configuration directory cannot be determined (e.g., if the HOME directory is not set).
///
/// # Examples
/// ```
/// // On Linux, this might print something like: "/home/username/.config"
/// match novade_core::utils::paths::get_config_base_dir() {
///     Ok(path) => println!("Config base directory: {}", path.display()),
///     Err(e) => eprintln!("Error getting config base dir: {}", e),
/// }
/// ```
pub fn get_config_base_dir() -> Result<PathBuf, CoreError> {
    BaseDirs::new()
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable {
            dir_type: "Config Base".to_string()
        }))
}

/// Returns the primary base directory for user-specific data files.
///
/// This path typically corresponds to `$XDG_DATA_HOME` on Linux systems
/// (e.g., `~/.local/share`). On other platforms, it resolves to the conventional
/// user-specific data directory.
///
/// # Errors
/// Returns [`CoreError::Config(ConfigError::DirectoryUnavailable)`] if the base
/// data directory cannot be determined.
///
/// # Examples
/// ```
/// // On Linux, this might print something like: "/home/username/.local/share"
/// match novade_core::utils::paths::get_data_base_dir() {
///     Ok(path) => println!("Data base directory: {}", path.display()),
///     Err(e) => eprintln!("Error getting data base dir: {}", e),
/// }
/// ```
pub fn get_data_base_dir() -> Result<PathBuf, CoreError> {
    BaseDirs::new()
        .map(|dirs| dirs.data_dir().to_path_buf())
        .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable {
            dir_type: "Data Base".to_string()
        }))
}

/// Returns the primary base directory for user-specific cache files.
///
/// This path typically corresponds to `$XDG_CACHE_HOME` on Linux systems
/// (e.g., `~/.cache`). On other platforms, it resolves to the conventional
/// user-specific cache directory.
///
/// # Errors
/// Returns [`CoreError::Config(ConfigError::DirectoryUnavailable)`] if the base
/// cache directory cannot be determined.
///
/// # Examples
/// ```
/// // On Linux, this might print something like: "/home/username/.cache"
/// match novade_core::utils::paths::get_cache_base_dir() {
///     Ok(path) => println!("Cache base directory: {}", path.display()),
///     Err(e) => eprintln!("Error getting cache base dir: {}", e),
/// }
/// ```
pub fn get_cache_base_dir() -> Result<PathBuf, CoreError> {
    BaseDirs::new()
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable {
            dir_type: "Cache Base".to_string()
        }))
}

/// Returns the primary base directory for user-specific state files.
///
/// This path typically corresponds to `$XDG_STATE_HOME` on Linux systems
/// (e.g., `~/.local/state`). If `$XDG_STATE_HOME` is not set on Linux,
/// it falls back to `$HOME/.local/state`.
///
/// For non-Linux platforms, this function currently uses `BaseDirs::data_local_dir()`
/// as a common fallback, as `directories-next::BaseDirs` does not provide a generic
/// `state_dir()` method. This behavior might need platform-specific adjustments
/// for stricter XDG compliance or platform conventions on macOS/Windows.
///
/// # Errors
/// Returns [`CoreError::Config(ConfigError::DirectoryUnavailable)`] if the base
/// state directory cannot be determined.
///
/// # Examples
/// ```
/// // On Linux, this might print something like: "/home/username/.local/state"
/// match novade_core::utils::paths::get_state_base_dir() {
///     Ok(path) => println!("State base directory: {}", path.display()),
///     Err(e) => eprintln!("Error getting state base dir: {}", e),
/// }
/// ```
pub fn get_state_base_dir() -> Result<PathBuf, CoreError> {
    BaseDirs::new()
        .map(|dirs| {
            #[cfg(target_os = "linux")]
            {
                match std::env::var("XDG_STATE_HOME") {
                    Ok(state_home) if !state_home.is_empty() => PathBuf::from(state_home),
                    _ => dirs.home_dir().join(".local/state"),
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                // For non-Linux, use data_local_dir as a common fallback
                // or a platform-specific equivalent if directories-next provides one.
                // The spec implies data_local_dir for other OS.
                // If data_local_dir is not suitable, this might need adjustment.
                // For instance, on macOS, Application Support/YourApp/state might be more appropriate.
                // However, directories-next doesn't have a generic state_dir() for BaseDirs.
                // Using data_local_dir is a common pattern but might not perfectly match XDG for all OS.
                dirs.data_local_dir().to_path_buf() 
            }
        })
        .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable {
            dir_type: "State Base".to_string()
        }))
}

/// Returns the application-specific configuration directory for NovaDE.
///
/// This path is derived using `ProjectDirs` based on the `QUALIFIER`, `ORGANIZATION`,
/// and `APPLICATION` constants. For example, on Linux, this typically resolves to
/// `~/.config/NovaDE/NovaDE`.
///
/// # Errors
/// Returns [`CoreError::Config(ConfigError::DirectoryUnavailable)`] if the application-specific
/// configuration directory cannot be determined (e.g., if `ProjectDirs` cannot be initialized).
///
/// # Examples
/// ```
/// // On Linux, this might print something like: "/home/username/.config/NovaDE/NovaDE"
/// match novade_core::utils::paths::get_app_config_dir() {
///     Ok(path) => println!("App config directory: {}", path.display()),
///     Err(e) => eprintln!("Error getting app config dir: {}", e),
/// }
/// ```
pub fn get_app_config_dir() -> Result<PathBuf, CoreError> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable {
            dir_type: "App Config".to_string()
        }))
}

/// Returns the application-specific data directory for NovaDE.
///
/// This path is derived using `ProjectDirs`. For example, on Linux, this typically
/// resolves to `~/.local/share/NovaDE/NovaDE`.
///
/// # Errors
/// Returns [`CoreError::Config(ConfigError::DirectoryUnavailable)`] if the application-specific
/// data directory cannot be determined.
///
/// # Examples
/// ```
/// // On Linux, this might print: "/home/username/.local/share/NovaDE/NovaDE"
/// match novade_core::utils::paths::get_app_data_dir() {
///     Ok(path) => println!("App data directory: {}", path.display()),
///     Err(e) => eprintln!("Error getting app data dir: {}", e),
/// }
/// ```
pub fn get_app_data_dir() -> Result<PathBuf, CoreError> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.data_dir().to_path_buf())
        .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable {
            dir_type: "App Data".to_string()
        }))
}

/// Returns the application-specific cache directory for NovaDE.
///
/// This path is derived using `ProjectDirs`. For example, on Linux, this typically
/// resolves to `~/.cache/NovaDE/NovaDE`.
///
/// # Errors
/// Returns [`CoreError::Config(ConfigError::DirectoryUnavailable)`] if the application-specific
/// cache directory cannot be determined.
///
/// # Examples
/// ```
/// // On Linux, this might print: "/home/username/.cache/NovaDE/NovaDE"
/// match novade_core::utils::paths::get_app_cache_dir() {
///     Ok(path) => println!("App cache directory: {}", path.display()),
///     Err(e) => eprintln!("Error getting app cache dir: {}", e),
/// }
/// ```
pub fn get_app_cache_dir() -> Result<PathBuf, CoreError> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable {
            dir_type: "App Cache".to_string()
        }))
}

/// Returns the application-specific state directory for NovaDE.
///
/// This path is constructed by appending `ORGANIZATION/APPLICATION`
/// (i.e., `NovaDE/NovaDE`) to the path returned by [`get_state_base_dir()`].
/// For example, on Linux, this typically resolves to `~/.local/state/NovaDE/NovaDE`.
///
/// # Errors
/// Returns [`CoreError::Config(ConfigError::DirectoryUnavailable)`] if the base state
/// directory cannot be determined (propagated from `get_state_base_dir`).
///
/// # Examples
/// ```
/// // On Linux, this might print: "/home/username/.local/state/NovaDE/NovaDE"
/// match novade_core::utils::paths::get_app_state_dir() {
///     Ok(path) => println!("App state directory: {}", path.display()),
///     Err(e) => eprintln!("Error getting app state dir: {}", e),
/// }
/// ```
pub fn get_app_state_dir() -> Result<PathBuf, CoreError> {
    // ProjectDirs doesn't have a dedicated state_dir() method.
    // We construct it by appending ORGANIZATION/APPLICATION to the system's state_base_dir.
    get_state_base_dir().map(|base_state| {
        base_state.join(ORGANIZATION).join(APPLICATION)
    })
    // The error from get_state_base_dir will propagate if it fails.
    // If get_state_base_dir succeeds but joining somehow fails (unlikely for PathBuf),
    // that specific error isn't caught here, but PathBuf join is robust.
    // If ProjectDirs itself failed to initialize (e.g. no home dir),
    // it wouldn't be caught by this structure since we rely on get_state_base_dir first.
    // However, get_state_base_dir (via BaseDirs) would likely fail first in such a scenario.
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to assert that a path is absolute and likely valid (non-empty)
    fn assert_is_valid_path(res: Result<PathBuf, CoreError>, dir_type: &str) {
        match res {
            Ok(path) => {
                println!("Path for {}: {:?}", dir_type, path);
                assert!(path.is_absolute(), "Path for {} is not absolute: {:?}", dir_type, path);
                assert!(!path.as_os_str().is_empty(), "Path for {} is empty", dir_type);
            }
            Err(e) => {
                // On some CI environments, HOME might not be set, leading to errors.
                // We'll print the error and not fail the test outright if it's DirectoryUnavailable.
                if let CoreError::Config(ConfigError::DirectoryUnavailable { ref dir_type, .. }) = e {
                    eprintln!("Could not determine path for {}: {:?}", dir_type, e);
                } else {
                    panic!("Expected Ok or DirectoryUnavailable for {}, got {:?}", dir_type, e);
                }
            }
        }
    }

    #[test]
    fn test_get_config_base_dir() {
        assert_is_valid_path(get_config_base_dir(), "Config Base");
    }

    #[test]
    fn test_get_data_base_dir() {
        assert_is_valid_path(get_data_base_dir(), "Data Base");
    }

    #[test]
    fn test_get_cache_base_dir() {
        assert_is_valid_path(get_cache_base_dir(), "Cache Base");
    }

    #[test]
    fn test_get_state_base_dir() {
        // This test might be more prone to issues in CI if XDG_STATE_HOME is expected
        // and not set, and home_dir().join(".local/state") path also has issues.
        assert_is_valid_path(get_state_base_dir(), "State Base");
    }

    #[test]
    fn test_get_app_config_dir() {
        assert_is_valid_path(get_app_config_dir(), "App Config");
    }

    #[test]
    fn test_get_app_data_dir() {
        assert_is_valid_path(get_app_data_dir(), "App Data");
    }

    #[test]
    fn test_get_app_cache_dir() {
        assert_is_valid_path(get_app_cache_dir(), "App Cache");
    }
    
    #[test]
    fn test_get_app_state_dir() {
        assert_is_valid_path(get_app_state_dir(), "App State");
    }
}
