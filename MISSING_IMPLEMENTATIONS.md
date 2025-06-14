# NovaDE Missing Implementations & Tasks

This document lists features, components, and refinements that appear to be missing or incomplete based on an analysis of the source code against available documentation and inferred project goals. It serves as a potential task list for further development.

## Table of Contents
- [Core Layer (`novade-core`)](#core-layer-novade-core)
- [Domain Layer (`novade-domain`)](#domain-layer-novade-domain)
- [System Layer (`novade-system`)](#system-layer-novade-system)
- [UI Layer (`novade-ui`)](#ui-layer-novade-ui)
- [Feature Specifics & Refinements](#feature-specifics--refinements)
- [General/Cross-Cutting](#generalcross-cutting)

---

## Core Layer (`novade-core`)

### Configuration (`src/config/`)
- **Generic Config Components (as per SPEC-MODULE-CORE-CONFIG-v1.0.0.md):**
    - Implement the generic `ConfigProvider` trait.
    - Implement the full `ConfigManager` with dynamic capabilities:
        - `set_value`
        - `remove_value`
        - `save_config`
    - Implement `ConfigStore` for in-memory representation if different from just `CoreConfig`.
    - Implement `ConfigPath` abstraction for hierarchical key access if current struct field access is insufficient for dynamic paths.
    - Implement `ConfigValue` enum if a more dynamic value representation than direct struct fields is needed by `ConfigManager`.
- **Multiple Configuration Format Support:**
    - Add parsers and loaders for YAML, JSON, INI, and ENV variables as specified in `ConfigFormat` and `ConfigLoader` spec. Current implementation is TOML-focused.
- **Configuration Watching (`ConfigWatcher`):**
    - Implement live monitoring of configuration files for changes and reloading, as specified.
- **Schema-Based Validation (`ConfigSchema`):**
    - Implement the `ConfigSchema` structure and associated validation logic in `ConfigValidator` or `ConfigManager` to validate configurations against a defined schema, as per spec. Current validation is basic and hardcoded.
- **System Health Dashboard Config:**
    - Add `diagnostic_ping_target: Option<String>` to `SystemHealthDashboardConfig` in `novade-core/src/types/system_health.rs` and update its `Default` impl and TOML parsing if necessary.

### Error Handling (`src/error.rs`)
- **Abstract Error Components (as per SPEC-MODULE-CORE-ERRORS-v1.0.0.md):**
    - Implement `ErrorKind` enum for fine-grained error categorization if current enums are insufficient.
    - Implement `ErrorContext` struct for richer contextual information attached to errors.
    - Implement `ErrorReporter`, `ErrorHandler`, and `ErrorPolicy` components for a more structured and configurable error handling/reporting strategy.
- **Tooling Integration:**
    - Explicitly integrate `anyhow` for error propagation patterns if desired for more ergonomic error handling in library code.
    - Integrate `backtrace` crate for capturing backtraces with errors, as mentioned in spec.

### Logging (`src/logging.rs`)
- **Abstract Logging Components (as per SPEC-MODULE-CORE-LOGGING-v1.0.0.md):**
    - Implement `LogManager` as a central explicit management component if current `tracing` setup is not sufficient.
    - Implement `LoggerFactory` if dynamic creation of distinctly configured loggers is needed beyond `tracing`'s target-based filtering.
    - Define and implement `LogAppender`, `LogFilter`, `LogFormatter` traits if custom extensions beyond `tracing-subscriber` layers are required.
    - Implement `Logger` struct with per-logger level control if needed beyond `tracing`'s per-target/module filtering.
- **WorkerGuard Handling:**
    - Address the `TODO` for proper `WorkerGuard` handling in async file logging to ensure logs are always flushed, especially on shutdown. This involves storing the guard for the lifetime of the application.

### Utilities (`src/utils/`)
- **Filesystem Utilities:**
    - Evaluate if more fs utilities are needed based on `SPEC-COMPONENT-CORE-UTILS` (e.g., file writing, copying, listing, permissions).
- **String/Async Utilities:**
    - Re-evaluate if `string_utils` or `async_utils` (mentioned as removed from spec) are needed elsewhere or if their functionality should be re-introduced if core tasks require them.

---

## Domain Layer (`novade-domain`)

### Global Settings (`src/global_settings/`)
- **Transactional Updates:** Consider if batching multiple setting updates into a single transaction (and single save operation) is required.
- **Schema Definition/Migration:** Define a clear process or mechanism for managing the evolution of the `GlobalDesktopSettings` schema and handling data migrations if the structure changes between versions.
- **Fine-grained Validation:** Implement detailed validation rules per setting within `GlobalDesktopSettings::validate_recursive()` or via a separate schema mechanism if not already exhaustive.

### Theming (`src/theming/`)
- **`ThemeProvider` Implementation:** Provide concrete implementations for `ThemeProvider` (e.g., filesystem-based that loads themes from standard XDG dirs or app-specific theme dirs). The current `DefaultThemingEngine` uses a mock in tests. `initialize_domain_layer` in `novade-domain/lib.rs` sets up paths, but the actual file loading logic for themes and tokens by `ThemingEngine::new` wasn't fully explored.
- **Dynamic Token Overrides:** Flesh out `custom_user_token_overrides` in `ThemingConfiguration` and ensure the engine can apply these.

### AI Interaction (`src/ai_interaction_service/`, `src/user_centric_services/ai_interaction/`)
- **Full Feature Implementation:** The structure for AI services, consent, and model profiles is present. A full review against detailed AI feature specifications is needed to identify specific missing logic or capabilities.
- **Error Handling:** Ensure `AIInteractionError` covers all necessary cases.

### Notifications (`src/notifications_rules/`, `src/user_centric_services/notifications_core/`)
- **Persistence for Notifications:** While rules have a `FilesystemNotificationRulesProvider`, ensure `DefaultNotificationService` also has robust persistence for active/historical notifications if required by specs (e.g., showing missed notifications after reboot).
- **Action Handling:** Complete implementation for all `RuleAction` types in the rules engine and their execution via `NotificationService`.

### Window Management Policy (`src/window_management_policy/`)
- **Policy Details:** The `types.rs` within this module would define various policies (tiling, focus, etc.). Ensure all specified policy options and their enforcement logic are implemented.
- **Integration with System Layer:** Define how these domain-level policies are communicated to and enforced by the `novade-system` window manager/compositor.

### Power Management (`src/power_management/`)
- **Full Service Logic:** The structure for `PowerManagementError` exists. The actual service implementation connecting to system power services (e.g., UPower via D-Bus) needs to be fully implemented.

### General Domain Layer
- **Complete Service Implementations:** For modules like `cpu_usage_service`, ensure the services are fully implemented beyond stubs or basic structures.
- **Inter-Service Communication/Events:** Refine and complete the event-driven communication between different domain services.

---

## System Layer (`novade-system`)

### Compositor (`src/compositor/`)
- **Rendering - WGPU:**
    - **DMABUF Import:** Implement `create_texture_from_dmabuf()` in `NovaWgpuRenderer` for zero-copy buffer sharing. This is critical for performance.
    - **RenderElement Support:** Fully implement rendering for `RenderElement::SolidColor` and `RenderElement::Cursor` in `NovaWgpuRenderer::render_frame()`.
    - **OpenGL/Vulkan Paths:** Clarify the status of the GLES2 and Vulkan rendering paths within `DesktopState::commit()` and `compositor/renderers/`. If WGPU is the sole focus, remove or properly stub out the old paths. If they are meant as fallbacks, they need to be updated to work with the current `FrameRenderer` trait and `DesktopState` structure.
- **Shell Protocols:**
    - **Layer Shell:** Investigate and implement `wlr-layer-shell` or equivalent for panels, notifications, wallpapers if not already complete in `compositor/shell/`. The `NovaDE Compositor Entwicklungsplan.md` mentioned this.
    - **Other Protocols:** Implement other relevant Wayland protocols as per full system specification (e.g., input methods, screen sharing, activation, etc.).
- **Output Management:** Full handling for multi-monitor dynamic configuration, mode setting, and events based on `smithay::output::OutputManagerState`.
- **Damage Tracking:** Ensure optimal damage tracking and repaint scheduling is fully implemented and tested with the WGPU renderer.
- **Scene Graph / Composition Logic:** Verify `composition_engine.rs` and `scene_graph.rs` for complex scene management, including transparency, layering, and transformations.
- **XWayland Integration:** The compositor plans mentioned XWayland support. Investigate its current status.

### Input (`src/input/`)
- **`FocusManager` Implementation:** Ensure `FocusManager` correctly handles focus logic for keyboard, pointer, and touch, and dispatches events to Wayland clients via `smithay::input::Seat`.
- **Device Configuration:** Full application of `InputConfig` to all device types and specific devices.
- **Udev Hotplugging:** Complete and test udev hotplug handling for input devices via `UdevHandler` and `DeviceManager`.
- **Advanced Input Features:** Implement features like tablet input, gestures (if planned beyond basic touch), keyboard layouts/switching fully.

### D-Bus Integration (`src/dbus_integration/`)
- **Service Exposure:** Implement D-Bus services that NovaDE system layer needs to expose (e.g., screenshot, input configuration, compositor information).
- **Service Consumption:** Implement clients for system D-Bus services NovaDE depends on (e.g., systemd-logind, UPower, NetworkManager, if not handled by domain layer directly).
- **`DbusServiceManager`:** Fully implement this for managing D-Bus service registrations and client proxies.

### Other System Services
- **Audio Management (`audio_management.rs`):** Implement.
- **Display Management (`display_management.rs`):** Implement (distinct from compositor output management, could be for display properties, brightness etc.).
- **Network Manager (`network_manager/`):** Implement system-level network interactions if any, or ensure domain layer handles it.
- **Power Management (`power_management/`):** Implement system-level power interactions (e.g., initiating sleep/hibernate, responding to power events).

---

## UI Layer (`novade-ui`)

### UI Strategy Clarification
- **Dual Toolkits (GTK vs. Iced):** The most critical task is to clarify the UI strategy. Determine if both GTK/Libadwaita (in `main.rs`) and Iced (in `app.rs`/`lib.rs`) are intended to coexist, if one is primary, or if one is to be deprecated. This impacts all other UI tasks.
- **Focus for Documentation:** Assuming GTK/Libadwaita is primary based on current activity, the following points focus there.

### GTK/Libadwaita UI
- **Domain Service Integration:**
    - **Settings (`settings_ui.rs`):** Fully connect `NovaSettingsWindow` to `novade-domain::global_settings::GlobalSettingsService` to load, display, and save actual settings. Implement UI for all settings categories.
    - **Theming (`theming_gtk.rs`):** Implement proper integration with `novade-domain::theming::ThemingEngine`. Subscribe to theme changes, apply them to the GTK application (e.g., AdwStyleManager, dynamic CSS). Provide UI for theme selection.
    - **Notifications (`notification_ui.rs`, `notification_client/`):** Fully integrate `NotificationClient` with `novade-domain::user_centric_services::notifications_core::NotificationService` (likely via D-Bus if the service is there). Implement `NotificationPopupWidget` display logic.
    - **Application Launcher (`application_launcher.rs`):** Connect to domain/system services for application discovery and launching.
    - **Other Services:** Connect UI elements (e.g., panel widgets) to relevant domain services (CPU usage, network status, power status, active window information from `ActiveWindowService`).
- **Shell Components (`src/shell/`):**
    - **Panel Widgets:** Complete implementation and functionality for all planned panel widgets. Ensure they are robust and efficient.
    - **Taskbar/Window List:** Implement a functional taskbar within `SimpleTaskbar` or a dedicated component.
    - **Desktop Experience:** Implement desktop right-click menus, icon views (if planned).
- **Missing UI Modules (from `lib.rs` re-exports):**
    - `window_manager_ui.rs`: Define and implement.
    - `desktop_ui.rs`: Define and implement.
    - `panel_ui.rs` (beyond the panel widgets, the main orchestrator if any).
    - `theme_ui.rs` (actual UI for theme selection).
    - `workspace_ui.rs` (UI elements for workspace interaction beyond the indicator, e.g., overview).
    - `system_tray.rs`: Implement.
- **Error Handling:** Implement user-facing error dialogues or notifications for errors originating from UI interactions or underlying service failures.
- **Compositor Integration (`compositor_integration.rs`):** Clarify and implement its role. This might involve custom Wayland protocols or interfaces for UI-compositor interaction if needed beyond what standard protocols provide.
- **Input Integration (`input_integration.rs`):** Clarify and implement its role, especially if handling specific UI input behaviors not covered by general Wayland input.

### Iced UI (if to be continued)
- **Clarify Purpose:** Define its role in the NovaDE architecture.
- **Complete Sub-components:** Fully implement `DesktopUi`, `PanelUi`, `WindowManagerUi`, etc., within the Iced framework.
- **SystemContext Integration:** Ensure `novade_system::SystemContext` is fully utilized and provides necessary data/abstractions for an Iced UI.

---

## Feature Specifics & Refinements

### System Health Dashboard

- **Domain Layer (`novade-domain`):**
  - **Alarm Logic:**
    - Implement full duration-based alerting (e.g., CPU high for X seconds *consecutively*). Current implementation checks instantaneous values.
    - Add alerts for other metrics (network inactivity, high temperature) as specified or desired, based on `SystemHealthDashboardConfig`.
    - Persist active alerts across service restarts (optional, would require a persistence strategy).
    - Provide a mechanism to acknowledge or clear alerts via the `SystemHealthService` (e.g., `acknowledge_alert(alert_id: String)`).
  - **Periodic Evaluation Task:** Implement a proper background task/ticker within `DefaultSystemHealthService` to call `evaluate_alerts` periodically, instead of the current approach where `get_active_alerts` triggers it. This ensures alerts are evaluated even if the UI isn't actively polling.
- **System Layer (`novade-system`):**
  - **Log Streaming:** Enhance `JournaldLogHarvester::stream_logs` to use `sd_journal_wait()` for more efficient, non-polling based streaming, likely involving `tokio::task::spawn_blocking` or an async-friendly journald library.
  - **Disk SMART Diagnostics:** Fully implement the "Disk SMART Health" test in `BasicDiagnosticsRunner` or a new dedicated runner. This includes:
    - Discovering available block devices (e.g., by listing `/dev/sd*`, `/dev/nvme*` or parsing `/proc/partitions`).
    - Dynamically generating `DiagnosticTestInfo` for each relevant disk.
    - Executing `smartctl -H /dev/sdX` (or equivalent) and parsing its output to determine health status (`Passed`, `Failed`, etc.) and relevant attributes to include in `DiagnosticTestResult::details`.
  - **Collector Robustness:** Improve error handling and data validation in all collectors (e.g., handling missing `/proc` or `/sys` files gracefully for specific metrics, more specific error types instead of generic strings in `MetricCollectorError`).
  - **Configurable Ping Target:** Ensure the ping target for `BasicDiagnosticsRunner` is configurable via `CoreConfig` (e.g. add `diagnostic_ping_target: Option<String>` to `SystemHealthDashboardConfig`).
- **UI Layer (`novade-ui`):**
  - **Log Viewer Panel:**
    - Implement live log streaming UI controls (Start/Stop buttons) and connect to `SystemHealthService::stream_logs`. Display new entries as they arrive.
    - Add UI for selecting a `TimeRange` for log queries.
    - Add UI for filtering by log component/source if `LogFilter::component_filter` is to be used.
    - Improve log entry formatting and presentation (e.g., custom styling for different levels beyond basic tags, clickable timestamps, structured field display).
  - **Metrics Panel:**
    - Consider graphical representations for metrics (e.g., simple sparklines, bar charts, or small line graphs using GTK drawing capabilities or a charting library) instead of just labels for better visualization.
    - Allow configuration of which metrics/devices are displayed or filterable within the panel (e.g., selecting specific network interfaces or disk devices).
  - **Diagnostics Panel:**
    - Provide more detailed progress updates during test execution, especially for potentially long-running tests.
    - Allow cancellation of running tests if feasible.
  - **Alerts Panel:**
    - Implement UI for acknowledging or clearing alerts (requires corresponding methods in `SystemHealthService`).
    - Allow sorting and filtering of displayed alerts (e.g., by severity, timestamp, acknowledged status).
  - **General UI:**
    - Provide a user-friendly way to display configuration errors if `SystemHealthDashboardConfig` has issues (e.g., invalid thresholds).
    - Implement more polished error display within panels (e.g., using `gtk::InfoBar` or dedicated error labels) instead of just setting primary widget text to an error message.
    - Internationalization for all UI text using `gettextrs` or a similar library.
    - Ensure all UI updates triggered by async operations are correctly performed on the GTK main thread (current use of `glib::MainContext::default().spawn_local` is good, maintain this pattern).

---

## General/Cross-Cutting
- **Documentation:**
    - Create/update detailed specification documents for modules where they are missing or where implementation has significantly diverged (especially for UI strategy and compositor rendering).
    - Add comprehensive code comments and Rustdoc documentation.
- **Testing:**
    - Expand unit test coverage across all layers (many TODOs added for this).
    - Implement integration tests for service interactions between layers (TODOs added for this).
    - Implement UI tests (if feasible with chosen toolkits) (TODOs added for this).
- **Performance Profiling and Optimization:** Conduct thorough performance testing, especially for compositor, rendering, and high-frequency domain events/metric collection.
- **Security Review:** Conduct a security review, especially for D-Bus interfaces, file handling, and process management (e.g., `ping` and `smartctl` execution).
- **Build System & CI/CD:**
    - Ensure `Cargo.toml` files correctly reflect inter-crate dependencies.
    - Refine build scripts (`build.rs` in `novade-ui`) for robustness.
    - Expand CI/CD pipeline for more comprehensive testing and checks.

---
*(End of `MISSING_IMPLEMENTATIONS.md`)*
