//! Library module for the novade-core crate.
//!
//! This is the root module for the novade-core crate, which provides
//! fundamental building blocks and utilities used throughout the
//! NovaDE desktop environment.

pub mod error;
pub mod types;
pub mod config;
pub mod logging;
pub mod utils;

// Re-export key types for convenience
pub use error::{CoreError, ConfigError, LoggingError};
pub use types::{Point, Size, Rect, RectInt, Color, ColorFormat, Orientation, Direction};
pub use config::{CoreConfig, LoggingConfig, ApplicationConfig, SystemConfig, ConfigLoader, ConfigProvider};
pub use logging::{initialize_logging, is_initialized};
pub use utils::{
    spawn_task, timeout, sleep, interval,
    ensure_directory_exists, read_file_to_string, write_string_to_file, copy_file, get_all_files,
    truncate_string, format_bytes, to_snake_case, to_camel_case, to_pascal_case, to_kebab_case
};
