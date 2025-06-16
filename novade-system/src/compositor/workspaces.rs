// ANCHOR: CompositorWorkspaceDefinition
//! Defines the compositor-specific workspace structures.

use std::sync::{Arc, RwLock};
use uuid::Uuid;
use crate::compositor::shell::xdg_shell::types::DomainWindowIdentifier; // Adjusted path

/// Represents a single workspace within the compositor.
/// This is a runtime structure for managing live windows on a workspace.
#[derive(Debug)]
pub struct CompositorWorkspace {
    pub id: Uuid,
    pub name: String,
    // ANCHOR: AddOutputNameToCompositorWorkspace
    pub output_name: String, // Name of the output this workspace belongs to
    // ANCHOR_END: AddOutputNameToCompositorWorkspace
    /// List of window identifiers belonging to this workspace.
    /// `DomainWindowIdentifier` is used to refer to `ManagedWindow`s stored in `DesktopState`.
    pub windows: RwLock<Vec<DomainWindowIdentifier>>,
    // ANCHOR: AddTilingLayoutToWorkspace
    pub tiling_layout: Arc<RwLock<TilingLayout>>,
    // ANCHOR_END: AddTilingLayoutToWorkspace
}

// ANCHOR: DefineTilingLayoutEnum
/// Defines the available tiling layout modes for a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilingLayout {
    /// No tiling, windows are floating.
    None,
    /// Master-stack layout: one master window, others stacked.
    MasterStack,
    // SideBySide, // Example for another layout
}
// ANCHOR_END: DefineTilingLayoutEnum

impl CompositorWorkspace {
    /// Creates a new, empty workspace, defaulting to no tiling, associated with an output.
    pub fn new(name: String, output_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            output_name,
            windows: RwLock::new(Vec::new()),
            tiling_layout: Arc::new(RwLock::new(TilingLayout::None)), // Default to floating
        }
    }

    /// Adds a window (by its DomainWindowIdentifier) to this workspace.
    pub fn add_window(&self, window_id: DomainWindowIdentifier) {
        let mut windows_guard = self.windows.write().unwrap();
        if !windows_guard.contains(&window_id) {
            windows_guard.push(window_id);
        }
    }

    /// Removes a window from this workspace.
    pub fn remove_window(&self, window_id: &DomainWindowIdentifier) {
        let mut windows_guard = self.windows.write().unwrap();
        windows_guard.retain(|id| id != window_id);
    }

    /// Checks if a window is part of this workspace.
    pub fn contains_window(&self, window_id: &DomainWindowIdentifier) -> bool {
        self.windows.read().unwrap().contains(window_id)
    }

    /// Returns a clone of the list of window identifiers in this workspace.
    pub fn window_ids(&self) -> Vec<DomainWindowIdentifier> {
        self.windows.read().unwrap().clone()
    }
}
// ANCHOR_END: CompositorWorkspaceDefinition

// ANCHOR: ModRsWorkspacesModule
// This file (workspaces.rs) should be part of a module.
// If novade-system/src/compositor/mod.rs exists, add `pub mod workspaces;` there.
// If novade-system/src/lib.rs is the main library file for the crate,
// and compositor is a module within it, the path might be `crate::compositor::workspaces`.
// For now, this file defines the struct. Module integration is a separate step if needed by compiler.
// ANCHOR_END: ModRsWorkspacesModule
