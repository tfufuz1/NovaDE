use serde::{Deserialize, Serialize};
use super::types::{WorkspaceId, WorkspaceLayoutType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceRenamedData {
    pub id: WorkspaceId,
    pub old_name: String,
    pub new_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceLayoutChangedData {
    pub id: WorkspaceId,
    pub old_layout: WorkspaceLayoutType,
    pub new_layout: WorkspaceLayoutType,
}
