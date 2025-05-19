//! Notification module for the NovaDE domain layer.
//!
//! This module provides functionality for managing notifications
//! in the NovaDE desktop environment.

pub mod core;
pub mod service;
pub mod provider;

// Re-export key types for convenience
pub use core::{Notification, NotificationId, NotificationPriority, NotificationAction};
pub use service::{NotificationService, DefaultNotificationService};
pub use provider::{NotificationProvider, SystemNotificationProvider};
