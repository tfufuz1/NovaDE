//! Update the notification module to export the default manager.
//!
//! This module provides notification functionality for the NovaDE desktop environment,
//! allowing applications to send notifications to users.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::error::{DomainError, NotificationError};
use crate::entities::value_objects::Timestamp;

mod default_manager;

pub use default_manager::DefaultNotificationManager;

/// Represents the urgency level of a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationUrgency {
    /// Low urgency, can be viewed at user's leisure
    Low,
    /// Normal urgency, should be viewed soon
    Normal,
    /// High urgency, should be viewed immediately
    Critical,
}

/// Represents the category of a notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationCategory {
    /// System-related notification
    System,
    /// Application-related notification
    Application(String),
    /// User-defined category
    Custom(String),
}

/// Represents a notification action that can be taken by the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationAction {
    /// The action ID
    pub id: String,
    /// The action label
    pub label: String,
    /// Whether this is the default action
    pub is_default: bool,
}

/// Represents a notification in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    /// Unique identifier for the notification
    pub notification_id: String,
    /// The notification title
    pub title: String,
    /// The notification body
    pub body: String,
    /// The notification icon, if any
    pub icon: Option<String>,
    /// The notification category
    pub category: NotificationCategory,
    /// The notification urgency
    pub urgency: NotificationUrgency,
    /// Whether the notification has been read
    pub read: bool,
    /// Whether the notification can be dismissed
    pub dismissible: bool,
    /// The notification actions
    pub actions: Vec<NotificationAction>,
    /// The notification creation timestamp
    pub created_at: Timestamp,
    /// The notification expiration timestamp, if any
    pub expires_at: Option<Timestamp>,
}

/// Interface for the notification manager.
#[async_trait]
pub trait NotificationManager: Send + Sync {
    /// Creates a new notification.
    ///
    /// # Arguments
    ///
    /// * `title` - The notification title
    /// * `body` - The notification body
    /// * `category` - The notification category
    /// * `urgency` - The notification urgency
    /// * `icon` - The notification icon, if any
    /// * `dismissible` - Whether the notification can be dismissed
    /// * `actions` - The notification actions
    /// * `expires_at` - The notification expiration timestamp, if any
    ///
    /// # Returns
    ///
    /// A `Result` containing the created notification ID.
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
    ) -> Result<String, DomainError>;
    
    /// Gets a notification by ID.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The notification ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the notification if found.
    async fn get_notification(&self, notification_id: &str) -> Result<Notification, DomainError>;
    
    /// Lists all notifications.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all notifications.
    async fn list_notifications(&self) -> Result<Vec<Notification>, DomainError>;
    
    /// Lists notifications by category.
    ///
    /// # Arguments
    ///
    /// * `category` - The notification category
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of notifications in the specified category.
    async fn list_notifications_by_category(&self, category: &NotificationCategory) -> Result<Vec<Notification>, DomainError>;
    
    /// Lists unread notifications.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of unread notifications.
    async fn list_unread_notifications(&self) -> Result<Vec<Notification>, DomainError>;
    
    /// Marks a notification as read.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The notification ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn mark_as_read(&self, notification_id: &str) -> Result<(), DomainError>;
    
    /// Marks all notifications as read.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn mark_all_as_read(&self) -> Result<(), DomainError>;
    
    /// Dismisses a notification.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The notification ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn dismiss_notification(&self, notification_id: &str) -> Result<(), DomainError>;
    
    /// Dismisses all notifications.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn dismiss_all_notifications(&self) -> Result<(), DomainError>;
    
    /// Executes a notification action.
    ///
    /// # Arguments
    ///
    /// * `notification_id` - The notification ID
    /// * `action_id` - The action ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn execute_action(&self, notification_id: &str, action_id: &str) -> Result<(), DomainError>;
}
