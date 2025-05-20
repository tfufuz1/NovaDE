//! Notification service module for the NovaDE domain layer.
//!
//! This module provides the notification service interface and implementation
//! for managing notifications in the NovaDE desktop environment.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc, Duration};
use crate::common_events::{DomainEvent, NotificationEvent};
use crate::error::{DomainResult, NotificationError};
use crate::notifications::core::{Notification, NotificationId, NotificationPriority, NotificationAction};
use crate::notifications::provider::NotificationProvider;

/// Interface for the notification service.
#[async_trait]
pub trait NotificationService: Send + Sync {
    /// Creates a new notification.
    ///
    /// # Arguments
    ///
    /// * `notification` - The notification to create
    ///
    /// # Returns
    ///
    /// The created notification.
    async fn create_notification(&self, notification: Notification) -> DomainResult<Notification>;

    /// Gets a notification by ID.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The ID of the notification
    ///
    /// # Returns
    ///
    /// The notification, or an error if it doesn't exist.
    async fn get_notification(&self, notification_id: NotificationId) -> DomainResult<Notification>;

    /// Gets all notifications.
    ///
    /// # Returns
    ///
    /// A vector of all notifications.
    async fn get_all_notifications(&self) -> DomainResult<Vec<Notification>>;

    /// Gets notifications by source.
    ///
    /// # Arguments
    ///
    /// * `source` - The source of the notifications
    ///
    /// # Returns
    ///
    /// A vector of notifications from the specified source.
    async fn get_notifications_by_source(&self, source: &str) -> DomainResult<Vec<Notification>>;

    /// Gets notifications by priority.
    ///
    /// # Arguments
    ///
    /// * `priority` - The priority of the notifications
    ///
    /// # Returns
    ///
    /// A vector of notifications with the specified priority.
    async fn get_notifications_by_priority(&self, priority: NotificationPriority) -> DomainResult<Vec<Notification>>;

    /// Updates a notification.
    ///
    /// # Arguments
    ///
    /// * `notification` - The updated notification
    ///
    /// # Returns
    ///
    /// The updated notification.
    async fn update_notification(&self, notification: Notification) -> DomainResult<Notification>;

    /// Dismisses a notification.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The ID of the notification to dismiss
    ///
    /// # Returns
    ///
    /// `Ok(())` if the notification was dismissed, or an error if it doesn't exist.
    async fn dismiss_notification(&self, notification_id: NotificationId) -> DomainResult<()>;

    /// Dismisses all notifications.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all notifications were dismissed.
    async fn dismiss_all_notifications(&self) -> DomainResult<()>;

    /// Dismisses notifications by source.
    ///
    /// # Arguments
    ///
    /// * `source` - The source of the notifications to dismiss
    ///
    /// # Returns
    ///
    /// `Ok(())` if the notifications were dismissed.
    async fn dismiss_notifications_by_source(&self, source: &str) -> DomainResult<()>;

    /// Performs an action on a notification.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The ID of the notification
    /// * `action_id` - The ID of the action to perform
    ///
    /// # Returns
    ///
    /// `Ok(())` if the action was performed, or an error if the notification or action doesn't exist.
    async fn perform_action(&self, notification_id: NotificationId, action_id: &str) -> DomainResult<()>;

    /// Creates a simple notification.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the notification
    /// * `body` - The body of the notification
    /// * `source` - The source of the notification
    ///
    /// # Returns
    ///
    /// The created notification.
    async fn notify(&self, title: &str, body: &str, source: &str) -> DomainResult<Notification>;

    /// Creates a notification with a specified priority.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the notification
    /// * `body` - The body of the notification
    /// * `source` - The source of the notification
    /// * `priority` - The priority of the notification
    ///
    /// # Returns
    ///
    /// The created notification.
    async fn notify_with_priority(
        &self,
        title: &str,
        body: &str,
        source: &str,
        priority: NotificationPriority,
    ) -> DomainResult<Notification>;

    /// Creates a notification with actions.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the notification
    /// * `body` - The body of the notification
    /// * `source` - The source of the notification
    /// * `actions` - The actions for the notification
    ///
    /// # Returns
    ///
    /// The created notification.
    async fn notify_with_actions(
        &self,
        title: &str,
        body: &str,
        source: &str,
        actions: Vec<NotificationAction>,
    ) -> DomainResult<Notification>;
}

/// Default implementation of the notification service.
pub struct DefaultNotificationService {
    /// The notifications, keyed by ID.
    notifications: Arc<RwLock<HashMap<NotificationId, Notification>>>,
    /// The notification provider.
    provider: Arc<dyn NotificationProvider>,
    /// The event publisher function.
    event_publisher: Box<dyn Fn(DomainEvent<NotificationEvent>) + Send + Sync>,
}

impl DefaultNotificationService {
    /// Creates a new default notification service.
    ///
    /// # Arguments
    ///
    /// * `provider` - The notification provider
    /// * `event_publisher` - A function to publish notification events
    ///
    /// # Returns
    ///
    /// A new `DefaultNotificationService`.
    pub fn new<F>(
        provider: Arc<dyn NotificationProvider>,
        event_publisher: F,
    ) -> Self
    where
        F: Fn(DomainEvent<NotificationEvent>) + Send + Sync + 'static,
    {
        DefaultNotificationService {
            notifications: Arc::new(RwLock::new(HashMap::new())),
            provider,
            event_publisher: Box::new(event_publisher),
        }
    }

    /// Publishes a notification event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to publish
    fn publish_event(&self, event: NotificationEvent) {
        let domain_event = DomainEvent::new(event, "NotificationService");
        (self.event_publisher)(domain_event);
    }

    /// Removes expired notifications.
    ///
    /// # Returns
    ///
    /// The number of notifications removed.
    async fn remove_expired_notifications(&self) -> usize {
        let expired_ids = {
            let notifications = self.notifications.read().unwrap();
            notifications
                .values()
                .filter(|n| n.is_expired())
                .map(|n| n.id())
                .collect::<Vec<_>>()
        };

        let mut count = 0;
        for id in expired_ids {
            if self.dismiss_notification(id).await.is_ok() {
                count += 1;
            }
        }

        count
    }
}

#[async_trait]
impl NotificationService for DefaultNotificationService {
    async fn create_notification(&self, notification: Notification) -> DomainResult<Notification> {
        notification.validate()?;

        let notification_id = notification.id();
        let title = notification.title().to_string();
        let source = notification.source().to_string();

        {
            let mut notifications = self.notifications.write().unwrap();
            notifications.insert(notification_id, notification.clone());
        }

        // Send the notification to the provider
        self.provider.send_notification(&notification).await?;

        self.publish_event(NotificationEvent::NotificationCreated {
            notification_id,
            title,
            source,
        });

        Ok(notification)
    }

    async fn get_notification(&self, notification_id: NotificationId) -> DomainResult<Notification> {
        // Remove expired notifications
        self.remove_expired_notifications().await;

        let notifications = self.notifications.read().unwrap();

        notifications
            .get(&notification_id)
            .cloned()
            .ok_or_else(|| NotificationError::NotFound(notification_id.to_string()).into())
    }

    async fn get_all_notifications(&self) -> DomainResult<Vec<Notification>> {
        // Remove expired notifications
        self.remove_expired_notifications().await;

        let notifications = self.notifications.read().unwrap();

        let mut result: Vec<Notification> = notifications.values().cloned().collect();
        result.sort_by(|a, b| b.created_at().cmp(&a.created_at()));

        Ok(result)
    }

    async fn get_notifications_by_source(&self, source: &str) -> DomainResult<Vec<Notification>> {
        // Remove expired notifications
        self.remove_expired_notifications().await;

        let notifications = self.notifications.read().unwrap();

        let mut result: Vec<Notification> = notifications
            .values()
            .filter(|n| n.source() == source)
            .cloned()
            .collect();

        result.sort_by(|a, b| b.created_at().cmp(&a.created_at()));

        Ok(result)
    }

    async fn get_notifications_by_priority(&self, priority: NotificationPriority) -> DomainResult<Vec<Notification>> {
        // Remove expired notifications
        self.remove_expired_notifications().await;

        let notifications = self.notifications.read().unwrap();

        let mut result: Vec<Notification> = notifications
            .values()
            .filter(|n| n.priority() == priority)
            .cloned()
            .collect();

        result.sort_by(|a, b| b.created_at().cmp(&a.created_at()));

        Ok(result)
    }

    async fn update_notification(&self, notification: Notification) -> DomainResult<Notification> {
        notification.validate()?;

        let notification_id = notification.id();
        let title = notification.title().to_string();
        let source = notification.source().to_string();

        {
            let mut notifications = self.notifications.write().unwrap();

            if !notifications.contains_key(&notification_id) {
                return Err(NotificationError::NotFound(notification_id.to_string()).into());
            }

            notifications.insert(notification_id, notification.clone());
        }

        // Update the notification in the provider
        self.provider.update_notification(&notification).await?;

        self.publish_event(NotificationEvent::NotificationUpdated {
            notification_id,
            title,
            source,
        });

        Ok(notification)
    }

    async fn dismiss_notification(&self, notification_id: NotificationId) -> DomainResult<()> {
        let notification = {
            let mut notifications = self.notifications.write().unwrap();

            if !notifications.contains_key(&notification_id) {
                return Err(NotificationError::NotFound(notification_id.to_string()).into());
            }

            notifications.remove(&notification_id).unwrap()
        };

        // Dismiss the notification in the provider
        self.provider.dismiss_notification(notification_id).await?;

        self.publish_event(NotificationEvent::NotificationDismissed {
            notification_id,
            title: notification.title().to_string(),
            source: notification.source().to_string(),
        });

        Ok(())
    }

    async fn dismiss_all_notifications(&self) -> DomainResult<()> {
        let notification_ids = {
            let notifications = self.notifications.read().unwrap();
            notifications.keys().cloned().collect::<Vec<_>>()
        };

        for id in notification_ids {
            let _ = self.dismiss_notification(id).await;
        }

        self.publish_event(NotificationEvent::AllNotificationsDismissed);

        Ok(())
    }

    async fn dismiss_notifications_by_source(&self, source: &str) -> DomainResult<()> {
        let notification_ids = {
            let notifications = self.notifications.read().unwrap();
            notifications
                .values()
                .filter(|n| n.source() == source)
                .map(|n| n.id())
                .collect::<Vec<_>>()
        };

        for id in notification_ids {
            let _ = self.dismiss_notification(id).await;
        }

        self.publish_event(NotificationEvent::SourceNotificationsDismissed {
            source: source.to_string(),
        });

        Ok(())
    }

    async fn perform_action(&self, notification_id: NotificationId, action_id: &str) -> DomainResult<()> {
        let notification = self.get_notification(notification_id).await?;

        // Verify the action exists
        if !notification.actions().iter().any(|a| a.id == action_id) {
            return Err(NotificationError::ActionNotFound(action_id.to_string()).into());
        }

        // Perform the action in the provider
        self.provider
            .perform_action(notification_id, action_id)
            .await?;

        self.publish_event(NotificationEvent::NotificationActionPerformed {
            notification_id,
            action_id: action_id.to_string(),
            title: notification.title().to_string(),
            source: notification.source().to_string(),
        });

        Ok(())
    }

    async fn notify(&self, title: &str, body: &str, source: &str) -> DomainResult<Notification> {
        let notification = Notification::new(title, body, source);
        self.create_notification(notification).await
    }

    async fn notify_with_priority(
        &self,
        title: &str,
        body: &str,
        source: &str,
        priority: NotificationPriority,
    ) -> DomainResult<Notification> {
        let mut notification = Notification::new(title, body, source);
        notification.set_priority(priority);
        self.create_notification(notification).await
    }

    async fn notify_with_actions(
        &self,
        title: &str,
        body: &str,
        source: &str,
        actions: Vec<NotificationAction>,
    ) -> DomainResult<Notification> {
        let mut notification = Notification::new(title, body, source);
        for action in actions {
            notification.add_action(action);
        }
        self.create_notification(notification).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        NotificationProvider {}

        #[async_trait]
        impl NotificationProvider for NotificationProvider {
            async fn send_notification(&self, notification: &Notification) -> DomainResult<()>;
            async fn update_notification(&self, notification: &Notification) -> DomainResult<()>;
            async fn dismiss_notification(&self, notification_id: NotificationId) -> DomainResult<()>;
            async fn perform_action(&self, notification_id: NotificationId, action_id: &str) -> DomainResult<()>;
        }
    }

    struct TestContext {
        service: DefaultNotificationService,
        provider: Arc<MockNotificationProvider>,
        events: Arc<Mutex<Vec<NotificationEvent>>>,
    }

    impl TestContext {
        fn new() -> Self {
            let events = Arc::new(Mutex::new(Vec::new()));
            let events_clone = events.clone();

            let provider = Arc::new(MockNotificationProvider::new());

            let service = DefaultNotificationService::new(
                provider.clone(),
                move |event| {
                    let mut events = events_clone.lock().unwrap();
                    events.push(event.payload);
                },
            );

            TestContext {
                service,
                provider,
                events,
            }
        }

        fn get_events(&self) -> Vec<NotificationEvent> {
            let events = self.events.lock().unwrap();
            events.clone()
        }

        fn clear_events(&self) {
            let mut events = self.events.lock().unwrap();
            events.clear();
        }
    }

    #[tokio::test]
    async fn test_create_notification() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_send_notification()
            .returning(|_| Ok(()));

        let notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        let notification_id = notification.id();

        let created = ctx.service.create_notification(notification.clone()).await.unwrap();

        assert_eq!(created.id(), notification_id);

        let retrieved = ctx.service.get_notification(notification_id).await.unwrap();
        assert_eq!(retrieved.id(), notification_id);

        let events = ctx.get_events();
        assert_eq!(events.len(), 1);

        match &events[0] {
            NotificationEvent::NotificationCreated {
                notification_id: id,
                title,
                source,
            } => {
                assert_eq!(*id, notification_id);
                assert_eq!(title, "Test Notification");
                assert_eq!(source, "Test Source");
            },
            _ => panic!("Expected NotificationCreated event"),
        }
    }

    #[tokio::test]
    async fn test_update_notification() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_send_notification()
            .returning(|_| Ok(()));

        ctx.provider
            .expect_update_notification()
            .returning(|_| Ok(()));

        let mut notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        let notification_id = notification.id();

        ctx.service.create_notification(notification.clone()).await.unwrap();
        ctx.clear_events();

        notification.set_title("Updated Title");

        let updated = ctx.service.update_notification(notification.clone()).await.unwrap();

        assert_eq!(updated.title(), "Updated Title");

        let retrieved = ctx.service.get_notification(notification_id).await.unwrap();
        assert_eq!(retrieved.title(), "Updated Title");

        let events = ctx.get_events();
        assert_eq!(events.len(), 1);

        match &events[0] {
            NotificationEvent::NotificationUpdated {
                notification_id: id,
                title,
                source,
            } => {
                assert_eq!(*id, notification_id);
                assert_eq!(title, "Updated Title");
                assert_eq!(source, "Test Source");
            },
            _ => panic!("Expected NotificationUpdated event"),
        }
    }

    #[tokio::test]
    async fn test_dismiss_notification() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_send_notification()
            .returning(|_| Ok(()));

        ctx.provider
            .expect_dismiss_notification()
            .returning(|_| Ok(()));

        let notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        let notification_id = notification.id();

        ctx.service.create_notification(notification.clone()).await.unwrap();
        ctx.clear_events();

        ctx.service.dismiss_notification(notification_id).await.unwrap();

        let result = ctx.service.get_notification(notification_id).await;
        assert!(result.is_err());

        let events = ctx.get_events();
        assert_eq!(events.len(), 1);

        match &events[0] {
            NotificationEvent::NotificationDismissed {
                notification_id: id,
                title,
                source,
            } => {
                assert_eq!(*id, notification_id);
                assert_eq!(title, "Test Notification");
                assert_eq!(source, "Test Source");
            },
            _ => panic!("Expected NotificationDismissed event"),
        }
    }

    #[tokio::test]
    async fn test_dismiss_all_notifications() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_send_notification()
            .returning(|_| Ok(()));

        ctx.provider
            .expect_dismiss_notification()
            .returning(|_| Ok(()));

        let notification1 = Notification::new(
            "Test Notification 1",
            "This is a test notification",
            "Test Source",
        );

        let notification2 = Notification::new(
            "Test Notification 2",
            "This is another test notification",
            "Test Source",
        );

        ctx.service.create_notification(notification1.clone()).await.unwrap();
        ctx.service.create_notification(notification2.clone()).await.unwrap();
        ctx.clear_events();

        ctx.service.dismiss_all_notifications().await.unwrap();

        let notifications = ctx.service.get_all_notifications().await.unwrap();
        assert!(notifications.is_empty());

        let events = ctx.get_events();
        assert!(events.iter().any(|e| matches!(e, NotificationEvent::AllNotificationsDismissed)));
    }

    #[tokio::test]
    async fn test_dismiss_notifications_by_source() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_send_notification()
            .returning(|_| Ok(()));

        ctx.provider
            .expect_dismiss_notification()
            .returning(|_| Ok(()));

        let notification1 = Notification::new(
            "Test Notification 1",
            "This is a test notification",
            "Source A",
        );

        let notification2 = Notification::new(
            "Test Notification 2",
            "This is another test notification",
            "Source B",
        );

        ctx.service.create_notification(notification1.clone()).await.unwrap();
        ctx.service.create_notification(notification2.clone()).await.unwrap();
        ctx.clear_events();

        ctx.service.dismiss_notifications_by_source("Source A").await.unwrap();

        let notifications = ctx.service.get_all_notifications().await.unwrap();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].source(), "Source B");

        let events = ctx.get_events();
        assert!(events.iter().any(|e| matches!(e, NotificationEvent::SourceNotificationsDismissed { source } if source == "Source A")));
    }

    #[tokio::test]
    async fn test_perform_action() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_send_notification()
            .returning(|_| Ok(()));

        ctx.provider
            .expect_perform_action()
            .returning(|_, _| Ok(()));

        let mut notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        notification.add_action(NotificationAction::new("open", "Open"));
        let notification_id = notification.id();

        ctx.service.create_notification(notification.clone()).await.unwrap();
        ctx.clear_events();

        ctx.service.perform_action(notification_id, "open").await.unwrap();

        let events = ctx.get_events();
        assert_eq!(events.len(), 1);

        match &events[0] {
            NotificationEvent::NotificationActionPerformed {
                notification_id: id,
                action_id,
                title,
                source,
            } => {
                assert_eq!(*id, notification_id);
                assert_eq!(action_id, "open");
                assert_eq!(title, "Test Notification");
                assert_eq!(source, "Test Source");
            },
            _ => panic!("Expected NotificationActionPerformed event"),
        }
    }

    #[tokio::test]
    async fn test_notify_methods() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_send_notification()
            .returning(|_| Ok(()));

        // Test notify
        let notification1 = ctx.service.notify(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        ).await.unwrap();

        assert_eq!(notification1.title(), "Test Notification");
        assert_eq!(notification1.priority(), NotificationPriority::Normal);
        assert!(notification1.actions().is_empty());

        // Test notify_with_priority
        let notification2 = ctx.service.notify_with_priority(
            "Priority Notification",
            "This is a priority notification",
            "Test Source",
            NotificationPriority::High,
        ).await.unwrap();

        assert_eq!(notification2.title(), "Priority Notification");
        assert_eq!(notification2.priority(), NotificationPriority::High);

        // Test notify_with_actions
        let actions = vec![
            NotificationAction::new("open", "Open"),
            NotificationAction::new("dismiss", "Dismiss"),
        ];

        let notification3 = ctx.service.notify_with_actions(
            "Action Notification",
            "This is a notification with actions",
            "Test Source",
            actions,
        ).await.unwrap();

        assert_eq!(notification3.title(), "Action Notification");
        assert_eq!(notification3.actions().len(), 2);
    }

    #[tokio::test]
    async fn test_expired_notifications() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_send_notification()
            .returning(|_| Ok(()));

        ctx.provider
            .expect_dismiss_notification()
            .returning(|_| Ok(()));

        let mut notification = Notification::new(
            "Expiring Notification",
            "This notification will expire",
            "Test Source",
        );
        notification.expires_in(Duration::seconds(-1)); // Already expired
        let notification_id = notification.id();

        ctx.service.create_notification(notification.clone()).await.unwrap();

        // Getting all notifications should remove expired ones
        let notifications = ctx.service.get_all_notifications().await.unwrap();
        assert!(notifications.is_empty());

        // Trying to get the expired notification should fail
        let result = ctx.service.get_notification(notification_id).await;
        assert!(result.is_err());
    }
}
