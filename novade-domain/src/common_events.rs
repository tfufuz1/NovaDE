use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::shared_types::{ApplicationId, UserSessionState};
// Corrected path for WorkspaceId, assuming it's a type alias or struct in that location
use crate::workspaces::common_types::WorkspaceId;


// --- UserActivityType Enum ---
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

// --- ShutdownReason Enum ---
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

// --- UserActivityDetectedEvent Struct ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserActivityDetectedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub activity_type: UserActivityType,
    pub current_session_state: UserSessionState,
    pub active_application_id: Option<ApplicationId>,
    pub active_workspace_id: Option<WorkspaceId>,
}

impl UserActivityDetectedEvent {
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

// --- SystemShutdownInitiatedEvent Struct ---
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

#[cfg(test)]
mod tests {
    use super::*;
    // Ensure UserSessionState is in scope for tests if not already by crate::shared_types
    // use crate::shared_types::UserSessionState; // Already imported via super::* effectively

    // Helper for WorkspaceId in tests.
    // Assuming WorkspaceId can be created from a Uuid or has a new_v4()
    // For this example, let's assume it can be constructed from Uuid for testing.
    // If WorkspaceId is just `pub type WorkspaceId = Uuid;` in its module, this is fine.
    // If not, test setup might need adjustment based on actual WorkspaceId definition.
    fn create_test_workspace_id() -> WorkspaceId {
        // This is a placeholder. The actual WorkspaceId might be a struct Uuid wrapper
        // or a direct type alias to Uuid. The actual implementation of WorkspaceId
        // will determine how it's created.
        // If it's `pub struct WorkspaceId(Uuid);` with `pub fn new(id: Uuid) -> Self { Self(id) }`
        // or `impl From<Uuid> for WorkspaceId`
        // then `WorkspaceId::from(Uuid::new_v4())` or `WorkspaceId::new(Uuid::new_v4())`
        // If it's `pub type WorkspaceId = Uuid;` then `Uuid::new_v4()`
        // The path `crate::workspaces::common_types::WorkspaceId` suggests it's defined there.
        // For now, assuming it's constructible like this for tests.
        WorkspaceId::from(Uuid::new_v4())
    }


    // --- UserActivityType Tests ---
    #[test]
    fn user_activity_type_serde() {
        let activity = UserActivityType::ApplicationFocused;
        let serialized = serde_json::to_string(&activity).unwrap();
        assert_eq!(serialized, "\"ApplicationFocused\"");
        let deserialized: UserActivityType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, activity);

        let activity_kh = UserActivityType::KeyPressed;
        let serialized_kh = serde_json::to_string(&activity_kh).unwrap();
        assert_eq!(serialized_kh, "\"KeyPressed\"");
        let deserialized_kh: UserActivityType = serde_json::from_str(&serialized_kh).unwrap();
        assert_eq!(deserialized_kh, activity_kh);
    }

    // --- ShutdownReason Tests ---
    #[test]
    fn shutdown_reason_default() {
        assert_eq!(ShutdownReason::default(), ShutdownReason::UserRequest);
    }

    #[test]
    fn shutdown_reason_serde() {
        let reason = ShutdownReason::LowBattery;
        let serialized = serde_json::to_string(&reason).unwrap();
        assert_eq!(serialized, "\"LowBattery\"");
        let deserialized: ShutdownReason = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, reason);

        let reason_default = ShutdownReason::default();
        let serialized_default = serde_json::to_string(&reason_default).unwrap();
        assert_eq!(serialized_default, "\"UserRequest\"");
        let deserialized_default: ShutdownReason = serde_json::from_str(&serialized_default).unwrap();
        assert_eq!(deserialized_default, reason_default);
    }

    // --- UserActivityDetectedEvent Tests ---
    #[test]
    fn user_activity_detected_event_new() {
        let app_id = ApplicationId::new("test.app");
        let ws_id = create_test_workspace_id();

        let event = UserActivityDetectedEvent::new(
            UserActivityType::MouseClicked,
            UserSessionState::Active,
            Some(app_id.clone()),
            Some(ws_id.clone()),
        );

        assert_eq!(event.activity_type, UserActivityType::MouseClicked);
        assert_eq!(event.current_session_state, UserSessionState::Active);
        assert_eq!(event.active_application_id, Some(app_id));
        assert_eq!(event.active_workspace_id, Some(ws_id));
        assert!(!event.event_id.is_nil());
    }

    #[test]
    fn user_activity_detected_event_serde() {
        let ws_id = create_test_workspace_id();
        // Capture the actual Uuid from WorkspaceId for assertion if it's a wrapper
        let ws_id_inner_uuid_str = ws_id.to_string(); // Assuming WorkspaceId implements Display like Uuid

        let event = UserActivityDetectedEvent {
            event_id: Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8").unwrap(),
            timestamp: chrono::DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z").unwrap().with_timezone(&Utc),
            activity_type: UserActivityType::WindowOpened,
            current_session_state: UserSessionState::Idle,
            active_application_id: Some(ApplicationId::new("another.app")),
            active_workspace_id: Some(ws_id.clone()),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        // Example of how to check parts of the JSON if timestamp or event_id are tricky
        assert!(serialized.starts_with("{\"event_id\":\"a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8\""));
        assert!(serialized.contains("\"timestamp\":\"2023-01-01T12:00:00Z\""));
        assert!(serialized.contains("\"activity_type\":\"WindowOpened\""));
        assert!(serialized.contains("\"current_session_state\":\"Idle\""));
        assert!(serialized.contains("\"active_application_id\":\"another.app\""));
        assert!(serialized.contains(&format!("\"active_workspace_id\":\"{}\"", ws_id_inner_uuid_str)));


        let deserialized: UserActivityDetectedEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }

    // --- SystemShutdownInitiatedEvent Tests ---
    #[test]
    fn system_shutdown_initiated_event_new() {
        let event = SystemShutdownInitiatedEvent::new(
            ShutdownReason::SystemUpdate,
            true,
            Some(300),
            Some("System is rebooting for an update.".to_string()),
        );

        assert_eq!(event.reason, ShutdownReason::SystemUpdate);
        assert_eq!(event.is_reboot, true);
        assert_eq!(event.delay_seconds, Some(300));
        assert_eq!(event.message, Some("System is rebooting for an update.".to_string()));
        assert!(!event.event_id.is_nil());
    }

    #[test]
    fn system_shutdown_initiated_event_serde() {
        let event = SystemShutdownInitiatedEvent {
            event_id: Uuid::parse_str("b1b2b3b4-c1c2-d1d2-e1e2-e3e4e5e6e7e8").unwrap(),
            timestamp: chrono::DateTime::parse_from_rfc3339("2023-02-01T10:30:00Z").unwrap().with_timezone(&Utc),
            reason: ShutdownReason::ApplicationRequest,
            is_reboot: false,
            delay_seconds: None,
            message: Some("App requested shutdown.".to_string()),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.starts_with("{\"event_id\":\"b1b2b3b4-c1c2-d1d2-e1e2-e3e4e5e6e7e8\""));
        assert!(serialized.contains("\"timestamp\":\"2023-02-01T10:30:00Z\""));
        assert!(serialized.contains("\"reason\":\"ApplicationRequest\""));
        assert!(serialized.contains("\"is_reboot\":false"));
        assert!(serialized.contains("\"delay_seconds\":null"));
        assert!(serialized.contains("\"message\":\"App requested shutdown.\""));
        assert!(serialized.ends_with("}"));


        let deserialized: SystemShutdownInitiatedEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }
}
