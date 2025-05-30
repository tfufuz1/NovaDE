//! Spatial Orientation and Direction Types.
//!
//! This module provides enums for representing 2D orientation (horizontal/vertical)
//! and cardinal directions (North, South, East, West). These are commonly used
//! in UI layout, navigation, and geometric calculations.
//!
//! # Types
//!
//! - [`Orientation`]: Represents whether something is oriented horizontally or vertically.
//! - [`Direction`]: Represents one of the four cardinal directions.
//!
//! # Examples
//!
//! ```
//! use novade_core::types::{Orientation, Direction};
//!
//! let current_orientation = Orientation::Horizontal;
//! assert!(current_orientation.is_horizontal());
//! assert_eq!(current_orientation.flip(), Orientation::Vertical);
//!
//! let current_direction = Direction::North;
//! assert!(current_direction.is_vertical());
//! assert_eq!(current_direction.opposite(), Direction::South);
//! assert_eq!(current_direction.orientation(), Orientation::Vertical);
//! ```

use std::fmt;
use serde::{Serialize, Deserialize};

/// Represents a general orientation in 2D space, either Horizontal or Vertical.
///
/// This is often used in UI layout systems, component arrangement, or any context
/// where a primary axis of alignment or movement is needed.
///
/// # Examples
///
/// ```
/// use novade_core::types::Orientation;
///
/// let panel_orientation = Orientation::Vertical;
/// if panel_orientation.is_vertical() {
///     // Arrange items in a column
/// }
/// assert_eq!(format!("{}", panel_orientation), "vertical");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Orientation {
    /// Represents a horizontal orientation (e.g., along the x-axis).
    #[default]
    Horizontal,
    /// Represents a vertical orientation (e.g., along the y-axis).
    Vertical,
}

impl Orientation {
    /// Checks if this orientation is `Horizontal`.
    ///
    /// # Returns
    ///
    /// `true` if the orientation is `Horizontal`, `false` otherwise.
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Orientation::Horizontal)
    }

    /// Checks if this orientation is `Vertical`.
    ///
    /// # Returns
    ///
    /// `true` if the orientation is `Vertical`, `false` otherwise.
    pub fn is_vertical(&self) -> bool {
        matches!(self, Orientation::Vertical)
    }

    /// Returns the opposite orientation.
    ///
    /// - `Horizontal` flips to `Vertical`.
    /// - `Vertical` flips to `Horizontal`.
    ///
    /// # Returns
    ///
    /// The flipped `Orientation`.
    ///
    /// # Examples
    /// ```
    /// use novade_core::types::Orientation;
    /// assert_eq!(Orientation::Horizontal.flip(), Orientation::Vertical);
    /// assert_eq!(Orientation::Vertical.flip(), Orientation::Horizontal);
    /// ```
    pub fn flip(&self) -> Self {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}

impl fmt::Display for Orientation {
    /// Formats the `Orientation` as a lowercase string (e.g., "horizontal", "vertical").
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Orientation::Horizontal => write!(f, "horizontal"),
            Orientation::Vertical => write!(f, "vertical"),
        }
    }
}

/// Represents one of the four cardinal directions in 2D space.
///
/// Useful for navigation, relative positioning, or directional input.
///
/// # Examples
/// ```
/// use novade_core::types::{Direction, Orientation};
///
/// let move_direction = Direction::East;
/// if move_direction.is_horizontal() {
///     // Adjust x-coordinate
/// }
/// assert_eq!(move_direction.opposite(), Direction::West);
/// assert_eq!(move_direction.orientation(), Orientation::Horizontal);
/// assert_eq!(format!("{}", move_direction), "east");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Represents the North direction (typically upwards or positive Y in some coordinate systems).
    North,
    /// Represents the South direction (typically downwards or negative Y).
    South,
    /// Represents the East direction (typically rightwards or positive X).
    East,
    /// Represents the West direction (typically leftwards or negative X).
    West,
}

impl Direction {
    /// Checks if this direction is horizontal (`East` or `West`).
    ///
    /// # Returns
    ///
    /// `true` if the direction is `East` or `West`, `false` otherwise.
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Direction::East | Direction::West)
    }

    /// Checks if this direction is vertical (`North` or `South`).
    ///
    /// # Returns
    ///
    /// `true` if the direction is `North` or `South`, `false` otherwise.
    pub fn is_vertical(&self) -> bool {
        matches!(self, Direction::North | Direction::South)
    }

    /// Returns the opposite cardinal direction.
    ///
    /// - `North` becomes `South`.
    /// - `South` becomes `North`.
    /// - `East` becomes `West`.
    /// - `West` becomes `East`.
    ///
    /// # Returns
    ///
    /// The opposite `Direction`.
    ///
    /// # Examples
    /// ```
    /// use novade_core::types::Direction;
    /// assert_eq!(Direction::North.opposite(), Direction::South);
    /// assert_eq!(Direction::East.opposite(), Direction::West);
    /// ```
    pub fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    /// Returns the general [`Orientation`] corresponding to this direction.
    ///
    /// - `North` and `South` map to `Orientation::Vertical`.
    /// - `East` and `West` map to `Orientation::Horizontal`.
    ///
    /// # Returns
    ///
    /// The `Orientation` of this direction.
    pub fn orientation(&self) -> Orientation {
        if self.is_horizontal() {
            Orientation::Horizontal
        } else {
            Orientation::Vertical
        }
    }
}

impl fmt::Display for Direction {
    /// Formats the `Direction` as a lowercase string (e.g., "north", "east").
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::North => write!(f, "north"),
            Direction::South => write!(f, "south"),
            Direction::East => write!(f, "east"),
            Direction::West => write!(f, "west"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json; // For testing serde

    #[test]
    fn test_orientation_is_horizontal() {
        assert!(Orientation::Horizontal.is_horizontal());
        assert!(!Orientation::Vertical.is_horizontal());
    }

    #[test]
    fn test_orientation_default() {
        assert_eq!(Orientation::default(), Orientation::Horizontal);
    }

    #[test]
    fn test_orientation_serde() {
        let horizontal = Orientation::Horizontal;
        let serialized_h = serde_json::to_string(&horizontal).unwrap();
        // Default serde representation for simple enums is just the variant name as a string
        assert_eq!(serialized_h, "\"Horizontal\"");
        let deserialized_h: Orientation = serde_json::from_str(&serialized_h).unwrap();
        assert_eq!(deserialized_h, Orientation::Horizontal);

        let vertical = Orientation::Vertical;
        let serialized_v = serde_json::to_string(&vertical).unwrap();
        assert_eq!(serialized_v, "\"Vertical\"");
        let deserialized_v: Orientation = serde_json::from_str(&serialized_v).unwrap();
        assert_eq!(deserialized_v, Orientation::Vertical);
    }

    // Tests for Direction remain unchanged

    #[test]
    fn test_orientation_is_vertical() {
        assert!(Orientation::Vertical.is_vertical());
        assert!(!Orientation::Horizontal.is_vertical());
    }

    #[test]
    fn test_orientation_flip() {
        assert_eq!(Orientation::Horizontal.flip(), Orientation::Vertical);
        assert_eq!(Orientation::Vertical.flip(), Orientation::Horizontal);
    }

    #[test]
    fn test_orientation_display() {
        assert_eq!(format!("{}", Orientation::Horizontal), "horizontal");
        assert_eq!(format!("{}", Orientation::Vertical), "vertical");
    }

    #[test]
    fn test_direction_is_horizontal() {
        assert!(Direction::East.is_horizontal());
        assert!(Direction::West.is_horizontal());
        assert!(!Direction::North.is_horizontal());
        assert!(!Direction::South.is_horizontal());
    }

    #[test]
    fn test_direction_is_vertical() {
        assert!(Direction::North.is_vertical());
        assert!(Direction::South.is_vertical());
        assert!(!Direction::East.is_vertical());
        assert!(!Direction::West.is_vertical());
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::South.opposite(), Direction::North);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::West.opposite(), Direction::East);
    }

    #[test]
    fn test_direction_orientation() {
        assert_eq!(Direction::North.orientation(), Orientation::Vertical);
        assert_eq!(Direction::South.orientation(), Orientation::Vertical);
        assert_eq!(Direction::East.orientation(), Orientation::Horizontal);
        assert_eq!(Direction::West.orientation(), Orientation::Horizontal);
    }

    #[test]
    fn test_direction_display() {
        assert_eq!(format!("{}", Direction::North), "north");
        assert_eq!(format!("{}", Direction::South), "south");
        assert_eq!(format!("{}", Direction::East), "east");
        assert_eq!(format!("{}", Direction::West), "west");
    }
}
