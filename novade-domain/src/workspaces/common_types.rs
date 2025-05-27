use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type WorkspaceId = String; // Using String for simplicity for now

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)] // Added PartialEq, Eq for easier testing/comparison
pub struct WorkspaceDescriptor {
    pub id: WorkspaceId,
    pub name: String,
    // pub icon_name: Option<String>, // Example of another field
    // pub layout_hint: Option<String>, // Example
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("Workspace not found: {0}")]
    NotFound(WorkspaceId),
    #[error("Workspace already exists: {0}")] // Added for completeness
    AlreadyExists(WorkspaceId),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Operation failed: {0}")] // Generic failure
    OperationFailed(String),
}
