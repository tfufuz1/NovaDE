use thiserror::Error;
use crate::workspaces::core::{WorkspaceId, WindowIdentifier}; // Corrected path as per previous steps

#[derive(Debug, Error)]
pub enum WindowPolicyError {
    #[error("Layout calculation error for workspace '{workspace_id}': {reason}")]
    LayoutCalculationError {
        workspace_id: WorkspaceId,
        reason: String,
    },

    #[error("Invalid policy configuration for setting '{setting_path}': {reason}")]
    InvalidPolicyConfiguration {
        setting_path: String,
        reason: String,
    },

    #[error("Window '{0}' not found for policy application.")]
    WindowNotFoundForPolicy(WindowIdentifier),

    #[error("Internal error in window management policy: {0}")]
    InternalError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid; 

    #[test]
    fn test_error_messages() {
        let ws_id = Uuid::new_v4();
        let win_id = WindowIdentifier::from("win-test-1");

        assert_eq!(
            format!("{}", WindowPolicyError::LayoutCalculationError { workspace_id: ws_id, reason: "Not enough space".to_string() }),
            format!("Layout calculation error for workspace '{}': Not enough space", ws_id)
        );
        assert_eq!(
            format!("{}", WindowPolicyError::InvalidPolicyConfiguration { setting_path: "gaps.inner".to_string(), reason: "Value too high".to_string() }),
            "Invalid policy configuration for setting 'gaps.inner': Value too high"
        );
        assert_eq!(
            format!("{}", WindowPolicyError::WindowNotFoundForPolicy(win_id.clone())),
            format!("Window '{}' not found for policy application.", win_id)
        );
        assert_eq!(
            format!("{}", WindowPolicyError::InternalError("Unexpected state".to_string())),
            "Internal error in window management policy: Unexpected state"
        );
    }
}
