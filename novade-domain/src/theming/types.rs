//! Defines the core data structures used throughout the NovaDE theming system.
//!
//! This module contains types for identifying themes and tokens, representing token values,
//! defining the structure of themes (including variants and accent color support),
//! storing user theme preferences, and representing the fully resolved theme state
//! that is applied to the UI.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use novade_core::types::Color as CoreColor;

// --- TokenIdentifier ---
/// A unique identifier for a design token.
///
/// Token identifiers are typically hierarchical, using dots as separators
/// (e.g., `color.background.primary`, `font.size.body`).
/// They must consist of ASCII alphanumeric characters, dots (.), or hyphens (-).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TokenIdentifier(String);

impl TokenIdentifier {
    /// Creates a new `TokenIdentifier`.
    /// Panics in debug mode if the ID string is empty or contains invalid characters.
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        debug_assert!(!id_str.is_empty() && id_str.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'), "TokenIdentifier: '{}' contains invalid characters or is empty", id_str);
        Self(id_str)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for TokenIdentifier {
    fn from(id: String) -> Self {
        debug_assert!(!id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'), "TokenIdentifier from String: '{}' contains invalid characters or is empty", id);
        Self(id)
    }
}

impl From<&str> for TokenIdentifier {
    fn from(id: &str) -> Self {
        debug_assert!(!id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'), "TokenIdentifier from &str: '{}' contains invalid characters or is empty", id);
        Self(id.to_string())
    }
}

impl fmt::Display for TokenIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --- TokenValue ---
/// Represents the actual value of a design token.
///
/// A `TokenValue` can be a direct value (like a color string, dimension, number)
/// or a reference to another `TokenIdentifier`. The specific types (Color, Dimension, etc.)
/// help in categorizing and validating tokens, although they are often resolved to strings
/// for final CSS output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")] // Ensures enum variants are serialized like "kebab-case" in JSON
pub enum TokenValue {
    /// A color value, typically a hex string (e.g., "#RRGGBB", "#RRGGBBAA") or CSS color name.
    Color(String),
    /// A sizing value, typically including units (e.g., "16px", "2em", "100%").
    Dimension(String),
    /// A font family string (e.g., "'Inter', sans-serif").
    FontFamily(String),
    /// A font weight string or number (e.g., "bold", "normal", "700").
    FontWeight(String),
    /// A font size, typically including units (e.g., "1rem", "12pt").
    FontSize(String),
    /// Letter spacing value, typically with units (e.g., "0.5px", "normal").
    LetterSpacing(String),
    /// Line height value, unitless (e.g., "1.5") or with units (e.g., "20px").
    LineHeight(String),
    /// A CSS border string (e.g., "1px solid #CCCCCC").
    Border(String),
    /// A CSS box-shadow string (e.g., "2px 2px 5px rgba(0,0,0,0.2)").
    Shadow(String),
    /// An opacity value, typically a float between 0.0 and 1.0.
    Opacity(f64),
    /// A generic number value.
    Number(f64),
    /// A generic string value.
    String(String),
    /// A reference to another `TokenIdentifier`. This allows tokens to inherit values.
    Reference(TokenIdentifier),
}

// --- RawToken ---
/// Represents a token as defined in a theme or token file, before resolution.
///
/// It includes the token's `id`, its `value` (which might be a direct value or a reference),
/// and optional metadata like `description` and `group`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawToken {
    /// The unique identifier for this token.
    /// When tokens are stored in a `TokenSet` (BTreeMap), this `id` field should
    /// typically match the key in the map. Serialization might skip this field if it's
    /// considered redundant with the map key in some contexts, though current setup includes it.
    #[serde(default, skip_serializing_if = "is_default_id_from_key")]
    pub id: TokenIdentifier,
    /// The `TokenValue` of this token.
    pub value: TokenValue,
    /// An optional description of the token's purpose or usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// An optional group name, used for organizing tokens (e.g., in a design tool or documentation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

// Helper for RawToken serde: if ID is same as key in map, it might be omitted.
// This is a simple default check; BTreeMap might not need this if id is always present.
#[allow(clippy::trivially_copy_pass_by_ref)] // Cloned anyway by TokenIdentifier
fn is_default_id_from_key(id: &TokenIdentifier) -> bool {
    id.as_str().is_empty() // Assuming default TokenIdentifier means an empty string, adjust if different.
}


// --- TokenSet ---
/// A collection of `RawToken`s, keyed by their `TokenIdentifier`.
///
/// This is the primary structure for storing sets of tokens, whether they are
/// global tokens, base tokens within a theme, or tokens specific to a theme variant.
/// Using `BTreeMap` ensures that tokens are iterated in a consistent (sorted by ID) order.
pub type TokenSet = BTreeMap<TokenIdentifier, RawToken>;

// --- ThemeIdentifier ---
/// A unique identifier for a theme (e.g., "nova-dark", "solarized-light").
/// Theme identifiers must consist of ASCII alphanumeric characters or hyphens (-).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ThemeIdentifier(String);

impl ThemeIdentifier {
    /// Creates a new `ThemeIdentifier`.
    /// Panics in debug mode if the ID string is empty or contains invalid characters.
    pub fn new(id: impl Into<String>) -> Self {
    Color(String), // Value is a CSS color string e.g. "#RRGGBB", "rgba(...)", "blue"
    Dimension(String), // Value is a CSS dimension string e.g. "16px", "2em"
    FontFamily(String), // Value is a CSS font-family string e.g. "'Inter', sans-serif"
    FontWeight(String), // Value is a CSS font-weight string e.g. "bold", "400"
    FontSize(String), // Value is a CSS font-size string e.g. "1rem", "12pt"
    LetterSpacing(String), // Value is a CSS letter-spacing string e.g. "0.5px"
    LineHeight(String), // Value is a CSS line-height string or unitless number e.g. "1.5", "20px"
    Border(String), // Value is a CSS border string e.g. "1px solid #000"
    Shadow(String), // Value is a CSS box-shadow string e.g. "2px 2px 5px rgba(0,0,0,0.2)"
    Opacity(f64), // Value is a number between 0.0 and 1.0
    Number(f64), // A unitless number
    String(String), // A generic string value
    Reference(TokenIdentifier), // A reference to another token by its ID
}

// --- RawToken ---
/// Represents a token as defined in a theme or token file, before resolution.
///
/// It includes the token's `id`, its `value` (which might be a direct value or a reference),
/// and optional metadata like `description` and `group`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawToken {
    /// The unique identifier for this token.
    /// When tokens are stored in a `TokenSet` (BTreeMap), this `id` field should
    /// typically match the key in the map. Serialization might skip this field if it's
    /// considered redundant with the map key in some contexts, though current setup includes it.
    #[serde(default, skip_serializing_if = "is_default_id_from_key")]
    pub id: TokenIdentifier,
    /// The `TokenValue` of this token.
    pub value: TokenValue,
    /// An optional description of the token's purpose or usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// An optional group name, used for organizing tokens (e.g., in a design tool or documentation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

// Helper for RawToken serde: if ID is same as key in map, it might be omitted.
// This is a simple default check; BTreeMap might not need this if id is always present.
#[allow(clippy::trivially_copy_pass_by_ref)] // Cloned anyway by TokenIdentifier
fn is_default_id_from_key(id: &TokenIdentifier) -> bool {
    id.as_str().is_empty() // Assuming default TokenIdentifier means an empty string, adjust if different.
}


// --- TokenSet ---
/// A collection of `RawToken`s, keyed by their `TokenIdentifier`.
///
/// This is the primary structure for storing sets of tokens, whether they are
/// global tokens, base tokens within a theme, or tokens specific to a theme variant.
/// Using `BTreeMap` ensures that tokens are iterated in a consistent (sorted by ID) order.
pub type TokenSet = BTreeMap<TokenIdentifier, RawToken>;

// --- ThemeIdentifier ---
/// A unique identifier for a theme (e.g., "nova-dark", "solarized-light").
/// Theme identifiers must consist of ASCII alphanumeric characters or hyphens (-).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ThemeIdentifier(String);

impl ThemeIdentifier {
    /// Creates a new `ThemeIdentifier`.
    /// Panics in debug mode if the ID string is empty or contains invalid characters.
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        debug_assert!(!id_str.is_empty() && id_str.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'), "ThemeIdentifier: '{}' contains invalid characters or is empty", id_str);
        Self(id_str)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ThemeIdentifier {
    fn from(id: String) -> Self {
        debug_assert!(!id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'), "ThemeIdentifier from String: '{}' contains invalid characters or is empty", id);
        Self(id)
    }
}

impl From<&str> for ThemeIdentifier {
    fn from(id: &str) -> Self {
        debug_assert!(!id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'), "ThemeIdentifier from &str: '{}' contains invalid characters or is empty", id);
        Self(id.to_string())
    }
}

impl fmt::Display for ThemeIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --- ColorSchemeType ---
/// Specifies the preferred color scheme, typically Light or Dark.
/// This is used to select appropriate theme variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ColorSchemeType {
    /// A light color scheme, typically with light backgrounds and dark text.
    #[default]
    Light,
    /// A dark color scheme, typically with dark backgrounds and light text.
    Dark,
}

// --- AccentColor ---
/// Represents a named accent color option available within a theme.
///
/// An accent color is a specific color value (defined by `CoreColor`) that can be
/// used by a theme to modify certain `accentable_tokens`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccentColor {
    /// An optional human-readable name for the accent color (e.g., "Sky Blue", "Crimson Red").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The actual color value, using `novade_core::types::Color`.
    pub value: CoreColor,
}
// Note: `novade_core::types::Color`'s `PartialEq` (and `Hash` if used in keys)
// needs to correctly handle floating point numbers if they are part of its structure.

// --- ThemeVariantDefinition ---
/// Defines a set of tokens that apply to a specific `ColorSchemeType`.
///
/// Theme variants allow a single `ThemeDefinition` to support multiple color schemes
/// (e.g., light and dark modes) by overriding or adding to the `base_tokens`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeVariantDefinition {
    /// The color scheme to which this variant's tokens apply.
    pub applies_to_scheme: ColorSchemeType,
    /// The set of tokens specific to this variant. These tokens will override
    /// `base_tokens` with the same `TokenIdentifier` when this variant is active.
    pub tokens: TokenSet,
}

// --- AccentModificationType ---
/// Specifies how an accent color should modify a base token's color value.
///
/// This enum defines the types of operations that can be performed when an
/// accent color is applied to an `accentable_token`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccentModificationType {
    /// Directly replaces the token's original color with the selected accent color.
    DirectReplace,
    /// Lightens the token's original color by a specified factor (0.0 to 1.0).
    /// The factor determines the amount of lightening, where 0.0 means no change
    /// and 1.0 means maximum lightening (approaching white, depending on implementation).
    Lighten(f32),
    /// Darkens the token's original color by a specified factor (0.0 to 1.0).
    /// The factor determines the amount of darkening, where 0.0 means no change
    /// and 1.0 means maximum darkening (approaching black, depending on implementation).
    Darken(f32),
}

// --- ThemeDefinition ---
/// The primary structure for defining a theme in NovaDE.
///
/// A `ThemeDefinition` encapsulates all aspects of a theme, including its identity,
/// metadata, base set of design tokens, variations for different color schemes (e.g., light/dark),
/// and support for accent colors.
///
/// This structure is typically deserialized from a `.theme.json` file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeDefinition {
    /// The unique identifier for this theme.
    pub id: ThemeIdentifier,
    /// The human-readable name of the theme (e.g., "Nova Default", "Solarized Dark").
    pub name: String,
    /// An optional longer description of the theme.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    /// The core set of design tokens that form the basis of this theme.
    /// These tokens apply unless overridden by a specific `ThemeVariantDefinition`
    /// or user preferences.
    pub base_tokens: TokenSet,
    
    /// A list of `ThemeVariantDefinition`s, allowing the theme to adapt to different
    /// `ColorSchemeType`s (e.g., providing a distinct set of tokens for dark mode).
    /// If empty, the `base_tokens` are used for all schemes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<ThemeVariantDefinition>,
    
    /// An optional list of `AccentColor`s that this theme supports.
    /// Users can select one of these to customize the theme's appearance further.
    /// If `None` or empty, the theme does not support user-selectable accent colors through this mechanism.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported_accent_colors: Option<Vec<AccentColor>>,
    
    /// An optional map defining which tokens are affected by the selected accent color
    /// and how they are modified. The key is the `TokenIdentifier` of the token to be
    /// accented, and the value is the `AccentModificationType` specifying the transformation.
    /// If `None` or empty, accent colors (even if supported) will not modify any tokens by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accentable_tokens: Option<HashMap<TokenIdentifier, AccentModificationType>>,
}

// --- AppliedThemeState ---
/// Represents the fully resolved state of the current theme, ready for UI consumption.
///
/// This struct is the output of the theming engine after processing the selected
/// `ThemeDefinition`, `ThemingConfiguration` (including preferred color scheme,
/// selected accent color, and user overrides), and global tokens.
/// It contains all the information a UI rendering system needs to style components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppliedThemeState {
    /// The `ThemeIdentifier` of the theme that was applied.
    pub theme_id: ThemeIdentifier,
    /// The `ColorSchemeType` (e.g., Light or Dark) for which this state was resolved.
    pub color_scheme: ColorSchemeType,
    /// The `AccentColor` that is currently active, if any. This includes its name (if provided
    /// in the `ThemeDefinition`) and its resolved `CoreColor` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_accent_color: Option<AccentColor>,
    /// A map of `TokenIdentifier` to their final, resolved string values.
    /// These are the values that should be directly used by the UI (e.g., as CSS custom properties).
    /// All references have been resolved, variants applied, accents processed, and user overrides incorporated.
    pub resolved_tokens: BTreeMap<TokenIdentifier, String>,
}

// --- ThemingConfiguration ---
/// Stores the user's theme preferences.
///
/// This configuration determines which theme is active, the preferred color scheme (light/dark),
/// any selected accent color, and custom token overrides set by the user.
/// It is typically loaded from and saved to a persistent storage (e.g., `theming.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemingConfiguration {
    /// The `ThemeIdentifier` of the theme selected by the user.
    pub selected_theme_id: ThemeIdentifier,
    /// The user's preferred `ColorSchemeType` (e.g., Light or Dark).
    pub preferred_color_scheme: ColorSchemeType,
    /// The `CoreColor` value of the accent color selected by the user.
    /// This should be one of the colors defined in the `supported_accent_colors` list
    /// of the selected `ThemeDefinition`, or `None` if no accent is chosen or supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_accent_color: Option<CoreColor>,
    /// An optional set of `RawToken`s provided by the user to override any tokens
    /// from the theme definition (base or variant) or global tokens.
    /// These overrides have the highest precedence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_user_token_overrides: Option<TokenSet>,
}

impl Default for ThemingConfiguration {
    /// Provides a default `ThemingConfiguration`.
    ///
    /// This default typically selects a system-default theme identifier and a light color scheme,
    /// with no accent color or user overrides. The actual fallback to a usable theme if
    /// "default-system" is not found is handled by the `ThemingEngine`.
    fn default() -> Self {
        Self {
            selected_theme_id: ThemeIdentifier::new("default-system"), // A conventional ID for a default theme
            preferred_color_scheme: ColorSchemeType::default(), // Defaults to Light
            selected_accent_color: None,
            custom_user_token_overrides: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::types::Color; // For AccentColor tests

    #[test]
    fn token_identifier_valid() {
        let ti = TokenIdentifier::new("core.primary-color");
        assert_eq!(ti.as_str(), "core.primary-color");
        assert_eq!(format!("{}", ti), "core.primary-color");
    }

    #[test]
    #[should_panic(expected = "TokenIdentifier: 'core/color!' contains invalid characters or is empty")]
    #[cfg(debug_assertions)]
    fn token_identifier_invalid_chars() {
        TokenIdentifier::new("core/color!");
    }

    #[test]
    #[should_panic(expected = "TokenIdentifier: '' contains invalid characters or is empty")]
    #[cfg(debug_assertions)]
    fn token_identifier_empty() {
        TokenIdentifier::new("");
    }
    
    #[test]
    fn token_identifier_from_str() {
        let ti = TokenIdentifier::from("semantic.font-size.body");
        assert_eq!(ti.as_str(), "semantic.font-size.body");
    }

    #[test]
    #[should_panic(expected = "TokenIdentifier from &str: 'invalid!' contains invalid characters or is empty")]
    #[cfg(debug_assertions)]
    fn token_identifier_from_str_invalid_panic() {
        TokenIdentifier::from("invalid!");
    }


    #[test]
    fn theme_identifier_valid() {
        let tid = ThemeIdentifier::new("my-custom-theme");
        assert_eq!(tid.as_str(), "my-custom-theme");
        assert_eq!(format!("{}", tid), "my-custom-theme");
    }

    #[test]
    #[should_panic(expected = "ThemeIdentifier: 'my_theme!' contains invalid characters or is empty")]
    #[cfg(debug_assertions)]
    fn theme_identifier_invalid_chars() {
        ThemeIdentifier::new("my_theme!"); 
    }
    
    #[test]
    #[should_panic(expected = "ThemeIdentifier: 'my.theme' contains invalid characters or is empty")]
    #[cfg(debug_assertions)]
    fn theme_identifier_invalid_chars_dots() {
        ThemeIdentifier::new("my.theme"); 
    }

    #[test]
    fn token_value_serde_color() {
        let color_val = TokenValue::Color("#FFFFFF".to_string());
        let serialized = serde_json::to_string(&color_val).unwrap();
        assert_eq!(serialized, r#"{"color":"#FFFFFF"}"#);
        let deserialized: TokenValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, color_val);
    }

    #[test]
    fn token_value_serde_reference() {
        let ref_val = TokenValue::Reference(TokenIdentifier::new("core.primary"));
        let serialized_ref = serde_json::to_string(&ref_val).unwrap();
        assert_eq!(serialized_ref, r#"{"reference":"core.primary"}"#);
        let deserialized_ref: TokenValue = serde_json::from_str(&serialized_ref).unwrap();
        assert_eq!(deserialized_ref, ref_val);
    }
        
    #[test]
    fn token_value_serde_opacity() {
        let opacity_val = TokenValue::Opacity(0.5);
        let serialized_opacity = serde_json::to_string(&opacity_val).unwrap();
        assert_eq!(serialized_opacity, r#"{"opacity":0.5}"#);
        let deserialized_opacity: TokenValue = serde_json::from_str(&serialized_opacity).unwrap();
        assert_eq!(deserialized_opacity, opacity_val);
    }

    #[test]
    fn raw_token_serde_full() {
        let token = RawToken {
            id: TokenIdentifier::new("brand.primary"),
            value: TokenValue::Color("blue".to_string()),
            description: Some("Primary brand color".to_string()),
            group: Some("brand-colors".to_string()),
        };
        let serialized = serde_json::to_string(&token).unwrap();
        let expected_json = r#"{"id":"brand.primary","value":{"color":"blue"},"description":"Primary brand color","group":"brand-colors"}"#;
        assert_eq!(serialized, expected_json);
        let deserialized: RawToken = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, token);
    }

    #[test]
    fn raw_token_serde_minimal() {
        let token = RawToken {
            id: TokenIdentifier::new("spacing.small"),
            value: TokenValue::Dimension("4px".to_string()),
            description: None,
            group: None,
        };
        let serialized = serde_json::to_string(&token).unwrap();
        let expected_json = r#"{"id":"spacing.small","value":{"dimension":"4px"}}"#;
        assert_eq!(serialized, expected_json);
        let deserialized: RawToken = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, token);
    }

    #[test]
    fn color_scheme_type_default_and_serde() {
        assert_eq!(ColorSchemeType::default(), ColorSchemeType::Light);
        let dark_scheme = ColorSchemeType::Dark;
        let serialized = serde_json::to_string(&dark_scheme).unwrap();
        assert_eq!(serialized, r#""Dark""#);
        let deserialized: ColorSchemeType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, dark_scheme);
    }

    #[test]
    fn accent_color_serde() {
        let accent = AccentColor {
            name: Some("Sky Blue".to_string()),
            value: Color::from_rgba(0.5, 0.7, 0.9, 1.0).unwrap(), // CoreColor returns Result
        };
        let serialized = serde_json::to_string(&accent).unwrap();
        // This depends on CoreColor's Serialize impl. Assuming it's struct-like.
        let expected_json_partial_value = r#""value":{"r":0.5,"g":0.7,"b":0.9,"a":1.0}"#; // Example
        assert!(serialized.contains(r#""name":"Sky Blue""#));
        assert!(serialized.contains(expected_json_partial_value)); 
        let deserialized: AccentColor = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, accent);
    }
    
    #[test]
    fn accent_modification_type_serde() {
        let mod_type = AccentModificationType::Lighten(0.25);
        let serialized = serde_json::to_string(&mod_type).unwrap();
        assert_eq!(serialized, r#"{"lighten":0.25}"#);
        let deserialized: AccentModificationType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, mod_type);

        let mod_type_direct = AccentModificationType::DirectReplace;
        let serialized_direct = serde_json::to_string(&mod_type_direct).unwrap();
        assert_eq!(serialized_direct, r#""direct-replace""#);
        let deserialized_direct: AccentModificationType = serde_json::from_str(&serialized_direct).unwrap();
        assert_eq!(deserialized_direct, mod_type_direct);
    }

    #[test]
    fn theme_definition_serde_minimal() {
        let theme_def = ThemeDefinition {
            id: ThemeIdentifier::new("my-theme"),
            name: "My Test Theme".to_string(),
            description: None,
            author: None,
            version: None,
            base_tokens: BTreeMap::new(),
            variants: vec![],
            supported_accent_colors: None,
            accentable_tokens: None,
        };
        let serialized = serde_json::to_string(&theme_def).unwrap();
        let expected_json = r#"{"id":"my-theme","name":"My Test Theme","base_tokens":{}}"#; // variants, supported_accent_colors, accentable_tokens skipped if empty/None due to serde attrs
        assert_eq!(serialized, expected_json);
        let deserialized: ThemeDefinition = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, theme_def);
    }

    #[test]
    fn applied_theme_state_serde() {
        let state = AppliedThemeState {
            theme_id: ThemeIdentifier::new("active-theme"),
            color_scheme: ColorSchemeType::Dark,
            active_accent_color: Some(AccentColor { name: None, value: Color::from_hex("#123456").unwrap() }),
            resolved_tokens: BTreeMap::from([(TokenIdentifier::new("text.color"), "white".to_string())]),
        };
        let serialized = serde_json::to_string(&state).unwrap();
        // Example, exact CoreColor serialization might vary
        assert!(serialized.contains(r#""theme_id":"active-theme""#));
        assert!(serialized.contains(r#""color_scheme":"Dark""#));
        assert!(serialized.contains(r#""active_accent_color":{"value":"#)); // Check for presence of value field
        assert!(serialized.contains(r#""resolved_tokens":{"text.color":"white"}""#));
        let deserialized: AppliedThemeState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, state);
    }

    #[test]
    fn theming_configuration_default_and_serde() {
        let config = ThemingConfiguration::default();
        assert_eq!(config.selected_theme_id.as_str(), "default-system");
        assert_eq!(config.preferred_color_scheme, ColorSchemeType::Light);
        assert!(config.selected_accent_color.is_none());
        assert!(config.custom_user_token_overrides.is_none());

        let serialized = serde_json::to_string(&config).unwrap();
        let expected_json = r#"{"selected_theme_id":"default-system","preferred_color_scheme":"Light"}"#; // Nones are skipped
        assert_eq!(serialized, expected_json);
        let deserialized: ThemingConfiguration = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, config);
    }
    
    // Test for RawToken's skip_serializing_if for id field
    #[test]
    fn raw_token_id_serialization_logic() {
        // This test is a bit conceptual as `is_default_id_from_key` is simple.
        // If TokenIdentifier::default() was `TokenIdentifier("")`
        let default_id_token = RawToken {
            id: TokenIdentifier("".into()), // Assuming this is the "default" that would be skipped
            value: TokenValue::Color("red".to_string()),
            description: None,
            group: None,
        };
        // If `id` field were truly optional or handled by BTreeMap key context,
        // this test would verify that.
        // With `#[serde(default, skip_serializing_if = "is_default_id_from_key")]`,
        // if `id` is empty, it should be skipped.
        let serialized = serde_json::to_string(&default_id_token).unwrap();
        if is_default_id_from_key(&default_id_token.id) {
            assert!(!serialized.contains("\"id\":"));
        } else {
            assert!(serialized.contains(&format!("\"id\":\"{}\"", default_id_token.id.as_str())));
        }

        let non_default_id_token = RawToken {
            id: TokenIdentifier("my-id".into()),
            value: TokenValue::Color("blue".to_string()),
            description: None,
            group: None,
        };
        let serialized_non_default = serde_json::to_string(&non_default_id_token).unwrap();
        assert!(serialized_non_default.contains("\"id\":\"my-id\""));
    }
}
