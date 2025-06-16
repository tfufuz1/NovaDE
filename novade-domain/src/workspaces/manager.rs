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
        tracing::info!("Created workspace with ID {:?} on monitor {:?}.", new_ws_id, self.get_workspace_by_id(new_ws_id).unwrap().monitor_id);
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
        // if let Some(ws_id) = self.window_to_workspace_map.get(&window_id) { // Key is core::WindowId
        //     return Some(*ws_id);
        // }
        // Fallback by iterating through workspaces if map is not used or out of sync
        for ws in &self.workspaces {
            if ws.windows.contains(&window_id) { // window_id is core::WindowId
                return Some(ws.id);
            }
        }
        None
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
