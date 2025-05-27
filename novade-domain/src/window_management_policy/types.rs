use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::workspaces::core::types::WindowIdentifier; 
use novade_core::types::{RectInt, Size}; 

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TilingMode {
    #[default]
    Manual,
    Columns,
    Rows,
    Spiral,
    MaximizedFocused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NewWindowPlacementStrategy {
    #[default]
    Smart,
    Center,
    Cascade,
    UnderMouse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GapSettings {
    pub screen_outer_horizontal: u16,
    pub screen_outer_vertical: u16,
    pub window_inner: u16,
}

impl Default for GapSettings {
    fn default() -> Self {
        Self {
            screen_outer_horizontal: 5,
            screen_outer_vertical: 5,
            window_inner: 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowSnappingPolicy {
    pub snap_to_screen_edges: bool,
    pub snap_to_other_windows: bool,
    pub snap_to_workspace_gaps: bool, // Simplified for now
    pub snap_distance_px: u16,
}

impl Default for WindowSnappingPolicy {
    fn default() -> Self {
        Self {
            snap_to_screen_edges: true,
            snap_to_other_windows: true,
            snap_to_workspace_gaps: true,
            snap_distance_px: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] // Added Default, Serialize, Deserialize
pub struct WindowPolicyOverrides {
    pub preferred_tiling_mode: Option<TilingMode>,
    pub is_always_floating: Option<bool>,
    pub fixed_size: Option<(u32, u32)>,       // width, height
    pub fixed_position: Option<(i32, i32)>,   // x, y
    pub min_size_override: Option<(u32, u32)>,  // width, height
    pub max_size_override: Option<(u32, u32)>,  // width, height
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FocusStealingPreventionLevel {
    None,
    Moderate, // Default
    Strict,
}

impl Default for FocusStealingPreventionLevel {
    fn default() -> Self {
        FocusStealingPreventionLevel::Moderate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] // Added Eq
pub struct FocusPolicy {
    pub focus_follows_mouse: bool,
    pub click_to_focus: bool,
    pub focus_new_windows_on_creation: bool,
    pub focus_new_windows_on_workspace_switch: bool,
    pub focus_stealing_prevention: FocusStealingPreventionLevel,
}

impl Default for FocusPolicy {
    fn default() -> Self {
        Self {
            focus_follows_mouse: false,
            click_to_focus: true,
            focus_new_windows_on_creation: true,
            focus_new_windows_on_workspace_switch: true,
            focus_stealing_prevention: FocusStealingPreventionLevel::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] // Added Eq
pub struct WindowGroupingPolicy {
    pub enable_manual_grouping: bool,
    // Future fields: auto_group_by_application, max_group_size, etc.
}

impl Default for WindowGroupingPolicy {
    fn default() -> Self {
        Self {
            enable_manual_grouping: true,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct WindowLayoutInfo {
    pub id: WindowIdentifier,
    pub requested_min_size: Option<Size<u32>>,
    pub is_fullscreen_requested: bool,
    pub is_maximized_requested: bool,
    // These fields are determined by a higher layer combining global workspace policy
    // with window-specific rules/overrides before calling calculate_workspace_layout.
    pub effective_tiling_mode_override: Option<TilingMode>,
    pub is_floating_override: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkspaceWindowLayout {
    pub window_geometries: HashMap<WindowIdentifier, RectInt>,
    pub occupied_area: Option<RectInt>, 
    pub tiling_mode_applied: TilingMode,
}


#[cfg(test)]
mod novade_core_placeholders {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub struct RectInt {
        pub x: i32,
        pub y: i32,
        pub width: u32,
        pub height: u32,
    }
    impl RectInt {
        pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self { Self {x,y,width,height}}
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub struct Size<T> {
        pub width: T,
        pub height: T,
    }
    impl<T> Size<T> {
        pub fn new(width: T, height: T) -> Self { Self {width, height}}
    }
}
