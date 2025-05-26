//! General Utilities for NovaDE Core.
//!
//! This module consolidates various utility submodules that provide common helper
//! functions and tools used across the NovaDE core library.
//!
//! # Submodules
//!
//! - [`fs`]: Filesystem utilities for operations like ensuring directory existence,
//!   reading from and writing to files.
//! - [`paths`]: Utilities for resolving standard XDG directories and application-specific paths.
//!
//! # Re-exports
//!
//! For convenience, some of the most commonly used utilities from the submodules
//! are re-exported here. Utilities not re-exported can be accessed via their
//! respective submodule paths (e.g., `novade_core::utils::paths::get_data_base_dir()`).

pub mod fs;
pub mod paths;

// Re-export key utilities for convenience

// From fs (filesystem utilities)
pub use fs::{ensure_dir_exists, read_to_string};

// From paths (XDG and application paths)
pub use paths::{
    get_config_base_dir, get_data_base_dir, get_cache_base_dir, get_state_base_dir,
    get_app_config_dir, get_app_data_dir, get_app_cache_dir, get_app_state_dir,
};

// Note: async_utils and string_utils modules have been removed as per specification.
