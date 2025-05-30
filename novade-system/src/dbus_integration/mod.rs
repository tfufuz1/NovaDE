// novade-system/src/dbus_integration/mod.rs
pub mod examples;
pub mod manager;

pub use manager::{DbusServiceManager, DbusManagerError, Result as DbusManagerResult};
