//! Theming module for the NovaDE domain layer.
//!
//! This module provides theming functionality for the NovaDE desktop environment,
//! allowing customization of the visual appearance of the desktop.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::error::{DomainError, ThemingError};
use crate::entities::value_objects::Timestamp;
use crate::types::color::Color;

mod themes;
mod default_theme_manager;

pub use themes::{
    default_light::DefaultLightTheme,
    default_dark::DefaultDarkTheme,
    high_contrast::HighContrastTheme,
};
pub use default_theme_manager::DefaultThemeManager;

/// Represents a theme component type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeComponentType {
    /// Background color
    Background,
    /// Foreground/text color
    Foreground,
    /// Primary accent color
    Primary,
    /// Secondary accent color
    Secondary,
    /// Success color
    Success,
    /// Warning color
    Warning,
    /// Error color
    Error,
    /// Info color
    Info,
    /// Border color
    Border,
    /// Shadow color
    Shadow,
    /// Hover state color
    Hover,
    /// Active state color
    Active,
    /// Disabled state color
    Disabled,
    /// Custom component type
    Custom(String),
}

/// Represents a theme variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeVariant {
    /// Light theme variant
    Light,
    /// Dark theme variant
    Dark,
    /// High contrast theme variant
    HighContrast,
    /// Custom theme variant
    Custom,
}

/// Represents a theme in the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    /// Unique identifier for the theme
    theme_id: String,
    /// The theme name
    name: String,
    /// The theme description
    description: String,
    /// The theme variant
    variant: ThemeVariant,
    /// The theme author
    author: String,
    /// The theme version
    version: String,
    /// The theme components
    components: HashMap<ThemeComponentType, Color>,
    /// Additional properties for the theme
    properties: HashMap<String, String>,
    /// The theme creation timestamp
    created_at: Timestamp,
    /// The theme last modified timestamp
    modified_at: Timestamp,
}

/// Interface for the theme manager.
#[async_trait]
pub trait ThemeManager: Send + Sync {
    /// Creates a new theme.
    ///
    /// # Arguments
    ///
    /// * `name` - The theme name
    /// * `description` - The theme description
    /// * `variant` - The theme variant
    /// * `author` - The theme author
    /// * `version` - The theme version
    /// * `components` - The theme components
    /// * `properties` - Additional properties for the theme
    ///
    /// # Returns
    ///
    /// A `Result` containing the created theme ID.
    async fn create_theme(
        &self,
        name: &str,
        description: &str,
        variant: ThemeVariant,
        author: &str,
        version: &str,
        components: HashMap<ThemeComponentType, Color>,
        properties: HashMap<String, String>,
    ) -> Result<String, DomainError>;
    
    /// Gets a theme by ID.
    ///
    /// # Arguments
    ///
    /// * `theme_id` - The theme ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the theme if found.
    async fn get_theme(&self, theme_id: &str) -> Result<Theme, DomainError>;
    
    /// Updates a theme.
    ///
    /// # Arguments
    ///
    /// * `theme_id` - The theme ID
    /// * `name` - The theme name
    /// * `description` - The theme description
    /// * `variant` - The theme variant
    /// * `author` - The theme author
    /// * `version` - The theme version
    /// * `components` - The theme components
    /// * `properties` - Additional properties for the theme
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn update_theme(
        &self,
        theme_id: &str,
        name: &str,
        description: &str,
        variant: ThemeVariant,
        author: &str,
        version: &str,
        components: HashMap<ThemeComponentType, Color>,
        properties: HashMap<String, String>,
    ) -> Result<(), DomainError>;
    
    /// Deletes a theme.
    ///
    /// # Arguments
    ///
    /// * `theme_id` - The theme ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn delete_theme(&self, theme_id: &str) -> Result<(), DomainError>;
    
    /// Lists all themes.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all themes.
    async fn list_themes(&self) -> Result<Vec<Theme>, DomainError>;
    
    /// Lists themes by variant.
    ///
    /// # Arguments
    ///
    /// * `variant` - The theme variant
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of themes with the specified variant.
    async fn list_themes_by_variant(&self, variant: ThemeVariant) -> Result<Vec<Theme>, DomainError>;
    
    /// Gets the active theme.
    ///
    /// # Returns
    ///
    /// A `Result` containing the active theme.
    async fn get_active_theme(&self) -> Result<Theme, DomainError>;
    
    /// Sets the active theme.
    ///
    /// # Arguments
    ///
    /// * `theme_id` - The theme ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn set_active_theme(&self, theme_id: &str) -> Result<(), DomainError>;
    
    /// Gets a theme component color.
    ///
    /// # Arguments
    ///
    /// * `component_type` - The component type
    ///
    /// # Returns
    ///
    /// A `Result` containing the component color.
    async fn get_component_color(&self, component_type: ThemeComponentType) -> Result<Color, DomainError>;
    
    /// Creates a derived theme from an existing theme.
    ///
    /// # Arguments
    ///
    /// * `base_theme_id` - The base theme ID
    /// * `name` - The new theme name
    /// * `description` - The new theme description
    /// * `author` - The new theme author
    /// * `version` - The new theme version
    /// * `component_overrides` - Component overrides for the new theme
    /// * `property_overrides` - Property overrides for the new theme
    ///
    /// # Returns
    ///
    /// A `Result` containing the created theme ID.
    async fn derive_theme(
        &self,
        base_theme_id: &str,
        name: &str,
        description: &str,
        author: &str,
        version: &str,
        component_overrides: HashMap<ThemeComponentType, Color>,
        property_overrides: HashMap<String, String>,
    ) -> Result<String, DomainError>;
    
    /// Imports a theme from a file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The file path
    ///
    /// # Returns
    ///
    /// A `Result` containing the imported theme ID.
    async fn import_theme(&self, file_path: &str) -> Result<String, DomainError>;
    
    /// Exports a theme to a file.
    ///
    /// # Arguments
    ///
    /// * `theme_id` - The theme ID
    /// * `file_path` - The file path
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn export_theme(&self, theme_id: &str, file_path: &str) -> Result<(), DomainError>;
}
