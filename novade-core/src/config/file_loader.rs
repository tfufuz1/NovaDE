//! File configuration loader for the NovaDE core layer.
//!
//! This module provides functionality for loading configuration from files
//! in the NovaDE desktop environment.

use std::fs;
use std::path::{Path, PathBuf};
use std::io;

use crate::error::ConfigError;
use crate::config::{CoreConfig, ConfigLoader};

/// File-based configuration loader.
///
/// This struct implements the `ConfigLoader` trait to load configuration
/// from TOML files.
#[derive(Debug)]
pub struct FileConfigLoader {
    /// The default configuration directory
    config_dir: PathBuf,
    /// The default configuration file name
    config_file: String,
}

impl FileConfigLoader {
    /// Creates a new file configuration loader with the default configuration path.
    ///
    /// # Returns
    ///
    /// A new `FileConfigLoader` with the default configuration path.
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let config_dir = PathBuf::from(format!("{}/.config/novade", home));
        
        FileConfigLoader {
            config_dir,
            config_file: "config.toml".to_string(),
        }
    }
    
    /// Creates a new file configuration loader with the specified configuration directory.
    ///
    /// # Arguments
    ///
    /// * `config_dir` - The configuration directory
    ///
    /// # Returns
    ///
    /// A new `FileConfigLoader` with the specified configuration directory.
    pub fn with_config_dir<P: AsRef<Path>>(config_dir: P) -> Self {
        FileConfigLoader {
            config_dir: config_dir.as_ref().to_path_buf(),
            config_file: "config.toml".to_string(),
        }
    }
    
    /// Creates a new file configuration loader with the specified configuration file name.
    ///
    /// # Arguments
    ///
    /// * `config_file` - The configuration file name
    ///
    /// # Returns
    ///
    /// A new `FileConfigLoader` with the specified configuration file name.
    pub fn with_config_file<S: Into<String>>(mut self, config_file: S) -> Self {
        self.config_file = config_file.into();
        self
    }
    
    /// Gets the default configuration file path.
    ///
    /// # Returns
    ///
    /// The default configuration file path.
    pub fn default_config_path(&self) -> PathBuf {
        self.config_dir.join(&self.config_file)
    }
    
    /// Parses TOML content into a `CoreConfig`.
    ///
    /// # Arguments
    ///
    /// * `content` - The TOML content to parse
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `CoreConfig` if successful,
    /// or a `ConfigError` if parsing failed.
    fn parse_toml(&self, content: &str) -> Result<CoreConfig, ConfigError> {
        toml::from_str(content).map_err(|e| ConfigError::ParseError(e))
    }
}

impl Default for FileConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader for FileConfigLoader {
    fn load_config(&self) -> Result<CoreConfig, ConfigError> {
        self.load_config_from_path(self.default_config_path())
    }
    
    fn load_config_from_path<P: AsRef<Path>>(&self, path: P) -> Result<CoreConfig, ConfigError> {
        let path = path.as_ref();
        
        // Check if the file exists
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.to_path_buf()));
        }
        
        // Read the file content
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::FileReadError(e))?;
        
        // Parse the TOML content
        self.parse_toml(&content)
    }
}

/// Configuration provider that loads configuration from a file.
///
/// This struct implements the `ConfigProvider` trait to provide access
/// to configuration loaded from a file.
#[derive(Debug)]
pub struct FileConfigProvider {
    /// The loaded configuration
    config: CoreConfig,
}

impl FileConfigProvider {
    /// Creates a new file configuration provider with the default configuration.
    ///
    /// # Returns
    ///
    /// A new `FileConfigProvider` with the default configuration.
    pub fn new() -> Self {
        FileConfigProvider {
            config: CoreConfig::default(),
        }
    }
    
    /// Creates a new file configuration provider with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use
    ///
    /// # Returns
    ///
    /// A new `FileConfigProvider` with the specified configuration.
    pub fn with_config(config: CoreConfig) -> Self {
        FileConfigProvider {
            config,
        }
    }
    
    /// Loads configuration from the default location.
    ///
    /// # Returns
    ///
    /// A `Result` containing `Self` if successful,
    /// or a `ConfigError` if loading failed.
    pub fn load() -> Result<Self, ConfigError> {
        let loader = FileConfigLoader::new();
        let config = match loader.load_config() {
            Ok(config) => config,
            Err(ConfigError::FileNotFound(_)) => CoreConfig::default(),
            Err(e) => return Err(e),
        };
        
        Ok(FileConfigProvider {
            config,
        })
    }
    
    /// Loads configuration from the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to load configuration from
    ///
    /// # Returns
    ///
    /// A `Result` containing `Self` if successful,
    /// or a `ConfigError` if loading failed.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let loader = FileConfigLoader::new();
        let config = match loader.load_config_from_path(path) {
            Ok(config) => config,
            Err(ConfigError::FileNotFound(_)) => CoreConfig::default(),
            Err(e) => return Err(e),
        };
        
        Ok(FileConfigProvider {
            config,
        })
    }
}

impl Default for FileConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::config::ConfigProvider for FileConfigProvider {
    fn get_config(&self) -> &CoreConfig {
        &self.config
    }
    
    fn watch_config<F>(&self, _callback: F) -> Result<(), ConfigError>
    where
        F: Fn(&CoreConfig) + Send + 'static
    {
        // File watching is not implemented yet
        Err(ConfigError::Generic("Config watching not implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;
    
    #[test]
    fn test_file_config_loader_new() {
        let loader = FileConfigLoader::new();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        assert_eq!(loader.config_dir, PathBuf::from(format!("{}/.config/novade", home)));
        assert_eq!(loader.config_file, "config.toml");
    }
    
    #[test]
    fn test_file_config_loader_with_config_dir() {
        let loader = FileConfigLoader::with_config_dir("/etc/novade");
        assert_eq!(loader.config_dir, PathBuf::from("/etc/novade"));
        assert_eq!(loader.config_file, "config.toml");
    }
    
    #[test]
    fn test_file_config_loader_with_config_file() {
        let loader = FileConfigLoader::new().with_config_file("custom.toml");
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        assert_eq!(loader.config_dir, PathBuf::from(format!("{}/.config/novade", home)));
        assert_eq!(loader.config_file, "custom.toml");
    }
    
    #[test]
    fn test_file_config_loader_default_config_path() {
        let loader = FileConfigLoader::new();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        assert_eq!(
            loader.default_config_path(),
            PathBuf::from(format!("{}/.config/novade/config.toml", home))
        );
    }
    
    #[test]
    fn test_file_config_loader_parse_toml() {
        let loader = FileConfigLoader::new();
        
        // Valid TOML
        let content = r#"
            [logging]
            level = "debug"
            log_to_file = true
            
            [application]
            name = "test-app"
        "#;
        
        let config = loader.parse_toml(content).unwrap();
        assert_eq!(config.logging.level, "debug");
        assert!(config.logging.log_to_file);
        assert_eq!(config.application.name, "test-app");
        
        // Invalid TOML
        let content = r#"
            [logging
            level = "debug"
        "#;
        
        assert!(loader.parse_toml(content).is_err());
    }
    
    #[test]
    fn test_file_config_loader_load_config_from_path() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Create a config file
        let content = r#"
            [logging]
            level = "debug"
            log_to_file = true
            
            [application]
            name = "test-app"
        "#;
        
        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        // Load the config
        let loader = FileConfigLoader::new();
        let config = loader.load_config_from_path(&config_path).unwrap();
        
        assert_eq!(config.logging.level, "debug");
        assert!(config.logging.log_to_file);
        assert_eq!(config.application.name, "test-app");
        
        // Test with non-existent file
        let non_existent_path = temp_dir.path().join("non_existent.toml");
        let result = loader.load_config_from_path(&non_existent_path);
        
        match result {
            Err(ConfigError::FileNotFound(_)) => (),
            _ => panic!("Expected FileNotFound error"),
        }
    }
    
    #[test]
    fn test_file_config_provider_new() {
        let provider = FileConfigProvider::new();
        assert_eq!(provider.config.logging.level, "info");
        assert_eq!(provider.config.application.name, "novade");
    }
    
    #[test]
    fn test_file_config_provider_with_config() {
        let mut config = CoreConfig::default();
        config.logging.level = "debug".to_string();
        config.application.name = "test-app".to_string();
        
        let provider = FileConfigProvider::with_config(config);
        assert_eq!(provider.get_config().logging.level, "debug");
        assert_eq!(provider.get_config().application.name, "test-app");
    }
    
    #[test]
    fn test_file_config_provider_load_from_path() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Create a config file
        let content = r#"
            [logging]
            level = "debug"
            log_to_file = true
            
            [application]
            name = "test-app"
        "#;
        
        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        // Load the config
        let provider = FileConfigProvider::load_from_path(&config_path).unwrap();
        
        assert_eq!(provider.get_config().logging.level, "debug");
        assert!(provider.get_config().logging.log_to_file);
        assert_eq!(provider.get_config().application.name, "test-app");
        
        // Test with non-existent file (should use defaults)
        let non_existent_path = temp_dir.path().join("non_existent.toml");
        let provider = FileConfigProvider::load_from_path(&non_existent_path).unwrap();
        
        assert_eq!(provider.get_config().logging.level, "info");
        assert!(!provider.get_config().logging.log_to_file);
        assert_eq!(provider.get_config().application.name, "novade");
    }
    
    #[test]
    fn test_file_config_provider_get_config() {
        let provider = FileConfigProvider::new();
        assert_eq!(provider.get_config().logging.level, "info");
        assert_eq!(provider.get_config().application.name, "novade");
    }
    
    #[test]
    fn test_file_config_provider_watch_config() {
        let provider = FileConfigProvider::new();
        let result = provider.watch_config(|_| {});
        assert!(result.is_err());
    }
}
