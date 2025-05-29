use thiserror::Error;
use novade_core::types::Color as CoreColor;
use crate::theming::types::{ThemeIdentifier, TokenIdentifier}; // Ensure this path is correct

#[derive(Error, Debug)]
pub enum ThemingError {
    #[error("Failed to parse token file '{filename}': {source_error}")]
    TokenFileParseError {
        filename: String,
        #[source]
        source_error: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    #[error("Invalid token value for token '{token_id}': {message}")]
    InvalidTokenValue {
        token_id: TokenIdentifier,
        message: String,
    },

    #[error("Cyclic token reference detected involving token '{token_id}'. Path: {path:?}")]
    CyclicTokenReference {
        token_id: TokenIdentifier,
        path: Vec<TokenIdentifier>,
    },

    #[error("Theme with ID '{theme_id}' not found")]
    ThemeNotFound {
        theme_id: ThemeIdentifier,
    },

    #[error("Failed to apply accent color {accent_color_name_disp} (value: {accent_color_value:?}) to token '{token_id}': {reason}")]
    AccentColorApplicationError {
        token_id: TokenIdentifier,
        accent_color_name_disp: String, // Display representation of accent color name
        accent_color_value: CoreColor, 
        reason: String,
    },

    #[error("Failed to resolve token '{token_id}': {reason}")]
    TokenResolutionError {
        token_id: TokenIdentifier,
        reason: String,
    },
    
    #[error("Configuration error in theming: {message}")]
    ConfigurationError {
        message: String,
    },

    #[error("I/O error related to theming: {message}")]
    IoError {
        message: String,
        #[source]
        source_error: Option<Box<dyn std::error::Error + Send + Sync + 'static>>, // Make source optional
    },

    #[error("Filesystem operation failed for theming: {source_error}")]
    FilesystemError {
        #[from]
        source_error: novade_core::errors::CoreError,
    },
    
    #[error("An unknown theming error occurred: {context}")]
    UnknownError {
        context: String,
    },
}

// Helper for AccentColorApplicationError to display name nicely
impl AccentColorApplicationErrorDisplay {
    pub fn new(name: Option<&str>) -> String {
        name.map_or_else(|| "<unnamed>".to_string(), |n| format!("'{}'", n))
    }
}

// This is a marker struct to namespace the helper, not strictly necessary
// but can be good practice if more helpers are added.
pub struct AccentColorApplicationErrorDisplay;


// Example of how to use ThemingError::AccentColorApplicationError:
// ThemingError::AccentColorApplicationError {
//     token_id: TokenIdentifier::new("some-token"),
//     accent_color_name_disp: AccentColorApplicationErrorDisplay::new(Some("Bright Red")),
//     accent_color_value: CoreColor::from_hex("#FF0000").unwrap(),
//     reason: "Color conversion failed".to_string(),
// }

// Example of how you might construct a TokenFileParseError:
// ThemingError::TokenFileParseError {
//     filename: "my_tokens.json".to_string(),
//     source_error: Box::new(serde_json::Error::custom("some json error")), // Example source
// }

// Example of how you might construct an IoError:
// ThemingError::IoError {
//     message: "Could not read theme directory".to_string(),
//     source_error: Some(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))),
// }
