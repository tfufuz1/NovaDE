use async_trait::async_trait;
// use serde::{Deserialize, Serialize}; // serde_json is used directly, this might not be needed.
use std::sync::Arc;
use tracing::{debug, error, warn}; // info might be useful too

use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;
use novade_core::ConfigError; // Assuming ConfigError is part of novade_core and used by CoreError::Config

use crate::notifications::rules_types::NotificationRuleSet; // Was super::types::NotificationRuleSet
use crate::notifications::rules_errors::NotificationRulesError; // Was super::errors::NotificationRulesError
use crate::notifications::persistence_iface::NotificationRulesProvider; // Was super::persistence_iface::NotificationRulesProvider

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

                debug!("Notification rules deserialized. Rule count: {}", rules.len());
                Ok(rules)
            }
            Err(core_error) => {
                // Adapt is_not_found_error check
                match core_error {
                    CoreError::Config(ConfigError::NotFound { .. }) | CoreError::Io(ref io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
                        debug!("Notification rules file not found for key '{}'. Returning empty rule set.", self.config_key);
                        Ok(Vec::new()) // Empty rule set if no file
                    }
                    _ => {
                        error!("Failed to read notification rules file for key '{}': {}", self.config_key, core_error);
                        Err(NotificationRulesError::RulePersistenceError(core_error))
                    }
                }
            }
        }
    }

    async fn save_rules(&self, rules: &NotificationRuleSet) -> Result<(), NotificationRulesError> {
        debug!("Saving {} notification rules to key: {}", rules.len(), self.config_key);
        let serialized_content = serde_json::to_string_pretty(rules).map_err(|e| {
            error!("Failed to serialize notification rules for key '{}': {}", self.config_key, e);
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
