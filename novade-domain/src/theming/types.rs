use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug, Display};
use crate::shared_types::ApplicationId; // Assuming this might be used later, or for consistency
use novade_core::types::Color as CoreColor; // Renaming to avoid conflict if Color is defined here

// Regular expression for validating TokenIdentifier and ThemeIdentifier
// Allows alphanumeric characters, hyphens, and underscores. Must start with a letter.
// Based on typical CSS custom property naming conventions.
const IDENTIFIER_REGEX_STR: &str = r"^[a-zA-Z][a-zA-Z0-9_-]*$";

/// Represents a unique identifier for a design token.
///
/// It must conform to a specific pattern: start with a letter,
/// followed by alphanumeric characters, hyphens, or underscores.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TokenIdentifier(String);

impl Default for TokenIdentifier {
    fn default() -> Self {
        // Provide a valid default identifier string that passes validation.
        // This is important if TokenIdentifier is part of a struct that derives Default.
        TokenIdentifier::new("default-token-identifier")
    }
}

impl TokenIdentifier {
    /// Creates a new `TokenIdentifier`.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the provided `id` is empty or does not match
    /// the required pattern: `^[a-zA-Z][a-zA-Z0-9_-]*$`.
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        debug_assert!(!id_str.is_empty(), "TokenIdentifier darf nicht leer sein.");
        // Simple regex for now, can be replaced with a lazy_static regex instance
        debug_assert!(
            id_str.chars().next().map_or(false, |c| c.is_ascii_alphabetic()) &&
            id_str.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "TokenIdentifier '{}' hat ein ungültiges Format. Er muss mit einem Buchstaben beginnen und darf nur alphanumerische Zeichen, Bindestriche oder Unterstriche enthalten.",
            id_str
        );
        Self(id_str)
    }

    /// Returns a string slice of the token identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for TokenIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TokenIdentifier").field(&self.0).finish()
    }
}

impl Display for TokenIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for TokenIdentifier {
    fn from(id: String) -> Self {
        TokenIdentifier::new(id)
    }
}

impl From<&str> for TokenIdentifier {
    fn from(id: &str) -> Self {
        TokenIdentifier::new(id.to_string())
    }
}

/// Represents the value of a design token.
///
/// This enum covers various types of values that a token can hold,
/// including references to other tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TokenValue {
    Color(String), // Stores color as hex string, e.g., "#RRGGBBAA" or reference
    Dimension(String), // e.g., "16px", "2rem", "100%"
    FontFamily(String), // e.g., "Inter, sans-serif"
    FontWeight(String), // e.g., "400", "bold"
    FontSize(String), // e.g., "1rem", "16px"
    LineHeight(String), // e.g., "1.5", "150%"
    LetterSpacing(String), // e.g., "0.05em"
    Duration(String), // e.g., "250ms", "0.5s"
    BorderStyle(String), // e.g., "solid", "dashed"
    BorderWidth(String), // e.g., "1px"
    BoxShadow(String), // e.g., "0px 4px 8px rgba(0,0,0,0.1)"
    Opacity(String), // e.g., "0.8", "80%"
    Spacing(String), // e.g., "8px", "1rem"
    Typography(HashMap<String, String>), // For composite typography tokens
    Generic(String), // For any other string-based value
    Reference(TokenIdentifier), // Reference to another token
}

/// Represents a raw design token as defined in a token file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawToken {
    #[serde(alias = "name")] // Allow "name" as an alias for "id" during deserialization
    pub id: TokenIdentifier,
    pub value: TokenValue,
    #[serde(default)] // Optional description
    pub description: Option<String>,
    #[serde(default)] // Optional group for organization
    pub group: Option<String>,
    // TODO: Add metadata fields like "$extensions", "data", etc. if needed from Design Token Community Group spec.
}

/// A collection of raw tokens, typically representing a complete token set from a file or a theme.
/// Uses `BTreeMap` to ensure tokens are sorted by identifier, which can be useful for consistency.
pub type TokenSet = BTreeMap<TokenIdentifier, RawToken>;


/// Represents a unique identifier for a theme.
///
/// It must conform to a specific pattern: start with a letter,
/// followed by alphanumeric characters, hyphens, or underscores.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ThemeIdentifier(String);

impl Default for ThemeIdentifier {
    fn default() -> Self {
        // Provide a valid default identifier string that passes validation.
        ThemeIdentifier::new("default-theme-identifier")
    }
}

impl ThemeIdentifier {
    /// Creates a new `ThemeIdentifier`.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the provided `id` is empty or does not match
    /// the required pattern: `^[a-zA-Z][a-zA-Z0-9_-]*$`.
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        debug_assert!(!id_str.is_empty(), "ThemeIdentifier darf nicht leer sein.");
        debug_assert!(
            id_str.chars().next().map_or(false, |c| c.is_ascii_alphabetic()) &&
            id_str.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "ThemeIdentifier '{}' hat ein ungültiges Format. Er muss mit einem Buchstaben beginnen und darf nur alphanumerische Zeichen, Bindestriche oder Unterstriche enthalten.",
            id_str
        );
        Self(id_str)
    }

    /// Returns a string slice of the theme identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for ThemeIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ThemeIdentifier").field(&self.0).finish()
    }
}

impl Display for ThemeIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ThemeIdentifier {
    fn from(id: String) -> Self {
        ThemeIdentifier::new(id)
    }
}

impl From<&str> for ThemeIdentifier {
    fn from(id: &str) -> Self {
        ThemeIdentifier::new(id.to_string())
    }
}

/// Defines the basic color scheme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ColorSchemeType {
    #[default]
    Light,
    Dark,
}

/// Represents an accent color option for a theme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccentColor {
    /// Optional name for the accent color, e.g., "Sky Blue", "Crimson Red".
    pub name: Option<String>,
    /// The actual color value.
    pub value: CoreColor, // Assuming novade_core::types::Color handles its own serde
}

/// Defines a specific variant of a theme, usually for a light or dark scheme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeVariantDefinition {
    /// The color scheme this variant applies to.
    pub applies_to_scheme: ColorSchemeType,
    /// The set of tokens specific to this variant. These tokens override or extend the base tokens.
    pub tokens: TokenSet,
}

/// Defines how an accent color modifies a base token value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccentModificationType {
    /// Directly replaces the token's color value with the accent color.
    DirectReplace,
    /// Lightens the token's existing color value by a factor, mixing with the accent color.
    /// The f32 value should be between 0.0 (no change) and 1.0 (fully accent color).
    /// (Note: Actual color manipulation logic will be in `logic.rs`)
    Lighten(f32),
    /// Darkens the token's existing color value by a factor, mixing with the accent color.
    /// The f32 value should be between 0.0 (no change) and 1.0 (fully accent color).
    /// (Note: Actual color manipulation logic will be in `logic.rs`)
    Darken(f32),
    /// Tints the accent color by mixing it with the token's original color.
    /// The f32 value represents the mix ratio (0.0 = original color, 1.0 = accent color).
    TintWithOriginal(f32),
    // Add more modification types as needed, e.g., for opacity, saturation.
}

/// Defines the structure of a theme, including its base tokens, variants, and accent color options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub id: ThemeIdentifier,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub version: Option<String>, // e.g., "1.0.0"

    /// Base set of tokens that apply to all variants unless overridden.
    pub base_tokens: TokenSet,

    /// Specific token overrides for different color schemes (e.g., light, dark).
    pub variants: Vec<ThemeVariantDefinition>,

    /// Optional list of predefined accent colors supported by this theme.
    #[serde(default)]
    pub supported_accent_colors: Option<Vec<AccentColor>>,

    /// Defines which tokens are affected by the accent color and how.
    /// Maps a `TokenIdentifier` to an `AccentModificationType`.
    #[serde(default)]
    pub accentable_tokens: Option<HashMap<TokenIdentifier, AccentModificationType>>,
}

/// Represents the fully resolved state of a theme as currently applied to the UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppliedThemeState {
    pub theme_id: ThemeIdentifier,
    pub color_scheme: ColorSchemeType,
    pub active_accent_color: Option<CoreColor>, // The actual color value
    /// Fully resolved token values, where all references are processed and values are concrete strings.
    pub resolved_tokens: BTreeMap<TokenIdentifier, String>,
}

/// Configuration settings for the theming engine, typically managed by user preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ThemingConfiguration {
    /// The identifier of the currently selected theme.
    pub selected_theme_id: ThemeIdentifier, // Default might be tricky without a known fallback
    /// The user's preferred color scheme (light or dark).
    pub preferred_color_scheme: ColorSchemeType,
    /// The user's selected accent color. If None, the theme's default might be used or accenting disabled.
    #[serde(default)]
    pub selected_accent_color: Option<CoreColor>,
    /// User-specific token overrides that take precedence over theme and variant tokens.
    #[serde(default)]
    pub custom_user_token_overrides: Option<TokenSet>,
}


#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::types::Color as CoreColor; // For AccentColor tests

    // --- TokenIdentifier Tests ---
    #[test]
    fn token_identifier_valid() {
        assert_eq!(TokenIdentifier::new("valid-id_123").as_str(), "valid-id_123");
        assert_eq!(TokenIdentifier::from("anotherValidID").as_str(), "anotherValidID");
    }

    #[test]
    #[should_panic(expected = "TokenIdentifier darf nicht leer sein.")]
    #[cfg(debug_assertions)]
    fn token_identifier_empty_panic() {
        TokenIdentifier::new("");
    }

    #[test]
    #[should_panic(expected = "hat ein ungültiges Format")]
    #[cfg(debug_assertions)]
    fn token_identifier_invalid_start_char_panic() {
        TokenIdentifier::new("1-invalid-start");
    }

    #[test]
    #[should_panic(expected = "hat ein ungültiges Format")]
    #[cfg(debug_assertions)]
    fn token_identifier_invalid_char_panic() {
        TokenIdentifier::new("invalid char!");
    }
    
    #[test]
    fn token_identifier_display() {
        let id = TokenIdentifier::new("test-id");
        assert_eq!(format!("{}", id), "test-id");
    }

    #[test]
    fn token_identifier_serde() {
        let id = TokenIdentifier::new("serde-test-token");
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "\"serde-test-token\"");
        let deserialized: TokenIdentifier = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, id);
    }

    // --- ThemeIdentifier Tests ---
    #[test]
    fn theme_identifier_valid() {
        assert_eq!(ThemeIdentifier::new("valid-theme_123").as_str(), "valid-theme_123");
    }

    #[test]
    #[should_panic(expected = "ThemeIdentifier darf nicht leer sein.")]
    #[cfg(debug_assertions)]
    fn theme_identifier_empty_panic() {
        ThemeIdentifier::new("");
    }

    #[test]
    #[should_panic(expected = "hat ein ungültiges Format")]
    #[cfg(debug_assertions)]
    fn theme_identifier_invalid_start_char_panic() {
        ThemeIdentifier::new("-invalid-theme-start");
    }
    
    #[test]
    fn theme_identifier_display() {
        let id = ThemeIdentifier::new("test-theme-id");
        assert_eq!(format!("{}", id), "test-theme-id");
    }

    #[test]
    fn theme_identifier_serde() {
        let id = ThemeIdentifier::new("serde-test-theme");
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "\"serde-test-theme\"");
        let deserialized: ThemeIdentifier = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, id);
    }

    // --- TokenValue Tests ---
    #[test]
    fn token_value_serde() {
        let val_color = TokenValue::Color("#RRGGBBAA".to_string());
        let ser_color = serde_json::to_string(&val_color).unwrap();
        assert_eq!(ser_color, r#"{"color":"#RRGGBBAA"}"#); // serde adds type for enum variants
        let de_color: TokenValue = serde_json::from_str(&ser_color).unwrap();
        assert_eq!(de_color, val_color);

        let val_ref = TokenValue::Reference(TokenIdentifier::new("ref-token"));
        let ser_ref = serde_json::to_string(&val_ref).unwrap();
        // Example: {"reference":"ref-token"} if TokenIdentifier serializes as string directly
        // Or: {"reference":{"0":"ref-token"}} if it's a tuple struct internally to serde
        // Based on TokenIdentifier's Serialize impl (transparent or direct string)
        // If TokenIdentifier is `#[derive(Serialize)] struct TokenIdentifier(String);`
        // and `serde_json::to_string` is used on `TokenIdentifier` directly, it's `""ref-token""`.
        // In an enum, it might be different. Let's assume TokenIdentifier serializes as its inner string for now.
        // The current TokenIdentifier does not have `#[serde(transparent)]` so it will be `{"reference":"ref-token"}`
        // No, TokenIdentifier is `struct TokenIdentifier(String);` which serializes as `string` if used directly.
        // Inside an enum like TokenValue, `Reference(TokenIdentifier)` will be `{"reference": "value-of-token-identifier"}`
        assert!(ser_ref.contains(r#""reference":"ref-token""#)); // Flexible check
        let de_ref: TokenValue = serde_json::from_str(&ser_ref).unwrap();
        assert_eq!(de_ref, val_ref);
    }

    // --- RawToken Tests ---
    #[test]
    fn raw_token_serde() {
        let token = RawToken {
            id: TokenIdentifier::new("token-1"),
            value: TokenValue::Dimension("16px".to_string()),
            description: Some("A test token".to_string()),
            group: Some("spacing".to_string()),
        };
        let serialized = serde_json::to_string(&token).unwrap();
        // Example: {"id":"token-1","value":{"dimension":"16px"},"description":"A test token","group":"spacing"}
        assert!(serialized.contains(r#""id":"token-1""#));
        assert!(serialized.contains(r#""value":{"dimension":"16px"}"#));
        let deserialized: RawToken = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, token);
    }
    
    #[test]
    fn raw_token_serde_name_alias() {
        // Constructing the JSON programmatically to avoid potential raw string literal issues
        let json_value = serde_json::json!({
            "name": "token-alias",
            "value": { "color": "#FF0000" },
            "description": "Test alias"
        });
        let json_with_name = serde_json::to_string(&json_value).unwrap();
        let deserialized: RawToken = serde_json::from_str(&json_with_name).unwrap();
        assert_eq!(deserialized.id, TokenIdentifier::new("token-alias"));
        assert_eq!(deserialized.value, TokenValue::Color("#FF0000".to_string()));
        assert_eq!(deserialized.description, Some("Test alias".to_string()));
        assert_eq!(deserialized.group, None);
    }


    // --- ColorSchemeType Tests ---
    #[test]
    fn color_scheme_type_default_and_serde() {
        assert_eq!(ColorSchemeType::default(), ColorSchemeType::Light);
        let scheme = ColorSchemeType::Dark;
        let serialized = serde_json::to_string(&scheme).unwrap();
        assert_eq!(serialized, "\"dark\""); // kebab-case
        let deserialized: ColorSchemeType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, scheme);
    }

    // --- AccentColor Tests ---
    #[test]
    fn accent_color_serde() {
        // Assuming CoreColor can be created from hex and serialized/deserialized appropriately.
        // This test depends on novade_core::types::Color's serde behavior.
        // For now, let's assume it serializes to its hex string.
        let accent = AccentColor {
            name: Some("Sky Blue".to_string()),
            value: CoreColor::from_rgba(0, 122, 204, 255), // Example: #007ACCFF
        };
        // If CoreColor serializes as its hex string:
        // Expected: {"name":"Sky Blue","value":"#007ACCFF"}
        let serialized = serde_json::to_string(&accent).unwrap();
        // We need to know CoreColor's exact serde format to make a strict assertion.
        // Let's assume it's an object like `{"r":0,"g":122,"b":204,"a":255}` or a hex string.
        // If it's a hex string directly from CoreColor's Serialize impl:
        // assert_eq!(serialized, r#"{"name":"Sky Blue","value":"#007accff"}"#);
        // If CoreColor serializes into an object:
        // assert!(serialized.contains(r#""name":"Sky Blue""#));
        // assert!(serialized.contains(r#""value":{"r":0,"g":122,"b":204,"a":255}"#));
        // For now, just ensure it serializes and deserializes.
        let deserialized: AccentColor = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, accent);
    }

    // --- ThemingConfiguration Tests ---
    #[test]
    fn theming_configuration_default() {
        // This test depends on ThemeIdentifier::default() which is not derived.
        // ThemingConfiguration::default() will use ThemeIdentifier("") if not careful.
        // Let's ensure ThemeIdentifier::default() provides a valid, though perhaps dummy, ID.
        // For now, ThemeIdentifier::default() is derived, which means ThemeIdentifier("".to_string()).
        // This will panic on `new("")`. So, `ThemingConfiguration::default()` would panic.
        // This means `ThemingConfiguration` cannot use `#[derive(Default)]`
        // unless `ThemeIdentifier` has a Default impl that provides a valid default ID.
        // Or, `ThemingConfiguration` needs a custom `Default` impl.
        
        // Let's provide a custom Default for ThemeIdentifier for testing Default derive on ThemingConfiguration
        // Or, more simply, test serde for ThemingConfiguration for now.
        // If ThemeIdentifier::default() is problematic, ThemingConfiguration::default() is too.
        // The current plan is that ThemeIdentifier derives Default. This is an issue for validation.
        // For now, I will skip testing Default for ThemingConfiguration directly here
        // and assume it will be constructed explicitly in usage.
        // The `generate_fallback_applied_state` and `ThemingEngine::new` handle initial config.
    // UPDATE: With Default for ThemeIdentifier, this test is now valid.
    }
    
    #[test]
    fn theming_configuration_default_derived() {
        let config = ThemingConfiguration::default();
        assert_eq!(config.selected_theme_id.as_str(), "default-theme-identifier"); // Check our new default
        assert_eq!(config.preferred_color_scheme, ColorSchemeType::Light); // Default for enum
        assert_eq!(config.selected_accent_color, None);
        assert_eq!(config.custom_user_token_overrides, None);
    }


    #[test]
    fn theming_configuration_serde() {
        let config = ThemingConfiguration {
            selected_theme_id: ThemeIdentifier::new("my-theme"),
            preferred_color_scheme: ColorSchemeType::Dark,
            selected_accent_color: Some(CoreColor::from_rgba(255,0,0,255)), // Red
            custom_user_token_overrides: None,
        };
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: ThemingConfiguration = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.selected_theme_id, config.selected_theme_id);
        assert_eq!(deserialized.preferred_color_scheme, config.preferred_color_scheme);
        assert_eq!(deserialized.selected_accent_color, config.selected_accent_color);
    }

    // --- AccentModificationType Tests ---
    #[test]
    fn accent_modification_type_serde() {
        let direct = AccentModificationType::DirectReplace;
        let ser_direct = serde_json::to_string(&direct).unwrap();
        assert_eq!(ser_direct, "\"direct-replace\""); // kebab-case
        let de_direct: AccentModificationType = serde_json::from_str(&ser_direct).unwrap();
        assert_eq!(de_direct, direct);

        let lighten = AccentModificationType::Lighten(0.5);
        let ser_lighten = serde_json::to_string(&lighten).unwrap();
        assert_eq!(ser_lighten, r#"{"lighten":0.5}"#); // kebab-case for enum variant name
        let de_lighten: AccentModificationType = serde_json::from_str(&ser_lighten).unwrap();
        assert_eq!(de_lighten, lighten);
    }
    
    // --- ThemeDefinition Tests ---
    #[test]
    fn theme_definition_serde_minimal() {
        let mut base_tokens = TokenSet::new();
        base_tokens.insert(
            TokenIdentifier::new("color-primary"),
            RawToken {
                id: TokenIdentifier::new("color-primary"),
                value: TokenValue::Color("#007ACC".to_string()),
                description: None,
                group: None,
            },
        );

        let theme_def = ThemeDefinition {
            id: ThemeIdentifier::new("test-theme"),
            name: "Test Theme".to_string(),
            description: None,
            author: None,
            version: None,
            base_tokens,
            variants: vec![],
            supported_accent_colors: None,
            accentable_tokens: None,
        };

        let serialized = serde_json::to_string(&theme_def).unwrap();
        // Check some key parts
        assert!(serialized.contains(r#""id":"test-theme""#));
        assert!(serialized.contains(r#""name":"Test Theme""#));
        assert!(serialized.contains(r#""color-primary":{"id":"color-primary","value":{"color":"#007ACC"},"description":null,"group":null}"#));
        
        let deserialized: ThemeDefinition = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, theme_def.id);
        assert_eq!(deserialized.name, theme_def.name);
        assert_eq!(deserialized.base_tokens.get(&TokenIdentifier::new("color-primary")), 
                   theme_def.base_tokens.get(&TokenIdentifier::new("color-primary")));
    }

    #[test]
    fn theme_definition_serde_full() {
        let mut base_tokens = TokenSet::new();
        base_tokens.insert(TokenIdentifier::new("global-spacing"), RawToken {
            id: TokenIdentifier::new("global-spacing"), value: TokenValue::Spacing("8px".to_string()), description: None, group: None
        });

        let mut light_variant_tokens = TokenSet::new();
        light_variant_tokens.insert(TokenIdentifier::new("color-background"), RawToken {
            id: TokenIdentifier::new("color-background"), value: TokenValue::Color("#FFFFFF".to_string()), description: None, group: None
        });
        
        let light_variant = ThemeVariantDefinition {
            applies_to_scheme: ColorSchemeType::Light,
            tokens: light_variant_tokens,
        };

        let mut accentable = HashMap::new();
        accentable.insert(TokenIdentifier::new("color-button-primary"), AccentModificationType::DirectReplace);

        let theme_def = ThemeDefinition {
            id: ThemeIdentifier::new("full-theme"),
            name: "Full Featured Theme".to_string(),
            description: Some("A theme with all fields populated".to_string()),
            author: Some("NovaDE Team".to_string()),
            version: Some("1.1.0".to_string()),
            base_tokens,
            variants: vec![light_variant],
            supported_accent_colors: Some(vec![AccentColor { name: Some("Sky Blue".to_string()), value: CoreColor::from_hex("#007ACCFF").unwrap() }]),
            accentable_tokens: Some(accentable),
        };
        
        let serialized = serde_json::to_string_pretty(&theme_def).unwrap();
        // println!("Serialized ThemeDefinition:\n{}", serialized); // For debug
        
        let deserialized: ThemeDefinition = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, theme_def.id);
        assert_eq!(deserialized.variants.len(), 1);
        assert_eq!(deserialized.variants[0].applies_to_scheme, ColorSchemeType::Light);
        assert!(deserialized.supported_accent_colors.is_some());
        assert_eq!(deserialized.supported_accent_colors.as_ref().unwrap().len(), 1);
        assert!(deserialized.accentable_tokens.is_some());
        assert!(deserialized.accentable_tokens.as_ref().unwrap().contains_key(&TokenIdentifier::new("color-button-primary")));
    }
}
