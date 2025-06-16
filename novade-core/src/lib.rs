//! # NovaDE Core Library (`novade-core`)
//!
//! `novade-core` is the foundational library for the NovaDE (Nova Desktop Environment) project.
//! It provides a comprehensive set of core functionalities, data types, and utilities
//! essential for building desktop components and applications within the NovaDE ecosystem.
//!
//! ## Purpose
//!
//! The primary purpose of this crate is to offer a stable, well-tested, and ergonomic
//! toolkit for common desktop environment tasks. This includes:
//!
//! - **Error Handling**: A unified error system through the `CoreError` enum and its
//!   associated specific error types like `ConfigError` and `ColorParseError`.
//! - **Core Data Types**: Fundamental data structures for geometry (`Point`, `Size`, `Rect`, `RectInt`),
//!   color representation (`Color`), application identification (`AppIdentifier`),
//!   status indicators (`Status`), `Orientation`, and more.
//! - **Configuration Management**: Utilities for loading, parsing, and validating
//!   application configuration, primarily through the `ConfigLoader` and `CoreConfig` structs,
//!   including global access to the configuration.
//! - **Logging**: A flexible logging framework built on top of the `tracing` crate,
//!   configurable for various outputs (console, file) and formats (text, JSON).
//! - **Utility Functions**: A collection of helper functions for filesystem operations (`utils::fs`)
//!   and XDG path resolution (`utils::paths`).
//!
//! ## Key Features
//!
//! - **Strong Typing**: Emphasis on clear and specific types to enhance code safety and clarity.
//! - **Comprehensive Error Handling**: Robust error types to manage various failure scenarios.
//! - **Configuration System**: TOML-based configuration loading with default fallbacks and validation.
//! - **Structured Logging**: Flexible and configurable logging suitable for debugging and monitoring.
//! - **Utility Belt**: Common utilities for everyday programming tasks relevant to desktop applications.
//!
//! ## Usage
//!
//! Add `novade-core` as a dependency in your `Cargo.toml`. Key components are re-exported
//! at the crate root for ease of use.
//!
//! ```rust,ignore
//! // Example: Initializing logging and loading configuration
//! use novade_core::config::{ConfigLoader, CoreConfig, initialize_core_config, get_core_config};
//! use novade_core::logging::init_logging; // Renamed
//! use novade_core::error::CoreError;
//!
//! fn main() -> Result<(), CoreError> {
//!     // Load configuration first
//!     let core_config = ConfigLoader::load().or_else(|e| {
//!         // Handle error, e.g. if config file not found, use defaults
//!         if matches!(e, CoreError::Config(config::ConfigError::NotFound { .. })) {
//!             eprintln!("Config file not found, using default settings. Error: {}", e);
//!             Ok(CoreConfig::default()) // Proceed with default config
//!         } else {
//!             Err(e) // Propagate other errors
//!         }
//!     })?;
//!     
//!     // Initialize global config
//!     initialize_core_config(core_config.clone())
//!         .map_err(|_| CoreError::Internal("Config already initialized".to_string()))?;
//!
//!     // Initialize logging based on the (potentially default) configuration
//!     init_logging(&get_core_config().logging, false)?;
//!
//!     tracing::info!("NovaDE Core initialized successfully.");
//!     // ... your application logic ...
//!     Ok(())
//! }
//! ```
//!
//! This crate aims to be the bedrock of NovaDE, providing reliable and consistent
//! core services.

/// Error handling types for the NovaDE core.
pub mod error;
/// Core data types used throughout NovaDE.
pub mod types;
/// Configuration management for NovaDE applications.
pub mod config;
/// Logging infrastructure for NovaDE.
pub mod logging;
/// Utility functions for common tasks.
pub mod utils;

use tracing_subscriber::EnvFilter;

/// Initializes core components of the NovaDE system, with a primary focus on the logging system.
///
/// This function should be called once at the beginning of an application's lifecycle
/// to set up the global tracing subscriber. The logging behavior is primarily configured
/// through the `RUST_LOG` environment variable (e.g., `RUST_LOG=info,novade_core=debug`).
///
/// It uses `tracing_subscriber::fmt` for formatted output and `EnvFilter` for
/// filtering log directives from the environment.
///
/// # Errors
///
/// Returns a [`CoreError::Logging`] if the initialization of the global tracing
/// subscriber fails. This commonly occurs if a global default subscriber has already
/// been set elsewhere in the application or by another library.
pub fn init() -> Result<(), crate::error::CoreError> {
    match tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init() // Use try_init() to handle errors
    {
        Ok(()) => {
            tracing::info!("NovaDE Core components initialized successfully via novade_core::init().");
            Ok(())
        }
        Err(e) => {
            // Convert the tracing error to CoreError::Logging(String)
            Err(crate::error::CoreError::Logging(format!(
                "Failed to initialize global tracing subscriber: {}",
                e
            )))
        }
    }
}

// Re-export key types for convenience
pub use error::{CoreError, ConfigError, ColorParseError}; // LoggingError removed, ColorParseError added
pub use types::{
    // --- app_identifier.rs ---
    AppIdentifier,
    // --- assistant.rs ---
    AssistantCommand, ContextInfo, UserIntent, SkillDefinition, AssistantPreferences,
    // --- color.rs ---
    Color,
    // --- display.rs ---
    DisplayConnector, DisplayMode, PhysicalProperties, DisplayStatus, DisplayLayout, Display, DisplayConfiguration,
    // --- events.rs ---
    CoreEvent, NotificationUrgency,
    // --- general.rs ---
    Timestamp, Uuid,
    // --- geometry.rs ---
    // f64 based
    Point, Size, Rectangle, Vector,
    // integer based
    PointInt, SizeInt, RectInt,
    // --- orientation.rs ---
    Orientation, Direction,
    // --- status.rs ---
    Status,
    // --- system_health.rs ---
    CpuMetrics, MemoryMetrics, DiskActivityMetrics, DiskSpaceMetrics, NetworkActivityMetrics, TemperatureMetric,
    LogPriority, LogEntry, LogSourceIdentifier, LogFilter, TimeRange,
    DiagnosticTestId, DiagnosticTestInfo, DiagnosticStatus, DiagnosticTestResult,
    AlertSeverity, AlertId, Alert, SystemHealthDashboardConfig,
};
pub use config::{
    CoreConfig, LoggingConfig, FeatureFlags, // Added FeatureFlags
    ConfigLoader, 
    initialize_core_config, get_core_config // Added global access functions
};
pub use logging::{init_logging, init_minimal_logging}; // Renamed initialize_logging
pub use utils::{
    // fs utilities
    ensure_dir_exists,
    read_to_string,
    // paths utilities
    get_config_base_dir,
    get_data_base_dir,
    get_cache_base_dir,
    get_state_base_dir,
    get_app_config_dir,
    get_app_data_dir,
    get_app_cache_dir,
    get_app_state_dir,
};

#[cfg(test)]
mod init_tests {
    use super::*;
    use std::sync::Once;
    use crate::error::CoreError; // For matching error type

    // Helper to ensure global logger state is managed across tests in this module.
    static TRACING_INIT: Once = Once::new();

    #[test]
    fn test_init_success() {
        // This test relies on the fact that `try_init` handles cases where a logger
        // might already be set by another test (especially when tests run concurrently
        // or out of specific order). If no logger is set, it initializes.
        // If one is set, try_init() itself will error, but our `init()` wraps this.
        // The critical part for *this function's own logic* is that it correctly
        // translates the Ok or Err from try_init.

        let result = crate::init(); // Call the function from the crate root

        // We accept Ok(()) or an error indicating it was already initialized.
        match result {
            Ok(()) => {
                tracing::info!("test_init_success: init() succeeded or was already initialized and handled gracefully by prior init.");
                // If it succeeded, subsequent calls in other tests will hit the "already initialized" path.
            }
            Err(CoreError::Logging(msg)) => {
                // This is also an acceptable outcome if another test ran `init()` first.
                assert!(msg.contains("Failed to initialize global tracing subscriber"), "Error message should indicate tracer init failure. Got: {}", msg);
                tracing::warn!("test_init_success: init() failed as expected (already initialized): {}", msg);
            }
            Err(e) => {
                panic!("init() failed with an unexpected error type: {:?}", e);
            }
        }
        // The main assertion is that it doesn't panic and returns a Result.
    }

    #[test]
    fn test_init_error_on_reinitialization() {
        // Ensure init() is called at least once successfully.
        TRACING_INIT.call_once(|| {
            match crate::init() { // Call the function from the crate root
                Ok(()) => tracing::info!("First init successful for reinitialization test."),
                Err(e) => {
                    // This could happen if another test ran `init` before this `Once` block.
                    // It's acceptable for this test's logic as long as a logger is set.
                    tracing::warn!("Initial init in TRACING_INIT failed (likely already set by another test): {}. Test will proceed.", e);
                }
            }
        });

        // At this point, a global subscriber should have been set (either by the Once block or a preceding test).
        // Try to initialize again. This specific call within this test should now robustly fail.
        let result = crate::init(); // Call the function from the crate root
        assert!(result.is_err(), "Second call to init() should return an error.");

        match result {
            Err(CoreError::Logging(msg)) => {
                assert!(msg.contains("Failed to initialize global tracing subscriber"), "Error message prefix mismatch. Got: {}", msg);
                // The exact error message from `tracing::subscriber::set_global_default` can vary slightly
                // or might be wrapped. The key is that it's a logging-related error from `try_init`.
                // Common phrases are "already been set" or "failed to set global default subscriber".
                assert!(msg.contains("global default subscriber has already been set") || msg.contains("another global logger was already installed"), "Error message should indicate already set. Got: {}", msg);
            }
            _ => panic!("Expected CoreError::Logging, but got {:?}", result),
        }
    }
}
