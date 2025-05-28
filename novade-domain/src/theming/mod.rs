//! Theming engine for NovaDE, responsible for managing design tokens,
//! theme definitions, and applying them to the UI.

// Publicly export modules within the theming crate
pub mod types;
pub mod errors;
pub mod logic;
pub mod service;
pub mod default_config_service; // Added new module
// default_themes is not a module of code, but a directory of assets.

#[cfg(test)]
mod errors_tests; // Include the error tests module
#[cfg(test)]
mod service_tests; // Include the service tests module

// Re-export key public types for easier access from outside `domain::theming`.
pub use types::{
    TokenIdentifier,
    TokenValue,
    RawToken,
    TokenSet,
    ThemeIdentifier,
    ColorSchemeType,
    AccentColor,
    ThemeVariantDefinition,
    AccentModificationType,
    ThemeDefinition,
    AppliedThemeState,
    ThemingConfiguration,
};
pub use errors::ThemingError;
pub use service::{ThemingEngine, ThemeChangedEvent};
pub use default_config_service::DefaultFileSystemConfigService; // Export the new service

// Constants for default file paths might also be useful to export if they
// are needed by consumers for configuration, but typically these are internal details.
// pub use service::{DEFAULT_GLOBAL_TOKENS_PATH, FALLBACK_THEME_PATH}; // Example if needed publicly

// Note: `logic.rs` contains the core token resolution and theme application logic.
// It's not typically part of the direct public API surface for consumers of the
// `ThemingEngine` service, but its components are used internally by the service.
// Specific functions from `logic.rs` are not re-exported here unless they are
// intended for advanced integration or utility purposes outside the engine itself.
