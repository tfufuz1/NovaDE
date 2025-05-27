use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIDataCategory {
    UserProfile,
    ApplicationUsage,
    FileSystemRead,
    ClipboardAccess,
    GenericText,
    // Add more specific categories as needed
}

impl AIDataCategory {
    pub fn description(&self) -> &'static str {
        match self {
            AIDataCategory::UserProfile => "Access to user profile information (e.g., name, preferences).",
            AIDataCategory::ApplicationUsage => "Access to data about which applications are used and how.",
            AIDataCategory::FileSystemRead => "Permission to read files from the user's file system.",
            AIDataCategory::ClipboardAccess => "Permission to access the content of the system clipboard.",
            AIDataCategory::GenericText => "Generic unstructured text provided by the user for processing.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIConsentStatus {
    Granted,
    Denied,
    PendingUserAction,
    NotRequired, // Default
}

impl Default for AIConsentStatus {
    fn default() -> Self {
        AIConsentStatus::NotRequired
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIModelCapability {
    TextGeneration,
    CodeGeneration,
    Summarization,
    ImageAnalysis,
    DataAnalysis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AIModelProfile {
    pub model_id: String,
    pub display_name: String,
    pub provider: String,
    pub capabilities: Vec<AIModelCapability>,
    pub required_consent_categories: Vec<AIDataCategory>,
    #[serde(default)] // For backward compatibility if older configs don't have it
    pub is_default_model: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIConsentScope {
    SessionOnly,
    PersistentUntilRevoked, // Default
    SpecificDuration, 
}

impl Default for AIConsentScope {
    fn default() -> Self {
        AIConsentScope::PersistentUntilRevoked
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AIConsent {
    pub id: Uuid,
    pub user_id: String, 
    pub model_id: String, 
    pub data_category: AIDataCategory,
    pub status: AIConsentStatus,
    pub scope: AIConsentScope,
    pub last_updated_timestamp: DateTime<Utc>,
}

impl AIConsent {
    pub fn new(
        user_id: String,
        model_id: String,
        data_category: AIDataCategory,
        status: AIConsentStatus,
        scope: AIConsentScope,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            model_id,
            data_category,
            status,
            scope,
            last_updated_timestamp: Utc::now(),
        }
    }
}

// --- Iteration 2 Additions ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachmentData {
    pub id: Uuid,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>, // For directly embedded text or OCR results
    #[serde(skip_serializing_if = "Option::is_none")]
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
            content_base64: None, // Content would be fetched/loaded by a service later
            text_content: None,
            description,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionParticipant {
    User,
    Assistant,
    System, // For system messages, errors, or context changes
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionHistoryEntry {
    pub entry_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub participant: InteractionParticipant,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_attachment_ids: Vec<Uuid>,
}

impl InteractionHistoryEntry {
     pub fn new_user_message(content: String, related_attachment_ids: Vec<Uuid>) -> Self {
        Self {
            entry_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            participant: InteractionParticipant::User,
            content,
            related_attachment_ids,
        }
    }
    pub fn new_assistant_message(content: String, related_attachment_ids: Vec<Uuid>) -> Self {
        Self {
            entry_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            participant: InteractionParticipant::Assistant,
            content,
            related_attachment_ids,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIInteractionContext {
    pub id: Uuid,
    pub creation_timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_model_id: Option<String>,
    pub consent_status: AIConsentStatus, // Overall status for the context's categories
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub associated_data_categories: Vec<AIDataCategory>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history_entries: Vec<InteractionHistoryEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<AttachmentData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_prompt_template: Option<String>,
    pub is_active: bool, // To mark if this context is still open or has been closed/completed
}

impl AIInteractionContext {
    pub fn new(
        associated_data_categories: Vec<AIDataCategory>,
        initial_attachments: Option<Vec<AttachmentData>>,
        user_prompt_template: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            creation_timestamp: Utc::now(),
            active_model_id: None,
            consent_status: AIConsentStatus::PendingUserAction, // Initial status, to be checked
            associated_data_categories,
            history_entries: Vec::new(),
            attachments: initial_attachments.unwrap_or_default(),
            user_prompt_template,
            is_active: true,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachment_data_new_text() {
        let attachment = AttachmentData::new_text("Hello world".to_string(), Some("Greeting".to_string()));
        assert_eq!(attachment.mime_type, "text/plain");
        assert_eq!(attachment.text_content, Some("Hello world".to_string()));
        assert!(attachment.source_uri.is_none());
    }

    #[test]
    fn attachment_data_new_from_uri() {
        let attachment = AttachmentData::new_from_uri("image/png".to_string(), "file:///tmp/img.png".to_string(), None);
        assert_eq!(attachment.mime_type, "image/png");
        assert_eq!(attachment.source_uri, Some("file:///tmp/img.png".to_string()));
        assert!(attachment.content_base64.is_none());
    }

    #[test]
    fn interaction_history_entry_new() {
        let entry = InteractionHistoryEntry::new_user_message("User says hi".to_string(), vec![]);
        assert_eq!(entry.participant, InteractionParticipant::User);
        assert_eq!(entry.content, "User says hi");
    }

    #[test]
    fn ai_interaction_context_new() {
        let categories = vec![AIDataCategory::GenericText];
        let context = AIInteractionContext::new(categories.clone(), None, None);
        assert_eq!(context.associated_data_categories, categories);
        assert!(context.is_active);
        assert_eq!(context.consent_status, AIConsentStatus::PendingUserAction);
        assert!(context.history_entries.is_empty());
    }
    
    #[test]
    fn ai_model_profile_default_is_default_model() {
        let profile_json = r#"
        {
            "model_id": "test-default",
            "display_name": "Test Default",
            "provider": "TestProvider",
            "capabilities": ["TextGeneration"],
            "required_consent_categories": ["GenericText"]
        }"#; // is_default_model is missing

        let deserialized_profile: AIModelProfile = serde_json::from_str(profile_json).unwrap();
        assert_eq!(deserialized_profile.is_default_model, false); // Should deserialize to false due to #[serde(default)]

         let profile_json_true = r#"
        {
            "model_id": "test-default-true",
            "display_name": "Test Default True",
            "provider": "TestProvider",
            "capabilities": ["TextGeneration"],
            "required_consent_categories": ["GenericText"],
            "is_default_model": true
        }"#;
        let deserialized_profile_true: AIModelProfile = serde_json::from_str(profile_json_true).unwrap();
        assert_eq!(deserialized_profile_true.is_default_model, true);
    }
}
