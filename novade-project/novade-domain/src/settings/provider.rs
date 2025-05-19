//! Settings provider module for the NovaDE domain layer.
//!
//! This module provides interfaces and implementations for loading
//! and saving settings in the NovaDE desktop environment.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::fs;
use tokio::fs as tokio_fs;
use crate::error::{DomainResult, SettingsError};
use crate::settings::core::Setting;

/// Interface for providing settings.
#[async_trait]
pub trait SettingsProvider: Send + Sync {
    /// Loads all settings.
    ///
    /// # Returns
    ///
    /// A vector of settings, or an error if loading failed.
    async fn load_settings(&self) -> DomainResult<Vec<Setting>>;
    
    /// Saves settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - The settings to save
    ///
    /// # Returns
    ///
    /// `Ok(())` if the settings were saved, or an error if saving failed.
    async fn save_settings(&self, settings: &[Setting]) -> DomainResult<()>;
}

/// File-based settings provider.
pub struct FileSettingsProvider {
    /// The path to the settings file.
    settings_file: PathBuf,
}

impl FileSettingsProvider {
    /// Creates a new file-based settings provider.
    ///
    /// # Arguments
    ///
    /// * `settings_file` - The path to the settings file
    ///
    /// # Returns
    ///
    /// A new `FileSettingsProvider`.
    pub fn new(settings_file: impl Into<PathBuf>) -> Self {
        FileSettingsProvider {
            settings_file: settings_file.into(),
        }
    }
    
    /// Ensures the settings directory exists.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the directory exists or was created, or an error if creation failed.
    fn ensure_directory(&self) -> DomainResult<()> {
        if let Some(parent) = self.settings_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| SettingsError::IoError(e.to_string()))?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl SettingsProvider for FileSettingsProvider {
    async fn load_settings(&self) -> DomainResult<Vec<Setting>> {
        if !self.settings_file.exists() {
            return Ok(Vec::new());
        }
        
        let content = tokio_fs::read_to_string(&self.settings_file).await
            .map_err(|e| SettingsError::LoadFailed(e.to_string()))?;
        
        let settings: Vec<Setting> = serde_json::from_str(&content)
            .map_err(|e| SettingsError::LoadFailed(e.to_string()))?;
        
        Ok(settings)
    }
    
    async fn save_settings(&self, settings: &[Setting]) -> DomainResult<()> {
        self.ensure_directory()?;
        
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| SettingsError::SaveFailed(e.to_string()))?;
        
        tokio_fs::write(&self.settings_file, content).await
            .map_err(|e| SettingsError::SaveFailed(e.to_string()))?;
        
        Ok(())
    }
}

/// In-memory settings provider for testing.
pub struct InMemorySettingsProvider {
    /// The settings.
    settings: Vec<Setting>,
}

impl InMemorySettingsProvider {
    /// Creates a new in-memory settings provider.
    ///
    /// # Returns
    ///
    /// A new `InMemorySettingsProvider`.
    pub fn new() -> Self {
        InMemorySettingsProvider {
            settings: Vec::new(),
        }
    }
    
    /// Creates a new in-memory settings provider with initial settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - The initial settings
    ///
    /// # Returns
    ///
    /// A new `InMemorySettingsProvider` with initial settings.
    pub fn with_settings(settings: Vec<Setting>) -> Self {
        InMemorySettingsProvider {
            settings,
        }
    }
}

#[async_trait]
impl SettingsProvider for InMemorySettingsProvider {
    async fn load_settings(&self) -> DomainResult<Vec<Setting>> {
        Ok(self.settings.clone())
    }
    
    async fn save_settings(&self, settings: &[Setting]) -> DomainResult<()> {
        // In a real implementation, this would save the settings
        // For testing, we just log the settings
        println!("Saving {} settings", settings.len());
        
        Ok(())
    }
}

impl Default for InMemorySettingsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::core::{SettingKey, SettingCategory};
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_file_settings_provider() {
        let temp_dir = TempDir::new().unwrap();
        let settings_file = temp_dir.path().join("settings.json");
        
        let provider = FileSettingsProvider::new(&settings_file);
        
        // Initially, no settings
        let settings = provider.load_settings().await.unwrap();
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
        provider.save_settings(&settings).await.unwrap();
        
        // Load the settings again
        let loaded = provider.load_settings().await.unwrap();
        assert_eq!(loaded.len(), 2);
        
        // Verify the settings
        let language = loaded.iter().find(|s| s.key().name == "language").unwrap();
        assert_eq!(language.value().as_string(), Some("en-US"));
        
        let theme = loaded.iter().find(|s| s.key().name == "theme").unwrap();
        assert_eq!(theme.value().as_string(), Some("light"));
    }
    
    #[tokio::test]
    async fn test_in_memory_settings_provider() {
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
        
        let provider = InMemorySettingsProvider::with_settings(settings.clone());
        
        // Load the settings
        let loaded = provider.load_settings().await.unwrap();
        assert_eq!(loaded.len(), 2);
        
        // Verify the settings
        let language = loaded.iter().find(|s| s.key().name == "language").unwrap();
        assert_eq!(language.value().as_string(), Some("en-US"));
        
        let theme = loaded.iter().find(|s| s.key().name == "theme").unwrap();
        assert_eq!(theme.value().as_string(), Some("light"));
        
        // Save the settings
        provider.save_settings(&settings).await.unwrap();
    }
}
