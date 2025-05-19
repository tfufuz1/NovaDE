//! Core workspace types for the NovaDE domain layer.
//!
//! This module provides the fundamental types and structures
//! for workspace management in the NovaDE desktop environment.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashSet;
use chrono::{DateTime, Utc};
use crate::shared_types::{EntityId, Version, Identifiable, Versionable};
use crate::error::{DomainResult, WorkspaceError};

/// A unique identifier for workspaces.
pub type WorkspaceId = EntityId;

/// A unique identifier for windows.
pub type WindowId = EntityId;

/// The type of a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkspaceType {
    /// A standard workspace for general use.
    Standard,
    /// A workspace for specific applications or tasks.
    Specialized,
    /// A temporary workspace.
    Temporary,
}

impl fmt::Display for WorkspaceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkspaceType::Standard => write!(f, "Standard"),
            WorkspaceType::Specialized => write!(f, "Specialized"),
            WorkspaceType::Temporary => write!(f, "Temporary"),
        }
    }
}

/// The state of a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkspaceState {
    /// The workspace is active and visible.
    Active,
    /// The workspace is inactive but visible.
    Inactive,
    /// The workspace is hidden.
    Hidden,
}

impl fmt::Display for WorkspaceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkspaceState::Active => write!(f, "Active"),
            WorkspaceState::Inactive => write!(f, "Inactive"),
            WorkspaceState::Hidden => write!(f, "Hidden"),
        }
    }
}

/// A workspace in the NovaDE desktop environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// The unique identifier of the workspace.
    id: WorkspaceId,
    /// The name of the workspace.
    name: String,
    /// The type of the workspace.
    workspace_type: WorkspaceType,
    /// The state of the workspace.
    state: WorkspaceState,
    /// The windows assigned to the workspace.
    windows: HashSet<WindowId>,
    /// The creation timestamp.
    created_at: DateTime<Utc>,
    /// The last update timestamp.
    updated_at: DateTime<Utc>,
    /// The version of the workspace.
    version: Version,
}

impl Workspace {
    /// Creates a new workspace with the specified name and type.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the workspace
    /// * `workspace_type` - The type of the workspace
    ///
    /// # Returns
    ///
    /// A new `Workspace` with the specified name and type.
    pub fn new(name: impl Into<String>, workspace_type: WorkspaceType) -> Self {
        let now = Utc::now();
        Workspace {
            id: WorkspaceId::new(),
            name: name.into(),
            workspace_type,
            state: WorkspaceState::Inactive,
            windows: HashSet::new(),
            created_at: now,
            updated_at: now,
            version: Version::initial(),
        }
    }

    /// Creates a new workspace with the specified ID, name, and type.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the workspace
    /// * `name` - The name of the workspace
    /// * `workspace_type` - The type of the workspace
    ///
    /// # Returns
    ///
    /// A new `Workspace` with the specified ID, name, and type.
    pub fn with_id(id: WorkspaceId, name: impl Into<String>, workspace_type: WorkspaceType) -> Self {
        let now = Utc::now();
        Workspace {
            id,
            name: name.into(),
            workspace_type,
            state: WorkspaceState::Inactive,
            windows: HashSet::new(),
            created_at: now,
            updated_at: now,
            version: Version::initial(),
        }
    }

    /// Gets the name of the workspace.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the name of the workspace.
    ///
    /// # Arguments
    ///
    /// * `name` - The new name of the workspace
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Gets the type of the workspace.
    pub fn workspace_type(&self) -> WorkspaceType {
        self.workspace_type
    }

    /// Sets the type of the workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_type` - The new type of the workspace
    pub fn set_workspace_type(&mut self, workspace_type: WorkspaceType) {
        self.workspace_type = workspace_type;
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Gets the state of the workspace.
    pub fn state(&self) -> WorkspaceState {
        self.state
    }

    /// Sets the state of the workspace.
    ///
    /// # Arguments
    ///
    /// * `state` - The new state of the workspace
    pub fn set_state(&mut self, state: WorkspaceState) {
        self.state = state;
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Activates the workspace.
    pub fn activate(&mut self) {
        self.set_state(WorkspaceState::Active);
    }

    /// Deactivates the workspace.
    pub fn deactivate(&mut self) {
        self.set_state(WorkspaceState::Inactive);
    }

    /// Hides the workspace.
    pub fn hide(&mut self) {
        self.set_state(WorkspaceState::Hidden);
    }

    /// Gets the windows assigned to the workspace.
    pub fn windows(&self) -> &HashSet<WindowId> {
        &self.windows
    }

    /// Adds a window to the workspace.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The ID of the window to add
    ///
    /// # Returns
    ///
    /// `true` if the window was added, `false` if it was already present.
    pub fn add_window(&mut self, window_id: WindowId) -> bool {
        let result = self.windows.insert(window_id);
        if result {
            self.updated_at = Utc::now();
            self.increment_version();
        }
        result
    }

    /// Removes a window from the workspace.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The ID of the window to remove
    ///
    /// # Returns
    ///
    /// `true` if the window was removed, `false` if it was not present.
    pub fn remove_window(&mut self, window_id: &WindowId) -> bool {
        let result = self.windows.remove(window_id);
        if result {
            self.updated_at = Utc::now();
            self.increment_version();
        }
        result
    }

    /// Checks if the workspace contains a window.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The ID of the window to check
    ///
    /// # Returns
    ///
    /// `true` if the workspace contains the window, `false` otherwise.
    pub fn contains_window(&self, window_id: &WindowId) -> bool {
        self.windows.contains(window_id)
    }

    /// Gets the creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Gets the last update timestamp.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Validates the workspace.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the workspace is valid, or an error if it is invalid.
    pub fn validate(&self) -> DomainResult<()> {
        if self.name.is_empty() {
            return Err(WorkspaceError::Invalid("Workspace name cannot be empty".to_string()).into());
        }
        Ok(())
    }
}

impl Identifiable for Workspace {
    fn id(&self) -> EntityId {
        self.id
    }
}

impl Versionable for Workspace {
    fn version(&self) -> Version {
        self.version
    }

    fn increment_version(&mut self) {
        self.version = self.version.next();
    }
}

impl fmt::Display for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Workspace[{}] '{}' ({}, {})",
            self.id, self.name, self.workspace_type, self.state
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_new() {
        let workspace = Workspace::new("Test Workspace", WorkspaceType::Standard);
        
        assert_eq!(workspace.name(), "Test Workspace");
        assert_eq!(workspace.workspace_type(), WorkspaceType::Standard);
        assert_eq!(workspace.state(), WorkspaceState::Inactive);
        assert!(workspace.windows().is_empty());
        assert_eq!(workspace.version(), Version::initial());
    }

    #[test]
    fn test_workspace_with_id() {
        let id = WorkspaceId::new();
        let workspace = Workspace::with_id(id, "Test Workspace", WorkspaceType::Standard);
        
        assert_eq!(workspace.id(), id);
        assert_eq!(workspace.name(), "Test Workspace");
        assert_eq!(workspace.workspace_type(), WorkspaceType::Standard);
    }

    #[test]
    fn test_workspace_set_name() {
        let mut workspace = Workspace::new("Test Workspace", WorkspaceType::Standard);
        let initial_version = workspace.version();
        
        workspace.set_name("New Name");
        
        assert_eq!(workspace.name(), "New Name");
        assert!(workspace.version() > initial_version);
    }

    #[test]
    fn test_workspace_set_type() {
        let mut workspace = Workspace::new("Test Workspace", WorkspaceType::Standard);
        let initial_version = workspace.version();
        
        workspace.set_workspace_type(WorkspaceType::Specialized);
        
        assert_eq!(workspace.workspace_type(), WorkspaceType::Specialized);
        assert!(workspace.version() > initial_version);
    }

    #[test]
    fn test_workspace_set_state() {
        let mut workspace = Workspace::new("Test Workspace", WorkspaceType::Standard);
        let initial_version = workspace.version();
        
        workspace.set_state(WorkspaceState::Active);
        
        assert_eq!(workspace.state(), WorkspaceState::Active);
        assert!(workspace.version() > initial_version);
    }

    #[test]
    fn test_workspace_activate_deactivate_hide() {
        let mut workspace = Workspace::new("Test Workspace", WorkspaceType::Standard);
        
        workspace.activate();
        assert_eq!(workspace.state(), WorkspaceState::Active);
        
        workspace.deactivate();
        assert_eq!(workspace.state(), WorkspaceState::Inactive);
        
        workspace.hide();
        assert_eq!(workspace.state(), WorkspaceState::Hidden);
    }

    #[test]
    fn test_workspace_add_window() {
        let mut workspace = Workspace::new("Test Workspace", WorkspaceType::Standard);
        let window_id = WindowId::new();
        let initial_version = workspace.version();
        
        let result = workspace.add_window(window_id);
        
        assert!(result);
        assert!(workspace.contains_window(&window_id));
        assert!(workspace.version() > initial_version);
        
        // Adding the same window again should return false
        let result = workspace.add_window(window_id);
        assert!(!result);
    }

    #[test]
    fn test_workspace_remove_window() {
        let mut workspace = Workspace::new("Test Workspace", WorkspaceType::Standard);
        let window_id = WindowId::new();
        
        workspace.add_window(window_id);
        let initial_version = workspace.version();
        
        let result = workspace.remove_window(&window_id);
        
        assert!(result);
        assert!(!workspace.contains_window(&window_id));
        assert!(workspace.version() > initial_version);
        
        // Removing a non-existent window should return false
        let result = workspace.remove_window(&window_id);
        assert!(!result);
    }

    #[test]
    fn test_workspace_validate() {
        let workspace = Workspace::new("Test Workspace", WorkspaceType::Standard);
        assert!(workspace.validate().is_ok());
        
        let mut invalid_workspace = Workspace::new("Test Workspace", WorkspaceType::Standard);
        invalid_workspace.set_name("");
        assert!(invalid_workspace.validate().is_err());
    }

    #[test]
    fn test_workspace_display() {
        let workspace = Workspace::new("Test Workspace", WorkspaceType::Standard);
        let display = format!("{}", workspace);
        
        assert!(display.contains("Test Workspace"));
        assert!(display.contains("Standard"));
        assert!(display.contains("Inactive"));
    }
}
