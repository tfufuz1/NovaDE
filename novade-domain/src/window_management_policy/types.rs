use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use novade_core::types::{RectInt, Size};
use crate::workspaces::core::WindowIdentifier; // Corrected path based on typical structure
use novade_core::types::geometry::{Rect, Point}; // Added for WindowPlacementInfo
use uuid::Uuid; // Added for WindowPlacementInfo

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TilingMode {
    #[default]
    Manual,
    Columns,
    Rows,
    Spiral,
    MaximizedFocused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct GapSettings {
    #[serde(default)]
    pub screen_outer_horizontal: u16,
    #[serde(default)]
    pub screen_outer_vertical: u16,
    #[serde(default)]
    pub window_inner: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct WindowSnappingPolicy {
    #[serde(default)]
    pub snap_to_screen_edges: bool,
    #[serde(default)]
    pub snap_to_other_windows: bool,
    #[serde(default)]
    pub snap_to_workspace_gaps: bool,
    #[serde(default)]
    pub snap_distance_px: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct WindowGroupingPolicy {
    #[serde(default)]
    pub enable_manual_grouping: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NewWindowPlacementStrategy {
    #[default]
    Smart,
    Center,
    Cascade,
    UnderMouse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FocusStealingPreventionLevel {
    None,
    #[default]
    Moderate,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct FocusPolicy {
    #[serde(default)]
    pub focus_follows_mouse: bool,
    #[serde(default)]
    pub click_to_focus: bool,
    #[serde(default)]
    pub focus_new_windows_on_creation: bool,
    #[serde(default)]
    pub focus_new_windows_on_workspace_switch: bool,
    #[serde(default)]
    pub focus_stealing_prevention: FocusStealingPreventionLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct WindowPolicyOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_tiling_mode: Option<TilingMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_always_floating: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_size: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_position: Option<(i32, i32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prevent_focus_stealing: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_size_override: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_size_override: Option<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct WorkspaceWindowLayout {
    #[serde(default)] // Ensure it defaults to empty map if missing in JSON
    pub window_geometries: HashMap<WindowIdentifier, RectInt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occupied_area: Option<RectInt>,
    #[serde(default)]
    pub tiling_mode_applied: TilingMode,
}

#[derive(Debug, Clone, PartialEq)] // Not serialized/deserialized as it's runtime.
pub struct WindowLayoutInfo {
    pub id: WindowIdentifier,
    pub requested_min_size: Option<Size<u32>>,
    pub requested_base_size: Option<Size<u32>>,
    pub is_fullscreen_requested: bool,
    pub is_maximized_requested: bool,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiling_mode_default_and_serde() {
        assert_eq!(TilingMode::default(), TilingMode::Manual);
        let mode = TilingMode::Spiral;
        let serialized = serde_json::to_string(&mode).unwrap();
        assert_eq!(serialized, "\"spiral\"");
        let deserialized: TilingMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, mode);
    }

    #[test]
    fn gap_settings_default_and_serde() {
        let default_gs = GapSettings::default();
        assert_eq!(default_gs.screen_outer_horizontal, 0);
        assert_eq!(default_gs.screen_outer_vertical, 0);
        assert_eq!(default_gs.window_inner, 0);
        
        let gs = GapSettings { screen_outer_horizontal: 10, screen_outer_vertical: 5, window_inner: 2 };
        let serialized = serde_json::to_string(&gs).unwrap();
        let expected_json = r#"{"screen_outer_horizontal":10,"screen_outer_vertical":5,"window_inner":2}"#;
        assert_eq!(serialized, expected_json);
        let deserialized: GapSettings = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, gs);
    }

    #[test]
    fn window_snapping_policy_default_and_serde() {
        let default_wsp = WindowSnappingPolicy::default();
        assert_eq!(default_wsp.snap_to_screen_edges, false);
        assert_eq!(default_wsp.snap_distance_px, 0);

        let wsp = WindowSnappingPolicy {
            snap_to_screen_edges: true,
            snap_to_other_windows: true,
            snap_to_workspace_gaps: false,
            snap_distance_px: 10,
        };
        let serialized = serde_json::to_string(&wsp).unwrap();
        let deserialized: WindowSnappingPolicy = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, wsp);
    }

    #[test]
    fn window_grouping_policy_default_and_serde() {
        let default_wgp = WindowGroupingPolicy::default();
        assert_eq!(default_wgp.enable_manual_grouping, false);

        let wgp = WindowGroupingPolicy { enable_manual_grouping: true };
        let serialized = serde_json::to_string(&wgp).unwrap();
        let deserialized: WindowGroupingPolicy = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, wgp);
    }

    #[test]
    fn new_window_placement_strategy_default_and_serde() {
        assert_eq!(NewWindowPlacementStrategy::default(), NewWindowPlacementStrategy::Smart);
        let strategy = NewWindowPlacementStrategy::Center;
        let serialized = serde_json::to_string(&strategy).unwrap();
        assert_eq!(serialized, "\"center\"");
        let deserialized: NewWindowPlacementStrategy = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, strategy);
    }

    #[test]
    fn focus_stealing_prevention_level_default_and_serde() {
        assert_eq!(FocusStealingPreventionLevel::default(), FocusStealingPreventionLevel::Moderate);
        let level = FocusStealingPreventionLevel::Strict;
        let serialized = serde_json::to_string(&level).unwrap();
        assert_eq!(serialized, "\"strict\"");
        let deserialized: FocusStealingPreventionLevel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, level);
    }

    #[test]
    fn focus_policy_default_and_serde() {
        let default_fp = FocusPolicy::default();
        assert_eq!(default_fp.focus_follows_mouse, false);
        assert_eq!(default_fp.focus_stealing_prevention, FocusStealingPreventionLevel::Moderate);

        let fp = FocusPolicy {
            focus_follows_mouse: true,
            click_to_focus: true, 
            focus_new_windows_on_creation: true,
            focus_new_windows_on_workspace_switch: true,
            focus_stealing_prevention: FocusStealingPreventionLevel::Strict,
        };
        let serialized = serde_json::to_string(&fp).unwrap();
        let deserialized: FocusPolicy = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, fp);
    }

    #[test]
    fn window_policy_overrides_default_and_serde() {
        let default_wpo = WindowPolicyOverrides::default();
        assert!(default_wpo.preferred_tiling_mode.is_none());
        assert!(default_wpo.fixed_size.is_none());
        
        let serialized_default = serde_json::to_string(&default_wpo).unwrap();
        assert_eq!(serialized_default, "{}"); // All fields are Option and skip_serializing_if none + default
        let deserialized_default: WindowPolicyOverrides = serde_json::from_str(&serialized_default).unwrap();
        assert_eq!(deserialized_default, default_wpo);

        let wpo = WindowPolicyOverrides {
            preferred_tiling_mode: Some(TilingMode::Columns),
            is_always_floating: Some(true),
            fixed_size: Some((800, 600)),
            fixed_position: Some((100, 100)),
            prevent_focus_stealing: Some(true),
            min_size_override: Some((50,50)),
            max_size_override: Some((1000,1000)),
        };
        let serialized = serde_json::to_string_pretty(&wpo).unwrap();
        let deserialized: WindowPolicyOverrides = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, wpo);
    }

    #[test]
    fn workspace_window_layout_default_and_serde() {
        let default_wwl = WorkspaceWindowLayout::default();
        assert!(default_wwl.window_geometries.is_empty());
        assert_eq!(default_wwl.tiling_mode_applied, TilingMode::Manual);

        let mut geometries = HashMap::new();
        let win_id1 = WindowIdentifier::from("win1");
        geometries.insert(win_id1.clone(), RectInt::new(0,0,100,100));
        let wwl = WorkspaceWindowLayout {
            window_geometries: geometries.clone(),
            occupied_area: Some(RectInt::new(0,0,800,600)),
            tiling_mode_applied: TilingMode::Columns,
        };
        let serialized = serde_json::to_string_pretty(&wwl).unwrap();
        let deserialized: WorkspaceWindowLayout = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.tiling_mode_applied, TilingMode::Columns);
        assert_eq!(deserialized.window_geometries.get(&win_id1).unwrap(), &RectInt::new(0,0,100,100));
        assert_eq!(deserialized, wwl); // Full equality check
    }
    
    #[test]
    fn window_layout_info_creation() {
        let win_id = WindowIdentifier::from("test-win");
        let info = WindowLayoutInfo {
            id: win_id.clone(),
            requested_min_size: Some(Size::new(100, 100)),
            requested_base_size: None,
            is_fullscreen_requested: false,
            is_maximized_requested: true,
        };
        assert_eq!(info.id, win_id);
        assert_eq!(info.is_maximized_requested, true);
        assert_eq!(info.requested_min_size, Some(Size::new(100,100)));
    }

// --- Structs for Window Placement and Policy Service ---

#[derive(Debug, Clone)]
pub struct WindowPlacementInfo {
    pub id: Uuid,
    pub geometry: Rect, // Using Rect from novade_core::types::geometry
}
}
