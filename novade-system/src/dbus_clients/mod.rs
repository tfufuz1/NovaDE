// novade-system/src/dbus_clients/mod.rs

// ANCHOR: UPowerClientModuleDeclaration
pub mod upower_client;

// It might be beneficial to have a common ClientError type or re-export specific ones
// For now, each client module can define its own error type.
// pub use upower_client::ClientError as UPowerClientError;
