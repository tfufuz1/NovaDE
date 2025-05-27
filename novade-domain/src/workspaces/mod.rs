// Declare main submodules
pub mod core;
pub mod config;
pub mod assignment; 
pub mod manager;    

// Re-export key public types from the core module
pub use self::core::{
    Workspace,
    WorkspaceId,
    WindowIdentifier,
    WorkspaceLayoutType,
    WorkspaceCoreError,
    WorkspaceRenamedData,
    WorkspaceLayoutChangedData,
};

// Re-export key public types from the config module
pub use self::config::{
    WorkspaceSnapshot,
    WorkspaceSetSnapshot,
    WorkspaceConfigError,
    WorkspaceConfigProvider,
    FilesystemConfigProvider,
};

// Re-export key public types from the assignment module
pub use self::assignment::{
    assign_window_to_workspace,
    remove_window_from_workspace,
    find_workspace_for_window,
    WindowAssignmentError,
};

// Re-export key public types from the manager module
pub use self::manager::{
    WorkspaceManagerService,
    DefaultWorkspaceManager,
    WorkspaceManagerError,
    WorkspaceEvent,
};
