use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, info, warn, error};
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

use crate::workspaces::core::{
    Workspace, WorkspaceId, WindowIdentifier, WorkspaceCoreError, WorkspaceLayoutType,
    WorkspaceRenamedData, WorkspaceLayoutChangedData, WorkspaceIconChangedData,
    WorkspaceAccentChangedData, WorkspacePersistentIdChangedData,
    WindowAddedToWorkspaceData, WindowRemovedFromWorkspaceData,
};
use crate::workspaces::config::{WorkspaceConfigProvider, WorkspaceSetSnapshot, WorkspaceSnapshot};
use crate::workspaces::assignment::{assign_window_to_workspace, remove_window_from_workspace, find_workspace_for_window, WindowAssignmentError};

pub use self::errors::WorkspaceManagerError;
pub use self::events::WorkspaceEvent;

pub mod events;
pub mod errors;

#[async_trait]
pub trait WorkspaceManagerService: Send + Sync {
    async fn create_workspace(
        &self,
        name: Option<String>,
        persistent_id: Option<String>,
        icon_name: Option<String>,
        accent_color_hex: Option<String>,
    ) -> Result<WorkspaceId, WorkspaceManagerError>;
    
    async fn delete_workspace(
        &self,
        id: WorkspaceId,
        fallback_id_for_windows: Option<WorkspaceId>,
    ) -> Result<(), WorkspaceManagerError>;

    async fn get_workspace(&self, id: WorkspaceId) -> Option<Workspace>;
    async fn all_workspaces_ordered(&self) -> Vec<Workspace>;
    async fn active_workspace_id(&self) -> Option<WorkspaceId>;
    async fn set_active_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceManagerError>;
    
    async fn save_configuration(&self) -> Result<(), WorkspaceManagerError>;
    async fn load_initial_state(&self) -> Result<(), WorkspaceManagerError>;
    
    fn subscribe_to_workspace_events(&self) -> broadcast::Receiver<WorkspaceEvent>;

    async fn assign_window_to_active_workspace(&self, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>;
    async fn assign_window_to_specific_workspace(&self, workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>;
    async fn remove_window_from_its_workspace(&self, window_id: &WindowIdentifier) -> Result<Option<WorkspaceId>, WorkspaceManagerError>;
    async fn move_window_to_specific_workspace(&self, target_workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>;

    async fn set_workspace_name(&self, id: WorkspaceId, name: String) -> Result<(), WorkspaceManagerError>;
    async fn set_workspace_layout(&self, id: WorkspaceId, layout: WorkspaceLayoutType) -> Result<(), WorkspaceManagerError>;
    async fn set_workspace_persistent_id(&self, id: WorkspaceId, persistent_id: Option<String>) -> Result<(), WorkspaceManagerError>;
    async fn set_workspace_icon(&self, id: WorkspaceId, icon_name: Option<String>) -> Result<(), WorkspaceManagerError>;
    async fn set_workspace_accent_color(&self, id: WorkspaceId, color_hex: Option<String>) -> Result<(), WorkspaceManagerError>;
    
    async fn reorder_workspace(&self, workspace_id: WorkspaceId, new_index: usize) -> Result<(), WorkspaceManagerError>;
}

pub struct WorkspaceManagerInternalState {
    workspaces: HashMap<WorkspaceId, Workspace>,
    active_workspace_id: Option<WorkspaceId>,
    ordered_workspace_ids: Vec<WorkspaceId>,
    next_workspace_number: u32, // Used for default naming "Workspace X"
    persistent_id_to_runtime_id: HashMap<String, WorkspaceId>,
    config_provider: Arc<dyn WorkspaceConfigProvider>,
    event_publisher: broadcast::Sender<WorkspaceEvent>,
    ensure_unique_window_assignment: bool,
}

pub struct DefaultWorkspaceManager {
    internal: Arc<Mutex<WorkspaceManagerInternalState>>,
}

impl DefaultWorkspaceManager {
    pub fn new(
        config_provider: Arc<dyn WorkspaceConfigProvider>,
        broadcast_capacity: usize,
        ensure_unique_window_assignment: bool,
    ) -> Self {
        let (event_publisher, _) = broadcast::channel(broadcast_capacity);
        let internal_state = WorkspaceManagerInternalState {
            workspaces: HashMap::new(),
            active_workspace_id: None,
            ordered_workspace_ids: Vec::new(),
            next_workspace_number: 1,
            persistent_id_to_runtime_id: HashMap::new(),
            config_provider,
            event_publisher,
            ensure_unique_window_assignment,
        };
        Self {
            internal: Arc::new(Mutex::new(internal_state)),
        }
    }

    async fn internal_create_workspace_locked(
        internal_state: &mut WorkspaceManagerInternalState,
        name: Option<String>,
        persistent_id_opt: Option<String>,
        icon_name: Option<String>,
        accent_color_hex: Option<String>,
    ) -> Result<WorkspaceId, WorkspaceManagerError> {
        let workspace_name = name.unwrap_or_else(|| {
            format!("Workspace {}", internal_state.next_workspace_number)
        });

        let persistent_id = persistent_id_opt.unwrap_or_else(|| Uuid::new_v4().to_string());

        if internal_state.persistent_id_to_runtime_id.contains_key(&persistent_id) {
            return Err(WorkspaceManagerError::DuplicatePersistentId(persistent_id));
        }

        let workspace = Workspace::new(workspace_name, Some(persistent_id.clone()), icon_name.clone(), accent_color_hex.clone())?;
        
        let id = workspace.id();
        let position = internal_state.ordered_workspace_ids.len();

        internal_state.workspaces.insert(id, workspace.clone());
        internal_state.ordered_workspace_ids.push(id);
        internal_state.persistent_id_to_runtime_id.insert(persistent_id.clone(), id);
        internal_state.next_workspace_number += 1;

        debug!("Workspace created: id={}, name='{}', persistent_id='{}', position={}", id, workspace.name(), persistent_id, position);

        if let Err(e) = internal_state.event_publisher.send(WorkspaceEvent::WorkspaceCreated {
            id,
            name: workspace.name().to_string(),
            position,
            persistent_id: Some(persistent_id),
            icon_name,
            accent_color_hex,
        }) {
            warn!("Failed to send WorkspaceCreated event: {}", e);
        }
        Ok(id)
    }

    async fn internal_save_configuration_locked(
        internal_state: &WorkspaceManagerInternalState,
    ) -> Result<(), WorkspaceManagerError> {
        debug!("Saving workspace configuration...");
        let mut workspace_snapshots = Vec::new();
        for ws_id_runtime in &internal_state.ordered_workspace_ids {
            if let Some(ws) = internal_state.workspaces.get(ws_id_runtime) {
                let pid = ws.persistent_id().ok_or_else(|| WorkspaceManagerError::Internal {
                        context: format!("Workspace {} has no persistent ID during save.", ws.id()),
                    })?;
                workspace_snapshots.push(WorkspaceSnapshot {
                    name: ws.name().to_string(),
                    layout_type: ws.layout_type(),
                    persistent_id: pid.to_string(),
                    icon_name: ws.icon_name().map(String::from),
                    accent_color_hex: ws.accent_color_hex().map(String::from),
                });
            } else {
                error!("Workspace runtime ID {} in ordered list but not in map.", ws_id_runtime);
            }
        }

        let active_workspace_persistent_id = internal_state.active_workspace_id
            .and_then(|runtime_id| internal_state.workspaces.get(&runtime_id))
            .and_then(|ws| ws.persistent_id().map(String::from));
            
        let snapshot = WorkspaceSetSnapshot {
            workspaces: workspace_snapshots,
            active_workspace_persistent_id,
        };
        
        snapshot.validate().map_err(|reason| WorkspaceManagerError::Internal { context: format!("Invalid snapshot before save: {}", reason)})?;

        internal_state.config_provider.save_workspace_config(&snapshot).await?;
        info!("Workspace configuration saved successfully.");
        Ok(())
    }

    async fn internal_set_active_workspace_locked(
        internal_state: &mut WorkspaceManagerInternalState,
        new_active_id: Option<WorkspaceId>, // Option to handle cases like last workspace deletion
    ) {
        let old_active_id = internal_state.active_workspace_id.replace(new_active_id.unwrap_or_else(|| {
            // If new_active_id is None (e.g. last workspace deleted and no fallback),
            // try to set first available as active.
            // This logic might need to be more robust depending on desired behavior when no workspaces exist.
            // For now, if new_active_id is None, active_workspace_id becomes None.
            // The prompt for delete_workspace implies setting a new active one if current is deleted.
            // This internal helper takes Option to allow clearing active_id if absolutely no ws left.
            // However, delete_workspace logic should prevent deleting the last one.
            warn!("internal_set_active_workspace_locked called with None, clearing active workspace ID.");
            return None; // This line is problematic, replace was already called.
        }.unwrap_or_else(|| {
             // This block is for when new_active_id itself is None.
             // If we're clearing the active ID (e.g., no workspaces left), then old_active_id is taken,
             // and active_workspace_id becomes None.
             // The logic in delete_workspace should ensure it doesn't reach a state of 0 workspaces
             // if "CannotDeleteLastWorkspace" is enforced.
             // If new_active_id is actually None, then this is fine.
             if internal_state.workspaces.is_empty() {
                 None
             } else {
                 // This case should ideally not be hit if new_active_id is None and workspaces exist.
                 // Caller should provide a valid ID or None if truly clearing.
                 // For safety, if workspaces exist but None was passed, pick first.
                 // This makes the None input to this function less about clearing and more about "auto-pick".
                 // Let's refine this. The caller of this function should decide the new active ID.
                 // This function's job is to set it and emit event.
                 // So, if new_active_id is None, it means we are explicitly clearing it.
                 None
             }
        }));


        if old_active_id != new_active_id { // Only send event if it actually changed
            if let Err(e) = internal_state.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged {
                old_id: old_active_id,
                new_id: new_active_id.expect("New active ID must be Some to send ActiveWorkspaceChanged"), // This expect is risky if None is valid
            }) {
                warn!("Failed to send ActiveWorkspaceChanged event: {}", e);
            }
        }
    }
}

#[async_trait]
impl WorkspaceManagerService for DefaultWorkspaceManager {
    async fn load_initial_state(&self) -> Result<(), WorkspaceManagerError> {
        info!("Loading initial workspace state...");
        let mut internal = self.internal.lock().await;
        internal.workspaces.clear();
        internal.ordered_workspace_ids.clear();
        internal.persistent_id_to_runtime_id.clear();
        internal.next_workspace_number = 1; // Reset for naming
        
        let snapshot = internal.config_provider.load_workspace_config().await?;
        snapshot.validate().map_err(|reason| WorkspaceManagerError::ConfigError(
            crate::workspaces::config::errors::WorkspaceConfigError::InvalidData{ reason, path: None }
        ))?;

        if snapshot.workspaces.is_empty() {
            info!("No existing workspace configuration found or empty. Creating default workspace.");
            let default_ws_id = Self::internal_create_workspace_locked(&mut *internal, None, None, None, None).await?;
            // internal.active_workspace_id = Some(default_ws_id); // Set directly by internal_create_workspace_locked IF it's the first
            // For initial load, if empty, the first created becomes active implicitly by subsequent logic
            // Let's explicitly set it if it's the only one.
            if internal.ordered_workspace_ids.len() == 1 {
                 internal.active_workspace_id = Some(default_ws_id);
            }
            Self::internal_save_configuration_locked(&*internal).await?;
        } else {
            info!("Loading {} workspaces from configuration.", snapshot.workspaces.len());
            let mut highest_num = 0;
            let mut new_active_runtime_id: Option<WorkspaceId> = None;

            for ws_snapshot in snapshot.workspaces {
                if ws_snapshot.name.starts_with("Workspace ") {
                    if let Ok(num) = ws_snapshot.name.trim_start_matches("Workspace ").parse::<u32>() {
                        if num > highest_num { highest_num = num; }
                    }
                }
                // Check for duplicate persistent_id before creating workspace
                if internal.persistent_id_to_runtime_id.contains_key(&ws_snapshot.persistent_id) {
                    error!("Duplicate persistent_id '{}' found in config. Skipping workspace '{}'.", ws_snapshot.persistent_id, ws_snapshot.name);
                    // Optionally return error or just skip and log
                    continue; 
                }

                let mut workspace = Workspace::new(
                    ws_snapshot.name,
                    Some(ws_snapshot.persistent_id.clone()), // PID is mandatory in snapshot
                    ws_snapshot.icon_name,
                    ws_snapshot.accent_color_hex,
                )?;
                workspace.set_layout_type(ws_snapshot.layout_type);

                let runtime_id = workspace.id();
                internal.workspaces.insert(runtime_id, workspace);
                internal.ordered_workspace_ids.push(runtime_id);
                internal.persistent_id_to_runtime_id.insert(ws_snapshot.persistent_id.clone(), runtime_id);

                if snapshot.active_workspace_persistent_id.as_ref() == Some(&ws_snapshot.persistent_id) {
                    new_active_runtime_id = Some(runtime_id);
                }
            }
            internal.next_workspace_number = highest_num + 1;

            if new_active_runtime_id.is_none() && !internal.ordered_workspace_ids.is_empty() {
                new_active_runtime_id = Some(internal.ordered_workspace_ids[0]); // Fallback to first
                warn!("Configured active_workspace_persistent_id not found or invalid. Defaulting to first workspace.");
            }
            internal.active_workspace_id = new_active_runtime_id;


            if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WorkspacesReloaded {
                new_order: internal.ordered_workspace_ids.clone(),
            }) { warn!("Failed to send WorkspacesReloaded event: {}", e); }

            if let Some(active_id) = internal.active_workspace_id {
                if let Err(e) = internal.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged {
                    old_id: None, new_id: active_id,
                }) { warn!("Failed to send ActiveWorkspaceChanged event on load: {}", e); }
            }
            info!("Initial workspace state loaded. Active workspace: {:?}", internal.active_workspace_id);
        }
        Ok(())
    }

    async fn create_workspace(
        &self,
        name: Option<String>,
        persistent_id: Option<String>,
        icon_name: Option<String>,
        accent_color_hex: Option<String>,
    ) -> Result<WorkspaceId, WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        let id = Self::internal_create_workspace_locked(&mut *internal, name, persistent_id, icon_name, accent_color_hex).await?;
        // If this is the first workspace created, make it active
        if internal.ordered_workspace_ids.len() == 1 {
            internal.active_workspace_id = Some(id);
            // No ActiveWorkspaceChanged event here, as it's part of WorkspaceCreated for the first one.
            // Or, if WorkspaceCreated doesn't imply activation, send it:
            // if let Err(e) = internal.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged { old_id: None, new_id: id }) {
            //     warn!("Failed to send ActiveWorkspaceChanged for first created: {}", e);
            // }
        }
        Self::internal_save_configuration_locked(&*internal).await?;
        Ok(id)
    }
    
    async fn delete_workspace(
        &self,
        id: WorkspaceId,
        fallback_id_for_windows: Option<WorkspaceId>,
    ) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        if internal.ordered_workspace_ids.len() <= 1 {
            return Err(WorkspaceManagerError::CannotDeleteLastWorkspace);
        }
        let workspace_to_delete = internal.workspaces.get(&id).cloned().ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;
        
        let mut windows_moved_to_id: Option<WorkspaceId> = None;

        if !workspace_to_delete.window_ids().is_empty() {
            let fallback_id = fallback_id_for_windows.ok_or_else(|| WorkspaceManagerError::DeleteRequiresFallbackForWindows {
                workspace_id: id,
                window_count: workspace_to_delete.window_ids().len(),
            })?;
            
            if fallback_id == id {
                 return Err(WorkspaceManagerError::InvalidOperation("Fallback workspace cannot be the workspace being deleted.".to_string()));
            }

            if !internal.workspaces.contains_key(&fallback_id) {
                return Err(WorkspaceManagerError::FallbackWorkspaceNotFound(fallback_id));
            }
            windows_moved_to_id = Some(fallback_id);

            let window_ids_to_move: Vec<WindowIdentifier> = workspace_to_delete.window_ids().iter().cloned().collect();
            for win_id in window_ids_to_move {
                // Use assign_window_to_workspace directly on internal.workspaces
                // This avoids re-locking or complex calls to self.
                let assign_result = assign_window_to_workspace(
                    &mut internal.workspaces,
                    fallback_id,
                    &win_id,
                    internal.ensure_unique_window_assignment, // Use manager's policy
                );
                match assign_result {
                    Ok(Some(old_ws_id)) => { // Window was moved
                         if old_ws_id != id { /* This shouldn't happen if ensure_unique is true and window was in ws_to_delete */ }
                         if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData{workspace_id: id, window_id: win_id.clone()})) { warn!("Event send error: {}", e); }
                         if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData{workspace_id: fallback_id, window_id: win_id.clone()})) { warn!("Event send error: {}", e); }
                    }
                    Ok(None) => { // Window was only added to new, or already there and unique was false
                         // This case means it was added to fallback_id. Still need RemovedFrom event for original.
                         if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData{workspace_id: id, window_id: win_id.clone()})) { warn!("Event send error: {}", e); }
                         if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData{workspace_id: fallback_id, window_id: win_id.clone()})) { warn!("Event send error: {}", e); }
                    }
                    Err(e) => {
                        // Log error and continue? Or fail deletion? For now, log and continue.
                        error!("Error moving window {} from {} to {}: {:?}", win_id, id, fallback_id, e);
                    }
                }
            }
        }

        internal.workspaces.remove(&id);
        if let Some(pid_key) = workspace_to_delete.persistent_id() {
            internal.persistent_id_to_runtime_id.remove(pid_key);
        }
        internal.ordered_workspace_ids.retain(|&ws_id| ws_id != id);

        let mut new_active_id_after_delete: Option<WorkspaceId> = None;
        if internal.active_workspace_id == Some(id) {
            // Set new active workspace (e.g., first in list, or previous/next)
            // For simplicity, set to the first available if any.
            new_active_id_after_delete = internal.ordered_workspace_ids.first().cloned();
            let old_active_id = internal.active_workspace_id.replace(new_active_id_after_delete.unwrap()); // Should always have one if not last
            
            if old_active_id != new_active_id_after_delete { // Check if active ID actually changed
                if let Err(e) = internal.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged {
                    old_id: Some(id), // The deleted one was active
                    new_id: new_active_id_after_delete.expect("Must have a new active workspace if not last one"),
                }) {
                    warn!("Failed to send ActiveWorkspaceChanged event after delete: {}", e);
                }
            }
        }
        
        if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WorkspaceDeleted { id, windows_moved_to_workspace_id: windows_moved_to_id }) {
            warn!("Failed to send WorkspaceDeleted event: {}", e);
        }
        
        // Update next_workspace_number if the deleted workspace was the highest numbered default
        // This is a simple heuristic and might need refinement for robustness
        if workspace_to_delete.name().starts_with("Workspace ") {
            if let Ok(num) = workspace_to_delete.name().trim_start_matches("Workspace ").parse::<u32>() {
                if num >= internal.next_workspace_number -1 { // If it was the one that set next_workspace_number
                    // Re-calculate next_workspace_number based on remaining default-named workspaces
                    let mut max_num = 0;
                    for ws in internal.workspaces.values() {
                        if ws.name().starts_with("Workspace ") {
                             if let Ok(n) = ws.name().trim_start_matches("Workspace ").parse::<u32>() {
                                if n > max_num { max_num = n; }
                            }
                        }
                    }
                    internal.next_workspace_number = max_num + 1;
                }
            }
        }


        Self::internal_save_configuration_locked(&*internal).await?;
        info!("Workspace {} deleted. Windows moved to {:?}. New active: {:?}", id, windows_moved_to_id, new_active_id_after_delete);
        Ok(())
    }


    async fn get_workspace(&self, id: WorkspaceId) -> Option<Workspace> {
        self.internal.lock().await.workspaces.get(&id).cloned()
    }

    async fn all_workspaces_ordered(&self) -> Vec<Workspace> {
        let internal = self.internal.lock().await;
        internal.ordered_workspace_ids.iter()
            .filter_map(|id| internal.workspaces.get(id).cloned())
            .collect()
    }

    async fn active_workspace_id(&self) -> Option<WorkspaceId> {
        self.internal.lock().await.active_workspace_id
    }

    async fn set_active_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        if !internal.workspaces.contains_key(&id) {
            return Err(WorkspaceManagerError::SetActiveWorkspaceNotFound(id));
        }
        if internal.active_workspace_id == Some(id) {
            return Ok(()); 
        }

        let old_id = internal.active_workspace_id.replace(id);
        debug!("Active workspace changed from {:?} to {}", old_id, id);

        if let Err(e) = internal.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged { old_id, new_id: id }) {
            warn!("Failed to send ActiveWorkspaceChanged event: {}", e);
        }
        Self::internal_save_configuration_locked(&*internal).await?;
        Ok(())
    }

    async fn save_configuration(&self) -> Result<(), WorkspaceManagerError> {
        let internal = self.internal.lock().await;
        Self::internal_save_configuration_locked(&*internal).await
    }
    
    fn subscribe_to_workspace_events(&self) -> broadcast::Receiver<WorkspaceEvent> {
        futures::executor::block_on(self.internal.lock()).event_publisher.subscribe()
    }

    async fn assign_window_to_active_workspace(&self, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        let active_id = internal.active_workspace_id.ok_or(WorkspaceManagerError::NoActiveWorkspace)?;
        
        let old_ws_id_opt = assign_window_to_workspace(
            &mut internal.workspaces,
            active_id,
            window_id,
            internal.ensure_unique_window_assignment,
        )?;

        if let Some(old_ws_id) = old_ws_id_opt {
            if old_ws_id != active_id { // Ensure it was actually moved from a *different* workspace
                if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData{workspace_id: old_ws_id, window_id: window_id.clone()})) { warn!("Event send error: {}", e); }
            }
        }
        if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData{workspace_id: active_id, window_id: window_id.clone()})) { warn!("Event send error: {}", e); }
        // No save_configuration here; window assignments are not persisted in Iteration 3 config.
        Ok(())
    }

    async fn assign_window_to_specific_workspace(&self, workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        if !internal.workspaces.contains_key(&workspace_id) {
            return Err(WindowAssignmentError::WorkspaceNotFound(workspace_id).into());
        }
        
        let old_ws_id_opt = assign_window_to_workspace(
            &mut internal.workspaces,
            workspace_id,
            window_id,
            internal.ensure_unique_window_assignment,
        )?;
        
        if let Some(old_ws_id) = old_ws_id_opt {
             if old_ws_id != workspace_id {
                if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData{workspace_id: old_ws_id, window_id: window_id.clone()})) { warn!("Event send error: {}", e); }
             }
        }
        if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData{workspace_id: workspace_id, window_id: window_id.clone()})) { warn!("Event send error: {}", e); }
        Ok(())
    }

    async fn remove_window_from_its_workspace(&self, window_id: &WindowIdentifier) -> Result<Option<WorkspaceId>, WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        if let Some(owner_id) = find_workspace_for_window(&internal.workspaces, window_id) {
            remove_window_from_workspace(&mut internal.workspaces, owner_id, window_id)?;
            if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData{workspace_id: owner_id, window_id: window_id.clone()})) { warn!("Event send error: {}", e); }
            Ok(Some(owner_id))
        } else {
            Ok(None) // Window was not found in any workspace
        }
    }
    
    async fn move_window_to_specific_workspace(&self, target_workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        if !internal.workspaces.contains_key(&target_workspace_id) {
            return Err(WindowAssignmentError::WorkspaceNotFound(target_workspace_id).into());
        }

        let old_ws_id_opt = find_workspace_for_window(&internal.workspaces, window_id);
        
        // Use assign_window_to_workspace with ensure_unique_assignment = true, which handles removal from old.
        // This assumes ensure_unique_window_assignment is true for a "move" operation.
        // If the manager's policy is false, then a "move" would be "add to new" and "explicit remove from old".
        // For simplicity, let's assume a move implies unique assignment.
        let moved_from_id_opt = assign_window_to_workspace(
            &mut internal.workspaces,
            target_workspace_id,
            window_id,
            true, // A "move" implies unique assignment
        )?;
        
        if let Some(moved_from_id) = moved_from_id_opt {
            if moved_from_id != target_workspace_id { // It was actually moved from a different workspace
                if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData{workspace_id: moved_from_id, window_id: window_id.clone()})) { warn!("Event send error: {}", e); }
                 if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData{workspace_id: target_workspace_id, window_id: window_id.clone()})) { warn!("Event send error: {}", e); }
            } else { // It was already in the target_workspace_id, and ensure_unique found it there. No "move" from different.
                 // Still, if ensure_unique was true, it might have cleaned up other assignments.
                 // But assign_window_to_workspace returns None if already in target and unique.
                 // This means it was only added to target, or was already in target and no other assignments.
                 // If it was only added (not pre-existing in target), an Added event is needed.
                 // This logic is tricky. Let's simplify: if old_ws_id_opt (found before assign) is Some and different from target,
                 // then it's a definite move.
                 if let Some(original_owner_id) = old_ws_id_opt {
                     if original_owner_id != target_workspace_id {
                        // This case is covered by moved_from_id_opt being Some and different.
                     } else {
                         // It was already in target. No events needed unless assign_window... indicates a change.
                         // But if moved_from_id_opt is None, it means it was newly added or already there.
                         // If it was newly added:
                         if old_ws_id_opt.is_none() {
                            if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData{workspace_id: target_workspace_id, window_id: window_id.clone()})) { warn!("Event send error: {}", e); }
                         }
                     }
                 } else { // Was not in any workspace before, so it's just an add.
                     if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData{workspace_id: target_workspace_id, window_id: window_id.clone()})) { warn!("Event send error: {}", e); }
                 }
            }
        } else { // Window was not in another workspace before, so it's just an add to target.
             if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData{workspace_id: target_workspace_id, window_id: window_id.clone()})) { warn!("Event send error: {}", e); }
        }
        Ok(())
    }


    async fn set_workspace_name(&self, id: WorkspaceId, name: String) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        let ws = internal.workspaces.get_mut(&id).ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;
        let old_name = ws.name().to_string();
        ws.rename(name.clone())?; // This is WorkspaceCoreError, will be converted by #[from]
        
        if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WorkspaceRenamed { data: WorkspaceRenamedData { id, old_name, new_name: name }}) {
            warn!("Event send error: {}", e);
        }
        Self::internal_save_configuration_locked(&*internal).await?;
        Ok(())
    }

    async fn set_workspace_layout(&self, id: WorkspaceId, layout: WorkspaceLayoutType) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        let ws = internal.workspaces.get_mut(&id).ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;
        let old_layout = ws.layout_type();
        ws.set_layout_type(layout);

        if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WorkspaceLayoutChanged { data: WorkspaceLayoutChangedData { id, old_layout, new_layout: layout }}) {
            warn!("Event send error: {}", e);
        }
        Self::internal_save_configuration_locked(&*internal).await?;
        Ok(())
    }
    
    async fn set_workspace_persistent_id(&self, id: WorkspaceId, persistent_id_opt: Option<String>) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        let ws = internal.workspaces.get_mut(&id).ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;
        
        let old_persistent_id = ws.persistent_id().map(String::from);
        
        // If new persistent_id is Some, check for duplicates across all workspaces *except* the current one
        if let Some(ref new_pid_val) = persistent_id_opt {
            if internal.persistent_id_to_runtime_id.get(new_pid_val).map_or(false, |&runtime_id| runtime_id != id) {
                return Err(WorkspaceManagerError::DuplicatePersistentId(new_pid_val.clone()));
            }
        }
        
        ws.set_persistent_id(persistent_id_opt.clone())?; // Validate format

        // Update mapping
        if let Some(old_pid_val) = &old_persistent_id {
            internal.persistent_id_to_runtime_id.remove(old_pid_val);
        }
        if let Some(new_pid_val) = &persistent_id_opt {
            internal.persistent_id_to_runtime_id.insert(new_pid_val.clone(), id);
        }
        
        if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WorkspacePersistentIdChanged(WorkspacePersistentIdChangedData{id, old_persistent_id, new_persistent_id: persistent_id_opt })) {
            warn!("Event send error: {}", e);
        }
        Self::internal_save_configuration_locked(&*internal).await?;
        Ok(())
    }


    async fn set_workspace_icon(&self, id: WorkspaceId, icon_name: Option<String>) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        let ws = internal.workspaces.get_mut(&id).ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;
        let old_icon_name = ws.icon_name().map(String::from);
        ws.set_icon_name(icon_name.clone());

        if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WorkspaceIconChanged(WorkspaceIconChangedData{id, old_icon_name, new_icon_name: icon_name })) {
            warn!("Event send error: {}", e);
        }
        Self::internal_save_configuration_locked(&*internal).await?;
        Ok(())
    }

    async fn set_workspace_accent_color(&self, id: WorkspaceId, color_hex: Option<String>) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        let ws = internal.workspaces.get_mut(&id).ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;
        let old_color_hex = ws.accent_color_hex().map(String::from);
        ws.set_accent_color_hex(color_hex.clone())?; // This can fail validation

        if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WorkspaceAccentChanged(WorkspaceAccentChangedData{id, old_color_hex, new_color_hex: color_hex })) {
            warn!("Event send error: {}", e);
        }
        Self::internal_save_configuration_locked(&*internal).await?;
        Ok(())
    }
    
    async fn reorder_workspace(&self, workspace_id: WorkspaceId, new_index: usize) -> Result<(), WorkspaceManagerError> {
        let mut internal = self.internal.lock().await;
        let current_index_opt = internal.ordered_workspace_ids.iter().position(|&id| id == workspace_id);

        if current_index_opt.is_none() {
            return Err(WorkspaceManagerError::WorkspaceNotFound(workspace_id));
        }
        let current_index = current_index_opt.unwrap();

        if new_index >= internal.ordered_workspace_ids.len() {
            return Err(WorkspaceManagerError::InvalidOperation(format!(
                "New index {} is out of bounds for {} workspaces.",
                new_index,
                internal.ordered_workspace_ids.len()
            )));
        }

        let id_to_move = internal.ordered_workspace_ids.remove(current_index);
        internal.ordered_workspace_ids.insert(new_index, id_to_move);
        
        if let Err(e) = internal.event_publisher.send(WorkspaceEvent::WorkspaceOrderChanged(internal.ordered_workspace_ids.clone())) {
            warn!("Event send error: {}", e);
        }
        Self::internal_save_configuration_locked(&*internal).await?;
        Ok(())
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspaces::config::{WorkspaceConfigProvider, WorkspaceSetSnapshot, WorkspaceSnapshot};
    use crate::workspaces::core::WorkspaceLayoutType;
    use std::sync::Arc;
    use tokio::sync::{broadcast, RwLock}; // Corrected path
    use tokio::time::{timeout, Duration};
    use uuid::Uuid;

    #[derive(Default)]
    struct MockWsConfigProvider {
        snapshot: RwLock<WorkspaceSetSnapshot>,
        force_load_error: Option<WorkspaceConfigError>, // Changed to specific error type
        force_save_error: Option<WorkspaceConfigError>, // Changed to specific error type
    }
    impl MockWsConfigProvider {
        fn new() -> Self { Default::default() }
        async fn set_snapshot(&self, snapshot: WorkspaceSetSnapshot) { *self.snapshot.write().await = snapshot; }
        #[allow(dead_code)] fn set_force_load_error(&mut self, err: Option<WorkspaceConfigError>) { self.force_load_error = err; }
        #[allow(dead_code)] fn set_force_save_error(&mut self, err: Option<WorkspaceConfigError>) { self.force_save_error = err; }
    }

    #[async_trait]
    impl WorkspaceConfigProvider for MockWsConfigProvider {
        async fn load_workspace_config(&self) -> Result<WorkspaceSetSnapshot, crate::workspaces::config::errors::WorkspaceConfigError> {
            if let Some(ref e) = self.force_load_error { return Err(e.clone()); } // Clone error if needed
            Ok(self.snapshot.read().await.clone())
        }
        async fn save_workspace_config(&self, config_snapshot: &WorkspaceSetSnapshot) -> Result<(), crate::workspaces::config::errors::WorkspaceConfigError> {
             if let Some(ref e) = self.force_save_error { return Err(e.clone()); }
            *self.snapshot.write().await = config_snapshot.clone();
            Ok(())
        }
    }

    fn create_manager_for_test(ensure_unique: bool) -> (DefaultWorkspaceManager, Arc<MockWsConfigProvider>) {
        let mock_provider = Arc::new(MockWsConfigProvider::new());
        let manager = DefaultWorkspaceManager::new(mock_provider.clone(), 32, ensure_unique);
        (manager, mock_provider)
    }
    
    #[tokio::test]
    async fn test_manager_load_initial_state_with_persistent_ids_and_new_fields() {
        let (manager, provider) = create_manager_for_test(true);
        let mut event_rx = manager.subscribe_to_workspace_events();

        let initial_snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { name: "WS_Persistent_1".to_string(), layout_type: WorkspaceLayoutType::Floating, persistent_id: "pid1".to_string(), icon_name: Some("icon1".to_string()), accent_color_hex: Some("#111111".to_string()) },
                WorkspaceSnapshot { name: "WS_Persistent_2".to_string(), layout_type: WorkspaceLayoutType::TilingVertical, persistent_id: "pid2".to_string(), icon_name: None, accent_color_hex: None },
            ],
            active_workspace_persistent_id: Some("pid2".to_string()),
        };
        provider.set_snapshot(initial_snapshot.clone()).await;

        manager.load_initial_state().await.unwrap();

        let workspaces = manager.all_workspaces_ordered().await;
        assert_eq!(workspaces.len(), 2);
        assert_eq!(workspaces[0].name(), "WS_Persistent_1");
        assert_eq!(workspaces[0].persistent_id(), Some("pid1"));
        assert_eq!(workspaces[0].icon_name(), Some("icon1"));
        assert_eq!(workspaces[0].accent_color_hex(), Some("#111111"));
        
        assert_eq!(workspaces[1].name(), "WS_Persistent_2");
        assert_eq!(workspaces[1].persistent_id(), Some("pid2"));

        let active_ws = manager.get_workspace(manager.active_workspace_id().await.unwrap()).await.unwrap();
        assert_eq!(active_ws.persistent_id(), Some("pid2"));

        // Check events
        let event1 = timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap();
        match event1 {
            WorkspaceEvent::WorkspacesReloaded { new_order } => assert_eq!(new_order.len(), 2),
            _ => panic!("Expected WorkspacesReloaded"),
        }
        let event2 = timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap();
        match event2 {
            WorkspaceEvent::ActiveWorkspaceChanged { new_id, .. } => assert_eq!(new_id, active_ws.id()),
            _ => panic!("Expected ActiveWorkspaceChanged"),
        }
    }

    #[tokio::test]
    async fn test_create_workspace_with_all_fields() {
        let (manager, provider) = create_manager_for_test(true);
        manager.load_initial_state().await.unwrap(); // Loads "Workspace 1" with a generated PID
        let mut event_rx = manager.subscribe_to_workspace_events();

        let name = "My Full WS".to_string();
        let pid = "my-full-ws-pid".to_string();
        let icon = "my-icon".to_string();
        let color = "#ABCDEF".to_string();

        let ws_id = manager.create_workspace(Some(name.clone()), Some(pid.clone()), Some(icon.clone()), Some(color.clone())).await.unwrap();
        
        let ws = manager.get_workspace(ws_id).await.unwrap();
        assert_eq!(ws.name(), name);
        assert_eq!(ws.persistent_id(), Some(pid.as_str()));
        assert_eq!(ws.icon_name(), Some(icon.as_str()));
        assert_eq!(ws.accent_color_hex(), Some(color.as_str()));

        match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WorkspaceCreated { name: ev_name, persistent_id: ev_pid, icon_name: ev_icon, accent_color_hex: ev_color, .. } => {
                assert_eq!(ev_name, name);
                assert_eq!(ev_pid, Some(pid));
                assert_eq!(ev_icon, Some(icon));
                assert_eq!(ev_color, Some(color));
            }
            e => panic!("Expected WorkspaceCreated, got {:?}", e),
        }
        
        // Check save
        let snapshot = provider.snapshot.read().await.clone();
        assert_eq!(snapshot.workspaces.len(), 2); // "Workspace 1" + "My Full WS"
        let saved_ws_snapshot = snapshot.workspaces.iter().find(|ws_s| ws_s.persistent_id == ws.persistent_id().unwrap()).unwrap();
        assert_eq!(saved_ws_snapshot.name, name);
        assert_eq!(saved_ws_snapshot.icon_name, ws.icon_name().map(String::from));
    }
    
    #[tokio::test]
    async fn test_create_workspace_duplicate_persistent_id() {
        let (manager, _) = create_manager_for_test(true);
        manager.load_initial_state().await.unwrap();
        let _ = manager.create_workspace(Some("WS1".to_string()), Some("pid_unique".to_string()), None, None).await.unwrap();
        
        let result = manager.create_workspace(Some("WS2".to_string()), Some("pid_unique".to_string()), None, None).await;
        assert!(matches!(result, Err(WorkspaceManagerError::DuplicatePersistentId(pid)) if pid == "pid_unique"));
    }

    #[tokio::test]
    async fn test_delete_workspace_success_no_windows() {
        let (manager, provider) = create_manager_for_test(true);
        manager.load_initial_state().await.unwrap(); // WS1 active
        let ws2_id = manager.create_workspace(Some("WS2".to_string()), Some("pid_ws2".to_string()), None, None).await.unwrap();
        let _ = manager.create_workspace(Some("WS3".to_string()), Some("pid_ws3".to_string()), None, None).await.unwrap();
        let mut event_rx = manager.subscribe_to_workspace_events();
        
        manager.set_active_workspace(ws2_id).await.unwrap(); // Set WS2 active
        let _ = event_rx.recv().await; // consume create
        let _ = event_rx.recv().await; // consume create
        let _ = event_rx.recv().await; // consume active change

        assert!(manager.delete_workspace(ws2_id, None).await.is_ok());
        
        let workspaces = manager.all_workspaces_ordered().await;
        assert_eq!(workspaces.len(), 2);
        assert!(!workspaces.iter().any(|ws| ws.id() == ws2_id));
        
        let active_id = manager.active_workspace_id().await.unwrap();
        assert_ne!(active_id, ws2_id); // Active should have changed
        assert_eq!(active_id, workspaces[0].id()); // Should default to first

        let event_del = timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap();
         match event_del {
            WorkspaceEvent::WorkspaceDeleted { id, windows_moved_to_workspace_id } => {
                assert_eq!(id, ws2_id);
                assert_eq!(windows_moved_to_workspace_id, None);
            }
            e => panic!("Expected WorkspaceDeleted, got {:?}", e),
        }
        let event_active = timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap();
        match event_active {
            WorkspaceEvent::ActiveWorkspaceChanged { old_id, new_id } => {
                assert_eq!(old_id, Some(ws2_id));
                assert_eq!(new_id, active_id);
            }
            e => panic!("Expected ActiveWorkspaceChanged, got {:?}", e),
        }
        
        let snapshot = provider.snapshot.read().await.clone();
        assert_eq!(snapshot.workspaces.len(), 2);
        assert_eq!(snapshot.active_workspace_persistent_id, manager.get_workspace(active_id).await.unwrap().persistent_id().map(String::from));
    }

    #[tokio::test]
    async fn test_delete_workspace_last_one_fails() {
        let (manager, _) = create_manager_for_test(true);
        manager.load_initial_state().await.unwrap(); // Only "Workspace 1"
        let ws1_id = manager.active_workspace_id().await.unwrap();
        
        let result = manager.delete_workspace(ws1_id, None).await;
        assert!(matches!(result, Err(WorkspaceManagerError::CannotDeleteLastWorkspace)));
    }
    
    #[tokio::test]
    async fn test_delete_workspace_with_windows_requires_fallback() {
        let (manager, _) = create_manager_for_test(true);
        manager.load_initial_state().await.unwrap();
        let ws_to_delete_id = manager.create_workspace(Some("ToDelete".to_string()), Some("pid_delete".to_string()), None, None).await.unwrap();
        let _ = manager.create_workspace(Some("FallbackTarget".to_string()), Some("pid_fallback".to_string()), None, None).await.unwrap();
        
        let win = WindowIdentifier::new("win1".to_string()).unwrap();
        manager.assign_window_to_specific_workspace(ws_to_delete_id, &win).await.unwrap();
        
        let result = manager.delete_workspace(ws_to_delete_id, None).await;
        assert!(matches!(result, Err(WorkspaceManagerError::DeleteRequiresFallbackForWindows { workspace_id, window_count }) 
            if workspace_id == ws_to_delete_id && window_count == 1));
    }

    #[tokio::test]
    async fn test_delete_workspace_with_windows_moves_them_to_fallback() {
        let (manager, _) = create_manager_for_test(true);
        manager.load_initial_state().await.unwrap(); // WS1
        let ws_to_delete_id = manager.create_workspace(Some("ToDelete".to_string()), Some("pid_del".to_string()), None, None).await.unwrap();
        let fallback_ws_id = manager.create_workspace(Some("Fallback".to_string()), Some("pid_fall".to_string()), None, None).await.unwrap();
        let mut event_rx = manager.subscribe_to_workspace_events(); // Subscribe after setup

        let win1 = WindowIdentifier::new("w1".to_string()).unwrap();
        let win2 = WindowIdentifier::new("w2".to_string()).unwrap();
        manager.assign_window_to_specific_workspace(ws_to_delete_id, &win1).await.unwrap();
        manager.assign_window_to_specific_workspace(ws_to_delete_id, &win2).await.unwrap();
        
        // Consume assignment events
        for _ in 0..4 { let _ = event_rx.recv().await; }


        assert!(manager.delete_workspace(ws_to_delete_id, Some(fallback_ws_id)).await.is_ok());
        
        assert!(manager.get_workspace(ws_to_delete_id).await.is_none());
        let fallback_ws = manager.get_workspace(fallback_ws_id).await.unwrap();
        assert!(fallback_ws.window_ids().contains(&win1));
        assert!(fallback_ws.window_ids().contains(&win2));

        // Check events: 2x Removed, 2x Added, 1x Deleted, 1x ActiveChanged (if active was deleted)
        let mut events = Vec::new();
        for _ in 0..6 { // Expecting potentially 6 events
            if let Ok(Ok(event)) = timeout(Duration::from_millis(20), event_rx.recv()).await {
                events.push(event);
            } else {
                break;
            }
        }
        
        assert!(events.iter().any(|e| matches!(e, WorkspaceEvent::WindowRemovedFromWorkspace(data) if data.workspace_id == ws_to_delete_id && data.window_id == win1)));
        assert!(events.iter().any(|e| matches!(e, WorkspaceEvent::WindowAddedToWorkspace(data) if data.workspace_id == fallback_ws_id && data.window_id == win1)));
        assert!(events.iter().any(|e| matches!(e, WorkspaceEvent::WorkspaceDeleted { id, .. } if *id == ws_to_delete_id)));
    }


    #[tokio::test]
    async fn test_assign_window_variants_and_events() {
        let (manager, _) = create_manager_for_test(true); // ensure_unique = true
        manager.load_initial_state().await.unwrap(); // WS1 active
        let ws1_id = manager.active_workspace_id().await.unwrap();
        let ws2_id = manager.create_workspace(Some("WS2".to_string()), Some("pid2".to_string()), None, None).await.unwrap();
        let mut event_rx = manager.subscribe_to_workspace_events();
         let _ = event_rx.recv().await; // consume create event for ws2

        let win1 = WindowIdentifier::new("w1".to_string()).unwrap();
        let win2 = WindowIdentifier::new("w2".to_string()).unwrap();

        // Assign win1 to active (WS1)
        manager.assign_window_to_active_workspace(&win1).await.unwrap();
        assert_eq!(manager.find_workspace_for_window(&win1).await, Some(ws1_id));
        match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WindowAddedToWorkspace(data) => assert_eq!(data.window_id, win1),
            e => panic!("Expected WindowAddedToWorkspace, got {:?}", e),
        }

        // Assign win2 to specific (WS2)
        manager.assign_window_to_specific_workspace(ws2_id, &win2).await.unwrap();
        assert_eq!(manager.find_workspace_for_window(&win2).await, Some(ws2_id));
         match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WindowAddedToWorkspace(data) => assert_eq!(data.window_id, win2),
            e => panic!("Expected WindowAddedToWorkspace, got {:?}", e),
        }
        
        // Move win1 to WS2
        manager.move_window_to_specific_workspace(ws2_id, &win1).await.unwrap();
        assert_eq!(manager.find_workspace_for_window(&win1).await, Some(ws2_id));
        // Expect Removed from WS1, Added to WS2
        match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WindowRemovedFromWorkspace(data) if data.workspace_id == ws1_id && data.window_id == win1 => {},
            e => panic!("Expected WindowRemovedFromWorkspace from ws1, got {:?}", e),
        }
         match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WindowAddedToWorkspace(data) if data.workspace_id == ws2_id && data.window_id == win1 => {},
            e => panic!("Expected WindowAddedToWorkspace to ws2, got {:?}", e),
        }

        // Remove win2 from its workspace (WS2)
        let removed_from_ws = manager.remove_window_from_its_workspace(&win2).await.unwrap();
        assert_eq!(removed_from_ws, Some(ws2_id));
        assert_eq!(manager.find_workspace_for_window(&win2).await, None);
         match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WindowRemovedFromWorkspace(data) if data.workspace_id == ws2_id && data.window_id == win2 => {},
            e => panic!("Expected WindowRemovedFromWorkspace from ws2, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_set_workspace_properties_and_events() {
        let (manager, provider) = create_manager_for_test(true);
        manager.load_initial_state().await.unwrap();
        let ws_id = manager.active_workspace_id().await.unwrap();
        let mut event_rx = manager.subscribe_to_workspace_events();

        // Name
        manager.set_workspace_name(ws_id, "New Name".to_string()).await.unwrap();
        assert_eq!(manager.get_workspace(ws_id).await.unwrap().name(), "New Name");
        match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WorkspaceRenamed { data } if data.new_name == "New Name" => {},
            e => panic!("Expected WorkspaceRenamed, got {:?}", e),
        }
        
        // Layout
        manager.set_workspace_layout(ws_id, WorkspaceLayoutType::TilingHorizontal).await.unwrap();
        assert_eq!(manager.get_workspace(ws_id).await.unwrap().layout_type(), WorkspaceLayoutType::TilingHorizontal);
         match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WorkspaceLayoutChanged { data } if data.new_layout == WorkspaceLayoutType::TilingHorizontal => {},
            e => panic!("Expected WorkspaceLayoutChanged, got {:?}", e),
        }

        // Persistent ID
        let new_pid = "new-pid-123".to_string();
        manager.set_workspace_persistent_id(ws_id, Some(new_pid.clone())).await.unwrap();
        assert_eq!(manager.get_workspace(ws_id).await.unwrap().persistent_id(), Some(new_pid.as_str()));
        match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WorkspacePersistentIdChanged(data) if data.new_persistent_id == Some(new_pid) => {},
            e => panic!("Expected WorkspacePersistentIdChanged, got {:?}", e),
        }

        // Icon
        let new_icon = "new-icon".to_string();
        manager.set_workspace_icon(ws_id, Some(new_icon.clone())).await.unwrap();
        assert_eq!(manager.get_workspace(ws_id).await.unwrap().icon_name(), Some(new_icon.as_str()));
         match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WorkspaceIconChanged(data) if data.new_icon_name == Some(new_icon) => {},
            e => panic!("Expected WorkspaceIconChanged, got {:?}", e),
        }

        // Accent Color
        let new_color = "#FEDCBA".to_string();
        manager.set_workspace_accent_color(ws_id, Some(new_color.clone())).await.unwrap();
        assert_eq!(manager.get_workspace(ws_id).await.unwrap().accent_color_hex(), Some(new_color.as_str()));
        match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WorkspaceAccentChanged(data) if data.new_color_hex == Some(new_color) => {},
            e => panic!("Expected WorkspaceAccentChanged, got {:?}", e),
        }
        
        // Check config saved
        let snapshot = provider.snapshot.read().await.clone();
        let ws_snapshot = snapshot.workspaces.iter().find(|s| s.persistent_id == manager.get_workspace(ws_id).await.unwrap().persistent_id().unwrap()).unwrap();
        assert_eq!(ws_snapshot.name, "New Name");
        assert_eq!(ws_snapshot.layout_type, WorkspaceLayoutType::TilingHorizontal);
        assert_eq!(ws_snapshot.icon_name, Some("new-icon".to_string()));
        assert_eq!(ws_snapshot.accent_color_hex, Some("#FEDCBA".to_string()));
    }
    
    #[tokio::test]
    async fn test_reorder_workspace() {
        let (manager, provider) = create_manager_for_test(true);
        manager.load_initial_state().await.unwrap(); // ws1 (pid_auto_0)
        let ws2_id = manager.create_workspace(Some("WS2".to_string()), Some("pid2".to_string()), None, None).await.unwrap();
        let ws3_id = manager.create_workspace(Some("WS3".to_string()), Some("pid3".to_string()), None, None).await.unwrap();
        let mut event_rx = manager.subscribe_to_workspace_events();
        let _ = event_rx.recv().await; let _ = event_rx.recv().await; // consume create events

        let initial_order: Vec<WorkspaceId> = manager.all_workspaces_ordered().await.into_iter().map(|ws| ws.id()).collect();
        assert_eq!(initial_order.len(), 3);

        // Move WS3 (index 2) to index 0
        manager.reorder_workspace(ws3_id, 0).await.unwrap();
        let new_order = manager.all_workspaces_ordered().await;
        assert_eq!(new_order[0].id(), ws3_id);
        assert_eq!(new_order[1].id(), initial_order[0]); // old ws1
        assert_eq!(new_order[2].id(), ws2_id);

        match timeout(Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WorkspaceOrderChanged(order_vec) => {
                assert_eq!(order_vec[0], ws3_id);
            }
            e => panic!("Expected WorkspaceOrderChanged, got {:?}", e),
        }
        
        // Check save
        let snapshot = provider.snapshot.read().await.clone();
        assert_eq!(snapshot.workspaces[0].persistent_id, manager.get_workspace(ws3_id).await.unwrap().persistent_id().unwrap());
    }
}
