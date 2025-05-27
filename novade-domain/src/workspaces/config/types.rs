use serde::{Deserialize, Serialize};
use crate::workspaces::core::types::WorkspaceLayoutType; // Corrected path

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub name: String,
    pub layout_type: WorkspaceLayoutType,
    // pub persistent_id: Option<String>, // Not in Iteration 1
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceSetSnapshot {
    pub workspaces: Vec<WorkspaceSnapshot>,
    pub active_workspace_index: Option<usize>,
}

impl WorkspaceSetSnapshot {
    /// Validates the snapshot.
    /// For now, it checks if active_workspace_index is valid if Some.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(index) = self.active_workspace_index {
            if index >= self.workspaces.len() {
                return Err(format!(
                    "active_workspace_index {} is out of bounds for {} workspaces.",
                    index,
                    self.workspaces.len()
                ));
            }
        }
        // Could add validation for individual WorkspaceSnapshots here if needed (e.g., name checks)
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
        assert!(snapshot.active_workspace_index.is_none());
    }

    #[test]
    fn workspace_set_snapshot_validate_valid_index() {
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { name: "WS1".to_string(), layout_type: WorkspaceLayoutType::Floating },
                WorkspaceSnapshot { name: "WS2".to_string(), layout_type: WorkspaceLayoutType::TilingVertical },
            ],
            active_workspace_index: Some(1),
        };
        assert!(snapshot.validate().is_ok());
    }

    #[test]
    fn workspace_set_snapshot_validate_invalid_index() {
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { name: "WS1".to_string(), layout_type: WorkspaceLayoutType::Floating },
            ],
            active_workspace_index: Some(1), // Index 1 is out of bounds
        };
        assert!(snapshot.validate().is_err());
    }

    #[test]
    fn workspace_set_snapshot_validate_none_index() {
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { name: "WS1".to_string(), layout_type: WorkspaceLayoutType::Floating },
            ],
            active_workspace_index: None,
        };
        assert!(snapshot.validate().is_ok());
    }
}
