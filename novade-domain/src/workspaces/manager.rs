// Copyright 2024 NovaDE Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Manages workspaces, their layouts, and the windows within them.

use std::sync::Arc;
use std::collections::HashMap;
use thiserror::Error;

use crate::workspaces::core::WindowId;
use crate::workspaces::traits::WindowManager; // Use the trait from the new traits.rs file


/// Unique identifier for a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceId(u32);

impl WorkspaceId {
    /// Creates a new `WorkspaceId`.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Gets the underlying `u32` value.
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for WorkspaceId {
    fn default() -> Self {
        Self(1) // Default workspace ID
    }
}

use crate::workspaces::tiling::TilingOptions; // Import TilingOptions
use novade_core::types::geometry::Rect; // For Rect in apply_layout

/// Defines the layout strategy for a workspace.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceLayout {
    /// Windows are freely placed and sized.
    Floating,
    /// Windows are arranged by a tiling algorithm, with specific options.
    Tiling(TilingOptions),
}

impl Default for WorkspaceLayout {
    fn default() -> Self {
        WorkspaceLayout::Floating
    }
}

// TilingLayout enum (MasterStack, Spiral) is now TilingOptions in tiling.rs

/// Represents a single workspace.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Unique identifier for the workspace.
    pub id: WorkspaceId,
    /// User-friendly name for the workspace.
    pub name: String,
    /// List of window IDs belonging to this workspace.
    pub windows: Vec<WindowId>,
    /// Whether this workspace is currently active (visible).
    pub active: bool,
    /// Layout strategy for this workspace.
    pub layout: WorkspaceLayout,
    /// ID of the monitor this workspace is primarily associated with.
    /// `None` could mean it's a global/unassigned workspace or currently not displayed.
    pub monitor_id: Option<String>,
}

impl Workspace {
    /// Creates a new workspace.
    pub fn new(id: WorkspaceId, name: String, monitor_id: Option<String>) -> Self {
        Self {
            id,
            name,
            windows: Vec::new(),
            active: false,
            layout: WorkspaceLayout::default(),
            monitor_id,
        }
    }

    /// Applies the current layout to the windows in this workspace.
    ///
    /// # Arguments
    /// * `screen_area`: The available `Rect` for arranging windows.
    ///
    /// # Returns
    /// A `HashMap` mapping `WindowId` to its calculated `Rect` geometry.
    /// Returns an empty map if layout is Floating and no window positions are stored (current behavior).
    pub fn apply_layout(&self, screen_area: Rect) -> HashMap<WindowId, Rect> {
        match &self.layout {
            WorkspaceLayout::Floating => {
                // ANCHOR: Floating layout needs to retrieve actual window positions.
                // For now, returns an empty map, implying positions are managed elsewhere directly by WindowManager
                // or should be stored per window within the workspace for floating mode.
                HashMap::new()
            }
            WorkspaceLayout::Tiling(tiling_options) => {
                let algorithm = tiling_options.as_algorithm();
                // ANCHOR: Ensure self.windows are sorted in a meaningful way if algorithm depends on order.
                // E.g., by insertion time, or by some user-defined order.
                algorithm.arrange(&self.windows, screen_area)
            }
        }
    }
}

/// Errors related to workspace management.
#[derive(Debug, Error, PartialEq)]
pub enum WorkspaceError {
    #[error("Workspace with ID {0:?} not found.")]
    NotFound(WorkspaceId),
    #[error("Workspace with ID {0:?} is not empty and cannot be removed.")]
    NotEmpty(WorkspaceId),
    #[error("Window with ID {0:?} not found in workspace {1:?}.")]
    WindowNotFoundInWorkspace(WindowId, WorkspaceId),
    #[error("Window with ID {0:?} already exists in workspace {1:?}.")]
    WindowAlreadyExists(WindowId, WorkspaceId),
    #[error("Cannot remove the last workspace.")]
    CannotRemoveLastWorkspace,
    // ANCHOR: Add more specific errors as needed
}

/// Manages multiple workspaces and their interactions with windows.
#[derive(Debug)]
pub struct WorkspaceManager {
    workspaces: Vec<Workspace>,
    active_workspace_id: Option<WorkspaceId>,
    next_workspace_id_counter: u32,
    window_manager: Arc<dyn WindowManager>,
    // ANCHOR: Window-to-workspace mapping for quick lookup, if needed.
    // window_to_workspace_map: HashMap<WindowId, WorkspaceId>, // Uses core::WindowId as key
}

impl WorkspaceManager {
    /// Creates a new `WorkspaceManager`.
    ///
    /// A default workspace named "Workspace 1" is created, assigned to the primary monitor (if any),
    /// and set as active.
    ///
    /// # Arguments
    ///
    /// * `window_manager` - A reference to the window manager for interacting with windows.
    pub fn new(window_manager: Arc<dyn WindowManager>) -> Self {
        // ANCHOR: Async call in sync constructor: `get_primary_output_id` is async.
        // Using block_on here for simplicity during initialization.
        // In a fully async setup, `WorkspaceManager::new` might need to be async,
        // or monitor info provided externally.
        let primary_monitor_id = futures::executor::block_on(window_manager.get_primary_output_id())
            .map_err(|e| tracing::warn!("Failed to get primary output ID during WM init: {}", e))
            .ok().flatten();

        let mut manager = Self {
            workspaces: Vec::new(),
            active_workspace_id: None,
            next_workspace_id_counter: 1, // Start IDs from 1
            window_manager,
            // window_to_workspace_map: HashMap::new(), // Key would be core::WindowId
        };

        // Create a default workspace, attempting to assign it to the primary monitor.
        let default_ws_name = "Workspace 1".to_string();
        let default_ws_id = manager.create_workspace_internal(default_ws_name, primary_monitor_id.clone());

        manager.active_workspace_id = Some(default_ws_id);
        if let Some(ws) = manager.get_mut_workspace_by_id_internal(default_ws_id) {
            ws.active = true;
            // ws.monitor_id is already set by create_workspace_internal
            tracing::info!("Default workspace {:?} created on monitor {:?}.", ws.id, ws.monitor_id);
        }
        manager
    }

    /// Internal helper to create a workspace and assign it a monitor.
    fn create_workspace_internal(&mut self, name: String, preferred_monitor_id: Option<String>) -> WorkspaceId {
        let id = WorkspaceId::new(self.next_workspace_id_counter);
        self.next_workspace_id_counter += 1;

        let final_monitor_id = preferred_monitor_id.or_else(|| {
            // ANCHOR: Async call in sync context (if preferred_monitor_id is None).
            // Fallback: try focused, then primary, then None.
            // This demonstrates more complex fallback logic.
            futures::executor::block_on(async {
                self.window_manager.get_focused_output_id().await
                    .unwrap_or(None) // unwrap_or from DomainResult
                    .or_else(|| {
                        futures::executor::block_on(self.window_manager.get_primary_output_id())
                            .unwrap_or(None)
                    })
            })
        });

        let workspace = Workspace::new(id, name, final_monitor_id);
        self.workspaces.push(workspace);
        id
    }

    /// Creates a new workspace with the given name, optionally on a specific monitor.
    /// If `monitor_id` is None, assigns according to internal logic (e.g., focused or primary monitor).
    pub fn create_workspace(&mut self, name: String, monitor_id: Option<String>) -> WorkspaceId {
        let new_ws_id = self.create_workspace_internal(name, monitor_id);
        // Log is now inside create_workspace_internal or should be done by caller if needed
        new_ws_id
    }

    /// Creates a new workspace specifically for a given monitor, using a default name if none provided.
    /// This is a convenience method often called when a window is moved to a monitor without an existing workspace.
    pub fn create_workspace_on_monitor(&mut self, monitor_id: String, name: Option<String>) -> WorkspaceId {
        // Changed Result<(), WorkspaceError> to WorkspaceId for create_workspace
        let ws_name = name.unwrap_or_else(|| {
            let mut default_name_base = format!("Workspace M{}", monitor_id);
            // Ensure name is unique if just using default
            let mut counter = 1;
            let mut final_name = default_name_base.clone();
            while self.workspaces.iter().any(|ws| ws.name == final_name) {
                final_name = format!("{} ({})", default_name_base, counter);
                counter += 1;
            }
            final_name
        });
        let new_ws_id = self.create_workspace_internal(ws_name, Some(monitor_id));
        tracing::info!("Created workspace {:?} on monitor {:?} (helper).", new_ws_id, self.get_workspace_by_id(new_ws_id).unwrap().monitor_id);
        new_ws_id
    }

    /// Removes a workspace by its ID.
    ///
    /// ANCHOR: Current implementation only allows removing empty workspaces.
    /// Future enhancements should handle moving windows to a default workspace.
    pub fn remove_workspace(&mut self, id: WorkspaceId) -> Result<(), WorkspaceError> {
        if self.workspaces.len() <= 1 {
            return Err(WorkspaceError::CannotRemoveLastWorkspace);
        }

        let ws_index = self.workspaces.iter().position(|ws| ws.id == id)
            .ok_or(WorkspaceError::NotFound(id))?;

        if !self.workspaces[ws_index].windows.is_empty() {
            // ANCHOR: For now, error out. Later, move windows to another workspace.
            return Err(WorkspaceError::NotEmpty(id));
        }

        self.workspaces.remove(ws_index);

        // If the active workspace was removed, switch to the first available one.
        if self.active_workspace_id == Some(id) {
            self.active_workspace_id = self.workspaces.first().map(|ws| ws.id);
            if let Some(new_active_id) = self.active_workspace_id {
                if let Some(new_active_ws) = self.get_mut_workspace_by_id_internal(new_active_id) {
                    new_active_ws.active = true;
                    // ANCHOR: Trigger window visibility changes via WindowManager for the new active workspace.
                }
            }
        }
        Ok(())
    }

    /// Switches the active workspace.
    ///
    /// This involves deactivating the old workspace and activating the new one.
    /// The actual window visibility changes (show/hide) are handled by the caller (e.g., `DesktopState`)
    /// using information from this method and the `WindowManager`.
    pub async fn switch_workspace(&mut self, new_active_id: WorkspaceId) -> Result<(), WorkspaceError> {
        if self.active_workspace_id == Some(new_active_id) {
            return Ok(()); // Already active
        }

        let target_ws_exists = self.workspaces.iter().any(|ws| ws.id == new_active_id);
        if !target_ws_exists {
            return Err(WorkspaceError::NotFound(new_active_id));
        }

        // Deactivate the current active workspace
        if let Some(old_id) = self.active_workspace_id {
            if let Some(old_ws) = self.get_mut_workspace_by_id_internal(old_id) {
                old_ws.active = false;
            }
        }

        // Activate the new workspace
        self.active_workspace_id = Some(new_active_id);
        if let Some(new_ws) = self.get_mut_workspace_by_id_internal(new_active_id) {
            new_ws.active = true;
            // ANCHOR: The caller (DesktopState) will use new_ws.monitor_id to get the correct
            // screen_area for applying layout and use new_ws.windows to show them.
            tracing::info!("Workspace {:?} on monitor {:?} is now active.", new_ws.id, new_ws.monitor_id);
        }

        // ANCHOR: Notify other parts of the system about workspace change (e.g., UI panels via events).
        Ok(())
    }

    /// Adds a window to the specified workspace.
    pub fn add_window_to_workspace(&mut self, window_id: WindowId, workspace_id: WorkspaceId) -> Result<(), WorkspaceError> {
        let workspace = self.get_mut_workspace_by_id_internal(workspace_id)
            .ok_or(WorkspaceError::NotFound(workspace_id))?;

        if workspace.windows.contains(&window_id) {
            return Err(WorkspaceError::WindowAlreadyExists(window_id, workspace_id));
        }
        workspace.windows.push(window_id);
        // self.window_to_workspace_map.insert(window_id, workspace_id);

        // ANCHOR: If the window is added to the currently active workspace, ensure it's shown.
        // if workspace.active {
        //     self.window_manager.show_window(window_id).await?;
        // } else {
        //     self.window_manager.hide_window(window_id).await?; // Ensure hidden if added to inactive
        // }
        Ok(())
    }

    /// Removes a window from a specific workspace.
    pub fn remove_window_from_workspace(&mut self, window_id: WindowId, workspace_id: WorkspaceId) -> Result<(), WorkspaceError> {
        let workspace = self.get_mut_workspace_by_id_internal(workspace_id)
            .ok_or(WorkspaceError::NotFound(workspace_id))?;

        if let Some(pos) = workspace.windows.iter().position(|&id| id == window_id) { // window_id is core::WindowId
            workspace.windows.remove(pos);
            // self.window_to_workspace_map.remove(&window_id); // Key is core::WindowId
            Ok(())
        } else {
            Err(WorkspaceError::WindowNotFoundInWorkspace(window_id, workspace_id))
        }
    }

    /// Moves a window to a different workspace.
    /// ANCHOR: This method might be more complex, involving removing from old and adding to new.
    pub fn move_window_to_workspace(&mut self, window_id: WindowId, target_workspace_id: WorkspaceId) -> Result<(), WorkspaceError> {
        let current_workspace_id = self.find_workspace_for_window(window_id);

        if let Some(id) = current_workspace_id {
            if id == target_workspace_id { return Ok(()); } // Already in the target workspace
            self.remove_window_from_workspace(window_id, id)?;
        }

        self.add_window_to_workspace(window_id, target_workspace_id)?;
        // ANCHOR: Handle window visibility based on active status of old and new workspaces.
        Ok(())
    }

    /// Finds the workspace ID that a given window belongs to.
    pub fn find_workspace_for_window(&self, window_id: WindowId) -> Option<WorkspaceId> { // window_id is core::WindowId
        for ws in &self.workspaces {
            if ws.windows.contains(&window_id) {
                return Some(ws.id);
            }
        }
        None
    }

    /// Moves a window to a specified target monitor.
    ///
    /// This involves:
    /// 1. Removing the window from its current workspace.
    /// 2. Finding or creating a workspace on the target monitor.
    /// 3. Adding the window to this target workspace.
    ///
    /// Returns the ID of the target workspace.
    /// ANCHOR: This method is currently synchronous and uses `block_on` for async WindowManager calls.
    /// This should be refactored to be fully async in a production environment.
    pub fn move_window_to_monitor(&mut self, window_id: WindowId, target_monitor_id: String) -> Result<WorkspaceId, WorkspaceError> {
        let source_workspace_id = self.find_workspace_for_window(window_id);

        // Remove from source workspace if it's in one
        if let Some(sws_id) = source_workspace_id {
            let source_ws = self.get_mut_workspace_by_id_internal(sws_id)
                .ok_or(WorkspaceError::NotFound(sws_id))?; // Should not happen if find_workspace_for_window worked
            source_ws.windows.retain(|&id| id != window_id);
            tracing::info!("Window {:?} removed from source workspace {:?}", window_id, sws_id);
        } else {
            tracing::warn!("Window {:?} not found in any workspace, cannot remove from source.", window_id);
        }

        // Find or create workspace on target monitor
        let target_workspace_id = self.workspaces.iter()
            .find(|ws| ws.monitor_id.as_ref() == Some(&target_monitor_id))
            .map(|ws| ws.id)
            .unwrap_or_else(|| {
                self.create_workspace_on_monitor(target_monitor_id.clone(), None)
            });

        // Add to target workspace
        let target_ws = self.get_mut_workspace_by_id_internal(target_workspace_id)
            .ok_or(WorkspaceError::NotFound(target_workspace_id))?; // Should exist as we just found/created it

        if !target_ws.windows.contains(&window_id) {
            target_ws.windows.push(window_id);
            tracing::info!("Window {:?} added to target workspace {:?} on monitor {}", window_id, target_workspace_id, target_monitor_id);
        } else {
            tracing::info!("Window {:?} already in target workspace {:?}", window_id, target_workspace_id);
        }

        Ok(target_workspace_id)
    }


    /// Returns an immutable reference to the active workspace, if any.
    pub fn get_active_workspace(&self) -> Option<&Workspace> {
        match self.active_workspace_id {
            Some(id) => self.get_workspace_by_id_internal(id),
            None => None,
        }
    }

    /// Returns a mutable reference to the active workspace, if any.
    pub fn get_mut_active_workspace(&mut self) -> Option<&mut Workspace> {
        match self.active_workspace_id {
            Some(id) => self.get_mut_workspace_by_id_internal(id),
            None => None,
        }
    }

    /// Returns an immutable reference to a workspace by its ID.
    pub fn get_workspace_by_id(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.get_workspace_by_id_internal(id)
    }

    /// Returns a mutable reference to a workspace by its ID.
    pub fn get_mut_workspace_by_id(&mut self, id: WorkspaceId) -> Option<&mut Workspace> {
        self.get_mut_workspace_by_id_internal(id)
    }

    /// Internal helper to get an immutable workspace.
    fn get_workspace_by_id_internal(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.workspaces.iter().find(|ws| ws.id == id)
    }

    /// Internal helper to get a mutable workspace.
    fn get_mut_workspace_by_id_internal(&mut self, id: WorkspaceId) -> Option<&mut Workspace> {
        self.workspaces.iter_mut().find(|ws| ws.id == id)
    }

    /// Returns a list of all workspaces.
    pub fn get_all_workspaces(&self) -> &Vec<Workspace> {
        &self.workspaces
    }

    /// Returns the ID of the active workspace.
    pub fn get_active_workspace_id(&self) -> Option<WorkspaceId> {
        self.active_workspace_id
    }
}

// Removed local placeholder `mod traits` as it's now imported from `crate::workspaces::traits`.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspaces::core::{Window, WindowId, WindowState, WindowType};
    use crate::workspaces::traits::{WindowManager as DomainWindowManager, DomainResult};
    use novade_core::types::geometry::{Point, Size, Rect};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex as StdMutex};

    // Mock WindowManager for testing WorkspaceManager
    #[derive(Debug, Default)]
    struct MockWindowManager {
        // Store calls or state if needed for assertions
        focused_output_id: Option<String>,
        primary_output_id: Option<String>,
        output_work_areas: HashMap<String, Rect>,
    }

    impl MockWindowManager {
        fn new() -> Self {
            let mut m = Self::default();
            // Setup a default primary monitor for tests
            let primary_id = "test-monitor-1".to_string();
            m.primary_output_id = Some(primary_id.clone());
            m.focused_output_id = Some(primary_id.clone());
            m.output_work_areas.insert(primary_id.clone(), Rect::new(Point::new(0,0), Size::new(1920,1080)));
            m
        }
    }

    #[async_trait]
    impl DomainWindowManager for MockWindowManager {
        async fn get_windows(&self) -> DomainResult<Vec<Window>> { Ok(vec![]) }
        async fn get_window(&self, _id: WindowId) -> DomainResult<Window> { Err("Not implemented for mock".to_string()) }
        async fn focus_window(&self, _id: WindowId) -> DomainResult<()> { Ok(()) }
        async fn move_window(&self, _id: WindowId, _position: Point) -> DomainResult<()> { Ok(()) }
        async fn resize_window(&self, _id: WindowId, _size: Size) -> DomainResult<()> { Ok(()) }
        async fn set_window_state(&self, _id: WindowId, _state: WindowState) -> DomainResult<()> { Ok(()) }
        async fn close_window(&self, _id: WindowId) -> DomainResult<()> { Ok(()) }
        async fn hide_window_for_workspace(&self, _id: WindowId) -> DomainResult<()> { Ok(()) }
        async fn show_window_for_workspace(&self, _id: WindowId) -> DomainResult<()> { Ok(()) }

        async fn get_primary_output_id(&self) -> DomainResult<Option<String>> { Ok(self.primary_output_id.clone()) }
        async fn get_output_work_area(&self, output_id: &str) -> DomainResult<Rect> {
            self.output_work_areas.get(output_id).cloned()
                .ok_or_else(|| "Output not found in mock".to_string())
        }
        async fn get_focused_output_id(&self) -> DomainResult<Option<String>> { Ok(self.focused_output_id.clone()) }
    }

    fn create_mock_wm_arc() -> Arc<dyn DomainWindowManager> {
        Arc::new(MockWindowManager::new())
    }

    #[test]
    fn test_new_workspace_manager() {
        let wm_mock = create_mock_wm_arc();
        let manager = WorkspaceManager::new(wm_mock);
        assert_eq!(manager.workspaces.len(), 1, "Should create one default workspace");
        let default_ws = &manager.workspaces[0];
        assert_eq!(default_ws.name, "Workspace 1");
        assert!(default_ws.active, "Default workspace should be active");
        assert_eq!(manager.active_workspace_id, Some(default_ws.id));
        // Default workspace should be on primary monitor if available
        assert_eq!(default_ws.monitor_id, Some("test-monitor-1".to_string()));
    }

    #[test]
    fn test_create_workspace() {
        let wm_mock = create_mock_wm_arc();
        let mut manager = WorkspaceManager::new(wm_mock);
        let initial_ws_count = manager.workspaces.len();

        let ws2_id = manager.create_workspace("Workspace 2".to_string(), Some("test-monitor-2".to_string()));
        assert_eq!(manager.workspaces.len(), initial_ws_count + 1);
        let ws2 = manager.get_workspace_by_id(ws2_id).unwrap();
        assert_eq!(ws2.name, "Workspace 2");
        assert!(!ws2.active); // New workspaces are not active by default
        assert_eq!(ws2.monitor_id, Some("test-monitor-2".to_string()));

        // Test creating workspace without specifying monitor (should use default logic)
        let ws3_id = manager.create_workspace("Workspace 3".to_string(), None);
        let ws3 = manager.get_workspace_by_id(ws3_id).unwrap();
        assert_eq!(ws3.monitor_id, Some("test-monitor-1".to_string())); // Mock defaults to primary
    }

    #[test]
    fn test_remove_workspace() {
        let wm_mock = create_mock_wm_arc();
        let mut manager = WorkspaceManager::new(wm_mock);
        let ws2_id = manager.create_workspace("Workspace 2".to_string(), None);

        // Remove ws2 (it's empty)
        assert!(manager.remove_workspace(ws2_id).is_ok());
        assert!(manager.get_workspace_by_id(ws2_id).is_none());
        assert_eq!(manager.workspaces.len(), 1); // Back to default

        // Try to remove last workspace
        let default_ws_id = manager.workspaces[0].id;
        assert_eq!(manager.remove_workspace(default_ws_id), Err(WorkspaceError::CannotRemoveLastWorkspace));

        // Try to remove non-empty workspace
        let ws3_id = manager.create_workspace("Workspace 3".to_string(), None);
        let window_id = WindowId::new();
        manager.add_window_to_workspace(window_id, ws3_id).unwrap();
        assert_eq!(manager.remove_workspace(ws3_id), Err(WorkspaceError::NotEmpty(ws3_id)));
    }

    #[tokio::test] // switch_workspace is async
    async fn test_switch_workspace() {
        let wm_mock = create_mock_wm_arc();
        let mut manager = WorkspaceManager::new(wm_mock);
        let ws1_id = manager.get_active_workspace_id().unwrap();
        let ws2_id = manager.create_workspace("Workspace 2".to_string(), None);

        assert_ne!(manager.active_workspace_id, Some(ws2_id));
        manager.switch_workspace(ws2_id).await.unwrap();
        assert_eq!(manager.active_workspace_id, Some(ws2_id));
        assert!(manager.get_workspace_by_id(ws2_id).unwrap().active);
        assert!(!manager.get_workspace_by_id(ws1_id).unwrap().active);

        // Switch to non-existent
        let non_existent_id = WorkspaceId::new(999);
        assert!(manager.switch_workspace(non_existent_id).await.is_err());
        assert_eq!(manager.active_workspace_id, Some(ws2_id)); // Should not change
    }

    #[test]
    fn test_window_management_in_workspaces() {
        let wm_mock = create_mock_wm_arc();
        let mut manager = WorkspaceManager::new(wm_mock);
        let ws1_id = manager.get_active_workspace_id().unwrap();
        let ws2_id = manager.create_workspace("Workspace 2".to_string(), None);
        let window_a = WindowId::new();
        let window_b = WindowId::new();

        // Add windows
        manager.add_window_to_workspace(window_a, ws1_id).unwrap();
        manager.add_window_to_workspace(window_b, ws2_id).unwrap();
        assert!(manager.get_workspace_by_id(ws1_id).unwrap().windows.contains(&window_a));
        assert!(manager.get_workspace_by_id(ws2_id).unwrap().windows.contains(&window_b));
        assert_eq!(manager.find_workspace_for_window(window_a), Some(ws1_id));
        assert_eq!(manager.find_workspace_for_window(window_b), Some(ws2_id));

        // Try adding existing window
        assert_eq!(manager.add_window_to_workspace(window_a, ws1_id), Err(WorkspaceError::WindowAlreadyExists(window_a, ws1_id)));

        // Move window
        manager.move_window_to_workspace(window_a, ws2_id).unwrap();
        assert!(!manager.get_workspace_by_id(ws1_id).unwrap().windows.contains(&window_a));
        assert!(manager.get_workspace_by_id(ws2_id).unwrap().windows.contains(&window_a));
        assert_eq!(manager.find_workspace_for_window(window_a), Some(ws2_id));

        // Remove window
        manager.remove_window_from_workspace(window_a, ws2_id).unwrap();
        assert!(!manager.get_workspace_by_id(ws2_id).unwrap().windows.contains(&window_a));
        assert_eq!(manager.find_workspace_for_window(window_a), None);

        // Try removing non-existent window
        assert_eq!(manager.remove_window_from_workspace(window_a, ws2_id), Err(WorkspaceError::WindowNotFoundInWorkspace(window_a, ws2_id)));
    }

    #[test]
    fn test_create_workspace_on_monitor() {
        let wm_mock = create_mock_wm_arc();
        let mut manager = WorkspaceManager::new(wm_mock); // Default ws on "test-monitor-1"

        let monitor2_id_str = "test-monitor-2".to_string();
        let ws_on_mon2_id = manager.create_workspace_on_monitor(monitor2_id_str.clone(), Some("CAD Apps".to_string()));
        let ws_on_mon2 = manager.get_workspace_by_id(ws_on_mon2_id).unwrap();
        assert_eq!(ws_on_mon2.monitor_id, Some(monitor2_id_str.clone()));
        assert_eq!(ws_on_mon2.name, "CAD Apps");

        // Test default naming
        let ws_on_mon2_default_name_id = manager.create_workspace_on_monitor(monitor2_id_str.clone(), None);
        let ws_on_mon2_default_name = manager.get_workspace_by_id(ws_on_mon2_default_name_id).unwrap();
        assert_eq!(ws_on_mon2_default_name.monitor_id, Some(monitor2_id_str.clone()));
        assert_eq!(ws_on_mon2_default_name.name, format!("Workspace M{}", monitor2_id_str));

        // Test unique default naming
        let ws_on_mon2_default_name2_id = manager.create_workspace_on_monitor(monitor2_id_str.clone(), None);
        let ws_on_mon2_default_name2 = manager.get_workspace_by_id(ws_on_mon2_default_name2_id).unwrap();
        assert_eq!(ws_on_mon2_default_name2.name, format!("Workspace M{} (1)", monitor2_id_str));
    }

    #[test]
    fn test_move_window_to_monitor() {
        let mut mock_wm = MockWindowManager::new();
        let monitor1_id = "monitor1".to_string();
        let monitor2_id = "monitor2".to_string();
        mock_wm.output_work_areas.insert(monitor1_id.clone(), Rect::new(Point::new(0,0), Size::new(1920,1080)));
        mock_wm.output_work_areas.insert(monitor2_id.clone(), Rect::new(Point::new(1920,0), Size::new(1920,1080)));
        mock_wm.primary_output_id = Some(monitor1_id.clone());
        mock_wm.focused_output_id = Some(monitor1_id.clone());

        let mut manager = WorkspaceManager::new(Arc::new(mock_wm));
        let window_id = WindowId::new();

        // Initial workspace (ws1) should be on monitor1
        let ws1_id = manager.get_active_workspace_id().unwrap();
        assert_eq!(manager.get_workspace_by_id(ws1_id).unwrap().monitor_id, Some(monitor1_id.clone()));
        manager.add_window_to_workspace(window_id, ws1_id).unwrap();
        assert!(manager.get_workspace_by_id(ws1_id).unwrap().windows.contains(&window_id));

        // Move window to monitor2
        let target_ws_id_on_mon2 = manager.move_window_to_monitor(window_id, monitor2_id.clone()).unwrap();
        let ws_on_mon2 = manager.get_workspace_by_id(target_ws_id_on_mon2).unwrap();
        assert_eq!(ws_on_mon2.monitor_id, Some(monitor2_id.clone()));
        assert!(ws_on_mon2.windows.contains(&window_id));
        assert!(!manager.get_workspace_by_id(ws1_id).unwrap().windows.contains(&window_id)); // Should be removed from old ws

        // Check if a new workspace was created or an existing one on monitor2 was used
        // In this test, it should create a new one.
        assert_ne!(ws1_id, target_ws_id_on_mon2);
    }

    // Tests for Workspace::apply_layout
    #[test]
    fn test_workspace_apply_floating_layout() {
        let initial_monitor_id = Some("monitor-float".to_string());
        let ws = Workspace::new(WorkspaceId::new(1), "Floating WS".to_string(), initial_monitor_id);
        // Add some windows, though their current positions are not tracked by Workspace for Floating mode yet
        // ws.windows.push(WindowId::new());

        let screen_area = Rect::new(Point::new(0,0), Size::new(1920,1080));
        let geometries = ws.apply_layout(screen_area);

        // Current behavior for Floating is to return empty map, as positions are externally managed.
        assert!(geometries.is_empty(), "Floating layout should return empty map currently");
        // ANCHOR: When floating window positions are stored in Workspace, this test should check they are returned.
    }

    #[test]
    fn test_workspace_apply_tiling_layout_master_stack() {
        let monitor_id = Some("monitor-tile".to_string());
        let master_stack_options = crate::workspaces::tiling::MasterStackLayout {
            num_master: 1,
            master_width_percentage: 0.5,
        };
        let layout = WorkspaceLayout::Tiling(crate::workspaces::tiling::TilingOptions::MasterStack(master_stack_options));

        let mut ws = Workspace::new(WorkspaceId::new(1), "Tiling WS".to_string(), monitor_id);
        let win_ids = (0..3).map(|i| WindowId::from_string(&format!("win{}", i + 1))).collect::<Vec<_>>();
        ws.windows = win_ids.clone();
        ws.layout = layout;

        let screen_area = Rect::new(Point::new(0,0), Size::new(1000,600));
        let geometries = ws.apply_layout(screen_area);

        assert_eq!(geometries.len(), 3);
        assert!(geometries.contains_key(&win_ids[0]));
        assert!(geometries.contains_key(&win_ids[1]));
        assert!(geometries.contains_key(&win_ids[2]));

        // Check master window (win_ids[0])
        let master_geom = geometries[&win_ids[0]];
        assert_eq!(master_geom.position.x, 0);
        assert_eq!(master_geom.position.y, 0);
        assert_eq!(master_geom.size.width, 500); // 50% of 1000
        assert_eq!(master_geom.size.height, 600); // Full height

        // Check stack windows (win_ids[1], win_ids[2])
        let stack1_geom = geometries[&win_ids[1]];
        assert_eq!(stack1_geom.position.x, 500);
        assert_eq!(stack1_geom.position.y, 0);
        assert_eq!(stack1_geom.size.width, 500);
        assert_eq!(stack1_geom.size.height, 300); // 600 / 2

        let stack2_geom = geometries[&win_ids[2]];
        assert_eq!(stack2_geom.position.x, 500);
        assert_eq!(stack2_geom.position.y, 300);
        assert_eq!(stack2_geom.size.width, 500);
        assert_eq!(stack2_geom.size.height, 300);
    }
}
