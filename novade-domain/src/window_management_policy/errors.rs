use thiserror::Error;
use crate::workspaces::core::types::WorkspaceId; // Assuming this is the correct path
use crate::global_settings::GlobalSettingsError; // Corrected path

#[derive(Error, Debug)] // Removed Clone, PartialEq, Eq as GlobalSettingsError is not guaranteed to derive them
pub enum WindowPolicyError {
    #[error("Layout calculation error for workspace '{workspace_id}': {reason}")]
    LayoutCalculationError {
        workspace_id: WorkspaceId,
        reason: String,
    },

    #[error("Invalid window management policy configuration for setting path '{setting_path}': {reason}")]
    InvalidPolicyConfiguration {
        setting_path: String,
        reason: String,
    },

    #[error("Error accessing global settings: {0}")]
    SettingsAccessError(#[from] GlobalSettingsError),

    #[error("Internal window management policy error: {0}")]
    InternalError(String),
}
