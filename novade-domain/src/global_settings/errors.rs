use thiserror::Error;
use novade_core::errors::CoreError; // Assuming this path

#[derive(Error, Debug)]
pub enum GlobalSettingsError {
    #[error("Setting path not found: {path_description}")]
    PathNotFound { path_description: String },

    #[error("Invalid value type for setting '{path_description}'. Expected type: {expected_type}, actual value preview: '{actual_value_preview}'")]
    InvalidValueType {
        path_description: String,
        expected_type: String,
        actual_value_preview: String,
    },

    #[error("Validation error for setting '{path_description}': {reason}")]
    ValidationError {
        path_description: String,
        reason: String,
    },

    #[error("Persistence error during operation '{operation}': {message}")]
    PersistenceError {
        operation: String,
        message: String,
        #[source]
        source: Option<CoreError>, // CoreError might need to be Box<dyn std::error::Error + Send + Sync + 'static> if CoreError itself is not directly suitable or to avoid tight coupling
    },

    #[error("Serialization error for setting '{path_description}': {source}")]
    SerializationError {
        path_description: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Deserialization error for setting '{path_description}': {source}")]
    DeserializationError {
        path_description: String,
        #[source]
        source: serde_json::Error,
    },
    
    #[error("TOML Deserialization error: {0}")]
    TomlDeserializationError(#[from] toml::de::Error),

    #[error("TOML Serialization error: {0}")]
    TomlSerializationError(#[from] toml::ser::Error),

    #[error("Underlying Core Error: {0}")]
    CoreError(#[from] CoreError), // For convenience if a CoreError needs to bubble up directly
}

// Helper to convert CoreError to a format suitable for PersistenceError source
// This might be more complex depending on CoreError's definition.
// For now, let's assume CoreError can be cloned or easily converted.
// If CoreError is an enum or complex, this would need adjustment.
// This is a simplified placeholder.
impl GlobalSettingsError {
    pub fn persistence_error_from_core(operation: String, message: String, core_error: CoreError) -> Self {
        // If CoreError needs specific handling to be a source, do it here.
        // For example, if it needs to be boxed or if only parts of it are relevant.
        GlobalSettingsError::PersistenceError {
            operation,
            message,
            source: Some(core_error),
        }
    }
}
