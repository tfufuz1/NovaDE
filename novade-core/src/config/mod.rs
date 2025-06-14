//! Configuration management for NovaDE Core.
//!
//! This module defines the core configuration structures (`CoreConfig`, `LoggingConfig`,
//! `FeatureFlags`), provides mechanisms for loading these configurations from files
//! (`ConfigLoader`), and offers global access to the loaded configuration.
//!
//! # Structures
//!
//! - [`CoreConfig`]: The main configuration structure for the entire application.
//! - [`LoggingConfig`]: Configuration specific to the logging subsystem.
//! - [`FeatureFlags`]: Configuration for enabling or disabling experimental features.
//!
//! # Loading Configuration
//!
//! The [`ConfigLoader`] struct is responsible for loading `CoreConfig` from a `config.toml`
//! file. It handles parsing, applying defaults, and validating the configuration.
//!
//! # Global Access
//!
//! Once loaded, the `CoreConfig` can be initialized globally using [`initialize_core_config()`].
//! Subsequent access to the configuration is then provided by [`get_core_config()`].
//!
//! # Example
//!
//! ```rust,ignore
//! // In your main application setup:
//! use novade_core::config::{ConfigLoader, initialize_core_config};
//! use novade_core::error::CoreError;
//!
//! fn setup() -> Result<(), CoreError> {
//!     let config = ConfigLoader::load()?;
//!     initialize_core_config(config)
//!         .map_err(|_config| CoreError::Internal("Config already initialized".to_string()))?;
//!     Ok(())
//! }
//!
//! // Elsewhere in the application:
//! use novade_core::config::get_core_config;
//!
//! fn use_config() {
//!     let config = get_core_config();
//!     println!("Log level: {}", config.logging.level);
//!     if config.feature_flags.experimental_feature_x {
//!         // ...
//!     }
//! }
//! ```

use crate::error::{ConfigError, CoreError};
use crate::utils; // For utils::paths and utils::fs
use once_cell::sync::Lazy; // Changed from OnceCell
use serde::Deserialize;
use std::sync::RwLock; // Added for RwLock
use std::path::PathBuf; // Still used by LoggingConfig

pub mod defaults;
pub mod loader; // Import the loader module
pub mod watcher; // Add watcher module
pub use loader::ConfigLoader; // Re-export ConfigLoader

// --- Configuration Data Structures ---

/// Core configuration for the NovaDE application.
///
/// Holds settings for various subsystems like logging and feature flags.
/// This structure is typically loaded from a `config.toml` file.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct CoreConfig {
    /// Logging configuration settings.
    #[serde(default = "defaults::default_logging_config")]
    pub logging: LoggingConfig,

    /// Feature flags for enabling/disabling experimental or optional features.
    #[serde(default = "defaults::default_feature_flags")]
    pub feature_flags: FeatureFlags,

    /// Compositor configuration settings.
    #[serde(default = "defaults::default_compositor_config")]
    pub compositor: CompositorConfig,
}

/// Configuration for the NovaDE Compositor.
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CompositorConfig {
    /// Performance-related settings for the compositor.
    #[serde(default = "defaults::default_performance_config")]
    pub performance: PerformanceConfig,

    /// Input device and mapping configuration.
    #[serde(default = "defaults::default_input_config")]
    pub input: InputConfig,

    /// Visual appearance and theming settings.
    #[serde(default = "defaults::default_visual_config")]
    pub visual: VisualConfig,
}

/// Compositor performance settings.
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PerformanceConfig {
    /// Preferred GPU or "auto" for automatic detection.
    #[serde(default = "defaults::default_gpu_preference")]
    pub gpu_preference: String,

    /// Quality preset (e.g., "low", "medium", "high", "custom").
    #[serde(default = "defaults::default_quality_preset")]
    pub quality_preset: String,

    /// Optional memory limit for GPU resources (in MB).
    #[serde(default)]
    pub memory_limit_mb: Option<u32>,

    /// Power management mode (e.g., "balanced", "performance", "power-saver").
    #[serde(default = "defaults::default_power_management_mode")]
    pub power_management_mode: String,

    /// Enable or disable adaptive tuning of performance settings.
    #[serde(default = "defaults::default_bool_true")] // Assuming true is a sensible default
    pub adaptive_tuning: bool,
}

/// Compositor input settings.
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InputConfig {
    /// Keyboard layout (e.g., "us", "de", "fr").
    #[serde(default = "defaults::default_keyboard_layout")]
    pub keyboard_layout: String,

    /// Pointer acceleration profile or factor.
    #[serde(default = "defaults::default_pointer_acceleration")]
    pub pointer_acceleration: String, // Could be a float or an enum-like string

    /// Enable or disable common touch gestures.
    #[serde(default = "defaults::default_bool_true")]
    pub enable_touch_gestures: bool,

    /// Configuration for multi-device support (e.g., "auto", "manual").
    #[serde(default = "defaults::default_multi_device_support")]
    pub multi_device_support: String,

    /// List of input profile names or paths.
    #[serde(default)]
    pub input_profiles: Vec<String>,
}

/// Compositor visual settings.
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VisualConfig {
    /// Current theme name.
    #[serde(default = "defaults::default_theme_name")]
    pub theme_name: String,

    /// Active color scheme (e.g., "light", "dark", "custom").
    #[serde(default = "defaults::default_color_scheme")]
    pub color_scheme: String,

    /// Default font name or family.
    #[serde(default = "defaults::default_font_settings")] // Assuming a struct or string for fonts
    pub font_settings: String, // Placeholder, might become a struct FontConfig

    /// Enable or disable UI animations.
    #[serde(default = "defaults::default_bool_true")]
    pub enable_animations: bool,

    /// List of paths to custom shader files.
    #[serde(default)]
    pub custom_shaders: Vec<String>,
}

/// Configuration for the logging subsystem.
///
/// Defines log level, optional file output path, and log message format.
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    /// The minimum log level to output (e.g., "info", "debug", "trace").
    #[serde(default = "defaults::default_log_level")]
    pub level: String,

    /// Optional path to a file where logs should be written.
    /// If `None`, logs might only go to stdout/stderr or other configured appenders.
    #[serde(default = "defaults::default_log_file_path")]
    pub file_path: Option<PathBuf>,

    /// The format for log messages (e.g., "text", "json").
    #[serde(default = "defaults::default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        defaults::default_logging_config()
    }
}

/// Configuration for feature flags.
///
/// Allows toggling experimental or optional features within the application.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct FeatureFlags {
    /// Example of an experimental feature flag.
    #[serde(default = "defaults::default_bool_false")]
    pub experimental_feature_x: bool,
}

// --- Global Config Access ---

/// Global configuration, initialized lazily with a default and protected by an RwLock.
static CORE_CONFIG: Lazy<RwLock<CoreConfig>> = Lazy::new(|| {
    // Initialize with a default configuration.
    // `initialize_core_config` will be used to set the first "real" configuration.
    RwLock::new(CoreConfig::default())
});

/// Initializes or updates the global `CoreConfig`.
///
/// This function acquires a write lock on the global configuration and replaces
/// the existing `CoreConfig` with the new one.
///
/// # Arguments
///
/// * `config`: The `CoreConfig` instance to set globally.
///
/// # Returns
///
/// * `Ok(())` if the configuration was successfully set.
/// * `Err(ConfigError::PoisonedLock)` if the `RwLock` was poisoned.
pub fn initialize_core_config(config: CoreConfig) -> Result<(), ConfigError> {
    match CORE_CONFIG.write() {
        Ok(mut guard) => {
            *guard = config;
            Ok(())
        }
        Err(poisoned) => {
            Err(ConfigError::PoisonedLock(poisoned.to_string()))
        }
    }
}

/// Updates the global `CoreConfig` with a new configuration.
///
/// This is intended for use by the configuration watcher or other internal mechanisms
/// that need to reload and apply a new configuration.
///
/// # Arguments
///
/// * `new_config`: The new `CoreConfig` instance.
///
/// # Returns
///
/// * `Ok(())` if the global configuration was successfully updated.
/// * `Err(ConfigError::PoisonedLock)` if the `RwLock` was poisoned.
pub(crate) fn update_global_config(new_config: CoreConfig) -> Result<(), ConfigError> {
    match CORE_CONFIG.write() {
        Ok(mut guard) => {
            *guard = new_config;
            Ok(())
        }
        Err(poisoned) => {
            Err(ConfigError::PoisonedLock(poisoned.to_string()))
        }
    }
}

/// Retrieves a clone of the globally initialized `CoreConfig`.
///
/// This function acquires a read lock on the global configuration and returns
/// a clone of the `CoreConfig`.
///
/// # Panics
///
/// Panics if the `RwLock` holding the configuration is poisoned. This indicates
/// a previous write attempt failed catastrophically.
pub fn get_cloned_core_config() -> CoreConfig {
    match CORE_CONFIG.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => {
            // This panic is consistent with the previous `expect` on OnceCell.get()
            // if the config wasn't initialized. Here, it means the lock is broken.
            panic!("CORE_CONFIG RwLock is poisoned: {}", poisoned);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    // Tests for ConfigLoader (load, validate_config) are in loader.rs.
    // Tests for global config access (initialize_core_config, get_cloned_core_config) are here.

    #[test]
    fn test_initialize_and_get_cloned_core_config() {
        let initial_default_config = get_cloned_core_config();
        assert_eq!(initial_default_config, CoreConfig::default(), "Initial config should be default from Lazy init");

        let mut test_config = CoreConfig::default();
        test_config.logging.level = "debug_test".to_string();
        test_config.feature_flags.experimental_feature_x = true;

        initialize_core_config(test_config.clone()).expect("Failed to initialize core config");
        
        let retrieved_config = get_cloned_core_config();
        assert_eq!(retrieved_config, test_config);
        assert_eq!(retrieved_config.logging.level, "debug_test");
        assert_eq!(retrieved_config.feature_flags.experimental_feature_x, true);

        // Test re-initialization (update)
        let mut updated_config = CoreConfig::default();
        updated_config.logging.level = "trace_updated".to_string();
        initialize_core_config(updated_config.clone()).expect("Failed to update core config");

        let retrieved_updated_config = get_cloned_core_config();
        assert_eq!(retrieved_updated_config, updated_config);
        assert_eq!(retrieved_updated_config.logging.level, "trace_updated");
    }

    #[test]
    fn test_update_global_config() {
        // Ensure a known initial state (can be default or set by initialize_core_config)
        let initial_config = CoreConfig {
            logging: LoggingConfig { level: "initial_for_update".to_string(), ..Default::default()},
            ..Default::default()
        };
        initialize_core_config(initial_config.clone()).unwrap();
        assert_eq!(get_cloned_core_config().logging.level, "initial_for_update");
        
        let mut new_config_for_update = CoreConfig::default();
        new_config_for_update.logging.level = "updated_by_watcher_sim".to_string();
        new_config_for_update.feature_flags.experimental_feature_x = true;

        update_global_config(new_config_for_update.clone()).expect("update_global_config failed");

        let retrieved_config = get_cloned_core_config();
        assert_eq!(retrieved_config, new_config_for_update);
        assert_eq!(retrieved_config.logging.level, "updated_by_watcher_sim");
        assert_eq!(retrieved_config.feature_flags.experimental_feature_x, true);
    }

    // The panic test for uninitialized get_core_config is no longer relevant
    // because `Lazy<RwLock<CoreConfig>>` ensures CORE_CONFIG is always initialized
    // with a default value (CoreConfig::default()).
    // `get_cloned_core_config` will panic if the lock is poisoned, but that's harder
    // to reliably trigger in a unit test without complex threading scenarios.
}
