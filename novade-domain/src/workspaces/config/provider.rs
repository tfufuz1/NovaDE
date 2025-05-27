use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, info, warn};
use crate::ports::config_service::ConfigServiceAsync; // Corrected path
use novade_core::CoreError;         // Corrected path

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
                let snapshot: WorkspaceSetSnapshot = toml::from_str(&toml_string).map_err(|e| { // Removed mut for snapshot
                    warn!("Failed to deserialize TOML workspace config for key '{}': {}", self.config_key, e);
                    WorkspaceConfigError::DeserializationError {
                        message: format!("Failed to parse TOML for key '{}'", self.config_key),
                        snippet: Some(toml_string.chars().take(200).collect()), // First 200 chars
                        source: Some(e),
                    }
                })?;
                
                if let Err(reason) = snapshot.validate() {
                    warn!("Loaded workspace configuration for key '{}' is invalid: {}. Proceeding with caution or default.", self.config_key, reason);
                    // return Err(WorkspaceConfigError::InvalidData { reason, path: Some(self.config_key.clone()) }); // Option to be strict
                }
                Ok(snapshot)
            }
            Err(core_error) => {
                // Assuming CoreError has a method to check for "Not Found" like errors
                // This part needs CoreError to have a way to identify specific error types.
                // For example, if CoreError::Config(ConfigError::NotFound { .. })
                // Or a method core_error.is_not_found().
                // The previous `if core_error.is_not_found_error()` was hypothetical.
                // Let's match against specific CoreError variants if possible,
                // otherwise, we might have to adjust this logic or CoreError itself.
                // For now, I'll assume a direct match is not straightforward without CoreError's definition visible here.
                // The mock used a custom is_not_found_error. Real CoreError might differ.
                // Reverting to a more generic error handling for now, or assuming that specific check is difficult here.
                // The most robust way is to match CoreError variants.
                // If CoreError is an enum with a NotFound variant or similar:
                match core_error {
                    CoreError::Config(config_error) => { // Assuming CoreError has a Config variant
                        if let novade_core::ConfigError::NotFound { .. } = config_error {
                             info!("Workspace config file for key '{}' not found. Returning default snapshot.", self.config_key);
                             Ok(WorkspaceSetSnapshot::default())
                        } else {
                            warn!(
                                "ConfigError encountered while reading workspace config file for key '{}': {}. Returning LoadError.",
                                self.config_key, config_error
                            );
                            Err(WorkspaceConfigError::LoadError {
                                path: self.config_key.clone(),
                                source: CoreError::Config(config_error), // Re-wrap if necessary
                            })
                        }
                    }
                    _ => { // Generic fallback for other CoreError types
                        warn!(
                            "CoreError encountered while reading workspace config file for key '{}': {}. Returning LoadError.",
                            self.config_key, core_error
                        );
                        Err(WorkspaceConfigError::LoadError {
                            path: self.config_key.clone(),
                            source: core_error,
                        })
                    }
                }
            }
        }
    }

    async fn save_workspace_config(&self, config_snapshot: &WorkspaceSetSnapshot) -> Result<(), WorkspaceConfigError> {
        debug!("Saving workspace config to filesystem using key: {}", self.config_key);
        
        if let Err(reason) = config_snapshot.validate() {
            return Err(WorkspaceConfigError::InvalidData { reason, path: Some(self.config_key.clone()) });
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

#[cfg(test)]
mod core_error_mock_for_workspaces { // This mock is local to this test module
    use std::fmt;
    use thiserror::Error;

    #[derive(Error, Debug, Clone)]
    pub enum CoreErrorType { // Renamed to avoid conflict if novade_core::CoreError is ever in scope directly
        NotFound,
        IoError,
        Other(String),
    }

    #[derive(Error, Debug, Clone)]
    pub struct TestMockCoreError { // Renamed to avoid conflict
        pub error_type: CoreErrorType,
        message: String,
    }

    impl fmt::Display for TestMockCoreError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "TestMockCoreError ({:?}): {}", self.error_type, self.message)
        }
    }

    impl TestMockCoreError {
        pub fn new(error_type: CoreErrorType, message: String) -> Self {
            Self { error_type, message }
        }
        // pub fn is_not_found_error(&self) -> bool { // This method is on the mock error, not novade_core::CoreError
        //     matches!(self.error_type, CoreErrorType::NotFound)
        // }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspaces::config::types::{WorkspaceSnapshot, WorkspaceSetSnapshot};
    use crate::workspaces::core::types::WorkspaceLayoutType; 
    // Use the renamed mock error from the local module for tests
    use crate::workspaces::config::provider::core_error_mock_for_workspaces::{TestMockCoreError, CoreErrorType as TestMockCoreErrorType};


    #[derive(Default)]
    struct MockConfigService {
        files: std::collections::HashMap<String, String>,
        force_read_error_type: Option<TestMockCoreErrorType>, // Uses the test-local enum
        force_write_error_type: Option<TestMockCoreErrorType>,
    }

    impl MockConfigService {
        fn new() -> Self { Self::default() }
        fn set_file_content(&mut self, key: &str, content: String) { self.files.insert(key.to_string(), content); }
        #[allow(dead_code)]
        fn set_read_error_type(&mut self, error_type: Option<TestMockCoreErrorType>) { self.force_read_error_type = error_type; }
        #[allow(dead_code)]
        fn set_write_error_type(&mut self, error_type: Option<TestMockCoreErrorType>) { self.force_write_error_type = error_type; }
    }

    #[async_trait]
    impl crate::ports::config_service::ConfigServiceAsync for MockConfigService {
        async fn read_config_file_string(&self, key: &str) -> Result<String, novade_core::CoreError> {
             if let Some(ref err_type) = self.force_read_error_type {
                let core_err = match err_type {
                    TestMockCoreErrorType::NotFound => novade_core::CoreError::Config(novade_core::ConfigError::NotFound{locations: vec![key.into()]}),
                    TestMockCoreErrorType::IoError => novade_core::CoreError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("mock io error for {}", key))),
                    TestMockCoreErrorType::Other(s) => novade_core::CoreError::Internal(s.clone()),
                };
                return Err(core_err);
            }
            self.files.get(key)
                .cloned()
                .ok_or_else(|| novade_core::CoreError::Config(novade_core::ConfigError::NotFound{locations: vec![key.into()]}))
        }
        async fn write_config_file_string(&self, _key: &str, _content: String) -> Result<(), novade_core::CoreError> {
            if let Some(ref err_type) = self.force_write_error_type {
                let core_err = match err_type {
                    TestMockCoreErrorType::NotFound => novade_core::CoreError::Config(novade_core::ConfigError::NotFound{locations: vec![]}), // Path might be irrelevant for write error not found
                    TestMockCoreErrorType::IoError => novade_core::CoreError::Io(std::io::Error::new(std::io::ErrorKind::Other, "mock io write error")),
                    TestMockCoreErrorType::Other(s) => novade_core::CoreError::Internal(s.clone()),
                };
                 return Err(core_err);
            }
            Ok(())
        }
        async fn read_file_to_string(&self, _path: &std::path::Path) -> Result<String, novade_core::CoreError> { unimplemented!() }
        async fn list_files_in_dir(&self, _dir_path: &std::path::Path, _extension: Option<&str>) -> Result<Vec<std::path::PathBuf>, novade_core::CoreError> { unimplemented!() }
        async fn get_config_dir(&self) -> Result<std::path::PathBuf, novade_core::CoreError> { unimplemented!() } 
        async fn get_data_dir(&self) -> Result<std::path::PathBuf, novade_core::CoreError> { unimplemented!() }
    }

    #[tokio::test]
    async fn test_load_workspace_config_handles_updated_snapshot_structure() {
        let mut mock_service_inner = MockConfigService::new();
        let expected_snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { 
                    name: "WS1".to_string(), 
                    layout_type: WorkspaceLayoutType::Floating,
                    persistent_id: "pid1".to_string(),
                    icon_name: Some("icon1".to_string()),
                    accent_color_hex: Some("#112233".to_string()),
                },
            ],
            active_workspace_persistent_id: Some("pid1".to_string()),
        };
        let toml_content = toml::to_string_pretty(&expected_snapshot).unwrap();
        mock_service_inner.set_file_content("ws_test_config.toml", toml_content);
        let mock_config_service = Arc::new(mock_service_inner);

        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());
        
        let loaded_snapshot = provider.load_workspace_config().await.unwrap();
        assert_eq!(loaded_snapshot, expected_snapshot);
        assert_eq!(loaded_snapshot.workspaces[0].persistent_id, "pid1");
        assert_eq!(loaded_snapshot.active_workspace_persistent_id, Some("pid1".to_string()));
    }
    
    #[tokio::test]
    async fn test_load_workspace_config_invalid_active_pid_in_file() {
        let mut mock_service_inner = MockConfigService::new();
        let snapshot_in_file = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { 
                    name: "WS1".to_string(), 
                    layout_type: WorkspaceLayoutType::Floating,
                    persistent_id: "pid1".to_string(), 
                    icon_name: None, accent_color_hex: None,
                },
            ],
            active_workspace_persistent_id: Some("pid_non_existent".to_string()),
        };
        let toml_content = toml::to_string_pretty(&snapshot_in_file).unwrap();
        mock_service_inner.set_file_content("ws_test_config.toml", toml_content);
        let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());

        let loaded_snapshot = provider.load_workspace_config().await.unwrap();
        assert_eq!(loaded_snapshot.active_workspace_persistent_id, Some("pid_non_existent".to_string()));
        assert!(loaded_snapshot.validate().is_err(), "Snapshot validation should fail due to inconsistent active_workspace_persistent_id");
    }


    #[tokio::test]
    async fn test_save_workspace_config_handles_updated_snapshot() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());
        let snapshot_to_save = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot {
                    name: "WS Save".to_string(),
                    layout_type: WorkspaceLayoutType::TilingVertical,
                    persistent_id: "pid_save".to_string(),
                    icon_name: None,
                    accent_color_hex: None,
                },
            ],
            active_workspace_persistent_id: Some("pid_save".to_string()),
        };
        
        let result = provider.save_workspace_config(&snapshot_to_save).await;
        assert!(result.is_ok());
    }
}
