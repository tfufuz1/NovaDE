use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, info, warn};
use novade_core::config::ConfigServiceAsync; // Assuming this path
use novade_core::errors::CoreError;         // Assuming this path

use super::errors::WorkspaceConfigError;
use super::types::WorkspaceSetSnapshot;

#[async_trait]
pub trait WorkspaceConfigProvider: Send + Sync {
    async fn load_workspace_config(&self) -> Result<WorkspaceSetSnapshot, WorkspaceConfigError>;
    async fn save_workspace_config(&self, config_snapshot: &WorkspaceSetSnapshot) -> Result<(), WorkspaceConfigError>;
}

pub struct FilesystemConfigProvider {
    pub config_service: Arc<dyn ConfigServiceAsync>,
    pub config_key: String, // e.g., "workspaces_config.toml"
}

impl FilesystemConfigProvider {
    pub fn new(config_service: Arc<dyn ConfigServiceAsync>, config_key: String) -> Self {
        Self {
            config_service,
            config_key,
        }
    }
}

#[async_trait]
impl WorkspaceConfigProvider for FilesystemConfigProvider {
    async fn load_workspace_config(&self) -> Result<WorkspaceSetSnapshot, WorkspaceConfigError> {
        debug!("Loading workspace config from filesystem using key: {}", self.config_key);
        match self.config_service.read_config_file_string(&self.config_key).await {
            Ok(toml_string) => {
                info!("Successfully read workspace config file for key: {}", self.config_key);
                toml::from_str(&toml_string).map_err(|e| {
                    warn!("Failed to deserialize TOML workspace config for key '{}': {}", self.config_key, e);
                    WorkspaceConfigError::DeserializationError {
                        message: format!("Failed to parse TOML for key '{}'", self.config_key),
                        snippet: Some(toml_string.chars().take(200).collect()), // First 200 chars
                        source: Some(e),
                    }
                })
            }
            Err(core_error) => {
                if core_error.is_not_found_error() { // Hypothetical method from CoreError
                    info!("Workspace config file for key '{}' not found. Returning default snapshot.", self.config_key);
                    Ok(WorkspaceSetSnapshot::default())
                } else {
                    warn!(
                        "CoreError encountered while reading workspace config file for key '{}': {}. Returning default snapshot.",
                        self.config_key, core_error
                    );
                    // Depending on strictness, might want to return an error instead of default for non-NotFound errors.
                    // For Iteration 1, returning default is acceptable for robustness.
                     Err(WorkspaceConfigError::LoadError {
                        path: self.config_key.clone(),
                        source: core_error,
                    })
                    // Ok(WorkspaceSetSnapshot::default()) // Or keep returning default for any read error
                }
            }
        }
    }

    async fn save_workspace_config(&self, config_snapshot: &WorkspaceSetSnapshot) -> Result<(), WorkspaceConfigError> {
        debug!("Saving workspace config to filesystem using key: {}", self.config_key);
        
        // Validate before saving (optional here, but good practice)
        if let Err(reason) = config_snapshot.validate() {
            return Err(WorkspaceConfigError::InvalidData { reason, path: None });
        }

        let toml_string = toml::to_string_pretty(config_snapshot).map_err(|e| {
            warn!("Failed to serialize workspace config to TOML for key '{}': {}", self.config_key, e);
            WorkspaceConfigError::SerializationError {
                message: format!("Failed to serialize workspace config for key '{}'", self.config_key),
                source: Some(e),
            }
        })?;

        self.config_service
            .write_config_file_string(&self.config_key, toml_string)
            .await
            .map_err(|core_error| {
                WorkspaceConfigError::SaveError {
                    path: self.config_key.clone(),
                    source: core_error,
                }
            })?;
        info!("Successfully saved workspace config for key: {}", self.config_key);
        Ok(())
    }
}

// Mock for CoreError's is_not_found_error for compilation.
// This should be part of the actual CoreError definition.
// Using the one from global_settings/providers/filesystem_provider.rs for now.
// Ideally, this would be a shared mock or defined in novade_core itself for testing.
#[cfg(test)]
mod core_error_mock_for_workspaces {
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
    use crate::workspaces::config::types::{WorkspaceSnapshot, WorkspaceSetSnapshot};
    use crate::workspaces::core::types::WorkspaceLayoutType; // Corrected path
    use crate::workspaces::config::provider::core_error_mock_for_workspaces::CoreError as MockCoreError;
    use crate::workspaces::config::provider::core_error_mock_for_workspaces::CoreErrorType as MockCoreErrorType;


    // Mock ConfigServiceAsync
    #[derive(Default)]
    struct MockConfigService {
        files: std::collections::HashMap<String, String>,
        force_read_error_type: Option<MockCoreErrorType>,
        force_write_error_type: Option<MockCoreErrorType>,
    }

    impl MockConfigService {
        fn new() -> Self {
            Self::default()
        }

        fn set_file_content(&mut self, key: &str, content: String) {
            self.files.insert(key.to_string(), content);
        }
        
        fn set_read_error_type(&mut self, error_type: Option<MockCoreErrorType>) {
            self.force_read_error_type = error_type;
        }

        fn set_write_error_type(&mut self, error_type: Option<MockCoreErrorType>) {
            self.force_write_error_type = error_type;
        }
    }

    #[async_trait]
    impl ConfigServiceAsync for MockConfigService {
        async fn read_config_file_string(&self, key: &str) -> Result<String, novade_core::errors::CoreError> {
             if let Some(ref err_type) = self.force_read_error_type {
                return Err(MockCoreError::new(err_type.clone(), format!("Forced read error on {}", key)));
            }
            self.files.get(key)
                .cloned()
                .ok_or_else(|| MockCoreError::new(MockCoreErrorType::NotFound, format!("File not found: {}", key)))
        }

        async fn write_config_file_string(&self, key: &str, content: String) -> Result<(), novade_core::errors::CoreError> {
            if let Some(ref err_type) = self.force_write_error_type {
                 return Err(MockCoreError::new(err_type.clone(), format!("Forced write error on {}", key)));
            }
            // In a real mock, you might want to store this to check it was called.
            println!("MockConfigService: write_config_file_string called for key {} with content:\n{}", key, content);
            Ok(())
        }
        
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
    async fn test_load_workspace_config_file_not_found() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());
        
        let snapshot = provider.load_workspace_config().await.unwrap();
        assert_eq!(snapshot, WorkspaceSetSnapshot::default());
    }

    #[tokio::test]
    async fn test_load_workspace_config_other_read_error() {
        let mut mock_service_inner = MockConfigService::new();
        mock_service_inner.set_read_error_type(Some(MockCoreErrorType::IoError));
        let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());
        
        let result = provider.load_workspace_config().await;
        assert!(result.is_err());
        match result.err().unwrap() {
            WorkspaceConfigError::LoadError { .. } => {} // Expected
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_load_workspace_config_success() {
        let mut mock_service_inner = MockConfigService::new();
        let expected_snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { name: "WS1".to_string(), layout_type: WorkspaceLayoutType::Floating },
                WorkspaceSnapshot { name: "WS2".to_string(), layout_type: WorkspaceLayoutType::TilingHorizontal },
            ],
            active_workspace_index: Some(0),
        };
        let toml_content = toml::to_string_pretty(&expected_snapshot).unwrap();
        mock_service_inner.set_file_content("ws_test_config.toml", toml_content);
        let mock_config_service = Arc::new(mock_service_inner);

        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());
        
        let loaded_snapshot = provider.load_workspace_config().await.unwrap();
        assert_eq!(loaded_snapshot, expected_snapshot);
    }

    #[tokio::test]
    async fn test_load_workspace_config_deserialization_error() {
        let mut mock_service_inner = MockConfigService::new();
        mock_service_inner.set_file_content("ws_test_config.toml", "this is not valid toml {".to_string());
        let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());
        
        let result = provider.load_workspace_config().await;
        assert!(result.is_err());
        match result.err().unwrap() {
            WorkspaceConfigError::DeserializationError { source, .. } => {
                assert!(source.is_some());
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_save_workspace_config_success() {
        let mock_config_service = Arc::new(MockConfigService::new()); // Writes are just printed in mock
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());
        let snapshot_to_save = WorkspaceSetSnapshot::default();
        
        let result = provider.save_workspace_config(&snapshot_to_save).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_save_workspace_config_write_error() {
        let mut mock_service_inner = MockConfigService::new();
        mock_service_inner.set_write_error_type(Some(MockCoreErrorType::IoError));
        let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());
        let snapshot_to_save = WorkspaceSetSnapshot::default();

        let result = provider.save_workspace_config(&snapshot_to_save).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            WorkspaceConfigError::SaveError { source, .. } => {
                assert!(source.is_not_found_error() == false); // Should be the IoError we set
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[test]
    fn test_workspace_set_snapshot_validate_in_provider_save() {
        // This test logic is implicitly covered by save_workspace_config if it calls validate.
        // If save_workspace_config directly calls validate, an invalid snapshot should fail there.
        let invalid_snapshot = WorkspaceSetSnapshot {
            workspaces: vec![WorkspaceSnapshot { name: "WS1".to_string(), layout_type: WorkspaceLayoutType::Floating }],
            active_workspace_index: Some(1), // Invalid index
        };
        // The FilesystemConfigProvider's save_workspace_config calls validate.
        // So, trying to save this should result in WorkspaceConfigError::InvalidData.
        // This test is more about the provider's behavior with validation.
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());

        // We need an async context to call save_workspace_config
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(provider.save_workspace_config(&invalid_snapshot));
        
        assert!(result.is_err());
        match result.err().unwrap() {
            WorkspaceConfigError::InvalidData { reason, .. } => {
                assert!(reason.contains("active_workspace_index"));
            }
            e => panic!("Expected InvalidData error, got {:?}", e),
        }
    }
}
