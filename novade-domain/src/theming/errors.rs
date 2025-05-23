use thiserror::Error;
use crate::theming::types::{ThemeIdentifier, TokenIdentifier};
// Assuming CoreError is the standard error type from novade_core
// For ThemingError to be Clone, CoreError also needs to be Clone or wrapped as String.
// Let's assume CoreError can be stringified for now if it's not Clone.
// use novade_core::errors::CoreError; 

/// Represents errors that can occur within the theming engine.
#[derive(Error, Debug, Clone)] // Added Clone
pub enum ThemingError {
    #[error("Fehler beim Parsen der Token-Datei '{file_path}': {source_message}")]
    TokenFileParseError {
        file_path: String,
        source_message: String, // Changed from serde_json::Error to String
    },

    #[error("Fehler beim Parsen der Theme-Definitionsdatei '{file_path}': {source_message}")]
    ThemeFileParseError {
        file_path: String,
        source_message: String, // Changed from serde_json::Error to String
    },

    #[error("Zyklische Referenz beim Auflösen des Tokens '{token_id}' entdeckt. Auflösungspfad: {path:?}")]
    CyclicTokenReference {
        token_id: TokenIdentifier,
        path: Vec<TokenIdentifier>,
    },

    #[error("Maximale Auflösungstiefe ({depth}) für Token '{token_id}' überschritten.")]
    MaxResolutionDepthExceeded {
        token_id: TokenIdentifier,
        depth: u8,
    },

    #[error("Theme mit der ID '{theme_id}' wurde nicht gefunden.")]
    ThemeNotFound { theme_id: ThemeIdentifier },

    #[error("Token mit der ID '{token_id}' wurde im aktuellen Kontext nicht gefunden.")]
    TokenNotFound { token_id: TokenIdentifier },

    #[error("Ungültiger Wert für Token '{token_id}': {message}")]
    InvalidTokenValue {
        token_id: TokenIdentifier,
        message: String,
    },

    #[error("Fehler beim Anwenden der Akzentfarbe auf Token '{token_id}': {message}")]
    AccentColorApplicationError {
        token_id: TokenIdentifier,
        message: String,
    },
    
    #[error("Ungültiger Bezeichner: '{identifier}'. {message}")]
    InvalidIdentifierFormat {
        identifier: String,
        message: String,
    },

    // If novade_core::errors::CoreError is not Clone, we must store its string representation.
    // For std::io::Error, its string representation can be stored.
    #[error("E/A-Fehler im Dateisystem: {0}")]
    FilesystemIoError(String), // Changed from #[from] std::io::Error

    // Assuming CoreError can be converted to a String.
    #[error("Core-Fehler: {0}")]
    CoreError(String), // Changed from #[from] CoreError

    #[error("Allgemeiner Konfigurationsfehler: {0}")]
    ConfigurationError(String),

    #[error("Interner Fehler der Theming-Engine: {0}")]
    InternalError(String),

    #[error("Serialisierungs- oder Deserialisierungsfehler: {0}")]
    SerdeError(String), // Generic serde error if not from a specific file
}

// Removed: impl From<serde_json::Error> for ThemingError 
// It's better to handle the conversion at the call site to provide file_path context.

// Helper for creating InvalidTokenValue errors easily
impl ThemingError {
    pub fn invalid_value(token_id: TokenIdentifier, message: impl Into<String>) -> Self {
        ThemingError::InvalidTokenValue {
            token_id,
            message: message.into(),
        }
    }
}
