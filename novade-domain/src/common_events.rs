use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::shared_types::{UserSessionState, ApplicationId};

// TODO: Replace with actual type from workspaces module: crate::workspaces::core::types::WorkspaceId
pub type WorkspaceId = Uuid; 

/// Represents the type of user activity detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserActivityType {
    MouseMoved,
    MouseClicked,
    MouseWheelScrolled,
    KeyPressed,
    TouchInteraction,
    WorkspaceSwitched,
    ApplicationFocused,
    WindowOpened,
    WindowClosed,
}

/// Event triggered when user activity is detected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserActivityDetectedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub activity_type: UserActivityType,
    pub current_session_state: UserSessionState,
    pub active_application_id: Option<ApplicationId>,
    pub active_workspace_id: Option<WorkspaceId>, // Using placeholder
}

impl UserActivityDetectedEvent {
    /// Creates a new `UserActivityDetectedEvent`.
    pub fn new(
        activity_type: UserActivityType,
        current_session_state: UserSessionState,
        active_application_id: Option<ApplicationId>,
        active_workspace_id: Option<WorkspaceId>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            activity_type,
            current_session_state,
            active_application_id,
            active_workspace_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_types::{UserSessionState, ApplicationId}; // Ensure ApplicationId is in scope
    use chrono::Utc;

    // Test for UserActivityType serde
    #[test]
    fn user_activity_type_serde() {
        let activity = UserActivityType::MouseMoved;
        let serialized = serde_json::to_string(&activity).unwrap();
        assert_eq!(serialized, "\"MouseMoved\"");
        let deserialized: UserActivityType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, activity);
    }

    // Test for UserActivityDetectedEvent new() and serde
    #[test]
    fn user_activity_detected_event_new_and_serde() {
        let app_id = ApplicationId::new("test_app");
        let workspace_id = Uuid::new_v4(); // Using placeholder type

        let event = UserActivityDetectedEvent::new(
            UserActivityType::KeyPressed,
            UserSessionState::Active,
            Some(app_id.clone()),
            Some(workspace_id),
        );

        assert_eq!(event.activity_type, UserActivityType::KeyPressed);
        assert_eq!(event.current_session_state, UserSessionState::Active);
        assert_eq!(event.active_application_id, Some(app_id));
        assert_eq!(event.active_workspace_id, Some(workspace_id));
        assert!(event.timestamp <= Utc::now());

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: UserActivityDetectedEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, event);
    }
    
    #[test]
    fn user_activity_detected_event_serde_optional_none() {
        let event = UserActivityDetectedEvent::new(
            UserActivityType::WorkspaceSwitched,
            UserSessionState::Idle,
            None,
            None,
        );

        assert_eq!(event.activity_type, UserActivityType::WorkspaceSwitched);
        assert_eq!(event.current_session_state, UserSessionState::Idle);
        assert_eq!(event.active_application_id, None);
        assert_eq!(event.active_workspace_id, None);

        let serialized = serde_json::to_string(&event).unwrap();
        // println!("Serialized UserActivityDetectedEvent (Nones): {}", serialized); // For debugging
        let deserialized: UserActivityDetectedEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, event);
    }

    // Test for ShutdownReason Default and serde
    #[test]
    fn shutdown_reason_default_and_serde() {
        let default_reason = ShutdownReason::default();
        assert_eq!(default_reason, ShutdownReason::UserRequest);

        let reason = ShutdownReason::LowBattery;
        let serialized = serde_json::to_string(&reason).unwrap();
        assert_eq!(serialized, "\"LowBattery\"");
        let deserialized: ShutdownReason = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, reason);
    }

    // Test for SystemShutdownInitiatedEvent new() and serde
    #[test]
    fn system_shutdown_initiated_event_new_and_serde() {
        let event = SystemShutdownInitiatedEvent::new(
            ShutdownReason::SystemUpdate,
            true,
            Some(300),
            Some("System will reboot for updates.".to_string()),
        );

        assert_eq!(event.reason, ShutdownReason::SystemUpdate);
        assert!(event.is_reboot);
        assert_eq!(event.delay_seconds, Some(300));
        assert_eq!(event.message, Some("System will reboot for updates.".to_string()));
        assert!(event.timestamp <= Utc::now());

        let serialized = serde_json::to_string(&event).unwrap();
        // println!("Serialized SystemShutdownInitiatedEvent: {}", serialized); // For debugging
        let deserialized: SystemShutdownInitiatedEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, event);
    }

    #[test]
    fn system_shutdown_initiated_event_serde_optional_none() {
        let event = SystemShutdownInitiatedEvent::new(
            ShutdownReason::UserRequest,
            false,
            None,
            None,
        );

        assert_eq!(event.reason, ShutdownReason::UserRequest);
        assert!(!event.is_reboot);
        assert_eq!(event.delay_seconds, None);
        assert_eq!(event.message, None);

        let serialized = serde_json::to_string(&event).unwrap();
        // println!("Serialized SystemShutdownInitiatedEvent (Nones): {}", serialized); // For debugging
        let deserialized: SystemShutdownInitiatedEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, event);
    }
}

/// Reason for system shutdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ShutdownReason {
    #[default]
    UserRequest,
    PowerButtonPress,
    LowBattery,
    SystemUpdate,
    ApplicationRequest,
    OsError,
    Unknown,
}

/// Event triggered when system shutdown is initiated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemShutdownInitiatedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub reason: ShutdownReason,
    pub is_reboot: bool,
    pub delay_seconds: Option<u32>,
    pub message: Option<String>,
}

impl SystemShutdownInitiatedEvent {
    /// Creates a new `SystemShutdownInitiatedEvent`.
    pub fn new(
        reason: ShutdownReason,
        is_reboot: bool,
        delay_seconds: Option<u32>,
        message: Option<String>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            reason,
            is_reboot,
            delay_seconds,
            message,
        }
    }
}
