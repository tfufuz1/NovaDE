//! Configuration Loading for NovaDE Core.
//!
//! This module provides the [`ConfigLoader`] struct, which is responsible for
//! loading, parsing, and validating the `CoreConfig` for the NovaDE system.
//! It handles locating the configuration file, deserializing it from TOML format,
//! applying default values, and performing validation checks on the loaded configuration.
//!
//! # Usage
//!
//! The primary way to use this module is through the static `ConfigLoader::load()` method:
//!
//! ```rust,ignore
//! use novade_core::config::ConfigLoader;
//! use novade_core::error::CoreError;
//!
//! match ConfigLoader::load() {
//!     Ok(config) => {
//!         // Use the loaded and validated config
//!         println!("Logging level: {}", config.logging.level);
//!     }
//!     Err(e) => {
//!         eprintln!("Failed to load configuration: {}", e);
//!         // Handle errors, perhaps by using minimal defaults or exiting
//!         // For example, initialize minimal logging:
//!         novade_core::logging::init_minimal_logging();
//!         tracing::error!("Configuration loading failed: {}", e);
//!     }
//! }
//! ```
//!
//! ## Configuration File Location
//!
//! `ConfigLoader::load()` attempts to load `config.toml` from the application-specific
//! configuration directory, as determined by `novade_core::utils::paths::get_app_config_dir()`.
//! If the file is not found, a default configuration is used.
//!
//! ## Validation
//!
//! After loading (or generating defaults), the configuration undergoes validation via
//! the `validate_config` method. This includes:
//! - Normalizing and validating log levels and formats.
//! - Resolving relative log file paths to absolute paths within the application's state directory.
//! - Ensuring necessary parent directories for log files are created.

use std::path::PathBuf;
use std::fs;
// Use CoreConfig from the parent module (config/mod.rs)
use crate::config::{CoreConfig, LoggingConfig, FeatureFlags, defaults};
use crate::error::{CoreError, ConfigError};
use crate::utils::paths::{get_app_config_dir, get_app_state_dir};
use crate::utils::fs as nova_fs; // Renamed to avoid conflict with std::fs

/// `ConfigLoader` provides static methods to load and validate `CoreConfig`.
///
/// This is an empty struct used as a namespace for configuration loading logic.
/// The main entry point is the `load()` method.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Loads and validates the `CoreConfig` for the application.
    ///
    /// This method performs the following steps:
    /// 1. Determines the path to the application-specific configuration file (`config.toml`)
    ///    using [`get_app_config_dir`].
    /// 2. Attempts to read the configuration file:
    ///    - If the file is found and read successfully, its TOML content is parsed.
    ///    - If the file is not found (`ErrorKind::NotFound`), a default `CoreConfig` is generated.
    ///      This behavior aligns with the specification: "Wenn die Konfigurationsdatei nicht existiert,
    ///      wird eine Standardkonfiguration verwendet."
    ///    - Other file read errors result in a [`CoreError::Config(ConfigError::ReadError)`].
    /// 3. Parses the TOML content into a [`CoreConfig`]. Parsing errors result in
    ///    [`CoreError::Config(ConfigError::ParseError)`].
    /// 4. Validates the loaded or default configuration using [`Self::validate_config`].
    ///    Validation errors result in [`CoreError::Config(ConfigError::ValidationError)`] or
    ///    filesystem errors from path resolution/creation.
    ///
    /// # Returns
    ///
    /// - `Ok(CoreConfig)`: The loaded and validated configuration.
    /// - `Err(CoreError)`: If any step (directory resolution, file reading, parsing, validation) fails.
    ///   Specific error variants within `CoreError` (like `ConfigError` or `Filesystem`) provide more detail.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // This example assumes a valid environment for path resolution.
    /// // In tests, path resolution might need mocking or careful setup.
    /// match novade_core::config::ConfigLoader::load() {
    ///     Ok(config) => println!("Successfully loaded config with log level: {}", config.logging.level),
    ///     Err(e) => eprintln!("Error loading config: {}", e),
    /// }
    /// ```
    pub fn load() -> Result<CoreConfig, CoreError> {
        let config_dir = get_app_config_dir()?; // Can return CoreError::Config(ConfigError::DirectoryUnavailable)
        let config_path = config_dir.join("config.toml");

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(e) => {
                return match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        // As per spec, if not found, use default and proceed with validation (which might create paths)
                        // The spec for load() says: "NotFound to CoreError::Config(ConfigError::NotFound...)"
                        // This implies we might want to error out here. However, typical config loading often
                        // falls back to defaults if a file is not found.
                        // The spec for ConfigLoader::load() in A1-Kernschicht.md says:
                        // "Wenn die Konfigurationsdatei nicht existiert, wird eine Standardkonfiguration verwendet."
                        // This means we should create a default config here, then validate it.
                        let mut default_config = CoreConfig::default();
                        // Ensure default config is validated (e.g., to create log paths if specified in defaults)
                        Self::validate_config(&mut default_config)?;
                        return Ok(default_config);
                    }
                    _ => Err(CoreError::Config(ConfigError::ReadError {
                        path: config_path.clone(),
                        source: e,
                    })),
                };
            }
        };
        
        let mut config: CoreConfig = toml::from_str(&content)
            .map_err(ConfigError::ParseError)?; // Implicitly CoreError::Config(ConfigError::ParseError(e))

        Self::validate_config(&mut config)?;
        Ok(config)
    }

    /// Validates the loaded `CoreConfig` and performs necessary adjustments.
    ///
    /// This internal method is called by `load()` after a configuration is successfully
    /// parsed or a default one is generated.
    ///
    /// Validation steps include:
    /// - Normalizing and validating the logging level (must be one of "trace", "debug", "info", "warn", "error").
    /// - Normalizing and validating the logging format (must be "text" or "json").
    /// - Resolving the logging file path:
    ///   - If `file_path` is `Some` and relative, it's made absolute against the application's state directory
    ///     (obtained via [`get_app_state_dir`]).
    ///   - Parent directories for the log file path are created if they don't exist using
    ///     [`nova_fs::ensure_directory_exists`].
    ///
    /// # Arguments
    ///
    /// * `config`: A mutable reference to the `CoreConfig` to validate and potentially modify (e.g., path resolution).
    ///
    /// # Errors
    ///
    /// Returns a `CoreError` if:
    /// - Validation fails (e.g., invalid log level or format), resulting in [`CoreError::Config(ConfigError::ValidationError)`].
    /// - A required application directory (like the state directory for log files) cannot be determined,
    ///   resulting in [`CoreError::Config(ConfigError::DirectoryUnavailable)`].
    /// - Filesystem operations (like creating log directories) fail, resulting in [`CoreError::Filesystem`].
    fn validate_config(config: &mut CoreConfig) -> Result<(), CoreError> {
        // Validate logging level
        let level_lower = config.logging.level.to_lowercase();
        match level_lower.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {
                config.logging.level = level_lower; // Normalize
            }
            _ => {
                return Err(CoreError::Config(ConfigError::ValidationError(format!(
                    "Invalid log level: '{}'. Must be one of trace, debug, info, warn, error.",
                    config.logging.level
                ))));
            }
        }

        // Validate logging format
        let format_lower = config.logging.format.to_lowercase();
        match format_lower.as_str() {
            "text" | "json" => {
                config.logging.format = format_lower; // Normalize
            }
            _ => {
                return Err(CoreError::Config(ConfigError::ValidationError(format!(
                    "Invalid log format: '{}'. Must be one of text, json.",
                    config.logging.format
                ))));
            }
        }

        // Handle log file path
        if let Some(relative_path) = &config.logging.file_path {
            if relative_path.is_absolute() {
                // If it's already absolute, use it as is, but ensure parent dir exists
                if let Some(parent_dir) = relative_path.parent() {
                    if !parent_dir.exists() {
                         nova_fs::ensure_dir_exists(parent_dir)?;
                    }
                }
            } else {
                // If relative, make it absolute to the app state directory
                let state_dir = get_app_state_dir()?;
                let absolute_path = state_dir.join(relative_path);
                
                if let Some(parent_dir) = absolute_path.parent() {
                     if !parent_dir.exists() {
                        nova_fs::ensure_dir_exists(parent_dir)?;
                    }
                }
                config.logging.file_path = Some(absolute_path);
            }
        }
        
        // FeatureFlags are part of CoreConfig. No specific validation needed here for them beyond parsing.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Use LoggingConfig from crate::config for tests as well
    use crate::config::LoggingConfig;
    use crate::utils::{self, paths::{get_app_config_dir, get_app_state_dir}}; // For test setup
    use std::env;
    use std::fs::{self, File}; // Added fs for create_temp_config_file and other operations
    use std::io::Write;
    use std::path::{Path, PathBuf}; // Added Path
    use tempfile::TempDir;


    // Helper to create a temporary config file
    fn create_temp_config_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let path = dir.join(filename);
        fs::write(&path, content).expect("Failed to write temp config file");
        path
    }

    // Helper to set up a temporary environment for config loading tests
    // This involves overriding where get_app_config_dir() and get_app_state_dir() look.
    // We can do this by setting XDG environment variables that `directories-next` respects.
    struct TestEnv {
        _temp_config_dir: TempDir, // Owns the temp dir, cleans up on drop
        _temp_state_dir: TempDir,
        original_xdg_config_home: Option<String>,
        original_xdg_state_home: Option<String>,
    }

    impl TestEnv {
        fn new() -> Self {
            let temp_config_dir = TempDir::new().unwrap();
            let temp_state_dir = TempDir::new().unwrap();

            let original_xdg_config_home = env::var("XDG_CONFIG_HOME").ok();
            let original_xdg_state_home = env::var("XDG_STATE_HOME").ok();

            env::set_var("XDG_CONFIG_HOME", temp_config_dir.path());
            env::set_var("XDG_STATE_HOME", temp_state_dir.path());

            // For ProjectDirs, it might also use XDG_DATA_HOME for subpaths if qualifier/org/app is used.
            // For simplicity, we assume get_app_config_dir will now effectively use temp_config_dir.path() / "NovaDE" (or app name)
            // and get_app_state_dir will use temp_state_dir.path() / "NovaDE" (or app name)
            // After setting XDG vars, resolve paths using the functions and ensure they exist.
            // This ensures that the directories ConfigLoader will attempt to use are actually created.
            let app_cfg_dir = get_app_config_dir().expect("TestEnv: Failed to resolve app config dir based on temp XDG_CONFIG_HOME");
            utils::fs::ensure_dir_exists(&app_cfg_dir).expect("TestEnv: Failed to create temp app config dir");

            let app_state_dir = get_app_state_dir().expect("TestEnv: Failed to resolve app state dir based on temp XDG_STATE_HOME");
            utils::fs::ensure_dir_exists(&app_state_dir).expect("TestEnv: Failed to create temp app state dir");

            Self {
                _temp_config_dir: temp_config_dir, // Keep ownership of the temp dirs
                _temp_state_dir: temp_state_dir,
                original_xdg_config_home,
                original_xdg_state_home,
            }
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            if let Some(val) = &self.original_xdg_config_home {
                env::set_var("XDG_CONFIG_HOME", val);
            } else {
                env::remove_var("XDG_CONFIG_HOME");
            }
            if let Some(val) = &self.original_xdg_state_home {
                env::set_var("XDG_STATE_HOME", val);
            } else {
                env::remove_var("XDG_STATE_HOME");
            }
        }
    }
    
    // Mock get_app_config_dir and get_app_state_dir for consistent testing environments
    // This is complex without a mocking library. For these tests, we'll rely on temp dirs
    // and override the default config path by directly manipulating where ConfigLoader looks,
    // or by ensuring the test environment has these dirs (less ideal for unit tests).
    // The current ConfigLoader::load() is hardcoded to use get_app_config_dir().
    // To make this testable without actual FS side effects in user dirs, we need to
    // either:
    // 1. Mock `get_app_config_dir` and `get_app_state_dir`. (Requires a mocking framework or feature flags)
    // 2. Parameterize `ConfigLoader::load` with the config path (breaks current spec of `load()`).
    // 3. Use environment variables during tests to control `directories-next` (as TestEnv does).
    //
    // For now, these tests will *try* to run, but might fail or have side effects in some CI/local envs
    // if `get_app_config_dir` points to a real but unwriteable location or similar.
    // A robust solution would involve abstracting path resolution for testing.
    // The TestEnv helper is a good step in this direction.

    #[test]
    fn test_config_loader_load_success() {
        let _test_env = TestEnv::new();
        let app_config_dir = get_app_config_dir().unwrap();
        
        let toml_content = r#"
[logging]
level = "debug"
format = "json"
file_path = "logs/app.log"

[feature_flags]
experimental_feature_x = true
        "#;
        create_temp_config_file(&app_config_dir, "config.toml", toml_content);

        let config = ConfigLoader::load().expect("ConfigLoader::load failed");

        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.logging.format, "json");
        assert!(config.logging.file_path.is_some());
        let log_path = config.logging.file_path.unwrap();
        assert!(log_path.is_absolute());
        let log_path_str = log_path.to_string_lossy();
        // Check that the path is under the state directory (which TestEnv sets up) and has the correct subpath.
        // The exact path will vary, so check for key components.
        assert!(log_path_str.contains("logs") && log_path_str.ends_with("app.log"));
        assert!(log_path.parent().unwrap().exists());

        assert_eq!(config.feature_flags.experimental_feature_x, true);
    }

    #[test]
    fn test_config_loader_load_default_when_not_found() {
        let _test_env = TestEnv::new(); // Sets up temp XDG dirs
        // Ensure config.toml does not exist
        let app_config_dir = get_app_config_dir().unwrap();
        let config_file_path = app_config_dir.join("config.toml");
        if config_file_path.exists() {
            fs::remove_file(&config_file_path).unwrap();
        }

        // Also ensure no default log file path exists from a previous test run, if defaults specify one.
        let default_log_path_opt = defaults::default_logging_config().file_path;
        if let Some(default_log_path_segment) = default_log_path_opt {
             let app_state_dir = get_app_state_dir().unwrap();
             let full_default_log_path = app_state_dir.join(default_log_path_segment);
             if full_default_log_path.exists() {
                 fs::remove_file(&full_default_log_path).unwrap();
             }
             if let Some(parent) = full_default_log_path.parent() {
                 if parent.exists() {
                    // We only want to remove the specific log file, not necessarily the whole logs dir
                    // if other tests might use it. But for a clean test of default creation,
                    // ensuring the parent does not pre-exist for the *default* path could be an option.
                    // However, validate_config is meant to *create* it.
                 }
             }
        }


        let result = ConfigLoader::load();
        assert!(result.is_ok(), "ConfigLoader::load failed when config file not found: {:?}", result.err());
        let config = result.unwrap();

        // Check it's the default config
        assert_eq!(config, CoreConfig::default());

        // If the default config specifies a log file, ensure its parent directory was created by validate_config
        if let Some(ref log_path) = config.logging.file_path {
            assert!(log_path.is_absolute(), "Default log path should be absolute after validation");
            if let Some(parent_dir) = log_path.parent() {
                assert!(parent_dir.exists(), "Parent directory for default log path was not created");
            }
        }
    }


    #[test]
    fn test_config_loader_load_parse_error() {
        let _test_env = TestEnv::new();
        let app_config_dir = get_app_config_dir().unwrap();
        let invalid_toml_content = "this is not valid toml content";
        create_temp_config_file(&app_config_dir, "config.toml", invalid_toml_content);

        let result = ConfigLoader::load();
        assert!(result.is_err());
        match result.err().unwrap() {
            CoreError::Config(ConfigError::ParseError(_)) => { /* Expected */ }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_config_loader_load_io_error_other_than_not_found() {
        let _test_env = TestEnv::new();
        let app_config_dir = get_app_config_dir().unwrap();
        let config_file_path = app_config_dir.join("config.toml");

        // Create the file, but then make it unreadable (e.g. by making its parent dir unreadable)
        // This is hard to do reliably cross-platform in tests.
        // A simpler IO error to simulate might be if the path is a directory.
        nova_fs::ensure_dir_exists(&config_file_path).unwrap(); // Create config.toml as a directory

        let result = ConfigLoader::load();
        assert!(result.is_err());
        match result.err().unwrap() {
            CoreError::Config(ConfigError::ReadError { path, source: _ }) => {
                 assert_eq!(path, config_file_path);
            }
            // This might also be CoreError::Io if the read_to_string itself fails due to "is a directory"
            // depending on OS and fs::read_to_string behavior.
            // For now, ReadError is the expectation from the original tests.
            e => panic!("Unexpected error type for ReadError: {:?}", e),
        }
        // Clean up the directory we created
        fs::remove_dir_all(&config_file_path).ok();
    }


    #[test]
    fn test_validate_config_valid_settings() {
        let _test_env = TestEnv::new(); // For state dir resolution
        let mut config = CoreConfig {
            logging: defaults::default_logging_config(),
            feature_flags: defaults::default_feature_flags(),
        };
        config.logging.level = "TRACE".to_string(); // Test case normalization
        config.logging.format = "JSON".to_string(); // Test case normalization
        config.logging.file_path = Some(PathBuf::from("my_app/log.txt")); // Relative path

        ConfigLoader::validate_config(&mut config).expect("Validation failed for valid settings");

        assert_eq!(config.logging.level, "trace");
        assert_eq!(config.logging.format, "json");
        let log_path = config.logging.file_path.unwrap();
        assert!(log_path.is_absolute());
        assert!(log_path.to_string_lossy().ends_with("my_app/log.txt"));
        assert!(log_path.parent().unwrap().exists());
    }

    #[test]
    fn test_validate_config_invalid_log_level() { // Replaced with version from mod.rs
        let mut config = CoreConfig::default();
        config.logging.level = "superlog".to_string();
        let result = ConfigLoader::validate_config(&mut config);
        assert!(matches!(result, Err(CoreError::Config(ConfigError::ValidationError(_)))));
        if let Err(CoreError::Config(ConfigError::ValidationError(msg))) = result {
            assert!(msg.contains("Invalid log level: 'superlog'"));
        }
    }

    #[test]
    fn test_validate_config_invalid_log_format() { // Replaced with version from mod.rs
        let mut config = CoreConfig::default();
        config.logging.format = "binary".to_string();
        let result = ConfigLoader::validate_config(&mut config);
        assert!(matches!(result, Err(CoreError::Config(ConfigError::ValidationError(_)))));
        if let Err(CoreError::Config(ConfigError::ValidationError(msg))) = result {
            assert!(msg.contains("Invalid log format: 'binary'"));
        }
    }

    #[test]
    fn test_validate_config_absolute_log_path() { // Added from mod.rs
        let _test_env = TestEnv::new();
        let temp_dir_for_log = TempDir::new().unwrap();
        let abs_log_path = temp_dir_for_log.path().join("sub/absolute.log");

        let mut config = CoreConfig::default();
        config.logging.file_path = Some(abs_log_path.clone());

        ConfigLoader::validate_config(&mut config).expect("Validation failed for absolute path");
        
        assert_eq!(config.logging.file_path.unwrap(), abs_log_path);
        assert!(abs_log_path.parent().unwrap().exists());
    }
    
    #[test]
    fn test_validate_config_log_path_is_root_parent() { // Added from mod.rs
        let _test_env = TestEnv::new();
        let mut config = CoreConfig::default();
        
        let log_file_name_only = PathBuf::from("logfile.log");

        config.logging.file_path = Some(log_file_name_only.clone());
        let result = ConfigLoader::validate_config(&mut config);
        assert!(result.is_ok(), "Validation failed for log file in state_dir root: {:?}", result.err());

        let app_state_dir = get_app_state_dir().unwrap();
        let expected_abs_path = app_state_dir.join(log_file_name_only);
        assert_eq!(config.logging.file_path, Some(expected_abs_path.clone()));
        assert!(expected_abs_path.parent().unwrap().exists());
    }

    // Note: test_validate_config_log_file_path_relative from original loader.rs tests
    // is covered by test_validate_config_valid_settings and test_config_loader_load_success,
    // which use relative paths initially and TestEnv for state dir.
}
