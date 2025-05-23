use crate::channel::ChannelId; // Assuming ChannelId is in channel.rs
use crate::user::UserId;     // Assuming UserId is in user.rs
use thiserror::Error;

/// Represents errors that can occur within the domain logic of the Vivox system.
#[derive(Error, Debug)]
pub enum DomainError {
    /// Occurs when a user is expected but not found.
    #[error("User not found with ID: {user_id}")]
    UserNotFound { user_id: UserId },

    /// Occurs when a channel is expected but not found.
    #[error("Channel not found with ID: {channel_id}")]
    ChannelNotFound { channel_id: ChannelId },

    /// Occurs when attempting to add a user to a channel they are already in.
    #[error("User {user_id} is already in channel {channel_id}")]
    UserAlreadyInChannel {
        user_id: UserId,
        channel_id: ChannelId,
    },

    /// Occurs when attempting an operation on a user in a channel, but they are not a member.
    #[error("User {user_id} is not in channel {channel_id}")]
    UserNotInChannel {
        user_id: UserId,
        channel_id: ChannelId,
    },

    /// Occurs when attempting to add a user to a channel that has reached its maximum capacity.
    #[error("Channel {channel_id} is full (max participants: {max_participants})")]
    ChannelFull {
        channel_id: ChannelId,
        max_participants: u32,
    },

    /// Occurs when a user attempts an action they do not have permission for.
    #[error("User {user_id} permission denied for action '{action}' on resource '{resource_id}'")]
    PermissionDenied {
        user_id: UserId,
        action: String,
        resource_id: String, // Could be a ChannelId or UserId, converted to String
    },

    /// Occurs when a provided display name is invalid.
    #[error("Invalid display name '{name}': {reason}")]
    InvalidDisplayName { name: String, reason: String },

    /// Occurs when a provided channel name is invalid.
    #[error("Invalid channel name '{name}': {reason}")]
    InvalidChannelName { name: String, reason: String },

    /// Occurs when a string representation of an ID (like UserId or ChannelId) is invalid.
    #[error("Invalid ID format for type '{id_type}': received '{received_value}'")]
    InvalidIDFormat {
        id_type: String,
        received_value: String,
    },

    /// Occurs when a channel topic is invalid.
    #[error("Invalid topic '{topic}': {reason}")]
    InvalidTopic { topic: String, reason: String },

    /// Occurs when the 'max_participants' value for a channel is invalid.
    #[error("Invalid max participants value '{value}': {reason}")]
    InvalidMaxParticipants { value: u32, reason: String },
}

// Note: PartialEq is not automatically derived by thiserror.
// For testing, errors are typically matched by variant or by their string representation.
