use serde::{Deserialize, Serialize};
use crate::workspaces::core::types::WorkspaceId; // Corrected path
use crate::workspaces::core::event_data::{ // Corrected path
    WindowAddedToWorkspaceData, WindowRemovedFromWorkspaceData,
    WorkspacePersistentIdChangedData, WorkspaceIconChangedData, WorkspaceAccentChangedData,
    WorkspaceRenamedData, WorkspaceLayoutChangedData, // For completeness if used directly
};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkspaceEvent {
    WorkspaceCreated {
        id: WorkspaceId,
        name: String,
        position: usize, // Index in the ordered list of workspaces
        persistent_id: Option<String>,
        icon_name: Option<String>,
        accent_color_hex: Option<String>,
    },
    WorkspaceDeleted {
        id: WorkspaceId,
        // If windows were moved, this indicates their new home.
        // If None, windows were either not present or not moved (e.g. last workspace deleted scenario handling might differ).
        windows_moved_to_workspace_id: Option<WorkspaceId>,
    },
    ActiveWorkspaceChanged {
        old_id: Option<WorkspaceId>,
        new_id: WorkspaceId,
    },
    WorkspacesReloaded { // From initial load
        new_order: Vec<WorkspaceId>, // Vec of WorkspaceId, representing the new order
    },
    WorkspaceOrderChanged { // From explicit reordering
        new_order: Vec<WorkspaceId>,
    },
    WindowAddedToWorkspace(WindowAddedToWorkspaceData),
    WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData),
    WorkspacePersistentIdChanged(WorkspacePersistentIdChangedData),
    WorkspaceIconChanged(WorkspaceIconChangedData),
    WorkspaceAccentChanged(WorkspaceAccentChangedData),
    // These can be added if the manager emits them directly,
    // or if consumers are expected to use these specific data structures from core events.
    // For now, assuming manager might emit more specific events or these can be used as payload.
    WorkspaceRenamed { data: WorkspaceRenamedData }, // Re-exporting for potential direct use
    WorkspaceLayoutChanged { data: WorkspaceLayoutChangedData }, // Re-exporting for potential direct use
}
