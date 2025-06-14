// novade-system/src/system_health_collectors/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("OS Interaction Error: Failed to read system resource '{resource}'. Cause: {io_error}")]
    OsResourceError {
        resource: String,
        #[source]
        io_error: std::io::Error,
    },

    #[error("OS Interaction Error: Failed to execute command '{command}'. Exit code: {exit_code:?}. Stderr: {stderr}")]
    CommandExecutionError {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },

    #[error("Data parsing error for '{data_source}': {message}")]
    DataParsingError {
        data_source: String,
        message: String,
        #[source]
        source_error: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Feature not implemented or not available on this system: {feature_name}")]
    NotImplemented(String),

    #[error("Permission denied accessing resource: {resource}")]
    PermissionDenied(String),

    #[error("Invalid parameter for collector: {param_name} - {description}")]
    InvalidParameter { param_name: String, description: String },

    #[error("An unexpected collection error occurred: {0}")]
    Unexpected(String),
}
