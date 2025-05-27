use serde::{Deserialize, Serialize};
use uuid::Uuid; 
use crate::user_centric_services::ai_interaction::types::{AIDataCategory, AIConsentStatus, AIConsentScope, AIInteractionContext};
use crate::user_centric_services::notifications_core::types::Notification; // Corrected path

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AIInteractionEvent {
    AIModelProfilesReloaded {
        profiles_count: usize,
    },
    AIConsentUpdated {
        user_id: String,
        model_id: String, 
        category: AIDataCategory,
        new_status: AIConsentStatus,
        scope: AIConsentScope,
    },
    AIInteractionInitiated { 
        context: AIInteractionContext, 
    },
    AIContextUpdated { 
        context_id: Uuid, 
        updated_field: String, 
        new_data_preview: Option<String>,
    },
}

// --- Notification Events ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationDismissReason {
    User,       // User explicitly closed it
    Timeout,    // Expired due to timeout
    ReplacedByApp, // Application requested its replacement (e.g. with a new notification ID)
    ClosedByApp,   // Application explicitly requested its closure
    Unknown,    // Reason not specified
}

impl Default for NotificationDismissReason {
    fn default() -> Self {
        NotificationDismissReason::Unknown
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationEvent {
    NotificationPosted { 
        notification: Notification,
    },
    NotificationDismissed { 
        notification_id: Uuid, 
        reason: NotificationDismissReason,
    },
    NotificationRead { 
        notification_id: Uuid,
    },
    NotificationActionInvoked { 
        notification_id: Uuid, 
        action_key: String,
    },
}
