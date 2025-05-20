//! Core theming types for the NovaDE domain layer.
//!
//! This module provides the fundamental types and structures
//! for theme management in the NovaDE desktop environment.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::shared_types::{EntityId, Version, Identifiable, Versionable};
use crate::error::{DomainResult, ThemingError};
use crate::theming::tokens::{ThemeToken, TokenValue};

/// A unique identifier for themes.
pub type ThemeId = EntityId;

/// The variant of a theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeVariant {
    /// A light theme variant.
    Light,
    /// A dark theme variant.
    Dark,
    /// A high contrast theme variant.
    HighContrast,
}

impl fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeVariant::Light => write!(f, "Light"),
            ThemeVariant::Dark => write!(f, "Dark"),
            ThemeVariant::HighContrast => write!(f, "High Contrast"),
        }
    }
}

/// Metadata for a theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMetadata {
    /// The name of the theme.
    pub name: String,
    /// The description of the theme.
    pub description: String,
    /// The author of the theme.
    pub author: String,
    /// The version of the theme.
    pub version: String,
    /// The variant of the theme.
    pub variant: ThemeVariant,
    /// The license of the theme.
    pub license: Option<String>,
    /// The URL of the theme.
    pub url: Option<String>,
    /// Additional metadata.
    pub additional: HashMap<String, String>,
}

impl ThemeMetadata {
    /// Creates new theme metadata.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the theme
    /// * `description` - The description of the theme
    /// * `author` - The author of the theme
    /// * `version` - The version of the theme
    /// * `variant` - The variant of the theme
    ///
    /// # Returns
    ///
    /// New theme metadata.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
        version: impl Into<String>,
        variant: ThemeVariant,
    ) -> Self {
        ThemeMetadata {
            name: name.into(),
            description: description.into(),
            author: author.into(),
            version: version.into(),
            variant,
            license: None,
            url: None,
            additional: HashMap::new(),
        }
    }

    /// Sets the license of the theme.
    ///
    /// # Arguments
    ///
    /// * `license` - The license of the theme
    ///
    /// # Returns
    ///
    /// The modified metadata.
    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    /// Sets the URL of the theme.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the theme
    ///
    /// # Returns
    ///
    /// The modified metadata.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Adds additional metadata.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the metadata
    /// * `value` - The value of the metadata
    ///
    /// # Returns
    ///
    /// The modified metadata.
    pub fn with_additional(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional.insert(key.into(), value.into());
        self
    }
}

/// A theme in the NovaDE desktop environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// The unique identifier of the theme.
    id: ThemeId,
    /// The metadata of the theme.
    metadata: ThemeMetadata,
    /// The tokens of the theme.
    tokens: HashMap<String, ThemeToken>,
    /// The parent theme ID, if any.
    parent_id: Option<ThemeId>,
    /// The creation timestamp.
    created_at: DateTime<Utc>,
    /// The last update timestamp.
    updated_at: DateTime<Utc>,
    /// The version of the theme.
    version: Version,
}

impl Theme {
    /// Creates a new theme.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata of the theme
    ///
    /// # Returns
    ///
    /// A new theme.
    pub fn new(metadata: ThemeMetadata) -> Self {
        let now = Utc::now();
        Theme {
            id: ThemeId::new(),
            metadata,
            tokens: HashMap::new(),
            parent_id: None,
            created_at: now,
            updated_at: now,
            version: Version::initial(),
        }
    }

    /// Creates a new theme with the specified ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the theme
    /// * `metadata` - The metadata of the theme
    ///
    /// # Returns
    ///
    /// A new theme with the specified ID.
    pub fn with_id(id: ThemeId, metadata: ThemeMetadata) -> Self {
        let now = Utc::now();
        Theme {
            id,
            metadata,
            tokens: HashMap::new(),
            parent_id: None,
            created_at: now,
            updated_at: now,
            version: Version::initial(),
        }
    }

    /// Gets the metadata of the theme.
    pub fn metadata(&self) -> &ThemeMetadata {
        &self.metadata
    }

    /// Sets the metadata of the theme.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The new metadata of the theme
    pub fn set_metadata(&mut self, metadata: ThemeMetadata) {
        self.metadata = metadata;
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Gets the tokens of the theme.
    pub fn tokens(&self) -> &HashMap<String, ThemeToken> {
        &self.tokens
    }

    /// Gets a token by path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the token
    ///
    /// # Returns
    ///
    /// The token, or `None` if it doesn't exist.
    pub fn get_token(&self, path: &str) -> Option<&ThemeToken> {
        self.tokens.get(path)
    }

    /// Sets a token.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the token
    /// * `token` - The token to set
    pub fn set_token(&mut self, path: impl Into<String>, token: ThemeToken) {
        self.tokens.insert(path.into(), token);
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Removes a token.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the token
    ///
    /// # Returns
    ///
    /// `true` if the token was removed, `false` if it didn't exist.
    pub fn remove_token(&mut self, path: &str) -> bool {
        let result = self.tokens.remove(path).is_some();
        if result {
            self.updated_at = Utc::now();
            self.increment_version();
        }
        result
    }

    /// Gets the parent theme ID.
    pub fn parent_id(&self) -> Option<ThemeId> {
        self.parent_id
    }

    /// Sets the parent theme ID.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The new parent theme ID
    pub fn set_parent_id(&mut self, parent_id: Option<ThemeId>) {
        self.parent_id = parent_id;
        self.updated_at = Utc::now();
        self.increment_version();
    }

    /// Gets the creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Gets the last update timestamp.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Validates the theme.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the theme is valid, or an error if it is invalid.
    pub fn validate(&self) -> DomainResult<()> {
        if self.metadata.name.is_empty() {
            return Err(ThemingError::Invalid("Theme name cannot be empty".to_string()).into());
        }
        Ok(())
    }

    /// Creates a default light theme.
    ///
    /// # Returns
    ///
    /// A default light theme.
    pub fn default_light() -> Self {
        let metadata = ThemeMetadata::new(
            "Default Light",
            "The default light theme for NovaDE",
            "NovaDE Team",
            "1.0.0",
            ThemeVariant::Light,
        );

        let mut theme = Theme::new(metadata);

        // Add default tokens
        theme.set_token("colors.background", ThemeToken::new(TokenValue::Color("#FFFFFF".to_string())));
        theme.set_token("colors.foreground", ThemeToken::new(TokenValue::Color("#000000".to_string())));
        theme.set_token("colors.primary", ThemeToken::new(TokenValue::Color("#0078D7".to_string())));
        theme.set_token("colors.secondary", ThemeToken::new(TokenValue::Color("#6C757D".to_string())));
        theme.set_token("colors.success", ThemeToken::new(TokenValue::Color("#28A745".to_string())));
        theme.set_token("colors.danger", ThemeToken::new(TokenValue::Color("#DC3545".to_string())));
        theme.set_token("colors.warning", ThemeToken::new(TokenValue::Color("#FFC107".to_string())));
        theme.set_token("colors.info", ThemeToken::new(TokenValue::Color("#17A2B8".to_string())));

        theme.set_token("spacing.small", ThemeToken::new(TokenValue::Dimension("4px".to_string())));
        theme.set_token("spacing.medium", ThemeToken::new(TokenValue::Dimension("8px".to_string())));
        theme.set_token("spacing.large", ThemeToken::new(TokenValue::Dimension("16px".to_string())));

        theme.set_token("typography.fontFamily", ThemeToken::new(TokenValue::String("'Segoe UI', sans-serif".to_string())));
        theme.set_token("typography.fontSize", ThemeToken::new(TokenValue::Dimension("14px".to_string())));
        theme.set_token("typography.fontWeight", ThemeToken::new(TokenValue::String("normal".to_string())));
        theme.set_token("typography.lineHeight", ThemeToken::new(TokenValue::Number(1.5)));

        theme
    }

    /// Creates a default dark theme.
    ///
    /// # Returns
    ///
    /// A default dark theme.
    pub fn default_dark() -> Self {
        let metadata = ThemeMetadata::new(
            "Default Dark",
            "The default dark theme for NovaDE",
            "NovaDE Team",
            "1.0.0",
            ThemeVariant::Dark,
        );

        let mut theme = Theme::new(metadata);

        // Add default tokens
        theme.set_token("colors.background", ThemeToken::new(TokenValue::Color("#1E1E1E".to_string())));
        theme.set_token("colors.foreground", ThemeToken::new(TokenValue::Color("#FFFFFF".to_string())));
        theme.set_token("colors.primary", ThemeToken::new(TokenValue::Color("#0078D7".to_string())));
        theme.set_token("colors.secondary", ThemeToken::new(TokenValue::Color("#6C757D".to_string())));
        theme.set_token("colors.success", ThemeToken::new(TokenValue::Color("#28A745".to_string())));
        theme.set_token("colors.danger", ThemeToken::new(TokenValue::Color("#DC3545".to_string())));
        theme.set_token("colors.warning", ThemeToken::new(TokenValue::Color("#FFC107".to_string())));
        theme.set_token("colors.info", ThemeToken::new(TokenValue::Color("#17A2B8".to_string())));

        theme.set_token("spacing.small", ThemeToken::new(TokenValue::Dimension("4px".to_string())));
        theme.set_token("spacing.medium", ThemeToken::new(TokenValue::Dimension("8px".to_string())));
        theme.set_token("spacing.large", ThemeToken::new(TokenValue::Dimension("16px".to_string())));

        theme.set_token("typography.fontFamily", ThemeToken::new(TokenValue::String("'Segoe UI', sans-serif".to_string())));
        theme.set_token("typography.fontSize", ThemeToken::new(TokenValue::Dimension("14px".to_string())));
        theme.set_token("typography.fontWeight", ThemeToken::new(TokenValue::String("normal".to_string())));
        theme.set_token("typography.lineHeight", ThemeToken::new(TokenValue::Number(1.5)));

        theme
    }

    /// Creates a default high contrast theme.
    ///
    /// # Returns
    ///
    /// A default high contrast theme.
    pub fn default_high_contrast() -> Self {
        let metadata = ThemeMetadata::new(
            "Default High Contrast",
            "The default high contrast theme for NovaDE",
            "NovaDE Team",
            "1.0.0",
            ThemeVariant::HighContrast,
        );

        let mut theme = Theme::new(metadata);

        // Add default tokens
        theme.set_token("colors.background", ThemeToken::new(TokenValue::Color("#000000".to_string())));
        theme.set_token("colors.foreground", ThemeToken::new(TokenValue::Color("#FFFFFF".to_string())));
        theme.set_token("colors.primary", ThemeToken::new(TokenValue::Color("#FFFF00".to_string())));
        theme.set_token("colors.secondary", ThemeToken::new(TokenValue::Color("#00FFFF".to_string())));
        theme.set_token("colors.success", ThemeToken::new(TokenValue::Color("#00FF00".to_string())));
        theme.set_token("colors.danger", ThemeToken::new(TokenValue::Color("#FF0000".to_string())));
        theme.set_token("colors.warning", ThemeToken::new(TokenValue::Color("#FFFF00".to_string())));
        theme.set_token("colors.info", ThemeToken::new(TokenValue::Color("#00FFFF".to_string())));

        theme.set_token("spacing.small", ThemeToken::new(TokenValue::Dimension("4px".to_string())));
        theme.set_token("spacing.medium", ThemeToken::new(TokenValue::Dimension("8px".to_string())));
        theme.set_token("spacing.large", ThemeToken::new(TokenValue::Dimension("16px".to_string())));

        theme.set_token("typography.fontFamily", ThemeToken::new(TokenValue::String("'Segoe UI', sans-serif".to_string())));
        theme.set_token("typography.fontSize", ThemeToken::new(TokenValue::Dimension("14px".to_string())));
        theme.set_token("typography.fontWeight", ThemeToken::new(TokenValue::String("bold".to_string())));
        theme.set_token("typography.lineHeight", ThemeToken::new(TokenValue::Number(1.5)));

        theme
    }
}

impl Identifiable for Theme {
    fn id(&self) -> EntityId {
        self.id
    }
}

impl Versionable for Theme {
    fn version(&self) -> Version {
        self.version
    }

    fn increment_version(&mut self) {
        self.version = self.version.next();
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Theme[{}] '{}' ({})",
            self.id, self.metadata.name, self.metadata.variant
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_variant_display() {
        assert_eq!(format!("{}", ThemeVariant::Light), "Light");
        assert_eq!(format!("{}", ThemeVariant::Dark), "Dark");
        assert_eq!(format!("{}", ThemeVariant::HighContrast), "High Contrast");
    }

    #[test]
    fn test_theme_metadata_new() {
        let metadata = ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );

        assert_eq!(metadata.name, "Test Theme");
        assert_eq!(metadata.description, "A test theme");
        assert_eq!(metadata.author, "Test Author");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.variant, ThemeVariant::Light);
        assert!(metadata.license.is_none());
        assert!(metadata.url.is_none());
        assert!(metadata.additional.is_empty());
    }

    #[test]
    fn test_theme_metadata_with_methods() {
        let metadata = ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        )
        .with_license("MIT")
        .with_url("https://example.com")
        .with_additional("key1", "value1")
        .with_additional("key2", "value2");

        assert_eq!(metadata.license, Some("MIT".to_string()));
        assert_eq!(metadata.url, Some("https://example.com".to_string()));
        assert_eq!(metadata.additional.len(), 2);
        assert_eq!(metadata.additional.get("key1"), Some(&"value1".to_string()));
        assert_eq!(metadata.additional.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_theme_new() {
        let metadata = ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );

        let theme = Theme::new(metadata.clone());

        assert_eq!(theme.metadata().name, metadata.name);
        assert!(theme.tokens().is_empty());
        assert!(theme.parent_id().is_none());
        assert_eq!(theme.version(), Version::initial());
    }

    #[test]
    fn test_theme_with_id() {
        let id = ThemeId::new();
        let metadata = ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );

        let theme = Theme::with_id(id, metadata.clone());

        assert_eq!(theme.id(), id);
        assert_eq!(theme.metadata().name, metadata.name);
    }

    #[test]
    fn test_theme_set_metadata() {
        let metadata1 = ThemeMetadata::new(
            "Test Theme 1",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );

        let metadata2 = ThemeMetadata::new(
            "Test Theme 2",
            "Another test theme",
            "Another Author",
            "2.0.0",
            ThemeVariant::Dark,
        );

        let mut theme = Theme::new(metadata1);
        let initial_version = theme.version();

        theme.set_metadata(metadata2.clone());

        assert_eq!(theme.metadata().name, metadata2.name);
        assert!(theme.version() > initial_version);
    }

    #[test]
    fn test_theme_token_operations() {
        let metadata = ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );

        let mut theme = Theme::new(metadata);
        let token = ThemeToken::new(TokenValue::Color("#FF0000".to_string()));

        // Set token
        theme.set_token("colors.primary", token.clone());
        assert_eq!(theme.tokens().len(), 1);
        assert_eq!(theme.get_token("colors.primary"), Some(&token));

        // Remove token
        let result = theme.remove_token("colors.primary");
        assert!(result);
        assert!(theme.tokens().is_empty());
        assert_eq!(theme.get_token("colors.primary"), None);

        // Remove non-existent token
        let result = theme.remove_token("colors.primary");
        assert!(!result);
    }

    #[test]
    fn test_theme_parent_id() {
        let metadata = ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );

        let mut theme = Theme::new(metadata);
        let parent_id = ThemeId::new();
        let initial_version = theme.version();

        theme.set_parent_id(Some(parent_id));

        assert_eq!(theme.parent_id(), Some(parent_id));
        assert!(theme.version() > initial_version);

        theme.set_parent_id(None);

        assert_eq!(theme.parent_id(), None);
    }

    #[test]
    fn test_theme_validate() {
        let metadata = ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );

        let theme = Theme::new(metadata);
        assert!(theme.validate().is_ok());

        let invalid_metadata = ThemeMetadata::new(
            "",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );

        let invalid_theme = Theme::new(invalid_metadata);
        assert!(invalid_theme.validate().is_err());
    }

    #[test]
    fn test_theme_default_light() {
        let theme = Theme::default_light();

        assert_eq!(theme.metadata().name, "Default Light");
        assert_eq!(theme.metadata().variant, ThemeVariant::Light);
        assert!(theme.tokens().len() > 0);
        assert!(theme.get_token("colors.background").is_some());
    }

    #[test]
    fn test_theme_default_dark() {
        let theme = Theme::default_dark();

        assert_eq!(theme.metadata().name, "Default Dark");
        assert_eq!(theme.metadata().variant, ThemeVariant::Dark);
        assert!(theme.tokens().len() > 0);
        assert!(theme.get_token("colors.background").is_some());
    }

    #[test]
    fn test_theme_default_high_contrast() {
        let theme = Theme::default_high_contrast();

        assert_eq!(theme.metadata().name, "Default High Contrast");
        assert_eq!(theme.metadata().variant, ThemeVariant::HighContrast);
        assert!(theme.tokens().len() > 0);
        assert!(theme.get_token("colors.background").is_some());
    }

    #[test]
    fn test_theme_display() {
        let metadata = ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );

        let theme = Theme::new(metadata);
        let display = format!("{}", theme);

        assert!(display.contains("Test Theme"));
        assert!(display.contains("Light"));
    }
}
