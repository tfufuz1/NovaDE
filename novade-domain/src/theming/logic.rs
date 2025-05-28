use crate::theming::types::{RawToken, TokenSet, TokenValue, TokenIdentifier, ThemeDefinition, ThemeIdentifier, ColorSchemeType, AccentColor, AccentModificationType, AppliedThemeState, ThemingConfiguration};
use crate::theming::errors::ThemingError;
use crate::ConfigServiceAsync; // Corrected path
use novade_core::config::ConfigFormat; // ConfigFormat is still from novade_core, assuming it's public
use novade_core::types::Color as CoreColor; // Assuming this exists
use novade_core::CoreError; // Corrected path
use std::collections::{HashMap, BTreeMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid; // For cache key if needed, or other unique identifiers
// Ensure serde_json is in scope if using json! macro, or for error types.
// It should be in Cargo.toml for novade-domain.
use serde_json;


pub const MAX_TOKEN_RESOLUTION_DEPTH: u8 = 16;

/// Loads raw tokens from a specified file path using the configuration service.
///
/// # Arguments
/// * `config_service` - An `Arc` to a `ConfigServiceAsync` implementation.
/// * `file_path` - The path to the token file.
///
/// # Returns
/// A `Result` containing the loaded `TokenSet` or a `ThemingError`.
pub async fn load_raw_tokens_from_file(
    config_service: Arc<dyn ConfigServiceAsync>,
    file_path: &str,
) -> Result<TokenSet, ThemingError> {
    // ConfigServiceAsync is expected to return a String representation of the file content.
    // The path given to it would be relative to its own configured root or an absolute path.
    // Assuming load_config_file_content_async is the correct method name from ConfigServiceAsync trait.
    // The trait defined in Turn 24 had read_config_file_string.
    let file_content = config_service
        .read_config_file_string(file_path) // Using method from trait defined in Turn 24
        .await
        .map_err(|core_err| {
            match core_err {
                // Assuming CoreError has variants like IoError, ConfigFormatError, NotFound
                // This mapping needs to be accurate based on novade_core::CoreError definition
                CoreError::Config(ref ce) => match ce { // Example if CoreError::Config wraps a ConfigError enum
                    novade_core::ConfigError::NotFound{..} => ThemingError::InternalError(format!("Konfigurationsdatei nicht gefunden: {}. Ursprungspfad: {}", core_err, file_path)),
                    _ => ThemingError::InternalError(format!("Core-Config-Fehler beim Laden der Token-Datei '{}': {}", file_path, core_err)),
                },
                // Add other CoreError variants as needed
                _ => ThemingError::InternalError(format!("Core-Fehler beim Laden der Token-Datei '{}': {}", file_path, core_err)),
            }
        })?;

    let raw_tokens: Vec<RawToken> = serde_json::from_str(&file_content)
        .map_err(|e| ThemingError::TokenFileParseError {
            file_path: file_path.to_string(),
            source: e, // Changed from source_message to source to match ThemingError definition potentially
        })?;

    let mut token_set = TokenSet::new();
    for token in raw_tokens {
        validate_raw_token_value(&token)?; 
        if token_set.insert(token.id.clone(), token).is_some() {
            // log::warn!("Duplicate token ID '{}' in file '{}'", token.id, file_path);
        }
    }
    Ok(token_set)
}

fn validate_raw_token_value(token: &RawToken) -> Result<(), ThemingError> {
    match &token.value {
        TokenValue::Opacity(s) => {
            if let Ok(val) = s.trim_end_matches('%').parse::<f32>() {
                let actual_val = if s.ends_with('%') { val / 100.0 } else { val };
                if !(0.0..=1.0).contains(&actual_val) {
                    return Err(ThemingError::InvalidTokenValue { // Corrected: direct use of variant
                        token_id: token.id.clone(),
                        message: format!("Opacity '{}' muss zwischen 0.0 und 1.0 (oder 0% und 100%) liegen.", s),
                    });
                }
            }
        }
        TokenValue::Color(s) => {
            if !s.starts_with('#') && !s.starts_with('{') {
                // log::warn!("Color token '{}' value '{}' does not start with '#' or '{{'. Assuming it's a reference or named color.", token.id, s);
            }
        }
        _ => {}
    }
    Ok(())
}

fn resolve_single_token_value<'a>(
    token_id: &TokenIdentifier,
    base_tokens: &'a TokenSet,
    variant_tokens: &'a TokenSet,
    user_overrides: &'a TokenSet,
    accent_color: Option<&'a CoreColor>,
    accentable_tokens_map: &'a HashMap<TokenIdentifier, AccentModificationType>,
    current_path: &mut Vec<TokenIdentifier>,
    depth: u8,
    resolved_cache: &mut HashMap<TokenIdentifier, Result<String, ThemingError>>, 
) -> Result<TokenValue, ThemingError> {
    if depth > MAX_TOKEN_RESOLUTION_DEPTH {
        return Err(ThemingError::MaxResolutionDepthExceeded {
            token_id: token_id.clone(),
            depth: MAX_TOKEN_RESOLUTION_DEPTH,
        });
    }

    if current_path.contains(token_id) {
        return Err(ThemingError::CyclicTokenReference {
            token_id: token_id.clone(),
            path: current_path.clone(),
        });
    }

    current_path.push(token_id.clone());

    let raw_token_opt = user_overrides
        .get(token_id)
        .or_else(|| variant_tokens.get(token_id))
        .or_else(|| base_tokens.get(token_id));

    let result = match raw_token_opt {
        Some(raw_token) => {
            // Step 1: Resolve the token value. If it's a reference, resolve it recursively.
            // Pass `None` for `accent_color` during this initial resolution phase to get the
            // true underlying value before considering accent application for the current token_id.
            let mut resolved_value = if let TokenValue::Reference(ref_id) = &raw_token.value {
                resolve_single_token_value(
                    ref_id,
                    base_tokens,
                    variant_tokens,
                    user_overrides,
                    None, // KEY CHANGE: Resolve reference without accent first
                    accentable_tokens_map, // Pass along for nested resolutions
                    current_path,
                    depth + 1,
                    resolved_cache,
                )?
            } else {
                // Not a reference, so clone its direct value
                raw_token.value.clone()
            };

            // Step 2: Apply accent color if:
            // - An `accent_color` is provided for the current resolution context.
            // - The *original* `token_id` (being resolved in this call) is marked as accentable.
            // - The *resolved value* (from Step 1) is a color.
            if let Some(accent) = accent_color {
                if let Some(modification_type) = accentable_tokens_map.get(token_id) {
                    if let TokenValue::Color(ref original_color_str) = resolved_value {
                        // The original_color_str is now a fully resolved color string (not a reference).
                        // So, we parse it directly.
                        let original_core_color = CoreColor::from_hex(original_color_str)
                            .map_err(|_| ThemingError::InvalidTokenValue {
                                token_id: token_id.clone(), // Error is for the current token if its resolved color is bad
                                message: format!("Resolved color '{}' for token '{}' is not a valid hex color for accent application.", original_color_str, token_id),
                            })?;
                        
                        resolved_value = TokenValue::Color(
                            apply_accent_to_color(&original_core_color, accent, modification_type)
                                .map_err(|e_msg| ThemingError::AccentColorApplicationError { token_id: token_id.clone(), message: e_msg })?
                                .to_hex_string(),
                        );
                    }
                    // If the resolved value is not a color (e.g., a dimension, spacing),
                    // it's not modified by the accent color, which is correct.
                }
            }
            Ok(resolved_value)
        }
        None => Err(ThemingError::TokenNotFound { token_id: token_id.clone() }),
    };

    current_path.pop();
    result
}

fn parse_color_string<'a>(
    color_str: &str,
    token_id_for_error: &TokenIdentifier, // ID of the token whose value is this color_str
    base_tokens: &'a TokenSet,
    variant_tokens: &'a TokenSet,
    user_overrides: &'a TokenSet,
    accentable_tokens_map: &'a HashMap<TokenIdentifier, AccentModificationType>,
    current_path: &mut Vec<TokenIdentifier>,
    depth: u8,
    resolved_cache: &mut HashMap<TokenIdentifier, Result<String, ThemingError>>,
) -> Result<CoreColor, ThemingError> {
    if color_str.starts_with('{') && color_str.ends_with('}') {
        let ref_id_str = color_str.trim_start_matches('{').trim_end_matches('}');
        let ref_id = TokenIdentifier::new(ref_id_str);
        
        // When resolving a reference *within* a color string (e.g. "{primary-color}"),
        // we expect it to resolve to a color. We pass `None` for accent_color here
        // because the accent application is determined by the top-level token, not this
        // intermediate reference.
        match resolve_single_token_value(
            &ref_id, 
            base_tokens, 
            variant_tokens, 
            user_overrides, 
            None, // Explicitly no accent for resolving the reference itself
            accentable_tokens_map, 
            current_path, 
            depth, // Note: depth here might need careful consideration if parse_color_string is called deep in a stack
            resolved_cache
        )? {
            TokenValue::Color(hex_color) => CoreColor::from_hex(&hex_color).map_err(|_| {
                ThemingError::InvalidTokenValue {
                    token_id: ref_id.clone(), // Error is for the referenced token if its color is invalid
                    message: format!("Referenzierter Farbwert '{}' ist kein gültiges Hex-Format.", hex_color),
                }
            }),
            other_value => Err(ThemingError::InvalidTokenValue {
                token_id: ref_id.clone(),
                message: format!("Referenzierter Token muss ein Farbwert sein, gefunden: {:?}", other_value),
            }),
        }
    } else {
        CoreColor::from_hex(color_str).map_err(|_| {
            ThemingError::InvalidTokenValue {
                token_id: token_id_for_error.clone(),
                message: format!("Farbwert '{}' ist kein gültiges Hex-Format.", color_str),
            }
        })
    }
}

fn resolved_token_value_to_string(token_value: &TokenValue, token_id: &TokenIdentifier) -> Result<String, ThemingError> {
    match token_value {
        TokenValue::Color(s) |
        TokenValue::Dimension(s) |
        TokenValue::FontFamily(s) |
        TokenValue::FontWeight(s) |
        TokenValue::FontSize(s) |
        TokenValue::LineHeight(s) |
        TokenValue::LetterSpacing(s) |
        TokenValue::Duration(s) |
        TokenValue::BorderStyle(s) |
        TokenValue::BorderWidth(s) |
        TokenValue::BoxShadow(s) |
        TokenValue::Opacity(s) |
        TokenValue::Spacing(s) |
        TokenValue::Generic(s) => Ok(s.clone()),
        TokenValue::Typography(map) => {
            serde_json::to_string(map).map_err(|e| ThemingError::SerdeError(format!("Fehler beim Serialisieren des Typografie-Tokens '{}': {}", token_id, e)))
        }
        TokenValue::Reference(ref_id) => {
            Err(ThemingError::InternalError(format!(
                "Unerwarteter nicht aufgelöster Verweis '{}' für Token '{}' angetroffen.", ref_id, token_id
            )))
        }
    }
}

fn apply_accent_to_color(
    original_color: &CoreColor,
    accent_color: &CoreColor,
    modification_type: &AccentModificationType,
) -> Result<CoreColor, String> {
    // Ensure factors are within a reasonable range if necessary, though CoreColor methods already clamp.
    // For factors used in Lighten/Darken/Interpolate, they are typically 0.0 to 1.0.
    // As per subtask instructions, Lighten, Darken, and TintWithOriginal will all use a mix/interpolation logic.
    match modification_type {
        AccentModificationType::DirectReplace => Ok(accent_color.clone()),
        AccentModificationType::Lighten(factor) => {
            if !(*factor >= 0.0 && *factor <= 1.0) {
                return Err(format!("Lighten factor must be between 0.0 and 1.0, got {}.", factor));
            }
            // Assuming CoreColor::mix(self, other, factor) means self*(1-factor) + other*factor
            // This implements: original_color.mix(accent_color, factor)
            Ok(original_color.mix(accent_color, *factor))
        }
        AccentModificationType::Darken(factor) => {
            if !(*factor >= 0.0 && *factor <= 1.0) {
                return Err(format!("Darken factor must be between 0.0 and 1.0, got {}.", factor));
            }
            // Assuming CoreColor::mix(self, other, factor) means self*(1-factor) + other*factor
            // This implements: original_color.mix(accent_color, factor)
            // The "darken" effect is achieved by the choice of accent_color and factor.
            Ok(original_color.mix(accent_color, *factor))
        }
        AccentModificationType::TintWithOriginal(factor) => {
            if !(*factor >= 0.0 && *factor <= 1.0) {
                return Err(format!("TintWithOriginal factor must be between 0.0 and 1.0, got {}.", factor));
            }
            // CoreColor::interpolate(self, other, t) is assumed to be equivalent to mix.
            // original_color.interpolate(accent_color, factor) is original*(1-factor) + accent*factor
            Ok(original_color.interpolate(accent_color, *factor))
        }
    }
}

pub fn validate_tokenset_for_cycles(_token_set: &TokenSet) -> Result<(), ThemingError> {
    Ok(())
}

pub async fn load_theme_definition_from_file(
    config_service: Arc<dyn ConfigServiceAsync>,
    file_path: &str,
) -> Result<ThemeDefinition, ThemingError> {
    let file_content = config_service
        .read_config_file_string(file_path) // Using method from trait defined in Turn 24
        .await
        .map_err(|core_err| match core_err {
            // Assuming CoreError has variants like IoError, ConfigFormatError, NotFound
             CoreError::Config(ref ce) => match ce { // Example if CoreError::Config wraps a ConfigError enum
                novade_core::ConfigError::NotFound{..} => ThemingError::InternalError(format!("Theme-Definitionsdatei nicht gefunden: {}. Ursprungspfad: {}", core_err, file_path)),
                _ => ThemingError::InternalError(format!("Core-Config-Fehler beim Laden der Theme-Definitionsdatei '{}': {}", file_path, core_err)),
            },
            _ => ThemingError::InternalError(format!("Core-Fehler beim Laden der Theme-Definitionsdatei '{}': {}", file_path, core_err)),
        })?;

    serde_json::from_str(&file_content).map_err(|e| {
        ThemingError::ThemeFileParseError {
            file_path: file_path.to_string(),
            source: e, // Changed from source_message
        }
    })
}

pub async fn load_and_validate_token_files(
    config_service: Arc<dyn ConfigServiceAsync>,
    file_paths: &[String], 
) -> Result<TokenSet, ThemingError> {
    if file_paths.is_empty() {
        return Ok(TokenSet::new()); 
    }
    load_raw_tokens_from_file(config_service, &file_paths[0]).await
}

pub async fn load_and_validate_theme_files(
    config_service: Arc<dyn ConfigServiceAsync>,
    theme_file_paths: &[String],
) -> Result<Vec<ThemeDefinition>, ThemingError> {
    let mut themes = Vec::new();
    if theme_file_paths.is_empty() {
        return Ok(themes);
    }
    for path in theme_file_paths {
        themes.push(load_theme_definition_from_file(config_service.clone(), path).await?);
    }
    Ok(themes)
}

pub fn resolve_tokens_for_config(
    global_tokens: &TokenSet,
    theme_definition: &ThemeDefinition,
    config: &ThemingConfiguration,
    resolved_cache: &mut HashMap<TokenIdentifier, Result<String, ThemingError>>,
) -> Result<AppliedThemeState, ThemingError> {
    let mut combined_base_tokens = global_tokens.clone();
    combined_base_tokens.extend(theme_definition.base_tokens.clone()); 

    let variant_tokens = theme_definition
        .variants
        .iter()
        .find(|v| v.applies_to_scheme == config.preferred_color_scheme)
        .map(|v| v.tokens.clone())
        .unwrap_or_default();

    let user_overrides = config.custom_user_token_overrides.as_ref().cloned().unwrap_or_default();
    let accentable_tokens_map = theme_definition.accentable_tokens.as_ref().cloned().unwrap_or_default();
    let mut resolved_tokens_map: BTreeMap<TokenIdentifier, String> = BTreeMap::new();
    let mut all_token_ids: HashSet<TokenIdentifier> = HashSet::new();
    all_token_ids.extend(combined_base_tokens.keys().cloned());
    all_token_ids.extend(variant_tokens.keys().cloned());
    all_token_ids.extend(user_overrides.keys().cloned());
    all_token_ids.extend(accentable_tokens_map.keys().cloned());

    for token_id in all_token_ids {
        if let Some(cached_result) = resolved_cache.get(&token_id) {
            match cached_result {
                Ok(val_str) => {
                    resolved_tokens_map.insert(token_id.clone(), val_str.clone());
                    continue;
                }
                Err(e) => { if e.is_cloneable() { resolved_cache.insert(token_id.clone(), Err(e.clone())); return Err(e.clone()); } } // Ensure error is Clone
            }
        }
        
        let mut current_path = Vec::new();
        let resolved_value_result = resolve_single_token_value(
            &token_id,
            &combined_base_tokens,
            &variant_tokens,
            &user_overrides,
            config.selected_accent_color.as_ref(),
            &accentable_tokens_map,
            &mut current_path,
            0,
            resolved_cache,
        );

        match resolved_value_result {
            Ok(value) => {
                match resolved_token_value_to_string(&value, &token_id) {
                    Ok(val_str) => {
                        resolved_tokens_map.insert(token_id.clone(), val_str.clone());
                        resolved_cache.insert(token_id.clone(), Ok(val_str)); 
                    }
                    Err(e) => {
                        if e.is_cloneable() { resolved_cache.insert(token_id.clone(), Err(e.clone())); }
                        return Err(e); 
                    }
                }
            }
            Err(e) => {
                if e.is_cloneable() { resolved_cache.insert(token_id.clone(), Err(e.clone())); }
                return Err(e); 
            }
        }
    }

    Ok(AppliedThemeState {
        theme_id: config.selected_theme_id.clone(),
        color_scheme: config.preferred_color_scheme,
        active_accent_color: config.selected_accent_color.clone(),
        resolved_tokens: resolved_tokens_map,
    })
}

pub fn generate_fallback_applied_state() -> AppliedThemeState {
    AppliedThemeState {
        theme_id: ThemeIdentifier::new("structural-fallback-theme"), 
        color_scheme: ColorSchemeType::Dark, 
        active_accent_color: None,
        resolved_tokens: BTreeMap::new(), 
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theming::types::{TokenIdentifier, RawToken, TokenValue, ColorSchemeType, ThemeIdentifier, ThemingConfiguration};
    // ConfigServiceAsync is now crate::ConfigServiceAsync due to lib.rs re-export
    // For tests, we use the mock defined below.
    // use crate::ConfigServiceAsync; 
    use novade_core::CoreError; 
    use novade_core::types::Color as CoreColor;
    use std::sync::Arc;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use serde_json::json; // For constructing test JSON values
    use std::path::PathBuf; // Required for MockConfigService trait implementation

    #[derive(Debug, Clone)]
    pub(crate) struct MockConfigService { // Made pub(crate)
        files: HashMap<String, String>, 
        should_error_on_load: bool,
        error_type: Option<CoreError>, 
    }

    impl MockConfigService {
        fn new() -> Self {
            Self {
                files: HashMap::new(),
                should_error_on_load: false,
                error_type: None,
            }
        }

        fn add_file(&mut self, path: &str, content: &str) {
            self.files.insert(path.to_string(), content.to_string());
        }

        #[allow(dead_code)] 
        fn set_error_on_load(&mut self, error: bool, error_type: Option<CoreError>) {
            self.should_error_on_load = error;
            self.error_type = error_type;
        }
    }

    #[async_trait]
    impl ConfigServiceAsync for MockConfigService { // Uses the trait from current crate scope
        // Corrected method name to match trait definition from Turn 24 (ports/config_service.rs)
        async fn read_config_file_string(&self, file_path: &str) -> Result<String, CoreError> {
            if self.should_error_on_load {
                return Err(self.error_type.clone().unwrap_or_else(|| CoreError::Internal("Mock error".to_string())));
            }
            self.files
                .get(file_path)
                .cloned()
                .ok_or_else(|| CoreError::Config(novade_core::ConfigError::NotFound{locations:vec![file_path.into()]})) // Use a more specific CoreError
        }
        
        // Dummy implementations for other ConfigServiceAsync methods
        async fn write_config_file_string(&self, _file_path: &str, _content: String) -> Result<(), CoreError> { unimplemented!() }
        async fn read_file_to_string(&self, _path: &Path) -> Result<String, CoreError> { unimplemented!() }
        async fn list_files_in_dir(&self, _dir_path: &Path, _extension: Option<&str>) -> Result<Vec<PathBuf>, CoreError> { unimplemented!() }
        async fn get_config_dir(&self) -> Result<PathBuf, CoreError> { unimplemented!() }
        async fn get_data_dir(&self) -> Result<PathBuf, CoreError> { unimplemented!() }

        // Old methods from a previous version of ConfigServiceAsync, remove if not in current trait
        // async fn load_config_file_content_async(&self, file_path: &str) -> Result<String, CoreError> { self.read_config_file_string(file_path).await }
        // async fn save_config_file_content_async(&self, _file_path: &str, _content: &str) -> Result<(), CoreError> { unimplemented!() }
        // async fn list_config_files_async(&self, _dir_path: &str) -> Result<Vec<String>, CoreError> { unimplemented!() }
        // fn get_config_file_path(&self, _app_id: &crate::shared_types::ApplicationId, _config_name: &str, _format: Option<ConfigFormat>) -> Result<String, CoreError> { unimplemented!() }
        // fn get_config_dir_path(&self, _app_id: &crate::shared_types::ApplicationId, _subdir: Option<&str>) -> Result<String, CoreError> { unimplemented!() }
        // fn ensure_config_dir_exists(&self, _app_id: &crate::shared_types::ApplicationId) -> Result<String, CoreError> { unimplemented!() }
    }

    #[tokio::test]
    async fn test_load_raw_tokens_valid_file() {
        let mut mock_service = MockConfigService::new();
        let file_path = "test_tokens.json";
        let file_content_value = json!([
            {"id": "color-red", "value": {"color": "#FF0000"}},
            {"id": "spacing-small", "value": {"spacing": "4px"}, "description": "Small space"}
        ]);
        mock_service.add_file(file_path, &file_content_value.to_string());

        let result = load_raw_tokens_from_file(Arc::new(mock_service), file_path).await;
        assert!(result.is_ok());
        let token_set = result.unwrap();
        assert_eq!(token_set.len(), 2);
        assert!(token_set.contains_key(&TokenIdentifier::new("color-red")));
        assert_eq!(
            token_set.get(&TokenIdentifier::new("spacing-small")).unwrap().description,
            Some("Small space".to_string())
        );
    }

    #[tokio::test]
    async fn test_load_raw_tokens_file_not_found() {
        let mock_service = MockConfigService::new(); 
        let file_path = "non_existent.json";
        let result = load_raw_tokens_from_file(Arc::new(mock_service), file_path).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::InternalError(msg) => {
                assert!(msg.contains("Konfigurationsdatei nicht gefunden") || msg.contains("NotFound"));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_load_raw_tokens_invalid_json() {
        let mut mock_service = MockConfigService::new();
        let file_path = "invalid.json";
        let file_content = r#"[{"id": "color-blue", "value": "not-an-object"}"#; 
        mock_service.add_file(file_path, file_content);
        let result = load_raw_tokens_from_file(Arc::new(mock_service), file_path).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::TokenFileParseError { file_path: fp, source: _ } => { // Matched source
                assert_eq!(fp, file_path);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_load_raw_tokens_duplicate_ids_overwrites() {
        let mut mock_service = MockConfigService::new();
        let file_path = "duplicates.json";
        let file_content_value = json!([
            {"id": "color-primary", "value": {"color": "#AAAAAA"}},
            {"id": "color-primary", "value": {"color": "#BBBBBB"}}
        ]); 
        mock_service.add_file(file_path, &file_content_value.to_string());
        let result = load_raw_tokens_from_file(Arc::new(mock_service), file_path).await;
        assert!(result.is_ok());
        let token_set = result.unwrap();
        assert_eq!(token_set.len(), 1);
        match token_set.get(&TokenIdentifier::new("color-primary")).unwrap().value {
            TokenValue::Color(ref c) if c == "#BBBBBB" => {},
            _ => panic!("Token value was not overwritten as expected."),
        }
    }
    
    #[test]
    fn test_validate_opacity_ok() {
        let token = RawToken {
            id: TokenIdentifier::new("opacity-ok"),
            value: TokenValue::Opacity("0.5".to_string()),
            description: None, group: None,
        };
        assert!(validate_raw_token_value(&token).is_ok());
        let token_percent = RawToken {
            id: TokenIdentifier::new("opacity-percent-ok"),
            value: TokenValue::Opacity("75%".to_string()),
            description: None, group: None,
        };
        assert!(validate_raw_token_value(&token_percent).is_ok());
    }

    #[test]
    fn test_validate_opacity_invalid_range() {
        let token = RawToken {
            id: TokenIdentifier::new("opacity-high"),
            value: TokenValue::Opacity("1.1".to_string()),
            description: None, group: None,
        };
        match validate_raw_token_value(&token) {
            Err(ThemingError::InvalidTokenValue { token_id, message }) => {
                assert_eq!(token_id.as_str(), "opacity-high");
                assert!(message.contains("muss zwischen 0.0 und 1.0"));
            }
            _ => panic!("Expected InvalidTokenValue error for opacity out of range."),
        }
        let token_percent_high = RawToken {
            id: TokenIdentifier::new("opacity-percent-high"),
            value: TokenValue::Opacity("150%".to_string()),
            description: None, group: None,
        };
         match validate_raw_token_value(&token_percent_high) {
            Err(ThemingError::InvalidTokenValue { .. }) => {},
            _ => panic!("Expected InvalidTokenValue error for opacity percent out of range."),
        }
    }
    
    #[test]
    fn test_validate_opacity_reference_is_ok_for_now() {
        let token = RawToken {
            id: TokenIdentifier::new("opacity-ref"),
            value: TokenValue::Opacity("{opacity.level.medium}".to_string()),
            description: None, group: None,
        };
        assert!(validate_raw_token_value(&token).is_ok());
    }
    
    #[test]
    fn test_resolve_direct_value() {
        let base_tokens = TokenSet::new();
        let mut variant_tokens = TokenSet::new();
        let user_overrides = TokenSet::new();
        let accentable_map = HashMap::new();
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();
        let token_id = TokenIdentifier::new("color-text");
        variant_tokens.insert(token_id.clone(), RawToken {
            id: token_id.clone(),
            value: TokenValue::Color("#333333".to_string()),
            description: None, group: None,
        });
        let result = resolve_single_token_value(
            &token_id, &base_tokens, &variant_tokens, &user_overrides,
            None, &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        assert!(result.is_ok());
        match result.unwrap() {
            TokenValue::Color(value_str) => assert_eq!(value_str, "#333333"),
            _ => panic!("Resolved value is not a Color or has incorrect value."),
        }
    }

    #[tokio::test]
    async fn test_resolve_reference() {
        let mut base_tokens = TokenSet::new();
        let variant_tokens = TokenSet::new();
        let user_overrides = TokenSet::new();
        let accentable_map = HashMap::new();
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();
        let token_ref_id = TokenIdentifier::new("color-primary-ref");
        let token_target_id = TokenIdentifier::new("color-blue");
        base_tokens.insert(token_ref_id.clone(), RawToken {
            id: token_ref_id.clone(),
            value: TokenValue::Reference(token_target_id.clone()),
            description: None, group: None,
        });
        base_tokens.insert(token_target_id.clone(), RawToken {
            id: token_target_id.clone(),
            value: TokenValue::Color("#0000FF".to_string()),
            description: None, group: None,
        });
        let result = resolve_single_token_value(
            &token_ref_id, &base_tokens, &variant_tokens, &user_overrides,
            None, &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        assert!(result.is_ok());
        match result.unwrap() {
            TokenValue::Color(value_str) => assert_eq!(value_str, "#0000FF"),
            other => panic!("Resolved value is not a Color or has incorrect value: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_resolve_cycle_detection() {
        let mut base_tokens = TokenSet::new();
        let variant_tokens = TokenSet::new();
        let user_overrides = TokenSet::new();
        let accentable_map = HashMap::new();
        let mut resolved_cache = HashMap::new(); 
        let mut current_path = Vec::new();
        let token_a_id = TokenIdentifier::new("token-a");
        let token_b_id = TokenIdentifier::new("token-b");
        base_tokens.insert(token_a_id.clone(), RawToken {
            id: token_a_id.clone(),
            value: TokenValue::Reference(token_b_id.clone()),
            description: None, group: None,
        });
        base_tokens.insert(token_b_id.clone(), RawToken {
            id: token_b_id.clone(),
            value: TokenValue::Reference(token_a_id.clone()), 
            description: None, group: None,
        });
        let result = resolve_single_token_value(
            &token_a_id, &base_tokens, &variant_tokens, &user_overrides,
            None, &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::CyclicTokenReference { token_id, path } => {
                assert_eq!(token_id, token_a_id); 
                assert_eq!(path, vec![token_a_id.clone(), token_b_id.clone()]); 
            }
            e => panic!("Unexpected error type for cycle: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_resolve_max_depth() {
        let mut base_tokens = TokenSet::new();
        let variant_tokens = TokenSet::new(); 
        let user_overrides = TokenSet::new(); 
        let accentable_map = HashMap::new(); 
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();
        let mut prev_token_id = TokenIdentifier::new("token-0");
        base_tokens.insert(prev_token_id.clone(), RawToken{
            id: prev_token_id.clone(),
            value: TokenValue::Color("#FFFFFF".to_string()), 
            description: None, group: None
        });
        for i in 1..(MAX_TOKEN_RESOLUTION_DEPTH + 2) {
            let current_token_id_str = format!("token-{}", i);
            let current_token_id = TokenIdentifier::new(&current_token_id_str);
            base_tokens.insert(current_token_id.clone(), RawToken{
                id: current_token_id.clone(),
                value: TokenValue::Reference(prev_token_id.clone()),
                description: None, group: None
            });
            prev_token_id = current_token_id;
        }
        let result = resolve_single_token_value(
            &prev_token_id, 
            &base_tokens, &variant_tokens, &user_overrides,
            None, &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::MaxResolutionDepthExceeded { token_id: _, depth } => {
                assert_eq!(depth, MAX_TOKEN_RESOLUTION_DEPTH);
            }
            e => panic!("Unexpected error type for max depth: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_resolve_token_not_found() {
        let base_tokens = TokenSet::new();
        let variant_tokens = TokenSet::new();
        let user_overrides = TokenSet::new();
        let accentable_map = HashMap::new();
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();
        let token_id = TokenIdentifier::new("non-existent-token");
        let result = resolve_single_token_value(
            &token_id, &base_tokens, &variant_tokens, &user_overrides,
            None, &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::TokenNotFound { token_id: tid } => {
                assert_eq!(tid, token_id);
            }
            e => panic!("Unexpected error type for token not found: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_resolve_with_user_override() {
        let mut base_tokens = TokenSet::new();
        let mut variant_tokens = TokenSet::new();
        let mut user_overrides = TokenSet::new();
        let accentable_map = HashMap::new();
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();
        let token_id = TokenIdentifier::new("color-text");
        base_tokens.insert(token_id.clone(), RawToken::new_test("color-text", TokenValue::Color("#Base".to_string())));
        variant_tokens.insert(token_id.clone(), RawToken::new_test("color-text", TokenValue::Color("#Variant".to_string())));
        user_overrides.insert(token_id.clone(), RawToken::new_test("color-text", TokenValue::Color("#User".to_string())));
        let result = resolve_single_token_value(
            &token_id, &base_tokens, &variant_tokens, &user_overrides,
            None, &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        assert_ok_is_color(&result, "#User");
    }

    #[tokio::test]
    async fn test_resolve_with_variant_override() {
        let mut base_tokens = TokenSet::new();
        let mut variant_tokens = TokenSet::new();
        let user_overrides = TokenSet::new(); 
        let accentable_map = HashMap::new();
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();
        let token_id = TokenIdentifier::new("color-text");
        base_tokens.insert(token_id.clone(), RawToken::new_test("color-text", TokenValue::Color("#Base".to_string())));
        variant_tokens.insert(token_id.clone(), RawToken::new_test("color-text", TokenValue::Color("#Variant".to_string())));
        let result = resolve_single_token_value(
            &token_id, &base_tokens, &variant_tokens, &user_overrides,
            None, &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        assert_ok_is_color(&result, "#Variant");
    }
    
    impl RawToken {
        fn new_test(id: &str, value: TokenValue) -> Self {
            Self { id: TokenIdentifier::new(id), value, description: None, group: None }
        }
    }
    fn assert_ok_is_color(result: &Result<TokenValue, ThemingError>, expected_color: &str) {
        match result {
            Ok(TokenValue::Color(value_str)) => assert_eq!(value_str, expected_color),
            Ok(other) => panic!("Resolved value is not a Color as expected, but {:?}", other),
            Err(e) => panic!("Expected Ok result, but got error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_resolve_with_accent_color_direct_replace() {
        let mut base_tokens = TokenSet::new();
        let variant_tokens = TokenSet::new();
        let user_overrides = TokenSet::new();
        let mut accentable_map = HashMap::new();
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();
        let token_id = TokenIdentifier::new("color-accentable");
        base_tokens.insert(token_id.clone(), RawToken::new_test("color-accentable", TokenValue::Color("#Original".to_string())));
        accentable_map.insert(token_id.clone(), AccentModificationType::DirectReplace);
        let accent_color = CoreColor::from_hex("#AccentColor").unwrap();
        let result = resolve_single_token_value(
            &token_id, &base_tokens, &variant_tokens, &user_overrides,
            Some(&accent_color), &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        assert_ok_is_color(&result, &accent_color.to_hex_string());
    }

    #[tokio::test]
    // This test needs to be re-evaluated due to the logic change.
    // Accent is applied if the *original token* is accentable and *resolved value* is color.
    // If color-accentable-ref refers to color-original, and color-accentable-ref is marked accentable,
    // then the resolved color-original value will be accented.
    #[tokio::test]
    async fn test_resolve_with_accent_color_on_reference_direct_replace() {
        let mut base_tokens = TokenSet::new();
        let variant_tokens = TokenSet::new();
        let user_overrides = TokenSet::new();
        let mut accentable_map = HashMap::new();
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();

        let token_accentable_ref_id = TokenIdentifier::new("color-accentable-ref");
        let token_original_color_id = TokenIdentifier::new("color-original");

        base_tokens.insert(token_original_color_id.clone(), RawToken::new_test(
            "color-original", TokenValue::Color("#OriginalColor".to_string()) // e.g. a plain blue
        ));
        base_tokens.insert(token_accentable_ref_id.clone(), RawToken::new_test(
            "color-accentable-ref", TokenValue::Reference(token_original_color_id.clone())
        ));

        // Mark the referencing token (color-accentable-ref) as accentable
        accentable_map.insert(token_accentable_ref_id.clone(), AccentModificationType::DirectReplace);
        
        let accent_color = CoreColor::from_hex("#UserAccent").unwrap(); // e.g. a user's chosen red

        let result = resolve_single_token_value(
            &token_accentable_ref_id, 
            &base_tokens, 
            &variant_tokens, 
            &user_overrides,
            Some(&accent_color), // Apply accent
            &accentable_map, 
            &mut current_path, 
            0, 
            &mut resolved_cache
        );

        // Expectation: color-accentable-ref resolves to color-original (#OriginalColor).
        // Then, because color-accentable-ref is marked for DirectReplace, #OriginalColor is replaced by #UserAccent.
        assert_ok_is_color(&result, "#UserAccent");
    }
    
    #[tokio::test]
    async fn test_resolve_non_color_token_not_accented() {
        let mut base_tokens = TokenSet::new();
        let variant_tokens = TokenSet::new();
        let user_overrides = TokenSet::new();
        let mut accentable_map = HashMap::new();
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();
        let token_id = TokenIdentifier::new("spacing-large");
        base_tokens.insert(token_id.clone(), RawToken::new_test("spacing-large", TokenValue::Dimension("32px".to_string())));
        accentable_map.insert(token_id.clone(), AccentModificationType::DirectReplace);
        let accent_color = CoreColor::from_hex("#AccentColor").unwrap();
        let result = resolve_single_token_value(
            &token_id, &base_tokens, &variant_tokens, &user_overrides,
            Some(&accent_color), &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        assert!(result.is_ok());
        match result.unwrap() {
            TokenValue::Dimension(value_str) => assert_eq!(value_str, "32px"), 
            other => panic!("Resolved value is not a Dimension or has incorrect value: {:?}", other),
        }
    }

    #[test]
    fn test_resolved_token_value_to_string_simple_types() {
        assert_eq!(resolved_token_value_to_string(&TokenValue::Color("#ABC".to_string()), &TokenIdentifier::default()).unwrap(), "#ABC");
        assert_eq!(resolved_token_value_to_string(&TokenValue::Dimension("10px".to_string()), &TokenIdentifier::default()).unwrap(), "10px");
        assert_eq!(resolved_token_value_to_string(&TokenValue::Generic("hello".to_string()), &TokenIdentifier::default()).unwrap(), "hello");
    }

    #[test]
    fn test_resolved_token_value_to_string_typography() {
        let mut typo_map = HashMap::new();
        typo_map.insert("fontFamily".to_string(), "Arial".to_string());
        typo_map.insert("fontSize".to_string(), "12pt".to_string());
        let token_value = TokenValue::Typography(typo_map);
        let result_str = resolved_token_value_to_string(&token_value, &TokenIdentifier::default()).unwrap();
        assert!(result_str.contains("\"fontFamily\":\"Arial\""));
        assert!(result_str.contains("\"fontSize\":\"12pt\""));
    }

    #[test]
    fn test_resolved_token_value_to_string_unresolved_reference_error() {
        let token_value = TokenValue::Reference(TokenIdentifier::new("unresolved-ref"));
        let result = resolved_token_value_to_string(&token_value, &TokenIdentifier::new("test-token"));
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::InternalError(msg) => {
                assert!(msg.contains("Unerwarteter nicht aufgelöster Verweis"));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_load_theme_definition_valid() {
        let mut mock_service = MockConfigService::new();
        let file_path = "valid_theme.theme.json";
        let file_content_value = json!({
            "id": "my-cool-theme",
            "name": "My Cool Theme",
            "base_tokens": {
                "color-background": {"id": "color-background", "value": {"color": "#111"}}
            },
            "variants": [
                {
                    "applies_to_scheme": "dark",
                    "tokens": { "color-text": {"id": "color-text", "value": {"color": "#EEE"}} }
                }
            ]
        });
        mock_service.add_file(file_path, &file_content_value.to_string());
        let result = load_theme_definition_from_file(Arc::new(mock_service), file_path).await;
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result.err());
        let theme_def = result.unwrap();
        assert_eq!(theme_def.id.as_str(), "my-cool-theme");
        assert_eq!(theme_def.name, "My Cool Theme");
        assert!(theme_def.base_tokens.contains_key(&TokenIdentifier::new("color-background")));
        assert_eq!(theme_def.variants.len(), 1);
        assert_eq!(theme_def.variants[0].applies_to_scheme, ColorSchemeType::Dark);
    }

    #[tokio::test]
    async fn test_load_theme_definition_invalid_json() {
        let mut mock_service = MockConfigService::new();
        let file_path = "invalid_theme.theme.json";
        let file_content = r#"{"id": "broken-theme", "name": "Broken", "base_tokens": "not-a-map"}"#;
        mock_service.add_file(file_path, file_content);
        let result = load_theme_definition_from_file(Arc::new(mock_service), file_path).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::ThemeFileParseError { file_path: fp, source: _ } => { // Matched source
                assert_eq!(fp, file_path);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[test]
    fn test_generate_fallback_applied_state() {
        let fallback_state = generate_fallback_applied_state();
        // Updated to check for "structural-fallback-theme" based on current logic.rs
        assert_eq!(fallback_state.theme_id.as_str(), "structural-fallback-theme"); 
        assert_eq!(fallback_state.color_scheme, ColorSchemeType::Dark); 
        assert!(fallback_state.resolved_tokens.is_empty()); 
        assert!(fallback_state.active_accent_color.is_none()); 
    }

    #[test]
    fn test_resolve_tokens_for_config_basic() {
        let global_tokens = TokenSet::new(); 
        let mut base_theme_tokens = TokenSet::new();
        base_theme_tokens.insert(
            TokenIdentifier::new("color-text-base"),
            RawToken::new_test("color-text-base", TokenValue::Color("#BaseText".to_string()))
        );
        base_theme_tokens.insert(
            TokenIdentifier::new("spacing-base"),
            RawToken::new_test("spacing-base", TokenValue::Spacing("8px".to_string()))
        );
        let mut dark_variant_tokens = TokenSet::new();
        dark_variant_tokens.insert(
            TokenIdentifier::new("color-text-base"), 
            RawToken::new_test("color-text-base", TokenValue::Color("#DarkVariantText".to_string()))
        );
        dark_variant_tokens.insert(
            TokenIdentifier::new("color-background-dark"),
            RawToken::new_test("color-background-dark", TokenValue::Color("#111111".to_string()))
        );
        let theme_definition = ThemeDefinition {
            id: ThemeIdentifier::new("test-theme"),
            name: "Test Theme Def".to_string(),
            base_tokens: base_theme_tokens,
            variants: vec![
                ThemeVariantDefinition {
                    applies_to_scheme: ColorSchemeType::Dark,
                    tokens: dark_variant_tokens,
                }
            ],
            supported_accent_colors: None,
            accentable_tokens: None,
            description: None, author: None, version: None,
        };
        let config = ThemingConfiguration {
            selected_theme_id: ThemeIdentifier::new("test-theme"),
            preferred_color_scheme: ColorSchemeType::Dark,
            selected_accent_color: None,
            custom_user_token_overrides: None,
        };
        let mut pass_cache = HashMap::new();
        let result = resolve_tokens_for_config(&global_tokens, &theme_definition, &config, &mut pass_cache);
        assert!(result.is_ok(), "resolve_tokens_for_config failed: {:?}", result.err());
        let applied_state = result.unwrap();
        assert_eq!(applied_state.theme_id.as_str(), "test-theme");
        assert_eq!(applied_state.color_scheme, ColorSchemeType::Dark);
        assert_eq!(applied_state.resolved_tokens.len(), 3); 
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("color-text-base")).unwrap(),
            "#DarkVariantText" 
        );
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("spacing-base")).unwrap(),
            "8px" 
        );
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("color-background-dark")).unwrap(),
            "#111111" 
        );
    }

    #[test]
    fn test_resolve_tokens_for_config_with_user_override_and_accent() {
        let mut global_tokens = TokenSet::new();
        global_tokens.insert(
            TokenIdentifier::new("global-opacity"),
            RawToken::new_test("global-opacity", TokenValue::Opacity("0.5".to_string()))
        );
        let mut base_theme_tokens = TokenSet::new();
        base_theme_tokens.insert(
            TokenIdentifier::new("color-primary"), 
            RawToken::new_test("color-primary", TokenValue::Color("#BasePrimary".to_string()))
        );
        base_theme_tokens.insert(
            TokenIdentifier::new("font-default"),
            RawToken::new_test("font-default", TokenValue::FontFamily("Arial".to_string()))
        );
        let mut user_overrides = TokenSet::new();
        user_overrides.insert(
            TokenIdentifier::new("font-default"), 
            RawToken::new_test("font-default", TokenValue::FontFamily("Roboto".to_string()))
        );
         user_overrides.insert( 
            TokenIdentifier::new("user-spacing"),
            RawToken::new_test("user-spacing", TokenValue::Spacing("12px".to_string()))
        );
        let mut accentable_map = HashMap::new();
        accentable_map.insert(TokenIdentifier::new("color-primary"), AccentModificationType::DirectReplace);
        let theme_definition = ThemeDefinition {
            id: ThemeIdentifier::new("accent-theme"),
            name: "Accent Test Theme".to_string(),
            base_tokens: base_theme_tokens,
            variants: vec![], 
            supported_accent_colors: None,
            accentable_tokens: Some(accentable_map),
            description: None, author: None, version: None,
        };
        let accent_color_val = CoreColor::from_hex("#UserAccent").unwrap();
        let config = ThemingConfiguration {
            selected_theme_id: ThemeIdentifier::new("accent-theme"),
            preferred_color_scheme: ColorSchemeType::Light, 
            selected_accent_color: Some(accent_color_val.clone()),
            custom_user_token_overrides: Some(user_overrides),
        };
        let mut pass_cache = HashMap::new();
        let result = resolve_tokens_for_config(&global_tokens, &theme_definition, &config, &mut pass_cache);
        assert!(result.is_ok(), "resolve_tokens_for_config failed: {:?}", result.err());
        let applied_state = result.unwrap();
        assert_eq!(applied_state.resolved_tokens.len(), 4); 
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("global-opacity")).unwrap(),
            "0.5"
        );
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("color-primary")).unwrap(),
            &accent_color_val.to_hex_string() 
        );
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("font-default")).unwrap(),
            "Roboto" 
        );
         assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("user-spacing")).unwrap(),
            "12px" 
        );
    }

    #[test]
    fn test_resolve_tokens_for_config_reference_resolution() {
        let mut global_tokens = TokenSet::new();
        global_tokens.insert(
            TokenIdentifier::new("color-brand-core"),
            RawToken::new_test("color-brand-core", TokenValue::Color("#BrandCore".to_string()))
        );
        let mut base_theme_tokens = TokenSet::new();
        base_theme_tokens.insert(
            TokenIdentifier::new("color-primary-ref"),
            RawToken::new_test("color-primary-ref", TokenValue::Reference(TokenIdentifier::new("color-brand-core")))
        );
         base_theme_tokens.insert(
            TokenIdentifier::new("spacing-large"),
            RawToken::new_test("spacing-large", TokenValue::Spacing("32px".to_string()))
        );
        let theme_definition = ThemeDefinition {
            id: ThemeIdentifier::new("ref-theme"),
            name: "Ref Test Theme".to_string(),
            base_tokens: base_theme_tokens,
            variants: vec![],
            supported_accent_colors: None,
            accentable_tokens: None,
            description: None, author: None, version: None,
        };
        let config = ThemingConfiguration {
            selected_theme_id: ThemeIdentifier::new("ref-theme"),
            preferred_color_scheme: ColorSchemeType::Light,
            selected_accent_color: None,
            custom_user_token_overrides: None,
        };
        let mut pass_cache = HashMap::new();
        let result = resolve_tokens_for_config(&global_tokens, &theme_definition, &config, &mut pass_cache);
        assert!(result.is_ok(), "resolve_tokens_for_config failed: {:?}", result.err());
        let applied_state = result.unwrap();
        assert_eq!(applied_state.resolved_tokens.len(), 3); 
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("color-primary-ref")).unwrap(),
            "#BrandCore" 
        );
         assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("color-brand-core")).expect("color-brand-core missing"),
            "#BrandCore" 
        );
         assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("spacing-large")).expect("spacing-large missing"),
            "32px"
        );
    }

    // --- New tests for resolve_single_token_value ---

    #[tokio::test]
    async fn test_resolve_chained_references() {
        let mut base_tokens = TokenSet::new();
        let token_a = TokenIdentifier::new("token-a-chain");
        let token_b = TokenIdentifier::new("token-b-chain");
        let token_c = TokenIdentifier::new("token-c-chain");
        base_tokens.insert(token_a.clone(), RawToken::new_test("token-a-chain", TokenValue::Reference(token_b.clone())));
        base_tokens.insert(token_b.clone(), RawToken::new_test("token-b-chain", TokenValue::Reference(token_c.clone())));
        base_tokens.insert(token_c.clone(), RawToken::new_test("token-c-chain", TokenValue::Color("#ChainEnd".to_string())));

        let result = resolve_single_token_value(
            &token_a, &base_tokens, &TokenSet::new(), &TokenSet::new(),
            None, &HashMap::new(), &mut Vec::new(), 0, &mut HashMap::new()
        );
        assert_ok_is_color(&result, "#ChainEnd");
    }

    #[tokio::test]
    async fn test_resolve_reference_to_overridden_token() {
        let mut base_tokens = TokenSet::new();
        let mut user_overrides = TokenSet::new();
        let token_ref = TokenIdentifier::new("token-ref-override");
        let token_target = TokenIdentifier::new("token-target-override");

        base_tokens.insert(token_ref.clone(), RawToken::new_test("token-ref-override", TokenValue::Reference(token_target.clone())));
        base_tokens.insert(token_target.clone(), RawToken::new_test("token-target-override", TokenValue::Color("#BaseTarget".to_string())));
        user_overrides.insert(token_target.clone(), RawToken::new_test("token-target-override", TokenValue::Color("#UserTarget".to_string())));
        
        let result = resolve_single_token_value(
            &token_ref, &base_tokens, &TokenSet::new(), &user_overrides,
            None, &HashMap::new(), &mut Vec::new(), 0, &mut HashMap::new()
        );
        // The reference should resolve to the overridden value of token_target
        assert_ok_is_color(&result, "#UserTarget");
    }

    #[tokio::test]
    async fn test_resolve_reference_to_non_existent_token() {
        let mut base_tokens = TokenSet::new();
        let token_ref = TokenIdentifier::new("token-ref-nonexistent");
        let token_target_nonexistent = TokenIdentifier::new("token-target-nonexistent");
        base_tokens.insert(token_ref.clone(), RawToken::new_test("token-ref-nonexistent", TokenValue::Reference(token_target_nonexistent.clone())));

        let result = resolve_single_token_value(
            &token_ref, &base_tokens, &TokenSet::new(), &TokenSet::new(),
            None, &HashMap::new(), &mut Vec::new(), 0, &mut HashMap::new()
        );
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::TokenNotFound { token_id } => {
                assert_eq!(token_id, token_target_nonexistent);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_resolve_self_reference_cycle() {
        let mut base_tokens = TokenSet::new();
        let token_a = TokenIdentifier::new("token-a-self-cycle");
        base_tokens.insert(token_a.clone(), RawToken::new_test("token-a-self-cycle", TokenValue::Reference(token_a.clone())));

        let result = resolve_single_token_value(
            &token_a, &base_tokens, &TokenSet::new(), &TokenSet::new(),
            None, &HashMap::new(), &mut Vec::new(), 0, &mut HashMap::new()
        );
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::CyclicTokenReference { token_id, path } => {
                assert_eq!(token_id, token_a);
                assert_eq!(path, vec![token_a.clone()]);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
     #[tokio::test]
    async fn test_resolve_error_propagation_from_reference() {
        let mut base_tokens = TokenSet::new();
        let token_a = TokenIdentifier::new("token-a-err-prop");
        let token_b = TokenIdentifier::new("token-b-err-prop"); // This will be missing
        base_tokens.insert(token_a.clone(), RawToken::new_test("token-a-err-prop", TokenValue::Reference(token_b.clone())));

        let result = resolve_single_token_value(
            &token_a, &base_tokens, &TokenSet::new(), &TokenSet::new(),
            None, &HashMap::new(), &mut Vec::new(), 0, &mut HashMap::new()
        );
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::TokenNotFound { token_id } => {
                assert_eq!(token_id, token_b); // Error should be for the missing token B
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    // --- Tests for apply_accent_to_color ---

    #[test]
    fn test_apply_accent_direct_replace() {
        let original = CoreColor::from_hex("#FF0000").unwrap(); // Red
        let accent = CoreColor::from_hex("#00FF00").unwrap();   // Green
        let result = apply_accent_to_color(&original, &accent, &AccentModificationType::DirectReplace).unwrap();
        assert_eq!(result, accent);
    }

    #[test]
    fn test_apply_accent_lighten_mix_logic() { // Renamed to reflect mix logic
        let original_color = CoreColor::from_hex("#FF0000").unwrap(); // Red (1,0,0)
        let accent_color = CoreColor::from_hex("#00FF00").unwrap();   // Green (0,1,0)

        // Factor 0.0: should be original_color
        // original.mix(accent, 0.0) = original * 1.0 + accent * 0.0 = original
        let result_f0 = apply_accent_to_color(&original_color, &accent_color, &AccentModificationType::Lighten(0.0)).unwrap();
        assert_eq!(result_f0.to_rgba8(), original_color.to_rgba8());

        // Factor 0.5: 50/50 mix of original and accent
        // R: 1*0.5 + 0*0.5 = 0.5
        // G: 0*0.5 + 1*0.5 = 0.5
        // B: 0*0.5 + 0*0.5 = 0.0
        // Expected: #808000 (approx, if non-alpha hex)
        let expected_f05 = CoreColor::new(0.5, 0.5, 0.0, 1.0); // Assuming mix result
        let result_f05 = apply_accent_to_color(&original_color, &accent_color, &AccentModificationType::Lighten(0.5)).unwrap();
        assert_eq!(result_f05.to_rgba8(), expected_f05.to_rgba8());

        // Factor 1.0: should be accent_color
        // original.mix(accent, 1.0) = original * 0.0 + accent * 1.0 = accent
        let result_f1 = apply_accent_to_color(&original_color, &accent_color, &AccentModificationType::Lighten(1.0)).unwrap();
        assert_eq!(result_f1.to_rgba8(), accent_color.to_rgba8());

        // Example with different colors: Blue (#0000FF) lightened with Yellow (#FFFF00) by 0.25
        // Original: Blue (0,0,1), Accent: Yellow (1,1,0)
        // Factor: 0.25 (25% Yellow, 75% Blue)
        // R: 0*0.75 + 1*0.25 = 0.25
        // G: 0*0.75 + 1*0.25 = 0.25
        // B: 1*0.75 + 0*0.25 = 0.75
        // Expected: #4040BF
        let blue = CoreColor::from_hex("#0000FF").unwrap();
        let yellow = CoreColor::from_hex("#FFFF00").unwrap();
        let expected_blue_yellow_mix = CoreColor::new(0.25, 0.25, 0.75, 1.0);
        let result_blue_yellow_mix = apply_accent_to_color(&blue, &yellow, &AccentModificationType::Lighten(0.25)).unwrap();
        assert_eq!(result_blue_yellow_mix.to_rgba8(), expected_blue_yellow_mix.to_rgba8());
    }

    #[test]
    fn test_apply_accent_darken_mix_logic() { // Renamed to reflect mix logic
        let original_color = CoreColor::from_hex("#00FF00").unwrap(); // Green (0,1,0)
        let accent_color = CoreColor::from_hex("#FF0000").unwrap();   // Red (1,0,0) - "darkening" effect depends on this choice

        // Factor 0.0: should be original_color
        let result_f0 = apply_accent_to_color(&original_color, &accent_color, &AccentModificationType::Darken(0.0)).unwrap();
        assert_eq!(result_f0.to_rgba8(), original_color.to_rgba8());

        // Factor 0.75: 75% accent_color, 25% original_color
        // R: 0*0.25 + 1*0.75 = 0.75
        // G: 1*0.25 + 0*0.75 = 0.25
        // B: 0*0.25 + 0*0.75 = 0.0
        // Expected: #BF4000
        let expected_f075 = CoreColor::new(0.75, 0.25, 0.0, 1.0);
        let result_f075 = apply_accent_to_color(&original_color, &accent_color, &AccentModificationType::Darken(0.75)).unwrap();
        assert_eq!(result_f075.to_rgba8(), expected_f075.to_rgba8());

        // Factor 1.0: should be accent_color
        let result_f1 = apply_accent_to_color(&original_color, &accent_color, &AccentModificationType::Darken(1.0)).unwrap();
        assert_eq!(result_f1.to_rgba8(), accent_color.to_rgba8());
    }

    #[test]
    fn test_apply_accent_tint_with_original() {
        let original = CoreColor::from_hex("#FF0000").unwrap(); // Red (1,0,0,1)
        let accent = CoreColor::from_hex("#0000FF").unwrap();   // Blue (0,0,1,1)

        // Factor 0.0: original color
        let result_f0 = apply_accent_to_color(&original, &accent, &AccentModificationType::TintWithOriginal(0.0)).unwrap();
        assert_eq!(result_f0.to_rgba8(), original.to_rgba8());

        // Factor 0.5: 50/50 mix
        // R: 1*0.5 + 0*0.5 = 0.5
        // G: 0*0.5 + 0*0.5 = 0.0
        // B: 0*0.5 + 1*0.5 = 0.5
        // A: 1*0.5 + 1*0.5 = 1.0 (Alpha interpolation: self.a + (other.a - self.a) * t)
        // So, Color::new(0.5, 0.0, 0.5, 1.0) which is #800080
        let result_f05 = apply_accent_to_color(&original, &accent, &AccentModificationType::TintWithOriginal(0.5)).unwrap();
        assert_eq!(result_f05.to_rgba8(), CoreColor::new(0.5, 0.0, 0.5, 1.0).to_rgba8());
        assert_eq!(result_f05.to_hex_string(false), "#800080");


        // Factor 1.0: accent color
        let result_f1 = apply_accent_to_color(&original, &accent, &AccentModificationType::TintWithOriginal(1.0)).unwrap();
        assert_eq!(result_f1.to_rgba8(), accent.to_rgba8());
        
        // Test with different alphas
        let original_alpha = CoreColor::new(1.0, 0.0, 0.0, 0.8); // Red 80%
        let accent_alpha = CoreColor::new(0.0, 0.0, 1.0, 0.4);   // Blue 40%
        // Factor 0.5:
        // R: 0.5, G: 0.0, B: 0.5 (as above)
        // A: 0.8 * 0.5 + 0.4 * 0.5 = 0.4 + 0.2 = 0.6
        // CoreColor::interpolate alpha: self.a + (other.a - self.a) * t = 0.8 + (0.4 - 0.8) * 0.5 = 0.8 + (-0.4 * 0.5) = 0.8 - 0.2 = 0.6
        let result_alpha_f05 = apply_accent_to_color(&original_alpha, &accent_alpha, &AccentModificationType::TintWithOriginal(0.5)).unwrap();
        assert_eq!(result_alpha_f05.to_rgba8(), CoreColor::new(0.5, 0.0, 0.5, 0.6).to_rgba8());
    }

    #[test]
    fn test_apply_accent_invalid_factor() {
        let original = CoreColor::RED;
        let accent = CoreColor::BLUE;
        assert!(apply_accent_to_color(&original, &accent, &AccentModificationType::Lighten(-0.1)).is_err());
        assert!(apply_accent_to_color(&original, &accent, &AccentModificationType::Lighten(1.1)).is_err());
        assert!(apply_accent_to_color(&original, &accent, &AccentModificationType::Darken(-0.1)).is_err());
        assert!(apply_accent_to_color(&original, &accent, &AccentModificationType::Darken(1.1)).is_err());
        assert!(apply_accent_to_color(&original, &accent, &AccentModificationType::TintWithOriginal(-0.1)).is_err());
        assert!(apply_accent_to_color(&original, &accent, &AccentModificationType::TintWithOriginal(1.1)).is_err());
    }
}
