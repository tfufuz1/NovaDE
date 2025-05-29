use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use regex::Regex; // For validation

use super::types::{WorkspaceId, WindowIdentifier, WorkspaceLayoutType};
use super::errors::{WorkspaceCoreError, MAX_WORKSPACE_NAME_LENGTH};

lazy_static::lazy_static! {
    // Basic alphanumeric, hyphen, underscore. No leading/trailing hyphens/underscores.
    static ref PERSISTENT_ID_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9_-]*[a-zA-Z0-9])?$").unwrap();
    // Hex color: #RRGGBB or #RRGGBBAA
    static ref HEX_COLOR_REGEX: Regex = Regex::new(r"^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{8})$").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)] // Derived PartialEq for now
pub struct Workspace {
    id: WorkspaceId,
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    persistent_id: Option<String>,
    layout_type: WorkspaceLayoutType,
    window_ids: HashSet<WindowIdentifier>,
    created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    accent_color_hex: Option<String>,
}

impl Workspace {
    pub fn new(
        name: String,
        persistent_id: Option<String>,
        icon_name: Option<String>,
        accent_color_hex: Option<String>,
    ) -> Result<Self, WorkspaceCoreError> {
        // Validate name
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

        // Validate persistent_id
        if let Some(pid) = &persistent_id {
            if pid.is_empty() || !PERSISTENT_ID_REGEX.is_match(pid) {
                return Err(WorkspaceCoreError::InvalidPersistentId(pid.clone()));
            }
        }

        // Validate accent_color_hex
        if let Some(hex) = &accent_color_hex {
            if !HEX_COLOR_REGEX.is_match(hex) {
                return Err(WorkspaceCoreError::InvalidAccentColorFormat(hex.clone()));
            }
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name,
            persistent_id,
            layout_type: WorkspaceLayoutType::default(),
            window_ids: HashSet::new(),
            created_at: Utc::now(),
            icon_name,
            accent_color_hex,
        })
    }

    // Getters
    pub fn id(&self) -> WorkspaceId { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn persistent_id(&self) -> Option<&str> { self.persistent_id.as_deref() }
    pub fn layout_type(&self) -> WorkspaceLayoutType { self.layout_type }
    pub fn window_ids(&self) -> &HashSet<WindowIdentifier> { &self.window_ids }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn icon_name(&self) -> Option<&str> { self.icon_name.as_deref() }
    pub fn accent_color_hex(&self) -> Option<&str> { self.accent_color_hex.as_deref() }

    // Setters & Methods
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

    pub fn set_persistent_id(&mut self, pid: Option<String>) -> Result<(), WorkspaceCoreError> {
        if let Some(p) = &pid {
            if p.is_empty() || !PERSISTENT_ID_REGEX.is_match(p) {
                return Err(WorkspaceCoreError::InvalidPersistentId(p.clone()));
            }
        }
        self.persistent_id = pid;
        Ok(())
    }

    pub fn set_icon_name(&mut self, icon: Option<String>) {
        // Basic validation: allow empty string in Option, or non-empty string.
        // More complex validation (e.g. icon naming conventions) could be added.
        if let Some(i_name) = &icon {
            if i_name.is_empty() {
                self.icon_name = None; // Treat empty string as None
                return;
            }
        }
        self.icon_name = icon;
    }

    pub fn set_accent_color_hex(&mut self, color_hex: Option<String>) -> Result<(), WorkspaceCoreError> {
        if let Some(hex) = &color_hex {
            if !HEX_COLOR_REGEX.is_match(hex) {
                return Err(WorkspaceCoreError::InvalidAccentColorFormat(hex.clone()));
            }
        }
        self.accent_color_hex = color_hex;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::thread; // For testing Send/Sync

    #[test]
    fn workspace_new_valid() {
        let ws = Workspace::new("Test WS".to_string(), Some("test-ws-pid".to_string()), None, Some("#123456".to_string())).unwrap();
        assert_eq!(ws.name(), "Test WS");
        assert_eq!(ws.persistent_id(), Some("test-ws-pid"));
        assert_eq!(ws.accent_color_hex(), Some("#123456"));
        assert_eq!(ws.layout_type(), WorkspaceLayoutType::default());
        assert!(ws.window_ids().is_empty());
    }

    #[test]
    fn workspace_new_name_empty() {
        let result = Workspace::new("".to_string(), None, None, None);
        assert!(matches!(result, Err(WorkspaceCoreError::NameCannotBeEmpty)));
    }

    #[test]
    fn workspace_new_name_too_long() {
        let long_name = "a".repeat(MAX_WORKSPACE_NAME_LENGTH + 1);
        let result = Workspace::new(long_name.clone(), None, None, None);
        assert!(matches!(result, Err(WorkspaceCoreError::NameTooLong { name, .. }) if name == long_name));
    }

    #[test]
    fn workspace_new_persistent_id_invalid() {
        let result = Workspace::new("Valid Name".to_string(), Some("invalid pid!".to_string()), None, None);
        assert!(matches!(result, Err(WorkspaceCoreError::InvalidPersistentId(pid)) if pid == "invalid pid!"));
        
        let result_empty_pid = Workspace::new("Valid Name".to_string(), Some("".to_string()), None, None);
        assert!(matches!(result_empty_pid, Err(WorkspaceCoreError::InvalidPersistentId(pid)) if pid.is_empty()));
    }
    
    #[test]
    fn workspace_new_persistent_id_valid_edge_cases() {
        assert!(Workspace::new("Valid Name".to_string(), Some("a".to_string()), None, None).is_ok());
        assert!(Workspace::new("Valid Name".to_string(), Some("a-b".to_string()), None, None).is_ok());
        assert!(Workspace::new("Valid Name".to_string(), Some("a_b".to_string()), None, None).is_ok());
        assert!(Workspace::new("Valid Name".to_string(), Some("a1".to_string()), None, None).is_ok());
        // Invalid: leading/trailing hyphen/underscore
        assert!(matches!(Workspace::new("Valid Name".to_string(), Some("-abc".to_string()), None, None), Err(WorkspaceCoreError::InvalidPersistentId(_))));
        assert!(matches!(Workspace::new("Valid Name".to_string(), Some("abc-".to_string()), None, None), Err(WorkspaceCoreError::InvalidPersistentId(_))));
    }


    #[test]
    fn workspace_new_accent_color_invalid() {
        let result = Workspace::new("Valid Name".to_string(), None, None, Some("invalidcolor".to_string()));
        assert!(matches!(result, Err(WorkspaceCoreError::InvalidAccentColorFormat(hex)) if hex == "invalidcolor"));
        
        let result_short = Workspace::new("Valid Name".to_string(), None, None, Some("#123".to_string()));
        assert!(matches!(result_short, Err(WorkspaceCoreError::InvalidAccentColorFormat(hex)) if hex == "#123"));
    }
    
    #[test]
    fn workspace_new_accent_color_valid() {
        assert!(Workspace::new("Valid Name".to_string(), None, None, Some("#RRGGBB".replace("R", "1").replace("G", "2").replace("B", "3"))).is_ok());
        assert!(Workspace::new("Valid Name".to_string(), None, None, Some("#RRGGBBAA".replace("R", "A").replace("G", "B").replace("B", "C").replace("A", "D"))).is_ok());
    }


    #[test]
    fn workspace_rename_valid() {
        let mut ws = Workspace::new("Old Name".to_string(), None, None, None).unwrap();
        ws.rename("New Name".to_string()).unwrap();
        assert_eq!(ws.name(), "New Name");
    }

    #[test]
    fn workspace_rename_invalid() {
        let mut ws = Workspace::new("Old Name".to_string(), None, None, None).unwrap();
        let result = ws.rename("".to_string());
        assert!(matches!(result, Err(WorkspaceCoreError::NameCannotBeEmpty)));
        assert_eq!(ws.name(), "Old Name"); // Name should not have changed
    }

    #[test]
    fn workspace_set_layout_type() {
        let mut ws = Workspace::new("Test".to_string(), None, None, None).unwrap();
        ws.set_layout_type(WorkspaceLayoutType::TilingVertical);
        assert_eq!(ws.layout_type(), WorkspaceLayoutType::TilingVertical);
    }

    #[test]
    fn workspace_add_remove_window_ids() {
        let mut ws = Workspace::new("Test".to_string(), None, None, None).unwrap();
        let win_id1 = WindowIdentifier::from("win1");
        let win_id2 = WindowIdentifier::from("win2");

        assert!(ws.add_window_id(win_id1.clone()));
        assert!(ws.window_ids().contains(&win_id1));
        assert!(!ws.add_window_id(win_id1.clone())); // Already present

        assert!(ws.add_window_id(win_id2.clone()));
        assert_eq!(ws.window_ids().len(), 2);

        assert!(ws.remove_window_id(&win_id1));
        assert!(!ws.window_ids().contains(&win_id1));
        assert!(!ws.remove_window_id(&win_id1)); // Not present anymore
        assert_eq!(ws.window_ids().len(), 1);
    }
    
    #[test]
    fn workspace_set_icon_name() {
        let mut ws = Workspace::new("Test".to_string(), None, None, None).unwrap();
        ws.set_icon_name(Some("my-icon".to_string()));
        assert_eq!(ws.icon_name(), Some("my-icon"));
        ws.set_icon_name(None);
        assert_eq!(ws.icon_name(), None);
        ws.set_icon_name(Some("".to_string())); // Empty string should be treated as None
        assert_eq!(ws.icon_name(), None);
    }

    #[test]
    fn workspace_serde() {
        let ws = Workspace::new("Serde Test".to_string(), Some("serde-pid".to_string()), Some("icon".to_string()), Some("#ABCDEF".to_string())).unwrap();
        let serialized = serde_json::to_string_pretty(&ws).unwrap();
        
        // Check a few fields
        assert!(serialized.contains("\"name\": \"Serde Test\""));
        assert!(serialized.contains("\"persistent_id\": \"serde-pid\""));
        assert!(serialized.contains("\"layout_type\": \"floating\"")); // Default
        assert!(serialized.contains("\"accent_color_hex\": \"#ABCDEF\""));


        let deserialized: Workspace = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ws, deserialized); // Requires PartialEq
    }
    
    #[test]
    fn workspace_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Workspace>();

        let ws = Workspace::new("Test".to_string(), None, None, None).unwrap();
        thread::spawn(move || {
            // Use ws
            let _ = ws.name();
        }).join().unwrap();
    }
}
