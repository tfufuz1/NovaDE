use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::user_centric_services::notifications_core::types::{NotificationUrgency, NotificationAction as CoreNotificationAction}; // Corrected paths
use crate::global_settings::paths::SettingPath; // Corrected path

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleConditionValue {
    String(String),
    Boolean(bool),
    Integer(i64),
    Urgency(NotificationUrgency),
    Regex(String), // Stores the regex pattern as a string
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleConditionOperator {
    // String/General
    Is,
    IsNot,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    MatchesRegex,
    NotMatchesRegex,
    // Numeric/Urgency comparison
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleConditionField {
    ApplicationName,
    Summary,
    Body,
    Urgency,
    Category,
    HintExists(String), // Check if a hint with the given key exists
    HintValue(String),  // Check the value of a hint with the given key (value matched via RuleConditionValue)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleRuleCondition {
    pub field: RuleConditionField,
    pub operator: RuleConditionOperator,
    pub value: RuleConditionValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleCondition {
    Simple(SimpleRuleCondition),
    And(Vec<RuleCondition>),
    Or(Vec<RuleCondition>),
    Not(Box<RuleCondition>),
    SettingIsTrue(SettingPath), // Path to a boolean global setting
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleAction {
    SuppressNotification,
    SetUrgency(NotificationUrgency),
    AddActionToNotification(CoreNotificationAction), // Use the core NotificationAction
    SetHint(String, serde_json::Value), // key, value
    PlaySound(String), // sound_name_or_path
    MarkAsPersistent(bool), // true for persistent, false for default (transient if not specified)
    SetTimeoutMs(Option<u32>), // None for system default, Some(0) for never, Some(ms) for specific
    SetCategory(String),
    StopProcessingFurtherRules,
    LogMessage(String), // For debugging rules
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationRule {
    pub id: Uuid,
    pub name: String,
    pub condition: RuleCondition,
    pub actions: Vec<RuleAction>,
    #[serde(default = "default_is_enabled")]
    pub is_enabled: bool,
    #[serde(default)]
    pub priority: i32, // Higher numbers = higher priority
}

fn default_is_enabled() -> bool {
    true
}

impl Default for NotificationRule {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            condition: RuleCondition::And(Vec::new()),
            actions: Vec::new(),
            is_enabled: true,
            priority: 0,
        }
    }
}

pub type NotificationRuleSet = Vec<NotificationRule>;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_centric_services::notifications_core::types::NotificationActionType;

    #[test]
    fn rule_condition_value_serialization() {
        let val_str = RuleConditionValue::String("hello".to_string());
        assert_eq!(serde_json::to_string(&val_str).unwrap(), r#"{"String":"hello"}"#);
        let val_bool = RuleConditionValue::Boolean(true);
        assert_eq!(serde_json::to_string(&val_bool).unwrap(), r#"{"Boolean":true}"#);
        let val_int = RuleConditionValue::Integer(123);
        assert_eq!(serde_json::to_string(&val_int).unwrap(), r#"{"Integer":123}"#);
        let val_urgency = RuleConditionValue::Urgency(NotificationUrgency::Critical);
        assert_eq!(serde_json::to_string(&val_urgency).unwrap(), r#"{"Urgency":"Critical"}"#);
        let val_regex = RuleConditionValue::Regex("^test$".to_string());
        assert_eq!(serde_json::to_string(&val_regex).unwrap(), r#"{"Regex":"^test$"}"#);
    }

    #[test]
    fn rule_action_serialization() {
        let action_suppress = RuleAction::SuppressNotification;
        assert_eq!(serde_json::to_string(&action_suppress).unwrap(), r#""SuppressNotification""#);
        
        let action_set_urgency = RuleAction::SetUrgency(NotificationUrgency::Low);
        assert_eq!(serde_json::to_string(&action_set_urgency).unwrap(), r#"{"SetUrgency":"Low"}"#);

        let core_action = CoreNotificationAction {
            key: "my_key".to_string(),
            label: "My Label".to_string(),
            action_type: NotificationActionType::Callback,
        };
        let action_add_action = RuleAction::AddActionToNotification(core_action);
        let expected_json = r#"{"AddActionToNotification":{"key":"my_key","label":"My Label","action_type":"Callback"}}"#;
        assert_eq!(serde_json::to_string(&action_add_action).unwrap(), expected_json);

        let action_set_hint = RuleAction::SetHint("sound".to_string(), serde_json::json!("ding.ogg"));
        assert_eq!(serde_json::to_string(&action_set_hint).unwrap(), r#"{"SetHint":["sound","ding.ogg"]}"#);
    }
    
    #[test]
    fn rule_condition_field_serialization() {
        let field_app_name = RuleConditionField::ApplicationName;
        assert_eq!(serde_json::to_string(&field_app_name).unwrap(), r#""ApplicationName""#);
        let field_hint_exists = RuleConditionField::HintExists("image-path".to_string());
        assert_eq!(serde_json::to_string(&field_hint_exists).unwrap(), r#"{"HintExists":"image-path"}"#);
    }

     #[test]
    fn rule_condition_setting_is_true_serialization() {
        // Example path, ensure SettingPath itself is serializable if this is to work.
        // SettingPath would need to implement Serialize.
        // For this test, we assume SettingPath can be serialized to a string or structured object.
        // Let's create a dummy SettingPath variant for testing if not already available.
        // Assuming SettingPath::Appearance(AppearanceSettingPath::EnableAnimations)
        let dummy_setting_path = SettingPath::Appearance(crate::global_settings::paths::AppearanceSettingPath::EnableAnimations);
        let cond = RuleCondition::SettingIsTrue(dummy_setting_path);
        
        // The exact JSON depends on SettingPath's Serialize impl.
        // If it serializes to its string representation:
        let expected_json = r#"{"SettingIsTrue":"appearance.enable_animations"}"#;
        assert_eq!(serde_json::to_string(&cond).unwrap(), expected_json);
    }
}
