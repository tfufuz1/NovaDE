//! Error handling for the NovaDE UI layer.
//!
//! This module provides error types and utilities for the NovaDE UI layer.

use thiserror::Error;
use std::path::PathBuf;

/// UI error type.
#[derive(Error, Debug)]
pub enum UiError {
    /// Asset loading error.
    #[error("Failed to load asset: {0}")]
    AssetLoadError(String),
    
    /// Widget creation error.
    #[error("Failed to create widget: {0}")]
    WidgetCreationError(String),
    
    /// Theme error.
    #[error("Theme error: {0}")]
    ThemeError(String),
    
    /// Layout error.
    #[error("Layout error: {0}")]
    LayoutError(String),
    
    /// Event handling error.
    #[error("Event handling error: {0}")]
    EventHandlingError(String),
    
    /// System integration error.
    #[error("System integration error: {0}")]
    SystemIntegrationError(String),
    
    /// File error.
    #[error("File error: {path:?} - {message}")]
    FileError {
        /// The file path.
        path: PathBuf,
        /// The error message.
        message: String,
    },
    
    /// Image processing error.
    #[error("Image processing error: {0}")]
    ImageProcessingError(String),
    
    /// Font error.
    #[error("Font error: {0}")]
    FontError(String),
    
    /// Window error.
    #[error("Window error: {0}")]
    WindowError(String),
    
    /// Rendering error.
    #[error("Rendering error: {0}")]
    RenderingError(String),
    
    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// JSON error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Other error.
    #[error("{0}")]
    Other(String),
}

/// Result type for UI operations.
pub type UiResult<T> = Result<T, UiError>;

/// Converts a string to a UI error.
///
/// # Arguments
///
/// * `message` - The error message
/// * `kind` - The error kind
///
/// # Returns
///
/// A UI error.
pub fn to_ui_error(message: impl Into<String>, kind: UiErrorKind) -> UiError {
    match kind {
        UiErrorKind::AssetLoad => UiError::AssetLoadError(message.into()),
        UiErrorKind::WidgetCreation => UiError::WidgetCreationError(message.into()),
        UiErrorKind::Theme => UiError::ThemeError(message.into()),
        UiErrorKind::Layout => UiError::LayoutError(message.into()),
        UiErrorKind::EventHandling => UiError::EventHandlingError(message.into()),
        UiErrorKind::SystemIntegration => UiError::SystemIntegrationError(message.into()),
        UiErrorKind::File { path } => UiError::FileError {
            path,
            message: message.into(),
        },
        UiErrorKind::ImageProcessing => UiError::ImageProcessingError(message.into()),
        UiErrorKind::Font => UiError::FontError(message.into()),
        UiErrorKind::Window => UiError::WindowError(message.into()),
        UiErrorKind::Rendering => UiError::RenderingError(message.into()),
        UiErrorKind::Other => UiError::Other(message.into()),
    }
}

/// UI error kind.
pub enum UiErrorKind {
    /// Asset loading error.
    AssetLoad,
    /// Widget creation error.
    WidgetCreation,
    /// Theme error.
    Theme,
    /// Layout error.
    Layout,
    /// Event handling error.
    EventHandling,
    /// System integration error.
    SystemIntegration,
    /// File error.
    File {
        /// The file path.
        path: PathBuf,
    },
    /// Image processing error.
    ImageProcessing,
    /// Font error.
    Font,
    /// Window error.
    Window,
    /// Rendering error.
    Rendering,
    /// Other error.
    Other,
}
