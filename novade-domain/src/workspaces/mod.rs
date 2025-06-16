pub mod common_types;
pub mod events;
pub mod manager;
pub mod core;
pub mod config;
pub mod assignment;
pub mod traits;
pub mod tiling; // Added tiling module

pub use common_types::*;
// Note: The line `pub use events::*;` might refer to an older events.rs at `novade-domain/src/workspaces/events.rs`.
// The new WorkspaceEvent is specifically in `manager::events`.
// Depending on whether the old `events.rs` is still needed, this might need adjustment.
// For now, assuming it's for other, potentially higher-level events, and we add the new specific one.

// Re-export manager service, its default implementation, its specific event type, and its error type.
// ANCHOR: Commenting out old manager re-exports to prioritize new structures.
// pub use manager::{
//     WorkspaceManagerService, DefaultWorkspaceManager,
//     WorkspaceEvent, // This is from manager::events
//     WorkspaceManagerError
// };

// Re-export newly defined basic workspace and manager structures
pub use super::manager::{
    WorkspaceId,
    WorkspaceLayout,
    TilingLayout,
    Workspace,
    WorkspaceError,
    WorkspaceManager
};
pub use super::traits::WindowManager;
pub use super::tiling::{TilingAlgorithm, MasterStackLayout, SpiralLayout, TilingOptions}; // Re-export tiling types

// Re-export core types, errors, and the Workspace struct
pub use crate::workspaces::core::{
    // WorkspaceId is now in manager, WindowIdentifier might be different or from core
    WindowIdentifier, WorkspaceLayoutType, // From core::types (assuming WorkspaceId is not from here anymore)
    // Workspace is now in manager. If there's a different core::Workspace, it needs clarification.
    // For now, assuming manager::Workspace is the primary one.
    // Workspace, // From core::workspace
    WorkspaceCoreError,                                // From core::errors
    // Event data structs from core::event_data
    WorkspaceRenamedData, WorkspaceLayoutChangedData, WindowAddedToWorkspaceData,
    WindowRemovedFromWorkspaceData, WorkspacePersistentIdChangedData,
    WorkspaceIconChangedData, WorkspaceAccentChangedData,
    // Also re-export WindowId and Window from core as they are fundamental types used by WindowManager trait
    WindowId as CoreWindowId, // Alias if manager::WorkspaceId is different
    Window as CoreWindow, // Alias if manager::Workspace's WindowId refers to this
    WindowState as CoreWindowState,
};
// Re-export assignment errors
pub use crate::workspaces::assignment::WindowAssignmentError;


// The comment about not adding `pub use` for core, config, and assignment
// is now addressed by specifically re-exporting the necessary items from core and assignment.
// config re-exports can be handled when that module is finalized.

// Correcting re-exports based on where types are now defined:
// WorkspaceId, Workspace, WorkspaceLayout, TilingLayout, WorkspaceError, WorkspaceManager are from manager.rs
// WindowId, Window, WindowState from core.rs are used by the WindowManager trait.
// Ensure there are no conflicts if names are the same.
// The `pub use super::manager::{WorkspaceId, ...}` already handles the manager types.
// The `pub use crate::workspaces::core::{...}` needs to be accurate.
// `novade_domain::workspaces::core::WindowId` is used by the trait.
// `novade_domain::workspaces::manager::WorkspaceId` is a newtype u32. These are different.
// This needs careful aliasing or renaming.

// Let's clarify the re-exports for core types needed by the trait vs types defined in manager.
// The trait `WindowManager` uses `crate::workspaces::core::{Window as DomainWindow, WindowId, WindowState as DomainWindowState}`.
// So these specific core types must be correctly exported.
// `manager.rs` defines its own `WorkspaceId`.
// For clarity, I'll remove the aliased re-exports from core that might conflict with manager's own types for now,
// and ensure the trait's dependencies `core::WindowId`, `core::Window`, `core::WindowState` are clearly available.
// The `pub use super::manager::{WorkspaceId, ...}` handles manager specific types.
// The `pub use crate::workspaces::core::{WindowId as CoreWindowId, ...}` was an attempt to disambiguate.
// Let's ensure `core::WindowId` is the one used by the trait.
// `novade-domain/src/workspaces/core.rs` presumably defines `WindowId`, `Window`, `WindowState`.
