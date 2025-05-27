use thiserror::Error;
use crate::workspaces::core::types::{WorkspaceId, WindowIdentifier}; // Corrected path

#[derive(Error, Debug)]
pub enum WorkspaceManagerError {
    #[error("Workspace with ID '{0}' not found.")]
    WorkspaceNotFound(WorkspaceId),

    #[error("Failed to set active workspace to '{0}': workspace not found.")]
    SetActiveWorkspaceNotFound(WorkspaceId),

    #[error("No active workspace is currently set.")]
    NoActiveWorkspace,
    
    #[error("Cannot delete the last workspace.")]
    CannotDeleteLastWorkspace,

    #[error("Workspace '{workspace_id}' contains {window_count} windows. A fallback workspace ID must be provided to delete it.")]
    DeleteRequiresFallbackForWindows {
        workspace_id: WorkspaceId,
        window_count: usize,
    },

    #[error("Fallback workspace with ID '{0}' not found during delete operation.")]
    FallbackWorkspaceNotFound(WorkspaceId),

    #[error("A workspace with persistent ID '{0}' already exists.")]
    DuplicatePersistentId(String),
    
    #[error("Window '{0}' not found in any workspace.")]
    WindowNotAssigned(WindowIdentifier),

    #[error("Workspace core error: {0}")]
    CoreError(#[from] crate::workspaces::core::errors::WorkspaceCoreError),

    #[error("Workspace configuration error: {0}")]
    ConfigError(#[from] crate::workspaces::config::errors::WorkspaceConfigError),

    #[error("Window assignment error: {0}")]
    AssignmentError(#[from] crate::workspaces::assignment::errors::WindowAssignmentError),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String), // For general operational errors, e.g., reordering out of bounds

    #[error("Internal workspace manager error: {context}")]
    Internal { context: String },
}
