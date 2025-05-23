use crate::error::DomainError;
use crate::user::{User, UserId};
use crate::channel::{Channel, ChannelId}; // Added Channel and ChannelId
use async_trait::async_trait;

/// A trait defining the operations for persisting and retrieving User entities.
///
/// This trait is part of the domain layer and will be implemented by the
/// infrastructure layer.
#[async_trait]
pub trait UserRepository: Send + Sync { // Send + Sync are common for async traits shared across threads
    /// Saves a user entity.
    /// This can be used for both creating a new user and updating an existing one.
    async fn save(&self, user: &User) -> Result<(), DomainError>;

    /// Finds a user by their unique ID.
    /// Returns `Ok(None)` if no user is found with the given ID.
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError>;

    /// Deletes a user by their unique ID.
    /// Should succeed even if the user does not exist to ensure idempotency.
    async fn delete(&self, id: &UserId) -> Result<(), DomainError>;

    // Example of an additional query method that might be useful:
    // /// Finds users whose display name contains the given fragment.
    // /// This is an example and can be expanded based on specific needs.
    // async fn find_by_display_name_fragment(&self, name_fragment: &str) -> Result<Vec<User>, DomainError>;
}

// The ChannelRepository trait will be added here later.

/// A trait defining the operations for persisting and retrieving Channel entities.
///
/// This trait is part of the domain layer and will be implemented by the
/// infrastructure layer.
#[async_trait]
pub trait ChannelRepository: Send + Sync { // Send + Sync are common for async traits
    /// Saves a channel entity.
    /// This can be used for both creating a new channel and updating an existing one.
    async fn save(&self, channel: &Channel) -> Result<(), DomainError>;

    /// Finds a channel by its unique ID.
    /// Returns `Ok(None)` if no channel is found with the given ID.
    async fn find_by_id(&self, id: &ChannelId) -> Result<Option<Channel>, DomainError>;

    /// Deletes a channel by its unique ID.
    /// Should succeed even if the channel does not exist to ensure idempotency.
    async fn delete(&self, id: &ChannelId) -> Result<(), DomainError>;

    // Example of an additional query method that might be useful:
    // /// Finds channels whose name contains the given fragment.
    // async fn find_by_name_fragment(&self, name_fragment: &str) -> Result<Vec<Channel>, DomainError>;
}
