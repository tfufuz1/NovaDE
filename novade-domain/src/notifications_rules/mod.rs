// Main module for notification rules engine, types, and persistence.

pub mod types;
pub mod errors;
pub mod persistence_iface; // Placeholder for persistence trait
pub mod persistence;       // Placeholder for persistence implementation
pub mod engine;            // Placeholder for the NotificationRulesEngine trait and its impl

// Re-exports for easier access by consumers of the crate.
// These will be populated as the types and service trait are defined.
// Example:
pub use types::{NotificationRule, RuleCondition, RuleAction, RuleConditionField, RuleConditionOperator, RuleConditionValue, SimpleRuleCondition, NotificationRuleSet}; // Added other types for completeness
pub use errors::NotificationRulesError;
pub use types::{NotificationRule, RuleCondition, RuleAction, RuleConditionField, RuleConditionOperator, RuleConditionValue, SimpleRuleCondition, NotificationRuleSet};
pub use errors::NotificationRulesError;
pub use persistence_iface::NotificationRulesProvider;
pub use persistence::FilesystemNotificationRulesProvider;
pub use engine::{NotificationRulesEngine, DefaultNotificationRulesEngine, RuleProcessingResult}; // Updated
