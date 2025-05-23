#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::collections::HashMap; // For mock file storage
    use async_trait::async_trait;
    use tempfile::NamedTempFile; // For creating temporary files for testing
    use std::fs;

    use novade_core::config::ConfigServiceAsync;
    use novade_core::errors::CoreError; // Assuming CoreError is clonable for mock
    // If CoreError is not Clone, the mock needs to handle it, e.g. by storing Arc<CoreError> or String.
    // For simplicity, assuming it's Clone or can be easily stringified for mock errors.

    use crate::global_settings_management::types::{GlobalDesktopSettings, AppearanceSettings, ColorScheme};
    use crate::global_settings_management::errors::GlobalSettingsError;
    use crate::global_settings_management::persistence::FilesystemSettingsProvider;
    use crate::global_settings_management::persistence_iface::SettingsPersistenceProvider;


    // --- Mock ConfigServiceAsync ---
    #[derive(Debug, Clone, Default)]
    struct MockConfigService {
        // Store content as String. For "not found", the file just won't be in the map.
        // For other errors, we'll use dedicated error flags.
        files: Arc<std::sync::Mutex<HashMap<String, String>>>,
        error_on_load: Option<CoreError>,
        error_on_save: Option<CoreError>,
    }

    impl MockConfigService {
        fn new() -> Self {
            Default::default()
        }

        // Helper to set file content for the mock
        #[allow(dead_code)] // Used in some tests
        fn add_file_content(&self, path: &str, content: String) {
            let mut files = self.files.lock().unwrap();
            files.insert(path.to_string(), content);
        }
        
        #[allow(dead_code)] // Used in some tests
        fn remove_file(&self, path: &str) {
            let mut files = self.files.lock().unwrap();
            files.remove(path);
        }

        #[allow(dead_code)] // Used in some tests
        fn set_error_on_load(&mut self, error: Option<CoreError>) {
            self.error_on_load = error;
        }
        
        #[allow(dead_code)] // Used in some tests
        fn set_error_on_save(&mut self, error: Option<CoreError>) {
            self.error_on_save = error;
        }
    }

    #[async_trait]
    impl ConfigServiceAsync for MockConfigService {
        // Simplified for test: path is converted to String for map key
        async fn load_config_file_content_async(&self, path_str: &str) -> Result<String, CoreError> {
            if let Some(err) = &self.error_on_load {
                return Err(err.clone());
            }
            let files = self.files.lock().unwrap();
            match files.get(path_str) {
                Some(content) => Ok(content.clone()),
                None => Err(CoreError::NotFound(path_str.to_string())),
            }
        }

        async fn save_config_file_content_async(&self, path_str: &str, content: &str) -> Result<(), CoreError> {
            if let Some(err) = &self.error_on_save {
                return Err(err.clone());
            }
            let mut files = self.files.lock().unwrap();
            files.insert(path_str.to_string(), content.to_string());
            Ok(())
        }
        
        // Other methods not used by these tests can have dummy implementations
        async fn list_config_files_async(&self, _dir_path: &str) -> Result<Vec<String>, CoreError> {
            unimplemented!("list_config_files_async not needed for these tests")
        }
        fn get_config_file_path(&self, _app_id: &crate::shared_types::ApplicationId, _config_name: &str, _format: Option<novade_core::config::ConfigFormat>) -> Result<String, CoreError> {
            unimplemented!("get_config_file_path not needed for these tests")
        }
        fn get_config_dir_path(&self, _app_id: &crate::shared_types::ApplicationId, _subdir: Option<&str>) -> Result<String, CoreError> {
            unimplemented!("get_config_dir_path not needed for these tests")
        }
         fn ensure_config_dir_exists(&self, _app_id: &crate::shared_types::ApplicationId) -> Result<String, CoreError> {
            unimplemented!("ensure_config_dir_exists not needed for these tests")
        }
    }

    fn test_path() -> PathBuf {
        // Using a fixed name for simplicity in mock, real temp file not strictly needed for mock.
        // If testing with real filesystem, NamedTempFile is good.
        PathBuf::from("test_settings.toml")
    }

    #[tokio::test]
    async fn test_load_settings_success() {
        let mock_service = Arc::new(MockConfigService::new());
        let settings_path = test_path();
        
        let mut expected_settings = GlobalDesktopSettings::default();
        expected_settings.appearance.color_scheme = ColorScheme::Dark;
        expected_settings.appearance.active_theme_name = "TestDark".to_string();
        let toml_content = toml::to_string_pretty(&expected_settings).unwrap();
        
        mock_service.add_file_content(settings_path.to_str().unwrap(), toml_content);

        let provider = FilesystemSettingsProvider::new(mock_service.clone(), settings_path);
        let loaded_settings = provider.load_global_settings().await.unwrap();
        
        assert_eq!(loaded_settings, expected_settings);
    }

    #[tokio::test]
    async fn test_load_settings_file_not_found_returns_default() {
        let mock_service = Arc::new(MockConfigService::new());
        let settings_path = test_path(); // This file won't be in the mock service

        let provider = FilesystemSettingsProvider::new(mock_service.clone(), settings_path);
        let loaded_settings = provider.load_global_settings().await.unwrap();
        
        assert_eq!(loaded_settings, GlobalDesktopSettings::default());
    }

    #[tokio::test]
    async fn test_load_settings_corrupted_toml() {
        let mock_service = Arc::new(MockConfigService::new());
        let settings_path = test_path();
        let corrupted_toml_content = "this is not valid toml content {{{{";
        
        mock_service.add_file_content(settings_path.to_str().unwrap(), corrupted_toml_content.to_string());

        let provider = FilesystemSettingsProvider::new(mock_service.clone(), settings_path);
        let result = provider.load_global_settings().await;
        
        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::DeserializationError { source_message: _ } => { /* Expected */ }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_load_settings_io_error() {
        let mut mock_service_inner = MockConfigService::new();
        // Simulate a core I/O error other than NotFound
        let core_io_error = CoreError::IoError("Simulated I/O error".to_string());
        mock_service_inner.set_error_on_load(Some(core_io_error.clone()));
        let mock_service = Arc::new(mock_service_inner);
        
        let settings_path = test_path();
        let provider = FilesystemSettingsProvider::new(mock_service.clone(), settings_path.clone());
        let result = provider.load_global_settings().await;
        
        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::PersistenceError { operation, message, source } => {
                assert_eq!(operation, "load");
                assert!(message.contains(&format!("{:?}", settings_path)));
                assert_eq!(source, Some(core_io_error.to_string()));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_save_settings_success() {
        let mock_service = Arc::new(MockConfigService::new());
        let settings_path = test_path();
        
        let settings_to_save = GlobalDesktopSettings {
            appearance: AppearanceSettings { color_scheme: ColorScheme::Light, ..Default::default() },
            ..Default::default()
        };

        let provider = FilesystemSettingsProvider::new(mock_service.clone(), settings_path.clone());
        let save_result = provider.save_global_settings(&settings_to_save).await;
        assert!(save_result.is_ok());

        // Verify content was "saved" to mock
        let files_map = mock_service.files.lock().unwrap();
        let saved_content = files_map.get(settings_path.to_str().unwrap()).unwrap();
        
        let deserialized_settings: GlobalDesktopSettings = toml::from_str(saved_content).unwrap();
        assert_eq!(deserialized_settings, settings_to_save);
    }

    #[tokio::test]
    async fn test_save_settings_io_error() {
        let mut mock_service_inner = MockConfigService::new();
        let core_io_error = CoreError::IoError("Simulated save I/O error".to_string());
        mock_service_inner.set_error_on_save(Some(core_io_error.clone()));
        let mock_service = Arc::new(mock_service_inner);

        let settings_path = test_path();
        let settings_to_save = GlobalDesktopSettings::default();
        
        let provider = FilesystemSettingsProvider::new(mock_service.clone(), settings_path.clone());
        let result = provider.save_global_settings(&settings_to_save).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::PersistenceError { operation, message, source } => {
                assert_eq!(operation, "save");
                assert!(message.contains(&format!("{:?}", settings_path)));
                assert_eq!(source, Some(core_io_error.to_string()));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    // Serialization errors from toml::to_string_pretty are harder to trigger with derive(Serialize)
    // unless there's a custom Serialize impl that can fail, or types like f32::NAN which TOML might reject.
    // GlobalDesktopSettings is unlikely to produce such an error with standard TOML.
}
