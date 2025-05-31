use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use uuid::Uuid;

use crate::notifications::types::{Notification, NotificationFilterCriteria, NotificationSortOrder};
use crate::notifications::errors::NotificationError;
use crate::notifications::persistence_iface::{NotificationPersistence, NotificationRepository};

#[derive(Default)]
pub struct InMemoryNotificationPersistence {
    active_notifications: Arc<RwLock<VecDeque<Notification>>>,
}

impl InMemoryNotificationPersistence {
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait]
impl NotificationPersistence for InMemoryNotificationPersistence {
    async fn save_active_notification(&self, notification: &Notification) -> Result<(), NotificationError> {
        let mut notifications = self.active_notifications.write()
            .map_err(|e| NotificationError::InternalError(format!("Failed to acquire write lock for active_notifications: {}", e)))?;
        notifications.push_back(notification.clone());
        Ok(())
    }

    async fn update_active_notification(&self, notification: &Notification) -> Result<(), NotificationError> {
        let mut notifications = self.active_notifications.write()
            .map_err(|e| NotificationError::InternalError(format!("Failed to acquire write lock for active_notifications: {}", e)))?;
        if let Some(index) = notifications.iter().position(|n| n.id == notification.id) {
            notifications[index] = notification.clone();
            Ok(())
        } else {
            Err(NotificationError::NotFound(notification.id))
        }
    }

    async fn delete_active_notification(&self, notification_id: Uuid) -> Result<(), NotificationError> {
        let mut notifications = self.active_notifications.write()
            .map_err(|e| NotificationError::InternalError(format!("Failed to acquire write lock for active_notifications: {}", e)))?;
        if let Some(index) = notifications.iter().position(|n| n.id == notification_id) {
            notifications.remove(index);
            Ok(())
        } else {
            Err(NotificationError::NotFound(notification_id))
        }
    }

    async fn load_all_active_notifications(&self) -> Result<Vec<Notification>, NotificationError> {
        let notifications = self.active_notifications.read()
            .map_err(|e| NotificationError::InternalError(format!("Failed to acquire read lock for active_notifications: {}", e)))?;
        Ok(notifications.iter().cloned().collect())
    }
}

#[async_trait]
impl NotificationRepository for InMemoryNotificationPersistence {
    async fn get_active_notification_by_id(&self, id: Uuid) -> Result<Option<Notification>, NotificationError> {
        let notifications = self.active_notifications.read()
            .map_err(|e| NotificationError::InternalError(format!("Failed to acquire read lock for active_notifications: {}", e)))?;
        Ok(notifications.iter().find(|n| n.id == id).cloned())
    }

    async fn get_all_active_notifications(
        &self,
        _filter: Option<&NotificationFilterCriteria>, // Filter logic to be implemented in service layer or later
        _sort_order: Option<NotificationSortOrder>    // Sort logic to be implemented in service layer or later
    ) -> Result<VecDeque<Notification>, NotificationError> {
        let notifications = self.active_notifications.read()
            .map_err(|e| NotificationError::InternalError(format!("Failed to acquire read lock for active_notifications: {}", e)))?;
        Ok(notifications.clone()) // Return a clone of the VecDeque
    }
}

// --- FilesystemNotificationHistoryProvider ---

// Original imports from user_centric_services/notifications_core/persistence.rs, adapted:
// use std::collections::VecDeque; // Already imported above for InMemoryNotificationPersistence
// use std::sync::Arc; // Already imported above for InMemoryNotificationPersistence
// use async_trait::async_trait; // Already imported above
use tracing::{debug, info, warn, error}; // Changed from log to tracing
use std::sync::Arc; // Ensure Arc is in scope for ConfigServiceAsync

use crate::notifications::types::Notification; // Was super::types::Notification
use crate::notifications::errors::NotificationError; // Was super::errors::NotificationError
use crate::notifications::persistence_iface::NotificationHistoryProvider; // Was super::persistence_iface::NotificationHistoryProvider

use novade_core::config::ConfigServiceAsync; // Was crate::ports::config_service::ConfigServiceAsync
use novade_core::errors::CoreError;       // Was novade_core::CoreError (assuming it's in errors module)
use novade_core::ConfigError; // Assuming ConfigError is part of novade_core and used by CoreError::Config

pub struct FilesystemNotificationHistoryProvider {
    pub config_service: Arc<dyn ConfigServiceAsync>,
    pub history_config_key: String,
}

impl FilesystemNotificationHistoryProvider {
    pub fn new(config_service: Arc<dyn ConfigServiceAsync>, history_config_key: String) -> Self {
        Self {
            config_service,
            history_config_key,
        }
    }
}

#[async_trait]
impl NotificationHistoryProvider for FilesystemNotificationHistoryProvider {
    async fn load_history(&self) -> Result<VecDeque<Notification>, NotificationError> {
        debug!("Loading notification history from key '{}'", self.history_config_key);
        match self.config_service.read_config_file_string(&self.history_config_key).await {
            Ok(toml_string) => {
                toml::from_str(&toml_string).map_err(|e| {
                    error!("Failed to deserialize TOML notification history from key '{}': {}", self.history_config_key, e);
                    NotificationError::InternalError(format!("History deserialization failed: {}", e))
                })
            }
            Err(core_error) => {
                // Adapt is_not_found_error check
                match core_error {
                    CoreError::Config(ConfigError::NotFound { .. }) | CoreError::Io(ref io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
                        info!("Notification history file (key '{}') not found. Returning empty history.", self.history_config_key);
                        Ok(VecDeque::new())
                    }
                    _ => {
                        error!("CoreError loading notification history (key '{}'): {}", self.history_config_key, core_error);
                        Err(NotificationError::PersistenceError{
                            operation: "load_history".to_string(),
                            source_message: "Failed to read history file".to_string(),
                            source: Some(core_error),
                        })
                    }
                }
            }
        }
    }

    async fn save_history(&self, history: &VecDeque<Notification>) -> Result<(), NotificationError> {
        debug!("Saving {} notification history items to key '{}'", history.len(), self.history_config_key);
        let toml_string = toml::to_string_pretty(history).map_err(|e| {
            error!("Failed to serialize notification history to TOML for key '{}': {}", self.history_config_key, e);
            NotificationError::InternalError(format!("History serialization failed: {}", e))
        })?;

        self.config_service.write_config_file_string(&self.history_config_key, toml_string).await
            .map_err(|core_error| {
                 NotificationError::PersistenceError{
                    operation: "save_history".to_string(),
                    source_message: "Failed to write history file".to_string(),
                    source: Some(core_error),
                }
            })?;
        info!("Notification history saved successfully to key '{}'", self.history_config_key);
        Ok(())
    }
}
