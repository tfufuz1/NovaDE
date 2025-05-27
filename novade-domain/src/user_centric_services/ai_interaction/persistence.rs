use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, info, warn, error};
use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;
use crate::user_centric_services::ai_interaction::types::{AIConsent, AIModelProfile}; // Corrected path
use crate::user_centric_services::ai_interaction::errors::AIInteractionError; // Corrected path
use crate::user_centric_services::ai_interaction::persistence_iface::{AIConsentProvider, AIModelProfileProvider}; // Corrected path

// --- FilesystemAIConsentProvider ---
pub struct FilesystemAIConsentProvider {
    pub config_service: Arc<dyn ConfigServiceAsync>,
    pub user_consents_path_prefix: String, // e.g., "ai/consents/"
}

impl FilesystemAIConsentProvider {
    pub fn new(config_service: Arc<dyn ConfigServiceAsync>, user_consents_path_prefix: String) -> Self {
        Self { config_service, user_consents_path_prefix }
    }

    fn get_consent_file_key(&self, user_id: &str) -> String {
        format!("{}{}.consents.toml", self.user_consents_path_prefix, user_id)
    }
}

#[async_trait]
impl AIConsentProvider for FilesystemAIConsentProvider {
    async fn load_consents_for_user(&self, user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError> {
        let file_key = self.get_consent_file_key(user_id);
        debug!("Loading consents for user '{}' from key '{}'", user_id, file_key);

        match self.config_service.read_config_file_string(&file_key).await {
            Ok(toml_string) => {
                toml::from_str(&toml_string).map_err(|e| {
                    error!("Failed to deserialize TOML consents for user '{}' from key '{}': {}", user_id, file_key, e);
                    AIInteractionError::InternalError(format!("Consent deserialization failed: {}", e))
                })
            }
            Err(core_error) => {
                if core_error.is_not_found_error() {
                    info!("Consent file for user '{}' (key '{}') not found. Returning empty list.", user_id, file_key);
                    Ok(Vec::new())
                } else {
                    error!("CoreError loading consents for user '{}' (key '{}'): {}", user_id, file_key, core_error);
                    Err(AIInteractionError::persistence_error_from_core("load_consents".to_string(), format!("Failed to read consent file for user {}", user_id), core_error))
                }
            }
        }
    }

    async fn save_consent(&self, consent: &AIConsent) -> Result<(), AIInteractionError> {
        let user_id = &consent.user_id;
        let file_key = self.get_consent_file_key(user_id);
        debug!("Saving consent for user '{}' to key '{}'", user_id, file_key);

        // Load existing, then add/update, then save all. This is not atomic.
        // For higher concurrency, a different strategy might be needed (e.g., per-consent files or a DB).
        let mut consents = self.load_consents_for_user(user_id).await.unwrap_or_else(|err| {
            warn!("Error loading consents for user '{}' during save operation (key '{}'): {:?}. Starting with empty list for this save.", user_id, file_key, err);
            Vec::new()
        });

        if let Some(existing_consent) = consents.iter_mut().find(|c| c.id == consent.id) {
            *existing_consent = consent.clone();
            debug!("Updated existing consent ID '{}' for user '{}'", consent.id, user_id);
        } else {
            consents.push(consent.clone());
            debug!("Added new consent ID '{}' for user '{}'", consent.id, user_id);
        }

        let toml_string = toml::to_string_pretty(&consents).map_err(|e| {
            error!("Failed to serialize consents to TOML for user '{}' (key '{}'): {}", user_id, file_key, e);
            AIInteractionError::InternalError(format!("Consent serialization failed: {}", e))
        })?;

        self.config_service.write_config_file_string(&file_key, toml_string).await.map_err(|core_error| {
            AIInteractionError::persistence_error_from_core("save_consent".to_string(), format!("Failed to write consent file for user {}", user_id), core_error)
        })?;
        info!("Successfully saved consents for user '{}' to key '{}'", user_id, file_key);
        Ok(())
    }
}

// --- FilesystemAIModelProfileProvider ---
pub struct FilesystemAIModelProfileProvider {
    pub config_service: Arc<dyn ConfigServiceAsync>,
    pub profiles_config_key: String, // e.g., "ai/model_profiles.toml"
}

impl FilesystemAIModelProfileProvider {
    pub fn new(config_service: Arc<dyn ConfigServiceAsync>, profiles_config_key: String) -> Self {
        Self { config_service, profiles_config_key }
    }
}

#[async_trait]
impl AIModelProfileProvider for FilesystemAIModelProfileProvider {
    async fn load_model_profiles(&self) -> Result<Vec<AIModelProfile>, AIInteractionError> {
        debug!("Loading AI model profiles from key '{}'", self.profiles_config_key);
        match self.config_service.read_config_file_string(&self.profiles_config_key).await {
            Ok(toml_string) => {
                toml::from_str(&toml_string).map_err(|e| {
                    error!("Failed to deserialize TOML AI model profiles from key '{}': {}", self.profiles_config_key, e);
                    AIInteractionError::InternalError(format!("Model profile deserialization failed: {}",e))
                })
            }
            Err(core_error) => {
                 if core_error.is_not_found_error() {
                    info!("AI model profiles file (key '{}') not found. Returning empty list.", self.profiles_config_key);
                    Ok(Vec::new())
                } else {
                    error!("CoreError loading AI model profiles (key '{}'): {}", self.profiles_config_key, core_error);
                    Err(AIInteractionError::persistence_error_from_core("load_model_profiles".to_string(), "Failed to read model profiles file".to_string(), core_error))
                }
            }
        }
    }
}


// Mock for CoreError's is_not_found_error for compilation.
// This should be part of the actual CoreError definition.
#[cfg(test)]
mod core_error_mock_for_ai_persistence {
    use std::fmt;
    use thiserror::Error;

    #[derive(Error, Debug, Clone)]
    pub enum CoreErrorType { NotFound, IoError, Other(String) }

    #[derive(Error, Debug, Clone)]
    pub struct CoreError { pub error_type: CoreErrorType, message: String }

    impl fmt::Display for CoreError { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "CoreError ({:?}): {}", self.error_type, self.message) } }
    impl CoreError {
        pub fn new(error_type: CoreErrorType, message: String) -> Self { Self { error_type, message } }
        pub fn is_not_found_error(&self) -> bool { matches!(self.error_type, CoreErrorType::NotFound) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_centric_services::ai_interaction::types::{AIDataCategory, AIConsentStatus, AIConsentScope, AIModelCapability};
    use crate::user_centric_services::ai_interaction::persistence::core_error_mock_for_ai_persistence::{CoreError as MockCoreError, CoreErrorType as MockCoreErrorType};
    use novade_core::config::ConfigServiceAsync; // Keep for trait bound
    use std::collections::HashMap;
    use uuid::Uuid;

    #[derive(Default)]
    struct MockConfigService {
        files: Arc<RwLock<HashMap<String, String>>>, // Use RwLock for interior mutability
        force_read_error_type: Option<MockCoreErrorType>,
        force_write_error_type: Option<MockCoreErrorType>,
    }
    impl MockConfigService {
        fn new() -> Self { Default::default() }
        async fn set_file_content(&self, key: &str, content: String) { self.files.write().await.insert(key.to_string(), content); }
        #[allow(dead_code)] fn set_read_error_type(&mut self, error_type: Option<MockCoreErrorType>) { self.force_read_error_type = error_type; }
        #[allow(dead_code)] fn set_write_error_type(&mut self, error_type: Option<MockCoreErrorType>) { self.force_write_error_type = error_type; }
    }
    
    use tokio::sync::RwLock; // Ensure RwLock is in scope

    #[async_trait]
    impl ConfigServiceAsync for MockConfigService {
        async fn read_config_file_string(&self, key: &str) -> Result<String, novade_core::errors::CoreError> {
            if let Some(ref err_type) = self.force_read_error_type {
                return Err(MockCoreError::new(err_type.clone(), format!("Forced read error on {}", key)));
            }
            match self.files.read().await.get(key) {
                Some(content) => Ok(content.clone()),
                None => Err(MockCoreError::new(MockCoreErrorType::NotFound, format!("File not found: {}", key))),
            }
        }
        async fn write_config_file_string(&self, key: &str, content: String) -> Result<(), novade_core::errors::CoreError> {
            if let Some(ref err_type) = self.force_write_error_type {
                return Err(MockCoreError::new(err_type.clone(), format!("Forced write error on {}", key)));
            }
            self.files.write().await.insert(key.to_string(), content);
            Ok(())
        }
        async fn read_file_to_string(&self, _path: &std::path::Path) -> Result<String, novade_core::errors::CoreError> { unimplemented!() }
        async fn list_files_in_dir(&self, _dir_path: &std::path::Path, _extension: Option<&str>) -> Result<Vec<std::path::PathBuf>, novade_core::errors::CoreError> { unimplemented!() }
        async fn get_config_dir(&self) -> std::path::PathBuf { unimplemented!() }
        async fn get_data_dir(&self) -> std::path::PathBuf { unimplemented!() }
    }

    // --- AIConsentProvider Tests ---
    #[tokio::test]
    async fn test_consent_provider_load_no_file() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemAIConsentProvider::new(mock_config_service, "ai_test/consents/".to_string());
        let consents = provider.load_consents_for_user("user_no_file").await.unwrap();
        assert!(consents.is_empty());
    }

    #[tokio::test]
    async fn test_consent_provider_save_and_load_consent() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemAIConsentProvider::new(mock_config_service, "ai_test/consents/".to_string());
        let user_id = "user_save_load";
        
        let consent1 = AIConsent::new(user_id.to_string(), "model1".to_string(), AIDataCategory::GenericText, AIConsentStatus::Granted, AIConsentScope::PersistentUntilRevoked);
        provider.save_consent(&consent1).await.unwrap();

        let loaded_consents = provider.load_consents_for_user(user_id).await.unwrap();
        assert_eq!(loaded_consents.len(), 1);
        assert_eq!(loaded_consents[0], consent1);

        // Save another, ensure it appends/updates correctly
        let consent2 = AIConsent::new(user_id.to_string(), "model1".to_string(), AIDataCategory::UserProfile, AIConsentStatus::Denied, AIConsentScope::SessionOnly);
        provider.save_consent(&consent2).await.unwrap();
        
        let loaded_consents_updated = provider.load_consents_for_user(user_id).await.unwrap();
        assert_eq!(loaded_consents_updated.len(), 2);
        assert!(loaded_consents_updated.contains(&consent1));
        assert!(loaded_consents_updated.contains(&consent2));
        
        // Update consent1
        let mut updated_consent1 = consent1.clone();
        updated_consent1.status = AIConsentStatus::Denied;
        provider.save_consent(&updated_consent1).await.unwrap();
        
        let loaded_final = provider.load_consents_for_user(user_id).await.unwrap();
        assert_eq!(loaded_final.len(), 2);
        let final_c1 = loaded_final.iter().find(|c| c.id == consent1.id).unwrap();
        assert_eq!(final_c1.status, AIConsentStatus::Denied);
    }
    
    #[tokio::test]
    async fn test_consent_provider_load_deserialization_error() {
        let mock_config_service = Arc::new(MockConfigService::new());
        mock_config_service.set_file_content("ai_test/consents/user_bad_toml.consents.toml", "this is not valid toml {}{".to_string()).await;
        let provider = FilesystemAIConsentProvider::new(mock_config_service, "ai_test/consents/".to_string());
        
        let result = provider.load_consents_for_user("user_bad_toml").await;
        assert!(result.is_err());
        match result.err().unwrap() {
            AIInteractionError::InternalError(msg) => assert!(msg.contains("Consent deserialization failed")),
            _ => panic!("Unexpected error type"),
        }
    }

    // --- AIModelProfileProvider Tests ---
    #[tokio::test]
    async fn test_profile_provider_load_no_file() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemAIModelProfileProvider::new(mock_config_service, "ai_test/model_profiles.toml".to_string());
        let profiles = provider.load_model_profiles().await.unwrap();
        assert!(profiles.is_empty());
    }

    #[tokio::test]
    async fn test_profile_provider_load_success() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let profiles_data = vec![
            AIModelProfile { model_id: "m1".to_string(), display_name: "Model 1".to_string(), provider: "P1".to_string(), capabilities: vec![AIModelCapability::TextGeneration], required_consent_categories: vec![AIDataCategory::GenericText], is_default_model: true },
            AIModelProfile { model_id: "m2".to_string(), display_name: "Model 2".to_string(), provider: "P2".to_string(), capabilities: vec![AIModelCapability::CodeGeneration], required_consent_categories: vec![], is_default_model: false },
        ];
        let toml_content = toml::to_string_pretty(&profiles_data).unwrap();
        mock_config_service.set_file_content("ai_test/model_profiles.toml", toml_content).await;
        
        let provider = FilesystemAIModelProfileProvider::new(mock_config_service, "ai_test/model_profiles.toml".to_string());
        let loaded_profiles = provider.load_model_profiles().await.unwrap();
        assert_eq!(loaded_profiles.len(), 2);
        assert_eq!(loaded_profiles, profiles_data);
    }
}
