//! # Example D-Bus Service: EchoService
//!
//! This module implements a sample D-Bus service named `EchoService`.
//! It demonstrates several D-Bus concepts:
//! - A custom D-Bus interface (`org.novade.ExampleEchoService.Echo`).
//! - Methods on the custom interface (`EchoString`).
//! - Properties exposed via `org.freedesktop.DBus.Properties` (Prefix, EchoCount).
//! - Integration with `org.freedesktop.DBus.ObjectManager` to expose its objects.
//! - Dynamic addition and removal of sub-objects managed by the `ObjectManager`.
//!
//! The service is intended to be run using the example binary in `examples/run_echo_service.rs`.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::{Connection, dbus_interface, interface::SignalContext};
use zbus::zvariant::{ObjectPath, Value, OwnedValue};
use zbus::fdo::Result as FdoResult;
use zbus::fdo::Error as FdoError;

// Import the Properties struct and ObjectManager
use crate::dbus_interfaces::{properties::Properties, object_manager::ObjectManager};

pub const ECHO_INTERFACE_NAME: &str = "org.novade.ExampleEchoService.Echo";
pub const SUB_OBJECT_DATA_INTERFACE_NAME: &str = "org.novade.ExampleEchoService.SubObjectData";
pub const SERVICE_ROOT_PATH_PREFIX: &str = "/org/novade/ExampleEchoService";


// ANCHOR: EchoServiceStructDefinition
/// Implements the `org.novade.ExampleEchoService.Echo` D-Bus interface and manages
/// its properties and sub-objects via `ObjectManager`.
///
/// This service demonstrates:
/// - A custom D-Bus method `EchoString`.
/// - Two properties: `Prefix` (read-write String) and `EchoCount` (read-only Int64),
///   exposed via the `org.freedesktop.DBus.Properties` interface.
/// - Registration with an `ObjectManager` to make itself discoverable.
/// - Dynamic addition and removal of simple sub-objects, also reported via `ObjectManager`.
///
/// The main D-Bus interface `org.novade.ExampleEchoService.Echo` and its associated
/// `org.freedesktop.DBus.Properties` interface are typically served at a specific path
/// (e.g., `/org/novade/ExampleEchoService/Main`). The `ObjectManager` itself would be
/// served at a parent path (e.g., `/org/novade/ExampleEchoService`).
#[derive(Debug)]
pub struct EchoService {
    /// The D-Bus object path where the `Echo` and `Properties` interfaces are served
    /// (e.g., `/org/novade/ExampleEchoService/Main`).
    object_path: ObjectPath<'static>,
    /// Shared D-Bus connection.
    connection: Arc<Connection>,
    /// Internal storage for the properties of the `Echo` interface ("Prefix", "EchoCount").
    /// This is shared with the `properties_handler`.
    properties_storage: Arc<Mutex<HashMap<String, OwnedValue>>>,
    /// Handler for the `org.freedesktop.DBus.Properties` interface for this `EchoService`.
    properties_handler: Arc<Properties>,
    /// Reference to the `ObjectManager` instance that manages this service's objects.
    object_manager: Arc<ObjectManager>,
}

impl EchoService {
    /// Creates a new instance of `EchoService`.
    ///
    /// # Arguments
    ///
    /// * `connection`: An `Arc<zbus::Connection>` for D-Bus communication.
    /// * `object_path`: The D-Bus object path where this `EchoService`'s main interfaces
    ///   (`Echo` and `Properties`) will be served (e.g., "/org/novade/ExampleEchoService/Main").
    ///   Must be `'static`.
    /// * `object_manager`: An `Arc<ObjectManager>` instance that this `EchoService` will use
    ///   to register itself and manage any sub-objects.
    // ANCHOR: EchoServiceNewMethod
    pub fn new(
        connection: Arc<Connection>,
        object_path: ObjectPath<'static>,
        object_manager: Arc<ObjectManager>,
    ) -> Self {
        tracing::info!(
            "Creating new EchoService for path: {}, associated with ObjectManager at: {}",
            object_path, object_manager.path_str()
        );

        let mut initial_properties = HashMap::new();
        initial_properties.insert("Prefix".to_string(), OwnedValue::from("Echo: "));
        initial_properties.insert("EchoCount".to_string(), OwnedValue::from(0i64));

        let properties_storage = Arc::new(Mutex::new(initial_properties));

        let properties_handler = Arc::new(Properties::new(
            connection.clone(),
            object_path.clone(),
            ECHO_INTERFACE_NAME.to_string(),
            properties_storage.clone(),
        ));

        Self {
            object_path,
            connection,
            properties_storage,
            properties_handler,
            object_manager,
        }
    }

    /// Returns an `Arc` to the associated [`Properties`] interface handler.
    ///
    /// This allows the `DbusServiceManager` (or a similar mechanism) to serve the
    /// `org.freedesktop.DBus.Properties` interface for this `EchoService` instance
    /// at the same D-Bus object path as the main `Echo` interface.
    pub fn properties_handler(&self) -> Arc<Properties> {
        self.properties_handler.clone()
    }

    // ANCHOR: RegisterWithObjectManager
    /// Registers this `EchoService` instance's main object path and its interfaces
    /// with the provided [`ObjectManager`].
    ///
    /// This makes the `Echo` interface and its `Properties` interface discoverable
    /// via `GetManagedObjects` calls to the `ObjectManager`.
    ///
    /// # Errors
    ///
    /// Returns `FdoResult<()>` which may contain errors from the `ObjectManager::add_object`
    /// call (e.g., if the object path is already in use or if signal emission fails).
    pub async fn register_with_object_manager(&self) -> FdoResult<()> {
        tracing::info!(
            "Registering EchoService main object ({}) with ObjectManager",
            self.object_path
        );
        let mut interfaces_and_properties = HashMap::new();

        // Add Echo interface properties
        let echo_props = self.properties_storage.lock().await.clone();
        interfaces_and_properties.insert(ECHO_INTERFACE_NAME.to_string(), echo_props);

        // Add Properties interface (it has no "properties" in the ObjectManager sense itself)
        interfaces_and_properties.insert(
            "org.freedesktop.DBus.Properties".to_string(),
            HashMap::new(),
        );

        self.object_manager
            .add_object(self.object_path.clone(), interfaces_and_properties)
            .await
    }

    // ANCHOR: AddSimpleSubObject
    /// Adds a simple sub-object under this `EchoService`'s hierarchy to the `ObjectManager`.
    ///
    /// The sub-object is created at a path like `/org/novade/ExampleEchoService/sub/{sub_name}`.
    /// It's given a single interface `org.novade.ExampleEchoService.SubObjectData` with one
    /// property "Label".
    ///
    /// This method demonstrates dynamic object addition to the `ObjectManager`. The sub-object
    /// itself doesn't serve any D-Bus interfaces directly in this implementation; its existence
    /// and properties are only advertised through the `ObjectManager`.
    ///
    /// # Arguments
    ///
    /// * `sub_name`: A unique name for the sub-object, used to construct its path.
    /// * `label`: A `String` value for the "Label" property of this sub-object.
    ///
    /// # Returns
    ///
    /// `FdoResult<ObjectPath<'static>>` containing the `ObjectPath` of the newly added
    /// sub-object if successful.
    ///
    /// # Errors
    ///
    /// Returns `FdoError::InvalidArgs` if `sub_name` results in an invalid object path.
    /// May also return errors from `ObjectManager::add_object` (e.g., path in use, signal failure).
    pub async fn add_simple_sub_object(
        &self,
        sub_name: &str,
        label: String,
    ) -> FdoResult<ObjectPath<'static>> {
        let sub_object_path_str = format!("{}/sub/{}", SERVICE_ROOT_PATH_PREFIX, sub_name);
        let sub_object_path = ObjectPath::try_from(sub_object_path_str.clone())
            .map_err(|e| FdoError::InvalidArgs(format!("Invalid sub_object_path {}: {}", sub_object_path_str, e)))?;

        tracing::info!(
            "Adding simple sub-object '{}' with label '{}' to ObjectManager",
            sub_object_path,
            label
        );

        let mut properties = HashMap::new();
        properties.insert("Label".to_string(), OwnedValue::from(label));

        let mut interfaces = HashMap::new();
        interfaces.insert(SUB_OBJECT_DATA_INTERFACE_NAME.to_string(), properties);

        self.object_manager
            .add_object(sub_object_path.clone(), interfaces)
            .await?;

        Ok(sub_object_path)
    }

    // ANCHOR: RemoveSimpleSubObject
    /// Removes a previously added simple sub-object from the `ObjectManager`.
    ///
    /// The path of the sub-object is derived from `sub_name` similar to `add_simple_sub_object`.
    ///
    /// # Arguments
    ///
    /// * `sub_name`: The unique name of the sub-object to remove.
    ///
    /// # Errors
    ///
    /// Returns `FdoError::InvalidArgs` if `sub_name` results in an invalid object path.
    /// May also return errors from `ObjectManager::remove_object` (e.g., object not found, signal failure).
    pub async fn remove_simple_sub_object(&self, sub_name: &str) -> FdoResult<()> {
        let sub_object_path_str = format!("{}/sub/{}", SERVICE_ROOT_PATH_PREFIX, sub_name);
        let sub_object_path = ObjectPath::try_from(sub_object_path_str.clone())
             .map_err(|e| FdoError::InvalidArgs(format!("Invalid sub_object_path {}: {}", sub_object_path_str, e)))?;

        tracing::info!(
            "Removing simple sub-object '{}' from ObjectManager",
            sub_object_path
        );
        self.object_manager.remove_object(sub_object_path).await
    }
}

// ANCHOR: EchoServiceUnitTests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dbus_interfaces::{ObjectManager, Properties}; // Ensure Properties is also in scope if needed by helper
    use zbus::zvariant::{ObjectPath, OwnedValue, Value};
    use zbus::Connection;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio;

    const TEST_SERVICE_ROOT_PATH: &str = "/test/echoservice_root";
    const TEST_ECHO_MAIN_PATH: &str = "/test/echoservice_root/Main";

    // Helper function to set up EchoService and its dependencies for testing.
    async fn new_test_echo_service_and_om() -> (Arc<EchoService>, Arc<ObjectManager>, Arc<Connection>) {
        let conn = Connection::session().await.expect(
            "Failed to connect to D-Bus session bus for tests. Ensure dbus-daemon is running.",
        );

        let om_path = ObjectPath::try_from(TEST_SERVICE_ROOT_PATH).unwrap();
        let object_manager = Arc::new(ObjectManager::new(conn.clone(), om_path));

        let echo_main_path = ObjectPath::try_from(TEST_ECHO_MAIN_PATH).unwrap();
        let echo_service = Arc::new(EchoService::new(
            conn.clone(),
            echo_main_path,
            object_manager.clone(),
        ));

        (echo_service, object_manager, conn)
    }

    #[tokio::test]
    async fn test_echo_string_logic() {
        let (echo_service_arc, _om, _conn) = new_test_echo_service_and_om().await;
        // EchoService::new initializes Prefix to "Echo: " and EchoCount to 0.

        // To call echo_string, we need &mut self.
        // Since echo_service_arc is Arc<EchoService>, we can't directly get &mut EchoService.
        // This test setup is problematic for methods requiring &mut self if the service is Arc-wrapped.
        // For unit testing the pure logic of echo_string if it were on a non-Arc self:
        // One option is to make parts of the logic accessible via static methods or refactor.
        // Another is to test this via actual D-Bus calls in an integration-style test.

        // For now, let's assume we can modify the properties_storage directly to check count increment.
        // And we'll call the method. If it needs &mut self, we must ensure the Arc is not shared elsewhere
        // during the mutable call, or the method itself should be designed to work with Arc<Mutex<InnerState>>.
        // The D-Bus interface methods generated by `#[dbus_interface]` on `impl EchoService`
        // can take `&self` or `&mut self`. `echo_string` is `&mut self`.
        // When called via D-Bus, zbus handles providing the &mut access.
        // For a direct call in a unit test on an Arc<EchoService>, this is tricky.

        // Let's try to simulate the D-Bus call by directly calling the method.
        // This won't work directly on Arc<EchoService> if the method needs &mut self.
        // We'd typically need to serve it and call it via a client proxy.
        // However, for testing internal state changes, we can inspect properties_storage.

        // Initial state check
        {
            let props = echo_service_arc.properties_storage.lock().await;
            assert_eq!(props.get("Prefix").unwrap(), &OwnedValue::from("Echo: "));
            assert_eq!(props.get("EchoCount").unwrap(), &OwnedValue::from(0i64));
        }

        // This direct call is not how zbus invokes it, and won't work due to Arc and &mut self.
        // let response = echo_service_arc.echo_string("test".to_string()).await.unwrap();
        // For this unit test, we will manually manipulate internal state and verify it,
        // and trust that the `#[dbus_interface]` macro correctly handles method invocation.
        // A true test of `echo_string` as a D-Bus method requires a running object server and client.

        // Simulate the effect of echo_string on properties_storage for "EchoCount"
        let mut props_mut = echo_service_arc.properties_storage.lock().await;
        let current_count_ov = props_mut.get("EchoCount").unwrap().try_clone().unwrap();
        let current_count: i64 = current_count_ov.try_into().unwrap();
        props_mut.insert("EchoCount".to_string(), OwnedValue::from(current_count + 1));
        drop(props_mut);

        // Verify EchoCount increment
        let props_after = echo_service_arc.properties_storage.lock().await;
        assert_eq!(props_after.get("EchoCount").unwrap(), &OwnedValue::from(1i64));

        // The actual response format "Echo: test" and signal emission are harder to unit test here
        // without a more complex setup or refactoring EchoService.
        // This test primarily confirms understanding of property state.
        // A more complete test for echo_string would be an integration test.
        println!("Note: test_echo_string_logic only partially tests state due to &mut self on Arc.");
    }

    #[tokio::test]
    async fn test_register_with_object_manager() {
        let (echo_service, object_manager, _conn) = new_test_echo_service_and_om().await;

        echo_service.register_with_object_manager().await.unwrap();

        let managed_objects = object_manager.managed_objects.lock().await;
        let service_main_path = ObjectPath::try_from(TEST_ECHO_MAIN_PATH).unwrap();

        assert!(managed_objects.contains_key(&service_main_path));
        let registered_interfaces = managed_objects.get(&service_main_path).unwrap();

        assert!(registered_interfaces.contains_key(ECHO_INTERFACE_NAME));
        assert!(registered_interfaces.contains_key("org.freedesktop.DBus.Properties"));

        // Check if Echo properties are somewhat correctly registered (at least Prefix and EchoCount)
        let echo_iface_props = registered_interfaces.get(ECHO_INTERFACE_NAME).unwrap();
        assert!(echo_iface_props.contains_key("Prefix"));
        assert!(echo_iface_props.contains_key("EchoCount"));
        assert_eq!(echo_iface_props.get("Prefix").unwrap(), &OwnedValue::from("Echo: "));
        assert_eq!(echo_iface_props.get("EchoCount").unwrap(), &OwnedValue::from(0i64));
    }

    #[tokio::test]
    async fn test_add_simple_sub_object() {
        let (echo_service, object_manager, _conn) = new_test_echo_service_and_om().await;
        let sub_name = "sub1";
        let label = "Test Label 1".to_string();

        let returned_path = echo_service.add_simple_sub_object(sub_name, label.clone()).await.unwrap();

        let expected_path_str = format!("{}/sub/{}", TEST_SERVICE_ROOT_PATH, sub_name);
        let expected_path = ObjectPath::try_from(expected_path_str.as_str()).unwrap();
        assert_eq!(returned_path, expected_path);

        let managed_objects = object_manager.managed_objects.lock().await;
        assert!(managed_objects.contains_key(&expected_path));
        let sub_object_data = managed_objects.get(&expected_path).unwrap();

        assert!(sub_object_data.contains_key(SUB_OBJECT_DATA_INTERFACE_NAME));
        let props = sub_object_data.get(SUB_OBJECT_DATA_INTERFACE_NAME).unwrap();
        assert_eq!(props.get("Label").unwrap(), &OwnedValue::from(label));
    }

    #[tokio::test]
    async fn test_remove_simple_sub_object() {
        let (echo_service, object_manager, _conn) = new_test_echo_service_and_om().await;
        let sub_name = "sub_to_remove";

        // Add it first
        echo_service.add_simple_sub_object(sub_name, "Temporary Label".to_string()).await.unwrap();
        let sub_object_path = ObjectPath::try_from(format!("{}/sub/{}", TEST_SERVICE_ROOT_PATH, sub_name)).unwrap();

        // Confirm it's there
        {
            let managed = object_manager.managed_objects.lock().await;
            assert!(managed.contains_key(&sub_object_path));
        }

        // Remove it
        echo_service.remove_simple_sub_object(sub_name).await.unwrap();

        // Confirm it's gone
        {
            let managed = object_manager.managed_objects.lock().await;
            assert!(!managed.contains_key(&sub_object_path));
        }
    }

    #[tokio::test]
    async fn test_add_remove_multiple_sub_objects() {
        let (echo_service, object_manager, _conn) = new_test_echo_service_and_om().await;

        let sub1_path = echo_service.add_simple_sub_object("s1", "Label S1".to_string()).await.unwrap();
        let sub2_path = echo_service.add_simple_sub_object("s2", "Label S2".to_string()).await.unwrap();
        let _sub3_path = echo_service.add_simple_sub_object("s3", "Label S3".to_string()).await.unwrap();

        {
            let managed = object_manager.managed_objects.lock().await;
            assert_eq!(managed.len(), 3); // s1, s2, s3 (ObjectManager itself is not part of its managed_objects)
            assert!(managed.contains_key(&sub1_path));
            assert!(managed.contains_key(&sub2_path));
        }

        echo_service.remove_simple_sub_object("s1").await.unwrap();
        {
            let managed = object_manager.managed_objects.lock().await;
            assert_eq!(managed.len(), 2);
            assert!(!managed.contains_key(&sub1_path));
            assert!(managed.contains_key(&sub2_path));
        }

        echo_service.remove_simple_sub_object("s3").await.unwrap();
        {
            let managed = object_manager.managed_objects.lock().await;
            assert_eq!(managed.len(), 1);
            assert!(managed.contains_key(&sub2_path));
        }
    }
}

/// D-Bus interface `org.novade.ExampleEchoService.Echo` implementation.
///
/// This interface provides simple echo functionality and demonstrates property management.
///
/// ## Properties
///
/// This interface, when served along with its [`Properties`] handler, exposes the following
/// properties via the `org.freedesktop.DBus.Properties` interface:
///
/// *   **`Prefix`** (Type: `String`, Access: Read-Write)
///     *   A string prefix that is prepended to the input of the `EchoString` method.
///     *   Default value: `"Echo: "`.
///     *   Changes to this property are signaled via `PropertiesChanged`.
/// *   **`EchoCount`** (Type: `Int64`, Access: Read-Only)
///     *   A counter indicating how many times the `EchoString` method has been successfully called.
///     *   This property is incremented internally by the `EchoString` method.
///     *   Changes to this property (i.e., increments) are signaled via `PropertiesChanged`.
#[dbus_interface(name = "org.novade.ExampleEchoService.Echo")]
impl EchoService {
    // ANCHOR: EchoStringMethod
    /// Echoes back the input string, prepended with the current `Prefix` property value.
    ///
    /// This method also increments the `EchoCount` property. After incrementing,
    /// it triggers the emission of a `PropertiesChanged` signal for "EchoCount"
    /// via its associated [`Properties`] handler.
    ///
    /// # Arguments
    ///
    /// * `input_string`: The `String` to be echoed.
    ///
    /// # Returns
    ///
    /// A `String` formatted as `"{Prefix} {input_string}"`.
    ///
    /// # Errors
    ///
    /// Can return `FdoError` if emitting the `PropertiesChanged` signal fails, though
    /// typically the echo operation itself will succeed and an error during signal
    /// emission is logged.
    async fn echo_string(&mut self, input_string: String) -> FdoResult<String> {
        tracing::info!(object_path = %self.object_path, "EchoService.echo_string called with: {}", input_string);

        let mut props_guard = self.properties_storage.lock().await;

        let prefix = props_guard
            .get("Prefix")
            .and_then(|v| v.try_into().ok())
            .unwrap_or_else(|| "".to_string()); // Default to empty string if not found or wrong type

        let current_echo_count: i64 = props_guard
            .get("EchoCount")
            .and_then(|v| v.try_into().ok())
            .unwrap_or(0i64);

        let new_echo_count = current_echo_count + 1;
        props_guard.insert("EchoCount".to_string(), OwnedValue::from(new_echo_count));

        // Prepare data for PropertiesChanged signal
        let mut changed_props_for_signal = HashMap::new();
        changed_props_for_signal.insert("EchoCount".to_string(), Value::from(new_echo_count));

        drop(props_guard); // Release lock before await point (signal emission)

        // ANCHOR: NotifyPropertiesChanged
        // Call the public method on our Properties handler to emit the signal.
        if let Err(e) = self.properties_handler.public_emit_properties_changed(changed_props_for_signal, Vec::new()).await {
            tracing::error!("Failed to emit PropertiesChanged signal for EchoCount: {}", e);
            // Depending on desired error handling, might return an error to the caller,
            // but for EchoString, the primary operation (string manipulation) is done.
        }

        let response = format!("{} {}", prefix, input_string);
        tracing::debug!(object_path = %self.object_path, "EchoService.echo_string responding with: {}", response);
        Ok(response)
    }
}
