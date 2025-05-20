//! Types module for the NovaDE core layer.
//!
//! This module provides fundamental data types used throughout the
//! NovaDE desktop environment, including geometric primitives,
//! color handling, and orientation types.

pub mod geometry;
pub mod color;
pub mod orientation;

// Re-export key types for convenience
pub use geometry::{Point, Size, Rect, RectInt};
pub use color::{Color, ColorFormat};
pub use orientation::{Orientation, Direction};
