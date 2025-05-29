use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// --- ApplicationId ---

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord)]
pub struct ApplicationId(String);

impl ApplicationId {
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        debug_assert!(!id_str.is_empty(), "ApplicationId cannot be empty");
        Self(id_str)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ApplicationId {
    fn from(id: String) -> Self {
        debug_assert!(!id.is_empty(), "ApplicationId cannot be empty when converting from String");
        Self(id)
    }
}

impl From<&str> for ApplicationId {
    fn from(id: &str) -> Self {
        debug_assert!(!id.is_empty(), "ApplicationId cannot be empty when converting from &str");
        Self(id.to_string())
    }
}

impl fmt::Display for ApplicationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --- UserSessionState ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UserSessionState {
    #[default]
    Active,
    Locked,
    Idle,
}

// --- ResourceIdentifier ---

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceIdentifier {
    pub r#type: String,
    pub id: String,
    pub label: Option<String>,
}

impl ResourceIdentifier {
    pub fn new(resource_type: impl Into<String>, resource_id: impl Into<String>, label: Option<String>) -> Self {
        let rt_str = resource_type.into();
        let rid_str = resource_id.into();
        debug_assert!(!rt_str.is_empty(), "ResourceIdentifier type cannot be empty");
        debug_assert!(!rid_str.is_empty(), "ResourceIdentifier id cannot be empty");
        Self {
            r#type: rt_str,
            id: rid_str,
            label,
        }
    }

    pub fn file(path: impl Into<String>, label: Option<String>) -> Self {
        let path_str = path.into();
        debug_assert!(!path_str.is_empty(), "ResourceIdentifier file path cannot be empty");
        Self {
            r#type: "file".to_string(),
            id: path_str,
            label,
        }
    }

    pub fn url(url_str: impl Into<String>, label: Option<String>) -> Self {
        let url_s = url_str.into();
        debug_assert!(!url_s.is_empty(), "ResourceIdentifier URL cannot be empty");
        Self {
            r#type: "url".to_string(),
            id: url_s,
            label,
        }
    }

    pub fn new_uuid(resource_type: impl Into<String>, label: Option<String>) -> Self {
        let rt_str = resource_type.into();
        debug_assert!(!rt_str.is_empty(), "ResourceIdentifier type for UUID cannot be empty");
        Self {
            r#type: rt_str,
            id: Uuid::new_v4().to_string(),
            label,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // --- ApplicationId Tests ---
    #[test]
    fn application_id_new() {
        let app_id = ApplicationId::new("test.app");
        assert_eq!(app_id.as_str(), "test.app");
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn application_id_new_empty_panic_debug() {
        ApplicationId::new("");
    }

    #[test]
    fn application_id_as_str() {
        let app_id = ApplicationId::new("test.app");
        assert_eq!(app_id.as_str(), "test.app");
    }

    #[test]
    fn application_id_from_string() {
        let app_id = ApplicationId::from("test.app".to_string());
        assert_eq!(app_id.as_str(), "test.app");
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn application_id_from_string_empty_panic_debug() {
        ApplicationId::from("".to_string());
    }

    #[test]
    fn application_id_from_str() {
        let app_id = ApplicationId::from("test.app");
        assert_eq!(app_id.as_str(), "test.app");
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn application_id_from_str_empty_panic_debug() {
        ApplicationId::from("");
    }

    #[test]
    fn application_id_display() {
        let app_id = ApplicationId::new("test.app");
        assert_eq!(format!("{}", app_id), "test.app");
    }

    #[test]
    fn application_id_serde() {
        let app_id = ApplicationId::new("test.app");
        let serialized = serde_json::to_string(&app_id).unwrap();
        assert_eq!(serialized, "\"test.app\"");
        let deserialized: ApplicationId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, app_id);
    }
    
    #[test]
    fn application_id_default() {
        let app_id_default = ApplicationId::default();
        assert_eq!(app_id_default.as_str(), ""); 
        // Note: This default is an empty string. Using it with `ApplicationId::new(app_id_default.0)`
        // would panic in debug builds. This is acceptable as per requirements.
    }

    // --- UserSessionState Tests ---
    #[test]
    fn user_session_state_default() {
        assert_eq!(UserSessionState::default(), UserSessionState::Active);
    }

    #[test]
    fn user_session_state_serde() {
        let state = UserSessionState::Locked;
        let serialized = serde_json::to_string(&state).unwrap();
        assert_eq!(serialized, "\"Locked\"");
        let deserialized: UserSessionState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, state);

        let state_active = UserSessionState::Active;
        let serialized_active = serde_json::to_string(&state_active).unwrap();
        assert_eq!(serialized_active, "\"Active\"");
        let deserialized_active: UserSessionState = serde_json::from_str(&serialized_active).unwrap();
        assert_eq!(deserialized_active, state_active);

        let state_idle = UserSessionState::Idle;
        let serialized_idle = serde_json::to_string(&state_idle).unwrap();
        assert_eq!(serialized_idle, "\"Idle\"");
        let deserialized_idle: UserSessionState = serde_json::from_str(&serialized_idle).unwrap();
        assert_eq!(deserialized_idle, state_idle);
    }

    // --- ResourceIdentifier Tests ---
    #[test]
    fn resource_identifier_new() {
        let res_id = ResourceIdentifier::new("type_a", "id_123", Some("Label A".to_string()));
        assert_eq!(res_id.r#type, "type_a");
        assert_eq!(res_id.id, "id_123");
        assert_eq!(res_id.label, Some("Label A".to_string()));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn resource_identifier_new_empty_type_panic_debug() {
        ResourceIdentifier::new("", "id_123", None);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn resource_identifier_new_empty_id_panic_debug() {
        ResourceIdentifier::new("type_a", "", None);
    }

    #[test]
    fn resource_identifier_file() {
        let res_id = ResourceIdentifier::file("/path/to/file.txt", Some("Document".to_string()));
        assert_eq!(res_id.r#type, "file");
        assert_eq!(res_id.id, "/path/to/file.txt");
        assert_eq!(res_id.label, Some("Document".to_string()));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn resource_identifier_file_empty_path_panic_debug() {
        ResourceIdentifier::file("", Some("Document".to_string()));
    }


    #[test]
    fn resource_identifier_url() {
        let res_id = ResourceIdentifier::url("https://example.com/resource", Some("Website".to_string()));
        assert_eq!(res_id.r#type, "url");
        assert_eq!(res_id.id, "https://example.com/resource");
        assert_eq!(res_id.label, Some("Website".to_string()));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn resource_identifier_url_empty_url_panic_debug() {
         ResourceIdentifier::url("", Some("Website".to_string()));
    }

    #[test]
    fn resource_identifier_new_uuid() {
        let res_id = ResourceIdentifier::new_uuid("user_prefs", Some("Preferences".to_string()));
        assert_eq!(res_id.r#type, "user_prefs");
        assert!(!res_id.id.is_empty()); // UUID should not be empty
        assert!(Uuid::parse_str(&res_id.id).is_ok()); // Check if ID is a valid UUID
        assert_eq!(res_id.label, Some("Preferences".to_string()));
    }
    
    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn resource_identifier_new_uuid_empty_type_panic_debug() {
        ResourceIdentifier::new_uuid("", Some("Preferences".to_string()));
    }

    #[test]
    fn resource_identifier_serde() {
        let res_id = ResourceIdentifier {
            r#type: "test_type".to_string(),
            id: "test_id".to_string(),
            label: Some("test_label".to_string()),
        };
        let serialized = serde_json::to_string(&res_id).unwrap();
        let expected_json = r#"{"r#type":"test_type","id":"test_id","label":"test_label"}"#; // Note: r#type might serialize as type
        // Adjusting expected JSON based on typical serde behavior for raw identifiers
        let expected_json_adj = r#"{"type":"test_type","id":"test_id","label":"test_label"}"#;
        assert_eq!(serialized, expected_json_adj);
        
        let deserialized: ResourceIdentifier = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, res_id);

        let res_id_no_label = ResourceIdentifier {
            r#type: "another_type".to_string(),
            id: "another_id".to_string(),
            label: None,
        };
        let serialized_no_label = serde_json::to_string(&res_id_no_label).unwrap();
        let expected_json_no_label = r#"{"type":"another_type","id":"another_id","label":null}"#;
        assert_eq!(serialized_no_label, expected_json_no_label);
        let deserialized_no_label: ResourceIdentifier = serde_json::from_str(&serialized_no_label).unwrap();
        assert_eq!(deserialized_no_label, res_id_no_label);
    }
}
