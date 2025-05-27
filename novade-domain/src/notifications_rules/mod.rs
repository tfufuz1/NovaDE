// Declare submodules
pub mod types;
pub mod errors;
pub mod persistence_iface;
pub mod engine;
pub mod persistence; // Added in Iteration 2

// Re-export main public types, traits, and errors
pub use self::types::{
    RuleConditionValue,
    RuleConditionOperator,
    RuleConditionField,
    SimpleRuleCondition,
    RuleCondition,
    RuleAction,
    NotificationRule,
    NotificationRuleSet,
};
pub use self::errors::NotificationRulesError;
pub use self::persistence_iface::NotificationRulesProvider;
pub use self::persistence::FilesystemNotificationRulesProvider; // Added in Iteration 2
pub use self::engine::{
    RuleProcessingResult,
    NotificationRulesEngine,
    DefaultNotificationRulesEngine,
};
