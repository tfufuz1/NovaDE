//! # UPower D-Bus Client
//!
//! This module provides a client for interacting with the `org.freedesktop.UPower`
//! D-Bus service. UPower is a system service that provides information about power
//! sources, batteries, and power management.
//!
//! The client allows listing power devices, querying their properties (e.g., percentage,
//! state, type), and potentially listening for signals related to power events.
//!
//! ## Features:
//! - List all power devices known to UPower.
//! - Get detailed structured information for each device.
//! - Get raw D-Bus properties for a device.
//!
//! ## Usage Example (Conceptual)
//! ```no_run
//! # use novade_system::dbus_clients::upower_client::UPowerClient;
//! # use novade_system::dbus_integration::DbusServiceManager;
//! # use std::sync::Arc;
//! # async fn run() -> anyhow::Result<()> {
//! // DbusServiceManager would typically be created once and shared.
//! let dbus_mgr = Arc::new(DbusServiceManager::new_session().await?);
//! let upower_client = UPowerClient::new(dbus_mgr);
//!
//! if let Ok(devices) = upower_client.list_devices().await {
//!     for device_path in devices {
//!         if let Ok(details) = upower_client.get_device_details(&device_path).await {
//!             println!("Device: {}, Type: {:?}, Percentage: {:?}",
//!                 details.path.as_str(), details.type_, details.percentage);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use zbus::{dbus_proxy, zvariant::{ObjectPath, OwnedObjectPath, Value, Dict, Array}};
use tracing;

use crate::dbus_integration::{DbusServiceManager, DbusManagerError};

// ANCHOR: UPowerConstants
/// D-Bus service name for UPower.
pub const ORG_FREEDESKTOP_UPOWER: &str = "org.freedesktop.UPower";
/// Default D-Bus object path for the main UPower interface.
pub const UPOWER_PATH_STR: &str = "/org/freedesktop/UPower";
/// D-Bus interface name for UPower devices.
pub const DEVICE_INTERFACE_NAME: &str = "org.freedesktop.UPower.Device";

// ANCHOR: UPowerProxyDefinition
/// D-Bus proxy for the main `org.freedesktop.UPower` interface.
///
/// This interface provides methods to enumerate devices and access global UPower properties.
#[dbus_proxy(
    interface = "org.freedesktop.UPower",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
trait UPowerProxy {
    /// Enumerates all power devices known to UPower.
    ///
    /// Returns a list of object paths, each representing a distinct power device
    /// (e.g., battery, AC adapter).
    async fn enumerate_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    // TODO: Consider adding other UPower methods if necessary:
    // async fn get_display_device(&self) -> zbus::Result<OwnedObjectPath>;
    // async fn get_critical_action(&self) -> zbus::Result<String>;

    // Signals available on org.freedesktop.UPower that could be handled:
    // #[dbus_proxy(signal)]
    // async fn device_added(&self, device: OwnedObjectPath) -> zbus::Result<()>;
    // #[dbus_proxy(signal)]
    // async fn device_removed(&self, device: OwnedObjectPath) -> zbus::Result<()>;
    // #[dbus_proxy(signal)]
    // async fn device_changed(&self, device: OwnedObjectPath) -> zbus::Result<()>; // UPower >= 0.99
}

// ANCHOR: UPowerDeviceProxyDefinition
/// D-Bus proxy for the `org.freedesktop.UPower.Device` interface.
///
/// This interface is implemented by individual power device objects (e.g., a specific battery).
/// It provides detailed properties about the device's state, capacity, type, etc.
/// The `default_path` is not set here as it varies for each device.
#[dbus_proxy(
    interface = "org.freedesktop.UPower.Device",
    default_service = "org.freedesktop.UPower"
)]
trait UPowerDeviceProxy {
    /// Indicates if the device is rechargeable (e.g., a battery).
    #[dbus_proxy(property)]
    async fn is_rechargeable(&self) -> zbus::Result<bool>;

    /// For AC adapter type devices, indicates if line power is currently connected.
    #[dbus_proxy(property)]
    async fn line_power_online(&self) -> zbus::Result<bool>;

    /// The charge percentage of the device (e.g., battery level).
    #[dbus_proxy(property)]
    async fn percentage(&self) -> zbus::Result<f64>;

    /// The current state of the device.
    ///
    /// Values are defined by UPower:
    /// 0: Unknown
    /// 1: Charging
    /// 2: Discharging
    /// 3: Empty
    /// 4: Fully Charged
    /// 5: Pending Charge
    /// 6: Pending Discharge
    #[dbus_proxy(property)]
    async fn state(&self) -> zbus::Result<u32>;

    /// The technology of the device (e.g., Lithium Ion).
    /// Values are UPower-defined enums.
    #[dbus_proxy(property)]
    async fn technology(&self) -> zbus::Result<u32>;

    /// The temperature of the device, if available.
    #[dbus_proxy(property)]
    async fn temperature(&self) -> zbus::Result<f64>;

    /// The type of the device.
    ///
    /// Values are defined by UPower:
    /// 0: Unknown
    /// 1: Line Power (e.g., AC adapter)
    /// 2: Battery
    /// 3: UPS (Uninterruptible Power Supply)
    /// 4: Monitor (e.g., some displays report power info)
    /// 5: Mouse
    /// 6: Keyboard
    /// 7: PDA
    /// 8: Phone
    /// 9: Gaming Input
    /// 10: Bluetooth Generic
    /// 11: Tablet
    #[dbus_proxy(property(name = "Type"))]
    async fn type_(&self) -> zbus::Result<u32>;

    /// The vendor name of the device.
    #[dbus_proxy(property)]
    async fn vendor(&self) -> zbus::Result<String>;

    /// The model name of the device.
    #[dbus_proxy(property)]
    async fn model(&self) -> zbus::Result<String>;

    /// An icon name representing the device or its state.
    #[dbus_proxy(property)]
    async fn icon_name(&self) -> zbus::Result<String>;

    /// The native system path of the device (e.g., `/sys/class/power_supply/BAT0`).
    #[dbus_proxy(property)]
    async fn native_path(&self) -> zbus::Result<String>;

    /// Current capacity relative to design capacity, as a percentage.
    #[dbus_proxy(property)]
    async fn capacity(&self) -> zbus::Result<f64>;

    /// Current energy level in Watt-hours (Wh).
    #[dbus_proxy(property)]
    async fn energy(&self) -> zbus::Result<f64>;

    /// Energy level when empty in Watt-hours (Wh).
    #[dbus_proxy(property)]
    async fn energy_empty(&self) -> zbus::Result<f64>;

    /// Energy level when full in Watt-hours (Wh).
    #[dbus_proxy(property)]
    async fn energy_full(&self) -> zbus::Result<f64>;

    /// Design energy level when full in Watt-hours (Wh).
    #[dbus_proxy(property)]
    async fn energy_full_design(&self) -> zbus::Result<f64>;

    /// Current energy rate (power) in Watts (W). Positive for charging, negative for discharging.
    #[dbus_proxy(property)]
    async fn energy_rate(&self) -> zbus::Result<f64>;

    /// Estimated time until the device is empty, in seconds.
    #[dbus_proxy(property)]
    async fn time_to_empty(&self) -> zbus::Result<i64>;

    /// Estimated time until the device is fully charged, in seconds.
    #[dbus_proxy(property)]
    async fn time_to_full(&self) -> zbus::Result<i64>;

    /// Requests the device to refresh its data.
    async fn refresh(&self) -> zbus::Result<()>;
}

// ANCHOR: UPowerDeviceDataStruct
/// A structured representation of commonly used data from a UPower device.
///
/// Fields are `Option` types to gracefully handle cases where UPower might not
/// provide a specific property for a device.
#[derive(Debug, Clone)]
pub struct UPowerDeviceData {
    /// The D-Bus object path of the device.
    pub path: OwnedObjectPath,
    /// Whether the device is rechargeable.
    pub is_rechargeable: Option<bool>,
    /// For AC adapters, whether line power is connected.
    pub line_power_online: Option<bool>,
    /// Charge percentage (0.0 to 100.0).
    pub percentage: Option<f64>,
    /// Current charging/discharging state (UPower enum).
    pub state: Option<u32>,
    /// Battery technology (UPower enum).
    pub technology: Option<u32>,
    /// Device temperature in Celsius.
    pub temperature: Option<f64>,
    /// Type of device (e.g., Battery, Line Power) (UPower enum).
    pub type_: Option<u32>,
    /// Device vendor string.
    pub vendor: Option<String>,
    /// Device model string.
    pub model: Option<String>,
    /// Suggested icon name for the device.
    pub icon_name: Option<String>,
    /// Native system path (e.g., in `/sysfs`).
    pub native_path: Option<String>,
    /// Current capacity as a percentage of design capacity.
    pub capacity: Option<f64>,
    /// Current energy level (Wh).
    pub energy: Option<f64>,
    /// Energy level when full (Wh).
    pub energy_full: Option<f64>,
    /// Design energy level when full (Wh).
    pub energy_full_design: Option<f64>,
    /// Current power rate (W); positive for charge, negative for discharge.
    pub energy_rate: Option<f64>,
    /// Estimated time to empty (seconds).
    pub time_to_empty: Option<i64>,
    /// Estimated time to full (seconds).
    pub time_to_full: Option<i64>,
}


// ANCHOR: ClientErrorDefinition
/// Errors that can occur when interacting with the UPower service via [`UPowerClient`].
#[derive(Debug, Error)]
pub enum ClientError {
    /// An error occurred within the [`DbusServiceManager`].
    #[error("D-Bus manager error: {0}")]
    DbusManagerFailed(#[from] DbusManagerError),
    /// A general D-Bus operation failed (e.g., method call, property access).
    #[error("D-Bus operation failed: {0}")]
    ZbusFailed(#[from] zbus::Error),
    /// Failed to convert a string into a valid D-Bus `ObjectPath`.
    #[error("Failed to convert path string '{path}' to ObjectPath: {source}")]
    InvalidObjectPath {
        /// The path string that caused the error.
        path: String,
        /// The underlying `zbus::zvariant::Error`.
        source: zbus::zvariant::Error,
    },
    /// A specific D-Bus property was not found on an interface.
    #[error("Property not found: {name} on interface {interface} at path {path}")]
    PropertyNotFound {
        /// Name of the property.
        name: String,
        /// Interface where the property was expected.
        interface: String,
        /// Object path of the D-Bus object.
        path: String,
    },
    /// A UPower device was not found at the specified object path.
    #[error("Device not found at path: {path}")]
    DeviceNotFound {
        /// The object path where the device was expected.
        path: String,
    },
}

/// A specialized `Result` type for [`UPowerClient`] operations.
pub type Result<T> = std::result::Result<T, ClientError>;

// ANCHOR: UPowerClientStructDefinition
/// A D-Bus client for interacting with the `org.freedesktop.UPower` service.
///
/// This client uses a [`DbusServiceManager`] to create D-Bus proxies and communicate
/// with UPower. It provides methods to list power devices and retrieve detailed
/// information about them.
#[derive(Debug, Clone)]
pub struct UPowerClient {
    /// The D-Bus service manager used for creating proxies.
    dbus_manager: Arc<DbusServiceManager>,
}

impl UPowerClient {
    /// Creates a new `UPowerClient`.
    ///
    /// # Arguments
    ///
    /// * `dbus_manager`: An `Arc<DbusServiceManager>` that the client will use
    ///   to interact with the D-Bus, primarily for creating proxies.
    // ANCHOR: UPowerClientNewMethod
    pub fn new(dbus_manager: Arc<DbusServiceManager>) -> Self {
        tracing::info!("Creating new UPowerClient");
        Self { dbus_manager }
    }

    // Client methods will be implemented here

    // ANCHOR: ListDevicesMethod
    /// Lists all power devices recognized by the UPower service.
    ///
    /// This method calls `EnumerateDevices` on the main `org.freedesktop.UPower` interface.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<OwnedObjectPath>` where each path represents a
    /// UPower device, or a `ClientError` if the operation fails.
    ///
    /// # Errors
    ///
    /// Can return `ClientError::InvalidObjectPath` if the UPower service path is malformed (unlikely),
    /// `ClientError::DbusManagerFailed` if proxy creation fails, or
    /// `ClientError::ZbusFailed` for D-Bus errors during the method call.
    #[tracing::instrument(skip(self))]
    pub async fn list_devices(&self) -> Result<Vec<OwnedObjectPath>> {
        tracing::info!("Listing UPower devices");
        let upower_service_path = ObjectPath::try_from(UPOWER_PATH_STR)
            .map_err(|e| ClientError::InvalidObjectPath { path: UPOWER_PATH_STR.to_string(), source: e})?;

        let proxy = self.dbus_manager.create_proxy::<UPowerProxy<'_>>(
            ORG_FREEDESKTOP_UPOWER.try_into().unwrap(), // WellKnownName
            upower_service_path,
        ).await?;

        proxy.enumerate_devices().await.map_err(ClientError::ZbusFailed)
    }

    // ANCHOR: GetDeviceDetailsMethod
    /// Retrieves structured, detailed information for a specific UPower device.
    ///
    /// This method creates a proxy for the `org.freedesktop.UPower.Device` interface
    /// at the given `device_path` and fetches various properties to populate
    /// the [`UPowerDeviceData`] struct. Properties that are not available or
    /// fail to retrieve are set to `None` in the returned struct.
    ///
    /// # Arguments
    ///
    /// * `device_path`: The D-Bus `ObjectPath` of the UPower device to query.
    ///
    /// # Returns
    ///
    /// A `Result` containing `UPowerDeviceData` with details of the device,
    /// or a `ClientError` if proxy creation fails or the path is invalid.
    /// Individual property fetch errors are logged but do not cause this method to fail overall.
    ///
    /// # Errors
    ///
    /// Can return `ClientError::InvalidObjectPath` if the provided `device_path` is malformed,
    /// or `ClientError::DbusManagerFailed` if proxy creation fails.
    #[tracing::instrument(skip(self), fields(device_path = %device_path))]
    pub async fn get_device_details(&self, device_path: &ObjectPath<'_>) -> Result<UPowerDeviceData> {
        tracing::info!("Getting details for UPower device: {}", device_path);
        let owned_device_path = OwnedObjectPath::try_from(device_path.clone())
            .map_err(|e| ClientError::InvalidObjectPath { path: device_path.to_string(), source: e})?;

        let proxy = self.dbus_manager.create_proxy::<UPowerDeviceProxy<'_>>(
            ORG_FREEDESKTOP_UPOWER.try_into().unwrap(),
            owned_device_path.clone(), // path for the specific device
        ).await?;

        // Helper to wrap property fetching and optionalize it
        async fn get_prop<T, F, Fut>(f: F) -> Option<T>
        where
            F: FnOnce() -> Fut,
            Fut: std::future::Future<Output = zbus::Result<T>>,
        {
            match f().await {
                Ok(val) => Some(val),
                Err(e) => {
                    tracing::warn!("Failed to get property: {}", e); // Log specific property error
                    None
                }
            }
        }

        let data = UPowerDeviceData {
            path: owned_device_path,
            is_rechargeable: get_prop(|| proxy.is_rechargeable()).await,
            line_power_online: get_prop(|| proxy.line_power_online()).await,
            percentage: get_prop(|| proxy.percentage()).await,
            state: get_prop(|| proxy.state()).await,
            technology: get_prop(|| proxy.technology()).await,
            temperature: get_prop(|| proxy.temperature()).await,
            type_: get_prop(|| proxy.type_()).await,
            vendor: get_prop(|| proxy.vendor()).await,
            model: get_prop(|| proxy.model()).await,
            icon_name: get_prop(|| proxy.icon_name()).await,
            native_path: get_prop(|| proxy.native_path()).await,
            capacity: get_prop(|| proxy.capacity()).await,
            energy: get_prop(|| proxy.energy()).await,
            energy_full: get_prop(|| proxy.energy_full()).await,
            energy_full_design: get_prop(|| proxy.energy_full_design()).await,
            energy_rate: get_prop(|| proxy.energy_rate()).await,
            time_to_empty: get_prop(|| proxy.time_to_empty()).await,
            time_to_full: get_prop(|| proxy.time_to_full()).await,
        };
        Ok(data)
    }

    // ANCHOR: GetDevicePropertiesRawMethod (using Properties interface)
    /// Retrieves all D-Bus properties for a specific UPower device's interface as a raw map.
    ///
    /// This method uses the standard `org.freedesktop.DBus.Properties` interface's `GetAll` method
    /// to fetch all properties associated with the `org.freedesktop.UPower.Device` interface
    /// on the specified `device_path`.
    ///
    /// # Arguments
    ///
    /// * `device_path`: The D-Bus `ObjectPath` of the UPower device.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashMap<String, Value<'static>>` where keys are property names
    /// and values are their corresponding D-Bus values, or a `ClientError` if the operation fails.
    ///
    /// # Errors
    ///
    /// Can return `ClientError::DbusManagerFailed` if proxy creation fails, or
    /// `ClientError::ZbusFailed` for D-Bus errors during the `GetAll` call.
    #[tracing::instrument(skip(self), fields(device_path = %device_path))]
    pub async fn get_device_properties_raw(&self, device_path: &ObjectPath<'_>) -> Result<HashMap<String, Value<'static>>> {
        tracing::info!("Getting all raw properties for UPower device: {}", device_path);

        let properties_proxy = self.dbus_manager.create_proxy::<zbus::fdo::PropertiesProxy<'_>>(
            ORG_FREEDESKTOP_UPOWER.try_into().unwrap(),
            device_path.to_owned(),
        ).await?;

        properties_proxy.get_all(DEVICE_INTERFACE_NAME).await.map_err(ClientError::ZbusFailed)
    }

    // ANCHOR: GetSpecificDevicePropertyRawMethod (using Properties interface)
    /// Retrieves a single D-Bus property for a specific UPower device's interface as a raw value.
    ///
    /// This method uses the standard `org.freedesktop.DBus.Properties` interface's `Get` method.
    ///
    /// # Arguments
    ///
    /// * `device_path`: The D-Bus `ObjectPath` of the UPower device.
    /// * `property_name`: The name of the property to retrieve from the `org.freedesktop.UPower.Device` interface.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Value<'static>` of the property, or a `ClientError` if the operation fails.
    ///
    /// # Errors
    ///
    /// Can return `ClientError::DbusManagerFailed` if proxy creation fails, or
    /// `ClientError::ZbusFailed` for D-Bus errors during the `Get` call (e.g., property not found).
    #[tracing::instrument(skip(self), fields(device_path = %device_path, property_name = %property_name))]
    pub async fn get_specific_device_property_raw(&self, device_path: &ObjectPath<'_>, property_name: &str) -> Result<Value<'static>> {
        tracing::info!("Getting raw property '{}' for UPower device: {}", property_name, device_path);

        let properties_proxy = self.dbus_manager.create_proxy::<zbus::fdo::PropertiesProxy<'_>>(
            ORG_FREEDESKTOP_UPOWER.try_into().unwrap(),
            device_path.to_owned(),
        ).await?;

        properties_proxy.get(DEVICE_INTERFACE_NAME, property_name).await.map_err(ClientError::ZbusFailed)
    }
}
