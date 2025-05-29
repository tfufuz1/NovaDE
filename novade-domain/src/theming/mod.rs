// Main module file for theming

pub mod types;
pub mod errors;
pub mod logic;
pub mod service;
pub mod events; // Added events module

// Re-exports
pub use errors::ThemingError;
pub use events::ThemeChangedEvent; // Added event re-export
pub use types::{
    TokenIdentifier, TokenValue, RawToken, TokenSet,
    ThemeIdentifier, ColorSchemeType, AccentColor,
    ThemeVariantDefinition, ThemeDefinition, AccentModificationType,
    AppliedThemeState, ThemingConfiguration,
};
pub use service::ThemingEngine; // Uncommented ThemingEngine re-export
// pub use service::ThemingEngineService; // ThemingEngineService trait is not used per plan
