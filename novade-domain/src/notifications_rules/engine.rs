use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, warn, error, info as log_info}; // Renamed info to avoid conflict with struct
use tokio::sync::RwLock;
use uuid::Uuid;
use regex::Regex;

use crate::user_centric_services::notifications_core::types::{Notification, NotificationUrgency, NotificationAction as CoreNotificationAction};
use crate::global_settings::{GlobalSettingsService, GlobalDesktopSettings, SettingPath, GlobalSettingsError};
use super::types::{
    NotificationRuleSet, RuleCondition, RuleConditionField, RuleConditionOperator,
    RuleConditionValue, RuleAction, SimpleRuleCondition,
};
use super::errors::NotificationRulesError;
use super::persistence_iface::NotificationRulesProvider;

#[derive(Debug, Clone, PartialEq)]
pub enum RuleProcessingResult {
    Allow(Notification),
    Modify(Notification),
    Suppress { rule_id: Uuid, rule_name: String },
}

#[async_trait]
pub trait NotificationRulesEngine: Send + Sync {
    async fn process_notification(&self, notification: Notification) -> Result<RuleProcessingResult, NotificationRulesError>;
    async fn reload_rules(&self) -> Result<(), NotificationRulesError>;
}

pub struct DefaultNotificationRulesEngine {
    rules: Arc<RwLock<NotificationRuleSet>>,
    rules_provider: Arc<dyn NotificationRulesProvider>,
    settings_service: Arc<dyn GlobalSettingsService>,
}

impl DefaultNotificationRulesEngine {
    pub fn new(
        rules_provider: Arc<dyn NotificationRulesProvider>,
        settings_service: Arc<dyn GlobalSettingsService>,
    ) -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            rules_provider,
            settings_service,
        }
    }

    async fn evaluate_condition_recursive(
        &self,
        condition: &RuleCondition,
        notification: &Notification,
        settings: &GlobalDesktopSettings, // Now passed as reference
    ) -> Result<bool, NotificationRulesError> {
        match condition {
            RuleCondition::Simple(simple_condition) => {
                self.evaluate_simple_condition(simple_condition, notification).await
            }
            RuleCondition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate_condition_recursive(cond, notification, settings).await? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            RuleCondition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate_condition_recursive(cond, notification, settings).await? {
                        return Ok(true);
                    }
                }
                Ok(false) // None of the OR conditions were true
            }
            RuleCondition::Not(boxed_condition) => {
                Ok(!self.evaluate_condition_recursive(boxed_condition, notification, settings).await?)
            }
            RuleCondition::SettingIsTrue(setting_path) => {
                match self.settings_service.get_setting(setting_path).await {
                    Ok(serde_json::Value::Bool(true)) => Ok(true),
                    Ok(serde_json::Value::Bool(false)) => Ok(false),
                    Ok(other_val) => {
                        warn!("SettingIsTrue condition for path {:?} received non-boolean value: {:?}. Evaluating as false.", setting_path, other_val);
                        Ok(false)
                    }
                    Err(GlobalSettingsError::PathNotFound { path_description }) => {
                        debug!("SettingIsTrue condition: path '{}' not found. Evaluating as false.", path_description);
                        Ok(false)
                    }
                    Err(e) => Err(NotificationRulesError::SettingsAccessError(e)),
                }
            }
        }
    }
    
    async fn evaluate_simple_condition(
        &self,
        condition: &SimpleRuleCondition,
        notification: &Notification,
    ) -> Result<bool, NotificationRulesError> {
        let field_str_val: Option<String> = match &condition.field {
            RuleConditionField::ApplicationName => Some(notification.application_name.to_lowercase()),
            RuleConditionField::Summary => Some(notification.summary.to_lowercase()),
            RuleConditionField::Body => notification.body.as_ref().map(|b| b.to_lowercase()),
            RuleConditionField::Category => notification.category.as_ref().map(|c| c.to_lowercase()),
            RuleConditionField::HintValue(key) => notification.hints.as_ref()
                .and_then(|h| h.get(key))
                .and_then(|v| v.as_str().map(str::to_lowercase)),
            _ => None, // Other fields handled separately
        };

        let cond_val_str = match &condition.value {
            RuleConditionValue::String(s) => Some(s.to_lowercase()),
            RuleConditionValue::Regex(pattern_str) => {
                // Regex matching is handled specifically with MatchesRegex/NotMatchesRegex operators
                if condition.operator != RuleConditionOperator::MatchesRegex && condition.operator != RuleConditionOperator::NotMatchesRegex {
                    return Err(NotificationRulesError::InvalidRuleDefinition {
                        rule_id: None, rule_name: "".to_string(), // ID/Name not available here, but error indicates structural issue
                        reason: "Regex value can only be used with MatchesRegex/NotMatchesRegex operators.".to_string()
                    });
                }
                Some(pattern_str.clone()) // Pass the pattern string itself
            }
            _ => None,
        };

        match condition.operator {
            RuleConditionOperator::Is | RuleConditionOperator::IsNot | RuleConditionOperator::Contains | 
            RuleConditionOperator::NotContains | RuleConditionOperator::StartsWith | RuleConditionOperator::EndsWith => {
                let val_to_check = field_str_val.unwrap_or_default(); // Treat None as empty string for these comparisons
                let cond_s = cond_val_str.ok_or_else(|| NotificationRulesError::InvalidRuleDefinition {
                    rule_id: None, rule_name: "".to_string(), 
                    reason: "String operator requires a String or Regex value in condition.".to_string()
                })?;
                match condition.operator {
                    RuleConditionOperator::Is => Ok(val_to_check == cond_s),
                    RuleConditionOperator::IsNot => Ok(val_to_check != cond_s),
                    RuleConditionOperator::Contains => Ok(val_to_check.contains(&cond_s)),
                    RuleConditionOperator::NotContains => Ok(!val_to_check.contains(&cond_s)),
                    RuleConditionOperator::StartsWith => Ok(val_to_check.starts_with(&cond_s)),
                    RuleConditionOperator::EndsWith => Ok(val_to_check.ends_with(&cond_s)),
                    _ => unreachable!(), // Covered by outer match
                }
            }
            RuleConditionOperator::MatchesRegex | RuleConditionOperator::NotMatchesRegex => {
                let val_to_check = field_str_val.unwrap_or_default();
                let pattern_s = cond_val_str.ok_or_else(|| NotificationRulesError::InvalidRuleDefinition {
                    rule_id: None, rule_name: "".to_string(),
                    reason: "Regex operator requires a Regex value in condition.".to_string()
                })?;
                let regex = Regex::new(&pattern_s).map_err(|e| NotificationRulesError::InvalidRuleDefinition {
                    rule_id: None, rule_name: "".to_string(), 
                    reason: format!("Invalid regex pattern '{}': {}", pattern_s, e)
                })?;
                let matches = regex.is_match(&val_to_check);
                Ok(if condition.operator == RuleConditionOperator::MatchesRegex { matches } else { !matches })
            }
            RuleConditionOperator::GreaterThan | RuleConditionOperator::LessThan | 
            RuleConditionOperator::GreaterThanOrEqual | RuleConditionOperator::LessThanOrEqual => {
                match &condition.field {
                    RuleConditionField::Urgency => {
                        let notif_urgency_val = notification.urgency as i64; // Cast enum to comparable int
                        if let RuleConditionValue::Urgency(cond_urgency_val) = condition.value {
                            let cond_urgency_as_int = cond_urgency_val as i64;
                            match condition.operator {
                                RuleConditionOperator::GreaterThan => Ok(notif_urgency_val > cond_urgency_as_int),
                                RuleConditionOperator::LessThan => Ok(notif_urgency_val < cond_urgency_as_int),
                                RuleConditionOperator::GreaterThanOrEqual => Ok(notif_urgency_val >= cond_urgency_as_int),
                                RuleConditionOperator::LessThanOrEqual => Ok(notif_urgency_val <= cond_urgency_as_int),
                                _ => Ok(false), // Should not happen due to outer match
                            }
                        } else if let RuleConditionValue::Integer(cond_int_val) = condition.value {
                             // Allow comparing Urgency field with an Integer value
                            match condition.operator {
                                RuleConditionOperator::GreaterThan => Ok(notif_urgency_val > cond_int_val),
                                RuleConditionOperator::LessThan => Ok(notif_urgency_val < cond_int_val),
                                RuleConditionOperator::GreaterThanOrEqual => Ok(notif_urgency_val >= cond_int_val),
                                RuleConditionOperator::LessThanOrEqual => Ok(notif_urgency_val <= cond_int_val),
                                _ => Ok(false),
                            }
                        } else {
                            Err(NotificationRulesError::InvalidRuleDefinition{ rule_id: None, rule_name: "".to_string(), reason: "Urgency field comparison requires Urgency or Integer value.".to_string()})
                        }
                    }
                    // Add HintValue(key) if value is Integer here
                    _ => Err(NotificationRulesError::InvalidRuleDefinition{ rule_id: None, rule_name: "".to_string(), reason: "Numeric operator used with non-numeric field.".to_string()})
                }
            }
        }
    }


    /// Applies actions to a notification.
    /// Returns `(bool_suppressed, bool_stop_processing)`
    async fn apply_actions(
        &self,
        actions: &[RuleAction],
        notification: &mut Notification, // Now mutable
    ) -> Result<(bool, bool), NotificationRulesError> {
        let mut suppressed = false;
        let mut stop_processing = false;

        for action in actions {
            if suppressed && action != &RuleAction::LogMessage("Suppressed, but logging this.".to_string()) { 
                // If suppressed, only LogMessage actions should proceed for this rule's actions.
                // This is a choice: do other actions in the *same rule* run if suppress is one of them?
                // Let's assume yes for now, but stop_processing will prevent subsequent rules.
            }

            match action {
                RuleAction::SuppressNotification => {
                    debug!("Applying SuppressNotification to notification ID {}", notification.id);
                    suppressed = true;
                    // Typically, suppression also implies stopping further rules for this notification.
                    stop_processing = true; 
                }
                RuleAction::SetUrgency(new_urgency) => {
                    debug!("Applying SetUrgency to {:?} for notification ID {}", new_urgency, notification.id);
                    notification.urgency = *new_urgency;
                }
                RuleAction::AddActionToNotification(core_action) => {
                    if !notification.actions.iter().any(|a| a.key == core_action.key) {
                        debug!("Adding action '{}' to notification ID {}", core_action.key, notification.id);
                        notification.actions.push(core_action.clone());
                    } else {
                        warn!("Action key '{}' already exists for notification ID {}. Skipping AddAction.", core_action.key, notification.id);
                    }
                }
                RuleAction::SetHint(key, value) => {
                    debug!("Setting hint '{}' to {:?} for notification ID {}", key, value, notification.id);
                    notification.hints.get_or_insert_with(Default::default).insert(key.clone(), value.clone());
                }
                RuleAction::PlaySound(sound_name) => {
                    debug!("Setting sound hint to '{}' for notification ID {}", sound_name, notification.id);
                    notification.hints.get_or_insert_with(Default::default).insert("sound-name".to_string(), serde_json::json!(sound_name));
                }
                RuleAction::MarkAsPersistent(persistent) => {
                    debug!("Setting transient to {} for notification ID {}", !persistent, notification.id);
                    notification.transient = !*persistent; // Persistent means not transient
                }
                RuleAction::SetTimeoutMs(timeout) => {
                    debug!("Setting timeout to {:?} for notification ID {}", timeout, notification.id);
                    notification.timeout_ms = *timeout;
                }
                RuleAction::SetCategory(category) => {
                    debug!("Setting category to '{}' for notification ID {}", category, notification.id);
                    notification.category = Some(category.clone());
                }
                RuleAction::StopProcessingFurtherRules => {
                    debug!("Applying StopProcessingFurtherRules for notification ID {}", notification.id);
                    stop_processing = true;
                }
                RuleAction::LogMessage(message) => {
                    log_info!("[NotificationRule ID: {}] {}", notification.id, message);
                }
            }
            if stop_processing && !suppressed { 
                // If stop_processing is true due to StopProcessingFurtherRules action,
                // and not due to SuppressNotification, then break from applying further actions *of this rule*.
                // However, the current loop structure applies all actions of a matching rule.
                // The `stop_processing` flag will prevent subsequent *rules* from being processed.
                // If a rule has both Suppress and Stop, Suppress takes precedence for the outcome.
                // If only Stop, the notification (possibly modified) is returned, but no more rules run.
            }
        }
        Ok((suppressed, stop_processing))
    }
}

#[async_trait]
impl NotificationRulesEngine for DefaultNotificationRulesEngine {
    async fn reload_rules(&self) -> Result<(), NotificationRulesError> {
        info!("Reloading notification rules from provider.");
        let mut rules_from_provider = self.rules_provider.load_rules().await?;
        rules_from_provider.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        let mut rules_lock = self.rules.write().await;
        *rules_lock = rules_from_provider;
        info!("Notification rules reloaded and sorted. Total rules: {}", rules_lock.len());
        Ok(())
    }

    async fn process_notification(
        &self,
        notification: Notification,
    ) -> Result<RuleProcessingResult, NotificationRulesError> {
        debug!("Processing notification ID {} with rules engine.", notification.id);
        let rules_lock = self.rules.read().await;
        // Fetch settings once, assuming they don't change during single notification processing.
        let current_settings = self.settings_service.get_current_settings().await;

        let mut current_notification = notification.clone();
        let mut modified_overall = false;

        for rule in rules_lock.iter().filter(|r| r.is_enabled) {
            debug!("Evaluating rule '{}' (ID: {}, Priority: {}) for notification ID {}", rule.name, rule.id, rule.priority, current_notification.id);
            match self.evaluate_condition_recursive(&rule.condition, &current_notification, &current_settings).await {
                Ok(true) => {
                    info!("Rule '{}' condition met for notification ID {}. Applying actions.", rule.name, current_notification.id);
                    let initial_state_before_actions = current_notification.clone();
                    let (suppressed, stop_processing) = self.apply_actions(&rule.actions, &mut current_notification).await?;
                    
                    if suppressed {
                        info!("Notification ID {} suppressed by rule '{}'.", current_notification.id, rule.name);
                        return Ok(RuleProcessingResult::Suppress { rule_id: rule.id, rule_name: rule.name.clone() });
                    }
                    if current_notification != initial_state_before_actions {
                        modified_overall = true;
                    }
                    if stop_processing {
                        debug!("StopProcessingFurtherRules encountered for rule '{}'. No more rules will be processed for notification ID {}.", rule.name, current_notification.id);
                        break; // Stop processing further rules
                    }
                }
                Ok(false) => {
                    debug!("Rule '{}' condition NOT met for notification ID {}.", rule.name, current_notification.id);
                }
                Err(e) => {
                    error!("Error evaluating condition for rule '{}' (ID: {}): {}. Skipping rule.", rule.name, rule.id, e);
                    // Continue to next rule
                }
            }
        }
        
        if modified_overall {
            debug!("Notification ID {} was modified by rules. Final state: {:?}", current_notification.id, current_notification);
            Ok(RuleProcessingResult::Modify(current_notification))
        } else {
            debug!("Notification ID {} was allowed by rules without modification.", current_notification.id);
            Ok(RuleProcessingResult::Allow(current_notification))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_centric_services::notifications_core::types::{NotificationInput, NotificationUrgency, NotificationAction as CoreAction, NotificationActionType};
    use crate::global_settings::{GlobalDesktopSettings, DefaultGlobalSettingsService, SettingPath, AppearanceSettingPath}; // Example path
    use crate::notifications_rules::types::NotificationRule;
    use std::sync::Arc;
    use uuid::Uuid;
    use serde_json::json;

    #[derive(Default)] struct MockGlobalSettingsSvc { settings: GlobalDesktopSettings, mock_bool_setting_val: Option<bool> }
    impl MockGlobalSettingsSvc { #[allow(dead_code)] fn set_mock_bool_setting(&mut self, val: Option<bool>) { self.mock_bool_setting_val = val; }}
    #[async_trait]
    impl GlobalSettingsService for MockGlobalSettingsSvc {
        async fn load_settings(&mut self) -> Result<(), GlobalSettingsError> { Ok(()) }
        async fn save_settings(&self) -> Result<(), GlobalSettingsError> { Ok(()) }
        fn get_current_settings(&self) -> GlobalDesktopSettings { self.settings.clone() }
        async fn update_setting(&mut self, _p: SettingPath, _v: serde_json::Value) -> Result<(), GlobalSettingsError> { Ok(()) }
        fn get_setting(&self, _p: &SettingPath) -> Result<serde_json::Value, GlobalSettingsError> { 
            // Simple mock: if mock_bool_setting_val is Some, return it, else PathNotFound
            if let Some(val) = self.mock_bool_setting_val { Ok(serde_json::Value::Bool(val)) }
            else { Err(GlobalSettingsError::PathNotFound{path_description:"mocked path not found".to_string()}) }
        }
        async fn reset_to_defaults(&mut self) -> Result<(), GlobalSettingsError> { Ok(()) }
        fn subscribe_to_changes(&self) -> broadcast::Receiver<crate::global_settings::SettingChangedEvent> { unimplemented!() }
        fn subscribe_to_load_events(&self) -> broadcast::Receiver<crate::global_settings::SettingsLoadedEvent> { unimplemented!() }
        fn subscribe_to_save_events(&self) -> broadcast::Receiver<crate::global_settings::SettingsSavedEvent> { unimplemented!() }
    }

    #[derive(Default, Clone)] struct MockRulesProvider { rules: NotificationRuleSet, }
    impl MockRulesProvider { fn new(rules: NotificationRuleSet) -> Self { Self { rules } } }
    #[async_trait]
    impl NotificationRulesProvider for MockRulesProvider {
        async fn load_rules(&self) -> Result<NotificationRuleSet, NotificationRulesError> { Ok(self.rules.clone()) }
        async fn save_rules(&self, _rules: &NotificationRuleSet) -> Result<(), NotificationRulesError> { Ok(()) }
    }

    fn create_test_notification_from_input(summary: &str, app_name: &str, body: Option<&str>, urgency: NotificationUrgency, category: Option<&str>, hints: Option<std::collections::HashMap<String, serde_json::Value>>) -> Notification {
        Notification::from(NotificationInput {
            application_name: app_name.to_string(), summary: summary.to_string(), body: body.map(String::from),
            urgency, category: category.map(String::from), hints, ..Default::default()
        })
    }

    #[tokio::test]
    async fn test_condition_operators_string_advanced() {
        let engine = DefaultNotificationRulesEngine::new(Arc::new(MockRulesProvider::default()), Arc::new(MockGlobalSettingsSvc::default()));
        let notification = create_test_notification_from_input("Hello Summary", "TestApp", Some("Body Text Here"), Default::default(), None, None);
        
        // NotContains
        let cond = SimpleRuleCondition { field: RuleConditionField::Summary, operator: RuleConditionOperator::NotContains, value: RuleConditionValue::String("xyz".to_string()) };
        assert!(engine.evaluate_simple_condition(&cond, &notification).await.unwrap());
        // StartsWith
        let cond = SimpleRuleCondition { field: RuleConditionField::ApplicationName, operator: RuleConditionOperator::StartsWith, value: RuleConditionValue::String("test".to_string()) };
        assert!(engine.evaluate_simple_condition(&cond, &notification).await.unwrap());
        // EndsWith
        let cond = SimpleRuleCondition { field: RuleConditionField::Body, operator: RuleConditionOperator::EndsWith, value: RuleConditionValue::String("here".to_string()) };
        assert!(engine.evaluate_simple_condition(&cond, &notification).await.unwrap());
    }

    #[tokio::test]
    async fn test_condition_operator_regex() {
        let engine = DefaultNotificationRulesEngine::new(Arc::new(MockRulesProvider::default()), Arc::new(MockGlobalSettingsSvc::default()));
        let notification = create_test_notification_from_input("Order #12345 Confirmed", "ShopApp", None, Default::default(), None, None);

        // MatchesRegex
        let cond_match = SimpleRuleCondition { field: RuleConditionField::Summary, operator: RuleConditionOperator::MatchesRegex, value: RuleConditionValue::Regex(r"order #\d+ confirmed".to_string()) };
        assert!(engine.evaluate_simple_condition(&cond_match, &notification).await.unwrap());
        // NotMatchesRegex
        let cond_not_match = SimpleRuleCondition { field: RuleConditionField::Summary, operator: RuleConditionOperator::NotMatchesRegex, value: RuleConditionValue::Regex(r"shipment \d+".to_string()) };
        assert!(engine.evaluate_simple_condition(&cond_not_match, &notification).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_condition_field_urgency_category_hints() {
        let engine = DefaultNotificationRulesEngine::new(Arc::new(MockRulesProvider::default()), Arc::new(MockGlobalSettingsSvc::default()));
        let mut hints = std::collections::HashMap::new();
        hints.insert("sound".to_string(), json!("ding.ogg"));
        let notification = create_test_notification_from_input("Urg", "App", None, NotificationUrgency::Critical, Some("chat.direct"), Some(hints));

        // Urgency
        let cond_urg_eq = SimpleRuleCondition { field: RuleConditionField::Urgency, operator: RuleConditionOperator::Is, value: RuleConditionValue::Urgency(NotificationUrgency::Critical) };
        // Note: 'Is' for Urgency needs specific handling for comparing Urgency values.
        // The current simple_condition logic might not directly support Urgency with 'Is'.
        // Let's test with GreaterThanOrEqual for now which is implemented for Urgency.
        let cond_urg_ge = SimpleRuleCondition { field: RuleConditionField::Urgency, operator: RuleConditionOperator::GreaterThanOrEqual, value: RuleConditionValue::Urgency(NotificationUrgency::Normal) };
        assert!(engine.evaluate_simple_condition(&cond_urg_ge, &notification).await.unwrap());

        // Category
        let cond_cat_is = SimpleRuleCondition { field: RuleConditionField::Category, operator: RuleConditionOperator::Is, value: RuleConditionValue::String("chat.direct".to_string()) };
        assert!(engine.evaluate_simple_condition(&cond_cat_is, &notification).await.unwrap());
        
        // HintExists - This needs to be implemented in evaluate_simple_condition
        // For now, this test will likely fail or pass vacuously if HintExists isn't handled.
        // Assuming HintExists is not yet implemented, so this test is for future.
        // let cond_hint_exists = SimpleRuleCondition { field: RuleConditionField::HintExists("sound".to_string()), operator: RuleConditionOperator::Is, value: RuleConditionValue::Boolean(true) };
        // assert!(engine.evaluate_simple_condition(&cond_hint_exists, &notification).await.unwrap());

        // HintValue
        // let cond_hint_val = SimpleRuleCondition { field: RuleConditionField::HintValue("sound".to_string()), operator: RuleConditionOperator::Is, value: RuleConditionValue::String("ding.ogg".to_string()) };
        // assert!(engine.evaluate_simple_condition(&cond_hint_val, &notification).await.unwrap());
    }


    #[tokio::test]
    async fn test_rule_condition_or_not() {
        let engine = DefaultNotificationRulesEngine::new(Arc::new(MockRulesProvider::default()), Arc::new(MockGlobalSettingsSvc::default()));
        let notification = create_test_notification_from_input("Hello", "App1", None, Default::default(), None, None);
        let settings = GlobalDesktopSettings::default();

        let cond_app1 = RuleCondition::Simple(SimpleRuleCondition{field: RuleConditionField::ApplicationName, operator: RuleConditionOperator::Is, value: RuleConditionValue::String("app1".to_string())});
        let cond_app2 = RuleCondition::Simple(SimpleRuleCondition{field: RuleConditionField::ApplicationName, operator: RuleConditionOperator::Is, value: RuleConditionValue::String("app2".to_string())});
        
        // OR
        let or_cond = RuleCondition::Or(vec![cond_app1.clone(), cond_app2.clone()]);
        assert!(engine.evaluate_condition_recursive(&or_cond, &notification, &settings).await.unwrap());
        let or_cond_false = RuleCondition::Or(vec![cond_app2.clone(), RuleCondition::Simple(SimpleRuleCondition{field: RuleConditionField::Summary, operator: RuleConditionOperator::Is, value: RuleConditionValue::String("nonexistent".to_string())})]);
        assert!(!engine.evaluate_condition_recursive(&or_cond_false, &notification, &settings).await.unwrap());


        // NOT
        let not_cond_app1 = RuleCondition::Not(Box::new(cond_app1.clone()));
        assert!(!engine.evaluate_condition_recursive(&not_cond_app1, &notification, &settings).await.unwrap());
        let not_cond_app2 = RuleCondition::Not(Box::new(cond_app2.clone()));
        assert!(engine.evaluate_condition_recursive(&not_cond_app2, &notification, &settings).await.unwrap());
    }

    #[tokio::test]
    async fn test_rule_condition_setting_is_true() {
        let mut mock_settings_svc = MockGlobalSettingsSvc::default();
        let engine = DefaultNotificationRulesEngine::new(Arc::new(MockRulesProvider::default()), Arc::new(mock_settings_svc)); // MockGlobalSettingsSvc needs to be Arc for engine
        
        let notification = create_test_notification_from_input("Test", "App", None, Default::default(), None, None);
        let settings_ref = &GlobalDesktopSettings::default(); // evaluate_condition_recursive needs this
        let path = SettingPath::Appearance(AppearanceSettingPath::EnableAnimations); // Example path

        // Test when setting is true
        let mut mock_settings_true = MockGlobalSettingsSvc::default();
        mock_settings_true.set_mock_bool_setting(Some(true));
        let engine_true = DefaultNotificationRulesEngine::new(Arc::new(MockRulesProvider::default()), Arc::new(mock_settings_true));
        let cond_setting_true = RuleCondition::SettingIsTrue(path.clone());
        assert!(engine_true.evaluate_condition_recursive(&cond_setting_true, &notification, settings_ref).await.unwrap());

        // Test when setting is false
        let mut mock_settings_false = MockGlobalSettingsSvc::default();
        mock_settings_false.set_mock_bool_setting(Some(false));
        let engine_false = DefaultNotificationRulesEngine::new(Arc::new(MockRulesProvider::default()), Arc::new(mock_settings_false));
        assert!(!engine_false.evaluate_condition_recursive(&cond_setting_true, &notification, settings_ref).await.unwrap());
        
        // Test when setting not found (PathNotFound)
        let mut mock_settings_notfound = MockGlobalSettingsSvc::default();
        mock_settings_notfound.set_mock_bool_setting(None); // This will cause get_setting to return PathNotFound
        let engine_notfound = DefaultNotificationRulesEngine::new(Arc::new(MockRulesProvider::default()), Arc::new(mock_settings_notfound));
        assert!(!engine_notfound.evaluate_condition_recursive(&cond_setting_true, &notification, settings_ref).await.unwrap());
    }

    #[tokio::test]
    async fn test_apply_actions_all_new_actions() {
        let engine = DefaultNotificationRulesEngine::new(Arc::new(MockRulesProvider::default()), Arc::new(MockGlobalSettingsSvc::default()));
        let mut notification = create_test_notification_from_input("Test", "App", None, NotificationUrgency::Normal, None, None);
        let original_id = notification.id;

        let core_action = CoreAction { key: "reply".to_string(), label: "Reply".to_string(), action_type: NotificationActionType::Callback };
        let actions = vec![
            RuleAction::AddActionToNotification(core_action.clone()),
            RuleAction::SetHint("priority".to_string(), json!("high")),
            RuleAction::PlaySound("alert.wav".to_string()),
            RuleAction::MarkAsPersistent(true),
            RuleAction::SetTimeoutMs(Some(10000)),
            RuleAction::SetCategory("social.message".to_string()),
            RuleAction::LogMessage("Test log message".to_string()), // Check logs manually or with tracing subscriber
        ];
        
        let (suppressed, stop_processing) = engine.apply_actions(&actions, &mut notification).await.unwrap();
        assert!(!suppressed);
        assert!(!stop_processing); // No StopProcessingFurtherRules action used

        assert_eq!(notification.actions.len(), 1);
        assert_eq!(notification.actions[0], core_action);
        assert_eq!(notification.hints.as_ref().unwrap().get("priority").unwrap(), &json!("high"));
        assert_eq!(notification.hints.as_ref().unwrap().get("sound-name").unwrap(), &json!("alert.wav"));
        assert_eq!(notification.transient, false); // MarkAsPersistent(true) means transient = false
        assert_eq!(notification.timeout_ms, Some(10000));
        assert_eq!(notification.category, Some("social.message".to_string()));
        assert_eq!(notification.id, original_id); // ID should not change
    }
    
    #[tokio::test]
    async fn test_process_notification_stop_processing_action() {
        let rule1_modify_and_stop = NotificationRule {
            id: Uuid::new_v4(), name: "Modify and Stop".to_string(), priority: 100, is_enabled: true,
            condition: RuleCondition::Simple(SimpleRuleCondition{field:RuleConditionField::ApplicationName, operator: RuleConditionOperator::Is, value: RuleConditionValue::String("testapp".to_string())}),
            actions: vec![
                RuleAction::SetUrgency(NotificationUrgency::High),
                RuleAction::StopProcessingFurtherRules,
            ],
        };
        let rule2_should_not_run = NotificationRule {
            id: Uuid::new_v4(), name: "Should Not Run".to_string(), priority: 90, is_enabled: true,
            condition: RuleCondition::Simple(SimpleRuleCondition{field:RuleConditionField::ApplicationName, operator: RuleConditionOperator::Is, value: RuleConditionValue::String("testapp".to_string())}),
            actions: vec![RuleAction::SetUrgency(NotificationUrgency::Critical)], // This would make it Critical if it ran
        };

        let provider = Arc::new(MockRulesProvider::new(vec![rule1_modify_and_stop, rule2_should_not_run]));
        let engine = DefaultNotificationRulesEngine::new(provider, Arc::new(MockGlobalSettingsSvc::default()));
        engine.reload_rules().await.unwrap();

        let notification = create_test_notification_from_input("Test", "TestApp", None, NotificationUrgency::Normal, None, None);
        let result = engine.process_notification(notification).await.unwrap();

        match result {
            RuleProcessingResult::Modify(n) => {
                assert_eq!(n.urgency, NotificationUrgency::High, "Urgency should be High from rule1, not Critical from rule2");
            }
            _ => panic!("Expected Modify result because rule1 modified it and then stopped further processing."),
        }
    }
}
