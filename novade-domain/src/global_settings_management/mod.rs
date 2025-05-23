//! Manages global desktop settings for NovaDE.
//!
//! This module provides types for defining settings, paths for accessing
//! specific settings, error types, and an interface for persistence.

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
};
pub use paths::{
    SettingPath,
    AppearanceSettingPath,
    FontSettingPath,
    WorkspaceSettingPath,
    InputBehaviorSettingPath,
    PowerManagementPolicySettingPath,
    DefaultApplicationsSettingPath,
    SettingPathParseError, // Also re-export the parse error for FromStr
};
pub use errors::GlobalSettingsError;
pub use persistence_iface::SettingsPersistenceProvider;
