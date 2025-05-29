use serde::{Deserialize, Serialize};
use crate::workspaces::core::WorkspaceLayoutType; // Ensure this path is correct

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub persistent_id: String,
    pub name: String,
    pub layout_type: WorkspaceLayoutType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent_color_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WorkspaceSetSnapshot {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workspaces: Vec<WorkspaceSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_workspace_persistent_id: Option<String>,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_snapshot_serde() {
        let snapshot = WorkspaceSnapshot {
            persistent_id: "pid1".to_string(),
            name: "Test Workspace 1".to_string(),
            layout_type: WorkspaceLayoutType::TilingVertical,
            icon_name: Some("icon-arch".to_string()),
            accent_color_hex: Some("#FF00FF".to_string()),
        };
        let serialized = serde_json::to_string_pretty(&snapshot).unwrap();
        let deserialized: WorkspaceSnapshot = serde_json::from_str(&serialized).unwrap();
        assert_eq!(snapshot, deserialized);
    }

    #[test]
    fn workspace_snapshot_serde_optional_fields_none() {
        let snapshot = WorkspaceSnapshot {
            persistent_id: "pid2".to_string(),
            name: "Test Workspace 2".to_string(),
            layout_type: WorkspaceLayoutType::Floating,
            icon_name: None,
            accent_color_hex: None,
        };
        let serialized = serde_json::to_string_pretty(&snapshot).unwrap();
        assert!(!serialized.contains("icon_name"));
        assert!(!serialized.contains("accent_color_hex"));

        let deserialized: WorkspaceSnapshot = serde_json::from_str(&serialized).unwrap();
        assert_eq!(snapshot, deserialized);
        assert_eq!(deserialized.icon_name, None);
        assert_eq!(deserialized.accent_color_hex, None);
    }

    #[test]
    fn workspace_set_snapshot_default() {
        let default_snapshot = WorkspaceSetSnapshot::default();
        assert!(default_snapshot.workspaces.is_empty());
        assert_eq!(default_snapshot.active_workspace_persistent_id, None);
    }

    #[test]
    fn workspace_set_snapshot_serde() {
        let set_snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot {
                    persistent_id: "main".to_string(),
                    name: "Main".to_string(),
                    layout_type: WorkspaceLayoutType::Maximized,
                    icon_name: None,
                    accent_color_hex: None,
                },
                WorkspaceSnapshot {
                    persistent_id: "dev".to_string(),
                    name: "Development".to_string(),
                    layout_type: WorkspaceLayoutType::TilingHorizontal,
                    icon_name: Some("code-icon".to_string()),
                    accent_color_hex: None,
                },
            ],
            active_workspace_persistent_id: Some("main".to_string()),
        };
        let serialized = serde_json::to_string_pretty(&set_snapshot).unwrap();
        let deserialized: WorkspaceSetSnapshot = serde_json::from_str(&serialized).unwrap();
        assert_eq!(set_snapshot, deserialized);
    }
    
    #[test]
    fn workspace_set_snapshot_serde_empty_and_none() {
        let set_snapshot = WorkspaceSetSnapshot::default(); 
        let serialized = serde_json::to_string_pretty(&set_snapshot).unwrap();
        
        let deserialized: WorkspaceSetSnapshot = serde_json::from_str(&serialized).unwrap();
        assert_eq!(set_snapshot, deserialized);
        assert!(deserialized.workspaces.is_empty());
        assert!(deserialized.active_workspace_persistent_id.is_none());

        // Check that optional fields are indeed omitted or handled as default by serde
        // For a struct that is `Default` and has `skip_serializing_if` on all fields,
        // it might serialize to an empty object `{}`.
        // If `workspaces: Vec` uses `#[serde(default, skip_serializing_if = "Vec::is_empty")]`,
        // it will be omitted if empty.
        // If `active_workspace_persistent_id: Option<String>` uses `#[serde(default, skip_serializing_if = "Option::is_none")]`,
        // it will be omitted if None.
        // Thus, a default WorkspaceSetSnapshot should serialize to "{}"
        assert_eq!(serialized, "{}");
    }
}
