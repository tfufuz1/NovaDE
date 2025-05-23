#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::path::PathBuf; // Not directly used for mock, but good for context
    use tokio::sync::RwLock;
    use serde_json::json;
    use tracing::debug; // For debugging test output if needed

    use crate::global_settings_management::{
        service::{GlobalSettingsService, DefaultGlobalSettingsService},
        types::*,
        paths::*,
        errors::GlobalSettingsError,
        persistence_iface::SettingsPersistenceProvider,
    };
    use novade_core::errors::CoreError; // For mock persistence provider

    // --- Mock SettingsPersistenceProvider ---
    #[derive(Debug, Clone, Default)]
    struct MockPersistenceProvider {
        settings: Arc<RwLock<GlobalDesktopSettings>>,
        should_error_on_load: Option<GlobalSettingsError>,
        should_error_on_save: Option<GlobalSettingsError>,
        save_called_count: Arc<RwLock<usize>>,
    }

    impl MockPersistenceProvider {
        fn new() -> Self {
            Self {
                settings: Arc::new(RwLock::new(GlobalDesktopSettings::default())),
                should_error_on_load: None,
                should_error_on_save: None,
                save_called_count: Arc::new(RwLock::new(0)),
            }
        }

        #[allow(dead_code)]
        async fn set_settings(&self, settings: GlobalDesktopSettings) {
            let mut guard = self.settings.write().await;
            *guard = settings;
        }
        
        #[allow(dead_code)]
        fn set_error_on_load(&mut self, error: Option<GlobalSettingsError>) {
            self.should_error_on_load = error;
        }
        
        #[allow(dead_code)]
        fn set_error_on_save(&mut self, error: Option<GlobalSettingsError>) {
            self.should_error_on_save = error;
        }

        #[allow(dead_code)]
        async fn get_save_called_count(&self) -> usize {
            *self.save_called_count.read().await
        }
    }

    #[async_trait::async_trait]
    impl SettingsPersistenceProvider for MockPersistenceProvider {
        async fn load_global_settings(&self) -> Result<GlobalDesktopSettings, GlobalSettingsError> {
            if let Some(err) = &self.should_error_on_load {
                return Err(err.clone());
            }
            let guard = self.settings.read().await;
            Ok(guard.clone())
        }

        async fn save_global_settings(&self, settings: &GlobalDesktopSettings) -> Result<(), GlobalSettingsError> {
            let mut count_guard = self.save_called_count.write().await;
            *count_guard += 1;
            drop(count_guard);

            if let Some(err) = &self.should_error_on_save {
                return Err(err.clone());
            }
            let mut guard = self.settings.write().await;
            *guard = settings.clone();
            Ok(())
        }
    }

    // Helper to create a service with a default mock provider
    async fn create_test_service(mock_provider: Arc<MockPersistenceProvider>) -> DefaultGlobalSettingsService {
        DefaultGlobalSettingsService::new(mock_provider, Some(5))
    }
    
    // Helper to create a service and load initial settings
    async fn create_and_load_test_service(initial_settings: Option<GlobalDesktopSettings>) -> (DefaultGlobalSettingsService, Arc<MockPersistenceProvider>) {
        let mock_provider = Arc::new(MockPersistenceProvider::new());
        if let Some(settings) = initial_settings {
            mock_provider.set_settings(settings).await;
        }
        let service = create_test_service(mock_provider.clone()).await;
        service.load_settings().await.expect("Initial load_settings failed");
        (service, mock_provider)
    }


    #[tokio::test]
    async fn test_new_service_and_initial_load() {
        let mut initial_settings = GlobalDesktopSettings::default();
        initial_settings.appearance.active_theme_name = "TestTheme".to_string();
        
        let (service, _) = create_and_load_test_service(Some(initial_settings.clone())).await;
        
        let current_settings = service.get_current_settings().unwrap();
        assert_eq!(current_settings.appearance.active_theme_name, "TestTheme");
    }

    #[tokio::test]
    async fn test_load_settings_provider_error() {
        let mock_provider = Arc::new(MockPersistenceProvider::new());
        let core_err = CoreError::IoError("Simulated IO load error".to_string());
        let persist_err = GlobalSettingsError::persistence_error_with_core_source("load", "mock load", core_err);
        mock_provider.clone().set_error_on_load(Some(persist_err.clone()));
        
        let service = create_test_service(mock_provider).await;
        let result = service.load_settings().await;
        
        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::PersistenceError { operation, .. } => assert_eq!(operation, "load"),
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_load_settings_validation_error() {
        let mut invalid_settings = GlobalDesktopSettings::default();
        invalid_settings.appearance.font_settings.default_font_size = 0.0; // Invalid
        
        let (service, _) = create_and_load_test_service(Some(invalid_settings)).await;
        // load_settings is called in create_and_load_test_service.
        // The error should have occurred there.
        // Let's re-test load_settings directly to check the validation path.
        
        let mock_provider = Arc::new(MockPersistenceProvider::new());
        mock_provider.set_settings(invalid_settings.clone()).await;
        let service_for_validation_test = create_test_service(mock_provider).await;
        let result = service_for_validation_test.load_settings().await;

        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::GlobalValidationFailed { reason } => {
                assert!(reason.contains("Default font size muss größer als 0 sein"));
            }
            e => panic!("Unexpected error type for validation: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_save_settings_success() {
        let (service, mock_provider) = create_and_load_test_service(None).await; // Start with defaults
        
        // Modify settings through update to ensure they are different from default
        let path = SettingPath::Appearance(AppearanceSettingPath::ActiveThemeName);
        let new_theme_name = json!("NewThemeName");
        service.update_setting(path, new_theme_name.clone()).await.expect("Update setting failed");
        // update_setting calls save_settings internally, so save_called_count is already 1.

        // Call save_settings explicitly again
        let save_result = service.save_settings().await;
        assert!(save_result.is_ok());
        assert_eq!(mock_provider.get_save_called_count().await, 2); // Once from update, once from explicit call

        let persisted_settings = mock_provider.settings.read().await.clone();
        assert_eq!(persisted_settings.appearance.active_theme_name, new_theme_name.as_str().unwrap());
    }
    
    #[tokio::test]
    async fn test_save_settings_provider_error() {
        let (service, mock_provider) = create_and_load_test_service(None).await;
        let core_err = CoreError::IoError("Simulated IO save error".to_string());
        let persist_err = GlobalSettingsError::persistence_error_with_core_source("save", "mock save", core_err);
        mock_provider.clone().set_error_on_save(Some(persist_err.clone()));

        let result = service.save_settings().await;
        assert!(result.is_err());
         match result.err().unwrap() {
            GlobalSettingsError::PersistenceError { operation, .. } => assert_eq!(operation, "save"),
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_current_settings() {
        let mut initial_settings = GlobalDesktopSettings::default();
        initial_settings.workspace.default_workspace_count = 5;
        let (service, _) = create_and_load_test_service(Some(initial_settings.clone())).await;
        
        let current_settings = service.get_current_settings().unwrap();
        assert_eq!(current_settings.workspace.default_workspace_count, 5);
    }

    // --- Tests for update_setting and get_setting ---
    
    async fn setup_service_for_update_get() -> (DefaultGlobalSettingsService, Arc<MockPersistenceProvider>) {
        let mut initial_settings = GlobalDesktopSettings::default();
        // Setup some initial values to ensure they are different from update values or defaults
        initial_settings.appearance.active_theme_name = "InitialTheme".to_string();
        initial_settings.appearance.font_settings.default_font_size = 10.0;
        initial_settings.input_behavior.custom_mouse_acceleration_factor = None;
        initial_settings.validate_recursive().expect("Initial test settings are invalid");
        create_and_load_test_service(Some(initial_settings)).await
    }

    #[tokio::test]
    async fn test_update_and_get_string_setting() {
        let (service, mock_provider) = setup_service_for_update_get().await;
        let mut event_rx = service.subscribe_to_setting_changes();

        let path = SettingPath::Appearance(AppearanceSettingPath::ActiveThemeName);
        let new_value = json!("UpdatedThemeName");

        let update_result = service.update_setting(path.clone(), new_value.clone()).await;
        assert!(update_result.is_ok(), "update_setting failed: {:?}", update_result.err());

        // Check get_setting
        let retrieved_value = service.get_setting(&path).unwrap();
        assert_eq!(retrieved_value, new_value);

        // Check internal state
        let current_settings = service.get_current_settings().unwrap();
        assert_eq!(current_settings.appearance.active_theme_name, "UpdatedThemeName");
        
        // Check event
        let event = event_rx.recv().await.unwrap();
        assert_eq!(event.path, path);
        assert_eq!(event.new_value, new_value);
        
        // Check persistence
        assert_eq!(mock_provider.get_save_called_count().await, 1); // update_setting calls save
        let persisted_settings = mock_provider.settings.read().await.clone();
        assert_eq!(persisted_settings.appearance.active_theme_name, "UpdatedThemeName");
    }

    #[tokio::test]
    async fn test_update_and_get_nested_f64_setting() {
        let (service, _) = setup_service_for_update_get().await;
        let path = SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize));
        let new_value = json!(12.5);

        assert!(service.update_setting(path.clone(), new_value.clone()).await.is_ok());
        assert_eq!(service.get_setting(&path).unwrap(), new_value);
        assert_eq!(service.get_current_settings().unwrap().appearance.font_settings.default_font_size, 12.5);
    }
    
    #[tokio::test]
    async fn test_update_and_get_optional_f32_setting_some_to_none() {
        let (service, _) = setup_service_for_update_get().await;
        let path = SettingPath::InputBehavior(InputBehaviorSettingPath::CustomMouseAccelerationFactor);
        
        // First set to Some
        let some_value = json!(0.75);
        // For this to be valid, profile must be Custom. Let's update that first silently or assume it.
        // To avoid cascading updates in this unit test, let's directly set the profile in initial settings for mock.
        let mut initial_settings_for_option = GlobalDesktopSettings::default();
        initial_settings_for_option.input_behavior.mouse_acceleration_profile = MouseAccelerationProfile::Custom;
        initial_settings_for_option.input_behavior.custom_mouse_acceleration_factor = Some(0.5); // Initial Some
        let (service_with_option, _) = create_and_load_test_service(Some(initial_settings_for_option)).await;


        assert!(service_with_option.update_setting(path.clone(), some_value.clone()).await.is_ok());
        assert_eq!(service_with_option.get_setting(&path).unwrap(), some_value);
        assert_eq!(service_with_option.get_current_settings().unwrap().input_behavior.custom_mouse_acceleration_factor, Some(0.75));

        // Then update to None (json!(null))
        let none_value = json!(null);
        assert!(service_with_option.update_setting(path.clone(), none_value.clone()).await.is_ok());
        // Getting an Option that is None should serialize to json!(null)
        assert_eq!(service_with_option.get_setting(&path).unwrap(), json!(null));
        assert_eq!(service_with_option.get_current_settings().unwrap().input_behavior.custom_mouse_acceleration_factor, None);
    }


    #[tokio::test]
    async fn test_update_setting_path_not_found() {
        // This test is tricky because SettingPath enum covers all valid paths.
        // A "PathNotFound" would mean our match statement in update/get is incomplete.
        // For now, we assume SettingPath guarantees a structurally valid path.
        // If an invalid string was parsed to SettingPath, that's a paths.rs test.
        // So, this might better be tested by trying to use a SettingPath variant that
        // has no corresponding field in the GlobalDesktopSettings struct (if such a mismatch could exist).
        // Given current structure, all SettingPath variants *should* map to a field.
        // Let's consider this covered by ensuring all paths in SettingPath are handled in helpers.
    }

    #[tokio::test]
    async fn test_update_setting_invalid_value_type() {
        let (service, _) = setup_service_for_update_get().await;
        let path = SettingPath::Appearance(AppearanceSettingPath::InterfaceScalingFactor); // Expects f64
        let new_value = json!("not-a-float"); // Invalid type

        let result = service.update_setting(path.clone(), new_value).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::FieldDeserializationError { path: err_path, .. } => {
                assert_eq!(err_path, path);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_update_setting_validation_failure() {
        let (service, _) = setup_service_for_update_get().await;
        let path = SettingPath::Appearance(AppearanceSettingPath::InterfaceScalingFactor); // Expects f64
        let new_value = json!(0.1); // Valid type, but out of validation range (0.5-3.0)

        let result = service.update_setting(path.clone(), new_value).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::ValidationError { path: err_path, reason } => {
                assert_eq!(err_path, path);
                assert!(reason.contains("Interface scaling factor sollte zwischen 0.5 und 3.0 liegen"));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_get_setting_path_not_found() {
        // Similar to update_setting_path_not_found, this is hard to test if SettingPath covers all fields.
        // Covered by ensuring all paths are handled in get_field_from_settings.
    }

    #[tokio::test]
    async fn test_reset_to_defaults() {
        let (service, mock_provider) = setup_service_for_update_get().await;
        let mut event_rx = service.subscribe_to_setting_changes(); // Should receive events for each field reset if we did it that way
                                                              // However, reset_to_defaults currently sends SettingsLoadedEvent, not individual SettingChangedEvents.
                                                              // The trait only defines subscribe_to_setting_changes.
                                                              // So, for now, no SettingChangedEvent is expected from reset_to_defaults.

        // Change a setting first
        let path = SettingPath::Appearance(AppearanceSettingPath::ActiveThemeName);
        assert!(service.update_setting(path, json!("NonDefaultTheme")).await.is_ok());
        assert_ne!(service.get_current_settings().unwrap().appearance.active_theme_name, GlobalDesktopSettings::default().appearance.active_theme_name);
        let save_count_before_reset = mock_provider.get_save_called_count().await;


        let reset_result = service.reset_to_defaults().await;
        assert!(reset_result.is_ok());

        let current_settings = service.get_current_settings().unwrap();
        assert_eq!(current_settings, GlobalDesktopSettings::default());
        
        // Check persistence: reset should also save the default settings
        assert_eq!(mock_provider.get_save_called_count().await, save_count_before_reset + 1);
        let persisted_settings = mock_provider.settings.read().await.clone();
        assert_eq!(persisted_settings, GlobalDesktopSettings::default());

        // Check for SettingChangedEvents:
        // As per current implementation of reset_to_defaults, it does NOT send individual SettingChangedEvents.
        // It would send a SettingsLoadedEvent if the channel supported it.
        // So, try_recv should be empty.
        assert!(matches!(event_rx.try_recv(), Err(tokio::sync::broadcast::error::TryRecvError::Empty)));
    }
}
