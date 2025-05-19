//! Workspace management module for the NovaDE domain layer.
//!
//! This module provides workspace management functionality for the NovaDE desktop environment,
//! allowing users to organize their work in virtual workspaces.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::error::{DomainError, WorkspaceError};
use crate::entities::value_objects::Timestamp;

mod services;

pub use services::default_workspace_service::DefaultWorkspaceService;

/// Represents a workspace in the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    /// Unique identifier for the workspace
    workspace_id: String,
    /// The workspace name
    name: String,
    /// The workspace description
    description: String,
    /// The workspace icon, if any
    icon: Option<String>,
    /// The workspace creation timestamp
    created_at: Timestamp,
    /// The workspace last modified timestamp
    modified_at: Timestamp,
    /// The workspace last accessed timestamp
    last_accessed_at: Timestamp,
    /// Additional properties for the workspace
    properties: HashMap<String, String>,
}

/// Represents a window reference in a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceWindowRef {
    /// Unique identifier for the window reference
    ref_id: String,
    /// The workspace ID
    workspace_id: String,
    /// The window ID
    window_id: String,
    /// The window reference creation timestamp
    created_at: Timestamp,
}

/// Interface for the workspace service.
#[async_trait]
pub trait WorkspaceService: Send + Sync {
    /// Creates a new workspace.
    ///
    /// # Arguments
    ///
    /// * `name` - The workspace name
    /// * `description` - The workspace description
    /// * `icon` - The workspace icon, if any
    /// * `properties` - Additional properties for the workspace
    ///
    /// # Returns
    ///
    /// A `Result` containing the created workspace ID.
    async fn create_workspace(
        &self,
        name: &str,
        description: &str,
        icon: Option<&str>,
        properties: HashMap<String, String>,
    ) -> Result<String, DomainError>;
    
    /// Gets a workspace by ID.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the workspace if found.
    async fn get_workspace(&self, workspace_id: &str) -> Result<Workspace, DomainError>;
    
    /// Updates a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    /// * `name` - The workspace name
    /// * `description` - The workspace description
    /// * `icon` - The workspace icon, if any
    /// * `properties` - Additional properties for the workspace
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn update_workspace(
        &self,
        workspace_id: &str,
        name: &str,
        description: &str,
        icon: Option<&str>,
        properties: HashMap<String, String>,
    ) -> Result<(), DomainError>;
    
    /// Deletes a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn delete_workspace(&self, workspace_id: &str) -> Result<(), DomainError>;
    
    /// Lists all workspaces.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all workspaces.
    async fn list_workspaces(&self) -> Result<Vec<Workspace>, DomainError>;
    
    /// Adds a window to a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    /// * `window_id` - The window ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the created window reference ID.
    async fn add_window_to_workspace(
        &self,
        workspace_id: &str,
        window_id: &str,
    ) -> Result<String, DomainError>;
    
    /// Removes a window from a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    /// * `window_id` - The window ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn remove_window_from_workspace(
        &self,
        workspace_id: &str,
        window_id: &str,
    ) -> Result<(), DomainError>;
    
    /// Lists all windows in a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all window references in the workspace.
    async fn list_windows_in_workspace(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<WorkspaceWindowRef>, DomainError>;
    
    /// Gets the workspace containing a window.
    ///
    /// # Arguments
    ///
    /// * `window_id` - The window ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the workspace if found.
    async fn get_workspace_for_window(
        &self,
        window_id: &str,
    ) -> Result<Workspace, DomainError>;
    
    /// Activates a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn activate_workspace(&self, workspace_id: &str) -> Result<(), DomainError>;
    
    /// Gets the active workspace.
    ///
    /// # Returns
    ///
    /// A `Result` containing the active workspace.
    async fn get_active_workspace(&self) -> Result<Workspace, DomainError>;
}
