// ANCHOR: OutputConfigDefinition
//! Defines structures for output configuration.

use smithay::utils::{Size, Point, Logical};

/// Configuration for a display output.
/// Used for initializing outputs in the compositor.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub name: String,
    pub resolution: Size<i32, Logical>, // e.g., (1920, 1080)
    pub position: Point<i32, Logical>,   // Position in the global compositor space, e.g., (0,0) or (1920, 0)
    pub scale: i32,                      // e.g., 1 or 2
    pub is_primary: bool,
    // Add other relevant properties like refresh rate, transform, etc. if needed later.
    // For example: pub transform: smithay::utils::Transform,
    // pub refresh_rate: u32, // in mHz
}
// ANCHOR_END: OutputConfigDefinition
