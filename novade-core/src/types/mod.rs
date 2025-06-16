//! Core data types used throughout NovaDE.
//!
//! This module consolidates common data structures used across the NovaDE system.
//! It re-exports types from its submodules for easier access. Key categories include:
//!
//! - **Application Identification**: [`AppIdentifier`] for uniquely naming applications.
//! - **Assistant**: Types like [`AssistantCommand`], [`ContextInfo`] for the smart assistant.
//! - **Color**: The [`Color`] struct and related utilities.
//! - **Display**: Structs and enums for display management (e.g., [`Display`], [`DisplayMode`]).
//! - **Events**: Core system events like [`CoreEvent`] and [`NotificationUrgency`].
//! - **General**: Universal types such as [`Uuid`] and [`Timestamp`].
//! - **Geometry**: Primitives like [`Point`], [`Size`], [`Rectangle`], [`Vector`], and their integer counterparts.
//! - **Orientation**: [`Orientation`] and [`Direction`] enums.
//! - **Status**: A generic [`Status`] enum.
//! - **System Health**: Metrics and configurations for system monitoring (e.g., [`CpuMetrics`], [`SystemHealthDashboardConfig`]).
//!
//! Many types are designed to be serializable and deserializable using Serde.

// Declare submodules
pub mod app_identifier;
pub mod assistant;
pub mod color;
// pub mod enums; // enums.rs is now empty after Orientation was moved
pub mod display;
pub mod geometry;
pub mod status;
pub mod orientation; // Declare the orientation module
pub mod system_health;
pub mod events;
pub mod general;

// Re-export public types for easier access
pub use app_identifier::AppIdentifier;
pub use self::assistant::{AssistantCommand, ContextInfo, UserIntent, SkillDefinition, AssistantPreferences};
pub use color::Color; // ColorParseError is now in crate::error
pub use display::*;
pub use crate::error::ColorParseError; // Re-export ColorParseError from crate::error
pub use orientation::{Orientation, Direction}; // Added Direction
// pub use geometry::{Point, Rect, RectInt, Size}; // Old line removed

// Re-export f64-based geometry types as the primary API for this crate level
/// A 2D point with `f64` coordinates. Alias for [`geometry::Point<f64>`].
pub type Point = self::geometry::Point<f64>;
/// A 2D size (width and height) with `f64` dimensions. Alias for [`geometry::Size<f64>`].
pub type Size = self::geometry::Size<f64>;
/// A 2D rectangle with `f64` coordinates and dimensions. Alias for [`geometry::Rect<f64>`].
pub type Rectangle = self::geometry::Rect<f64>;
/// A 2D vector with `f64` components, typically used for displacement or direction.
/// Re-exported from [`geometry::Vector`].
pub use self::geometry::Vector;

// Re-export integer-based geometry types
pub use self::geometry::{PointInt, RectInt, SizeInt};

pub use status::Status;
pub use system_health::*;
pub use events::{CoreEvent, NotificationUrgency};
pub use general::{Timestamp, Uuid};

// Note on RectInt:
// The re-export `pub use geometry::{Point, Size, Rect, RectInt};` assumes `RectInt` is a public type in `geometry.rs`.
// `PointInt` and `SizeInt` are helper types for `RectInt` and are public within `geometry.rs` for `RectInt` to use,
// but they are not re-exported at this `types` module level. (Note: This comment is now slightly outdated as they are re-exported above)
//
// Note on Serde:
// Serde support for types like Point<T>, Size<T>, Rect<T> is conditional on the `serde` feature flag for the crate.
// This module's re-exports make the types available; actual serde support is handled by derive attributes
// within the respective struct/enum definitions, typically like:
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// The problem description implies serde is generally expected, so the derives in the files will reflect that.
// The `serde(bound(...))` attribute is used for generic types.
