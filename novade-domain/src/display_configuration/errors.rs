// novade-domain/src/display_configuration/errors.rs
use thiserror::Error;
use novade_core::errors::CoreError; // Assuming a CoreError type exists

#[derive(Error, Debug)]
pub enum DisplayConfigurationError {
    #[error("Persistence error: {0}")]
    Persistence(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Conflict error: {0}")]
    Conflict(String),
    #[error("Underlying system error: {0}")]
    SystemError(String), // To wrap errors from novade-system
    #[error("Core error: {0}")]
    CoreError(#[from] CoreError),
    #[error("Display with ID {0} not found")]
    DisplayNotFound(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization/Deserialization error: {0}")]
    SerdeError(String), // For serde_json, serde_yaml, etc.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, DisplayConfigurationError>;
