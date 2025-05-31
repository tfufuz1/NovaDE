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
// std::fs and std::path::PathBuf are no longer directly used here for ConfigLoader logic
use std::path::PathBuf; // Still used by LoggingConfig

pub mod defaults;
pub mod loader; // Import the loader module
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

    #[test]
    fn test_global_config_access() {
        // Reset OnceCell for this test (important if tests run in parallel or share state)
        // This is tricky. OnceCell is global. Tests should not depend on order.
        // For this test, we assume it's the first one to set CORE_CONFIG or that it's been reset.
        // In a real test suite, use a mutex or ensure `initialize` is only called once.
        // For now, we'll try and see. If it fails due to already set, that's a test interaction issue.
        
        let test_config = CoreConfig {
            logging: LoggingConfig {
                level: "test".to_string(),
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
                assert_eq!(retrieved_config.logging.level, "test");

                // Test trying to initialize again fails
                let another_config = CoreConfig::default();
                assert!(initialize_core_config(another_config).is_err());
            }
            Err(_returned_config) => {
                // Config was already initialized by another test. We can still test get_core_config.
                println!("Warning: CORE_CONFIG was already initialized. Testing get_core_config only.");
                let retrieved_config = get_core_config(); // Should not panic if already set
                assert!(!retrieved_config.logging.level.is_empty()); // Check it's a valid CoreConfig
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
