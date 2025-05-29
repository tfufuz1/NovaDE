use crate::global_settings::paths::SettingPath;
use novade_core::errors::CoreError;
use serde_json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlobalSettingsError {
    #[error("Setting path not found: {path}")]
    PathNotFound { path: SettingPath },

    #[error("Invalid value type for path '{path}'. Expected type: {expected_type}, actual value preview: '{actual_value_preview}'")]
    InvalidValueType {
        path: SettingPath,
        expected_type: String,
        actual_value_preview: String,
    },

    #[error("Validation failed for path '{path}': {reason}")]
    ValidationError {
        path: SettingPath,
        reason: String,
    },

    #[error("Serialization error for path '{path}': {source}")]
    SerializationError {
        path: SettingPath,
        #[source]
        source: serde_json::Error,
    },

    #[error("Deserialization error for path '{path}': {source}")]
    DeserializationError {
        path: SettingPath,
        #[source]
        source: serde_json::Error,
    },

    #[error("Persistence error during operation '{operation}': {message}")]
    PersistenceError {
        operation: String,
        message: String,
        #[source]
        source: Option<CoreError>,
    },

    #[error("Core error: {0}")]
    CoreError(#[from] CoreError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl GlobalSettingsError {
    pub fn persistence_error_no_source(operation: impl Into<String>, message: impl Into<String>) -> Self {
        GlobalSettingsError::PersistenceError {
            operation: operation.into(),
            message: message.into(),
            source: None,
        }
    }
}

// Example helper function for creating a ValidationError more easily for a general struct validation
// This is not strictly part of the definition but can be a utility.
pub fn new_validation_error(path_segment: &str, reason: impl Into<String>) -> GlobalSettingsError {
    // This is a bit of a hack for the path, as we don't have a specific sub-path here.
    // Ideally, the validation logic would know the exact SettingPath.
    // For a general struct validation, we might use a placeholder or a root path.
    // For now, let's assume the caller can construct a meaningful SettingPath.
    // This helper might be better placed where SettingPath can be properly constructed.
    // Or, the ValidationError could take a String path for more flexibility if needed.
    GlobalSettingsError::ValidationError {
        // This path construction is symbolic, actual path construction may vary
        path: paths::SettingPath::from_str(path_segment).unwrap_or_else(|_| {
            // Fallback if path segment is not a valid top-level path, which is likely for general validation.
            // This indicates a need for a more flexible path in ValidationError or better path construction by caller.
            // For now, this is a placeholder.
            // A better approach might be to have ValidationError take path: String if it's general.
            // Or, the validate() methods should return errors specific to their sub-paths.
            // Let's assume the caller constructs the correct, specific SettingPath.
            // This helper is thus less useful without more context.
            // Example: SettingPath::Appearance(AppearanceSettingPath::ActiveThemeName)
            // This example is removed as it's not robust.
            panic!("Placeholder path used in new_validation_error, actual path required.")
        }),
        reason: reason.into(),
    }
}

// Re-import SettingPath locally if used in helpers like above.
// It's already imported at the top of the file.
use crate::global_settings::paths;
