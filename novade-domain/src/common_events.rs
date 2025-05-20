//! Common events module for the NovaDE domain layer.
//!
//! This module provides event types and utilities for event-driven
//! communication between different parts of the domain layer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::shared_types::EntityId;

/// A domain event representing something that happened in the domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent<T> {
    /// The unique identifier of the event
    pub event_id: EntityId,
    /// The timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    /// The payload of the event
    pub payload: T,
    /// The source of the event (e.g., component or module name)
    pub source: String,
}

impl<T> DomainEvent<T> {
    /// Creates a new domain event with the specified payload and source.
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload of the event
    /// * `source` - The source of the event
    ///
    /// # Returns
    ///
    /// A new `DomainEvent` with the specified payload and source.
    pub fn new(payload: T, source: impl Into<String>) -> Self {
        DomainEvent {
            event_id: EntityId::new(),
            timestamp: Utc::now(),
            payload,
            source: source.into(),
        }
    }
}

impl<T: fmt::Debug> fmt::Display for DomainEvent<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Event[{}] from {} at {}: {:?}",
            self.event_id, self.source, self.timestamp, self.payload
        )
    }
}

/// Workspace-related events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceEvent {
    /// A workspace was created.
    WorkspaceCreated {
        /// The ID of the workspace
        workspace_id: EntityId,
        /// The name of the workspace
        name: String,
    },
    /// A workspace was updated.
    WorkspaceUpdated {
        /// The ID of the workspace
        workspace_id: EntityId,
        /// The name of the workspace
        name: String,
    },
    /// A workspace was deleted.
    WorkspaceDeleted {
        /// The ID of the workspace
        workspace_id: EntityId,
    },
    /// A window was assigned to a workspace.
    WindowAssigned {
        /// The ID of the workspace
        workspace_id: EntityId,
        /// The ID of the window
        window_id: EntityId,
    },
    /// A window was removed from a workspace.
    WindowRemoved {
        /// The ID of the workspace
        workspace_id: EntityId,
        /// The ID of the window
        window_id: EntityId,
    },
    /// The active workspace changed.
    ActiveWorkspaceChanged {
        /// The ID of the new active workspace
        workspace_id: EntityId,
    },
}

/// Theming-related events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemingEvent {
    /// A theme was loaded.
    ThemeLoaded {
        /// The ID of the theme
        theme_id: EntityId,
        /// The name of the theme
        name: String,
    },
    /// A theme was applied.
    ThemeApplied {
        /// The ID of the theme
        theme_id: EntityId,
        /// The name of the theme
        name: String,
    },
    /// A theme was updated.
    ThemeUpdated {
        /// The ID of the theme
        theme_id: EntityId,
        /// The name of the theme
        name: String,
    },
    /// A theme was deleted.
    ThemeDeleted {
        /// The ID of the theme
        theme_id: EntityId,
    },
}

/// Global settings-related events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GlobalSettingsEvent {
    /// A setting was changed.
    SettingChanged {
        /// The key of the setting
        key: String,
        /// The new value of the setting (serialized)
        value: String,
    },
    /// Settings were loaded.
    SettingsLoaded,
    /// Settings were saved.
    SettingsSaved,
    /// Settings were reset to defaults.
    SettingsReset,
}

/// Window policy-related events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowPolicyEvent {
    /// A policy was applied to a window.
    PolicyApplied {
        /// The ID of the policy
        policy_id: EntityId,
        /// The ID of the window
        window_id: EntityId,
    },
    /// A policy was updated.
    PolicyUpdated {
        /// The ID of the policy
        policy_id: EntityId,
    },
    /// A policy was deleted.
    PolicyDeleted {
        /// The ID of the policy
        policy_id: EntityId,
    },
}

/// Notification-related events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationEvent {
    /// A notification was created.
    NotificationCreated {
        /// The ID of the notification
        notification_id: EntityId,
        /// The title of the notification
        title: String,
        /// The body of the notification
        body: String,
        /// The urgency of the notification
        urgency: NotificationUrgency,
    },
    /// A notification was shown.
    NotificationShown {
        /// The ID of the notification
        notification_id: EntityId,
    },
    /// A notification was dismissed.
    NotificationDismissed {
        /// The ID of the notification
        notification_id: EntityId,
    },
    /// A notification was acted upon.
    NotificationActioned {
        /// The ID of the notification
        notification_id: EntityId,
        /// The ID of the action
        action_id: String,
    },
}

/// The urgency level of a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationUrgency {
    /// Low urgency.
    Low,
    /// Normal urgency.
    Normal,
    /// High urgency.
    High,
    /// Critical urgency.
    Critical,
}

impl fmt::Display for NotificationUrgency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationUrgency::Low => write!(f, "Low"),
            NotificationUrgency::Normal => write!(f, "Normal"),
            NotificationUrgency::High => write!(f, "High"),
            NotificationUrgency::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_domain_event_new() {
        let payload = "test payload";
        let source = "test source";
        let event = DomainEvent::new(payload, source);
        
        assert_eq!(event.payload, payload);
        assert_eq!(event.source, source);
    }
    
    #[test]
    fn test_domain_event_display() {
        let payload = "test payload";
        let source = "test source";
        let event = DomainEvent::new(payload, source);
        
        let display = format!("{}", event);
        assert!(display.contains("test source"));
        assert!(display.contains("test payload"));
    }
    
    #[test]
    fn test_notification_urgency_display() {
        assert_eq!(format!("{}", NotificationUrgency::Low), "Low");
        assert_eq!(format!("{}", NotificationUrgency::Normal), "Normal");
        assert_eq!(format!("{}", NotificationUrgency::High), "High");
        assert_eq!(format!("{}", NotificationUrgency::Critical), "Critical");
    }
}
