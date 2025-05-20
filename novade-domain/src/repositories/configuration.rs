//! Configuration repository interface for the NovaDE domain layer.
//!
//! This module provides the repository interface for accessing and persisting
//! configurations in the NovaDE desktop environment.

use async_trait::async_trait;
use crate::error::DomainError;
use crate::entities::Configuration;

/// Repository interface for configurations.
#[async_trait]
pub trait ConfigurationRepository: Send + Sync {
    /// Finds a configuration by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The configuration ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the configuration if found, or `None` if not found.
    async fn find_by_id(&self, id: &str) -> Result<Option<Configuration>, DomainError>;
    
    /// Finds configurations for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of configurations for the user.
    async fn find_for_user(&self, user_id: &str) -> Result<Vec<Configuration>, DomainError>;
    
    /// Finds a configuration by user ID and name.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `name` - The configuration name
    ///
    /// # Returns
    ///
    /// A `Result` containing the configuration if found, or `None` if not found.
    async fn find_by_user_and_name(&self, user_id: &str, name: &str) -> Result<Option<Configuration>, DomainError>;
    
    /// Saves a configuration.
    ///
    /// If the configuration already exists, it will be updated.
    /// If it doesn't exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The configuration to save
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn save(&self, configuration: &Configuration) -> Result<(), DomainError>;
    
    /// Deletes a configuration.
    ///
    /// # Arguments
    ///
    /// * `id` - The configuration ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
    
    /// Deletes all configurations for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn delete_for_user(&self, user_id: &str) -> Result<(), DomainError>;
}
