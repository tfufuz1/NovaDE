use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, info, warn};
use novade_core::config::ConfigServiceAsync; // Assuming this path
use novade_core::errors::CoreError;         // Assuming this path

use crate::global_settings::errors::GlobalSettingsError;
use crate::global_settings::persistence_iface::SettingsPersistenceProvider;
use crate::global_settings::types::GlobalDesktopSettings;

pub struct FilesystemSettingsProvider {
    pub config_service: Arc<dyn ConfigServiceAsync>,
    pub config_key: String, // e.g., "global_settings.toml"
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
        debug!("Loading global settings from filesystem using key: {}", self.config_key);
        match self.config_service.read_config_file_string(&self.config_key).await {
            Ok(toml_string) => {
                info!("Successfully read settings file for key: {}", self.config_key);
                toml::from_str(&toml_string).map_err(|e| {
                    warn!("Failed to deserialize TOML settings for key '{}': {}", self.config_key, e);
                    GlobalSettingsError::TomlDeserializationError(e)
                })
            }
            Err(core_error) => {
                // Check if the error is because the file was not found
                // This depends on how CoreError is structured. Let's assume CoreError has a method like `is_not_found()`
                // or we match on a specific error variant if CoreError is an enum.
                // For this example, let's assume any read error means "try default",
                // but log a warning if it's not a simple "not found" type of error.
                if core_error.is_not_found_error() { // Hypothetical method
                    info!("Settings file for key '{}' not found. Returning default settings.", self.config_key);
                } else {
                    warn!(
                        "CoreError encountered while reading settings file for key '{}': {}. Returning default settings.",
                        self.config_key, core_error
                    );
                }
                Ok(GlobalDesktopSettings::default())
            }
        }
    }

    async fn save_global_settings(&self, settings: &GlobalDesktopSettings) -> Result<(), GlobalSettingsError> {
        debug!("Saving global settings to filesystem using key: {}", self.config_key);
        let toml_string = toml::to_string_pretty(settings).map_err(|e| {
            warn!("Failed to serialize settings to TOML for key '{}': {}", self.config_key, e);
            GlobalSettingsError::TomlSerializationError(e)
        })?;

        self.config_service
            .write_config_file_string(&self.config_key, toml_string)
            .await
            .map_err(|core_error| {
                GlobalSettingsError::persistence_error_from_core(
                    "save_global_settings".to_string(),
                    format!("Failed to write settings for key '{}'", self.config_key),
                    core_error,
                )
            })?;
        info!("Successfully saved global settings for key: {}", self.config_key);
        Ok(())
    }
}

// Mock for CoreError's is_not_found_error for compilation.
// This should be part of the actual CoreError definition.
#[cfg(test)]
mod core_error_mock {
    use std::fmt;
    use thiserror::Error;

    #[derive(Error, Debug, Clone)]
    pub enum CoreErrorType {
        NotFound,
        IoError,
        Other(String),
    }

    #[derive(Error, Debug, Clone)]
    pub struct CoreError {
        pub error_type: CoreErrorType,
        message: String,
        // source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    }

    impl fmt::Display for CoreError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "CoreError ({:?}): {}", self.error_type, self.message)
        }
    }


    impl CoreError {
        pub fn new(error_type: CoreErrorType, message: String) -> Self {
            Self { error_type, message }
        }
        pub fn is_not_found_error(&self) -> bool {
            matches!(self.error_type, CoreErrorType::NotFound)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_settings::types::{AppearanceSettings, ColorScheme, InputBehaviorSettings};
    use crate::global_settings::providers::filesystem_provider::core_error_mock::CoreError as MockCoreError; // Use the mock
    use crate::global_settings::providers::filesystem_provider::core_error_mock::CoreErrorType as MockCoreErrorType; // Use the mock


    // Mock ConfigServiceAsync
    #[derive(Default)]
    struct MockConfigService {
        files: std::collections::HashMap<String, String>,
        force_read_error: Option<MockCoreError>,
        force_write_error: Option<MockCoreError>,
    }

    impl MockConfigService {
        fn new() -> Self {
            Self::default()
        }

        #[allow(dead_code)] // Used in tests
        fn set_file_content(&mut self, key: &str, content: String) {
            self.files.insert(key.to_string(), content);
        }
        
        #[allow(dead_code)] // Used in tests
        fn set_read_error(&mut self, error: MockCoreError) {
            self.force_read_error = Some(error);
        }

        #[allow(dead_code)] // Used in tests
        fn set_write_error(&mut self, error: MockCoreError) {
            self.force_write_error = Some(error);
        }
    }

    #[async_trait]
    impl ConfigServiceAsync for MockConfigService {
        async fn read_config_file_string(&self, key: &str) -> Result<String, novade_core::errors::CoreError> {
             if let Some(ref err) = self.force_read_error {
                return Err(err.clone());
            }
            self.files.get(key)
                .cloned()
                .ok_or_else(|| MockCoreError::new(MockCoreErrorType::NotFound, format!("File not found: {}", key)))
        }

        async fn write_config_file_string(&self, key: &str, content: String) -> Result<(), novade_core::errors::CoreError> {
            if let Some(ref err) = self.force_write_error {
                return Err(err.clone());
            }
            // In a real mock, you might want to store this to check it was called.
            // For simplicity, we're not using self.files here for write.
            println!("MockConfigService: write_config_file_string called for key {} with content:\n{}", key, content);
            Ok(())
        }
        
        // Required by the trait being used in other modules, even if not directly by this provider's tests
        async fn read_file_to_string(&self, _path: &std::path::Path) -> Result<String, novade_core::errors::CoreError> {
            unimplemented!("read_file_to_string not needed for these specific tests")
        }
        async fn list_files_in_dir(&self, _dir_path: &std::path::Path, _extension: Option<&str>) -> Result<Vec<std::path::PathBuf>, novade_core::errors::CoreError> {
             unimplemented!("list_files_in_dir not needed for these specific tests")
        }
        async fn get_config_dir(&self) -> std::path::PathBuf {
            unimplemented!("get_config_dir not needed for these specific tests")
        }
        async fn get_data_dir(&self) -> std::path::PathBuf {
            unimplemented!("get_data_dir not needed for these specific tests")
        }

    }


    #[tokio::test]
    async fn test_load_global_settings_file_not_found() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemSettingsProvider::new(mock_config_service, "test_settings.toml".to_string());
        
        let settings = provider.load_global_settings().await.unwrap();
        assert_eq!(settings, GlobalDesktopSettings::default());
    }

    #[tokio::test]
    async fn test_load_global_settings_other_read_error() {
        let mut mock_service_inner = MockConfigService::new();
        mock_service_inner.set_read_error(MockCoreError::new(MockCoreErrorType::IoError, "Disk read failure".to_string()));
        let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemSettingsProvider::new(mock_config_service, "test_settings.toml".to_string());
        
        let settings = provider.load_global_settings().await.unwrap();
        // Should still return default if there's a read error other than NotFound
        assert_eq!(settings, GlobalDesktopSettings::default());
    }

    #[tokio::test]
    async fn test_load_global_settings_success() {
        let mut mock_service_inner = MockConfigService::new();
        let expected_settings = GlobalDesktopSettings {
            appearance: AppearanceSettings {
                active_theme_name: "custom_theme".to_string(),
                color_scheme: ColorScheme::Dark,
                enable_animations: false,
            },
            input_behavior: InputBehaviorSettings {
                mouse_sensitivity: 1.5,
                natural_scrolling_touchpad: false,
            },
        };
        let toml_content = toml::to_string_pretty(&expected_settings).unwrap();
        mock_service_inner.set_file_content("test_settings.toml", toml_content);
        let mock_config_service = Arc::new(mock_service_inner);

        let provider = FilesystemSettingsProvider::new(mock_config_service, "test_settings.toml".to_string());
        
        let loaded_settings = provider.load_global_settings().await.unwrap();
        assert_eq!(loaded_settings, expected_settings);
    }

    #[tokio::test]
    async fn test_load_global_settings_deserialization_error() {
        let mut mock_service_inner = MockConfigService::new();
        mock_service_inner.set_file_content("test_settings.toml", "this is not valid toml content {".to_string());
         let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemSettingsProvider::new(mock_config_service, "test_settings.toml".to_string());
        
        let result = provider.load_global_settings().await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::TomlDeserializationError(_) => {} // Expected
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_save_global_settings_success() {
        let mock_config_service = Arc::new(MockConfigService::new()); // Writes are just printed in mock
        let provider = FilesystemSettingsProvider::new(mock_config_service, "test_settings.toml".to_string());
        let settings_to_save = GlobalDesktopSettings::default();
        
        let result = provider.save_global_settings(&settings_to_save).await;
        assert!(result.is_ok());
        // In a more complex mock, you'd check that MockConfigService received the correct content.
    }

    #[tokio::test]
    async fn test_save_global_settings_write_error() {
        let mut mock_service_inner = MockConfigService::new();
        mock_service_inner.set_write_error(MockCoreError::new(MockCoreErrorType::IoError, "Disk write failure".to_string()));
        let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemSettingsProvider::new(mock_config_service, "test_settings.toml".to_string());
        let settings_to_save = GlobalDesktopSettings::default();

        let result = provider.save_global_settings(&settings_to_save).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::PersistenceError { operation, message, source } => {
                assert_eq!(operation, "save_global_settings");
                assert!(message.contains("Failed to write settings"));
                assert!(source.is_some());
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    // Serialization error for save is harder to trigger with GlobalDesktopSettings
    // as it should always serialize correctly if its fields do.
    // A custom type that can fail serialization would be needed for a direct test.
}
