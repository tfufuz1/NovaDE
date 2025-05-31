use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::shared_types::ApplicationId;

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationUrgency {
    Low,
    #[default]
    Normal,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationActionType {
    #[default]
    Callback,
    OpenLink,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationAction {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub action_type: NotificationActionType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub application_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_icon: Option<String>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<NotificationAction>,
    #[serde(default)]
    pub urgency: NotificationUrgency,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub is_read: bool,
    #[serde(default)]
    pub is_dismissed: bool,
    #[serde(default)]
    pub transient: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub hints: HashMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
}

impl Notification {
    pub fn new(application_name: String, summary: String, urgency: NotificationUrgency) -> Self {
        Self {
            id: Uuid::new_v4(),
            application_name,
            application_icon: None,
            summary,
            body: None,
            actions: Vec::new(),
            urgency,
            timestamp: Utc::now(),
            is_read: false,
            is_dismissed: false,
            transient: false,
            category: None,
            hints: HashMap::new(),
            timeout_ms: None,
        }
    }

    pub fn mark_as_read(&mut self) {
        self.is_read = true;
    }

    pub fn dismiss(&mut self) {
        self.is_dismissed = true;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct NotificationInput {
    pub application_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_icon: Option<String>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<NotificationAction>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urgency: Option<NotificationUrgency>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transient: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hints: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct NotificationStats {
    pub num_active: usize,
    pub num_history: usize,
    pub num_unread_active: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DismissReason {
    ByUser,
    Expired,
    Replaced,
    AppClosed,
    SystemShutdown,
    AppScopeClear,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationFilterCriteria {
    Unread(bool),
    Application(ApplicationId),
    Urgency(NotificationUrgency),
    Category(String),
    HasActionWithKey(String),
    BodyContains(String),
    SummaryContains(String),
    IsTransient(bool),
    TimeRange {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        start: Option<DateTime<Utc>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        end: Option<DateTime<Utc>>,
    },
    And(Vec<NotificationFilterCriteria>),
    Or(Vec<NotificationFilterCriteria>),
    Not(Box<NotificationFilterCriteria>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationSortOrder {
    TimestampAscending,
    #[default]
    TimestampDescending,
    UrgencyAscending,
    UrgencyDescending,
    ApplicationNameAscending,
    ApplicationNameDescending,
    SummaryAscending,
    SummaryDescending,
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn notification_urgency_default_and_serde() {
        assert_eq!(NotificationUrgency::default(), NotificationUrgency::Normal);
        let urgency = NotificationUrgency::Critical;
        let serialized = serde_json::to_string(&urgency).unwrap();
        assert_eq!(serialized, "\"critical\"");
        let deserialized: NotificationUrgency = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, urgency);
    }

    #[test]
    fn notification_action_type_default_and_serde() {
        assert_eq!(NotificationActionType::default(), NotificationActionType::Callback);
        let action_type = NotificationActionType::OpenLink;
        let serialized = serde_json::to_string(&action_type).unwrap();
        assert_eq!(serialized, "\"open-link\"");
        let deserialized: NotificationActionType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, action_type);
    }

    #[test]
    fn notification_action_serde() {
        let action = NotificationAction {
            key: "reply".to_string(), label: "Reply Now".to_string(),
            action_type: NotificationActionType::Callback,
        };
        let serialized = serde_json::to_string(&action).unwrap();
        let deserialized: NotificationAction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(action, deserialized);
    }

    #[test]
    fn notification_new_and_methods() {
        let mut notif = Notification::new("MyApp".to_string(), "Test Summary".to_string(), NotificationUrgency::Low);
        assert_eq!(notif.application_name, "MyApp");
        assert_eq!(notif.urgency, NotificationUrgency::Low);
        notif.mark_as_read(); assert_eq!(notif.is_read, true);
        notif.dismiss(); assert_eq!(notif.is_dismissed, true);
    }

    #[test]
    fn notification_serde() {
        let notif = Notification::new("TestApp".to_string(), "Hello".to_string(), NotificationUrgency::Critical);
        let serialized = serde_json::to_string_pretty(&notif).unwrap();
        let deserialized: Notification = serde_json::from_str(&serialized).unwrap();
        assert_eq!(notif.id, deserialized.id);
        assert_eq!(notif.summary, deserialized.summary);
    }

    #[test]
    fn notification_serde_empty_actions_and_hints() {
        let mut notif = Notification::new("TestApp".to_string(), "Hello".to_string(), NotificationUrgency::Normal);
        notif.actions = vec![]; notif.hints = HashMap::new();
        let serialized = serde_json::to_string(&notif).unwrap();
        assert!(!serialized.contains("\"actions\":"));
        assert!(!serialized.contains("\"hints\":"));
        let deserialized: Notification = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.actions.is_empty() && deserialized.hints.is_empty());
    }

    #[test]
    fn notification_input_default_and_serde() {
        let default_input = NotificationInput::default();
        assert_eq!(default_input.application_name, "");
        let input = NotificationInput { application_name: "My App".to_string(), summary: "Input Summary".to_string(), urgency: Some(NotificationUrgency::Critical), ..Default::default() };
        let serialized = serde_json::to_string(&input).unwrap();
        let deserialized: NotificationInput = serde_json::from_str(&serialized).unwrap();
        assert_eq!(input, deserialized);
    }

    #[test]
    fn notification_stats_default_and_serde() {
        let default_stats = NotificationStats::default();
        assert_eq!(default_stats.num_active, 0);
        let stats = NotificationStats { num_active: 5, num_history: 100, num_unread_active: 2 };
        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: NotificationStats = serde_json::from_str(&serialized).unwrap();
        assert_eq!(stats, deserialized);
    }

    #[test]
    fn dismiss_reason_serde() {
        let reason = DismissReason::Expired;
        let serialized = serde_json::to_string(&reason).unwrap();
        assert_eq!(serialized, "\"expired\"");
        let deserialized: DismissReason = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, reason);
    }

    #[test]
    fn notification_filter_criteria_serde() {
        let criteria = NotificationFilterCriteria::Application(ApplicationId::new("app.id"));
        let serialized = serde_json::to_string(&criteria).unwrap();
        assert_eq!(serialized, r#"{"application":"app.id"}"#);
        let deserialized: NotificationFilterCriteria = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, criteria);

        let criteria_complex = NotificationFilterCriteria::And(vec![ NotificationFilterCriteria::Urgency(NotificationUrgency::Critical), NotificationFilterCriteria::Not(Box::new(NotificationFilterCriteria::IsTransient(true))), ]);
        let serialized_complex = serde_json::to_string_pretty(&criteria_complex).unwrap();
        let deserialized_complex: NotificationFilterCriteria = serde_json::from_str(&serialized_complex).unwrap();
        assert_eq!(deserialized_complex, criteria_complex);
    }

    #[test]
    fn notification_sort_order_default_and_serde() {
        assert_eq!(NotificationSortOrder::default(), NotificationSortOrder::TimestampDescending);
        let order = NotificationSortOrder::ApplicationNameAscending;
        let serialized = serde_json::to_string(&order).unwrap();
        assert_eq!(serialized, "\"application-name-ascending\"");
        let deserialized: NotificationSortOrder = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, order);
    }
}
