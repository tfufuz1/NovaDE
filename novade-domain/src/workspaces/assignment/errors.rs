use thiserror::Error;
use crate::workspaces::core::{WorkspaceId, WindowIdentifier}; // Corrected path

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WindowAssignmentError {
    #[error("Workspace with ID '{0}' not found.")]
    WorkspaceNotFound(WorkspaceId),

    #[error("Window '{window_id}' is already assigned to workspace '{workspace_id}'.")]
    WindowAlreadyAssigned { // This might be more of a notice than an error depending on usage
        workspace_id: WorkspaceId,
        window_id: WindowIdentifier,
    },

    #[error("Window '{window_id}' is not assigned to workspace '{workspace_id}'.")]
    WindowNotAssignedToWorkspace {
        workspace_id: WorkspaceId,
        window_id: WindowIdentifier,
    },

    #[error("Source workspace with ID '{0}' not found for move operation.")]
    SourceWorkspaceNotFound(WorkspaceId),

    #[error("Target workspace with ID '{0}' not found for move operation.")]
    TargetWorkspaceNotFound(WorkspaceId),

    #[error("Window '{window_id}' not found on source workspace '{workspace_id}' during move operation.")]
    WindowNotOnSourceWorkspace {
        workspace_id: WorkspaceId,
        window_id: WindowIdentifier,
    },

    #[error("Cannot move window '{window_id}' to the same workspace '{workspace_id}'.")]
    CannotMoveToSameWorkspace {
        workspace_id: WorkspaceId,
        window_id: WindowIdentifier,
    },

    #[error("Assignment rule violation: {reason}. Window: {window_id:?}, Target Workspace: {target_workspace_id:?}")]
    RuleViolation {
        reason: String,
        window_id: Option<WindowIdentifier>,
        target_workspace_id: Option<WorkspaceId>,
    },

    #[error("Internal error in window assignment: {context}")]
    Internal { context: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_error_messages() {
        let ws_id = Uuid::new_v4();
        let win_id = WindowIdentifier::from("win1");

        assert_eq!(
            format!("{}", WindowAssignmentError::WorkspaceNotFound(ws_id)),
            format!("Workspace with ID '{}' not found.", ws_id)
        );
        assert_eq!(
            format!("{}", WindowAssignmentError::WindowAlreadyAssigned { workspace_id: ws_id, window_id: win_id.clone() }),
            format!("Window 'win1' is already assigned to workspace '{}'.", ws_id)
        );
        assert_eq!(
            format!("{}", WindowAssignmentError::WindowNotAssignedToWorkspace { workspace_id: ws_id, window_id: win_id.clone() }),
            format!("Window 'win1' is not assigned to workspace '{}'.", ws_id)
        );
        assert_eq!(
            format!("{}", WindowAssignmentError::SourceWorkspaceNotFound(ws_id)),
            format!("Source workspace with ID '{}' not found for move operation.", ws_id)
        );
        assert_eq!(
            format!("{}", WindowAssignmentError::TargetWorkspaceNotFound(ws_id)),
            format!("Target workspace with ID '{}' not found for move operation.", ws_id)
        );
        assert_eq!(
            format!("{}", WindowAssignmentError::WindowNotOnSourceWorkspace { workspace_id: ws_id, window_id: win_id.clone() }),
            format!("Window 'win1' not found on source workspace '{}' during move operation.", ws_id)
        );
        assert_eq!(
            format!("{}", WindowAssignmentError::CannotMoveToSameWorkspace { workspace_id: ws_id, window_id: win_id.clone() }),
            format!("Cannot move window 'win1' to the same workspace '{}'.", ws_id)
        );
        assert_eq!(
            format!("{}", WindowAssignmentError::RuleViolation { reason: "Test rule".to_string(), window_id: Some(win_id.clone()), target_workspace_id: Some(ws_id) }),
            format!("Assignment rule violation: Test rule. Window: Some(WindowIdentifier(\"win1\")), Target Workspace: Some({})", ws_id)
        );
         assert_eq!(
            format!("{}", WindowAssignmentError::Internal { context: "Oops".to_string() }),
            "Internal error in window assignment: Oops"
        );
    }
}
