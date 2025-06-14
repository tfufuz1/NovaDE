// novade-core/src/types/assistant.rs

//! Contains data types related to the Context-Aware Smart Assistant.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Represents a recognized command to be executed by the assistant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssistantCommand {
    /// The recognized intent of the command.
    pub intent: String,
    /// Parameters extracted from the user input, specific to the intent.
    pub parameters: Option<HashMap<String, String>>, // Example: {"app_name": "Firefox", "action": "open"}
    /// Optional: Confidence score of the intent recognition (0.0 to 1.0).
    pub confidence: Option<f32>,
}

/// Holds contextual information relevant to the assistant's operation.
/// This information helps the assistant to provide more relevant responses or actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ContextInfo {
    /// Identifier of the currently active application or window.
    pub active_application_id: Option<String>,
    /// Title of the currently active window.
    pub active_window_title: Option<String>,
    /// Current system time (e.g., ISO 8601 timestamp).
    pub current_time: Option<String>,
    /// User's current location (if permission granted and available).
    pub location: Option<String>, // e.g., "City, Country" or GPS coordinates
    /// User-specific preferences that might affect assistant behavior.
    pub user_preferences: HashMap<String, String>, // e.g., {"preferred_music_service": "Spotify"}
    /// Recent user interactions or commands.
    pub interaction_history: Vec<String>, // Limited history for short-term context
}

/// Represents the user's intent as derived from their input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserIntent {
    /// The raw input string from the user.
    pub raw_input: String,
    /// The primary recognized intent.
    pub primary_intent_name: String,
    /// Entities or slots extracted from the input.
    /// Example: For "Play music by Queen", entities could be {"artist": "Queen"}.
    pub entities: HashMap<String, String>,
    /// Optional: Alternative intents if the primary one is ambiguous.
    pub alternative_intents: Option<Vec<AssistantCommand>>,
}

/// Defines the metadata and capabilities of an assistant skill/plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillDefinition {
    /// Unique identifier for the skill.
    pub id: String, // e.g., "com.novade.skills.weather"
    /// Human-readable name of the skill.
    pub name: String,
    /// Version of the skill.
    pub version: String,
    /// Author or provider of the skill.
    pub author: String,
    /// A brief description of what the skill does.
    pub description: String,
    /// Invocation phrases or intents that this skill can handle.
    pub supported_intents: Vec<String>,
    /// Configuration parameters required or supported by the skill.
    pub configuration_schema: Option<HashMap<String, String>>, // e.g., {"api_key": "string"}
    /// Permissions required by the skill (e.g., "network_access", "read_calendar").
    pub required_permissions: Vec<String>,
}

/// Represents user-specific preferences for the assistant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AssistantPreferences {
    /// Whether the assistant is enabled.
    pub enabled: bool,
    /// Primary language for voice and text interaction.
    pub language: String, // e.g., "en-US", "de-DE"
    /// Activation method (e.g., "hotword", "keybinding").
    pub activation_method: String,
    /// Specific hotword if activation_method is "hotword".
    pub hotword: Option<String>,
    /// Keybinding if activation_method is "keybinding".
    pub keybinding: Option<String>,
    /// Voice feedback enabled/disabled.
    pub voice_feedback_enabled: bool,
    /// Data sharing and privacy settings.
    pub privacy_settings: HashMap<String, bool>, // e.g., {"share_usage_data": true, "keep_history": false}
    /// Configuration for enabled skills.
    pub skill_settings: HashMap<String, HashMap<String, String>>, // skill_id -> {setting_key: value}
}

// TODO: Further refine these types as implementation progresses.
// TODO: Consider using more specific types for parameters, entities, and settings where appropriate.
