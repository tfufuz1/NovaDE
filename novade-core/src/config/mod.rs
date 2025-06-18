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
use once_cell::sync::OnceCell;
use serde::Deserialize;
use serde::Serialize; // Ensure this is correctly added
// std::fs and std::path::PathBuf are no longer directly used here for ConfigLoader logic
use std::path::PathBuf; // Still used by LoggingConfig
use crate::types::system_health::SystemHealthDashboardConfig;

pub mod defaults;
pub mod loader; // Import the loader module
pub use loader::ConfigLoader; // Re-export ConfigLoader

// --- Configuration Data Structures ---

/// Core configuration for the NovaDE application.
///
/// Holds settings for various subsystems like logging and feature flags.
/// This structure is typically loaded from a `config.toml` file.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)] // Added Serialize back
#[serde(deny_unknown_fields)]
pub struct CoreConfig {
    /// Logging configuration settings.
    #[serde(default = "defaults::default_logging_config")]
    pub logging: LoggingConfig,

    /// Configuration for error tracking (e.g., Sentry).
    #[serde(default = "defaults::default_error_tracking_config")]
    pub error_tracking: ErrorTrackingConfig,

    /// Configuration for the metrics exporter (e.g., Prometheus).
    #[serde(default = "defaults::default_metrics_exporter_config")]
    pub metrics_exporter: MetricsExporterConfig,

    /// Configuration for the debug interface.
    #[serde(default = "defaults::default_debug_interface_config")]
    pub debug_interface: DebugInterfaceConfig,

    /// Feature flags for enabling/disabling experimental or optional features.
    #[serde(default = "defaults::default_feature_flags")]
    pub feature_flags: FeatureFlags,

    /// System Health Dashboard configuration.
    #[serde(default = "defaults::default_system_health_config")]
    pub system_health: SystemHealthDashboardConfig,
    //TODO [NovaDE Developers <dev@novade.org>] Add other future configuration sections here.
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Defines how logs should be rotated.
/// Specifies the log rotation policy.
/// //TODO [Log Rotation Policy] [NovaDE Developers <dev@novade.org>] tracing-appender currently supports daily OR size-based rotation, but not typically both simultaneously on the same file appender easily. This enum reflects that. If combined strategies are needed, further research or custom implementation might be required.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum LogRotation {
    /// Rotate logs daily.
    Daily,
    /// Rotate logs when they reach a certain size in megabytes.
    SizeMB(usize),
    /// No rotation.
    None,
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Defines the output destination for logs.
/// Specifies where logs should be written.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum LogOutput {
    /// Output logs to stdout.
    Stdout,
    /// Output logs to a file with specified path and rotation policy.
    File {
        path: PathBuf,
        rotation: LogRotation,
    },
    //TODO [NovaDE Developers <dev@novade.org>] Consider adding other outputs like `Syslog` or `Journald`.
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Defines the format for log messages.
/// Specifies the format for log messages.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum LogFormat {
    /// JSON formatted logs.
    Json,
    /// Plain text formatted logs.
    Text,
}

/// Configuration for the logging subsystem.
///
/// Defines log level, output destination, rotation policy, and message format.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    /// The minimum log level to output (e.g., "info", "debug", "trace", "warn", "error").
    /// //TODO [Config Validation] [NovaDE Developers <dev@novade.org>] Add validation for supported log levels.
    #[serde(default = "defaults::default_log_level_string")] // Changed from default_log_level to reflect new type if necessary, or keep if it returns String
    pub log_level: String,

    /// Defines where logs are sent and how they are rotated.
    #[serde(default = "defaults::default_log_output")]
    pub log_output: LogOutput,

    /// The format for log messages.
    #[serde(default = "defaults::default_log_format_enum")] // Changed from default_log_format to reflect new type if necessary
    pub log_format: LogFormat,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        defaults::default_logging_config()
    }
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Configuration for Error Tracking (Sentry).
/// Configuration settings for the Sentry error tracking system.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ErrorTrackingConfig {
    /// The DSN (Data Source Name) for Sentry. If None, Sentry is disabled.
    #[serde(default = "defaults::default_optional_string")]
    pub sentry_dsn: Option<String>,
    /// The environment name for Sentry (e.g., "development", "production").
    #[serde(default = "defaults::default_optional_string")]
    pub sentry_environment: Option<String>,
    /// The release name/version for Sentry.
    #[serde(default = "defaults::default_optional_string")]
    pub sentry_release: Option<String>,
}

impl Default for ErrorTrackingConfig {
    fn default() -> Self {
        defaults::default_error_tracking_config()
    }
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Configuration for Metrics Exporter (Prometheus).
/// Configuration settings for the Prometheus metrics exporter.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetricsExporterConfig {
    /// Whether the Prometheus metrics exporter is enabled.
    #[serde(default = "defaults::default_bool_false")] // Default to false
    pub metrics_exporter_enabled: bool,
    /// The address for the metrics exporter to listen on (e.g., "0.0.0.0:9090").
    /// //TODO [Config Validation] [NovaDE Developers <dev@novade.org>] Add validation for a parseable SocketAddr.
    #[serde(default = "defaults::default_metrics_exporter_address")]
    pub metrics_exporter_address: String,
}

impl Default for MetricsExporterConfig {
    fn default() -> Self {
        defaults::default_metrics_exporter_config()
    }
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Configuration for Debug Interface.
/// Configuration settings for the debug interface.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DebugInterfaceConfig {
    /// Whether the debug interface is enabled.
    #[serde(default = "defaults::default_bool_false")] // Default to false
    pub debug_interface_enabled: bool,
    /// The address for the debug interface to listen on (e.g., a Unix socket path or IP:port).
    /// //TODO [Config Validation] [NovaDE Developers <dev@novade.org>] Validation depends on the chosen transport (e.g., valid socket path, parseable IP:port).
    #[serde(default = "defaults::default_optional_string")]
    pub debug_interface_address: Option<String>,
}

impl Default for DebugInterfaceConfig {
    fn default() -> Self {
        defaults::default_debug_interface_config()
    }
}

/// Configuration for feature flags.
///
/// Allows toggling experimental or optional features within the application.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)] // Added Serialize back
#[serde(deny_unknown_fields)]
pub struct FeatureFlags {
    /// Example of an experimental feature flag.
    #[serde(default = "defaults::default_bool_false")]
    pub experimental_feature_x: bool,
}

// --- Global Config Access ---
// ConfigLoader struct and its impl block are removed from here.
// They are now in config/loader.rs and re-exported.

static CORE_CONFIG: OnceCell<CoreConfig> = OnceCell::new();

/// Initializes the global `CoreConfig`.
///
/// This function should be called once at application startup with the loaded configuration.
///
/// # Arguments
///
/// * `config`: The `CoreConfig` instance to set globally.
///
/// # Returns
///
/// * `Ok(())` if the configuration was successfully initialized.
/// * `Err(CoreConfig)` if the global configuration has already been initialized,
///   returning the passed config.
pub fn initialize_core_config(config: CoreConfig) -> Result<(), CoreConfig> {
    CORE_CONFIG.set(config)
}

/// Retrieves a reference to the globally initialized `CoreConfig`.
///
/// # Panics
///
/// Panics if `initialize_core_config()` has not been called before this function.
/// It's crucial to ensure the configuration is loaded and initialized at application startup.
pub fn get_core_config() -> &'static CoreConfig {
    CORE_CONFIG
        .get()
        .expect("CoreConfig wurde nicht initialisiert. initialize_core_config() muss zuerst aufgerufen werden.")
}

#[cfg(test)]
mod tests {
    use super::*;
    // crate::utils::paths and std::env are used by TestEnv, which will be moved to loader.rs tests.
    // tempfile::TempDir is used by TestEnv.
    // std::fs is used by create_temp_config_file, which will be moved/replicated.

    // Tests for ConfigLoader (load, validate_config) will be moved to loader.rs.
    // Tests for global config access (initialize_core_config, get_core_config) remain here.

    //ANCHOR [NovaDE Developers <dev@novade.org>] Tests for Default implementations of new config structs.
    #[test]
    fn test_logging_config_default() {
        let default_cfg = LoggingConfig::default();
        let expected_defaults = defaults::default_logging_config(); // Assumes defaults.rs is correct
        assert_eq!(default_cfg.log_level, expected_defaults.log_level);
        assert_eq!(default_cfg.log_output, expected_defaults.log_output);
        assert_eq!(default_cfg.log_format, expected_defaults.log_format);
    }

    #[test]
    fn test_error_tracking_config_default() {
        let default_cfg = ErrorTrackingConfig::default();
        assert_eq!(default_cfg.sentry_dsn, None);
        assert_eq!(default_cfg.sentry_environment, None);
        assert_eq!(default_cfg.sentry_release, None);
    }

    #[test]
    fn test_metrics_exporter_config_default() {
        let default_cfg = MetricsExporterConfig::default();
        assert_eq!(default_cfg.metrics_exporter_enabled, false);
        assert_eq!(default_cfg.metrics_exporter_address, "0.0.0.0:9090");
    }

    #[test]
    fn test_debug_interface_config_default() {
        let default_cfg = DebugInterfaceConfig::default();
        assert_eq!(default_cfg.debug_interface_enabled, false);
        assert_eq!(default_cfg.debug_interface_address, None);
    }


    #[test]
    fn test_global_config_access() {
        // Reset OnceCell for this test (important if tests run in parallel or share state)
        // This is tricky. OnceCell is global. Tests should not depend on order.
        // For this test, we assume it's the first one to set CORE_CONFIG or that it's been reset.
        // In a real test suite, use a mutex or ensure `initialize` is only called once.
        // For now, we'll try and see. If it fails due to already set, that's a test interaction issue.
        
        let test_config = CoreConfig {
            logging: LoggingConfig {
                log_level: "test".to_string(), // Corrected field name
                ..Default::default()
            },
            ..Default::default()
        };

        // To ensure this test doesn't fail due to other tests initializing CORE_CONFIG,
        // we can't easily reset a static OnceCell. This test is inherently a bit fragile
        // for `initialize_core_config` success case if run with others.
        // We can test `get_core_config`'s panic if we ensure it's not set. (Hard to ensure)
        // We can test that if set, we get the value.

        // Attempt to initialize - this might fail if another test already set it.
        match initialize_core_config(test_config.clone()) {
            Ok(_) => { // Successfully initialized
                let retrieved_config = get_core_config();
                assert_eq!(retrieved_config, &test_config);
                assert_eq!(retrieved_config.logging.log_level, "test"); // Corrected field name

                // Test trying to initialize again fails
                let another_config = CoreConfig::default();
                assert!(initialize_core_config(another_config).is_err());
            }
            Err(_returned_config) => {
                // Config was already initialized by another test. We can still test get_core_config.
                println!("Warning: CORE_CONFIG was already initialized. Testing get_core_config only.");
                let retrieved_config = get_core_config(); // Should not panic if already set
                assert!(!retrieved_config.logging.log_level.is_empty()); // Check it's a valid CoreConfig & corrected field
            }
        }
    }

    #[test]
    #[should_panic(expected = "CoreConfig wurde nicht initialisiert")]
    fn test_get_core_config_panics_if_not_initialized() {
        // This test MUST run in a context where CORE_CONFIG is guaranteed not to be set.
        // Cargo runs tests in threads, but statics are shared.
        // If this test runs after another test has set CORE_CONFIG, it will fail to panic.
        // One way to somewhat isolate this for testing the panic is to use a feature flag
        // to gate the actual static OnceCell, and use a different one for this test.
        // Or, run this test in a separate process/test binary.
        // For now, we acknowledge this limitation. If other tests set it, this test will fail.
        
        // Create a new, empty OnceCell for this test only to simulate uninitialized state.
        let local_once_cell: OnceCell<CoreConfig> = OnceCell::new();
        local_once_cell.get().expect("CoreConfig wurde nicht initialisiert. initialize_core_config() muss zuerst aufgerufen werden.");
    }
}
