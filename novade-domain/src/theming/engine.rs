// This module is deprecated. Please use novade-domain/src/theming/service.rs::ThemingEngine instead.
//! Theming engine module for the NovaDE domain layer.
//!
//! This module provides the theming engine interface and implementation
//! for theme management in the NovaDE desktop environment.

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use crate::common_events::{DomainEvent, ThemingEvent};
use crate::error::{DomainResult, ThemingError};
use crate::theming::core::{Theme, ThemeId, ThemeVariant};
use crate::theming::tokens::{TokenValue, TokenResolutionContext};
use crate::theming::provider::ThemeProvider;

/// Interface for the theming engine.
#[async_trait]
#[deprecated(note = "Use novade-domain/src/theming/service.rs::ThemingEngine instead")]
pub trait ThemingEngine: Send + Sync {
    /// Gets a theme by ID.
    ///
    /// # Arguments
    ///
    /// * `theme_id` - The ID of the theme
    ///
    /// # Returns
    ///
    /// The theme, or an error if it doesn't exist.
    async fn get_theme(&self, theme_id: ThemeId) -> DomainResult<Theme>;

    /// Gets all themes.
    ///
    /// # Returns
    ///
    /// A vector of all themes.
    async fn get_all_themes(&self) -> DomainResult<Vec<Theme>>;

    /// Gets themes by variant.
    ///
    /// # Arguments
    ///
    /// * `variant` - The variant of the themes to get
    ///
    /// # Returns
    ///
    /// A vector of themes with the specified variant.
    async fn get_themes_by_variant(&self, variant: ThemeVariant) -> DomainResult<Vec<Theme>>;

    /// Creates a new theme.
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme to create
    ///
    /// # Returns
    ///
    /// The created theme.
    async fn create_theme(&self, theme: Theme) -> DomainResult<Theme>;

    /// Updates a theme.
    ///
    /// # Arguments
    ///
    /// * `theme` - The updated theme
    ///
    /// # Returns
    ///
    /// The updated theme.
    async fn update_theme(&self, theme: Theme) -> DomainResult<Theme>;

    /// Deletes a theme.
    ///
    /// # Arguments
    ///
    /// * `theme_id` - The ID of the theme to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` if the theme was deleted, or an error if it doesn't exist.
    async fn delete_theme(&self, theme_id: ThemeId) -> DomainResult<()>;

    /// Gets the active theme.
    ///
    /// # Returns
    ///
    /// The active theme, or an error if no theme is active.
    async fn get_active_theme(&self) -> DomainResult<Theme>;

    /// Sets the active theme.
    ///
    /// # Arguments
    ///
    /// * `theme_id` - The ID of the theme to set as active
    ///
    /// # Returns
    ///
    /// The activated theme.
    async fn set_active_theme(&self, theme_id: ThemeId) -> DomainResult<Theme>;

    /// Resolves a token value.
    ///
    /// # Arguments
    ///
    /// * `token_path` - The path of the token to resolve
    ///
    /// # Returns
    ///
    /// The resolved token value, or an error if the token doesn't exist.
    async fn resolve_token(&self, token_path: &str) -> DomainResult<TokenValue>;

    /// Resolves all tokens.
    ///
    /// # Returns
    ///
    /// A map of token paths to resolved token values.
    async fn resolve_all_tokens(&self) -> DomainResult<HashMap<String, TokenValue>>;
}

/// Default implementation of the theming engine.
#[deprecated(note = "Use novade-domain/src/theming/service.rs::ThemingEngine instead")]
pub struct DefaultThemingEngine {
    /// The themes, keyed by ID.
    themes: Arc<RwLock<HashMap<ThemeId, Theme>>>,
    /// The active theme ID.
    active_theme_id: Arc<RwLock<Option<ThemeId>>>,
    /// The theme provider.
    theme_provider: Arc<dyn ThemeProvider>,
    /// The event publisher function.
    event_publisher: Box<dyn Fn(DomainEvent<ThemingEvent>) + Send + Sync>,
}

impl DefaultThemingEngine {
    /// Creates a new default theming engine.
    ///
    /// # Arguments
    ///
    /// * `theme_provider` - The theme provider
    /// * `event_publisher` - A function to publish theming events
    ///
    /// # Returns
    ///
    /// A new `DefaultThemingEngine`.
    pub fn new<F>(
        theme_provider: Arc<dyn ThemeProvider>,
        event_publisher: F,
    ) -> Self
    where
        F: Fn(DomainEvent<ThemingEvent>) + Send + Sync + 'static,
    {
        DefaultThemingEngine {
            themes: Arc::new(RwLock::new(HashMap::new())),
            active_theme_id: Arc::new(RwLock::new(None)),
            theme_provider,
            event_publisher: Box::new(event_publisher),
        }
    }

    /// Initializes the theming engine with default themes.
    ///
    /// # Returns
    ///
    /// `Ok(())` if initialization was successful, or an error if it failed.
    pub async fn initialize(&self) -> DomainResult<()> {
        // Load themes from the provider
        let provider_themes = self.theme_provider.load_themes().await?;
        
        {
            let mut themes = self.themes.write().unwrap();
            
            // Add provider themes
            for theme in provider_themes {
                themes.insert(theme.id(), theme);
            }
            
            // Add default themes if they don't exist
            if !themes.values().any(|t| t.metadata().variant == ThemeVariant::Light) {
                let light_theme = Theme::default_light();
                themes.insert(light_theme.id(), light_theme);
            }
            
            if !themes.values().any(|t| t.metadata().variant == ThemeVariant::Dark) {
                let dark_theme = Theme::default_dark();
                themes.insert(dark_theme.id(), dark_theme);
            }
            
            if !themes.values().any(|t| t.metadata().variant == ThemeVariant::HighContrast) {
                let high_contrast_theme = Theme::default_high_contrast();
                themes.insert(high_contrast_theme.id(), high_contrast_theme);
            }
        }
        
        // Set the active theme if none is set
        {
            let active_theme_id = self.active_theme_id.read().unwrap().clone();
            
            if active_theme_id.is_none() {
                // Find a light theme to set as active
                let themes = self.themes.read().unwrap();
                let light_theme = themes.values()
                    .find(|t| t.metadata().variant == ThemeVariant::Light)
                    .cloned();
                
                if let Some(theme) = light_theme {
                    drop(themes);
                    self.set_active_theme(theme.id()).await?;
                }
            }
        }
        
        Ok(())
    }

    /// Publishes a theming event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to publish
    fn publish_event(&self, event: ThemingEvent) {
        let domain_event = DomainEvent::new(event, "ThemingEngine");
        (self.event_publisher)(domain_event);
    }
}

#[async_trait]
impl ThemingEngine for DefaultThemingEngine {
    async fn get_theme(&self, theme_id: ThemeId) -> DomainResult<Theme> {
        let themes = self.themes.read().unwrap();
        
        themes
            .get(&theme_id)
            .cloned()
            .ok_or_else(|| ThemingError::NotFound(theme_id.to_string()).into())
    }

    async fn get_all_themes(&self) -> DomainResult<Vec<Theme>> {
        let themes = self.themes.read().unwrap();
        
        let mut result: Vec<Theme> = themes.values().cloned().collect();
        result.sort_by(|a, b| a.metadata().name.cmp(&b.metadata().name));
        
        Ok(result)
    }

    async fn get_themes_by_variant(&self, variant: ThemeVariant) -> DomainResult<Vec<Theme>> {
        let themes = self.themes.read().unwrap();
        
        let mut result: Vec<Theme> = themes.values()
            .filter(|t| t.metadata().variant == variant)
            .cloned()
            .collect();
        
        result.sort_by(|a, b| a.metadata().name.cmp(&b.metadata().name));
        
        Ok(result)
    }

    async fn create_theme(&self, theme: Theme) -> DomainResult<Theme> {
        theme.validate()?;
        
        let theme_id = theme.id();
        let name = theme.metadata().name.clone();
        
        {
            let mut themes = self.themes.write().unwrap();
            themes.insert(theme_id, theme.clone());
        }
        
        // Save the theme to the provider
        self.theme_provider.save_theme(&theme).await?;
        
        self.publish_event(ThemingEvent::ThemeLoaded {
            theme_id,
            name,
        });
        
        Ok(theme)
    }

    async fn update_theme(&self, theme: Theme) -> DomainResult<Theme> {
        theme.validate()?;
        
        let theme_id = theme.id();
        let name = theme.metadata().name.clone();
        
        {
            let mut themes = self.themes.write().unwrap();
            
            if !themes.contains_key(&theme_id) {
                return Err(ThemingError::NotFound(theme_id.to_string()).into());
            }
            
            themes.insert(theme_id, theme.clone());
        }
        
        // Save the theme to the provider
        self.theme_provider.save_theme(&theme).await?;
        
        self.publish_event(ThemingEvent::ThemeUpdated {
            theme_id,
            name,
        });
        
        Ok(theme)
    }

    async fn delete_theme(&self, theme_id: ThemeId) -> DomainResult<()> {
        // Check if this is the active theme
        {
            let active_theme_id = self.active_theme_id.read().unwrap().clone();
            
            if active_theme_id == Some(theme_id) {
                return Err(ThemingError::Invalid(format!("Cannot delete the active theme {}", theme_id)).into());
            }
        }
        
        // Remove the theme
        let theme = {
            let mut themes = self.themes.write().unwrap();
            
            if !themes.contains_key(&theme_id) {
                return Err(ThemingError::NotFound(theme_id.to_string()).into());
            }
            
            themes.remove(&theme_id).unwrap()
        };
        
        // Delete the theme from the provider
        self.theme_provider.delete_theme(theme_id).await?;
        
        self.publish_event(ThemingEvent::ThemeDeleted {
            theme_id,
        });
        
        Ok(())
    }

    async fn get_active_theme(&self) -> DomainResult<Theme> {
        let active_id = {
            let active_id = self.active_theme_id.read().unwrap().clone();
            active_id.ok_or_else(|| ThemingError::Invalid("No active theme".to_string()))?
        };
        
        self.get_theme(active_id).await
    }

    async fn set_active_theme(&self, theme_id: ThemeId) -> DomainResult<Theme> {
        // Verify the theme exists
        let theme = self.get_theme(theme_id).await?;
        
        // Set the active theme ID
        {
            let mut active_theme_id = self.active_theme_id.write().unwrap();
            *active_theme_id = Some(theme_id);
        }
        
        self.publish_event(ThemingEvent::ThemeApplied {
            theme_id,
            name: theme.metadata().name.clone(),
        });
        
        Ok(theme)
    }

    async fn resolve_token(&self, token_path: &str) -> DomainResult<TokenValue> {
        let active_theme = self.get_active_theme().await?;
        
        // Get the token from the active theme
        let token = active_theme.get_token(token_path)
            .ok_or_else(|| ThemingError::TokenNotFound(token_path.to_string()))?;
        
        // Create a resolution context
        let mut tokens = HashMap::new();
        for (path, token) in active_theme.tokens() {
            tokens.insert(path.clone(), token.value().clone());
        }
        
        // If the active theme has a parent, add its tokens
        if let Some(parent_id) = active_theme.parent_id() {
            if let Ok(parent_theme) = self.get_theme(parent_id).await {
                for (path, token) in parent_theme.tokens() {
                    if !tokens.contains_key(path) {
                        tokens.insert(path.clone(), token.value().clone());
                    }
                }
            }
        }
        
        let context = TokenResolutionContext::new(tokens, 10);
        
        // Resolve the token
        let resolved = context.resolve(&token.value());
        
        Ok(resolved)
    }

    async fn resolve_all_tokens(&self) -> DomainResult<HashMap<String, TokenValue>> {
        let active_theme = self.get_active_theme().await?;
        
        // Collect all token paths
        let mut token_paths = HashSet::new();
        for path in active_theme.tokens().keys() {
            token_paths.insert(path.clone());
        }
        
        // If the active theme has a parent, add its token paths
        if let Some(parent_id) = active_theme.parent_id() {
            if let Ok(parent_theme) = self.get_theme(parent_id).await {
                for path in parent_theme.tokens().keys() {
                    token_paths.insert(path.clone());
                }
            }
        }
        
        // Resolve each token
        let mut result = HashMap::new();
        for path in token_paths {
            if let Ok(value) = self.resolve_token(&path).await {
                result.insert(path, value);
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use mockall::predicate::*;
    use mockall::mock;
    
    mock! {
        ThemeProvider {}
        
        #[async_trait]
        impl ThemeProvider for ThemeProvider {
            async fn load_themes(&self) -> DomainResult<Vec<Theme>>;
            async fn save_theme(&self, theme: &Theme) -> DomainResult<()>;
            async fn delete_theme(&self, theme_id: ThemeId) -> DomainResult<()>;
        }
    }
    
    struct TestContext {
        engine: DefaultThemingEngine,
        provider: Arc<MockThemeProvider>,
        events: Arc<Mutex<Vec<ThemingEvent>>>,
    }
    
    impl TestContext {
        fn new() -> Self {
            let events = Arc::new(Mutex::new(Vec::new()));
            let events_clone = events.clone();
            
            let provider = Arc::new(MockThemeProvider::new());
            
            let engine = DefaultThemingEngine::new(
                provider.clone(),
                move |event| {
                    let mut events = events_clone.lock().unwrap();
                    events.push(event.payload);
                },
            );
            
            TestContext {
                engine,
                provider,
                events,
            }
        }
        
        fn get_events(&self) -> Vec<ThemingEvent> {
            let events = self.events.lock().unwrap();
            events.clone()
        }
        
        fn clear_events(&self) {
            let mut events = self.events.lock().unwrap();
            events.clear();
        }
    }
    
    #[tokio::test]
    async fn test_create_theme() {
        let ctx = TestContext::new();
        
        ctx.provider
            .expect_save_theme()
            .returning(|_| Ok(()));
        
        let metadata = crate::theming::core::ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );
        
        let theme = Theme::new(metadata);
        let theme_id = theme.id();
        
        let created = ctx.engine.create_theme(theme.clone()).await.unwrap();
        
        assert_eq!(created.id(), theme_id);
        
        let retrieved = ctx.engine.get_theme(theme_id).await.unwrap();
        assert_eq!(retrieved.id(), theme_id);
        
        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            ThemingEvent::ThemeLoaded { theme_id: id, name } => {
                assert_eq!(*id, theme_id);
                assert_eq!(name, "Test Theme");
            },
            _ => panic!("Expected ThemeLoaded event"),
        }
    }
    
    #[tokio::test]
    async fn test_update_theme() {
        let ctx = TestContext::new();
        
        ctx.provider
            .expect_save_theme()
            .returning(|_| Ok(()));
        
        let metadata = crate::theming::core::ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );
        
        let mut theme = Theme::new(metadata);
        let theme_id = theme.id();
        
        ctx.engine.create_theme(theme.clone()).await.unwrap();
        ctx.clear_events();
        
        let new_metadata = crate::theming::core::ThemeMetadata::new(
            "Updated Theme",
            "An updated test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );
        
        theme.set_metadata(new_metadata);
        
        let updated = ctx.engine.update_theme(theme.clone()).await.unwrap();
        
        assert_eq!(updated.metadata().name, "Updated Theme");
        
        let retrieved = ctx.engine.get_theme(theme_id).await.unwrap();
        assert_eq!(retrieved.metadata().name, "Updated Theme");
        
        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            ThemingEvent::ThemeUpdated { theme_id: id, name } => {
                assert_eq!(*id, theme_id);
                assert_eq!(name, "Updated Theme");
            },
            _ => panic!("Expected ThemeUpdated event"),
        }
    }
    
    #[tokio::test]
    async fn test_delete_theme() {
        let ctx = TestContext::new();
        
        ctx.provider
            .expect_save_theme()
            .returning(|_| Ok(()));
        
        ctx.provider
            .expect_delete_theme()
            .returning(|_| Ok(()));
        
        let metadata = crate::theming::core::ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );
        
        let theme = Theme::new(metadata);
        let theme_id = theme.id();
        
        ctx.engine.create_theme(theme.clone()).await.unwrap();
        ctx.clear_events();
        
        ctx.engine.delete_theme(theme_id).await.unwrap();
        
        let result = ctx.engine.get_theme(theme_id).await;
        assert!(result.is_err());
        
        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            ThemingEvent::ThemeDeleted { theme_id: id } => {
                assert_eq!(*id, theme_id);
            },
            _ => panic!("Expected ThemeDeleted event"),
        }
    }
    
    #[tokio::test]
    async fn test_set_active_theme() {
        let ctx = TestContext::new();
        
        ctx.provider
            .expect_save_theme()
            .returning(|_| Ok(()));
        
        let metadata = crate::theming::core::ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );
        
        let theme = Theme::new(metadata);
        let theme_id = theme.id();
        
        ctx.engine.create_theme(theme.clone()).await.unwrap();
        ctx.clear_events();
        
        let activated = ctx.engine.set_active_theme(theme_id).await.unwrap();
        
        assert_eq!(activated.id(), theme_id);
        
        let active = ctx.engine.get_active_theme().await.unwrap();
        assert_eq!(active.id(), theme_id);
        
        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            ThemingEvent::ThemeApplied { theme_id: id, name } => {
                assert_eq!(*id, theme_id);
                assert_eq!(name, "Test Theme");
            },
            _ => panic!("Expected ThemeApplied event"),
        }
    }
    
    #[tokio::test]
    async fn test_resolve_token() {
        let ctx = TestContext::new();
        
        ctx.provider
            .expect_save_theme()
            .returning(|_| Ok(()));
        
        let metadata = crate::theming::core::ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );
        
        let mut theme = Theme::new(metadata);
        let theme_id = theme.id();
        
        // Add tokens
        theme.set_token("colors.primary", crate::theming::tokens::ThemeToken::new(TokenValue::Color("#FF0000".to_string())));
        theme.set_token("colors.secondary", crate::theming::tokens::ThemeToken::new(TokenValue::Reference("colors.primary".to_string())));
        
        ctx.engine.create_theme(theme.clone()).await.unwrap();
        ctx.engine.set_active_theme(theme_id).await.unwrap();
        
        // Resolve direct token
        let primary = ctx.engine.resolve_token("colors.primary").await.unwrap();
        assert_eq!(primary, TokenValue::Color("#FF0000".to_string()));
        
        // Resolve reference token
        let secondary = ctx.engine.resolve_token("colors.secondary").await.unwrap();
        assert_eq!(secondary, TokenValue::Color("#FF0000".to_string()));
        
        // Resolve non-existent token
        let result = ctx.engine.resolve_token("colors.nonexistent").await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_resolve_all_tokens() {
        let ctx = TestContext::new();
        
        ctx.provider
            .expect_save_theme()
            .returning(|_| Ok(()));
        
        let metadata = crate::theming::core::ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );
        
        let mut theme = Theme::new(metadata);
        let theme_id = theme.id();
        
        // Add tokens
        theme.set_token("colors.primary", crate::theming::tokens::ThemeToken::new(TokenValue::Color("#FF0000".to_string())));
        theme.set_token("colors.secondary", crate::theming::tokens::ThemeToken::new(TokenValue::Reference("colors.primary".to_string())));
        
        ctx.engine.create_theme(theme.clone()).await.unwrap();
        ctx.engine.set_active_theme(theme_id).await.unwrap();
        
        let tokens = ctx.engine.resolve_all_tokens().await.unwrap();
        
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens.get("colors.primary"), Some(&TokenValue::Color("#FF0000".to_string())));
        assert_eq!(tokens.get("colors.secondary"), Some(&TokenValue::Color("#FF0000".to_string())));
    }
    
    #[tokio::test]
    async fn test_initialize() {
        let ctx = TestContext::new();
        
        ctx.provider
            .expect_load_themes()
            .returning(|| Ok(Vec::new()));
        
        ctx.engine.initialize().await.unwrap();
        
        // Should have created default themes
        let themes = ctx.engine.get_all_themes().await.unwrap();
        assert!(themes.len() >= 3);
        
        // Should have set a light theme as active
        let active = ctx.engine.get_active_theme().await.unwrap();
        assert_eq!(active.metadata().variant, ThemeVariant::Light);
    }
}
