pub mod application_manager; // Added for assistant integration
pub mod dbus_integration;
pub mod dbus_interfaces; // Added new module
pub mod input;
pub mod network_manager; // New module path
pub mod system_health_collectors;
pub mod power_management; // New module path
pub mod renderer; // Added this line
pub mod filesystem_service; // Added for assistant integration
pub mod system_services;
pub mod system_settings_service; // Added for assistant integration
pub mod window_info_provider;
pub mod window_mechanics;
// pub mod wayland_compositor_core; // Replaced by the new compositor module
//ANCHOR [NovaDE Developers <dev@novade.org>] Added debug_interface module.
pub mod debug_interface;

// NovaDE Wayland Compositor module
pub mod compositor;

pub use application_manager::{ApplicationManager, AppInfo}; // Added for assistant integration
pub use filesystem_service::{FileSystemService, FileInfo, UserContext}; // Added for assistant integration
pub use system_settings_service::{SystemSettingsService, SystemSettingInfo}; // Added for assistant integration
pub use window_info_provider::{FocusedWindowDetails, SystemWindowInfoProvider, StubSystemWindowInfoProvider, WaylandWindowInfoProvider};
//ANCHOR [NovaDE Developers <dev@novade.org>] Re-export DebugInterface components.
pub use debug_interface::{DebugInterface, DebugCommand, ProfilerTarget, StateSnapshot};

pub mod dbus_menu_provider; // Assuming this was added from a previous task
pub use dbus_menu_provider::{DBusMenuError, DBusMenuProvider, StubDBusMenuProvider}; // Assuming
pub use system_services::SystemServices;

/// Starts the NovaDE Wayland Compositor.
///
/// This function initializes and runs the main event loop for the compositor.
/// It should typically be called from the `main.rs` of the `novade-system` crate
/// or a higher-level application manager.
pub fn start_wayland_compositor() -> Result<(), compositor::errors::CompositorError> {
    // Setup tracing subscriber for logging
    // This could be done here or in main.rs. Doing it here ensures it's set up if this lib function is called.
    // tracing_subscriber::fmt::init(); // Basic subscriber. Consider more configurable options.

    // TODO: Load compositor configuration from NovaDE settings or a config file.
    // let config = load_compositor_config();

    compositor::core::run_compositor()
}
