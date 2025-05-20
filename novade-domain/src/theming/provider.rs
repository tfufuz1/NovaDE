//! Theme provider module for the NovaDE domain layer.
//!
//! This module provides interfaces and implementations for loading
//! and saving themes in the NovaDE desktop environment.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use tokio::fs as tokio_fs;
use crate::error::{DomainResult, ThemingError};
use crate::theming::core::{Theme, ThemeId};

/// Interface for providing themes.
#[async_trait]
pub trait ThemeProvider: Send + Sync {
    /// Loads all available themes.
    ///
    /// # Returns
    ///
    /// A vector of themes, or an error if loading failed.
    async fn load_themes(&self) -> DomainResult<Vec<Theme>>;
    
    /// Saves a theme.
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme to save
    ///
    /// # Returns
    ///
    /// `Ok(())` if the theme was saved, or an error if saving failed.
    async fn save_theme(&self, theme: &Theme) -> DomainResult<()>;
    
    /// Deletes a theme.
    ///
    /// # Arguments
    ///
    /// * `theme_id` - The ID of the theme to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` if the theme was deleted, or an error if deletion failed.
    async fn delete_theme(&self, theme_id: ThemeId) -> DomainResult<()>;
}

/// File-based theme provider.
pub struct FileThemeProvider {
    /// The directory where themes are stored.
    themes_dir: PathBuf,
    /// Cache of loaded themes.
    theme_cache: HashMap<ThemeId, Theme>,
}

impl FileThemeProvider {
    /// Creates a new file-based theme provider.
    ///
    /// # Arguments
    ///
    /// * `themes_dir` - The directory where themes are stored
    ///
    /// # Returns
    ///
    /// A new `FileThemeProvider`.
    pub fn new(themes_dir: impl Into<PathBuf>) -> Self {
        let themes_dir = themes_dir.into();
        
        // Create the themes directory if it doesn't exist
        if !themes_dir.exists() {
            fs::create_dir_all(&themes_dir).unwrap_or_else(|e| {
                eprintln!("Failed to create themes directory: {}", e);
            });
        }
        
        FileThemeProvider {
            themes_dir,
            theme_cache: HashMap::new(),
        }
    }
    
    /// Gets the path to a theme file.
    ///
    /// # Arguments
    ///
    /// * `theme_id` - The ID of the theme
    ///
    /// # Returns
    ///
    /// The path to the theme file.
    fn get_theme_path(&self, theme_id: ThemeId) -> PathBuf {
        self.themes_dir.join(format!("{}.json", theme_id))
    }
    
    /// Loads a theme from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the theme file
    ///
    /// # Returns
    ///
    /// The loaded theme, or an error if loading failed.
    async fn load_theme_from_file(&self, path: impl AsRef<Path>) -> DomainResult<Theme> {
        let content = tokio_fs::read_to_string(path.as_ref()).await
            .map_err(|e| ThemingError::LoadFailed(e.to_string()))?;
        
        let theme: Theme = serde_json::from_str(&content)
            .map_err(|e| ThemingError::LoadFailed(e.to_string()))?;
        
        Ok(theme)
    }
    
    /// Saves a theme to a file.
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme to save
    /// * `path` - The path to save the theme to
    ///
    /// # Returns
    ///
    /// `Ok(())` if the theme was saved, or an error if saving failed.
    async fn save_theme_to_file(&self, theme: &Theme, path: impl AsRef<Path>) -> DomainResult<()> {
        let content = serde_json::to_string_pretty(theme)
            .map_err(|e| ThemingError::SaveFailed(e.to_string()))?;
        
        tokio_fs::write(path.as_ref(), content).await
            .map_err(|e| ThemingError::SaveFailed(e.to_string()))?;
        
        Ok(())
    }
}

#[async_trait]
impl ThemeProvider for FileThemeProvider {
    async fn load_themes(&self) -> DomainResult<Vec<Theme>> {
        let mut themes = Vec::new();
        
        // Read the themes directory
        let entries = match tokio_fs::read_dir(&self.themes_dir).await {
            Ok(entries) => entries,
            Err(e) => {
                return Err(ThemingError::LoadFailed(format!("Failed to read themes directory: {}", e)).into());
            }
        };
        
        // Load each theme file
        let mut entry = entries;
        while let Ok(Some(file)) = entry.next_entry().await {
            let path = file.path();
            
            // Skip non-JSON files
            if path.extension().map_or(false, |ext| ext == "json") {
                match self.load_theme_from_file(&path).await {
                    Ok(theme) => {
                        themes.push(theme);
                    },
                    Err(e) => {
                        eprintln!("Failed to load theme from {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        Ok(themes)
    }
    
    async fn save_theme(&self, theme: &Theme) -> DomainResult<()> {
        let path = self.get_theme_path(theme.id());
        self.save_theme_to_file(theme, path).await
    }
    
    async fn delete_theme(&self, theme_id: ThemeId) -> DomainResult<()> {
        let path = self.get_theme_path(theme_id);
        
        if path.exists() {
            tokio_fs::remove_file(path).await
                .map_err(|e| ThemingError::DeleteFailed(e.to_string()))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::theming::core::{ThemeMetadata, ThemeVariant};
    
    #[tokio::test]
    async fn test_file_theme_provider() {
        let temp_dir = TempDir::new().unwrap();
        let provider = FileThemeProvider::new(temp_dir.path());
        
        // Initially, no themes
        let themes = provider.load_themes().await.unwrap();
        assert!(themes.is_empty());
        
        // Create a theme
        let metadata = ThemeMetadata::new(
            "Test Theme",
            "A test theme",
            "Test Author",
            "1.0.0",
            ThemeVariant::Light,
        );
        
        let theme = Theme::new(metadata);
        let theme_id = theme.id();
        
        // Save the theme
        provider.save_theme(&theme).await.unwrap();
        
        // Load themes again
        let themes = provider.load_themes().await.unwrap();
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].id(), theme_id);
        
        // Delete the theme
        provider.delete_theme(theme_id).await.unwrap();
        
        // Load themes again
        let themes = provider.load_themes().await.unwrap();
        assert!(themes.is_empty());
    }
}
