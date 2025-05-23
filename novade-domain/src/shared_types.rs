use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};

/// Represents a unique identifier for an application.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord)]
pub struct ApplicationId(String);

impl ApplicationId {
    /// Creates a new `ApplicationId`.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the provided `id` is empty.
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        debug_assert!(!id_str.is_empty(), "ApplicationId darf nicht leer sein.");
        // Potential character set validation can be added here.
        // For now, we'll stick to the non-empty constraint.
        Self(id_str)
    }

    /// Returns a string slice of the application ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for ApplicationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ApplicationId").field(&self.0).finish()
    }
}

impl Display for ApplicationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ApplicationId {
    fn from(id: String) -> Self {
        debug_assert!(!id.is_empty(), "ApplicationId darf nicht leer sein.");
        Self(id)
    }
}

impl From<&str> for ApplicationId {
    fn from(id: &str) -> Self {
        debug_assert!(!id.is_empty(), "ApplicationId darf nicht leer sein.");
        Self(id.to_string())
    }
}

/// Represents the state of a user session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UserSessionState {
    /// The session is active.
    #[default]
    Active,
    /// The session is locked.
    Locked,
    /// The session is idle.
    Idle,
}

/// Represents a generic resource identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceIdentifier {
    /// The type of the resource (e.g., "file", "url", "user").
    pub r#type: String,
    /// The unique identifier for the resource.
    pub id: String,
    /// An optional human-readable label for the resource.
    pub label: Option<String>,
}

impl ResourceIdentifier {
    /// Creates a new `ResourceIdentifier`.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `resource_type` or `resource_id` is empty.
    pub fn new(
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        label: Option<String>,
    ) -> Self {
        let type_str = resource_type.into();
        let id_str = resource_id.into();
        debug_assert!(!type_str.is_empty(), "ResourceIdentifier type darf nicht leer sein.");
        debug_assert!(!id_str.is_empty(), "ResourceIdentifier id darf nicht leer sein.");
        Self {
            r#type: type_str,
            id: id_str,
            label,
        }
    }

    /// Creates a new `ResourceIdentifier` for a file.
    pub fn file(path: impl Into<String>, label: Option<String>) -> Self {
        Self::new("file", path.into(), label)
    }

    /// Creates a new `ResourceIdentifier` for a URL.
    pub fn url(url_str: impl Into<String>, label: Option<String>) -> Self {
        Self::new("url", url_str.into(), label)
    }

    /// Creates a new `ResourceIdentifier` with a generated UUID as the ID.
    pub fn new_uuid(resource_type: impl Into<String>, label: Option<String>) -> Self {
        Self::new(resource_type, uuid::Uuid::new_v4().to_string(), label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Tests for ApplicationId
    #[test]
    fn application_id_new_and_as_str() {
        let app_id = ApplicationId::new("test_app");
        assert_eq!(app_id.as_str(), "test_app");
    }

    #[test]
    fn application_id_from_string() {
        let app_id = ApplicationId::from("test_app_string".to_string());
        assert_eq!(app_id.as_str(), "test_app_string");
    }

    #[test]
    fn application_id_from_str() {
        let app_id = ApplicationId::from("test_app_str");
        assert_eq!(app_id.as_str(), "test_app_str");
    }

    #[test]
    fn application_id_display() {
        let app_id = ApplicationId::new("display_test");
        assert_eq!(format!("{}", app_id), "display_test");
    }

    #[test]
    fn application_id_serde() {
        let app_id = ApplicationId::new("serde_test");
        let serialized = serde_json::to_string(&app_id).unwrap();
        assert_eq!(serialized, "\"serde_test\"");
        let deserialized: ApplicationId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, app_id);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ApplicationId darf nicht leer sein.")]
    fn application_id_new_empty_panic() {
        ApplicationId::new("");
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ApplicationId darf nicht leer sein.")]
    fn application_id_from_string_empty_panic() {
        ApplicationId::from("".to_string());
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ApplicationId darf nicht leer sein.")]
    fn application_id_from_str_empty_panic() {
        ApplicationId::from("");
    }

    // Tests for UserSessionState
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
    }

    // Tests for ResourceIdentifier
    #[test]
    fn resource_identifier_new() {
        let res_id = ResourceIdentifier::new("user", "123", Some("Test User".to_string()));
        assert_eq!(res_id.r#type, "user");
        assert_eq!(res_id.id, "123");
        assert_eq!(res_id.label, Some("Test User".to_string()));
    }

    #[test]
    fn resource_identifier_file() {
        let res_id = ResourceIdentifier::file("/path/to/file.txt", Some("Document".to_string()));
        assert_eq!(res_id.r#type, "file");
        assert_eq!(res_id.id, "/path/to/file.txt");
        assert_eq!(res_id.label, Some("Document".to_string()));
    }

    #[test]
    fn resource_identifier_url() {
        let res_id = ResourceIdentifier::url("https://example.com", None);
        assert_eq!(res_id.r#type, "url");
        assert_eq!(res_id.id, "https://example.com");
        assert_eq!(res_id.label, None);
    }

    #[test]
    fn resource_identifier_new_uuid() {
        let res_id = ResourceIdentifier::new_uuid("session", Some("Active Session".to_string()));
        assert_eq!(res_id.r#type, "session");
        assert!(!res_id.id.is_empty());
        // Check if the ID is a valid UUID
        assert!(Uuid::parse_str(&res_id.id).is_ok());
        assert_eq!(res_id.label, Some("Active Session".to_string()));
    }

    #[test]
    fn resource_identifier_serde() {
        let res_id = ResourceIdentifier {
            r#type: "product".to_string(),
            id: "prod_abc".to_string(),
            label: Some("Amazing Product".to_string()),
        };
        let serialized = serde_json::to_string(&res_id).unwrap();
        let expected_json = r#"{"type":"product","id":"prod_abc","label":"Amazing Product"}"#;
        assert_eq!(serialized, expected_json);
        let deserialized: ResourceIdentifier = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, res_id);

        let res_id_no_label = ResourceIdentifier {
            r#type: "order".to_string(),
            id: "ord_123".to_string(),
            label: None,
        };
        let serialized_no_label = serde_json::to_string(&res_id_no_label).unwrap();
        let expected_json_no_label = r#"{"type":"order","id":"ord_123","label":null}"#;
        assert_eq!(serialized_no_label, expected_json_no_label);
        let deserialized_no_label: ResourceIdentifier =
            serde_json::from_str(&serialized_no_label).unwrap();
        assert_eq!(deserialized_no_label, res_id_no_label);
    }
    
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ResourceIdentifier type darf nicht leer sein.")]
    fn resource_identifier_new_empty_type_panic() {
        ResourceIdentifier::new("", "some_id", None);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ResourceIdentifier id darf nicht leer sein.")]
    fn resource_identifier_new_empty_id_panic() {
        ResourceIdentifier::new("some_type", "", None);
    }
}
