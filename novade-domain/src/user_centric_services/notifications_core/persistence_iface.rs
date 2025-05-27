use async_trait::async_trait;
use std::collections::VecDeque;
use super::types::Notification;
use super::errors::NotificationError;

#[async_trait]
pub trait NotificationHistoryProvider: Send + Sync {
    async fn load_history(&self) -> Result<VecDeque<Notification>, NotificationError>;
    async fn save_history(&self, history: &VecDeque<Notification>) -> Result<(), NotificationError>;
    // Optional: async fn clear_history_storage(&self) -> Result<(), NotificationError>;
}
