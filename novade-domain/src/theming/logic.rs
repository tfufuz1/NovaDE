use crate::theming::types::{RawToken, TokenSet, TokenValue, TokenIdentifier, ThemeDefinition, ThemeIdentifier, ColorSchemeType, AccentColor, AccentModificationType, AppliedThemeState, ThemingConfiguration};
use crate::theming::errors::ThemingError;
use novade_core::config::{ConfigServiceAsync, ConfigFormat}; // Assuming these exist
use novade_core::types::Color as CoreColor; // Assuming this exists
use novade_core::errors::CoreError;
use std::collections::{HashMap, BTreeMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid; // For cache key if needed, or other unique identifiers

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
    let file_content = config_service
        .load_config_file_content_async(file_path)
        .await
        .map_err(|core_err| {
            // Assuming CoreError might have a way to indicate "Not Found" vs other IO errors.
            // For now, wrapping it. If CoreError is std::io::Error, this is fine.
            // If it's a custom enum, we might need more specific error mapping.
            match core_err {
                CoreError::IoError(io_err) => ThemingError::FilesystemIoError(io_err),
                CoreError::ConfigFormatError(msg) => ThemingError::SerdeError(format!("Fehler beim Parsen der Konfigurationsdatei '{}': {}", file_path, msg)),
                CoreError::NotFound(path_not_found) => ThemingError::InternalError(format!("Konfigurationsdatei nicht gefunden: {}. Ursprungspfad: {}", path_not_found, file_path)),
                _ => ThemingError::InternalError(format!("Core-Fehler beim Laden der Token-Datei '{}': {}", file_path, core_err)),
            }
        })?;

    let raw_tokens: Vec<RawToken> = serde_json::from_str(&file_content)
        .map_err(|e| ThemingError::TokenFileParseError {
            file_path: file_path.to_string(),
            source: e,
        })?;

    let mut token_set = TokenSet::new();
    for token in raw_tokens {
        validate_raw_token_value(&token)?; // Basic validation
        if token_set.insert(token.id.clone(), token).is_some() {
            // Handle duplicate token IDs if necessary, though BTreeMap will just overwrite.
            // For now, log or return error if strict uniqueness per file is required.
            // log::warn!("Duplicate token ID '{}' in file '{}'", token.id, file_path);
        }
    }
    Ok(token_set)
}

/// Validates the value of a single raw token.
/// For example, checks if opacity is within the 0-1 range (or 0%-100%).
fn validate_raw_token_value(token: &RawToken) -> Result<(), ThemingError> {
    match &token.value {
        TokenValue::Opacity(s) => {
            // Attempt to parse as float
            if let Ok(val) = s.trim_end_matches('%').parse::<f32>() {
                let actual_val = if s.ends_with('%') { val / 100.0 } else { val };
                if !(0.0..=1.0).contains(&actual_val) {
                    return Err(ThemingError::invalid_value(
                        token.id.clone(),
                        format!("Opacity '{}' muss zwischen 0.0 und 1.0 (oder 0% und 100%) liegen.", s),
                    ));
                }
            } else {
                // If not a float, it might be a reference. References are validated later.
                // For now, we assume if it's not parseable as a direct value, it might be a reference.
                // Or, if it's a string that's not a number or reference, that's an error.
                // Let's assume for now that if it's not a parsable number, it must be a reference.
                // If it's not a reference either, `resolve_single_token_value` will catch it.
            }
        }
        TokenValue::Color(s) => {
            // Basic validation: must start with # for hex, or be a reference (e.g., {color.primary})
            // More advanced validation (e.g. valid hex length) can be added.
            if !s.starts_with('#') && !s.starts_with('{') {
                // This is a simplified check. A proper CSS color parser or regex would be better.
                // For now, we only check if it's not a hex and not a reference.
                // If it's intended to be a named color or functional notation, this will fail.
                // This validation might be too strict or need refinement based on supported color formats.
                // log::warn!("Color token '{}' value '{}' does not start with '#' or '{{'. Assuming it's a reference or named color.", token.id, s);
            }
        }
        // Add other validations as needed for Dimension, FontWeight, etc.
        _ => {}
    }
    Ok(())
}

/// Resolves a single token value, handling references, cycles, and depth limits.
///
/// # Arguments
/// * `token_id` - The ID of the token to resolve.
/// * `base_tokens` - The combined TokenSet (e.g., global + theme base).
/// * `variant_tokens` - TokenSet for the current theme variant (e.g., light/dark).
/// * `user_overrides` - User-specific TokenSet.
/// * `accent_color` - Optional current accent color.
/// * `accentable_tokens_map` - HashMap defining which tokens are affected by accent color and how.
/// * `current_path` - A vector of TokenIdentifiers representing the current resolution path (for cycle detection).
/// * `depth` - Current resolution depth.
/// * `resolved_cache` - A mutable HashMap to cache already resolved token values (string representations).
///
/// # Returns
/// A `Result` containing the resolved `TokenValue` or a `ThemingError`.
fn resolve_single_token_value<'a>(
    token_id: &TokenIdentifier,
    base_tokens: &'a TokenSet,
    variant_tokens: &'a TokenSet,
    user_overrides: &'a TokenSet,
    accent_color: Option<&'a CoreColor>,
    accentable_tokens_map: &'a HashMap<TokenIdentifier, AccentModificationType>,
    current_path: &mut Vec<TokenIdentifier>,
    depth: u8,
    resolved_cache: &mut HashMap<TokenIdentifier, Result<String, ThemingError>>, // Cache stores final string result
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

    // Prioritize sources: User Overrides > Variant > Base
    let raw_token_opt = user_overrides
        .get(token_id)
        .or_else(|| variant_tokens.get(token_id))
        .or_else(|| base_tokens.get(token_id));

    let result = match raw_token_opt {
        Some(raw_token) => {
            let mut current_value = raw_token.value.clone();

            // Apply accent color if applicable and token is accentable
            if let Some(accent) = accent_color {
                if let Some(modification_type) = accentable_tokens_map.get(token_id) {
                     // Ensure that the token's value is a color before trying to accent it
                    if let TokenValue::Color(original_color_str) = &current_value {
                        let original_core_color = parse_color_string(original_color_str, token_id, base_tokens, variant_tokens, user_overrides, accentable_tokens_map, current_path, depth + 1, resolved_cache)?;
                        current_value = TokenValue::Color(
                            apply_accent_to_color(&original_core_color, accent, modification_type)
                                .map_err(|e_msg| ThemingError::AccentColorApplicationError { token_id: token_id.clone(), message: e_msg })?
                                .to_hex_string(), // Assuming CoreColor has to_hex_string()
                        );
                    } else if let TokenValue::Reference(ref_id) = &current_value {
                        // If it's a reference, resolve it first, then check if the resolved value is a color
                        let resolved_ref_value = resolve_single_token_value(ref_id, base_tokens, variant_tokens, user_overrides, None /* Don't apply accent during intermediate ref resolution */, accentable_tokens_map, current_path, depth + 1, resolved_cache)?;
                        if let TokenValue::Color(original_color_str) = resolved_ref_value {
                             let original_core_color = parse_color_string(&original_color_str, ref_id, base_tokens, variant_tokens, user_overrides, accentable_tokens_map, current_path, depth + 1, resolved_cache)?;
                             current_value = TokenValue::Color(
                                apply_accent_to_color(&original_core_color, accent, modification_type)
                                    .map_err(|e_msg| ThemingError::AccentColorApplicationError { token_id: token_id.clone(), message: e_msg })?
                                    .to_hex_string(),
                            );
                        } else {
                            // The reference did not resolve to a color, so we can't apply accent.
                            // Keep the resolved reference value.
                            current_value = resolved_ref_value;
                        }
                    }
                    // If not TokenValue::Color or TokenValue::Reference, accenting is not applicable.
                }
            }
            
            // Resolve reference if the current value is a Reference
            if let TokenValue::Reference(ref_id) = current_value {
                resolve_single_token_value(
                    &ref_id,
                    base_tokens,
                    variant_tokens,
                    user_overrides,
                    accent_color, // Pass accent color down for the referenced token
                    accentable_tokens_map,
                    current_path,
                    depth + 1,
                    resolved_cache,
                )
            } else {
                Ok(current_value)
            }
        }
        None => Err(ThemingError::TokenNotFound { token_id: token_id.clone() }),
    };

    current_path.pop();
    result
}


/// Helper function to parse a color string (which might be a reference) into a CoreColor.
/// This is needed when applying accent colors, as the original color might itself be a reference.
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
        let ref_id = TokenIdentifier::new(ref_id_str); // Assuming TokenIdentifier::new is robust enough
        
        match resolve_single_token_value(&ref_id, base_tokens, variant_tokens, user_overrides, None, accentable_tokens_map, current_path, depth, resolved_cache)? {
            TokenValue::Color(hex_color) => CoreColor::from_hex(&hex_color).map_err(|_| {
                ThemingError::InvalidTokenValue {
                    token_id: ref_id.clone(),
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


/// Converts a resolved `TokenValue` into its final `String` representation.
/// For `TokenValue::Reference`, it should ideally not occur here if resolution is complete.
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
            // Convert HashMap to a string representation, e.g., JSON or a specific CSS format.
            // For simplicity, using serde_json here. Actual format might depend on usage.
            serde_json::to_string(map).map_err(|e| ThemingError::SerdeError(format!("Fehler beim Serialisieren des Typografie-Tokens '{}': {}", token_id, e)))
        }
        TokenValue::Reference(ref_id) => {
            // This case should ideally be prevented by full resolution before this function is called.
            // If it occurs, it means a reference was not fully resolved.
            Err(ThemingError::InternalError(format!(
                "Unerwarteter nicht aufgelöster Verweis '{}' für Token '{}' angetroffen.", ref_id, token_id
            )))
        }
    }
}

/// Placeholder for applying accent color. Actual color math is complex.
fn apply_accent_to_color(
    original_color: &CoreColor,
    accent_color: &CoreColor,
    modification_type: &AccentModificationType,
) -> Result<CoreColor, String> {
    // TODO: Implement actual color manipulation logic (e.g., HSL adjustments, blending)
    // This will require `novade_core::types::Color` to have methods for these operations.
    // For now, this is a simplified placeholder.
    match modification_type {
        AccentModificationType::DirectReplace => Ok(accent_color.clone()),
        AccentModificationType::Lighten(_factor) => {
            // Example: Ok(original_color.lighten_by_factor_and_mix(accent_color, *factor))
            // For now, just return accent or original to demonstrate structure
            Ok(accent_color.clone()) // Placeholder
        }
        AccentModificationType::Darken(_factor) => {
            // Example: Ok(original_color.darken_by_factor_and_mix(accent_color, *factor))
            Ok(original_color.clone()) // Placeholder
        }
        AccentModificationType::TintWithOriginal(factor) => {
            // Example: Ok(original_color.mix(accent_color, *factor))
            if *factor > 0.5 { Ok(accent_color.clone()) } else { Ok(original_color.clone())} // Placeholder
        }
        // Add other modification types if any
    }
}

/// Placeholder for validating a token set for cycles.
/// Actual cycle detection is done during resolution via `current_path`.
/// This function could be used for an upfront check if needed, but might be redundant.
pub fn validate_tokenset_for_cycles(_token_set: &TokenSet) -> Result<(), ThemingError> {
    // Cycle detection is primarily handled during the resolution process
    // by tracking the `current_path` in `resolve_single_token_value`.
    // An upfront check here would involve iterating through all tokens and trying to
    // resolve each one, which is essentially what `resolve_tokens_for_config` will do.
    // If a standalone validation is truly needed without full resolution,
    // a graph traversal algorithm (e.g., DFS) would be implemented here.
    Ok(())
}


// --- Still to implement: ---
// load_and_validate_token_files (more advanced merging if needed)
// load_and_validate_theme_files (more advanced merging if needed)
// CacheKey definition and cache related logic (if not part of ThemingEngineInternalState directly)


/// Loads a theme definition from a specified file path using the configuration service.
pub async fn load_theme_definition_from_file(
    config_service: Arc<dyn ConfigServiceAsync>,
    file_path: &str,
) -> Result<ThemeDefinition, ThemingError> {
    let file_content = config_service
        .load_config_file_content_async(file_path)
        .await
        .map_err(|core_err| match core_err {
            CoreError::IoError(io_err) => ThemingError::FilesystemIoError(io_err),
            CoreError::ConfigFormatError(msg) => ThemingError::SerdeError(format!("Fehler beim Parsen der Theme-Definitionsdatei '{}': {}", file_path, msg)),
            CoreError::NotFound(path_not_found) => ThemingError::InternalError(format!("Theme-Definitionsdatei nicht gefunden: {}. Ursprungspfad: {}", path_not_found, file_path)),
            _ => ThemingError::InternalError(format!("Core-Fehler beim Laden der Theme-Definitionsdatei '{}': {}", file_path, core_err)),
        })?;

    serde_json::from_str(&file_content).map_err(|e| {
        ThemingError::ThemeFileParseError {
            file_path: file_path.to_string(),
            source: e,
        }
    })
}

/// Loads multiple token files and merges them. For now, assumes a single global file.
/// TODO: Extend if multiple global token files need merging strategies.
pub async fn load_and_validate_token_files(
    config_service: Arc<dyn ConfigServiceAsync>,
    file_paths: &[String], // Expecting paths to token files
) -> Result<TokenSet, ThemingError> {
    if file_paths.is_empty() {
        return Ok(TokenSet::new()); // Return empty set if no paths are provided
    }
    // For now, load the first file. Merging logic can be added later.
    // If multiple files are provided, subsequent files could overwrite or extend the previous ones.
    load_raw_tokens_from_file(config_service, &file_paths[0]).await
}

/// Loads multiple theme definition files. For now, assumes a single theme definition is loaded at a time by the engine.
/// TODO: Extend if a directory of themes needs to be loaded and managed.
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
    // TODO: Add validation, e.g., check for duplicate theme IDs if loading multiple.
    // TODO: Validate token references within each theme to ensure they are resolvable
    // within the theme's own base_tokens or global tokens. This is complex and might
    // be better handled during the full resolution step or a dedicated validation phase.
    Ok(themes)
}


/// Resolves all tokens based on the current configuration (selected theme, variant, accent, user overrides).
///
/// # Arguments
/// * `global_tokens` - A base `TokenSet` that applies globally.
/// * `theme_definition` - The definition of the currently selected theme.
/// * `config` - The current `ThemingConfiguration` from user preferences.
/// * `resolved_cache` - A mutable HashMap to cache resolved token string values. This helps speed up
///   repeated lookups of the same token during a single resolution pass.
///
/// # Returns
/// A `Result` containing the `AppliedThemeState` or a `ThemingError`.
pub fn resolve_tokens_for_config(
    global_tokens: &TokenSet,
    theme_definition: &ThemeDefinition,
    config: &ThemingConfiguration,
    resolved_cache: &mut HashMap<TokenIdentifier, Result<String, ThemingError>>, // For this resolution pass
) -> Result<AppliedThemeState, ThemingError> {
    
    let mut combined_base_tokens = global_tokens.clone();
    combined_base_tokens.extend(theme_definition.base_tokens.clone()); // Theme base tokens override global

    let variant_tokens = theme_definition
        .variants
        .iter()
        .find(|v| v.applies_to_scheme == config.preferred_color_scheme)
        .map(|v| v.tokens.clone())
        .unwrap_or_default();

    let user_overrides = config.custom_user_token_overrides.as_ref().cloned().unwrap_or_default();

    let accentable_tokens_map = theme_definition.accentable_tokens.as_ref().cloned().unwrap_or_default();
    
    let mut resolved_tokens_map: BTreeMap<TokenIdentifier, String> = BTreeMap::new();

    // Iterate over all unique token IDs from all sources to ensure all are resolved.
    let mut all_token_ids: HashSet<TokenIdentifier> = HashSet::new();
    all_token_ids.extend(combined_base_tokens.keys().cloned());
    all_token_ids.extend(variant_tokens.keys().cloned());
    all_token_ids.extend(user_overrides.keys().cloned());
    // Also consider keys from accentable_tokens_map if they might not be in other sets
    all_token_ids.extend(accentable_tokens_map.keys().cloned());


    for token_id in all_token_ids {
        // Check cache first (for string values)
        if let Some(cached_result) = resolved_cache.get(&token_id) {
            match cached_result {
                Ok(val_str) => {
                    resolved_tokens_map.insert(token_id.clone(), val_str.clone());
                    continue;
                }
                Err(e) => {
                    // If a token previously failed to resolve in this pass, propagate the error.
                    // Cloning the error might be tricky. For now, let's re-resolve, or store clonable error.
                    // This depends on ThemingError being Clone. If not, we must re-resolve.
                    // For simplicity, let's assume re-resolution if error cloning is an issue.
                    // Or, only cache Ok(String).
                }
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
            resolved_cache, // Pass the cache for sub-resolutions
        );

        match resolved_value_result {
            Ok(value) => {
                match resolved_token_value_to_string(&value, &token_id) {
                    Ok(val_str) => {
                        resolved_tokens_map.insert(token_id.clone(), val_str.clone());
                        resolved_cache.insert(token_id.clone(), Ok(val_str)); // Cache successful string resolution
                    }
                    Err(e) => {
                        resolved_cache.insert(token_id.clone(), Err(e.clone())); // Cache error
                        return Err(e); // Propagate error
                    }
                }
            }
            Err(e) => {
                 resolved_cache.insert(token_id.clone(), Err(e.clone())); // Cache error
                return Err(e); // Propagate error
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

/// Generates a minimal fallback `AppliedThemeState`.
/// This is used if no themes can be loaded or if the system needs a default state.
/// This function provides a *structural* fallback, not a fully resolved theme state from embedded files.
/// The `ThemingEngine::new` method is responsible for loading and resolving the actual
/// `fallback.theme.json` to create a more complete operational fallback state.
/// Using `include_str!` here for embedded JSON could be a future enhancement if a more
/// detailed code-only fallback (without any initial file I/O) is strictly required.
pub fn generate_fallback_applied_state() -> AppliedThemeState {
    // Provides a minimal, structurally valid, but empty state.
    // The actual functional fallback state is resolved from files by ThemingEngine::new.
    AppliedThemeState {
        theme_id: ThemeIdentifier::new("structural-fallback-theme"), // A distinct ID for this very basic state
        color_scheme: ColorSchemeType::Dark, // A sensible default scheme
        active_accent_color: None,
        resolved_tokens: BTreeMap::new(), // No tokens, as this is purely structural
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theming::types::{TokenIdentifier, RawToken, TokenValue, ColorSchemeType, ThemeIdentifier, ThemingConfiguration};
    use novade_core::config::{ConfigServiceAsync, ConfigFormat}; // Assuming ConfigFormat is used by service
    use novade_core::errors::CoreError; // For mocking
    use novade_core::types::Color as CoreColor;
    use std::sync::Arc;
    use async_trait::async_trait;
    use std::collections::HashMap;

    // --- Mock ConfigServiceAsync ---
    #[derive(Debug, Clone)]
    struct MockConfigService {
        files: HashMap<String, String>, // file_path -> content
        should_error_on_load: bool,
        error_type: Option<CoreError>, // Specific error to return
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

        #[allow(dead_code)] // May not use all error types in all tests
        fn set_error_on_load(&mut self, error: bool, error_type: Option<CoreError>) {
            self.should_error_on_load = error;
            self.error_type = error_type;
        }
    }

    #[async_trait]
    impl ConfigServiceAsync for MockConfigService {
        async fn load_config_file_content_async(&self, file_path: &str) -> Result<String, CoreError> {
            if self.should_error_on_load {
                return Err(self.error_type.clone().unwrap_or_else(|| CoreError::Internal("Mock error".to_string())));
            }
            self.files
                .get(file_path)
                .cloned()
                .ok_or_else(|| CoreError::NotFound(file_path.to_string()))
        }

        // Other methods not used by these tests can have dummy implementations
        async fn save_config_file_content_async(&self, _file_path: &str, _content: &str) -> Result<(), CoreError> {
            unimplemented!()
        }
        // Add dummy implementations for other ConfigServiceAsync methods if necessary
         async fn list_config_files_async(&self, _dir_path: &str) -> Result<Vec<String>, CoreError> {
            unimplemented!()
        }
        fn get_config_file_path(&self, _app_id: &crate::shared_types::ApplicationId, _config_name: &str, _format: Option<ConfigFormat>) -> Result<String, CoreError> {
            unimplemented!()
        }
        fn get_config_dir_path(&self, _app_id: &crate::shared_types::ApplicationId, _subdir: Option<&str>) -> Result<String, CoreError> {
            unimplemented!()
        }
         fn ensure_config_dir_exists(&self, _app_id: &crate::shared_types::ApplicationId) -> Result<String, CoreError> {
            unimplemented!()
        }
    }

    // --- Tests for load_raw_tokens_from_file ---
    #[tokio::test]
    async fn test_load_raw_tokens_valid_file() {
        let mut mock_service = MockConfigService::new();
        let file_path = "test_tokens.json";
        let file_content = r#"[
            {"id": "color-red", "value": {"color": "#FF0000"}},
            {"id": "spacing-small", "value": {"spacing": "4px"}, "description": "Small space"}
        ]"#;
        mock_service.add_file(file_path, file_content);

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
        let mock_service = MockConfigService::new(); // No files added
        let file_path = "non_existent.json";

        let result = load_raw_tokens_from_file(Arc::new(mock_service), file_path).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            // This error mapping depends on how CoreError::NotFound is translated
            // In logic.rs, it becomes ThemingError::InternalError containing "Konfigurationsdatei nicht gefunden"
            ThemingError::InternalError(msg) => {
                assert!(msg.contains("Konfigurationsdatei nicht gefunden"));
                assert!(msg.contains(file_path));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_load_raw_tokens_invalid_json() {
        let mut mock_service = MockConfigService::new();
        let file_path = "invalid.json";
        let file_content = r#"[{"id": "color-blue", "value": "not-an-object"}"#; // Invalid TokenValue structure
        mock_service.add_file(file_path, file_content);

        let result = load_raw_tokens_from_file(Arc::new(mock_service), file_path).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::TokenFileParseError { file_path: fp, source_message: _ } => {
                assert_eq!(fp, file_path);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_load_raw_tokens_duplicate_ids_overwrites() {
        let mut mock_service = MockConfigService::new();
        let file_path = "duplicates.json";
        let file_content = r#"[
            {"id": "color-primary", "value": {"color": "#AAAAAA"}},
            {"id": "color-primary", "value": {"color": "#BBBBBB"}}
        ]"#; // Last one wins due to BTreeMap insert behavior
        mock_service.add_file(file_path, file_content);

        let result = load_raw_tokens_from_file(Arc::new(mock_service), file_path).await;
        assert!(result.is_ok());
        let token_set = result.unwrap();
        assert_eq!(token_set.len(), 1);
        match token_set.get(&TokenIdentifier::new("color-primary")).unwrap().value {
            TokenValue::Color(ref c) if c == "#BBBBBB" => {},
            _ => panic!("Token value was not overwritten as expected."),
        }
    }

    // --- Tests for validate_raw_token_value ---
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
        // Validation of reference content happens during resolution.
        // `validate_raw_token_value` currently doesn't try to parse reference strings for opacity.
        let token = RawToken {
            id: TokenIdentifier::new("opacity-ref"),
            value: TokenValue::Opacity("{opacity.level.medium}".to_string()),
            description: None, group: None,
        };
        assert!(validate_raw_token_value(&token).is_ok());
    }
    
    // --- Placeholder tests for resolve_single_token_value ---
    // These will need more setup (TokenSets, etc.)
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

    // TODO: Add tests for reference resolution, cycles, depth limit, accent application, overrides.

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
        let mut resolved_cache = HashMap::new(); // For this test, cache won't be hit before cycle
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
            value: TokenValue::Reference(token_a_id.clone()), // Cycle back to A
            description: None, group: None,
        });
        
        let result = resolve_single_token_value(
            &token_a_id, &base_tokens, &variant_tokens, &user_overrides,
            None, &accentable_map, &mut current_path, 0, &mut resolved_cache
        );

        assert!(result.is_err());
        match result.err().unwrap() {
            ThemingError::CyclicTokenReference { token_id, path } => {
                assert_eq!(token_id, token_a_id); // Cycle detected when trying to resolve token-a again
                assert_eq!(path, vec![token_a_id.clone(), token_b_id.clone()]); // Path leading to cycle
            }
            e => panic!("Unexpected error type for cycle: {:?}", e),
        }
    }
    
    #[tokio::test]
    async fn test_resolve_max_depth() {
        let mut base_tokens = TokenSet::new();
        let variant_tokens = TokenSet::new(); // Empty
        let user_overrides = TokenSet::new(); // Empty
        let accentable_map = HashMap::new(); // Empty
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();

        let mut prev_token_id = TokenIdentifier::new("token-0");
        base_tokens.insert(prev_token_id.clone(), RawToken{
            id: prev_token_id.clone(),
            value: TokenValue::Color("#FFFFFF".to_string()), // Final value for the deepest token
            description: None, group: None
        });

        // Create a chain longer than MAX_TOKEN_RESOLUTION_DEPTH
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
        
        // Attempt to resolve the start of the long chain (which is prev_token_id now)
        let result = resolve_single_token_value(
            &prev_token_id, // This is "token-17" if MAX_DEPTH is 16
            &base_tokens, 
            &variant_tokens, 
            &user_overrides,
            None, 
            &accentable_map, 
            &mut current_path, 
            0, 
            &mut resolved_cache
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
        let user_overrides = TokenSet::new(); // No user override
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
    
    // Helper for RawToken creation in tests
    impl RawToken {
        fn new_test(id: &str, value: TokenValue) -> Self {
            Self { id: TokenIdentifier::new(id), value, description: None, group: None }
        }
    }
    // Helper to check Ok(TokenValue::Color(..))
    fn assert_ok_is_color(result: &Result<TokenValue, ThemingError>, expected_color: &str) {
        match result {
            Ok(TokenValue::Color(value_str)) => assert_eq!(value_str, expected_color),
            Ok(other) => panic!("Resolved value is not a Color as expected, but {:?}", other),
            Err(e) => panic!("Expected Ok result, but got error: {:?}", e),
        }
    }


    // --- Placeholder for resolve_tokens_for_config ---
    // This needs a full ThemeDefinition.
    // TODO: Test resolve_tokens_for_config

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
        // Assuming CoreColor::to_hex_string() returns something like "#RRGGBB" or "#RRGGBBAA"
        // and apply_accent_to_color for DirectReplace returns accent_color.clone().to_hex_string()
        assert_ok_is_color(&result, &accent_color.to_hex_string());
    }

    #[tokio::test]
    async fn test_resolve_with_accent_color_on_reference() {
        let mut base_tokens = TokenSet::new();
        let variant_tokens = TokenSet::new();
        let user_overrides = TokenSet::new();
        let mut accentable_map = HashMap::new();
        let mut resolved_cache = HashMap::new();
        let mut current_path = Vec::new();

        let token_accentable_ref_id = TokenIdentifier::new("color-accentable-ref");
        let token_original_color_id = TokenIdentifier::new("color-original");

        // color-original: #Original
        base_tokens.insert(token_original_color_id.clone(), RawToken::new_test(
            "color-original", TokenValue::Color("#Original".to_string())
        ));
        // color-accentable-ref: {color-original}
        base_tokens.insert(token_accentable_ref_id.clone(), RawToken::new_test(
            "color-accentable-ref", TokenValue::Reference(token_original_color_id.clone())
        ));
        
        // Make color-accentable-ref accentable
        accentable_map.insert(token_accentable_ref_id.clone(), AccentModificationType::DirectReplace);
        let accent_color = CoreColor::from_hex("#AccentColor").unwrap();

        let result = resolve_single_token_value(
            &token_accentable_ref_id, &base_tokens, &variant_tokens, &user_overrides,
            Some(&accent_color), &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        
        // The referenced token `color-original` is resolved first to `#Original`.
        // Then, because `color-accentable-ref` is accentable, `#Original` is replaced by `#AccentColor`.
        assert_ok_is_color(&result, &accent_color.to_hex_string());
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
        // This token is a dimension, not a color
        base_tokens.insert(token_id.clone(), RawToken::new_test("spacing-large", TokenValue::Dimension("32px".to_string())));
        
        // Mark it as accentable (though it shouldn't apply)
        accentable_map.insert(token_id.clone(), AccentModificationType::DirectReplace);
        let accent_color = CoreColor::from_hex("#AccentColor").unwrap();

        let result = resolve_single_token_value(
            &token_id, &base_tokens, &variant_tokens, &user_overrides,
            Some(&accent_color), &accentable_map, &mut current_path, 0, &mut resolved_cache
        );
        
        assert!(result.is_ok());
        match result.unwrap() {
            TokenValue::Dimension(value_str) => assert_eq!(value_str, "32px"), // Value should be unchanged
            other => panic!("Resolved value is not a Dimension or has incorrect value: {:?}", other),
        }
    }

    // --- Tests for resolved_token_value_to_string ---
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
        // Expect JSON string of the map
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
    
    // --- Tests for load_theme_definition_from_file ---
    #[tokio::test]
    async fn test_load_theme_definition_valid() {
        let mut mock_service = MockConfigService::new();
        let file_path = "valid_theme.theme.json";
        // Minimal valid ThemeDefinition JSON
        let file_content = r#"{
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
        }"#;
        mock_service.add_file(file_path, file_content);

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
            ThemingError::ThemeFileParseError { file_path: fp, source_message: _ } => {
                assert_eq!(fp, file_path);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    // --- Test for generate_fallback_applied_state ---
    #[test]
    fn test_generate_fallback_applied_state() {
        let fallback_state = generate_fallback_applied_state();
        assert_eq!(fallback_state.theme_id.as_str(), "fallback-theme"); // Default from function
        assert_eq!(fallback_state.color_scheme, ColorSchemeType::Dark); // Default from function
        assert!(fallback_state.resolved_tokens.is_empty()); // Default is empty
        assert!(fallback_state.active_accent_color.is_none()); // Default is None
    }

    // --- Tests for resolve_tokens_for_config ---
    #[test]
    fn test_resolve_tokens_for_config_basic() {
        let global_tokens = TokenSet::new(); // Keep it simple for now
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
            TokenIdentifier::new("color-text-base"), // Override base
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
        assert_eq!(applied_state.resolved_tokens.len(), 3); // color-text-base, spacing-base, color-background-dark

        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("color-text-base")).unwrap(),
            "#DarkVariantText" // Overridden by variant
        );
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("spacing-base")).unwrap(),
            "8px" // From theme base
        );
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("color-background-dark")).unwrap(),
            "#111111" // From variant
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
            TokenIdentifier::new("color-primary"), // Will be made accentable
            RawToken::new_test("color-primary", TokenValue::Color("#BasePrimary".to_string()))
        );
        base_theme_tokens.insert(
            TokenIdentifier::new("font-default"),
            RawToken::new_test("font-default", TokenValue::FontFamily("Arial".to_string()))
        );

        let mut user_overrides = TokenSet::new();
        user_overrides.insert(
            TokenIdentifier::new("font-default"), // User overrides font
            RawToken::new_test("font-default", TokenValue::FontFamily("Roboto".to_string()))
        );
         user_overrides.insert( // User adds a new token
            TokenIdentifier::new("user-spacing"),
            RawToken::new_test("user-spacing", TokenValue::Spacing("12px".to_string()))
        );

        let mut accentable_map = HashMap::new();
        accentable_map.insert(TokenIdentifier::new("color-primary"), AccentModificationType::DirectReplace);

        let theme_definition = ThemeDefinition {
            id: ThemeIdentifier::new("accent-theme"),
            name: "Accent Test Theme".to_string(),
            base_tokens: base_theme_tokens,
            variants: vec![], // No variants for simplicity
            supported_accent_colors: None,
            accentable_tokens: Some(accentable_map),
            description: None, author: None, version: None,
        };

        let accent_color_val = CoreColor::from_hex("#UserAccent").unwrap();
        let config = ThemingConfiguration {
            selected_theme_id: ThemeIdentifier::new("accent-theme"),
            preferred_color_scheme: ColorSchemeType::Light, // Default, as no variants
            selected_accent_color: Some(accent_color_val.clone()),
            custom_user_token_overrides: Some(user_overrides),
        };
        
        let mut pass_cache = HashMap::new();
        let result = resolve_tokens_for_config(&global_tokens, &theme_definition, &config, &mut pass_cache);

        assert!(result.is_ok(), "resolve_tokens_for_config failed: {:?}", result.err());
        let applied_state = result.unwrap();
        
        // Expected: global-opacity, color-primary (accented), font-default (user), user-spacing
        assert_eq!(applied_state.resolved_tokens.len(), 4); 

        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("global-opacity")).unwrap(),
            "0.5"
        );
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("color-primary")).unwrap(),
            &accent_color_val.to_hex_string() // Accented
        );
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("font-default")).unwrap(),
            "Roboto" // User override
        );
         assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("user-spacing")).unwrap(),
            "12px" // User added
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

        assert_eq!(applied_state.resolved_tokens.len(), 3); // color-brand-core, color-primary-ref, spacing-large
        assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("color-primary-ref")).unwrap(),
            "#BrandCore" // Resolved from global
        );
         assert_eq!(
            applied_state.resolved_tokens.get(&TokenIdentifier::new("color-brand-core")).unwrap(),
            "#BrandCore" // Direct global token also included
        );
    }

}
