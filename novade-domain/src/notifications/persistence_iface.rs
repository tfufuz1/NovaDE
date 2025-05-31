use async_trait::async_trait;
use uuid::Uuid;
use std::collections::VecDeque; // For NotificationHistoryProvider and NotificationRepository

// Types and Errors from this crate
use crate::notifications::types::{Notification, NotificationFilterCriteria, NotificationSortOrder};
// NotificationRuleSet is not directly used in the provided snippets for NotificationHistoryProvider,
// NotificationPersistence, or NotificationRepository. It IS used by NotificationRulesProvider.
use crate::notifications::rules_types::NotificationRuleSet;
use crate::notifications::errors::NotificationError;
use crate::notifications::rules_errors::NotificationRulesError;

#[async_trait]
pub trait NotificationHistoryProvider: Send + Sync {
    async fn load_history(&self) -> Result<VecDeque<Notification>, NotificationError>;
    async fn save_history(&self, history: &VecDeque<Notification>) -> Result<(), NotificationError>;
}

#[async_trait]
pub trait NotificationRulesProvider: Send + Sync {
    async fn load_rules(&self) -> Result<NotificationRuleSet, NotificationRulesError>;
    async fn save_rules(&self, rules: &NotificationRuleSet) -> Result<(), NotificationRulesError>;
}

#[async_trait]
pub trait NotificationPersistence: Send + Sync {
    async fn save_active_notification(&self, notification: &Notification) -> Result<(), NotificationError>;
    async fn update_active_notification(&self, notification: &Notification) -> Result<(), NotificationError>;
    async fn delete_active_notification(&self, notification_id: Uuid) -> Result<(), NotificationError>;
    async fn load_all_active_notifications(&self) -> Result<Vec<Notification>, NotificationError>;
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn get_active_notification_by_id(&self, id: Uuid) -> Result<Option<Notification>, NotificationError>;
    async fn get_all_active_notifications(
        &self,
        filter: Option<&NotificationFilterCriteria>,
        sort_order: Option<NotificationSortOrder>
    ) -> Result<VecDeque<Notification>, NotificationError>;
}
