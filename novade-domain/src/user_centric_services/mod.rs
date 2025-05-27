// Declare submodules
pub mod ai_interaction;
pub mod events; 
pub mod notifications_core; 

// Re-export main public types, traits, and errors from the ai_interaction module
pub use self::ai_interaction::{
    AIDataCategory,
    AIConsentStatus,
    AIModelCapability,
    AIModelProfile,
    AIConsentScope,
    AIConsent,
    AIInteractionError,
    AIConsentProvider,
    AIModelProfileProvider,
    AIInteractionLogicService,
    DefaultAIInteractionLogicService,
    AttachmentData,
    InteractionParticipant,
    InteractionHistoryEntry,
    AIInteractionContext,
    FilesystemAIConsentProvider,
    FilesystemAIModelProfileProvider,
};

// Re-export events (which now include both AIInteractionEvent and NotificationEvent)
pub use self::events::{AIInteractionEvent, NotificationEvent, NotificationDismissReason};

// Re-export main public types, traits, and errors from the notifications_core module
pub use self::notifications_core::{
    NotificationUrgency,
    NotificationActionType,
    NotificationAction,
    NotificationInput,
    Notification,
    NotificationError,
    NotificationService,
    DefaultNotificationService,
    NotificationHistoryProvider, // Added in Iteration 2
};
