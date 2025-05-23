//! Configuration Management for NovaDE Core.
//!
//! This module provides the structures and mechanisms for handling configuration
//! within the NovaDE core library. It defines how configuration is structured,
//! loaded, validated, and accessed.
//!
//! ## Key Components:
//!
//! - **Submodules**:
//!   - [`types`]: Contains the primary configuration struct definitions like [`CoreConfig`]
//!     and [`LoggingConfig`]. These structs define the schema of the configuration.
//!   - [`defaults`]: Provides functions that return default values for various
//!     configuration settings. These are used when a configuration file is missing
//!     or incomplete.
//!   - [`loader`]: Implements the logic for loading configuration data, typically from
//!     a TOML file. The central piece is the [`ConfigLoader`] struct.
//!   - [`provider`]: Defines the [`ConfigProvider`] trait for accessing configuration data
//!     and includes implementations like [`FileConfigProvider`] for file-based configurations.
//!     (Note: `FileConfigProvider`'s loading mechanism might need refactoring after
//!     changes to `ConfigLoader`).
//!
//! - **Core Structs (Re-exported)**:
//!   - [`CoreConfig`]: The root configuration structure for the entire core layer.
//!   - [`LoggingConfig`]: Configuration specific to the logging subsystem.
//!
//! - **Core Functionality (Re-exported)**:
//!   - [`ConfigLoader`]: An empty struct with static methods (e.g., `load()`) to load
//!     and validate the `CoreConfig`.
//!   - [`ConfigProvider`]: A trait for components that provide access to `CoreConfig`.
//!   - [`FileConfigProvider`]: An example implementation of `ConfigProvider`.
//!
//! ## Configuration Loading Process:
//!
//! 1. The `ConfigLoader::load()` method is called.
//! 2. It attempts to find and read a configuration file (e.g., `config.toml`) from
//!    the application-specific configuration directory (determined by `utils::paths`).
//! 3. If the file is not found, a default `CoreConfig` is generated.
//! 4. If the file is found, its content (expected to be TOML) is parsed into `CoreConfig`.
//!    Parsing errors are mapped to [`crate::error::ConfigError::ParseError`].
//! 5. The resulting `CoreConfig` (either loaded or default) undergoes validation
//!    (e.g., normalizing log levels, resolving relative log file paths). Validation
//!    errors are mapped to [`crate::error::ConfigError::ValidationError`].
//!
//! # Examples
//!
//! ```rust,ignore
//! // How to load configuration
//! use novade_core::config::ConfigLoader;
//! use novade_core::error::CoreError;
//!
//! match ConfigLoader::load() {
//!     Ok(config) => {
//!         println!("Loaded log level: {}", config.logging.level);
//!         // Use the config...
//!     }
//!     Err(e) => {
//!         eprintln!("Failed to load configuration: {}", e);
//!         // Handle error, perhaps by falling back to minimal defaults or exiting.
//!         // For instance, initialize minimal logging:
//!         novade_core::logging::init_minimal_logging();
//!         tracing::error!("Configuration error: {}", e);
//!     }
//! }
//! ```

pub mod defaults;
pub mod types; 
pub mod loader; // Renamed from file_loader
pub mod provider; // Added for ConfigProvider

// use std::path::Path; // No longer needed here
// use crate::error::ConfigError; // No longer needed here, used internally by loader/provider

// Re-export new config types and loader/provider
pub use types::{CoreConfig, LoggingConfig};
pub use loader::ConfigLoader; // New ConfigLoader struct
pub use provider::{ConfigProvider, FileConfigProvider}; // Moved ConfigProvider trait and FileConfigProvider struct


// The ConfigLoader trait definition is removed from here.
// The ConfigProvider trait definition is removed from here (moved to provider.rs).


#[cfg(test)]
mod tests {
    use super::*; // Imports CoreConfig, LoggingConfig from the re-export above
    use crate::config::defaults as config_defaults; // For direct comparison if needed
    use std::path::PathBuf; // For testing Option<PathBuf>

    #[test]
    fn test_new_core_config_default() {
        let config = CoreConfig::default();
        // Check that logging config within core_config matches the default LoggingConfig
        let default_log_config = LoggingConfig::default();
        assert_eq!(config.logging.level, default_log_config.level);
        assert_eq!(config.logging.file_path, default_log_config.file_path);
        assert_eq!(config.logging.format, default_log_config.format);
    }
    
    #[test]
    fn test_new_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, config_defaults::default_log_level_spec());
        assert_eq!(config.file_path, config_defaults::default_log_file_path_spec());
        assert_eq!(config.format, config_defaults::default_log_format_spec());
    }

    // Example of a serde deserialization test for the new CoreConfig
    #[test]
    fn test_core_config_deserialize_minimal() {
        let json_data = r#"{
            "logging": {
                "level": "debug"
            }
        }"#;
        let config: CoreConfig = serde_json::from_str(json_data).expect("Failed to deserialize CoreConfig");
        
        assert_eq!(config.logging.level, "debug");
        // file_path and format should take their defaults from LoggingConfig's defaults
        assert_eq!(config.logging.file_path, config_defaults::default_log_file_path_spec());
        assert_eq!(config.logging.format, config_defaults::default_log_format_spec());
    }

    #[test]
    fn test_core_config_deserialize_full_logging() {
         let json_data = r#"{
            "logging": {
                "level": "trace",
                "file_path": "/var/log/app.log",
                "format": "json"
            }
        }"#;
        let config: CoreConfig = serde_json::from_str(json_data).expect("Failed to deserialize CoreConfig");
        
        assert_eq!(config.logging.level, "trace");
        assert_eq!(config.logging.file_path, Some(PathBuf::from("/var/log/app.log")));
        assert_eq!(config.logging.format, "json");
    }
    
    // Tests for ApplicationConfig and SystemConfig defaults are removed
    // as these structs are no longer part of CoreConfig directly.
}
