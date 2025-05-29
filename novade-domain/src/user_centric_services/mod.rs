// Main module for user-centric services.

pub mod ai_interaction;
pub mod notifications_core; // Placeholder for now
pub mod events;

// Re-exports for key public types/traits.
pub use ai_interaction::{AIInteractionLogicService, AIInteractionError, DefaultAIInteractionLogicService, AIConsentProvider, AIModelProfileProvider, FilesystemAIConsentProvider, FilesystemAIModelProfileProvider}; // Added Default impl and providers
pub use events::{UserCentricEvent, AIInteractionEventEnum, NotificationEventEnum}; // UserCentricEvent was missing, AI/Notification enums re-exported from lib.rs via events.rs
pub use notifications_core::NotificationError;
// pub use notifications_core::NotificationService; // Re-export when service trait is defined
