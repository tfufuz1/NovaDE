//! Configuration Data Structures for NovaDE Core.
//!
//! This module defines the primary structures used to represent the configuration
//! of the NovaDE core system. These structs are typically populated by deserializing
//! a configuration file (e.g., TOML).
//!
//! # Key Structs
//! - [`CoreConfig`]: The root configuration structure.
//! - [`LoggingConfig`]: Configuration specific to the logging subsystem.
//!
//! These structs utilize `serde` for deserialization and apply default values
//! for fields not present in the configuration source, referencing functions
//! from the [`super::defaults`] module. They also enforce that no unknown
//! fields are present during deserialization via `#[serde(deny_unknown_fields)]`.

use serde::Deserialize;
use std::path::PathBuf;
use super::defaults; // Assuming defaults.rs is in the same module parent (config)

/// Configuration settings for the logging subsystem.
///
/// Defines parameters such as the log level, optional log file path, and log format.
/// These settings are used by the `novade_core::logging` module to initialize
/// the global logger.
///
/// # Examples
///
/// ```
/// use novade_core::config::LoggingConfig;
/// use std::path::PathBuf;
///
/// // Default logging configuration
/// let default_log_config = LoggingConfig::default();
/// assert_eq!(default_log_config.level, "info");
/// assert_eq!(default_log_config.file_path, None);
/// assert_eq!(default_log_config.format, "text");
///
/// // Example of how it might be deserialized (conceptual)
/// let toml_str = r#"
/// level = "debug"
/// file_path = "/var/log/novade_app.log"
/// format = "json"
/// "#;
/// let log_config: LoggingConfig = toml::from_str(toml_str).unwrap();
/// assert_eq!(log_config.level, "debug");
/// assert_eq!(log_config.file_path, Some(PathBuf::from("/var/log/novade_app.log")));
/// assert_eq!(log_config.format, "json");
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    /// The minimum log level to record.
    /// Valid values (case-insensitive): "trace", "debug", "info", "warn", "error".
    /// Defaults to "info" via [`defaults::default_log_level_spec`].
    #[serde(default = "defaults::default_log_level_spec")]
    pub level: String,
    /// Optional path to a file where logs should be written.
    /// If `None`, file logging is disabled.
    /// Relative paths are resolved against the application's state directory.
    /// Defaults to `None` via [`defaults::default_log_file_path_spec`].
    #[serde(default = "defaults::default_log_file_path_spec")]
    pub file_path: Option<PathBuf>,
    /// The format for log messages written to a file.
    /// Valid values (case-insensitive): "text", "json".
    /// Defaults to "text" via [`defaults::default_log_format_spec`].
    #[serde(default = "defaults::default_log_format_spec")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: defaults::default_log_level_spec(),
            file_path: defaults::default_log_file_path_spec(),
            format: defaults::default_log_format_spec(),
        }
    }
}

/// Root configuration structure for the NovaDE core system.
///
/// This struct aggregates all core configuration settings. Currently, it primarily
/// includes logging configuration. Other subsystems like application settings or
/// feature flags might be added here in the future.
///
/// It is designed to be deserialized from a configuration source (e.g., a TOML file)
/// and uses default values for missing sections or fields.
///
/// # Examples
///
/// ```
/// use novade_core::config::{CoreConfig, LoggingConfig};
///
/// // Default core configuration
/// let core_config = CoreConfig::default();
/// assert_eq!(core_config.logging.level, "info"); // Inherits default from LoggingConfig
///
/// // Example of how it might be deserialized (conceptual)
/// let toml_str = r#"
/// [logging]
/// level = "warn"
/// format = "json"
/// "#;
/// let loaded_config: CoreConfig = toml::from_str(toml_str).unwrap();
/// assert_eq!(loaded_config.logging.level, "warn");
/// assert_eq!(loaded_config.logging.format, "json");
/// assert_eq!(loaded_config.logging.file_path, None); // Default for file_path
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreConfig {
    /// Configuration for the logging subsystem.
    /// Defaults are provided by [`defaults::default_core_logging_config`].
    #[serde(default = "defaults::default_core_logging_config")]
    pub logging: LoggingConfig,
    // FeatureFlags are omitted for now as per current focus.
    // Example for future:
    // #[serde(default)]
    // pub features: FeatureFlags,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            logging: defaults::default_core_logging_config(),
        }
    }
}

// No FeatureFlags struct for now.

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_logging_config_default_values() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info"); // Assuming default_log_level_spec returns "info"
        assert_eq!(config.file_path, None); // Assuming default_log_file_path_spec returns None
        assert_eq!(config.format, "text"); // Assuming default_log_format_spec returns "text"
    }

    #[test]
    fn test_core_config_default_values() {
        let core_config = CoreConfig::default();
        let logging_config = LoggingConfig::default(); // Get default LoggingConfig for comparison
        assert_eq!(core_config.logging.level, logging_config.level);
        assert_eq!(core_config.logging.file_path, logging_config.file_path);
        assert_eq!(core_config.logging.format, logging_config.format);
    }

    #[test]
    fn test_logging_config_deserialize_empty() {
        let json = "{}";
        let config: LoggingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.level, defaults::default_log_level_spec());
        assert_eq!(config.file_path, defaults::default_log_file_path_spec());
        assert_eq!(config.format, defaults::default_log_format_spec());
    }

    #[test]
    fn test_logging_config_deserialize_partial() {
        let json = r#"{"level": "debug"}"#;
        let config: LoggingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.level, "debug");
        assert_eq!(config.file_path, defaults::default_log_file_path_spec());
        assert_eq!(config.format, defaults::default_log_format_spec());

        let json_with_path = r#"{"file_path": "/var/log/novade.log"}"#;
        let config_with_path: LoggingConfig = serde_json::from_str(json_with_path).unwrap();
        assert_eq!(config_with_path.file_path, Some(PathBuf::from("/var/log/novade.log")));
    }
    
    #[test]
    fn test_logging_config_deserialize_full() {
        let json = r#"{"level": "trace", "file_path": "/tmp/app.log", "format": "json"}"#;
        let config: LoggingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.level, "trace");
        assert_eq!(config.file_path, Some(PathBuf::from("/tmp/app.log")));
        assert_eq!(config.format, "json");
    }

    #[test]
    fn test_core_config_deserialize_empty() {
        let json = "{}";
        let config: CoreConfig = serde_json::from_str(json).unwrap();
        // This will use default_core_logging_config, which itself uses other defaults
        let default_log_conf = defaults::default_core_logging_config();
        assert_eq!(config.logging.level, default_log_conf.level);
        assert_eq!(config.logging.file_path, default_log_conf.file_path);
        assert_eq!(config.logging.format, default_log_conf.format);
    }

    #[test]
    fn test_core_config_deserialize_with_logging() {
        let json = r#"{
            "logging": {
                "level": "warn",
                "file_path": "/var/log/core.log",
                "format": "json"
            }
        }"#;
        let config: CoreConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.logging.level, "warn");
        assert_eq!(config.logging.file_path, Some(PathBuf::from("/var/log/core.log")));
        assert_eq!(config.logging.format, "json");
    }

    #[test]
    #[should_panic] // Because deny_unknown_fields is used
    fn test_logging_config_deserialize_unknown_field() {
        let json = r#"{"level": "info", "unknown_field": "value"}"#;
        let _config: LoggingConfig = serde_json::from_str(json).unwrap();
    }

    #[test]
    #[should_panic] // Because deny_unknown_fields is used
    fn test_core_config_deserialize_unknown_field() {
        let json = r#"{"logging": {}, "unknown_field": "value"}"#;
        let _config: CoreConfig = serde_json::from_str(json).unwrap();
    }
}
