use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

pub use self::errors::{WorkspaceCoreError, MAX_WORKSPACE_NAME_LENGTH};
pub use self::types::{WindowIdentifier, WorkspaceId, WorkspaceLayoutType};
pub use self::event_data::{WorkspaceLayoutChangedData, WorkspaceRenamedData};

// Publicly declare submodules for external use if necessary, or keep them private to `core`
pub mod types;
pub mod errors;
pub mod event_data;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    id: WorkspaceId,
    name: String,
    layout_type: WorkspaceLayoutType,
    window_ids: HashSet<WindowIdentifier>,
    created_at: DateTime<Utc>,
}

impl Workspace {
    pub fn new(name: String) -> Result<Self, WorkspaceCoreError> {
        if name.is_empty() {
            return Err(WorkspaceCoreError::NameCannotBeEmpty);
        }
        if name.len() > MAX_WORKSPACE_NAME_LENGTH {
            return Err(WorkspaceCoreError::NameTooLong {
                name: name.clone(),
                max_len: MAX_WORKSPACE_NAME_LENGTH,
                actual_len: name.len(),
            });
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name,
            layout_type: WorkspaceLayoutType::default(),
            window_ids: HashSet::new(),
            created_at: Utc::now(),
        })
    }

    // Getters
    pub fn id(&self) -> WorkspaceId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn layout_type(&self) -> WorkspaceLayoutType {
        self.layout_type
    }

    pub fn window_ids(&self) -> &HashSet<WindowIdentifier> {
        &self.window_ids
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    // Methods
    pub fn rename(&mut self, new_name: String) -> Result<(), WorkspaceCoreError> {
        if new_name.is_empty() {
            return Err(WorkspaceCoreError::NameCannotBeEmpty);
        }
        if new_name.len() > MAX_WORKSPACE_NAME_LENGTH {
            return Err(WorkspaceCoreError::NameTooLong {
                name: new_name.clone(),
                max_len: MAX_WORKSPACE_NAME_LENGTH,
                actual_len: new_name.len(),
            });
        }
        self.name = new_name;
        Ok(())
    }

    pub fn set_layout_type(&mut self, layout_type: WorkspaceLayoutType) {
        self.layout_type = layout_type;
    }

    pub(crate) fn add_window_id(&mut self, window_id: WindowIdentifier) -> bool {
        self.window_ids.insert(window_id)
    }

    pub(crate) fn remove_window_id(&mut self, window_id: &WindowIdentifier) -> bool {
        self.window_ids.remove(window_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_new_valid_name() {
        let ws = Workspace::new("Test WS".to_string()).unwrap();
        assert_eq!(ws.name(), "Test WS");
        assert_eq!(ws.layout_type(), WorkspaceLayoutType::Floating);
        assert!(ws.window_ids().is_empty());
    }

    #[test]
    fn workspace_new_empty_name_error() {
        let result = Workspace::new("".to_string());
        assert!(matches!(result, Err(WorkspaceCoreError::NameCannotBeEmpty)));
    }

    #[test]
    fn workspace_new_name_too_long_error() {
        let long_name = "a".repeat(MAX_WORKSPACE_NAME_LENGTH + 1);
        let result = Workspace::new(long_name.clone());
        match result {
            Err(WorkspaceCoreError::NameTooLong { name, max_len, actual_len }) => {
                assert_eq!(name, long_name);
                assert_eq!(max_len, MAX_WORKSPACE_NAME_LENGTH);
                assert_eq!(actual_len, long_name.len());
            }
            _ => panic!("Expected NameTooLong error"),
        }
    }

    #[test]
    fn workspace_rename_valid() {
        let mut ws = Workspace::new("Old Name".to_string()).unwrap();
        ws.rename("New Name".to_string()).unwrap();
        assert_eq!(ws.name(), "New Name");
    }

    #[test]
    fn workspace_rename_empty_error() {
        let mut ws = Workspace::new("Valid Name".to_string()).unwrap();
        let result = ws.rename("".to_string());
        assert!(matches!(result, Err(WorkspaceCoreError::NameCannotBeEmpty)));
    }

    #[test]
    fn workspace_rename_too_long_error() {
        let mut ws = Workspace::new("Valid Name".to_string()).unwrap();
        let long_name = "b".repeat(MAX_WORKSPACE_NAME_LENGTH + 1);
        let result = ws.rename(long_name.clone());
        assert!(matches!(result, Err(WorkspaceCoreError::NameTooLong { .. })));
    }

    #[test]
    fn workspace_set_layout_type() {
        let mut ws = Workspace::new("Layout Test".to_string()).unwrap();
        ws.set_layout_type(WorkspaceLayoutType::TilingVertical);
        assert_eq!(ws.layout_type(), WorkspaceLayoutType::TilingVertical);
    }

    #[test]
    fn workspace_add_remove_window_id() {
        let mut ws = Workspace::new("Window Test".to_string()).unwrap();
        let win_id1 = WindowIdentifier::new("win1".to_string()).unwrap();
        let win_id2 = WindowIdentifier::new("win2".to_string()).unwrap();

        assert!(ws.add_window_id(win_id1.clone()));
        assert_eq!(ws.window_ids().len(), 1);
        assert!(ws.window_ids().contains(&win_id1));

        // Adding same ID again should return false
        assert!(!ws.add_window_id(win_id1.clone()));
        assert_eq!(ws.window_ids().len(), 1);

        assert!(ws.add_window_id(win_id2.clone()));
        assert_eq!(ws.window_ids().len(), 2);

        assert!(ws.remove_window_id(&win_id1));
        assert_eq!(ws.window_ids().len(), 1);
        assert!(!ws.window_ids().contains(&win_id1));

        // Removing non-existent ID should return false
        assert!(!ws.remove_window_id(&win_id1));
        assert_eq!(ws.window_ids().len(), 1);

        assert!(ws.remove_window_id(&win_id2));
        assert!(ws.window_ids().is_empty());
    }

    #[test]
    fn workspace_getters() {
        let name = "Getter Test".to_string();
        let ws = Workspace::new(name.clone()).unwrap();
        let id = ws.id(); // Capture for comparison
        let created_at = *ws.created_at(); // Capture for comparison

        assert_eq!(ws.id(), id);
        assert_eq!(ws.name(), name);
        assert_eq!(ws.layout_type(), WorkspaceLayoutType::Floating);
        assert!(ws.window_ids().is_empty());
        assert_eq!(*ws.created_at(), created_at);
    }
}
