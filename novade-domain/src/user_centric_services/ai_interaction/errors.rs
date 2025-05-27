use thiserror::Error;
use novade_core::errors::CoreError; // Assuming this path
use super::types::AIDataCategory; // Assuming this path

#[derive(Error, Debug)]
pub enum AIInteractionError {
    #[error("AI model '{0}' not found.")]
    ModelNotFound(String),

    #[error("Consent check failed for model '{model_id}' regarding data category '{category:?}': {reason}")]
    ConsentCheckFailed {
        model_id: String,
        category: AIDataCategory,
        reason: String,
    },

    #[error("Persistence error during operation '{operation}': {message}")]
    PersistenceError {
        operation: String,
        message: String, // Added for more context
        #[source]
        source: Option<CoreError>, // Keeping Option for cases where source might not be CoreError directly
    },

    #[error("Internal AI interaction error: {0}")]
    InternalError(String),
}

// Helper to create PersistenceError from CoreError if needed
impl AIInteractionError {
    pub fn persistence_error_from_core(operation: String, message: String, core_error: CoreError) -> Self {
        AIInteractionError::PersistenceError {
            operation,
            message,
            source: Some(core_error),
        }
    }
}
