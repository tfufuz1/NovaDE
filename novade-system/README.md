# novade-system

<!-- ANCHOR [NovaDE Developers <dev@novade.org>] README v1.1 -->

System layer for the NovaDE desktop environment.

## Overview

The `novade-system` crate provides the implementations for system-level services and interfaces required by the NovaDE desktop environment. It acts as the bridge between the abstract domain logic (`novade-domain`) and the underlying operating system capabilities. This includes the Wayland compositor, input handling, D-Bus interfaces, system health monitoring, and various other hardware and OS interactions.

It relies on `novade-core` for foundational utilities like logging, configuration, and core data types.

## Key Features & Modules

*   **Compositor**: Wayland compositor implementation (using Smithay).
*   **Input Handling**: Manages input devices (e.g., keyboard, mouse, touch) via libraries like `libinput` and `xkbcommon`.
*   **D-Bus Interfaces**: Provides and consumes D-Bus services for inter-process communication within the desktop.
*   **Audio Management**: Integration with PipeWire for audio stream and device management.
*   **System Health & Monitoring (`system_health_collectors`)**:
    *   **Performance Collectors**: Modules for gathering metrics on CPU, memory (system-wide and per-subsystem), frame timings (for compositor performance), and GPU (NVIDIA via NVML, with TODOs for AMD/Intel).
    *   **Metrics Exporter**: Exposes collected system metrics in Prometheus format via an HTTP endpoint (`/metrics`). This is configurable and can be enabled/disabled. See `docs/CONFIGURATION.md` and `docs/features/SYSTEM-HEALTH-MONITORING-AND-DIAGNOSTICS-v1.0.0.md`.
    *   **Regression Detection**: Basic mechanisms within collectors to detect performance regressions against baselines. A more generic `RegressionDetector` is also provided.
*   **Debug Interface (`debug_interface`)**:
    *   Provides placeholder functionalities for runtime introspection, state dumping, and control of profiling/memory tools.
    *   Designed to be configurable (enabled/disabled, address for access).
    *   Intended for developer use and advanced diagnostics. Details in `docs/features/SYSTEM-HEALTH-MONITORING-AND-DIAGNOSTICS-v1.0.0.md`.
*   **Power Management**: Interface for system power operations (suspend, hibernate, reboot, shutdown).
*   **Filesystem Service**: Provides filesystem interaction capabilities.
*   **Application Management**: Manages application lifecycle and information.
*   **System Settings Service**: Manages system-wide settings.
*   **Window Info Provider**: Provides information about active windows.
*   **Portals**: (Planned) Backend for XDG Desktop Portals.
*   **MCP Client**: (If applicable) Client for the Model Context Protocol.

## Configuration

Many features of `novade-system`, particularly the metrics exporter and debug interface, are configurable via the central `config.toml` file processed by `novade-core`. Refer to `docs/CONFIGURATION.md` for detailed options.

## Dependencies

Key dependencies include:
- `novade-core` (for core utilities, types, logging, config)
- `novade-domain` (for domain logic interfaces)
- `smithay` (for Wayland compositor)
- `libinput`, `xkbcommon` (for input)
- `zbus` (for D-Bus)
- `pipewire-rs` (for audio)
- `psutil`, `prometheus`, `warp`, `nvml-wrapper` (for monitoring and metrics)
- `serde`, `serde_json`, `tokio`

## Thread Safety

Modules in the System Layer are designed with thread safety in mind, utilizing appropriate synchronization primitives like `Arc`, `Mutex`, and `RwLock` where shared state is involved, particularly for services and collectors that may be accessed or run asynchronously.

<!-- ANCHOR_END -->
