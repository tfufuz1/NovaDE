//! Project repository interface for the NovaDE domain layer.
//!
//! This module provides the repository interface for accessing and persisting
//! projects in the NovaDE desktop environment.

use async_trait::async_trait;
use crate::error::DomainError;
use crate::entities::{Project, Status};
use crate::entities::value_objects::Timestamp;

/// Repository interface for projects.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Finds a project by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The project ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the project if found, or `None` if not found.
    async fn find_by_id(&self, id: &str) -> Result<Option<Project>, DomainError>;
    
    /// Saves a project.
    ///
    /// If the project already exists, it will be updated.
    /// If it doesn't exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `project` - The project to save
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn save(&self, project: &Project) -> Result<(), DomainError>;
    
    /// Deletes a project.
    ///
    /// # Arguments
    ///
    /// * `id` - The project ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
    
    /// Finds projects by status.
    ///
    /// # Arguments
    ///
    /// * `status` - The project status
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of projects with the specified status.
    async fn find_by_status(&self, status: Status) -> Result<Vec<Project>, DomainError>;
    
    /// Finds projects due before a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - The timestamp
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of projects due before the timestamp.
    async fn find_due_before(&self, timestamp: Timestamp) -> Result<Vec<Project>, DomainError>;
    
    /// Finds projects by tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of projects with the specified tag.
    async fn find_by_tag(&self, tag: &str) -> Result<Vec<Project>, DomainError>;
    
    /// Lists all projects.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all projects.
    async fn list_all(&self) -> Result<Vec<Project>, DomainError>;
}
