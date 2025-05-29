use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, warn};

use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;

use super::types::{WorkspaceSetSnapshot, WorkspaceSnapshot};
use super::errors::WorkspaceConfigError;

// --- WorkspaceConfigProvider Trait ---

#[async_trait]
pub trait WorkspaceConfigProvider: Send + Sync {
    async fn load_workspace_config(&self) -> Result<WorkspaceSetSnapshot, WorkspaceConfigError>;
    async fn save_workspace_config(&self, config_snapshot: &WorkspaceSetSnapshot) -> Result<(), WorkspaceConfigError>;
}

// --- FilesystemConfigProvider Implementation ---

pub struct FilesystemConfigProvider {
    config_service: Arc<dyn ConfigServiceAsync>,
    config_key: String,
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
        debug!("Loading workspace config from key: {}", self.config_key);
        match self.config_service.read_config_file_string(&self.config_key).await {
            Ok(content) => {
                debug!("Successfully read workspace config file content for key: {}", self.config_key);
                let snapshot: WorkspaceSetSnapshot = toml::from_str(&content).map_err(|e| {
                    warn!("Failed to deserialize workspace config from key '{}': {}", self.config_key, e);
                    WorkspaceConfigError::DeserializationError {
                        message: format!("Failed to parse TOML content for key '{}'", self.config_key),
                        snippet: Some(content.chars().take(100).collect()),
                        source: Some(e),
                    }
                })?;

                debug!("Workspace config deserialized. Validating...");
                
                let mut seen_pids = HashSet::new();
                for ws_snapshot in &snapshot.workspaces {
                    if ws_snapshot.persistent_id.is_empty() {
                        return Err(WorkspaceConfigError::invalid_data(
                            "WorkspaceSnapshot contains an empty persistent_id.".to_string(),
                            Some(format!("workspaces[name={}]", ws_snapshot.name)),
                        ));
                    }
                    if !seen_pids.insert(&ws_snapshot.persistent_id) {
                        return Err(WorkspaceConfigError::DuplicatePersistentIdInLoadedSet {
                            persistent_id: ws_snapshot.persistent_id.clone(),
                        });
                    }
                }

                if let Some(active_pid) = &snapshot.active_workspace_persistent_id {
                    if active_pid.is_empty() {
                        // Treat Some("") as effectively None, or an error if strict.
                        // For loading, let's be somewhat lenient and allow manager to treat Some("") as None.
                        // Alternatively, could error:
                        // return Err(WorkspaceConfigError::invalid_data(
                        //     "active_workspace_persistent_id cannot be an empty string if Some.".to_string(),
                        //     Some("active_workspace_persistent_id".to_string()),
                        // ));
                    } else if !seen_pids.contains(active_pid) {
                        return Err(WorkspaceConfigError::PersistentIdNotFoundInLoadedSet {
                            persistent_id: active_pid.clone(),
                        });
                    }
                }
                
                debug!("Workspace config validated successfully for key: {}", self.config_key);
                Ok(snapshot)
            }
            Err(e) => {
                if e.is_not_found() {
                    debug!("Workspace config file not found for key '{}'. Returning default (empty) config.", self.config_key);
                    Ok(WorkspaceSetSnapshot::default())
                } else {
                    warn!("Failed to read workspace config file for key '{}': {}", self.config_key, e);
                    Err(WorkspaceConfigError::LoadError {
                        path: self.config_key.clone(),
                        source: e,
                    })
                }
            }
        }
    }

    async fn save_workspace_config(&self, config_snapshot: &WorkspaceSetSnapshot) -> Result<(), WorkspaceConfigError> {
        debug!("Saving workspace config to key: {}", self.config_key);
        
        let mut seen_pids = HashSet::new();
        for ws_snapshot in &config_snapshot.workspaces {
            if ws_snapshot.persistent_id.is_empty() {
                return Err(WorkspaceConfigError::invalid_data(
                    "Attempted to save a WorkspaceSnapshot with an empty persistent_id.".to_string(),
                    Some(format!("workspaces[name={}]", ws_snapshot.name)),
                ));
            }
            if !seen_pids.insert(&ws_snapshot.persistent_id) {
                return Err(WorkspaceConfigError::DuplicatePersistentIdInLoadedSet {
                    persistent_id: ws_snapshot.persistent_id.clone(),
                });
            }
        }
        if let Some(active_pid) = &config_snapshot.active_workspace_persistent_id {
            if active_pid.is_empty() {
                 return Err(WorkspaceConfigError::invalid_data(
                    "Attempted to save with an empty active_workspace_persistent_id string.".to_string(),
                    Some("active_workspace_persistent_id".to_string()),
                ));
            }
            if !seen_pids.contains(active_pid) {
                return Err(WorkspaceConfigError::PersistentIdNotFoundInLoadedSet {
                    persistent_id: active_pid.clone(),
                });
            }
        }

        let serialized_content = toml::to_string_pretty(config_snapshot).map_err(|e| {
            warn!("Failed to serialize workspace config for key '{}': {}", self.config_key, e);
            WorkspaceConfigError::SerializationError {
                message: format!("Failed to serialize workspace configuration for key '{}'", self.config_key),
                source: Some(e),
            }
        })?;

        self.config_service
            .write_config_file_string(&self.config_key, serialized_content)
            .await
            .map_err(|e| {
                warn!("Failed to write workspace config to config service for key '{}': {}", self.config_key, e);
                WorkspaceConfigError::SaveError {
                    path: self.config_key.clone(),
                    source: e,
                }
            })?;
        debug!("Workspace config saved successfully to key: {}", self.config_key);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::config::MockConfigServiceAsync;
    use crate::workspaces::core::WorkspaceLayoutType;
    use std::io;
    use uuid::Uuid; // For default WorkspaceSnapshot pid

    fn new_mock_arc_config_service() -> Arc<MockConfigServiceAsync> {
        Arc::new(MockConfigServiceAsync::new())
    }
    
    // Default implementation for WorkspaceSnapshot for easier test setup
    impl Default for WorkspaceSnapshot {
        fn default() -> Self {
            Self {
                persistent_id: Uuid::new_v4().to_string(), // Unique default PID for tests
                name: "Default WS".to_string(),
                layout_type: WorkspaceLayoutType::default(),
                icon_name: None,
                accent_color_hex: None,
            }
        }
    }


    #[tokio::test]
    async fn load_config_success() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { persistent_id: "pid1".to_string(), name: "WS1".to_string(), ..Default::default() }
            ],
            active_workspace_persistent_id: Some("pid1".to_string()),
        };
        let toml_content = toml::to_string_pretty(&snapshot).unwrap();

        mock_config_service.expect_read_config_file_string()
            .withf(|key| key == "ws.toml")
            .returning(move |_| Ok(toml_content.clone()));

        let provider = FilesystemConfigProvider::new(Arc::new(mock_config_service), "ws.toml".to_string());
        let loaded_snapshot = provider.load_workspace_config().await.unwrap();
        assert_eq!(loaded_snapshot, snapshot);
    }

    #[tokio::test]
    async fn load_config_not_found_returns_default() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let not_found_error = CoreError::IoError("not found".to_string(), Some(Arc::new(io::Error::new(io::ErrorKind::NotFound, "not found"))));
        mock_config_service.expect_read_config_file_string()
            .returning(move |_| Err(not_found_error.clone()));

        let provider = FilesystemConfigProvider::new(Arc::new(mock_config_service), "ws.toml".to_string());
        let loaded_snapshot = provider.load_workspace_config().await.unwrap();
        assert_eq!(loaded_snapshot, WorkspaceSetSnapshot::default());
    }

    #[tokio::test]
    async fn load_config_corrupted_toml() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        mock_config_service.expect_read_config_file_string()
            .returning(|_| Ok("this is not toml {{{{".to_string()));
        
        let provider = FilesystemConfigProvider::new(Arc::new(mock_config_service), "ws.toml".to_string());
        let result = provider.load_workspace_config().await;
        assert!(matches!(result, Err(WorkspaceConfigError::DeserializationError { .. })));
    }

    #[tokio::test]
    async fn load_config_duplicate_pids() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { persistent_id: "pid1".to_string(), name: "WS1".to_string(), ..Default::default() },
                WorkspaceSnapshot { persistent_id: "pid1".to_string(), name: "WS2".to_string(), ..Default::default() },
            ],
            active_workspace_persistent_id: Some("pid1".to_string()),
        };
        let toml_content = toml::to_string_pretty(&snapshot).unwrap();
        mock_config_service.expect_read_config_file_string().returning(move |_| Ok(toml_content.clone()));

        let provider = FilesystemConfigProvider::new(Arc::new(mock_config_service), "ws.toml".to_string());
        let result = provider.load_workspace_config().await;
        assert!(matches!(result, Err(WorkspaceConfigError::DuplicatePersistentIdInLoadedSet { persistent_id }) if persistent_id == "pid1"));
    }
    
    #[tokio::test]
    async fn load_config_active_pid_not_found() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { persistent_id: "pid1".to_string(), name: "WS1".to_string(), ..Default::default() },
            ],
            active_workspace_persistent_id: Some("pid_non_existent".to_string()),
        };
        let toml_content = toml::to_string_pretty(&snapshot).unwrap();
        mock_config_service.expect_read_config_file_string().returning(move |_| Ok(toml_content.clone()));

        let provider = FilesystemConfigProvider::new(Arc::new(mock_config_service), "ws.toml".to_string());
        let result = provider.load_workspace_config().await;
        assert!(matches!(result, Err(WorkspaceConfigError::PersistentIdNotFoundInLoadedSet { persistent_id }) if persistent_id == "pid_non_existent"));
    }
    
    #[tokio::test]
    async fn load_config_empty_persistent_id_in_snapshot_is_error() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { persistent_id: "".to_string(), name: "WS1".to_string(), ..Default::default() },
            ],
            active_workspace_persistent_id: None,
        };
        let toml_content = toml::to_string_pretty(&snapshot).unwrap();
        mock_config_service.expect_read_config_file_string().returning(move |_| Ok(toml_content.clone()));

        let provider = FilesystemConfigProvider::new(Arc::new(mock_config_service), "ws.toml".to_string());
        let result = provider.load_workspace_config().await;
        assert!(matches!(result, Err(WorkspaceConfigError::InvalidData { reason, .. }) if reason.contains("empty persistent_id")));
    }

    #[tokio::test]
    async fn save_config_success() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { persistent_id: "pid1".to_string(), name: "WS1".to_string(), ..Default::default() }
            ],
            active_workspace_persistent_id: Some("pid1".to_string()),
        };
        let expected_toml_content = toml::to_string_pretty(&snapshot).unwrap();

        mock_config_service.expect_write_config_file_string()
            .withf(move |key, content| key == "ws_save.toml" && content == expected_toml_content)
            .times(1)
            .returning(|_, _| Ok(()));

        let provider = FilesystemConfigProvider::new(Arc::new(mock_config_service), "ws_save.toml".to_string());
        let result = provider.save_workspace_config(&snapshot).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn save_config_invalid_data_empty_pid_in_workspaces() {
        let mock_config_service = new_mock_arc_config_service();
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { persistent_id: "".to_string(), name: "WS1".to_string(), ..Default::default() }
            ],
            active_workspace_persistent_id: None,
        };
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws.toml".to_string());
        let result = provider.save_workspace_config(&snapshot).await;
        assert!(matches!(result, Err(WorkspaceConfigError::InvalidData { reason, .. }) if reason.contains("empty persistent_id")));
    }

    #[tokio::test]
    async fn save_config_invalid_data_empty_active_pid() {
        let mock_config_service = new_mock_arc_config_service();
        let snapshot = WorkspaceSetSnapshot {
            workspaces: vec![
                WorkspaceSnapshot { persistent_id: "pid1".to_string(), name: "WS1".to_string(), ..Default::default() }
            ],
            active_workspace_persistent_id: Some("".to_string()), // Invalid: Some("")
        };
        let provider = FilesystemConfigProvider::new(mock_config_service, "ws.toml".to_string());
        let result = provider.save_workspace_config(&snapshot).await;
        assert!(matches!(result, Err(WorkspaceConfigError::InvalidData { reason, .. }) if reason.contains("empty active_workspace_persistent_id")));
    }

    #[tokio::test]
    async fn save_config_write_error() {
        let mut mock_config_service = MockConfigServiceAsync::new();
        let snapshot = WorkspaceSetSnapshot::default(); // Valid default snapshot
        let write_error = CoreError::IoError("Simulated write error".to_string(), None);

        mock_config_service.expect_write_config_file_string()
            .returning(move |_, _| Err(write_error.clone()));
            
        let provider = FilesystemConfigProvider::new(Arc::new(mock_config_service), "ws.toml".to_string());
        let result = provider.save_workspace_config(&snapshot).await;
        assert!(matches!(result, Err(WorkspaceConfigError::SaveError { .. })));
    }
}
