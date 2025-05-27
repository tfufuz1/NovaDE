use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NotificationUrgency {
    Low,
    #[default]
    Normal,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NotificationActionType {
    #[default]
    Callback, // Invokes a callback in the sending application
    OpenLink, // Opens a URL
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationAction {
    pub key: String, // Unique identifier for the action within this notification
    pub label: String, // Text displayed on the button/action item
    pub action_type: NotificationActionType,
    // pub target_link: Option<String>, // For OpenLink, future iteration
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationInput {
    pub application_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_icon: Option<String>, // e.g., path or icon name
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<NotificationAction>,
    #[serde(default)]
    pub urgency: NotificationUrgency,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>, // e.g., "email.new", "chat.mention"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hints: Option<HashMap<String, serde_json::Value>>, // e.g., "sound-name", "image-path"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>, // 0 for persistent, None for default system timeout
    #[serde(default)]
    pub transient: bool, // Default: false. If true, bypasses history.
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub application_name: String,
    pub application_icon: Option<String>,
    pub summary: String,
    pub body: Option<String>,
    pub actions: Vec<NotificationAction>,
    pub urgency: NotificationUrgency,
    pub category: Option<String>,
    pub hints: Option<HashMap<String, serde_json::Value>>,
    pub timeout_ms: Option<u32>,
    pub transient: bool,
    pub timestamp: DateTime<Utc>,
    pub is_read: bool,
    pub is_dismissed: bool,
}

impl Notification {
    pub fn new(input: NotificationInput, id: Uuid, timestamp: DateTime<Utc>) -> Self {
        Self {
            id,
            application_name: input.application_name,
            application_icon: input.application_icon,
            summary: input.summary,
            body: input.body,
            actions: input.actions,
            urgency: input.urgency,
            category: input.category,
            hints: input.hints,
            timeout_ms: input.timeout_ms,
            transient: input.transient,
            timestamp,
            is_read: false,
            is_dismissed: false,
        }
    }
}

impl From<NotificationInput> for Notification {
    fn from(input: NotificationInput) -> Self {
        Notification::new(input, Uuid::new_v4(), Utc::now())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_urgency_default() {
        assert_eq!(NotificationUrgency::default(), NotificationUrgency::Normal);
    }

    #[test]
    fn notification_action_type_default() {
        assert_eq!(NotificationActionType::default(), NotificationActionType::Callback);
    }

    #[test]
    fn notification_from_input() {
        let input = NotificationInput {
            application_name: "TestApp".to_string(),
            application_icon: None,
            summary: "Test Summary".to_string(),
            body: Some("Test Body".to_string()),
            actions: vec![NotificationAction {
                key: "ack".to_string(),
                label: "Acknowledge".to_string(),
                action_type: NotificationActionType::Callback,
            }],
            urgency: NotificationUrgency::Critical,
            category: Some("test.category".to_string()),
            hints: None,
            timeout_ms: Some(5000),
            transient: true,
        };
        let notification = Notification::from(input.clone());

        assert_eq!(notification.application_name, input.application_name);
        assert_eq!(notification.summary, input.summary);
        assert_eq!(notification.urgency, input.urgency);
        assert_eq!(notification.transient, input.transient);
        assert_eq!(notification.actions.len(), 1);
        assert!(!notification.id.is_nil());
        assert!(!notification.is_read);
        assert!(!notification.is_dismissed);
        assert!(notification.timestamp <= Utc::now());
    }

    #[test]
    fn notification_input_default_values_via_serde() {
        let json_minimal = r#"
        {
            "application_name": "MinimalApp",
            "summary": "Minimal Summary"
        }
        "#;
        let input: NotificationInput = serde_json::from_str(json_minimal).unwrap();
        assert_eq!(input.application_name, "MinimalApp");
        assert_eq!(input.summary, "Minimal Summary");
        assert_eq!(input.application_icon, None);
        assert!(input.actions.is_empty());
        assert_eq!(input.urgency, NotificationUrgency::Normal); // Default
        assert_eq!(input.transient, false); // Default
        assert_eq!(input.timeout_ms, None); // Default
    }
}
