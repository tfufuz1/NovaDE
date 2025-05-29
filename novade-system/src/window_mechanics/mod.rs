// novade-system/src/window_mechanics/mod.rs

//! # Window Mechanics Module
//!
//! This module handles interactive window operations such as move and resize,
//! typically initiated by user input and managed through pointer grabs.
//! It defines the state and handlers for these operations.

pub mod interactive_ops;
// pub mod types; // If MoveResizeState or other types grow complex enough
// pub mod errors; // If specific errors for window mechanics are needed

// Re-export key types for easier access from other modules (e.g., input handling)
pub use interactive_ops::{MoveResizeState, InteractiveOpType, NovaMoveGrab, NovaResizeGrab};
