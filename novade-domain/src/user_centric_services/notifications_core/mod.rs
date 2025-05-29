// Main module for core notification logic, types, and services.

pub mod types;
pub mod errors;
pub mod persistence_iface; // For notification history persistence
pub mod persistence;       // For filesystem implementation of persistence
pub mod service;           // For the NotificationService trait and its impl

// Re-exports for easier access by consumers of this submodule or parent modules.
pub use types::{
    Notification,
    NotificationInput,
    NotificationUrgency,
    NotificationAction,
    NotificationActionType,
    NotificationStats,
    DismissReason,
    NotificationFilterCriteria,
    NotificationSortOrder,
};
pub use errors::NotificationError;

// When service.rs and persistence_iface.rs are implemented, re-export their main traits:
// pub use service::NotificationService;
// pub use persistence_iface::NotificationHistoryProvider;

// No unit tests in this mod.rs file. Tests are in respective files.
