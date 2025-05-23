//! Default Configuration Values for NovaDE Core.
//!
//! This module provides functions that return default values for various
//! configuration settings defined in [`super::types`]. These functions are
//! used by `serde` when deserializing configuration structures if specific
//! fields are missing from the configuration source.
//!
//! The defaults aim to provide a sensible out-of-the-box experience while
//! allowing users to override them as needed.

use std::path::PathBuf;
use super::types::LoggingConfig;

/// Returns the default log level specification string.
///
/// This value is used as the default for [`LoggingConfig::level`].
/// Currently defaults to `"info"`.
pub fn default_log_level_spec() -> String {
    "info".to_string()
}

/// Returns the default log file path specification.
///
/// This value is used as the default for [`LoggingConfig::file_path`].
/// Currently defaults to `None`, meaning file logging is disabled by default.
/// If a default path were desired, it could be constructed here (e.g., using
/// functions from `crate::utils::paths`).
pub fn default_log_file_path_spec() -> Option<PathBuf> {
    None // No default log file path; logging to file is disabled unless explicitly configured.
}

/// Returns the default log format specification string.
///
/// This value is used as the default for [`LoggingConfig::format`].
/// Currently defaults to `"text"`.
pub fn default_log_format_spec() -> String {
    "text".to_string()
}

/// Constructs a default [`LoggingConfig`] instance.
///
/// This function is used as the `serde` default for the `logging` field
/// in [`super::types::CoreConfig`]. It aggregates the individual default
/// settings for logging level, file path, and format.
pub fn default_core_logging_config() -> LoggingConfig {
    LoggingConfig {
        level: default_log_level_spec(),
        file_path: default_log_file_path_spec(),
        format: default_log_format_spec(),
    }
}

// Removed old default functions that are no longer directly used by the new CoreConfig/LoggingConfig structure
// - default_log_to_console() (not part of new LoggingConfig spec)
// - default_app_name(), default_app_version(), default_data_dir(), etc. (part of removed ApplicationConfig)
// - default_worker_threads(), default_use_hardware_acceleration() (part of removed SystemConfig)

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_log_level_spec() {
        assert_eq!(default_log_level_spec(), "info");
    }
    
    #[test]
    fn test_default_log_file_path_spec() {
        assert_eq!(default_log_file_path_spec(), None);
    }
    
    #[test]
    fn test_default_log_format_spec() {
        assert_eq!(default_log_format_spec(), "text");
    }
    
    #[test]
    fn test_default_core_logging_config() {
        let logging_config = default_core_logging_config();
        assert_eq!(logging_config.level, "info");
        assert_eq!(logging_config.file_path, None);
        assert_eq!(logging_config.format, "text");
    }
}
