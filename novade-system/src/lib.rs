pub mod dbus_integration;
pub mod dbus_interfaces; // Added new module
pub mod input;
pub mod network_manager; // New module path
pub mod system_health_collectors;
pub mod power_management; // New module path
pub mod renderer; // Added this line
pub mod system_services;
pub mod window_info_provider;
pub use window_info_provider::{FocusedWindowDetails, SystemWindowInfoProvider, StubSystemWindowInfoProvider, WaylandWindowInfoProvider};

pub mod dbus_menu_provider; // Assuming this was added from a previous task
pub use dbus_menu_provider::{DBusMenuError, DBusMenuProvider, StubDBusMenuProvider}; // Assuming
pub use system_services::SystemServices;
