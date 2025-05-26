//! Manages global desktop settings and application-specific configurations for NovaDE.
//!
//! This module provides:
//! * Types for defining various settings categories, including appearance, workspace,
//!   input behavior, power management, default applications, and custom settings
//!   for individual applications.
//! * Path structures for uniquely addressing each setting.
//! * Error types for handling issues related to settings management.
//! * An interface for settings persistence.
//!
//! ## Application-Specific Settings
//!
//! Applications can store and retrieve their own settings within the global settings
//! framework. This is achieved using the `SettingPath::Application` variant, which
//! encapsulates an `ApplicationSettingPath`.
//!
//! ### Example Usage:
//!
//! To update an application setting:
//! ```rust,ignore
//! use novade_domain::global_settings_management::{
//!     SettingPath, ApplicationSettingPath, GlobalSettingsService
//! };
//! use serde_json::json;
//!
//! async fn update_app_setting(service: &impl GlobalSettingsService) {
//!     let path = SettingPath::Application(ApplicationSettingPath {
//!         app_id: "com.example.my_app".to_string(),
//!         key: "ui.theme.variant".to_string(),
//!     });
//!     let value = json!("dark");
//!     service.update_setting(path, value).await.unwrap();
//! }
//! ```
//!
//! To retrieve an application setting:
//! ```rust,ignore
//! use novade_domain::global_settings_management::{
//!     SettingPath, ApplicationSettingPath, GlobalSettingsService
//! };
//!
//! async fn get_app_setting(service: &impl GlobalSettingsService) -> Option<serde_json::Value> {
//!     let path = SettingPath::Application(ApplicationSettingPath {
//!         app_id: "com.example.my_app".to_string(),
//!         key: "ui.theme.variant".to_string(),
//!     });
//!     service.get_setting(&path).ok()
//! }
//! ```
//!
//! The `app_id` should be a unique identifier for the application, typically in
//! reverse domain name notation (e.g., "com.example.my_app"). The `key` can be
//! any string, potentially hierarchical using dots (e.g., "ui.theme.variant").
//! The value is stored as a `serde_json::Value`, allowing for flexibility.

// Declare submodules
pub mod types;
pub mod paths;
pub mod errors;
pub mod persistence_iface;

#[cfg(test)]
mod types_tests; // Include the types tests module
#[cfg(test)]
mod paths_tests; // Include the paths tests module
#[cfg(test)]
mod errors_tests; // Include the errors tests module
#[cfg(test)]
mod persistence_tests; // Include the persistence tests module

// Re-export key public types for easier access from outside this module.
pub use types::{
    GlobalDesktopSettings,
    AppearanceSettings,
    FontSettings,
    WorkspaceSettings,
    InputBehaviorSettings,
    PowerManagementPolicySettings,
    DefaultApplicationsSettings,
    ColorScheme,
    FontHinting,
    FontAntialiasing,
    MouseAccelerationProfile,
    LidCloseAction,
    WorkspaceSwitchingBehavior,
    ApplicationSettingGroup, // Added for visibility
};
pub use paths::{
    SettingPath,
    AppearanceSettingPath,
    FontSettingPath,
    WorkspaceSettingPath,
    InputBehaviorSettingPath,
    PowerManagementPolicySettingPath,
    DefaultApplicationsSettingPath,
    ApplicationSettingPath, // Added for visibility
    SettingPathParseError, // Also re-export the parse error for FromStr
};
pub use errors::GlobalSettingsError;
pub use persistence_iface::SettingsPersistenceProvider;
