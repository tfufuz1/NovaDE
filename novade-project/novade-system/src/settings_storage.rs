//! Settings storage module for the NovaDE system layer.
//!
//! This module provides settings storage functionality for the NovaDE desktop environment,
//! persisting settings to the file system.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::fs;
use tokio::fs as tokio_fs;
use novade_domain::settings::core::{Setting, SettingKey, SettingCategory};
use novade_domain::settings::service::SettingsStorage as DomainSettingsStorage;
use crate::error::{SystemError, SystemResult, to_system_error, SystemErrorKind};

/// Settings storage interface.
#[async_trait]
pub trait SettingsStorage: Send + Sync {
    /// Loads all settings.
    ///
    /// # Returns
    ///
    /// A vector of settings, or an error if loading failed.
    async fn load_settings(&self) -> SystemResult<Vec<Setting>>;
    
    /// Saves settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - The settings to save
    ///
    /// # Returns
    ///
    /// `Ok(())` if the settings were saved, or an error if saving failed.
    async fn save_settings(&self, settings: &[Setting]) -> SystemResult<()>;
    
    /// Gets the settings file path.
    ///
    /// # Returns
    ///
    /// The settings file path.
    fn get_settings_path(&self) -> PathBuf;
}

/// File-based settings storage implementation.
pub struct FileSettingsStorage {
    /// The settings file path.
    settings_file: PathBuf,
}

impl FileSettingsStorage {
    /// Creates a new file-based settings storage.
    ///
    /// # Returns
    ///
    /// A new file-based settings storage.
    pub fn new() -> SystemResult<Self> {
        // Determine the settings directory
        let settings_dir = dirs::config_dir()
            .ok_or_else(|| to_system_error("Could not determine config directory", SystemErrorKind::SettingsStorage))?
            .join("novade");
        
        // Create the settings directory if it doesn't exist
        fs::create_dir_all(&settings_dir)
            .map_err(|e| to_system_error(format!("Could not create settings directory: {}", e), SystemErrorKind::SettingsStorage))?;
        
        let settings_file = settings_dir.join("settings.json");
        
        Ok(FileSettingsStorage {
            settings_file,
        })
    }
    
    /// Creates a new file-based settings storage with a custom path.
    ///
    /// # Arguments
    ///
    /// * `settings_file` - The settings file path
    ///
    /// # Returns
    ///
    /// A new file-based settings storage.
    pub fn with_path(settings_file: impl Into<PathBuf>) -> Self {
        FileSettingsStorage {
            settings_file: settings_file.into(),
        }
    }
    
    /// Ensures the settings directory exists.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the directory exists or was created, or an error if creation failed.
    fn ensure_directory(&self) -> SystemResult<()> {
        if let Some(parent) = self.settings_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| to_system_error(format!("Could not create settings directory: {}", e), SystemErrorKind::SettingsStorage))?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl SettingsStorage for FileSettingsStorage {
    async fn load_settings(&self) -> SystemResult<Vec<Setting>> {
        if !self.settings_file.exists() {
            return Ok(Vec::new());
        }
        
        let content = tokio_fs::read_to_string(&self.settings_file).await
            .map_err(|e| to_system_error(format!("Could not read settings file: {}", e), SystemErrorKind::SettingsStorage))?;
        
        let settings: Vec<Setting> = serde_json::from_str(&content)
            .map_err(|e| to_system_error(format!("Could not parse settings file: {}", e), SystemErrorKind::SettingsStorage))?;
        
        Ok(settings)
    }
    
    async fn save_settings(&self, settings: &[Setting]) -> SystemResult<()> {
        self.ensure_directory()?;
        
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| to_system_error(format!("Could not serialize settings: {}", e), SystemErrorKind::SettingsStorage))?;
        
        tokio_fs::write(&self.settings_file, content).await
            .map_err(|e| to_system_error(format!("Could not write settings file: {}", e), SystemErrorKind::SettingsStorage))?;
        
        Ok(())
    }
    
    fn get_settings_path(&self) -> PathBuf {
        self.settings_file.clone()
    }
}

#[async_trait]
impl DomainSettingsStorage for FileSettingsStorage {
    async fn load_settings(&self) -> novade_domain::error::DomainResult<Vec<Setting>> {
        let settings = <Self as SettingsStorage>::load_settings(self).await
            .map_err(|e| novade_domain::error::SettingsError::LoadFailed(e.to_string()))?;
        
        Ok(settings)
    }
    
    async fn save_settings(&self, settings: &[Setting]) -> novade_domain::error::DomainResult<()> {
        <Self as SettingsStorage>::save_settings(self, settings).await
            .map_err(|e| novade_domain::error::SettingsError::SaveFailed(e.to_string()))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_file_settings_storage() {
        let temp_dir = TempDir::new().unwrap();
        let settings_file = temp_dir.path().join("settings.json");
        
        let storage = FileSettingsStorage::with_path(&settings_file);
        
        // Initially, no settings
        let settings = storage.load_settings().await.unwrap();
        assert!(settings.is_empty());
        
        // Create some settings
        let settings = vec![
            Setting::new(
                SettingKey::new(SettingCategory::General, "language"),
                "en-US",
            ),
            Setting::new(
                SettingKey::new(SettingCategory::Appearance, "theme"),
                "light",
            ),
        ];
        
        // Save the settings
        storage.save_settings(&settings).await.unwrap();
        
        // Load the settings again
        let loaded = storage.load_settings().await.unwrap();
        assert_eq!(loaded.len(), 2);
        
        // Verify the settings
        let language = loaded.iter().find(|s| s.key().name == "language").unwrap();
        assert_eq!(language.value().as_string(), Some("en-US"));
        
        let theme = loaded.iter().find(|s| s.key().name == "theme").unwrap();
        assert_eq!(theme.value().as_string(), Some("light"));
        
        // Verify the settings path
        assert_eq!(storage.get_settings_path(), settings_file);
    }
}
