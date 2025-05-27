use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord)]
pub struct ApplicationId(String);

impl ApplicationId {
    pub fn new(id: String) -> Self {
        debug_assert!(!id.is_empty(), "ApplicationId cannot be empty");
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ApplicationId {
    fn from(id: String) -> Self {
        debug_assert!(!id.is_empty(), "ApplicationId cannot be empty");
        Self(id)
    }
}

impl From<&str> for ApplicationId {
    fn from(id: &str) -> Self {
        debug_assert!(!id.is_empty(), "ApplicationId cannot be empty");
        Self(id.to_string())
    }
}

impl fmt::Display for ApplicationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserSessionState {
    Active,
    Locked,
    Idle,
}

impl Default for UserSessionState {
    fn default() -> Self {
        UserSessionState::Active
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceIdentifier {
    pub r#type: String,
    pub id: String,
    pub label: Option<String>,
}

impl ResourceIdentifier {
    pub fn new(r#type: String, id: String, label: Option<String>) -> Self {
        debug_assert!(!r#type.is_empty(), "ResourceIdentifier type cannot be empty");
        debug_assert!(!id.is_empty(), "ResourceIdentifier id cannot be empty");
        Self { r#type, id, label }
    }

    pub fn file(path: String, label: Option<String>) -> Self {
        Self::new("file".to_string(), path, label)
    }

    pub fn url(url: String, label: Option<String>) -> Self {
        Self::new("url".to_string(), url, label)
    }

    pub fn new_uuid(r#type: String, label: Option<String>) -> Self {
        Self::new(r#type, uuid::Uuid::new_v4().to_string(), label)
    }
}
