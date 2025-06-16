// novade-system/src/window_mechanics/manager.rs

use std::collections::HashMap;
use tracing::{info, debug, error, warn};

// Import our own error and data types
use super::error::WindowManagerError;
use super::data_types::{
    WindowId, WindowInfo, WindowRect, WindowState,
    WorkspaceId, WorkspaceInfo, TilingLayout, ScreenInfo,
};
use novade_core::types::geometry::{Point, Size};

/// Manages the arrangement, sizing, and state of windows and workspaces.
///
/// The `WindowManager` is responsible for applying tiling algorithms, handling
/// window focus, managing workspaces, and reacting to window events (like creation
/// or closure). It will eventually interact with the Smithay compositor state
/// to apply these changes to the actual Wayland surfaces.
#[derive(Debug)]
pub struct WindowManager {
    /// A map of all known windows, keyed by their `WindowId`.
    windows: HashMap<WindowId, WindowInfo>,
    /// A map of all workspaces, keyed by their `WorkspaceId`.
    workspaces: HashMap<WorkspaceId, WorkspaceInfo>,
    /// The ID of the currently active workspace.
    active_workspace_id: WorkspaceId,
    // /// Shared access to the global compositor state.
    // /// This will be needed for interacting with Smithay surfaces.
    // /// Example type: smithay_compositor_state: Arc<RwLock<CompositorState>>,
}

impl WindowManager {
    /// Creates a new `WindowManager`.
    ///
    /// Initializes with a default workspace. In a real scenario, this might
    /// take configuration for screen setups or access to the compositor state.
    pub fn new() -> Result<Self, WindowManagerError> {
        info!("Initializing WindowManager.");

        let mut workspaces = HashMap::new();
        let default_workspace_id = WorkspaceId::new_v4();
        let default_workspace_name = "Workspace 1".to_string();
        let default_layout = TilingLayout::Tall;

        let default_workspace = WorkspaceInfo::new(
            default_workspace_id,
            default_workspace_name.clone(),
            default_layout,
        );
        workspaces.insert(default_workspace_id, default_workspace);

        debug!(
            "Default workspace created: ID={}, Name='{}', Layout={:?}",
            default_workspace_id, default_workspace_name, default_layout
        );

        Ok(WindowManager {
            windows: HashMap::new(),
            workspaces,
            active_workspace_id: default_workspace_id,
        })
    }

    /// Handles the registration of a new window.
    ///
    /// A new `WindowInfo` is created and associated with a generated `WindowId`.
    /// The window is added to the currently active workspace, and the layout
    /// for that workspace is recalculated.
    ///
    /// # Arguments
    /// * `surface_title`: The title of the new window.
    ///
    /// # Returns
    /// A `Result` containing the `WindowId` of the newly created window, or
    /// a `WindowManagerError`.
    pub fn on_new_window(&mut self, surface_title: String) -> Result<WindowId, WindowManagerError> {
        let window_id = WindowId::new_v4();
        let initial_rect = WindowRect::default();
        let initial_state = WindowState::Tiled;

        let window_info = WindowInfo::new(
            window_id,
            surface_title.clone(),
            initial_rect,
            initial_state,
        );

        self.windows.insert(window_id, window_info);
        debug!("Registered new window: ID={}, Title='{}'", window_id, surface_title);

        let active_workspace = self.workspaces.get_mut(&self.active_workspace_id)
            .ok_or_else(|| {
                error!("Active workspace ID {} not found during new window creation.", self.active_workspace_id);
                WindowManagerError::Other(format!("Active workspace {} not found", self.active_workspace_id))
            })?;

        active_workspace.windows.push(window_id);
        info!("Added window {} to workspace {}", window_id, active_workspace.id);

        self.recalculate_layout(self.active_workspace_id)?;

        Ok(window_id)
    }

    /// Handles the closure of an existing window.
    ///
    /// The window is removed from the manager's list and from any workspace
    /// it belonged to. The layout for affected workspaces is then recalculated.
    ///
    /// # Arguments
    /// * `window_id`: The ID of the window to be closed.
    ///
    /// # Returns
    /// A `Result` indicating success, or an `InvalidWindowId` error or `Other` error.
    pub fn on_window_closed(&mut self, window_id: &WindowId) -> Result<(), WindowManagerError> {
        if self.windows.remove(window_id).is_none() {
            warn!("Attempted to close non-existent window: {}", window_id);
            return Err(WindowManagerError::InvalidWindowId(window_id.to_string()));
        }
        info!("Window {} closed and removed from manager.", window_id);

        let mut affected_workspaces = Vec::new();
        for (id, workspace_info) in self.workspaces.iter_mut() {
            if workspace_info.windows.contains(window_id) {
                workspace_info.windows.retain(|id_in_list| id_in_list != window_id);
                debug!("Removed window {} from workspace {}", window_id, id);
                affected_workspaces.push(*id);
            }
        }

        for ws_id in affected_workspaces {
            self.recalculate_layout(ws_id)?;
        }
        Ok(())
    }

    /// Recalculates and applies the layout for a given workspace.
    ///
    /// # Arguments
    /// * `workspace_id`: The ID of the workspace to recalculate.
    ///
    /// # Returns
    /// A `Result` indicating success, or an error.
    pub fn recalculate_layout(&mut self, workspace_id: WorkspaceId) -> Result<(), WindowManagerError> {
        let workspace = self.workspaces.get(&workspace_id)
            .ok_or_else(|| {
                error!("Workspace ID {} not found during layout recalculation.", workspace_id);
                WindowManagerError::Other(format!("Workspace {} not found for layout", workspace_id))
            })?;

        let screen_info = ScreenInfo { width: 1920.0, height: 1080.0 };

        debug!("Recalculating layout for workspace {} ({}) using {:?} on screen {}x{}",
            workspace.id, workspace.name, workspace.current_layout, screen_info.width, screen_info.height);

        let mut tiled_windows_in_workspace: Vec<&WindowInfo> = Vec::new();
        for id_in_ws in &workspace.windows {
            if let Some(win_info) = self.windows.get(id_in_ws) {
                if win_info.state == WindowState::Tiled {
                    tiled_windows_in_workspace.push(win_info);
                }
            } else {
                warn!("Window ID {} found in workspace {} but not in main window map.", id_in_ws, workspace_id);
            }
        }

        if tiled_windows_in_workspace.is_empty() {
            debug!("No tiled windows in workspace {} to arrange.", workspace_id);
            return Ok(());
        }

        let new_rects = match workspace.current_layout {
            TilingLayout::Tall => Self::calculate_tall_layout(&screen_info, &tiled_windows_in_workspace),
            TilingLayout::Grid => Self::calculate_grid_layout(&screen_info, &tiled_windows_in_workspace),
            TilingLayout::Fibonacci => {
                warn!("Fibonacci layout is not yet implemented for workspace {}. Defaulting to Tall.", workspace_id);
                Self::calculate_tall_layout(&screen_info, &tiled_windows_in_workspace)
            }
        }?;

        if new_rects.len() != tiled_windows_in_workspace.len() {
            error!("Layout algorithm returned {} rects for {} windows in workspace {}.", new_rects.len(), tiled_windows_in_workspace.len(), workspace_id);
            return Err(WindowManagerError::LayoutCalculation("Mismatch between windows and rectangles".to_string()));
        }

        for (window_info_ref, new_rect) in tiled_windows_in_workspace.iter().zip(new_rects) {
            if let Some(mutable_window_info) = self.windows.get_mut(&window_info_ref.id) {
                mutable_window_info.rect = new_rect;
                debug!("Updated window {} rect to: {:?}", mutable_window_info.id, new_rect);
            }
        }

        info!("Layout recalculated and applied for workspace {}", workspace_id);
        Ok(())
    }

    /// Switches the active workspace.
    ///
    /// # Arguments
    /// * `target_workspace_id`: The ID of the workspace to switch to.
    ///
    /// # Returns
    /// A `Result` indicating success, or an error.
    pub fn switch_workspace(&mut self, target_workspace_id: WorkspaceId) -> Result<(), WindowManagerError> {
        if !self.workspaces.contains_key(&target_workspace_id) {
            warn!("Attempted to switch to non-existent workspace: {}", target_workspace_id);
            return Err(WindowManagerError::Other(format!("Workspace {} does not exist", target_workspace_id)));
        }

        if self.active_workspace_id == target_workspace_id {
            debug!("Attempted to switch to already active workspace: {}", target_workspace_id);
            return Ok(());
        }

        info!("Switching active workspace from {} to {}", self.active_workspace_id, target_workspace_id);
        self.active_workspace_id = target_workspace_id;
        self.recalculate_layout(self.active_workspace_id)?;
        Ok(())
    }

    /// Sets the state of a specific window.
    ///
    /// # Arguments
    /// * `window_id`: The ID of the window.
    /// * `new_state`: The new `WindowState` to apply.
    ///
    /// # Returns
    /// A `Result` indicating success, or `InvalidWindowId`.
    pub fn set_window_state(&mut self, window_id: WindowId, new_state: WindowState) -> Result<(), WindowManagerError> {
        let window_info = self.windows.get_mut(&window_id)
            .ok_or_else(|| {
                warn!("Cannot set state for non-existent window: {}", window_id);
                WindowManagerError::InvalidWindowId(window_id.to_string())
            })?;

        if window_info.state == new_state {
            debug!("Window {} is already in state {:?}. No change.", window_id, new_state);
            return Ok(());
        }

        info!("Setting window {} state from {:?} to {:?}", window_id, window_info.state, new_state);
        window_info.state = new_state;

        let mut parent_workspace_id: Option<WorkspaceId> = None;
        for (ws_id, ws_info) in &self.workspaces {
            if ws_info.windows.contains(&window_id) {
                parent_workspace_id = Some(*ws_id);
                break;
            }
        }

        if let Some(ws_id) = parent_workspace_id {
            debug!("Recalculating layout for workspace {} due to window {} state change.", ws_id, window_id);
            self.recalculate_layout(ws_id)?;
        } else {
            warn!("Window {} changed state but was not found in any workspace.", window_id);
        }
        Ok(())
    }

    // --- Static Layout Calculation Methods ---
    pub(crate) fn calculate_tall_layout(
        screen: &ScreenInfo,
        windows_in_layout: &[&WindowInfo],
    ) -> Result<Vec<WindowRect>, WindowManagerError> {
        if windows_in_layout.is_empty() {
            return Ok(Vec::new());
        }
        let num_windows = windows_in_layout.len();
        let mut rects = Vec::with_capacity(num_windows);
        let master_width_ratio = 0.6;
        let master_width = screen.width * master_width_ratio;
        let stack_width = screen.width * (1.0 - master_width_ratio);

        rects.push(WindowRect {
            origin: Point::new(0.0, 0.0),
            size: Size::new(master_width, screen.height),
        });

        if num_windows > 1 {
            let num_stack_windows = num_windows - 1;
            let stack_window_height = screen.height / num_stack_windows as f64;
            for i in 0..num_stack_windows {
                rects.push(WindowRect {
                    origin: Point::new(master_width, i as f64 * stack_window_height),
                    size: Size::new(stack_width, stack_window_height),
                });
            }
        }
        debug!("Calculated Tall layout for {} windows on screen {}x{}", num_windows, screen.width, screen.height);
        Ok(rects)
    }

    pub(crate) fn calculate_grid_layout(
        screen: &ScreenInfo,
        windows_in_layout: &[&WindowInfo],
    ) -> Result<Vec<WindowRect>, WindowManagerError> {
        if windows_in_layout.is_empty() {
            return Ok(Vec::new());
        }
        let num_windows = windows_in_layout.len();
        let mut rects = Vec::with_capacity(num_windows);
        let cols = (num_windows as f64).sqrt().ceil() as usize;
        let rows = (num_windows as f64 / cols as f64).ceil() as usize;

        if cols == 0 || rows == 0 {
            return Err(WindowManagerError::LayoutCalculation(
                format!("Grid layout: Invalid column ({}) or row ({}) count for {} windows.", cols, rows, num_windows)
            ));
        }
        let cell_width = screen.width / cols as f64;
        let cell_height = screen.height / rows as f64;

        for i in 0..num_windows {
            let row_idx = i / cols;
            let col_idx = i % cols;
            rects.push(WindowRect {
                origin: Point::new(col_idx as f64 * cell_width, row_idx as f64 * cell_height),
                size: Size::new(cell_width, cell_height),
            });
        }
        debug!("Calculated Grid layout for {} windows on screen {}x{}", num_windows, screen.width, screen.height);
        Ok(rects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::types::geometry::{Point, Size};

    fn setup_manager() -> WindowManager {
        WindowManager::new().expect("Failed to create WindowManager for testing")
    }

    #[test]
    fn window_manager_new_initializes_default_workspace() {
        let manager = setup_manager();
        assert_eq!(manager.windows.len(), 0);
        assert_eq!(manager.workspaces.len(), 1);
        let active_ws_id = manager.active_workspace_id;
        let default_ws = manager.workspaces.get(&active_ws_id).unwrap();
        assert_eq!(default_ws.name, "Workspace 1");
        assert_eq!(default_ws.current_layout, TilingLayout::Tall);
    }

    #[test]
    fn on_new_window_adds_to_active_workspace_and_recalculates() {
        let mut manager = setup_manager();
        let initial_active_ws_id = manager.active_workspace_id;
        let window_title = "Test Window 1".to_string();
        let result = manager.on_new_window(window_title.clone());
        assert!(result.is_ok());
        let window_id = result.unwrap();

        assert!(manager.windows.contains_key(&window_id));
        let window_info = manager.windows.get(&window_id).unwrap();
        assert_eq!(window_info.title, window_title);
        assert_eq!(window_info.state, WindowState::Tiled);

        let active_workspace = manager.workspaces.get(&initial_active_ws_id).unwrap();
        assert!(active_workspace.windows.contains(&window_id));
        assert_eq!(active_workspace.windows.len(), 1);

        let expected_width = 1920.0 * 0.6;
        assert_eq!(window_info.rect.origin, Point::new(0.0, 0.0));
        assert_eq!(window_info.rect.size, Size::new(expected_width, 1080.0));
    }

    #[test]
    fn on_new_window_multiple_windows_trigger_layout() {
        let mut manager = setup_manager();
        let active_ws_id = manager.active_workspace_id;
        let id1 = manager.on_new_window("W1".to_string()).unwrap();
        let id2 = manager.on_new_window("W2".to_string()).unwrap();

        let ws = manager.workspaces.get(&active_ws_id).unwrap();
        assert_eq!(ws.windows.len(), 2);

        let win1_info = manager.windows.get(&id1).unwrap();
        let win2_info = manager.windows.get(&id2).unwrap();
        let master_width = 1920.0 * 0.6;
        let stack_width = 1920.0 * 0.4;

        assert_eq!(win1_info.rect.size, Size::new(master_width, 1080.0));
        assert_eq!(win2_info.rect.origin, Point::new(master_width, 0.0));
        assert_eq!(win2_info.rect.size, Size::new(stack_width, 1080.0));
    }

    #[test]
    fn on_window_closed_removes_from_manager_and_workspace() {
        let mut manager = setup_manager();
        let window_id = manager.on_new_window("Test Window".to_string()).unwrap();
        let initial_active_ws_id = manager.active_workspace_id;

        assert!(manager.windows.contains_key(&window_id));
        assert!(manager.workspaces.get(&initial_active_ws_id).unwrap().windows.contains(&window_id));

        let close_result = manager.on_window_closed(&window_id);
        assert!(close_result.is_ok());

        assert!(!manager.windows.contains_key(&window_id));
        let active_workspace_after_close = manager.workspaces.get(&initial_active_ws_id).unwrap();
        assert!(!active_workspace_after_close.windows.contains(&window_id));
        assert!(active_workspace_after_close.windows.is_empty());
    }

    #[test]
    fn on_window_closed_non_existent_id() {
        let mut manager = setup_manager();
        let fake_window_id = WindowId::new_v4();
        let result = manager.on_window_closed(&fake_window_id);
        assert!(result.is_err());
        match result.unwrap_err() {
            WindowManagerError::InvalidWindowId(id_str) => assert_eq!(id_str, fake_window_id.to_string()),
            _ => panic!("Expected InvalidWindowId error"),
        }
    }

    #[test]
    fn on_window_closed_recalculates_layout_for_remaining() {
        let mut manager = setup_manager();
        let active_ws_id = manager.active_workspace_id;
        let id1 = manager.on_new_window("W1".to_string()).unwrap();
        let id2 = manager.on_new_window("W2".to_string()).unwrap();

        let close_result = manager.on_window_closed(&id2);
        assert!(close_result.is_ok());

        let ws = manager.workspaces.get(&active_ws_id).unwrap();
        assert_eq!(ws.windows.len(), 1);
        assert!(ws.windows.contains(&id1));

        let win1_info = manager.windows.get(&id1).unwrap();
        let expected_master_width = 1920.0 * 0.6;
        assert_eq!(win1_info.rect.origin, Point::new(0.0, 0.0));
        assert_eq!(win1_info.rect.size, Size::new(expected_master_width, 1080.0));
    }

    #[test]
    fn switch_workspace_updates_active_id_and_recalculates() {
        let mut manager = setup_manager();
        let initial_active_id = manager.active_workspace_id;
        let new_ws_id = WorkspaceId::new_v4();
        let new_ws_info = WorkspaceInfo::new(new_ws_id, "Workspace 2".to_string(), TilingLayout::Grid);
        manager.workspaces.insert(new_ws_id, new_ws_info.clone());

        let window_in_new_ws_id = WindowId::new_v4();
        let win_info = WindowInfo::new(window_in_new_ws_id, "WinInNewWS".to_string(), WindowRect::default(), WindowState::Tiled);
        manager.windows.insert(window_in_new_ws_id, win_info);
        manager.workspaces.get_mut(&new_ws_id).unwrap().windows.push(window_in_new_ws_id);

        let switch_result = manager.switch_workspace(new_ws_id);
        assert!(switch_result.is_ok());
        assert_eq!(manager.active_workspace_id, new_ws_id);
        assert_ne!(manager.active_workspace_id, initial_active_id);

        let switched_win_info = manager.windows.get(&window_in_new_ws_id).unwrap();
        assert_eq!(switched_win_info.rect.origin, Point::new(0.0,0.0));
        assert_eq!(switched_win_info.rect.size, Size::new(1920.0,1080.0));
    }

    #[test]
    fn switch_workspace_to_non_existent_id() {
        let mut manager = setup_manager();
        let fake_ws_id = WorkspaceId::new_v4();
        let result = manager.switch_workspace(fake_ws_id);
        assert!(result.is_err());
        match result.unwrap_err() {
            WindowManagerError::Other(msg) => assert!(msg.contains("does not exist")),
            _ => panic!("Expected Other error"),
        }
    }

    #[test]
    fn switch_workspace_to_same_id_is_ok_and_no_op_effectively() {
        let mut manager = setup_manager();
        let initial_active_id = manager.active_workspace_id;
        let win_id = manager.on_new_window("W1".to_string()).unwrap();
        let rect_before_switch = manager.windows.get(&win_id).unwrap().rect;

        let switch_result = manager.switch_workspace(initial_active_id);
        assert!(switch_result.is_ok());
        assert_eq!(manager.active_workspace_id, initial_active_id);

        let rect_after_switch = manager.windows.get(&win_id).unwrap().rect;
        assert_eq!(rect_before_switch, rect_after_switch);
    }

    #[test]
    fn set_window_state_updates_state_and_recalculates() {
        let mut manager = setup_manager();
        let window_id = manager.on_new_window("Test Window".to_string()).unwrap();
        let initial_rect = manager.windows.get(&window_id).unwrap().rect;
        assert_eq!(manager.windows.get(&window_id).unwrap().state, WindowState::Tiled);

        let set_state_result = manager.set_window_state(window_id, WindowState::Floating);
        assert!(set_state_result.is_ok());

        let window_info_after_state_change = manager.windows.get(&window_id).unwrap();
        assert_eq!(window_info_after_state_change.state, WindowState::Floating);
        // Recalculate layout should not affect floating window's preserved rect
        // but the current implementation of recalculate_layout in the test code
        // always updates rects for Tiled windows. If it were Floating, it would be skipped.
        // The key here is that if it becomes Floating, the next Tiled layout won't touch it.
        // Let's verify that by adding another tiled window.

        let _window2_id = manager.on_new_window("Another Window".to_string()).unwrap();
        let rect_of_floating_window_after_new_tiled = manager.windows.get(&window_id).unwrap().rect;
        assert_eq!(window_info_after_state_change.rect, rect_of_floating_window_after_new_tiled, "Floating window rect should not change when other tiled windows are added/rearranged.");


        let set_state_back_result = manager.set_window_state(window_id, WindowState::Tiled);
        assert!(set_state_back_result.is_ok());
        let window_info_after_tiled_again = manager.windows.get(&window_id).unwrap();
        assert_eq!(window_info_after_tiled_again.state, WindowState::Tiled);
        // Now that it's Tiled again, its rect should be updated by the layout.
        // It will be the first window, so it gets the master pane.
        let expected_master_width = 1920.0 * 0.6;
        assert_eq!(window_info_after_tiled_again.rect.size.width, expected_master_width);
    }

    #[test]
    fn set_window_state_for_non_existent_id() {
        let mut manager = setup_manager();
        let fake_window_id = WindowId::new_v4();
        let result = manager.set_window_state(fake_window_id, WindowState::Floating);
        assert!(result.is_err());
        match result.unwrap_err() {
            WindowManagerError::InvalidWindowId(id_str) => assert_eq!(id_str, fake_window_id.to_string()),
            _ => panic!("Expected InvalidWindowId error"),
        }
    }

    #[test]
    fn recalculate_layout_handles_fibonacci_gracefully_for_now() {
        let mut manager = setup_manager();
        let active_ws_id = manager.active_workspace_id;
        manager.workspaces.get_mut(&active_ws_id).unwrap().current_layout = TilingLayout::Fibonacci;
        let window_id = manager.on_new_window("Test Window".to_string()).unwrap();

        let window_info = manager.windows.get(&window_id).unwrap();
        let expected_width = 1920.0 * 0.6;
        assert_eq!(window_info.rect.origin, Point::new(0.0, 0.0));
        assert_eq!(window_info.rect.size, Size::new(expected_width, 1080.0));
    }

    #[test]
    fn recalculate_layout_no_tiled_windows() {
        let mut manager = setup_manager();
        let active_ws_id = manager.active_workspace_id;
        let window_id = manager.on_new_window("Floating Window".to_string()).unwrap();

        // Preserve its initial rect (which might be default or from a previous layout)
        let initial_rect_before_floating = manager.windows.get(&window_id).unwrap().rect;
        manager.set_window_state(window_id, WindowState::Floating).unwrap();
        // After setting to floating, its rect should remain what it was.
        let rect_after_floating = manager.windows.get(&window_id).unwrap().rect;
        assert_eq!(initial_rect_before_floating, rect_after_floating);

        let recalc_result = manager.recalculate_layout(active_ws_id);
        assert!(recalc_result.is_ok());
        let rect_after_recalc = manager.windows.get(&window_id).unwrap().rect;
        // Since it's floating, recalculate_layout should not have changed its rect.
        assert_eq!(rect_after_floating, rect_after_recalc);
    }

    // --- Tests for Tall Layout ---
    #[test]
    fn tall_layout_no_windows() {
        let screen = ScreenInfo::new(1920.0, 1080.0);
        let windows_in_layout: Vec<&WindowInfo> = Vec::new();
        let rects = WindowManager::calculate_tall_layout(&screen, &windows_in_layout).unwrap();
        assert!(rects.is_empty());
    }

    #[test]
    fn tall_layout_one_window() {
        let screen = ScreenInfo::new(1920.0, 1080.0);
        let win_info = WindowInfo { id: WindowId::new_v4(), title: "Test".into(), rect: Default::default(), state: WindowState::Tiled };
        let windows_in_layout = vec![&win_info];
        let rects = WindowManager::calculate_tall_layout(&screen, &windows_in_layout).unwrap();
        assert_eq!(rects.len(), 1);
        let master_width = 1920.0 * 0.6;
        assert_eq!(rects[0].origin, Point::new(0.0, 0.0));
        assert_eq!(rects[0].size, Size::new(master_width, 1080.0));
    }

    #[test]
    fn tall_layout_two_windows() {
        let screen = ScreenInfo::new(1920.0, 1080.0);
        let win1_info = WindowInfo { id: WindowId::new_v4(), title: "W1".into(), rect: Default::default(), state: WindowState::Tiled };
        let win2_info = WindowInfo { id: WindowId::new_v4(), title: "W2".into(), rect: Default::default(), state: WindowState::Tiled };
        let windows_in_layout = vec![&win1_info, &win2_info];
        let rects = WindowManager::calculate_tall_layout(&screen, &windows_in_layout).unwrap();
        assert_eq!(rects.len(), 2);
        let master_width = 1920.0 * 0.6;
        let stack_width = 1920.0 * 0.4;
        assert_eq!(rects[0].origin, Point::new(0.0, 0.0));
        assert_eq!(rects[0].size, Size::new(master_width, 1080.0));
        assert_eq!(rects[1].origin, Point::new(master_width, 0.0));
        assert_eq!(rects[1].size, Size::new(stack_width, 1080.0));
    }

    #[test]
    fn tall_layout_three_windows() {
        let screen = ScreenInfo::new(1920.0, 1080.0);
        let win1 = WindowInfo { id: WindowId::new_v4(), title: "W1".into(), rect: Default::default(), state: WindowState::Tiled };
        let win2 = WindowInfo { id: WindowId::new_v4(), title: "W2".into(), rect: Default::default(), state: WindowState::Tiled };
        let win3 = WindowInfo { id: WindowId::new_v4(), title: "W3".into(), rect: Default::default(), state: WindowState::Tiled };
        let windows_in_layout = vec![&win1, &win2, &win3];
        let rects = WindowManager::calculate_tall_layout(&screen, &windows_in_layout).unwrap();
        assert_eq!(rects.len(), 3);
        let master_width = 1920.0 * 0.6;
        let stack_width = 1920.0 * 0.4;
        let stack_window_height = 1080.0 / 2.0;
        assert_eq!(rects[0].origin, Point::new(0.0, 0.0));
        assert_eq!(rects[0].size, Size::new(master_width, 1080.0));
        assert_eq!(rects[1].origin, Point::new(master_width, 0.0));
        assert_eq!(rects[1].size, Size::new(stack_width, stack_window_height));
        assert_eq!(rects[2].origin, Point::new(master_width, stack_window_height));
        assert_eq!(rects[2].size, Size::new(stack_width, stack_window_height));
    }

    // --- Tests for Grid Layout ---
    #[test]
    fn grid_layout_no_windows() {
        let screen = ScreenInfo::new(1920.0, 1080.0);
        let windows_in_layout: Vec<&WindowInfo> = Vec::new();
        let rects = WindowManager::calculate_grid_layout(&screen, &windows_in_layout).unwrap();
        assert!(rects.is_empty());
    }

    #[test]
    fn grid_layout_one_window() {
        let screen = ScreenInfo::new(1920.0, 1080.0);
        let win_info = WindowInfo { id: WindowId::new_v4(), title: "Test".into(), rect: Default::default(), state: WindowState::Tiled };
        let windows_in_layout = vec![&win_info];
        let rects = WindowManager::calculate_grid_layout(&screen, &windows_in_layout).unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].origin, Point::new(0.0, 0.0));
        assert_eq!(rects[0].size, Size::new(1920.0, 1080.0));
    }

    #[test]
    fn grid_layout_two_windows() {
        let screen = ScreenInfo::new(1920.0, 1080.0);
        let win1 = WindowInfo { id: WindowId::new_v4(), title: "W1".into(), rect: Default::default(), state: WindowState::Tiled };
        let win2 = WindowInfo { id: WindowId::new_v4(), title: "W2".into(), rect: Default::default(), state: WindowState::Tiled };
        let windows_in_layout = vec![&win1, &win2];
        let rects = WindowManager::calculate_grid_layout(&screen, &windows_in_layout).unwrap();
        assert_eq!(rects.len(), 2);
        let cell_width = 1920.0 / 2.0;
        let cell_height = 1080.0 / 1.0;
        assert_eq!(rects[0].origin, Point::new(0.0 * cell_width, 0.0));
        assert_eq!(rects[0].size, Size::new(cell_width, cell_height));
        assert_eq!(rects[1].origin, Point::new(1.0 * cell_width, 0.0));
        assert_eq!(rects[1].size, Size::new(cell_width, cell_height));
    }

    #[test]
    fn grid_layout_four_windows() {
        let screen = ScreenInfo::new(1920.0, 1080.0);
        let infos: Vec<WindowInfo> = (0..4).map(|i| WindowInfo { id: WindowId::new_v4(), title: format!("W{}", i), rect: Default::default(), state: WindowState::Tiled }).collect();
        let window_refs: Vec<&WindowInfo> = infos.iter().collect();
        let rects = WindowManager::calculate_grid_layout(&screen, &window_refs).unwrap();
        assert_eq!(rects.len(), 4);
        let cell_width = 1920.0 / 2.0;
        let cell_height = 1080.0 / 2.0;
        assert_eq!(rects[0].origin, Point::new(0.0 * cell_width, 0.0 * cell_height));
        assert_eq!(rects[0].size, Size::new(cell_width, cell_height));
        assert_eq!(rects[1].origin, Point::new(1.0 * cell_width, 0.0 * cell_height));
        assert_eq!(rects[1].size, Size::new(cell_width, cell_height));
        assert_eq!(rects[2].origin, Point::new(0.0 * cell_width, 1.0 * cell_height));
        assert_eq!(rects[2].size, Size::new(cell_width, cell_height));
        assert_eq!(rects[3].origin, Point::new(1.0 * cell_width, 1.0 * cell_height));
        assert_eq!(rects[3].size, Size::new(cell_width, cell_height));
    }

    #[test]
    fn grid_layout_three_windows() {
        let screen = ScreenInfo::new(1920.0, 1080.0);
        let infos: Vec<WindowInfo> = (0..3).map(|i| WindowInfo {id: WindowId::new_v4(), title: format!("W{}", i), rect: Default::default(), state: WindowState::Tiled }).collect();
        let window_refs: Vec<&WindowInfo> = infos.iter().collect();
        let rects = WindowManager::calculate_grid_layout(&screen, &window_refs).unwrap();
        assert_eq!(rects.len(), 3);
        let cell_width = 1920.0 / 2.0;
        let cell_height = 1080.0 / 2.0;
        assert_eq!(rects[0].origin, Point::new(0.0 * cell_width, 0.0 * cell_height));
        assert_eq!(rects[0].size, Size::new(cell_width, cell_height));
        assert_eq!(rects[1].origin, Point::new(1.0 * cell_width, 0.0 * cell_height));
        assert_eq!(rects[1].size, Size::new(cell_width, cell_height));
        assert_eq!(rects[2].origin, Point::new(0.0 * cell_width, 1.0 * cell_height));
        assert_eq!(rects[2].size, Size::new(cell_width, cell_height));
    }
}
