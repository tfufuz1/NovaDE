use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::shared_types::{ApplicationId, UserSessionState};

// Forward declaration for WorkspaceId
// This will be properly defined in the workspaces module
// For now, we can use a placeholder or a newtype if strictness is needed immediately.
// However, the prompt implies it's okay if it doesn't resolve yet,
// which suggests a direct path usage is fine.
// If compilation errors arise due to this, a temporary definition might be needed here or in lib.rs.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserActivityType {
    MouseMoved,
    MouseClicked,
    MouseWheel,
    KeyPressed,
    TextInput,
    // Add other relevant activity types as needed
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserActivityDetectedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub activity_type: UserActivityType,
    pub current_session_state: UserSessionState,
    pub active_application_id: Option<ApplicationId>,
    pub active_workspace_id: Option<crate::workspaces::core::types::WorkspaceId>, // Placeholder path
}

impl UserActivityDetectedEvent {
    pub fn new(
        activity_type: UserActivityType,
        current_session_state: UserSessionState,
        active_application_id: Option<ApplicationId>,
        active_workspace_id: Option<crate::workspaces::core::types::WorkspaceId>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShutdownReason {
    UserRequest,
    PowerButtonPress,
    LowBattery,
    SystemUpdate,
    ApplicationError,
    OsError,
    HardwareFailure,
    ScheduledRestart,
    SecurityPolicy,
    Other,
}

impl Default for ShutdownReason {
    fn default() -> Self {
        ShutdownReason::Other
    }
}

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
