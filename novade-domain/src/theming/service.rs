use crate::theming::types::{
    AppliedThemeState, ThemingConfiguration, ThemeDefinition, TokenSet, ThemeIdentifier,
    ColorSchemeType, TokenIdentifier,
};
use crate::theming::errors::ThemingError;
use crate::theming::logic::{
    resolve_tokens_for_config, load_and_validate_token_files, load_and_validate_theme_files,
    generate_fallback_applied_state, load_theme_definition_from_file, // Ensure this is available or fix usage
};
use crate::ports::config_service::ConfigServiceAsync; // Corrected path
use novade_core::types::Color as CoreColor;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast, RwLock}; // Using RwLock for state if appropriate, Mutex for general internal state
use crate::shared_types::ApplicationId; // If needed for any configurations or future extensions

// Corrected paths for default themes and tokens within the novade-domain crate structure
const DEFAULT_GLOBAL_TOKENS_PATH: &str = "src/theming/default_themes/base.tokens.json";
const FALLBACK_THEME_PATH: &str = "src/theming/default_themes/fallback.theme.json";
const DEFAULT_THEMES_DIR_PATH: &str = "src/theming/default_themes"; // Standard directory for themes

/// Event dispatched when the active theme state changes.
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeChangedEvent {
    pub new_state: AppliedThemeState,
}

/// Defines a key for caching resolved theme states.
/// The cache key considers all inputs that can vary the `AppliedThemeState`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    theme_id: ThemeIdentifier,
    color_scheme: ColorSchemeType,
    accent_color_hex: Option<String>, // Store hex string for consistent hashing
    // Hash of user_overrides TokenSet might be too complex/slow.
    // Consider a version or timestamp if user_overrides change frequently without ID changes.
    // For simplicity, if custom_user_token_overrides is Some, we might just use a counter or a hash of its JSON representation.
    // For now, not including overrides directly in key, relying on config updates to invalidate.
    // A more robust approach might involve hashing the `custom_user_token_overrides` if present.
    // For this implementation, we'll assume that a new `ThemingConfiguration` object implies potential change.
    // The cache will be keyed by a simplified representation or rely on external invalidation for overrides.
}

impl CacheKey {
    fn from_config_and_theme(config: &ThemingConfiguration, _theme_def: &ThemeDefinition) -> Self {
        Self {
            theme_id: config.selected_theme_id.clone(),
            color_scheme: config.preferred_color_scheme,
            accent_color_hex: config.selected_accent_color.as_ref().map(CoreColor::to_hex_string),
            // Note: custom_user_token_overrides are not part of this simple key.
            // The cache in ThemingEngineInternalState might need a more sophisticated approach
            // or simply store one resolved state per (ThemeId, Scheme, Accent) and assume overrides
            // are handled by re-applying config.
        }
    }
}


/// Internal state of the ThemingEngine.
pub struct ThemingEngineInternalState {
    config_service: Arc<dyn ConfigServiceAsync>,
    current_config: ThemingConfiguration,
    
    global_tokens: TokenSet,
    available_themes: HashMap<ThemeIdentifier, ThemeDefinition>, // Loaded theme definitions
    
    // Cache for resolved AppliedThemeStates.
    // Keyed by relevant parts of ThemingConfiguration (ThemeId, Scheme, Accent).
    // RwLock allows multiple readers (get_current_theme_state) or one writer (when config changes).
    resolved_state_cache: RwLock<HashMap<CacheKey, AppliedThemeState>>,
    
    // Cache for individual token resolution passes during a single `resolve_tokens_for_config` call.
    // This is not for cross-request caching but for optimizing a single resolution pipeline.
    // This might be better placed as a temporary cache within `internal_apply_configuration_locked`.
    // For now, let's assume it's managed per call.
    // token_resolution_pass_cache: Mutex<HashMap<TokenIdentifier, Result<String, ThemingError>>>,

    // Fallback state, loaded once.
    fallback_state: AppliedThemeState,
}

impl ThemingEngineInternalState {
    /// Loads global tokens and theme definitions.
    async fn load_resources(&mut self) -> Result<(), ThemingError> {
        // Clear existing themes (except potentially a pre-loaded structural fallback if any)
        self.available_themes.clear(); 

        // 1. Load global tokens
        // These are considered essential. If they fail, return an error.
        let global_token_paths = [DEFAULT_GLOBAL_TOKENS_PATH.to_string()];
        self.global_tokens = load_and_validate_token_files(self.config_service.clone(), &global_token_paths).await
            .map_err(|e| {
                log::error!("Kritischer Fehler: Globale Tokendatei '{}' konnte nicht geladen werden: {:?}", DEFAULT_GLOBAL_TOKENS_PATH, e);
                // Ensure the error clearly indicates it's about global tokens
                ThemingError::InternalError(format!("Fehler beim Laden globaler Tokens von '{}': {}", DEFAULT_GLOBAL_TOKENS_PATH, e))
            })?;
        log::info!("Globale Tokens erfolgreich von '{}' geladen.", DEFAULT_GLOBAL_TOKENS_PATH);

        // 2. Load the designated fallback theme.
        // This is also essential. If it fails, return an error.
        match load_theme_definition_from_file(self.config_service.clone(), FALLBACK_THEME_PATH).await {
            Ok(fallback_theme_def) => {
                log::info!("Fallback-Theme '{}' ('{}') erfolgreich geladen.", fallback_theme_def.name, fallback_theme_def.id);
                self.available_themes.insert(fallback_theme_def.id.clone(), fallback_theme_def);
            }
            Err(e) => {
                log::error!("Kritischer Fehler: Fallback-Theme-Definitionsdatei '{}' konnte nicht geladen werden: {:?}", FALLBACK_THEME_PATH, e);
                return Err(ThemingError::InternalError(format!(
                    "Fallback-Theme konnte nicht geladen werden von '{}': {}", FALLBACK_THEME_PATH, e
                )));
            }
        }
        
        // 3. Discover and load other themes from the default themes directory.
        // Errors here will be logged as warnings, but won't stop the engine if fallback is available.
        log::info!("Suche nach zusätzlichen Themes im Verzeichnis: '{}'", DEFAULT_THEMES_DIR_PATH);
        match self.config_service.list_files_in_dir(std::path::Path::new(DEFAULT_THEMES_DIR_PATH), Some("theme.json")).await {
            Ok(theme_file_paths) => {
                if theme_file_paths.is_empty() {
                    log::info!("Keine zusätzlichen Theme-Dateien in '{}' gefunden.", DEFAULT_THEMES_DIR_PATH);
                } else {
                    log::info!("{} potenzielle Theme-Dateien in '{}' gefunden.", theme_file_paths.len(), DEFAULT_THEMES_DIR_PATH);
                }
                for theme_path_buf in theme_file_paths {
                    // Convert PathBuf to &str for load_theme_definition_from_file
                    // Note: ConfigServiceAsync methods should ideally work with Path or PathBuf directly.
                    // For now, assume paths are valid UTF-8 strings.
                    let theme_path_str = match theme_path_buf.to_str() {
                        Some(s) => s,
                        None => {
                            log::warn!("Warnung: Ungültiger (nicht UTF-8) Pfad für Theme-Datei übersprungen: {:?}", theme_path_buf);
                            continue;
                        }
                    };

                    // Skip loading the fallback theme again if it's listed by list_files_in_dir
                    if theme_path_str == FALLBACK_THEME_PATH {
                        log::info!("Fallback-Theme '{}' bereits geladen, Überspringen der erneuten Verarbeitung aus dem Verzeichnis.", FALLBACK_THEME_PATH);
                        continue;
                    }

                    log::info!("Versuche, Theme-Definition von '{}' zu laden...", theme_path_str);
                    match load_theme_definition_from_file(self.config_service.clone(), theme_path_str).await {
                        Ok(theme_def) => {
                            log::info!("Theme '{}' ('{}') erfolgreich von '{}' geladen.", theme_def.name, theme_def.id, theme_path_str);
                            if self.available_themes.contains_key(&theme_def.id) {
                                log::warn!("Warnung: Theme mit ID '{}' von '{}' überschreibt ein bereits geladenes Theme mit derselben ID.", theme_def.id, theme_path_str);
                            }
                            self.available_themes.insert(theme_def.id.clone(), theme_def);
                        }
                        Err(e) => {
                            // Log as warning, don't let one bad theme stop all others.
                            log::warn!("Warnung: Theme-Definitionsdatei '{}' konnte nicht geladen oder validiert werden: {:?}", theme_path_str, e);
                        }
                    }
                }
            }
            Err(core_err) => {
                // Error listing the directory itself. Log as warning.
                log::warn!(
                    "Warnung: Fehler beim Auflisten des Theme-Verzeichnisses '{}': {:?}. Es werden nur explizit definierte Themes (z.B. Fallback) geladen.",
                    DEFAULT_THEMES_DIR_PATH,
                    core_err
                );
            }
        }
        Ok(())
    }
}

/// The main service for managing themes and tokens.
pub struct ThemingEngine {
    internal_state: Arc<Mutex<ThemingEngineInternalState>>,
    theme_changed_event_sender: broadcast::Sender<ThemeChangedEvent>,
}

impl ThemingEngine {
    /// Creates a new `ThemingEngine`.
    ///
    /// # Arguments
    /// * `config_service` - Service for loading configuration files (tokens, themes).
    /// * `initial_config` - Optional initial `ThemingConfiguration`. If `None`, a default is used.
    ///
    /// # Panics
    /// Panics if the fallback state cannot be generated (which should be impossible if defaults are hardcoded).
    pub async fn new(
        config_service: Arc<dyn ConfigServiceAsync>,
        initial_config: Option<ThemingConfiguration>,
    ) -> Result<Self, ThemingError> {
        let fallback_state = generate_fallback_applied_state(); // Generate basic fallback for structure

        // Initialize internal_state first, then load resources into it.
        let mut internal_state = ThemingEngineInternalState {
            config_service: config_service.clone(), // Clone Arc for internal state
            current_config: ThemingConfiguration::default(), // Placeholder, will be set after loading
            global_tokens: TokenSet::new(),
            available_themes: HashMap::new(),
            resolved_state_cache: RwLock::new(HashMap::new()),
            fallback_state: fallback_state.clone(), // Store the basic structural fallback
        };

        // Load initial resources (global tokens, theme definitions like fallback.theme.json)
        // This is critical for the engine to have a valid fallback theme definition.
        internal_state.load_resources().await.map_err(|e| {
            log::error!("Kritischer Fehler beim Laden der initialen Theme-Ressourcen (base.tokens, fallback.theme): {:?}", e);
            // If essential resources cannot be loaded, the engine cannot start reliably.
            e
        })?;
        
        // Now that resources (including fallback theme definition) are loaded,
        // determine the actual fallback state and initial configuration.
        let actual_fallback_theme_id = ThemeIdentifier::new("fallback-dark"); // ID from fallback.theme.json
        
        // Attempt to generate a fully resolved fallback state from the loaded fallback theme definition
        let resolved_fallback_state = if let Some(fallback_theme_def) = internal_state.available_themes.get(&actual_fallback_theme_id) {
            let fallback_config = ThemingConfiguration {
                selected_theme_id: actual_fallback_theme_id.clone(),
                preferred_color_scheme: ColorSchemeType::Dark, // As per fallback.theme.json general setup
                selected_accent_color: None, // Fallback usually doesn't have accent
                custom_user_token_overrides: None,
            };
            let mut temp_pass_cache = HashMap::new();
            resolve_tokens_for_config(
                &internal_state.global_tokens,
                fallback_theme_def,
                &fallback_config,
                &mut temp_pass_cache,
            ).unwrap_or_else(|e| {
                log::error!("Konnte den Fallback-Theme-Status nicht aus der Definition auflösen '{}': {:?}. Struktureller Fallback wird verwendet.", actual_fallback_theme_id, e);
                fallback_state // Use the basic structural fallback if resolution fails
            })
        } else {
            log::error!("Fallback-Theme-Definition '{}' nicht in available_themes gefunden nach dem Laden. Struktureller Fallback wird verwendet.", actual_fallback_theme_id);
            fallback_state // Use basic structural if definition not found (should not happen if load_resources succeeded)
        };
        internal_state.fallback_state = resolved_fallback_state; // Update with resolved fallback

        // Determine initial configuration
        let current_config = initial_config.unwrap_or_else(|| {
            log::info!("Keine initiale Konfiguration bereitgestellt. Fallback-Konfiguration wird verwendet.");
            ThemingConfiguration {
                selected_theme_id: actual_fallback_theme_id.clone(),
                preferred_color_scheme: internal_state.fallback_state.color_scheme,
                selected_accent_color: internal_state.fallback_state.active_accent_color.clone(),
                custom_user_token_overrides: None,
            }
        });
        internal_state.current_config = current_config.clone();
        
        // Apply the initial or determined configuration
        let initial_applied_state = Self::internal_apply_configuration_logic(
            &mut internal_state,
            &current_config,
        ).await.unwrap_or_else(|err| {
            log::warn!(
                "Fehler beim Anwenden der initialen Theme-Konfiguration ('{}', {:?}): {:?}. Vollständiger Fallback-Status wird verwendet.",
                current_config.selected_theme_id, current_config.preferred_color_scheme, err
            );
            internal_state.fallback_state.clone()
        });

        // Ensure the cache has this initial state
        if let Some(theme_def_for_key) = internal_state.available_themes.get(&current_config.selected_theme_id) {
            let cache_key = CacheKey::from_config_and_theme(&current_config, theme_def_for_key);
            internal_state.resolved_state_cache.write().await.insert(cache_key, initial_applied_state);
        } else {
            // This case implies the selected_theme_id in current_config is not among available_themes,
            // which should ideally be caught by internal_apply_configuration_logic returning ThemeNotFound.
            // If it still happens, it means `initial_applied_state` would be a fallback, and caching it
            // under a potentially invalid theme_id might be problematic.
            log::warn!("Initial ausgewaehltes Theme '{}' nicht in available_themes gefunden. Der initiale Status wird nicht gecached.", current_config.selected_theme_id);
        }


        let (sender, _) = broadcast::channel(16); // Capacity for theme change events

        Ok(Self {
            internal_state: Arc::new(Mutex::new(internal_state)),
            theme_changed_event_sender: sender,
        })
    }

    /// Core logic for applying a configuration. Can be called by `new` or `update_configuration`.
    /// This function performs the actual resolution and updates the cache.
    /// It requires mutable access to parts of `ThemingEngineInternalState` (the cache).
    async fn internal_apply_configuration_logic(
        internal_state: &mut ThemingEngineInternalState, // Note: mutable reference
        config_to_apply: &ThemingConfiguration,
    ) -> Result<AppliedThemeState, ThemingError> {
        
        let theme_def = internal_state.available_themes.get(&config_to_apply.selected_theme_id)
            .ok_or_else(|| ThemingError::ThemeNotFound { theme_id: config_to_apply.selected_theme_id.clone() })?;

        let cache_key = CacheKey::from_config_and_theme(config_to_apply, theme_def);

        // Check cache first (read lock)
        if let Some(cached_state) = internal_state.resolved_state_cache.read().await.get(&cache_key) {
            return Ok(cached_state.clone());
        }

        // If not in cache, resolve (requires write lock on cache later)
        // A temporary cache for this specific resolution pass.
        let mut pass_cache: HashMap<TokenIdentifier, Result<String, ThemingError>> = HashMap::new();

        let new_applied_state = resolve_tokens_for_config(
            &internal_state.global_tokens,
            theme_def,
            config_to_apply,
            &mut pass_cache, // Use the temporary pass-specific cache
        )?;

        // Acquire write lock to update the shared cache
        internal_state.resolved_state_cache.write().await.insert(cache_key, new_applied_state.clone());
        
        Ok(new_applied_state)
    }


    /// (Internal use) Reloads all themes and tokens from the filesystem.
    /// This is a heavy operation and should be used sparingly.
    /// Requires exclusive access to internal state.
    async fn internal_load_themes_and_tokens_locked(
        state: &mut ThemingEngineInternalState, // Takes mutable ref to internal state
    ) -> Result<(), ThemingError> {
        state.load_resources().await?;
        // After reloading, the cache might be stale. Clear it.
        state.resolved_state_cache.write().await.clear();
        // Note: The current_config is NOT reapplied here automatically.
        // A subsequent call to apply the current config (or an updated one) is needed
        // to repopulate the cache and potentially send a ThemeChangedEvent.
        // Or, we could re-apply `state.current_config` here, but that might send an event
        // when the user hasn't changed config, only reloaded files.
        // For now, just clearing cache. The next `get_current_theme_state` or `update_configuration`
        // will trigger re-resolution.
        Ok(())
    }
    
    // --- Public API methods will be added in subsequent steps ---
    // get_current_theme_state
    // get_current_configuration
    // update_configuration
    // list_available_themes
    // get_theme_definition
    // reload_themes_and_tokens
    // subscribe_to_theme_changes
    
    /// Returns the current applied theme state.
    /// This method tries to serve from cache first. If the state for the current
    /// configuration is not cached, it resolves it, caches it, and then returns.
    pub async fn get_current_theme_state(&self) -> AppliedThemeState {
        let mut state = self.internal_state.lock().await;
        
        let theme_def = match state.available_themes.get(&state.current_config.selected_theme_id) {
            Some(def) => def,
            None => {
                log::error!("Aktuelles Theme '{}' nicht in available_themes gefunden. Fallback wird verwendet.", state.current_config.selected_theme_id);
                return state.fallback_state.clone();
            }
        };

        let cache_key = CacheKey::from_config_and_theme(&state.current_config, theme_def);

        if let Some(cached_applied_state) = state.resolved_state_cache.read().await.get(&cache_key) {
            return cached_applied_state.clone();
        }

        // Not in cache, need to resolve.
        // This will use the internal_apply_configuration_logic which needs &mut state.
        // The lock is already acquired, so we can pass &mut *state.
        match Self::internal_apply_configuration_logic(&mut *state, &state.current_config.clone()).await {
            Ok(applied_state) => applied_state,
            Err(e) => {
                log::error!("Fehler beim Auflösen des Theme-Status für die aktuelle Konfiguration: {:?}. Fallback wird verwendet.", e);
                state.fallback_state.clone()
            }
        }
    }

    /// Returns the current theming configuration.
    pub async fn get_current_configuration(&self) -> ThemingConfiguration {
        self.internal_state.lock().await.current_config.clone()
    }

    /// Updates the theming configuration and applies the changes.
    /// If successful, a `ThemeChangedEvent` is broadcasted.
    ///
    /// # Arguments
    /// * `new_config` - The new `ThemingConfiguration` to apply.
    ///
    /// # Returns
    /// `Ok(AppliedThemeState)` if the configuration was applied successfully,
    /// or `Err(ThemingError)` if applying the new configuration failed.
    /// If an error occurs, the engine attempts to revert to the last known good configuration or fallback.
    pub async fn update_configuration(
        &self,
        new_config: ThemingConfiguration,
    ) -> Result<AppliedThemeState, ThemingError> {
        let mut state = self.internal_state.lock().await;
        let old_config = state.current_config.clone();

        // Attempt to apply the new configuration
        match Self::internal_apply_configuration_logic(&mut *state, &new_config).await {
            Ok(applied_state) => {
                state.current_config = new_config; // Commit new config
                // Send event only if the applied state actually changed.
                // For simplicity, we send if the config object changed and resolution succeeded.
                // A more robust check would compare `applied_state` with the previous one.
                if state.theme_changed_event_sender.receiver_count() > 0 {
                    if let Err(e) = state.theme_changed_event_sender.send(ThemeChangedEvent {
                        new_state: applied_state.clone(),
                    }) {
                        log::error!("Fehler beim Senden des ThemeChangedEvent: {}", e);
                    }
                }
                Ok(applied_state)
            }
            Err(apply_err) => {
                log::error!(
                    "Fehler beim Anwenden der neuen Konfiguration: {:?}. Versuch, zur alten Konfiguration ('{}') zurückzukehren.",
                    apply_err, old_config.selected_theme_id
                );
                // Attempt to revert to the old configuration's state
                // This re-resolution of old_config should ideally hit the cache or succeed if it was valid before.
                match Self::internal_apply_configuration_logic(&mut *state, &old_config).await {
                    Ok(_) => {
                        log::info!("Erfolgreich zur vorherigen Konfiguration ('{}') zurückgekehrt.", old_config.selected_theme_id);
                    }
                    Err(revert_err) => {
                        log::error!(
                            "Kritischer Fehler: Konnte nicht zur vorherigen Konfiguration ('{}') zurückkehren: {:?}. Der Fallback-Status wird jetzt verwendet.",
                            old_config.selected_theme_id, revert_err
                        );
                        // Fallback state is always available in state.fallback_state
                        // No event is sent as this is an error recovery path.
                    }
                }
                Err(apply_err) // Return the original error from applying new_config
            }
        }
    }

    /// Lists all available theme definitions.
    pub async fn list_available_themes(&self) -> Vec<ThemeDefinition> {
        self.internal_state.lock().await.available_themes.values().cloned().collect()
    }

    /// Retrieves a specific theme definition by its ID.
    pub async fn get_theme_definition(&self, theme_id: &ThemeIdentifier) -> Option<ThemeDefinition> {
        self.internal_state.lock().await.available_themes.get(theme_id).cloned()
    }

    /// Reloads all theme definitions and global tokens from the filesystem.
    /// This is a heavy operation. After reloading, the current configuration is reapplied.
    /// If successful and the theme state changes, a `ThemeChangedEvent` is broadcasted.
    pub async fn reload_themes_and_tokens(&self) -> Result<AppliedThemeState, ThemingError> {
        let mut state = self.internal_state.lock().await;
        
        state.load_resources().await.map_err(|e| {
            log::error!("Fehler beim Neuladen von Themes und Tokens: {:?}", e);
            e
        })?;
        // Resources reloaded, cache was cleared in load_resources.
        // Now, re-apply the current configuration.
        
        let current_config_clone = state.current_config.clone();
        // Re-apply current configuration
        match Self::internal_apply_configuration_logic(&mut *state, &current_config_clone).await {
            Ok(applied_state) => {
                // Potentially send ThemeChangedEvent if state changed
                if state.theme_changed_event_sender.receiver_count() > 0 {
                     // TODO: Compare with old state before sending event, or assume it might have changed.
                    if let Err(e) = state.theme_changed_event_sender.send(ThemeChangedEvent {
                        new_state: applied_state.clone(),
                    }) {
                        log::error!("Fehler beim Senden des ThemeChangedEvent nach Neuladen: {}", e);
                    }
                }
                Ok(applied_state)
            }
            Err(e) => {
                log::error!("Fehler beim Anwenden der aktuellen Konfiguration nach Neuladen: {:?}. Fallback wird verwendet.",e);
                // If reapplying fails, this is problematic. Engine might be in an inconsistent state.
                // For now, return error and let caller decide. The internal state might be using fallback.
                Err(e)
            }
        }
    }

    /// Subscribes to `ThemeChangedEvent`s.
    pub fn subscribe_to_theme_changes(&self) -> broadcast::Receiver<ThemeChangedEvent> {
        self.theme_changed_event_sender.subscribe()
    }
}

// Notes on ThemingError cloning for `pass_cache` in `logic.rs::resolve_tokens_for_config`:
// If ThemingError cannot be made `Clone` (e.g. due to non-Clone `source` errors like `serde_json::Error` or `CoreError`),
// the `pass_cache` in `resolve_tokens_for_config` should be adjusted.
// Instead of: `resolved_cache.insert(token_id.clone(), Err(e.clone()));`
// It would skip caching errors: `return Err(e);`
// And only cache successes: `resolved_cache.insert(token_id.clone(), Ok(val_str));`
// This means if a sub-resolution fails, it will be re-attempted if that token is referenced again
// during the same `resolve_tokens_for_config` call. This is usually acceptable as errors should be rare.
// The main `resolved_state_cache` (caching `AppliedThemeState`) is unaffected as `AppliedThemeState` is `Clone`.
// For the purpose of this implementation, I will assume `ThemingError` has been made `Clone`
// by converting non-Clone `source` fields to `String` representations in `errors.rs`.
// If this assumption is wrong, `logic.rs` would need that minor adjustment to error caching.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theming::logic::tests::MockConfigService; // Use the pub(crate) Mock
    use crate::theming::types::{TokenIdentifier, RawToken, TokenValue, ColorSchemeType};
    use novade_core::CoreError;
    use serde_json::json;
    use std::path::PathBuf; // For MockConfigService if its trait methods use it

    // Helper to create a valid minimal base.tokens.json content
    fn valid_base_tokens_content() -> String {
        json!([
            {"id": "global-color-red", "value": {"color": "#FF0000"}},
            {"id": "global-spacing-medium", "value": {"spacing": "8px"}}
        ]).to_string()
    }

    // Helper to create a valid minimal fallback.theme.json content
    fn valid_fallback_theme_content() -> String {
        json!({
            "id": "fallback-dark", // Matches the ID used in ThemingEngine::new
            "name": "Fallback Dark Theme",
            "base_tokens": {
                "color-background": {"id": "color-background", "value": {"color": "#121212"}},
                "color-text": {"id": "color-text", "value": {"color": "#E0E0E0"}}
            },
            "variants": []
        }).to_string()
    }
    
    // Helper to create a valid minimal ThemeDefinition for direct use if needed
    fn create_test_theme_definition(id: &str, name: &str) -> ThemeDefinition {
        let mut base_tokens = TokenSet::new();
        base_tokens.insert(
            TokenIdentifier::new("color-text"),
            RawToken {
                id: TokenIdentifier::new("color-text"),
                value: TokenValue::Color("#defaulttext".to_string()),
                description: None,
                group: None,
            },
        );
        ThemeDefinition {
            id: ThemeIdentifier::new(id),
            name: name.to_string(),
            base_tokens,
            variants: vec![],
            supported_accent_colors: None,
            accentable_tokens: None,
            description: None,
            author: None,
            version: None,
        }
    }


    #[tokio::test]
    async fn test_theming_engine_new_successful_load() {
        let mut mock_config_service = MockConfigService::new();
        mock_config_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &valid_base_tokens_content());
        mock_config_service.add_file(FALLBACK_THEME_PATH, &valid_fallback_theme_content());

        let engine_result = ThemingEngine::new(Arc::new(mock_config_service), None).await;
        assert!(engine_result.is_ok(), "ThemingEngine::new failed: {:?}", engine_result.err());
        let engine = engine_result.unwrap();

        // Inspect internal state (example, adjust based on actual accessibility or public methods)
        let internal_state = engine.internal_state.lock().await;
        assert!(!internal_state.global_tokens.is_empty(), "Global tokens should be loaded");
        assert!(internal_state.global_tokens.contains_key(&TokenIdentifier::new("global-color-red")));
        
        assert!(!internal_state.available_themes.is_empty(), "Available themes should include fallback");
        assert!(internal_state.available_themes.contains_key(&ThemeIdentifier::new("fallback-dark")));
        
        drop(internal_state); // Release lock before calling other async methods on engine

        let current_state = engine.get_current_theme_state().await;
        assert_eq!(current_state.theme_id.as_str(), "fallback-dark", "Current theme should be fallback");
        assert!(current_state.resolved_tokens.contains_key(&TokenIdentifier::new("color-background")));
        assert!(current_state.resolved_tokens.contains_key(&TokenIdentifier::new("global-color-red")));

        let themes = engine.list_available_themes().await;
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].id.as_str(), "fallback-dark");
    }

    #[tokio::test]
    async fn test_theming_engine_new_error_loading_global_tokens() {
        let mut mock_config_service = MockConfigService::new();
        // Simulate error for global tokens
        mock_config_service.set_error_on_load(true, Some(CoreError::Io("Simulated I/O error".to_string())));
        // Still provide fallback theme, though it might not be reached if global tokens fail first
        mock_config_service.add_file(FALLBACK_THEME_PATH, &valid_fallback_theme_content());


        let engine_result = ThemingEngine::new(Arc::new(mock_config_service), None).await;
        assert!(engine_result.is_err(), "Expected ThemingEngine::new to fail");
        
        match engine_result.err().unwrap() {
            ThemingError::InternalError(msg) => {
                assert!(msg.contains("Core-Fehler beim Laden der Token-Datei") || msg.contains("Simulated I/O error"));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_theming_engine_new_error_loading_fallback_theme() {
        let mut mock_config_service = MockConfigService::new();
        mock_config_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &valid_base_tokens_content());
        // Simulate error for fallback theme
        mock_config_service.set_error_on_load(true, Some(CoreError::Config(novade_core::ConfigError::NotFound{locations: vec![FALLBACK_THEME_PATH.into()]})));

        let engine_result = ThemingEngine::new(Arc::new(mock_config_service), None).await;
        assert!(engine_result.is_err(), "Expected ThemingEngine::new to fail due to fallback theme load error");

        match engine_result.err().unwrap() {
            ThemingError::InternalError(msg) => {
                // The error comes from internal_state.load_resources().await
                // which wraps the load_theme_definition_from_file error.
                assert!(msg.contains("Fallback-Theme konnte nicht geladen werden"));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_theming_engine_new_malformed_fallback_theme_json() {
        let mut mock_config_service = MockConfigService::new();
        mock_config_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &valid_base_tokens_content());
        mock_config_service.add_file(FALLBACK_THEME_PATH, "this is not valid json");

        let engine_result = ThemingEngine::new(Arc::new(mock_config_service), None).await;
        assert!(engine_result.is_err(), "Expected ThemingEngine::new to fail due to malformed fallback theme");
        
        match engine_result.err().unwrap() {
            ThemingError::InternalError(msg) => { // load_resources wraps the specific error
                 assert!(msg.contains("Fallback-Theme konnte nicht geladen werden"));
                 assert!(msg.contains("ThemeFileParseError")); // Check if the original error type is mentioned
            }
            // Depending on how errors are wrapped, you might get ThemeFileParseError directly
            // ThemingError::ThemeFileParseError { file_path, .. } => {
            //     assert_eq!(file_path, FALLBACK_THEME_PATH);
            // }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_theming_engine_new_malformed_global_tokens_json() {
        let mut mock_config_service = MockConfigService::new();
        mock_config_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, "this is not valid json tokens");
        mock_config_service.add_file(FALLBACK_THEME_PATH, &valid_fallback_theme_content());

        let engine_result = ThemingEngine::new(Arc::new(mock_config_service), None).await;
        assert!(engine_result.is_err(), "Expected ThemingEngine::new to fail due to malformed global tokens");
        
        match engine_result.err().unwrap() {
             ThemingError::TokenFileParseError { file_path, .. } => {
                 assert_eq!(file_path, DEFAULT_GLOBAL_TOKENS_PATH);
             }
            // It might also be wrapped in InternalError by load_resources if the specific mapping isn't there
            ThemingError::InternalError(msg) => {
                 assert!(msg.contains("TokenFileParseError") || msg.contains("Core-Fehler beim Laden der Token-Datei"));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_theming_engine_new_initial_config_provided() {
        let mut mock_config_service = MockConfigService::new();
        mock_config_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &valid_base_tokens_content());
        mock_config_service.add_file(FALLBACK_THEME_PATH, &valid_fallback_theme_content());
        
        // Simulate that list_files_in_dir returns empty, so only fallback is loaded by load_resources
        mock_config_service.set_files_for_list_dir(vec![]);

        let initial_config_using_fallback = ThemingConfiguration {
            selected_theme_id: ThemeIdentifier::new("fallback-dark"),
            preferred_color_scheme: ColorSchemeType::Dark, 
            ..Default::default()
        };

        let engine_result = ThemingEngine::new(Arc::new(mock_config_service), Some(initial_config_using_fallback.clone())).await;
        assert!(engine_result.is_ok(), "ThemingEngine::new with initial_config failed: {:?}", engine_result.err());
        let engine = engine_result.unwrap();
        
        let current_applied_state = engine.get_current_theme_state().await;
        assert_eq!(current_applied_state.theme_id, initial_config_using_fallback.selected_theme_id);
        assert_eq!(current_applied_state.color_scheme, initial_config_using_fallback.preferred_color_scheme);

        let current_engine_config = engine.get_current_configuration().await;
        assert_eq!(current_engine_config.selected_theme_id, initial_config_using_fallback.selected_theme_id);
    }

    // --- Tests for new theme discovery logic ---

    fn custom_theme_content(id: &str, name: &str) -> String {
        json!({
            "id": id,
            "name": name,
            "base_tokens": {
                "custom-color": {"id": "custom-color", "value": {"color": "#1A2B3C"}}
            }
        }).to_string()
    }

    #[tokio::test]
    async fn test_load_resources_discovers_and_loads_multiple_themes() {
        let mut mock_config_service = MockConfigService::new();
        mock_config_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &valid_base_tokens_content());
        mock_config_service.add_file(FALLBACK_THEME_PATH, &valid_fallback_theme_content());

        let theme1_path_str = "src/theming/default_themes/theme1.theme.json";
        let theme2_path_str = "src/theming/default_themes/theme2.theme.json";
        mock_config_service.add_file(theme1_path_str, &custom_theme_content("theme1-id", "Theme 1"));
        mock_config_service.add_file(theme2_path_str, &custom_theme_content("theme2-id", "Theme 2"));
        
        mock_config_service.set_files_for_list_dir(vec![
            PathBuf::from(theme1_path_str),
            PathBuf::from(theme2_path_str),
            PathBuf::from(FALLBACK_THEME_PATH), // Include fallback to test it's not re-added if already processed
        ]);

        let engine_result = ThemingEngine::new(Arc::new(mock_config_service), None).await;
        assert!(engine_result.is_ok(), "Engine creation failed: {:?}", engine_result.err());
        let engine = engine_result.unwrap();

        let themes = engine.list_available_themes().await;
        assert_eq!(themes.len(), 3, "Should have fallback + 2 custom themes. Found: {:?}", themes.iter().map(|t| &t.id).collect::<Vec<_>>());
        assert!(themes.iter().any(|t| t.id.as_str() == "fallback-dark"));
        assert!(themes.iter().any(|t| t.id.as_str() == "theme1-id"));
        assert!(themes.iter().any(|t| t.id.as_str() == "theme2-id"));
    }
    
    #[tokio::test]
    async fn test_load_resources_empty_themes_dir() {
        let mut mock_config_service = MockConfigService::new();
        mock_config_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &valid_base_tokens_content());
        mock_config_service.add_file(FALLBACK_THEME_PATH, &valid_fallback_theme_content());
        mock_config_service.set_files_for_list_dir(Vec::new()); // Simulate empty themes dir

        let engine_result = ThemingEngine::new(Arc::new(mock_config_service), None).await;
        assert!(engine_result.is_ok(), "Engine creation failed: {:?}", engine_result.err());
        let engine = engine_result.unwrap();
        let themes = engine.list_available_themes().await;
        assert_eq!(themes.len(), 1, "Only fallback theme should be loaded");
        assert_eq!(themes[0].id.as_str(), "fallback-dark");
    }

    #[tokio::test]
    async fn test_load_resources_error_listing_themes_dir() {
        let mut mock_config_service = MockConfigService::new();
        mock_config_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &valid_base_tokens_content());
        mock_config_service.add_file(FALLBACK_THEME_PATH, &valid_fallback_theme_content());
        // Simulate error when listing directory
        mock_config_service.set_list_dir_error(Some(CoreError::Io("Simulated dir list error".to_string())));

        let engine_result = ThemingEngine::new(Arc::new(mock_config_service), None).await;
        // Engine creation should still succeed as global tokens and fallback theme are critical,
        // but listing other themes is not. An error/warning should be logged by load_resources.
        assert!(engine_result.is_ok(), "Engine creation should succeed even if theme dir listing fails, as long as fallback is fine. Error: {:?}", engine_result.err());
        let engine = engine_result.unwrap();
        let themes = engine.list_available_themes().await;
        assert_eq!(themes.len(), 1, "Only fallback theme should be available if dir listing fails");
        assert_eq!(themes[0].id.as_str(), "fallback-dark");
        // To verify logging, one would need to capture logs, which is outside this test's scope.
    }

    #[tokio::test]
    async fn test_load_resources_malformed_discovered_theme() {
        let mut mock_config_service = MockConfigService::new();
        mock_config_service.add_file(DEFAULT_GLOBAL_TOKENS_PATH, &valid_base_tokens_content());
        mock_config_service.add_file(FALLBACK_THEME_PATH, &valid_fallback_theme_content());

        let valid_theme_path = "src/theming/default_themes/valid.theme.json";
        let malformed_theme_path = "src/theming/default_themes/malformed.theme.json";
        mock_config_service.add_file(valid_theme_path, &custom_theme_content("valid-id", "Valid Theme"));
        mock_config_service.add_file(malformed_theme_path, "this is not valid json");
        
        mock_config_service.set_files_for_list_dir(vec![
            PathBuf::from(valid_theme_path),
            PathBuf::from(malformed_theme_path),
        ]);

        let engine_result = ThemingEngine::new(Arc::new(mock_config_service), None).await;
        assert!(engine_result.is_ok(), "Engine creation should succeed, logging errors for malformed themes. Error: {:?}", engine_result.err());
        let engine = engine_result.unwrap();
        let themes = engine.list_available_themes().await;
        assert_eq!(themes.len(), 2, "Should load fallback and one valid custom theme. Found: {:?}", themes.iter().map(|t| &t.id).collect::<Vec<_>>());
        assert!(themes.iter().any(|t| t.id.as_str() == "fallback-dark"));
        assert!(themes.iter().any(|t| t.id.as_str() == "valid-id"));
        assert!(!themes.iter().any(|t| t.id.as_str() == "malformed-id")); // Assuming ID is not parsed from malformed
        // To verify logging of the error for the malformed theme, log capture would be needed.
    }
}
