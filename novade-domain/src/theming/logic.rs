use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde_json;
use tracing::{debug, warn, error};

use novade_core::config::ConfigServiceAsync;
use novade_core::errors::CoreError;
use novade_core::types::Color as CoreColor;

use super::errors::ThemingError;
use super::types::{
    AccentColor, AccentModificationType, AppliedThemeState, ColorSchemeType, RawToken,
    ThemeDefinition, ThemeIdentifier, ThemeVariantDefinition, ThemingConfiguration,
    TokenIdentifier, TokenSet, TokenValue,
};

pub const MAX_TOKEN_RESOLUTION_DEPTH: u8 = 16;

// Corrected paths for include_str!
const FALLBACK_THEME_JSON: &str = include_str!("default_themes/fallback.theme.json");
const BASE_TOKENS_JSON: &str = include_str!("default_themes/base.tokens.json"); // Currently empty

// --- File Loading and Validation (async) ---

/// Loads raw tokens from a single JSON file.
pub async fn load_raw_tokens_from_file(
    path: &Path,
    config_service: &Arc<dyn ConfigServiceAsync>,
) -> Result<TokenSet, ThemingError> {
    debug!("Loading raw tokens from file: {:?}", path);
    let file_content = config_service
        .read_file_to_string(path)
        .await
        .map_err(|e| ThemingError::IoError {
            message: format!("Failed to read token file: {:?}", path),
            source_error: Some(Box::new(e)),
        })?;

    let raw_tokens_vec: Vec<RawToken> = serde_json::from_str(&file_content).map_err(|e| {
        ThemingError::TokenFileParseError {
            filename: path.to_string_lossy().into_owned(),
            source_error: Box::new(e),
        }
    })?;

    let mut token_set = TokenSet::new();
    let mut seen_ids = HashSet::new();

    for token in raw_tokens_vec {
        // Check for duplicate TokenIdentifiers in the source Vec
        if !seen_ids.insert(token.id.clone()) {
            return Err(ThemingError::InvalidTokenValue { // Using InvalidTokenValue as InvalidTokenData is not defined
                token_id: token.id.clone(),
                message: format!("Duplicate TokenIdentifier '{}' found in file {:?}", token.id, path),
            });
        }

        // Validate token value
        match &token.value {
            TokenValue::Opacity(o) if !(*o >= 0.0 && *o <= 1.0) => { // Corrected condition
                return Err(ThemingError::InvalidTokenValue {
                    token_id: token.id.clone(),
                    message: format!("Opacity value {} for token '{}' is outside the valid range [0.0, 1.0]", o, token.id),
                });
            }
            _ => {}
        }
        token_set.insert(token.id.clone(), token);
    }
    Ok(token_set)
}

/// Loads and validates multiple token files, merging them.
pub async fn load_and_validate_token_files(
    paths: &[PathBuf],
    config_service: &Arc<dyn ConfigServiceAsync>,
) -> Result<TokenSet, ThemingError> {
    let mut merged_tokens = TokenSet::new();
    for path in paths {
        let tokens = load_raw_tokens_from_file(path.as_path(), config_service).await?;
        for (id, token) in tokens {
            if merged_tokens.contains_key(&id) {
                debug!("Overriding token '{}' from file {:?}", id, path);
            }
            merged_tokens.insert(id, token);
        }
    }
    validate_tokenset_for_cycles(&merged_tokens)?;
    Ok(merged_tokens)
}

/// Loads a theme definition from a single JSON file.
pub async fn load_theme_definition_from_file(
    path: &Path,
    theme_id_from_path: &ThemeIdentifier,
    config_service: &Arc<dyn ConfigServiceAsync>,
) -> Result<ThemeDefinition, ThemingError> {
    debug!("Loading theme definition from file: {:?}", path);
    let file_content = config_service
        .read_file_to_string(path)
        .await
        .map_err(|e| ThemingError::IoError {
            message: format!("Failed to read theme file: {:?}", path),
            source_error: Some(Box::new(e)),
        })?;

    let theme_def: ThemeDefinition = serde_json::from_str(&file_content).map_err(|e| {
        ThemingError::ConfigurationError {
            message: format!("Failed to parse theme definition file {:?}: {}", path, e),
        }
    })?;

    if &theme_def.id != theme_id_from_path {
        return Err(ThemingError::ConfigurationError {
            message: format!(
                "Theme ID mismatch in file {:?}. Expected '{}', found '{}'",
                path, theme_id_from_path, theme_def.id
            ),
        });
    }
    Ok(theme_def)
}

/// Loads and validates multiple theme definition files.
pub async fn load_and_validate_theme_files(
    paths: &[PathBuf], // Each path is expected to be a theme file, e.g., my-theme.theme.json
    global_tokens: &TokenSet,
    config_service: &Arc<dyn ConfigServiceAsync>,
) -> Result<Vec<ThemeDefinition>, ThemingError> {
    let mut theme_definitions = Vec::new();
    for path in paths {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let id_part = filename.split(".theme.json").next().unwrap_or("");
        if id_part.is_empty() {
            warn!("Could not determine theme ID from path: {:?}", path);
            continue;
        }
        let theme_id_from_path = ThemeIdentifier::from(id_part.to_string());

        let theme_def = load_theme_definition_from_file(path.as_path(), &theme_id_from_path, config_service).await?;
        validate_theme_definition_references(&theme_def, global_tokens)?;
        theme_definitions.push(theme_def);
    }
    Ok(theme_definitions)
}


// --- Validation Logic (sync) ---

pub fn validate_tokenset_for_cycles(tokens: &TokenSet) -> Result<(), ThemingError> {
    let mut visited = HashMap::new();
    enum VisitState { Visiting, Visited }

    for token_id in tokens.keys() {
        if !visited.contains_key(token_id) {
            let mut path = VecDeque::new(); // Using VecDeque for efficient front removal if needed, though not strictly necessary here
            detect_cycle_dfs(token_id, tokens, &mut visited, &mut path)?;
        }
    }
    Ok(())
}

fn detect_cycle_dfs<'a>(
    current_id: &'a TokenIdentifier,
    tokens: &'a TokenSet,
    visited: &mut HashMap<&'a TokenIdentifier, VisitState>,
    path: &mut VecDeque<&'a TokenIdentifier>, // Changed to VecDeque
) -> Result<(), ThemingError> {
    visited.insert(current_id, VisitState::Visiting);
    path.push_back(current_id);

    if let Some(raw_token) = tokens.get(current_id) {
        if let TokenValue::Reference(referenced_id) = &raw_token.value {
            match visited.get(referenced_id) {
                Some(VisitState::Visiting) => {
                    let mut cycle_path = path.iter().map(|&tid| tid.clone()).collect::<Vec<_>>();
                    cycle_path.push(referenced_id.clone()); // Add the cycle-completing node
                    return Err(ThemingError::CyclicTokenReference {
                        token_id: referenced_id.clone(),
                        path: cycle_path,
                    });
                }
                Some(VisitState::Visited) => {}
                None => {
                    if tokens.contains_key(referenced_id) { // Only recurse if the token exists
                        detect_cycle_dfs(referenced_id, tokens, visited, path)?;
                    }
                    // Missing references are handled by validate_theme_definition_references or during resolution
                }
            }
        }
    }

    path.pop_back();
    visited.insert(current_id, VisitState::Visited);
    Ok(())
}

pub fn validate_theme_definition_references(
    theme_def: &ThemeDefinition,
    global_tokens: &TokenSet,
) -> Result<(), ThemingError> {
    // Create a temporary combined set of all known token IDs for reference checking
    let mut known_ids: HashSet<&TokenIdentifier> = global_tokens.keys().collect();
    known_ids.extend(theme_def.base_tokens.keys());
    // For variants, they can reference global, base, or tokens within the same variant.

    let check_references_in_set = |token_set: &TokenSet, current_set_keys: &HashSet<&TokenIdentifier>, context: &str| -> Result<(), ThemingError> {
        for (id, raw_token) in token_set {
            if let TokenValue::Reference(referenced_id) = &raw_token.value {
                // A token can reference: global tokens, theme base tokens, or other tokens in its own set (e.g., within a variant)
                if !known_ids.contains(referenced_id) && !current_set_keys.contains(referenced_id) {
                    return Err(ThemingError::TokenResolutionError {
                        token_id: id.clone(),
                        reason: format!(
                            "In theme '{}', {}: Reference to undefined token '{}'",
                            theme_def.id, context, referenced_id
                        ),
                    });
                }
            }
        }
        Ok(())
    };
    
    let base_token_keys: HashSet<&TokenIdentifier> = theme_def.base_tokens.keys().collect();
    check_references_in_set(&theme_def.base_tokens, &base_token_keys, "base_tokens")?;

    for variant in &theme_def.variants {
        let variant_token_keys: HashSet<&TokenIdentifier> = variant.tokens.keys().collect();
        // When checking variant tokens, they can also reference base_tokens.
        // So, `known_ids` (global + base) and `variant_token_keys` must be checked.
        // A simpler approach for variants: create a temporary known_ids set for each variant check
        let mut variant_known_ids = known_ids.clone();
        variant_known_ids.extend(variant_token_keys.iter().cloned());

        for (id, raw_token) in &variant.tokens {
             if let TokenValue::Reference(referenced_id) = &raw_token.value {
                if !variant_known_ids.contains(referenced_id) {
                     return Err(ThemingError::TokenResolutionError {
                        token_id: id.clone(),
                        reason: format!(
                            "In theme '{}', variant for scheme {:?}: Reference to undefined token '{}'",
                            theme_def.id, variant.applies_to_scheme, referenced_id
                        ),
                    });
                }
             }
        }
    }
    Ok(())
}


// --- Token Resolution Pipeline (sync) ---

pub fn resolve_single_token_value(
    original_id: &TokenIdentifier, // The ID we are trying to resolve for the final map
    current_id_to_resolve: &TokenIdentifier, // The ID of the token currently being processed (could be a reference)
    current_value: &TokenValue,    // The value of current_id_to_resolve
    all_tokens: &BTreeMap<TokenIdentifier, TokenValue>,
    visited_path: &mut Vec<TokenIdentifier>, // Path of reference lookups
    current_depth: u8,
    max_depth: u8,
) -> Result<String, ThemingError> {
    if current_depth > max_depth {
        return Err(ThemingError::TokenResolutionError {
            token_id: original_id.clone(),
            reason: format!("Maximum reference depth ({}) exceeded. Path: {:?}", max_depth, visited_path),
        });
    }

    // Check for cycle related to current_id_to_resolve
    if visited_path.contains(current_id_to_resolve) {
        let mut cycle_path = visited_path.clone();
        cycle_path.push(current_id_to_resolve.clone());
        return Err(ThemingError::CyclicTokenReference {
            token_id: current_id_to_resolve.clone(), // The token that completed the cycle
            path: cycle_path,
        });
    }

    visited_path.push(current_id_to_resolve.clone());

    let result = match current_value {
        TokenValue::Reference(referenced_id) => {
            if let Some(next_value) = all_tokens.get(referenced_id) {
                // Resolve the referenced token, passing the original_id for error reporting
                resolve_single_token_value(original_id, referenced_id, next_value, all_tokens, visited_path, current_depth + 1, max_depth)
            } else {
                Err(ThemingError::TokenResolutionError {
                    token_id: original_id.clone(), // Error is for the original token we tried to resolve
                    reason: format!("Reference to undefined token '{}' from token '{}'", referenced_id, current_id_to_resolve),
                })
            }
        }
        TokenValue::Color(s) => Ok(s.clone()),
        TokenValue::Dimension(s) => Ok(s.clone()),
        TokenValue::FontFamily(s) => Ok(s.clone()),
        TokenValue::FontWeight(s) => Ok(s.clone()),
        TokenValue::FontSize(s) => Ok(s.clone()),
        TokenValue::LetterSpacing(s) => Ok(s.clone()),
        TokenValue::LineHeight(s) => Ok(s.clone()),
        TokenValue::Border(s) => Ok(s.clone()),
        TokenValue::Shadow(s) => Ok(s.clone()),
        TokenValue::Opacity(o) => Ok(format!("{:.prec$}", o, prec = 2)), // Ensure 2 decimal places
        TokenValue::Number(n) => Ok(n.to_string()),
        TokenValue::String(s) => Ok(s.clone()),
    };

    visited_path.pop();
    result
}


pub fn resolve_tokens_for_config(
    config: &ThemingConfiguration,
    theme_def: &ThemeDefinition,
    global_tokens: &TokenSet,
    accentable_tokens_map: &HashMap<TokenIdentifier, AccentModificationType>,
) -> Result<BTreeMap<TokenIdentifier, String>, ThemingError> {
    let mut current_intermediate_tokens = BTreeMap::new();

    for (id, raw_token) in global_tokens {
        current_intermediate_tokens.insert(id.clone(), raw_token.value.clone());
    }
    for (id, raw_token) in &theme_def.base_tokens {
        current_intermediate_tokens.insert(id.clone(), raw_token.value.clone());
    }
    
    if let Some(variant_def) = theme_def.variants.iter().find(|v| v.applies_to_scheme == config.preferred_color_scheme) {
        for (id, raw_token) in &variant_def.tokens {
            current_intermediate_tokens.insert(id.clone(), raw_token.value.clone());
        }
    }

    if let Some(selected_accent_core_color) = &config.selected_accent_color {
        for (token_id_to_accent, modification_type) in accentable_tokens_map {
            let base_value = current_intermediate_tokens.get(token_id_to_accent).cloned();

            if let Some(TokenValue::Color(base_color_str)) = base_value {
                match CoreColor::from_hex(&base_color_str) {
                    Ok(base_core_color) => {
                        let modified_core_color = match modification_type {
                            AccentModificationType::DirectReplace => selected_accent_core_color.clone(),
                            AccentModificationType::Lighten(factor) => base_core_color.lighten(*factor),
                            AccentModificationType::Darken(factor) => base_core_color.darken(*factor),
                        };
                        current_intermediate_tokens.insert(token_id_to_accent.clone(), TokenValue::Color(modified_core_color.to_hex_string()));
                    }
                    Err(e) => {
                        return Err(ThemingError::AccentColorApplicationError {
                            token_id: token_id_to_accent.clone(),
                            accent_color_name_disp: super::errors::AccentColorApplicationErrorDisplay::new(None),
                            accent_color_value: selected_accent_core_color.clone(),
                            reason: format!("Failed to parse base color string '{}' for token '{}': {:?}", base_color_str, token_id_to_accent, e),
                        });
                    }
                }
            } else if base_value.is_some() {
                 return Err(ThemingError::AccentColorApplicationError {
                    token_id: token_id_to_accent.clone(),
                    accent_color_name_disp: super::errors::AccentColorApplicationErrorDisplay::new(None),
                    accent_color_value: selected_accent_core_color.clone(),
                    reason: format!("Token '{}' is not a color token, cannot apply accent.", token_id_to_accent),
                });
            }
        }
    }

    if let Some(user_overrides) = &config.custom_user_token_overrides {
        for (id, raw_token) in user_overrides {
            current_intermediate_tokens.insert(id.clone(), raw_token.value.clone());
        }
    }

    let mut final_css_tokens = BTreeMap::new();
    for (id, value) in &current_intermediate_tokens {
        let final_string = resolve_single_token_value(
            id, // original_id is the key we are trying to populate in final_css_tokens
            id, // current_id_to_resolve starts as the same
            value,
            &mut Vec::new(),
            0,
            MAX_TOKEN_RESOLUTION_DEPTH,
        )?;
        final_css_tokens.insert(id.clone(), final_string);
    }

    Ok(final_css_tokens)
}


// --- Fallback Theme Logic (sync) ---

pub fn generate_fallback_theme_definition_and_tokens() -> Result<(ThemeDefinition, TokenSet), ThemingError> {
    let fallback_theme_def: ThemeDefinition = serde_json::from_str(FALLBACK_THEME_JSON)
        .map_err(|e| ThemingError::ConfigurationError {
            message: format!("Failed to parse fallback.theme.json: {}. Content snippet: {}", e, &FALLBACK_THEME_JSON.chars().take(100).collect::<String>()),
        })?;
    
    // Fallback theme's base_tokens are defined within its own JSON.
    let fallback_theme_tokens = fallback_theme_def.base_tokens.clone();

    // Validate the fallback theme definition itself (e.g., references within its own base_tokens)
    // For this validation, global_tokens is empty as fallback is self-contained.
    validate_theme_definition_references(&fallback_theme_def, &TokenSet::new())?;
    validate_tokenset_for_cycles(&fallback_theme_tokens)?;

    Ok((fallback_theme_def, fallback_theme_tokens))
}

pub fn generate_fallback_applied_state() -> AppliedThemeState {
    match generate_fallback_theme_definition_and_tokens() {
        Ok((fallback_theme_def, _)) => { 
            let fallback_config = ThemingConfiguration {
                selected_theme_id: fallback_theme_def.id.clone(),
                preferred_color_scheme: ColorSchemeType::Dark,
                selected_accent_color: None,
                custom_user_token_overrides: None,
            };
            
            let accentable_map = fallback_theme_def.accentable_tokens.clone().unwrap_or_default();

            match resolve_tokens_for_config(
                &fallback_config,
                &fallback_theme_def,
                &TokenSet::new(), 
                &accentable_map,
            ) {
                Ok(resolved_tokens) => AppliedThemeState {
                    theme_id: fallback_theme_def.id,
                    color_scheme: fallback_config.preferred_color_scheme,
                    active_accent_color: None,
                    resolved_tokens,
                },
                Err(e) => {
                    error!("Failed to resolve fallback theme tokens: {:?}. This indicates a problem with the embedded fallback theme definition or resolution logic.", e);
                    AppliedThemeState {
                        theme_id: ThemeIdentifier::new("fallback-error"),
                        color_scheme: ColorSchemeType::Dark,
                        active_accent_color: None,
                        resolved_tokens: BTreeMap::from([
                            (TokenIdentifier::new("error.text"), "#FF0000".to_string()),
                            (TokenIdentifier::new("error.background"), "#000000".to_string()),
                        ]),
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to generate fallback theme definition: {:?}. Critical error with embedded fallback JSON.", e);
            AppliedThemeState {
                theme_id: ThemeIdentifier::new("fallback-critical-error"),
                color_scheme: ColorSchemeType::Dark,
                active_accent_color: None,
                resolved_tokens: BTreeMap::from([
                    (TokenIdentifier::new("critical.error.text"), "#FF0000".to_string()),
                    (TokenIdentifier::new("critical.error.background"), "#000000".to_string()),
                ]),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::config::MockConfigServiceAsync;
    use std::sync::Arc;
    use tokio;

    // Helper to create a new mock service for each test
    fn new_mock_config_service() -> MockConfigServiceAsync {
        MockConfigServiceAsync::new()
    }
    
    fn arc_mock_service(mock: MockConfigServiceAsync) -> Arc<dyn ConfigServiceAsync> {
        Arc::new(mock)
    }


    #[tokio::test]
    async fn test_load_raw_tokens_from_file_simple_valid() {
        let mut mock_service_impl = new_mock_config_service();
        let path = PathBuf::from("test.json");
        let content = r#"[
            {"id": "color.blue", "value": {"color": "blue"}, "description": "Blue color"},
            {"id": "size.small", "value": {"dimension": "10px"}}
        ]"#;
        mock_service_impl.expect_read_file_to_string()
            .withf(move |p| p == Path::new("test.json"))
            .returning(move |_| Ok(content.to_string()));
        
        let mock_service_arc = arc_mock_service(mock_service_impl);
        let result = load_raw_tokens_from_file(&path, &mock_service_arc).await;
        assert!(result.is_ok(), "Result was not OK: {:?}", result.err());
        let token_set = result.unwrap();
        assert_eq!(token_set.len(), 2);
        assert!(token_set.contains_key(&TokenIdentifier::new("color.blue")));
    }

    #[tokio::test]
    async fn test_load_raw_tokens_from_file_duplicate_id() {
        let mut mock_service_impl = new_mock_config_service();
        let path = PathBuf::from("duplicate.json");
        let content = r#"[
            {"id": "color.blue", "value": {"color": "blue"}},
            {"id": "color.blue", "value": {"color": "darkblue"}}
        ]"#;
        mock_service_impl.expect_read_file_to_string()
            .returning(move |_| Ok(content.to_string()));

        let mock_service_arc = arc_mock_service(mock_service_impl);
        let result = load_raw_tokens_from_file(&path, &mock_service_arc).await;
        assert!(matches!(result, Err(ThemingError::InvalidTokenValue { token_id, .. }) if token_id.as_str() == "color.blue"));
    }


    #[tokio::test]
    async fn test_load_raw_tokens_from_file_invalid_opacity() {
        let mut mock_service_impl = new_mock_config_service();
        let path = PathBuf::from("invalid_opacity.json");
        let content = r#"[{"id": "alpha.invalid", "value": {"opacity": 1.5}}]"#;
         mock_service_impl.expect_read_file_to_string()
            .returning(move |_| Ok(content.to_string()));
        
        let mock_service_arc = arc_mock_service(mock_service_impl);
        let result = load_raw_tokens_from_file(&path, &mock_service_arc).await;
        assert!(matches!(result, Err(ThemingError::InvalidTokenValue { token_id, .. }) if token_id.as_str() == "alpha.invalid"));
    }
    
    #[test]
    fn test_validate_tokenset_no_cycle() {
        let mut tokens = TokenSet::new();
        tokens.insert(TokenIdentifier::new("a"), RawToken {id: TokenIdentifier::new("a"), value: TokenValue::Reference(TokenIdentifier::new("b")), description: None, group: None });
        tokens.insert(TokenIdentifier::new("b"), RawToken {id: TokenIdentifier::new("b"), value: TokenValue::Color("blue".into()), description: None, group: None });
        assert!(validate_tokenset_for_cycles(&tokens).is_ok());
    }

    #[test]
    fn test_validate_tokenset_with_cycle() {
        let mut tokens = TokenSet::new();
        tokens.insert(TokenIdentifier::new("a"), RawToken {id: TokenIdentifier::new("a"), value: TokenValue::Reference(TokenIdentifier::new("b")), description: None, group: None });
        tokens.insert(TokenIdentifier::new("b"), RawToken {id: TokenIdentifier::new("b"), value: TokenValue::Reference(TokenIdentifier::new("a")), description: None, group: None });
        let result = validate_tokenset_for_cycles(&tokens);
        assert!(matches!(result, Err(ThemingError::CyclicTokenReference { token_id, .. }) if token_id.as_str() == "a" || token_id.as_str() == "b"));
    }
    
    #[test]
    fn test_resolve_single_token_direct() {
        let all_tokens = BTreeMap::from([
            (TokenIdentifier::new("color.blue"), TokenValue::Color("blue".to_string()))
        ]);
        let id = TokenIdentifier::new("color.blue");
        let val = TokenValue::Color("blue".to_string());
        let result = resolve_single_token_value(&id, &id, &val, &all_tokens, &mut Vec::new(), 0, MAX_TOKEN_RESOLUTION_DEPTH);
        assert_eq!(result.unwrap(), "blue");
    }

    #[test]
    fn test_resolve_single_token_reference() {
        let all_tokens = BTreeMap::from([
            (TokenIdentifier::new("alias.blue"), TokenValue::Reference(TokenIdentifier::new("color.blue"))),
            (TokenIdentifier::new("color.blue"), TokenValue::Color("blue".to_string())),
        ]);
        let id = TokenIdentifier::new("alias.blue"); // Original ID we want to resolve
        let val = TokenValue::Reference(TokenIdentifier::new("color.blue")); // Value of alias.blue
        let result = resolve_single_token_value(&id, &id, &val, &all_tokens, &mut Vec::new(), 0, MAX_TOKEN_RESOLUTION_DEPTH);
        assert_eq!(result.unwrap(), "blue");
    }

    #[test]
    fn test_resolve_single_token_cycle() {
        let all_tokens = BTreeMap::from([
            (TokenIdentifier::new("cycle.a"), TokenValue::Reference(TokenIdentifier::new("cycle.b"))),
            (TokenIdentifier::new("cycle.b"), TokenValue::Reference(TokenIdentifier::new("cycle.a"))),
        ]);
        let id = TokenIdentifier::new("cycle.a");
        let val = TokenValue::Reference(TokenIdentifier::new("cycle.b"));
        let result = resolve_single_token_value(&id, &id, &val, &all_tokens, &mut Vec::new(), 0, MAX_TOKEN_RESOLUTION_DEPTH);
        assert!(matches!(result, Err(ThemingError::CyclicTokenReference { token_id, .. }) if token_id.as_str() == "cycle.a" || token_id.as_str() == "cycle.b"));
    }
    
    #[test]
    fn test_resolve_single_token_max_depth() {
        let mut tokens_map = BTreeMap::new();
        // depth.0 -> depth.1 -> ... -> depth.MAX+1 -> color
        for i in 0..=MAX_TOKEN_RESOLUTION_DEPTH {
            tokens_map.insert(
                TokenIdentifier::new(format!("depth.{}", i)),
                TokenValue::Reference(TokenIdentifier::new(format!("depth.{}", i + 1))),
            );
        }
        tokens_map.insert(
            TokenIdentifier::new(format!("depth.{}", MAX_TOKEN_RESOLUTION_DEPTH + 1)),
            TokenValue::Color("deep color".to_string()),
        );

        let id_to_resolve = TokenIdentifier::new("depth.0");
        let initial_value = tokens_map.get(&id_to_resolve).unwrap();
        let result = resolve_single_token_value(&id_to_resolve, &id_to_resolve, initial_value, &tokens_map, &mut Vec::new(), 0, MAX_TOKEN_RESOLUTION_DEPTH);
        
        match result {
            Err(ThemingError::TokenResolutionError { token_id, reason }) => {
                 assert_eq!(token_id.as_str(), "depth.0");
                 assert!(reason.contains("Maximum reference depth"));
            }
            _ => panic!("Expected TokenResolutionError (MaxDepth), got {:?}", result),
        }
    }

    #[test]
    fn test_generate_fallback_theme_def() {
        let result = generate_fallback_theme_definition_and_tokens();
        assert!(result.is_ok(), "Fallback generation failed: {:?}", result.err());
        let (theme_def, tokens) = result.unwrap();
        assert_eq!(theme_def.id.as_str(), "fallback");
        assert!(theme_def.base_tokens.contains_key(&TokenIdentifier::new("color.text.primary")));
        assert_eq!(tokens.len(), theme_def.base_tokens.len());
    }

    #[test]
    fn test_generate_fallback_applied_state_resolves() {
        let state = generate_fallback_applied_state();
        assert_eq!(state.theme_id.as_str(), "fallback");
        assert_ne!(state.theme_id.as_str(), "fallback-error", "Fallback theme resolution failed unexpectedly: {:?}", state.resolved_tokens.get(&TokenIdentifier::new("error.text")));
        assert_ne!(state.theme_id.as_str(), "fallback-critical-error", "Fallback theme definition failed unexpectedly");
        assert!(state.resolved_tokens.contains_key(&TokenIdentifier::new("color.text.primary")));
        assert_eq!(state.resolved_tokens.get(&TokenIdentifier::new("color.text.primary")).unwrap(), "#eeeeee");
    }

    #[test]
    fn test_resolve_tokens_for_config_basic() {
        let global_tokens = TokenSet::new(); 
        let theme_id = ThemeIdentifier::new("test-theme");
        let mut base_tokens = TokenSet::new();
        base_tokens.insert(
            TokenIdentifier::new("color.primary"),
            RawToken { id: TokenIdentifier::new("color.primary"), value: TokenValue::Color("blue".to_string()), description: None, group: None }
        );
        base_tokens.insert(
            TokenIdentifier::new("color.text"),
            RawToken { id: TokenIdentifier::new("color.text"), value: TokenValue::Reference(TokenIdentifier::new("color.primary")), description: None, group: None }
        );

        let theme_def = ThemeDefinition {
            id: theme_id.clone(),
            name: "Test Theme".to_string(),
            description: None, author: None, version: None,
            base_tokens,
            variants: vec![],
            supported_accent_colors: None,
            accentable_tokens: None,
        };
        let config = ThemingConfiguration {
            selected_theme_id: theme_id,
            preferred_color_scheme: ColorSchemeType::Light,
            selected_accent_color: None,
            custom_user_token_overrides: None,
        };

        let accentable_map = HashMap::new();
        let result = resolve_tokens_for_config(&config, &theme_def, &global_tokens, &accentable_map);
        assert!(result.is_ok(), "Resolution failed: {:?}", result.err());
        let resolved = result.unwrap();
        assert_eq!(resolved.get(&TokenIdentifier::new("color.primary")).unwrap(), "blue");
        assert_eq!(resolved.get(&TokenIdentifier::new("color.text")).unwrap(), "blue");
    }
}
