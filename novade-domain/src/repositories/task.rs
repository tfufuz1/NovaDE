//! Task repository interface for the NovaDE domain layer.
//!
//! This module provides the repository interface for accessing and persisting
//! tasks in the NovaDE desktop environment.

use async_trait::async_trait;
use crate::error::DomainError;
use crate::entities::{Task, Status, Priority};
use crate::entities::value_objects::Timestamp;

/// Repository interface for tasks.
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// Finds a task by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The task ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the task if found, or `None` if not found.
    async fn find_by_id(&self, id: &str) -> Result<Option<Task>, DomainError>;
    
    /// Saves a task.
    ///
    /// If the task already exists, it will be updated.
    /// If it doesn't exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to save
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn save(&self, task: &Task) -> Result<(), DomainError>;
    
    /// Deletes a task.
    ///
    /// # Arguments
    ///
    /// * `id` - The task ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
    
    /// Finds tasks for a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project ID
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of tasks for the project.
    async fn find_for_project(&self, project_id: &str) -> Result<Vec<Task>, DomainError>;
    
    /// Finds tasks by status.
    ///
    /// # Arguments
    ///
    /// * `status` - The task status
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of tasks with the specified status.
    async fn find_by_status(&self, status: Status) -> Result<Vec<Task>, DomainError>;
    
    /// Finds tasks by priority.
    ///
    /// # Arguments
    ///
    /// * `priority` - The task priority
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of tasks with the specified priority.
    async fn find_by_priority(&self, priority: Priority) -> Result<Vec<Task>, DomainError>;
    
    /// Finds tasks due before a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - The timestamp
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of tasks due before the timestamp.
    async fn find_due_before(&self, timestamp: Timestamp) -> Result<Vec<Task>, DomainError>;
    
    /// Finds tasks by tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of tasks with the specified tag.
    async fn find_by_tag(&self, tag: &str) -> Result<Vec<Task>, DomainError>;
    
    /// Lists all tasks.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all tasks.
    async fn list_all(&self) -> Result<Vec<Task>, DomainError>;
}
