use thiserror::Error;
use uuid::Uuid;
use novade_core::errors::CoreError; // Assuming this path
use crate::global_settings::GlobalSettingsError; // Corrected path

#[derive(Error, Debug)]
pub enum NotificationRulesError {
    #[error("Invalid rule definition for rule '{rule_name}' (ID: {rule_id:?}): {reason}")]
    InvalidRuleDefinition {
        rule_id: Option<Uuid>,
        rule_name: String,
        reason: String,
    },

    #[error("Error evaluating condition for rule '{rule_name}' (ID: {rule_id}): {details}")]
    ConditionEvaluationError {
        rule_id: Uuid,
        rule_name: String,
        details: String,
    },

    #[error("Error applying action for rule '{rule_name}' (ID: {rule_id}): {details}")]
    ActionApplicationError {
        rule_id: Uuid,
        rule_name: String,
        details: String,
    },

    #[error("Rule persistence error: {0}")]
    RulePersistenceError(#[from] CoreError), // For errors from ConfigServiceAsync via provider

    #[error("Error accessing global settings for rule evaluation: {0}")]
    SettingsAccessError(#[from] GlobalSettingsError),

    #[error("Internal notification rules engine error: {0}")]
    InternalError(String),
}
