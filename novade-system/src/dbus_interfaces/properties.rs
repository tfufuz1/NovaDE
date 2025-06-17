//! # D-Bus `org.freedesktop.DBus.Properties` Interface Implementation
//!
//! This module provides a Rust implementation of the standard D-Bus `Properties`
//! interface. It allows D-Bus services to expose properties of their objects,
//! enabling clients to get or set these properties and to be notified of changes.
//!
//! The main struct [`Properties`] is designed to be served alongside other
//! D-Bus interfaces on a specific object path. It requires a shared data store
//! (`properties_storage`) for the actual property values.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::{Connection, dbus_interface, interface, SignalContext};
use zbus::zvariant::{ObjectPath, Value, OwnedValue};
use zbus::fdo::Result as FdoResult;
use zbus::fdo::Error as FdoError;

// ANCHOR: PropertiesStructDefinition
/// Implements the `org.freedesktop.DBus.Properties` D-Bus interface.
///
/// This struct is responsible for managing the properties of a *single* D-Bus interface
/// on a given object path. It allows clients to get and set property values and
/// emits the `PropertiesChanged` signal when properties are modified.
///
/// An instance of `Properties` should be created for each interface on an object
/// that needs to expose properties. All instances managing properties for the same
/// object path would typically be served by the same D-Bus object server.
///
/// # Example
/// ```no_run
/// # use novade_system::dbus_interfaces::Properties;
/// # use zbus::zvariant::{ObjectPath, OwnedValue};
/// # use zbus::Connection;
/// # use std::sync::Arc;
/// # use std::collections::HashMap;
/// # use tokio::sync::Mutex;
/// # async fn run() -> anyhow::Result<()> {
/// let conn = Arc::new(Connection::session().await?);
/// let obj_path = ObjectPath::try_from("/org/novade/MyObject")?;
/// let interface_name = "org.novade.MyInterface".to_string();
/// let property_store = Arc::new(Mutex::new(HashMap::<String, OwnedValue>::new()));
///
/// let props_interface = Properties::new(
///     conn.clone(),
///     obj_path.into_owned(), // Properties expects ObjectPath<'static>
///     interface_name,
///     property_store.clone()
/// );
/// // This props_interface can then be served by a DbusServiceManager or zbus ObjectServer.
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Properties {
    /// D-Bus connection used for emitting signals like `PropertiesChanged`.
    connection: Arc<Connection>,
    /// The D-Bus object path this `Properties` instance is associated with.
    object_path: ObjectPath<'static>,
    /// The specific D-Bus interface name (e.g., "org.example.MyInterface")
    /// for which this instance manages properties.
    interface_name: String,
    /// The actual storage for the properties. This `Arc<Mutex<...>>` is typically
    /// shared with the main D-Bus interface implementation that owns these properties,
    /// allowing it to modify them directly while this `Properties` struct provides
    /// D-Bus access.
    properties_storage: Arc<Mutex<HashMap<String, OwnedValue>>>,
}

impl Properties {
    /// Creates a new `Properties` interface handler for a specific D-Bus interface on an object.
    ///
    /// # Arguments
    ///
    /// * `connection`: An `Arc<zbus::Connection>` used for emitting signals.
    /// * `object_path`: The D-Bus object path that this `Properties` instance will be associated with.
    ///   The path must be `'static` as it's stored within the struct.
    /// * `interface_name`: The name of the D-Bus interface (e.g., "com.example.MyInterface")
    ///   whose properties are being managed.
    /// * `properties_storage`: A shared `Arc<Mutex<HashMap<String, OwnedValue>>>` which serves as
    ///   the backing store for the properties. This map holds property names (as `String`)
    ///   and their values (as `OwnedValue`).
    // ANCHOR: PropertiesNewMethod
    pub fn new(
        connection: Arc<Connection>,
        object_path: ObjectPath<'static>,
        interface_name: String,
        properties_storage: Arc<Mutex<HashMap<String, OwnedValue>>>,
    ) -> Self {
        tracing::info!(
            "Creating new Properties handler for object '{}', interface '{}'",
            object_path,
            interface_name
        );
        Self {
            connection,
            object_path,
            interface_name,
            properties_storage,
        }
    }

    // Helper method to emit PropertiesChanged signal will be added here
    // ANCHOR: EmitPropertiesChangedHelper
    async fn emit_properties_changed(
        &self,
        changed_properties: HashMap<String, Value<'_>>,
        invalidated_properties: Vec<String>,
    ) -> zbus::Result<()> {
        if changed_properties.is_empty() && invalidated_properties.is_empty() {
            return Ok(());
        }
        tracing::debug!(
            "Internal: Emitting PropertiesChanged signal for interface '{}' on object '{}'. Changed: {:?}, Invalidated: {:?}",
            self.interface_name,
            self.object_path,
            changed_properties.keys(),
            invalidated_properties
        );
        Self::properties_changed(
            &self.connection, // Use the connection directly for emitting top-level signals
            &self.interface_name,
            changed_properties,
            invalidated_properties,
        )
        .await
    }

    // ANCHOR: PublicEmitPropertiesChangedMethod
    /// Allows an associated service to trigger a `PropertiesChanged` D-Bus signal.
    ///
    /// This method is intended to be called by the D-Bus interface implementation that
    /// "owns" the properties (and shares the `properties_storage` with this `Properties` instance).
    /// It should be used when the service changes a property value internally (i.e., not in response
    /// to a D-Bus `Set` call on this `Properties` interface).
    ///
    /// # Arguments
    ///
    /// * `changed_properties`: A map where keys are property names (as `String`) that have changed,
    ///   and values are their new `zbus::zvariant::Value`. These should be the current values.
    /// * `invalidated_properties`: A list of property names (as `String`) that have been removed
    ///   or whose values are no longer valid and should be considered unset by clients.
    ///
    /// # Errors
    ///
    /// Returns a `zbus::Result<()>` which will be an `Err` if emitting the D-Bus signal fails.
    #[tracing::instrument(skip(self, changed_properties, invalidated_properties), fields(interface_name = %self.interface_name, object_path = %self.object_path))]
    pub async fn public_emit_properties_changed(
        &self,
        changed_properties: HashMap<String, Value<'_>>,
        invalidated_properties: Vec<String>,
    ) -> zbus::Result<()> {
        tracing::info!(
            "Public request to emit PropertiesChanged for interface '{}' on object '{}'",
            self.interface_name, self.object_path
        );
        self.emit_properties_changed(changed_properties, invalidated_properties).await
    }
}

/// D-Bus methods for `org.freedesktop.DBus.Properties`.
#[dbus_interface(name = "org.freedesktop.DBus.Properties")]
impl Properties {
    // ANCHOR: GetMethod
    /// Retrieves the value of a single property.
    ///
    /// Implements the `Get` method of the `org.freedesktop.DBus.Properties` interface.
    ///
    /// # Arguments
    ///
    /// * `interface_name`: The name of the interface to which the property belongs.
    ///   This must match the `interface_name` this `Properties` instance was constructed with.
    /// * `property_name`: The name of the property to retrieve.
    ///
    /// # Errors
    ///
    /// * `org.freedesktop.DBus.Error.InvalidArgs`: If `interface_name` does not match,
    ///   or if `property_name` does not exist on the interface.
    /// * Other D-Bus errors if the underlying property storage access fails unexpectedly.
    #[dbus_interface(name = "Get")]
    async fn get(&self, interface_name: String, property_name: String) -> FdoResult<OwnedValue> {
        tracing::debug!(
            "Get property '{}' for interface '{}' on object '{}'",
            property_name,
            interface_name,
            self.object_path
        );
        if interface_name != self.interface_name {
            tracing::warn!(
                "Get: Interface name mismatch. Expected '{}', got '{}'",
                self.interface_name,
                interface_name
            );
            return Err(FdoError::InvalidArgs(format!(
                "Interface name mismatch. Expected '{}', got '{}'",
                self.interface_name, interface_name
            )));
        }

        let properties = self.properties_storage.lock().await;
        match properties.get(&property_name) {
            Some(value) => Ok(value.clone()),
            None => {
                tracing::warn!("Get: Property '{}' not found on interface '{}'", property_name, self.interface_name);
                Err(FdoError::InvalidArgs(format!(
                    "Property '{}' not found on interface '{}'",
                    property_name, self.interface_name
                )))
            }
        }
    }

    // ANCHOR: SetMethod
    /// Sets the value of a single property.
    ///
    /// Implements the `Set` method of the `org.freedesktop.DBus.Properties` interface.
    /// After successfully setting the property value in the shared `properties_storage`,
    /// this method will emit a `PropertiesChanged` signal indicating the change.
    ///
    /// **Note**: This basic implementation assumes all properties managed are read-write if they exist,
    /// or can be created if they don't. A more sophisticated implementation might involve
    /// metadata to check for read-only properties and prevent setting them, returning an
    /// appropriate D-Bus error (e.g., `org.freedesktop.DBus.Error.PropertyReadOnly`).
    ///
    /// # Arguments
    ///
    /// * `interface_name`: The name of the interface to which the property belongs. Must match.
    /// * `property_name`: The name of the property to set. If it doesn't exist, it may be created.
    /// * `value`: The new `zbus::zvariant::Value` for the property.
    ///
    /// # Errors
    ///
    /// * `org.freedesktop.DBus.Error.InvalidArgs`: If `interface_name` does not match.
    /// * May return other D-Bus errors if cloning the value fails or if signal emission fails.
    #[dbus_interface(name = "Set")]
    async fn set(&mut self, interface_name: String, property_name: String, value: Value<'_>) -> FdoResult<()> {
        tracing::debug!(
            "Set property '{}' for interface '{}' on object '{}' to value: {:?}",
            property_name,
            interface_name,
            self.object_path,
            value
        );
        if interface_name != self.interface_name {
            tracing::warn!(
                "Set: Interface name mismatch. Expected '{}', got '{}'",
                self.interface_name,
                interface_name
            );
            return Err(FdoError::InvalidArgs(format!(
                "Interface name mismatch. Expected '{}', got '{}'",
                self.interface_name, interface_name
            )));
        }

        // For simplicity, we allow setting existing or new properties.
        // A more robust implementation would check if the property exists and if it's writable.
        // If property_name does not exist, it will be created.
        let owned_value = OwnedValue::from(value.try_clone()?);
        let mut properties = self.properties_storage.lock().await;
        properties.insert(property_name.clone(), owned_value.clone());
        drop(properties); // Release lock before emitting signal

        let mut changed_map = HashMap::new();
        // The signal expects Value<'_>, not OwnedValue
        changed_map.insert(property_name, Value::from(owned_value));

        // Emit PropertiesChanged signal
        // Collecting to owned values first for the signal map
        let signal_changed_props: HashMap<String, Value<'_>> = changed_map;

        if let Err(e) = self.emit_properties_changed(signal_changed_props, Vec::new()).await {
            tracing::error!("Failed to emit PropertiesChanged signal: {}", e);
            // This error typically shouldn't prevent the Set operation itself from succeeding locally,
            // but it's a problem for observers. Depending on strictness, one might return an error here.
            // For now, log and continue.
        }
        Ok(())
    }

    // ANCHOR: GetAllMethod
    /// Retrieves all properties of the interface.
    ///
    /// Implements the `GetAll` method of the `org.freedesktop.DBus.Properties` interface.
    ///
    /// # Arguments
    ///
    /// * `interface_name`: The name of the interface for which to retrieve all properties.
    ///   Must match the `interface_name` this `Properties` instance was constructed with.
    ///
    /// # Errors
    ///
    /// * `org.freedesktop.DBus.Error.InvalidArgs`: If `interface_name` does not match.
    #[dbus_interface(name = "GetAll")]
    async fn get_all(&self, interface_name: String) -> FdoResult<HashMap<String, OwnedValue>> {
        tracing::debug!(
            "GetAll properties for interface '{}' on object '{}'",
            interface_name,
            self.object_path
        );
        if interface_name != self.interface_name {
            tracing::warn!(
                "GetAll: Interface name mismatch. Expected '{}', got '{}'",
                self.interface_name,
                interface_name
            );
            return Err(FdoError::InvalidArgs(format!(
                "Interface name mismatch. Expected '{}', got '{}'",
                self.interface_name, interface_name
            )));
        }

        let properties = self.properties_storage.lock().await;
        Ok(properties.clone())
    }

    // ANCHOR: PropertiesChangedSignal
    /// D-Bus signal emitted when one or more properties of an interface have changed.
    ///
    /// This signal is part of the `org.freedesktop.DBus.Properties` interface specification.
    /// It is emitted by the `Set` method of this struct, or when `public_emit_properties_changed`
    /// is called.
    ///
    /// # Arguments
    ///
    /// * `interface_name`: The name of the D-Bus interface whose properties have changed.
    ///   This will be the `interface_name` this `Properties` instance was constructed with.
    /// * `changed_properties`: A dictionary mapping property names (as `String`) to their new
    ///   values (as `zbus::zvariant::Value`). This includes both newly set properties and
    ///   properties whose values were modified.
    /// * `invalidated_properties`: A list of property names (as `String`) that were removed
    ///   or whose values are no longer available. This implementation primarily uses `changed_properties`
    ///   and typically passes an empty `Vec` for `invalidated_properties` unless specifically designed
    ///   to invalidate/remove properties.
    #[dbus_interface(signal)]
    async fn properties_changed(
        signal_ctxt: &SignalContext<'_>,
        interface_name: &str,
        changed_properties: HashMap<String, Value<'_>>,
        invalidated_properties: Vec<String>,
    ) -> zbus::Result<()>;
}

// ANCHOR: PropertiesUnitTests
#[cfg(test)]
mod tests {
    use super::*;
    // Value is already in super, but OwnedValue might be needed directly.
    use zbus::zvariant::{OwnedValue, Dict};
    use zbus::Connection; // Explicit import for Arc<Connection>
    use std::collections::HashMap;
    use tokio; // Ensure tokio is available for async tests
    use std::sync::Arc;

    const TEST_INTERFACE_NAME: &str = "org.novade.TestInterface1";
    const OTHER_INTERFACE_NAME: &str = "org.novade.OtherInterface";

    // Helper for tests to create a new Properties instance, its storage, and a Connection.
    async fn new_test_properties_handler_with_storage() -> (
        Properties,
        Arc<Mutex<HashMap<String, OwnedValue>>>,
        Arc<Connection>, // Returned for completeness, though not directly used in most asserts
    ) {
        let conn = Connection::session().await.expect(
            "Failed to connect to session bus for test. Ensure dbus-daemon is running.",
        );
        let object_path = ObjectPath::try_from("/test/properties_object").unwrap();

        let mut initial_props = HashMap::new();
        initial_props.insert("StringProp".to_string(), OwnedValue::from("InitialString"));
        initial_props.insert("IntProp".to_string(), OwnedValue::from(123i32));
        initial_props.insert("BoolProp".to_string(), OwnedValue::from(true));
        let properties_storage = Arc::new(Mutex::new(initial_props));

        let props_handler = Properties::new(
            conn.clone(),
            object_path,
            TEST_INTERFACE_NAME.to_string(),
            properties_storage.clone(),
        );
        (props_handler, properties_storage, conn)
    }

    #[tokio::test]
    async fn test_get_property_existing() {
        let (props_handler, _storage, _conn) = new_test_properties_handler_with_storage().await;
        let result = props_handler.get(TEST_INTERFACE_NAME.to_string(), "StringProp".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), OwnedValue::from("InitialString"));

        let result_int = props_handler.get(TEST_INTERFACE_NAME.to_string(), "IntProp".to_string()).await;
        assert!(result_int.is_ok());
        assert_eq!(result_int.unwrap(), OwnedValue::from(123i32));
    }

    #[tokio::test]
    async fn test_get_property_non_existing() {
        let (props_handler, _storage, _conn) = new_test_properties_handler_with_storage().await;
        let result = props_handler.get(TEST_INTERFACE_NAME.to_string(), "NonExistentProp".to_string()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            FdoError::InvalidArgs(msg) => assert!(msg.contains("Property 'NonExistentProp' not found")),
            e => panic!("Expected InvalidArgs, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_property_wrong_interface() {
        let (props_handler, _storage, _conn) = new_test_properties_handler_with_storage().await;
        let result = props_handler.get(OTHER_INTERFACE_NAME.to_string(), "StringProp".to_string()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            FdoError::InvalidArgs(msg) => assert!(msg.contains("Interface name mismatch")),
            e => panic!("Expected InvalidArgs for wrong interface, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_set_property_existing() {
        let (mut props_handler, storage, _conn) = new_test_properties_handler_with_storage().await;
        let result = props_handler.set(TEST_INTERFACE_NAME.to_string(), "IntProp".to_string(), Value::from(456i32)).await;
        assert!(result.is_ok());

        let props_map = storage.lock().await;
        assert_eq!(props_map.get("IntProp").unwrap(), &OwnedValue::from(456i32));
    }

    #[tokio::test]
    async fn test_set_property_new() {
        let (mut props_handler, storage, _conn) = new_test_properties_handler_with_storage().await;
        let result = props_handler.set(TEST_INTERFACE_NAME.to_string(), "NewFloatProp".to_string(), Value::from(78.9f64)).await;
        assert!(result.is_ok());

        let props_map = storage.lock().await;
        assert_eq!(props_map.get("NewFloatProp").unwrap(), &OwnedValue::from(78.9f64));
        assert_eq!(props_map.len(), 4); // 3 initial + 1 new
    }

    #[tokio::test]
    async fn test_set_property_wrong_interface() {
        let (mut props_handler, storage, _conn) = new_test_properties_handler_with_storage().await;
        let original_value = storage.lock().await.get("StringProp").unwrap().try_clone().unwrap();

        let result = props_handler.set(OTHER_INTERFACE_NAME.to_string(), "StringProp".to_string(), Value::from("AttemptedChange")).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            FdoError::InvalidArgs(msg) => assert!(msg.contains("Interface name mismatch")),
            e => panic!("Expected InvalidArgs for wrong interface on set, got {:?}", e),
        }

        let props_map_after = storage.lock().await;
        assert_eq!(props_map_after.get("StringProp").unwrap(), &original_value); // Ensure no change
    }

    #[tokio::test]
    async fn test_get_all_properties_correct_interface() {
        let (props_handler, storage, _conn) = new_test_properties_handler_with_storage().await;

        // Optionally add more diverse props for GetAll
        {
            let mut props_map = storage.lock().await;
            props_map.insert("ExtraString".to_string(), OwnedValue::from("ForGetAll"));
        }

        let result = props_handler.get_all(TEST_INTERFACE_NAME.to_string()).await;
        assert!(result.is_ok());
        let all_props_returned = result.unwrap();

        let expected_props_map = storage.lock().await.clone();
        assert_eq!(all_props_returned.len(), expected_props_map.len());
        for (k, v) in expected_props_map {
            assert_eq!(all_props_returned.get(&k).unwrap(), &v);
        }
    }

    #[tokio::test]
    async fn test_get_all_properties_wrong_interface() {
        let (props_handler, _storage, _conn) = new_test_properties_handler_with_storage().await;
        let result = props_handler.get_all(OTHER_INTERFACE_NAME.to_string()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            FdoError::InvalidArgs(msg) => assert!(msg.contains("Interface name mismatch")),
            e => panic!("Expected InvalidArgs for wrong interface on GetAll, got {:?}", e),
        }
    }
}
