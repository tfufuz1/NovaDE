use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, info, warn, error};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use super::types::{
    AIDataCategory, AIConsentStatus, AIModelCapability, AIModelProfile, AIConsentScope, AIConsent,
    AIInteractionContext, AttachmentData, InteractionHistoryEntry, InteractionParticipant,
};
use super::errors::AIInteractionError;
use super::persistence_iface::{AIConsentProvider, AIModelProfileProvider};
// Assuming persistence.rs is in the same module (ai_interaction)
use super::persistence::{FilesystemAIConsentProvider, FilesystemAIModelProfileProvider}; 
use crate::user_centric_services::events::AIInteractionEvent;

#[async_trait]
pub trait AIInteractionLogicService: Send + Sync {
    async fn get_consent_status(
        &self,
        user_id: &str,
        model_id: &str,
        category: AIDataCategory,
    ) -> Result<AIConsentStatus, AIInteractionError>;

    async fn list_available_models(&self) -> Result<Vec<AIModelProfile>, AIInteractionError>;

    async fn grant_consent(
        &self,
        user_id: &str,
        model_id: &str,
        category: AIDataCategory,
        scope: AIConsentScope,
    ) -> Result<(), AIInteractionError>;

    async fn revoke_consent(
        &self,
        user_id: &str,
        model_id: &str,
        category: AIDataCategory,
    ) -> Result<(), AIInteractionError>;
    
    async fn load_data_into_cache(&self, user_id: Option<&str>) -> Result<(), AIInteractionError>;

    // --- Iteration 2 Methods ---
    async fn initiate_interaction(
        &self,
        user_id: &str, // User ID is needed for consent checks
        relevant_categories: Vec<AIDataCategory>,
        initial_attachments: Option<Vec<AttachmentData>>,
        prompt_template: Option<String>,
    ) -> Result<Uuid, AIInteractionError>;

    async fn get_interaction_context(&self, context_id: Uuid) -> Result<AIInteractionContext, AIInteractionError>;
    
    async fn add_attachment_to_context(&self, context_id: Uuid, attachment: AttachmentData) -> Result<(), AIInteractionError>;
    
    async fn update_interaction_history(&self, context_id: Uuid, entry: InteractionHistoryEntry) -> Result<(), AIInteractionError>;
    
    async fn get_consent_status_for_interaction(
        &self,
        context_id: Uuid,
        model_id: &str, // Model to check against
        // If None, use context.associated_data_categories. If Some, check these specific ones.
        required_categories_override: Option<&[AIDataCategory]>, 
    ) -> Result<AIConsentStatus, AIInteractionError>;
    
    async fn get_default_model(&self) -> Result<Option<AIModelProfile>, AIInteractionError>;

    async fn set_interaction_active_model(&self, context_id: Uuid, model_id: Option<String>) -> Result<(), AIInteractionError>;
}

pub struct DefaultAIInteractionLogicService {
    model_profiles_cache: Arc<RwLock<Vec<AIModelProfile>>>,
    user_consents_cache: Arc<RwLock<HashMap<String, Vec<AIConsent>>>>, // Key: user_id
    active_contexts: Arc<RwLock<HashMap<Uuid, AIInteractionContext>>>, // Key: context_id (Uuid)
    profile_provider: Arc<dyn AIModelProfileProvider>, // Now using the trait
    consent_provider: Arc<dyn AIConsentProvider>,   // Now using the trait
    event_publisher: broadcast::Sender<AIInteractionEvent>,
}

impl DefaultAIInteractionLogicService {
    pub fn new(
        // Changed to accept trait objects for providers
        profile_provider: Arc<dyn AIModelProfileProvider>,
        consent_provider: Arc<dyn AIConsentProvider>,
        event_publisher: broadcast::Sender<AIInteractionEvent>,
    ) -> Self {
        Self {
            model_profiles_cache: Arc::new(RwLock::new(Vec::new())),
            user_consents_cache: Arc::new(RwLock::new(HashMap::new())),
            active_contexts: Arc::new(RwLock::new(HashMap::new())),
            profile_provider,
            consent_provider,
            event_publisher,
        }
    }

    // Helper to check consent for multiple categories and determine overall status
    async fn check_overall_consent(
        &self,
        user_id: &str,
        model_id: &str,
        categories: &[AIDataCategory],
    ) -> Result<AIConsentStatus, AIInteractionError> {
        if categories.is_empty() {
            return Ok(AIConsentStatus::NotRequired);
        }
        let mut final_status = AIConsentStatus::Granted; // Assume granted until proven otherwise
        for category in categories {
            match self.get_consent_status(user_id, model_id, *category).await? {
                AIConsentStatus::Denied => return Ok(AIConsentStatus::Denied), // One denial means overall denial
                AIConsentStatus::PendingUserAction => final_status = AIConsentStatus::PendingUserAction,
                AIConsentStatus::NotRequired if final_status == AIConsentStatus::Granted => {
                    // If current overall is Granted, but one category is NotRequired,
                    // the overall status might become NotRequired if all others are also NotRequired.
                    // This logic might need refinement based on how "NotRequired" interacts.
                    // For now, if any required category is Granted or Pending, that takes precedence.
                    // If all are NotRequired, then overall is NotRequired.
                    // If a mix of Granted and NotRequired, overall is Granted.
                    // If a mix of Pending and NotRequired, overall is Pending.
                    // This simplified logic: if any is Pending, result is Pending unless a Denied is found.
                },
                _ => {} // Granted or NotRequired (when final_status is already Pending)
            }
        }
        Ok(final_status)
    }
}

#[async_trait]
impl AIInteractionLogicService for DefaultAIInteractionLogicService {
    async fn load_data_into_cache(&self, user_id_opt: Option<&str>) -> Result<(), AIInteractionError> {
        info!("Loading AI interaction data into cache. User: {:?}", user_id_opt);
        let profiles = self.profile_provider.load_model_profiles().await?;
        let mut profiles_cache_lock = self.model_profiles_cache.write().await;
        *profiles_cache_lock = profiles;
        info!("Loaded {} model profiles into cache.", profiles_cache_lock.len());
        if let Err(e) = self.event_publisher.send(AIInteractionEvent::AIModelProfilesReloaded {
            profiles_count: profiles_cache_lock.len(),
        }) {
            warn!("Failed to send AIModelProfilesReloaded event: {}", e);
        }

        if let Some(user_id) = user_id_opt {
            let consents = self.consent_provider.load_consents_for_user(user_id).await?;
            let mut consent_cache_lock = self.user_consents_cache.write().await;
            consent_cache_lock.insert(user_id.to_string(), consents);
            info!("Loaded consents for user '{}' into cache.", user_id);
        }
        Ok(())
    }

    async fn get_consent_status( &self, user_id: &str, model_id: &str, category: AIDataCategory,) -> Result<AIConsentStatus, AIInteractionError> {
        debug!("Getting consent status for user '{}', model '{}', category '{:?}'", user_id, model_id, category);
        let consents_cache_lock = self.user_consents_cache.read().await;
        let user_consents = consents_cache_lock.get(user_id);

        if let Some(consents) = user_consents {
            if let Some(consent) = consents.iter().find(|c| (c.model_id == model_id || c.model_id == "*") && c.data_category == category) {
                return Ok(consent.status);
            }
        }
        
        let profiles_cache_lock = self.model_profiles_cache.read().await;
        let model_profile = profiles_cache_lock.iter().find(|p| p.model_id == model_id);

        match model_profile {
            Some(profile) => {
                if profile.required_consent_categories.contains(&category) {
                    Ok(AIConsentStatus::PendingUserAction)
                } else {
                    Ok(AIConsentStatus::NotRequired)
                }
            }
            None => {
                if model_id == "*" { Ok(AIConsentStatus::PendingUserAction) } 
                else { Err(AIInteractionError::ModelNotFound(model_id.to_string())) }
            }
        }
    }

    async fn list_available_models(&self) -> Result<Vec<AIModelProfile>, AIInteractionError> {
        debug!("Listing available AI models from cache.");
        let profiles_lock = self.model_profiles_cache.read().await;
        Ok(profiles_lock.clone())
    }

    async fn grant_consent( &self, user_id: &str, model_id: &str, category: AIDataCategory, scope: AIConsentScope,) -> Result<(), AIInteractionError> {
        info!("Granting consent for user '{}', model '{}', category '{:?}', scope '{:?}'", user_id, model_id, category, scope);
        let mut consents_cache_lock = self.user_consents_cache.write().await;
        let user_consents = consents_cache_lock.entry(user_id.to_string()).or_insert_with(Vec::new);
        let new_status = AIConsentStatus::Granted;

        if let Some(existing_consent) = user_consents.iter_mut().find(|c| c.model_id == model_id && c.data_category == category) {
            existing_consent.status = new_status;
            existing_consent.scope = scope;
            existing_consent.last_updated_timestamp = chrono::Utc::now();
            self.consent_provider.save_consent(existing_consent).await?;
        } else {
            let new_consent = AIConsent::new(user_id.to_string(), model_id.to_string(), category, new_status, scope);
            self.consent_provider.save_consent(&new_consent).await?;
            user_consents.push(new_consent);
        }
        if let Err(e) = self.event_publisher.send(AIInteractionEvent::AIConsentUpdated { user_id: user_id.to_string(), model_id: model_id.to_string(), category, new_status, scope, }) {
            warn!("Failed to send AIConsentUpdated event: {}", e);
        }
        Ok(())
    }

    async fn revoke_consent( &self, user_id: &str, model_id: &str, category: AIDataCategory,) -> Result<(), AIInteractionError> {
        info!("Revoking consent for user '{}', model '{}', category '{:?}'", user_id, model_id, category);
        let mut consents_cache_lock = self.user_consents_cache.write().await;
        let user_consents = consents_cache_lock.entry(user_id.to_string()).or_insert_with(Vec::new);
        let new_status = AIConsentStatus::Denied;
        let mut scope_for_event = AIConsentScope::default();

        if let Some(existing_consent) = user_consents.iter_mut().find(|c| c.model_id == model_id && c.data_category == category) {
            existing_consent.status = new_status;
            existing_consent.last_updated_timestamp = chrono::Utc::now();
            scope_for_event = existing_consent.scope;
            self.consent_provider.save_consent(existing_consent).await?;
        } else {
            let new_denied_consent = AIConsent::new(user_id.to_string(), model_id.to_string(), category, new_status, AIConsentScope::PersistentUntilRevoked);
            scope_for_event = new_denied_consent.scope;
            self.consent_provider.save_consent(&new_denied_consent).await?;
            user_consents.push(new_denied_consent);
        }
        if let Err(e) = self.event_publisher.send(AIInteractionEvent::AIConsentUpdated { user_id: user_id.to_string(), model_id: model_id.to_string(), category, new_status, scope: scope_for_event,}) {
            warn!("Failed to send AIConsentUpdated event for denial: {}", e);
        }
        Ok(())
    }

    // --- Iteration 2 Method Implementations ---

    async fn initiate_interaction(
        &self,
        user_id: &str,
        relevant_categories: Vec<AIDataCategory>,
        initial_attachments: Option<Vec<AttachmentData>>,
        prompt_template: Option<String>,
    ) -> Result<Uuid, AIInteractionError> {
        info!("Initiating AI interaction for user '{}', categories: {:?}", user_id, relevant_categories);
        let default_model_opt = self.get_default_model().await?;
        let active_model_id = default_model_opt.as_ref().map(|m| m.model_id.clone());

        let initial_consent_status = if let Some(ref model) = default_model_opt {
            self.check_overall_consent(user_id, &model.model_id, &relevant_categories).await?
        } else {
            // If no default model, or if we want to check general consent for categories without a model yet
            self.check_overall_consent(user_id, "*", &relevant_categories).await?
        };

        let mut context = AIInteractionContext::new(
            relevant_categories,
            initial_attachments,
            prompt_template,
        );
        context.active_model_id = active_model_id;
        context.consent_status = initial_consent_status;
        let context_id = context.id;

        let mut contexts_lock = self.active_contexts.write().await;
        contexts_lock.insert(context_id, context.clone());

        if let Err(e) = self.event_publisher.send(AIInteractionEvent::AIInteractionInitiated { context }) {
            warn!("Failed to send AIInteractionInitiated event: {}", e);
        }
        debug!("AI Interaction context {} initiated.", context_id);
        Ok(context_id)
    }

    async fn get_interaction_context(&self, context_id: Uuid) -> Result<AIInteractionContext, AIInteractionError> {
        let contexts_lock = self.active_contexts.read().await;
        contexts_lock.get(&context_id).cloned()
            .ok_or_else(|| AIInteractionError::InternalError(format!("Interaction context with ID {} not found.", context_id)))
    }

    async fn add_attachment_to_context(&self, context_id: Uuid, attachment: AttachmentData) -> Result<(), AIInteractionError> {
        let mut contexts_lock = self.active_contexts.write().await;
        let context = contexts_lock.get_mut(&context_id)
            .ok_or_else(|| AIInteractionError::InternalError(format!("Interaction context with ID {} not found for adding attachment.", context_id)))?;
        
        context.attachments.push(attachment.clone());
        context.consent_status = AIConsentStatus::PendingUserAction; // Adding attachment might require re-check

        if let Err(e) = self.event_publisher.send(AIInteractionEvent::AIContextUpdated {
            context_id,
            updated_field: "attachment_added".to_string(),
            new_data_preview: Some(format!("Attachment ID: {}", attachment.id)),
        }) {
            warn!("Failed to send AIContextUpdated (attachment_added) event: {}", e);
        }
        debug!("Attachment {} added to context {}.", attachment.id, context_id);
        Ok(())
    }

    async fn update_interaction_history(&self, context_id: Uuid, entry: InteractionHistoryEntry) -> Result<(), AIInteractionError> {
        let mut contexts_lock = self.active_contexts.write().await;
        let context = contexts_lock.get_mut(&context_id)
            .ok_or_else(|| AIInteractionError::InternalError(format!("Interaction context with ID {} not found for updating history.", context_id)))?;
        
        let preview = entry.content.chars().take(50).collect();
        context.history_entries.push(entry);

        if let Err(e) = self.event_publisher.send(AIInteractionEvent::AIContextUpdated {
            context_id,
            updated_field: "history_entry_added".to_string(),
            new_data_preview: Some(preview),
        }) {
            warn!("Failed to send AIContextUpdated (history_entry_added) event: {}", e);
        }
        debug!("History entry added to context {}.", context_id);
        Ok(())
    }

    async fn get_consent_status_for_interaction(
        &self,
        context_id: Uuid,
        model_id: &str,
        required_categories_override: Option<&[AIDataCategory]>,
    ) -> Result<AIConsentStatus, AIInteractionError> {
        let contexts_lock = self.active_contexts.read().await;
        let context = contexts_lock.get(&context_id)
            .ok_or_else(|| AIInteractionError::InternalError(format!("Interaction context with ID {} not found.", context_id)))?;
        
        // TODO: Determine user_id from context if available, or require as param. For now, assuming a placeholder.
        let user_id = "placeholder_user_from_context"; 

        let categories_to_check = required_categories_override.unwrap_or(&context.associated_data_categories);
        self.check_overall_consent(user_id, model_id, categories_to_check).await
    }

    async fn get_default_model(&self) -> Result<Option<AIModelProfile>, AIInteractionError> {
        let profiles_lock = self.model_profiles_cache.read().await;
        Ok(profiles_lock.iter().find(|p| p.is_default_model).cloned())
    }

    async fn set_interaction_active_model(&self, context_id: Uuid, model_id: Option<String>) -> Result<(), AIInteractionError> {
        let mut contexts_lock = self.active_contexts.write().await;
        let context = contexts_lock.get_mut(&context_id)
            .ok_or_else(|| AIInteractionError::InternalError(format!("Interaction context with ID {} not found for setting model.", context_id)))?;
        
        context.active_model_id = model_id.clone();
        // Setting a new model might require re-evaluating consent status for the context's categories
        context.consent_status = AIConsentStatus::PendingUserAction; 

        if let Err(e) = self.event_publisher.send(AIInteractionEvent::AIContextUpdated {
            context_id,
            updated_field: "active_model_id_set".to_string(),
            new_data_preview: model_id,
        }) {
            warn!("Failed to send AIContextUpdated (active_model_id_set) event: {}", e);
        }
        debug!("Active model for context {} set to {:?}.", context_id, context.active_model_id);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_centric_services::events::AIInteractionEvent;
    use std::sync::Arc;
    use tokio::sync::broadcast;
    use tokio::time::{timeout, Duration};
    use crate::user_centric_services::ai_interaction::persistence::{FilesystemAIConsentProvider, FilesystemAIModelProfileProvider}; // For concrete types in new() if needed

    // Mock Providers (from Iteration 1 tests, can be reused/adapted)
    #[derive(Default, Clone)] struct MockAIModelProfileProvider { profiles: Vec<AIModelProfile>, force_error: bool, }
    impl MockAIModelProfileProvider { fn new() -> Self { Default::default() } fn set_profiles(&mut self, profiles: Vec<AIModelProfile>) { self.profiles = profiles; } }
    #[async_trait] impl AIModelProfileProvider for MockAIModelProfileProvider { async fn load_model_profiles(&self) -> Result<Vec<AIModelProfile>, AIInteractionError> { if self.force_error { Err(AIInteractionError::InternalError("Mock profile error".to_string())) } else { Ok(self.profiles.clone()) } } }

    #[derive(Default, Clone)] struct MockAIConsentProvider { consents: HashMap<String, Vec<AIConsent>>, force_error: bool, }
    impl MockAIConsentProvider { fn new() -> Self { Default::default() } fn add_consent(&mut self, consent: AIConsent) { self.consents.entry(consent.user_id.clone()).or_default().push(consent); } }
    #[async_trait] impl AIConsentProvider for MockAIConsentProvider { async fn load_consents_for_user(&self, user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError> { if self.force_error { Err(AIInteractionError::InternalError("Mock consent error".to_string())) } else { Ok(self.consents.get(user_id).cloned().unwrap_or_default()) } } async fn save_consent(&self, consent_to_save: &AIConsent) -> Result<(), AIInteractionError> { if self.force_error { Err(AIInteractionError::InternalError("Mock save error".to_string())) } else { let _ = self.consents.entry(consent_to_save.user_id.clone()).or_default().iter_mut().find(|c| c.id == consent_to_save.id).map(|c| *c = consent_to_save.clone()).unwrap_or_else(|| self.consents.entry(consent_to_save.user_id.clone()).or_default().push(consent_to_save.clone())); Ok(()) } } }
    
    fn test_profile1_default() -> AIModelProfile { AIModelProfile { model_id: "model1_default".to_string(), display_name: "Model One (Default)".to_string(), provider: "TestProv".to_string(), capabilities: vec![AIModelCapability::TextGeneration], required_consent_categories: vec![AIDataCategory::GenericText], is_default_model: true } }
    fn test_profile2_no_default() -> AIModelProfile { AIModelProfile { model_id: "model2_specific".to_string(), display_name: "Model Two".to_string(), provider: "TestProv".to_string(), capabilities: vec![AIModelCapability::CodeGeneration], required_consent_categories: vec![AIDataCategory::FileSystemRead], is_default_model: false } }


    #[tokio::test]
    async fn test_initiate_interaction_with_default_model() {
        let mut profile_provider = MockAIModelProfileProvider::new();
        profile_provider.set_profiles(vec![test_profile1_default(), test_profile2_no_default()]);
        let consent_provider = MockAIConsentProvider::new(); // No consents initially
        let (tx, mut rx) = broadcast::channel(16);
        let service = DefaultAIInteractionLogicService::new(Arc::new(profile_provider), Arc::new(consent_provider), tx);
        service.load_data_into_cache(Some("user_init_test")).await.unwrap(); // Load profiles

        let context_id = service.initiate_interaction(
            "user_init_test", 
            vec![AIDataCategory::GenericText], // Matches default model's reqs
            None, None
        ).await.unwrap();

        let context = service.get_interaction_context(context_id).await.unwrap();
        assert_eq!(context.active_model_id, Some("model1_default".to_string()));
        assert_eq!(context.associated_data_categories, vec![AIDataCategory::GenericText]);
        assert_eq!(context.consent_status, AIConsentStatus::PendingUserAction); // Default model requires GenericText, no consent yet

        match timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap() {
            AIInteractionEvent::AIInteractionInitiated { context: event_ctx } => {
                assert_eq!(event_ctx.id, context_id);
                assert_eq!(event_ctx.active_model_id, Some("model1_default".to_string()));
            }
            _ => panic!("Wrong event type"),
        }
    }
    
    #[tokio::test]
    async fn test_initiate_interaction_no_default_model() {
        let mut profile_provider = MockAIModelProfileProvider::new();
        profile_provider.set_profiles(vec![test_profile2_no_default()]); // No default model
        let consent_provider = MockAIConsentProvider::new();
        let (tx, _) = broadcast::channel(16);
        let service = DefaultAIInteractionLogicService::new(Arc::new(profile_provider), Arc::new(consent_provider), tx);
        service.load_data_into_cache(Some("user_no_default")).await.unwrap();

        let context_id = service.initiate_interaction("user_no_default", vec![AIDataCategory::GenericText], None, None).await.unwrap();
        let context = service.get_interaction_context(context_id).await.unwrap();
        assert!(context.active_model_id.is_none());
        // Consent for "*" and GenericText is PendingUserAction by default if no consent record
        assert_eq!(context.consent_status, AIConsentStatus::PendingUserAction); 
    }

    #[tokio::test]
    async fn test_add_attachment_and_history_and_events() {
        let (tx, mut rx) = broadcast::channel(16);
        let service = DefaultAIInteractionLogicService::new(Arc::new(MockAIModelProfileProvider::new()), Arc::new(MockAIConsentProvider::new()), tx);
        service.load_data_into_cache(Some("user_attach_hist")).await.unwrap();
        let context_id = service.initiate_interaction("user_attach_hist", vec![], None, None).await.unwrap();
        let _ = rx.recv().await; // Consume InteractionInitiated event

        // Add attachment
        let attachment = AttachmentData::new_text("Test content".to_string(), None);
        service.add_attachment_to_context(context_id, attachment.clone()).await.unwrap();
        let context_after_attach = service.get_interaction_context(context_id).await.unwrap();
        assert_eq!(context_after_attach.attachments.len(), 1);
        assert_eq!(context_after_attach.attachments[0].id, attachment.id);
        match timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap() {
            AIInteractionEvent::AIContextUpdated { context_id: cid, updated_field, .. } => {
                assert_eq!(cid, context_id); assert_eq!(updated_field, "attachment_added");
            }
            _ => panic!("Wrong event for attachment"),
        }

        // Add history entry
        let history_entry = InteractionHistoryEntry::new_user_message("Hello AI".to_string(), vec![attachment.id]);
        service.update_interaction_history(context_id, history_entry.clone()).await.unwrap();
        let context_after_history = service.get_interaction_context(context_id).await.unwrap();
        assert_eq!(context_after_history.history_entries.len(), 1);
        assert_eq!(context_after_history.history_entries[0].content, "Hello AI");
        match timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap() {
            AIInteractionEvent::AIContextUpdated { context_id: cid, updated_field, .. } => {
                assert_eq!(cid, context_id); assert_eq!(updated_field, "history_entry_added");
            }
            _ => panic!("Wrong event for history"),
        }
    }

    #[tokio::test]
    async fn test_get_consent_status_for_interaction_logic() {
        let mut profile_provider = MockAIModelProfileProvider::new();
        profile_provider.set_profiles(vec![test_profile1_default()]); // model1_default requires GenericText
        let mut consent_provider = MockAIConsentProvider::new();
        let user_id = "user_consent_interaction";
        
        let (tx, _) = broadcast::channel(16);
        let service = DefaultAIInteractionLogicService::new(Arc::new(profile_provider), Arc::new(consent_provider.clone()), tx); // Clone for later direct use
        service.load_data_into_cache(Some(user_id)).await.unwrap();

        // Context associated with GenericText, active model is model1_default
        let context_id = service.initiate_interaction(user_id, vec![AIDataCategory::GenericText], None, None).await.unwrap();
        
        // Initially, consent should be PendingUserAction
        let status1 = service.get_consent_status_for_interaction(context_id, "model1_default", None).await.unwrap();
        assert_eq!(status1, AIConsentStatus::PendingUserAction);

        // Grant consent for GenericText to model1_default
        service.grant_consent(user_id, "model1_default", AIDataCategory::GenericText, AIConsentScope::PersistentUntilRevoked).await.unwrap();
        
        // Now, consent for interaction should be Granted
        let status2 = service.get_consent_status_for_interaction(context_id, "model1_default", None).await.unwrap();
        assert_eq!(status2, AIConsentStatus::Granted);
        
        // Check with an override category that is not consented
        let status3 = service.get_consent_status_for_interaction(context_id, "model1_default", Some(&[AIDataCategory::UserProfile])).await.unwrap();
        assert_eq!(status3, AIConsentStatus::PendingUserAction); // UserProfile also required by model1_default
    }

    #[tokio::test]
    async fn test_get_default_model_found_and_not_found() {
        let mut profile_provider_with_default = MockAIModelProfileProvider::new();
        profile_provider_with_default.set_profiles(vec![test_profile1_default(), test_profile2_no_default()]);
        let (tx, _) = broadcast::channel(16);
        let service_with_default = DefaultAIInteractionLogicService::new(Arc::new(profile_provider_with_default), Arc::new(MockAIConsentProvider::new()), tx.clone());
        service_with_default.load_data_into_cache(None).await.unwrap();
        let default_model = service_with_default.get_default_model().await.unwrap();
        assert!(default_model.is_some());
        assert_eq!(default_model.unwrap().model_id, "model1_default");

        let mut profile_provider_no_default = MockAIModelProfileProvider::new();
        profile_provider_no_default.set_profiles(vec![test_profile2_no_default()]); // Only non-default
        let service_no_default = DefaultAIInteractionLogicService::new(Arc::new(profile_provider_no_default), Arc::new(MockAIConsentProvider::new()), tx);
        service_no_default.load_data_into_cache(None).await.unwrap();
        let no_default_model = service_no_default.get_default_model().await.unwrap();
        assert!(no_default_model.is_none());
    }
    
    #[tokio::test]
    async fn test_set_interaction_active_model_and_event() {
        let (tx, mut rx) = broadcast::channel(16);
        let service = DefaultAIInteractionLogicService::new(Arc::new(MockAIModelProfileProvider::new()), Arc::new(MockAIConsentProvider::new()), tx);
        service.load_data_into_cache(Some("user_set_model")).await.unwrap();
        let context_id = service.initiate_interaction("user_set_model", vec![], None, None).await.unwrap();
        let _ = rx.recv().await; // Consume InteractionInitiated

        service.set_interaction_active_model(context_id, Some("new_model_id".to_string())).await.unwrap();
        let context = service.get_interaction_context(context_id).await.unwrap();
        assert_eq!(context.active_model_id, Some("new_model_id".to_string()));
        assert_eq!(context.consent_status, AIConsentStatus::PendingUserAction); // Should reset to pending

        match timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap() {
            AIInteractionEvent::AIContextUpdated { context_id: cid, updated_field, new_data_preview } => {
                assert_eq!(cid, context_id);
                assert_eq!(updated_field, "active_model_id_set");
                assert_eq!(new_data_preview, Some("new_model_id".to_string()));
            }
            _ => panic!("Wrong event for set_interaction_active_model"),
        }
    }
}
