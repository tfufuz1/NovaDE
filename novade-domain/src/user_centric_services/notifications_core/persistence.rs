use std::collections::VecDeque;
use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, info, warn, error};
use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;

use super::types::Notification;
use super::errors::NotificationError;
use super::persistence_iface::NotificationHistoryProvider;

pub struct FilesystemNotificationHistoryProvider {
    pub config_service: Arc<dyn ConfigServiceAsync>,
    pub history_config_key: String,
}

impl FilesystemNotificationHistoryProvider {
    pub fn new(config_service: Arc<dyn ConfigServiceAsync>, history_config_key: String) -> Self {
        Self {
            config_service,
            history_config_key,
        }
    }
}

#[async_trait]
impl NotificationHistoryProvider for FilesystemNotificationHistoryProvider {
    async fn load_history(&self) -> Result<VecDeque<Notification>, NotificationError> {
        debug!("Loading notification history from key '{}'", self.history_config_key);
        match self.config_service.read_config_file_string(&self.history_config_key).await {
            Ok(toml_string) => {
                toml::from_str(&toml_string).map_err(|e| {
                    error!("Failed to deserialize TOML notification history from key '{}': {}", self.history_config_key, e);
                    // Consider adding a specific deserialization error variant to NotificationError
                    // For now, mapping to a general HistoryPersistenceError or InternalError.
                    // Let's use InternalError for now if CoreError doesn't fit well for toml errors.
                    NotificationError::InternalError(format!("History deserialization failed: {}", e))
                })
            }
            Err(core_error) => {
                if core_error.is_not_found_error() { // Assuming CoreError has this method
                    info!("Notification history file (key '{}') not found. Returning empty history.", self.history_config_key);
                    Ok(VecDeque::new())
                } else {
                    error!("CoreError loading notification history (key '{}'): {}", self.history_config_key, core_error);
                    Err(NotificationError::history_persistence_error_from_core("load_history".to_string(), "Failed to read history file".to_string(), core_error))
                }
            }
        }
    }

    async fn save_history(&self, history: &VecDeque<Notification>) -> Result<(), NotificationError> {
        debug!("Saving {} notification history items to key '{}'", history.len(), self.history_config_key);
        let toml_string = toml::to_string_pretty(history).map_err(|e| {
            error!("Failed to serialize notification history to TOML for key '{}': {}", self.history_config_key, e);
            // Similar to load, consider a specific serialization error variant.
            NotificationError::InternalError(format!("History serialization failed: {}", e))
        })?;

        self.config_service.write_config_file_string(&self.history_config_key, toml_string).await
            .map_err(|core_error| {
                NotificationError::history_persistence_error_from_core("save_history".to_string(), "Failed to write history file".to_string(), core_error)
            })?;
        info!("Notification history saved successfully to key '{}'", self.history_config_key);
        Ok(())
    }
}


// Mock for CoreError's is_not_found_error for compilation.
// This should be part of the actual CoreError definition.
#[cfg(test)]
mod core_error_mock_for_notifications_persistence {
    use std::fmt;
    use thiserror::Error;

    #[derive(Error, Debug, Clone)]
    pub enum CoreErrorType { NotFound, IoError, Other(String) }

    #[derive(Error, Debug, Clone)]
    pub struct CoreError { pub error_type: CoreErrorType, message: String }

    impl fmt::Display for CoreError { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "CoreError ({:?}): {}", self.error_type, self.message) } }
    impl CoreError {
        pub fn new(error_type: CoreErrorType, message: String) -> Self { Self { error_type, message } }
        pub fn is_not_found_error(&self) -> bool { matches!(self.error_type, CoreErrorType::NotFound) }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_centric_services::notifications_core::types::{NotificationInput, NotificationUrgency};
    use crate::user_centric_services::notifications_core::persistence::core_error_mock_for_notifications_persistence::{CoreError as MockCoreError, CoreErrorType as MockCoreErrorType};
    use novade_core::config::ConfigServiceAsync; // Keep for trait bound
    use std::collections::HashMap;
    use tokio::sync::RwLock; // Ensure RwLock is in scope
    use uuid::Uuid;

    #[derive(Default)]
    struct MockConfigService {
        files: Arc<RwLock<HashMap<String, String>>>,
        force_read_error_type: Option<MockCoreErrorType>,
        force_write_error_type: Option<MockCoreErrorType>,
    }
    impl MockConfigService {
        fn new() -> Self { Default::default() }
        async fn set_file_content(&self, key: &str, content: String) { self.files.write().await.insert(key.to_string(), content); }
        #[allow(dead_code)] fn set_read_error_type(&mut self, error_type: Option<MockCoreErrorType>) { self.force_read_error_type = error_type; }
        #[allow(dead_code)] fn set_write_error_type(&mut self, error_type: Option<MockCoreErrorType>) { self.force_write_error_type = error_type; }
    }

    #[async_trait]
    impl ConfigServiceAsync for MockConfigService {
        async fn read_config_file_string(&self, key: &str) -> Result<String, novade_core::errors::CoreError> {
            if let Some(ref err_type) = self.force_read_error_type {
                return Err(MockCoreError::new(err_type.clone(), format!("Forced read error on {}", key)));
            }
            match self.files.read().await.get(key) {
                Some(content) => Ok(content.clone()),
                None => Err(MockCoreError::new(MockCoreErrorType::NotFound, format!("File not found: {}", key))),
            }
        }
        async fn write_config_file_string(&self, key: &str, content: String) -> Result<(), novade_core::errors::CoreError> {
            if let Some(ref err_type) = self.force_write_error_type {
                return Err(MockCoreError::new(err_type.clone(), format!("Forced write error on {}", key)));
            }
            self.files.write().await.insert(key.to_string(), content);
            Ok(())
        }
        async fn read_file_to_string(&self, _path: &std::path::Path) -> Result<String, novade_core::errors::CoreError> { unimplemented!() }
        async fn list_files_in_dir(&self, _dir_path: &std::path::Path, _extension: Option<&str>) -> Result<Vec<std::path::PathBuf>, novade_core::errors::CoreError> { unimplemented!() }
        async fn get_config_dir(&self) -> std::path::PathBuf { unimplemented!() }
        async fn get_data_dir(&self) -> std::path::PathBuf { unimplemented!() }
    }

    fn create_test_notification(id: Uuid, summary: &str) -> Notification {
        let input = NotificationInput {
            application_name: "TestApp".to_string(),
            application_icon: None,
            summary: summary.to_string(),
            body: Some(format!("Body for {}", summary)),
            actions: vec![], urgency: NotificationUrgency::Normal, category: None, hints: None, timeout_ms: None, transient: false,
        };
        Notification::new(input, id, chrono::Utc::now())
    }

    #[tokio::test]
    async fn test_load_history_file_not_found() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemNotificationHistoryProvider::new(mock_config_service, "test_history.toml".to_string());
        let history = provider.load_history().await.unwrap();
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_save_and_load_history() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemNotificationHistoryProvider::new(mock_config_service.clone(), "test_history.toml".to_string());
        
        let mut history_to_save = VecDeque::new();
        let n1_id = Uuid::new_v4();
        let n2_id = Uuid::new_v4();
        history_to_save.push_back(create_test_notification(n1_id, "Notif1"));
        history_to_save.push_back(create_test_notification(n2_id, "Notif2"));

        provider.save_history(&history_to_save).await.unwrap();

        // Verify content was written (optional, if mock stores it for inspection)
        // let file_content = mock_config_service.files.read().await.get("test_history.toml").cloned().unwrap();
        // assert!(file_content.contains("Notif1"));

        let loaded_history = provider.load_history().await.unwrap();
        assert_eq!(loaded_history.len(), 2);
        assert_eq!(loaded_history[0].id, n1_id);
        assert_eq!(loaded_history[1].id, n2_id);
    }
    
    #[tokio::test]
    async fn test_save_empty_history() {
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemNotificationHistoryProvider::new(mock_config_service.clone(), "empty_history.toml".to_string());
        let empty_history = VecDeque::new();
        provider.save_history(&empty_history).await.unwrap();
        
        let loaded_history = provider.load_history().await.unwrap();
        assert!(loaded_history.is_empty());
    }

    #[tokio::test]
    async fn test_load_history_deserialization_error() {
        let mock_config_service = Arc::new(MockConfigService::new());
        mock_config_service.set_file_content("bad_history.toml", "this is not valid toml content".to_string()).await;
        let provider = FilesystemNotificationHistoryProvider::new(mock_config_service, "bad_history.toml".to_string());
        
        let result = provider.load_history().await;
        assert!(result.is_err());
        match result.err().unwrap() {
            NotificationError::InternalError(msg) => assert!(msg.contains("History deserialization failed")),
            _ => panic!("Unexpected error type"),
        }
    }

    #[tokio::test]
    async fn test_save_history_serialization_error() {
        // This is hard to trigger with VecDeque<Notification> as it should always serialize if Notification does.
        // A custom type that can fail serialization would be needed for a direct test.
        // We'll assume toml::to_string_pretty works for valid inputs.
        // If Notification had a field that couldn't be serialized by TOML (e.g. deeply nested unsupported type), this could fail.
        // For now, this test is more of a placeholder for the error mapping.
        let mock_config_service = Arc::new(MockConfigService::new());
        let provider = FilesystemNotificationHistoryProvider::new(mock_config_service, "test_history.toml".to_string());
        
        // Create a Notification that might be problematic IF it had complex, non-TOML-friendly fields.
        // With current Notification struct, this should not fail serialization.
        let history_with_potentially_problematic_data = VecDeque::from(vec![
            create_test_notification(Uuid::new_v4(), "Problem? \u{FFFF}"), // U+FFFF is not valid in TOML strings
        ]);
        
        let result = provider.save_history(&history_with_potentially_problematic_data).await;
        // Expecting error because U+FFFF is not valid in TOML strings.
        assert!(result.is_err());
         match result.err().unwrap() {
            NotificationError::InternalError(msg) => assert!(msg.contains("History serialization failed")),
            e => panic!("Expected InternalError due to TOML serialization, got {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_load_history_config_service_read_error() {
        let mut mock_service_inner = MockConfigService::new();
        mock_service_inner.set_read_error_type(Some(MockCoreErrorType::IoError)); // Simulate an I/O error
        let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemNotificationHistoryProvider::new(mock_config_service, "error_history.toml".to_string());

        let result = provider.load_history().await;
        assert!(result.is_err());
        match result.err().unwrap() {
            NotificationError::HistoryPersistenceError { operation, source, .. } => {
                assert_eq!(operation, "load_history");
                assert!(source.to_string().contains("Forced read error"));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[tokio::test]
    async fn test_save_history_config_service_write_error() {
        let mut mock_service_inner = MockConfigService::new();
        mock_service_inner.set_write_error_type(Some(MockCoreErrorType::IoError)); // Simulate an I/O error
        let mock_config_service = Arc::new(mock_service_inner);
        let provider = FilesystemNotificationHistoryProvider::new(mock_config_service, "error_history.toml".to_string());
        let history_to_save = VecDeque::new();

        let result = provider.save_history(&history_to_save).await;
        assert!(result.is_err());
         match result.err().unwrap() {
            NotificationError::HistoryPersistenceError { operation, source, .. } => {
                assert_eq!(operation, "save_history");
                assert!(source.to_string().contains("Forced write error"));
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
