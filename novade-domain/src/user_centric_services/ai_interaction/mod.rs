// Declare submodules
pub mod types;
pub mod errors;
pub mod persistence_iface;
pub mod service;
pub mod persistence; // Added for Iteration 2

// Re-export main public types, traits, and errors
pub use self::types::{
    AIDataCategory,
    AIConsentStatus,
    AIModelCapability,
    AIModelProfile,
    AIConsentScope,
    AIConsent,
    // Iteration 2 types
    AttachmentData,
    InteractionParticipant,
    InteractionHistoryEntry,
    AIInteractionContext,
};
pub use self::errors::AIInteractionError;
pub use self::persistence_iface::{
    AIConsentProvider,
    AIModelProfileProvider,
};
pub use self::persistence::{ // Added for Iteration 2
    FilesystemAIConsentProvider,
    FilesystemAIModelProfileProvider,
};
pub use self::service::{
    AIInteractionLogicService,
    DefaultAIInteractionLogicService,
};
