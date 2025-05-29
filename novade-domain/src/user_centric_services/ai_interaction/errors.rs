use thiserror::Error;
use uuid::Uuid;
use novade_core::errors::CoreError;
// Assuming novade_core::config::ConfigError is a concrete error type.
// If it's an enum or needs specific handling, adjust as needed.
use novade_core::config::ConfigError as CoreConfigError; 

use super::types::AIDataCategory;

#[derive(Debug, Error)]
pub enum AIInteractionError {
    #[error("AI interaction context with ID '{0}' not found.")]
    ContextNotFound(Uuid),

    #[error("Consent already provided with ID '{consent_id}'.")]
    ConsentAlreadyProvided { consent_id: Uuid },

    #[error("Consent check failed for model '{model_id}' and category '{category:?}': {reason}")]
    ConsentCheckFailed {
        model_id: String,
        category: AIDataCategory,
        reason: String,
    },

    #[error("No suitable AI model available.")]
    NoModelAvailable,

    #[error("AI model with ID '{0}' not found.")]
    ModelNotFound(String),

    #[error("Invalid attachment: {0}")]
    InvalidAttachment(String),

    #[error("Consent storage error during operation '{operation}': {source_message}{}", .source.as_ref().map(|s| format!(": {}", s)).unwrap_or_default())]
    ConsentStorageError {
        operation: String,
        source_message: String,
        #[source]
        source: Option<CoreError>,
    },

    #[error("Failed to load AI model profiles: {source_message}{}", .source.as_ref().map(|s| format!(": {}", s)).unwrap_or_default())]
    ModelProfileLoadError {
        source_message: String,
        #[source]
        source: CoreError, // Changed from Option<CoreError> as load errors usually have a source
    },
    
    #[error("Failed to save AI model profiles: {source_message}{}", .source.as_ref().map(|s| format!(": {}", s)).unwrap_or_default())]
    ModelProfileSaveError {
        source_message: String,
        #[source]
        source: CoreError,
    },

    #[error("API key not found in secrets for name: '{secret_name}'.")]
    ApiKeyNotFoundInSecrets { secret_name: String },

    #[error("AI Model endpoint unreachable for model '{model_id}' at URL '{url}': {message}")]
    ModelEndpointUnreachable {
        model_id: String,
        url: String,
        message: String,
    },

    #[error("No default AI model is configured.")]
    NoDefaultModelConfigured,

    #[error("Core configuration error: {0}")]
    CoreConfigError(#[from] CoreConfigError),

    #[error("Internal error in AI interaction service: {0}")]
    InternalError(String),

    #[error("Base64 encoding/decoding error: {0}")]
    Base64EncodingError(String),
}

impl AIInteractionError {
    pub fn consent_storage_error_no_source(operation: impl Into<String>, message: impl Into<String>) -> Self {
        AIInteractionError::ConsentStorageError {
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

    #[test]
    fn test_error_messages_display() {
        let context_id = Uuid::new_v4();
        let consent_id = Uuid::new_v4();
        let model_id = "test-model".to_string();
        let category = AIDataCategory::GenericText;
        let reason = "User has not granted permission.".to_string();
        let core_error_io_src = Some(Arc::new(io::Error::new(io::ErrorKind::Other, "dummy underlying")));
        let core_error_io = CoreError::IoError("dummy io error".to_string(), core_error_io_src.clone());
        let core_config_error = CoreConfigError::new("dummy core config error".to_string());

        assert_eq!(format!("{}", AIInteractionError::ContextNotFound(context_id)), format!("AI interaction context with ID '{}' not found.", context_id));
        assert_eq!(format!("{}", AIInteractionError::ConsentAlreadyProvided { consent_id }), format!("Consent already provided with ID '{}'.", consent_id));
        assert_eq!(format!("{}", AIInteractionError::ConsentCheckFailed { model_id: model_id.clone(), category, reason: reason.clone() }), format!("Consent check failed for model 'test-model' and category 'GenericText': User has not granted permission."));
        assert_eq!(format!("{}", AIInteractionError::NoModelAvailable), "No suitable AI model available.");
        assert_eq!(format!("{}", AIInteractionError::ModelNotFound(model_id.clone())), format!("AI model with ID '{}' not found.", model_id));
        assert_eq!(format!("{}", AIInteractionError::InvalidAttachment("Unsupported format".to_string())), "Invalid attachment: Unsupported format");
        
        assert_eq!(
            format!("{}", AIInteractionError::ConsentStorageError { operation: "save".to_string(), source_message: "Disk full".to_string(), source: Some(core_error_io.clone()) }),
            "Consent storage error during operation 'save': Disk full: dummy io error: dummy underlying" // Adjusted for CoreError's Display
        );
         assert_eq!(
            format!("{}", AIInteractionError::ConsentStorageError { operation: "load".to_string(), source_message: "File missing".to_string(), source: None }),
            "Consent storage error during operation 'load': File missing"
        );

        assert_eq!(
            format!("{}", AIInteractionError::ModelProfileLoadError { source_message: "File corrupt".to_string(), source: core_error_io.clone() }),
            "Failed to load AI model profiles: File corrupt: dummy io error: dummy underlying" // Adjusted
        );
         assert_eq!(
            format!("{}", AIInteractionError::ModelProfileSaveError { source_message: "Cannot write".to_string(), source: core_error_io.clone() }),
            "Failed to save AI model profiles: Cannot write: dummy io error: dummy underlying" // Adjusted
        );
        assert_eq!(format!("{}", AIInteractionError::ApiKeyNotFoundInSecrets { secret_name: "SECRET_AI_KEY".to_string() }), "API key not found in secrets for name: 'SECRET_AI_KEY'.");
        assert_eq!(
            format!("{}", AIInteractionError::ModelEndpointUnreachable { model_id: model_id.clone(), url: "http://localhost/ai".to_string(), message: "Connection refused".to_string() }),
            "AI Model endpoint unreachable for model 'test-model' at URL 'http://localhost/ai': Connection refused"
        );
        assert_eq!(format!("{}", AIInteractionError::NoDefaultModelConfigured), "No default AI model is configured.");
        assert_eq!(format!("{}", AIInteractionError::CoreConfigError(core_config_error)), "Core configuration error: dummy core config error"); // Assuming CoreConfigError displays its inner message
        assert_eq!(format!("{}", AIInteractionError::InternalError("Unexpected state".to_string())), "Internal error in AI interaction service: Unexpected state");
        assert_eq!(format!("{}", AIInteractionError::Base64EncodingError("Invalid padding".to_string())), "Base64 encoding/decoding error: Invalid padding");
    }
}
