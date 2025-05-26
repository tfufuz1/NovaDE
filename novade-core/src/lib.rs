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

pub mod error;
pub mod types;
pub mod config;
pub mod logging;
pub mod utils;

// Re-export key types for convenience
pub use error::{CoreError, ConfigError, ColorParseError}; // LoggingError removed, ColorParseError added
pub use types::{
    Point, Size, Rect, RectInt, Color, Orientation, // ColorFormat, Direction removed
    AppIdentifier, Status // ColorParseError removed from here
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
