//! General Utilities for NovaDE Core.
//!
//! This module consolidates various utility submodules that provide common helper
//! functions and tools used across the NovaDE core library.
//!
//! # Submodules
//!
//! - [`async_utils`]: Utilities for asynchronous programming, such as task spawning
//!   and timeouts.
//! - [`fs`]: Filesystem utilities for operations like ensuring directory existence,
//!   reading from and writing to files. (Formerly `file_utils`).
//! - [`string_utils`]: Helper functions for string manipulation, such as truncation,
//!   byte formatting, and case conversions.
//! - [`paths`]: Utilities for resolving standard XDG directories and application-specific paths.
//!
//! # Re-exports
//!
//! For convenience, some of the most commonly used utilities from the submodules
//! are re-exported here. Utilities not re-exported can be accessed via their
//! respective submodule paths (e.g., `novade_core::utils::paths::get_config_base_dir()`).

pub mod async_utils;
pub mod fs; // Renamed from file_utils
pub mod string_utils;
pub mod paths;

// Re-export key utilities for convenience

// From async_utils
pub use async_utils::{spawn_task, timeout};

// From fs (filesystem utilities)
pub use fs::{ensure_directory_exists, read_file_to_string, write_string_to_file};

// From string_utils
pub use string_utils::{truncate_string, format_bytes};

// Note: Functions from the `paths` module (e.g., `get_config_base_dir`) and
// string case conversion functions (e.g., `to_snake_case`) are not re-exported
// here by default and should be accessed via their specific submodule, e.g.,
// `crate::utils::paths::get_app_config_dir()` or `crate::utils::string_utils::to_snake_case()`.
