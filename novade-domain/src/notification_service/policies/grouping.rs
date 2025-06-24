// novade-domain/src/notification_service/policies/grouping.rs

use crate::user_centric_services::notifications_core::types::Notification;
use std::collections::HashMap;
use thiserror::Error;
use tracing::instrument;

/// Represents an error that can occur during the notification grouping process.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum GroupingError {
    #[error("An unexpected error occurred while grouping notifications: {0}")]
    Unexpected(String),
    // Add more specific errors as needed
}

/// A trait for defining notification grouping policies.
///
/// Implementors of this trait define how a list of notifications should be
/// organized into groups.
pub trait NotificationGroupingPolicy: Send + Sync {
    /// Groups the given notifications based on the policy's criteria.
    ///
    /// # Arguments
    ///
    /// * `notifications`: A slice of `Notification` objects to be grouped.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashMap` where:
    /// - The key is a `String` representing the group identifier (e.g., application name, category).
    /// - The value is a `Vec<Notification>` containing notifications belonging to that group.
    /// The notifications within the vector are clones of the input notifications.
    ///
    /// Or a `GroupingError` if an issue occurs during grouping.
    #[instrument(skip(self, notifications), fields(notification_count = notifications.len()))]
    fn group_notifications(
        &self,
        notifications: &[Notification],
    ) -> Result<HashMap<String, Vec<Notification>>, GroupingError>;
}

/// The default notification grouping policy.
///
/// This policy groups notifications by their `application_name`.
#[derive(Debug, Default, Clone)]
pub struct DefaultGroupingPolicy;

impl DefaultGroupingPolicy {
    /// Creates a new `DefaultGroupingPolicy`.
    pub fn new() -> Self {
        DefaultGroupingPolicy
    }
}

impl NotificationGroupingPolicy for DefaultGroupingPolicy {
    fn group_notifications(
        &self,
        notifications: &[Notification],
    ) -> Result<HashMap<String, Vec<Notification>>, GroupingError> {
        tracing::debug!(
            "Grouping {} notifications with DefaultGroupingPolicy by application_name.",
            notifications.len()
        );
        let mut grouped_notifications: HashMap<String, Vec<Notification>> = HashMap::new();

        for notification in notifications {
            grouped_notifications
                .entry(notification.application_name.clone())
                .or_default()
                .push(notification.clone());
        }

        tracing::trace!(
            "Notifications grouped into {} groups.",
            grouped_notifications.len()
        );
        Ok(grouped_notifications)
    }
}
