use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use log::{debug, warn, error};

use crate::workspaces::core::types::{WorkspaceId, WindowIdentifier};
use novade_core::types::{RectInt, Size};
use crate::global_settings::{
    GlobalSettingsService, SettingPath,
    // Assuming paths like these will be defined in global_settings::paths:
    // paths::WindowManagementSettingPath::FocusPolicy,
    // paths::WindowManagementSettingPath::GroupingPolicy,
    // paths::WindowManagementSettingPath::SnappingPolicy,
};


use super::types::{
    TilingMode, NewWindowPlacementStrategy, GapSettings, WindowSnappingPolicy,
    WindowLayoutInfo, WorkspaceWindowLayout, WindowPolicyOverrides, FocusPolicy, WindowGroupingPolicy,
};
use super::errors::WindowPolicyError;

#[async_trait]
pub trait WindowManagementPolicyService: Send + Sync {
    async fn calculate_workspace_layout(
        &self,
        workspace_id: WorkspaceId,
        windows_info: &[(WindowLayoutInfo, Option<WindowPolicyOverrides>)], // Updated
        current_layout_snapshot: &WorkspaceWindowLayout,
        available_area: RectInt, // This is the overall screen/monitor area for the workspace
        workspace_tiling_mode_global: TilingMode, // Global tiling mode for the workspace
        focused_window_id: Option<&WindowIdentifier>,
    ) -> Result<WorkspaceWindowLayout, WindowPolicyError>;

    async fn get_initial_window_geometry(
        &self,
        window_info_with_overrides: &(WindowLayoutInfo, Option<WindowPolicyOverrides>), // Updated
        active_layout_on_workspace: &WorkspaceWindowLayout,
        available_area: RectInt, // Overall screen/monitor area
        placement_strategy_global: NewWindowPlacementStrategy, // Global placement strategy
    ) -> Result<RectInt, WindowPolicyError>;

    async fn calculate_snap_target(
        &self,
        current_geometry: RectInt,
        workspace_area: RectInt, // Area for snapping to edges (usually after outer gaps)
        other_windows_on_workspace: &[RectInt], // Geometries of other windows
    ) -> Result<Option<RectInt>, WindowPolicyError>;
    
    async fn get_effective_focus_policy(&self) -> Result<FocusPolicy, WindowPolicyError>;
    async fn get_effective_grouping_policy(&self) -> Result<WindowGroupingPolicy, WindowPolicyError>;
}

pub struct DefaultWindowManagementPolicyService {
    settings_service: Arc<dyn GlobalSettingsService>,
}

impl DefaultWindowManagementPolicyService {
    pub fn new(settings_service: Arc<dyn GlobalSettingsService>) -> Self {
        Self { settings_service }
    }

    // Helper to fetch specific policy settings, with fallback to default.
    // This will be more robust once GlobalSettings has specific paths for these.
    async fn get_gap_settings_internal(&self) -> GapSettings {
        // Placeholder: Use a real SettingPath once defined in global_settings
        // e.g., self.settings_service.get_setting(&SettingPath::WindowManagement(WMSettingsPath::GapSettings))
        // .await.ok().and_then(|json_val| serde_json::from_value(json_val).ok()).unwrap_or_default()
        warn!("GapSettings path not yet defined in GlobalSettings. Using default.");
        GapSettings::default()
    }

    async fn get_snapping_policy_internal(&self) -> WindowSnappingPolicy {
        warn!("SnappingPolicy path not yet defined in GlobalSettings. Using default.");
        WindowSnappingPolicy::default()
    }
}


const DEFAULT_WINDOW_WIDTH: u32 = 800;
const DEFAULT_WINDOW_HEIGHT: u32 = 600;
const CASCADE_OFFSET_X: i32 = 30;
const CASCADE_OFFSET_Y: i32 = 30;


#[async_trait]
impl WindowManagementPolicyService for DefaultWindowManagementPolicyService {
    async fn calculate_workspace_layout(
        &self,
        workspace_id: WorkspaceId,
        windows_info: &[(WindowLayoutInfo, Option<WindowPolicyOverrides>)],
        _current_layout_snapshot: &WorkspaceWindowLayout, // Not directly used if we recalculate all based on overrides
        available_area: RectInt,
        workspace_tiling_mode_global: TilingMode,
        focused_window_id: Option<&WindowIdentifier>,
    ) -> Result<WorkspaceWindowLayout, WindowPolicyError> {
        let gap_settings = self.get_gap_settings_internal().await;
        
        debug!(
            "Calculating layout for workspace {}, global mode: {:?}, available_area: {:?}, {} windows, gaps: {:?}",
            workspace_id, workspace_tiling_mode_global, available_area, windows_info.len(), gap_settings
        );
        
        let layout_area = RectInt::new(
            available_area.x + gap_settings.screen_outer_horizontal as i32,
            available_area.y + gap_settings.screen_outer_vertical as i32,
            available_area.width.saturating_sub(gap_settings.screen_outer_horizontal * 2),
            available_area.height.saturating_sub(gap_settings.screen_outer_vertical * 2),
        );
        
        if layout_area.width == 0 || layout_area.height == 0 {
            return Err(WindowPolicyError::LayoutCalculationError{
                workspace_id,
                reason: "Available layout area is zero after applying screen gaps.".to_string()
            });
        }

        let mut new_geometries = HashMap::new();
        let mut floating_windows_info = Vec::new();
        let mut tiled_windows_info = Vec::new();

        // Separate windows based on overrides (floating, fixed position/size)
        for (win_layout_info, overrides_opt) in windows_info {
            let mut is_floating = win_layout_info.is_floating_override.unwrap_or(false);
            let mut fixed_pos: Option<(i32,i32)> = None;
            let mut fixed_size: Option<(u32,u32)> = None;

            if let Some(overrides) = overrides_opt {
                if overrides.is_always_floating == Some(true) {
                    is_floating = true;
                }
                if let Some(pos) = overrides.fixed_position {
                    fixed_pos = Some(pos);
                    is_floating = true; // Fixed position implies floating
                }
                if let Some(size) = overrides.fixed_size {
                    fixed_size = Some(size);
                     // Fixed size doesn't strictly imply floating, but often treated so.
                     // If not floating, it might be a constraint for tiling.
                     // For simplicity now, if it has fixed_pos or is_always_floating, it's floating.
                }
            }
            
            if is_floating {
                let size = fixed_size.map_or_else(
                    || win_layout_info.requested_min_size.map_or(
                        Size{width:DEFAULT_WINDOW_WIDTH, height:DEFAULT_WINDOW_HEIGHT}, 
                        |s| Size{width:s.width, height:s.height}),
                    |fs| Size{width: fs.0, height: fs.1}
                );
                let pos = fixed_pos.map_or_else(
                    // If no fixed pos, but floating, place it (e.g., center or cascade)
                    // For now, put it at origin of layout_area if not specified, to be snapped later
                    || (layout_area.x, layout_area.y), 
                    |fp| (fp.0, fp.1)
                );
                new_geometries.insert(win_layout_info.id.clone(), RectInt::new(pos.0, pos.1, size.width, size.height));
                floating_windows_info.push((win_layout_info, overrides_opt)); // Keep track for snapping or other logic
            } else {
                tiled_windows_info.push((win_layout_info, overrides_opt));
            }
        }

        // Determine effective tiling mode for the tiled windows
        // For Iteration 3, we'll use workspace_tiling_mode_global primarily.
        // Window-specific preferred_tiling_mode override is complex if multiple windows have different preferences.
        // A simple approach: if any tiled window has a preferred_tiling_mode, it might influence.
        // Or, if workspace_tiling_mode_global is Manual, check overrides.
        // For now, let's keep it simple: use workspace_tiling_mode_global for the tiled group.
        // A more advanced system might check overrides if workspace_tiling_mode_global == Manual.
        let active_tiling_mode_for_group = workspace_tiling_mode_global;


        // Layout for non-floating (tiled) windows
        match active_tiling_mode_for_group {
            TilingMode::Manual => { // Manual for the group of tiled windows
                for (win_info, _overrides) in &tiled_windows_info {
                    // In pure manual mode for the group, they'd keep their current relative positions or cascade.
                    // For simplicity, let's place them using a basic strategy if they don't have fixed positions.
                    // This part needs more thought for "manual tiling". If they were previously tiled, how to maintain?
                    // For now, treat as new windows needing placement if not handled by fixed overrides.
                     let initial_geom = self.get_initial_window_geometry(
                        &(*win_info, _overrides.clone()), // Pass overrides
                        &WorkspaceWindowLayout{ window_geometries: new_geometries.clone(), ..Default::default()}, // Pass current state of geometries
                        layout_area,
                        None, // Use default placement from settings
                    ).await?;
                    new_geometries.insert(win_info.id.clone(), initial_geom);
                }
            }
            TilingMode::Columns | TilingMode::Rows => {
                let num_tiled_windows = tiled_windows_info.len() as u32;
                if num_tiled_windows > 0 {
                    let total_gap_space = (num_tiled_windows.saturating_sub(1)) * gap_settings.window_inner as u32;
                    if active_tiling_mode_for_group == TilingMode::Columns {
                        let width_per_window = layout_area.width.saturating_sub(total_gap_space) / num_tiled_windows;
                        if width_per_window == 0 { return Err(WindowPolicyError::LayoutCalculationError { workspace_id, reason: "Not enough width for column layout.".to_string() }); }
                        let mut current_x = layout_area.x;
                        for (win_info, _overrides) in &tiled_windows_info {
                            new_geometries.insert(win_info.id.clone(), RectInt::new(current_x, layout_area.y, width_per_window, layout_area.height));
                            current_x += width_per_window as i32 + gap_settings.window_inner as i32;
                        }
                    } else { // Rows
                        let height_per_window = layout_area.height.saturating_sub(total_gap_space) / num_tiled_windows;
                        if height_per_window == 0 { return Err(WindowPolicyError::LayoutCalculationError { workspace_id, reason: "Not enough height for row layout.".to_string() }); }
                        let mut current_y = layout_area.y;
                        for (win_info, _overrides) in &tiled_windows_info {
                            new_geometries.insert(win_info.id.clone(), RectInt::new(layout_area.x, current_y, layout_area.width, height_per_window));
                            current_y += height_per_window as i32 + gap_settings.window_inner as i32;
                        }
                    }
                }
            }
            TilingMode::Spiral => {
                 let windows_for_spiral: Vec<WindowLayoutInfo> = tiled_windows_info.iter().map(|(info, _)| (*info).clone()).collect(); // Cloned Vec
                 self.layout_spiral_recursive_internal(&windows_for_spiral, layout_area, gap_settings.window_inner, &mut new_geometries, true);
            }
            TilingMode::MaximizedFocused => {
                if let Some(focused_id) = focused_window_id {
                    if tiled_windows_info.iter().any(|(w, _)| &w.id == focused_id) {
                        new_geometries.insert(focused_id.clone(), layout_area);
                        // Other tiled windows are hidden
                    } else if floating_windows_info.iter().any(|(w,_)| &w.id == focused_id) {
                        // Focused window is floating, handle its geometry as already set.
                        // Other tiled windows are laid out normally (e.g. columns/rows or fallback)
                        // This implies MaximizedFocused for tiled group should only apply if focused is tiled.
                        // For now, if focused is floating, tiled windows get default layout.
                        warn!("MaximizedFocused: Focused window is floating. Tiled windows get default layout.");
                        let temp_tiled_infos: Vec<WindowLayoutInfo> = tiled_windows_info.iter().map(|(i,_)| i.clone()).collect();
                        self.layout_tiled_group_fallback(&temp_tiled_infos, layout_area, &gap_settings, &mut new_geometries, workspace_id)?;

                    } else {
                        warn!("MaximizedFocused: Focused window not in layout list. Tiled group gets default layout.");
                         let temp_tiled_infos: Vec<WindowLayoutInfo> = tiled_windows_info.iter().map(|(i,_)| i.clone()).collect();
                         self.layout_tiled_group_fallback(&temp_tiled_infos, layout_area, &gap_settings, &mut new_geometries, workspace_id)?;
                    }
                } else {
                    warn!("MaximizedFocused: No focused window. Tiled group gets default layout.");
                    let temp_tiled_infos: Vec<WindowLayoutInfo> = tiled_windows_info.iter().map(|(i,_)| i.clone()).collect();
                    self.layout_tiled_group_fallback(&temp_tiled_infos, layout_area, &gap_settings, &mut new_geometries, workspace_id)?;
                }
            }
        }

        Ok(WorkspaceWindowLayout {
            window_geometries: new_geometries,
            occupied_area: Some(layout_area), // Simplified
            tiling_mode_applied: active_tiling_mode_for_group,
        })
    }

    async fn get_initial_window_geometry(
        &self,
        window_info_with_overrides: &(WindowLayoutInfo, Option<WindowPolicyOverrides>),
        active_layout_on_workspace: &WorkspaceWindowLayout,
        available_area: RectInt, // Overall screen area
        placement_strategy_global: NewWindowPlacementStrategy,
    ) -> Result<RectInt, WindowPolicyError> {
        let (window_info, overrides_opt) = window_info_with_overrides;
        
        let gap_settings = self.get_gap_settings_internal().await;
        let layout_area = RectInt::new( // Area after outer gaps
            available_area.x + gap_settings.screen_outer_horizontal as i32,
            available_area.y + gap_settings.screen_outer_vertical as i32,
            available_area.width.saturating_sub(gap_settings.screen_outer_horizontal * 2),
            available_area.height.saturating_sub(gap_settings.screen_outer_vertical * 2),
        );

        if let Some(overrides) = overrides_opt {
            if let Some(pos) = overrides.fixed_position {
                let size = overrides.fixed_size.map_or_else(
                    || window_info.requested_min_size.map_or(Size{width:DEFAULT_WINDOW_WIDTH, height:DEFAULT_WINDOW_HEIGHT}, |s| s),
                    |(w,h)| Size{width:w, height:h}
                );
                return Ok(RectInt::new(pos.0, pos.1, size.width.min(layout_area.width), size.height.min(layout_area.height)));
            }
            if let Some(size) = overrides.fixed_size {
                // Fixed size but no fixed position, use placement strategy for position
                let w_width = size.0.min(layout_area.width);
                let w_height = size.1.min(layout_area.height);
                // Fall through to placement logic with fixed size
                return self.place_window_with_strategy(window_info, w_width, w_height, active_layout_on_workspace, layout_area, placement_strategy_global).await;
            }
        }
        
        let default_size = Size { width: DEFAULT_WINDOW_WIDTH, height: DEFAULT_WINDOW_HEIGHT };
        let window_size = window_info.requested_min_size.unwrap_or(default_size);
        let w_width = window_size.width.min(layout_area.width);
        let w_height = window_size.height.min(layout_area.height);

        self.place_window_with_strategy(window_info, w_width, w_height, active_layout_on_workspace, layout_area, placement_strategy_global).await
    }
    
    async fn calculate_snap_target(
        &self,
        mut current_geometry: RectInt, // Make mutable to apply snap
        workspace_area: RectInt, // Area for screen edge snapping (e.g. layout_area)
        other_windows_on_workspace: &[RectInt],
    ) -> Result<Option<RectInt>, WindowPolicyError> {
        let snapping_policy = self.get_snapping_policy_internal().await;
        if !snapping_policy.snap_to_screen_edges && !snapping_policy.snap_to_other_windows {
            return Ok(None);
        }

        let mut snapped = false;
        let snap_dist = snapping_policy.snap_distance_px as i32;

        // Screen Edges
        if snapping_policy.snap_to_screen_edges {
            // Left edge
            if (current_geometry.x - workspace_area.x).abs() <= snap_dist { current_geometry.x = workspace_area.x; snapped = true; }
            // Right edge
            if (current_geometry.x + current_geometry.width as i32 - (workspace_area.x + workspace_area.width as i32)).abs() <= snap_dist {
                current_geometry.x = workspace_area.x + workspace_area.width as i32 - current_geometry.width as i32; snapped = true;
            }
            // Top edge
            if (current_geometry.y - workspace_area.y).abs() <= snap_dist { current_geometry.y = workspace_area.y; snapped = true; }
            // Bottom edge
            if (current_geometry.y + current_geometry.height as i32 - (workspace_area.y + workspace_area.height as i32)).abs() <= snap_dist {
                current_geometry.y = workspace_area.y + workspace_area.height as i32 - current_geometry.height as i32; snapped = true;
            }
        }

        // Other Windows
        if snapping_policy.snap_to_other_windows {
            for other_rect in other_windows_on_workspace {
                // Snap current_geometry's left to other_rect's right
                if (current_geometry.x - (other_rect.x + other_rect.width as i32)).abs() <= snap_dist { current_geometry.x = other_rect.x + other_rect.width as i32; snapped = true; }
                // Snap current_geometry's right to other_rect's left
                if (current_geometry.x + current_geometry.width as i32 - other_rect.x).abs() <= snap_dist { current_geometry.x = other_rect.x - current_geometry.width as i32; snapped = true; }
                // Snap current_geometry's top to other_rect's bottom
                if (current_geometry.y - (other_rect.y + other_rect.height as i32)).abs() <= snap_dist { current_geometry.y = other_rect.y + other_rect.height as i32; snapped = true; }
                // Snap current_geometry's bottom to other_rect's top
                if (current_geometry.y + current_geometry.height as i32 - other_rect.y).abs() <= snap_dist { current_geometry.y = other_rect.y - current_geometry.height as i32; snapped = true; }
            }
        }
        
        // snap_to_workspace_gaps: Deferred for Iteration 3 due to complexity.

        Ok(if snapped { Some(current_geometry) } else { None })
    }

    async fn get_effective_focus_policy(&self) -> Result<FocusPolicy, WindowPolicyError> {
        // Placeholder: Use a real SettingPath once defined
        // e.g. self.settings_service.get_setting(&SettingPath::WindowManagement(WMSettingsPath::FocusPolicy)) ...
        warn!("FocusPolicy path not yet defined in GlobalSettings. Using default.");
        Ok(FocusPolicy::default())
    }

    async fn get_effective_grouping_policy(&self) -> Result<WindowGroupingPolicy, WindowPolicyError> {
        warn!("GroupingPolicy path not yet defined in GlobalSettings. Using default.");
        Ok(WindowGroupingPolicy::default())
    }
}

// Helper methods extracted for clarity
impl DefaultWindowManagementPolicyService {
    async fn place_window_with_strategy(
        &self,
        window_info: &WindowLayoutInfo,
        w_width: u32,
        w_height: u32,
        active_layout_on_workspace: &WorkspaceWindowLayout,
        layout_area: RectInt, // Area after outer gaps
        placement_strategy: NewWindowPlacementStrategy,
    ) -> Result<RectInt, WindowPolicyError> {
         match placement_strategy {
            NewWindowPlacementStrategy::Center => {
                let x = layout_area.x + (layout_area.width.saturating_sub(w_width) / 2) as i32;
                let y = layout_area.y + (layout_area.height.saturating_sub(w_height) / 2) as i32;
                Ok(RectInt::new(x, y, w_width, w_height))
            }
            NewWindowPlacementStrategy::Smart => {
                let mut x = layout_area.x;
                let mut y = layout_area.y;
                let mut attempts = 0;
                let max_attempts = (layout_area.width / CASCADE_OFFSET_X.max(1) as u32) + (layout_area.height / CASCADE_OFFSET_Y.max(1) as u32) + 5;

                loop {
                    let candidate_rect = RectInt::new(x, y, w_width, w_height);
                    let overlaps = active_layout_on_workspace.window_geometries.values().any(|existing_rect| {
                        candidate_rect.x < existing_rect.x + existing_rect.width as i32 &&
                        candidate_rect.x + candidate_rect.width as i32 > existing_rect.x &&
                        candidate_rect.y < existing_rect.y + existing_rect.height as i32 &&
                        candidate_rect.y + candidate_rect.height as i32 > existing_rect.y
                    });

                    if !overlaps && 
                       (candidate_rect.x + candidate_rect.width as i32) <= (layout_area.x + layout_area.width as i32) &&
                       (candidate_rect.y + candidate_rect.height as i32) <= (layout_area.y + layout_area.height as i32) {
                        return Ok(candidate_rect);
                    }

                    x += CASCADE_OFFSET_X;
                    if x + w_width as i32 > layout_area.x + layout_area.width as i32 {
                        x = layout_area.x;
                        y += CASCADE_OFFSET_Y;
                    }
                    if y + w_height as i32 > layout_area.y + layout_area.height as i32 {
                        warn!("Smart placement failed for {:?}, falling to Center.", window_info.id);
                        return self.place_window_with_strategy(window_info, w_width, w_height, active_layout_on_workspace, layout_area, NewWindowPlacementStrategy::Center).await;
                    }
                    attempts +=1;
                    if attempts > max_attempts {
                         warn!("Smart placement max attempts for {:?}, falling to Center.", window_info.id);
                         return self.place_window_with_strategy(window_info, w_width, w_height, active_layout_on_workspace, layout_area, NewWindowPlacementStrategy::Center).await;
                    }
                }
            }
            NewWindowPlacementStrategy::Cascade => {
                let num_existing_windows = active_layout_on_workspace.window_geometries.len();
                let offset_multiplier = num_existing_windows % 10;
                let x = layout_area.x + (CASCADE_OFFSET_X * offset_multiplier as i32);
                let y = layout_area.y + (CASCADE_OFFSET_Y * offset_multiplier as i32);

                if x + w_width as i32 > layout_area.x + layout_area.width as i32 ||
                   y + w_height as i32 > layout_area.y + layout_area.height as i32 {
                    warn!("Cascade placement out of bounds for {:?}, falling to Center.", window_info.id);
                    return self.place_window_with_strategy(window_info, w_width, w_height, active_layout_on_workspace, layout_area, NewWindowPlacementStrategy::Center).await;
                }
                Ok(RectInt::new(x, y, w_width, w_height))
            }
            NewWindowPlacementStrategy::UnderMouse => {
                warn!("UnderMouse placement not implemented, falling to Center.");
                self.place_window_with_strategy(window_info, w_width, w_height, active_layout_on_workspace, layout_area, NewWindowPlacementStrategy::Center).await
            }
        }
    }
    
    fn layout_spiral_recursive_internal(
        &self,
        windows: &[WindowLayoutInfo],
        area: RectInt,
        gap: u16,
        geometries: &mut HashMap<WindowIdentifier, RectInt>,
        alternate_split: bool,
    ) {
        if windows.is_empty() || area.width == 0 || area.height == 0 { return; }
        if windows.len() == 1 { geometries.insert(windows[0].id.clone(), area); return; }

        let (win_info, rest_windows) = windows.split_at(1);
        let gap_u32 = gap as u32;

        let (area1, area2) = if alternate_split { // Horizontal split
            let split_point = area.width / 2;
            let area1_w = split_point.saturating_sub(gap_u32 / 2);
            let area2_x_offset = split_point + gap_u32.saturating_sub(gap_u32 / 2);
            let area2_w = area.width.saturating_sub(area2_x_offset);
            (RectInt::new(area.x, area.y, area1_w, area.height),
             RectInt::new(area.x + area2_x_offset as i32, area.y, area2_w, area.height))
        } else { // Vertical split
            let split_point = area.height / 2;
            let area1_h = split_point.saturating_sub(gap_u32 / 2);
            let area2_y_offset = split_point + gap_u32.saturating_sub(gap_u32 / 2);
            let area2_h = area.height.saturating_sub(area2_y_offset);
            (RectInt::new(area.x, area.y, area.width, area1_h),
             RectInt::new(area.x, area.y + area2_y_offset as i32, area.width, area2_h))
        };
        
        if area1.width > 0 && area1.height > 0 { geometries.insert(win_info[0].id.clone(), area1); }
        self.layout_spiral_recursive_internal(rest_windows, area2, gap, geometries, !alternate_split);
    }

    fn layout_tiled_group_fallback(
        &self,
        tiled_windows_info: &[WindowLayoutInfo],
        layout_area: RectInt,
        gap_settings: &GapSettings,
        new_geometries: &mut HashMap<WindowIdentifier, RectInt>,
        workspace_id: WorkspaceId, // For error reporting
    ) -> Result<(), WindowPolicyError> {
        // Fallback to simple columns for the tiled group
        if tiled_windows_info.is_empty() { return Ok(()); }
        let num_tiled_windows = tiled_windows_info.len() as u32;
        let total_gap_space = (num_tiled_windows.saturating_sub(1)) * gap_settings.window_inner as u32;
        let width_per_window = layout_area.width.saturating_sub(total_gap_space) / num_tiled_windows;

        if width_per_window == 0 {
            return Err(WindowPolicyError::LayoutCalculationError {
                workspace_id,
                reason: "Not enough width for fallback column layout in MaximizedFocused mode.".to_string(),
            });
        }
        let mut current_x = layout_area.x;
        for (win_info, _) in tiled_windows_info.iter().map(|info| (info, None::<WindowPolicyOverrides>)) { // Add None for overrides
            new_geometries.insert(win_info.id.clone(), RectInt::new(current_x, layout_area.y, width_per_window, layout_area.height));
            current_x += width_per_window as i32 + gap_settings.window_inner as i32;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspaces::core::types::WindowIdentifier;
    use novade_core::types::{RectInt, Size};
    use crate::global_settings::{GlobalSettingsService, GlobalDesktopSettings, SettingPath, GlobalSettingsError};
    use std::sync::Arc;
    use tokio::sync::broadcast; // For mock GlobalSettingsService if it uses broadcast
    use uuid::Uuid;

    #[derive(Default)]
    struct MockGlobalSettings {
        settings: GlobalDesktopSettings,
        snapping_policy: Option<WindowSnappingPolicy>,
        focus_policy: Option<FocusPolicy>,
        grouping_policy: Option<WindowGroupingPolicy>,
        // Add fields for gap_settings, default_tiling_mode, etc. if you want to control them directly
    }

    impl MockGlobalSettings {
        fn new() -> Self { Default::default() }
        #[allow(dead_code)]
        fn set_snapping_policy(&mut self, policy: WindowSnappingPolicy) { self.snapping_policy = Some(policy); }
        #[allow(dead_code)]
        fn set_focus_policy(&mut self, policy: FocusPolicy) { self.focus_policy = Some(policy); }
    }

    #[async_trait]
    impl GlobalSettingsService for MockGlobalSettings {
        async fn load_settings(&mut self) -> Result<(), GlobalSettingsError> { Ok(()) }
        async fn save_settings(&self) -> Result<(), GlobalSettingsError> { Ok(()) }
        fn get_current_settings(&self) -> GlobalDesktopSettings { self.settings.clone() }
        
        async fn get_setting(&self, path: &SettingPath) -> Result<serde_json::Value, GlobalSettingsError> {
            // This mock needs to be more sophisticated to return different values based on path
            // For now, to test specific policies, we'll check if a policy is set in the mock itself.
            // This bypasses actual SettingPath logic for these specific tests.
            // A real test would involve defining these paths in global_settings and mocking get_setting properly.
            if let Some(ref policy) = self.snapping_policy {
                // Assuming a placeholder path for snapping policy for now
                if format!("{:?}", path).contains("SnappingPolicyPlaceholder") { // Example check
                    return Ok(serde_json::to_value(policy.clone()).unwrap());
                }
            }
             if let Some(ref policy) = self.focus_policy {
                if format!("{:?}", path).contains("FocusPolicyPlaceholder") {
                    return Ok(serde_json::to_value(policy.clone()).unwrap());
                }
            }
            // Fallback for other paths needed by the service (e.g., GapSettings)
            // For this iteration, the service has hardcoded fallbacks if get_setting fails.
            Err(GlobalSettingsError::PathNotFound { path_description: format!("{:?}", path) })
        }

        async fn update_setting(&mut self, _path: SettingPath, _value: serde_json::Value) -> Result<(), GlobalSettingsError> { Ok(()) }
        async fn reset_to_defaults(&mut self) -> Result<(), GlobalSettingsError> { Ok(()) }
        fn subscribe_to_changes(&self) -> broadcast::Receiver<crate::global_settings::SettingChangedEvent> { unimplemented!() }
        fn subscribe_to_load_events(&self) -> broadcast::Receiver<crate::global_settings::SettingsLoadedEvent> { unimplemented!() }
        fn subscribe_to_save_events(&self) -> broadcast::Receiver<crate::global_settings::SettingsSavedEvent> { unimplemented!() }
    }

    fn win_layout_info_tuple(id_str: &str, overrides: Option<WindowPolicyOverrides>) -> (WindowLayoutInfo, Option<WindowPolicyOverrides>) {
        (
            WindowLayoutInfo {
                id: WindowIdentifier::new(id_str.to_string()).unwrap(),
                requested_min_size: None,
                is_fullscreen_requested: false,
                is_maximized_requested: false,
                effective_tiling_mode_override: None, // Will be set by higher layer
                is_floating_override: None, // Will be set by higher layer
            },
            overrides,
        )
    }
    fn empty_snapshot() -> WorkspaceWindowLayout { WorkspaceWindowLayout::default() }
    fn default_area() -> RectInt { RectInt::new(0,0, 1920, 1080) } // Overall screen area
    
    // Helper to get layout_area after default outer gaps
    fn default_layout_area() -> RectInt {
        let gaps = GapSettings::default();
        RectInt::new(
            default_area().x + gaps.screen_outer_horizontal as i32,
            default_area().y + gaps.screen_outer_vertical as i32,
            default_area().width.saturating_sub(gaps.screen_outer_horizontal*2),
            default_area().height.saturating_sub(gaps.screen_outer_vertical*2)
        )
    }


    #[tokio::test]
    async fn test_calculate_layout_with_floating_override() {
        let service = DefaultWindowManagementPolicyService::new(Arc::new(MockGlobalSettings::new()));
        let floating_override = Some(WindowPolicyOverrides { is_always_floating: Some(true), ..Default::default() });
        let windows = [win_layout_info_tuple("w1_float", floating_override)];
        
        let layout = service.calculate_workspace_layout(Uuid::new_v4(), &windows, &empty_snapshot(), default_area(), TilingMode::Columns, None).await.unwrap();
        
        assert_eq!(layout.window_geometries.len(), 1);
        let geom = layout.window_geometries.get(&windows[0].0.id).unwrap();
        // Should be placed by its fixed_pos or default initial placement logic for floating, not column tiling
        // Default floating placement is origin of layout_area for now
        assert_eq!(geom.x, default_layout_area().x); 
        assert_eq!(geom.y, default_layout_area().y);
        assert_eq!(geom.width, DEFAULT_WINDOW_WIDTH); // Default size
    }

    #[tokio::test]
    async fn test_get_initial_geometry_with_fixed_pos_size_override() {
        let service = DefaultWindowManagementPolicyService::new(Arc::new(MockGlobalSettings::new()));
        let fixed_override = Some(WindowPolicyOverrides { 
            fixed_position: Some((100, 150)), 
            fixed_size: Some((300, 200)),
            ..Default::default() 
        });
        let window_input = win_layout_info_tuple("w_fixed", fixed_override);
        
        let geom = service.get_initial_window_geometry(&window_input, &empty_snapshot(), default_area(), NewWindowPlacementStrategy::Center).await.unwrap();
        
        assert_eq!(geom.x, 100);
        assert_eq!(geom.y, 150);
        assert_eq!(geom.width, 300);
        assert_eq!(geom.height, 200);
    }

    #[tokio::test]
    async fn test_calculate_snap_target_to_screen_edges() {
        let mut mock_settings = MockGlobalSettings::new();
        mock_settings.set_snapping_policy(WindowSnappingPolicy {
            snap_to_screen_edges: true,
            snap_to_other_windows: false,
            snap_to_workspace_gaps: false,
            snap_distance_px: 10,
        });
        let service = DefaultWindowManagementPolicyService::new(Arc::new(mock_settings));
        let layout_area = default_layout_area(); // e.g., x:5, y:5, w:1910, h:1070

        // Snap left
        let current_geom_left = RectInt::new(layout_area.x + 8, 100, 200, 200);
        let snapped_left = service.calculate_snap_target(current_geom_left, layout_area, &[]).await.unwrap().unwrap();
        assert_eq!(snapped_left.x, layout_area.x);

        // Snap right
        let current_geom_right = RectInt::new(layout_area.x + layout_area.width as i32 - 200 - 7, 100, 200, 200);
        let snapped_right = service.calculate_snap_target(current_geom_right, layout_area, &[]).await.unwrap().unwrap();
        assert_eq!(snapped_right.x, layout_area.x + layout_area.width as i32 - 200);
        
        // Snap top
        let current_geom_top = RectInt::new(100, layout_area.y + 3, 200, 200);
        let snapped_top = service.calculate_snap_target(current_geom_top, layout_area, &[]).await.unwrap().unwrap();
        assert_eq!(snapped_top.y, layout_area.y);

        // Snap bottom
        let current_geom_bottom = RectInt::new(100, layout_area.y + layout_area.height as i32 - 200 - 4, 200, 200);
        let snapped_bottom = service.calculate_snap_target(current_geom_bottom, layout_area, &[]).await.unwrap().unwrap();
        assert_eq!(snapped_bottom.y, layout_area.y + layout_area.height as i32 - 200);
    }
    
    #[tokio::test]
    async fn test_calculate_snap_target_to_other_windows() {
        let mut mock_settings = MockGlobalSettings::new();
        mock_settings.set_snapping_policy(WindowSnappingPolicy {
            snap_to_screen_edges: false,
            snap_to_other_windows: true,
            snap_to_workspace_gaps: false,
            snap_distance_px: 10,
        });
        let service = DefaultWindowManagementPolicyService::new(Arc::new(mock_settings));
        let layout_area = default_layout_area();
        
        let other_win_rect = RectInt::new(layout_area.x + 300, layout_area.y + 100, 400, 300);
        let others = [other_win_rect];

        // Snap current_geometry's left to other_rect's right
        let current_geom_left_snap = RectInt::new(other_win_rect.x + other_win_rect.width as i32 + 5, layout_area.y + 100, 100, 100);
        let snapped = service.calculate_snap_target(current_geom_left_snap, layout_area, &others).await.unwrap().unwrap();
        assert_eq!(snapped.x, other_win_rect.x + other_win_rect.width as i32);
        
        // Snap current_geometry's right to other_rect's left
        let current_geom_right_snap = RectInt::new(other_win_rect.x - 100 - 8, layout_area.y + 100, 100, 100);
        let snapped2 = service.calculate_snap_target(current_geom_right_snap, layout_area, &others).await.unwrap().unwrap();
        assert_eq!(snapped2.x, other_win_rect.x - 100);
    }

    #[tokio::test]
    async fn test_get_effective_focus_policy_uses_defaults_on_error() {
        let mut mock_settings = MockGlobalSettings::new();
        // No specific focus policy set in mock, so get_setting will err, leading to default
        let service = DefaultWindowManagementPolicyService::new(Arc::new(mock_settings));
        let policy = service.get_effective_focus_policy().await.unwrap();
        assert_eq!(policy, FocusPolicy::default());
    }
    
    // Example test if you could set the policy in the mock for get_setting
    // #[tokio::test]
    // async fn test_get_effective_focus_policy_fetches_from_settings() {
    //     let mut mock_settings = MockGlobalSettings::new();
    //     let custom_policy = FocusPolicy { focus_follows_mouse: true, ..Default::default() };
    //     mock_settings.set_focus_policy(custom_policy.clone()); // Needs mock to use this for specific path
    //     let service = DefaultWindowManagementPolicyService::new(Arc::new(mock_settings));
    //     // This test would pass if MockGlobalSettings::get_setting was implemented to return this
    //     // for the correct SettingPath. For now, it will use default due to PathNotFound.
    //     let policy = service.get_effective_focus_policy().await.unwrap();
    //     // assert_eq!(policy, custom_policy); // This would be the ideal assertion
    //     assert_eq!(policy, FocusPolicy::default()); // Current behavior due to mock limitations
    // }

}
