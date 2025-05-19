//! Utilities module for the NovaDE core layer.
//!
//! This module provides utility functions and types used throughout the
//! NovaDE desktop environment.

pub mod async_utils;
pub mod file_utils;
pub mod string_utils;

// Re-export key utilities for convenience
pub use async_utils::{spawn_task, timeout};
pub use file_utils::{ensure_directory_exists, read_file_to_string, write_string_to_file};
pub use string_utils::{truncate_string, format_bytes};
