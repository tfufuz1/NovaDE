use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, warn};

use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;

use super::types::NotificationRuleSet;
use super::errors::NotificationRulesError;
use super::persistence_iface::NotificationRulesProvider;

// --- FilesystemNotificationRulesProvider ---

#[derive(Debug)]
pub struct FilesystemNotificationRulesProvider {
    config_service: Arc<dyn ConfigServiceAsync>,
    config_key: String,
}

impl FilesystemNotificationRulesProvider {
    pub fn new(config_service: Arc<dyn ConfigServiceAsync>, config_key: String) -> Self {
        Self {
            config_service,
            config_key,
        }
    }
}

#[async_trait]
impl NotificationRulesProvider for FilesystemNotificationRulesProvider {
    async fn load_rules(&self) -> Result<NotificationRuleSet, NotificationRulesError> {
        debug!("Loading notification rules from key: {}", self.config_key);
        match self.config_service.read_config_file_string(&self.config_key).await {
            Ok(content) => {
                debug!("Successfully read rules file content for key: {}", self.config_key);
                let rules: NotificationRuleSet = serde_json::from_str(&content).map_err(|e| {
                    warn!("Failed to deserialize notification rules from key '{}': {}", self.config_key, e);
                    NotificationRulesError::RuleParsingError {
                        details: format!("Failed to parse JSON content for rules from key '{}': {}", self.config_key, e),
                        source: Some(e),
                    }
                })?;
                
                // Further validation of individual rules (e.g., regex compilation) could be added here
                // or be the responsibility of the rules engine after loading.
                // For now, successfully parsing the structure is the main goal of this provider.
                debug!("Notification rules deserialized. Rule count: {}", rules.len());
                Ok(rules)
            }
            Err(e) => {
                if e.is_not_found() {
                    debug!("Notification rules file not found for key '{}'. Returning empty rule set.", self.config_key);
                    Ok(Vec::new()) // Empty rule set if no file
                } else {
                    error!("Failed to read notification rules file for key '{}': {}", self.config_key, e);
                    Err(NotificationRulesError::RulePersistenceError(e))
                }
            }
        }
    }

    async fn save_rules(&self, rules: &NotificationRuleSet) -> Result<(), NotificationRulesError> {
        debug!("Saving {} notification rules to key: {}", rules.len(), self.config_key);
        let serialized_content = serde_json::to_string_pretty(rules).map_err(|e| {
            error!("Failed to serialize notification rules for key '{}': {}", self.config_key, e);
            // Using RuleParsingError for serialization errors as per prompt's error structure.
            // A dedicated SerializationError variant in NotificationRulesError might be cleaner.
            NotificationRulesError::RuleParsingError { 
                details: format!("Failed to serialize rules to JSON for key '{}': {}", self.config_key, e),
                source: Some(e),
            }
        })?;

        self.config_service.write_config_file_string(&self.config_key, &serialized_content).await
            .map_err(|e| {
                error!("Failed to write notification rules file to key '{}': {}", self.config_key, e);
                NotificationRulesError::RulePersistenceError(e)
            })?;
        debug!("Notification rules saved successfully to key: {}", self.config_key);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::config::MockConfigServiceAsync;
    use crate::notifications_rules::types::{NotificationRule, RuleCondition, RuleAction, SimpleRuleCondition, RuleConditionField, RuleConditionOperator, RuleConditionValue};
    use uuid::Uuid;
    use std::io;

    fn new_mock_arc_config_service() -> Arc<MockConfigServiceAsync> {
        Arc::new(MockConfigServiceAsync::new())
    }

    fn create_test_rule(name: &str) -> NotificationRule {
        NotificationRule {
            id: Uuid::new_v4(),
            name: name.to_string(),
            condition: RuleCondition::Simple(SimpleRuleCondition {
                field: RuleConditionField::ApplicationName,
                operator: RuleConditionOperator::Is,
                value: RuleConditionValue::String("TestApp".to_string()),
            }),
            actions: vec![RuleAction::SuppressNotification],
            is_enabled: true,
            priority: 0,
        }
    }

    #[tokio::test]
    async fn test_load_rules_success() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let rules = vec![create_test_rule("Rule1")];
        let rules_json = serde_json::to_string_pretty(&rules).unwrap();

        mock_config_service.expect_read_config_file_string()
            .withf(|key| key == "test_rules.json")
            .times(1)
            .returning(move |_| Ok(rules_json.clone()));

        let provider = FilesystemNotificationRulesProvider::new(Arc::new(mock_config_service), "test_rules.json".to_string());
        let loaded_rules = provider.load_rules().await.unwrap();
        assert_eq!(loaded_rules.len(), 1);
        assert_eq!(loaded_rules[0].name, "Rule1");
    }

    #[tokio::test]
    async fn test_load_rules_not_found_returns_empty() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let not_found_error = CoreError::IoError(
            "Simulated file not found".to_string(), 
            Some(Arc::new(io::Error::new(io::ErrorKind::NotFound, "not found")))
        );

        mock_config_service.expect_read_config_file_string()
            .times(1)
            .returning(move |_| Err(not_found_error.clone()));

        let provider = FilesystemNotificationRulesProvider::new(Arc::new(mock_config_service), "non_existent_rules.json".to_string());
        let loaded_rules = provider.load_rules().await.unwrap();
        assert!(loaded_rules.is_empty());
    }

    #[tokio::test]
    async fn test_load_rules_corrupted_content() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let corrupted_json = "this is not valid json {{{{";

        mock_config_service.expect_read_config_file_string()
            .times(1)
            .returning(move |_| Ok(corrupted_json.to_string()));

        let provider = FilesystemNotificationRulesProvider::new(Arc::new(mock_config_service), "corrupted_rules.json".to_string());
        let result = provider.load_rules().await;
        assert!(matches!(result, Err(NotificationRulesError::RuleParsingError { .. })));
    }
    
    #[tokio::test]
    async fn test_load_rules_other_read_error() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let generic_io_error = CoreError::IoError(
            "Simulated generic IO error".to_string(), 
            Some(Arc::new(io::Error::new(io::ErrorKind::Other, "other error")))
        );
        mock_config_service.expect_read_config_file_string()
            .times(1)
            .returning(move |_| Err(generic_io_error.clone()));

        let provider = FilesystemNotificationRulesProvider::new(Arc::new(mock_config_service), "error_rules.json".to_string());
        let result = provider.load_rules().await;
        assert!(matches!(result, Err(NotificationRulesError::RulePersistenceError(_))));
    }

    #[tokio::test]
    async fn test_save_rules_success() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let rules = vec![create_test_rule("RuleToSave")];
        let expected_rules_json = serde_json::to_string_pretty(&rules).unwrap();
        
        mock_config_service.expect_write_config_file_string()
            .withf(move |key, content| {
                key == "test_save_rules.json" && content == expected_rules_json
            })
            .times(1)
            .returning(|_, _| Ok(()));

        let provider = FilesystemNotificationRulesProvider::new(Arc::new(mock_config_service), "test_save_rules.json".to_string());
        let result = provider.save_rules(&rules).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_save_rules_write_error() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let rules = vec![create_test_rule("RuleSaveFail")];
        let write_error = CoreError::IoError("Simulated write error".to_string(), None);

        mock_config_service.expect_write_config_file_string()
            .times(1)
            .returning(move |_, _| Err(write_error.clone()));
            
        let provider = FilesystemNotificationRulesProvider::new(Arc::new(mock_config_service), "write_fail_rules.json".to_string());
        let result = provider.save_rules(&rules).await;
        assert!(matches!(result, Err(NotificationRulesError::RulePersistenceError(_))));
    }
}
