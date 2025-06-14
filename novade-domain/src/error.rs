//! Error module for the NovaDE domain layer.
//!
//! This module defines domain-specific errors for the NovaDE desktop environment.

use thiserror::Error;
// The import `use crate::core::error::CoreError;` was present in Turn 23's read.
// This path is problematic if `core` is not a module at `crate::core`.
// `novade_core::CoreError` is the correct path if `novade_core` is a dependency.
// Assuming novade_core is correctly listed in Cargo.toml and CoreError is pub use'd there.
use novade_core::CoreError; // Corrected path, assuming novade_core is a dependency and CoreError is public.

/// A general Result type for domain operations.
pub type DomainResult<T> = Result<T, DomainError>;

/// The primary error type for the domain layer.
#[derive(Debug, Error)]
pub enum DomainError {
    /// Core error.
    #[error(transparent)]
    Core(#[from] CoreError),
    
    /// Workspace error.
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    
    /// Theming error.
    #[error(transparent)]
    Theming(#[from] ThemingError),
    
    /// AI error.
    #[error(transparent)]
    AI(#[from] AIError),
    
    /// Notification error.
    #[error(transparent)]
    Notification(#[from] NotificationError),
    
    /// Window management error.
    #[error(transparent)]
    WindowManagement(#[from] WindowManagementError),
    
    /// Power management error.
    #[error(transparent)]
    PowerManagement(#[from] PowerManagementError),
    
    /// Other error.
    #[error("Domain error: {0}")]
    Other(String),

    // Errors for AI Interaction Service
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
    #[error("NLP processing failed: {0}")]
    NlpError(String),
    #[error("Skill execution failed: {0}")]
    SkillError(String),
    #[error("Configuration error related to AI/Assistant: {0}")]
    AssistantConfigError(String),
}

/// Workspace error type.
#[derive(Debug, Error)]
pub enum WorkspaceError {
    /// Workspace not found.
    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),
    
    /// Window not in workspace.
    #[error("Window {window_id} not in workspace {workspace_id}")]
    WindowNotInWorkspace {
        /// Workspace ID
        workspace_id: String,
        /// Window ID
        window_id: String,
    },
    
    /// Window not in any workspace.
    #[error("Window not in any workspace: {0}")]
    WindowNotInAnyWorkspace(String),
    
    /// Window reference not found.
    #[error("Window reference not found: {0}")]
    WindowReferenceNotFound(String),
    
    /// No active workspace.
    #[error("No active workspace")]
    NoActiveWorkspace,
    
    /// Cannot delete active workspace.
    #[error("Cannot delete active workspace")]
    CannotDeleteActiveWorkspace,
    
    /// Other error.
    #[error("Workspace error: {0}")]
    Other(String),
}

/// Theming error type.
#[derive(Debug, Error)]
pub enum ThemingError {
    /// Theme not found.
    #[error("Theme not found: {0}")]
    ThemeNotFound(String),
    
    /// Component not found.
    #[error("Component not found: {0}")]
    ComponentNotFound(String),
    
    /// Cannot delete active theme.
    #[error("Cannot delete active theme")]
    CannotDeleteActiveTheme,
    
    /// File not found.
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    /// File read error.
    #[error("File read error: {0}")]
    FileReadError(String),
    
    /// File write error.
    #[error("File write error: {0}")]
    FileWriteError(String),
    
    /// Invalid theme format.
    #[error("Invalid theme format: {0}")]
    InvalidThemeFormat(String),
    
    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Other error.
    #[error("Theming error: {0}")]
    Other(String),
}

/// AI error type.
#[derive(Debug, Error)]
pub enum AIError {
    /// Feature not found.
    #[error("Feature not found: {0}")]
    FeatureNotFound(String),
    
    /// Consent not found.
    #[error("Consent not found for user {user_id} and feature {feature_id}")]
    ConsentNotFound {
        /// User ID
        user_id: String,
        /// Feature ID
        feature_id: String,
    },
    
    /// Request not found.
    #[error("Request not found: {0}")]
    RequestNotFound(String),
    
    /// Response not found.
    #[error("Response not found: {0}")]
    ResponseNotFound(String),
    
    /// Consent required.
    #[error("Consent required for feature {0}")]
    ConsentRequired(String),
    
    /// Other error.
    #[error("AI error: {0}")]
    Other(String),
}

/// Notification error type.
#[derive(Debug, Error)]
pub enum NotificationError {
    /// Notification not found.
    #[error("Notification not found: {0}")]
    NotificationNotFound(String),
    
    /// Action not found.
    #[error("Action {action_id} not found for notification {notification_id}")]
    ActionNotFound {
        /// Notification ID
        notification_id: String,
        /// Action ID
        action_id: String,
    },
    
    /// Not dismissible.
    #[error("Notification is not dismissible: {0}")]
    NotDismissible(String),
    
    /// Other error.
    #[error("Notification error: {0}")]
    Other(String),
}

/// Window management error type.
#[derive(Debug, Error)]
pub enum WindowManagementError {
    /// Policy not found.
    #[error("Policy not found: {0}")]
    PolicyNotFound(String),
    
    /// No policy found.
    #[error("No policy found for window type: {0}")]
    NoPolicyFound(String),
    
    /// Window not found.
    #[error("Window not found: {0}")]
    WindowNotFound(String),
    
    /// Action not allowed.
    #[error("Action {action:?} not allowed for window {window_id}")]
    ActionNotAllowed {
        /// Window ID
        window_id: String,
        /// Action
        action: crate::window_management::WindowAction,
    },
    
    /// Other error.
    #[error("Window management error: {0}")]
    Other(String),
}

/// Power management error type.
#[derive(Debug, Error)]
pub enum PowerManagementError {
    /// Device not found (e.g., a specific UPower device).
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// D-Bus communication error.
    #[error("D-Bus communication error: {0}")]
    DbusCommunicationError(String),
    
    /// Operation not supported.
    #[error("Operation not supported: {0}")]
    OperationNotSupported(String),
    
    /// Sleep inhibited.
    #[error("Sleep inhibited: {0}")]
    SleepInhibited(String),
    
    /// Inhibitor not found.
    #[error("Inhibitor not found: {0}")]
    InhibitorNotFound(String),

    // Renamed BatteryNotFound to DeviceNotFound, so this specific variant is removed
    // If BatteryNotFound specifically is needed elsewhere, it can be a new variant or use Other.
    // For now, DeviceNotFound covers the UPower device case.

    /// Other error.
    #[error("Power management error: {0}")]
    Other(String),
}
