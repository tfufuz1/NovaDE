//! Default implementation of the theme manager.
//!
//! This module provides a default implementation of the theme manager
//! for the NovaDE desktop environment.

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use async_trait::async_trait;
use uuid::Uuid;
use serde_json;
use crate::error::{DomainError, ThemingError};
use crate::entities::value_objects::Timestamp;
use crate::types::color::Color;
use super::{ThemeManager, Theme, ThemeVariant, ThemeComponentType};
use super::themes::{DefaultLightTheme, DefaultDarkTheme, HighContrastTheme};

/// Default implementation of the theme manager.
pub struct DefaultThemeManager {
    themes: HashMap<String, Theme>,
    active_theme_id: String,
}

impl DefaultThemeManager {
    /// Creates a new default theme manager.
    pub fn new() -> Result<Self, DomainError> {
        let mut manager = Self {
            themes: HashMap::new(),
            active_theme_id: String::new(),
        };
        
        // Create default themes
        let light_theme = DefaultLightTheme::create();
        let dark_theme = DefaultDarkTheme::create();
        let high_contrast_theme = HighContrastTheme::create();
        
        // Add default themes
        let light_id = manager.create_theme(
            &light_theme.name,
            &light_theme.description,
            light_theme.variant,
            &light_theme.author,
            &light_theme.version,
            light_theme.components,
            light_theme.properties,
        ).await?;
        
        let dark_id = manager.create_theme(
            &dark_theme.name,
            &dark_theme.description,
            dark_theme.variant,
            &dark_theme.author,
            &dark_theme.version,
            dark_theme.components,
            dark_theme.properties,
        ).await?;
        
        let high_contrast_id = manager.create_theme(
            &high_contrast_theme.name,
            &high_contrast_theme.description,
            high_contrast_theme.variant,
            &high_contrast_theme.author,
            &high_contrast_theme.version,
            high_contrast_theme.components,
            high_contrast_theme.properties,
        ).await?;
        
        // Set default light theme as active
        manager.set_active_theme(&light_id).await?;
        
        Ok(manager)
    }
}

#[async_trait]
impl ThemeManager for DefaultThemeManager {
    async fn create_theme(
        &self,
        name: &str,
        description: &str,
        variant: ThemeVariant,
        author: &str,
        version: &str,
        components: HashMap<ThemeComponentType, Color>,
        properties: HashMap<String, String>,
    ) -> Result<String, DomainError> {
        let theme_id = Uuid::new_v4().to_string();
        let now = Timestamp::now();
        
        let theme = Theme {
            theme_id: theme_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            variant,
            author: author.to_string(),
            version: version.to_string(),
            components,
            properties,
            created_at: now,
            modified_at: now,
        };
        
        let mut themes = self.themes.clone();
        themes.insert(theme_id.clone(), theme);
        
        // Update self
        *self = Self {
            themes,
            active_theme_id: self.active_theme_id.clone(),
        };
        
        Ok(theme_id)
    }
    
    async fn get_theme(&self, theme_id: &str) -> Result<Theme, DomainError> {
        self.themes.get(theme_id)
            .cloned()
            .ok_or_else(|| ThemingError::ThemeNotFound(theme_id.to_string()).into())
    }
    
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
    ) -> Result<(), DomainError> {
        if !self.themes.contains_key(theme_id) {
            return Err(ThemingError::ThemeNotFound(theme_id.to_string()).into());
        }
        
        let mut themes = self.themes.clone();
        
        let now = Timestamp::now();
        let created_at = themes.get(theme_id).unwrap().created_at;
        
        let theme = Theme {
            theme_id: theme_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            variant,
            author: author.to_string(),
            version: version.to_string(),
            components,
            properties,
            created_at,
            modified_at: now,
        };
        
        themes.insert(theme_id.to_string(), theme);
        
        // Update self
        *self = Self {
            themes,
            active_theme_id: self.active_theme_id.clone(),
        };
        
        Ok(())
    }
    
    async fn delete_theme(&self, theme_id: &str) -> Result<(), DomainError> {
        if !self.themes.contains_key(theme_id) {
            return Err(ThemingError::ThemeNotFound(theme_id.to_string()).into());
        }
        
        if self.active_theme_id == theme_id {
            return Err(ThemingError::CannotDeleteActiveTheme.into());
        }
        
        let mut themes = self.themes.clone();
        themes.remove(theme_id);
        
        // Update self
        *self = Self {
            themes,
            active_theme_id: self.active_theme_id.clone(),
        };
        
        Ok(())
    }
    
    async fn list_themes(&self) -> Result<Vec<Theme>, DomainError> {
        Ok(self.themes.values().cloned().collect())
    }
    
    async fn list_themes_by_variant(&self, variant: ThemeVariant) -> Result<Vec<Theme>, DomainError> {
        Ok(self.themes.values()
            .filter(|t| t.variant == variant)
            .cloned()
            .collect())
    }
    
    async fn get_active_theme(&self) -> Result<Theme, DomainError> {
        self.get_theme(&self.active_theme_id).await
    }
    
    async fn set_active_theme(&self, theme_id: &str) -> Result<(), DomainError> {
        if !self.themes.contains_key(theme_id) {
            return Err(ThemingError::ThemeNotFound(theme_id.to_string()).into());
        }
        
        // Update self
        *self = Self {
            themes: self.themes.clone(),
            active_theme_id: theme_id.to_string(),
        };
        
        Ok(())
    }
    
    async fn get_component_color(&self, component_type: ThemeComponentType) -> Result<Color, DomainError> {
        let active_theme = self.get_active_theme().await?;
        
        active_theme.components.get(&component_type)
            .cloned()
            .ok_or_else(|| ThemingError::ComponentNotFound(format!("{:?}", component_type)).into())
    }
    
    async fn derive_theme(
        &self,
        base_theme_id: &str,
        name: &str,
        description: &str,
        author: &str,
        version: &str,
        component_overrides: HashMap<ThemeComponentType, Color>,
        property_overrides: HashMap<String, String>,
    ) -> Result<String, DomainError> {
        let base_theme = self.get_theme(base_theme_id).await?;
        
        // Merge components and properties
        let mut components = base_theme.components.clone();
        for (k, v) in component_overrides {
            components.insert(k, v);
        }
        
        let mut properties = base_theme.properties.clone();
        for (k, v) in property_overrides {
            properties.insert(k, v);
        }
        
        // Create new theme
        self.create_theme(
            name,
            description,
            base_theme.variant,
            author,
            version,
            components,
            properties,
        ).await
    }
    
    async fn import_theme(&self, file_path: &str) -> Result<String, DomainError> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(ThemingError::FileNotFound(file_path.to_string()).into());
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| ThemingError::FileReadError(e.to_string()))?;
        
        let theme: Theme = serde_json::from_str(&content)
            .map_err(|e| ThemingError::InvalidThemeFormat(e.to_string()))?;
        
        // Create new theme with imported data
        self.create_theme(
            &theme.name,
            &theme.description,
            theme.variant,
            &theme.author,
            &theme.version,
            theme.components,
            theme.properties,
        ).await
    }
    
    async fn export_theme(&self, theme_id: &str, file_path: &str) -> Result<(), DomainError> {
        let theme = self.get_theme(theme_id).await?;
        
        let content = serde_json::to_string_pretty(&theme)
            .map_err(|e| ThemingError::SerializationError(e.to_string()))?;
        
        fs::write(file_path, content)
            .map_err(|e| ThemingError::FileWriteError(e.to_string()))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_theme() {
        let manager = DefaultThemeManager::new().unwrap();
        
        let mut components = HashMap::new();
        components.insert(ThemeComponentType::Background, Color::rgb(255, 255, 255));
        components.insert(ThemeComponentType::Foreground, Color::rgb(0, 0, 0));
        
        let mut properties = HashMap::new();
        properties.insert("font-family".to_string(), "Arial".to_string());
        
        let theme_id = manager.create_theme(
            "Test Theme",
            "A test theme",
            ThemeVariant::Light,
            "Test Author",
            "1.0.0",
            components.clone(),
            properties.clone(),
        ).await.unwrap();
        
        assert!(!theme_id.is_empty());
        
        let theme = manager.get_theme(&theme_id).await.unwrap();
        assert_eq!(theme.name, "Test Theme");
        assert_eq!(theme.description, "A test theme");
        assert_eq!(theme.variant, ThemeVariant::Light);
        assert_eq!(theme.author, "Test Author");
        assert_eq!(theme.version, "1.0.0");
        assert_eq!(theme.components, components);
        assert_eq!(theme.properties, properties);
    }
    
    #[tokio::test]
    async fn test_update_theme() {
        let manager = DefaultThemeManager::new().unwrap();
        
        let mut components = HashMap::new();
        components.insert(ThemeComponentType::Background, Color::rgb(255, 255, 255));
        
        let theme_id = manager.create_theme(
            "Test Theme",
            "A test theme",
            ThemeVariant::Light,
            "Test Author",
            "1.0.0",
            components.clone(),
            HashMap::new(),
        ).await.unwrap();
        
        let mut updated_components = HashMap::new();
        updated_components.insert(ThemeComponentType::Background, Color::rgb(0, 0, 0));
        updated_components.insert(ThemeComponentType::Foreground, Color::rgb(255, 255, 255));
        
        let mut updated_properties = HashMap::new();
        updated_properties.insert("font-family".to_string(), "Arial".to_string());
        
        manager.update_theme(
            &theme_id,
            "Updated Theme",
            "An updated theme",
            ThemeVariant::Dark,
            "Updated Author",
            "2.0.0",
            updated_components.clone(),
            updated_properties.clone(),
        ).await.unwrap();
        
        let theme = manager.get_theme(&theme_id).await.unwrap();
        assert_eq!(theme.name, "Updated Theme");
        assert_eq!(theme.description, "An updated theme");
        assert_eq!(theme.variant, ThemeVariant::Dark);
        assert_eq!(theme.author, "Updated Author");
        assert_eq!(theme.version, "2.0.0");
        assert_eq!(theme.components, updated_components);
        assert_eq!(theme.properties, updated_properties);
    }
    
    #[tokio::test]
    async fn test_delete_theme() {
        let manager = DefaultThemeManager::new().unwrap();
        
        // Create a new theme
        let theme_id = manager.create_theme(
            "Test Theme",
            "A test theme",
            ThemeVariant::Light,
            "Test Author",
            "1.0.0",
            HashMap::new(),
            HashMap::new(),
        ).await.unwrap();
        
        // Delete the theme
        manager.delete_theme(&theme_id).await.unwrap();
        
        // Verify the theme is deleted
        let result = manager.get_theme(&theme_id).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_active_theme() {
        let manager = DefaultThemeManager::new().unwrap();
        
        // Create a new theme
        let theme_id = manager.create_theme(
            "Test Theme",
            "A test theme",
            ThemeVariant::Light,
            "Test Author",
            "1.0.0",
            HashMap::new(),
            HashMap::new(),
        ).await.unwrap();
        
        // Set as active
        manager.set_active_theme(&theme_id).await.unwrap();
        
        // Get active theme
        let active_theme = manager.get_active_theme().await.unwrap();
        assert_eq!(active_theme.theme_id, theme_id);
    }
    
    #[tokio::test]
    async fn test_derive_theme() {
        let manager = DefaultThemeManager::new().unwrap();
        
        // Create a base theme
        let mut base_components = HashMap::new();
        base_components.insert(ThemeComponentType::Background, Color::rgb(255, 255, 255));
        base_components.insert(ThemeComponentType::Foreground, Color::rgb(0, 0, 0));
        
        let mut base_properties = HashMap::new();
        base_properties.insert("font-family".to_string(), "Arial".to_string());
        
        let base_theme_id = manager.create_theme(
            "Base Theme",
            "A base theme",
            ThemeVariant::Light,
            "Test Author",
            "1.0.0",
            base_components.clone(),
            base_properties.clone(),
        ).await.unwrap();
        
        // Create component and property overrides
        let mut component_overrides = HashMap::new();
        component_overrides.insert(ThemeComponentType::Background, Color::rgb(240, 240, 240));
        component_overrides.insert(ThemeComponentType::Primary, Color::rgb(0, 120, 215));
        
        let mut property_overrides = HashMap::new();
        property_overrides.insert("font-size".to_string(), "14px".to_string());
        
        // Derive a new theme
        let derived_theme_id = manager.derive_theme(
            &base_theme_id,
            "Derived Theme",
            "A derived theme",
            "Derived Author",
            "1.0.0",
            component_overrides.clone(),
            property_overrides.clone(),
        ).await.unwrap();
        
        // Get the derived theme
        let derived_theme = manager.get_theme(&derived_theme_id).await.unwrap();
        
        // Check that overrides were applied
        assert_eq!(derived_theme.components.get(&ThemeComponentType::Background).unwrap(), &Color::rgb(240, 240, 240));
        assert_eq!(derived_theme.components.get(&ThemeComponentType::Primary).unwrap(), &Color::rgb(0, 120, 215));
        assert_eq!(derived_theme.components.get(&ThemeComponentType::Foreground).unwrap(), &Color::rgb(0, 0, 0));
        
        assert_eq!(derived_theme.properties.get("font-family").unwrap(), "Arial");
        assert_eq!(derived_theme.properties.get("font-size").unwrap(), "14px");
    }
    
    #[tokio::test]
    async fn test_list_themes_by_variant() {
        let manager = DefaultThemeManager::new().unwrap();
        
        // Create themes with different variants
        manager.create_theme(
            "Light Theme 1",
            "A light theme",
            ThemeVariant::Light,
            "Test Author",
            "1.0.0",
            HashMap::new(),
            HashMap::new(),
        ).await.unwrap();
        
        manager.create_theme(
            "Light Theme 2",
            "Another light theme",
            ThemeVariant::Light,
            "Test Author",
            "1.0.0",
            HashMap::new(),
            HashMap::new(),
        ).await.unwrap();
        
        manager.create_theme(
            "Dark Theme",
            "A dark theme",
            ThemeVariant::Dark,
            "Test Author",
            "1.0.0",
            HashMap::new(),
            HashMap::new(),
        ).await.unwrap();
        
        // List light themes
        let light_themes = manager.list_themes_by_variant(ThemeVariant::Light).await.unwrap();
        assert_eq!(light_themes.len(), 3); // 2 created + 1 default
        
        // List dark themes
        let dark_themes = manager.list_themes_by_variant(ThemeVariant::Dark).await.unwrap();
        assert_eq!(dark_themes.len(), 2); // 1 created + 1 default
    }
    
    #[tokio::test]
    async fn test_get_component_color() {
        let manager = DefaultThemeManager::new().unwrap();
        
        // Create a theme with specific components
        let mut components = HashMap::new();
        components.insert(ThemeComponentType::Background, Color::rgb(255, 255, 255));
        components.insert(ThemeComponentType::Foreground, Color::rgb(0, 0, 0));
        
        let theme_id = manager.create_theme(
            "Test Theme",
            "A test theme",
            ThemeVariant::Light,
            "Test Author",
            "1.0.0",
            components.clone(),
            HashMap::new(),
        ).await.unwrap();
        
        // Set as active
        manager.set_active_theme(&theme_id).await.unwrap();
        
        // Get component colors
        let background = manager.get_component_color(ThemeComponentType::Background).await.unwrap();
        let foreground = manager.get_component_color(ThemeComponentType::Foreground).await.unwrap();
        
        assert_eq!(background, Color::rgb(255, 255, 255));
        assert_eq!(foreground, Color::rgb(0, 0, 0));
    }
}
