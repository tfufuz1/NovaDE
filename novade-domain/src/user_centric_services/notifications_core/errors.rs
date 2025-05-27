use thiserror::Error;
use uuid::Uuid;
// Forward declaration for NotificationRulesError - actual import will be from its module
// For now, define a simple placeholder if the module doesn't exist.
// If it exists, use: use crate::notifications_rules::errors::NotificationRulesError;

// Placeholder for NotificationRulesError if the module is not yet created
// This allows NotificationError to compile.
// Once notifications_rules::errors is created, this should be removed and the proper import used.
#[cfg(not(feature = "actual_rules_error_type_exists"))] // Use a feature flag or conditional compilation
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Placeholder Rules Error: {0}")]
pub struct NotificationRulesError(String);


#[derive(Error, Debug)] // Removed Clone, PartialEq, Eq as CoreError and RulesError might not derive them
pub enum NotificationError {
    #[error("Notification with ID '{0}' not found.")]
    NotFound(Uuid),

    #[error("Invalid data for notification field '{field}': {reason}")]
    InvalidData { field: String, reason: String },

    #[error("Action '{action_key}' not found for notification ID '{notification_id}'.")]
    ActionNotFound {
        notification_id: Uuid,
        action_key: String,
    },

    #[error("Rule application error: {source}")]
    RuleApplicationError {
        #[cfg(feature = "actual_rules_error_type_exists")]
        #[source]
        source: crate::notifications_rules::errors::NotificationRulesError, // Actual path
        #[cfg(not(feature = "actual_rules_error_type_exists"))]
        #[source] // Keep as source even for placeholder for consistency
        source: NotificationRulesError, // Placeholder
    },

    #[error("Notification history persistence error during operation '{operation}': {message}")]
    HistoryPersistenceError {
        operation: String,
        message: String, // Added for more context
        #[source]
        source: novade_core::errors::CoreError, // Assuming CoreError is available and suitable
    },

    #[error("Internal notification error: {0}")]
    InternalError(String),
}

// Helper for HistoryPersistenceError if needed, though #[from] might be better if CoreError is simple
impl NotificationError {
    pub fn history_persistence_error_from_core(operation: String, message: String, core_error: novade_core::errors::CoreError) -> Self {
        NotificationError::HistoryPersistenceError {
            operation,
            message,
            source: core_error,
        }
    }
}
