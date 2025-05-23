//! Error handling for the NovaDE core layer.
//!
//! This module provides error types and utilities for error handling
//! throughout the NovaDE desktop environment. It defines a set of
//! error types using the `thiserror` crate for ergonomic error
//! definition and handling.
//!
//! The main error type for this crate is [`CoreError`], which encapsulates
//! more specific errors like [`ConfigError`] and [`LoggingError`].
//!
//! # Examples
//!
//! ```rust,ignore
//! // Example of how a function might return a CoreError
//! use novade_core::error::CoreError;
//!
//! fn do_something_risky() -> Result<(), CoreError> {
//!     // ... some operation ...
//!     // If something goes wrong:
//!     // return Err(CoreError::Internal("Something went wrong".to_string()));
//!     Ok(())
//! }
//! ```

use std::io;
use std::path::PathBuf;
use std::vec::Vec; // Added for ConfigError::NotFound
use thiserror::Error;
use toml; // Added for ConfigError::ParseError

/// Core error type for the NovaDE desktop environment.
///
/// This enum represents all possible errors that can occur in the core layer.
/// It is designed to be used as a common error type throughout the application,
/// often by wrapping more specific error types.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Errors related to configuration loading, parsing, or validation.
    /// Wraps a [`ConfigError`].
    #[error("Configuration Error: {0}")]
    Config(#[from] ConfigError),

    /// Errors that occur during the initialization of the logging system.
    /// Contains a descriptive message of the failure.
    #[error("Logging Initialization Failed: {0}")]
    LoggingInitialization(String),

    /// Errors related to filesystem operations, such as creating directories or reading files,
    /// that are not covered by more specific configuration or logging I/O errors.
    /// Includes a message, the path involved, and the source I/O error.
    #[error("Filesystem Error: {message} (Path: {path:?})")]
    Filesystem {
        message: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// General I/O errors not covered by other specific variants.
    /// Wraps a `std::io::Error`.
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors due to invalid input provided to a function or method.
    /// Contains a descriptive message.
    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    /// Catch-all for unexpected internal errors within the core library.
    /// Contains a descriptive message.
    #[error("An unexpected internal error occurred: {0}")]
    Internal(String),
}

/// Error type for configuration-related operations.
///
/// This enum represents errors that can occur during configuration
/// loading, parsing, or access. It is typically wrapped by [`CoreError::Config`].
#[derive(Debug, Error)]
pub enum ConfigError {
    /// An error occurred while attempting to read a configuration file.
    /// Includes the path to the file and the source I/O error.
    #[error("Failed to read configuration file from {path:?}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// An error occurred while parsing a configuration file (e.g., invalid TOML).
    /// Wraps a `toml::de::Error`.
    #[error("Failed to parse configuration file: {0}")]
    ParseError(#[from] toml::de::Error),

    /// An error occurred due to invalid configuration values after successful parsing.
    /// Contains a descriptive message of the validation failure.
    #[error("Configuration validation failed: {0}")]
    ValidationError(String),

    /// A configuration file was not found at any of the expected locations.
    /// Contains a list of paths that were checked.
    #[error("Configuration file not found at expected locations: {locations:?}")]
    NotFound { locations: Vec<PathBuf> },

    /// A required base directory (e.g., XDG config/data home) could not be determined.
    /// Contains a string identifying the type of directory that was unavailable.
    #[error("Could not determine base directory for {dir_type}")]
    DirectoryUnavailable { dir_type: String },
}

/// Error type for logging-related operations.
///
/// This enum represents errors that can occur during logging
/// initialization or operation.
///
/// **Note:** This enum is distinct from [`CoreError::LoggingInitialization`].
/// `LoggingError` might be used for more specific logging operational failures
/// if the logging system itself provides them, while `CoreError::LoggingInitialization`
/// is specifically for failures during the setup process in `novade_core::logging`.
#[derive(Error, Debug)]
pub enum LoggingError {
    /// Failed to initialize the logging system.
    /// This variant is somewhat superseded by `CoreError::LoggingInitialization`
    /// but is kept for potential specific uses or if `novade_core::logging::initialize_logging`
    /// itself needs to return a more specific error before wrapping it.
    #[error("Failed to initialize logging: {0}")]
    InitializationError(String),

    /// Failed to set or parse a log filter (e.g., from a configuration string).
    #[error("Failed to set log filter: {0}")]
    FilterError(String),

    /// An I/O error occurred during logging, such as failing to write to a log file.
    /// Wraps a `std::io::Error`.
    #[error("Logging I/O error: {0}")]
    IoError(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};
    use std::error::Error; // To use the .source() method

    // --- CoreError Tests ---

    #[test]
    fn test_core_error_config_variant() {
        let original_config_err = ConfigError::ValidationError("Test validation".to_string());
        let core_err = CoreError::Config(original_config_err);
        
        assert_eq!(format!("{}", core_err), "Configuration Error: Configuration validation failed: Test validation");
        assert!(core_err.source().is_some());
        match core_err.source().unwrap().downcast_ref::<ConfigError>() {
            Some(ConfigError::ValidationError(msg)) => assert_eq!(msg, "Test validation"),
            _ => panic!("Incorrect source for CoreError::Config"),
        }
    }

    #[test]
    fn test_core_error_logging_initialization_variant() {
        let err_msg = "Failed to init logger".to_string();
        let core_err = CoreError::LoggingInitialization(err_msg.clone());
        
        assert_eq!(format!("{}", core_err), format!("Logging Initialization Failed: {}", err_msg));
        assert!(core_err.source().is_none());
    }

    #[test]
    fn test_core_error_filesystem_variant() {
        let path = PathBuf::from("/tmp/test.txt");
        let io_err_source = IoError::new(ErrorKind::PermissionDenied, "Permission denied for fs");
        let core_err = CoreError::Filesystem {
            message: "File operation failed".to_string(),
            path: path.clone(),
            source: io_err_source,
        };
        
        assert_eq!(format!("{}", core_err), format!("Filesystem Error: File operation failed (Path: {:?})", path));
        assert!(core_err.source().is_some());
        assert_eq!(core_err.source().unwrap().downcast_ref::<IoError>().unwrap().kind(), ErrorKind::PermissionDenied);
    }

    #[test]
    fn test_core_error_io_variant() {
        let io_err_source = IoError::new(ErrorKind::NotFound, "File not found for io");
        let core_err = CoreError::Io(io_err_source); // Uses #[from]

        assert_eq!(format!("{}", core_err), "I/O Error: File not found for io");
        assert!(core_err.source().is_some());
        assert_eq!(core_err.source().unwrap().downcast_ref::<IoError>().unwrap().kind(), ErrorKind::NotFound);
    }
    
    #[test]
    fn test_core_error_invalid_input_variant() {
        let err_msg = "Invalid input provided".to_string();
        let core_err = CoreError::InvalidInput(err_msg.clone());
        
        assert_eq!(format!("{}", core_err), format!("Invalid Input: {}", err_msg));
        assert!(core_err.source().is_none());
    }

    #[test]
    fn test_core_error_internal_variant() {
        let err_msg = "An internal error occurred".to_string();
        let core_err = CoreError::Internal(err_msg.clone());
        
        assert_eq!(format!("{}", core_err), format!("An unexpected internal error occurred: {}", err_msg));
        assert!(core_err.source().is_none());
    }

    // --- ConfigError Tests ---

    #[test]
    fn test_config_error_read_error_variant() {
        let path = PathBuf::from("/config/read_test.toml");
        let io_err_source = IoError::new(ErrorKind::NotFound, "Config file not found for read");
        let config_err = ConfigError::ReadError {
            path: path.clone(),
            source: io_err_source,
        };
        
        assert_eq!(format!("{}", config_err), format!("Failed to read configuration file from {:?}", path));
        assert!(config_err.source().is_some());
        assert_eq!(config_err.source().unwrap().downcast_ref::<IoError>().unwrap().kind(), ErrorKind::NotFound);
    }

    #[test]
    fn test_config_error_parse_error_variant() {
        // Create a dummy toml::de::Error (this is a bit tricky as its fields are private)
        // We'll parse an invalid TOML string to get a real one.
        let invalid_toml_content = "this is not valid toml";
        let toml_err_source: toml::de::Error = toml::from_str(invalid_toml_content).unwrap_err();
        let toml_err_display = format!("{}", toml_err_source); // Capture display before moving
        
        let config_err = ConfigError::ParseError(toml_err_source);
        
        assert_eq!(format!("{}", config_err), format!("Failed to parse configuration file: {}", toml_err_display));
        assert!(config_err.source().is_some());
        // Check if source is toml::de::Error by trying to downcast.
        // The actual error might be wrapped further by thiserror if `#[source]` was also used with `#[from]`.
        // Here, `#[from]` makes `toml::de::Error` the direct source.
        assert!(config_err.source().unwrap().is::<toml::de::Error>());
    }

    #[test]
    fn test_config_error_validation_error_variant() {
        let err_msg = "Validation failed".to_string();
        let config_err = ConfigError::ValidationError(err_msg.clone());
        
        assert_eq!(format!("{}", config_err), format!("Configuration validation failed: {}", err_msg));
        assert!(config_err.source().is_none());
    }

    #[test]
    fn test_config_error_not_found_variant() {
        let locations = vec![PathBuf::from("/path/1"), PathBuf::from("/path/2")];
        let config_err = ConfigError::NotFound { locations: locations.clone() };
        
        assert_eq!(format!("{}", config_err), format!("Configuration file not found at expected locations: {:?}", locations));
        assert!(config_err.source().is_none());
    }

    #[test]
    fn test_config_error_directory_unavailable_variant() {
        let dir_type = "XDG_CONFIG_HOME".to_string();
        let config_err = ConfigError::DirectoryUnavailable { dir_type: dir_type.clone() };
        
        assert_eq!(format!("{}", config_err), format!("Could not determine base directory for {}", dir_type));
        assert!(config_err.source().is_none());
    }

    // --- LoggingError Tests ---

    #[test]
    fn test_logging_error_initialization_error_variant() {
        let err_msg = "Failed to init subsystem".to_string();
        let log_err = LoggingError::InitializationError(err_msg.clone());
        
        assert_eq!(format!("{}", log_err), format!("Failed to initialize logging: {}", err_msg));
        assert!(log_err.source().is_none());
    }

    #[test]
    fn test_logging_error_filter_error_variant() {
        let err_msg = "Invalid filter string".to_string();
        let log_err = LoggingError::FilterError(err_msg.clone());
        
        assert_eq!(format!("{}", log_err), format!("Failed to set log filter: {}", err_msg));
        assert!(log_err.source().is_none());
    }

    #[test]
    fn test_logging_error_io_error_variant() {
        let io_err_source = IoError::new(ErrorKind::BrokenPipe, "Logging pipe broken");
        let log_err = LoggingError::IoError(io_err_source); // Uses #[from]

        assert_eq!(format!("{}", log_err), "Logging I/O error: Logging pipe broken");
        assert!(log_err.source().is_some());
        assert_eq!(log_err.source().unwrap().downcast_ref::<IoError>().unwrap().kind(), ErrorKind::BrokenPipe);
    }
}
