use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, warn};

use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;

use super::types::GlobalDesktopSettings;
use super::errors::GlobalSettingsError;
use super::paths::SettingPath; // Will be updated with Root variant

// --- SettingsPersistenceProvider Trait ---

#[async_trait]
pub trait SettingsPersistenceProvider: Send + Sync {
    async fn load_global_settings(&self) -> Result<GlobalDesktopSettings, GlobalSettingsError>;
    async fn save_global_settings(&self, settings: &GlobalDesktopSettings) -> Result<(), GlobalSettingsError>;
}

// --- FilesystemSettingsProvider Implementation ---

pub struct FilesystemSettingsProvider {
    config_service: Arc<dyn ConfigServiceAsync>,
    config_key: String,
}

impl FilesystemSettingsProvider {
    pub fn new(config_service: Arc<dyn ConfigServiceAsync>, config_key: String) -> Self {
        Self {
            config_service,
            config_key,
        }
    }
}

#[async_trait]
impl SettingsPersistenceProvider for FilesystemSettingsProvider {
    async fn load_global_settings(&self) -> Result<GlobalDesktopSettings, GlobalSettingsError> {
        debug!("Loading global settings from key: {}", self.config_key);
        match self.config_service.read_config_file_string(&self.config_key).await {
            Ok(content) => {
                debug!("Successfully read settings file content for key: {}", self.config_key);
                let settings: GlobalDesktopSettings = toml::from_str(&content).map_err(|e| {
                    warn!("Failed to deserialize settings from key '{}': {}", self.config_key, e);
                    GlobalSettingsError::DeserializationError {
                        path: SettingPath::Root, // Using new Root variant
                        source: e,
                    }
                })?;

                debug!("Successfully deserialized settings for key: {}. Validating...", self.config_key);
                // validate_recursive will be updated to use new SettingPath variants
                settings.validate_recursive().map_err(|e| { 
                     warn!("Validation failed for deserialized settings from key '{}': {:?}", self.config_key, e);
                     e 
                })?;
                
                debug!("Settings validated successfully for key: {}", self.config_key);
                Ok(settings)
            }
            Err(e) => {
                // Assuming CoreError has is_not_found() or similar functionality
                if e.is_not_found() { 
                    debug!("Settings file not found for key '{}'. Returning default settings.", self.config_key);
                    Ok(GlobalDesktopSettings::default())
                } else {
                    warn!("Failed to read settings file for key '{}': {}", self.config_key, e);
                    Err(GlobalSettingsError::PersistenceError {
                        operation: "load".to_string(),
                        message: format!("Failed to read settings from config service for key '{}'", self.config_key),
                        source: Some(e),
                    })
                }
            }
        }
    }

    async fn save_global_settings(&self, settings: &GlobalDesktopSettings) -> Result<(), GlobalSettingsError> {
        debug!("Saving global settings to key: {}", self.config_key);
        
        // Validate before saving
        settings.validate_recursive().map_err(|e| {
            warn!("Validation failed before saving settings for key '{}': {:?}", self.config_key, e);
            e
        })?;

        let serialized_content = toml::to_string_pretty(settings).map_err(|e| {
            warn!("Failed to serialize settings for key '{}': {}", self.config_key, e);
            GlobalSettingsError::SerializationError {
                path: SettingPath::Root, // Using new Root variant
                source: e,
            }
        })?;

        self.config_service
            .write_config_file_string(&self.config_key, &serialized_content)
            .await
            .map_err(|e| {
                warn!("Failed to write settings to config service for key '{}': {}", self.config_key, e);
                GlobalSettingsError::PersistenceError {
                    operation: "save".to_string(),
                    message: format!("Failed to write settings to config service for key '{}'", self.config_key),
                    source: Some(e),
                }
            })?;
        debug!("Global settings saved successfully to key: {}", self.config_key);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::config::MockConfigServiceAsync;
    use crate::global_settings::types::*;
    use crate::global_settings::paths::{AppearanceSettingPath, FontSettingPath, WorkspaceSettingPath, InputBehaviorSettingPath, PowerManagementPolicySettingPath, DefaultApplicationsSettingPath};
    use std::io;

    fn new_mock_arc_config_service() -> Arc<MockConfigServiceAsync> {
        Arc::new(MockConfigServiceAsync::new())
    }

    #[tokio::test]
    async fn test_load_global_settings_success() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let settings = GlobalDesktopSettings::default();
        let settings_toml = toml::to_string_pretty(&settings).unwrap();

        mock_config_service.expect_read_config_file_string()
            .withf(|key| key == "test_settings.toml")
            .returning(move |_| Ok(settings_toml.clone()));

        let provider = FilesystemSettingsProvider::new(Arc::new(mock_config_service), "test_settings.toml".to_string());
        let loaded_settings = provider.load_global_settings().await.unwrap();
        assert_eq!(loaded_settings, settings);
    }

    #[tokio::test]
    async fn test_load_global_settings_not_found_returns_default() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let not_found_error = CoreError::IoError(
            "Simulated file not found".to_string(), 
            Some(Arc::new(io::Error::new(io::ErrorKind::NotFound, "not found")))
        );

        mock_config_service.expect_read_config_file_string()
            .returning(move |_| Err(not_found_error.clone()));

        let provider = FilesystemSettingsProvider::new(Arc::new(mock_config_service), "non_existent.toml".to_string());
        let loaded_settings = provider.load_global_settings().await.unwrap();
        assert_eq!(loaded_settings, GlobalDesktopSettings::default());
    }

    #[tokio::test]
    async fn test_load_global_settings_corrupted_content() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let corrupted_toml = "this is not valid toml content {{{{";

        mock_config_service.expect_read_config_file_string()
            .returning(move |_| Ok(corrupted_toml.to_string()));

        let provider = FilesystemSettingsProvider::new(Arc::new(mock_config_service), "corrupted.toml".to_string());
        let result = provider.load_global_settings().await;
        assert!(matches!(result, Err(GlobalSettingsError::DeserializationError { path: SettingPath::Root, .. })));
    }

    #[tokio::test]
    async fn test_load_global_settings_validation_error() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let mut invalid_settings = GlobalDesktopSettings::default();
        invalid_settings.appearance.interface_scaling_factor = 0.1; // Invalid value

        let invalid_settings_toml = toml::to_string_pretty(&invalid_settings).unwrap();
        mock_config_service.expect_read_config_file_string()
            .returning(move |_| Ok(invalid_settings_toml.clone()));

        let provider = FilesystemSettingsProvider::new(Arc::new(mock_config_service), "invalid_data.toml".to_string());
        let result = provider.load_global_settings().await;
        
        assert!(matches!(result, Err(GlobalSettingsError::ValidationError { .. })));
        if let Err(GlobalSettingsError::ValidationError { path, reason }) = result {
            assert_eq!(path, SettingPath::AppearanceRoot); 
            assert!(reason.contains("Interface scaling factor"));
        } else {
            panic!("Expected ValidationError, got {:?}", result);
        }
    }
    
    #[tokio::test]
    async fn test_load_global_settings_other_read_error() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let generic_io_error = CoreError::IoError(
            "Simulated generic IO error".to_string(), 
            Some(Arc::new(io::Error::new(io::ErrorKind::Other, "other error")))
        );
        mock_config_service.expect_read_config_file_string()
            .returning(move |_| Err(generic_io_error.clone()));

        let provider = FilesystemSettingsProvider::new(Arc::new(mock_config_service), "error.toml".to_string());
        let result = provider.load_global_settings().await;
        assert!(matches!(result, Err(GlobalSettingsError::PersistenceError { operation, .. }) if operation == "load"));
    }

    #[tokio::test]
    async fn test_save_global_settings_success() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let settings = GlobalDesktopSettings::default();
        
        mock_config_service.expect_write_config_file_string()
            .withf(move |key, content| {
                key == "test_save.toml" && 
                toml::from_str::<GlobalDesktopSettings>(content).unwrap() == settings
            })
            .returning(|_, _| Ok(()));

        let provider = FilesystemSettingsProvider::new(Arc::new(mock_config_service), "test_save.toml".to_string());
        let result = provider.save_global_settings(&settings).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_save_global_settings_validation_error_before_saving() {
        let mock_config_service = new_mock_arc_config_service(); // No calls expected
        let mut invalid_settings = GlobalDesktopSettings::default();
        invalid_settings.appearance.active_theme_name = "".to_string(); // Invalid

        let provider = FilesystemSettingsProvider::new(mock_config_service, "validation_fail_save.toml".to_string());
        let result = provider.save_global_settings(&invalid_settings).await;

        assert!(matches!(result, Err(GlobalSettingsError::ValidationError { .. })));
        if let Err(GlobalSettingsError::ValidationError { path, reason }) = result {
            assert_eq!(path, SettingPath::AppearanceRoot);
            assert!(reason.contains("Active theme name cannot be empty"));
        } else {
            panic!("Expected ValidationError, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_save_global_settings_write_error() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let settings = GlobalDesktopSettings::default();
        let write_error = CoreError::IoError("Simulated write error".to_string(), None);

        mock_config_service.expect_write_config_file_string()
            .returning(move |_, _| Err(write_error.clone()));
            
        let provider = FilesystemSettingsProvider::new(Arc::new(mock_config_service), "write_fail.toml".to_string());
        let result = provider.save_global_settings(&settings).await;
        assert!(matches!(result, Err(GlobalSettingsError::PersistenceError { operation, .. }) if operation == "save"));
    }
}
