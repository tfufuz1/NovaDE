// Main module for global desktop settings and state management.

pub mod types;
pub mod paths;
pub mod errors;
pub mod events;
pub mod persistence_iface; // For the trait defining how settings are saved/loaded
pub mod service;           // For the GlobalSettingsService implementation

// Re-exports for easier access by consumers of the crate.
// These will be populated as the types are defined.

// Example re-exports (will be uncommented/adjusted as types are implemented)
pub use self::types::GlobalDesktopSettings; // Uncommented
pub use self::paths::SettingPath; // Uncommented
pub use self::errors::GlobalSettingsError; // Uncommented
pub use self::events::{SettingChangedEvent, SettingsLoadedEvent, SettingsSavedEvent}; // Uncommented
pub use self::service::{GlobalSettingsService, DefaultGlobalSettingsService}; // Updated
pub use self::persistence_iface::{SettingsPersistenceProvider, FilesystemSettingsProvider}; // Added
