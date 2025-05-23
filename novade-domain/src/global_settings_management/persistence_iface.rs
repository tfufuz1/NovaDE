use crate::global_settings_management::types::GlobalDesktopSettings;
use crate::global_settings_management::errors::GlobalSettingsError;
use async_trait::async_trait;

/// Defines the interface for a persistence provider that can load and save
/// global desktop settings.
#[async_trait]
pub trait SettingsPersistenceProvider: Send + Sync {
    /// Loads the global desktop settings from the persistence layer.
    ///
    /// # Returns
    /// A `Result` containing the loaded `GlobalDesktopSettings` or a `GlobalSettingsError`
    /// if loading fails (e.g., file not found, parsing error).
    async fn load_global_settings(&self) -> Result<GlobalDesktopSettings, GlobalSettingsError>;

    /// Saves the provided global desktop settings to the persistence layer.
    ///
    /// # Arguments
    /// * `settings` - A reference to the `GlobalDesktopSettings` to be saved.
    ///
    /// # Returns
    /// A `Result` indicating success or a `GlobalSettingsError` if saving fails
    /// (e.g., I/O error, serialization error).
    async fn save_global_settings(&self, settings: &GlobalDesktopSettings) -> Result<(), GlobalSettingsError>;
}
