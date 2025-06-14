use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::workspaces::core::{
    Workspace, WorkspaceId, WindowIdentifier, WorkspaceLayoutType,
    event_data::*, // Import all event data structs
};
use crate::workspaces::assignment;
use crate::workspaces::config::{
    WorkspaceConfigProvider, WorkspaceSetSnapshot, WorkspaceSnapshot,
};
use super::events::WorkspaceEvent; // Manager-level events from parent events.rs
use super::errors::WorkspaceManagerError; // Manager-level errors from parent errors.rs


// --- WorkspaceManagerService Trait ---

#[async_trait]
pub trait WorkspaceManagerService: Send + Sync {
    async fn load_or_initialize_workspaces(&self) -> Result<(), WorkspaceManagerError>;
    async fn create_workspace(&self, name: Option<String>, persistent_id: Option<String>, icon_name: Option<String>, accent_color_hex: Option<String>) -> Result<WorkspaceId, WorkspaceManagerError>;
    async fn delete_workspace(&self, id: WorkspaceId, fallback_id_for_windows: Option<WorkspaceId>) -> Result<(), WorkspaceManagerError>;
    fn get_workspace(&self, id: WorkspaceId) -> Option<Workspace>;
    fn all_workspaces_ordered(&self) -> Vec<Workspace>;
    fn active_workspace_id(&self) -> Option<WorkspaceId>;
    async fn set_active_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceManagerError>;
    async fn assign_window_to_active_workspace(&self, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>;
    async fn assign_window_to_specific_workspace(&self, workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>;
    async fn remove_window_from_its_workspace(&self, window_id: &WindowIdentifier) -> Result<Option<WorkspaceId>, WorkspaceManagerError>;
    async fn move_window_to_specific_workspace(&self, target_workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>;
    async fn rename_workspace(&self, id: WorkspaceId, new_name: String) -> Result<(), WorkspaceManagerError>;
    async fn set_workspace_layout(&self, id: WorkspaceId, layout_type: WorkspaceLayoutType) -> Result<(), WorkspaceManagerError>;
    async fn set_workspace_icon(&self, id: WorkspaceId, icon_name: Option<String>) -> Result<(), WorkspaceManagerError>;
    async fn set_workspace_accent_color(&self, id: WorkspaceId, color_hex: Option<String>) -> Result<(), WorkspaceManagerError>;
    async fn save_configuration(&self) -> Result<(), WorkspaceManagerError>;
    fn subscribe_to_workspace_events(&self) -> broadcast::Receiver<WorkspaceEvent>;
    async fn reorder_workspace(&self, workspace_id: WorkspaceId, new_index: usize) -> Result<(), WorkspaceManagerError>;

    // TODO: Assistant Integration - Needed by Smart Assistant
    // Consider methods like:
    // fn get_active_workspace_details(&self) -> Option<SomeWorkspaceDetailStruct>; // Currently active_workspace_id() and get_workspace() can be combined.
    // fn list_workspaces_summary(&self) -> Vec<SomeWorkspaceSummaryStruct>; // Currently all_workspaces_ordered() provides full details.
    // async fn switch_to_workspace_by_name(&self, name: &str) -> Result<(), WorkspaceManagerError>; // Useful for natural language commands.
}

// --- WorkspaceManagerInternalState Struct ---
pub struct WorkspaceManagerInternalState {
    workspaces: HashMap<WorkspaceId, Workspace>,
    active_workspace_id: Option<WorkspaceId>,
    ordered_workspace_ids: Vec<WorkspaceId>,
    next_workspace_number: u32,
    config_provider: Arc<dyn WorkspaceConfigProvider>,
    event_publisher: broadcast::Sender<WorkspaceEvent>,
    ensure_unique_window_assignment: bool,
}

impl WorkspaceManagerInternalState {
    async fn save_configuration(&self) -> Result<(), WorkspaceConfigError> {
        let mut ws_snapshots = Vec::new();
        for ws_id in &self.ordered_workspace_ids {
            if let Some(ws) = self.workspaces.get(ws_id) {
                ws_snapshots.push(WorkspaceSnapshot {
                    persistent_id: ws.persistent_id().map_or_else(
                        || format!("{}{}",crate::workspaces::core::DEFAULT_PERSISTENT_ID_PREFIX, ws.id()), // Fallback to auto-PID using ID
                        |s| s.to_string()
                    ),
                    name: ws.name().to_string(),
                    layout_type: ws.layout_type(),
                    icon_name: ws.icon_name().map(String::from),
                    accent_color_hex: ws.accent_color_hex().map(String::from),
                });
            }
        }
        
        let active_pid = self.active_workspace_id
            .and_then(|active_id| self.workspaces.get(&active_id))
            .and_then(|ws| ws.persistent_id().map(String::from)
            .or_else(|| Some(format!("{}{}",crate::workspaces::core::DEFAULT_PERSISTENT_ID_PREFIX, ws.id()))));

        let snapshot = WorkspaceSetSnapshot {
            workspaces: ws_snapshots,
            active_workspace_persistent_id: active_pid,
        };
        self.config_provider.save_workspace_config(&snapshot).await
    }

    fn create_workspace_locked(
        &mut self,
        name: Option<String>,
        persistent_id: Option<String>,
        icon_name: Option<String>,
        accent_color_hex: Option<String>,
    ) -> Result<WorkspaceId, WorkspaceManagerError> {
        if let Some(pid) = &persistent_id {
            if self.workspaces.values().any(|ws| ws.persistent_id() == Some(pid.as_str())) {
                return Err(WorkspaceManagerError::DuplicatePersistentId(pid.clone()));
            }
        }

        let workspace_name = name.unwrap_or_else(|| {
            let mut num_to_try = self.next_workspace_number;
            loop {
                let potential_name = format!("Workspace {}", num_to_try);
                if !self.workspaces.values().any(|ws| ws.name() == potential_name) {
                    self.next_workspace_number = num_to_try + 1;
                    break potential_name;
                }
                num_to_try += 1;
            }
        });
        
        let new_ws = Workspace::new(workspace_name.clone(), persistent_id.clone(), icon_name.clone(), accent_color_hex.clone())?;
        let new_id = new_ws.id();
        let position = self.ordered_workspace_ids.len();

        self.workspaces.insert(new_id, new_ws);
        self.ordered_workspace_ids.push(new_id);

        let _ = self.event_publisher.send(WorkspaceEvent::WorkspaceCreated {
            id: new_id, name: workspace_name, persistent_id, position, icon_name, accent_color_hex,
        });
        
        Ok(new_id)
    }
}

// --- DefaultWorkspaceManager Implementation ---
#[derive(Clone)]
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
            workspaces: HashMap::new(), active_workspace_id: None, ordered_workspace_ids: Vec::new(),
            next_workspace_number: 1, config_provider, event_publisher, ensure_unique_window_assignment,
        };
        Self { internal: Arc::new(Mutex::new(internal_state)) }
    }
}

#[async_trait]
impl WorkspaceManagerService for DefaultWorkspaceManager {
    async fn load_or_initialize_workspaces(&self) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        info!("Loading or initializing workspaces...");

        match guard.config_provider.load_workspace_config().await {
            Ok(snapshot) if snapshot.workspaces.is_empty() && snapshot.active_workspace_persistent_id.is_none() => {
                info!("No existing config or empty. Creating default workspace.");
                let default_ws_id = guard.create_workspace_locked(None, None, None, None)?;
                guard.active_workspace_id = Some(default_ws_id);
                let _ = guard.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged { old_id: None, new_id: default_ws_id });
                guard.save_configuration().await?;
            }
            Ok(snapshot) => {
                info!("Loaded workspace configuration. Processing {} snapshots.", snapshot.workspaces.len());
                guard.workspaces.clear(); guard.ordered_workspace_ids.clear();
                let mut temp_pid_to_id_map = HashMap::new();

                for ws_snapshot in snapshot.workspaces {
                    let effective_pid = if ws_snapshot.persistent_id.starts_with(crate::workspaces::core::DEFAULT_PERSISTENT_ID_PREFIX) {
                        None // Treat auto-generated PIDs as if they were None for creation logic
                    } else {
                        Some(ws_snapshot.persistent_id.clone())
                    };
                    let ws = Workspace::new(ws_snapshot.name.clone(), effective_pid, ws_snapshot.icon_name.clone(), ws_snapshot.accent_color_hex.clone())?;
                    let ws_id = ws.id();
                    guard.workspaces.insert(ws_id, ws);
                    guard.ordered_workspace_ids.push(ws_id);
                    if !ws_snapshot.persistent_id.is_empty() {
                        temp_pid_to_id_map.insert(ws_snapshot.persistent_id, ws_id);
                    }
                }

                if let Some(active_pid) = snapshot.active_workspace_persistent_id {
                     if !active_pid.is_empty() {
                        guard.active_workspace_id = temp_pid_to_id_map.get(&active_pid).cloned();
                     }
                }
                if guard.active_workspace_id.is_none() && !guard.ordered_workspace_ids.is_empty() {
                    guard.active_workspace_id = Some(guard.ordered_workspace_ids[0]);
                }

                let mut max_num = 0;
                for ws in guard.workspaces.values() {
                    if ws.name().starts_with("Workspace ") {
                        if let Ok(num) = ws.name()["Workspace ".len()..].parse::<u32>() { if num > max_num { max_num = num; } }
                    }
                }
                guard.next_workspace_number = max_num + 1;

                let _ = guard.event_publisher.send(WorkspaceEvent::WorkspacesReloaded { new_order: guard.ordered_workspace_ids.clone(), active_workspace_id: guard.active_workspace_id });
                if let Some(active_id) = guard.active_workspace_id {
                     let _ = guard.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged { old_id: None, new_id: active_id });
                }
                info!("Workspaces reloaded. Active: {:?}. Order: {:?}", guard.active_workspace_id, guard.ordered_workspace_ids);
            }
            Err(e) => {
                error!("Failed to load workspace config: {:?}. Creating default setup.", e);
                guard.workspaces.clear(); guard.ordered_workspace_ids.clear(); guard.active_workspace_id = None; guard.next_workspace_number = 1;
                let default_ws_id = guard.create_workspace_locked(None, None, None, None)?;
                guard.active_workspace_id = Some(default_ws_id);
                let _ = guard.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged { old_id: None, new_id: default_ws_id });
                if let Err(save_err) = guard.save_configuration().await {
                     error!("CRITICAL: Failed to save emergency default config: {:?}", save_err);
                     return Err(WorkspaceManagerError::ConfigError(save_err));
                }
            }
        }
        Ok(())
    }

    async fn create_workspace(&self, name: Option<String>, persistent_id: Option<String>, icon_name: Option<String>, accent_color_hex: Option<String>) -> Result<WorkspaceId, WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        let new_id = guard.create_workspace_locked(name, persistent_id, icon_name, accent_color_hex)?;
        if guard.active_workspace_id.is_none() {
            guard.active_workspace_id = Some(new_id);
            let _ = guard.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged { old_id: None, new_id });
        }
        guard.save_configuration().await?; Ok(new_id)
    }

    async fn delete_workspace(&self, id: WorkspaceId, fallback_id_for_windows: Option<WorkspaceId>) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        if guard.workspaces.len() <= 1 { return Err(WorkspaceManagerError::CannotDeleteLastWorkspace); }
        let ws_to_delete = guard.workspaces.get(&id).ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;

        if !ws_to_delete.window_ids().is_empty() && fallback_id_for_windows.is_none() {
            return Err(WorkspaceManagerError::DeleteRequiresFallbackForWindows { workspace_id: id, window_count: ws_to_delete.window_ids().len() });
        }
        let fallback_ws_id = if let Some(fallback_id) = fallback_id_for_windows {
            if !guard.workspaces.contains_key(&fallback_id) || fallback_id == id { return Err(WorkspaceManagerError::FallbackWorkspaceNotFound(fallback_id)); }
            Some(fallback_id)
        } else { None };
        
        let windows_to_move: Vec<WindowIdentifier> = ws_to_delete.window_ids().iter().cloned().collect();
        if let Some(target_fallback_id) = fallback_ws_id {
            // To avoid borrowing issues with HashMap, collect window IDs first, then iterate and modify.
            // The actual move operation needs careful handling of mutable borrows.
            // This simplified version assumes direct manipulation or a helper that can take parts of `guard`.
            for window_id in windows_to_move { // Iterate over a clone
                // Re-fetch target workspace mutably inside loop if necessary, or structure assignment carefully
                // For simplicity, assume assignment functions can handle this, or direct manipulation:
                guard.workspaces.get_mut(&id).unwrap().remove_window_id(&window_id); // Remove from deleting
                guard.workspaces.get_mut(&target_fallback_id).unwrap().add_window_id(window_id.clone()); // Add to fallback
                let _ = guard.event_publisher.send(WorkspaceEvent::WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData { workspace_id: id, window_id: window_id.clone() }));
                let _ = guard.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData { workspace_id: target_fallback_id, window_id }));
            }
        }

        guard.workspaces.remove(&id);
        guard.ordered_workspace_ids.retain(|ws_id| *ws_id != id);
        let old_active_id = guard.active_workspace_id;
        if guard.active_workspace_id == Some(id) {
            guard.active_workspace_id = guard.ordered_workspace_ids.first().cloned();
            if old_active_id != guard.active_workspace_id {
                 if let Some(new_active_id) = guard.active_workspace_id {
                    let _ = guard.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged { old_id: Some(id), new_id: new_active_id });
                 } else {
                    error!("No active workspace after deleting previously active one."); // Should be caught by len <= 1
                    let _ = guard.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged { old_id: Some(id), new_id: Uuid::nil() }); // Signal error/no active
                 }
            }
        }
        let _ = guard.event_publisher.send(WorkspaceEvent::WorkspaceDeleted { id, windows_moved_to_workspace_id: fallback_ws_id });
        let _ = guard.event_publisher.send(WorkspaceEvent::WorkspaceOrderChanged(guard.ordered_workspace_ids.clone()));
        guard.save_configuration().await?; Ok(())
    }

    fn get_workspace(&self, id: WorkspaceId) -> Option<Workspace> { self.internal.blocking_read().workspaces.get(&id).cloned() }
    fn all_workspaces_ordered(&self) -> Vec<Workspace> {
        let guard = self.internal.blocking_read();
        guard.ordered_workspace_ids.iter().filter_map(|id| guard.workspaces.get(id).cloned()).collect()
    }
    fn active_workspace_id(&self) -> Option<WorkspaceId> { self.internal.blocking_read().active_workspace_id }

    async fn set_active_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        if !guard.workspaces.contains_key(&id) { return Err(WorkspaceManagerError::SetActiveWorkspaceNotFound(id)); }
        let old_id = guard.active_workspace_id; if old_id == Some(id) { return Ok(()); }
        guard.active_workspace_id = Some(id);
        let _ = guard.event_publisher.send(WorkspaceEvent::ActiveWorkspaceChanged { old_id, new_id: id });
        guard.save_configuration().await?; Ok(())
    }

    async fn assign_window_to_active_workspace(&self, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        let active_id = guard.active_workspace_id.ok_or(WorkspaceManagerError::NoActiveWorkspace)?;
        assignment::assign_window_to_workspace(&mut guard.workspaces, active_id, window_id, guard.ensure_unique_window_assignment)?;
        let _ = guard.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData { workspace_id: active_id, window_id: window_id.clone() }));
        Ok(())
    }

    async fn assign_window_to_specific_workspace(&self, workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        if !guard.workspaces.contains_key(&workspace_id) { return Err(WorkspaceManagerError::WorkspaceNotFound(workspace_id)); }
        assignment::assign_window_to_workspace(&mut guard.workspaces, workspace_id, window_id, guard.ensure_unique_window_assignment)?;
        let _ = guard.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData { workspace_id, window_id: window_id.clone() }));
        Ok(())
    }

    async fn remove_window_from_its_workspace(&self, window_id: &WindowIdentifier) -> Result<Option<WorkspaceId>, WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        if let Some(source_ws_id) = assignment::find_workspace_for_window(&guard.workspaces, window_id) {
            assignment::remove_window_from_workspace(&mut guard.workspaces, source_ws_id, window_id)?;
            let _ = guard.event_publisher.send(WorkspaceEvent::WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData { workspace_id: source_ws_id, window_id: window_id.clone() }));
            Ok(Some(source_ws_id))
        } else { Ok(None) }
    }

    async fn move_window_to_specific_workspace(&self, target_workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        let source_workspace_id = assignment::find_workspace_for_window(&guard.workspaces, window_id).ok_or_else(|| WindowAssignmentError::WindowNotAssignedToWorkspace { workspace_id: Uuid::nil(), window_id: window_id.clone() })?;
        assignment::move_window_to_workspace(&mut guard.workspaces, source_workspace_id, target_workspace_id, window_id)?;
        let _ = guard.event_publisher.send(WorkspaceEvent::WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData { workspace_id: source_workspace_id, window_id: window_id.clone() }));
        let _ = guard.event_publisher.send(WorkspaceEvent::WindowAddedToWorkspace(WindowAddedToWorkspaceData { workspace_id: target_workspace_id, window_id: window_id.clone() }));
        Ok(())
    }

    async fn rename_workspace(&self, id: WorkspaceId, new_name: String) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        let ws = guard.workspaces.get_mut(&id).ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;
        let old_name = ws.name().to_string(); if old_name == new_name { return Ok(()); }
        ws.rename(new_name.clone())?;
        let _ = guard.event_publisher.send(WorkspaceEvent::WorkspaceRenamed(WorkspaceRenamedData { id, old_name, new_name }));
        guard.save_configuration().await?; Ok(())
    }

    async fn set_workspace_layout(&self, id: WorkspaceId, layout_type: WorkspaceLayoutType) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        let ws = guard.workspaces.get_mut(&id).ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;
        let old_layout = ws.layout_type(); if old_layout == layout_type { return Ok(()); }
        ws.set_layout_type(layout_type);
        let _ = guard.event_publisher.send(WorkspaceEvent::WorkspaceLayoutChanged(WorkspaceLayoutChangedData { id, old_layout, new_layout: layout_type }));
        guard.save_configuration().await?; Ok(())
    }
    
    async fn set_workspace_icon(&self, id: WorkspaceId, icon_name: Option<String>) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        let ws = guard.workspaces.get_mut(&id).ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;
        let old_icon_name = ws.icon_name().map(String::from); if old_icon_name == icon_name { return Ok(()); }
        ws.set_icon_name(icon_name.clone());
        let _ = guard.event_publisher.send(WorkspaceEvent::WorkspaceIconChanged(WorkspaceIconChangedData{id, old_icon_name, new_icon_name: icon_name}));
        guard.save_configuration().await?; Ok(())
    }

    async fn set_workspace_accent_color(&self, id: WorkspaceId, color_hex: Option<String>) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        let ws = guard.workspaces.get_mut(&id).ok_or(WorkspaceManagerError::WorkspaceNotFound(id))?;
        let old_color_hex = ws.accent_color_hex().map(String::from); if old_color_hex == color_hex { return Ok(()); }
        ws.set_accent_color_hex(color_hex.clone())?;
        let _ = guard.event_publisher.send(WorkspaceEvent::WorkspaceAccentChanged(WorkspaceAccentChangedData{id, old_color_hex, new_color_hex: color_hex}));
        guard.save_configuration().await?; Ok(())
    }

    async fn save_configuration(&self) -> Result<(), WorkspaceManagerError> {
        let guard = self.internal.lock().await;
        guard.save_configuration().await.map_err(WorkspaceManagerError::ConfigError)
    }

    fn subscribe_to_workspace_events(&self) -> broadcast::Receiver<WorkspaceEvent> {
        self.internal.blocking_read().event_publisher.subscribe()
    }

    async fn reorder_workspace(&self, workspace_id: WorkspaceId, new_index: usize) -> Result<(), WorkspaceManagerError> {
        let mut guard = self.internal.lock().await;
        let current_len = guard.ordered_workspace_ids.len();
        if new_index >= current_len { return Err(WorkspaceManagerError::InvalidWorkspaceIndex(new_index)); }
        let current_index = guard.ordered_workspace_ids.iter().position(|id| *id == workspace_id).ok_or(WorkspaceManagerError::WorkspaceNotFound(workspace_id))?;
        if current_index == new_index { return Ok(()); }
        let id_to_move = guard.ordered_workspace_ids.remove(current_index);
        guard.ordered_workspace_ids.insert(new_index, id_to_move);
        let _ = guard.event_publisher.send(WorkspaceEvent::WorkspaceOrderChanged(guard.ordered_workspace_ids.clone()));
        guard.save_configuration().await?; Ok(())
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspaces::config::MockWorkspaceConfigProvider;
    use tokio::sync::broadcast::error::RecvError;
    use crate::workspaces::core::DEFAULT_PERSISTENT_ID_PREFIX;


    fn create_test_manager(ensure_unique: bool) -> (DefaultWorkspaceManager, Arc<MockWorkspaceConfigProvider>) {
        let mock_provider = Arc::new(MockWorkspaceConfigProvider::new());
        let manager = DefaultWorkspaceManager::new(mock_provider.clone(), 32, ensure_unique);
        (manager, mock_provider)
    }
    
    #[tokio::test]
    async fn test_load_or_initialize_empty_config_creates_default() {
        let (manager, mock_provider) = create_test_manager(true);
        mock_provider.expect_load_workspace_config().times(1).returning(|| Ok(WorkspaceSetSnapshot::default()));
        mock_provider.expect_save_workspace_config().times(1).returning(|snap| {
            assert_eq!(snap.workspaces.len(), 1);
            assert_eq!(snap.workspaces[0].name, "Workspace 1");
            assert!(snap.workspaces[0].persistent_id.starts_with(DEFAULT_PERSISTENT_ID_PREFIX));
            assert_eq!(snap.active_workspace_persistent_id, Some(snap.workspaces[0].persistent_id.clone()));
            Ok(())
        });

        let mut event_rx = manager.subscribe_to_workspace_events();
        manager.load_or_initialize_workspaces().await.unwrap();

        let workspaces = manager.all_workspaces_ordered();
        assert_eq!(workspaces.len(), 1); assert_eq!(workspaces[0].name(), "Workspace 1");
        assert_eq!(manager.active_workspace_id(), Some(workspaces[0].id()));

        match tokio::time::timeout(std::time::Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WorkspaceCreated { name, .. } => assert_eq!(name, "Workspace 1"),
            e => panic!("Expected WorkspaceCreated, got {:?}", e),
        }
        match tokio::time::timeout(std::time::Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::ActiveWorkspaceChanged { new_id, .. } => assert_eq!(new_id, workspaces[0].id()),
            e => panic!("Expected ActiveWorkspaceChanged, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_load_or_initialize_with_existing_config() {
        let (manager, mock_provider) = create_test_manager(true);
        let ws1_pid = "ws1-pid".to_string(); let ws2_pid = "ws2-pid".to_string();
        let existing_snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { persistent_id: ws1_pid.clone(), name: "First WS".to_string(), ..Default::default() },
                WorkspaceSnapshot { persistent_id: ws2_pid.clone(), name: "Second WS".to_string(), ..Default::default() },
            ], active_workspace_persistent_id: Some(ws2_pid.clone()),
        };
        mock_provider.expect_load_workspace_config().times(1).returning(move || Ok(existing_snapshot.clone()));

        let mut event_rx = manager.subscribe_to_workspace_events();
        manager.load_or_initialize_workspaces().await.unwrap();

        let workspaces = manager.all_workspaces_ordered();
        assert_eq!(workspaces.len(), 2); assert_eq!(workspaces[0].name(), "First WS"); assert_eq!(workspaces[1].name(), "Second WS");
        let active_id = manager.active_workspace_id().unwrap();
        assert_eq!(manager.get_workspace(active_id).unwrap().persistent_id(), Some(ws2_pid.as_str()));

        match tokio::time::timeout(std::time::Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WorkspacesReloaded { new_order, active_workspace_id } => {
                assert_eq!(new_order.len(), 2); assert_eq!(active_workspace_id, Some(active_id));
            }, e => panic!("Expected WorkspacesReloaded, got {:?}", e),
        }
        match tokio::time::timeout(std::time::Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::ActiveWorkspaceChanged { new_id, .. } => assert_eq!(new_id, active_id),
            e => panic!("Expected ActiveWorkspaceChanged, got {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_create_delete_workspace() {
        let (manager, mock_provider) = create_test_manager(true);
        mock_provider.expect_load_workspace_config().times(1).returning(|| Ok(WorkspaceSetSnapshot::default()));
        mock_provider.expect_save_workspace_config().times(3).returning(|_| Ok(())); // Initial, Create, Delete
        manager.load_or_initialize_workspaces().await.unwrap();

        let ws2_id = manager.create_workspace(Some("My WS2".to_string()), None, None, None).await.unwrap();
        assert_eq!(manager.all_workspaces_ordered().len(), 2);
        assert_eq!(manager.get_workspace(ws2_id).unwrap().name(), "My WS2");

        let default_ws_id = manager.all_workspaces_ordered()[0].id();
        manager.delete_workspace(ws2_id, Some(default_ws_id)).await.unwrap();
        assert_eq!(manager.all_workspaces_ordered().len(), 1);
        assert!(manager.get_workspace(ws2_id).is_none());
    }

    #[tokio::test]
    async fn test_cannot_delete_last_workspace() {
        let (manager, mock_provider) = create_test_manager(true);
        mock_provider.expect_load_workspace_config().times(1).returning(|| Ok(WorkspaceSetSnapshot::default()));
        mock_provider.expect_save_workspace_config().times(1).returning(|_| Ok(()));
        manager.load_or_initialize_workspaces().await.unwrap();
        let last_ws_id = manager.all_workspaces_ordered()[0].id();
        assert!(matches!(manager.delete_workspace(last_ws_id, None).await, Err(WorkspaceManagerError::CannotDeleteLastWorkspace)));
    }
    
    #[tokio::test]
    async fn test_window_assignment_and_events() {
        let (manager, mock_provider) = create_test_manager(true);
        mock_provider.expect_load_workspace_config().times(1).returning(|| Ok(WorkspaceSetSnapshot::default()));
        mock_provider.expect_save_workspace_config().times(2).returning(|_| Ok(())); // Initial, Create WS2
        manager.load_or_initialize_workspaces().await.unwrap();

        let ws1_id = manager.active_workspace_id().unwrap();
        let win1 = WindowIdentifier::from("win1");
        let mut event_rx = manager.subscribe_to_workspace_events();
        // Clear initial events from load_or_initialize
        while let Ok(Ok(_)) = tokio::time::timeout(std::time::Duration::from_millis(1), event_rx.recv()).await {}


        manager.assign_window_to_active_workspace(&win1).await.unwrap();
        assert!(manager.get_workspace(ws1_id).unwrap().window_ids().contains(&win1));
        match tokio::time::timeout(std::time::Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
            WorkspaceEvent::WindowAddedToWorkspace(data) => { assert_eq!(data.workspace_id, ws1_id); assert_eq!(data.window_id, win1); },
            e => panic!("Expected WindowAddedToWorkspace, got {:?}", e),
        }
        
        let ws2_id = manager.create_workspace(Some("WS2".to_string()), None, None, None).await.unwrap();
        while let Ok(Ok(event)) = tokio::time::timeout(std::time::Duration::from_millis(1), event_rx.recv()).await {
            if matches!(event, WorkspaceEvent::WorkspaceCreated{id, ..} if id == ws2_id) { break; }
        } // Consume create event
        if manager.active_workspace_id() == Some(ws2_id) { // If active changed, consume that too
             while let Ok(Ok(event)) = tokio::time::timeout(std::time::Duration::from_millis(1), event_rx.recv()).await {
                if matches!(event, WorkspaceEvent::ActiveWorkspaceChanged{new_id, ..} if new_id == ws2_id ) { break; }
            }
        }


        manager.move_window_to_specific_workspace(ws2_id, &win1).await.unwrap();
        assert!(!manager.get_workspace(ws1_id).unwrap().window_ids().contains(&win1));
        assert!(manager.get_workspace(ws2_id).unwrap().window_ids().contains(&win1));

        let mut got_removed = false; let mut got_added = false;
        for _ in 0..2 {
             match tokio::time::timeout(std::time::Duration::from_millis(10), event_rx.recv()).await.unwrap().unwrap() {
                WorkspaceEvent::WindowRemovedFromWorkspace(data) => { assert_eq!(data.workspace_id, ws1_id); assert_eq!(data.window_id, win1); got_removed = true; },
                WorkspaceEvent::WindowAddedToWorkspace(data) => { assert_eq!(data.workspace_id, ws2_id); assert_eq!(data.window_id, win1); got_added = true; },
                e => panic!("Expected WindowRemoved or WindowAdded, got {:?}", e),
            }
        }
        assert!(got_removed && got_added);
    }
}
