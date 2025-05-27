use thiserror::Error;
use crate::workspaces::core::types::{WindowIdentifier, WorkspaceId};

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum WindowAssignmentError {
    #[error("Workspace with ID '{0}' not found.")]
    WorkspaceNotFound(WorkspaceId),

    #[error("Window '{window_id}' is already assigned to workspace '{workspace_id}'.")]
    WindowAlreadyAssigned {
        workspace_id: WorkspaceId,
        window_id: WindowIdentifier,
    },
    
    #[error("Window '{window_id}' not found in workspace '{workspace_id}' for removal.")]
    WindowNotFoundInWorkspace {
        workspace_id: WorkspaceId,
        window_id: WindowIdentifier,
    },

    #[error("Assignment rule violation: {reason}. Window: {window_id:?}, Target Workspace: {target_workspace_id:?}")]
    RuleViolation { 
        reason: String, 
        window_id: Option<WindowIdentifier>, 
        target_workspace_id: Option<WorkspaceId> 
    },

    #[error("Internal window assignment error: {context}")]
    Internal { context: String },
}
