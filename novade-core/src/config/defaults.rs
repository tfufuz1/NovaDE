//! Default configuration values for NovaDE Core.
//!
//! These functions are used by `serde`'s `default` attribute in the configuration
//! structures to provide sensible default values when they are not specified in
//! the configuration file.

use crate::config::{
    LoggingConfig, FeatureFlags, CompositorConfig, PerformanceConfig, InputConfig, VisualConfig,
}; // Use types from config::mod
use std::path::PathBuf;

// --- Defaults for CompositorConfig ---

/// Returns the default `CompositorConfig`.
pub(super) fn default_compositor_config() -> CompositorConfig {
    CompositorConfig {
        performance: default_performance_config(),
        input: default_input_config(),
        visual: default_visual_config(),
    }
}

/// Returns the default `PerformanceConfig`.
pub(super) fn default_performance_config() -> PerformanceConfig {
    PerformanceConfig {
        gpu_preference: default_gpu_preference(),
        quality_preset: default_quality_preset(),
        memory_limit_mb: None, // No limit by default
        power_management_mode: default_power_management_mode(),
        adaptive_tuning: default_bool_true(), // Assuming true is a general utility
    }
}

/// Returns the default `InputConfig`.
pub(super) fn default_input_config() -> InputConfig {
    InputConfig {
        keyboard_layout: default_keyboard_layout(),
        pointer_acceleration: default_pointer_acceleration(),
        enable_touch_gestures: default_bool_true(),
        multi_device_support: default_multi_device_support(),
        input_profiles: Vec::new(), // Empty list by default
    }
}

/// Returns the default `VisualConfig`.
pub(super) fn default_visual_config() -> VisualConfig {
    VisualConfig {
        theme_name: default_theme_name(),
        color_scheme: default_color_scheme(),
        font_settings: default_font_settings(),
        enable_animations: default_bool_true(),
        custom_shaders: Vec::new(), // Empty list by default
    }
}

// --- Defaults for PerformanceConfig fields ---

/// Default GPU preference: "auto".
pub(super) fn default_gpu_preference() -> String {
    "auto".to_string()
}

/// Default quality preset: "medium".
pub(super) fn default_quality_preset() -> String {
    "medium".to_string()
}

/// Default power management mode: "balanced".
pub(super) fn default_power_management_mode() -> String {
    "balanced".to_string()
}

// --- Defaults for InputConfig fields ---

/// Default keyboard layout: "us".
pub(super) fn default_keyboard_layout() -> String {
    "us".to_string()
}

/// Default pointer acceleration: "default".
pub(super) fn default_pointer_acceleration() -> String {
    "default".to_string() // Or perhaps a sensible numeric string like "1.0"
}

/// Default multi-device support: "auto".
pub(super) fn default_multi_device_support() -> String {
    "auto".to_string()
}

// --- Defaults for VisualConfig fields ---

/// Default theme name: "NovaDefault".
pub(super) fn default_theme_name() -> String {
    "NovaDefault".to_string()
}

/// Default color scheme: "light".
pub(super) fn default_color_scheme() -> String {
    "light".to_string()
}

/// Default font settings: "system-ui".
pub(super) fn default_font_settings() -> String {
    "system-ui".to_string() // A common request for system default fonts
}


// --- General Utility Defaults ---

/// Returns a default boolean value of `true`.
pub(super) fn default_bool_true() -> bool {
    true
}

// --- Defaults for LoggingConfig and FeatureFlags (existing) ---

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

    #[test]
    fn test_default_bool_true() {
        assert_eq!(default_bool_true(), true);
    }

    // --- Tests for CompositorConfig defaults ---

    #[test]
    fn test_default_gpu_preference() {
        assert_eq!(default_gpu_preference(), "auto");
    }

    #[test]
    fn test_default_quality_preset() {
        assert_eq!(default_quality_preset(), "medium");
    }

    #[test]
    fn test_default_power_management_mode() {
        assert_eq!(default_power_management_mode(), "balanced");
    }

    #[test]
    fn test_default_performance_config_values() {
        let pc = default_performance_config();
        assert_eq!(pc.gpu_preference, "auto");
        assert_eq!(pc.quality_preset, "medium");
        assert_eq!(pc.memory_limit_mb, None);
        assert_eq!(pc.power_management_mode, "balanced");
        assert_eq!(pc.adaptive_tuning, true);
    }

    #[test]
    fn test_default_keyboard_layout() {
        assert_eq!(default_keyboard_layout(), "us");
    }

    #[test]
    fn test_default_pointer_acceleration() {
        assert_eq!(default_pointer_acceleration(), "default");
    }

    #[test]
    fn test_default_multi_device_support() {
        assert_eq!(default_multi_device_support(), "auto");
    }

    #[test]
    fn test_default_input_config_values() {
        let ic = default_input_config();
        assert_eq!(ic.keyboard_layout, "us");
        assert_eq!(ic.pointer_acceleration, "default");
        assert_eq!(ic.enable_touch_gestures, true);
        assert_eq!(ic.multi_device_support, "auto");
        assert!(ic.input_profiles.is_empty());
    }

    #[test]
    fn test_default_theme_name() {
        assert_eq!(default_theme_name(), "NovaDefault");
    }

    #[test]
    fn test_default_color_scheme() {
        assert_eq!(default_color_scheme(), "light");
    }

    #[test]
    fn test_default_font_settings() {
        assert_eq!(default_font_settings(), "system-ui");
    }

    #[test]
    fn test_default_visual_config_values() {
        let vc = default_visual_config();
        assert_eq!(vc.theme_name, "NovaDefault");
        assert_eq!(vc.color_scheme, "light");
        assert_eq!(vc.font_settings, "system-ui");
        assert_eq!(vc.enable_animations, true);
        assert!(vc.custom_shaders.is_empty());
    }

    #[test]
    fn test_default_compositor_config_values() {
        let cc = default_compositor_config();
        assert_eq!(cc.performance, default_performance_config());
        assert_eq!(cc.input, default_input_config());
        assert_eq!(cc.visual, default_visual_config());
    }
}
