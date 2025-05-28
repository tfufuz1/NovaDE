#[cfg(test)]
mod tests {
    use super::super::service::{ThemingEngine, ThemeChangedEvent, DEFAULT_GLOBAL_TOKENS_PATH, FALLBACK_THEME_PATH};
    use super::super::types::{ThemingConfiguration, ThemeIdentifier, ColorSchemeType, AppliedThemeState, TokenIdentifier, TokenSet, RawToken, TokenValue, ThemeDefinition};
    use super::super::errors::ThemingError;
    use novade_core::config::{ConfigServiceAsync, ConfigFormat};
    use novade_core::errors::CoreError;
    use novade_core::types::Color as CoreColor;
    use std::sync::Arc;
    use std::collections::HashMap;
    use async_trait::async_trait;
    use tokio::time::{timeout, Duration};

    // --- Mock ConfigServiceAsync (copied from logic_tests.rs, consider centralizing if used more) ---
    #[derive(Debug, Clone, Default)]
    struct MockConfigService {
        files: HashMap<String, String>,
        error_on_load: Option<CoreError>,
        error_on_save: Option<CoreError>,
        error_on_list: Option<CoreError>,
    }

    impl MockConfigService {
        fn new() -> Self {
            Default::default()
        }

        fn add_file(&mut self, path: &str, content: &str) {
            self.files.insert(path.to_string(), content.to_string());
        }
        
        #[allow(dead_code)]
        fn set_error_on_load(&mut self, error: Option<CoreError>) {
            self.error_on_load = error;
        }
    }

    #[async_trait]
    impl ConfigServiceAsync for MockConfigService {
        async fn read_config_file_string(&self, file_path: &str) -> Result<String, CoreError> { // Renamed method
            if let Some(err) = &self.error_on_load {
                return Err(err.clone());
            }
            self.files
                .get(file_path)
                .cloned()
                // Make error consistent with DefaultFileSystemConfigService and logic.rs expectations
                .ok_or_else(|| CoreError::Config(novade_core::ConfigError::NotFound{locations: vec![file_path.into()]}))
        }

        // --- Dummy implementations for other ConfigServiceAsync methods if needed by tests ---
        // --- (Copied from DefaultFileSystemConfigService for consistency, can be unimplemented!) ---
        async fn write_config_file_string(&self, _file_path: &str, _content: String) -> Result<(), CoreError> { 
            if let Some(err) = &self.error_on_save { return Err(err.clone()); }
            // In a mock, you might store the written content if needed for assertions
            Ok(())
        }
        async fn read_file_to_string(&self, path: &std::path::Path) -> Result<String, CoreError> { 
            self.read_config_file_string(path.to_str().unwrap_or_default()).await
        }
        async fn list_files_in_dir(&self, _dir_path: &std::path::Path, _extension: Option<&str>) -> Result<Vec<std::path::PathBuf>, CoreError> { 
            if let Some(err) = &self.error_on_list { return Err(err.clone()); }
            Ok(self.files.keys().map(std::path::PathBuf::from).collect())
        }
        async fn get_config_dir(&self) -> Result<std::path::PathBuf, CoreError> { 
            Ok(std::path::PathBuf::from("./mock_config_dir")) // Placeholder
        }
        async fn get_data_dir(&self) -> Result<std::path::PathBuf, CoreError> { 
            Ok(std::path::PathBuf::from("./mock_data_dir")) // Placeholder
        }
    }
    
    fn get_valid_base_tokens_json() -> String {
        r#"[
            {"id": "color-global-black", "value": {"color": "#000000"}},
            {"id": "spacing-global-medium", "value": {"spacing": "8px"}}
        ]"#.to_string()
    }

    fn get_valid_fallback_theme_json() -> String {
        r#"{
            "id": "fallback-dark",
            "name": "Test Fallback Dark",
            "base_tokens": {
                "color-background": {"id": "color-background", "value": {"color": "#1E1E1E"}},
                "color-foreground": {"id": "color-foreground", "value": {"color": "#D4D4D4"}}
            },
            "variants": []
        }"#.to_string()
    }

    #[tokio::test]
    async fn test_theming_engine_new_success() {
        let mut mock_service = MockConfigService::new();
        mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        mock_service.add_file(FALLBACK_THEME_PATH, &get_valid_fallback_theme_json());

        let engine_result = ThemingEngine::new(Arc::new(mock_service), None).await;
        assert!(engine_result.is_ok(), "Engine creation failed: {:?}", engine_result.err());
        let engine = engine_result.unwrap();

        let current_config = engine.get_current_configuration().await;
        assert_eq!(current_config.selected_theme_id.as_str(), "fallback-dark"); // Default initial config

        let current_state = engine.get_current_theme_state().await;
        assert_eq!(current_state.theme_id.as_str(), "fallback-dark");
        assert_eq!(current_state.color_scheme, ColorSchemeType::Dark); // from fallback theme logic
        assert!(current_state.resolved_tokens.contains_key(&TokenIdentifier::new("color-background")));
        assert!(current_state.resolved_tokens.contains_key(&TokenIdentifier::new("color-global-black")));
    }

    #[tokio::test]
    async fn test_theming_engine_new_missing_base_tokens_critical_error() {
        let mut mock_service = MockConfigService::new();
        // Missing DEFAULT_GLOBAL_TOKENS_PATH
        mock_service.add_file(FALLBACK_THEME_PATH, &get_valid_fallback_theme_json());

        let engine_result = ThemingEngine::new(Arc::new(mock_service), None).await;
        assert!(engine_result.is_err());
        match engine_result.err().unwrap() {
            ThemingError::InternalError(msg) if msg.contains("Konfigurationsdatei nicht gefunden") && msg.contains(DEFAULT_GLOBAL_TOKENS_PATH) => {},
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_theming_engine_new_missing_fallback_theme_critical_error() {
        let mut mock_service = MockConfigService::new();
        mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        // Missing FALLBACK_THEME_PATH

        let engine_result = ThemingEngine::new(Arc::new(mock_service), None).await;
        assert!(engine_result.is_err());
        match engine_result.err().unwrap() {
            ThemingError::InternalError(msg) if msg.contains("Fallback-Theme konnte nicht geladen werden") && msg.contains(FALLBACK_THEME_PATH) => {},
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_theming_engine_new_with_initial_config() {
        let mut mock_service = MockConfigService::new();
        mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        mock_service.add_file(FALLBACK_THEME_PATH, &get_valid_fallback_theme_json());
        // Add another theme file for the initial config to select
        let custom_theme_id_str = "my-custom-initial-theme";
        let custom_theme_path = "src/theming/default_themes/custom.theme.json"; // Needs to be loaded by load_resources
         let custom_theme_json = format!(r#"{{
            "id": "{}",
            "name": "My Custom Initial Theme",
            "base_tokens": {{
                "color-custom-primary": {{"id": "color-custom-primary", "value": {{"color": "#CustomPrimary"}}}}
            }},
            "variants": []
        }}"#, custom_theme_id_str);
        mock_service.add_file(custom_theme_path, &custom_theme_json);
        
        // To make ThemingEngineInternalState::load_resources find this custom theme,
        // we need to adjust how themes are loaded or provide the path.
        // For this test, we'll assume FALLBACK_THEME_PATH is the only one explicitly loaded by name in load_resources for now.
        // The test for `list_available_themes` or loading multiple themes would be more involved.
        // So, let's make the initial config select the fallback theme to simplify this specific test.
        // A more robust `load_resources` would scan a directory or take a list of theme files.

        let initial_config = ThemingConfiguration {
            selected_theme_id: ThemeIdentifier::new("fallback-dark"), // Select the loaded fallback theme
            preferred_color_scheme: ColorSchemeType::Light, // Different from fallback's default dark
            selected_accent_color: Some(CoreColor::from_hex("#FF0000").unwrap()),
            custom_user_token_overrides: None,
        };

        let engine_result = ThemingEngine::new(Arc::new(mock_service), Some(initial_config.clone())).await;
        assert!(engine_result.is_ok(), "Engine creation failed: {:?}", engine_result.err());
        let engine = engine_result.unwrap();

        let current_config = engine.get_current_configuration().await;
        assert_eq!(current_config.selected_theme_id, initial_config.selected_theme_id);
        assert_eq!(current_config.preferred_color_scheme, initial_config.preferred_color_scheme);
        assert_eq!(current_config.selected_accent_color, initial_config.selected_accent_color);

        let current_state = engine.get_current_theme_state().await;
        assert_eq!(current_state.theme_id, initial_config.selected_theme_id);
        assert_eq!(current_state.color_scheme, initial_config.preferred_color_scheme);
        assert_eq!(current_state.active_accent_color, initial_config.selected_accent_color);
    }

    // More tests to come: update_configuration, reload_themes_and_tokens, event broadcasting, caching

    #[tokio::test]
    async fn test_update_configuration_success_and_event() {
        let mut mock_service = MockConfigService::new();
        mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        mock_service.add_file(FALLBACK_THEME_PATH, &get_valid_fallback_theme_json());
        
        // Add a "light-theme"
        let light_theme_id_str = "light-theme";
        let light_theme_path = "src/theming/default_themes/light.theme.json";
        let light_theme_json = format!(r#"{{
            "id": "{}", "name": "Light Theme",
            "base_tokens": {{ "color-text": {{"id":"color-text", "value":{{"color":"#000000"}}}} }},
            "variants": [ {{"applies_to_scheme": "light", "tokens": {{}} }} ]
        }}"#, light_theme_id_str);
        mock_service.add_file(light_theme_path, &light_theme_json);

        // Modify ThemingEngineInternalState::load_resources to load this theme too.
        // This is tricky as load_resources is internal. For this test, we'll assume
        // that if FALLBACK_THEME_PATH is "src/theming/default_themes/fallback.theme.json",
        // then other themes in "src/theming/default_themes/" might be loaded by a future
        // improved load_resources. For now, we need to ensure `available_themes` in the
        // engine's state gets populated.
        // A simple way for testing is to have `load_theme_definition_from_file` also add to `available_themes`
        // or have `ThemingEngine::new` load all themes found by `list_config_files_async`.
        // The current `ThemingEngineInternalState::load_resources` only loads FALLBACK_THEME_PATH.
        // So, let's adjust the test to reflect what's currently implemented or
        // make a small adjustment to load_resources if it's trivial for testing.

        // Simpler approach for now: The engine is initialized, then we load the new theme definition
        // into its state manually for the purpose of this test, or assume it's loaded.
        // For a clean test, it's better if the engine can load it.
        // Let's assume `ThemingEngine::new` or `reload_themes_and_tokens` would populate `available_themes`.
        // We will test `update_configuration` to a theme that is assumed to be loaded.
        // The current `load_resources` in `service.rs` loads `FALLBACK_THEME_PATH`.
        // If we want to switch to `light-theme`, it must be in `available_themes`.
        // The test `test_theming_engine_new_success` confirms `fallback-dark` is loaded.
        // We'll need to use `reload_themes_and_tokens` or modify the mock to make `light-theme` available.

        // Let's re-initialize the engine and then try to update.
        // We need a ThemeDefinition for "light-theme" to be available.
        // The easiest way is to make `load_resources` load it.
        // Since `load_resources` is not directly testable without more complex setup,
        // we will assume `light-theme` is made available through some mechanism.
        // For this test, we will focus on the update logic assuming the theme is loadable.
        // We'll prepare the mock_service to have the light theme file.
        // The `ThemingEngine::new` will load `fallback-dark`.
        // Then, `update_configuration` will try to apply `light-theme`.
        // It will fail if `light-theme` is not in `available_themes`.
        // This means `ThemingEngine::internal_apply_configuration_logic` will fail ThemeNotFound.
        // So, the test setup must ensure `light-theme` is loaded into `available_themes`.
        // This implies we need a way to add themes after `new()`, e.g. via `reload_themes_and_tokens`
        // or by making `load_resources` more flexible.

        // For this specific test of `update_configuration`, let's assume that the `light-theme`
        // was loaded during engine initialization (e.g., `FALLBACK_THEME_PATH` was temporarily
        // pointed to `light_theme_json` for the purpose of getting it into `available_themes`).
        // This is a bit of a hack. A cleaner way would be a method like `load_theme_definition_into_engine`.
        // Or, `reload_themes_and_tokens` should be called first if it could pick up new themes.

        let engine = ThemingEngine::new(Arc::new(mock_service.clone()), None).await.unwrap();
        let mut rx = engine.subscribe_to_theme_changes();

        // Manually insert the light theme into the engine's state for this test.
        // This is not ideal but helps test update_configuration's logic directly.
        // In a real scenario, this theme would be loaded by `load_resources`.
        let light_theme_def: ThemeDefinition = serde_json::from_str(&light_theme_json).unwrap();
        engine.internal_state.lock().await.available_themes.insert(ThemeIdentifier::new(light_theme_id_str), light_theme_def.clone());


        let new_config = ThemingConfiguration {
            selected_theme_id: ThemeIdentifier::new(light_theme_id_str),
            preferred_color_scheme: ColorSchemeType::Light,
            selected_accent_color: None,
            custom_user_token_overrides: None,
        };

        let update_result = engine.update_configuration(new_config.clone()).await;
        assert!(update_result.is_ok(), "update_configuration failed: {:?}", update_result.err());
        let applied_state = update_result.unwrap();

        assert_eq!(applied_state.theme_id.as_str(), light_theme_id_str);
        assert_eq!(applied_state.color_scheme, ColorSchemeType::Light);
        assert!(applied_state.resolved_tokens.contains_key(&TokenIdentifier::new("color-text")));

        // Check for event
        match timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Ok(event)) => {
                assert_eq!(event.new_state, applied_state);
            }
            Ok(Err(e)) => panic!("Error receiving event: {}", e),
            Err(_) => panic!("Timeout waiting for theme change event"),
        }
        
        // Verify internal config updated
        let internal_config = engine.get_current_configuration().await;
        assert_eq!(internal_config, new_config);
    }

    #[tokio::test]
    async fn test_update_configuration_theme_not_found_reverts() {
        let mut mock_service = MockConfigService::new();
        mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        mock_service.add_file(FALLBACK_THEME_PATH, &get_valid_fallback_theme_json());

        let engine = ThemingEngine::new(Arc::new(mock_service), None).await.unwrap();
        let initial_config = engine.get_current_configuration().await;
        let initial_state = engine.get_current_theme_state().await; // Cache it

        let non_existent_theme_id = ThemeIdentifier::new("non-existent-theme");
        let new_config_bad = ThemingConfiguration {
            selected_theme_id: non_existent_theme_id.clone(),
            preferred_color_scheme: ColorSchemeType::Dark,
            ..Default::default() // Other fields don't matter as much for this test
        };

        let update_result = engine.update_configuration(new_config_bad.clone()).await;
        assert!(update_result.is_err());
        match update_result.err().unwrap() {
            ThemingError::ThemeNotFound { theme_id } => {
                assert_eq!(theme_id, non_existent_theme_id);
            }
            e => panic!("Expected ThemeNotFound, got {:?}", e),
        }

        // Check if config reverted
        let current_config_after_fail = engine.get_current_configuration().await;
        assert_eq!(current_config_after_fail, initial_config);

        // Check if state reverted (or remained the same)
        let current_state_after_fail = engine.get_current_theme_state().await;
        assert_eq!(current_state_after_fail, initial_state);
    }
    
        // TODO: test_reload_themes_and_tokens (partially done, may need more scenarios)
    // TODO: test_get_current_theme_state_caching (partially done)
    // TODO: test_list_available_themes (partially done)
    // TODO: test_get_theme_definition (partially done)

    #[tokio::test]
    async fn test_reload_themes_and_tokens_success() {
        let mut mock_service = MockConfigService::new();
        mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        mock_service.add_file(FALLBACK_THEME_PATH, &get_valid_fallback_theme_json());

        let engine = ThemingEngine::new(Arc::new(mock_service.clone()), None).await.unwrap();
        let mut rx = engine.subscribe_to_theme_changes();
        
        // Initial state based on fallback
        let initial_state = engine.get_current_theme_state().await;
        assert_eq!(initial_state.theme_id.as_str(), "fallback-dark");

        // Modify a file that reload_themes_and_tokens will pick up.
        // Let's change a token in base.tokens.json for simplicity.
        let mut new_base_tokens_json = serde_json::from_str::<serde_json::Value>(&get_valid_base_tokens_json()).unwrap();
        // Update an existing token or add a new one.
        // Let's assume "color-global-black" changes its value.
        // This requires parsing, modifying, and re-serializing JSON, or just providing new content.
        let updated_base_tokens_content = r#"[
            {"id": "color-global-black", "value": {"color": "#NEWBLACK"}}, 
            {"id": "spacing-global-medium", "value": {"spacing": "8px"}}
        ]"#;
        // Update the mock service with the new content for the global tokens file
        let mut reloaded_mock_service = mock_service.clone(); // Clone to modify for reload
        reloaded_mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, updated_base_tokens_content);
        
        // Replace the engine's config service with the one that has updated file content.
        // This is a bit of a hack for testing. In a real scenario, the ConfigService itself would see changes.
        engine.internal_state.lock().await.config_service = Arc::new(reloaded_mock_service);


        let reload_result = engine.reload_themes_and_tokens().await;
        assert!(reload_result.is_ok(), "reload_themes_and_tokens failed: {:?}", reload_result.err());
        let reloaded_state = reload_result.unwrap();

        // Check if the state reflects the reloaded tokens
        assert_eq!(reloaded_state.theme_id.as_str(), "fallback-dark"); // Theme ID should remain the same
        assert_eq!(
            reloaded_state.resolved_tokens.get(&TokenIdentifier::new("color-global-black")).unwrap(),
            "#NEWBLACK"
        );

        // Check for event
        match timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Ok(event)) => {
                assert_eq!(event.new_state, reloaded_state);
            }
            Ok(Err(e)) => panic!("Error receiving event after reload: {}", e),
            Err(_) => panic!("Timeout waiting for theme change event after reload"),
        }
    }

    #[tokio::test]
    async fn test_get_current_theme_state_caching() {
        let mut mock_service = MockConfigService::new();
        mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        mock_service.add_file(FALLBACK_THEME_PATH, &get_valid_fallback_theme_json());

        let engine = ThemingEngine::new(Arc::new(mock_service.clone()), None).await.unwrap();
        
        // First call, should resolve and cache
        let state1 = engine.get_current_theme_state().await;

        // To test caching, we'd ideally want to verify that `resolve_tokens_for_config` is NOT called again.
        // This is hard without internal counters or more complex mocking of `logic` functions.
        // A simpler way is to modify the underlying data source (mock_service.files)
        // and see if `get_current_theme_state` returns the OLD (cached) data.
        
        let mut new_mock_service_files = mock_service.files.clone();
        new_mock_service_files.insert(DEFAULT_GLOBAL_TOKENS_PATH.to_string(), r#"[{"id": "color-completely-different", "value": {"color": "#DIFFERENT"}}]"#.to_string());
        
        // IMPORTANT: The ThemingEngine holds an Arc to the MockConfigService.
        // To simulate data source changing *without* telling the engine to reload,
        // we would need the MockConfigService itself to have mutable internal state for its files,
        // perhaps behind a Mutex, so it can be changed after the Arc is cloned.
        // For this test structure, let's assume the ConfigService is immutable after engine creation for `get_current_theme_state`.
        // The test for `reload_themes_and_tokens` covers when the engine is explicitly told data has changed.
        
        // So, instead, we verify that multiple calls return the same (cloned) state object quickly.
        // This doesn't prove `resolve_tokens_for_config` wasn't called, but it's a basic check.
        let state2 = engine.get_current_theme_state().await;
        assert_eq!(state1, state2, "State should be the same (from cache)");

        // Now, let's change the configuration, which should result in a new state.
        let mut current_config = engine.get_current_configuration().await;
        current_config.preferred_color_scheme = ColorSchemeType::Light; // Assuming fallback has dark and light variants or this causes re-resolution.
                                                                        // The provided fallback.theme.json does not have a light variant.
                                                                        // Let's add one to the mock file for this test.
        let fallback_with_light_variant_json = r#"{
            "id": "fallback-dark", "name": "Test Fallback Dark",
            "base_tokens": {
                "color-background": {"id": "color-background", "value": {"color": "#1E1E1E"}},
                "color-foreground": {"id": "color-foreground", "value": {"color": "#D4D4D4"}}
            },
            "variants": [
                {"applies_to_scheme": "dark", "tokens": {}}, 
                {"applies_to_scheme": "light", "tokens": {"color-background": {"id":"color-background", "value": {"color": "#FEFEFE"}}}}
            ]
        }"#;
        let mut new_mock_service = MockConfigService::new();
        new_mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        new_mock_service.add_file(FALLBACK_THEME_PATH, &fallback_with_light_variant_json);
        
        let engine_for_variant_test = ThemingEngine::new(Arc::new(new_mock_service), Some(current_config.clone())).await.unwrap();

        let state_light_config = engine_for_variant_test.get_current_theme_state().await;
        assert_eq!(state_light_config.color_scheme, ColorSchemeType::Light);
        assert_eq!(state_light_config.resolved_tokens.get(&TokenIdentifier::new("color-background")).unwrap(), "#FEFEFE");
        
        // Call again, should be cached for the light config
        let state_light_config_cached = engine_for_variant_test.get_current_theme_state().await;
        assert_eq!(state_light_config_cached, state_light_config);
    }
    
    #[tokio::test]
    async fn test_list_and_get_themes() {
        let mut mock_service = MockConfigService::new();
        mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        mock_service.add_file(FALLBACK_THEME_PATH, &get_valid_fallback_theme_json());
        // `load_resources` currently only loads FALLBACK_THEME_PATH by its specific name.
        // To test listing multiple themes, `load_resources` would need to scan a directory
        // or be given multiple theme paths.
        // For now, `list_available_themes` will only list the fallback theme.
        
        let engine = ThemingEngine::new(Arc::new(mock_service), None).await.unwrap();

        let available_themes = engine.list_available_themes().await;
        assert_eq!(available_themes.len(), 1);
        assert_eq!(available_themes[0].id.as_str(), "fallback-dark");

        let definition = engine.get_theme_definition(&ThemeIdentifier::new("fallback-dark")).await;
        assert!(definition.is_some());
        assert_eq!(definition.unwrap().id.as_str(), "fallback-dark");

        let non_existent_def = engine.get_theme_definition(&ThemeIdentifier::new("no-such-theme")).await;
        assert!(non_existent_def.is_none());
    }

    #[tokio::test]
    async fn test_subscribe_and_receive_event_on_update() {
        let mut mock_service = MockConfigService::new();
        mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        mock_service.add_file(FALLBACK_THEME_PATH, &get_valid_fallback_theme_json());
        
        let light_theme_id_str = "light-event-theme";
        let light_theme_path = "src/theming/default_themes/light_event.theme.json";
        let light_theme_json = format!(r#"{{
            "id": "{}", "name": "Light Event Theme",
            "base_tokens": {{ "evt-color": {{"id":"evt-color", "value":{{"color":"#EVENT00"}}}} }},
            "variants": [ {{"applies_to_scheme": "light", "tokens": {{}} }} ]
        }}"#, light_theme_id_str);
        mock_service.add_file(light_theme_path, &light_theme_json);

        let engine = ThemingEngine::new(Arc::new(mock_service.clone()), None).await.unwrap();
        
        // Manually insert the light theme for the test
        let light_theme_def: ThemeDefinition = serde_json::from_str(&light_theme_json).unwrap();
        engine.internal_state.lock().await.available_themes.insert(ThemeIdentifier::new(light_theme_id_str), light_theme_def.clone());

        let mut rx1 = engine.subscribe_to_theme_changes();
        let mut rx2 = engine.subscribe_to_theme_changes();

        let new_config = ThemingConfiguration {
            selected_theme_id: ThemeIdentifier::new(light_theme_id_str),
            preferred_color_scheme: ColorSchemeType::Light,
            ..Default::default()
        };

        let update_result = engine.update_configuration(new_config.clone()).await;
        assert!(update_result.is_ok());
        let applied_state = update_result.unwrap();

        let event1 = timeout(Duration::from_millis(100), rx1.recv()).await;
        assert!(event1.is_ok(), "rx1 timeout or channel closed");
        assert_eq!(event1.unwrap().unwrap().new_state, applied_state, "rx1 received incorrect event");

        let event2 = timeout(Duration::from_millis(100), rx2.recv()).await;
        assert!(event2.is_ok(), "rx2 timeout or channel closed");
        assert_eq!(event2.unwrap().unwrap().new_state, applied_state, "rx2 received incorrect event");
    }
    
    #[tokio::test]
    async fn test_no_event_if_update_fails() {
        let mut mock_service = MockConfigService::new();
        mock_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &get_valid_base_tokens_json());
        mock_service.add_file(FALLBACK_THEME_PATH, &get_valid_fallback_theme_json());

        let engine = ThemingEngine::new(Arc::new(mock_service), None).await.unwrap();
        let mut rx = engine.subscribe_to_theme_changes();

        let new_config_bad = ThemingConfiguration {
            selected_theme_id: ThemeIdentifier::new("theme-does-not-exist"),
            ..Default::default()
        };

        let _ = engine.update_configuration(new_config_bad).await; // Expected to fail

        // Try to receive an event, should timeout if none sent
        match timeout(Duration::from_millis(50), rx.recv()).await {
            Ok(Ok(event)) => panic!("Received unexpected event: {:?}", event),
            Ok(Err(_)) => {}, // RecvError, means sender dropped or channel lagged (not expected here)
            Err(_) => {}, // Timeout, this is the expected outcome (no event)
        }
    }

    // --- Test for DefaultFileSystemConfigService ---
    use crate::theming::default_config_service::DefaultFileSystemConfigService;
    use std::fs as std_fs;
    use std::io::Write;

    // Helper to create dummy files for DefaultFileSystemConfigService tests
    fn setup_temp_theme_files(base_content: &str, fallback_content: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let theming_dir = temp_dir.path().join("src/theming/default_themes");
        std_fs::create_dir_all(&theming_dir).expect("Failed to create temp theming dir");

        let base_path = theming_dir.join("base.tokens.json");
        let fallback_path = theming_dir.join("fallback.theme.json");

        let mut base_file = std_fs::File::create(&base_path).expect("Failed to create temp base tokens file");
        base_file.write_all(base_content.as_bytes()).expect("Failed to write temp base tokens");

        let mut fallback_file = std_fs::File::create(&fallback_path).expect("Failed to create temp fallback theme file");
        fallback_file.write_all(fallback_content.as_bytes()).expect("Failed to write temp fallback theme");
        
        // The DEFAULT_GLOBAL_TOKENS_PATH and FALLBACK_THEME_PATH are relative like "src/theming/..."
        // We need to run the test as if the temp_dir is the crate root.
        // This is hard to achieve directly.
        // Alternative: Modify ThemingEngine to accept base_path for testing, or have
        // DefaultFileSystemConfigService take a root path.
        // For now, this helper creates the files, but the test will use fixed paths
        // assuming they are present relative to where `cargo test` is run (crate root).
        // This helper is more for direct testing of DefaultFileSystemConfigService if needed.
        // The ThemingEngine test will rely on the actual files being present.

        (temp_dir, base_path, fallback_path)
    }


    #[tokio::test]
    async fn test_theming_engine_new_default_loading_with_filesystem_service() {
        // This test assumes that `src/theming/default_themes/base.tokens.json`
        // and `src/theming/default_themes/fallback.theme.json` exist relative to the
        // crate root where `cargo test` is executed.

        // 1. Create dummy content for these files if they don't exist or to control test input.
        //    This is tricky because tests run from crate root, so paths are relative to it.
        //    Let's ensure the files exist for the test.
        let base_tokens_content = get_valid_base_tokens_json();
        let fallback_theme_content = get_valid_fallback_theme_json();

        // Create files in expected locations if they don't exist (for local/CI test runs)
        // Note: This approach of writing to src might be problematic and is generally
        // not good practice for tests. Tests should ideally use temp files or mocks.
        // However, DefaultFileSystemConfigService reads from fixed paths.
        // A better DefaultFileSystemConfigService would take a root path in constructor.
        
        // For this test, we'll just proceed assuming the files are there as per project structure.
        // If they are not, this test will fail, which is an indicator of missing default assets.

        let fs_config_service = Arc::new(DefaultFileSystemConfigService::new());

        let engine_result = ThemingEngine::new(fs_config_service, None).await;
        
        if let Err(ref e) = engine_result {
            // Provide more diagnostic information if it fails
            if let ThemingError::InternalError(msg) = e {
                if msg.contains("Konfigurationsdatei nicht gefunden") {
                    panic!("File not found during ThemingEngine::new: {}. Ensure default theme/token files exist at '{}' and '{}'. Original error: {:?}", 
                           msg, DEFAULT_GLOBAL_TOKENS_PATH, FALLBACK_THEME_PATH, e);
                }
            }
            panic!("ThemingEngine::new failed with DefaultFileSystemConfigService: {:?}", e);
        }
        assert!(engine_result.is_ok(), "ThemingEngine::new with DefaultFileSystemConfigService failed.");
        
        let engine = engine_result.unwrap();

        let current_config = engine.get_current_configuration().await;
        // Default theme ID comes from FALLBACK_THEME_PATH which is "fallback-dark" in get_valid_fallback_theme_json
        assert_eq!(current_config.selected_theme_id.as_str(), "fallback-dark"); 

        let current_state = engine.get_current_theme_state().await;
        assert_eq!(current_state.theme_id.as_str(), "fallback-dark");
        assert_eq!(current_state.color_scheme, ColorSchemeType::Dark); // Default from logic
        
        // Check for tokens from fallback.theme.json
        assert!(current_state.resolved_tokens.contains_key(&TokenIdentifier::new("color-background")), "Missing color-background from fallback theme");
        assert_eq!(current_state.resolved_tokens.get(&TokenIdentifier::new("color-background")).unwrap(), "#1E1E1E");
        
        // Check for tokens from base.tokens.json
        assert!(current_state.resolved_tokens.contains_key(&TokenIdentifier::new("color-global-black")), "Missing color-global-black from base tokens");
        assert_eq!(current_state.resolved_tokens.get(&TokenIdentifier::new("color-global-black")).unwrap(), "#000000");
        assert!(current_state.resolved_tokens.contains_key(&TokenIdentifier::new("spacing-global-medium")), "Missing spacing-global-medium from base tokens");
        assert_eq!(current_state.resolved_tokens.get(&TokenIdentifier::new("spacing-global-medium")).unwrap(), "8px");
    }

    #[tokio::test]
    async fn test_theming_engine_discovers_multiple_themes_with_filesystem_service() {
        // This test relies on:
        // 1. `src/theming/default_themes/base.tokens.json` (for global tokens)
        // 2. `src/theming/default_themes/fallback.theme.json`
        // 3. `src/theming/default_themes/another.theme.json` (created in a previous step)

        // Note: This test uses DefaultFileSystemConfigService which reads from the actual filesystem.
        // Ensure the above files exist in their correct locations relative to the crate root
        // when `cargo test` is executed. The `another.theme.json` should have been created by
        // a prior step in the agent's execution plan.

        let fs_config_service = Arc::new(DefaultFileSystemConfigService::new());
        let engine_result = ThemingEngine::new(fs_config_service, None).await;

        if let Err(ref e) = engine_result {
            // Provide more diagnostic information if it fails, especially for file not found.
            if let ThemingError::InternalError(msg) = e {
                if msg.contains("Konfigurationsdatei nicht gefunden") || msg.contains("Failed to read file") {
                    panic!(
                        "File not found during ThemingEngine::new in multi-theme discovery test: {}. \
                        Ensure default theme/token files (base.tokens.json, fallback.theme.json, another.theme.json) \
                        exist at their expected paths relative to the crate root (e.g., 'src/theming/default_themes/'). Original error: {:?}", 
                        msg, e
                    );
                }
            }
            panic!("ThemingEngine::new with DefaultFileSystemConfigService failed during multi-theme discovery test: {:?}", e);
        }
        assert!(engine_result.is_ok(), "ThemingEngine::new with DefaultFileSystemConfigService failed.");
        
        let engine = engine_result.unwrap();
        let available_themes = engine.list_available_themes().await;

        // Check for at least "fallback-dark" and "another-test-theme".
        // There might be other .theme.json files if more were added to the default_themes directory.
        let found_fallback = available_themes.iter().any(|t| t.id.as_str() == "fallback-dark");
        let found_another = available_themes.iter().any(|t| t.id.as_str() == "another-test-theme");

        assert!(found_fallback, 
            "Fallback theme 'fallback-dark' not found. Available themes: {:?}", 
            available_themes.iter().map(|t| t.id.clone()).collect::<Vec<_>>()
        );
        assert!(found_another, 
            "Discovered theme 'another-test-theme' not found. Available themes: {:?}", 
            available_themes.iter().map(|t| t.id.clone()).collect::<Vec<_>>()
        );
        
        // Optionally, check properties of 'another-test-theme' to ensure it was loaded correctly
        if let Some(another_theme) = engine.get_theme_definition(&ThemeIdentifier::new("another-test-theme")).await {
            assert_eq!(another_theme.name, "Another Test Theme");
            assert_eq!(another_theme.variants.len(), 1, "Expected 1 variant for another-test-theme");
            if !another_theme.variants.is_empty() {
                assert_eq!(another_theme.variants[0].applies_to_scheme, ColorSchemeType::Light);
            }
        } else {
            panic!("'another-test-theme' was listed by id but could not be retrieved by get_theme_definition.");
        }

        // --- Assertions for "default-dark-from-rust" ---
        let found_converted_dark_theme = available_themes.iter().any(|t| t.id.as_str() == "default-dark-from-rust");
        assert!(found_converted_dark_theme,
            "Converted theme 'default-dark-from-rust' not found. Available themes: {:?}",
            available_themes.iter().map(|t| t.id.clone()).collect::<Vec<_>>()
        );

        if let Some(converted_theme) = engine.get_theme_definition(&ThemeIdentifier::new("default-dark-from-rust")).await {
            assert_eq!(converted_theme.name, "Default Dark (Converted)");
            assert_eq!(converted_theme.author.as_deref(), Some("NovaDE Team"));
            assert!(converted_theme.base_tokens.contains_key(&TokenIdentifier::new("color-background")));
            assert!(converted_theme.base_tokens.contains_key(&TokenIdentifier::new("property-font-family")));

            // Check a specific color token value
            let bg_token = converted_theme.base_tokens.get(&TokenIdentifier::new("color-background")).unwrap();
            match &bg_token.value {
                TokenValue::Color(color_val) => assert_eq!(color_val, "#202020FF"),
                other => panic!("Expected color-background to be a Color value, got {:?}", other),
            }

            // Check a specific property token value
            let font_token = converted_theme.base_tokens.get(&TokenIdentifier::new("property-font-family")).unwrap();
            match &font_token.value {
                TokenValue::FontFamily(font_val) => assert_eq!(font_val, "Segoe UI, sans-serif"),
                other => panic!("Expected property-font-family to be a FontFamily value, got {:?}", other),
            }
            
            assert_eq!(converted_theme.variants.len(), 1);
            assert_eq!(converted_theme.variants[0].applies_to_scheme, ColorSchemeType::Dark);
            assert!(converted_theme.variants[0].tokens.is_empty(), "Expected converted dark theme's dark variant to have empty tokens (all in base)");

        } else {
            panic!("'default-dark-from-rust' was listed by id but could not be retrieved by get_theme_definition.");
        }

        // --- Assertions for "default-light-from-rust" ---
        let found_converted_light_theme = available_themes.iter().any(|t| t.id.as_str() == "default-light-from-rust");
        assert!(found_converted_light_theme,
            "Converted theme 'default-light-from-rust' not found. Available themes: {:?}",
            available_themes.iter().map(|t| t.id.clone()).collect::<Vec<_>>()
        );
        if let Some(converted_theme) = engine.get_theme_definition(&ThemeIdentifier::new("default-light-from-rust")).await {
            assert_eq!(converted_theme.name, "Default Light (Converted)");
            assert!(converted_theme.base_tokens.contains_key(&TokenIdentifier::new("color-background")));
            let bg_token = converted_theme.base_tokens.get(&TokenIdentifier::new("color-background")).unwrap();
            match &bg_token.value {
                TokenValue::Color(color_val) => assert_eq!(color_val, "#F8F8F8FF"),
                other => panic!("Expected light theme color-background to be a Color value, got {:?}", other),
            }
            assert_eq!(converted_theme.variants[0].applies_to_scheme, ColorSchemeType::Light);
        } else {
            panic!("'default-light-from-rust' was listed but could not be retrieved by get_theme_definition.");
        }

        // --- Assertions for "high-contrast-from-rust" ---
        let found_high_contrast_theme = available_themes.iter().any(|t| t.id.as_str() == "high-contrast-from-rust");
        assert!(found_high_contrast_theme,
            "Converted theme 'high-contrast-from-rust' not found. Available themes: {:?}",
            available_themes.iter().map(|t| t.id.clone()).collect::<Vec<_>>()
        );
        if let Some(converted_theme) = engine.get_theme_definition(&ThemeIdentifier::new("high-contrast-from-rust")).await {
            assert_eq!(converted_theme.name, "High Contrast (Converted)");
            assert!(converted_theme.base_tokens.contains_key(&TokenIdentifier::new("color-background")));
            let bg_token = converted_theme.base_tokens.get(&TokenIdentifier::new("color-background")).unwrap();
            match &bg_token.value {
                TokenValue::Color(color_val) => assert_eq!(color_val, "#000000FF"),
                other => panic!("Expected high-contrast color-background to be a Color value, got {:?}", other),
            }
            let font_size_token = converted_theme.base_tokens.get(&TokenIdentifier::new("property-font-size")).unwrap();
             match &font_size_token.value {
                TokenValue::FontSize(val) => assert_eq!(val, "16px"),
                other => panic!("Expected high-contrast property-font-size to be a FontSize value, got {:?}", other),
            }
            assert_eq!(converted_theme.variants[0].applies_to_scheme, ColorSchemeType::Dark); // As per our conversion
        } else {
            panic!("'high-contrast-from-rust' was listed but could not be retrieved by get_theme_definition.");
        }
    }

    #[tokio::test]
    async fn test_theming_engine_switch_themes_and_apply_config() {
        // This test uses DefaultFileSystemConfigService and assumes the presence of:
        // - src/theming/default_themes/base.tokens.json
        // - src/theming/default_themes/fallback.theme.json
        // - src/theming/default_themes/another.theme.json
        // - src/theming/default_themes/default_dark_converted.theme.json
        // - src/theming/default_themes/default_light_converted.theme.json
        // - src/theming/default_themes/high_contrast_converted.theme.json

        let fs_config_service = Arc::new(DefaultFileSystemConfigService::new());
        let engine_result = ThemingEngine::new(fs_config_service, None).await;
        assert!(engine_result.is_ok(), "Engine ::new failed: {:?}", engine_result.err());
        let engine = engine_result.unwrap();

        // --- 1. Initial State Verification (should be fallback-dark) ---
        let initial_state = engine.get_current_theme_state().await;
        assert_eq!(initial_state.theme_id.as_str(), "fallback-dark");
        assert_eq!(initial_state.color_scheme, ColorSchemeType::Dark); // Default for fallback-dark
        assert_eq!(initial_state.resolved_tokens.get(&TokenIdentifier::new("color-background")).unwrap(), "#1E1E1EFF");
        assert_eq!(initial_state.resolved_tokens.get(&TokenIdentifier::new("color-panel-background")).unwrap(), "#252526FF");
        let initial_primary_color = initial_state.resolved_tokens.get(&TokenIdentifier::new("color-primary")).unwrap().clone();


        // --- 2. Switch Theme to "another-test-theme" (Light scheme by its definition) ---
        let another_theme_id = ThemeIdentifier::new("another-test-theme");
        let config_another_theme = ThemingConfiguration {
            selected_theme_id: another_theme_id.clone(),
            preferred_color_scheme: ColorSchemeType::Light, // This theme's variant is light
            selected_accent_color: None,
            custom_user_token_overrides: None,
        };
        let update_result_another = engine.update_configuration(config_another_theme).await;
        assert!(update_result_another.is_ok(), "Update to 'another-test-theme' failed: {:?}", update_result_another.err());
        
        let state_another_theme = engine.get_current_theme_state().await;
        assert_eq!(state_another_theme.theme_id, another_theme_id);
        assert_eq!(state_another_theme.color_scheme, ColorSchemeType::Light);
        // "another-test-theme" has "color-text": "#111111" in its light variant
        assert_eq!(state_another_theme.resolved_tokens.get(&TokenIdentifier::new("color-text")).unwrap(), "#111111");


        // --- 3. Switch back to "fallback-dark" and change Scheme to Light ---
        let fallback_theme_id = ThemeIdentifier::new("fallback-dark");
        let config_fallback_light = ThemingConfiguration {
            selected_theme_id: fallback_theme_id.clone(),
            preferred_color_scheme: ColorSchemeType::Light, // Explicitly switch to light variant
            selected_accent_color: None,
            custom_user_token_overrides: None,
        };
        let update_result_fallback_light = engine.update_configuration(config_fallback_light).await;
        assert!(update_result_fallback_light.is_ok(), "Update to 'fallback-dark' (Light) failed: {:?}", update_result_fallback_light.err());

        let state_fallback_light = engine.get_current_theme_state().await;
        assert_eq!(state_fallback_light.theme_id, fallback_theme_id);
        assert_eq!(state_fallback_light.color_scheme, ColorSchemeType::Light);
        // Values from fallback.theme.json's light variant
        assert_eq!(state_fallback_light.resolved_tokens.get(&TokenIdentifier::new("color-background")).unwrap(), "#FFFFFFFF");
        assert_eq!(state_fallback_light.resolved_tokens.get(&TokenIdentifier::new("color-foreground")).unwrap(), "#000000FF");
        assert_eq!(state_fallback_light.resolved_tokens.get(&TokenIdentifier::new("color-panel-background")).unwrap(), "#F3F3F3FF");
        // Primary color should still be from base if not overridden in variant
        assert_eq!(state_fallback_light.resolved_tokens.get(&TokenIdentifier::new("color-primary")).unwrap(), &initial_primary_color);


        // --- 4. Apply Accent Color to "fallback-dark" (Light scheme) ---
        let accent_crimson_hex = "#DC143CFF"; // Crimson Red from fallback.theme.json
        let accent_crimson = CoreColor::from_hex(accent_crimson_hex).unwrap();
        let config_fallback_accent = ThemingConfiguration {
            selected_theme_id: fallback_theme_id.clone(),
            preferred_color_scheme: ColorSchemeType::Light, // Keep light scheme
            selected_accent_color: Some(accent_crimson.clone()),
            custom_user_token_overrides: None,
        };
        let update_result_fallback_accent = engine.update_configuration(config_fallback_accent).await;
        assert!(update_result_fallback_accent.is_ok(), "Update to 'fallback-dark' (Light with Accent) failed: {:?}", update_result_fallback_accent.err());

        let state_fallback_accent = engine.get_current_theme_state().await;
        assert_eq!(state_fallback_accent.theme_id, fallback_theme_id);
        assert_eq!(state_fallback_accent.color_scheme, ColorSchemeType::Light);
        assert_eq!(state_fallback_accent.active_accent_color, Some(accent_crimson));
        
        // "color-primary" is accentable with "direct-replace" in fallback.theme.json
        assert_eq!(
            state_fallback_accent.resolved_tokens.get(&TokenIdentifier::new("color-primary")).unwrap(),
            &accent_crimson.to_hex_string() // Expecting exact match due to direct-replace
        );
        // Other colors should remain from the light variant
        assert_eq!(state_fallback_accent.resolved_tokens.get(&TokenIdentifier::new("color-background")).unwrap(), "#FFFFFFFF");


        // --- 5. Switch to "default-dark-from-rust" and apply a different accent ---
        let default_dark_conv_id = ThemeIdentifier::new("default-dark-from-rust");
        let accent_blue_hex = "#007ACCFF"; // Default Blue from fallback.theme.json (can be any valid color)
        let accent_blue = CoreColor::from_hex(accent_blue_hex).unwrap();

        // "default-dark-from-rust" does not have accentable_tokens defined in its JSON.
        // So, applying an accent color should not change any tokens unless we modify its definition.
        // For this test, we'll assume it has no accentable tokens, so color-primary should not change.
        
        // Let's quickly check if "default-dark-from-rust" has "color-primary-default"
        let def_dark_theme_def = engine.get_theme_definition(&default_dark_conv_id).await;
        assert!(def_dark_theme_def.is_some(), "'default-dark-from-rust' definition not found");
        let def_dark_primary_original_value = def_dark_theme_def.unwrap().base_tokens
            .get(&TokenIdentifier::new("color-primary-default"))
            .map(|rt| rt.value.to_string()) // This to_string() might not be ideal for TokenValue comparison
            .unwrap_or_else(|| panic!("color-primary-default not in default-dark-from-rust base_tokens"));
            
        // To properly get the string value from TokenValue::Color for comparison:
        let def_dark_primary_original_hex = if let Some(raw_token) = engine.get_theme_definition(&default_dark_conv_id).await.unwrap().base_tokens.get(&TokenIdentifier::new("color-primary-default")) {
            if let TokenValue::Color(hex) = &raw_token.value {
                hex.clone()
            } else {
                panic!("color-primary-default is not a Color TokenValue in default-dark-from-rust");
            }
        } else {
            panic!("color-primary-default not found in default-dark-from-rust base_tokens for value check");
        };


        let config_dd_accent = ThemingConfiguration {
            selected_theme_id: default_dark_conv_id.clone(),
            preferred_color_scheme: ColorSchemeType::Dark, // Its default scheme
            selected_accent_color: Some(accent_blue.clone()),
            custom_user_token_overrides: None,
        };
        let update_result_dd_accent = engine.update_configuration(config_dd_accent).await;
        assert!(update_result_dd_accent.is_ok(), "Update to 'default-dark-from-rust' (with Accent) failed: {:?}", update_result_dd_accent.err());
        
        let state_dd_accent = engine.get_current_theme_state().await;
        assert_eq!(state_dd_accent.theme_id, default_dark_conv_id);
        assert_eq!(state_dd_accent.active_accent_color, Some(accent_blue));
        // Since "default-dark-from-rust" has no "accentable_tokens" defined, "color-primary-default" should NOT change.
        assert_eq!(
            state_dd_accent.resolved_tokens.get(&TokenIdentifier::new("color-primary-default")).unwrap(),
            &def_dark_primary_original_hex
        );


        // --- 6. Clear Accent Color on "fallback-dark" ---
        let config_fallback_no_accent = ThemingConfiguration {
            selected_theme_id: fallback_theme_id.clone(),
            preferred_color_scheme: ColorSchemeType::Light, // Keep light scheme
            selected_accent_color: None, // Clear accent
            custom_user_token_overrides: None,
        };
        let update_result_fallback_no_accent = engine.update_configuration(config_fallback_no_accent).await;
        assert!(update_result_fallback_no_accent.is_ok(), "Update to 'fallback-dark' (Light, No Accent) failed: {:?}", update_result_fallback_no_accent.err());

        let state_fallback_no_accent = engine.get_current_theme_state().await;
        assert_eq!(state_fallback_no_accent.theme_id, fallback_theme_id);
        assert_eq!(state_fallback_no_accent.color_scheme, ColorSchemeType::Light);
        assert!(state_fallback_no_accent.active_accent_color.is_none());
        // "color-primary" should revert to its non-accented value for the light scheme.
        // The light scheme variant for fallback-dark does NOT override color-primary, so it comes from base.
        assert_eq!(state_fallback_no_accent.resolved_tokens.get(&TokenIdentifier::new("color-primary")).unwrap(), &initial_primary_color);
    }
}
