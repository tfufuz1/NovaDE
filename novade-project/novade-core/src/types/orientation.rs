//! Orientation module for the NovaDE core layer.
//!
//! This module provides orientation and direction types used throughout the
//! NovaDE desktop environment.

use std::fmt;

/// Orientation in 2D space.
///
/// This enum represents horizontal or vertical orientation,
/// which is commonly used for layout and UI components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    /// Horizontal orientation (left to right or right to left)
    Horizontal,
    /// Vertical orientation (top to bottom or bottom to top)
    Vertical,
}

impl Orientation {
    /// Checks if this orientation is horizontal.
    ///
    /// # Returns
    ///
    /// `true` if the orientation is horizontal, `false` otherwise.
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Orientation::Horizontal)
    }

    /// Checks if this orientation is vertical.
    ///
    /// # Returns
    ///
    /// `true` if the orientation is vertical, `false` otherwise.
    pub fn is_vertical(&self) -> bool {
        matches!(self, Orientation::Vertical)
    }

    /// Flips the orientation.
    ///
    /// # Returns
    ///
    /// The opposite orientation (horizontal becomes vertical and vice versa).
    pub fn flip(&self) -> Self {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Orientation::Horizontal => write!(f, "horizontal"),
            Orientation::Vertical => write!(f, "vertical"),
        }
    }
}

/// Direction in 2D space.
///
/// This enum represents cardinal directions (north, south, east, west),
/// which are commonly used for navigation and layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// North direction (up)
    North,
    /// South direction (down)
    South,
    /// East direction (right)
    East,
    /// West direction (left)
    West,
}

impl Direction {
    /// Checks if this direction is horizontal.
    ///
    /// # Returns
    ///
    /// `true` if the direction is east or west, `false` otherwise.
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Direction::East | Direction::West)
    }

    /// Checks if this direction is vertical.
    ///
    /// # Returns
    ///
    /// `true` if the direction is north or south, `false` otherwise.
    pub fn is_vertical(&self) -> bool {
        matches!(self, Direction::North | Direction::South)
    }

    /// Gets the opposite direction.
    ///
    /// # Returns
    ///
    /// The opposite direction (north becomes south, east becomes west, etc.).
    pub fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    /// Gets the orientation of this direction.
    ///
    /// # Returns
    ///
    /// `Orientation::Horizontal` for east and west, `Orientation::Vertical` for north and south.
    pub fn orientation(&self) -> Orientation {
        if self.is_horizontal() {
            Orientation::Horizontal
        } else {
            Orientation::Vertical
        }
    }
}

impl fmt::Display for Direction {
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

    #[test]
    fn test_orientation_is_horizontal() {
        assert!(Orientation::Horizontal.is_horizontal());
        assert!(!Orientation::Vertical.is_horizontal());
    }

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
