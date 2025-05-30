pub mod dbus_integration;
pub mod dbus_interfaces; // Added new module
pub mod window_info_provider;
pub use window_info_provider::{FocusedWindowDetails, SystemWindowInfoProvider, StubSystemWindowInfoProvider, WaylandWindowInfoProvider};

pub mod dbus_menu_provider; // Assuming this was added from a previous task
pub use dbus_menu_provider::{DBusMenuError, DBusMenuProvider, StubDBusMenuProvider}; // Assuming
