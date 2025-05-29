use serde::{Deserialize, Serialize};
use crate::workspaces::core::{
    WorkspaceId,
    event_data::{ // Corrected path to event_data
        WorkspaceRenamedData, WorkspaceLayoutChangedData, WindowAddedToWorkspaceData,
        WindowRemovedFromWorkspaceData, WorkspacePersistentIdChangedData,
        WorkspaceIconChangedData, WorkspaceAccentChangedData,
    }
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkspaceEvent {
    WorkspaceCreated {
        id: WorkspaceId,
        name: String,
        persistent_id: Option<String>,
        position: usize, 
        icon_name: Option<String>,
        accent_color_hex: Option<String>,
    },
    WorkspaceDeleted {
        id: WorkspaceId,
        windows_moved_to_workspace_id: Option<WorkspaceId>,
    },
    ActiveWorkspaceChanged {
        old_id: Option<WorkspaceId>,
        new_id: WorkspaceId,
    },
    WorkspaceRenamed(WorkspaceRenamedData),
    WorkspaceLayoutChanged(WorkspaceLayoutChangedData),
    WindowAddedToWorkspace(WindowAddedToWorkspaceData),
    WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData),
    WorkspaceOrderChanged(Vec<WorkspaceId>), 
    WorkspacesReloaded {
        new_order: Vec<WorkspaceId>,
        active_workspace_id: Option<WorkspaceId>,
    },
    WorkspacePersistentIdChanged(WorkspacePersistentIdChangedData),
    WorkspaceIconChanged(WorkspaceIconChangedData),
    WorkspaceAccentChanged(WorkspaceAccentChangedData),
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspaces::core::types::{WindowIdentifier, WorkspaceLayoutType};
    use uuid::Uuid;

    #[test]
    fn workspace_event_created_serde() {
        let event = WorkspaceEvent::WorkspaceCreated {
            id: Uuid::new_v4(),
            name: "Test WS".to_string(),
            persistent_id: Some("test-ws-pid".to_string()),
            position: 0,
            icon_name: Some("icon".to_string()),
            accent_color_hex: Some("#123456".to_string()),
        };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: WorkspaceEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn workspace_event_deleted_serde() {
        let event = WorkspaceEvent::WorkspaceDeleted {
            id: Uuid::new_v4(),
            windows_moved_to_workspace_id: Some(Uuid::new_v4()),
        };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: WorkspaceEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn workspace_event_active_changed_serde() {
        let event = WorkspaceEvent::ActiveWorkspaceChanged {
            old_id: Some(Uuid::new_v4()),
            new_id: Uuid::new_v4(),
        };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: WorkspaceEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn workspace_event_renamed_serde() {
        let data = WorkspaceRenamedData { id: Uuid::new_v4(), old_name: "Old".to_string(), new_name: "New".to_string() };
        let event = WorkspaceEvent::WorkspaceRenamed(data.clone());
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: WorkspaceEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }
    
    #[test]
    fn workspace_event_layout_changed_serde() {
        let data = WorkspaceLayoutChangedData { 
            id: Uuid::new_v4(), 
            old_layout: WorkspaceLayoutType::Floating, 
            new_layout: WorkspaceLayoutType::TilingHorizontal 
        };
        let event = WorkspaceEvent::WorkspaceLayoutChanged(data.clone());
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: WorkspaceEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn workspace_event_window_added_serde() {
        let data = WindowAddedToWorkspaceData {
            workspace_id: Uuid::new_v4(),
            window_id: WindowIdentifier::from("win-1"),
        };
        let event = WorkspaceEvent::WindowAddedToWorkspace(data.clone());
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: WorkspaceEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }
    
    #[test]
    fn workspace_event_order_changed_serde() {
        let event = WorkspaceEvent::WorkspaceOrderChanged(vec![Uuid::new_v4(), Uuid::new_v4()]);
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: WorkspaceEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn workspace_event_reloaded_serde() {
        let event = WorkspaceEvent::WorkspacesReloaded {
            new_order: vec![Uuid::new_v4(), Uuid::new_v4()],
            active_workspace_id: Some(Uuid::new_v4()),
        };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: WorkspaceEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }
}
