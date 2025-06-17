//! Core algorithms for the NovaDE theming system.
//!
//! This module provides functions for loading, validating, and resolving theme definitions
//! (`ThemeDefinition`) and token sets (`TokenSet`) based on a given `ThemingConfiguration`.
//! It handles the complexities of token referencing, cycle detection, application of
//! theme variants (for light/dark schemes), accent color modifications, and user overrides.
//! The primary entry point for resolving a full theme configuration into a set of
//! applicable styles is `resolve_tokens_for_config`. This module also includes logic
//! for generating a fallback theme state if primary theme loading or resolution fails.

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

/// Loads raw tokens from a single JSON file using the provided `ConfigServiceAsync`.
///
/// Each token in the file is validated (e.g., for valid opacity values).
/// Duplicate token identifiers within the same file will result in an error.
///
/// # Arguments
/// * `path`: Path to the JSON file containing an array of `RawToken` objects.
/// * `config_service`: Service used to read the file content.
///
/// # Returns
/// A `TokenSet` containing the loaded tokens, or a `ThemingError` if loading,
/// parsing, or validation fails.
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

/// Loads and validates `RawToken`s from multiple JSON files, merging them into a single `TokenSet`.
///
/// Tokens from files later in the `paths` list will override tokens with the same ID
/// from earlier files. After merging, the combined `TokenSet` is validated for circular references.
///
/// # Arguments
/// * `paths`: A slice of `PathBuf`s, each pointing to a JSON token file.
/// * `config_service`: Service used to read the file contents.
///
/// # Returns
/// A merged `TokenSet` from all files, or a `ThemingError` if any file operation,
/// parsing, or validation (including cycle detection) fails.
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

/// Loads a `ThemeDefinition` from a single JSON file using the `ConfigServiceAsync`.
///
/// It also verifies that the `id` field within the loaded `ThemeDefinition` matches
/// the `theme_id_from_path` (derived from the filename).
///
/// # Arguments
/// * `path`: Path to the `.theme.json` file.
/// * `theme_id_from_path`: The `ThemeIdentifier` expected, typically derived from the filename.
/// * `config_service`: Service used to read the file content.
///
/// # Returns
/// The loaded `ThemeDefinition`, or a `ThemingError` if file operations, parsing,
/// or ID validation fails.
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

/// Loads and validates multiple `ThemeDefinition` files from the given paths.
///
/// For each theme file, it:
/// 1. Derives the expected `ThemeIdentifier` from the filename.
/// 2. Loads the `ThemeDefinition` using `load_theme_definition_from_file`.
/// 3. Validates all token references within the loaded theme definition (base and variants)
///    against the provided `global_tokens` and tokens defined within the theme itself.
///
/// # Arguments
/// * `paths`: A slice of `PathBuf`s, each pointing to a `.theme.json` file.
/// * `global_tokens`: A `TokenSet` of globally available tokens that can be referenced by themes.
/// * `config_service`: Service used to read file contents.
///
/// # Returns
/// A `Vec` of loaded and validated `ThemeDefinition`s, or a `ThemingError` if any
/// operation fails for any theme.
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

/// Validates a `TokenSet` for circular dependencies among `TokenValue::Reference` entries.
///
/// # Arguments
/// * `tokens`: The `TokenSet` to validate.
///
/// # Returns
/// `Ok(())` if no cycles are detected, otherwise `Err(ThemingError::CyclicTokenReference)`.
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

/// Validates that all `TokenValue::Reference` entries within a `ThemeDefinition`
/// (including its base tokens and all variant tokens) point to known tokens.
///
/// A reference is considered valid if it points to:
/// 1. A token within the same `TokenSet` (e.g., another token in `base_tokens` or within the same variant's `tokens`).
/// 2. A token in the theme's `base_tokens` (if checking a variant).
/// 3. A token in the provided `global_tokens`.
///
/// # Arguments
/// * `theme_def`: The `ThemeDefinition` to validate.
/// * `global_tokens`: A `TokenSet` of globally available tokens.
///
/// # Returns
/// `Ok(())` if all references are valid, otherwise `Err(ThemingError::TokenResolutionError)`
/// indicating an undefined token reference.
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

/// Recursively resolves a single `TokenValue` to its final string representation.
///
/// This function handles `TokenValue::Reference` by looking up the referenced token
/// in `all_tokens` and resolving it. It detects circular references and enforces
/// a maximum resolution depth to prevent infinite loops. Other `TokenValue` variants
/// are converted directly to their string form.
///
/// # Arguments
/// * `original_id`: The `TokenIdentifier` of the token for which resolution was initially requested.
///   Used for error reporting to provide context.
/// * `current_id_to_resolve`: The `TokenIdentifier` of the token currently being processed.
///   This might be the `original_id` or an ID from a reference chain.
/// * `current_value`: The `TokenValue` of the `current_id_to_resolve`.
/// * `all_tokens`: A map of all available token identifiers to their `TokenValue`s at the current
///   stage of resolution (e.g., after merging base, variant, global, and user tokens, but before
///   resolving references and applying functions like opacity formatting).
/// * `visited_path`: A vector used to track the chain of references being followed, for cycle detection.
/// * `current_depth`: The current depth in the reference chain.
/// * `max_depth`: The maximum allowed depth for reference chains.
///
/// # Returns
/// The resolved string value of the token, or a `ThemingError` if resolution fails
/// (e.g., cycle detected, max depth exceeded, undefined reference).
pub fn resolve_single_token_value(
    original_id: &TokenIdentifier, // The ID we are trying to resolve for the final map
    current_id_to_resolve: &TokenIdentifier, // The ID of the token currently being processed (could be a reference)
    current_value: &TokenValue,    // The value of current_id_to_resolve
    all_tokens: &BTreeMap<TokenIdentifier, TokenValue>, // Map of available token values for lookup
    visited_path: &mut Vec<TokenIdentifier>, // Path of reference lookups for cycle detection
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


/// Resolves a complete `ThemingConfiguration` into a final set of string-based token values.
///
/// This is the main function for processing a theme into a state ready for UI consumption.
/// The process involves several steps:
/// 1. **Merging**: Combines tokens from `global_tokens`, the `theme_def.base_tokens`,
///    and the appropriate `theme_def.variants[n].tokens` based on `config.preferred_color_scheme`.
///    Tokens from later stages override earlier ones (Variant > Base > Global).
/// 2. **Accent Application**: If `config.selected_accent_color` is set and the `theme_def`
///    has `accentable_tokens`, it modifies the colors of specified tokens according to
///    their `AccentModificationType`.
/// 3. **User Overrides**: Applies `config.custom_user_token_overrides`, which take the
///    highest precedence, potentially overriding any token from previous steps.
/// 4. **Reference Resolution**: Iterates through the resulting merged set of tokens and resolves
///    all `TokenValue::Reference` entries to their final string values using
///    `resolve_single_token_value`. This also handles formatting for types like `Opacity`.
///
/// # Arguments
/// * `config`: The `ThemingConfiguration` specifying user preferences (selected theme, scheme, accent, overrides).
/// * `theme_def`: The `ThemeDefinition` for the `config.selected_theme_id`.
/// * `global_tokens`: A `TokenSet` of globally available raw tokens.
/// * `accentable_tokens_map`: A pre-processed map from `theme_def.accentable_tokens` for efficient lookup.
///
/// # Returns
/// A `BTreeMap` where keys are `TokenIdentifier`s and values are their fully resolved string
/// representations, or a `ThemingError` if any part of the process fails.
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

/// Loads and validates the embedded fallback `ThemeDefinition` and its associated `TokenSet`.
///
/// The fallback theme is defined in `default_themes/fallback.theme.json`. This function
/// parses this JSON, validates it for internal consistency (references, cycles), and
/// returns the theme definition and its base tokens.
///
/// # Returns
/// A `Result` containing a tuple of (`ThemeDefinition`, `TokenSet`) for the fallback theme,
/// or a `ThemingError` if parsing or validation of the embedded fallback theme fails.
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

/// Generates a fully resolved `AppliedThemeState` for the system's fallback theme.
///
/// This function is called when the `ThemingEngine` cannot apply any user-specified
/// or default theme (e.g., due to missing files, corrupted definitions, or resolution errors).
/// It ensures that the application always has a usable, albeit basic, theme.
///
/// The process involves:
/// 1. Loading the fallback `ThemeDefinition` and its tokens using `generate_fallback_theme_definition_and_tokens`.
/// 2. Creating a default `ThemingConfiguration` that selects the fallback theme and a default scheme (e.g., Dark).
/// 3. Resolving this configuration using `resolve_tokens_for_config`.
///
/// If any of these steps fail (which would indicate a critical issue with the embedded
/// fallback theme itself), an error is logged, and an even more basic, hardcoded error
/// theme state is returned.
///
/// # Returns
/// An `AppliedThemeState` representing the resolved fallback theme.
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

    // --- Enhanced Tests for resolve_tokens_for_config ---

    fn create_raw_token(id_str: &str, value: TokenValue) -> RawToken {
        RawToken {
            id: TokenIdentifier::new(id_str),
            value,
            description: Some(format!("Test token {}", id_str)),
            group: Some("TestGroup".to_string()),
        }
    }

    fn create_test_theme_def(id_str: &str, base_tokens: TokenSet, variants: Vec<ThemeVariantDefinition>, supported_accents: Option<Vec<AccentColor>>, accentable: Option<HashMap<TokenIdentifier, AccentModificationType>>) -> ThemeDefinition {
        ThemeDefinition {
            id: ThemeIdentifier::new(id_str),
            name: format!("Theme {}", id_str),
            description: Some(format!("Description for theme {}", id_str)),
            author: Some("Test Author".to_string()),
            version: Some("1.0.0".to_string()),
            base_tokens,
            variants,
            supported_accent_colors: supported_accents,
            accentable_tokens: accentable,
        }
    }

    #[test]
    fn test_resolve_variant_application() {
        let mut base_tokens = TokenSet::new();
        base_tokens.insert(TokenIdentifier::new("color.background"), create_raw_token("color.background", TokenValue::Color("white".to_string())));
        base_tokens.insert(TokenIdentifier::new("color.text"), create_raw_token("color.text", TokenValue::Color("black".to_string())));
        base_tokens.insert(TokenIdentifier::new("spacing.base"), create_raw_token("spacing.base", TokenValue::Dimension("4px".to_string())));

        let mut dark_variant_tokens = TokenSet::new();
        dark_variant_tokens.insert(TokenIdentifier::new("color.background"), create_raw_token("color.background", TokenValue::Color("black".to_string()))); // Override
        dark_variant_tokens.insert(TokenIdentifier::new("color.variant.specific"), create_raw_token("color.variant.specific", TokenValue::Color("grey".to_string()))); // New

        let dark_variant = ThemeVariantDefinition {
            applies_to_scheme: ColorSchemeType::Dark,
            tokens: dark_variant_tokens,
        };

        let theme_def = create_test_theme_def("variant-theme", base_tokens, vec![dark_variant], None, None);
        let global_tokens = TokenSet::new();
        let accentable_map = HashMap::new();

        // Test Dark Variant
        let config_dark = ThemingConfiguration {
            selected_theme_id: theme_def.id.clone(),
            preferred_color_scheme: ColorSchemeType::Dark,
            selected_accent_color: None,
            custom_user_token_overrides: None,
        };
        let resolved_dark = resolve_tokens_for_config(&config_dark, &theme_def, &global_tokens, &accentable_map).unwrap();

        assert_eq!(resolved_dark.get(&TokenIdentifier::new("color.background")).unwrap(), "black", "Dark variant override failed");
        assert_eq!(resolved_dark.get(&TokenIdentifier::new("color.text")).unwrap(), "black", "Base token should persist if not overridden in variant");
        assert_eq!(resolved_dark.get(&TokenIdentifier::new("color.variant.specific")).unwrap(), "grey", "Variant specific token missing");
        assert_eq!(resolved_dark.get(&TokenIdentifier::new("spacing.base")).unwrap(), "4px", "Base token (non-color) should persist");

        // Test Light Variant (should use base tokens as no light variant defined)
        let config_light = ThemingConfiguration {
            selected_theme_id: theme_def.id.clone(),
            preferred_color_scheme: ColorSchemeType::Light,
            ..config_dark // rest is same
        };
        let resolved_light = resolve_tokens_for_config(&config_light, &theme_def, &global_tokens, &accentable_map).unwrap();
        assert_eq!(resolved_light.get(&TokenIdentifier::new("color.background")).unwrap(), "white", "Light scheme should use base token");
        assert_eq!(resolved_light.get(&TokenIdentifier::new("color.text")).unwrap(), "black");
        assert!(!resolved_light.contains_key(&TokenIdentifier::new("color.variant.specific")), "Variant specific token should not be present for light scheme");
    }

    #[test]
    fn test_resolve_accent_color_application() {
        let mut base_tokens = TokenSet::new();
        base_tokens.insert(TokenIdentifier::new("color.primary"), create_raw_token("color.primary", TokenValue::Color("#0000FF".to_string()))); // Blue
        base_tokens.insert(TokenIdentifier::new("color.secondary"), create_raw_token("color.secondary", TokenValue::Color("#00FF00".to_string()))); // Green
        base_tokens.insert(TokenIdentifier::new("size.font"), create_raw_token("size.font", TokenValue::Dimension("16px".to_string())));


        let supported_accents = vec![
            AccentColor { name: Some("Test Red".to_string()), value: CoreColor::from_hex("#FF0000").unwrap() }, // Red
        ];
        let red_accent_value = CoreColor::from_hex("#FF0000").unwrap();

        let mut accentable = HashMap::new();
        accentable.insert(TokenIdentifier::new("color.primary"), AccentModificationType::DirectReplace);
        accentable.insert(TokenIdentifier::new("color.secondary"), AccentModificationType::Lighten(0.5)); // Lighten green

        let theme_def = create_test_theme_def("accent-theme", base_tokens, vec![], Some(supported_accents), Some(accentable));
        let global_tokens = TokenSet::new();

        // Scenario 1: DirectReplace
        let config_replace = ThemingConfiguration {
            selected_theme_id: theme_def.id.clone(),
            preferred_color_scheme: ColorSchemeType::Light,
            selected_accent_color: Some(red_accent_value.clone()),
            custom_user_token_overrides: None,
        };
        let resolved_replace = resolve_tokens_for_config(&config_replace, &theme_def, &global_tokens, &theme_def.accentable_tokens.as_ref().unwrap()).unwrap();
        assert_eq!(resolved_replace.get(&TokenIdentifier::new("color.primary")).unwrap().to_lowercase(), "#ff0000", "DirectReplace failed");

        // Scenario 2: Lighten (Green #00FF00 lighten 0.5 -> #80ff80)
        // Note: CoreColor::lighten behavior needs to be consistent. Assuming it is.
        let expected_lightened_green = CoreColor::from_hex("#00FF00").unwrap().lighten(0.5).to_hex_string();
        assert_eq!(resolved_replace.get(&TokenIdentifier::new("color.secondary")).unwrap().to_lowercase(), expected_lightened_green.to_lowercase(), "Lighten failed");

        // Scenario 3: Darken (Add a darken case)
        let mut accentable_darken = theme_def.accentable_tokens.clone().unwrap();
        accentable_darken.insert(TokenIdentifier::new("color.primary"), AccentModificationType::Darken(0.5)); // Darken blue
        let theme_def_darken = ThemeDefinition { accentable_tokens: Some(accentable_darken), ..theme_def.clone() };
        let resolved_darken = resolve_tokens_for_config(&config_replace, &theme_def_darken, &global_tokens, &theme_def_darken.accentable_tokens.as_ref().unwrap()).unwrap();
        let expected_darkened_blue = CoreColor::from_hex("#0000FF").unwrap().darken(0.5).to_hex_string();
        assert_eq!(resolved_darken.get(&TokenIdentifier::new("color.primary")).unwrap().to_lowercase(), expected_darkened_blue.to_lowercase(), "Darken failed");

        // Scenario 4: No accent color selected
        let config_no_accent = ThemingConfiguration { selected_accent_color: None, ..config_replace.clone() };
        let resolved_no_accent = resolve_tokens_for_config(&config_no_accent, &theme_def, &global_tokens, &theme_def.accentable_tokens.as_ref().unwrap()).unwrap();
        assert_eq!(resolved_no_accent.get(&TokenIdentifier::new("color.primary")).unwrap().to_lowercase(), "#0000ff", "No accent should use base color");

        // Scenario 5: Accenting a non-color token (should error)
        let mut accentable_error = theme_def.accentable_tokens.clone().unwrap();
        accentable_error.insert(TokenIdentifier::new("size.font"), AccentModificationType::DirectReplace);
        let theme_def_error = ThemeDefinition { accentable_tokens: Some(accentable_error), ..theme_def.clone() };
        let result_error = resolve_tokens_for_config(&config_replace, &theme_def_error, &global_tokens, &theme_def_error.accentable_tokens.as_ref().unwrap());
        assert!(matches!(result_error, Err(ThemingError::AccentColorApplicationError {token_id, .. }) if token_id.as_str() == "size.font" ), "Accenting non-color should error");
    }

    #[test]
    fn test_resolve_user_token_overrides() {
        let mut base_tokens = TokenSet::new();
        base_tokens.insert(TokenIdentifier::new("spacing.medium"), create_raw_token("spacing.medium", TokenValue::Dimension("8px".to_string())));
        base_tokens.insert(TokenIdentifier::new("color.background"), create_raw_token("color.background", TokenValue::Color("white".to_string())));

        let theme_def = create_test_theme_def("override-theme", base_tokens, vec![], None, None);
        let global_tokens = TokenSet::new();
        let accentable_map = HashMap::new();

        let mut user_overrides = TokenSet::new();
        user_overrides.insert(TokenIdentifier::new("spacing.medium"), create_raw_token("spacing.medium", TokenValue::Dimension("10px".to_string()))); // Override
        user_overrides.insert(TokenIdentifier::new("user.custom"), create_raw_token("user.custom", TokenValue::String("my-value".to_string()))); // New

        let config = ThemingConfiguration {
            selected_theme_id: theme_def.id.clone(),
            preferred_color_scheme: ColorSchemeType::Light,
            selected_accent_color: None,
            custom_user_token_overrides: Some(user_overrides),
        };

        let resolved = resolve_tokens_for_config(&config, &theme_def, &global_tokens, &accentable_map).unwrap();
        assert_eq!(resolved.get(&TokenIdentifier::new("spacing.medium")).unwrap(), "10px", "User override for existing token failed");
        assert_eq!(resolved.get(&TokenIdentifier::new("user.custom")).unwrap(), "my-value", "User specific token failed");
        assert_eq!(resolved.get(&TokenIdentifier::new("color.background")).unwrap(), "white", "Base token should persist if not overridden by user");
    }

    #[test]
    fn test_resolve_interaction_variant_accent_override() {
        let mut base_tokens = TokenSet::new(); // color.primary: blue
        base_tokens.insert(TokenIdentifier::new("color.primary"), create_raw_token("color.primary", TokenValue::Color("#0000FF".to_string())));

        let mut dark_variant_tokens = TokenSet::new(); // color.primary: purple (#800080) in dark variant
        dark_variant_tokens.insert(TokenIdentifier::new("color.primary"), create_raw_token("color.primary", TokenValue::Color("#800080".to_string())));
        let dark_variant = ThemeVariantDefinition { applies_to_scheme: ColorSchemeType::Dark, tokens: dark_variant_tokens };

        let supported_accents = vec![AccentColor { name: Some("Test Green".to_string()), value: CoreColor::from_hex("#00FF00").unwrap() }]; // Green accent
        let green_accent_value = CoreColor::from_hex("#00FF00").unwrap();
        let mut accentable = HashMap::new(); // color.primary is accentable by direct replace
        accentable.insert(TokenIdentifier::new("color.primary"), AccentModificationType::DirectReplace);

        let theme_def = create_test_theme_def("interaction-theme", base_tokens, vec![dark_variant], Some(supported_accents), Some(accentable.clone()));
        let global_tokens = TokenSet::new();

        // 1. User override takes highest precedence
        let mut user_overrides = TokenSet::new(); // color.primary: yellow (#FFFF00) by user
        user_overrides.insert(TokenIdentifier::new("color.primary"), create_raw_token("color.primary", TokenValue::Color("#FFFF00".to_string())));
        let config_with_override = ThemingConfiguration {
            selected_theme_id: theme_def.id.clone(),
            preferred_color_scheme: ColorSchemeType::Dark, // Dark variant active
            selected_accent_color: Some(green_accent_value.clone()), // Green accent selected
            custom_user_token_overrides: Some(user_overrides),
        };
        let resolved_override = resolve_tokens_for_config(&config_with_override, &theme_def, &global_tokens, &accentable).unwrap();
        assert_eq!(resolved_override.get(&TokenIdentifier::new("color.primary")).unwrap().to_lowercase(), "#ffff00", "User override should take precedence");

        // 2. No user override: Accent applies to Variant token
        let config_variant_accent = ThemingConfiguration {
            custom_user_token_overrides: None, // No user override
            ..config_with_override.clone()
        };
        let resolved_variant_accent = resolve_tokens_for_config(&config_variant_accent, &theme_def, &global_tokens, &accentable).unwrap();
        // Dark variant is purple (#800080), accent is green (#00FF00). DirectReplace means green wins.
        assert_eq!(resolved_variant_accent.get(&TokenIdentifier::new("color.primary")).unwrap().to_lowercase(), "#00ff00", "Accent should apply to variant token");

        // 3. No user override, no accent: Variant token used
        let config_variant_only = ThemingConfiguration {
            selected_accent_color: None, // No accent
            custom_user_token_overrides: None,
            ..config_with_override.clone()
        };
        let resolved_variant_only = resolve_tokens_for_config(&config_variant_only, &theme_def, &global_tokens, &accentable).unwrap();
        assert_eq!(resolved_variant_only.get(&TokenIdentifier::new("color.primary")).unwrap().to_lowercase(), "#800080", "Variant token should be used");

        // 4. No user override, no accent, light scheme: Base token used
        let config_base_only = ThemingConfiguration {
            preferred_color_scheme: ColorSchemeType::Light, // Light scheme (no variant defined for it)
            selected_accent_color: None,
            custom_user_token_overrides: None,
            ..config_with_override.clone()
        };
        let resolved_base_only = resolve_tokens_for_config(&config_base_only, &theme_def, &global_tokens, &accentable).unwrap();
        assert_eq!(resolved_base_only.get(&TokenIdentifier::new("color.primary")).unwrap().to_lowercase(), "#0000ff", "Base token should be used for light scheme");
    }

    #[test]
    fn test_resolve_reference_resolution_with_overrides_variants() {
        let mut base_tokens = TokenSet::new();
        base_tokens.insert(TokenIdentifier::new("base.actual"), create_raw_token("base.actual", TokenValue::Color("blue".to_string())));
        base_tokens.insert(TokenIdentifier::new("base.ref"), create_raw_token("base.ref", TokenValue::Reference(TokenIdentifier::new("base.actual"))));
        base_tokens.insert(TokenIdentifier::new("another.actual"), create_raw_token("another.actual", TokenValue::Color("yellow".to_string())));

        let mut dark_variant_tokens = TokenSet::new();
        dark_variant_tokens.insert(TokenIdentifier::new("base.actual"), create_raw_token("base.actual", TokenValue::Color("red".to_string()))); // Variant overrides base.actual
        let dark_variant = ThemeVariantDefinition { applies_to_scheme: ColorSchemeType::Dark, tokens: dark_variant_tokens };

        let theme_def = create_test_theme_def("ref-theme", base_tokens, vec![dark_variant], None, None);
        let global_tokens = TokenSet::new();
        let accentable_map = HashMap::new();

        // 1. Reference resolves to variant's overridden value
        let config_variant = ThemingConfiguration {
            selected_theme_id: theme_def.id.clone(),
            preferred_color_scheme: ColorSchemeType::Dark, // Dark variant active
            selected_accent_color: None,
            custom_user_token_overrides: None,
        };
        let resolved_variant = resolve_tokens_for_config(&config_variant, &theme_def, &global_tokens, &accentable_map).unwrap();
        assert_eq!(resolved_variant.get(&TokenIdentifier::new("base.ref")).unwrap(), "red", "Reference should resolve to variant's value");

        // 2. User override of the target (`base.actual`) affects reference
        let mut user_overrides_target = TokenSet::new();
        user_overrides_target.insert(TokenIdentifier::new("base.actual"), create_raw_token("base.actual", TokenValue::Color("green".to_string())));
        let config_user_target = ThemingConfiguration {
            custom_user_token_overrides: Some(user_overrides_target),
            ..config_variant.clone()
        };
        let resolved_user_target = resolve_tokens_for_config(&config_user_target, &theme_def, &global_tokens, &accentable_map).unwrap();
        assert_eq!(resolved_user_target.get(&TokenIdentifier::new("base.ref")).unwrap(), "green", "Reference should resolve to user-overridden target value");

        // 3. User override of the reference itself (`base.ref`)
        let mut user_overrides_ref = TokenSet::new();
        user_overrides_ref.insert(TokenIdentifier::new("base.ref"), create_raw_token("base.ref", TokenValue::Reference(TokenIdentifier::new("another.actual"))));
        let config_user_ref = ThemingConfiguration {
            custom_user_token_overrides: Some(user_overrides_ref),
            ..config_variant.clone()
        };
        let resolved_user_ref = resolve_tokens_for_config(&config_user_ref, &theme_def, &global_tokens, &accentable_map).unwrap();
        assert_eq!(resolved_user_ref.get(&TokenIdentifier::new("base.ref")).unwrap(), "yellow", "User-overridden reference should point to new target's value");
    }
}
