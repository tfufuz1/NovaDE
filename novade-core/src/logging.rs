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
//ANCHOR [NovaDE Developers <dev@novade.org>] Added config enums.
use crate::config::{LogFormat, LogOutput, LogRotation};
use tracing_appender::rolling::{RollingFileAppender, Rotation as TracingRotation};


/// # Arguments
///
/// * `log_path`: Path to the log file.
/// * `rotation_policy`: The `LogRotation` policy.
/// * `log_format`: The `LogFormat` for messages.
///
/// # Returns
///
/// A boxed `Layer` for file logging, or `CoreError` on failure.
fn create_file_layer(
    log_path: &Path,
    rotation_policy: &LogRotation,
    log_format: &LogFormat,
) -> Result<Box<dyn Layer<Registry> + Send + Sync + 'static>, CoreError> {
    // Ensure parent directory exists
    if let Some(parent) = log_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() { // Check if parent is not root or empty
            utils::fs::ensure_dir_exists(parent)?;
        }
    }

    let file_appender = match rotation_policy {
        LogRotation::Daily => RollingFileAppender::new(
            TracingRotation::DAILY,
            log_path.parent().unwrap_or_else(|| Path::new(".")),
            log_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("novade-core.log")),
        ),
        LogRotation::SizeMB(size_mb) => {
            // tracing_appender's RollingFileAppender with `Rotation::NEVER` can be used with a custom
            // wrapper that handles size-based rotation, or one might need to find/build a specific
            // size-based rotating appender. For now, we'll use Rotation::NEVER as a stand-in
            // and acknowledge this is not full size-based rotation by `tracing-appender` itself.
            // A true size-based rotation often involves multiple numbered log files.
            //TODO [Log Rotation Policy] [NovaDE Developers <dev@novade.org>] Implement actual size-based rotation. `tracing-appender`'s default `Rotation::NEVER` doesn't do rotation by size itself. This requires a more complex setup, potentially with a custom file watching/rotating mechanism or a different crate. The `size_mb` parameter is currently unused with `Rotation::NEVER`.
            eprintln!("[WARN] SizeMB rotation specified but not fully implemented with tracing_appender's default RollingFileAppender. Using non-rotating file. Size: {}MB", size_mb);
            RollingFileAppender::new(
                TracingRotation::NEVER, // Placeholder, does not rotate by size.
                log_path.parent().unwrap_or_else(|| Path::new(".")),
                log_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("novade-core.log")),
            )
        }
        LogRotation::None => RollingFileAppender::new(
            TracingRotation::NEVER,
            log_path.parent().unwrap_or_else(|| Path::new(".")),
            log_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("novade-core.log")),
        ),
    };

    //ANCHOR [NovaDE Developers <dev@novade.org>] Create a non-blocking writer.
    //TODO [NovaDE Developers <dev@novade.org>] The _guard MUST be kept for the lifetime of the logger.
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);
    let _logging_guard = _guard; // Assign to a named variable to be managed (e.g. returned and stored by caller)

    match log_format {
        LogFormat::Json => {
            let layer = fmt::layer()
                .json()
                .with_writer(non_blocking_writer)
                .with_ansi(false);
            Ok(Box::new(layer))
        }
        LogFormat::Text | _ => { // Default to text
            let layer = fmt::layer()
                .with_writer(non_blocking_writer)
                .with_ansi(false);
            Ok(Box::new(layer))
        }
    }
}

/// Initializes the global logging system based on the provided [`LoggingConfig`].
///
/// Configures and sets the global `tracing` subscriber.
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
    //ANCHOR [NovaDE Developers <dev@novade.org>] Determine log level from config.
    let level_filter_str = match config.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE.to_string(),
        "debug" => Level::DEBUG.to_string(),
        "info" => Level::INFO.to_string(),
        "warn" => Level::WARN.to_string(),
        "error" => Level::ERROR.to_string(),
        invalid_level => {
            return Err(CoreError::Logging(format!(
                "Invalid log_level in config: {}",
                invalid_level
            )));
        }
    };

    //ANCHOR [NovaDE Developers <dev@novade.org>] Configure EnvFilter for all layers, respecting RUST_LOG.
    // Determine the filter string first.
    let filter_directive_str = EnvFilter::try_from_default_env()
        .map(|filter| filter.to_string()) // Attempt to get directives if RUST_LOG is set
        .unwrap_or_else(|_| level_filter_str.clone()); // Fallback to configured level_filter_str

    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync + 'static>> = Vec::new();

    //ANCHOR [NovaDE Developers <dev@novade.org>] Configure layer based on log_output config.
    match &config.log_output {
        LogOutput::Stdout => {
            let stdout_env_filter = EnvFilter::try_new(&filter_directive_str)
                .unwrap_or_else(|_| EnvFilter::new(Level::INFO.to_string())); // Fallback if directive is bad

            match config.log_format {
                LogFormat::Json => {
                    let layer = fmt::layer()
                        .with_writer(stdout)
                        .with_ansi(atty::is(atty::Stream::Stdout))
                        .json() // Apply .json() before .with_filter()
                        .with_filter(stdout_env_filter);
                    layers.push(layer.boxed());
                }
                LogFormat::Text => {
                    let layer = fmt::layer()
                        .with_writer(stdout)
                        .with_ansi(atty::is(atty::Stream::Stdout))
                        .with_filter(stdout_env_filter);
                    layers.push(layer.boxed());
                }
            };
        }
        LogOutput::File { path, rotation } => {
            // For file layer, create_file_layer already handles format (json/text) internally.
            // So we just apply the filter to the layer it returns.
            let file_env_filter = EnvFilter::try_new(&filter_directive_str)
                .unwrap_or_else(|_| EnvFilter::new(Level::INFO.to_string())); // Fallback if directive is bad
            let file_layer_base = create_file_layer(path, rotation, &config.log_format)?;
            layers.push(file_layer_base.with_filter(file_env_filter).boxed());
        }
    }

    //TODO [NovaDE Developers <dev@novade.org>] Integrate SentryLayer here.
    // Example: if sentry is enabled in ErrorTrackingConfig:
    // layers.push(crate::error_tracking::get_sentry_tracing_layer().with_filter(env_filter_for_sentry));
    // Ensure `env_filter_for_sentry` is appropriate (e.g., might want different verbosity for Sentry breadcrumbs).

    //ANCHOR [NovaDE Developers <dev@novade.org>] Combine layers for the tracing subscriber.
    //TODO [NovaDE Developers <dev@novade.org>] Explore log aggregation for multi-process/distributed scenarios.
    //TODO [NovaDE Developers <dev@novade.org>] Implement more advanced filtering capabilities if needed.

    if layers.is_empty() {
        // This might happen if config is malformed or no output is specified.
        // Fallback to a minimal stdout logger to ensure some logging is available.
        eprintln!("[WARN] No logging layers configured. Falling back to minimal stdout logger (info level).");
        let fallback_filter = EnvFilter::new(Level::INFO.to_string());
        let fallback_layer = fmt::layer().with_writer(stdout).with_filter(fallback_filter).boxed();
        layers.push(fallback_layer);
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
    //ANCHOR [NovaDE Developers <dev@novade.org>] Import new config types for tests.
    use crate::config::{LoggingConfig, LogOutput, LogRotation, LogFormat};
    //TODO [Test Output Capture] [NovaDE Developers <dev@novade.org>] For more thorough testing, especially of log content and formats, consider using a library or custom setup to capture stdout/stderr or read from temporary log files immediately after tracing events. This is complex due to tracing's async nature and potential for output buffering.
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
        //ANCHOR [NovaDE Developers <dev@novade.org>] Updated create_file_layer call.
        let result = create_file_layer(&log_path, &LogRotation::Daily, &LogFormat::Text);
        assert!(result.is_ok(), "create_file_layer failed for text format: {:?}", result.err());
        assert!(log_path.parent().unwrap().exists());
    }

    #[test]
    fn test_create_file_layer_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_json.log");
        //ANCHOR [NovaDE Developers <dev@novade.org>] Updated create_file_layer call.
        let result = create_file_layer(&log_path, &LogRotation::None, &LogFormat::Json);
        assert!(result.is_ok(), "create_file_layer failed for json format: {:?}", result.err());
        assert!(log_path.parent().unwrap().exists());
    }

    #[test]
    //ANCHOR [NovaDE Developers <dev@novade.org>] Test for LogRotation::SizeMB setup.
    fn test_create_file_layer_size_mb_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_size_rotation.log");
        let result = create_file_layer(&log_path, &LogRotation::SizeMB(50), &LogFormat::Text);
        assert!(result.is_ok(), "create_file_layer failed for SizeMB rotation: {:?}", result.err());
        assert!(log_path.parent().unwrap().exists());
        // Note: This test primarily ensures the setup doesn't panic and prepares the file.
        // Actual size-based rotation is not fully implemented by the default appender with Rotation::NEVER.
        // A warning about this is printed by `create_file_layer`.
    }
    
    #[test]
    fn test_create_file_layer_ensures_parent_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let nested_log_path = temp_dir.path().join("new_parent_dir/nested_log.log");
        assert!(!nested_log_path.parent().unwrap().exists());
        //ANCHOR [NovaDE Developers <dev@novade.org>] Updated create_file_layer call.
        let result = create_file_layer(&nested_log_path, &LogRotation::SizeMB(10), &LogFormat::Text);
        assert!(result.is_ok(), "create_file_layer failed: {:?}", result.err());
        assert!(nested_log_path.parent().unwrap().exists(), "Parent directory was not created");
    }

    #[test]
    fn test_init_logging_invalid_level_returns_error() {
        ensure_clean_logger_state();
        //ANCHOR [NovaDE Developers <dev@novade.org>] Updated LoggingConfig for test.
        let config = LoggingConfig {
            log_level: "supertrace".to_string(),
            log_output: LogOutput::Stdout,
            log_format: LogFormat::Text,
        };
        let result = init_logging(&config, false);
        assert!(result.is_err());
        match result.err().unwrap() {
            CoreError::Logging(msg) => {
                assert!(msg.contains("Invalid log_level in config: supertrace"));
            }
            other_error => panic!("Unexpected error type: {:?}", other_error),
        }
    }
    
    #[test]
    fn test_init_logging_stdout_text() { // Renamed for clarity
        ensure_clean_logger_state();
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_output: LogOutput::Stdout,
            log_format: LogFormat::Text,
        };
        let result = init_logging(&config, false);
        assert!(result.is_ok(), "init_logging failed for stdout text: {:?}", result.err());
        tracing::info!("Stdout text logging test: Info message.");
        tracing::debug!("Stdout text logging test: Debug message. (Should not be visible)");
    }

    #[test]
    fn test_init_logging_stdout_json() {
        ensure_clean_logger_state();
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_output: LogOutput::Stdout,
            log_format: LogFormat::Json,
        };
        let result = init_logging(&config, false);
        assert!(result.is_ok(), "init_logging failed for stdout json: {:?}", result.err());
        tracing::info!(message="Stdout JSON logging test", key="value");
        // Note: Verifying JSON output on stdout would require capturing stdout.
    }


    #[test]
    fn test_init_logging_with_file_text_daily_rotation() { // Renamed for clarity
        ensure_clean_logger_state();
        let temp_dir = TempDir::new().unwrap();
        let log_file_path = temp_dir.path().join("app_text_daily.log");
        let config = LoggingConfig {
            log_level: "debug".to_string(),
            log_output: LogOutput::File { path: log_file_path.clone(), rotation: LogRotation::Daily },
            log_format: LogFormat::Text,
        };
        let result = init_logging(&config, false);
        assert!(result.is_ok(), "init_logging failed for file (text, daily): {:?}", result.err());
        
        tracing::debug!("File logging (text, daily) test: Debug message.");
        tracing::info!("File logging (text, daily) test: Info message.");
        
        // Check if the specific log file exists. Rotation might add date stamps,
        // so direct check of `log_file_path` might not work for rotated files immediately.
        // For non-rotated or initial file, it should exist.
        // More robust check would be to find file with pattern if rotation occurred.
        // For this test, we assume it writes to the base name before first rotation.
        if log_file_path.exists() {
            let content = std_fs::read_to_string(&log_file_path).unwrap_or_default();
            assert!(content.contains("File logging (text, daily) test: Debug message."));
            assert!(content.contains("File logging (text, daily) test: Info message."));
        } else {
            // Search for files that start with "app_text_daily.log" in the temp_dir
            let mut found = false;
            for entry in std_fs::read_dir(temp_dir.path()).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() && path.file_name().unwrap().to_string_lossy().starts_with("app_text_daily.log") {
                    let content = std_fs::read_to_string(&path).unwrap_or_default();
                    if content.contains("File logging (text, daily) test: Debug message.") &&
                       content.contains("File logging (text, daily) test: Info message.") {
                        found = true;
                        break;
                    }
                }
            }
            assert!(found, "Log file with expected content not found for test_init_logging_with_file_text_daily_rotation. Path: {}", log_file_path.display());
        }
    }
    
    #[test]
    fn test_init_logging_with_file_json_no_rotation() { // Renamed for clarity
        ensure_clean_logger_state();
        let temp_dir = TempDir::new().unwrap();
        let log_file_path = temp_dir.path().join("app_json_none.log");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_output: LogOutput::File { path: log_file_path.clone(), rotation: LogRotation::None },
            log_format: LogFormat::Json,
        };
        let result = init_logging(&config, false);
        assert!(result.is_ok(), "init_logging failed for file (json, none): {:?}", result.err());
        
        tracing::info!(message = "File logging (json, none) test", key = "value");
        
        if log_file_path.exists() {
            let content = std_fs::read_to_string(&log_file_path).unwrap_or_default();
            assert!(content.contains("\"message\":\"File logging (json, none) test\""));
            assert!(content.contains("\"key\":\"value\""));
        } else {
            panic!("Log file {} not found for test_init_logging_with_file_json_no_rotation.", log_file_path.display());
        }
    }

    #[test]
    fn test_init_logging_reload_true_does_not_error_if_already_set() {
        ensure_clean_logger_state();
        let config1 = LoggingConfig { log_level: "info".to_string(), log_output: LogOutput::Stdout, log_format: LogFormat::Text };
        init_logging(&config1, false).expect("First init failed");

        let config2 = LoggingConfig { log_level: "debug".to_string(), log_output: LogOutput::Stdout, log_format: LogFormat::Text };
        let result = init_logging(&config2, true); 
        assert!(result.is_ok(), "Reloading logging should not error, but got: {:?}", result.err());
        tracing::info!("Reload test: Info after first init."); // Should follow config1 rules
        tracing::debug!("Reload test: Debug after attempting reload."); // Behavior depends on how `try_init` handles re-init with different layers. Sentry docs say it replaces.
    }

    #[test]
    fn test_init_logging_reload_false_errors_if_already_set() {
        ensure_clean_logger_state();
        let config1 = LoggingConfig { log_level: "info".to_string(), log_output: LogOutput::Stdout, log_format: LogFormat::Text };
        init_logging(&config1, false).expect("First init failed");

        let config2 = LoggingConfig { log_level: "debug".to_string(), log_output: LogOutput::Stdout, log_format: LogFormat::Text };
        let result = init_logging(&config2, false);
        assert!(result.is_err(), "Second init with is_reload=false should error");
        match result.err().unwrap() {
            CoreError::Logging(msg) => {
                assert!(msg.contains("Failed to set global tracing subscriber"));
            }
            other_error => panic!("Unexpected error type: {:?}", other_error),
        }
    }
}
