use thiserror::Error;
use uuid::Uuid;
use novade_core::errors::CoreError;
use crate::notifications_rules::errors::NotificationRulesError;

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Notification with ID '{0}' not found.")]
    NotFound(Uuid),

    #[error("Invalid input data for notification field '{field}': {reason}")]
    InvalidInputData {
        field: String,
        reason: String,
    },

    #[error("Notification history is full (max: {max_history}). Cannot add notification: '{incoming_summary}'.")]
    HistoryFull {
        max_history: usize,
        incoming_summary: String,
    },

    #[error("Action with key '{action_key}' not found for notification ID '{notification_id}'.")]
    ActionNotFound {
        notification_id: Uuid,
        action_key: String,
    },

    #[error("Failed to invoke action '{action_key}' for notification ID '{notification_id}': {reason}")]
    ActionInvocationFailed {
        notification_id: Uuid,
        action_key: String,
        reason: String,
    },

    #[error("Invalid notification filter criteria: {0}")]
    InvalidFilterCriteria(String),

    #[error("Persistence error during operation '{operation}': {source_message}{}", .source.as_ref().map(|s| format!(": {}", s)).unwrap_or_default())]
    PersistenceError {
        operation: String,
        source_message: String,
        #[source]
        source: Option<CoreError>,
    },

    #[error("Notification rule engine error: {0}")]
    RuleEngineError(#[from] NotificationRulesError),

    #[error("Internal error in notification service: {0}")]
    InternalError(String),
}

impl NotificationError {
    pub fn persistence_error_no_source(operation: impl Into<String>, message: impl Into<String>) -> Self {
        NotificationError::PersistenceError {
            operation: operation.into(),
            source_message: message.into(),
            source: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc;
    use crate::notifications_rules::errors::RuleParsingError; // Example specific error from rules

    #[test]
    fn test_error_messages_display() {
        let notification_id = Uuid::new_v4();
        let action_key = "test_action".to_string();
        let core_error_io_src = Some(Arc::new(io::Error::new(io::ErrorKind::Other, "dummy underlying")));
        let core_error_io = CoreError::IoError("dummy io error".to_string(), core_error_io_src.clone());

        assert_eq!(format!("{}", NotificationError::NotFound(notification_id)), format!("Notification with ID '{}' not found.", notification_id));
        assert_eq!(format!("{}", NotificationError::InvalidInputData { field: "summary".to_string(), reason: "Cannot be empty".to_string() }), "Invalid input data for notification field 'summary': Cannot be empty");
        assert_eq!(format!("{}", NotificationError::HistoryFull { max_history: 100, incoming_summary: "New Notif".to_string() }), "Notification history is full (max: 100). Cannot add notification: 'New Notif'.");
        assert_eq!(format!("{}", NotificationError::ActionNotFound { notification_id, action_key: action_key.clone() }), format!("Action with key 'test_action' not found for notification ID '{}'.", notification_id));
        assert_eq!(format!("{}", NotificationError::ActionInvocationFailed { notification_id, action_key: action_key.clone(), reason: "Callback failed".to_string() }), format!("Failed to invoke action 'test_action' for notification ID '{}': Callback failed", notification_id));
        assert_eq!(format!("{}", NotificationError::InvalidFilterCriteria("Bad regex".to_string())), "Invalid notification filter criteria: Bad regex");
        
        assert_eq!(
            format!("{}", NotificationError::PersistenceError { operation: "load".to_string(), source_message: "Disk read failed".to_string(), source: Some(core_error_io.clone()) }),
            "Persistence error during operation 'load': Disk read failed: dummy io error: dummy underlying"
        );
        assert_eq!(
            format!("{}", NotificationError::persistence_error_no_source("save".to_string(), "Could not write".to_string())),
            "Persistence error during operation 'save': Could not write"
        );

        let rule_parse_error = RuleParsingError { line_number: Some(10), message: "Syntax error in rule".to_string() };
        let rule_engine_error = NotificationRulesError::RuleParsingError(rule_parse_error);
        assert_eq!(format!("{}", NotificationError::RuleEngineError(rule_engine_error)), "Notification rule engine error: Failed to parse rule definition: Syntax error in rule at line 10");
        
        assert_eq!(format!("{}", NotificationError::InternalError("Unexpected state".to_string())), "Internal error in notification service: Unexpected state");
    }
}
