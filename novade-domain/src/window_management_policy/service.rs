use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock; // Not used directly on settings_service, but settings_service might use it
use tracing::{debug, warn};

use novade_core::types::{RectInt, Size};
use crate::workspaces::core::{WorkspaceId, WindowIdentifier};
use crate::global_settings::GlobalSettingsService;
// Assuming GlobalDesktopSettings has a field `window_management_policy: WindowManagementGlobalPolicy`
// And WindowManagementGlobalPolicy contains fields like default_tiling_mode, gap_settings etc.
// For now, let's define a local placeholder if the actual structure isn't available.
// Ideally, these would be part of crate::global_settings::types::GlobalDesktopSettings
// For the purpose of this implementation, I will assume that global_settings.window_management_policy
// is a struct that mirrors the policy types directly.

// Placeholder for where these settings would be in GlobalDesktopSettings
// This should ideally be part of `crate::global_settings::types::GlobalDesktopSettings`
// For example:
// pub struct GlobalDesktopSettings {
//     ...
//     pub window_management: WindowManagementGlobalPolicy,
//     ...
// }
#[derive(Debug, Clone, Default)] // Added Default for mock
pub struct WindowManagementGlobalPolicy {
    pub default_tiling_mode: TilingMode,
    pub gap_settings: GapSettings,
    pub snapping_policy: WindowSnappingPolicy,
    pub focus_policy: FocusPolicy,
    pub new_window_placement_strategy: NewWindowPlacementStrategy,
}


use super::types::{
    TilingMode, GapSettings, WindowSnappingPolicy, NewWindowPlacementStrategy, 
    WorkspaceWindowLayout, WindowPolicyOverrides, WindowLayoutInfo, FocusPolicy, 
    FocusStealingPreventionLevel
};
use super::errors::WindowPolicyError;

// --- WindowManagementPolicyService Trait ---

#[async_trait]
pub trait WindowManagementPolicyService: Send + Sync {
    async fn calculate_workspace_layout(
        &self,
        workspace_id: WorkspaceId,
        windows_to_layout: &[WindowLayoutInfo],
        available_area: RectInt,
        workspace_current_tiling_mode: TilingMode,
        focused_window_id: Option<&WindowIdentifier>,
        window_specific_overrides: &HashMap<WindowIdentifier, WindowPolicyOverrides>
    ) -> Result<WorkspaceWindowLayout, WindowPolicyError>;

    async fn get_initial_window_geometry(
        &self,
        window_info: &WindowLayoutInfo,
        is_transient_for: Option<&WindowIdentifier>,
        parent_geometry: Option<RectInt>,
        workspace_id: WorkspaceId,
        active_layout_on_workspace: &WorkspaceWindowLayout,
        available_area: RectInt,
        window_specific_overrides: &Option<WindowPolicyOverrides>
    ) -> Result<RectInt, WindowPolicyError>;

    async fn calculate_snap_target(
        &self,
        moving_window_id: &WindowIdentifier,
        current_geometry: RectInt,
        other_windows_on_workspace: &[(&WindowIdentifier, &RectInt)],
        workspace_area: RectInt,
        snapping_policy: &WindowSnappingPolicy,
        gap_settings: &GapSettings
    ) -> Option<RectInt>;

    async fn get_effective_tiling_mode_for_workspace(&self, workspace_id: WorkspaceId) -> Result<TilingMode, WindowPolicyError>;
    async fn get_effective_gap_settings_for_workspace(&self, workspace_id: WorkspaceId) -> Result<GapSettings, WindowPolicyError>;
    async fn get_effective_snapping_policy(&self) -> Result<WindowSnappingPolicy, WindowPolicyError>;
    async fn get_effective_focus_policy(&self) -> Result<FocusPolicy, WindowPolicyError>;
    async fn get_effective_new_window_placement_strategy(&self) -> Result<NewWindowPlacementStrategy, WindowPolicyError>;

    // TODO: Assistant Integration - Needed by Smart Assistant
    // While this service is about policy, the assistant might trigger actions that depend on or change policy,
    // or query window states. Actual window manipulation (focus, close, move, resize) might belong
    // to a different service (e.g., a 'WindowControlService' or 'WindowManagerService').
    // If such a service exists, similar comments would apply there.
    // Examples:
    // fn get_focused_window_title(&self) -> Option<String>; // Needs access to window state
    // fn list_open_windows_summary(&self) -> Vec<WindowSummary>; // WindowSummary { id: WindowIdentifier, title: String, app_name: String }
    // async fn focus_window_by_criteria(&self, criteria: WindowSearchCriteria) -> Result<(), WindowPolicyError>; // criteria could be title substring, app_name, etc.
    // async fn close_window_by_criteria(&self, criteria: WindowSearchCriteria) -> Result<(), WindowPolicyError>;
    // async fn set_window_state(&self, window_id: &WindowIdentifier, state: WindowDesiredState) -> Result<(), WindowPolicyError>; // state: Maximized, Minimized, TiledLeft etc.
}

// --- DefaultWindowManagementPolicyService Implementation ---

pub struct DefaultWindowManagementPolicyService {
    settings_service: Arc<dyn GlobalSettingsService>,
}

impl DefaultWindowManagementPolicyService {
    pub fn new(settings_service: Arc<dyn GlobalSettingsService>) -> Self {
        Self { settings_service }
    }

    // Private helper functions for layout algorithms
    fn calculate_column_layout(
        &self,
        windows: &[&WindowLayoutInfo], // Changed to slice of refs
        available_area: RectInt,
        gaps: &GapSettings,
        _overrides: &HashMap<WindowIdentifier, WindowPolicyOverrides>, // Placeholder for respecting overrides
    ) -> HashMap<WindowIdentifier, RectInt> {
        let mut geometries = HashMap::new();
        if windows.is_empty() { return geometries; }

        let num_windows = windows.len() as u16;
        let total_gap_space = gaps.window_inner * (num_windows.saturating_sub(1));
        let available_width_for_windows = available_area.w.saturating_sub(total_gap_space as i32);
        
        if available_width_for_windows <= 0 {
            if let Some(first_win) = windows.first() {
                 geometries.insert(first_win.id.clone(), available_area);
            }
            return geometries;
        }

        let col_width = (available_width_for_windows as u16 / num_windows) as i32;
        let mut current_x = available_area.x;

        for window_info in windows {
            let final_width = col_width.max(window_info.requested_min_size.map_or(0, |s| s.w) as i32);
            let final_height = available_area.h.max(window_info.requested_min_size.map_or(0, |s| s.h) as i32);
            geometries.insert(
                window_info.id.clone(),
                RectInt::new(current_x, available_area.y, final_width, final_height),
            );
            current_x += final_width + gaps.window_inner as i32;
        }
        geometries
    }

    fn calculate_row_layout(
        &self,
        windows: &[&WindowLayoutInfo],
        available_area: RectInt,
        gaps: &GapSettings,
        _overrides: &HashMap<WindowIdentifier, WindowPolicyOverrides>,
    ) -> HashMap<WindowIdentifier, RectInt> {
        let mut geometries = HashMap::new();
        if windows.is_empty() { return geometries; }

        let num_windows = windows.len() as u16;
        let total_gap_space = gaps.window_inner * (num_windows.saturating_sub(1));
        let available_height_for_windows = available_area.h.saturating_sub(total_gap_space as i32);

        if available_height_for_windows <= 0 {
            if let Some(first_win) = windows.first() {
                 geometries.insert(first_win.id.clone(), available_area);
            }
            return geometries;
        }
        
        let row_height = (available_height_for_windows as u16 / num_windows) as i32;
        let mut current_y = available_area.y;

        for window_info in windows {
            let final_width = available_area.w.max(window_info.requested_min_size.map_or(0, |s| s.w) as i32);
            let final_height = row_height.max(window_info.requested_min_size.map_or(0, |s| s.h) as i32);
            geometries.insert(
                window_info.id.clone(),
                RectInt::new(available_area.x, current_y, final_width, final_height),
            );
            current_y += final_height + gaps.window_inner as i32;
        }
        geometries
    }
    
    fn calculate_spiral_layout(
        &self,
        windows: &[&WindowLayoutInfo],
        available_area: RectInt,
        gaps: &GapSettings,
        overrides: &HashMap<WindowIdentifier, WindowPolicyOverrides>,
    ) -> HashMap<WindowIdentifier, RectInt> {
        warn!("Spiral layout is not yet implemented, falling back to column layout.");
        self.calculate_column_layout(windows, available_area, gaps, overrides)
    }
}


#[async_trait]
impl WindowManagementPolicyService for DefaultWindowManagementPolicyService {
    async fn get_effective_tiling_mode_for_workspace(&self, _workspace_id: WorkspaceId) -> Result<TilingMode, WindowPolicyError> {
        let settings = self.settings_service.get_current_settings();
        // TODO: Replace with actual path in GlobalDesktopSettings when defined.
        // Ok(settings.window_management_policy.default_tiling_mode)
        Ok(WindowManagementGlobalPolicy::default().default_tiling_mode) 
    }

    async fn get_effective_gap_settings_for_workspace(&self, _workspace_id: WorkspaceId) -> Result<GapSettings, WindowPolicyError> {
        let settings = self.settings_service.get_current_settings();
        // TODO: Replace with actual path
        // Ok(settings.window_management_policy.gap_settings)
        Ok(WindowManagementGlobalPolicy::default().gap_settings)
    }

    async fn get_effective_snapping_policy(&self) -> Result<WindowSnappingPolicy, WindowPolicyError> {
        let settings = self.settings_service.get_current_settings();
        // TODO: Replace with actual path
        // Ok(settings.window_management_policy.snapping_policy)
        Ok(WindowManagementGlobalPolicy::default().snapping_policy)
    }

    async fn get_effective_focus_policy(&self) -> Result<FocusPolicy, WindowPolicyError> {
        let settings = self.settings_service.get_current_settings();
        // TODO: Replace with actual path
        // Ok(settings.window_management_policy.focus_policy)
        Ok(WindowManagementGlobalPolicy::default().focus_policy)
    }
    
    async fn get_effective_new_window_placement_strategy(&self) -> Result<NewWindowPlacementStrategy, WindowPolicyError> {
        let settings = self.settings_service.get_current_settings();
        // TODO: Replace with actual path
        // Ok(settings.window_management_policy.new_window_placement_strategy)
        Ok(WindowManagementGlobalPolicy::default().new_window_placement_strategy)
    }

    async fn calculate_workspace_layout(
        &self,
        workspace_id: WorkspaceId,
        windows_to_layout: &[WindowLayoutInfo],
        available_area: RectInt,
        workspace_current_tiling_mode: TilingMode,
        focused_window_id: Option<&WindowIdentifier>,
        window_specific_overrides: &HashMap<WindowIdentifier, WindowPolicyOverrides>
    ) -> Result<WorkspaceWindowLayout, WindowPolicyError> {
        debug!("Calculating layout for workspace {:?}, mode: {:?}, available: {:?}", workspace_id, workspace_current_tiling_mode, available_area);
        let gap_settings = self.get_effective_gap_settings_for_workspace(workspace_id).await?;

        let effective_area = RectInt {
            x: available_area.x + gap_settings.screen_outer_horizontal as i32,
            y: available_area.y + gap_settings.screen_outer_vertical as i32,
            w: available_area.w.saturating_sub((gap_settings.screen_outer_horizontal as i32 * 2)),
            h: available_area.h.saturating_sub((gap_settings.screen_outer_vertical as i32 * 2)),
        };
        
        if effective_area.w <= 0 || effective_area.h <= 0 {
            return Err(WindowPolicyError::LayoutCalculationError { workspace_id, reason: "Available area too small after outer gaps.".to_string() });
        }

        let tileable_windows: Vec<&WindowLayoutInfo> = windows_to_layout.iter().filter(|info| {
            !window_specific_overrides.get(&info.id).and_then(|ovr| ovr.is_always_floating).unwrap_or(false)
        }).collect();

        let mut window_geometries = HashMap::new();

        match workspace_current_tiling_mode {
            TilingMode::Manual => {
                for (i, win_info) in tileable_windows.iter().enumerate() {
                    let size = win_info.requested_base_size.unwrap_or(Size::new(600, 400));
                     window_geometries.insert(win_info.id.clone(), RectInt {
                        x: effective_area.x + (i as i32 * 30).min(effective_area.w - size.w as i32), 
                        y: effective_area.y + (i as i32 * 30).min(effective_area.h - size.h as i32),
                        w: size.w as i32, h: size.h as i32,
                    });
                }
            }
            TilingMode::Columns => { window_geometries = self.calculate_column_layout(&tileable_windows, effective_area, &gap_settings, window_specific_overrides); }
            TilingMode::Rows => { window_geometries = self.calculate_row_layout(&tileable_windows, effective_area, &gap_settings, window_specific_overrides); }
            TilingMode::Spiral => { window_geometries = self.calculate_spiral_layout(&tileable_windows, effective_area, &gap_settings, window_specific_overrides); }
            TilingMode::MaximizedFocused => {
                if let Some(focused_id) = focused_window_id {
                    if tileable_windows.iter().any(|w| &w.id == focused_id) {
                        window_geometries.insert(focused_id.clone(), effective_area);
                    } else {
                        window_geometries = self.calculate_column_layout(&tileable_windows, effective_area, &gap_settings, window_specific_overrides);
                    }
                } else {
                    window_geometries = self.calculate_column_layout(&tileable_windows, effective_area, &gap_settings, window_specific_overrides);
                }
            }
        }
        
        for win_info in windows_to_layout { // Add floating windows
            if window_specific_overrides.get(&win_info.id).and_then(|ovr| ovr.is_always_floating).unwrap_or(false) {
                if !window_geometries.contains_key(&win_info.id) {
                     let size = win_info.requested_base_size.unwrap_or(Size::new(500,350));
                     window_geometries.insert(win_info.id.clone(), RectInt::new(effective_area.x + 70, effective_area.y + 70, size.w as i32, size.h as i32));
                }
            }
        }

        Ok(WorkspaceWindowLayout { window_geometries, occupied_area: Some(effective_area), tiling_mode_applied: workspace_current_tiling_mode })
    }

    async fn get_initial_window_geometry(
        &self,
        window_info: &WindowLayoutInfo,
        _is_transient_for: Option<&WindowIdentifier>, // Simplified: not using _is_transient_for directly yet
        parent_geometry: Option<RectInt>,
        _workspace_id: WorkspaceId,
        _active_layout_on_workspace: &WorkspaceWindowLayout, // Simplified: not using current layout yet
        available_area: RectInt,
        window_specific_overrides: &Option<WindowPolicyOverrides>
    ) -> Result<RectInt, WindowPolicyError> {
        let base_size = window_info.requested_base_size.unwrap_or(Size::new(600, 400));
        let mut w = base_size.w as i32;
        let mut h = base_size.h as i32;
        let mut x = available_area.x + 50; // Default placement
        let mut y = available_area.y + 50;

        if let Some(overrides) = window_specific_overrides {
            if let Some(size_override) = overrides.fixed_size {
                w = size_override.0 as i32; h = size_override.1 as i32;
            }
            if let Some(pos_override) = overrides.fixed_position {
                x = pos_override.0; y = pos_override.1;
                return Ok(RectInt::new(x, y, w, h)); // Fixed pos and maybe fixed size
            }
        }
        
        if let Some(parent_rect) = parent_geometry {
            x = parent_rect.x + (parent_rect.w - w) / 2;
            y = parent_rect.y + (parent_rect.h - h) / 2;
        } else {
            let strategy = self.get_effective_new_window_placement_strategy().await?;
            match strategy {
                NewWindowPlacementStrategy::Center => { x = available_area.x + (available_area.w - w) / 2; y = available_area.y + (available_area.h - h) / 2; }
                NewWindowPlacementStrategy::Smart | NewWindowPlacementStrategy::Cascade | NewWindowPlacementStrategy::UnderMouse => { /* Placeholder */ }
            }
        }
        Ok(RectInt::new(x.max(available_area.x), y.max(available_area.y), w, h))
    }

    async fn calculate_snap_target(
        &self,
        _moving_window_id: &WindowIdentifier,
        current_geometry: RectInt,
        other_windows_on_workspace: &[(&WindowIdentifier, &RectInt)],
        workspace_area: RectInt,
        snapping_policy: &WindowSnappingPolicy,
        gap_settings: &GapSettings
    ) -> Option<RectInt> {
        if !snapping_policy.snap_to_screen_edges && !snapping_policy.snap_to_other_windows { return None; }
        let snap_dist = snapping_policy.snap_distance_px as i32;
        let mut best_snap_x = current_geometry.x; let mut best_snap_y = current_geometry.y;
        let mut min_dx = snap_dist + 1; let mut min_dy = snap_dist + 1;

        let current_edges = [current_geometry.x, current_geometry.x + current_geometry.w, current_geometry.y, current_geometry.y + current_geometry.h];
        let screen_edges = [workspace_area.x, workspace_area.x + workspace_area.w, workspace_area.y, workspace_area.y + workspace_area.h];

        if snapping_policy.snap_to_screen_edges {
            if (current_edges[0] - screen_edges[0]).abs() <= min_dx { min_dx = (current_edges[0] - screen_edges[0]).abs(); best_snap_x = screen_edges[0]; }
            if (current_edges[1] - screen_edges[1]).abs() <= min_dx { min_dx = (current_edges[1] - screen_edges[1]).abs(); best_snap_x = screen_edges[1] - current_geometry.w; }
            if (current_edges[2] - screen_edges[2]).abs() <= min_dy { min_dy = (current_edges[2] - screen_edges[2]).abs(); best_snap_y = screen_edges[2]; }
            if (current_edges[3] - screen_edges[3]).abs() <= min_dy { min_dy = (current_edges[3] - screen_edges[3]).abs(); best_snap_y = screen_edges[3] - current_geometry.h; }
        }
        if snapping_policy.snap_to_other_windows {
            for (_, other_rect) in other_windows_on_workspace {
                let other_edges = [other_rect.x, other_rect.x + other_rect.w, other_rect.y, other_rect.y + other_rect.h];
                let gap = gap_settings.window_inner as i32;
                // Snap left of current to right of other
                if (current_edges[0] - (other_edges[1] + gap)).abs() <= min_dx { min_dx = (current_edges[0] - (other_edges[1] + gap)).abs(); best_snap_x = other_edges[1] + gap; }
                // Snap right of current to left of other
                if (current_edges[1] - (other_edges[0] - gap)).abs() <= min_dx { min_dx = (current_edges[1] - (other_edges[0] - gap)).abs(); best_snap_x = other_edges[0] - gap - current_geometry.w; }
                // Snap top of current to bottom of other
                if (current_edges[2] - (other_edges[3] + gap)).abs() <= min_dy { min_dy = (current_edges[2] - (other_edges[3] + gap)).abs(); best_snap_y = other_edges[3] + gap; }
                // Snap bottom of current to top of other
                if (current_edges[3] - (other_edges[2] - gap)).abs() <= min_dy { min_dy = (current_edges[3] - (other_edges[2] - gap)).abs(); best_snap_y = other_edges[2] - gap - current_geometry.h; }
            }
        }
        if min_dx <= snap_dist || min_dy <= snap_dist { // If any edge snapped
            let final_x = if min_dx <= snap_dist { best_snap_x } else { current_geometry.x };
            let final_y = if min_dy <= snap_dist { best_snap_y } else { current_geometry.y };
            Some(RectInt::new(final_x, final_y, current_geometry.w, current_geometry.h))
        } else { None }
    }
}

#[cfg(test)]
pub struct MockGlobalSettingsService {
    pub settings: crate::global_settings::types::GlobalDesktopSettings, // Use actual GlobalDesktopSettings
}
#[cfg(test)]
impl MockGlobalSettingsService {
    pub fn new() -> Self { Self { settings: crate::global_settings::types::GlobalDesktopSettings::default() } }
    #[allow(dead_code)] // Potentially unused if tests don't modify settings directly
    pub fn set_policy_settings(&mut self, _policy: WindowManagementGlobalPolicy) {
        // self.settings.window_management_policy = policy; // TODO: Update when GlobalDesktopSettings has this
    }
}
#[cfg(test)]
#[async_trait]
impl GlobalSettingsService for MockGlobalSettingsService {
    async fn load_settings(&self) -> Result<(), crate::global_settings::errors::GlobalSettingsError> { Ok(()) }
    async fn save_settings(&self) -> Result<(), crate::global_settings::errors::GlobalSettingsError> { Ok(()) }
    fn get_current_settings(&self) -> crate::global_settings::types::GlobalDesktopSettings { self.settings.clone() }
    async fn update_setting(&self, _path: crate::global_settings::paths::SettingPath, _value: serde_json::Value) -> Result<(), crate::global_settings::errors::GlobalSettingsError> { Ok(()) }
    fn get_setting(&self, _path: &crate::global_settings::paths::SettingPath) -> Result<serde_json::Value, crate::global_settings::errors::GlobalSettingsError> { Ok(serde_json::Value::Null) }
    async fn reset_to_defaults(&self) -> Result<(), crate::global_settings::errors::GlobalSettingsError> { Ok(()) }
    fn subscribe_to_setting_changes(&self) -> broadcast::Receiver<crate::global_settings::events::SettingChangedEvent> { broadcast::channel(1).1 }
    fn subscribe_to_settings_loaded(&self) -> broadcast::Receiver<crate::global_settings::events::SettingsLoadedEvent> { broadcast::channel(1).1 }
    fn subscribe_to_settings_saved(&self) -> broadcast::Receiver<crate::global_settings::events::SettingsSavedEvent> { broadcast::channel(1).1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    // use crate::global_settings::types::WindowManagementPolicySettings as GlobalWMPolicySettings; // Actual path

    fn create_test_window_layout_info(id_str: &str) -> WindowLayoutInfo {
        WindowLayoutInfo {
            id: WindowIdentifier::from(id_str), requested_min_size: None,
            requested_base_size: Some(Size::new(200, 150)), is_fullscreen_requested: false, is_maximized_requested: false,
        }
    }

    #[tokio::test]
    async fn test_get_effective_policies_return_defaults() {
        let mock_settings_service = Arc::new(MockGlobalSettingsService::new());
        let policy_service = DefaultWindowManagementPolicyService::new(mock_settings_service);
        assert_eq!(policy_service.get_effective_tiling_mode_for_workspace(Uuid::new_v4()).await.unwrap(), TilingMode::default());
        assert_eq!(policy_service.get_effective_gap_settings_for_workspace(Uuid::new_v4()).await.unwrap(), GapSettings::default());
    }
    
    #[tokio::test]
    async fn test_calculate_workspace_layout_column_basic() {
        let mock_settings_service = Arc::new(MockGlobalSettingsService::new());
        let policy_service = DefaultWindowManagementPolicyService::new(mock_settings_service);
        let windows_vec = vec![create_test_window_layout_info("win1"), create_test_window_layout_info("win2")];
        let windows_refs: Vec<&WindowLayoutInfo> = windows_vec.iter().collect();
        let area = RectInt::new(0, 0, 800, 600);
        let layout = policy_service.calculate_workspace_layout(Uuid::new_v4(), &windows_refs, area, TilingMode::Columns, None, &HashMap::new()).await.unwrap();
        assert_eq!(layout.tiling_mode_applied, TilingMode::Columns); assert_eq!(layout.window_geometries.len(), 2);
        assert_eq!(layout.window_geometries.get(&windows_vec[0].id).unwrap().w, 400);
        assert_eq!(layout.window_geometries.get(&windows_vec[1].id).unwrap().w, 400);
    }
    
    #[tokio::test]
    async fn test_calculate_workspace_layout_maximized_focused() {
        let mock_settings_service = Arc::new(MockGlobalSettingsService::new());
        let policy_service = DefaultWindowManagementPolicyService::new(mock_settings_service);
        let win1 = create_test_window_layout_info("win1"); let win2 = create_test_window_layout_info("win2");
        let windows_vec = vec![win1.clone(), win2.clone()];
        let windows_refs: Vec<&WindowLayoutInfo> = windows_vec.iter().collect();
        let area = RectInt::new(0, 0, 800, 600);
        let layout = policy_service.calculate_workspace_layout(Uuid::new_v4(), &windows_refs, area, TilingMode::MaximizedFocused, Some(&win1.id), &HashMap::new()).await.unwrap();
        assert_eq!(layout.window_geometries.len(), 1);
        assert_eq!(*layout.window_geometries.get(&win1.id).unwrap(), area);
    }

    #[tokio::test]
    async fn test_get_initial_window_geometry_center_default() {
        let mock_settings_service = Arc::new(MockGlobalSettingsService::new()); // Uses default NewWindowPlacementStrategy::Smart
        let policy_service = DefaultWindowManagementPolicyService::new(mock_settings_service);
        let win_info = create_test_window_layout_info("win1"); // base size 200x150
        let available_area = RectInt::new(0, 0, 1000, 800);
        let rect = policy_service.get_initial_window_geometry(&win_info, None, None, Uuid::new_v4(), &WorkspaceWindowLayout::default(), available_area, &None).await.unwrap();
        // Default strategy is Smart, current placeholder for Smart is 50,50
        assert_eq!(rect, RectInt::new(50, 50, 200, 150));
    }
    
    #[tokio::test]
    async fn test_calculate_snap_target_no_snap() {
        let mock_settings_service = Arc::new(MockGlobalSettingsService::new());
        let policy_service = DefaultWindowManagementPolicyService::new(mock_settings_service);
        let snapping_policy = WindowSnappingPolicy { snap_to_screen_edges: false, snap_to_other_windows: false, ..Default::default() };
        assert!(policy_service.calculate_snap_target(&WindowIdentifier::from("win1"), RectInt::new(100,100,200,200), &[], RectInt::new(0,0,800,600), &snapping_policy, &GapSettings::default()).await.is_none());
    }

    #[tokio::test]
    async fn test_calculate_snap_target_to_screen_edge() {
        let mock_settings_service = Arc::new(MockGlobalSettingsService::new());
        let policy_service = DefaultWindowManagementPolicyService::new(mock_settings_service);
        let snapping_policy = WindowSnappingPolicy { snap_to_screen_edges: true, snap_distance_px: 10, ..Default::default() };
        let current_geom = RectInt::new(5, 100, 200, 200); // 5px from left edge
        let target = policy_service.calculate_snap_target(&WindowIdentifier::from("win1"), current_geom, &[], RectInt::new(0,0,800,600), &snapping_policy, &GapSettings::default()).await;
        assert_eq!(target.unwrap().x, 0);
    }
}
