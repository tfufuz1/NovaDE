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
//!   associated specific error types like `ConfigError` and `LoggingError`.
//! - **Core Data Types**: Fundamental data structures for geometry (`Point`, `Size`, `Rect`, `RectInt`),
//!   color representation (`Color`, `ColorFormat`), application identification (`AppIdentifier`),
//!   status indicators (`Status`), and more.
//! - **Configuration Management**: Utilities for loading, parsing, and validating
//!   application configuration, primarily through the `ConfigLoader` and `CoreConfig` structs.
//! - **Logging**: A flexible logging framework built on top of the `tracing` crate,
//!   configurable for various outputs (console, file) and formats (text, JSON).
//! - **Utility Functions**: A collection of helper functions for filesystem operations (`utils::fs`),
//!   path resolution (`utils::paths`), asynchronous tasks (`utils::async_utils`), and
//!   string manipulation (`utils::string_utils`).
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
//! use novade_core::config::{ConfigLoader, CoreConfig};
//! use novade_core::logging::initialize_logging;
//! use novade_core::error::CoreError;
//!
//! fn main() -> Result<(), CoreError> {
//!     // Load configuration first
//!     let core_config = ConfigLoader::load()?;
//!
//!     // Initialize logging based on the loaded configuration
//!     initialize_logging(&core_config.logging, false)?;
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
pub use error::{CoreError, ConfigError, LoggingError};
pub use types::{
    Point, Size, Rect, RectInt, Color, ColorFormat, Orientation, Direction, 
    color::ColorParseError, AppIdentifier, Status
};
pub use config::{CoreConfig, LoggingConfig, ConfigLoader, ConfigProvider};
pub use logging::{initialize_logging, init_minimal_logging}; // Removed is_initialized
pub use utils::{
    spawn_task, timeout, sleep, interval, // From async_utils
    ensure_directory_exists, read_file_to_string, write_string_to_file, copy_file, get_all_files, // From fs (formerly file_utils)
    // Note: Path functions from utils::paths are not re-exported by default here.
    // Note: String case conversions from utils::string_utils are not re-exported by default here.
    truncate_string, format_bytes, // From string_utils
    // For case conversions, use string_utils::to_snake_case etc.
    // For path utils, use utils::paths::get_config_base_dir etc.
};
