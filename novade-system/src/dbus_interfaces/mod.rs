// novade-system/src/dbus_interfaces/mod.rs

// novade-system/src/dbus_interfaces/mod.rs

// ANCHOR: AddObjectManagerModule
pub mod object_manager;
pub mod notifications_server;
// ANCHOR: AddPropertiesModule
pub mod properties;
// ANCHOR: AddExampleEchoServiceModule
pub mod example_echo_service;

// ANCHOR: ExportObjectManager
pub use object_manager::ObjectManager;
// ANCHOR: ExportProperties
pub use properties::Properties;
// ANCHOR: ExportExampleEchoService
pub use example_echo_service::EchoService;
pub use notifications_server::NotificationsDBusService; // Assuming this was meant to be NotificationsServer or similar
// TODO: Review NotificationsDBusService, might be NotificationsServer from previous tasks.
// For now, keeping it as is to avoid breaking other parts if it's used elsewhere.
