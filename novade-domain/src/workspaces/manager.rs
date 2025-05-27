use tokio::sync::broadcast;
use super::common_types::*;
use super::events::WorkspaceEvent;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[async_trait]
pub trait WorkspaceManager: Send + Sync {
    async fn list_workspaces(&self) -> Result<Vec<WorkspaceDescriptor>, WorkspaceError>;
    async fn set_active_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceError>;
    async fn get_active_workspace_id(&self) -> Result<Option<WorkspaceId>, WorkspaceError>; // Added for convenience
    fn subscribe_to_workspace_events(&self) -> Result<broadcast::Receiver<WorkspaceEvent>, WorkspaceError>;
    // Future methods:
    // async fn create_workspace(&self, name: String) -> Result<WorkspaceDescriptor, WorkspaceError>;
    // async fn delete_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceError>;
    // async fn rename_workspace(&self, id: WorkspaceId, new_name: String) -> Result<(), WorkspaceError>;
}

pub struct StubWorkspaceManager {
    workspaces: Mutex<Vec<WorkspaceDescriptor>>,
    active_id: Mutex<Option<WorkspaceId>>,
    event_sender: broadcast::Sender<WorkspaceEvent>,
}

impl StubWorkspaceManager {
    pub fn new() -> Self {
        let initial_workspaces = vec![
            WorkspaceDescriptor { id: "ws1".to_string(), name: "Desktop 1".to_string() },
            WorkspaceDescriptor { id: "ws2".to_string(), name: "Coding Space".to_string() },
            WorkspaceDescriptor { id: "ws3".to_string(), name: "Browser".to_string() },
            WorkspaceDescriptor { id: "ws4".to_string(), name: "Gaming".to_string() },
        ];
        let initial_active_id = Some("ws1".to_string());
        let (sender, _) = broadcast::channel(32); 
        Self {
            workspaces: Mutex::new(initial_workspaces),
            active_id: Mutex::new(initial_active_id),
            event_sender: sender,
        }
    }
}

#[async_trait]
impl WorkspaceManager for StubWorkspaceManager {
    async fn list_workspaces(&self) -> Result<Vec<WorkspaceDescriptor>, WorkspaceError> {
        Ok(self.workspaces.lock().expect("Mutex poisoned").clone())
    }

    async fn set_active_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceError> {
        let mut active_id_guard = self.active_id.lock().expect("Mutex poisoned");
        let old_id = active_id_guard.clone();
        
        // Check if workspace exists
        if !self.workspaces.lock().expect("Mutex poisoned").iter().any(|ws| ws.id == id) {
            return Err(WorkspaceError::NotFound(id));
        }

        *active_id_guard = Some(id.clone());
        
        if old_id.as_ref() != Some(&id) { // Only send event if changed
            if let Err(e) = self.event_sender.send(WorkspaceEvent::ActiveWorkspaceChanged { new_id: id, old_id }) {
                // Log error or handle, but don't necessarily make the operation fail for this stub
                eprintln!("Failed to send ActiveWorkspaceChanged event: {}", e);
            }
        }
        Ok(())
    }
    
    async fn get_active_workspace_id(&self) -> Result<Option<WorkspaceId>, WorkspaceError> {
        Ok(self.active_id.lock().expect("Mutex poisoned").clone())
    }

    fn subscribe_to_workspace_events(&self) -> Result<broadcast::Receiver<WorkspaceEvent>, WorkspaceError> {
        Ok(self.event_sender.subscribe())
    }
}

impl Default for StubWorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}
