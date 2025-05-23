use crate::error::DomainError; // Using the actual DomainError
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use std::collections::HashMap; // Added for Channel's active_participants

use crate::user::UserId; // Added for ChannelMembership

/// A strongly-typed identifier for a channel.
///
/// Wraps a UUID to ensure type safety and provide a central place for ID logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(Uuid);

impl ChannelId {
    /// Creates a new `ChannelId` with a unique V4 UUID.
    pub fn new() -> Self {
        ChannelId(Uuid::new_v4())
    }

    /// Creates a `ChannelId` from an existing `Uuid`.
    pub fn from_uuid(uuid: Uuid) -> Self {
        ChannelId(uuid)
    }

    /// Returns a reference to the underlying `Uuid`.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ChannelId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for ChannelId {
    type Error = DomainError; // Changed from String

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value)
            .map(ChannelId)
            .map_err(|_| DomainError::InvalidIDFormat { // Changed error mapping
                id_type: "ChannelId".to_string(),
                received_value: value.to_string(),
            })
    }
}

// Other channel-related structs and enums (Channel, ChannelType, etc.) will be added here later.

/// Represents the type of a communication channel.
///
/// Each variant defines specific behavior or properties of the channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChannelType {
    /// A channel where audio attenuates with distance.
    Positional {
        /// The range within which audio is clearly audible.
        range: f32,
    },
    /// A channel where audio is directed.
    /// (Coordinates might represent a direction vector or a source position based on convention)
    Directional {
        x: f32,
        y: f32,
        z: f32,
    },
    /// An echo channel that plays back what a user says. Useful for testing.
    Echo,
    /// A standard group channel without special audio processing like positional or directional.
    Group,
}

// Other channel-related structs (ChannelPermissions, UserAudioProperties, Channel entity) will be added here later.

/// Defines the permissions a user has within a specific channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelPermissions {
    /// Can the user speak in the channel?
    pub can_speak: bool,
    /// Can the user hear others in the channel?
    pub can_listen: bool,
    /// Can the user send text messages in the channel?
    pub can_text: bool,
    /// Can the user kick other participants from the channel?
    pub can_kick: bool,
    /// Does the user have moderator privileges in the channel?
    pub is_moderator: bool,
}

impl ChannelPermissions {
    /// Creates a new set of channel permissions.
    pub fn new(
        can_speak: bool,
        can_listen: bool,
        can_text: bool,
        can_kick: bool,
        is_moderator: bool,
    ) -> Self {
        Self {
            can_speak,
            can_listen,
            can_text,
            can_kick,
            is_moderator,
        }
    }

    /// Returns default permissions for a regular participant.
    /// Typically includes speaking, listening, and texting, but no administrative rights.
    pub fn default_participant() -> Self {
        Self {
            can_speak: true,
            can_listen: true,
            can_text: true,
            can_kick: false,
            is_moderator: false,
        }
    }

    /// Returns permissions for a channel moderator.
    /// Typically includes all standard interaction permissions plus administrative rights.
    pub fn moderator() -> Self {
        Self {
            can_speak: true,
            can_listen: true,
            can_text: true,
            can_kick: true,
            is_moderator: true,
        }
    }
}

// Other channel-related structs (UserAudioProperties, Channel entity) will be added here later.

/// Represents user-specific audio settings, often in the context of another user or a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAudioProperties {
    /// The audio volume, typically on a scale of 0 (silent) to 100 (full volume).
    pub volume: u8,
    /// Whether the audio from a source is muted locally by the listening user.
    /// This is distinct from a user being globally muted or muted in a channel by a moderator.
    pub is_locally_muted: bool,
}

impl UserAudioProperties {
    /// Creates new audio properties.
    ///
    /// The `volume` is clamped to the range 0-100.
    pub fn new(volume: u8, is_locally_muted: bool) -> Self {
        Self {
            volume: volume.clamp(0, 100), // Ensure volume is within the valid range
            is_locally_muted,
        }
    }
}

impl Default for UserAudioProperties {
    /// Default audio properties: full volume and not locally muted.
    fn default() -> Self {
        Self {
            volume: 100,
            is_locally_muted: false,
        }
    }
}

// The Channel entity implementation will be added here later.

/// Represents a communication channel where users can interact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    /// The unique identifier for the channel.
    pub id: ChannelId,
    /// The channel's name. Must be non-empty and at most 64 characters.
    pub name: String,
    /// An optional topic or description for the channel.
    /// If Some, must be non-empty and at most 128 characters.
    pub topic: Option<String>,
    /// The type of the channel, defining its audio processing behavior.
    pub channel_type: ChannelType,
    /// Optional maximum number of participants. If None, the channel has no limit.
    /// If Some, the value must be greater than 0.
    pub max_participants: Option<u32>,
    /// A map of active participants in the channel, keyed by their `UserId`.
    /// Value is a `ChannelMembership` struct detailing their status and permissions.
    pub active_participants: HashMap<UserId, ChannelMembership>,
}

impl Channel {
    /// Creates a new communication channel.
    ///
    /// # Arguments
    /// * `id` - The unique ID for the channel.
    /// * `name` - The channel's name. Must be non-empty and at most 64 characters.
    /// * `channel_type` - The type of the channel.
    /// * `max_participants` - Optional maximum number of participants. If Some, must be greater than 0.
    ///
    /// # Errors
    /// Returns a String error if validation fails (e.g., invalid name, invalid max_participants).
    /// This will be replaced by specific `DomainError` variants later.
    pub fn new(
        id: ChannelId,
        name: String,
        channel_type: ChannelType,
        max_participants: Option<u32>,
    ) -> Result<Self, DomainError> { // Changed return type
        if name.is_empty() {
            return Err(DomainError::InvalidChannelName { // Changed error variant
                name: name.clone(),
                reason: "Channel name cannot be empty.".to_string(),
            });
        }
        if name.chars().count() > 64 {
            return Err(DomainError::InvalidChannelName { // Changed error variant
                name: name.clone(),
                reason: "Channel name cannot exceed 64 characters.".to_string(),
            });
        }
        if let Some(max) = max_participants {
            if max == 0 {
                return Err(DomainError::InvalidMaxParticipants { // Changed error variant
                    value: max,
                    reason: "max_participants must be greater than 0 if specified.".to_string(),
                });
            }
        }

        Ok(Self {
            id,
            name,
            topic: None,
            channel_type,
            max_participants,
            active_participants: HashMap::new(),
        })
    }

    /// Sets or updates the topic of the channel.
    ///
    /// # Arguments
    /// * `topic` - The new topic. If Some, must be non-empty and at most 128 characters.
    ///             If None, the topic is cleared.
    ///
    /// # Errors
    /// Returns a String error if the topic validation fails.
    /// This will be replaced by `DomainError::InvalidChannelTopic` later.
    pub fn set_topic(&mut self, topic: Option<String>) -> Result<(), DomainError> { // Changed return type
        if let Some(t) = &topic {
            if t.is_empty() {
                return Err(DomainError::InvalidTopic { // Changed error variant
                    topic: t.clone(),
                    reason: "Channel topic cannot be empty if Some.".to_string(),
                });
            }
            if t.chars().count() > 128 {
                return Err(DomainError::InvalidTopic { // Changed error variant
                    topic: t.clone(),
                    reason: "Channel topic cannot exceed 128 characters.".to_string(),
                });
            }
        }
        self.topic = topic;
        Ok(())
    }

    /// Adds a user to the channel with the given initial permissions.
    ///
    /// # Errors
    /// Returns a String error if the channel is full or the user is already in the channel.
    /// These will be replaced by `DomainError::ChannelFull` and `DomainError::UserAlreadyInChannel`.
    pub fn add_user(
        &mut self,
        user_id: UserId,
        initial_permissions: ChannelPermissions,
    ) -> Result<(), DomainError> { // Changed return type
        if self.is_full() {
            // If is_full() is true, self.max_participants must be Some.
            return Err(DomainError::ChannelFull { // Changed error variant
                channel_id: self.id,
                max_participants: self.max_participants.unwrap_or(0), // unwrap_or(0) as a fallback, though is_full implies Some.
            });
        }
        if self.active_participants.contains_key(&user_id) {
            return Err(DomainError::UserAlreadyInChannel { // Changed error variant
                user_id,
                channel_id: self.id,
            });
        }

        let membership = ChannelMembership::new(user_id, initial_permissions);
        self.active_participants.insert(user_id, membership);
        Ok(())
    }

    /// Removes a user from the channel.
    ///
    /// # Errors
    /// Returns a String error if the user is not found in the channel.
    /// This will be replaced by `DomainError::UserNotInChannel`.
    pub fn remove_user(&mut self, user_id: &UserId) -> Result<(), DomainError> { // Changed return type
        if self.active_participants.remove(user_id).is_none() {
            return Err(DomainError::UserNotInChannel { // Changed error variant
                user_id: *user_id,
                channel_id: self.id,
            });
        }
        Ok(())
    }

    /// Retrieves a reference to a participant's membership details.
    pub fn get_participant_membership(&self, user_id: &UserId) -> Option<&ChannelMembership> {
        self.active_participants.get(user_id)
    }

    /// Retrieves a mutable reference to a participant's membership details.
    pub fn get_mut_participant_membership(&mut self, user_id: &UserId) -> Option<&mut ChannelMembership> {
        self.active_participants.get_mut(user_id)
    }

    /// Updates the permissions for a user already in the channel.
    ///
    /// # Errors
    /// Returns a String error if the user is not found in the channel.
    /// This will be replaced by `DomainError::UserNotInChannel`.
    pub fn update_user_permissions(
        &mut self,
        user_id: &UserId,
        new_permissions: ChannelPermissions,
    ) -> Result<(), DomainError> { // Changed return type
        match self.active_participants.get_mut(user_id) {
            Some(membership) => {
                membership.permissions = new_permissions;
                Ok(())
            }
            None => Err(DomainError::UserNotInChannel { // Changed error variant
                user_id: *user_id,
                channel_id: self.id,
            }),
        }
    }

    /// Checks if the channel has reached its maximum participant capacity.
    /// If `max_participants` is None, the channel is never full.
    pub fn is_full(&self) -> bool {
        if let Some(max) = self.max_participants {
            self.active_participants.len() >= max as usize
        } else {
            false // No limit
        }
    }

    /// Returns the current number of participants in the channel.
    pub fn participant_count(&self) -> usize {
        self.active_participants.len()
    }
}

/// Represents a user's membership details within a specific channel.
/// This includes their permissions and audio state for that channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelMembership {
    /// The ID of the user this membership pertains to.
    pub user_id: UserId,
    /// The permissions granted to this user in the channel.
    pub permissions: ChannelPermissions,
    /// Whether the user is muted in this channel (e.g., by a moderator).
    /// This is different from `is_globally_muted` on the `User` struct or
    /// `is_locally_muted` in `UserAudioProperties`.
    pub is_muted_in_channel: bool,
    /// Audio properties specific to this user's presence in the channel.
    /// For example, this could represent their microphone's transmission volume
    /// or specific settings applied to their audio stream by the channel.
    pub audio_properties: UserAudioProperties,
}

impl ChannelMembership {
    /// Creates new channel membership details for a user.
    ///
    /// By default, the user is not muted in the channel, and audio properties are set to default.
    pub fn new(user_id: UserId, permissions: ChannelPermissions) -> Self {
        Self {
            user_id,
            permissions,
            is_muted_in_channel: false,
            audio_properties: UserAudioProperties::default(),
        }
    }

    /// Mutes the user within this channel context.
    pub fn mute_in_channel(&mut self) {
        self.is_muted_in_channel = true;
    }

    /// Unmutes the user within this channel context.
    pub fn unmute_in_channel(&mut self) {
        self.is_muted_in_channel = false;
    }

    /// Updates the user's audio properties for this channel.
    pub fn update_audio_properties(&mut self, properties: UserAudioProperties) {
        self.audio_properties = properties;
    }
}

// The Channel entity implementation will be added here later.
