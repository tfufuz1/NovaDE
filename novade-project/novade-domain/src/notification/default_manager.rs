//! Default implementation of the notification manager.
//!
//! This module provides a default implementation of the notification manager
//! for the NovaDE desktop environment.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use crate::error::{DomainError, NotificationError};
use crate::entities::value_objects::Timestamp;
use super::{NotificationManager, Notification, NotificationCategory, NotificationUrgency, NotificationAction};

/// Default implementation of the notification manager.
pub struct DefaultNotificationManager {
    notifications: HashMap<String, Notification>,
}

impl DefaultNotificationManager {
    /// Creates a new default notification manager.
    pub fn new() -> Self {
        Self {
            notifications: HashMap::new(),
        }
    }
    
    /// Removes expired notifications.
    pub fn cleanup_expired(&mut self) {
        let now = Timestamp::now();
        self.notifications.retain(|_, notification| {
            if let Some(expires_at) = notification.expires_at {
                expires_at.datetime() > now.datetime()
            } else {
                true
            }
        });
    }
}

#[async_trait]
impl NotificationManager for DefaultNotificationManager {
    async fn create_notification(
        &self,
        title: &str,
        body: &str,
        category: NotificationCategory,
        urgency: NotificationUrgency,
        icon: Option<&str>,
        dismissible: bool,
        actions: Vec<NotificationAction>,
        expires_at: Option<Timestamp>,
    ) -> Result<String, DomainError> {
        let notification_id = Uuid::new_v4().to_string();
        
        let notification = Notification {
            notification_id: notification_id.clone(),
            title: title.to_string(),
            body: body.to_string(),
            icon: icon.map(|s| s.to_string()),
            category,
            urgency,
            read: false,
            dismissible,
            actions,
            created_at: Timestamp::now(),
            expires_at,
        };
        
        let mut notifications = self.notifications.clone();
        notifications.insert(notification_id.clone(), notification);
        
        // Update self
        *self = Self {
            notifications,
        };
        
        // Clean up expired notifications
        self.cleanup_expired();
        
        Ok(notification_id)
    }
    
    async fn get_notification(&self, notification_id: &str) -> Result<Notification, DomainError> {
        self.notifications.get(notification_id)
            .cloned()
            .ok_or_else(|| NotificationError::NotificationNotFound(notification_id.to_string()).into())
    }
    
    async fn list_notifications(&self) -> Result<Vec<Notification>, DomainError> {
        Ok(self.notifications.values().cloned().collect())
    }
    
    async fn list_notifications_by_category(&self, category: &NotificationCategory) -> Result<Vec<Notification>, DomainError> {
        Ok(self.notifications.values()
            .filter(|n| &n.category == category)
            .cloned()
            .collect())
    }
    
    async fn list_unread_notifications(&self) -> Result<Vec<Notification>, DomainError> {
        Ok(self.notifications.values()
            .filter(|n| !n.read)
            .cloned()
            .collect())
    }
    
    async fn mark_as_read(&self, notification_id: &str) -> Result<(), DomainError> {
        if !self.notifications.contains_key(notification_id) {
            return Err(NotificationError::NotificationNotFound(notification_id.to_string()).into());
        }
        
        let mut notifications = self.notifications.clone();
        
        if let Some(notification) = notifications.get_mut(notification_id) {
            notification.read = true;
        }
        
        // Update self
        *self = Self {
            notifications,
        };
        
        Ok(())
    }
    
    async fn mark_all_as_read(&self) -> Result<(), DomainError> {
        let mut notifications = self.notifications.clone();
        
        for notification in notifications.values_mut() {
            notification.read = true;
        }
        
        // Update self
        *self = Self {
            notifications,
        };
        
        Ok(())
    }
    
    async fn dismiss_notification(&self, notification_id: &str) -> Result<(), DomainError> {
        let notification = self.notifications.get(notification_id)
            .ok_or_else(|| NotificationError::NotificationNotFound(notification_id.to_string()))?;
        
        if !notification.dismissible {
            return Err(NotificationError::NotDismissible(notification_id.to_string()).into());
        }
        
        let mut notifications = self.notifications.clone();
        notifications.remove(notification_id);
        
        // Update self
        *self = Self {
            notifications,
        };
        
        Ok(())
    }
    
    async fn dismiss_all_notifications(&self) -> Result<(), DomainError> {
        let mut notifications = self.notifications.clone();
        
        // Only remove dismissible notifications
        notifications.retain(|_, notification| !notification.dismissible);
        
        // Update self
        *self = Self {
            notifications,
        };
        
        Ok(())
    }
    
    async fn execute_action(&self, notification_id: &str, action_id: &str) -> Result<(), DomainError> {
        let notification = self.notifications.get(notification_id)
            .ok_or_else(|| NotificationError::NotificationNotFound(notification_id.to_string()))?;
        
        // Check if the action exists
        if !notification.actions.iter().any(|a| a.id == action_id) {
            return Err(NotificationError::ActionNotFound {
                notification_id: notification_id.to_string(),
                action_id: action_id.to_string(),
            }.into());
        }
        
        // In a real implementation, this would execute the action
        // For now, we just mark the notification as read
        self.mark_as_read(notification_id).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_notification() {
        let manager = DefaultNotificationManager::new();
        
        let notification_id = manager.create_notification(
            "Test Title",
            "Test Body",
            NotificationCategory::System,
            NotificationUrgency::Normal,
            None,
            true,
            vec![],
            None,
        ).await.unwrap();
        
        assert!(!notification_id.is_empty());
        
        let notification = manager.get_notification(&notification_id).await.unwrap();
        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.body, "Test Body");
        assert_eq!(notification.category, NotificationCategory::System);
        assert_eq!(notification.urgency, NotificationUrgency::Normal);
        assert_eq!(notification.read, false);
        assert_eq!(notification.dismissible, true);
        assert!(notification.actions.is_empty());
        assert_eq!(notification.expires_at, None);
    }
    
    #[tokio::test]
    async fn test_mark_as_read() {
        let manager = DefaultNotificationManager::new();
        
        let notification_id = manager.create_notification(
            "Test Title",
            "Test Body",
            NotificationCategory::System,
            NotificationUrgency::Normal,
            None,
            true,
            vec![],
            None,
        ).await.unwrap();
        
        manager.mark_as_read(&notification_id).await.unwrap();
        
        let notification = manager.get_notification(&notification_id).await.unwrap();
        assert_eq!(notification.read, true);
    }
    
    #[tokio::test]
    async fn test_dismiss_notification() {
        let manager = DefaultNotificationManager::new();
        
        let notification_id = manager.create_notification(
            "Test Title",
            "Test Body",
            NotificationCategory::System,
            NotificationUrgency::Normal,
            None,
            true,
            vec![],
            None,
        ).await.unwrap();
        
        manager.dismiss_notification(&notification_id).await.unwrap();
        
        let result = manager.get_notification(&notification_id).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_non_dismissible_notification() {
        let manager = DefaultNotificationManager::new();
        
        let notification_id = manager.create_notification(
            "Test Title",
            "Test Body",
            NotificationCategory::System,
            NotificationUrgency::Normal,
            None,
            false,
            vec![],
            None,
        ).await.unwrap();
        
        let result = manager.dismiss_notification(&notification_id).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_execute_action() {
        let manager = DefaultNotificationManager::new();
        
        let action = NotificationAction {
            id: "test_action".to_string(),
            label: "Test Action".to_string(),
            is_default: true,
        };
        
        let notification_id = manager.create_notification(
            "Test Title",
            "Test Body",
            NotificationCategory::System,
            NotificationUrgency::Normal,
            None,
            true,
            vec![action],
            None,
        ).await.unwrap();
        
        manager.execute_action(&notification_id, "test_action").await.unwrap();
        
        let notification = manager.get_notification(&notification_id).await.unwrap();
        assert_eq!(notification.read, true);
    }
    
    #[tokio::test]
    async fn test_expired_notifications() {
        let manager = DefaultNotificationManager::new();
        
        // Create a notification that expires in the past
        use chrono::Duration;
        let past = Timestamp::new(chrono::Utc::now() - Duration::hours(1));
        
        let notification_id = manager.create_notification(
            "Expired Notification",
            "This notification has expired",
            NotificationCategory::System,
            NotificationUrgency::Normal,
            None,
            true,
            vec![],
            Some(past),
        ).await.unwrap();
        
        // Cleanup should remove this notification
        manager.cleanup_expired();
        
        let result = manager.get_notification(&notification_id).await;
        assert!(result.is_err());
    }
}
