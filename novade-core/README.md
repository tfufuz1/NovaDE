# novade-core

<!-- ANCHOR [NovaDE Developers <dev@novade.org>] README v1.1 -->

Core layer for the NovaDE desktop environment, providing fundamental building blocks and utilities.

## Overview

The `novade-core` crate provides the foundational components used throughout the NovaDE desktop environment. It aims to offer a stable, well-tested, and ergonomic toolkit for common desktop environment tasks. This includes:

*   **Unified Error Handling**: Robust error types (`CoreError`, `ConfigError`, etc.) using `thiserror` for managing various failure scenarios.
*   **Structured Logging**: A flexible logging framework built upon the `tracing` crate (`tracing`, `tracing-subscriber`, `tracing-appender`). It supports multiple log levels, JSON/text formats, stdout/file outputs, and log rotation. Configuration is highly flexible, including runtime adjustments via `RUST_LOG`.
*   **Error Tracking**: Integration with Sentry for automatic error and panic reporting, configurable via DSN. Includes support for breadcrumbs and custom context.
*   **Configuration Management**: A TOML-based system (`serde`, `toml`) for loading application configuration (`CoreConfig`) with default fallbacks and validation capabilities. See `docs/CONFIGURATION.md` for detailed options.
*   **Core Data Types**: Fundamental data structures for geometry, color representation, application identification, system health metrics, and more.
*   **Utility Functions**: Common helper functions, for example, for filesystem operations and XDG path resolution.

## Features

*   **Error Handling**: Unified error system (`CoreError`, specific errors like `ConfigError`).
*   **Structured Logging**:
    *   Based on `tracing`.
    *   Configurable levels, outputs (stdout, file), formats (JSON, text).
    *   Log rotation (daily, size-based placeholder).
    *   Runtime filtering via `RUST_LOG`.
    *   Refer to `docs/SPEC-CORE-LOGGING-SYSTEM-v1.1.0.md` for system design.
*   **Error Tracking (Sentry)**:
    *   Automatic panic reporting.
    *   Manual error reporting with `capture_error()`.
    *   Breadcrumb support with `add_breadcrumb()`.
    *   Configurable via `ErrorTrackingConfig`.
*   **Configuration System**:
    *   Loads `config.toml` into `CoreConfig`.
    *   Provides default values for all settings.
    *   Global access to configuration.
    *   Includes settings for logging, error tracking, metrics exporter (Prometheus), debug interface, and feature flags.
    *   Detailed options in `docs/CONFIGURATION.md`.
*   **Basic Data Types**: Geometry, Color, AppIdentifier, Status, System Health Metrics, etc.
*   **Utilities**: Filesystem helpers, XDG path resolution.

## Usage

Add `novade-core` as a dependency in your `Cargo.toml`:
```toml
[dependencies]
novade-core = { path = "../novade-core" } # Or version from crates.io
```

Initialize core services (like logging and configuration) early in your application:
```rust,ignore
use novade_core::config::{ConfigLoader, initialize_core_config, get_core_config, CoreConfig};
use novade_core::logging::init_logging;
use novade_core::error_tracking::init_error_tracking;
use novade_core::error::CoreError;

fn main() -> Result<(), CoreError> {
    // 1. Load configuration
    let core_config = ConfigLoader::load().unwrap_or_else(|e| {
        eprintln!("Failed to load config.toml (Error: {}), using default configuration.", e);
        CoreConfig::default()
    });

    // 2. Initialize global config (must be done only once)
    if let Err(_) = initialize_core_config(core_config.clone()) {
        eprintln!("Warning: CoreConfig was already initialized. This might be an issue if configurations differ.");
    }
    let active_config = get_core_config(); // Now global config is accessible

    // 3. Initialize logging based on the loaded (or default) configuration
    // The `false` for is_reload indicates this is the initial setup.
    if let Err(e) = init_logging(&active_config.logging, false) {
        eprintln!("Failed to initialize logging: {}. Some logs might be missed.", e);
    }

    // 4. Initialize error tracking (Sentry)
    init_error_tracking(&active_config.error_tracking);

    tracing::info!("NovaDE Core initialized successfully with configured logging and error tracking.");

    // ... your application logic ...

    // Example: Capturing an error
    // if let Err(e) = some_fallible_operation() {
    //     novade_core::error_tracking::capture_error(&e, None);
    // }

    Ok(())
}
```

Refer to `docs/DEVELOPER_GUIDE.md` for more detailed usage of logging and error tracking APIs.

<!-- ANCHOR_END -->
