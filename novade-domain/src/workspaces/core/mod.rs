use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use regex::Regex; // For validation

pub use self::errors::{WorkspaceCoreError, MAX_WORKSPACE_NAME_LENGTH};
pub use self::types::{WindowIdentifier, WorkspaceId, WorkspaceLayoutType};
pub use self::event_data::*; // Re-export all event data structs

// Publicly declare submodules for external use if necessary, or keep them private to `core`
pub mod types;
pub mod errors;
pub mod event_data;


lazy_static::lazy_static! {
    static ref PERSISTENT_ID_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9-_]+$").unwrap();
    static ref ACCENT_COLOR_HEX_REGEX: Regex = Regex::new(r"^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{8})$").unwrap();
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    id: WorkspaceId,
    name: String,
    layout_type: WorkspaceLayoutType,
    window_ids: HashSet<WindowIdentifier>,
    created_at: DateTime<Utc>,
    persistent_id: Option<String>,
    icon_name: Option<String>,
    accent_color_hex: Option<String>,
}

impl Workspace {
    pub fn new(
        name: String,
        persistent_id: Option<String>,
        icon_name: Option<String>,
        accent_color_hex: Option<String>,
    ) -> Result<Self, WorkspaceCoreError> {
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

        if let Some(pid) = &persistent_id {
            if pid.is_empty() {
                // Allowing empty Option<String> is fine, but if Some(String), it should not be empty string.
                // Or decide if Some("") should be treated as None. For now, an empty string is invalid if Some.
                return Err(WorkspaceCoreError::InvalidPersistentId("Persistent ID cannot be an empty string if provided.".to_string()));
            }
            if !PERSISTENT_ID_REGEX.is_match(pid) {
                return Err(WorkspaceCoreError::InvalidPersistentId(format!(
                    "Persistent ID '{}' contains invalid characters. Use only a-z, A-Z, 0-9, -, _.",
                    pid
                )));
            }
        }

        if let Some(color) = &accent_color_hex {
            if !ACCENT_COLOR_HEX_REGEX.is_match(color) {
                return Err(WorkspaceCoreError::InvalidAccentColorFormat(color.clone()));
            }
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name,
            layout_type: WorkspaceLayoutType::default(),
            window_ids: HashSet::new(),
            created_at: Utc::now(),
            persistent_id,
            icon_name,
            accent_color_hex,
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

    pub fn persistent_id(&self) -> Option<&str> {
        self.persistent_id.as_deref()
    }

    pub fn icon_name(&self) -> Option<&str> {
        self.icon_name.as_deref()
    }

    pub fn accent_color_hex(&self) -> Option<&str> {
        self.accent_color_hex.as_deref()
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

    pub(crate) fn clear_window_ids(&mut self) {
        self.window_ids.clear();
    }

    pub(crate) fn replace_window_ids(&mut self, new_window_ids: HashSet<WindowIdentifier>) {
        self.window_ids = new_window_ids;
    }


    pub fn set_persistent_id(&mut self, pid: Option<String>) -> Result<(), WorkspaceCoreError> {
        if let Some(p) = &pid {
            if p.is_empty() {
                 return Err(WorkspaceCoreError::InvalidPersistentId("Persistent ID cannot be an empty string if provided.".to_string()));
            }
            if !PERSISTENT_ID_REGEX.is_match(p) {
                return Err(WorkspaceCoreError::InvalidPersistentId(format!(
                    "Persistent ID '{}' contains invalid characters. Use only a-z, A-Z, 0-9, -, _.",
                    p
                )));
            }
        }
        self.persistent_id = pid;
        Ok(())
    }

    pub fn set_icon_name(&mut self, icon: Option<String>) {
        // Basic validation: disallow empty string if Some, treat as None.
        self.icon_name = icon.filter(|s| !s.is_empty());
    }

    pub fn set_accent_color_hex(&mut self, color_hex: Option<String>) -> Result<(), WorkspaceCoreError> {
        if let Some(color) = &color_hex {
            if !ACCENT_COLOR_HEX_REGEX.is_match(color) {
                return Err(WorkspaceCoreError::InvalidAccentColorFormat(color.clone()));
            }
        }
        self.accent_color_hex = color_hex;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_workspace(name: &str) -> Workspace {
        Workspace::new(name.to_string(), None, None, None).unwrap()
    }

    #[test]
    fn workspace_new_with_valid_optional_fields() {
        let ws = Workspace::new(
            "Test WS".to_string(),
            Some("valid-pid_123".to_string()),
            Some("icon-name".to_string()),
            Some("#AABBCC".to_string()),
        ).unwrap();
        assert_eq!(ws.name(), "Test WS");
        assert_eq!(ws.persistent_id(), Some("valid-pid_123"));
        assert_eq!(ws.icon_name(), Some("icon-name"));
        assert_eq!(ws.accent_color_hex(), Some("#AABBCC"));
    }

    #[test]
    fn workspace_new_with_empty_optional_fields_as_none() {
        let ws = Workspace::new(
            "Test WS".to_string(),
            None, // Explicit None
            Some("".to_string()), // Empty string for icon name should be treated as None by setter later
            None, // Explicit None
        ).unwrap();
        // Note: Workspace::new doesn't filter empty icon_name to None, set_icon_name does.
        // This test is for `new`. If `icon_name` was Some(""), it would remain Some("").
        // Let's adjust the expectation or the constructor.
        // For now, constructor passes it through. set_icon_name filters.
        // To test the filtering logic, we'd call set_icon_name.
        // Let's test constructor behavior accurately:
        assert_eq!(ws.icon_name(), Some("")); // Constructor passes empty string if Some("")

        let mut ws_for_setter = create_valid_workspace("SetterTest");
        ws_for_setter.set_icon_name(Some("".to_string()));
        assert_eq!(ws_for_setter.icon_name(), None); // set_icon_name filters empty string to None

         let ws_none = Workspace::new(
            "Test WS None".to_string(),
            None,
            None,
            None,
        ).unwrap();
        assert_eq!(ws_none.persistent_id(), None);
        assert_eq!(ws_none.icon_name(), None);
        assert_eq!(ws_none.accent_color_hex(), None);
    }


    #[test]
    fn workspace_new_invalid_persistent_id_chars() {
        let result = Workspace::new("Test".to_string(), Some("invalid pid!".to_string()), None, None);
        assert!(matches!(result, Err(WorkspaceCoreError::InvalidPersistentId(_))));
    }
    
    #[test]
    fn workspace_new_empty_string_persistent_id() {
        let result = Workspace::new("Test".to_string(), Some("".to_string()), None, None);
        assert!(matches!(result, Err(WorkspaceCoreError::InvalidPersistentId(msg)) if msg.contains("empty string")));
    }

    #[test]
    fn workspace_new_invalid_accent_color_format() {
        let result_short = Workspace::new("Test".to_string(), None, None, Some("#123".to_string()));
        assert!(matches!(result_short, Err(WorkspaceCoreError::InvalidAccentColorFormat(_))));

        let result_long = Workspace::new("Test".to_string(), None, None, Some("#123456789".to_string()));
        assert!(matches!(result_long, Err(WorkspaceCoreError::InvalidAccentColorFormat(_))));
        
        let result_no_hash = Workspace::new("Test".to_string(), None, None, Some("AABBCC".to_string()));
        assert!(matches!(result_no_hash, Err(WorkspaceCoreError::InvalidAccentColorFormat(_))));

        let result_invalid_char = Workspace::new("Test".to_string(), None, None, Some("#AABBXX".to_string()));
        assert!(matches!(result_invalid_char, Err(WorkspaceCoreError::InvalidAccentColorFormat(_))));
    }
    
    #[test]
    fn workspace_new_valid_accent_color_formats() {
        assert!(Workspace::new("Test".to_string(), None, None, Some("#AABBCC".to_string())).is_ok());
        assert!(Workspace::new("Test".to_string(), None, None, Some("#aabbcc".to_string())).is_ok());
        assert!(Workspace::new("Test".to_string(), None, None, Some("#123456".to_string())).is_ok());
        assert!(Workspace::new("Test".to_string(), None, None, Some("#AABBCCDD".to_string())).is_ok()); // 8-digit hex
    }


    #[test]
    fn workspace_set_persistent_id_valid_and_invalid() {
        let mut ws = create_valid_workspace("PID Test");
        assert!(ws.set_persistent_id(Some("valid-id".to_string())).is_ok());
        assert_eq!(ws.persistent_id(), Some("valid-id"));

        assert!(ws.set_persistent_id(None).is_ok());
        assert_eq!(ws.persistent_id(), None);

        let result_invalid = ws.set_persistent_id(Some("invalid id!".to_string()));
        assert!(matches!(result_invalid, Err(WorkspaceCoreError::InvalidPersistentId(_))));
        assert_eq!(ws.persistent_id(), None); // Should not have changed

        let result_empty = ws.set_persistent_id(Some("".to_string()));
        assert!(matches!(result_empty, Err(WorkspaceCoreError::InvalidPersistentId(msg)) if msg.contains("empty string")));
    }

    #[test]
    fn workspace_set_icon_name() {
        let mut ws = create_valid_workspace("Icon Test");
        ws.set_icon_name(Some("my-icon".to_string()));
        assert_eq!(ws.icon_name(), Some("my-icon"));

        ws.set_icon_name(None);
        assert_eq!(ws.icon_name(), None);

        // Test that empty string becomes None
        ws.set_icon_name(Some("".to_string()));
        assert_eq!(ws.icon_name(), None);
    }

    #[test]
    fn workspace_set_accent_color_hex_valid_and_invalid() {
        let mut ws = create_valid_workspace("Color Test");
        assert!(ws.set_accent_color_hex(Some("#FF00FF".to_string())).is_ok());
        assert_eq!(ws.accent_color_hex(), Some("#FF00FF"));

        assert!(ws.set_accent_color_hex(None).is_ok());
        assert_eq!(ws.accent_color_hex(), None);

        let result_invalid = ws.set_accent_color_hex(Some("invalidcolor".to_string()));
        assert!(matches!(result_invalid, Err(WorkspaceCoreError::InvalidAccentColorFormat(_))));
        assert_eq!(ws.accent_color_hex(), None); // Should not have changed
    }
    
    #[test]
    fn workspace_clear_and_replace_window_ids() {
        let mut ws = create_valid_workspace("Window Clear/Replace Test");
        let win1 = WindowIdentifier::new("win1".to_string()).unwrap();
        let win2 = WindowIdentifier::new("win2".to_string()).unwrap();
        let win3 = WindowIdentifier::new("win3".to_string()).unwrap();

        ws.add_window_id(win1.clone());
        ws.add_window_id(win2.clone());
        assert_eq!(ws.window_ids().len(), 2);

        ws.clear_window_ids();
        assert!(ws.window_ids().is_empty());
        
        let mut new_set = HashSet::new();
        new_set.insert(win2.clone());
        new_set.insert(win3.clone());
        
        ws.replace_window_ids(new_set);
        assert_eq!(ws.window_ids().len(), 2);
        assert!(ws.window_ids().contains(&win2));
        assert!(ws.window_ids().contains(&win3));
        assert!(!ws.window_ids().contains(&win1));
    }
}
