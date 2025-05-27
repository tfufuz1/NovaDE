use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, info, warn, error};
use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;

use super::types::NotificationRuleSet;
use super::errors::NotificationRulesError;
use super::persistence_iface::NotificationRulesProvider;

pub struct FilesystemNotificationRulesProvider {
    pub config_service: Arc<dyn ConfigServiceAsync>,
    pub rules_config_key: String, // e.g., "notifications/rules.toml"
}

impl FilesystemNotificationRulesProvider {
    pub fn new(config_service: Arc<dyn ConfigServiceAsync>, rules_config_key: String) -> Self {
        Self {
            config_service,
            rules_config_key,
        }
    }
}

#[async_trait]
impl NotificationRulesProvider for FilesystemNotificationRulesProvider {
    async fn load_rules(&self) -> Result<NotificationRuleSet, NotificationRulesError> {
        debug!("Loading notification rules from key '{}'", self.rules_config_key);
        match self.config_service.read_config_file_string(&self.rules_config_key).await {
            Ok(toml_string) => {
                toml::from_str(&toml_string).map_err(|e| {
                    error!("Failed to deserialize TOML notification rules from key '{}': {}", self.rules_config_key, e);
                    NotificationRulesError::InternalError(format!("Rule deserialization failed: {}", e))
                })
            }
            Err(core_error) => {
                if core_error.is_not_found_error() { // Assuming CoreError has this method
                    info!("Notification rules file (key '{}') not found. Returning empty rule set.", self.rules_config_key);
                    Ok(Vec::new())
                } else {
                    error!("CoreError loading notification rules (key '{}'): {}", self.rules_config_key, core_error);
                    Err(NotificationRulesError::RulePersistenceError(core_error))
                }
            }
        }
    }

    async fn save_rules(&self, rules: &NotificationRuleSet) -> Result<(), NotificationRulesError> {
        debug!("Saving {} notification rules to key '{}'", rules.len(), self.rules_config_key);
        let toml_string = toml::to_string_pretty(rules).map_err(|e| {
            error!("Failed to serialize notification rules to TOML for key '{}': {}", self.rules_config_key, e);
            NotificationRulesError::InternalError(format!("Rule serialization failed: {}", e))
        })?;

        self.config_service.write_config_file_string(&self.rules_config_key, toml_string).await
            .map_err(NotificationRulesError::RulePersistenceError)?;
        info!("Notification rules saved successfully to key '{}'", self.rules_config_key);
        Ok(())
    }
}


// Mock for CoreError's is_not_found_error for compilation.
#[cfg(test)]
mod core_error_mock_for_rules_persistence {
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
    use crate::notifications_rules::types::{NotificationRule, RuleCondition, RuleAction}; // Corrected path
    use crate::notifications_rules::persistence::core_error_mock_for_rules_persistence::{CoreError as MockCoreError, CoreErrorType as MockCoreErrorType}; // Corrected path
    use novade_core::config::ConfigServiceAsync; // Keep for trait bound
    use std::collections::HashMap;
    use tokio::sync::RwLock; // Ensure RwLock is in scope

    #[derive(Default)]
    struct MockConfigService {
        files: Arc<RwLock<HashMap<String, String>>>,
        force_read_error_type: Option<MockCoreErrorType>,
        force_write_error_type: Option<MockCoreErrorType>,
    }
    impl MockConfigService {
        fn new() -> Self { Default::default() }
        async fn set_file_content(&self, key: &str, content: String) { self.files.write().await.insert(key.to_string(), content); }
        #[allow(dead_code)] fn set_read_error_type(&mut self, error_type: Option<MockCoreErrorType>) { self.force_read_error_type = error_type; }
        #[allow(dead_code)] fn set_write_error_type(&mut self, error_type: Option<MockCoreErrorType>) { self.force_write_error_type = error_type; }
    }

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

    #[tokio::test]
    async fn test_load_rules_file_not_found() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemNotificationRulesProvider::new(mock_config_service, "test_rules.toml".to_string());
        let rules = provider.load_rules().await.unwrap();
        assert!(rules.is_empty());
    }

    #[tokio::test]
    async fn test_save_and_load_rules() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemNotificationRulesProvider::new(mock_config_service.clone(), "test_rules.toml".to_string());
        
        let rules_to_save = vec![
            NotificationRule { name: "Rule1".to_string(), ..Default::default() },
            NotificationRule { name: "Rule2".to_string(), actions: vec![RuleAction::SuppressNotification], ..Default::default() },
        ];

        provider.save_rules(&rules_to_save).await.unwrap();

        let loaded_rules = provider.load_rules().await.unwrap();
        assert_eq!(loaded_rules.len(), 2);
        assert_eq!(loaded_rules[0].name, "Rule1");
        assert_eq!(loaded_rules[1].name, "Rule2");
    }
    
    #[tokio::test]
    async fn test_load_rules_deserialization_error() {
        let mock_config_service = Arc::new(MockConfigService::new());
        mock_config_service.set_file_content("bad_rules.toml", "this is not valid toml {}{".to_string()).await;
        let provider = FilesystemNotificationRulesProvider::new(mock_config_service, "bad_rules.toml".to_string());
        
        let result = provider.load_rules().await;
        assert!(result.is_err());
        match result.err().unwrap() {
            NotificationRulesError::InternalError(msg) => assert!(msg.contains("Rule deserialization failed")),
            _ => panic!("Unexpected error type"),
        }
    }
    
    #[tokio::test]
    async fn test_load_rules_config_service_read_error() {
        let mut mock_service_inner = MockConfigService::new();
        mock_service_inner.set_read_error_type(Some(MockCoreErrorType::IoError));
        let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemNotificationRulesProvider::new(mock_config_service, "error_rules.toml".to_string());

        let result = provider.load_rules().await;
        assert!(result.is_err());
        match result.err().unwrap() {
            NotificationRulesError::RulePersistenceError(core_err) => {
                assert!(core_err.to_string().contains("Forced read error"));
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
