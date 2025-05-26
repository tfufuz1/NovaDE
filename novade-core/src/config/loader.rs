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
use crate::config::types::CoreConfig;
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
                        // This means we should return a default config here, then validate it.
                        let mut default_config = CoreConfig::default();
                        Self::validate_config(&mut default_config)?; // Validate default config (e.g. to create log paths)
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
        
        // FeatureFlags are not part of CoreConfig yet.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::LoggingConfig;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn setup_test_config_file(temp_dir: &TempDir, filename: &str, content: &str) -> PathBuf {
        let config_path = temp_dir.path().join(filename);
        let mut file = File::create(&config_path).expect("Failed to create test config file");
        file.write_all(content.as_bytes()).expect("Failed to write to test config file");
        config_path
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
    // 3. Use environment variables during tests to control `directories-next` (can be flaky).
    //
    // For now, these tests will *try* to run, but might fail or have side effects in some CI/local envs
    // if `get_app_config_dir` points to a real but unwriteable location or similar.
    // A robust solution would involve abstracting path resolution for testing.

    #[test]
    fn test_load_valid_config() {
        // This test is tricky due to hardcoded paths in load().
        // We'd ideally mock get_app_config_dir.
        // For now, we can't easily test the positive load path without such mocking
        // or writing to the actual user config dir, which is bad.
        // So, we'll focus on testing validate_config and parsing logic with controlled inputs.
        
        // Test parsing and validation part assuming content is read.
        let content = r#"
[logging]
level = "DEBUG"
format = "JSON"
file_path = "app.log" 
        "#;
        let mut config: CoreConfig = toml::from_str(content).unwrap();
        
        // Mock state dir for validation if file_path is relative
        // This part is hard because validate_config calls get_app_state_dir()
        // Let's assume for this specific test that file_path validation is tested separately
        // or that get_app_state_dir() returns something sensible in test env.
        // To avoid FS operations in this specific unit test for validation logic:
        let temp_state_dir = TempDir::new().unwrap();
        let original_get_app_state_dir = std::env::var_os("XDG_STATE_HOME");
        std::env::set_var("XDG_STATE_HOME", temp_state_dir.path());

        let validation_result = ConfigLoader::validate_config(&mut config);
        
        if let Some(original_var) = original_get_app_state_dir {
            std::env::set_var("XDG_STATE_HOME", original_var);
        } else {
            std::env::remove_var("XDG_STATE_HOME");
        }
        
        assert!(validation_result.is_ok());
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.logging.format, "json");
        assert!(config.logging.file_path.is_some());
        if let Some(p) = config.logging.file_path {
            assert!(p.is_absolute());
            assert!(p.ends_with("app.log"));
            if let Some(parent) = p.parent() {
                assert!(parent.exists()); // validate_config should create it
            }
        }
    }

    #[test]
    fn test_load_default_if_not_found() {
        // This test also relies on get_app_config_dir behavior.
        // If we assume get_app_config_dir returns a path to a temp dir where config.toml doesn't exist:
        // A proper test would involve setting up a temporary config dir, ensuring config.toml is NOT there,
        // then calling load(). This is hard due to the static nature of get_app_config_dir.
        // For now, we check if default config is valid.
        let mut default_config = CoreConfig::default();
        
        let temp_state_dir = TempDir::new().unwrap();
        let original_get_app_state_dir = std::env::var_os("XDG_STATE_HOME");
        std::env::set_var("XDG_STATE_HOME", temp_state_dir.path());

        let validation_result = ConfigLoader::validate_config(&mut default_config);

        if let Some(original_var) = original_get_app_state_dir {
            std::env::set_var("XDG_STATE_HOME", original_var);
        } else {
            std::env::remove_var("XDG_STATE_HOME");
        }

        assert!(validation_result.is_ok());
        // Check if default log file path (if any) is handled correctly
        if let Some(log_path) = default_config.logging.file_path {
            assert!(log_path.is_absolute());
            if let Some(parent) = log_path.parent(){
                assert!(parent.exists());
            }
        }
    }

    #[test]
    fn test_validate_config_invalid_log_level() {
        let mut config = CoreConfig {
            logging: LoggingConfig {
                level: "invalid_level".to_string(),
                file_path: None,
                format: "text".to_string(),
            },
        };
        let result = ConfigLoader::validate_config(&mut config);
        assert!(matches!(result, Err(CoreError::Config(ConfigError::ValidationError(_)))));
        if let Err(CoreError::Config(ConfigError::ValidationError(msg))) = result {
            assert!(msg.contains("Invalid log level"));
        }
    }

    #[test]
    fn test_validate_config_invalid_log_format() {
        let mut config = CoreConfig {
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                format: "invalid_format".to_string(),
            },
        };
        let result = ConfigLoader::validate_config(&mut config);
        assert!(matches!(result, Err(CoreError::Config(ConfigError::ValidationError(_)))));
         if let Err(CoreError::Config(ConfigError::ValidationError(msg))) = result {
            assert!(msg.contains("Invalid log format"));
        }
    }

    #[test]
    fn test_validate_config_log_file_path_relative() {
        let temp_state_dir = TempDir::new().unwrap();
        // Mock get_app_state_dir by setting XDG_STATE_HOME, assuming paths::get_app_state_dir honors it.
        let original_state_home = std::env::var_os("XDG_STATE_HOME");
        std::env::set_var("XDG_STATE_HOME", temp_state_dir.path().to_str().unwrap());

        let mut config = CoreConfig {
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some(PathBuf::from("logs/app.log")),
                format: "text".to_string(),
            },
        };
        
        let result = ConfigLoader::validate_config(&mut config);
        assert!(result.is_ok());
        assert!(config.logging.file_path.is_some());
        let expected_path = temp_state_dir.path().join("logs/app.log");
        assert_eq!(config.logging.file_path, Some(expected_path.clone()));
        assert!(expected_path.parent().unwrap().exists()); // ensure_directory_exists should have created it.

        // Restore XDG_STATE_HOME
        if let Some(val) = original_state_home {
            std::env::set_var("XDG_STATE_HOME", val);
        } else {
            std::env::remove_var("XDG_STATE_HOME");
        }
    }
    
    #[test]
    fn test_validate_config_log_file_path_absolute() {
        let temp_log_dir = TempDir::new().unwrap();
        let log_file_path = temp_log_dir.path().join("absolute_app.log");

        let mut config = CoreConfig {
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some(log_file_path.clone()),
                format: "text".to_string(),
            },
        };
        
        // Parent directory for the absolute path might not exist.
        // Let's ensure it does for the test, or that validate_config creates it.
        // The spec for validate_config implies ensure_directory_exists is called.
        if let Some(parent) = log_file_path.parent() {
             if !parent.exists() { std::fs::create_dir_all(parent).unwrap(); } // Pre-create for simplicity if needed
        }

        let result = ConfigLoader::validate_config(&mut config);
        assert!(result.is_ok());
        assert_eq!(config.logging.file_path, Some(log_file_path.clone()));
        if let Some(parent) = log_file_path.parent() {
            assert!(parent.exists());
        }
    }
    
    // Test for parsing error - this is more of an integration test for load()
    // but we can test the toml::from_str part if load() was refactored to take content.
    // For now, this kind of test would be better in a full load() test.
    // e.g., create a temp file with invalid toml and call load()
    // (which is hard due to hardcoded paths in load()).
}
