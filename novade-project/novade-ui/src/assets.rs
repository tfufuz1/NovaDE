//! Assets module for the NovaDE UI layer.
//!
//! This module provides asset management functionality for the NovaDE UI layer.

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use iced::widget::image::Handle;
use crate::error::{UiError, UiResult, to_ui_error, UiErrorKind};

/// Asset manager.
pub struct AssetManager {
    /// The asset cache.
    cache: Arc<Mutex<HashMap<String, Handle>>>,
    /// The asset directory.
    asset_dir: PathBuf,
}

impl AssetManager {
    /// Creates a new asset manager.
    ///
    /// # Returns
    ///
    /// A new asset manager.
    pub fn new() -> Self {
        // In a real implementation, this would determine the asset directory
        // For now, we'll use a placeholder directory
        let asset_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/usr/share/novade"))
            .join("assets");
        
        AssetManager {
            cache: Arc::new(Mutex::new(HashMap::new())),
            asset_dir,
        }
    }
    
    /// Creates a new asset manager with a custom asset directory.
    ///
    /// # Arguments
    ///
    /// * `asset_dir` - The asset directory
    ///
    /// # Returns
    ///
    /// A new asset manager.
    pub fn with_asset_dir(asset_dir: impl Into<PathBuf>) -> Self {
        AssetManager {
            cache: Arc::new(Mutex::new(HashMap::new())),
            asset_dir: asset_dir.into(),
        }
    }
    
    /// Gets an image asset.
    ///
    /// # Arguments
    ///
    /// * `path` - The asset path
    ///
    /// # Returns
    ///
    /// The image handle, or an error if loading failed.
    pub fn get_image(&self, path: impl AsRef<Path>) -> UiResult<Handle> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // Check if the asset is already cached
        {
            let cache = self.cache.lock().unwrap();
            if let Some(handle) = cache.get(&path_str) {
                return Ok(handle.clone());
            }
        }
        
        // Load the asset
        let asset_path = self.asset_dir.join(&path_str);
        
        // In a real implementation, this would load the image from the file system
        // For now, we'll just return an error
        Err(to_ui_error(
            format!("Failed to load image: {}", path_str),
            UiErrorKind::AssetLoad,
        ))
    }
    
    /// Gets a placeholder icon.
    ///
    /// # Returns
    ///
    /// A placeholder icon handle.
    pub fn get_placeholder_icon(&self) -> Handle {
        // In a real implementation, this would load a placeholder icon
        // For now, we'll just return an empty handle
        Handle::from_pixels(16, 16, vec![0; 16 * 16 * 4])
    }
    
    /// Preloads assets.
    ///
    /// # Arguments
    ///
    /// * `paths` - The asset paths to preload
    ///
    /// # Returns
    ///
    /// `Ok(())` if all assets were preloaded, or an error if loading failed.
    pub fn preload(&self, paths: &[impl AsRef<Path>]) -> UiResult<()> {
        for path in paths {
            self.get_image(path)?;
        }
        
        Ok(())
    }
    
    /// Clears the asset cache.
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}
