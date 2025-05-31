//! Notification module for the NovaDE domain layer.
//!
//! This module provides functionality for managing notifications
//! in the NovaDE desktop environment.

pub mod types;
pub mod errors;
pub mod rules_errors;
pub mod persistence_iface;
pub mod persistence;
pub mod engine;
pub mod rules_types;
pub mod service;
pub mod rules_persistence; // Added

// Re-export primary error types
pub use errors::NotificationError;
pub use rules_errors::NotificationRulesError;

// Re-export persistence interfaces
pub use persistence_iface::{
    NotificationHistoryProvider,
    NotificationRulesProvider,
    NotificationPersistence,
    NotificationRepository
};

// Re-export concrete persistence implementations
pub use persistence::{InMemoryNotificationPersistence, FilesystemNotificationHistoryProvider};
pub use rules_persistence::FilesystemNotificationRulesProvider;

// Re-export engine components
pub use engine::{NotificationRulesEngine, DefaultNotificationRulesEngine, RuleProcessingResult};
