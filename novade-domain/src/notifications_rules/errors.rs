use thiserror::Error;
use uuid::Uuid;
use regex; // For regex::Error
use crate::global_settings::errors::GlobalSettingsError;
use novade_core::errors::CoreError;
use serde_json; // Added for serde_json::Error

#[derive(Debug, Error)]
pub enum NotificationRulesError {
    #[error("Invalid rule definition for rule '{rule_name}' (ID: {rule_id:?}): {reason}")]
    InvalidRuleDefinition {
        rule_id: Option<Uuid>,
        rule_name: String,
        reason: String,
    },

    #[error("Condition evaluation error for rule '{rule_name}' (ID: {rule_id:?}): {details}{}", .source.as_ref().map(|s| format!(": {}", s)).unwrap_or_default())]
    ConditionEvaluationError {
        rule_id: Option<Uuid>,
        rule_name: String,
        details: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },

    #[error("Action application error for rule '{rule_name}' (ID: {rule_id:?}): {details}")]
    ActionApplicationError {
        rule_id: Option<Uuid>,
        rule_name: String,
        details: String,
    },

    #[error("Settings access error: {0}")]
    SettingsAccessError(#[from] GlobalSettingsError),

    #[error("Rule persistence error: {0}")]
    RulePersistenceError(#[from] CoreError),

    #[error("Invalid regex pattern '{pattern}': {source}")]
    InvalidRegex {
        pattern: String,
        #[source]
        source: regex::Error,
    },
    
    #[error("Failed to parse rule definition: {details}{}", .source.as_ref().map(|s| format!(": {}", s)).unwrap_or_default())]
    RuleParsingError {
        details: String,
        #[source]
        source: Option<serde_json::Error>, // Changed to use serde_json::Error as source
    },

    #[error("Internal error in notification rules engine: {0}")]
    InternalError(String),
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_settings::paths::SettingPath;
    use std::io;
    use std::sync::Arc;

    // Helper to create a dummy serde_json::Error for testing
    fn dummy_serde_json_error() -> serde_json::Error {
        serde_json::from_str::<i32>("invalid json").unwrap_err()
    }

    #[test]
    fn test_error_messages_display() {
        let rule_id = Some(Uuid::new_v4());
        let rule_name = "Test Rule".to_string();

        assert_eq!(
            format!("{}", NotificationRulesError::InvalidRuleDefinition { rule_id, rule_name: rule_name.clone(), reason: "Missing action".to_string() }),
            format!("Invalid rule definition for rule 'Test Rule' (ID: Some({})): Missing action", rule_id.unwrap())
        );
        
        let dummy_source_error_fmt = std::fmt::Error; // Example of a simple error that is Send + Sync + 'static
        let dummy_source_error = Box::new(dummy_source_error_fmt);
        assert_eq!(
            format!("{}", NotificationRulesError::ConditionEvaluationError { rule_id, rule_name: rule_name.clone(), details: "Regex mismatch".to_string(), source: Some(dummy_source_error) }),
            format!("Condition evaluation error for rule 'Test Rule' (ID: Some({})): Regex mismatch: an error occurred when formatting an argument", rule_id.unwrap())
        );
        
        assert_eq!(
            format!("{}", NotificationRulesError::ConditionEvaluationError { rule_id, rule_name: rule_name.clone(), details: "No source".to_string(), source: None }),
            format!("Condition evaluation error for rule 'Test Rule' (ID: Some({})): No source", rule_id.unwrap())
        );

        assert_eq!(
            format!("{}", NotificationRulesError::ActionApplicationError { rule_id, rule_name: rule_name.clone(), details: "Sound not found".to_string() }),
            format!("Action application error for rule 'Test Rule' (ID: Some({})): Sound not found", rule_id.unwrap())
        );

        let dummy_setting_path = SettingPath::Root; 
        let gs_error = GlobalSettingsError::ValidationError { path: dummy_setting_path, reason: "Setting invalid".to_string() };
        assert_eq!(
            format!("{}", NotificationRulesError::SettingsAccessError(gs_error)),
            "Settings access error: Validation failed for path 'root': Setting invalid"
        );

        let core_error_io_src = Some(Arc::new(io::Error::new(io::ErrorKind::PermissionDenied, "access denied")));
        let core_error = CoreError::IoError("File system error".to_string(), core_error_io_src);
        assert_eq!(
            format!("{}", NotificationRulesError::RulePersistenceError(core_error)),
            "Rule persistence error: File system error: access denied"
        );

        let regex_error_source = regex::Regex::new("[").unwrap_err(); 
        let regex_error_source_string = regex_error_source.to_string();
        assert_eq!(
            format!("{}", NotificationRulesError::InvalidRegex { pattern: "[".to_string(), source: regex_error_source }),
            format!("Invalid regex pattern '[': {}", regex_error_source_string)
        );

        let serde_error_source = dummy_serde_json_error();
        let serde_error_string = serde_error_source.to_string(); // Capture for comparison
        assert_eq!(
            format!("{}", NotificationRulesError::RuleParsingError{ details: "Bad JSON syntax".to_string(), source: Some(serde_error_source) }),
            format!("Failed to parse rule definition: Bad JSON syntax: {}", serde_error_string)
        );
        assert_eq!(
            format!("{}", NotificationRulesError::RuleParsingError{ details: "Bad JSON syntax no source".to_string(), source: None }),
            "Failed to parse rule definition: Bad JSON syntax no source"
        );

        assert_eq!(
            format!("{}", NotificationRulesError::InternalError("State corrupted".to_string())),
            "Internal error in notification rules engine: State corrupted"
        );
    }
}
