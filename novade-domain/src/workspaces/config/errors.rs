use thiserror::Error;
use novade_core::errors::CoreError; // Assuming this path

#[derive(Error, Debug)]
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

    #[error("Failed to serialize workspace configuration: {message}")]
    SerializationError {
        message: String,
        #[source]
        source: Option<toml::ser::Error>, // Option because sometimes it might be a custom logic error
    },

    #[error("Failed to deserialize workspace configuration: {message}")]
    DeserializationError {
        message: String,
        snippet: Option<String>, // For context if parsing fails
        #[source]
        source: Option<toml::de::Error>, // Option because sometimes it might be a custom logic error
    },

    #[error("Invalid workspace configuration data: {reason}")]
    InvalidData {
        reason: String,
        path: Option<String>, // Optional path to specific field if known
    },
}
