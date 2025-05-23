use crate::channel::{ChannelId, ChannelPermissions, ChannelType};
use crate::user::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Note: Consider if uuid should be a direct dependency here if IDs are directly used,
// or if UserId/ChannelId (which wrap Uuid) are sufficient.
// For now, UserId/ChannelId are fine.

/// Event indicating a user has successfully joined a channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserJoinedChannel {
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub timestamp: DateTime<Utc>,
}

/// Event indicating a user has left a channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLeftChannel {
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub timestamp: DateTime<Utc>,
}

/// Event indicating a user has been muted within a specific channel.
/// This is distinct from global mutes or local user-to-user mutes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMutedInChannel {
    pub user_id: UserId,
    pub channel_id: ChannelId,
    // pub muted_by: Option<UserId>, // Optional: Who performed the mute?
    pub timestamp: DateTime<Utc>,
}

/// Event indicating a user has been unmuted within a specific channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserUnmutedInChannel {
    pub user_id: UserId,
    pub channel_id: ChannelId,
    // pub unmuted_by: Option<UserId>, // Optional: Who performed the unmute?
    pub timestamp: DateTime<Utc>,
}

/// Event indicating a new channel has been created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelCreated {
    pub channel_id: ChannelId,
    pub name: String,
    pub channel_type: ChannelType, // Assuming ChannelType is Cloneable and Serializable
    pub creator_id: Option<UserId>, // User who initiated channel creation
    pub timestamp: DateTime<Utc>,
}

/// Event indicating a channel has been deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelDeleted {
    pub channel_id: ChannelId,
    pub timestamp: DateTime<Utc>,
}

/// Event indicating the topic of a channel has changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelTopicChanged {
    pub channel_id: ChannelId,
    pub new_topic: String, // Consider Option<String> if topic can be cleared
    pub timestamp: DateTime<Utc>,
}

/// Event indicating a user's permissions within a channel have been updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPermissionsUpdated {
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub new_permissions: ChannelPermissions, // Assuming ChannelPermissions is Cloneable and Serializable
    // pub updated_by: Option<UserId>, // Optional: Who performed the update?
    pub timestamp: DateTime<Utc>,
}
