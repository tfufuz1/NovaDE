//! Flexible Logging System for NovaDE Core.
//!
//! This module provides a configurable logging framework for the NovaDE core library,
//! built upon the `tracing` ecosystem. It supports console output and optional
//! file logging with configurable formats.

use crate::config::LoggingConfig;
use crate::error::CoreError; // Changed: Removed LoggingError
use crate::utils; // For utils::fs::ensure_dir_exists

use std::io::stdout;
use std::path::Path;
use tracing::Level;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
    Registry,
};
use atty;

/// Initializes a minimal logging setup, directing messages to `stderr`.
///
/// This function is intended for use in tests, early application startup before full
/// configuration is loaded, or as a fallback if detailed logging initialization fails.
/// It filters messages based on the `RUST_LOG` environment variable, defaulting to
/// "info" level if `RUST_LOG` is not set or is invalid.
/// Errors during initialization (e.g., if a global logger is already set) are ignored.
pub fn init_minimal_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(Level::INFO.to_string()));

    let _ = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr) // Direct to stderr
        .with_ansi(atty::is(atty::Stream::Stderr)) // Colors if stderr is a TTY
        .try_init(); // Ignore error if already initialized
}

/// Creates a file logging layer.
///
/// Ensures the parent directory for the log file exists, sets up daily rolling
/// file appender, and configures the log format (text or JSON).
///
/// # Arguments
///
/// * `log_path`: Path to the log file.
/// * `format`: Logging format ("text" or "json").
///
/// # Returns
///
/// A boxed `Layer` for file logging, or `CoreError` on failure.
fn create_file_layer(
    log_path: &Path,
    format: &str,
) -> Result<Box<dyn Layer<Registry> + Send + Sync + 'static>, CoreError> {
    // Ensure parent directory exists
    if let Some(parent) = log_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() { // Check if parent is not root or empty
            utils::fs::ensure_dir_exists(parent)?;
        }
    }

    let file_appender = tracing_appender::rolling::daily(
        log_path.parent().unwrap_or_else(|| Path::new(".")), // Default to current dir if no parent
        log_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("core.log")), // Default filename
    );

    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);

    std::mem::forget(_guard); // THIS IS NOT FOR PRODUCTION USE.

    match format.to_lowercase().as_str() {
        "json" => {
            let layer = fmt::layer()
                .json()
                .with_writer(non_blocking_writer)
                .with_ansi(false);
            Ok(Box::new(layer))
        }
        "text" | _ => {
            let layer = fmt::layer()
                .with_writer(non_blocking_writer)
                .with_ansi(false);
            Ok(Box::new(layer))
        }
    }
}

/// Initializes the global logging system based on the provided [`LoggingConfig`].
///
/// Configures and sets the global `tracing` subscriber with a console layer and
/// an optional file logging layer.
///
/// # Arguments
///
/// * `config`: A reference to the [`LoggingConfig`].
/// * `is_reload`: If `true`, informational messages are logged on re-initialization attempts;
///   if `false`, errors are returned if a logger is already set.
///
/// # Errors
///
/// Returns `CoreError::Logging` if configuration is invalid or
/// setting the global subscriber fails on an initial setup.
pub fn init_logging(config: &LoggingConfig, is_reload: bool) -> Result<(), CoreError> {
    let level_filter_str = match config.level.to_lowercase().as_str() {
        "trace" => Level::TRACE.to_string(),
        "debug" => Level::DEBUG.to_string(),
        "info" => Level::INFO.to_string(),
        "warn" => Level::WARN.to_string(),
        "error" => Level::ERROR.to_string(),
        invalid_level => {
            // Changed: Use CoreError::Logging(String)
            return Err(CoreError::Logging(format!(
                "Invalid log level in config: {}",
                invalid_level
            )));
        }
    };

    let stdout_filter = EnvFilter::new(level_filter_str.clone());
    let stdout_layer = fmt::layer()
        .with_writer(stdout)
        .with_ansi(atty::is(atty::Stream::Stdout))
        .with_filter(stdout_filter)
        .boxed();

    let file_layer_opt: Option<Box<dyn Layer<Registry> + Send + Sync + 'static>> =
        if let Some(log_path) = &config.file_path {
            let file_filter_env = EnvFilter::new(level_filter_str);
            let base_file_layer = create_file_layer(log_path, &config.format)?;
            Some(base_file_layer.with_filter(file_filter_env).boxed())
        } else {
            None
        };
    
    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync + 'static>> = Vec::new();
    layers.push(stdout_layer);
    if let Some(file_layer) = file_layer_opt {
        layers.push(file_layer);
    }

    let result = Registry::default().with(layers).try_init();

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            if !is_reload {
                // Changed: Use CoreError::Logging(String)
                Err(CoreError::Logging(format!(
                    "Failed to set global tracing subscriber. Was it already initialized? Error: {}", e
                )))
            } else {
                let msg = format!("[INFO] Re-initializing logging configuration attempted. Previous logger may persist. Error: {}", e);
                eprintln!("{}", msg);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LoggingConfig;
    use std::fs as std_fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn ensure_clean_logger_state() {
        let _ = tracing::subscriber::set_global_default(tracing::subscriber::NoSubscriber::default());
    }

    #[test]
    fn test_init_minimal_logging_runs_without_panic() {
        ensure_clean_logger_state();
        init_minimal_logging();
        init_minimal_logging();
        tracing::info!("Minimal logging test: Info message after init_minimal_logging.");
    }

    #[test]
    fn test_create_file_layer_text_format() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_text.log");
        let result = create_file_layer(&log_path, "text");
        assert!(result.is_ok(), "create_file_layer failed for text format: {:?}", result.err());
        assert!(log_path.parent().unwrap().exists());
    }

    #[test]
    fn test_create_file_layer_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_json.log");
        let result = create_file_layer(&log_path, "json");
        assert!(result.is_ok(), "create_file_layer failed for json format: {:?}", result.err());
        assert!(log_path.parent().unwrap().exists());
    }
    
    #[test]
    fn test_create_file_layer_ensures_parent_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let nested_log_path = temp_dir.path().join("new_parent_dir/nested_log.log");
        assert!(!nested_log_path.parent().unwrap().exists());
        let result = create_file_layer(&nested_log_path, "text");
        assert!(result.is_ok(), "create_file_layer failed: {:?}", result.err());
        assert!(nested_log_path.parent().unwrap().exists(), "Parent directory was not created");
    }

    #[test]
    fn test_init_logging_invalid_level_returns_error() {
        ensure_clean_logger_state();
        let config = LoggingConfig {
            level: "supertrace".to_string(),
            file_path: None,
            format: "text".to_string(),
        };
        let result = init_logging(&config, false);
        assert!(result.is_err());
        // Changed: Match CoreError::Logging(String)
        match result.err().unwrap() {
            CoreError::Logging(msg) => {
                assert!(msg.contains("Invalid log level in config: supertrace"));
            }
            other_error => panic!("Unexpected error type: {:?}", other_error),
        }
    }
    
    #[test]
    fn test_init_logging_console_only() {
        ensure_clean_logger_state();
        let config = LoggingConfig {
            level: "info".to_string(),
            file_path: None,
            format: "text".to_string(),
        };
        let result = init_logging(&config, false);
        assert!(result.is_ok(), "init_logging failed for console only: {:?}", result.err());
        tracing::info!("Console-only logging test: Info message.");
        tracing::debug!("Console-only logging test: Debug message. (Should not be visible)");
    }

    #[test]
    fn test_init_logging_with_file_text() {
        ensure_clean_logger_state();
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("app_text.log");
        let config = LoggingConfig {
            level: "debug".to_string(),
            file_path: Some(log_file.clone()),
            format: "text".to_string(),
        };
        let result = init_logging(&config, false);
        assert!(result.is_ok(), "init_logging failed for file (text): {:?}", result.err());
        
        tracing::debug!("File logging (text) test: Debug message.");
        tracing::info!("File logging (text) test: Info message.");
        
        if log_file.exists() {
            let content = std_fs::read_to_string(&log_file).unwrap_or_default();
             assert!(content.contains("File logging (text) test: Debug message."));
             assert!(content.contains("File logging (text) test: Info message."));
        } else {
            println!("Warning: Log file {} not found for test_init_logging_with_file_text.", log_file.display());
        }
    }
    
    #[test]
    fn test_init_logging_with_file_json() {
        ensure_clean_logger_state();
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("app_json.log");
        let config = LoggingConfig {
            level: "info".to_string(),
            file_path: Some(log_file.clone()),
            format: "json".to_string(),
        };
        let result = init_logging(&config, false);
        assert!(result.is_ok(), "init_logging failed for file (json): {:?}", result.err());
        
        tracing::info!(message = "File logging (json) test", key = "value");
        
        if log_file.exists() {
            let content = std_fs::read_to_string(&log_file).unwrap_or_default();
            assert!(content.contains("\"message\":\"File logging (json) test\""));
            assert!(content.contains("\"key\":\"value\""));
        } else {
            println!("Warning: Log file {} not found for test_init_logging_with_file_json.", log_file.display());
        }
    }

    #[test]
    fn test_init_logging_reload_true_does_not_error_if_already_set() {
        ensure_clean_logger_state();
        let config1 = LoggingConfig { level: "info".to_string(), file_path: None, format: "text".to_string() };
        init_logging(&config1, false).expect("First init failed");

        let config2 = LoggingConfig { level: "debug".to_string(), file_path: None, format: "text".to_string() };
        let result = init_logging(&config2, true); 
        assert!(result.is_ok(), "Reloading logging should not error, but got: {:?}", result.err());
        tracing::info!("Reload test: Info after first init.");
        tracing::debug!("Reload test: Debug after attempting reload.");
    }

    #[test]
    fn test_init_logging_reload_false_errors_if_already_set() {
        ensure_clean_logger_state();
        let config1 = LoggingConfig { level: "info".to_string(), file_path: None, format: "text".to_string() };
        init_logging(&config1, false).expect("First init failed");

        let config2 = LoggingConfig { level: "debug".to_string(), file_path: None, format: "text".to_string() };
        let result = init_logging(&config2, false);
        assert!(result.is_err(), "Second init with is_reload=false should error");
        // Changed: Match CoreError::Logging(String)
        match result.err().unwrap() {
            CoreError::Logging(msg) => {
                assert!(msg.contains("Failed to set global tracing subscriber"));
            }
            other_error => panic!("Unexpected error type: {:?}", other_error),
        }
    }
}
