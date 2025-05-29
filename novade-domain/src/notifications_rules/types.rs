use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::user_centric_services::notifications_core::types::{NotificationUrgency, NotificationAction as CoreNotificationAction};
use crate::global_settings::paths::SettingPath;
use novade_core::types::Color as CoreColor;

// --- RuleConditionValue Enum ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleConditionValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Urgency(NotificationUrgency),
    Regex(String),
}

// --- RuleConditionOperator Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleConditionOperator {
    Is,
    IsNot,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    MatchesRegex,
    NotMatchesRegex,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

// --- RuleConditionField Enum ---
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleConditionField {
    ApplicationName,
    Summary,
    Body,
    Urgency,
    Category,
    HintExists(String),
    HintValue(String),
}

// --- SimpleRuleCondition Struct ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleRuleCondition {
    pub field: RuleConditionField,
    pub operator: RuleConditionOperator,
    pub value: RuleConditionValue,
}

// --- RuleCondition Enum (recursive) ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleCondition {
    Simple(SimpleRuleCondition),
    SettingIsTrue(SettingPath),
    And(Vec<RuleCondition>),
    Or(Vec<RuleCondition>),
    Not(Box<RuleCondition>),
}

// --- RuleAction Enum ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleAction {
    SuppressNotification,
    SetUrgency(NotificationUrgency),
    AddActionToNotification(CoreNotificationAction),
    SetHint(String, serde_json::Value),
    PlaySound(String),
    MarkAsPersistent(bool),
    SetTimeoutMs(Option<u32>),
    SetCategory(String),
    SetSummary(String),
    SetBody(String),
    SetIcon(String),
    SetAccentColor(Option<CoreColor>),
    StopProcessingFurtherRules,
    LogMessage(String),
}

// --- NotificationRule Struct ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationRule {
    pub id: Uuid,
    pub name: String,
    pub condition: RuleCondition,
    pub actions: Vec<RuleAction>,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
    #[serde(default)]
    pub priority: i32,
}

fn default_true() -> bool { true }

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

// --- NotificationRuleSet Type Alias ---
pub type NotificationRuleSet = Vec<NotificationRule>;


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::user_centric_services::notifications_core::types::NotificationActionType;
    use crate::global_settings::paths::SettingPathParseError; // For testing SettingIsTrue

    #[test]
    fn rule_condition_value_serde() {
        let val_str = RuleConditionValue::String("test".to_string());
        let ser_str = serde_json::to_string(&val_str).unwrap();
        assert_eq!(ser_str, r#"{"string":"test"}"#);
        assert_eq!(serde_json::from_str::<RuleConditionValue>(&ser_str).unwrap(), val_str);

        let val_urgency = RuleConditionValue::Urgency(NotificationUrgency::Critical);
        let ser_urgency = serde_json::to_string(&val_urgency).unwrap();
        assert_eq!(ser_urgency, r#"{"urgency":"critical"}"#);
        assert_eq!(serde_json::from_str::<RuleConditionValue>(&ser_urgency).unwrap(), val_urgency);
        
        let val_regex = RuleConditionValue::Regex("test.*".to_string());
        let ser_regex = serde_json::to_string(&val_regex).unwrap();
        assert_eq!(ser_regex, r#"{"regex":"test.*"}"#);
        assert_eq!(serde_json::from_str::<RuleConditionValue>(&ser_regex).unwrap(), val_regex);
    }

    #[test]
    fn rule_condition_operator_serde() {
        let op = RuleConditionOperator::MatchesRegex;
        let ser = serde_json::to_string(&op).unwrap();
        assert_eq!(ser, "\"matches-regex\"");
        assert_eq!(serde_json::from_str::<RuleConditionOperator>(&ser).unwrap(), op);
    }

    #[test]
    fn rule_condition_field_serde() {
        let field_app = RuleConditionField::ApplicationName;
        let ser_app = serde_json::to_string(&field_app).unwrap();
        assert_eq!(ser_app, "\"application-name\"");
        assert_eq!(serde_json::from_str::<RuleConditionField>(&ser_app).unwrap(), field_app);

        let field_hint_val = RuleConditionField::HintValue("color".to_string());
        let ser_hint_val = serde_json::to_string(&field_hint_val).unwrap();
        assert_eq!(ser_hint_val, r#"{"hint-value":"color"}"#);
        assert_eq!(serde_json::from_str::<RuleConditionField>(&ser_hint_val).unwrap(), field_hint_val);
    }

    #[test]
    fn simple_rule_condition_serde() {
        let simple_cond = SimpleRuleCondition {
            field: RuleConditionField::Summary,
            operator: RuleConditionOperator::Contains,
            value: RuleConditionValue::String("urgent".to_string()),
        };
        let ser = serde_json::to_string(&simple_cond).unwrap();
        let de: SimpleRuleCondition = serde_json::from_str(&ser).unwrap();
        assert_eq!(simple_cond, de);
    }

    #[test]
    fn rule_condition_recursive_serde() {
        let cond = RuleCondition::And(vec![
            RuleCondition::Simple(SimpleRuleCondition {
                field: RuleConditionField::Urgency,
                operator: RuleConditionOperator::Is,
                value: RuleConditionValue::Urgency(NotificationUrgency::Critical), // Corrected from High
            }),
            RuleCondition::Not(Box::new(RuleCondition::SettingIsTrue(SettingPath::Root))),
        ]);
        let ser = serde_json::to_string_pretty(&cond).unwrap();
        let de: RuleCondition = serde_json::from_str(&ser).unwrap();
        assert_eq!(cond, de);
    }
    
    #[test]
    fn rule_condition_setting_is_true_serde() {
        let setting_path = SettingPath::Root;
        let cond = RuleCondition::SettingIsTrue(setting_path.clone());
        let ser = serde_json::to_string(&cond).unwrap();
        assert_eq!(ser, r#"{"setting-is-true":"root"}"#); // SettingPath::Root serializes to "root"
        let de: RuleCondition = serde_json::from_str(&ser).unwrap();
        assert_eq!(cond, de);
    }

    #[test]
    fn rule_action_serde() {
        let action_suppress = RuleAction::SuppressNotification;
        let ser_suppress = serde_json::to_string(&action_suppress).unwrap();
        assert_eq!(ser_suppress, "\"suppress-notification\"");
        assert_eq!(serde_json::from_str::<RuleAction>(&ser_suppress).unwrap(), action_suppress);

        let action_hint = RuleAction::SetHint("color".to_string(), json!("#FF0000"));
        let ser_hint = serde_json::to_string(&action_hint).unwrap();
        assert_eq!(ser_hint, r#"{"set-hint":["color","#FF0000"]}"#);
        assert_eq!(serde_json::from_str::<RuleAction>(&ser_hint).unwrap(), action_hint);
        
        let action_color = RuleAction::SetAccentColor(Some(CoreColor::from_hex("#123456").unwrap()));
        let ser_color = serde_json::to_string(&action_color).unwrap();
        // CoreColor's Serialize impl will determine this exact string.
        // Assuming it's something like {"r": R, "g": G, "b": B, "a": A } or "#RRGGBBAA" string
        // For now, let's check it contains the variant name
        assert!(ser_color.contains("set-accent-color"));
        let de_color: RuleAction = serde_json::from_str(&ser_color).unwrap();
        assert_eq!(action_color, de_color);
    }
    
    #[test]
    fn rule_action_add_notification_action_serde() {
        let core_action = CoreNotificationAction {
            key: "my-action".to_string(), label: "Click Me".to_string(),
            action_type: NotificationActionType::Callback,
        };
        let rule_action = RuleAction::AddActionToNotification(core_action.clone());
        let ser = serde_json::to_string(&rule_action).unwrap();
        let expected_json = r#"{"add-action-to-notification":{"key":"my-action","label":"Click Me","action_type":"callback"}}"#;
        assert_eq!(ser, expected_json);
        let de: RuleAction = serde_json::from_str(&ser).unwrap();
        assert_eq!(rule_action, de);
    }

    #[test]
    fn notification_rule_default_and_serde() {
        let rule_default = NotificationRule::default();
        assert_eq!(rule_default.name, "");
        assert_eq!(rule_default.is_enabled, true);
        assert_eq!(rule_default.priority, 0);
        assert_eq!(rule_default.condition, RuleCondition::And(Vec::new()));

        let rule = NotificationRule {
            id: Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8").unwrap(), // Fixed UUID for test
            name: "Test Rule".to_string(),
            condition: RuleCondition::Simple(SimpleRuleCondition {
                field: RuleConditionField::ApplicationName, operator: RuleConditionOperator::Is,
                value: RuleConditionValue::String("Firefox".to_string()),
            }),
            actions: vec![RuleAction::SuppressNotification],
            is_enabled: false, priority: 10,
        };
        let ser = serde_json::to_string_pretty(&rule).unwrap();
        let de: NotificationRule = serde_json::from_str(&ser).unwrap();
        assert_eq!(rule, de);
    }

    #[test]
    fn notification_rule_set_serde() {
        let rule1_id = Uuid::parse_str("b1b2b3b4-c1c2-d1d2-e1e2-e3e4e5e6e7e8").unwrap();
        let rule1 = NotificationRule { id: rule1_id, ..Default::default() };
        let rule_set: NotificationRuleSet = vec![rule1.clone(), NotificationRule::default()];
        let ser = serde_json::to_string(&rule_set).unwrap();
        let de: NotificationRuleSet = serde_json::from_str(&ser).unwrap();
        assert_eq!(rule_set.len(), de.len());
        assert_eq!(rule_set[0], de[0]);
    }
}
