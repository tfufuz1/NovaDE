// Declare submodules
pub mod types;
pub mod errors;
pub mod service;
pub mod persistence_iface; 
pub mod persistence; // Added in Iteration 3

// Re-export main public types, service trait, and errors
pub use self::types::{
    NotificationUrgency,
    NotificationActionType,
    NotificationAction,
    NotificationInput,
    Notification,
};
pub use self::errors::NotificationError;
pub use self::service::{
    NotificationService,
    DefaultNotificationService,
};
pub use self::persistence_iface::NotificationHistoryProvider;
pub use self::persistence::FilesystemNotificationHistoryProvider; // Added in Iteration 3
