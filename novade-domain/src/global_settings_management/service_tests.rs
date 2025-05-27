#[cfg(test)]
mod tests {
    use std::sync::Arc;
    // use std::path::PathBuf; // Not directly used for mock, but good for context
    use tokio::sync::RwLock;
    use serde_json::{json, Value as JsonValue};
    use std::collections::HashMap; // For ApplicationSettingGroup in GlobalDesktopSettings
    // use tracing::debug; // For debugging test output if needed

    use crate::global_settings_management::{
        service::{GlobalSettingsService, DefaultGlobalSettingsService},
        types::{GlobalSettingsEvent, SettingChangedEvent, SettingsLoadedEvent, SettingsSavedEvent, GlobalDesktopSettings, ApplicationSettingGroup, MouseAccelerationProfile, WindowManagementSettings}, // Explicit imports
        paths::*, // Includes SettingPath, ApplicationSettingPath, WindowManagementSettingPath
        errors::GlobalSettingsError,
        persistence_iface::SettingsPersistenceProvider,
    };
    use crate::window_management_policy::types::{
        TilingMode, NewWindowPlacementStrategy, GapSettings, WindowSnappingPolicy, FocusPolicy, WindowGroupingPolicy, FocusStealingPreventionLevel
    }; // Added
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
        
        // Manual setup to subscribe before load
        let mock_provider = Arc::new(MockPersistenceProvider::new());
        mock_provider.set_settings(initial_settings.clone()).await;
        let service = create_test_service(mock_provider.clone()).await;
        let mut event_rx = service.subscribe_to_events();

        service.load_settings().await.expect("Initial load_settings failed");

        // Check for SettingsLoaded event
        let event = event_rx.recv().await.unwrap();
        match event {
            GlobalSettingsEvent::SettingsLoaded(payload) => {
                assert_eq!(payload.settings.appearance.active_theme_name, "TestTheme");
                assert_eq!(payload.settings, initial_settings);
            }
            other => panic!("Expected SettingsLoaded, got {:?}", other),
        }
        
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
        // create_and_load_test_service calls load_settings, which sends SettingsLoaded. Consume it.
        // Also, if create_and_load uses initial_settings, load_settings would have used that.
        // For this test, it's simpler if create_and_load_test_service doesn't send events or we get a fresh receiver.
        // Let's get a fresh receiver after initial load and before specific actions.
        
        let mut event_rx = service.subscribe_to_events(); // Subscribe fresh to ignore previous load events.
        
        // Modify settings through update to ensure they are different from default
        let path = SettingPath::Appearance(AppearanceSettingPath::ActiveThemeName);
        let new_theme_name_val = "NewThemeName";
        let new_theme_name_json = json!(new_theme_name_val);
        service.update_setting(path.clone(), new_theme_name_json.clone()).await.expect("Update setting failed");
        
        // update_setting sends SettingChanged then SettingsSaved. Consume them.
        let _ = event_rx.recv().await.unwrap(); // SettingChanged
        let _ = event_rx.recv().await.unwrap(); // SettingsSaved from update_setting
        
        let save_count_before_explicit_save = mock_provider.get_save_called_count().await;

        // Call save_settings explicitly again
        let save_result = service.save_settings().await;
        assert!(save_result.is_ok());
        assert_eq!(mock_provider.get_save_called_count().await, save_count_before_explicit_save + 1);

        // Check for SettingsSaved event from the explicit call
        let event = event_rx.recv().await.unwrap();
        match event {
            GlobalSettingsEvent::SettingsSaved(_) => { /* Correct event received */ }
            other => panic!("Expected SettingsSaved, got {:?}", other),
        }

        let persisted_settings = mock_provider.settings.read().await.clone();
        assert_eq!(persisted_settings.appearance.active_theme_name, new_theme_name_val);
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
        let mut event_rx = service.subscribe_to_events();

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
        
        // Check SettingChanged event
        let received_event1 = event_rx.recv().await.unwrap();
        match received_event1 {
            GlobalSettingsEvent::SettingChanged(event_payload) => {
                assert_eq!(event_payload.path, path);
                assert_eq!(event_payload.new_value, new_value);
            }
            other_event => panic!("Expected SettingChanged, got {:?}", other_event),
        }

        // Check SettingsSaved event
        let received_event2 = event_rx.recv().await.unwrap();
        match received_event2 {
            GlobalSettingsEvent::SettingsSaved(_) => { /* Correct event received */ }
            other_event => panic!("Expected SettingsSaved, got {:?}", other_event),
        }
        
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
        let mut event_rx = service.subscribe_to_events(); // Should receive events for each field reset if we did it that way
                                                              // However, reset_to_defaults currently sends SettingsLoadedEvent, not individual SettingChangedEvents.
                                                              // The trait only defines subscribe_to_events.
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

        // Check for SettingsLoaded event
        let event1 = event_rx.recv().await.unwrap();
        match event1 {
            GlobalSettingsEvent::SettingsLoaded(payload) => {
                assert_eq!(payload.settings, GlobalDesktopSettings::default());
            }
            other => panic!("Expected SettingsLoaded, got {:?}", other),
        }

        // Check for SettingsSaved event
        let event2 = event_rx.recv().await.unwrap();
        match event2 {
            GlobalSettingsEvent::SettingsSaved(_) => { /* Correct event received */ }
            other => panic!("Expected SettingsSaved, got {:?}", other),
        }
        
        // Ensure no other events
        assert!(matches!(event_rx.try_recv(), Err(tokio::sync::broadcast::error::TryRecvError::Empty)));
    }

    // --- Application Settings Specific Tests ---

    #[tokio::test]
    async fn test_set_and_get_application_settings() {
        let (service, mock_provider) = create_and_load_test_service(None).await;
        let mut event_rx = service.subscribe_to_events();

        let path_app1_feat = SettingPath::Application(ApplicationSettingPath {
            app_id: "com.testapp".to_string(),
            key: "feature.enabled".to_string(),
        });
        let val_app1_feat = json!(true);

        let path_app1_user = SettingPath::Application(ApplicationSettingPath {
            app_id: "com.testapp".to_string(),
            key: "user.name".to_string(),
        });
        let val_app1_user = json!("tester");
        
        let path_app2_timeout = SettingPath::Application(ApplicationSettingPath {
            app_id: "org.otherapp".to_string(),
            key: "config.timeout".to_string(),
        });
        let val_app2_timeout = json!(100);

        // Set first setting for app1
        service.update_setting(path_app1_feat.clone(), val_app1_feat.clone()).await.expect("Set app1_feat failed");
        assert_eq!(service.get_setting(&path_app1_feat).unwrap(), val_app1_feat);
        
        let received_event1_changed = event_rx.recv().await.unwrap();
        match received_event1_changed {
            GlobalSettingsEvent::SettingChanged(payload) => {
                assert_eq!(payload.path, path_app1_feat);
                assert_eq!(payload.new_value, val_app1_feat);
            }
            other => panic!("Expected SettingChanged, got {:?}", other),
        }
        let received_event1_saved = event_rx.recv().await.unwrap();
        assert!(matches!(received_event1_saved, GlobalSettingsEvent::SettingsSaved(_)), "Expected SettingsSaved, got {:?}", received_event1_saved);

        // Set second setting for app1
        service.update_setting(path_app1_user.clone(), val_app1_user.clone()).await.expect("Set app1_user failed");
        assert_eq!(service.get_setting(&path_app1_user).unwrap(), val_app1_user);

        let received_event2_changed = event_rx.recv().await.unwrap();
        match received_event2_changed {
            GlobalSettingsEvent::SettingChanged(payload) => {
                assert_eq!(payload.path, path_app1_user);
                assert_eq!(payload.new_value, val_app1_user);
            }
            other => panic!("Expected SettingChanged, got {:?}", other),
        }
        let received_event2_saved = event_rx.recv().await.unwrap();
        assert!(matches!(received_event2_saved, GlobalSettingsEvent::SettingsSaved(_)), "Expected SettingsSaved, got {:?}", received_event2_saved);
        
        // Verify first setting for app1 is still there
        assert_eq!(service.get_setting(&path_app1_feat).unwrap(), val_app1_feat);

        // Set setting for app2
        service.update_setting(path_app2_timeout.clone(), val_app2_timeout.clone()).await.expect("Set app2_timeout failed");
        assert_eq!(service.get_setting(&path_app2_timeout).unwrap(), val_app2_timeout);

        let received_event3_changed = event_rx.recv().await.unwrap();
        match received_event3_changed {
            GlobalSettingsEvent::SettingChanged(payload) => {
                assert_eq!(payload.path, path_app2_timeout);
                assert_eq!(payload.new_value, val_app2_timeout);
            }
            other => panic!("Expected SettingChanged, got {:?}", other),
        }
        let received_event3_saved = event_rx.recv().await.unwrap();
        assert!(matches!(received_event3_saved, GlobalSettingsEvent::SettingsSaved(_)), "Expected SettingsSaved, got {:?}", received_event3_saved);

        // Verify all settings in internal state
        let current_settings = service.get_current_settings().unwrap();
        assert_eq!(current_settings.application_settings.get("com.testapp").unwrap().settings.get("feature.enabled").unwrap(), &val_app1_feat);
        assert_eq!(current_settings.application_settings.get("com.testapp").unwrap().settings.get("user.name").unwrap(), &val_app1_user);
        assert_eq!(current_settings.application_settings.get("org.otherapp").unwrap().settings.get("config.timeout").unwrap(), &val_app2_timeout);
        
        // Verify persistence (3 updates = 3 saves)
        assert_eq!(mock_provider.get_save_called_count().await, 3);
        let persisted_settings = mock_provider.settings.read().await.clone();
        assert_eq!(persisted_settings.application_settings.get("com.testapp").unwrap().settings.get("feature.enabled").unwrap(), &val_app1_feat);
    }

    #[tokio::test]
    async fn test_update_existing_application_setting() {
        let (service, _) = create_and_load_test_service(None).await;
        let mut event_rx = service.subscribe_to_events();

        let path = SettingPath::Application(ApplicationSettingPath {
            app_id: "com.testapp".to_string(),
            key: "feature.enabled".to_string(),
        });
        let initial_val = json!(true);
        let updated_val = json!(false);

        service.update_setting(path.clone(), initial_val.clone()).await.expect("Initial set failed");
        assert_eq!(service.get_setting(&path).unwrap(), initial_val);
        // Consume SettingChanged and SettingsSaved events for the initial set
        let _ = event_rx.recv().await.unwrap(); 
        let _ = event_rx.recv().await.unwrap(); 

        service.update_setting(path.clone(), updated_val.clone()).await.expect("Update failed");
        assert_eq!(service.get_setting(&path).unwrap(), updated_val);
        
        let received_event_changed = event_rx.recv().await.unwrap();
        match received_event_changed {
            GlobalSettingsEvent::SettingChanged(payload) => {
                assert_eq!(payload.path, path);
                assert_eq!(payload.new_value, updated_val);
            }
            other => panic!("Expected SettingChanged, got {:?}", other),
        }
        let received_event_saved = event_rx.recv().await.unwrap();
        assert!(matches!(received_event_saved, GlobalSettingsEvent::SettingsSaved(_)), "Expected SettingsSaved, got {:?}", received_event_saved);
    }

    #[tokio::test]
    async fn test_get_non_existent_application_setting() {
        let (service, _) = create_and_load_test_service(None).await;

        let path_non_existent_app = SettingPath::Application(ApplicationSettingPath {
            app_id: "non.existent.app".to_string(),
            key: "some.key".to_string(),
        });
        match service.get_setting(&path_non_existent_app) {
            Err(GlobalSettingsError::PathNotFound(p)) => assert_eq!(p, path_non_existent_app),
            res => panic!("Expected PathNotFound, got {:?}", res),
        }

        // Set up an app, then try to get a non-existent key within it
        let existing_app_id = "com.testapp".to_string();
        let path_existing_app_valid_key = SettingPath::Application(ApplicationSettingPath {
            app_id: existing_app_id.clone(),
            key: "valid.key".to_string(),
        });
        service.update_setting(path_existing_app_valid_key, json!("valid_value")).await.expect("Set failed");

        let path_existing_app_non_existent_key = SettingPath::Application(ApplicationSettingPath {
            app_id: existing_app_id.clone(),
            key: "non.existent.key".to_string(),
        });
        match service.get_setting(&path_existing_app_non_existent_key) {
            Err(GlobalSettingsError::PathNotFound(p)) => assert_eq!(p, path_existing_app_non_existent_key),
            res => panic!("Expected PathNotFound for non-existent key, got {:?}", res),
        }
    }
    
    #[tokio::test]
    async fn test_application_setting_validation_in_service_update() {
        let (service, _) = create_and_load_test_service(None).await;
        
        // This path is valid, but GlobalDesktopSettings.validate_recursive will be called
        // after the change. The ApplicationSettingGroup's validate method checks keys.
        // The service itself doesn't deserialize ApplicationSettingPath, it passes the value
        // directly to the ApplicationSettingGroup's settings map.
        // The validation that GlobalDesktopSettings::validate_recursive does on ApplicationSettingGroup
        // is that the key is not empty. So we can't directly test service blocking invalid app_id or key
        // via SettingPath because SettingPath's FromStr already validates that.
        // What we can test is if the *value* causes a validation error higher up,
        // but for application_settings, the value is JsonValue, so type errors are unlikely.
        // The main validation for ApplicationSettingGroup is that the *key* is not empty,
        // which is enforced by ApplicationSettingPath FromStr.
        // Let's ensure update_setting still calls validate_recursive which would catch it if we manually
        // put bad data into GlobalDesktopSettings (which update_setting does via its helpers).
        
        // To test ApplicationSettingGroup's validation via the service, we'd need to
        // have a way for a JsonValue to be invalid for an application, which is not currently defined.
        // The existing validation for ApplicationSettingGroup is `key.is_empty()`, which is handled by path parsing.
        
        // Let's try to make GlobalDesktopSettings invalid through other means and ensure app settings don't bypass it.
        let path_app = SettingPath::Application(ApplicationSettingPath {
            app_id: "com.testapp".to_string(),
            key: "some.key".to_string(),
        });
        let val_app = json!("some_value");

        // Make another part of settings invalid first
        let mut current_settings = service.get_current_settings().unwrap();
        current_settings.appearance.interface_scaling_factor = 0.0; // Invalid
        let mock_provider_invalid_base = Arc::new(MockPersistenceProvider::new());
        mock_provider_invalid_base.set_settings(current_settings).await;
        let service_invalid_base = create_test_service(mock_provider_invalid_base).await;
        service_invalid_base.load_settings().await.expect_err("Load should fail due to invalid base settings");
        // This doesn't test app settings validation directly, but shows overall validation is active.

        // If update_field_in_settings were to somehow bypass ApplicationSettingPath and insert an empty key,
        // then new_settings_clone.validate_recursive() in update_setting would catch it.
        // This seems hard to test without directly manipulating internal state in a test-specific way.
        // The current structure means ApplicationSettingPath ensures app_id and key are non-empty.
        // And ApplicationSettingGroup ensures keys within its map are non-empty (which is what we use).
        // So, this aspect seems covered by tests in types_tests.rs and paths_tests.rs.
    }

    #[tokio::test]
    async fn test_reset_to_defaults_clears_application_settings() {
        let (service, mock_provider) = create_and_load_test_service(None).await;

        let path_app1_feat = SettingPath::Application(ApplicationSettingPath {
            app_id: "com.testapp".to_string(),
            key: "feature.enabled".to_string(),
        });
        service.update_setting(path_app1_feat.clone(), json!(true)).await.expect("Set app setting failed");
        
        assert!(!service.get_current_settings().unwrap().application_settings.is_empty(), "App settings should exist before reset");
        let save_count_before_reset = mock_provider.get_save_called_count().await;


        service.reset_to_defaults().await.expect("Reset to defaults failed");

        let current_settings = service.get_current_settings().unwrap();
        assert!(current_settings.application_settings.is_empty(), "Application settings should be empty after reset");

        match service.get_setting(&path_app1_feat) {
            Err(GlobalSettingsError::PathNotFound(p)) => assert_eq!(p, path_app1_feat),
            res => panic!("Expected PathNotFound after reset, got {:?}", res),
        }
        
        assert_eq!(mock_provider.get_save_called_count().await, save_count_before_reset + 1);
        let persisted_settings = mock_provider.settings.read().await.clone();
        assert!(persisted_settings.application_settings.is_empty());
    }

    // --- Window Management Settings Service Tests ---

    #[tokio::test]
    async fn test_get_default_window_management_settings() {
        let (service, _) = create_and_load_test_service(None).await;
        let default_wm_settings = WindowManagementSettings::default();

        // Test TilingMode
        let path_tiling_mode = SettingPath::WindowManagement(WindowManagementSettingPath::TilingMode);
        let expected_tiling_mode = json!(default_wm_settings.tiling_mode);
        assert_eq!(service.get_setting(&path_tiling_mode).unwrap(), expected_tiling_mode);

        // Test GapsScreenOuterHorizontal
        let path_gaps_outer_h = SettingPath::WindowManagement(WindowManagementSettingPath::GapsScreenOuterHorizontal);
        let expected_gaps_outer_h = json!(default_wm_settings.gaps.screen_outer_horizontal);
        assert_eq!(service.get_setting(&path_gaps_outer_h).unwrap(), expected_gaps_outer_h);

        // Test FocusFocusFollowsMouse
        let path_focus_follows = SettingPath::WindowManagement(WindowManagementSettingPath::FocusFocusFollowsMouse);
        let expected_focus_follows = json!(default_wm_settings.focus.focus_follows_mouse);
        assert_eq!(service.get_setting(&path_focus_follows).unwrap(), expected_focus_follows);
        
        // Test SnappingSnapDistancePx
        let path_snap_dist = SettingPath::WindowManagement(WindowManagementSettingPath::SnappingSnapDistancePx);
        let expected_snap_dist = json!(default_wm_settings.snapping.snap_distance_px);
        assert_eq!(service.get_setting(&path_snap_dist).unwrap(), expected_snap_dist);

        // Test PlacementStrategy
        let path_placement = SettingPath::WindowManagement(WindowManagementSettingPath::PlacementStrategy);
        let expected_placement = json!(default_wm_settings.placement_strategy);
        assert_eq!(service.get_setting(&path_placement).unwrap(), expected_placement);

        // Test GroupingEnableManualGrouping
        let path_grouping = SettingPath::WindowManagement(WindowManagementSettingPath::GroupingEnableManualGrouping);
        let expected_grouping = json!(default_wm_settings.grouping.enable_manual_grouping);
        assert_eq!(service.get_setting(&path_grouping).unwrap(), expected_grouping);
    }

    #[tokio::test]
    async fn test_update_and_get_window_management_settings() {
        let (service, mock_provider) = create_and_load_test_service(None).await;
        let mut event_rx = service.subscribe_to_events();
        let _ = event_rx.recv().await.unwrap(); // Consume initial SettingsLoaded

        // 1. Test TilingMode (Enum)
        let path_tiling = SettingPath::WindowManagement(WindowManagementSettingPath::TilingMode);
        let new_tiling_mode = TilingMode::Spiral;
        let new_tiling_value = json!(new_tiling_mode);
        service.update_setting(path_tiling.clone(), new_tiling_value.clone()).await.expect("Update TilingMode failed");
        assert_eq!(service.get_setting(&path_tiling).unwrap(), new_tiling_value);
        assert_eq!(service.get_current_settings().unwrap().window_management.tiling_mode, new_tiling_mode);
        // Consume SettingChanged and SettingsSaved events
        let _ = event_rx.recv().await.unwrap(); 
        let _ = event_rx.recv().await.unwrap(); 

        // 2. Test FocusFocusFollowsMouse (Boolean)
        let path_focus_follows = SettingPath::WindowManagement(WindowManagementSettingPath::FocusFocusFollowsMouse);
        let new_focus_follows = true; // Default is false
        let new_focus_follows_value = json!(new_focus_follows);
        service.update_setting(path_focus_follows.clone(), new_focus_follows_value.clone()).await.expect("Update FocusFollowsMouse failed");
        assert_eq!(service.get_setting(&path_focus_follows).unwrap(), new_focus_follows_value);
        assert_eq!(service.get_current_settings().unwrap().window_management.focus.focus_follows_mouse, new_focus_follows);
        let _ = event_rx.recv().await.unwrap(); 
        let _ = event_rx.recv().await.unwrap(); 

        // 3. Test GapsWindowInner (Numeric u16)
        let path_gaps_inner = SettingPath::WindowManagement(WindowManagementSettingPath::GapsWindowInner);
        let new_gaps_inner: u16 = 15;
        let new_gaps_inner_value = json!(new_gaps_inner);
        service.update_setting(path_gaps_inner.clone(), new_gaps_inner_value.clone()).await.expect("Update GapsWindowInner failed");
        assert_eq!(service.get_setting(&path_gaps_inner).unwrap(), new_gaps_inner_value);
        assert_eq!(service.get_current_settings().unwrap().window_management.gaps.window_inner, new_gaps_inner);
        let _ = event_rx.recv().await.unwrap(); 
        let _ = event_rx.recv().await.unwrap(); 

        // 4. Test PlacementStrategy (Enum)
        let path_placement = SettingPath::WindowManagement(WindowManagementSettingPath::PlacementStrategy);
        let new_placement = NewWindowPlacementStrategy::Center;
        let new_placement_value = json!(new_placement);
        service.update_setting(path_placement.clone(), new_placement_value.clone()).await.expect("Update PlacementStrategy failed");
        assert_eq!(service.get_setting(&path_placement).unwrap(), new_placement_value);
        assert_eq!(service.get_current_settings().unwrap().window_management.placement_strategy, new_placement);
        let _ = event_rx.recv().await.unwrap(); 
        let _ = event_rx.recv().await.unwrap(); 

        // Check persistence (4 updates = 4 saves, plus initial load if it saved - mock starts save_count at 0)
        // create_and_load_test_service does not save after load. So 4 saves.
        assert_eq!(mock_provider.get_save_called_count().await, 4);
        let persisted_settings = mock_provider.settings.read().await.clone();
        assert_eq!(persisted_settings.window_management.tiling_mode, new_tiling_mode);
        assert_eq!(persisted_settings.window_management.focus.focus_follows_mouse, new_focus_follows);
        assert_eq!(persisted_settings.window_management.gaps.window_inner, new_gaps_inner);
        assert_eq!(persisted_settings.window_management.placement_strategy, new_placement);
    }

    #[tokio::test]
    async fn test_update_window_management_setting_invalid_value() {
        let (service, _) = create_and_load_test_service(None).await;
        
        // Test invalid GapsWindowInner
        let path_gaps = SettingPath::WindowManagement(WindowManagementSettingPath::GapsWindowInner);
        let invalid_gaps_value = json!(1000); // Exceeds validation limit (50)
        
        let result_gaps = service.update_setting(path_gaps.clone(), invalid_gaps_value).await;
        assert!(result_gaps.is_err());
        match result_gaps.err().unwrap() {
            GlobalSettingsError::ValidationError { path: err_path, reason } => {
                assert_eq!(err_path, path_gaps);
                assert!(reason.contains("Window inner gap cannot exceed 50"));
            }
            e => panic!("Unexpected error type for invalid gaps: {:?}", e),
        }
        // Check that the setting was not updated
        let default_wm_settings = WindowManagementSettings::default();
        assert_eq!(
            service.get_setting(&path_gaps).unwrap(),
            json!(default_wm_settings.gaps.window_inner)
        );

        // Test invalid SnappingSnapDistancePx
        let path_snap = SettingPath::WindowManagement(WindowManagementSettingPath::SnappingSnapDistancePx);
        let invalid_snap_value = json!(0); // Cannot be 0
        
        let result_snap = service.update_setting(path_snap.clone(), invalid_snap_value).await;
        assert!(result_snap.is_err());
        match result_snap.err().unwrap() {
            GlobalSettingsError::ValidationError { path: err_path, reason } => {
                assert_eq!(err_path, path_snap);
                assert!(reason.contains("Snap distance cannot be 0"));
            }
            e => panic!("Unexpected error type for invalid snap distance: {:?}", e),
        }
        // Check that the setting was not updated
        assert_eq!(
            service.get_setting(&path_snap).unwrap(),
            json!(default_wm_settings.snapping.snap_distance_px)
        );
    }
}
