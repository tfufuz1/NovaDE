//! Common enumerations used in NovaDE core types.

use serde::{Deserialize, Serialize};

/// Represents the orientation of UI elements or layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Orientation {
    /// Horizontal orientation (e.g., items arranged side-by-side).
    Horizontal,
    /// Vertical orientation (e.g., items arranged one below the other).
    Vertical,
}

impl Default for Orientation {
    /// Returns `Orientation::Horizontal` by default.
    fn default() -> Self {
        Orientation::Horizontal
    }
}

impl Orientation {
    /// Toggles the orientation.
    ///
    /// `Horizontal` becomes `Vertical`, and `Vertical` becomes `Horizontal`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use novade_core::types::enums::Orientation;
    /// let mut orientation = Orientation::Horizontal;
    /// orientation = orientation.toggle();
    /// assert_eq!(orientation, Orientation::Vertical);
    /// orientation = orientation.toggle();
    /// assert_eq!(orientation, Orientation::Horizontal);
    /// ```
    pub fn toggle(&self) -> Self {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;
    use std::fmt; // Required for fmt::Debug
    use std::hash::Hash; // Required for std::hash::Hash

    assert_impl_all!(Orientation: std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Default, Serialize, Deserialize<'static>, Send, Sync);

    #[test]
    fn orientation_default_is_horizontal() {
        assert_eq!(Orientation::default(), Orientation::Horizontal);
    }

    #[test]
    fn orientation_toggle_works() {
        assert_eq!(Orientation::Horizontal.toggle(), Orientation::Vertical);
        assert_eq!(Orientation::Vertical.toggle(), Orientation::Horizontal);
    }

    #[test]
    fn orientation_serde() {
        let horizontal = Orientation::Horizontal;
        let serialized_h = serde_json::to_string(&horizontal).unwrap();
        assert_eq!(serialized_h, "\"Horizontal\"");
        let deserialized_h: Orientation = serde_json::from_str(&serialized_h).unwrap();
        assert_eq!(deserialized_h, Orientation::Horizontal);

        let vertical = Orientation::Vertical;
        let serialized_v = serde_json::to_string(&vertical).unwrap();
        assert_eq!(serialized_v, "\"Vertical\"");
        let deserialized_v: Orientation = serde_json::from_str(&serialized_v).unwrap();
        assert_eq!(deserialized_v, Orientation::Vertical);
    }
}
