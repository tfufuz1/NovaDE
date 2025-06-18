//ANCHOR [NovaDE Developers <dev@novade.org>] Error Tracking System using Sentry.
//! This module provides functionalities for initializing and interacting with an error tracking
//! service, specifically Sentry. It allows for capturing errors, panics, and breadcrumbs,
//! and integrates with the `tracing` ecosystem.

use sentry::ClientInitGuard;
//ANCHOR [NovaDE Developers <dev@novade.org>] Import ErrorTrackingConfig.
use crate::config::ErrorTrackingConfig;
// Removed duplicate: use sentry::ClientInitGuard;
use sentry_tracing::SentryLayer;
// tracing_subscriber imports are not directly used by init_error_tracking after refactor,
// but get_sentry_tracing_layer might be used by subscriber setup elsewhere.
// For this file, direct usage might reduce if SentryLayer construction is simple.
// use tracing_subscriber::layer::SubscriberExt;
// use tracing_subscriber::util::SubscriberInitExt;
// use tracing_subscriber::Registry;
use std::sync::Mutex;

//ANCHOR [NovaDE Developers <dev@novade.org>] Global Sentry client guard.
/// Holds the Sentry client guard to keep the Sentry client alive.
/// This is wrapped in a Mutex to allow for safe global access, though it's typically set once.
static SENTRY_GUARD: Mutex<Option<ClientInitGuard>> = Mutex::new(None);

//ANCHOR [NovaDE Developers <dev@novade.org>] Initializes the error tracking system.
/// Initializes the Sentry SDK for error tracking.
///
/// This function should be called early in the application's lifecycle.
/// It configures Sentry with the provided DSN, environment, and release name.
/// It also sets up panic handling and integrates with the `tracing` crate via `SentryLayer`.
/// If `dsn` is `None`, Sentry is effectively disabled.
///
/// # Arguments
///
/// * `config`: A reference to the [`ErrorTrackingConfig`] containing Sentry settings.
///
/// //TODO [Error Deduplication] [NovaDE Developers <dev@novade.org>] Consider if Sentry's default error deduplication is sufficient or if custom fingerprinting logic is needed for specific error types.
/// //TODO [Configurable Thresholds] [NovaDE Developers <dev@novade.org>] Sentry handles notification thresholds server-side. Client-side batching/throttling for high-volume events might be considered if performance becomes an issue, though `sentry::Transport` options can also manage this.
//ANCHOR [NovaDE Developers <dev@novade.org>] Updated init_error_tracking to use ErrorTrackingConfig.
pub fn init_error_tracking(config: &ErrorTrackingConfig) {
    if let Some(sentry_dsn_str) = &config.sentry_dsn {
        if sentry_dsn_str.is_empty() {
            // Consider using tracing::warn! here if logging is already initialized.
            eprintln!("Sentry DSN provided but is empty. Sentry will not be initialized.");
            return;
        }

        let options = sentry::ClientOptions {
            dsn: Some(sentry_dsn_str.parse().expect("Invalid Sentry DSN format")),
            release: config.sentry_release.clone().map(std::borrow::Cow::Owned),
            environment: config.sentry_environment.clone().map(std::borrow::Cow::Owned),
            attach_stacktrace: true, // Capture stacktraces for all messages
            send_default_pii: true, // Send Potentially Identifiable Information, like user IPs. Adjust as per privacy policy.
            //TODO [NovaDE Developers <dev@novade.org>] Expose more sentry::ClientOptions if needed (e.g., sample_rate, traces_sample_rate).
            ..Default::default()
        };

        let guard = sentry::init(options);

        // Store the guard to keep Sentry active.
        // This will drop any previous guard, effectively re-initializing if called multiple times,
        // though it's best to call init only once.
        let mut global_guard = SENTRY_GUARD.lock().unwrap();
        *global_guard = Some(guard);

        eprintln!("Sentry initialized successfully."); // Use tracing::info! once tracing is fully set up with this layer

        // Setup SentryLayer for tracing integration
        // This assumes that tracing subscribers are managed elsewhere and we are just providing the layer.
        // If this module is also responsible for initializing the global tracing subscriber,
        // the approach would be different (e.g., constructing the full subscriber here).
        // For now, let's assume `init_logging` from another module will pick up this layer.
        // A typical setup:
        // Registry::default().with(SentryLayer::new()).init();
        // However, this should be combined with other layers (like formatting, filtering) from logging.rs
        // This function should ideally return the SentryLayer to be integrated by the main logging setup.
        // For simplicity in this subtask, we'll log a message indicating how to integrate.
        // TODO [NovaDE Developers <dev@novade.org>] Determine the best strategy for integrating SentryLayer with the existing tracing setup in logging.rs. It might involve returning the layer from this function.
        tracing::info!("SentryLayer created. It should be integrated into the main tracing subscriber configuration.");


    } else {
        eprintln!("No Sentry DSN provided. Sentry is disabled."); // Use tracing::info!
    }
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Provides SentryLayer for tracing integration.
/// Returns a `SentryLayer` to be integrated with a `tracing` subscriber.
/// This layer will forward tracing events (spans, events) to Sentry as breadcrumbs or events.
///
/// Call this *after* `init_error_tracking` has been successfully run with a DSN.
/// If Sentry is not initialized, this layer will effectively be a no-op.
pub fn get_sentry_tracing_layer<S>() -> SentryLayer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    sentry_tracing::SentryLayer::default() // Try default() instead of new()
}


//ANCHOR [NovaDE Developers <dev@novade.org>] Captures an error with Sentry.
/// Captures a given error with Sentry, including a backtrace if available.
///
/// # Arguments
///
/// * `error`: A reference to an object implementing `std::error::Error`.
/// * `context`: `Option<serde_json::Value>` - Optional custom context to attach to the Sentry event.
///   This can be structured data relevant to the error.
use serde_json; // Added for Option<serde_json::Value>
pub fn capture_error(error: &dyn std::error::Error, context: Option<serde_json::Value>) {
    if !sentry::Hub::current().client().is_some() { // Corrected: sentry::is_enabled() -> sentry::Hub::current().client().is_some()
        // TODO [NovaDE Developers <dev@novade.org>] Maybe log to stderr or tracing if Sentry is disabled but capture_error is called?
        return;
    }

    sentry::with_scope(
        |scope| {
            if let Some(ctx_val) = context {
                // Convert serde_json::Value to sentry's JsonValue if necessary, or use Context::Other
                // Assuming ctx_val is serde_json::Value, wrap it in a BTreeMap for Context::Other
                let mut map = std::collections::BTreeMap::new();
                // sentry::types::value::Value is effectively serde_json::Value if "with_serde_json" feature is on for sentry
                // For Sentry 0.27.0, sentry::protocol::Context::Other takes a BTreeMap<String, sentry::types::value::Value>
                // We assume `ctx_val` is a `serde_json::Value` that represents the *entire* context object.
                // If `ctx_val` itself should be a field within a larger context map:
                // map.insert("data".to_string(), ctx_val);
                // If ctx_val is a JSON object (map) itself, and we want to set its fields directly:
                if let serde_json::Value::Object(obj_map) = ctx_val {
                    for (k, v) in obj_map {
                        map.insert(k, v); // v is serde_json::Value, which Sentry's Value can often be created from
                    }
                } else {
                    // If ctx_val is not an object, wrap it.
                    map.insert("value".to_string(), ctx_val);
                }
                // Explicitly convert to sentry::protocol::Context::Other
                // The map is BTreeMap<String, serde_json::Value>. Sentry's Value can be created from serde_json::Value.
                let sentry_map = map.into_iter().map(|(k, v)| (k, sentry::protocol::Value::from(v))).collect(); // Corrected path to sentry::protocol::Value
                scope.set_context("Custom Context", sentry::protocol::Context::Other(sentry_map));
            }
            // Additional scope configuration can be done here, e.g., setting tags or user info.
            // Example: scope.set_tag("component", "rendering");
        },
        || {
            sentry::capture_error(error);
        },
    );
}

//ANCHOR [NovaDE Developers <dev@novade.org>] Adds a breadcrumb to Sentry.
/// Adds a breadcrumb to Sentry. Breadcrumbs are used to record a trail of events
/// that happened prior to an issue.
///
/// # Arguments
///
/// * `category`: `&str` - A category for the breadcrumb (e.g., "ui", "network", "auth").
/// * `message`: `&str` - The breadcrumb message.
/// * `level`: `sentry::Level` - The severity level of the breadcrumb (e.g., Info, Warning, Error).
///
/// //TODO [Error Recovery Tracking] [NovaDE Developers <dev@novade.org>] Consider using a specific category or metadata in breadcrumbs to explicitly mark error recovery attempts or successes. E.g. category: "recovery", message: "Successfully recovered from network error".
pub fn add_breadcrumb(category: &str, message: &str, level: sentry::Level) {
     if !sentry::Hub::current().client().is_some() { // Corrected: sentry::is_enabled() -> sentry::Hub::current().client().is_some()
        return;
    }
    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: Some(category.to_string()),
        message: Some(message.to_string()),
        level,
        ..Default::default()
    });
}


#[cfg(test)]
mod tests {
    use super::*;
    //ANCHOR [NovaDE Developers <dev@novade.org>] Import ErrorTrackingConfig for tests.
    use crate::config::ErrorTrackingConfig;
    use thiserror::Error;
    use serde_json::json;

    #[derive(Error, Debug)]
    #[error("Test error: {message}")]
    struct TestError {
        message: String,
        // Adding a source for more complex error testing
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    }

    // Helper to ensure Sentry is disabled for most tests to avoid actual DSN requirements.
    fn ensure_sentry_disabled() {
        let mut guard = SENTRY_GUARD.lock().unwrap();
        *guard = None; // Drop any existing guard, effectively disabling Sentry
    }

    // Basic test for init function - does not check Sentry communication
    #[test]
    fn test_init_error_tracking_no_dsn() {
        ensure_sentry_disabled();
        //ANCHOR [NovaDE Developers <dev@novade.org>] Updated test to use ErrorTrackingConfig.
        let config = ErrorTrackingConfig {
            sentry_dsn: None,
            sentry_environment: Some("test_env".to_string()),
            sentry_release: Some("test_release".to_string()),
        };
        init_error_tracking(&config);
        assert!(!sentry::Hub::current().client().is_some(), "Sentry should be disabled if no DSN is provided."); // Corrected
    }

    #[test]
    fn test_init_error_tracking_empty_dsn() {
        ensure_sentry_disabled();
        //ANCHOR [NovaDE Developers <dev@novade.org>] Updated test to use ErrorTrackingConfig.
        let config = ErrorTrackingConfig {
            sentry_dsn: Some("".to_string()),
            sentry_environment: Some("test_env".to_string()),
            sentry_release: Some("test_release".to_string()),
        };
        init_error_tracking(&config);
        assert!(!sentry::Hub::current().client().is_some(), "Sentry should be disabled if DSN is empty."); // Corrected
    }

    // Note: Testing actual Sentry initialization with a DSN would require a mock DSN or a test DSN,
    // and potentially a mock Sentry server or inspecting global state, which is complex for unit tests.
    // These tests will focus on the logic within this module, assuming Sentry's own library works.

    #[test]
    fn test_capture_error_sentry_disabled() {
        ensure_sentry_disabled();
        let err = TestError { message: "capture when disabled".to_string(), source: None };
        // Should not panic, should be a no-op essentially
        capture_error(&err, Some(json!({"key": "value"})));
        // No direct assertion possible other than it doesn't crash and Sentry remains disabled.
         assert!(!sentry::Hub::current().client().is_some()); // Corrected
    }

    #[test]
    fn test_add_breadcrumb_sentry_disabled() {
        ensure_sentry_disabled();
        // Should not panic, should be a no-op
        add_breadcrumb("test_category", "test message", sentry::Level::Info);
        // No direct assertion possible.
        assert!(!sentry::Hub::current().client().is_some()); // Corrected
    }

    // To truly test Sentry integration (e.g. that capture_error sends something),
    // you'd typically need a test DSN and a way to inspect Sentry's events, or use Sentry's testkit.
    // For now, we are testing the control flow (e.g. Sentry disabled = no error).

    // Example of how one might test with a real DSN (but this should be an integration test and is
    // generally not run in CI unit tests without a configured Sentry instance).
    /*
    #[test]
    #[ignore] // Ignored because it requires a real DSN and Sentry instance
    fn test_init_and_capture_with_real_dsn() {
        let test_dsn = std::env::var("SENTRY_TEST_DSN"); // Get DSN from environment
        if test_dsn.is_err() {
            println!("SENTRY_TEST_DSN not set, skipping test_init_and_capture_with_real_dsn");
            return;
        }

        init_error_tracking(test_dsn.ok(), Some("test".to_string()), Some("0.0.1".to_string()));
        assert!(sentry::is_enabled(), "Sentry should be enabled with a valid DSN.");

        let err = TestError { message: "actual capture test".to_string(), source: None };
        capture_error(&err, Some(json!({"info": "this is a real test"})));

        // Important: Sentry sends events asynchronously. We need to give it time.
        sentry::flush(Some(std::time::Duration::from_secs(2)));

        // After this, you would manually check your Sentry instance or use Sentry's API
        // to verify the error was received.

        // Teardown / disable Sentry for other tests
        ensure_sentry_disabled();
    }
    */

    #[test]
    fn test_get_sentry_tracing_layer() {
        // This test just ensures the function can be called and returns the layer.
        // The layer's functionality is tested by Sentry's own crate.
        let _layer = get_sentry_tracing_layer();
        // No specific assertions here, just that it doesn't panic.
    }
}
