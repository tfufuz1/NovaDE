use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use regex::Regex;
use tracing::{debug, error, info, warn};

use crate::notifications::types::{Notification, NotificationAction as CoreNotificationAction, NotificationUrgency}; // MODIFIED
use crate::global_settings::{GlobalSettingsService, paths::SettingPath, errors::GlobalSettingsError};

// MODIFIED use block for rules_types
use crate::notifications::rules_types::{
    NotificationRuleSet, NotificationRule, RuleCondition, RuleAction,
    RuleConditionField, RuleConditionOperator, RuleConditionValue, SimpleRuleCondition,
};
use crate::notifications::rules_errors::NotificationRulesError; // MODIFIED
use crate::notifications::persistence_iface::NotificationRulesProvider; // MODIFIED

// --- RuleProcessingResult Enum ---
#[derive(Debug, Clone, PartialEq)]
pub enum RuleProcessingResult {
    Allow(Notification),
    Suppress { rule_id: Uuid },
}

// --- NotificationRulesEngine Trait ---
#[async_trait]
pub trait NotificationRulesEngine: Send + Sync {
    async fn reload_rules(&self) -> Result<(), NotificationRulesError>;
    async fn process_notification(&self, notification: Notification) -> Result<RuleProcessingResult, NotificationRulesError>;
    async fn get_rules(&self) -> Result<NotificationRuleSet, NotificationRulesError>;
    async fn update_rules(&self, new_rules: NotificationRuleSet) -> Result<(), NotificationRulesError>;
}

// --- DefaultNotificationRulesEngine Struct ---
pub struct DefaultNotificationRulesEngine {
    rules: Arc<RwLock<NotificationRuleSet>>,
    rules_provider: Arc<dyn NotificationRulesProvider>,
    settings_service: Arc<dyn GlobalSettingsService>,
    regex_cache: Arc<RwLock<HashMap<String, Result<Regex, NotificationRulesError>>>>,
}

impl DefaultNotificationRulesEngine {
    pub async fn new(
        rules_provider: Arc<dyn NotificationRulesProvider>,
        settings_service: Arc<dyn GlobalSettingsService>,
    ) -> Result<Arc<Self>, NotificationRulesError> {
        let engine = Arc::new(Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            rules_provider,
            settings_service,
            regex_cache: Arc::new(RwLock::new(HashMap::new())),
        });
        engine.load_rules_internal(false).await?;
        Ok(engine)
    }

    async fn load_rules_internal(&self, is_reload: bool) -> Result<(), NotificationRulesError> {
        let log_prefix = if is_reload { "Reloading" } else { "Initial load of" };
        debug!("{} notification rules...", log_prefix);

        let mut loaded_rules = self.rules_provider.load_rules().await?;

        let mut regex_cache_guard = self.regex_cache.write().await;
        regex_cache_guard.clear();

        for rule in &loaded_rules {
            // Pass the acquired lock guard (or rather, operate within its scope)
            self.validate_and_cache_regex_in_condition_recursive(&rule.condition, &mut regex_cache_guard)?;
        }
        drop(regex_cache_guard); // Release write lock on regex_cache

        loaded_rules.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.name.cmp(&b.name)));

        let mut rules_guard = self.rules.write().await;
        *rules_guard = loaded_rules;

        info!("Notification rules {} successfully. Rule count: {}", if is_reload { "reloaded" } else { "loaded" }, rules_guard.len());
        Ok(())
    }

    // Note: regex_cache is passed as &mut to operate under a single write lock acquired in load_rules_internal.
    fn validate_and_cache_regex_in_condition_recursive(
        &self,
        condition: &RuleCondition,
        regex_cache: &mut HashMap<String, Result<Regex, NotificationRulesError>>, // Pass mutable ref to the cache
    ) -> Result<(), NotificationRulesError> {
        match condition {
            RuleCondition::Simple(simple_cond) => {
                if simple_cond.operator == RuleConditionOperator::MatchesRegex || simple_cond.operator == RuleConditionOperator::NotMatchesRegex {
                    if let RuleConditionValue::Regex(pattern_str) = &simple_cond.value {
                        if !regex_cache.contains_key(pattern_str) {
                            match Regex::new(pattern_str) {
                                Ok(re) => { regex_cache.insert(pattern_str.clone(), Ok(re)); }
                                Err(e) => {
                                    let err = NotificationRulesError::InvalidRegex { pattern: pattern_str.clone(), source: e };
                                    // Store a cloneable representation of the error. regex::Error is Clone.
                                    regex_cache.insert(pattern_str.clone(), Err(NotificationRulesError::InvalidRegex { pattern: pattern_str.clone(), source: err.source().clone() }));
                                    return Err(NotificationRulesError::InvalidRegex { pattern: pattern_str.clone(), source: regex_cache.get(pattern_str).unwrap().as_ref().unwrap_err().downcast_ref::<regex::Error>().expect("Cached error not regex::Error").clone() });
                                }
                            }
                        }
                        // If it's already in cache and is an error, propagate it to ensure loading fails
                        if let Some(Err(cached_err)) = regex_cache.get(pattern_str) {
                             return Err(cached_err.clone_for_propagation_if_needed());
                        }
                    } else {
                        return Err(NotificationRulesError::InvalidRuleDefinition {
                            rule_id: None, rule_name: "UnknownRuleDuringValidation".to_string(), // Context is lost here
                            reason: format!("Regex operator used with non-Regex value: {:?}", simple_cond.value),
                        });
                    }
                }
            }
            RuleCondition::And(conditions) | RuleCondition::Or(conditions) => {
                for cond in conditions { self.validate_and_cache_regex_in_condition_recursive(cond, regex_cache)?; }
            }
            RuleCondition::Not(condition) => { self.validate_and_cache_regex_in_condition_recursive(condition.as_ref(), regex_cache)?; }
            _ => {}
        }
        Ok(())
    }

    async fn evaluate_condition_recursive(&self, condition: &RuleCondition, notification: &Notification, rule_name_for_error: &str, rule_id_for_error: Option<Uuid>) -> Result<bool, NotificationRulesError> {
        match condition {
            RuleCondition::Simple(simple_cond) => self.evaluate_simple_condition(simple_cond, notification, rule_name_for_error, rule_id_for_error).await,
            RuleCondition::SettingIsTrue(setting_path) => {
                match self.settings_service.get_setting(setting_path).await {
                    Ok(serde_json::Value::Bool(true)) => Ok(true),
                    Ok(_) => Ok(false), // Any other value or type is considered false for this condition
                    Err(GlobalSettingsError::PathNotFound { .. }) => { // Specific error handling for PathNotFound
                        warn!("Rule '{}' (ID: {:?}): Setting path {:?} not found. Condition defaults to false.", rule_name_for_error, rule_id_for_error, setting_path);
                        Ok(false)
                    }
                    Err(e) => { // Other GlobalSettingsError types
                        warn!("Rule '{}' (ID: {:?}): Failed to get setting {:?} for condition: {}. Assuming false.", rule_name_for_error, rule_id_for_error, setting_path, e);
                        Ok(false) // Default to false if setting access fails for other reasons
                    }
                }
            }
            RuleCondition::And(conditions) => {
                for cond in conditions { if !self.evaluate_condition_recursive(cond, notification, rule_name_for_error, rule_id_for_error).await? { return Ok(false); } }
                Ok(true)
            }
            RuleCondition::Or(conditions) => {
                for cond in conditions { if self.evaluate_condition_recursive(cond, notification, rule_name_for_error, rule_id_for_error).await? { return Ok(true); } }
                Ok(false)
            }
            RuleCondition::Not(condition) => Ok(!self.evaluate_condition_recursive(condition.as_ref(), notification, rule_name_for_error, rule_id_for_error).await?),
        }
    }

    async fn evaluate_simple_condition(&self, simple_cond: &SimpleRuleCondition, notification: &Notification, rule_name_for_error: &str, rule_id_for_error: Option<Uuid>) -> Result<bool, NotificationRulesError> {
        let field_str_value_opt: Option<String> = match &simple_cond.field {
            RuleConditionField::ApplicationName => Some(notification.application_name.clone()),
            RuleConditionField::Summary => Some(notification.summary.clone()),
            RuleConditionField::Body => notification.body.clone(),
            RuleConditionField::Urgency => Some(serde_json::to_string(&notification.urgency).unwrap_or_default().trim_matches('"').to_string()),
            RuleConditionField::Category => notification.category.clone(),
            RuleConditionField::HintExists(key) => return Ok(notification.hints.contains_key(key)),
            RuleConditionField::HintValue(key) => notification.hints.get(key).and_then(|v| match v {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Number(n) => Some(n.to_string()),
                serde_json::Value::Bool(b) => Some(b.to_string()),
                _ => None,
            }),
        };
        let field_str_value = field_str_value_opt.unwrap_or_default(); // Treat None as empty string for comparisons

        match &simple_cond.value {
            RuleConditionValue::String(rule_val_str) => {
                match simple_cond.operator {
                    RuleConditionOperator::Is => Ok(field_str_value == *rule_val_str),
                    RuleConditionOperator::IsNot => Ok(field_str_value != *rule_val_str),
                    RuleConditionOperator::Contains => Ok(field_str_value.contains(rule_val_str)),
                    RuleConditionOperator::NotContains => Ok(!field_str_value.contains(rule_val_str)),
                    RuleConditionOperator::StartsWith => Ok(field_str_value.starts_with(rule_val_str)),
                    RuleConditionOperator::EndsWith => Ok(field_str_value.ends_with(rule_val_str)),
                    _ => Err(NotificationRulesError::InvalidRuleDefinition { rule_id: rule_id_for_error, rule_name: rule_name_for_error.to_string(), reason: format!("Operator {:?} not applicable for String comparison on field {:?}.", simple_cond.operator, simple_cond.field)})
                }
            }
            RuleConditionValue::Regex(pattern_str) => {
                let regex_cache_guard = self.regex_cache.read().await;
                let regex_result = regex_cache_guard.get(pattern_str).ok_or_else(|| NotificationRulesError::InternalError(format!("Regex pattern '{}' not pre-compiled/cached. Rule: '{}' (ID: {:?})", pattern_str, rule_name_for_error, rule_id_for_error)))?;
                match regex_result {
                    Ok(re) => match simple_cond.operator {
                        RuleConditionOperator::MatchesRegex => Ok(re.is_match(&field_str_value)),
                        RuleConditionOperator::NotMatchesRegex => Ok(!re.is_match(&field_str_value)),
                        _ => Err(NotificationRulesError::InvalidRuleDefinition { rule_id: rule_id_for_error, rule_name: rule_name_for_error.to_string(), reason: format!("Operator {:?} not applicable for Regex comparison on field {:?}.", simple_cond.operator, simple_cond.field)})
                    },
                    Err(cached_err) => Err(cached_err.clone_for_propagation_if_needed()),
                }
            }
            RuleConditionValue::Urgency(rule_urgency) => {
                if simple_cond.field == RuleConditionField::Urgency {
                    match simple_cond.operator {
                        RuleConditionOperator::Is => Ok(notification.urgency == *rule_urgency),
                        RuleConditionOperator::IsNot => Ok(notification.urgency != *rule_urgency),
                        _ => Err(NotificationRulesError::InvalidRuleDefinition { rule_id: rule_id_for_error, rule_name: rule_name_for_error.to_string(), reason: format!("Operator {:?} not applicable for Urgency comparison beyond Is/IsNot.", simple_cond.operator)})
                    }
                } else { Err(NotificationRulesError::InvalidRuleDefinition { rule_id: rule_id_for_error, rule_name: rule_name_for_error.to_string(), reason: format!("Field {:?} cannot be compared with Urgency value.", simple_cond.field)}) }
            }
            RuleConditionValue::Integer(rule_val_int) => {
                if let RuleConditionField::HintValue(key) = &simple_cond.field {
                    if let Some(serde_json::Value::Number(num)) = notification.hints.get(key) {
                        if let Some(val_int) = num.as_i64() {
                            match simple_cond.operator {
                                RuleConditionOperator::Is => Ok(val_int == *rule_val_int), RuleConditionOperator::IsNot => Ok(val_int != *rule_val_int),
                                RuleConditionOperator::GreaterThan => Ok(val_int > *rule_val_int), RuleConditionOperator::LessThan => Ok(val_int < *rule_val_int),
                                RuleConditionOperator::GreaterThanOrEqual => Ok(val_int >= *rule_val_int), RuleConditionOperator::LessThanOrEqual => Ok(val_int <= *rule_val_int),
                                _ => Err(NotificationRulesError::InvalidRuleDefinition { rule_id: rule_id_for_error, rule_name: rule_name_for_error.to_string(), reason: format!("Operator {:?} not applicable for Integer comparison on HintValue.", simple_cond.operator)})
                            }
                        } else { Ok(false) }
                    } else { Ok(false) }
                } else { Err(NotificationRulesError::InvalidRuleDefinition { rule_id: rule_id_for_error, rule_name: rule_name_for_error.to_string(), reason: format!("Integer comparison not supported for field {:?}", simple_cond.field)}) }
            }
            RuleConditionValue::Boolean(rule_val_bool) => {
                 if let RuleConditionField::HintValue(key) = &simple_cond.field {
                    if let Some(serde_json::Value::Bool(val_bool)) = notification.hints.get(key) {
                         match simple_cond.operator {
                            RuleConditionOperator::Is => Ok(val_bool == rule_val_bool), RuleConditionOperator::IsNot => Ok(val_bool != rule_val_bool),
                            _ => Err(NotificationRulesError::InvalidRuleDefinition { rule_id: rule_id_for_error, rule_name: rule_name_for_error.to_string(), reason: format!("Operator {:?} not applicable for Boolean comparison on HintValue.", simple_cond.operator)})
                         }
                    } else { Ok(false) }
                 } else { Err(NotificationRulesError::InvalidRuleDefinition { rule_id: rule_id_for_error, rule_name: rule_name_for_error.to_string(), reason: format!("Boolean comparison not supported for field {:?}", simple_cond.field)}) }
            }
        }
    }

    async fn apply_actions_internal(&self, actions: &[RuleAction], notification: &mut Notification, rule: &NotificationRule) -> Result<bool, NotificationRulesError> {
        let mut stop_processing = false;
        for action in actions {
            match action {
                RuleAction::SuppressNotification => { /* This specific action is handled by process_notification directly to return Suppress result. */ }
                RuleAction::SetUrgency(urgency) => notification.urgency = *urgency,
                RuleAction::AddActionToNotification(core_action) => notification.actions.push(core_action.clone()),
                RuleAction::SetHint(key, value) => { notification.hints.insert(key.clone(), value.clone()); },
                RuleAction::PlaySound(sound_path_or_id) => { notification.hints.insert("sound-file".to_string(), serde_json::Value::String(sound_path_or_id.clone())); },
                RuleAction::MarkAsPersistent(is_persistent) => notification.transient = !*is_persistent,
                RuleAction::SetTimeoutMs(timeout) => notification.timeout_ms = *timeout,
                RuleAction::SetCategory(category) => notification.category = Some(category.clone()),
                RuleAction::SetSummary(summary) => notification.summary = summary.clone(),
                RuleAction::SetBody(body) => notification.body = Some(body.clone()),
                RuleAction::SetIcon(icon_path) => notification.application_icon = Some(icon_path.clone()),
                RuleAction::SetAccentColor(color_opt) => {
                    if let Some(c) = color_opt { notification.hints.insert("accent-color".to_string(), serde_json::Value::String(c.to_hex_string())); }
                    else { notification.hints.remove("accent-color"); }
                },
                RuleAction::LogMessage(message) => { info!("Rule Action (Rule: '{}' ID: {:?}): {}", rule.name, rule.id, message); }
                RuleAction::StopProcessingFurtherRules => { stop_processing = true; break; }
            }
        }
        Ok(stop_processing)
    }
}

#[async_trait]
impl NotificationRulesEngine for DefaultNotificationRulesEngine {
    async fn reload_rules(&self) -> Result<(), NotificationRulesError> { self.load_rules_internal(true).await }

    async fn process_notification(&self, notification: Notification) -> Result<RuleProcessingResult, NotificationRulesError> {
        let rules_guard = self.rules.read().await;
        let rules_snapshot = rules_guard.clone();
        drop(rules_guard);

        let mut current_notification = notification;

        for rule in rules_snapshot.iter().filter(|r| r.is_enabled) {
            debug!("Processing rule: '{}' (ID: {:?}, Prio: {}) for notif ID {}", rule.name, rule.id, rule.priority, current_notification.id);
            match self.evaluate_condition_recursive(&rule.condition, &current_notification, &rule.name, Some(rule.id)).await {
                Ok(true) => {
                    debug!("Rule condition MET for rule: '{}'", rule.name);
                    if rule.actions.contains(&RuleAction::SuppressNotification) {
                        info!("Notification {} suppressed by rule '{}'", current_notification.id, rule.name);
                        return Ok(RuleProcessingResult::Suppress { rule_id: rule.id });
                    }
                    let stop_further = self.apply_actions_internal(&rule.actions, &mut current_notification, rule).await?;
                    if stop_further { debug!("StopProcessingFurtherRules action for rule '{}'", rule.name); break; }
                }
                Ok(false) => { /* Condition not met */ }
                Err(e) => { error!("Error evaluating condition for rule '{}' (ID: {:?}): {}. Skipping rule.", rule.name, rule.id, e); }
            }
        }
        Ok(RuleProcessingResult::Allow(current_notification))
    }

    async fn get_rules(&self) -> Result<NotificationRuleSet, NotificationRulesError> { Ok(self.rules.read().await.clone()) }

    async fn update_rules(&self, mut new_rules: NotificationRuleSet) -> Result<(), NotificationRulesError> {
        debug!("Updating notification rules. New rule count: {}", new_rules.len());
        let mut temp_regex_cache = HashMap::new();
        for rule in &new_rules {
            self.validate_and_cache_regex_in_condition_recursive(&rule.condition, &mut temp_regex_cache).map_err(|e| {
                NotificationRulesError::InvalidRuleDefinition { rule_id: Some(rule.id), rule_name: rule.name.clone(), reason: format!("Invalid regex in rule condition: {}", e) }
            })?;
        }

        new_rules.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.name.cmp(&b.name)));

        let mut rules_guard = self.rules.write().await;
        let mut regex_cache_guard = self.regex_cache.write().await;
        *rules_guard = new_rules.clone();
        *regex_cache_guard = temp_regex_cache;
        drop(rules_guard); drop(regex_cache_guard);

        self.rules_provider.save_rules(&new_rules).await?;
        info!("Notification rules updated and saved successfully.");
        Ok(())
    }
}

// Helper for NotificationRulesError for caching, made more robust
impl NotificationRulesError {
    fn clone_for_propagation_if_needed(&self) -> Self {
        match self {
            NotificationRulesError::InvalidRegex { pattern, source } => NotificationRulesError::InvalidRegex { pattern: pattern.clone(), source: source.clone() },
            // For other error types, if they were to be stored in a cache that needs Clone, they'd need similar logic.
            // However, the regex_cache specifically stores Result<Regex, NotificationRulesError> where Err variant is InvalidRegex.
            _ => NotificationRulesError::InternalError(format!("Attempted to clone an unexpected error type from regex cache: {}", self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Assuming MockNotificationRulesProvider is defined below within this test module or accessible via super
    // If it were moved to persistence_iface, it would be:
    // use crate::notifications::persistence_iface::MockNotificationRulesProvider;
    // For now, relying on local definition.

    // Assuming MockGlobalSettingsService is defined below within this test module
    // If it were central, path would be different.
    // use crate::global_settings::{MockGlobalSettingsService, SettingPathParseError};

    use serde_json::json;
    use crate::notifications::types::NotificationUrgency; // MODIFIED
    use tokio::sync::broadcast;

    // Copied MockNotificationRulesProvider from the original engine test module
    use crate::notifications::persistence_iface::NotificationRulesProvider; // For the trait
    use crate::notifications::rules_types::NotificationRuleSet; // For the type
    use crate::notifications::rules_errors::NotificationRulesError; // For the error
    use mockall::mock;
    use std::sync::Arc;

    mock! {
        pub NotificationRulesProvider {}
        #[async_trait]
        impl NotificationRulesProvider for NotificationRulesProvider {
            async fn load_rules(&self) -> Result<NotificationRuleSet, NotificationRulesError>;
            async fn save_rules(&self, rules: &NotificationRuleSet) -> Result<(), NotificationRulesError>;
        }
    }
    // End of copied MockNotificationRulesProvider

    // TestMockGlobalSettingsService is defined below, no direct 'use' for its struct needed here,
    // but its methods use GlobalSettingsService, SettingPath, GlobalSettingsError from global_settings
    use crate::global_settings::{GlobalSettingsService, paths::SettingPath, errors::GlobalSettingsError, types::GlobalDesktopSettings, events::{SettingChangedEvent, SettingsLoadedEvent, SettingsSavedEvent}}; // Added more specific types

    // MockGlobalSettingsService for testing
    #[derive(Debug)]
    pub struct TestMockGlobalSettingsService { settings: RwLock<HashMap<String, serde_json::Value>> }
    impl TestMockGlobalSettingsService {
        pub fn new() -> Self { Self { settings: RwLock::new(HashMap::new()) } }
        #[allow(dead_code)]
        pub async fn set_json_setting(&self, path_str: &str, value: serde_json::Value) { self.settings.write().await.insert(path_str.to_string(), value); }
    }
    #[async_trait]
    impl GlobalSettingsService for TestMockGlobalSettingsService {
        async fn load_settings(&self) -> Result<(), GlobalSettingsError> { Ok(()) }
        async fn save_settings(&self) -> Result<(), GlobalSettingsError> { Ok(()) }
        fn get_current_settings(&self) -> crate::global_settings::types::GlobalDesktopSettings { unimplemented!() }
        async fn update_setting(&self, _path: SettingPath, _value: serde_json::Value) -> Result<(), GlobalSettingsError> { Ok(()) }
        // Corrected to be async as per typical GlobalSettingsService trait
        async fn get_setting(&self, path: &SettingPath) -> Result<serde_json::Value, GlobalSettingsError> {
            self.settings.read().await.get(&path.to_string()).cloned().ok_or_else(|| GlobalSettingsError::PathNotFound { path: path.clone() })
        }
        async fn reset_to_defaults(&self) -> Result<(), GlobalSettingsError> { Ok(()) }
        fn subscribe_to_setting_changes(&self) -> broadcast::Receiver<crate::global_settings::events::SettingChangedEvent> { broadcast::channel(1).1 }
        fn subscribe_to_settings_loaded(&self) -> broadcast::Receiver<crate::global_settings::events::SettingsLoadedEvent> { broadcast::channel(1).1 }
        fn subscribe_to_settings_saved(&self) -> broadcast::Receiver<crate::global_settings::events::SettingsSavedEvent> { broadcast::channel(1).1 }
    }
    // Unsafe helper for SettingPath in tests, to be replaced by proper SettingPath variants for these settings if defined.
    // This is problematic as SettingPath does not have a simple from_str.
    // For testing, we might need to define specific paths or use a simpler mock.
    // For now, assuming SettingPath::Root or similar can be used, or specific paths are defined elsewhere.
    // Let's assume this unsafe helper is only for specific test cases and we'll fix paths if they cause issues.
    // impl SettingPath { fn from_str_unsafe_for_testing(_s: &str) -> Self { SettingPath::Root } }


    #[tokio::test]
    async fn test_engine_new_and_initial_load_sorts_rules() {
        let mock_rules_provider = Arc::new(MockNotificationRulesProvider::new());
        let mock_settings_service = Arc::new(TestMockGlobalSettingsService::new());
        let r_low = NotificationRule { name: "LowPrio".to_string(), priority: 5, ..Default::default() };
        let r_high = NotificationRule { name: "HighPrio".to_string(), priority: 10, ..Default::default() };
        let r_mid_a = NotificationRule { name: "MidPrioAlpha".to_string(), priority: 7, ..Default::default() };
        let r_mid_z = NotificationRule { name: "MidPrioZeta".to_string(), priority: 7, ..Default::default() };
        mock_rules_provider.expect_load_rules().times(1).returning(move || Ok(vec![r_low.clone(), r_mid_z.clone(), r_high.clone(), r_mid_a.clone()]));
        let engine = DefaultNotificationRulesEngine::new(mock_rules_provider, mock_settings_service).await.unwrap();
        let rules = engine.get_rules().await.unwrap();
        assert_eq!(rules.len(), 4);
        assert_eq!(rules[0].name, "HighPrio"); assert_eq!(rules[1].name, "MidPrioAlpha");
        assert_eq!(rules[2].name, "MidPrioZeta"); assert_eq!(rules[3].name, "LowPrio");
    }

    #[tokio::test]
    async fn test_process_notification_no_rules_match() {
        let mock_rules_provider = Arc::new(MockNotificationRulesProvider::new());
        let mock_settings_service = Arc::new(TestMockGlobalSettingsService::new());
        mock_rules_provider.expect_load_rules().times(1).returning(|| Ok(vec![]));
        let engine = DefaultNotificationRulesEngine::new(mock_rules_provider, mock_settings_service).await.unwrap();
        let notif = Notification::new("TestApp".into(), "Summary".into(), Default::default());
        let result = engine.process_notification(notif.clone()).await.unwrap();
        assert_eq!(result, RuleProcessingResult::Allow(notif));
    }

    #[tokio::test]
    async fn test_process_notification_simple_match_suppress() {
        let mock_rules_provider = Arc::new(MockNotificationRulesProvider::new());
        let mock_settings_service = Arc::new(TestMockGlobalSettingsService::new());
        let rule_id = Uuid::new_v4();
        let rules = vec![NotificationRule {id: rule_id, name: "SuppressTest".into(), is_enabled: true, condition: RuleCondition::Simple(SimpleRuleCondition { field: RuleConditionField::ApplicationName, operator: RuleConditionOperator::Is, value: RuleConditionValue::String("TestApp".into())}), actions: vec![RuleAction::SuppressNotification], ..Default::default()}];
        mock_rules_provider.expect_load_rules().times(1).returning(move || Ok(rules.clone()));
        let engine = DefaultNotificationRulesEngine::new(mock_rules_provider, mock_settings_service).await.unwrap();
        let notif = Notification::new("TestApp".into(), "Summary".into(), Default::default());
        let result = engine.process_notification(notif).await.unwrap();
        assert_eq!(result, RuleProcessingResult::Suppress { rule_id });
    }

    #[tokio::test]
    async fn test_process_notification_setting_is_true_condition() {
        let mock_rules_provider = Arc::new(MockNotificationRulesProvider::new());
        let settings_service = Arc::new(TestMockGlobalSettingsService::new());
        let setting_path_str = "dnd.is_enabled"; // Example path
        // For SettingPath::from_str_unsafe_for_testing to work, or use a real SettingPath variant
        let actual_setting_path = SettingPath::Root; // Placeholder - this test needs a valid SettingPath construction
        settings_service.set_json_setting(&actual_setting_path.to_string(), json!(true)).await; // Use path's string rep if keys are strings

        let rule_id = Uuid::new_v4();
        let rules = vec![NotificationRule {id: rule_id, name: "SettingTrueTest".into(), is_enabled: true, condition: RuleCondition::SettingIsTrue(actual_setting_path.clone()), actions: vec![RuleAction::SuppressNotification], ..Default::default()}];
        mock_rules_provider.expect_load_rules().times(1).returning(move || Ok(rules.clone()));
        let engine = DefaultNotificationRulesEngine::new(mock_rules_provider, settings_service).await.unwrap();
        let notif = Notification::new("AnyApp".into(), "Summary".into(), Default::default());
        let result = engine.process_notification(notif).await.unwrap();
        assert_eq!(result, RuleProcessingResult::Suppress { rule_id });
    }

    #[tokio::test]
    async fn test_process_notification_regex_match_modify_urgency() {
        let mock_rules_provider = Arc::new(MockNotificationRulesProvider::new());
        let mock_settings_service = Arc::new(TestMockGlobalSettingsService::new());
        let rule_id = Uuid::new_v4();
        let rules = vec![NotificationRule {id: rule_id, name: "RegexUrgency".into(), is_enabled: true, condition: RuleCondition::Simple(SimpleRuleCondition { field: RuleConditionField::Summary, operator: RuleConditionOperator::MatchesRegex, value: RuleConditionValue::Regex("(?i)urgent".into())}), actions: vec![RuleAction::SetUrgency(NotificationUrgency::Critical)], ..Default::default()}];
        mock_rules_provider.expect_load_rules().times(1).returning(move || Ok(rules.clone()));
        let engine = DefaultNotificationRulesEngine::new(mock_rules_provider, mock_settings_service).await.unwrap();
        let notif = Notification::new("TestApp".into(), "This is an URGENT message".into(), NotificationUrgency::Normal);
        let original_notif_clone = notif.clone();
        let result = engine.process_notification(notif).await.unwrap();
        match result { RuleProcessingResult::Allow(modified_notif) => { assert_eq!(modified_notif.urgency, NotificationUrgency::Critical); assert_eq!(modified_notif.summary, original_notif_clone.summary); } _ => panic!("Expected Allow"), }
    }
}
