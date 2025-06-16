//! Defines core event types for the NovaDE system.
//!
//! This module contains enumerations and structures related to events
//! that can occur within the NovaDE core or be broadcast to other parts
//! of the system. These events are designed to be serializable and cloneable.

use serde::{Deserialize, Serialize};
use uuid::Uuid; // For potential event IDs in the future

/// Represents core system-level events within NovaDE.
///
/// This enum serves as a central point for defining various significant occurrences
/// that other components or services might need to react to. It is designed to be
/// extensible with more specific event types as the system grows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreEvent {
    /// Indicates that a system shutdown sequence has been initiated.
    ///
    /// This event signals that the system is preparing to shut down.
    /// Future enhancements might include a reason for the shutdown or the source
    /// that initiated it.
    SystemShutdownInitiated,

    /// Represents a generic notification to be displayed to the user.
    ///
    /// Notifications typically include a title, a body message, and an urgency level.
    /// An ID is included for potential tracking or management of notifications.
    Notification {
        /// A unique identifier for the notification.
        id: Uuid,
        /// The title of the notification.
        title: String,
        /// The main content or body of the notification message.
        body: String,
        /// The urgency level of the notification.
        urgency: NotificationUrgency,
    },
    // Example:
    // /// Indicates a user session has started or ended.
    // UserSessionChanged { user_id: String, new_status: SessionStatus },
}

/// Defines the urgency levels for notifications.
///
/// This helps in categorizing notifications and how they might be presented
/// to the user (e.g., visually, audibly).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationUrgency {
    /// Low urgency, typically for informational messages that do not require immediate attention.
    Low,
    /// Normal urgency, for standard notifications. This is the default.
    Normal,
    /// Critical urgency, for important alerts that require immediate user awareness.
    Critical,
}

impl Default for NotificationUrgency {
    /// Returns `NotificationUrgency::Normal` as the default urgency.
    fn default() -> Self {
        NotificationUrgency::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    // Ensure common traits are implemented
    assert_impl_all!(CoreEvent: Send, Sync, std::fmt::Debug, Clone, PartialEq, Eq, Serialize);
    assert_impl_all!(NotificationUrgency: Send, Sync, std::fmt::Debug, Clone, Copy, PartialEq, Eq, Serialize, Default);

    #[test]
    fn core_event_creation_and_equality() {
        let event1 = CoreEvent::SystemShutdownInitiated;
        let event2 = CoreEvent::SystemShutdownInitiated;
        let event3 = CoreEvent::Notification {
            id: Uuid::new_v4(),
            title: "Hello".to_string(),
            body: "World".to_string(),
            urgency: NotificationUrgency::Normal,
        };
        // Create event4 with the same id as event3 for equality check
        let id_for_event4 = match &event3 {
            CoreEvent::Notification { id, .. } => *id,
            _ => panic!("event3 was not a Notification event"),
        };
        let event4 = CoreEvent::Notification {
            id: id_for_event4, // Same ID for comparison
            title: "Hello".to_string(),
            body: "World".to_string(),
            urgency: NotificationUrgency::Normal,
        };

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
        assert_eq!(event3, event4);
    }

    #[test]
    fn notification_urgency_default() {
        assert_eq!(NotificationUrgency::default(), NotificationUrgency::Normal);
    }

    #[test]
    fn core_event_serialization_deserialization() {
        let event = CoreEvent::Notification {
            id: Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8").unwrap(),
            title: "Test Event".to_string(),
            body: "This is a test event.".to_string(),
            urgency: NotificationUrgency::Critical,
        };

        let serialized = serde_json::to_string(&event).expect("Failed to serialize CoreEvent");
        let deserialized: CoreEvent = serde_json::from_str(&serialized).expect("Failed to deserialize CoreEvent");

        assert_eq!(event, deserialized);
    }

    #[test]
    fn system_shutdown_event_serialization_deserialization() {
        let event = CoreEvent::SystemShutdownInitiated;
        let serialized = serde_json::to_string(&event).expect("Failed to serialize CoreEvent");
        let deserialized: CoreEvent = serde_json::from_str(&serialized).expect("Failed to deserialize CoreEvent");
        assert_eq!(event, deserialized);
    }
}
