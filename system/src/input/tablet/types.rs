// system/src/input/tablet/types.rs
use serde::{Serialize, Deserialize};
use smithay::reexports::input::event::tablet_tool::{TabletToolType, TabletToolButtonState};
use smithay::utils::{Logical, Point};

// Example: Internal representation of a tablet tool's state if needed beyond Smithay's types.
// For now, we might rely mostly on Smithay's own types for tablet events and state.
// This file can be expanded as specific needs for internal state arise.

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StylusTipState {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabletToolInfo {
    pub tool_type: TabletToolType, // Pen, Eraser, Brush, etc.
    pub hardware_serial: u64, // From libinput event
    // Other relevant info like capabilities (pressure, tilt, distance) can be queried from smithay::input::TabletToolHandle
}

// Placeholder for now, can be expanded.
