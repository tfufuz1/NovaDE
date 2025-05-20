//! Core notification types for the NovaDE domain layer.
//!
//! This module provides the fundamental types and structures
//! for notification management in the NovaDE desktop environment.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use crate::shared_types::{EntityId, Version, Identifiable, Versionable};
use crate::error::{DomainResult, NotificationError};

/// A unique identifier for notifications.
pub type NotificationId = EntityId;

/// The priority of a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationPriority {
    /// A low priority notification.
    Low,
    /// A normal priority notification.
    Normal,
    /// A high priority notification.
    High,
    /// A critical priority notification.
    Critical,
}

impl fmt::Display for NotificationPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationPriority::Low => write!(f, "Low"),
            NotificationPriority::Normal => write!(f, "Normal"),
            NotificationPriority::High => write!(f, "High"),
            NotificationPriority::Critical => write!(f, "Critical"),
        }
    }
}

/// An action that can be performed on a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    /// The ID of the action.
    pub id: String,
    /// The label of the action.
    pub label: String,
    /// Whether the action is the default action.
    pub is_default: bool,
    /// Additional data for the action.
    pub data: HashMap<String, String>,
}

impl NotificationAction {
    /// Creates a new notification action.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the action
    /// * `label` - The label of the action
    ///
    /// # Returns
    ///
    /// A new notification action.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        NotificationAction {
            id: id.into(),
            label: label.into(),
            is_default: false,
            data: HashMap::new(),
        }
    }

    /// Sets whether the action is the default action.
    ///
    /// # Arguments
    ///
    /// * `is_default` - Whether the action is the default action
    ///
    /// # Returns
    ///
    /// The modified notification action.
    pub fn with_default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }

    /// Adds additional data to the action.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the data
    /// * `value` - The value of the data
    ///
    /// # Returns
    ///
    /// The modified notification action.
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }
}

/// A notification in the NovaDE desktop environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// The unique identifier of the notification.
    id: NotificationId,
    /// The title of the notification.
    title: String,
    /// The body of the notification.
    body: String,
    /// The source of the notification.
    source: String,
    /// The priority of the notification.
    priority: NotificationPriority,
    /// The icon of the notification.
    icon: Option<String>,
    /// The actions of the notification.
    actions: Vec<NotificationAction>,
    /// Whether the notification is persistent.
    persistent: bool,
    /// The expiration time of the notification.
    expires_at: Option<DateTime<Utc>>,
    /// The creation timestamp.
    created_at: DateTime<Utc>,
    /// The last update timestamp.
    updated_at: DateTime<Utc>,
    /// The version of the notification.
    version: Version,
}

impl Notification {
    /// Creates a new notification.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the notification
    /// * `body` - The body of the notification
    /// * `source` - The source of the notification
    ///
    /// # Returns
    ///
    /// A new notification.
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Notification {
            id: NotificationId::new(),
            title: title.into(),
            body: body.into(),
            source: source.into(),
            priority: NotificationPriority::Normal,
            icon: None,
            actions: Vec::new(),
            persistent: false,
            expires_at: None,
            created_at: now,
            updated_at: now,
            version: Version::initial(),
        }
    }

    /// Gets the title of the notification.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Sets the title of the notification.
    ///
    /// # Arguments
    ///
    /// * `title` - The new title of the notification
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Gets the body of the notification.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Sets the body of the notification.
    ///
    /// # Arguments
    ///
    /// * `body` - The new body of the notification
    pub fn set_body(&mut self, body: impl Into<String>) {
        self.body = body.into();
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Gets the source of the notification.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Sets the source of the notification.
    ///
    /// # Arguments
    ///
    /// * `source` - The new source of the notification
    pub fn set_source(&mut self, source: impl Into<String>) {
        self.source = source.into();
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Gets the priority of the notification.
    pub fn priority(&self) -> NotificationPriority {
        self.priority
    }

    /// Sets the priority of the notification.
    ///
    /// # Arguments
    ///
    /// * `priority` - The new priority of the notification
    pub fn set_priority(&mut self, priority: NotificationPriority) {
        self.priority = priority;
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Gets the icon of the notification.
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    /// Sets the icon of the notification.
    ///
    /// # Arguments
    ///
    /// * `icon` - The new icon of the notification
    pub fn set_icon(&mut self, icon: Option<String>) {
        self.icon = icon;
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Gets the actions of the notification.
    pub fn actions(&self) -> &[NotificationAction] {
        &self.actions
    }

    /// Adds an action to the notification.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to add
    pub fn add_action(&mut self, action: NotificationAction) {
        self.actions.push(action);
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Removes an action from the notification.
    ///
    /// # Arguments
    ///
    /// * `action_id` - The ID of the action to remove
    ///
    /// # Returns
    ///
    /// `true` if the action was removed, `false` if it wasn't found.
    pub fn remove_action(&mut self, action_id: &str) -> bool {
        let len = self.actions.len();
        self.actions.retain(|a| a.id != action_id);
        
        if self.actions.len() != len {
            self.updated_at = Utc::now();
            self.increment_version();
            true
        } else {
            false
        }
    }

    /// Gets the default action of the notification.
    ///
    /// # Returns
    ///
    /// The default action, or `None` if there is no default action.
    pub fn default_action(&self) -> Option<&NotificationAction> {
        self.actions.iter().find(|a| a.is_default)
    }

    /// Checks if the notification is persistent.
    pub fn is_persistent(&self) -> bool {
        self.persistent
    }

    /// Sets whether the notification is persistent.
    ///
    /// # Arguments
    ///
    /// * `persistent` - Whether the notification is persistent
    pub fn set_persistent(&mut self, persistent: bool) {
        self.persistent = persistent;
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Gets the expiration time of the notification.
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }

    /// Sets the expiration time of the notification.
    ///
    /// # Arguments
    ///
    /// * `expires_at` - The new expiration time of the notification
    pub fn set_expires_at(&mut self, expires_at: Option<DateTime<Utc>>) {
        self.expires_at = expires_at;
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Sets the expiration time of the notification to a duration from now.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration from now when the notification should expire
    pub fn expires_in(&mut self, duration: Duration) {
        self.expires_at = Some(Utc::now() + duration);
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Checks if the notification has expired.
    ///
    /// # Returns
    ///
    /// `true` if the notification has expired, `false` otherwise.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Gets the creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Gets the last update timestamp.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Validates the notification.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the notification is valid, or an error if it is invalid.
    pub fn validate(&self) -> DomainResult<()> {
        if self.title.is_empty() {
            return Err(NotificationError::Invalid("Notification title cannot be empty".to_string()).into());
        }
        
        if self.body.is_empty() {
            return Err(NotificationError::Invalid("Notification body cannot be empty".to_string()).into());
        }
        
        if self.source.is_empty() {
            return Err(NotificationError::Invalid("Notification source cannot be empty".to_string()).into());
        }
        
        Ok(())
    }
}

impl Identifiable for Notification {
    fn id(&self) -> EntityId {
        self.id
    }
}

impl Versionable for Notification {
    fn version(&self) -> Version {
        self.version
    }

    fn increment_version(&mut self) {
        self.version = self.version.next();
    }
}

impl fmt::Display for Notification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Notification[{}] '{}' ({})",
            self.id, self.title, self.priority
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_notification_priority_display() {
        assert_eq!(format!("{}", NotificationPriority::Low), "Low");
        assert_eq!(format!("{}", NotificationPriority::Normal), "Normal");
        assert_eq!(format!("{}", NotificationPriority::High), "High");
        assert_eq!(format!("{}", NotificationPriority::Critical), "Critical");
    }
    
    #[test]
    fn test_notification_action_new() {
        let action = NotificationAction::new("open", "Open");
        
        assert_eq!(action.id, "open");
        assert_eq!(action.label, "Open");
        assert!(!action.is_default);
        assert!(action.data.is_empty());
    }
    
    #[test]
    fn test_notification_action_with_default() {
        let action = NotificationAction::new("open", "Open").with_default(true);
        
        assert!(action.is_default);
    }
    
    #[test]
    fn test_notification_action_with_data() {
        let action = NotificationAction::new("open", "Open")
            .with_data("url", "https://example.com")
            .with_data("target", "_blank");
        
        assert_eq!(action.data.len(), 2);
        assert_eq!(action.data.get("url"), Some(&"https://example.com".to_string()));
        assert_eq!(action.data.get("target"), Some(&"_blank".to_string()));
    }
    
    #[test]
    fn test_notification_new() {
        let notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        
        assert_eq!(notification.title(), "Test Notification");
        assert_eq!(notification.body(), "This is a test notification");
        assert_eq!(notification.source(), "Test Source");
        assert_eq!(notification.priority(), NotificationPriority::Normal);
        assert!(notification.icon().is_none());
        assert!(notification.actions().is_empty());
        assert!(!notification.is_persistent());
        assert!(notification.expires_at().is_none());
        assert_eq!(notification.version(), Version::initial());
    }
    
    #[test]
    fn test_notification_setters() {
        let mut notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        
        notification.set_title("Updated Title");
        assert_eq!(notification.title(), "Updated Title");
        
        notification.set_body("Updated Body");
        assert_eq!(notification.body(), "Updated Body");
        
        notification.set_source("Updated Source");
        assert_eq!(notification.source(), "Updated Source");
        
        notification.set_priority(NotificationPriority::High);
        assert_eq!(notification.priority(), NotificationPriority::High);
        
        notification.set_icon(Some("icon.png".to_string()));
        assert_eq!(notification.icon(), Some("icon.png"));
        
        notification.set_persistent(true);
        assert!(notification.is_persistent());
        
        let expiration = Utc::now() + Duration::hours(1);
        notification.set_expires_at(Some(expiration));
        assert!(notification.expires_at().is_some());
    }
    
    #[test]
    fn test_notification_actions() {
        let mut notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        
        let action1 = NotificationAction::new("open", "Open").with_default(true);
        let action2 = NotificationAction::new("dismiss", "Dismiss");
        
        notification.add_action(action1);
        notification.add_action(action2);
        
        assert_eq!(notification.actions().len(), 2);
        assert_eq!(notification.default_action().unwrap().id, "open");
        
        let removed = notification.remove_action("open");
        assert!(removed);
        assert_eq!(notification.actions().len(), 1);
        assert!(notification.default_action().is_none());
        
        let removed = notification.remove_action("nonexistent");
        assert!(!removed);
    }
    
    #[test]
    fn test_notification_expiration() {
        let mut notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        
        assert!(!notification.is_expired());
        
        notification.expires_in(Duration::seconds(-1));
        assert!(notification.is_expired());
        
        notification.set_expires_at(None);
        assert!(!notification.is_expired());
    }
    
    #[test]
    fn test_notification_validate() {
        let notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        
        assert!(notification.validate().is_ok());
        
        let mut invalid_notification = notification.clone();
        invalid_notification.set_title("");
        assert!(invalid_notification.validate().is_err());
        
        let mut invalid_notification = notification.clone();
        invalid_notification.set_body("");
        assert!(invalid_notification.validate().is_err());
        
        let mut invalid_notification = notification.clone();
        invalid_notification.set_source("");
        assert!(invalid_notification.validate().is_err());
    }
    
    #[test]
    fn test_notification_display() {
        let notification = Notification::new(
            "Test Notification",
            "This is a test notification",
            "Test Source",
        );
        
        let display = format!("{}", notification);
        
        assert!(display.contains("Test Notification"));
        assert!(display.contains("Normal"));
    }
}
