//! Configuration module for the NovaDE core layer.
//!
//! This module provides configuration management utilities used throughout the
//! NovaDE desktop environment, including loading, parsing, and accessing
//! configuration.

pub mod defaults;
pub mod file_loader;

use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::error::ConfigError;

/// Root configuration structure for the NovaDE desktop environment.
///
/// This struct contains all configuration settings for the core layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
    
    /// Application configuration
    #[serde(default)]
    pub application: ApplicationConfig,
    
    /// System configuration
    #[serde(default)]
    pub system: SystemConfig,
}

impl Default for CoreConfig {
    fn default() -> Self {
        CoreConfig {
            logging: LoggingConfig::default(),
            application: ApplicationConfig::default(),
            system: SystemConfig::default(),
        }
    }
}

/// Logging configuration for the NovaDE desktop environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level filter
    #[serde(default = "defaults::default_log_level")]
    pub level: String,
    
    /// Whether to log to file
    #[serde(default)]
    pub log_to_file: bool,
    
    /// Log file path
    #[serde(default = "defaults::default_log_file")]
    pub log_file: String,
    
    /// Whether to log to console
    #[serde(default = "defaults::default_log_to_console")]
    pub log_to_console: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: defaults::default_log_level(),
            log_to_file: false,
            log_file: defaults::default_log_file(),
            log_to_console: defaults::default_log_to_console(),
        }
    }
}

/// Application configuration for the NovaDE desktop environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Application name
    #[serde(default = "defaults::default_app_name")]
    pub name: String,
    
    /// Application version
    #[serde(default = "defaults::default_app_version")]
    pub version: String,
    
    /// Application data directory
    #[serde(default = "defaults::default_data_dir")]
    pub data_dir: String,
    
    /// Application cache directory
    #[serde(default = "defaults::default_cache_dir")]
    pub cache_dir: String,
    
    /// Application config directory
    #[serde(default = "defaults::default_config_dir")]
    pub config_dir: String,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        ApplicationConfig {
            name: defaults::default_app_name(),
            version: defaults::default_app_version(),
            data_dir: defaults::default_data_dir(),
            cache_dir: defaults::default_cache_dir(),
            config_dir: defaults::default_config_dir(),
        }
    }
}

/// System configuration for the NovaDE desktop environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Number of worker threads
    #[serde(default = "defaults::default_worker_threads")]
    pub worker_threads: usize,
    
    /// Whether to use hardware acceleration
    #[serde(default = "defaults::default_use_hardware_acceleration")]
    pub use_hardware_acceleration: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        SystemConfig {
            worker_threads: defaults::default_worker_threads(),
            use_hardware_acceleration: defaults::default_use_hardware_acceleration(),
        }
    }
}

/// Interface for loading configuration.
pub trait ConfigLoader {
    /// Loads configuration from the default location.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `CoreConfig` if successful,
    /// or a `ConfigError` if loading failed.
    fn load_config(&self) -> Result<CoreConfig, ConfigError>;
    
    /// Loads configuration from the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to load configuration from
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `CoreConfig` if successful,
    /// or a `ConfigError` if loading failed.
    fn load_config_from_path<P: AsRef<Path>>(&self, path: P) -> Result<CoreConfig, ConfigError>;
}

/// Interface for accessing configuration.
pub trait ConfigProvider {
    /// Gets the current configuration.
    ///
    /// # Returns
    ///
    /// A reference to the current `CoreConfig`.
    fn get_config(&self) -> &CoreConfig;
    
    /// Watches for configuration changes.
    ///
    /// # Arguments
    ///
    /// * `callback` - A callback function to call when configuration changes
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn watch_config<F>(&self, callback: F) -> Result<(), ConfigError>
    where
        F: Fn(&CoreConfig) + Send + 'static;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_core_config_default() {
        let config = CoreConfig::default();
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.application.name, "novade");
    }
    
    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert!(!config.log_to_file);
        assert!(config.log_to_console);
    }
    
    #[test]
    fn test_application_config_default() {
        let config = ApplicationConfig::default();
        assert_eq!(config.name, "novade");
        assert_eq!(config.version, "0.1.0");
        assert!(!config.data_dir.is_empty());
        assert!(!config.cache_dir.is_empty());
        assert!(!config.config_dir.is_empty());
    }
    
    #[test]
    fn test_system_config_default() {
        let config = SystemConfig::default();
        assert!(config.worker_threads > 0);
        assert!(config.use_hardware_acceleration);
    }
}
