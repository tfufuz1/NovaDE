use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use novade_core::types::Color as CoreColor;

// --- TokenIdentifier ---
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TokenIdentifier(String);

impl TokenIdentifier {
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TokenValue {
    Color(String),
    Dimension(String),
    FontFamily(String),
    FontWeight(String),
    FontSize(String),
    LetterSpacing(String),
    LineHeight(String),
    Border(String),
    Shadow(String),
    Opacity(f64),
    Number(f64),
    String(String),
    Reference(TokenIdentifier),
}

// --- RawToken ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawToken {
    #[serde(default, skip_serializing_if = "is_default_id_from_key")] // Potentially skip if ID is map key
    pub id: TokenIdentifier, 
    pub value: TokenValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

// Helper for RawToken serde: if ID is same as key in map, it might be omitted.
// This is a simple default check; BTreeMap might not need this if id is always present.
#[allow(clippy::trivially_copy_pass_by_ref)] // Cloned anyway by TokenIdentifier
fn is_default_id_from_key(id: &TokenIdentifier) -> bool {
    id.as_str().is_empty() // Assuming default TokenIdentifier is empty, adjust if different
}


// --- TokenSet ---
pub type TokenSet = BTreeMap<TokenIdentifier, RawToken>;

// --- ThemeIdentifier ---
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ThemeIdentifier(String);

impl ThemeIdentifier {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ColorSchemeType {
    #[default]
    Light,
    Dark,
}

// --- AccentColor ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccentColor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: CoreColor,
}
// Note: CoreColor might require custom Eq/Hash if it contains f32 and is used in HashMaps directly.
// For AccentColor itself, PartialEq is derived. If AccentColor is used as a key, this becomes relevant.

// --- ThemeVariantDefinition ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeVariantDefinition {
    pub applies_to_scheme: ColorSchemeType,
    pub tokens: TokenSet,
}

// --- AccentModificationType ---
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccentModificationType {
    DirectReplace,
    Lighten(f32), // Factor 0.0 to 1.0
    Darken(f32),  // Factor 0.0 to 1.0
    // Opacity(f32), // Removed as per plan, can be added later if needed
    // Custom(String), // Removed as per plan
}

// --- ThemeDefinition ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub id: ThemeIdentifier,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    pub base_tokens: TokenSet,
    
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<ThemeVariantDefinition>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported_accent_colors: Option<Vec<AccentColor>>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accentable_tokens: Option<HashMap<TokenIdentifier, AccentModificationType>>,
}

// --- AppliedThemeState ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppliedThemeState {
    pub theme_id: ThemeIdentifier,
    pub color_scheme: ColorSchemeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_accent_color: Option<AccentColor>,
    pub resolved_tokens: BTreeMap<TokenIdentifier, String>,
}

// --- ThemingConfiguration ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemingConfiguration {
    pub selected_theme_id: ThemeIdentifier,
    pub preferred_color_scheme: ColorSchemeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_accent_color: Option<CoreColor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_user_token_overrides: Option<TokenSet>,
}

impl Default for ThemingConfiguration {
    fn default() -> Self {
        Self {
            selected_theme_id: ThemeIdentifier::new("default-system"),
            preferred_color_scheme: ColorSchemeType::default(),
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
