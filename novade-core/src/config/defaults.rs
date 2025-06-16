//! Default configuration values for NovaDE Core.
//!
//! These functions are used by `serde`'s `default` attribute in the configuration
//! structures to provide sensible default values when they are not specified in
//! the configuration file.

//ANCHOR [NovaDE Developers <dev@novade.org>] Import new config types.
use crate::config::{
    LoggingConfig, FeatureFlags, LogOutput, LogRotation, LogFormat,
    ErrorTrackingConfig, MetricsExporterConfig, DebugInterfaceConfig
};
use crate::types::system_health::SystemHealthDashboardConfig;
use std::path::PathBuf;

// --- Logging Defaults ---

//ANCHOR [NovaDE Developers <dev@novade.org>] Default LoggingConfig.
/// Returns the default `LoggingConfig`.
pub(super) fn default_logging_config() -> LoggingConfig {
    LoggingConfig {
        log_level: default_log_level_string(),
        log_output: default_log_output(),
        log_format: default_log_format_enum(),
    }
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Default log level string.
/// Returns the default log level string (`"info"`).
pub(super) fn default_log_level_string() -> String {
    "info".to_string()
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Default LogOutput.
/// Returns the default `LogOutput` (`Stdout`).
pub(super) fn default_log_output() -> LogOutput {
    LogOutput::Stdout // Default to stdout
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Default LogFormat enum.
/// Returns the default `LogFormat` (`Text`).
pub(super) fn default_log_format_enum() -> LogFormat {
    LogFormat::Text // Default to text format
}

// Note: default_log_file_path and default_log_format (string version) are no longer directly used by LoggingConfig
// but might be useful if other parts of the system expect these specific default values.
// For now, they are effectively replaced by default_log_output and default_log_format_enum.

// --- Error Tracking Defaults ---

//ANCHOR [NovaDE Developers <dev@novade.org>] Default ErrorTrackingConfig.
/// Returns the default `ErrorTrackingConfig`.
pub(super) fn default_error_tracking_config() -> ErrorTrackingConfig {
    ErrorTrackingConfig {
        sentry_dsn: default_optional_string(),
        sentry_environment: default_optional_string(),
        sentry_release: default_optional_string(),
    }
}

// --- Metrics Exporter Defaults ---

//ANCHOR [NovaDE Developers <dev@novade.org>] Default MetricsExporterConfig.
/// Returns the default `MetricsExporterConfig`.
pub(super) fn default_metrics_exporter_config() -> MetricsExporterConfig {
    MetricsExporterConfig {
        metrics_exporter_enabled: default_bool_false(),
        metrics_exporter_address: default_metrics_exporter_address(),
    }
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Default metrics exporter address.
/// Returns the default metrics exporter address (`"0.0.0.0:9090"`).
pub(super) fn default_metrics_exporter_address() -> String {
    "0.0.0.0:9090".to_string()
}

// --- Debug Interface Defaults ---

//ANCHOR [NovaDE Developers <dev@novade.org>] Default DebugInterfaceConfig.
/// Returns the default `DebugInterfaceConfig`.
pub(super) fn default_debug_interface_config() -> DebugInterfaceConfig {
    DebugInterfaceConfig {
        debug_interface_enabled: default_bool_false(),
        debug_interface_address: default_optional_string(),
    }
}

// --- General Defaults ---

//ANCHOR [NovaDE Developers <dev@novade.org>] Default Option<String>.
/// Returns a default `Option<String>` which is `None`.
pub(super) fn default_optional_string() -> Option<String> {
    None
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
    use std::path::PathBuf; // For testing LogOutput::File

    #[test]
    fn test_default_log_level_string() { // Renamed test
        assert_eq!(default_log_level_string(), "info");
    }

    // Tests for default_log_file_path and default_log_format (string versions) can be removed
    // if they are no longer used, or kept if they serve other purposes.
    // For now, removing them as LoggingConfig has changed.

    #[test]
    fn test_default_log_output() {
        assert_eq!(default_log_output(), LogOutput::Stdout);
    }

    #[test]
    fn test_default_log_format_enum() { // Renamed test
        assert_eq!(default_log_format_enum(), LogFormat::Text);
    }

    #[test]
    fn test_default_logging_config_values() {
        let lc = default_logging_config();
        assert_eq!(lc.log_level, "info");
        assert_eq!(lc.log_output, LogOutput::Stdout); // Check new field
        assert_eq!(lc.log_format, LogFormat::Text);   // Check new field
    }

    #[test]
    fn test_default_error_tracking_config_values() {
        let etc = default_error_tracking_config();
        assert_eq!(etc.sentry_dsn, None);
        assert_eq!(etc.sentry_environment, None);
        assert_eq!(etc.sentry_release, None);
    }

    #[test]
    fn test_default_metrics_exporter_config_values() {
        let mec = default_metrics_exporter_config();
        assert_eq!(mec.metrics_exporter_enabled, false);
        assert_eq!(mec.metrics_exporter_address, "0.0.0.0:9090");
    }

    #[test]
    fn test_default_debug_interface_config_values() {
        let dic = default_debug_interface_config();
        assert_eq!(dic.debug_interface_enabled, false);
        assert_eq!(dic.debug_interface_address, None);
    }
    
    #[test]
    fn test_default_optional_string() {
        assert_eq!(default_optional_string(), None);
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
