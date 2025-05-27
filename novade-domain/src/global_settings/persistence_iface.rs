use async_trait::async_trait;
use super::types::GlobalDesktopSettings;
use super::errors::GlobalSettingsError;

#[async_trait]
pub trait SettingsPersistenceProvider: Send + Sync {
    async fn load_global_settings(&self) -> Result<GlobalDesktopSettings, GlobalSettingsError>;
    async fn save_global_settings(&self, settings: &GlobalDesktopSettings) -> Result<(), GlobalSettingsError>;
}
