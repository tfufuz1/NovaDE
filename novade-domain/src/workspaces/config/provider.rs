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
                let mut snapshot: WorkspaceSetSnapshot = toml::from_str(&toml_string).map_err(|e| {
                    warn!("Failed to deserialize TOML workspace config for key '{}': {}", self.config_key, e);
                    WorkspaceConfigError::DeserializationError {
                        message: format!("Failed to parse TOML for key '{}'", self.config_key),
                        snippet: Some(toml_string.chars().take(200).collect()), // First 200 chars
                        source: Some(e),
                    }
                })?;
                
                // Validate the loaded snapshot (e.g. active_workspace_persistent_id exists)
                // This validation is now more critical with persistent_id.
                if let Err(reason) = snapshot.validate() {
                    warn!("Loaded workspace configuration for key '{}' is invalid: {}. Proceeding with caution or default.", self.config_key, reason);
                    // Depending on strictness, one might return an error here or try to recover.
                    // For now, let's allow it but log a warning. The manager will handle inconsistencies.
                    // Or, if validation should be strict:
                    // return Err(WorkspaceConfigError::InvalidData { reason, path: Some(self.config_key.clone()) });
                }
                
                // If active_workspace_persistent_id is Some, but no workspace with that persistent_id exists,
                // it's an invalid state. The manager should handle this by possibly setting it to None or first.
                // The provider itself doesn't change the data, just loads it.
                // The validation in WorkspaceSetSnapshot already checks this.

                Ok(snapshot)
            }
            Err(core_error) => {
                if core_error.is_not_found_error() { // Hypothetical method from CoreError
                    info!("Workspace config file for key '{}' not found. Returning default snapshot.", self.config_key);
                    Ok(WorkspaceSetSnapshot::default())
                } else {
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
    use crate::workspaces::core::types::WorkspaceLayoutType; 
    use crate::workspaces::config::provider::core_error_mock_for_workspaces::CoreError as MockCoreError;
    use crate::workspaces::config::provider::core_error_mock_for_workspaces::CoreErrorType as MockCoreErrorType;

    #[derive(Default)]
    struct MockConfigService {
        files: std::collections::HashMap<String, String>,
        force_read_error_type: Option<MockCoreErrorType>,
        force_write_error_type: Option<MockCoreErrorType>,
    }

    impl MockConfigService {
        fn new() -> Self { Self::default() }
        fn set_file_content(&mut self, key: &str, content: String) { self.files.insert(key.to_string(), content); }
        fn set_read_error_type(&mut self, error_type: Option<MockCoreErrorType>) { self.force_read_error_type = error_type; }
        fn set_write_error_type(&mut self, error_type: Option<MockCoreErrorType>) { self.force_write_error_type = error_type; }
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
        async fn write_config_file_string(&self, _key: &str, _content: String) -> Result<(), novade_core::errors::CoreError> {
            if let Some(ref err_type) = self.force_write_error_type {
                 return Err(MockCoreError::new(err_type.clone(), format!("Forced write error")));
            }
            Ok(())
        }
        async fn read_file_to_string(&self, _path: &std::path::Path) -> Result<String, novade_core::errors::CoreError> { unimplemented!() }
        async fn list_files_in_dir(&self, _dir_path: &std::path::Path, _extension: Option<&str>) -> Result<Vec<std::path::PathBuf>, novade_core::errors::CoreError> { unimplemented!() }
        async fn get_config_dir(&self) -> std::path::PathBuf { unimplemented!() }
        async fn get_data_dir(&self) -> std::path::PathBuf { unimplemented!() }
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
        // This test checks if loading a file with an active_workspace_persistent_id
        // that doesn't exist in its own workspace list is handled.
        // The provider should load it, but WorkspaceSetSnapshot::validate() would fail.
        // The manager is ultimately responsible for correcting this logical inconsistency.
        let mut mock_service_inner = MockConfigService::new();
        let snapshot_in_file = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { 
                    name: "WS1".to_string(), 
                    layout_type: WorkspaceLayoutType::Floating,
                    persistent_id: "pid1".to_string(), // Only pid1 exists
                    icon_name: None, accent_color_hex: None,
                },
            ],
            active_workspace_persistent_id: Some("pid_non_existent".to_string()), // This PID does not exist in the list
        };
        let toml_content = toml::to_string_pretty(&snapshot_in_file).unwrap();
        mock_service_inner.set_file_content("ws_test_config.toml", toml_content);
        let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws_test_config.toml".to_string());

        // load_workspace_config loads the data as is.
        // The validate() method on WorkspaceSetSnapshot would catch this.
        // The provider might log a warning if strict validation is done at load time,
        // but currently, it relies on the manager or consumer to validate further if needed.
        let loaded_snapshot = provider.load_workspace_config().await.unwrap();
        assert_eq!(loaded_snapshot.active_workspace_persistent_id, Some("pid_non_existent".to_string()));
        // At this stage, the provider has loaded what's in the file.
        // The `validate()` method of `WorkspaceSetSnapshot` would return an error for this `loaded_snapshot`.
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
        // In a more complex mock, one would verify the content written to the mock_config_service.
        // For this test, success of save is sufficient.
    }
}
