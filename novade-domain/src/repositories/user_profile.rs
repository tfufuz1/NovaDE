//! User profile repository interface for the NovaDE domain layer.
//!
//! This module provides the repository interface for accessing and persisting
//! user profiles in the NovaDE desktop environment.

use async_trait::async_trait;
use crate::error::DomainError;
use crate::entities::UserProfile;

/// Repository interface for user profiles.
#[async_trait]
pub trait UserProfileRepository: Send + Sync {
    /// Finds a user profile by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The user profile ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the user profile if found, or `None` if not found.
    async fn find_by_id(&self, id: &str) -> Result<Option<UserProfile>, DomainError>;
    
    /// Finds a user profile by email address.
    ///
    /// # Arguments
    ///
    /// * `email` - The email address
    ///
    /// # Returns
    ///
    /// A `Result` containing the user profile if found, or `None` if not found.
    async fn find_by_email(&self, email: &str) -> Result<Option<UserProfile>, DomainError>;
    
    /// Saves a user profile.
    ///
    /// If the user profile already exists, it will be updated.
    /// If it doesn't exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `profile` - The user profile to save
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn save(&self, profile: &UserProfile) -> Result<(), DomainError>;
    
    /// Deletes a user profile.
    ///
    /// # Arguments
    ///
    /// * `id` - The user profile ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
    
    /// Lists all user profiles.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all user profiles.
    async fn list_all(&self) -> Result<Vec<UserProfile>, DomainError>;
}
