use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::errors::WorkspaceCoreError; // This will be created in the next step.

// --- WorkspaceId ---
pub type WorkspaceId = Uuid;

// --- WindowIdentifier ---
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct WindowIdentifier(String);

impl WindowIdentifier {
    pub fn new(id: impl Into<String>) -> Result<Self, WorkspaceCoreError> {
        let id_str = id.into();
        if id_str.is_empty() {
            Err(WorkspaceCoreError::WindowIdentifierEmpty)
        } else {
            // Regex for basic validation: allow alphanumeric, dash, underscore, dot.
            // This is an example; specific requirements might differ.
            // For now, keeping it simple as per initial instructions (just non-empty).
            // let re = regex::Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap();
            // if !re.is_match(&id_str) {
            //     return Err(WorkspaceCoreError::WindowIdentifierInvalidFormat { id: id_str });
            // }
            Ok(Self(id_str))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for WindowIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for WindowIdentifier {
    fn from(s: &str) -> Self {
        // Per instructions, using expect.
        WindowIdentifier::new(s).expect("Invalid WindowIdentifier from &str: ID cannot be empty and must be valid.")
    }
}

impl TryFrom<String> for WindowIdentifier {
    type Error = WorkspaceCoreError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        WindowIdentifier::new(s)
    }
}


// --- WorkspaceLayoutType ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
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
    use static_assertions::assert_impl_all;

    // Ensure types implement expected traits
    assert_impl_all!(WindowIdentifier: Send, Sync);
    assert_impl_all!(WorkspaceLayoutType: Send, Sync);


    #[test]
    fn window_identifier_new_valid() {
        let id = WindowIdentifier::new("test-id-123.valid").unwrap();
        assert_eq!(id.as_str(), "test-id-123.valid");
    }

    #[test]
    fn window_identifier_new_empty_err() {
        let result = WindowIdentifier::new("");
        assert!(matches!(result, Err(WorkspaceCoreError::WindowIdentifierEmpty)));
    }

    #[test]
    fn window_identifier_display() {
        let id = WindowIdentifier::new("display-me").unwrap();
        assert_eq!(format!("{}", id), "display-me");
    }

    #[test]
    fn window_identifier_from_str_valid() {
        let id = WindowIdentifier::from("from-str-valid");
        assert_eq!(id.as_str(), "from-str-valid");
    }

    #[test]
    #[should_panic(expected = "Invalid WindowIdentifier from &str: ID cannot be empty and must be valid.")]
    fn window_identifier_from_str_empty_panics() {
        WindowIdentifier::from("");
    }
    
    #[test]
    fn window_identifier_try_from_string_valid() {
        let id_str = "try-from-string-valid".to_string();
        let id = WindowIdentifier::try_from(id_str).unwrap();
        assert_eq!(id.as_str(), "try-from-string-valid");
    }

    #[test]
    fn window_identifier_try_from_string_empty_err() {
        let id_str = "".to_string();
        let result = WindowIdentifier::try_from(id_str);
        assert!(matches!(result, Err(WorkspaceCoreError::WindowIdentifierEmpty)));
    }


    #[test]
    fn window_identifier_serde() {
        let id = WindowIdentifier::new("serde-id").unwrap();
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "\"serde-id\"");
        let deserialized: WindowIdentifier = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, id);
    }

    #[test]
    fn workspace_layout_type_default() {
        assert_eq!(WorkspaceLayoutType::default(), WorkspaceLayoutType::Floating);
    }

    #[test]
    fn workspace_layout_type_serde() {
        let layout = WorkspaceLayoutType::TilingHorizontal;
        let serialized = serde_json::to_string(&layout).unwrap();
        assert_eq!(serialized, "\"tiling-horizontal\"");
        let deserialized: WorkspaceLayoutType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, layout);

        let default_layout = WorkspaceLayoutType::default();
        let serialized_default = serde_json::to_string(&default_layout).unwrap();
        assert_eq!(serialized_default, "\"floating\"");
        let deserialized_default: WorkspaceLayoutType = serde_json::from_str(&serialized_default).unwrap();
        assert_eq!(deserialized_default, default_layout);
    }
}
