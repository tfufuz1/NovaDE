use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, warn};
use uuid::Uuid;

use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;

use super::types::{AIConsent, AIModelProfile};
use super::errors::AIInteractionError;
use super::persistence_iface::{AIConsentProvider, AIModelProfileProvider};

// --- FilesystemAIConsentProvider ---

#[derive(Debug)]
pub struct FilesystemAIConsentProvider {
    config_service: Arc<dyn ConfigServiceAsync>,
    consents_config_key: String,
    user_id: String,
}

impl FilesystemAIConsentProvider {
    pub fn new(config_service: Arc<dyn ConfigServiceAsync>, base_config_path: &str, user_id: String) -> Self {
        let consents_config_key = format!("{}/ai_consents/{}.json", base_config_path.trim_end_matches('/'), user_id);
        Self {
            config_service,
            consents_config_key,
            user_id,
        }
    }
}

#[async_trait]
impl AIConsentProvider for FilesystemAIConsentProvider {
    async fn load_consents_for_user(&self, user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError> {
        if self.user_id != user_id {
            return Err(AIInteractionError::InternalError(
                format!("Provider instance is for user '{}', but operation requested for user '{}'.", self.user_id, user_id)
            ));
        }
        debug!("Loading consents for user '{}' from key '{}'", self.user_id, self.consents_config_key);

        match self.config_service.read_config_file_string(&self.consents_config_key).await {
            Ok(content) => {
                serde_json::from_str(&content).map_err(|e| {
                    error!("Failed to deserialize consents for user '{}' from key '{}': {}", self.user_id, self.consents_config_key, e);
                    AIInteractionError::ConsentStorageError {
                        operation: "deserialize".to_string(),
                        source_message: format!("Failed to parse consent data for user '{}': {}", self.user_id, e),
                        source: None, 
                    }
                })
            }
            Err(e) => {
                if e.is_not_found() {
                    debug!("Consent file not found for user '{}' at key '{}'. Returning empty list.", self.user_id, self.consents_config_key);
                    Ok(Vec::new())
                } else {
                    error!("Failed to read consent file for user '{}' from key '{}': {}", self.user_id, self.consents_config_key, e);
                    Err(AIInteractionError::ConsentStorageError {
                        operation: "load".to_string(),
                        source_message: format!("Failed to read consent file for user '{}': {}", self.user_id, e),
                        source: Some(e),
                    })
                }
            }
        }
    }

    async fn save_consent(&self, consent: &AIConsent) -> Result<(), AIInteractionError> {
        if self.user_id != consent.user_id {
            return Err(AIInteractionError::InternalError(
                format!("Attempting to save consent for user '{}' using provider instance for user '{}'.", consent.user_id, self.user_id)
            ));
        }
        debug!("Saving consent ID '{}' for user '{}' to key '{}'", consent.id, self.user_id, self.consents_config_key);

        let mut consents = self.load_consents_for_user(&self.user_id).await.unwrap_or_else(|e| {
            warn!("Failed to load existing consents before save for user '{}': {}. Starting with an empty list.", self.user_id, e);
            Vec::new()
        });
        
        consents.retain(|c| c.id != consent.id);
        consents.push(consent.clone());

        let serialized_content = serde_json::to_string_pretty(&consents).map_err(|e| {
            error!("Failed to serialize consents for user '{}': {}", self.user_id, e);
            AIInteractionError::ConsentStorageError {
                operation: "serialize".to_string(),
                source_message: format!("Failed to serialize consent data for user '{}': {}", self.user_id, e),
                source: None,
            }
        })?;

        self.config_service.write_config_file_string(&self.consents_config_key, &serialized_content).await
            .map_err(|e| {
                error!("Failed to write consent file for user '{}' to key '{}': {}", self.user_id, self.consents_config_key, e);
                AIInteractionError::ConsentStorageError {
                    operation: "save".to_string(),
                    source_message: format!("Failed to write consent file for user '{}': {}", self.user_id, e),
                    source: Some(e),
                }
            })
    }

    async fn revoke_consent(&self, consent_id: Uuid, user_id: &str) -> Result<(), AIInteractionError> {
        if self.user_id != user_id {
             return Err(AIInteractionError::InternalError(
                format!("Attempting to revoke consent for user '{}' using provider instance for user '{}'.", user_id, self.user_id)
            ));
        }
        debug!("Revoking consent ID '{}' for user '{}' from key '{}'", consent_id, self.user_id, self.consents_config_key);

        let mut consents = self.load_consents_for_user(&self.user_id).await?;
        
        let consent_to_revoke = consents.iter_mut().find(|c| c.id == consent_id);

        match consent_to_revoke {
            Some(consent) => {
                consent.is_revoked = true;
                // Optionally, set a revoked_timestamp if the struct supports it
                // consent.revoked_timestamp = Some(Utc::now());
            }
            None => {
                warn!("Consent ID '{}' not found for user '{}' during revoke.", consent_id, self.user_id);
                // Using ContextNotFound as ConsentNotFound isn't defined. This could be improved.
                return Err(AIInteractionError::ContextNotFound(consent_id)); 
            }
        }

        let serialized_content = serde_json::to_string_pretty(&consents).map_err(|e| {
             error!("Failed to serialize consents after revoke for user '{}': {}", self.user_id, e);
            AIInteractionError::ConsentStorageError {
                operation: "serialize_after_revoke".to_string(),
                source_message: format!("Failed to serialize consent data for user '{}' after revoke: {}", self.user_id, e),
                source: None,
            }
        })?;
        
        self.config_service.write_config_file_string(&self.consents_config_key, &serialized_content).await
            .map_err(|e| {
                error!("Failed to write revoked consent file for user '{}': {}", self.user_id, e);
                AIInteractionError::ConsentStorageError {
                    operation: "save_after_revoke".to_string(),
                    source_message: format!("Failed to write revoked consent data for user '{}': {}", self.user_id, e),
                    source: Some(e),
                }
            })
    }
}


// --- FilesystemAIModelProfileProvider ---

#[derive(Debug)]
pub struct FilesystemAIModelProfileProvider {
    config_service: Arc<dyn ConfigServiceAsync>,
    profiles_config_key: String,
}

impl FilesystemAIModelProfileProvider {
    pub fn new(config_service: Arc<dyn ConfigServiceAsync>, config_key: String) -> Self {
        Self {
            config_service,
            profiles_config_key,
        }
    }
}

#[async_trait]
impl AIModelProfileProvider for FilesystemAIModelProfileProvider {
    async fn load_model_profiles(&self) -> Result<Vec<AIModelProfile>, AIInteractionError> {
        debug!("Loading model profiles from key '{}'", self.profiles_config_key);
        match self.config_service.read_config_file_string(&self.profiles_config_key).await {
            Ok(content) => {
                let profiles: Vec<AIModelProfile> = serde_json::from_str(&content).map_err(|e| {
                     error!("Failed to deserialize model profiles from key '{}': {}", self.profiles_config_key, e);
                    AIInteractionError::ModelProfileLoadError {
                        source_message: format!("Failed to parse model profiles data: {}", e),
                        source: CoreError::ConfigError(format!("Deserialization error: {}", e)),
                    }
                })?;

                let mut model_ids = std::collections::HashSet::new();
                let mut default_count = 0;
                for profile in &profiles {
                    if !model_ids.insert(&profile.model_id) {
                        let err_msg = format!("Duplicate model_id '{}' found in profiles.", profile.model_id);
                        return Err(AIInteractionError::ModelProfileLoadError {
                            source_message: err_msg, source: CoreError::ConfigError("Validation failed".to_string()),
                        });
                    }
                    if profile.is_default_model { default_count += 1; }
                }
                if default_count > 1 {
                    return Err(AIInteractionError::ModelProfileLoadError {
                        source_message: "Multiple default AI models configured.".to_string(),
                        source: CoreError::ConfigError("Validation failed".to_string()),
                    });
                }
                Ok(profiles)
            }
            Err(e) => {
                if e.is_not_found() {
                    debug!("Model profiles file not found at key '{}'. Returning empty list.", self.profiles_config_key);
                    Ok(Vec::new())
                } else {
                    error!("Failed to read model profiles file from key '{}': {}", self.profiles_config_key, e);
                    Err(AIInteractionError::ModelProfileLoadError {
                        source_message: format!("Failed to read model profiles file: {}", e), source: e,
                    })
                }
            }
        }
    }

    async fn save_model_profiles(&self, profiles: &[AIModelProfile]) -> Result<(), AIInteractionError> {
        debug!("Saving {} model profiles to key '{}'", profiles.len(), self.profiles_config_key);
        let mut model_ids = std::collections::HashSet::new();
        let mut default_count = 0;
        for profile in profiles {
            if !model_ids.insert(&profile.model_id) {
                return Err(AIInteractionError::ModelProfileSaveError {
                    source_message: format!("Duplicate model_id '{}' found during save validation.", profile.model_id),
                    source: CoreError::ConfigError("Validation failed".to_string()),
                });
            }
            if profile.is_default_model { default_count += 1; }
        }
        if default_count > 1 {
            return Err(AIInteractionError::ModelProfileSaveError {
                source_message: "Attempted to save multiple default AI models.".to_string(),
                source: CoreError::ConfigError("Validation failed".to_string()),
            });
        }

        let serialized_content = serde_json::to_string_pretty(profiles).map_err(|e| {
            error!("Failed to serialize model profiles: {}", e);
            AIInteractionError::ModelProfileSaveError {
                source_message: format!("Failed to serialize model profiles: {}", e),
                source: CoreError::ConfigError(format!("Serialization error: {}",e)),
            }
        })?;

        self.config_service.write_config_file_string(&self.profiles_config_key, &serialized_content).await
            .map_err(|e| {
                error!("Failed to write model profiles file to key '{}': {}", self.profiles_config_key, e);
                AIInteractionError::ModelProfileSaveError {
                    source_message: format!("Failed to write model profiles file: {}", e), source: e,
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::config::MockConfigServiceAsync;
    use crate::user_centric_services::ai_interaction::types::{AIDataCategory, AIConsentScope, AIConsentStatus};
    use chrono::Utc;
    use std::io;

    // Default impl for AIModelProfile for easier test setup
    impl Default for AIModelProfile {
        fn default() -> Self {
            Self {
                model_id: Uuid::new_v4().to_string(), display_name: "Default Model".to_string(),
                description: "Default description".to_string(), provider: "DefaultProvider".to_string(),
                required_consent_categories: vec![], capabilities: vec![], supports_streaming: false,
                endpoint_url: None, api_key_secret_name: None, is_default_model: false, sort_order: 0,
            }
        }
    }
    // Default impl for AIConsent for easier test setup
    impl Default for AIConsent {
        fn default() -> Self {
            Self {
                id: Uuid::new_v4(), user_id: "default_user".to_string(), model_id: "default_model".to_string(),
                data_category: AIDataCategory::GenericText, granted_timestamp: Utc::now(),
                expiry_timestamp: None, is_revoked: false, last_used_timestamp: None,
                consent_scope: AIConsentScope::default(),
            }
        }
    }

    fn new_mock_arc_config_service() -> Arc<MockConfigServiceAsync> {
        Arc::new(MockConfigServiceAsync::new())
    }

    #[tokio::test]
    async fn consent_load_success() {
        let mut mock_service = MockConfigServiceAsync::new();
        let user_id = "user123".to_string();
        let key = format!("test_path/ai_consents/{}.json", user_id);
        let consent1 = AIConsent { id: Uuid::new_v4(), user_id: user_id.clone(), ..Default::default() };
        let consents_vec = vec![consent1.clone()];
        let consents_json = serde_json::to_string(&consents_vec).unwrap();

        mock_service.expect_read_config_file_string().withf(move |k| k == key).returning(move |_| Ok(consents_json.clone()));
        let provider = FilesystemAIConsentProvider::new(Arc::new(mock_service), "test_path", user_id.clone());
        let loaded_consents = provider.load_consents_for_user(&user_id).await.unwrap();
        assert_eq!(loaded_consents.len(), 1);
        assert_eq!(loaded_consents[0].id, consent1.id);
    }

    #[tokio::test]
    async fn consent_load_not_found() {
        let mut mock_service = MockConfigServiceAsync::new();
        let user_id = "user404".to_string();
        let not_found_err = CoreError::IoError("not found".into(), Some(Arc::new(io::Error::new(io::ErrorKind::NotFound, "file not found"))));
        mock_service.expect_read_config_file_string().returning(move |_| Err(not_found_err.clone()));
        let provider = FilesystemAIConsentProvider::new(Arc::new(mock_service), "test_path", user_id.clone());
        let loaded_consents = provider.load_consents_for_user(&user_id).await.unwrap();
        assert!(loaded_consents.is_empty());
    }
    
    #[tokio::test]
    async fn consent_load_corrupted() {
        let mut mock_service = MockConfigServiceAsync::new();
        let user_id = "user_corrupt".to_string();
        mock_service.expect_read_config_file_string().returning(|_| Ok("corrupted json".to_string()));
        let provider = FilesystemAIConsentProvider::new(Arc::new(mock_service), "test_path", user_id.clone());
        let result = provider.load_consents_for_user(&user_id).await;
        assert!(matches!(result, Err(AIInteractionError::ConsentStorageError { operation, .. }) if operation == "deserialize"));
    }
    
    #[tokio::test]
    async fn consent_load_wrong_user_instance() {
        let provider = FilesystemAIConsentProvider::new(new_mock_arc_config_service(), "test_path", "user_A".to_string());
        assert!(matches!(provider.load_consents_for_user("user_B").await, Err(AIInteractionError::InternalError(_))));
    }

    #[tokio::test]
    async fn consent_save_new_and_update() {
        let mut mock_service = MockConfigServiceAsync::new();
        let user_id = "user_save".to_string();
        let consent1_id = Uuid::new_v4();

        // 1. Initial load for first save (not found -> empty vec)
        mock_service.expect_read_config_file_string().times(1).returning(|_| {
            let not_found_err = CoreError::IoError("not found".into(), Some(Arc::new(io::Error::new(io::ErrorKind::NotFound, "file not found"))));
            Err(not_found_err)
        });
        // 2. First write
        mock_service.expect_write_config_file_string().times(1).returning(|_, content_written| {
            let written_consents: Vec<AIConsent> = serde_json::from_str(content_written).unwrap();
            assert_eq!(written_consents.len(), 1);
            assert_eq!(written_consents[0].id, consent1_id);
            assert_eq!(written_consents[0].model_id, "m1_initial");
            Ok(())
        });
        // 3. Load for second save (should contain consent1)
        mock_service.expect_read_config_file_string().times(1).returning(move |_| {
            let existing_consent = AIConsent { id: consent1_id, user_id: user_id.clone(), model_id: "m1_initial".into(), ..Default::default() };
            Ok(serde_json::to_string(&vec![existing_consent]).unwrap())
        });
        // 4. Second write (update)
        mock_service.expect_write_config_file_string().times(1).returning(move |_, content_written| {
            let written_consents: Vec<AIConsent> = serde_json::from_str(content_written).unwrap();
            assert_eq!(written_consents.len(), 1);
            assert_eq!(written_consents[0].id, consent1_id);
            assert_eq!(written_consents[0].model_id, "m1_updated"); // Check if updated
            Ok(())
        });

        let provider = FilesystemAIConsentProvider::new(Arc::new(mock_service), "test_path", user_id.clone());
        
        let consent1 = AIConsent { id: consent1_id, user_id: user_id.clone(), model_id: "m1_initial".into(), ..Default::default() };
        provider.save_consent(&consent1).await.unwrap();
        
        let consent2_updated = AIConsent { id: consent1_id, user_id: user_id.clone(), model_id: "m1_updated".into(), data_category: AIDataCategory::UserProfile, ..Default::default() };
        provider.save_consent(&consent2_updated).await.unwrap();
    }

    #[tokio::test]
    async fn profile_load_success() {
        let mut mock_service = MockConfigServiceAsync::new();
        let key = "ai/model_profiles.json".to_string();
        let profile1 = AIModelProfile { model_id: "p1".into(), is_default_model: true, ..Default::default() };
        let profiles_json = serde_json::to_string(&vec![profile1.clone()]).unwrap();
        mock_service.expect_read_config_file_string().withf(move |k| k == key).returning(move |_| Ok(profiles_json.clone()));
        
        let provider = FilesystemAIModelProfileProvider::new(Arc::new(mock_service), "ai/model_profiles.json".to_string());
        let loaded_profiles = provider.load_model_profiles().await.unwrap();
        assert_eq!(loaded_profiles.len(), 1);
        assert_eq!(loaded_profiles[0].model_id, "p1");
    }

    #[tokio::test]
    async fn profile_load_duplicate_ids() {
        let mut mock_service = MockConfigServiceAsync::new();
        let profile1 = AIModelProfile { model_id: "p_dup".into(), ..Default::default() };
        let profile2 = AIModelProfile { model_id: "p_dup".into(), ..Default::default() };
        let profiles_json = serde_json::to_string(&vec![profile1, profile2]).unwrap();
        mock_service.expect_read_config_file_string().returning(move |_| Ok(profiles_json.clone()));
        let provider = FilesystemAIModelProfileProvider::new(Arc::new(mock_service), "ai/model_profiles.json".to_string());
        assert!(matches!(provider.load_model_profiles().await, Err(AIInteractionError::ModelProfileLoadError{ source_message, .. }) if source_message.contains("Duplicate model_id")));
    }

    #[tokio::test]
    async fn profile_save_success() {
        let mut mock_service = MockConfigServiceAsync::new();
        let profile1 = AIModelProfile { model_id: "p_save".into(), ..Default::default() };
        let profiles_to_save = vec![profile1];
        
        mock_service.expect_write_config_file_string().times(1).returning(|_, content| {
            let written: Vec<AIModelProfile> = serde_json::from_str(content).unwrap();
            assert_eq!(written.len(), 1); assert_eq!(written[0].model_id, "p_save"); Ok(())
        });
        let provider = FilesystemAIModelProfileProvider::new(Arc::new(mock_service), "ai/model_profiles.json".to_string());
        provider.save_model_profiles(&profiles_to_save).await.unwrap();
    }
}
