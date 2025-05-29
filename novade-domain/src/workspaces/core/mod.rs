// Main module for core workspace definitions.

pub mod types;
pub mod errors;
pub mod workspace; 
pub mod event_data;

// Re-exports for easier access from parent modules
pub use types::{WorkspaceId, WindowIdentifier, WorkspaceLayoutType};
pub use errors::{WorkspaceCoreError, MAX_WORKSPACE_NAME_LENGTH};
pub use workspace::Workspace;

// Re-exports for event data structs
pub use event_data::{
    WorkspaceRenamedData, WorkspaceLayoutChangedData, WindowAddedToWorkspaceData,
    WindowRemovedFromWorkspaceData, WorkspacePersistentIdChangedData,
    WorkspaceIconChangedData, WorkspaceAccentChangedData,
};
