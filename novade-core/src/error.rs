//! Error handling for the NovaDE core layer.
//!
//! This module provides error types and utilities for error handling
//! throughout the NovaDE desktop environment. It defines a set of
//! error types using the `thiserror` crate for ergonomic error
//! definition and handling.
//!
//! The main error type for this crate is [`CoreError`], which encapsulates
//! more specific errors like [`ConfigError`].
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

use std::path::PathBuf;
use thiserror::Error;
use toml; // Required for ConfigError::ParseError

/// Error type for color parsing, specifically for hex string conversion.
///
/// This error is used by `crate::types::color::Color::from_hex`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ColorParseError {
    /// The hex string has an invalid length.
    #[error("Invalid hex string length: expected 3, 4, 6, or 8 characters after '#', found {0}")]
    InvalidLength(usize),
    /// The hex string contains invalid characters.
    #[error("Invalid hex character: '{0}'")]
    InvalidHexCharacter(char),
    /// The hex string is missing the leading '#' prefix.
    #[error("Hex string must start with '#'")]
    MissingPrefix,
    /// An error occurred during hex decoding.
    #[error("Failed to decode hex component: {0}")]
    HexDecodingError(String),
}

/// The primary error type for the core infrastructure layer.
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

    /// Errors related to parsing color strings.
    /// Wraps a [`ColorParseError`].
    #[error("Color Parsing Error: {0}")]
    ColorParse(#[from] ColorParseError),

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

/// Specific errors related to configuration handling.
///
/// This enum represents errors that can occur during configuration
/// loading, parsing, or access. It is typically wrapped by [`CoreError::Config`].
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to read the configuration file.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};
    use std::error::Error; // To use the .source() method

    // --- ColorParseError Tests ---
    #[test]
    fn test_color_parse_error_display() {
        assert_eq!(
            format!("{}", ColorParseError::InvalidLength(5)),
            "Invalid hex string length: expected 3, 4, 6, or 8 characters after '#', found 5"
        );
        assert_eq!(
            format!("{}", ColorParseError::InvalidHexCharacter('X')),
            "Invalid hex character: 'X'"
        );
        assert_eq!(
            format!("{}", ColorParseError::MissingPrefix),
            "Hex string must start with '#'"
        );
        assert_eq!(
            format!("{}", ColorParseError::HexDecodingError("bad hex".to_string())),
            "Failed to decode hex component: bad hex"
        );
    }

    // --- CoreError Tests ---

    #[test]
    fn test_core_error_config_from_config_error() {
        let original_config_err = ConfigError::ValidationError("Test validation".to_string());
        let core_err: CoreError = original_config_err.into(); // Test #[from]

        assert_eq!(format!("{}", core_err), "Configuration Error: Configuration validation failed: Test validation");
        assert!(core_err.source().is_some());
        match core_err.source().unwrap().downcast_ref::<ConfigError>() {
            Some(ConfigError::ValidationError(msg)) => assert_eq!(msg, "Test validation"),
            _ => panic!("Incorrect source for CoreError::Config after conversion"),
        }
    }

    #[test]
    fn test_core_error_color_parse_from_color_parse_error() {
        let original_color_err = ColorParseError::InvalidLength(5);
        let core_err: CoreError = original_color_err.clone().into(); // Test #[from]

        assert_eq!(format!("{}", core_err), "Color Parsing Error: Invalid hex string length: expected 3, 4, 6, or 8 characters after '#', found 5");
        assert!(core_err.source().is_some());
        match core_err.source().unwrap().downcast_ref::<ColorParseError>() {
            Some(err) => assert_eq!(err, &original_color_err),
            _ => panic!("Incorrect source for CoreError::ColorParse after conversion"),
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
    fn test_core_error_io_variant_and_from_io_error() {
        let io_err_source = IoError::new(ErrorKind::NotFound, "File not found for io");
        // Test #[from] by converting IoError to CoreError
        let core_err: CoreError = io_err_source.into(); 

        assert_eq!(format!("{}", core_err), "I/O Error: File not found for io");
        assert!(core_err.source().is_some());
        // The direct source of CoreError::Io is the IoError itself
        match core_err.source().unwrap().downcast_ref::<IoError>() {
             Some(src_io_err) => assert_eq!(src_io_err.kind(), ErrorKind::NotFound),
             None => panic!("Source is not an IoError"),
        }

        // Also test direct instantiation if needed, though `#[from]` is the primary way
        let direct_io_err_source = IoError::new(ErrorKind::Interrupted, "Operation interrupted");
        let direct_core_err = CoreError::Io(direct_io_err_source);
        assert_eq!(format!("{}", direct_core_err), "I/O Error: Operation interrupted");
        assert!(direct_core_err.source().is_some());
         match direct_core_err.source().unwrap().downcast_ref::<IoError>() {
             Some(src_io_err) => assert_eq!(src_io_err.kind(), ErrorKind::Interrupted),
             None => panic!("Source is not an IoError for direct instantiation"),
        }
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
        let invalid_toml_content = "this is not valid toml";
        let toml_err_source: toml::de::Error = toml::from_str::<toml::Value>(invalid_toml_content).unwrap_err();
        let toml_err_display = format!("{}", toml_err_source); 
        
        // Test #[from] by converting toml::de::Error to ConfigError
        let config_err: ConfigError = toml_err_source.into(); 
        
        assert_eq!(format!("{}", config_err), format!("Failed to parse configuration file: {}", toml_err_display));
        assert!(config_err.source().is_some());
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

     #[test]
    fn test_core_error_from_config_error_conversion() {
        let config_err = ConfigError::ValidationError("Test validation from ConfigError".to_string());
        let core_error: CoreError = config_err.into(); // Explicit conversion

        match core_error {
            CoreError::Config(ref inner_err) => {
                if let ConfigError::ValidationError(msg) = inner_err {
                    assert_eq!(msg, "Test validation from ConfigError");
                } else {
                    panic!("Inner error is not ConfigError::ValidationError");
                }
            }
            _ => panic!("CoreError is not CoreError::Config variant"),
        }
        assert_eq!(
            format!("{}", core_error),
            "Configuration Error: Configuration validation failed: Test validation from ConfigError"
        );
    }

    #[test]
    fn test_core_error_from_io_error_conversion() {
        let io_err = IoError::new(ErrorKind::PermissionDenied, "Permission denied for direct conversion");
        let core_error: CoreError = io_err.into(); // Explicit conversion

        match core_error {
            CoreError::Io(ref inner_err) => {
                assert_eq!(inner_err.kind(), ErrorKind::PermissionDenied);
                assert_eq!(format!("{}", inner_err), "Permission denied for direct conversion");
            }
            _ => panic!("CoreError is not CoreError::Io variant"),
        }
        assert_eq!(
            format!("{}", core_error),
            "I/O Error: Permission denied for direct conversion"
        );
    }
}
