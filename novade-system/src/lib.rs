pub mod application_manager; // Added for assistant integration
pub mod dbus_integration;
pub mod dbus_interfaces; // Added new module
pub mod input;
pub mod network_manager; // New module path
pub mod power_management; // New module path
pub mod renderer; // Added this line
pub mod filesystem_service; // Added for assistant integration
pub mod system_services;
pub mod system_settings_service; // Added for assistant integration
pub mod window_info_provider;
pub use application_manager::{ApplicationManager, AppInfo}; // Added for assistant integration
pub use filesystem_service::{FileSystemService, FileInfo, UserContext}; // Added for assistant integration
pub use system_settings_service::{SystemSettingsService, SystemSettingInfo}; // Added for assistant integration
pub use window_info_provider::{FocusedWindowDetails, SystemWindowInfoProvider, StubSystemWindowInfoProvider, WaylandWindowInfoProvider};

pub mod dbus_menu_provider; // Assuming this was added from a previous task
pub use dbus_menu_provider::{DBusMenuError, DBusMenuProvider, StubDBusMenuProvider}; // Assuming
pub use system_services::SystemServices;
