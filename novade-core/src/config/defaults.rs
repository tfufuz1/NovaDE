//! Default configuration values for NovaDE Core.
//!
//! These functions are used by `serde`'s `default` attribute in the configuration
//! structures to provide sensible default values when they are not specified in
//! the configuration file.

use crate::config::{LoggingConfig, FeatureFlags}; // Use types from config::mod
use std::path::PathBuf;

/// Returns the default `LoggingConfig`.
///
/// Used by `CoreConfig` if the `logging` section is missing from `config.toml`.
pub(super) fn default_logging_config() -> LoggingConfig {
    LoggingConfig {
        level: default_log_level(),
        file_path: default_log_file_path(),
        format: default_log_format(),
    }
}

/// Returns the default log level string (`"info"`).
///
/// Used by `LoggingConfig` if `level` is not specified.
pub(super) fn default_log_level() -> String {
    "info".to_string()
}

/// Returns the default log file path (`None`).
///
/// Used by `LoggingConfig` if `file_path` is not specified.
pub(super) fn default_log_file_path() -> Option<PathBuf> {
    None // No log file by default
}

/// Returns the default log format string (`"text"`).
///
/// Used by `LoggingConfig` if `format` is not specified.
pub(super) fn default_log_format() -> String {
    "text".to_string()
}

/// Returns the default `FeatureFlags` configuration.
///
/// Used by `CoreConfig` if the `feature_flags` section is missing.
/// This simply calls `FeatureFlags::default()`.
pub(super) fn default_feature_flags() -> FeatureFlags {
    FeatureFlags::default()
}

/// Returns a default boolean value of `false`.
///
/// Used by individual feature flags if not specified.
pub(super) fn default_bool_false() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_log_level() {
        assert_eq!(default_log_level(), "info");
    }

    #[test]
    fn test_default_log_file_path() {
        assert_eq!(default_log_file_path(), None);
    }

    #[test]
    fn test_default_log_format() {
        assert_eq!(default_log_format(), "text");
    }

    #[test]
    fn test_default_logging_config_values() {
        let lc = default_logging_config();
        assert_eq!(lc.level, "info");
        assert_eq!(lc.file_path, None);
        assert_eq!(lc.format, "text");
    }
    
    #[test]
    fn test_default_feature_flags_values() {
        let ff = default_feature_flags();
        // This relies on FeatureFlags::default() being correct.
        // If FeatureFlags::default() is derived, its fields will use their own defaults.
        assert_eq!(ff, FeatureFlags { experimental_feature_x: false });
    }

    #[test]
    fn test_default_bool_false() {
        assert_eq!(default_bool_false(), false);
    }
}
