use thiserror::Error;
use crate::workspaces::core::WorkspaceId;
use crate::workspaces::core::errors::WorkspaceCoreError;
use crate::workspaces::assignment::errors::WindowAssignmentError;
use crate::workspaces::config::errors::WorkspaceConfigError;

#[derive(Debug, Error)]
pub enum WorkspaceManagerError {
    #[error("Workspace with ID '{0}' not found.")]
    WorkspaceNotFound(WorkspaceId),

    #[error("Cannot delete the last workspace.")]
    CannotDeleteLastWorkspace,

    #[error("Cannot delete workspace '{workspace_id}' as it contains {window_count} windows. A fallback workspace ID must be provided.")]
    DeleteRequiresFallbackForWindows {
        workspace_id: WorkspaceId,
        window_count: usize,
    },

    #[error("Fallback workspace with ID '{0}' not found during delete operation.")]
    FallbackWorkspaceNotFound(WorkspaceId),

    #[error("Core workspace error: {0}")]
    CoreError(#[from] WorkspaceCoreError),

    #[error("Window assignment error: {0}")]
    AssignmentError(#[from] WindowAssignmentError),

    #[error("Workspace configuration error: {0}")]
    ConfigError(#[from] WorkspaceConfigError),

    #[error("Workspace with ID '{0}' not found when trying to set it active.")]
    SetActiveWorkspaceNotFound(WorkspaceId),

    #[error("No active workspace is set.")]
    NoActiveWorkspace,

    #[error("A workspace with persistent ID '{0}' already exists.")]
    DuplicatePersistentId(String),

    #[error("Invalid workspace index: {0}. Must be within the current range of workspaces.")]
    InvalidWorkspaceIndex(usize),

    #[error("Internal error in workspace manager: {context}")]
    Internal { context: String },
}


#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use crate::workspaces::core::WindowIdentifier; 
    use novade_core::errors::CoreError as NovadeCoreError; // Alias to avoid confusion if needed

    #[test]
    fn test_error_messages() {
        let ws_id = Uuid::new_v4();
        let win_id = WindowIdentifier::from("win-test");

        assert_eq!(
            format!("{}", WorkspaceManagerError::WorkspaceNotFound(ws_id)),
            format!("Workspace with ID '{}' not found.", ws_id)
        );
        assert_eq!(
            format!("{}", WorkspaceManagerError::CannotDeleteLastWorkspace),
            "Cannot delete the last workspace."
        );
        assert_eq!(
            format!("{}", WorkspaceManagerError::DeleteRequiresFallbackForWindows { workspace_id: ws_id, window_count: 3 }),
            format!("Cannot delete workspace '{}' as it contains 3 windows. A fallback workspace ID must be provided.", ws_id)
        );
        assert_eq!(
            format!("{}", WorkspaceManagerError::FallbackWorkspaceNotFound(ws_id)),
            format!("Fallback workspace with ID '{}' not found during delete operation.", ws_id)
        );
        
        let core_err = WorkspaceCoreError::NameCannotBeEmpty;
        assert_eq!(format!("{}", WorkspaceManagerError::CoreError(core_err)), "Core workspace error: Workspace name cannot be empty.");

        let assign_err = WindowAssignmentError::WindowAlreadyAssigned { workspace_id: ws_id, window_id: win_id };
        assert!(format!("{}", WorkspaceManagerError::AssignmentError(assign_err)).starts_with("Window assignment error:"));

        let dummy_core_error = NovadeCoreError::ConfigError("dummy core config error".to_string());
        let config_err = WorkspaceConfigError::LoadError { path: "cfg_path".to_string(), source: dummy_core_error };
        assert!(format!("{}", WorkspaceManagerError::ConfigError(config_err)).starts_with("Workspace configuration error:"));

        assert_eq!(
            format!("{}", WorkspaceManagerError::SetActiveWorkspaceNotFound(ws_id)),
            format!("Workspace with ID '{}' not found when trying to set it active.", ws_id)
        );
        assert_eq!(
            format!("{}", WorkspaceManagerError::NoActiveWorkspace),
            "No active workspace is set."
        );
        assert_eq!(
            format!("{}", WorkspaceManagerError::DuplicatePersistentId("pid-exists".to_string())),
            "A workspace with persistent ID 'pid-exists' already exists."
        );
        assert_eq!(
            format!("{}", WorkspaceManagerError::InvalidWorkspaceIndex(5)),
            "Invalid workspace index: 5. Must be within the current range of workspaces."
        );
        assert_eq!(
            format!("{}", WorkspaceManagerError::Internal { context: "Critical failure".to_string() }),
            "Internal error in workspace manager: Critical failure"
        );
    }
}
