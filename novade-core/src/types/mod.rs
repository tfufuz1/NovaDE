//! Core data types used throughout NovaDE.
//!
//! This module consolidates common data structures like geometric primitives
//! (`Point`, `Size`, `Rect`), color representations (`Color`), and various
//! utility enums and identifiers.

// Declare submodules
pub mod app_identifier;
pub mod color;
pub mod enums;
pub mod geometry;
pub mod status;

// Re-export public types for easier access
pub use app_identifier::AppIdentifier;
pub use color::Color; // ColorParseError is now in crate::error
pub use crate::error::ColorParseError; // Re-export ColorParseError from crate::error
pub use enums::Orientation;
pub use geometry::{Point, Rect, RectInt, Size};
pub use status::Status;

// Note on RectInt:
// The re-export `pub use geometry::{Point, Size, Rect, RectInt};` assumes `RectInt` is a public type in `geometry.rs`.
// `PointInt` and `SizeInt` are helper types for `RectInt` and are public within `geometry.rs` for `RectInt` to use,
// but they are not re-exported at this `types` module level.
//
// Note on Serde:
// Serde support for types like Point<T>, Size<T>, Rect<T> is conditional on the `serde` feature flag for the crate.
// This module's re-exports make the types available; actual serde support is handled by derive attributes
// within the respective struct/enum definitions, typically like:
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// The problem description implies serde is generally expected, so the derives in the files will reflect that.
// The `serde(bound(...))` attribute is used for generic types.
