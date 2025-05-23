use crate::global_settings_management::paths::SettingPath;
use novade_core::errors::CoreError; // Assuming this can be cloned or stringified
use serde_json; // For serde_json::Error
use thiserror::Error;

#[derive(Error, Debug, Clone)] // Added Clone, assumes CoreError and serde_json::Error are handled
pub enum GlobalSettingsError {
    #[error("Einstellungspfad nicht gefunden: {path}")]
    PathNotFound { path: SettingPath },

    #[error("Ungültiger Wertetyp für Pfad {path}: Erwartet '{expected_type}', erhalten (Vorschau): '{actual_value_preview}'")]
    InvalidValueType {
        path: SettingPath,
        expected_type: String,
        actual_value_preview: String,
    },

    #[error("Validierungsfehler für Pfad {path}: {reason}")]
    ValidationError { path: SettingPath, reason: String },
    
    #[error("Validierungsfehler in globalen Einstellungen: {reason}")]
    GlobalValidationFailed { reason: String }, // For GlobalDesktopSettings::validate_recursive()

    #[error("Serialisierungsfehler (Pfad: {path_description:?}): {source_message}")]
    SerializationError {
        path_description: Option<String>, // More general than SettingPath for global issues
        source_message: String, 
    },

    #[error("Deserialisierungsfehler (Pfad: {path_description:?}): {source_message}")]
    DeserializationError {
        path_description: Option<String>, // Path might not be known or relevant for top-level deserialize
        source_message: String, 
    },
    
    // FieldDeserializationError is specific and can keep SettingPath
    #[error("Deserialisierungsfehler für Feld {path}: {source_message}")]
    FieldDeserializationError { 
        path: SettingPath,
        source_message: String,
    },

    #[error("Persistenzfehler während Operation '{operation}': {message}")]
    PersistenceError {
        operation: String,
        message: String,
        // Store Option<String> if CoreError is not Clone or if source is optional
        source: Option<String>, // Already simplified to Option<String> for clonability
    },

    #[error("Core-Fehler: {0}")]
    CoreError(String), // Store as String if CoreError is not Clone

    #[error("Interner Fehler: {0}")]
    InternalError(String),
}

// Helper to convert CoreError to Option<String> for PersistenceError
impl GlobalSettingsError {
    pub fn persistence_error_with_core_source(
        operation: impl Into<String>,
        message: impl Into<String>,
        core_error: CoreError,
    ) -> Self {
        GlobalSettingsError::PersistenceError {
            operation: operation.into(),
            message: message.into(),
            source: Some(core_error.to_string()),
        }
    }
    
    pub fn persistence_error_without_source(
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        GlobalSettingsError::PersistenceError {
            operation: operation.into(),
            message: message.into(),
            source: None,
        }
    }

    // Helper for converting serde_json::Error for SerializationError
    pub fn serialization_error(path_description: Option<String>, source: serde_json::Error) -> Self {
        GlobalSettingsError::SerializationError {
            path_description,
            source_message: source.to_string(),
        }
    }

    // Helper for converting serde_json::Error for DeserializationError (top-level or specific field)
    pub fn deserialization_error(path_description: Option<String>, source: serde_json::Error) -> Self {
        GlobalSettingsError::DeserializationError {
            path_description,
            source_message: source.to_string(),
        }
    }
    
    // Helper for converting serde_json::Error for FieldDeserializationError (field-level)
    // This remains specific to SettingPath for field-level updates.
    pub fn field_deserialization_error(path: SettingPath, source: serde_json::Error) -> Self {
        GlobalSettingsError::FieldDeserializationError {
            path,
            source_message: source.to_string(),
        }
    }
}

// If direct From<CoreError> is needed and CoreError is not Clone:
impl From<CoreError> for GlobalSettingsError {
    fn from(core_err: CoreError) -> Self {
        GlobalSettingsError::CoreError(core_err.to_string())
    }
}
