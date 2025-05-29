use thiserror::Error;
use novade_core::errors::CoreError;

#[derive(Debug, Error)]
pub enum WorkspaceConfigError {
    #[error("Failed to load workspace configuration from '{path}': {source}")]
    LoadError {
        path: String,
        #[source]
        source: CoreError,
    },

    #[error("Failed to save workspace configuration to '{path}': {source}")]
    SaveError {
        path: String,
        #[source]
        source: CoreError,
    },

    #[error("Invalid data in workspace configuration: {reason}{}", path.as_ref().map(|p| format!(" (path: {})", p)).unwrap_or_default())]
    InvalidData {
        reason: String,
        path: Option<String>,
    },

    #[error("Failed to serialize workspace configuration: {message}")]
    SerializationError {
        message: String,
        #[source]
        source: Option<toml::ser::Error>,
    },

    #[error("Failed to deserialize workspace configuration: {message}{}", snippet.as_ref().map(|s| format!(" (snippet: {:.50})", s)).unwrap_or_default())]
    DeserializationError {
        message: String,
        snippet: Option<String>,
        #[source]
        source: Option<toml::de::Error>,
    },

    #[error("Persistent ID '{persistent_id}' not found in the loaded set of workspaces.")]
    PersistentIdNotFoundInLoadedSet { persistent_id: String },

    #[error("Duplicate persistent ID '{persistent_id}' found in the loaded set of workspaces.")]
    DuplicatePersistentIdInLoadedSet { persistent_id: String },

    #[error("Version mismatch for workspace configuration. Expected: {expected:?}, Found: {found:?}")]
    VersionMismatch {
        expected: Option<String>, // Using Option<String> for flexibility
        found: Option<String>,
    },

    #[error("Internal error in workspace configuration: {context}")]
    Internal { context: String },
}

impl WorkspaceConfigError {
    pub fn invalid_data(reason: impl Into<String>, path: Option<String>) -> Self {
        WorkspaceConfigError::InvalidData { reason: reason.into(), path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc; // Required for Arc::new in CoreError::IoError

    #[test]
    fn test_error_messages() {
        let dummy_core_error_io = CoreError::IoError("dummy io error".to_string(), Some(Arc::new(io::Error::new(io::ErrorKind::Other, "dummy"))));
        let dummy_core_error_config = CoreError::ConfigError("dummy config error".to_string());

        assert_eq!(
            format!("{}", WorkspaceConfigError::LoadError { path: "path/to/load".to_string(), source: dummy_core_error_io.clone() }),
            "Failed to load workspace configuration from 'path/to/load': dummy io error"
        );
        assert_eq!(
            format!("{}", WorkspaceConfigError::SaveError { path: "path/to/save".to_string(), source: dummy_core_error_config.clone() }),
            "Failed to save workspace configuration to 'path/to/save': dummy config error"
        );
        assert_eq!(
            format!("{}", WorkspaceConfigError::InvalidData { reason: "Bad field".to_string(), path: Some("field_name".to_string()) }),
            "Invalid data in workspace configuration: Bad field (path: field_name)"
        );
        assert_eq!(
            format!("{}", WorkspaceConfigError::InvalidData { reason: "Bad file".to_string(), path: None }),
            "Invalid data in workspace configuration: Bad file"
        );
        
        let dummy_ser_error_val = "key = #".parse::<toml::Value>().unwrap_err().to_string(); // Simulate a toml::ser::Error by creating a toml::de::Error that is similar
        let ser_error_instance = toml::to_string("").map_err(|e| e).unwrap_err(); // Get an actual toml::ser::Error
        let ser_error_display = ser_error_instance.to_string();

        assert_eq!(
            format!("{}", WorkspaceConfigError::SerializationError { message: "TOML ser failed".to_string(), source: Some(ser_error_instance) }),
            format!("Failed to serialize workspace configuration: TOML ser failed: {}", ser_error_display)
        );

        let de_error_instance = "bad value".parse::<toml::Value>().unwrap_err(); // Get an actual toml::de::Error
        let de_error_display = de_error_instance.to_string();
        assert_eq!(
            format!("{}", WorkspaceConfigError::DeserializationError { message: "TOML de failed".to_string(), snippet: Some("bad data".to_string()), source: Some(de_error_instance) }),
            format!("Failed to deserialize workspace configuration: TOML de failed (snippet: bad data): {}", de_error_display)
        );
        
        assert_eq!(
            format!("{}", WorkspaceConfigError::PersistentIdNotFoundInLoadedSet { persistent_id: "pid1".to_string() }),
            "Persistent ID 'pid1' not found in the loaded set of workspaces."
        );
        assert_eq!(
            format!("{}", WorkspaceConfigError::DuplicatePersistentIdInLoadedSet { persistent_id: "pid2".to_string() }),
            "Duplicate persistent ID 'pid2' found in the loaded set of workspaces."
        );
        assert_eq!(
            format!("{}", WorkspaceConfigError::VersionMismatch { expected: Some("1.0".to_string()), found: Some("0.9".to_string()) }),
            "Version mismatch for workspace configuration. Expected: Some(\"1.0\"), Found: Some(\"0.9\")"
        );
        assert_eq!(
            format!("{}", WorkspaceConfigError::Internal { context: "Oops".to_string() }),
            "Internal error in workspace configuration: Oops"
        );
    }
}
