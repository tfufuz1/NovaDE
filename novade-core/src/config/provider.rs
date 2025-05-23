//! Configuration Provider Interface and Implementations.
//!
//! This module defines the [`ConfigProvider`] trait, which abstracts how configuration
//! data is accessed and potentially monitored for changes. It also includes a
//! basic implementation, [`FileConfigProvider`], which serves configuration
//! that has been loaded into memory (e.g., from a file or default values).
//!
//! # Key Components
//! - [`ConfigProvider`]: A trait for components that can provide access to the [`CoreConfig`].
//! - [`FileConfigProvider`]: An implementation of `ConfigProvider` that holds a `CoreConfig` instance.
//!
//! **Note on `FileConfigProvider` Loading**: The `load()` and `load_from_path()` methods
//! previously associated with `FileConfigProvider` (when it was coupled with the old `FileConfigLoader`)
//! are currently commented out. The primary mechanism for loading configuration is now through
//! the static methods on [`crate::config::loader::ConfigLoader`]. `FileConfigProvider` can be
//! instantiated with a `CoreConfig` obtained from `ConfigLoader::load()` or with default values.

use crate::config::types::CoreConfig;
use crate::error::ConfigError;
use std::path::Path; // Path is used in the commented-out load_from_path

/// Defines an interface for components that provide access to the application's `CoreConfig`.
///
/// Implementors of this trait are responsible for holding the configuration state and
/// allowing other parts of the system to retrieve it. They might also offer mechanisms
/// for watching configuration changes, though this is not yet fully implemented in
/// the provided `FileConfigProvider`.
///
/// # Examples
///
/// ```rust,ignore
/// use novade_core::config::{ConfigProvider, CoreConfig, FileConfigProvider};
///
/// // Assume `config` is a loaded CoreConfig instance
/// let config = CoreConfig::default(); // Or loaded via ConfigLoader
/// let provider: Box<dyn ConfigProvider> = Box::new(FileConfigProvider::with_config(config));
///
/// // Accessing configuration
/// let current_log_level = provider.get_config().logging.level;
/// println!("Current log level: {}", current_log_level);
///
/// // Watching for changes (conceptual, as FileConfigProvider's watch is a placeholder)
/// provider.watch_config(|new_config| {
///     println!("Config changed! New log level: {}", new_config.logging.level);
/// }).expect("Failed to set up config watch");
/// ```
pub trait ConfigProvider {
    /// Returns an immutable reference to the current [`CoreConfig`].
    ///
    /// This method provides read-only access to the active configuration.
    fn get_config(&self) -> &CoreConfig;
    
    /// Registers a callback function to be invoked when the configuration changes.
    ///
    /// The provided `callback` will receive a reference to the new `CoreConfig`.
    ///
    /// # Arguments
    ///
    /// * `callback`: A closure or function that takes `&CoreConfig` and is `Send + 'static`.
    ///
    /// # Errors
    ///
    /// May return a [`ConfigError`] if setting up the watch mechanism fails.
    /// The current `FileConfigProvider` returns an error as watching is not implemented.
    fn watch_config<F>(&self, callback: F) -> Result<(), ConfigError>
    where
        F: Fn(&CoreConfig) + Send + 'static;
}

/// A simple configuration provider that holds a `CoreConfig` instance in memory.
///
/// This provider is typically initialized with a `CoreConfig` that has been loaded
/// and validated by [`crate::config::loader::ConfigLoader`]. It does not actively
/// monitor a configuration file for changes; its `watch_config` method currently
/// returns an error.
///
/// # Examples
///
/// ```
/// use novade_core::config::{FileConfigProvider, CoreConfig, ConfigProvider};
///
/// // Create with default configuration
/// let default_provider = FileConfigProvider::new();
/// assert_eq!(default_provider.get_config().logging.level, "info");
///
/// // Create with a specific configuration
/// let mut custom_config = CoreConfig::default();
/// custom_config.logging.level = "debug".to_string();
/// let custom_provider = FileConfigProvider::with_config(custom_config);
/// assert_eq!(custom_provider.get_config().logging.level, "debug");
/// ```
#[derive(Debug)]
pub struct FileConfigProvider {
    /// The `CoreConfig` instance held by this provider.
    config: CoreConfig,
}

impl FileConfigProvider {
    /// Creates a new `FileConfigProvider` initialized with the default `CoreConfig`.
    ///
    /// This is useful for scenarios where a configuration file might be missing or
    /// for tests that require a basic configuration provider.
    pub fn new() -> Self {
        FileConfigProvider {
            config: CoreConfig::default(),
        }
    }
    
    /// Creates a new `FileConfigProvider` with a specific, pre-loaded `CoreConfig`.
    ///
    /// This is the typical way to instantiate `FileConfigProvider` after loading
    /// configuration using [`crate::config::loader::ConfigLoader::load()`].
    ///
    /// # Arguments
    ///
    /// * `config`: The `CoreConfig` instance to be provided.
    pub fn with_config(config: CoreConfig) -> Self {
        FileConfigProvider {
            config,
        }
    }
    
    // The `load` and `load_from_path` methods that were previously part of `FileConfigProvider`
    // (when it was tightly coupled with `FileConfigLoader`) have been effectively superseded
    // by the static `ConfigLoader::load()` method.
    //
    // If `FileConfigProvider` needed to independently load its configuration, these methods
    // would need to be reimplemented to use `crate::config::loader::ConfigLoader::load()`
    // or a similar mechanism. However, this might introduce complexities or circular
    // dependencies depending on how `ConfigLoader` itself is structured or used.
    //
    // For now, `FileConfigProvider` is a simple holder of a `CoreConfig`.
    // Example of how `load` might be re-implemented if needed (conceptual):
    /*
    /// Loads configuration using `ConfigLoader::load()` and creates a new `FileConfigProvider`.
    pub fn load() -> Result<Self, CoreError> { // Note: Error type might be CoreError now
        let config = crate::config::loader::ConfigLoader::load()?;
        Ok(Self::with_config(config))
    }
    */
}

impl Default for FileConfigProvider {
    /// Returns a `FileConfigProvider` initialized with `CoreConfig::default()`.
    /// Equivalent to `FileConfigProvider::new()`.
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigProvider for FileConfigProvider {
    fn get_config(&self) -> &CoreConfig {
        &self.config
    }
    
    /// Currently not implemented for `FileConfigProvider`.
    ///
    /// Returns a [`ConfigError::Generic`] indicating that watching is not supported.
    fn watch_config<F>(&self, _callback: F) -> Result<(), ConfigError>
    where
        F: Fn(&CoreConfig) + Send + 'static
    {
        Err(ConfigError::Generic("Config watching not implemented for FileConfigProvider".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Minimal tests for FileConfigProvider, focusing on construction.
    // Load tests would require more significant mocking or setup without the old loader.
    
    #[test]
    fn test_file_config_provider_new() {
        let provider = FileConfigProvider::new();
        // Check against default CoreConfig.logging values from types.rs's Default impl
        let core_conf_default = CoreConfig::default();
        assert_eq!(provider.config.logging.level, core_conf_default.logging.level);
    }
    
    #[test]
    fn test_file_config_provider_with_config() {
        let mut config = CoreConfig::default();
        config.logging.level = "debug".to_string();
        
        let provider = FileConfigProvider::with_config(config.clone()); // clone config for test
        assert_eq!(provider.get_config().logging.level, "debug");
    }
        
    #[test]
    fn test_file_config_provider_get_config() {
        let provider = FileConfigProvider::new();
        let core_conf_default = CoreConfig::default();
        assert_eq!(provider.get_config().logging.level, core_conf_default.logging.level);
    }
    
    #[test]
    fn test_file_config_provider_watch_config() {
        let provider = FileConfigProvider::new();
        let result = provider.watch_config(|_| {});
        assert!(result.is_err());
        match result {
            Err(ConfigError::Generic(msg)) => assert_eq!(msg, "Config watching not implemented"),
            _ => panic!("Expected Generic error"),
        }
    }
}
