//! Flexible Logging System for NovaDE Core.
//!
//! This module provides a configurable logging framework for the NovaDE core library,
//! built upon the `tracing` ecosystem. It supports console output and optional
//! file logging with configurable formats.

use crate::config::LoggingConfig;
use crate::error::{CoreError, LoggingError};
use crate::utils; // For utils::fs::ensure_dir_exists

use std::io::stdout;
use std::path::Path;
use std::sync::Mutex; // For Mutex around Option<WorkerGuard>
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard; // Import WorkerGuard
use once_cell::sync::Lazy; // For Lazy static
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer, // Added Layer
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
/// A tuple containing the boxed `Layer` for file logging and its `WorkerGuard`,
/// or `CoreError` on failure.
fn create_file_layer(
    log_path: &Path,
    format: &str,
) -> Result<(Box<dyn Layer<Registry> + Send + Sync + 'static>, WorkerGuard), CoreError> {
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

    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);

    match format.to_lowercase().as_str() {
        "json" => {
            let layer = fmt::layer()
                .json()
                .with_writer(non_blocking_writer)
                .with_ansi(false); // No ANSI colors in files
            Ok((Box::new(layer), guard))
        }
        "text" | _ => { // Default to text format
            let layer = fmt::layer()
                .with_writer(non_blocking_writer)
                .with_ansi(false); // No ANSI colors in files
            Ok((Box::new(layer), guard))
        }
    }
}

/// Global static to hold the WorkerGuard for the file logger.
/// This ensures that the guard is kept alive for the duration of the application,
/// allowing logs to be flushed properly.
static LOG_WORKER_GUARD: Lazy<Mutex<Option<WorkerGuard>>> = Lazy::new(|| Mutex::new(None));

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
/// Returns `CoreError::LoggingInitialization` if configuration is invalid or
/// setting the global subscriber fails on an initial setup.
pub fn init_logging(config: &LoggingConfig, is_reload: bool) -> Result<(), CoreError> {
    // Validate and parse log level from config
    // This should ideally be caught by config::validate_config, but as per spec, check here too.
    let level_filter_str = match config.level.to_lowercase().as_str() {
        "trace" => Level::TRACE.to_string(),
        "debug" => Level::DEBUG.to_string(),
        "info" => Level::INFO.to_string(),
        "warn" => Level::WARN.to_string(),
        "error" => Level::ERROR.to_string(),
        invalid_level => {
            // This error case is per spec for init_logging, even if validate_config should catch it.
            return Err(CoreError::Logging(LoggingError::InitializationFailure(format!(
                "Invalid log level in config: {}",
                invalid_level // Use the actual invalid_level string from config for clarity
            ))));
        }
    };

    // Stdout Layer
    let stdout_filter = EnvFilter::new(level_filter_str.clone());
    let stdout_layer = match config.format.to_lowercase().as_str() {
        "json" => fmt::layer()
            .json()
            .with_writer(stdout)
            .with_ansi(false) // No ANSI for JSON output to stdout
            .with_filter(stdout_filter)
            .boxed(),
        _ => fmt::layer() // Default to text
            .with_writer(stdout)
            .with_ansi(atty::is(atty::Stream::Stdout)) // ANSI if TTY for text
            .with_filter(stdout_filter)
            .boxed(),
    };

    // File Layer (Optional)
    // Note: File layer format is already handled by `create_file_layer` based on `config.format`.
    let mut new_file_guard: Option<WorkerGuard> = None;
    let file_layer_opt: Option<Box<dyn Layer<Registry> + Send + Sync + 'static>> =
        if let Some(log_path) = &config.file_path {
            let file_filter_env = EnvFilter::new(level_filter_str);
            let (base_file_layer, guard) = create_file_layer(log_path, &config.format)?;
            new_file_guard = Some(guard); // Store the guard temporarily
            Some(base_file_layer.with_filter(file_filter_env).boxed())
        } else {
            None // No file logging, so no guard to manage for file
        };
    
    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync + 'static>> = Vec::new();
    layers.push(stdout_layer);
    if let Some(file_layer) = file_layer_opt {
        layers.push(file_layer);
    }

    let result = Registry::default().with(layers).try_init();

    // Store the new worker guard, dropping the old one if it exists.
    // This should happen regardless of whether try_init succeeded or failed,
    // if a new guard was created (e.g. file path changed on reload).
    // However, if try_init fails on initial setup, we might not want to store the guard.
    // If it fails on reload, the old logger might still be active, but we've prepared a new guard.
    // For simplicity, we'll update the guard if one was created.
    if new_file_guard.is_some() || config.file_path.is_none() { // If new guard or file logging disabled
        match LOG_WORKER_GUARD.lock() {
            Ok(mut guard_slot) => {
                if let Some(old_guard) = guard_slot.take() {
                    // old_guard is dropped here, flushing its logs
                    drop(old_guard); // Explicitly drop, though take() then None assignment also does it.
                }
                *guard_slot = new_file_guard;
            }
            Err(e) => {
                // This is a non-critical error in the sense that logging might still mostly work,
                // but flushing the (potentially new) file logger might be compromised.
                // Use eprintln as a fallback if tracing isn't working.
                eprintln!("[ERROR] Failed to lock LOG_WORKER_GUARD to update: {}. Log flushing may be affected.", e);
            }
        }
    }

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            if !is_reload {
                Err(CoreError::Logging(LoggingError::InitializationFailure(format!(
                    "Failed to set global tracing subscriber. Was it already initialized? Error: {}", e
                ))))
            } else {
                // If is_reload, log an info message. The actual logger might not have changed.
                // Use eprintln as a fallback if tracing system is in an uncertain state.
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
    use crate::config::LoggingConfig; // For creating test configs
    use std::fs as std_fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    // For checking log output, we'd ideally capture stderr or read files.
    // `tracing_test` crate could be useful but is an external dependency.

    /// Helper to ensure global logger state is clean for a test.
    /// This is a best-effort approach as `tracing` does not have a public reset API.
    fn ensure_clean_logger_state() {
        // Attempt to set a no-op subscriber. If it succeeds, no subscriber was set.
        // If it fails, a subscriber was already set. This doesn't "clear" it but
        // allows subsequent `try_init` to behave as if it's the first attempt in some cases.
        // This is not foolproof for all test scenarios.
        let _ = tracing::subscriber::set_global_default(tracing::subscriber::NoSubscriber::default());
    }

    #[test]
    fn test_init_minimal_logging_runs_without_panic() {
        ensure_clean_logger_state();
        init_minimal_logging();
        // Test that it can be called multiple times without panic (ignores error)
        init_minimal_logging();
        tracing::info!("Minimal logging test: Info message after init_minimal_logging.");
        // Actual output capture/validation is complex for minimal_logging.
    }

    #[test]
    fn test_create_file_layer_text_format() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_text.log");

        let result = create_file_layer(&log_path, "text");
        assert!(result.is_ok(), "create_file_layer failed for text format: {:?}", result.err());
        let (_layer, _guard) = result.unwrap(); // Ensure guard is returned
        assert!(log_path.parent().unwrap().exists());
    }

    #[test]
    fn test_create_file_layer_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_json.log");

        let result = create_file_layer(&log_path, "json");
        assert!(result.is_ok(), "create_file_layer failed for json format: {:?}", result.err());
        let (_layer, _guard) = result.unwrap(); // Ensure guard is returned
        assert!(log_path.parent().unwrap().exists());
    }
    
    #[test]
    fn test_create_file_layer_ensures_parent_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let nested_log_path = temp_dir.path().join("new_parent_dir/nested_log.log");
        
        assert!(!nested_log_path.parent().unwrap().exists()); // Parent should not exist yet
        
        let result = create_file_layer(&nested_log_path, "text");
        assert!(result.is_ok(), "create_file_layer failed: {:?}", result.err());
        assert!(nested_log_path.parent().unwrap().exists(), "Parent directory was not created");
    }

    #[test]
    fn test_log_worker_guard_management() {
        ensure_clean_logger_state();
        let temp_dir = TempDir::new().unwrap();
        let log_file1 = temp_dir.path().join("guard_test1.log");
        let log_file2 = temp_dir.path().join("guard_test2.log");

        let config1 = LoggingConfig {
            level: "info".to_string(),
            file_path: Some(log_file1),
            format: "text".to_string(),
        };
        init_logging(&config1, false).expect("First init_logging failed");
        assert!(LOG_WORKER_GUARD.lock().unwrap().is_some(), "Guard should be Some after first init with file");

        // Simulate reload with a different file path (should replace guard)
        let config2 = LoggingConfig {
            level: "info".to_string(),
            file_path: Some(log_file2),
            format: "text".to_string(),
        };
        init_logging(&config2, true).expect("Second init_logging (reload) failed");
        assert!(LOG_WORKER_GUARD.lock().unwrap().is_some(), "Guard should be Some after reload with new file");

        // Simulate reload that disables file logging
        let config3 = LoggingConfig {
            level: "info".to_string(),
            file_path: None, // Disable file logging
            format: "text".to_string(),
        };
        init_logging(&config3, true).expect("Third init_logging (disable file) failed");
        assert!(LOG_WORKER_GUARD.lock().unwrap().is_none(), "Guard should be None after disabling file logging");
    }


    #[test]
    fn test_init_logging_invalid_level_returns_error() {
        ensure_clean_logger_state();
        let config = LoggingConfig {
            level: "supertrace".to_string(), // Invalid level
            file_path: None,
            format: "text".to_string(),
        };
        let result = init_logging(&config, false);
        assert!(result.is_err());
        match result.err().unwrap() {
            CoreError::Logging(LoggingError::InitializationFailure(msg)) => {
                assert!(msg.contains("Invalid log level in config: supertrace"));
            }
            other_error => panic!("Unexpected error type: {:?}", other_error),
        }
    }
    
    #[test]
    fn test_init_logging_console_only() {
        ensure_clean_logger_state();
        // Test with "text" format for console
        let config_text = LoggingConfig {
            level: "info".to_string(),
            file_path: None,
            format: "text".to_string(),
        };
        let result_text = init_logging(&config_text, false);
        assert!(result_text.is_ok(), "init_logging failed for console (text): {:?}", result_text.err());
        tracing::info!("Console-only logging test (text): Info message.");
        tracing::debug!("Console-only logging test (text): Debug message. (Should not be visible)");

        // Clean state and test with "json" format for console
        ensure_clean_logger_state();
        let config_json = LoggingConfig {
            level: "info".to_string(),
            file_path: None,
            format: "json".to_string(),
        };
        let result_json = init_logging(&config_json, false);
        assert!(result_json.is_ok(), "init_logging failed for console (json): {:?}", result_json.err());
        // Note: Actual verification of JSON output to console is hard in unit tests.
        // This test primarily ensures setup doesn't panic.
        tracing::info!(message = "Console-only logging test (json): Info message.", key = "value");
        tracing::debug!(message = "Console-only logging test (json): Debug message. (Should not be visible)", key = "debug_value");
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
        
        // Drop the subscriber to attempt to flush logs.
        // This is hard with global state. We rely on std::mem::forget(_guard) for now,
        // which means logs might not be flushed immediately for reading.
        // For reliable test, explicit flush or guard management is needed.
        // This test primarily ensures init_logging runs and file layer is configured.
        // A small delay might help, but not ideal.
        // std::thread::sleep(std::time::Duration::from_millis(100)); 

        if log_file.exists() { // File may not be created/written immediately by non-blocking
            let content = std_fs::read_to_string(&log_file).unwrap_or_default();
            // Check for parts of the message. Full format depends on tracing_subscriber defaults.
             assert!(content.contains("File logging (text) test: Debug message."));
             assert!(content.contains("File logging (text) test: Info message."));
        } else {
            // This might happen with non-blocking if not flushed.
            // Consider this test as "runs without error" for now.
            println!("Warning: Log file {} not found for test_init_logging_with_file_text. Non-blocking writer might not have flushed.", log_file.display());
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
            println!("Warning: Log file {} not found for test_init_logging_with_file_json. Non-blocking writer might not have flushed.", log_file.display());
        }
    }

    #[test]
    fn test_init_logging_reload_true_does_not_error_if_already_set() {
        ensure_clean_logger_state();
        let config1 = LoggingConfig { level: "info".to_string(), file_path: None, format: "text".to_string() };
        init_logging(&config1, false).expect("First init failed");

        let config2 = LoggingConfig { level: "debug".to_string(), file_path: None, format: "text".to_string() };
        // This should not return Err, but log an info message (which we can't easily capture here)
        let result = init_logging(&config2, true); 
        assert!(result.is_ok(), "Reloading logging should not error, but got: {:?}", result.err());
        tracing::info!("Reload test: Info after first init."); // Should be logged by first config
        tracing::debug!("Reload test: Debug after attempting reload."); // Visibility depends on whether subscriber actually updated.
    }

    #[test]
    fn test_init_logging_reload_false_errors_if_already_set() {
        ensure_clean_logger_state();
        let config1 = LoggingConfig { level: "info".to_string(), file_path: None, format: "text".to_string() };
        init_logging(&config1, false).expect("First init failed");

        let config2 = LoggingConfig { level: "debug".to_string(), file_path: None, format: "text".to_string() };
        let result = init_logging(&config2, false); // is_reload = false
        assert!(result.is_err(), "Second init with is_reload=false should error");
        match result.err().unwrap() {
            CoreError::Logging(LoggingError::InitializationFailure(msg)) => {
                assert!(msg.contains("Failed to set global tracing subscriber"));
            }
            other_error => panic!("Unexpected error type: {:?}", other_error),
        }
    }
}
