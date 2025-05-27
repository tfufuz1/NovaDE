use tokio::sync::broadcast;
use super::common_types::*;
use super::events::WorkspaceEvent;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[async_trait]
pub trait WorkspaceManager: Send + Sync {
    async fn list_workspaces(&self) -> Result<Vec<WorkspaceDescriptor>, WorkspaceError>;
    async fn set_active_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceError>;
    async fn get_active_workspace_id(&self) -> Result<Option<WorkspaceId>, WorkspaceError>;
    fn subscribe_to_workspace_events(&self) -> Result<broadcast::Receiver<WorkspaceEvent>, WorkspaceError>;
    async fn create_workspace(&self, name: String) -> Result<WorkspaceDescriptor, WorkspaceError>;
    async fn delete_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceError>;
    // async fn rename_workspace(&self, id: WorkspaceId, new_name: String) -> Result<(), WorkspaceError>; // Future
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

    async fn create_workspace(&self, name: String) -> Result<WorkspaceDescriptor, WorkspaceError> {
        let mut workspaces_guard = self.workspaces.lock().expect("Mutex poisoned for workspaces");
        // Simple ID generation for stub, ensure it's somewhat unique for the session
        // Find the highest current number in IDs like "wsX"
        let mut max_id_num = 0;
        for ws in workspaces_guard.iter() {
            if ws.id.starts_with("ws") {
                if let Ok(num) = ws.id[2..].parse::<usize>() {
                    if num > max_id_num {
                        max_id_num = num;
                    }
                }
            }
        }
        let new_id = format!("ws{}", max_id_num + 1);

        // Check for name collision (optional for stub, but good practice)
        if workspaces_guard.iter().any(|ws| ws.name == name) {
            // For a more robust stub, you might return an error here
            // return Err(WorkspaceError::Internal("Name already exists".to_string()));
            // Or modify the name slightly for the stub to avoid error
            // name = format!("{}-{}", name, new_id); 
        }

        let descriptor = WorkspaceDescriptor {
            id: new_id.clone(),
            name,
        };

        workspaces_guard.push(descriptor.clone());

        // Drop the lock before sending the event to avoid potential deadlocks
        // if a subscriber immediately calls back into the manager.
        drop(workspaces_guard); 

        if let Err(e) = self.event_sender.send(WorkspaceEvent::WorkspaceCreated { descriptor: descriptor.clone() }) {
             eprintln!("Failed to send WorkspaceCreated event: {}", e); // Log error
             // Not returning error for the operation itself for this stub, to keep it simpler
        }
        Ok(descriptor)
    }

    async fn delete_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceError> {
        let mut workspaces_guard = self.workspaces.lock().expect("Mutex poisoned for workspaces");
        let initial_len = workspaces_guard.len();
        
        let mut deleted_descriptor: Option<WorkspaceDescriptor> = None;
        workspaces_guard.retain(|ws| {
            if ws.id == id {
                deleted_descriptor = Some(ws.clone());
                false // Remove
            } else {
                true // Keep
            }
        });

        if workspaces_guard.len() == initial_len {
            return Err(WorkspaceError::NotFound(id));
        }
        
        // If the active workspace was deleted, update active_id and prepare event
        let mut active_id_guard = self.active_id.lock().expect("Mutex poisoned for active_id");
        let mut active_changed_event: Option<WorkspaceEvent> = None;

        if *active_id_guard == Some(id.clone()) {
            let old_active_id = active_id_guard.take(); // Becomes None
            
            // Set a new active workspace if possible (e.g., first in list)
            if let Some(new_active_descriptor) = workspaces_guard.first() {
                *active_id_guard = Some(new_active_descriptor.id.clone());
                active_changed_event = Some(WorkspaceEvent::ActiveWorkspaceChanged {
                    new_id: new_active_descriptor.id.clone(),
                    old_id, // old_active_id was Some(id)
                });
            } else { 
                // No workspaces left, active_id_guard is already None
                active_changed_event = Some(WorkspaceEvent::ActiveWorkspaceChanged {
                    // Use a more distinct representation for "no active workspace"
                    // For now, an empty string, but `Option<WorkspaceId>` handled by `None` is better.
                    // The event itself could carry Option<WorkspaceId> for new_id.
                    // Let's assume the event structure is fixed for now as new_id: WorkspaceId.
                    // This implies there should always be an active workspace if any exist.
                    // If no workspaces exist, active_id_guard is None and this specific event part might be skipped
                    // or new_id should be Option<WorkspaceId> in the event too.
                    // For this stub, if no workspaces left, old_id had a value, new_id might be problematic.
                    // Let's assume for now that if the list is empty, `active_id_guard` remains `None`
                    // and the primary event is WorkspaceDeleted.
                    // If the list is NOT empty, and active was deleted, we picked a new one.
                    // If the list becomes empty, active_id_guard is None.
                    // The `ActiveWorkspaceChanged` event should reflect this.
                    // If `new_id` in `ActiveWorkspaceChanged` cannot be `Option`, then this is tricky.
                    // Let's ensure old_active_id (which is Some(id)) is passed.
                    // If new active is None, and event structure requires a WorkspaceId for new_id:
                    // One option: don't send ActiveWorkspaceChanged if new active is None, rely on UI to check active after delete.
                    // Another option: define a sentinel ID for "no active workspace".
                    // For this stub, if *active_id_guard is now None, we don't set a new_id in ActiveWorkspaceChanged here.
                    // The fact that active_id_guard is None is the state.
                    // The event listener in connector will fetch active_id (which will be None)
                    // and update UI accordingly.
                    // So, let's refine the active_changed_event logic:
                    new_id: active_id_guard.clone().unwrap_or_else(|| "".to_string()), // If None, use empty.
                    old_id,
                });
            }
        }
        
        // Drop locks before sending events
        drop(workspaces_guard);
        drop(active_id_guard);

        if let Err(e) = self.event_sender.send(WorkspaceEvent::WorkspaceDeleted { id: id.clone() }) {
            eprintln!("Failed to send WorkspaceDeleted event: {}", e);
        }

        if let Some(event) = active_changed_event {
            if let Err(e) = self.event_sender.send(event) {
                 eprintln!("Failed to send ActiveWorkspaceChanged event after delete: {}", e);
            }
        }
        Ok(())
    }
}

impl Default for StubWorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}
