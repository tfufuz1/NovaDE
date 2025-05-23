//! Core Data Types for NovaDE.
//!
//! This module aggregates various fundamental data types utilized across the NovaDE core library
//! and potentially by other NovaDE components. It serves as a central point for accessing
//! common structures related to geometry, color representation, application identification,
//! status indication, and directional orientations.
//!
//! ## Submodules
//!
//! - [`geometry`]: Defines geometric primitives like points (`Point`), sizes (`Size`),
//!   and rectangles (`Rect`, `RectInt`).
//! - [`color`]: Provides color representations (`Color`), color formats (`ColorFormat`),
//!   and associated error types for parsing (`ColorParseError`).
//! - [`orientation`]: Includes types for representing screen orientation (`Orientation`)
//!   and general direction (`Direction`).
//! - [`application`]: Contains application-specific types such as unique identifiers
//!   (`AppIdentifier`) and status enums (`Status`).
//!
//! Key types from these submodules are re-exported here for convenient access.

pub mod geometry;
pub mod color;
pub mod orientation;
pub mod application; // Added new module

// Re-export key types for convenience
pub use geometry::{Point, Size, Rect, RectInt};
pub use color::{Color, ColorFormat, ColorParseError}; // Added ColorParseError based on previous tasks
pub use orientation::{Orientation, Direction};
pub use application::{AppIdentifier, Status}; // Added new types
