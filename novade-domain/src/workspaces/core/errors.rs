use thiserror::Error;

pub const MAX_WORKSPACE_NAME_LENGTH: usize = 64;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceCoreError {
    #[error("Workspace name cannot be empty.")]
    NameCannotBeEmpty,

    #[error("Workspace name '{name}' is too long. Max length: {max_len}, actual: {actual_len}.")]
    NameTooLong {
        name: String,
        max_len: usize,
        actual_len: usize,
    },

    #[error("Invalid persistent ID format: {0}. Expected a valid UUID string.")]
    InvalidPersistentId(String),

    #[error("Window identifier cannot be empty.")]
    WindowIdentifierEmpty,

    #[error("Invalid accent color format: {0}. Expected a hex color string (e.g., #RRGGBB).")]
    InvalidAccentColorFormat(String),
}
