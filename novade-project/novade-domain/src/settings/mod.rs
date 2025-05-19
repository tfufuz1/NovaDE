//! Global settings module for the NovaDE domain layer.
//!
//! This module provides functionality for managing global settings
//! in the NovaDE desktop environment.

pub mod core;
pub mod service;
pub mod provider;

// Re-export key types for convenience
pub use core::{Setting, SettingKey, SettingValue, SettingCategory};
pub use service::{SettingsService, DefaultSettingsService};
pub use provider::{SettingsProvider, FileSettingsProvider};
