use crate::error::DomainError; // Using the actual DomainError
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A strongly-typed identifier for a user.
///
/// Wraps a UUID to ensure type safety and provide a central place for ID logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    /// Creates a new `UserId` with a unique V4 UUID.
    pub fn new() -> Self {
        UserId(Uuid::new_v4())
    }

    /// Creates a `UserId` from an existing `Uuid`.
    pub fn from_uuid(uuid: Uuid) -> Self {
        UserId(uuid)
    }

    /// Returns a reference to the underlying `Uuid`.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for UserId {
    type Error = DomainError; // Changed from String

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value)
            .map(UserId)
            .map_err(|_| DomainError::InvalidIDFormat { // Changed error mapping
                id_type: "UserId".to_string(),
                received_value: value.to_string(),
            })
    }
}

// The User struct will be added here later in another step.
// pub struct User { ... }

// Assuming ChannelId will be imported or defined if user.rs needs it directly.
// For current_channel: Option<ChannelId>, we need ChannelId.
// It's better to put ChannelId in channel.rs and import it here.
// For now, let's assume it will be available via `crate::channel::ChannelId`.
// If the worker has issues with this, it can create a placeholder ChannelId in this file.
use crate::channel::ChannelId; // This line assumes channel.rs and its ChannelId are accessible.

/// Represents a user in the Vivox system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] // ChannelId is Eq
pub struct User {
    /// The unique identifier for the user.
    pub id: UserId,
    /// The user's display name. Must be non-empty and at most 32 characters.
    pub display_name: String,
    /// Whether the user is globally muted (affects all channels).
    pub is_globally_muted: bool,
    /// The ID of the channel the user is currently in, if any.
    pub current_channel: Option<ChannelId>,
}

impl User {
    /// Creates a new user.
    ///
    /// # Arguments
    /// * `id` - The unique ID for the user.
    /// * `display_name` - The user's display name. Must be non-empty and at most 32 characters.
    ///
    /// # Errors
    /// Returns an error string if the display name is invalid. This will be replaced
    /// by `DomainError::InvalidDisplayName` once `DomainError` is implemented.
    pub fn new(id: UserId, display_name: String) -> Result<Self, DomainError> { // Changed return type
        if display_name.is_empty() {
            // Changed error variant
            return Err(DomainError::InvalidDisplayName {
                name: display_name.clone(), // Cloned for error
                reason: "Display name cannot be empty.".to_string(),
            });
        }
        if display_name.chars().count() > 32 {
            // Changed error variant
            return Err(DomainError::InvalidDisplayName {
                name: display_name.clone(), // Cloned for error
                reason: "Display name cannot exceed 32 characters.".to_string(),
            });
        }
        Ok(Self {
            id,
            display_name,
            is_globally_muted: false,
            current_channel: None,
        })
    }

    /// Associates the user with a channel.
    pub fn join_channel(&mut self, channel_id: ChannelId) {
        self.current_channel = Some(channel_id);
    }

    /// Removes the user from their current channel.
    pub fn leave_channel(&mut self) {
        self.current_channel = None;
    }

    /// Globally mutes the user.
    pub fn mute_globally(&mut self) {
        self.is_globally_muted = true;
    }

    /// Globally unmutes the user.
    pub fn unmute_globally(&mut self) {
        self.is_globally_muted = false;
    }

    /// Updates the user's display name.
    ///
    /// # Arguments
    /// * `new_name` - The new display name. Must be non-empty and at most 32 characters.
    ///
    /// # Errors
    /// Returns an error string if the new display name is invalid. This will be replaced
    /// by `DomainError::InvalidDisplayName` once `DomainError` is implemented.
    pub fn update_display_name(&mut self, new_name: String) -> Result<(), DomainError> { // Changed return type
        if new_name.is_empty() {
            // Changed error variant
            return Err(DomainError::InvalidDisplayName {
                name: new_name.clone(), // Cloned for error
                reason: "Display name cannot be empty.".to_string(),
            });
        }
        if new_name.chars().count() > 32 {
            // Changed error variant
            return Err(DomainError::InvalidDisplayName {
                name: new_name.clone(), // Cloned for error
                reason: "Display name cannot exceed 32 characters.".to_string(),
            });
        }
        self.display_name = new_name;
        Ok(())
    }
}
