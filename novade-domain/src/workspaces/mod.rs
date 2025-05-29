pub mod common_types;
pub mod events;
pub mod manager;
pub mod core;
pub mod config;
pub mod assignment;

pub use common_types::*;
// Note: The line `pub use events::*;` might refer to an older events.rs at `novade-domain/src/workspaces/events.rs`.
// The new WorkspaceEvent is specifically in `manager::events`.
// Depending on whether the old `events.rs` is still needed, this might need adjustment.
// For now, assuming it's for other, potentially higher-level events, and we add the new specific one.

// Re-export manager service, its default implementation, its specific event type, and its error type.
pub use manager::{
    WorkspaceManagerService, DefaultWorkspaceManager, 
    WorkspaceEvent, // This is from manager::events
    WorkspaceManagerError
};

// Re-export core types, errors, and the Workspace struct
pub use crate::workspaces::core::{
    WorkspaceId, WindowIdentifier, WorkspaceLayoutType, // From core::types
    Workspace,                                         // From core::workspace
    WorkspaceCoreError,                                // From core::errors
    // Event data structs from core::event_data
    WorkspaceRenamedData, WorkspaceLayoutChangedData, WindowAddedToWorkspaceData,
    WindowRemovedFromWorkspaceData, WorkspacePersistentIdChangedData,
    WorkspaceIconChangedData, WorkspaceAccentChangedData,
};
// Re-export assignment errors
pub use crate::workspaces::assignment::WindowAssignmentError;


// The comment about not adding `pub use` for core, config, and assignment
// is now addressed by specifically re-exporting the necessary items from core and assignment.
// config re-exports can be handled when that module is finalized.
