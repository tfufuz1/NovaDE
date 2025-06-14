//! Default configuration values for NovaDE Core.
//!
//! These functions are used by `serde`'s `default` attribute in the configuration
//! structures to provide sensible default values when they are not specified in
//! the configuration file.

use crate::config::{LoggingConfig, FeatureFlags}; // Use types from config::mod
use crate::types::system_health::SystemHealthDashboardConfig;
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

/// Returns the default `SystemHealthDashboardConfig`.
///
/// Used by `CoreConfig` if the `system_health` section is missing.
pub(super) fn default_system_health_config() -> SystemHealthDashboardConfig {
    SystemHealthDashboardConfig::default()
}

// --- Assistant Configuration ---
// The following section outlines the default configuration structure for the Smart Assistant.
// Actual implementation would involve integrating `AssistantPreferences` from `crate::types::assistant`.

/*
Example structure for assistant configuration within a larger config file (e.g., TOML):

[assistant]
enabled = true
language = "en-US"
activation_method = "hotword" # Options: "hotword", "keybinding", "manual"
hotword = "Nova"
# keybinding = "Super+Space" # Example if using keybinding
voice_feedback_enabled = true

[assistant.privacy_settings]
share_usage_data = false
keep_history = true
allow_location_access = false

# Skill-specific settings would be nested, e.g.:
# [assistant.skills."com.novade.skills.weather"]
# api_key = "your_default_or_placeholder_api_key"
# preferred_units = "celsius"

*/

/// Placeholder function to illustrate where default assistant preferences would be defined.
#[allow(dead_code)]
fn default_assistant_preferences() -> crate::types::assistant::AssistantPreferences {
    use crate::types::assistant::AssistantPreferences;
    use std::collections::HashMap;

    AssistantPreferences {
        enabled: true,
        language: "en-US".to_string(),
        activation_method: "hotword".to_string(),
        hotword: Some("Nova".to_string()),
        keybinding: None,
        voice_feedback_enabled: true,
        privacy_settings: {
            let mut map = HashMap::new();
            map.insert("share_usage_data".to_string(), false);
            map.insert("keep_history".to_string(), true);
            map.insert("allow_location_access".to_string(), false);
            map
        },
        skill_settings: {
            let mut map = HashMap::new();
            // Example default skill setting
            let mut weather_settings = HashMap::new();
            weather_settings.insert("preferred_units".to_string(), "celsius".to_string());
            map.insert("com.novade.skills.weather".to_string(), weather_settings);
            map
        },
    }
}

// TODO: Integrate these defaults properly into the overall configuration loading mechanism.
// TODO: Ensure the main configuration struct in `novade-core/src/config/mod.rs`
//       includes a field for `AssistantPreferences`.

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
        // Note: If CoreConfig::default() is used, it will also include SystemHealthDashboardConfig::default()
        // due to the #[serde(default)] attributes.
    }

    #[test]
    fn test_default_bool_false() {
        assert_eq!(default_bool_false(), false);
    }

    #[test]
    fn test_default_system_health_config_values() {
        let shc = default_system_health_config();
        // Check against SystemHealthDashboardConfig::default()
        assert_eq!(shc, SystemHealthDashboardConfig::default());
        // Example check for a specific default value within SystemHealthDashboardConfig
        assert_eq!(shc.metric_refresh_interval_ms, 1000);
    }
}
