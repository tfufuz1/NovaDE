use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use super::errors::WorkspaceCoreError;

pub type WorkspaceId = Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct WindowIdentifier(String);

impl WindowIdentifier {
    pub fn new(id: String) -> Result<Self, WorkspaceCoreError> {
        if id.is_empty() {
            Err(WorkspaceCoreError::WindowIdentifierEmpty)
        } else {
            Ok(Self(id))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WindowIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for WindowIdentifier {
    fn from(s: &str) -> Self {
        // This From implementation assumes the input string is valid (non-empty).
        // If it could be empty, it should also return a Result or panic.
        // For simplicity in line with typical From impls, we'll assume valid input here.
        // The `new` constructor is the primary way to create one with validation.
        debug_assert!(!s.is_empty(), "WindowIdentifier created from empty string via From<&str>");
        Self(s.to_string())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WorkspaceLayoutType {
    #[default]
    Floating,
    TilingHorizontal,
    TilingVertical,
    Maximized,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_identifier_new_valid() {
        let id = WindowIdentifier::new("test_id".to_string()).unwrap();
        assert_eq!(id.as_str(), "test_id");
    }

    #[test]
    fn window_identifier_new_empty_error() {
        let result = WindowIdentifier::new("".to_string());
        assert!(matches!(result, Err(WorkspaceCoreError::WindowIdentifierEmpty)));
    }

    #[test]
    fn window_identifier_display() {
        let id = WindowIdentifier::new("display_id".to_string()).unwrap();
        assert_eq!(format!("{}", id), "display_id");
    }

    #[test]
    fn window_identifier_from_str() {
        let id_str = "from_str_id";
        let id = WindowIdentifier::from(id_str);
        assert_eq!(id.as_str(), id_str);
    }

    #[test]
    fn workspace_layout_type_default() {
        assert_eq!(WorkspaceLayoutType::default(), WorkspaceLayoutType::Floating);
    }
}
