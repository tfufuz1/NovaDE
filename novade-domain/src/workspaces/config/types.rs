use serde::{Deserialize, Serialize};
use crate::workspaces::core::types::WorkspaceLayoutType; // Corrected path

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub name: String,
    pub layout_type: WorkspaceLayoutType,
    pub persistent_id: String, // Changed from Option<String> to String, must be present
    pub icon_name: Option<String>,
    pub accent_color_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceSetSnapshot {
    pub workspaces: Vec<WorkspaceSnapshot>,
    pub active_workspace_persistent_id: Option<String>, // Changed from active_workspace_index
}

impl WorkspaceSetSnapshot {
    /// Validates the snapshot.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(active_pid) = &self.active_workspace_persistent_id {
            if !self.workspaces.iter().any(|ws| ws.persistent_id == *active_pid) {
                return Err(format!(
                    "active_workspace_persistent_id '{}' does not match any workspace persistent_id in the set.",
                    active_pid
                ));
            }
        }
        for ws_snapshot in &self.workspaces {
            if ws_snapshot.persistent_id.is_empty() {
                return Err(format!("Workspace snapshot for '{}' has an empty persistent_id.", ws_snapshot.name));
            }
            // Basic validation for name (could reuse MAX_WORKSPACE_NAME_LENGTH from core::errors if needed here)
            if ws_snapshot.name.is_empty() {
                 return Err("Workspace snapshot name cannot be empty.".to_string());
            }
            // Accent color validation could be added here if desired, but core::Workspace handles it on set.
        }
        // Check for duplicate persistent_ids
        let mut pids = std::collections::HashSet::new();
        for ws_snapshot in &self.workspaces {
            if !pids.insert(&ws_snapshot.persistent_id) {
                return Err(format!("Duplicate persistent_id '{}' found in workspace set snapshot.", ws_snapshot.persistent_id));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_set_snapshot_default() {
        let snapshot = WorkspaceSetSnapshot::default();
        assert!(snapshot.workspaces.is_empty());
        assert!(snapshot.active_workspace_persistent_id.is_none());
    }

    #[test]
    fn workspace_set_snapshot_validate_valid_active_pid() {
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { name: "WS1".to_string(), layout_type: WorkspaceLayoutType::Floating, persistent_id: "pid1".to_string(), icon_name: None, accent_color_hex: None },
                WorkspaceSnapshot { name: "WS2".to_string(), layout_type: WorkspaceLayoutType::TilingVertical, persistent_id: "pid2".to_string(), icon_name: None, accent_color_hex: None },
            ],
            active_workspace_persistent_id: Some("pid2".to_string()),
        };
        assert!(snapshot.validate().is_ok());
    }

    #[test]
    fn workspace_set_snapshot_validate_invalid_active_pid() {
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { name: "WS1".to_string(), layout_type: WorkspaceLayoutType::Floating, persistent_id: "pid1".to_string(), icon_name: None, accent_color_hex: None },
            ],
            active_workspace_persistent_id: Some("non_existent_pid".to_string()),
        };
        assert!(snapshot.validate().is_err());
        assert!(snapshot.validate().unwrap_err().contains("does not match any workspace persistent_id"));
    }
    
    #[test]
    fn workspace_set_snapshot_validate_empty_persistent_id_in_snapshot() {
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { name: "WS1".to_string(), layout_type: WorkspaceLayoutType::Floating, persistent_id: "".to_string(), icon_name: None, accent_color_hex: None },
            ],
            active_workspace_persistent_id: None,
        };
        assert!(snapshot.validate().is_err());
        assert!(snapshot.validate().unwrap_err().contains("empty persistent_id"));
    }

    #[test]
    fn workspace_set_snapshot_validate_duplicate_persistent_ids() {
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { name: "WS1".to_string(), layout_type: WorkspaceLayoutType::Floating, persistent_id: "pid1".to_string(), icon_name: None, accent_color_hex: None },
                WorkspaceSnapshot { name: "WS2".to_string(), layout_type: WorkspaceLayoutType::TilingVertical, persistent_id: "pid1".to_string(), icon_name: None, accent_color_hex: None }, // Duplicate PID
            ],
            active_workspace_persistent_id: Some("pid1".to_string()),
        };
        assert!(snapshot.validate().is_err());
        assert!(snapshot.validate().unwrap_err().contains("Duplicate persistent_id"));
    }


    #[test]
    fn workspace_set_snapshot_validate_none_active_pid() {
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { name: "WS1".to_string(), layout_type: WorkspaceLayoutType::Floating, persistent_id: "pid1".to_string(), icon_name: None, accent_color_hex: None },
            ],
            active_workspace_persistent_id: None,
        };
        assert!(snapshot.validate().is_ok());
    }
}
