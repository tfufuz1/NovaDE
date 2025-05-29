use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock}; // Using RwLock for shared data
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{debug, error, info, warn};

use super::types::{
    AIInteractionContext, AIConsent, AIModelProfile, AIDataCategory, AttachmentData,
    InteractionHistoryEntry, AIConsentStatus, AIConsentScope, AIModelCapability,
};
use super::errors::AIInteractionError;
use super::persistence_iface::{AIConsentProvider, AIModelProfileProvider};
use crate::user_centric_services::events::AIInteractionEventEnum;
// crate::shared_types::ApplicationId not directly used for user_id, String is used.

// --- AIInteractionLogicService Trait ---

#[async_trait]
pub trait AIInteractionLogicService: Send + Sync {
    async fn initiate_interaction(&self, relevant_categories: Vec<AIDataCategory>, initial_attachments: Option<Vec<AttachmentData>>, user_prompt_template: Option<String>) -> Result<Uuid, AIInteractionError>;
    async fn get_interaction_context(&self, context_id: Uuid) -> Result<AIInteractionContext, AIInteractionError>;
    async fn provide_consent(&self, context_id: Option<Uuid>, user_id: String, model_id: String, granted_categories: Vec<AIDataCategory>, consent_decision: bool, scope: AIConsentScope, expiry_timestamp: Option<DateTime<Utc>>) -> Result<Uuid, AIInteractionError>;
    async fn get_consent_status_for_interaction(&self, context_id: Uuid, model_id: &str, required_categories: &[AIDataCategory], user_id: &str) -> Result<AIConsentStatus, AIInteractionError>;
    async fn add_attachment_to_context(&self, context_id: Uuid, attachment: AttachmentData) -> Result<(), AIInteractionError>;
    async fn update_interaction_history(&self, context_id: Uuid, entry: InteractionHistoryEntry) -> Result<(), AIInteractionError>;
    async fn list_available_models(&self) -> Result<Vec<AIModelProfile>, AIInteractionError>;
    async fn get_model_profile(&self, model_id: &str) -> Result<AIModelProfile, AIInteractionError>;
    async fn get_default_model(&self) -> Result<AIModelProfile, AIInteractionError>;
    async fn reload_model_profiles(&self) -> Result<usize, AIInteractionError>;
    async fn get_all_user_consents(&self, user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError>;
    async fn revoke_user_consent(&self, consent_id: Uuid, user_id: &str) -> Result<(), AIInteractionError>;
    fn subscribe_to_ai_events(&self) -> broadcast::Receiver<AIInteractionEventEnum>;
}

// --- DefaultAIInteractionLogicService Implementation ---

pub struct DefaultAIInteractionLogicService {
    active_contexts: Arc<RwLock<HashMap<Uuid, AIInteractionContext>>>,
    model_profiles: Arc<RwLock<Vec<AIModelProfile>>>,
    user_consents_cache: Arc<RwLock<HashMap<String, Vec<AIConsent>>>>,
    consent_provider: Arc<dyn AIConsentProvider>,
    profile_provider: Arc<dyn AIModelProfileProvider>,
    event_publisher: broadcast::Sender<AIInteractionEventEnum>,
    default_user_id_for_service: String,
}

impl DefaultAIInteractionLogicService {
    pub async fn new(
        consent_provider: Arc<dyn AIConsentProvider>,
        profile_provider: Arc<dyn AIModelProfileProvider>,
        default_user_id: String,
        broadcast_capacity: usize,
    ) -> Result<Self, AIInteractionError> {
        let (event_publisher, _) = broadcast::channel(broadcast_capacity);
        let service = Self {
            active_contexts: Arc::new(RwLock::new(HashMap::new())),
            model_profiles: Arc::new(RwLock::new(Vec::new())),
            user_consents_cache: Arc::new(RwLock::new(HashMap::new())),
            consent_provider,
            profile_provider,
            event_publisher,
            default_user_id_for_service: default_user_id.clone(),
        };

        service.reload_model_profiles_internal().await.map_err(|e| {
            error!("Failed to load model profiles during service initialization: {}", e);
            e // Propagate error
        })?;
        service.load_consents_for_user_internal(&default_user_id).await.map_err(|e| {
            error!("Failed to load initial consents for default user '{}': {}", default_user_id, e);
            // Decide if this is fatal. For now, let's say it is.
            e
        })?;

        Ok(service)
    }

    async fn reload_model_profiles_internal(&self) -> Result<usize, AIInteractionError> {
        debug!("Reloading model profiles internally...");
        let profiles = self.profile_provider.load_model_profiles().await?;
        let profiles_count = profiles.len();
        let mut profiles_guard = self.model_profiles.write().await;
        *profiles_guard = profiles;
        debug!("Model profiles reloaded. Count: {}", profiles_count);
        Ok(profiles_count)
    }

    async fn load_consents_for_user_internal(&self, user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError> {
        debug!("Loading consents internally for user: {}", user_id);
        let consents = self.consent_provider.load_consents_for_user(user_id).await?;
        let mut cache_guard = self.user_consents_cache.write().await;
        cache_guard.insert(user_id.to_string(), consents.clone());
        debug!("Consents loaded into cache for user: {}. Count: {}", user_id, consents.len());
        Ok(consents)
    }
}

#[async_trait]
impl AIInteractionLogicService for DefaultAIInteractionLogicService {
    async fn initiate_interaction(
        &self,
        relevant_categories: Vec<AIDataCategory>,
        initial_attachments: Option<Vec<AttachmentData>>,
        user_prompt_template: Option<String>,
    ) -> Result<Uuid, AIInteractionError> {
        let context_id = Uuid::new_v4();
        let new_context = AIInteractionContext {
            id: context_id,
            creation_timestamp: Utc::now(),
            active_model_id: None, // Can be set later
            consent_status: AIConsentStatus::PendingUserAction, // Default, to be checked
            associated_data_categories: relevant_categories,
            history_entries: Vec::new(),
            attachments: initial_attachments.unwrap_or_default(),
            user_prompt_template,
            is_active: true,
        };

        self.active_contexts.write().await.insert(context_id, new_context.clone());
        debug!("Interaction initiated with context ID: {}", context_id);

        if self.event_publisher.send(AIInteractionEventEnum::InteractionInitiated { context: new_context }).is_err() {
            error!("Failed to send InteractionInitiated event for context {}", context_id);
        }
        Ok(context_id)
    }

    async fn get_interaction_context(&self, context_id: Uuid) -> Result<AIInteractionContext, AIInteractionError> {
        self.active_contexts.read().await.get(&context_id).cloned().ok_or(AIInteractionError::ContextNotFound(context_id))
    }

    async fn provide_consent(
        &self,
        context_id: Option<Uuid>,
        user_id: String, // Assuming this is the user whose consent is being set
        model_id: String,
        granted_categories: Vec<AIDataCategory>,
        consent_decision: bool,
        scope: AIConsentScope,
        expiry_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Uuid, AIInteractionError> {
        if granted_categories.is_empty() {
            warn!("provide_consent called with no granted_categories for user '{}', model '{}'.", user_id, model_id);
            // Decide if this is an error or a no-op. Let's make it an error.
            return Err(AIInteractionError::InternalError("No categories provided for consent.".to_string()));
        }
        
        let mut first_consent_id = Uuid::nil(); // Placeholder

        for (i, category) in granted_categories.iter().enumerate() {
            let new_consent = AIConsent {
                id: Uuid::new_v4(),
                user_id: user_id.clone(),
                model_id: model_id.clone(),
                data_category: *category,
                granted_timestamp: Utc::now(),
                expiry_timestamp,
                is_revoked: !consent_decision, // If decision is false, it's effectively revoked/denied
                last_used_timestamp: None,
                consent_scope: scope,
            };
            if i == 0 { first_consent_id = new_consent.id; }

            self.consent_provider.save_consent(&new_consent).await.map_err(|e| {
                // If one save fails, we might want to roll back previous saves in this loop.
                // For now, returning the first error.
                error!("Failed to save consent for user '{}', category {:?}: {}", user_id, category, e);
                e
            })?;
            
            // Update cache for this specific user_id
            let mut cache_guard = self.user_consents_cache.write().await;
            let user_consents = cache_guard.entry(user_id.clone()).or_default(); // Use or_default
            user_consents.retain(|c| !(c.model_id == model_id && c.data_category == *category && c.user_id == user_id)); // Remove old
            user_consents.push(new_consent.clone());

            if self.event_publisher.send(AIInteractionEventEnum::ConsentUpdated {
                user_id: user_id.clone(),
                model_id: model_id.clone(),
                category: *category,
                new_status: if consent_decision { AIConsentStatus::Granted } else { AIConsentStatus::Denied },
                scope,
            }).is_err() {
                error!("Failed to send ConsentUpdated event for user '{}', category {:?}", user_id, category);
            }
        }

        if let Some(ctx_id) = context_id { // Update context if provided
            let mut contexts_guard = self.active_contexts.write().await;
            if let Some(context) = contexts_guard.get_mut(&ctx_id) {
                // This is a simplified update. A full re-evaluation might be needed.
                // Call get_consent_status_for_interaction to correctly update the context's overall consent status.
                // This requires careful lock management if get_consent_status_for_interaction also takes locks.
                // For now, let's assume a simpler update or that get_consent_status_for_interaction is efficient with locks.
                let new_context_status = self.get_consent_status_for_interaction(ctx_id, &model_id, &context.associated_data_categories, &user_id).await
                    .unwrap_or(AIConsentStatus::PendingUserAction); // Fallback on error
                context.consent_status = new_context_status;
                if self.event_publisher.send(AIInteractionEventEnum::ContextUpdated{context_id: ctx_id, updated_field: "consent_status".to_string()}).is_err(){
                     error!("Failed to send ContextUpdated event for consent_status on context {}", ctx_id);
                }
            }
        }
        Ok(first_consent_id) // Return the ID of the first consent created in the batch
    }

    async fn get_consent_status_for_interaction(
        &self,
        _context_id: Uuid, // Context might influence user_id or model in future
        model_id: &str,
        required_categories: &[AIDataCategory],
        user_id: &str,
    ) -> Result<AIConsentStatus, AIInteractionError> {
        if required_categories.is_empty() { return Ok(AIConsentStatus::NotRequired); }

        let model_profiles_guard = self.model_profiles.read().await;
        let _model_profile = model_profiles_guard.iter().find(|p| p.model_id == model_id)
            .ok_or_else(|| AIInteractionError::ModelNotFound(model_id.to_string()))?;
        // TODO: Validate if required_categories are a subset of model_profile.required_consent_categories

        // Attempt to load consents if not in cache, carefully managing locks.
        let consents_loaded = {
            let cache_read_guard = self.user_consents_cache.read().await;
            if cache_read_guard.contains_key(user_id) {
                cache_read_guard.get(user_id).cloned().unwrap_or_default()
            } else {
                drop(cache_read_guard); // Release read lock before internal call that takes write lock
                // This internal call will load and cache.
                self.load_consents_for_user_internal(user_id).await?
            }
        };
        
        let mut overall_status = AIConsentStatus::Granted; // Assume granted until a required consent is missing/denied
        for req_category in required_categories {
            let found_and_valid_consent = consents_loaded.iter().any(|consent|
                consent.user_id == user_id &&
                consent.model_id == model_id && // TODO: Consider wildcard model_id consents (e.g., consent for all models of a provider)
                consent.data_category == *req_category &&
                !consent.is_revoked &&
                (consent.expiry_timestamp.is_none() || consent.expiry_timestamp.unwrap() > Utc::now())
            );
            if !found_and_valid_consent {
                // If any required category is not granted and valid, the overall status cannot be Granted.
                // Check if it's explicitly denied (revoked) for this specific model/category
                if consents_loaded.iter().any(|c| c.user_id == user_id && c.model_id == model_id && c.data_category == *req_category && c.is_revoked) {
                    return Ok(AIConsentStatus::Denied); // Explicitly denied for this category takes precedence
                }
                overall_status = AIConsentStatus::PendingUserAction; // At least one required category is not granted
            }
        }
        Ok(overall_status)
    }

    async fn add_attachment_to_context(&self, context_id: Uuid, attachment: AttachmentData) -> Result<(), AIInteractionError> {
        let mut contexts_guard = self.active_contexts.write().await;
        let context = contexts_guard.get_mut(&context_id).ok_or(AIInteractionError::ContextNotFound(context_id))?;
        context.attachments.push(attachment);
        debug!("Attachment added to context ID: {}", context_id);
        if self.event_publisher.send(AIInteractionEventEnum::ContextUpdated{context_id, updated_field: "attachments".to_string()}).is_err() {
            error!("Failed to send ContextUpdated event for attachments on context {}", context_id);
        }
        Ok(())
    }

    async fn update_interaction_history(&self, context_id: Uuid, entry: InteractionHistoryEntry) -> Result<(), AIInteractionError> {
        let mut contexts_guard = self.active_contexts.write().await;
        let context = contexts_guard.get_mut(&context_id).ok_or(AIInteractionError::ContextNotFound(context_id))?;
        context.history_entries.push(entry);
         debug!("History entry added to context ID: {}", context_id);
        if self.event_publisher.send(AIInteractionEventEnum::ContextUpdated{context_id, updated_field: "history_entries".to_string()}).is_err() {
            error!("Failed to send ContextUpdated event for history_entries on context {}", context_id);
        }
        Ok(())
    }

    async fn list_available_models(&self) -> Result<Vec<AIModelProfile>, AIInteractionError> { Ok(self.model_profiles.read().await.clone()) }
    async fn get_model_profile(&self, model_id: &str) -> Result<AIModelProfile, AIInteractionError> {
        self.model_profiles.read().await.iter().find(|p| p.model_id == model_id).cloned().ok_or_else(|| AIInteractionError::ModelNotFound(model_id.to_string()))
    }
    async fn get_default_model(&self) -> Result<AIModelProfile, AIInteractionError> {
        self.model_profiles.read().await.iter().find(|p| p.is_default_model).cloned().ok_or(AIInteractionError::NoDefaultModelConfigured)
    }

    async fn reload_model_profiles(&self) -> Result<usize, AIInteractionError> {
        let count = self.reload_model_profiles_internal().await?;
        if self.event_publisher.send(AIInteractionEventEnum::ModelProfilesReloaded { profiles_count: count }).is_err() {
            error!("Failed to send ModelProfilesReloaded event. Count: {}", count);
        }
        Ok(count)
    }

    async fn get_all_user_consents(&self, user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError> {
        let cache_read_guard = self.user_consents_cache.read().await;
        if let Some(consents) = cache_read_guard.get(user_id) {
            Ok(consents.clone())
        } else {
            drop(cache_read_guard); // Release read lock before potential write in load_consents_for_user_internal
            self.load_consents_for_user_internal(user_id).await
        }
    }

    async fn revoke_user_consent(&self, consent_id: Uuid, user_id: &str) -> Result<(), AIInteractionError> {
        self.consent_provider.revoke_consent(consent_id, user_id).await?;
        let updated_consents = self.consent_provider.load_consents_for_user(user_id).await?; // Re-load from source
        
        let revoked_consent_details = updated_consents.iter().find(|c| c.id == consent_id && c.is_revoked).cloned();

        let mut cache_guard = self.user_consents_cache.write().await;
        cache_guard.insert(user_id.to_string(), updated_consents);

        if let Some(details) = revoked_consent_details {
            if self.event_publisher.send(AIInteractionEventEnum::ConsentUpdated {
                user_id: user_id.to_string(), model_id: details.model_id.clone(), category: details.data_category,
                new_status: AIConsentStatus::Denied, scope: details.consent_scope,
            }).is_err() {
                error!("Failed to send ConsentUpdated event for revoke of consent {}", consent_id);
            }
        } else { warn!("Revoked consent ID {} not found or not marked revoked after reload for user {}.", consent_id, user_id); }
        Ok(())
    }

    fn subscribe_to_ai_events(&self) -> broadcast::Receiver<AIInteractionEventEnum> { self.event_publisher.subscribe() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_centric_services::ai_interaction::persistence_iface::{MockAIConsentProvider, MockAIModelProfileProvider};
    use tokio::sync::broadcast::error::RecvError;
    use chrono::Duration;
    use crate::user_centric_services::ai_interaction::types::{AIDataCategory, AIConsentStatus, AIConsentScope, AIModelProfile, AIConsent};


    // Default impls for tests
    impl Default for AIModelProfile {
        fn default() -> Self { Self { model_id: Uuid::new_v4().to_string(), display_name: "Default Model".into(), description: "".into(), provider: "".into(), required_consent_categories: vec![], capabilities: vec![], supports_streaming: false, endpoint_url: None, api_key_secret_name: None, is_default_model: false, sort_order: 0 } }
    }
    impl Default for AIConsent {
        fn default() -> Self { Self { id: Uuid::new_v4(), user_id: "default_user".into(), model_id: "default_model".into(), data_category: AIDataCategory::GenericText, granted_timestamp: Utc::now(), expiry_timestamp: None, is_revoked: false, last_used_timestamp: None, consent_scope: Default::default() } }
    }

    #[tokio::test]
    async fn test_initiate_interaction_success() {
        let consent_provider = Arc::new(MockAIConsentProvider::new());
        let mut profile_provider = MockAIModelProfileProvider::new();
        profile_provider.expect_load_model_profiles().returning(|| Ok(vec![])); 
        consent_provider.expect_load_consents_for_user().returning(|_| Ok(vec![]));

        let service = DefaultAIInteractionLogicService::new(consent_provider, Arc::new(profile_provider), "test_user".to_string(), 5).await.unwrap();
        let mut rx = service.subscribe_to_ai_events();

        let categories = vec![AIDataCategory::GenericText];
        let result = service.initiate_interaction(categories.clone(), None, None).await;
        assert!(result.is_ok());
        let context_id = result.unwrap();

        let context = service.get_interaction_context(context_id).await.unwrap();
        assert_eq!(context.id, context_id);
        assert_eq!(context.associated_data_categories, categories);
        assert!(context.is_active);

        match rx.try_recv() {
            Ok(AIInteractionEventEnum::InteractionInitiated { context: event_context }) => {
                assert_eq!(event_context.id, context_id);
            }
            res => panic!("Expected InteractionInitiated event, got {:?}", res),
        }
    }
    
    #[tokio::test]
    async fn test_provide_consent_and_get_status() {
        let user_id = "consent_user_test".to_string(); // Unique user_id for this test
        let model_id_str = "test_model_for_consent_status".to_string(); // Unique model_id
        let category = AIDataCategory::UserProfile;

        let mut consent_provider = MockAIConsentProvider::new();
        let mut profile_provider = MockAIModelProfileProvider::new();

        // Mock for new() - initial load of profiles and default user consents
        profile_provider.expect_load_model_profiles().times(1).returning(move || {
             Ok(vec![AIModelProfile { model_id: model_id_str.clone(), required_consent_categories: vec![category], ..Default::default() }])
        });
        // Initial load for default_user_id_for_service.
        consent_provider.expect_load_consents_for_user().times(1).returning(|uid_param| {
             if uid_param == "test_user_provide_consent_status" { Ok(Vec::new()) } else { Ok(Vec::new()) /* Default for others */ }
        });
        
        // Mock for provide_consent's save_consent call
        consent_provider.expect_save_consent().times(1).returning(|consent_arg| {
            assert_eq!(consent_arg.user_id, user_id);
            assert_eq!(consent_arg.model_id, model_id_str);
            assert_eq!(consent_arg.data_category, category);
            Ok(())
        });
        
        // Mock for load_consents_for_user_internal if called by get_consent_status_for_interaction
        // (it will be if cache is empty for this user, which it is initially)
        // This also covers the load within provide_consent if its cache logic misses.
        let user_id_clone_for_load = user_id.clone();
        let model_id_clone_for_load = model_id_str.clone();
        consent_provider.expect_load_consents_for_user().times(1).returning(move |uid_param| {
            if uid_param == user_id_clone_for_load {
                let saved_consent = AIConsent {
                    id: Uuid::new_v4(), user_id: user_id_clone_for_load.clone(), model_id: model_id_clone_for_load.clone(),
                    data_category: category, granted_timestamp: Utc::now(), is_revoked: false,
                    consent_scope: AIConsentScope::SessionOnly, ..Default::default()
                };
                Ok(vec![saved_consent])
            } else { Ok(Vec::new()) }
        });

        let service = DefaultAIInteractionLogicService::new(Arc::new(consent_provider), Arc::new(profile_provider), "test_user_provide_consent_status".to_string(), 5).await.unwrap();
        let mut rx = service.subscribe_to_ai_events();

        let consent_result = service.provide_consent(None, user_id.clone(), model_id_str.clone(), vec![category], true, AIConsentScope::SessionOnly, None).await;
        assert!(consent_result.is_ok());
        
        assert!(matches!(rx.try_recv(), Ok(AIInteractionEventEnum::ConsentUpdated { new_status, .. }) if new_status == AIConsentStatus::Granted ));

        let status = service.get_consent_status_for_interaction(Uuid::new_v4(), &model_id_str, &[category], &user_id).await.unwrap();
        assert_eq!(status, AIConsentStatus::Granted);
    }
    
    #[tokio::test]
    async fn test_get_default_model_none_configured() {
        let consent_provider = Arc::new(MockAIConsentProvider::new());
        let mut profile_provider = MockAIModelProfileProvider::new();
        profile_provider.expect_load_model_profiles().returning(|| Ok(vec![AIModelProfile{ model_id: "m1".into(), is_default_model: false, ..Default::default()}]));
        consent_provider.expect_load_consents_for_user().returning(|_| Ok(vec![]));
        let service = DefaultAIInteractionLogicService::new(consent_provider, Arc::new(profile_provider), "test_user".to_string(), 5).await.unwrap();
        assert!(matches!(service.get_default_model().await, Err(AIInteractionError::NoDefaultModelConfigured)));
    }

    #[tokio::test]
    async fn test_revoke_consent() {
        let user_id = "revoke_user_test".to_string(); // Unique user ID
        let consent_id_to_revoke = Uuid::new_v4();
        let model_id = "model_for_revoke_test".to_string(); // Unique model ID
        let category = AIDataCategory::LocationData;

        let mut consent_provider = MockAIConsentProvider::new();
        let mut profile_provider = MockAIModelProfileProvider::new();
        profile_provider.expect_load_model_profiles().returning(|| Ok(vec![]));

        let initial_consent = AIConsent { id: consent_id_to_revoke, user_id: user_id.clone(), model_id: model_id.clone(), data_category: category, is_revoked: false, ..Default::default() };
        
        // For new()
        consent_provider.expect_load_consents_for_user().times(1).returning(move |uid_param| {
            if uid_param == user_id { Ok(vec![initial_consent.clone()]) } else { Ok(Vec::new()) }
        });
        // For revoke_user_consent -> revoke_consent (provider call)
        consent_provider.expect_revoke_consent().times(1).withf(move |cid, uid| *cid == consent_id_to_revoke && uid == user_id).returning(|_,_| Ok(()));
        // For revoke_user_consent -> load_consents_for_user (after revoke to update cache)
        let user_id_clone_for_load = user_id.clone();
        let model_id_clone_for_load = model_id.clone();
        consent_provider.expect_load_consents_for_user().times(1).returning(move |uid_param| {
            if uid_param == user_id_clone_for_load {
                let revoked_consent = AIConsent { id: consent_id_to_revoke, user_id: user_id_clone_for_load.clone(), model_id: model_id_clone_for_load.clone(), data_category: category, is_revoked: true, ..Default::default() };
                Ok(vec![revoked_consent])
            } else { Ok(Vec::new()) }
        });

        let service = DefaultAIInteractionLogicService::new(Arc::new(consent_provider), Arc::new(profile_provider), user_id.clone(), 5).await.unwrap();
        let mut rx = service.subscribe_to_ai_events();

        let revoke_result = service.revoke_user_consent(consent_id_to_revoke, &user_id).await;
        assert!(revoke_result.is_ok());

        let updated_consents = service.get_all_user_consents(&user_id).await.unwrap();
        assert_eq!(updated_consents.len(), 1);
        assert!(updated_consents[0].is_revoked);

        assert!(matches!(rx.try_recv(), Ok(AIInteractionEventEnum::ConsentUpdated { new_status, .. }) if new_status == AIConsentStatus::Denied ));
    }
}
