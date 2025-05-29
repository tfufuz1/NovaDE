use serde::{Deserialize, Serialize};
use super::types::{WorkspaceId, WindowIdentifier, WorkspaceLayoutType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceRenamedData {
    pub id: WorkspaceId,
    pub old_name: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceLayoutChangedData {
    pub id: WorkspaceId,
    pub old_layout: WorkspaceLayoutType,
    pub new_layout: WorkspaceLayoutType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowAddedToWorkspaceData {
    pub workspace_id: WorkspaceId,
    pub window_id: WindowIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowRemovedFromWorkspaceData {
    pub workspace_id: WorkspaceId,
    pub window_id: WindowIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspacePersistentIdChangedData {
    pub id: WorkspaceId,
    pub old_persistent_id: Option<String>,
    pub new_persistent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceIconChangedData {
    pub id: WorkspaceId,
    pub old_icon_name: Option<String>,
    pub new_icon_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceAccentChangedData {
    pub id: WorkspaceId,
    pub old_color_hex: Option<String>,
    pub new_color_hex: Option<String>,
}


#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_workspace_renamed_data_serde() {
        let data = WorkspaceRenamedData {
            id: Uuid::new_v4(),
            old_name: "Old Name".to_string(),
            new_name: "New Name".to_string(),
        };
        let serialized = serde_json::to_string(&data).unwrap();
        let deserialized: WorkspaceRenamedData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_workspace_layout_changed_data_serde() {
        let data = WorkspaceLayoutChangedData {
            id: Uuid::new_v4(),
            old_layout: WorkspaceLayoutType::Floating,
            new_layout: WorkspaceLayoutType::TilingHorizontal,
        };
        let serialized = serde_json::to_string(&data).unwrap();
        let deserialized: WorkspaceLayoutChangedData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_window_added_to_workspace_data_serde() {
        let data = WindowAddedToWorkspaceData {
            workspace_id: Uuid::new_v4(),
            window_id: WindowIdentifier::from("win-123"),
        };
        let serialized = serde_json::to_string(&data).unwrap();
        let deserialized: WindowAddedToWorkspaceData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_window_removed_from_workspace_data_serde() {
        let data = WindowRemovedFromWorkspaceData {
            workspace_id: Uuid::new_v4(),
            window_id: WindowIdentifier::from("win-456"),
        };
        let serialized = serde_json::to_string(&data).unwrap();
        let deserialized: WindowRemovedFromWorkspaceData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(data, deserialized);
    }
    
    #[test]
    fn test_workspace_persistent_id_changed_data_serde() {
        let data = WorkspacePersistentIdChangedData {
            id: Uuid::new_v4(),
            old_persistent_id: Some("old-pid".to_string()),
            new_persistent_id: Some("new-pid".to_string()),
        };
        let serialized = serde_json::to_string(&data).unwrap();
        let deserialized: WorkspacePersistentIdChangedData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(data, deserialized);

        let data_none = WorkspacePersistentIdChangedData {
            id: Uuid::new_v4(),
            old_persistent_id: None,
            new_persistent_id: Some("new-pid-from-none".to_string()),
        };
        let serialized_none = serde_json::to_string(&data_none).unwrap();
        let deserialized_none: WorkspacePersistentIdChangedData = serde_json::from_str(&serialized_none).unwrap();
        assert_eq!(data_none, deserialized_none);
    }

    #[test]
    fn test_workspace_icon_changed_data_serde() {
        let data = WorkspaceIconChangedData {
            id: Uuid::new_v4(),
            old_icon_name: Some("old-icon".to_string()),
            new_icon_name: Some("new-icon".to_string()),
        };
        let serialized = serde_json::to_string(&data).unwrap();
        let deserialized: WorkspaceIconChangedData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(data, deserialized);
    }
    
    #[test]
    fn test_workspace_accent_changed_data_serde() {
        let data = WorkspaceAccentChangedData {
            id: Uuid::new_v4(),
            old_color_hex: Some("#112233".to_string()),
            new_color_hex: Some("#AABBCC".to_string()),
        };
        let serialized = serde_json::to_string(&data).unwrap();
        let deserialized: WorkspaceAccentChangedData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(data, deserialized);
    }
}
