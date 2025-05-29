use thiserror::Error;

pub const MAX_WORKSPACE_NAME_LENGTH: usize = 64;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WorkspaceCoreError {
    #[error("Invalid workspace name: {0}")]
    InvalidName(String),

    #[error("Workspace name cannot be empty.")]
    NameCannotBeEmpty,

    #[error("Workspace name '{name}' is too long. Max length: {max_len}, actual: {actual_len}.")]
    NameTooLong {
        name: String,
        max_len: usize,
        actual_len: usize,
    },

    #[error("Invalid persistent ID: '{0}'. Persistent ID must be alphanumeric, may contain hyphens or underscores, and cannot be empty.")]
    InvalidPersistentId(String),

    #[error("Window identifier cannot be empty.")]
    WindowIdentifierEmpty,
    
    // Example of a more specific format error if needed in future for WindowIdentifier
    // #[error("Window identifier '{id}' has an invalid format. Must match regex: {expected_format}")]
    // WindowIdentifierInvalidFormat { id: String, expected_format: String },

    #[error("Invalid accent color hex string: '{0}'. Must be in #RRGGBB or #RRGGBBAA format.")]
    InvalidAccentColorFormat(String),

    #[error("Internal error: {context}")]
    Internal { context: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        assert_eq!(
            format!("{}", WorkspaceCoreError::InvalidName("test!".to_string())),
            "Invalid workspace name: test!"
        );
        assert_eq!(
            format!("{}", WorkspaceCoreError::NameCannotBeEmpty),
            "Workspace name cannot be empty."
        );
        assert_eq!(
            format!("{}", WorkspaceCoreError::NameTooLong { name: "longname".to_string(), max_len: 5, actual_len: 8 }),
            "Workspace name 'longname' is too long. Max length: 5, actual: 8."
        );
        assert_eq!(
            format!("{}", WorkspaceCoreError::InvalidPersistentId("pid with space".to_string())),
            "Invalid persistent ID: 'pid with space'. Persistent ID must be alphanumeric, may contain hyphens or underscores, and cannot be empty."
        );
        assert_eq!(
            format!("{}", WorkspaceCoreError::WindowIdentifierEmpty),
            "Window identifier cannot be empty."
        );
        assert_eq!(
            format!("{}", WorkspaceCoreError::InvalidAccentColorFormat("#123".to_string())),
            "Invalid accent color hex string: '#123'. Must be in #RRGGBB or #RRGGBBAA format."
        );
        assert_eq!(
            format!("{}", WorkspaceCoreError::Internal { context: "Something went wrong".to_string() }),
            "Internal error: Something went wrong"
        );
    }
}
