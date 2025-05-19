//! Error handling for the NovaDE core layer.
//!
//! This module provides error types and utilities for error handling
//! throughout the NovaDE desktop environment. It defines a set of
//! error types using the `thiserror` crate for ergonomic error
//! definition and handling.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Core error type for the NovaDE desktop environment.
///
/// This enum represents all possible errors that can occur in the core layer.
/// It is designed to be used as a common error type throughout the application.
#[derive(Error, Debug)]
pub enum CoreError {
    /// An error occurred during I/O operations.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// An error occurred during configuration loading or parsing.
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// An error occurred during logging initialization or operation.
    #[error("Logging error: {0}")]
    Logging(#[from] LoggingError),

    /// A generic error with a custom message.
    #[error("{0}")]
    Generic(String),

    /// An error occurred with additional context.
    #[error("{context}: {source}")]
    WithContext {
        /// The error context description
        context: String,
        /// The source error
        source: Box<CoreError>,
    },
}

impl CoreError {
    /// Create a new generic error with the given message.
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        CoreError::Generic(msg.into())
    }

    /// Add context to an error.
    pub fn with_context<S: Into<String>>(self, context: S) -> Self {
        CoreError::WithContext {
            context: context.into(),
            source: Box::new(self),
        }
    }
}

/// Error type for configuration-related operations.
///
/// This enum represents errors that can occur during configuration
/// loading, parsing, or access.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// The configuration file was not found.
    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),

    /// The configuration file could not be read.
    #[error("Failed to read configuration file: {0}")]
    FileReadError(#[source] io::Error),

    /// The configuration file contains invalid TOML.
    #[error("Failed to parse TOML: {0}")]
    ParseError(#[from] toml::de::Error),

    /// A required configuration value is missing.
    #[error("Missing required configuration value: {0}")]
    MissingValue(String),

    /// A configuration value has an invalid type.
    #[error("Invalid configuration value type for {key}: expected {expected}")]
    InvalidValueType {
        /// The configuration key
        key: String,
        /// The expected type description
        expected: String,
    },

    /// A configuration value is out of the allowed range.
    #[error("Configuration value out of range for {key}: {message}")]
    ValueOutOfRange {
        /// The configuration key
        key: String,
        /// A message describing the valid range
        message: String,
    },
}

/// Error type for logging-related operations.
///
/// This enum represents errors that can occur during logging
/// initialization or operation.
#[derive(Error, Debug)]
pub enum LoggingError {
    /// Failed to initialize the logging system.
    #[error("Failed to initialize logging: {0}")]
    InitializationError(String),

    /// Failed to set the log filter.
    #[error("Failed to set log filter: {0}")]
    FilterError(String),

    /// An I/O error occurred during logging.
    #[error("Logging I/O error: {0}")]
    IoError(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_core_error_from_io_error() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let core_err = CoreError::from(io_err);
        
        match core_err {
            CoreError::Io(_) => assert!(true),
            _ => panic!("Expected CoreError::Io"),
        }
    }

    #[test]
    fn test_core_error_from_config_error() {
        let config_err = ConfigError::MissingValue("test_value".to_string());
        let core_err = CoreError::from(config_err);
        
        match core_err {
            CoreError::Config(_) => assert!(true),
            _ => panic!("Expected CoreError::Config"),
        }
    }

    #[test]
    fn test_core_error_from_logging_error() {
        let logging_err = LoggingError::InitializationError("test error".to_string());
        let core_err = CoreError::from(logging_err);
        
        match core_err {
            CoreError::Logging(_) => assert!(true),
            _ => panic!("Expected CoreError::Logging"),
        }
    }

    #[test]
    fn test_core_error_generic() {
        let err = CoreError::generic("test error");
        
        match err {
            CoreError::Generic(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected CoreError::Generic"),
        }
    }

    #[test]
    fn test_core_error_with_context() {
        let io_err = IoError::new(ErrorKind::PermissionDenied, "permission denied");
        let core_err = CoreError::from(io_err).with_context("while opening config file");
        
        match core_err {
            CoreError::WithContext { context, source: _ } => {
                assert_eq!(context, "while opening config file");
            },
            _ => panic!("Expected CoreError::WithContext"),
        }
    }

    #[test]
    fn test_error_display() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let core_err = CoreError::from(io_err);
        
        assert!(format!("{}", core_err).contains("I/O error"));
        
        let config_err = ConfigError::MissingValue("test_value".to_string());
        assert!(format!("{}", config_err).contains("Missing required configuration value"));
        
        let logging_err = LoggingError::InitializationError("test error".to_string());
        assert!(format!("{}", logging_err).contains("Failed to initialize logging"));
    }
}
