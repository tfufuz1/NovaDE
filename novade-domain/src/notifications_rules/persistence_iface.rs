use async_trait::async_trait;
use super::types::NotificationRuleSet;
use super::errors::NotificationRulesError;

#[async_trait]
pub trait NotificationRulesProvider: Send + Sync {
    async fn load_rules(&self) -> Result<NotificationRuleSet, NotificationRulesError>;
    async fn save_rules(&self, rules: &NotificationRuleSet) -> Result<(), NotificationRulesError>;
}
