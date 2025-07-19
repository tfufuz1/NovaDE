// novade-system/src/dbus_interfaces/mod.rs

// ANCHOR: AddObjectManagerModule
pub mod object_manager;
pub mod notifications_server;
// ANCHOR: AddPropertiesModule
pub mod properties;
pub mod window_management;
// ANCHOR: AddExampleEchoServiceModule
pub mod example_echo_service;
// ANCHOR: AddCoreSystemInterfaceModule
pub mod core_system_interface;
// ANCHOR: AddCoreSystemServiceModule
pub mod core_system_service; // Added new module for the service implementation

// ANCHOR: ExportObjectManager
pub use object_manager::ObjectManager;
// ANCHOR: ExportProperties
pub use properties::Properties;
// ANCHOR: ExportExampleEchoService
pub use example_echo_service::EchoService;
pub use notifications_server::NotificationsDBusService; // Assuming this was meant to be NotificationsServer or similar

// ANCHOR: ExportCoreSystemInterfaceAndTypes
pub use core_system_interface::CoreSystemInterface;
pub use core_system_interface::ComponentInfo;
pub use core_system_interface::ComponentStatus;

// ANCHOR: ExportCoreSystemService
pub use core_system_service::CoreSystemService; // Exported new service implementation
