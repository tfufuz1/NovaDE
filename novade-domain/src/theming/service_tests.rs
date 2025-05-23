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
        async fn load_config_file_content_async(&self, file_path: &str) -> Result<String, CoreError> {
            if let Some(err) = &self.error_on_load {
                return Err(err.clone());
            }
            self.files
                .get(file_path)
                .cloned()
                .ok_or_else(|| CoreError::NotFound(file_path.to_string()))
        }

        async fn save_config_file_content_async(&self, _file_path: &str, _content: &str) -> Result<(), CoreError> {
            if let Some(err) = &self.error_on_save {
                return Err(err.clone());
            }
            Ok(())
        }
        async fn list_config_files_async(&self, _dir_path: &str) -> Result<Vec<String>, CoreError> {
            if let Some(err) = &self.error_on_list {
                return Err(err.clone());
            }
            Ok(self.files.keys().cloned().collect())
        }
        fn get_config_file_path(&self, _app_id: &crate::shared_types::ApplicationId, _config_name: &str, _format: Option<ConfigFormat>) -> Result<String, CoreError> {
            unimplemented!("get_config_file_path not needed for these tests")
        }
        fn get_config_dir_path(&self, _app_id: &crate::shared_types::ApplicationId, _subdir: Option<&str>) -> Result<String, CoreError> {
            unimplemented!("get_config_dir_path not needed for these tests")
        }
        fn ensure_config_dir_exists(&self, _app_id: &crate::shared_types::ApplicationId) -> Result<String, CoreError> {
            unimplemented!("ensure_config_dir_exists not needed for these tests")
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
    
    // TODO: test_reload_themes_and_tokens
    // TODO: test_get_current_theme_state_caching
    // TODO: test_list_available_themes
    // TODO: test_get_theme_definition

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
}
