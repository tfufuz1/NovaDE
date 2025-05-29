use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap; // Not strictly needed for fields, but good practice if maps are used internally
use base64::{engine::general_purpose::STANDARD as Base64Standard, Engine as _};

// --- AIDataCategory Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AIDataCategory {
    UserProfile,
    ApplicationUsage,
    FileSystemRead,
    ClipboardAccess,
    LocationData,
    GenericText,
    GenericImage,
}

impl AIDataCategory {
    pub fn description(&self) -> &'static str {
        match self {
            AIDataCategory::UserProfile => "User profile information (e.g., name, preferences).",
            AIDataCategory::ApplicationUsage => "Data about applications used and their usage patterns.",
            AIDataCategory::FileSystemRead => "Permission to read files from the user's file system.",
            AIDataCategory::ClipboardAccess => "Access to read from or write to the system clipboard.",
            AIDataCategory::LocationData => "User's geographical location data.",
            AIDataCategory::GenericText => "General text input provided by the user.",
            AIDataCategory::GenericImage => "General image data provided by the user or screen captures.",
        }
    }
}

// --- AIConsentStatus Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AIConsentStatus {
    Granted,
    Denied,
    PendingUserAction,
    NotRequired,
}

impl Default for AIConsentStatus {
    fn default() -> Self {
        AIConsentStatus::PendingUserAction
    }
}

// --- AttachmentData Struct ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachmentData {
    pub id: Uuid,
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_base64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl AttachmentData {
    pub fn new_text(text_content: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            mime_type: "text/plain".to_string(),
            source_uri: None,
            content_base64: None,
            text_content: Some(text_content),
            description,
        }
    }

    pub fn new_from_uri(mime_type: String, source_uri: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            mime_type,
            source_uri: Some(source_uri),
            content_base64: None,
            text_content: None,
            description,
        }
    }

    pub fn new_from_binary_content(
        content: Vec<u8>,
        mime_type: String,
        description: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            mime_type,
            source_uri: None,
            content_base64: Some(Base64Standard.encode(content)),
            text_content: None,
            description,
        }
    }
}

// --- InteractionParticipant Enum ---
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InteractionParticipant {
    User,
    Assistant,
    System,
}

// --- InteractionHistoryEntry Struct ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionHistoryEntry {
    pub entry_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub participant: InteractionParticipant,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_attachment_ids: Vec<Uuid>,
}

// --- AIInteractionContext Struct ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIInteractionContext {
    pub id: Uuid,
    pub creation_timestamp: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_model_id: Option<String>,
    #[serde(default)] // Default for AIConsentStatus
    pub consent_status: AIConsentStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub associated_data_categories: Vec<AIDataCategory>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history_entries: Vec<InteractionHistoryEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<AttachmentData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_prompt_template: Option<String>,
    #[serde(default)]
    pub is_active: bool,
}

impl Default for AIInteractionContext { // Provide a sensible default
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            creation_timestamp: Utc::now(),
            active_model_id: None,
            consent_status: AIConsentStatus::default(),
            associated_data_categories: Vec::new(),
            history_entries: Vec::new(),
            attachments: Vec::new(),
            user_prompt_template: None,
            is_active: false,
        }
    }
}


// --- AIConsentScope Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AIConsentScope {
    #[default] // Changed default to SessionOnly as per instruction
    SessionOnly,
    PersistentUntilRevoked,
    SpecificDuration,
}

// --- AIConsent Struct ---
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AIConsent {
    pub id: Uuid,
    pub user_id: String,
    pub model_id: String,
    pub data_category: AIDataCategory,
    pub granted_timestamp: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry_timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub is_revoked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_timestamp: Option<DateTime<Utc>>,
    #[serde(default)] // Default for AIConsentScope
    pub consent_scope: AIConsentScope,
}

// --- AIModelCapability Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AIModelCapability {
    TextGeneration,
    CodeGeneration,
    Summarization,
    Translation,
    ImageAnalysis,
    FunctionCalling,
}

// --- AIModelProfile Struct ---
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AIModelProfile {
    pub model_id: String,
    pub display_name: String,
    pub description: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_consent_categories: Vec<AIDataCategory>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<AIModelCapability>,
    #[serde(default)]
    pub supports_streaming: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_secret_name: Option<String>,
    #[serde(default)]
    pub is_default_model: bool,
    #[serde(default)]
    pub sort_order: i32,
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn ai_data_category_description() {
        assert_eq!(AIDataCategory::UserProfile.description(), "User profile information (e.g., name, preferences).");
    }

    #[test]
    fn ai_data_category_serde() {
        let cat = AIDataCategory::FileSystemRead;
        let serialized = serde_json::to_string(&cat).unwrap();
        assert_eq!(serialized, "\"file-system-read\"");
        let deserialized: AIDataCategory = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, cat);
    }

    #[test]
    fn ai_consent_status_default_and_serde() {
        assert_eq!(AIConsentStatus::default(), AIConsentStatus::PendingUserAction);
        let status = AIConsentStatus::Granted;
        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(serialized, "\"granted\"");
        let deserialized: AIConsentStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, status);
    }

    #[test]
    fn attachment_data_new_text() {
        let att = AttachmentData::new_text("Hello".to_string(), Some("Greeting".to_string()));
        assert_eq!(att.mime_type, "text/plain");
        assert_eq!(att.text_content, Some("Hello".to_string()));
        assert_eq!(att.description, Some("Greeting".to_string()));
    }

    #[test]
    fn attachment_data_new_from_uri() {
        let att = AttachmentData::new_from_uri("image/png".to_string(), "file:///tmp/img.png".to_string(), None);
        assert_eq!(att.mime_type, "image/png");
        assert_eq!(att.source_uri, Some("file:///tmp/img.png".to_string()));
    }

    #[test]
    fn attachment_data_new_from_binary() {
        let content = vec![0u8, 1, 2, 3, 4, 5]; // Corrected vec definition
        let expected_base64 = Base64Standard.encode(&content);
        let att = AttachmentData::new_from_binary_content(content, "application/octet-stream".to_string(), None);
        assert_eq!(att.mime_type, "application/octet-stream");
        assert_eq!(att.content_base64, Some(expected_base64));
    }
    
    #[test]
    fn attachment_data_serde() {
        let att = AttachmentData::new_text("Test".to_string(), None);
        let serialized = serde_json::to_string(&att).unwrap();
        let deserialized: AttachmentData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(att, deserialized);
    }

    #[test]
    fn interaction_participant_serde() {
        let participant = InteractionParticipant::Assistant;
        let serialized = serde_json::to_string(&participant).unwrap();
        assert_eq!(serialized, "\"assistant\"");
        let deserialized: InteractionParticipant = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, participant);
    }

    #[test]
    fn interaction_history_entry_serde() {
        let entry = InteractionHistoryEntry {
            entry_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            participant: InteractionParticipant::User,
            content: "User query".to_string(),
            related_attachment_ids: vec![Uuid::new_v4()],
        };
        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: InteractionHistoryEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entry.entry_id, deserialized.entry_id); // Timestamps can be tricky
        assert_eq!(entry.participant, deserialized.participant);
        assert_eq!(entry.content, deserialized.content);
        assert_eq!(entry.related_attachment_ids, deserialized.related_attachment_ids);
    }
    
    #[test]
    fn interaction_history_entry_serde_empty_attachments() {
        let entry = InteractionHistoryEntry {
            entry_id: Uuid::new_v4(), timestamp: Utc::now(), participant: InteractionParticipant::User,
            content: "User query".to_string(), related_attachment_ids: vec![],
        };
        let serialized = serde_json::to_string(&entry).unwrap();
        assert!(!serialized.contains("related_attachment_ids"));
        let deserialized: InteractionHistoryEntry = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.related_attachment_ids.is_empty());
    }

    #[test]
    fn ai_interaction_context_default_and_serde() {
        let context_default = AIInteractionContext::default();
        assert_eq!(context_default.is_active, false);
        assert_eq!(context_default.consent_status, AIConsentStatus::default());

        let context = AIInteractionContext {
            id: Uuid::new_v4(), creation_timestamp: Utc::now(), active_model_id: Some("gpt-4".to_string()),
            consent_status: AIConsentStatus::Granted, associated_data_categories: vec![AIDataCategory::GenericText],
            history_entries: vec![], attachments: vec![], user_prompt_template: None, is_active: true,
        };
        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: AIInteractionContext = serde_json::from_str(&serialized).unwrap();
        assert_eq!(context.id, deserialized.id); // Timestamps can be tricky
    }

    #[test]
    fn ai_consent_scope_default_and_serde() {
        assert_eq!(AIConsentScope::default(), AIConsentScope::SessionOnly);
        let scope = AIConsentScope::PersistentUntilRevoked;
        let serialized = serde_json::to_string(&scope).unwrap();
        assert_eq!(serialized, "\"persistent-until-revoked\"");
        let deserialized: AIConsentScope = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, scope);
    }

    #[test]
    fn ai_consent_serde() {
        let consent = AIConsent {
            id: Uuid::new_v4(), user_id: "user1".to_string(), model_id: "model1".to_string(),
            data_category: AIDataCategory::ApplicationUsage, granted_timestamp: Utc::now(),
            expiry_timestamp: Some(Utc::now() + Duration::days(30)), is_revoked: false,
            last_used_timestamp: None, consent_scope: AIConsentScope::SpecificDuration,
        };
        let serialized = serde_json::to_string(&consent).unwrap();
        let deserialized: AIConsent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(consent.id, deserialized.id); // Timestamps
    }

    #[test]
    fn ai_model_capability_serde() {
        let cap = AIModelCapability::CodeGeneration;
        let serialized = serde_json::to_string(&cap).unwrap();
        assert_eq!(serialized, "\"code-generation\"");
        let deserialized: AIModelCapability = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, cap);
    }

    #[test]
    fn ai_model_profile_default_and_serde() {
        // Test with default for is_default_model and sort_order
        let profile = AIModelProfile {
            model_id: "text-gen-001".to_string(), display_name: "Text Generator v1".to_string(),
            description: "Generates text.".to_string(), provider: "NovadeAI".to_string(),
            required_consent_categories: vec![AIDataCategory::GenericText],
            capabilities: vec![AIModelCapability::TextGeneration, AIModelCapability::Summarization],
            supports_streaming: true, endpoint_url: Some("https://ai.novade.com/v1/models/text-gen-001".to_string()),
            api_key_secret_name: Some("NOVADE_AI_KEY".to_string()),
            is_default_model: true, // Explicitly set for this test
            sort_order: 10, // Explicitly set
        };
        let serialized = serde_json::to_string_pretty(&profile).unwrap();
        let deserialized: AIModelProfile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(profile, deserialized);
        assert_eq!(deserialized.is_default_model, true);
        assert_eq!(deserialized.sort_order, 10);
    }
    
    #[test]
    fn ai_model_profile_serde_minimal_check_defaults() {
        let profile_minimal_json = r#"{
            "model_id": "minimal-model",
            "display_name": "Minimal",
            "description": "Minimal desc",
            "provider": "TestProvider"
        }"#; // required_consent_categories and capabilities will default to empty vec
             // supports_streaming, is_default_model, sort_order will use their Default impl
        
        let deserialized: AIModelProfile = serde_json::from_str(profile_minimal_json).unwrap();
        assert_eq!(deserialized.model_id, "minimal-model");
        assert!(deserialized.required_consent_categories.is_empty());
        assert!(deserialized.capabilities.is_empty());
        assert_eq!(deserialized.supports_streaming, false); // Default for bool
        assert_eq!(deserialized.is_default_model, false); // Default for bool
        assert_eq!(deserialized.sort_order, 0); // Default for i32
        assert!(deserialized.endpoint_url.is_none());
        assert!(deserialized.api_key_secret_name.is_none());
    }
}
