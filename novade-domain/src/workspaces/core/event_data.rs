use serde::{Deserialize, Serialize};
use super::types::{WorkspaceId, WorkspaceLayoutType, WindowIdentifier}; // Added WindowIdentifier

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceIconChangedData {
    pub id: WorkspaceId,
    pub old_icon_name: Option<String>,
    pub new_icon_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceAccentChangedData {
    pub id: WorkspaceId,
    pub old_color_hex: Option<String>,
    pub new_color_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowAddedToWorkspaceData {
    pub workspace_id: WorkspaceId,
    pub window_id: WindowIdentifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowRemovedFromWorkspaceData {
    pub workspace_id: WorkspaceId,
    pub window_id: WindowIdentifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspacePersistentIdChangedData {
    pub id: WorkspaceId, // The runtime UUID
    pub old_persistent_id: Option<String>,
    pub new_persistent_id: Option<String>,
}
