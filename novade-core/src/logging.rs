//! Logging module for the NovaDE core layer.
//!
//! This module provides logging functionality used throughout the
//! NovaDE desktop environment, using the tracing framework for
//! structured logging.

use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Once;

use tracing::{Level, subscriber::set_global_default};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Registry,
};

use crate::error::LoggingError;
use crate::config::LoggingConfig;

// Ensure initialization happens only once
static INIT: Once = Once::new();
static mut INITIALIZED: bool = false;

/// Initializes the logging system with the given configuration.
///
/// This function sets up the tracing framework with the specified
/// configuration. It ensures that initialization happens only once.
///
/// # Arguments
///
/// * `config` - The logging configuration
///
/// # Returns
///
/// A `Result` indicating success or failure.
pub fn initialize_logging(config: &LoggingConfig) -> Result<(), LoggingError> {
    let mut result = Ok(());
    
    INIT.call_once(|| {
        result = match do_initialize_logging(config) {
            Ok(()) => {
                unsafe { INITIALIZED = true; }
                Ok(())
            },
            Err(e) => Err(e),
        };
    });
    
    if unsafe { INITIALIZED } {
        Ok(())
    } else {
        result
    }
}

/// Checks if the logging system has been initialized.
///
/// # Returns
///
/// `true` if the logging system has been initialized, `false` otherwise.
pub fn is_initialized() -> bool {
    unsafe { INITIALIZED }
}

/// Internal function to initialize the logging system.
///
/// # Arguments
///
/// * `config` - The logging configuration
///
/// # Returns
///
/// A `Result` indicating success or failure.
fn do_initialize_logging(config: &LoggingConfig) -> Result<(), LoggingError> {
    // Parse the log level
    let level = match config.level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => {
            return Err(LoggingError::FilterError(
                format!("Invalid log level: {}", config.level)
            ));
        }
    };
    
    // Create the environment filter
    let env_filter = EnvFilter::from_default_env()
        .add_directive(level.into());
    
    // Create the registry
    let mut layers = Vec::new();
    
    // Add console layer if enabled
    if config.log_to_console {
        let console_layer = fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_target(true);
        
        layers.push(console_layer.boxed());
    }
    
    // Add file layer if enabled
    if config.log_to_file {
        let file = match open_log_file(&config.log_file) {
            Ok(file) => file,
            Err(e) => {
                return Err(LoggingError::IoError(e));
            }
        };
        
        let file_layer = fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_target(true)
            .with_writer(file);
        
        layers.push(file_layer.boxed());
    }
    
    // Create and set the subscriber
    let subscriber = Registry::default()
        .with(env_filter)
        .with(layers);
    
    match set_global_default(subscriber) {
        Ok(()) => Ok(()),
        Err(e) => Err(LoggingError::InitializationError(
            format!("Failed to set global default subscriber: {}", e)
        )),
    }
}

/// Opens the log file for writing.
///
/// # Arguments
///
/// * `path` - The path to the log file
///
/// # Returns
///
/// A `Result` containing the opened file if successful,
/// or an `io::Error` if opening failed.
fn open_log_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.as_ref().parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    
    // Open the file for writing
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_initialize_logging_console_only() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            log_to_file: false,
            log_to_console: true,
            log_file: "novade.log".to_string(),
        };
        
        // Reset the initialization state for testing
        unsafe {
            INITIALIZED = false;
        }
        
        // Initialize logging
        let result = initialize_logging(&config);
        assert!(result.is_ok());
        assert!(is_initialized());
    }
    
    #[test]
    fn test_initialize_logging_with_file() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");
        
        let config = LoggingConfig {
            level: "info".to_string(),
            log_to_file: true,
            log_to_console: false,
            log_file: log_file.to_string_lossy().to_string(),
        };
        
        // Reset the initialization state for testing
        unsafe {
            INITIALIZED = false;
        }
        
        // Initialize logging
        let result = initialize_logging(&config);
        assert!(result.is_ok());
        assert!(is_initialized());
        
        // Check that the log file was created
        assert!(log_file.exists());
    }
    
    #[test]
    fn test_initialize_logging_invalid_level() {
        let config = LoggingConfig {
            level: "invalid".to_string(),
            log_to_file: false,
            log_to_console: true,
            log_file: "novade.log".to_string(),
        };
        
        // Reset the initialization state for testing
        unsafe {
            INITIALIZED = false;
        }
        
        // Initialize logging
        let result = initialize_logging(&config);
        assert!(result.is_err());
        assert!(!is_initialized());
        
        match result {
            Err(LoggingError::FilterError(_)) => (),
            _ => panic!("Expected FilterError"),
        }
    }
    
    #[test]
    fn test_initialize_logging_once() {
        let config1 = LoggingConfig {
            level: "debug".to_string(),
            log_to_file: false,
            log_to_console: true,
            log_file: "novade.log".to_string(),
        };
        
        let config2 = LoggingConfig {
            level: "error".to_string(),
            log_to_file: false,
            log_to_console: true,
            log_file: "novade.log".to_string(),
        };
        
        // Reset the initialization state for testing
        unsafe {
            INITIALIZED = false;
        }
        
        // Initialize logging with first config
        let result1 = initialize_logging(&config1);
        assert!(result1.is_ok());
        assert!(is_initialized());
        
        // Try to initialize again with second config
        let result2 = initialize_logging(&config2);
        assert!(result2.is_ok()); // Should succeed but not change anything
        assert!(is_initialized());
    }
}
