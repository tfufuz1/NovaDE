// Main module for AI Interaction specific logic, types, and services.

pub mod types;
pub mod errors;
pub mod persistence_iface;
pub mod persistence; // Placeholder for actual implementations
pub mod service;     // Placeholder for the AIInteractionLogicService trait and its impl

// Re-exports for easier access by consumers of this submodule or parent modules.
pub use types::{
    AIDataCategory,
    AIConsentStatus,
    AttachmentData,
    InteractionParticipant,
    InteractionHistoryEntry,
    AIInteractionContext,
    AIConsentScope,
    AIConsent,
    AIModelCapability,
    AIModelProfile,
};
pub use errors::AIInteractionError;
pub use persistence_iface::{AIConsentProvider, AIModelProfileProvider};
pub use persistence::{FilesystemAIConsentProvider, FilesystemAIModelProfileProvider};

// Re-export service trait and its default implementation:
pub use service::{AIInteractionLogicService, DefaultAIInteractionLogicService};

// No unit tests in this mod.rs file. Tests are in respective files.
