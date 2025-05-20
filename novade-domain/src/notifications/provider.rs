//! Notification provider module for the NovaDE domain layer.
//!
//! This module provides interfaces and implementations for sending
//! notifications to the system or other notification services.

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use crate::error::{DomainResult, NotificationError};
use crate::notifications::core::{Notification, NotificationId};

/// Interface for providing notification services.
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    /// Sends a notification.
    ///
    /// # Arguments
    ///
    /// * `notification` - The notification to send
    ///
    /// # Returns
    ///
    /// `Ok(())` if the notification was sent, or an error if sending failed.
    async fn send_notification(&self, notification: &Notification) -> DomainResult<()>;
    
    /// Updates a notification.
    ///
    /// # Arguments
    ///
    /// * `notification` - The updated notification
    ///
    /// # Returns
    ///
    /// `Ok(())` if the notification was updated, or an error if updating failed.
    async fn update_notification(&self, notification: &Notification) -> DomainResult<()>;
    
    /// Dismisses a notification.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The ID of the notification to dismiss
    ///
    /// # Returns
    ///
    /// `Ok(())` if the notification was dismissed, or an error if dismissal failed.
    async fn dismiss_notification(&self, notification_id: NotificationId) -> DomainResult<()>;
    
    /// Performs an action on a notification.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The ID of the notification
    /// * `action_id` - The ID of the action to perform
    ///
    /// # Returns
    ///
    /// `Ok(())` if the action was performed, or an error if it failed.
    async fn perform_action(&self, notification_id: NotificationId, action_id: &str) -> DomainResult<()>;
}

/// System notification provider that integrates with the operating system's
/// notification system.
pub struct SystemNotificationProvider {
    /// The application name to use for notifications.
    app_name: String,
    /// The active notifications, keyed by ID.
    active_notifications: Arc<Mutex<Vec<(NotificationId, u32)>>>,
}

impl SystemNotificationProvider {
    /// Creates a new system notification provider.
    ///
    /// # Arguments
    ///
    /// * `app_name` - The application name to use for notifications
    ///
    /// # Returns
    ///
    /// A new `SystemNotificationProvider`.
    pub fn new(app_name: impl Into<String>) -> Self {
        SystemNotificationProvider {
            app_name: app_name.into(),
            active_notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Gets the system notification ID for a notification.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The ID of the notification
    ///
    /// # Returns
    ///
    /// The system notification ID, or `None` if the notification is not active.
    fn get_system_id(&self, notification_id: NotificationId) -> Option<u32> {
        let active_notifications = self.active_notifications.lock().unwrap();
        active_notifications
            .iter()
            .find(|(id, _)| *id == notification_id)
            .map(|(_, system_id)| *system_id)
    }
    
    /// Adds a system notification ID for a notification.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The ID of the notification
    /// * `system_id` - The system notification ID
    fn add_system_id(&self, notification_id: NotificationId, system_id: u32) {
        let mut active_notifications = self.active_notifications.lock().unwrap();
        active_notifications.push((notification_id, system_id));
    }
    
    /// Removes a system notification ID for a notification.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The ID of the notification
    fn remove_system_id(&self, notification_id: NotificationId) {
        let mut active_notifications = self.active_notifications.lock().unwrap();
        active_notifications.retain(|(id, _)| *id != notification_id);
    }
}

#[async_trait]
impl NotificationProvider for SystemNotificationProvider {
    async fn send_notification(&self, notification: &Notification) -> DomainResult<()> {
        // In a real implementation, this would use the system's notification API
        // For now, we'll simulate it with a placeholder implementation
        
        // Generate a fake system notification ID
        let system_id = rand::random::<u32>();
        
        // Store the mapping between our notification ID and the system ID
        self.add_system_id(notification.id(), system_id);
        
        // Log the notification (in a real implementation, this would send to the system)
        println!(
            "System Notification [{}]: {} - {} ({})",
            system_id,
            notification.title(),
            notification.body(),
            notification.priority()
        );
        
        Ok(())
    }
    
    async fn update_notification(&self, notification: &Notification) -> DomainResult<()> {
        // Get the system notification ID
        let system_id = self.get_system_id(notification.id())
            .ok_or_else(|| NotificationError::NotFound(notification.id().to_string()))?;
        
        // In a real implementation, this would update the system notification
        println!(
            "Update System Notification [{}]: {} - {} ({})",
            system_id,
            notification.title(),
            notification.body(),
            notification.priority()
        );
        
        Ok(())
    }
    
    async fn dismiss_notification(&self, notification_id: NotificationId) -> DomainResult<()> {
        // Get the system notification ID
        let system_id = self.get_system_id(notification_id)
            .ok_or_else(|| NotificationError::NotFound(notification_id.to_string()))?;
        
        // In a real implementation, this would dismiss the system notification
        println!("Dismiss System Notification [{}]", system_id);
        
        // Remove the mapping
        self.remove_system_id(notification_id);
        
        Ok(())
    }
    
    async fn perform_action(&self, notification_id: NotificationId, action_id: &str) -> DomainResult<()> {
        // Get the system notification ID
        let system_id = self.get_system_id(notification_id)
            .ok_or_else(|| NotificationError::NotFound(notification_id.to_string()))?;
        
        // In a real implementation, this would perform the action on the system notification
        println!(
            "Perform Action [{}] on System Notification [{}]",
            action_id,
            system_id
        );
        
        Ok(())
    }
}

/// In-memory notification provider for testing.
pub struct InMemoryNotificationProvider {
    /// The sent notifications.
    sent: Arc<Mutex<Vec<Notification>>>,
    /// The dismissed notification IDs.
    dismissed: Arc<Mutex<Vec<NotificationId>>>,
    /// The performed actions.
    actions: Arc<Mutex<Vec<(NotificationId, String)>>>,
}

impl InMemoryNotificationProvider {
    /// Creates a new in-memory notification provider.
    ///
    /// # Returns
    ///
    /// A new `InMemoryNotificationProvider`.
    pub fn new() -> Self {
        InMemoryNotificationProvider {
            sent: Arc::new(Mutex::new(Vec::new())),
            dismissed: Arc::new(Mutex::new(Vec::new())),
            actions: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Gets the sent notifications.
    ///
    /// # Returns
    ///
    /// The sent notifications.
    pub fn get_sent(&self) -> Vec<Notification> {
        let sent = self.sent.lock().unwrap();
        sent.clone()
    }
    
    /// Gets the dismissed notification IDs.
    ///
    /// # Returns
    ///
    /// The dismissed notification IDs.
    pub fn get_dismissed(&self) -> Vec<NotificationId> {
        let dismissed = self.dismissed.lock().unwrap();
        dismissed.clone()
    }
    
    /// Gets the performed actions.
    ///
    /// # Returns
    ///
    /// The performed actions as (notification_id, action_id) pairs.
    pub fn get_actions(&self) -> Vec<(NotificationId, String)> {
        let actions = self.actions.lock().unwrap();
        actions.clone()
    }
}

#[async_trait]
impl NotificationProvider for InMemoryNotificationProvider {
    async fn send_notification(&self, notification: &Notification) -> DomainResult<()> {
        let mut sent = self.sent.lock().unwrap();
        sent.push(notification.clone());
        Ok(())
    }
    
    async fn update_notification(&self, notification: &Notification) -> DomainResult<()> {
        let mut sent = self.sent.lock().unwrap();
        
        if let Some(index) = sent.iter().position(|n| n.id() == notification.id()) {
            sent[index] = notification.clone();
            Ok(())
        } else {
            Err(NotificationError::NotFound(notification.id().to_string()).into())
        }
    }
    
    async fn dismiss_notification(&self, notification_id: NotificationId) -> DomainResult<()> {
        let mut dismissed = self.dismissed.lock().unwrap();
        dismissed.push(notification_id);
        Ok(())
    }
    
    async fn perform_action(&self, notification_id: NotificationId, action_id: &str) -> DomainResult<()> {
        let mut actions = self.actions.lock().unwrap();
        actions.push((notification_id, action_id.to_string()));
        Ok(())
    }
}

impl Default for InMemoryNotificationProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_in_memory_provider() {
        let provider = InMemoryNotificationProvider::new();
        
        let notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        let notification_id = notification.id();
        
        // Send notification
        provider.send_notification(&notification).await.unwrap();
        
        let sent = provider.get_sent();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].id(), notification_id);
        
        // Update notification
        let mut updated = notification.clone();
        updated.set_title("Updated Title");
        
        provider.update_notification(&updated).await.unwrap();
        
        let sent = provider.get_sent();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].title(), "Updated Title");
        
        // Dismiss notification
        provider.dismiss_notification(notification_id).await.unwrap();
        
        let dismissed = provider.get_dismissed();
        assert_eq!(dismissed.len(), 1);
        assert_eq!(dismissed[0], notification_id);
        
        // Perform action
        provider.perform_action(notification_id, "open").await.unwrap();
        
        let actions = provider.get_actions();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, notification_id);
        assert_eq!(actions[0].1, "open");
    }
}
