//! # D-Bus `org.freedesktop.DBus.ObjectManager` Interface Implementation
//!
//! This module provides a Rust implementation of the standard D-Bus `ObjectManager`
//! interface. This interface is used by D-Bus services to expose a collection of
//! objects and their interfaces to clients. It allows clients to discover all objects,
//! their interfaces, and properties with a single method call (`GetManagedObjects`),
//! and to be notified of objects and interfaces being added or removed via signals.
//!
//! The main struct [`ObjectManager`] is designed to be served at a specific root path
//! of a D-Bus service. Other services or components within the application can then
//! register their D-Bus objects (which typically reside at paths under this root path)
//! with this `ObjectManager`.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::{Connection, dbus_interface, interface, SignalContext};
use zbus::zvariant::{ObjectPath, Value, OwnedValue};
use zbus::fdo::Result as FdoResult; // Using FDO Result for D-Bus methods

// ANCHOR: ObjectManagerStructDefinition
/// Implements the `org.freedesktop.DBus.ObjectManager` D-Bus interface.
///
/// This struct maintains a collection of D-Bus objects, their interfaces, and associated properties.
/// It provides the `GetManagedObjects` method for clients to introspect the object hierarchy
/// and emits `InterfacesAdded` and `InterfacesRemoved` signals when objects or their interfaces
/// are dynamically added or removed.
///
/// Typically, a single instance of `ObjectManager` is served at a well-known root path for a service
/// (e.g., `/org/example/MyService`). Other components of the service then register their specific
/// D-Bus objects (which usually have paths like `/org/example/MyService/Object1`) with this manager.
///
/// # Example
/// ```no_run
/// # use novade_system::dbus_interfaces::ObjectManager;
/// # use zbus::zvariant::{ObjectPath, OwnedValue};
/// # use zbus::Connection;
/// # use std::sync::Arc;
/// # use std::collections::HashMap;
/// # async fn run() -> anyhow::Result<()> {
/// let conn = Arc::new(Connection::session().await?);
/// let manager_obj_path = ObjectPath::try_from("/org/novade/MyManagedService")?;
/// let object_manager = Arc::new(ObjectManager::new(conn.clone(), manager_obj_path.into_owned()));
///
/// // This object_manager can then be served using DbusServiceManager or zbus ObjectServer.
/// // Other parts of the application can then add objects to it:
/// let child_obj_path = ObjectPath::try_from("/org/novade/MyManagedService/Child1")?;
/// let mut ifaces = HashMap::new();
/// let mut props = HashMap::new();
/// props.insert("MyProperty".to_string(), OwnedValue::from("Example"));
/// ifaces.insert("org.novade.MyInterface".to_string(), props);
/// object_manager.add_object(child_obj_path.into_owned(), ifaces).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ObjectManager {
    /// D-Bus connection used for emitting `InterfacesAdded` and `InterfacesRemoved` signals.
    connection: Arc<Connection>,
    /// The D-Bus object path where this `ObjectManager` instance itself is served.
    manager_path: ObjectPath<'static>,
    /// Stores the managed objects.
    /// The outer `HashMap` maps an `ObjectPath` (of a managed object) to its interfaces.
    /// The next `HashMap` maps an interface name (e.g., "org.freedesktop.ColorManager.ColorPalette")
    /// to its properties.
    /// The innermost `HashMap` maps a property name to its `zbus::zvariant::OwnedValue`.
    managed_objects: Arc<Mutex<HashMap<ObjectPath<'static>, HashMap<String, HashMap<String, OwnedValue>>>>>,
}

impl ObjectManager {
    /// Creates a new `ObjectManager`.
    ///
    /// # Arguments
    ///
    /// * `connection`: An `Arc<zbus::Connection>` used for emitting signals.
    /// * `manager_path`: The D-Bus object path where this `ObjectManager` interface itself will be hosted.
    ///   This path must be `'static`.
    // ANCHOR: ObjectManagerNewMethod
    pub fn new(connection: Arc<Connection>, manager_path: ObjectPath<'static>) -> Self {
        tracing::info!("Creating new ObjectManager at path: {}", manager_path);
        Self {
            connection,
            manager_path,
            managed_objects: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns a reference to the D-Bus path where this `ObjectManager` instance is served.
    pub fn path(&self) -> &ObjectPath<'static> {
        &self.manager_path
    }

    /// Returns the D-Bus path as a string slice where this `ObjectManager` instance is served.
    pub fn path_str(&self) -> &str {
        self.manager_path.as_str()
    }

    // ANCHOR: AddObjectHelper
    /// Adds a new object to be managed by this `ObjectManager`.
    ///
    /// This method stores the object's path, interfaces, and properties. It then emits
    /// the `InterfacesAdded` D-Bus signal to notify clients about the new object.
    ///
    /// # Arguments
    ///
    /// * `path`: The D-Bus `ObjectPath` of the new object. This path must be `'static`.
    /// * `interfaces_and_properties`: A `HashMap` where keys are interface names (as `String`)
    ///   and values are another `HashMap` representing the properties of that interface
    ///   (property name `String` to `OwnedValue`).
    ///
    /// # Errors
    ///
    /// * `zbus::fdo::Error::ObjectPathInUse`: If an object with the given `path` already exists.
    /// * `zbus::fdo::Error::Failed`: If emitting the `InterfacesAdded` signal fails.
    #[tracing::instrument(skip(self, interfaces_and_properties), fields(path = %path))]
    pub async fn add_object(
        &self,
        path: ObjectPath<'static>,
        interfaces_and_properties: HashMap<String, HashMap<String, OwnedValue>>,
    ) -> FdoResult<()> {
        tracing::info!("Adding object at path: {}", path);
        let mut objects = self.managed_objects.lock().await;
        if objects.contains_key(&path) {
            tracing::warn!("Object path {} already exists. Not adding.", path);
            // Or update, depending on desired semantics. For now, let's treat as error/noop.
            return Err(zbus::fdo::Error::ObjectPathInUse(path.to_string()));
        }
        objects.insert(path.clone(), interfaces_and_properties.clone());
        drop(objects); // Release lock before emitting signal

        // Convert OwnedValue to Value for the signal
        let signal_props: HashMap<String, HashMap<String, Value<'_>>> = interfaces_and_properties
            .into_iter()
            .map(|(iface, props)| {
                let valued_props = props.into_iter().map(|(k, v)| (k, v.into())).collect();
                (iface, valued_props)
            })
            .collect();

        tracing::debug!("Emitting InterfacesAdded signal for path: {}", path);
        Self::interfaces_added(&self.connection, &path, signal_props)
            .await
            .map_err(|e| {
                tracing::error!("Failed to emit InterfacesAdded signal for {}: {}", path, e);
                zbus::fdo::Error::Failed(format!("Failed to emit InterfacesAdded signal: {}", e))
            })?;
        Ok(())
    }

    // ANCHOR: RemoveObjectHelper
    /// Removes an object from management by this `ObjectManager`.
    ///
    /// This method removes the object's data from the internal store. If the object existed
    /// and had interfaces, it then emits the `InterfacesRemoved` D-Bus signal, listing all
    /// interfaces that were part of the removed object.
    ///
    /// # Arguments
    ///
    /// * `path`: The D-Bus `ObjectPath` of the object to remove.
    ///
    /// # Errors
    ///
    /// * `zbus::fdo::Error::UnknownObject`: If no object with the given `path` is found.
    /// * `zbus::fdo::Error::Failed`: If emitting the `InterfacesRemoved` signal fails.
    #[tracing::instrument(skip(self), fields(path = %path))]
    pub async fn remove_object(&self, path: ObjectPath<'static>) -> FdoResult<()> {
        tracing::info!("Removing object at path: {}", path);
        let mut objects = self.managed_objects.lock().await;
        if let Some(removed_object_interfaces) = objects.remove(&path) {
            drop(objects); // Release lock before emitting signal

            let interface_names: Vec<String> = removed_object_interfaces.keys().cloned().collect();
            if !interface_names.is_empty() {
                tracing::debug!("Emitting InterfacesRemoved signal for path: {} with interfaces: {:?}", path, interface_names);
                Self::interfaces_removed(&self.connection, &path, interface_names)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to emit InterfacesRemoved signal for {}: {}", path, e);
                        zbus::fdo::Error::Failed(format!("Failed to emit InterfacesRemoved signal: {}", e))
                    })?;
            }
            Ok(())
        } else {
            tracing::warn!("Object path {} not found for removal.", path);
            Err(zbus::fdo::Error::UnknownObject(path.to_string()))
        }
    }

    // ANCHOR: AddInterfaceHelper
    /// Adds an interface to an existing managed object.
    ///
    /// This method adds the specified interface and its properties to an object already
    /// managed by this `ObjectManager`. It then emits the `InterfacesAdded` D-Bus signal
    /// for the newly added interface on that object.
    ///
    /// # Arguments
    ///
    /// * `object_path`: The `ObjectPath` of the existing object to modify. Must be `'static`.
    /// * `interface_name`: The name of the D-Bus interface to add (as `String`).
    /// * `properties`: A `HashMap` representing the properties of the new interface
    ///   (property name `String` to `OwnedValue`).
    ///
    /// # Errors
    ///
    /// * `zbus::fdo::Error::UnknownObject`: If the `object_path` does not correspond to a managed object.
    /// * `zbus::fdo::Error::Failed`: If an interface with the same `interface_name` already
    ///   exists on the object, or if emitting the `InterfacesAdded` signal fails.
    #[tracing::instrument(skip(self, properties), fields(object_path = %object_path, interface_name = %interface_name))]
    pub async fn add_interface(
        &self,
        object_path: ObjectPath<'static>,
        interface_name: String,
        properties: HashMap<String, OwnedValue>,
    ) -> FdoResult<()> {
        tracing::info!("Adding interface {} to object {}", interface_name, object_path);
        let mut objects = self.managed_objects.lock().await;
        match objects.get_mut(&object_path) {
            Some(object_interfaces) => {
                if object_interfaces.contains_key(&interface_name) {
                    tracing::warn!("Interface {} already exists on object {}. Not adding.", interface_name, object_path);
                    return Err(zbus::fdo::Error::Failed(format!("Interface {} already exists on object {}", interface_name, object_path)));
                }
                object_interfaces.insert(interface_name.clone(), properties.clone());
                drop(objects); // Release lock

                let mut signal_props_map = HashMap::new();
                let valued_props: HashMap<String, Value<'_>> = properties.into_iter().map(|(k,v)| (k, v.into())).collect();
                signal_props_map.insert(interface_name.clone(), valued_props);

                tracing::debug!("Emitting InterfacesAdded signal for new interface {} on object {}", interface_name, object_path);
                Self::interfaces_added(&self.connection, &object_path, signal_props_map)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to emit InterfacesAdded signal for {} on {}: {}", interface_name, object_path, e);
                        zbus::fdo::Error::Failed(format!("Failed to emit InterfacesAdded signal: {}", e))
                    })?;
                Ok(())
            }
            None => {
                tracing::warn!("Object {} not found when trying to add interface {}.", object_path, interface_name);
                Err(zbus::fdo::Error::UnknownObject(object_path.to_string()))
            }
        }
    }

    // ANCHOR: RemoveInterfaceHelper
    /// Removes an interface from an existing managed object.
    ///
    /// This method removes the specified interface and its properties from an object
    /// managed by this `ObjectManager`. It then emits the `InterfacesRemoved` D-Bus signal,
    /// listing the removed interface.
    ///
    /// # Arguments
    ///
    /// * `object_path`: The `ObjectPath` of the object from which to remove the interface. Must be `'static`.
    /// * `interface_name`: The name of the D-Bus interface to remove (as `String`).
    ///
    /// # Errors
    ///
    /// * `zbus::fdo::Error::UnknownObject`: If the `object_path` does not correspond to a managed object.
    /// * `zbus::fdo::Error::UnknownInterface`: If the specified `interface_name` does not exist on the object.
    /// * `zbus::fdo::Error::Failed`: If emitting the `InterfacesRemoved` signal fails.
    #[tracing::instrument(skip(self), fields(object_path = %object_path, interface_name = %interface_name))]
    pub async fn remove_interface(
        &self,
        object_path: ObjectPath<'static>,
        interface_name: String,
    ) -> FdoResult<()> {
        tracing::info!("Removing interface {} from object {}", interface_name, object_path);
        let mut objects = self.managed_objects.lock().await;
        match objects.get_mut(&object_path) {
            Some(object_interfaces) => {
                if object_interfaces.remove(&interface_name).is_some() {
                    drop(objects); // Release lock

                    let removed_interfaces_vec = vec![interface_name.clone()];
                    tracing::debug!("Emitting InterfacesRemoved signal for interface {} on object {}", interface_name, object_path);
                    Self::interfaces_removed(&self.connection, &object_path, removed_interfaces_vec)
                        .await
                        .map_err(|e| {
                            tracing::error!("Failed to emit InterfacesRemoved signal for {} on {}: {}", interface_name, object_path, e);
                            zbus::fdo::Error::Failed(format!("Failed to emit InterfacesRemoved signal: {}", e))
                        })?;
                    Ok(())
                } else {
                    tracing::warn!("Interface {} not found on object {} for removal.", interface_name, object_path);
                    Err(zbus::fdo::Error::UnknownInterface(interface_name))
                }
            }
            None => {
                tracing::warn!("Object {} not found when trying to remove interface {}.", object_path, interface_name);
                Err(zbus::fdo::Error::UnknownObject(object_path.to_string()))
            }
        }
    }
}

/// D-Bus methods for `org.freedesktop.DBus.ObjectManager`.
#[dbus_interface(name = "org.freedesktop.DBus.ObjectManager")]
impl ObjectManager {
    // ANCHOR: GetManagedObjectsMethod
    /// Retrieves all managed objects, their interfaces, and properties.
    ///
    /// Implements the `GetManagedObjects` method of the `org.freedesktop.DBus.ObjectManager` interface.
    /// This method returns a dictionary where keys are object paths and values are dictionaries
    /// mapping interface names to their properties.
    ///
    /// The returned structure is `HashMap<ObjectPath<'static>, HashMap<String, HashMap<String, OwnedValue>>>`.
    /// - Outer `HashMap` key: `ObjectPath<'static>` of a managed object.
    /// - Inner `HashMap` key: `String` representing an interface name on that object.
    /// - Innermost `HashMap` key: `String` representing a property name on that interface.
    /// - Innermost `HashMap` value: `OwnedValue` of the property.
    ///
    /// This allows a client to get a complete snapshot of all objects managed by this `ObjectManager`
    /// with a single D-Bus call.
    #[dbus_interface(name = "GetManagedObjects")]
    async fn get_managed_objects(&self) -> FdoResult<HashMap<ObjectPath<'static>, HashMap<String, HashMap<String, OwnedValue>>>> {
        tracing::debug!("Received GetManagedObjects call for manager at path: {}", self.manager_path);
        let objects = self.managed_objects.lock().await;
        // We need to clone the data to return OwnedValue, and also to avoid holding the lock unnecessarily.
        // The interface requires HashMap<String, HashMap<String, OwnedValue>>, so direct cloning is fine.
        Ok(objects.clone())
    }

    // ANCHOR: InterfacesAddedSignal
    /// D-Bus signal emitted when one or more objects/interfaces are added.
    ///
    /// This signal is part of the `org.freedesktop.DBus.ObjectManager` interface specification.
    /// It is emitted by `add_object` and `add_interface` methods of this struct.
    ///
    /// # Arguments
    ///
    /// * `object_path`: The `ObjectPath` of the object that has new interfaces.
    /// * `interfaces_and_properties`: A dictionary where keys are the newly added interface names (as `String`)
    ///   and values are dictionaries of their properties (property name `String` to `Value`).
    ///   If a whole new object is added, this will contain all its interfaces and properties.
    ///   If a new interface is added to an existing object, this will contain only that new interface.
    #[dbus_interface(signal)]
    async fn interfaces_added(
        signal_ctxt: &SignalContext<'_>,
        object_path: ObjectPath<'_>,
        interfaces_and_properties: HashMap<String, HashMap<String, Value<'_>>>,
    ) -> zbus::Result<()>;

    // ANCHOR: InterfacesRemovedSignal
    /// D-Bus signal emitted when one or more objects/interfaces are removed.
    ///
    /// This signal is part of the `org.freedesktop.DBus.ObjectManager` interface specification.
    /// It is emitted by `remove_object` and `remove_interface` methods of this struct.
    ///
    /// # Arguments
    ///
    /// * `object_path`: The `ObjectPath` of the object from which interfaces were removed.
    /// * `interfaces`: A list of interface names (as `String`) that were removed from the object.
    ///   If a whole object is removed, this list will contain all interfaces that were on that object.
    #[dbus_interface(signal)]
    async fn interfaces_removed(
        signal_ctxt: &SignalContext<'_>,
        object_path: ObjectPath<'_>,
        interfaces: Vec<String>,
    ) -> zbus::Result<()>;
}

// ANCHOR: ObjectManagerUnitTests
#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::{ObjectPath, OwnedValue}; // Value is already imported via super::*
    use std::collections::HashMap;
    use tokio; // Ensure tokio is available for async tests
    use std::sync::Arc;

    // Helper for tests to create a new ObjectManager instance and a Connection.
    // This makes tests behave more like integration tests as they require a session bus.
    async fn new_test_object_manager_with_conn() -> (ObjectManager, Arc<Connection>) {
        // Attempt to connect to the session bus. This is required because ObjectManager needs Arc<Connection>.
        // Ensure that a D-Bus session daemon is running when tests are executed.
        let conn = Connection::session().await.expect(
            "Failed to connect to session bus for test. Ensure dbus-daemon is running.",
        );
        let manager_path = ObjectPath::try_from("/test/object_manager").unwrap();
        let om = ObjectManager::new(conn.clone(), manager_path);
        (om, conn) // Return connection if needed for signal context, though not directly used for state checks
    }

    fn create_sample_props(key: &str, val_str: &str) -> HashMap<String, OwnedValue> {
        let mut props = HashMap::new();
        props.insert(key.to_string(), OwnedValue::from(val_str.to_string()));
        props
    }

    fn create_sample_interfaces_data(
        iface_name: &str,
        prop_key: &str,
        prop_val: &str,
    ) -> HashMap<String, HashMap<String, OwnedValue>> {
        let mut interfaces = HashMap::new();
        interfaces.insert(iface_name.to_string(), create_sample_props(prop_key, prop_val));
        interfaces
    }

    #[tokio::test]
    async fn test_add_object_basic() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path_str = "/test/object1";
        let obj_path = ObjectPath::try_from(obj_path_str).unwrap();
        let iface_name = "org.test.Interface1";
        let interfaces_data = create_sample_interfaces_data(iface_name, "Prop1", "Value1");

        om.add_object(obj_path.clone(), interfaces_data.clone()).await.unwrap();

        let managed = om.managed_objects.lock().await;
        assert!(managed.contains_key(&obj_path));
        let obj_data = managed.get(&obj_path).unwrap();
        assert!(obj_data.contains_key(iface_name));
        assert_eq!(
            obj_data.get(iface_name).unwrap().get("Prop1").unwrap(),
            &OwnedValue::from("Value1")
        );
    }

    #[tokio::test]
    async fn test_add_object_multiple_interfaces() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path_str = "/test/object_multi";
        let obj_path = ObjectPath::try_from(obj_path_str).unwrap();

        let mut interfaces_data = HashMap::new();
        interfaces_data.insert("org.test.InterfaceA".to_string(), create_sample_props("PropA", "ValA"));
        interfaces_data.insert("org.test.InterfaceB".to_string(), create_sample_props("PropB", "ValB"));

        om.add_object(obj_path.clone(), interfaces_data.clone()).await.unwrap();

        let managed = om.managed_objects.lock().await;
        let obj_data = managed.get(&obj_path).expect("Object not found");
        assert_eq!(obj_data.len(), 2);
        assert!(obj_data.contains_key("org.test.InterfaceA"));
        assert!(obj_data.contains_key("org.test.InterfaceB"));
        assert_eq!(
            obj_data.get("org.test.InterfaceA").unwrap().get("PropA").unwrap(),
            &OwnedValue::from("ValA")
        );
    }

    #[tokio::test]
    async fn test_add_object_already_exists() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path = ObjectPath::try_from("/test/object_exists").unwrap();
        let interfaces_data = create_sample_interfaces_data("org.test.Iface", "P", "V");

        om.add_object(obj_path.clone(), interfaces_data.clone()).await.unwrap();
        // Try adding again
        let result = om.add_object(obj_path.clone(), interfaces_data).await;
        assert!(result.is_err());
        if let Err(zbus::fdo::Error::ObjectPathInUse(p)) = result {
            assert_eq!(p, obj_path.as_str());
        } else {
            panic!("Expected ObjectPathInUse error, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_remove_object_existing() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path = ObjectPath::try_from("/test/object_to_remove").unwrap();
        let interfaces_data = create_sample_interfaces_data("org.test.Iface", "P", "V");
        om.add_object(obj_path.clone(), interfaces_data).await.unwrap();

        // Ensure it's there first
        {
            let managed_initial = om.managed_objects.lock().await;
            assert!(managed_initial.contains_key(&obj_path));
        }

        om.remove_object(obj_path.clone()).await.unwrap();

        let managed_after_remove = om.managed_objects.lock().await;
        assert!(!managed_after_remove.contains_key(&obj_path));
    }

    #[tokio::test]
    async fn test_remove_object_non_existing() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path = ObjectPath::try_from("/test/object_not_there").unwrap();

        let result = om.remove_object(obj_path.clone()).await;
        assert!(result.is_err());
        if let Err(zbus::fdo::Error::UnknownObject(p)) = result {
            assert_eq!(p, obj_path.as_str());
        } else {
            panic!("Expected UnknownObject error, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_add_interface_to_existing_object() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path = ObjectPath::try_from("/test/object_add_iface").unwrap();
        let initial_interfaces = create_sample_interfaces_data("org.test.Iface1", "P1", "V1");
        om.add_object(obj_path.clone(), initial_interfaces).await.unwrap();

        let new_iface_name = "org.test.Iface2".to_string();
        let new_iface_props = create_sample_props("P2", "V2");
        om.add_interface(obj_path.clone(), new_iface_name.clone(), new_iface_props.clone()).await.unwrap();

        let managed = om.managed_objects.lock().await;
        let obj_data = managed.get(&obj_path).unwrap();
        assert_eq!(obj_data.len(), 2); // Should have Iface1 and Iface2
        assert!(obj_data.contains_key("org.test.Iface1"));
        assert!(obj_data.contains_key(&new_iface_name));
        assert_eq!(obj_data.get(&new_iface_name).unwrap().get("P2").unwrap(), &OwnedValue::from("V2"));
    }

    #[tokio::test]
    async fn test_add_interface_to_non_existing_object() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path = ObjectPath::try_from("/test/object_not_there_for_iface").unwrap();
        let new_iface_name = "org.test.IfaceNew".to_string();
        let new_iface_props = create_sample_props("P", "V");

        let result = om.add_interface(obj_path.clone(), new_iface_name, new_iface_props).await;
        assert!(result.is_err());
        if let Err(zbus::fdo::Error::UnknownObject(p)) = result {
            assert_eq!(p, obj_path.as_str());
        } else {
            panic!("Expected UnknownObject error, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_add_interface_already_exists() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path = ObjectPath::try_from("/test/object_iface_exists").unwrap();
        let iface_name = "org.test.IfaceExisting";
        let initial_interfaces = create_sample_interfaces_data(iface_name, "P1", "V1");
        om.add_object(obj_path.clone(), initial_interfaces).await.unwrap();

        // Try to add the same interface again
        let new_props = create_sample_props("P2", "V2"); // Different props
        let result = om.add_interface(obj_path.clone(), iface_name.to_string(), new_props).await;
        assert!(result.is_err());
        // Expecting a generic "Failed" error as per current implementation
        if let Err(zbus::fdo::Error::Failed(msg)) = result {
             assert!(msg.contains("already exists"));
        } else {
            panic!("Expected Failed error for existing interface, got {:?}", result);
        }
    }


    #[tokio::test]
    async fn test_remove_interface_existing() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path = ObjectPath::try_from("/test/object_remove_iface").unwrap();
        let mut interfaces_data = create_sample_interfaces_data("org.test.Iface1", "P1", "V1");
        interfaces_data.insert("org.test.Iface2".to_string(), create_sample_props("P2", "V2"));
        om.add_object(obj_path.clone(), interfaces_data).await.unwrap();

        om.remove_interface(obj_path.clone(), "org.test.Iface1".to_string()).await.unwrap();

        let managed = om.managed_objects.lock().await;
        let obj_data = managed.get(&obj_path).unwrap();
        assert_eq!(obj_data.len(), 1);
        assert!(!obj_data.contains_key("org.test.Iface1"));
        assert!(obj_data.contains_key("org.test.Iface2"));
    }

    #[tokio::test]
    async fn test_remove_interface_last_one() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path = ObjectPath::try_from("/test/object_remove_last_iface").unwrap();
        let iface_name = "org.test.OnlyIface";
        let interfaces_data = create_sample_interfaces_data(iface_name, "P", "V");
        om.add_object(obj_path.clone(), interfaces_data).await.unwrap();

        om.remove_interface(obj_path.clone(), iface_name.to_string()).await.unwrap();

        let managed = om.managed_objects.lock().await;
        let obj_data = managed.get(&obj_path).unwrap();
        assert!(obj_data.is_empty()); // Object path still exists, but has no interfaces
    }

    #[tokio::test]
    async fn test_remove_interface_non_existing_on_object() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path = ObjectPath::try_from("/test/object_no_such_iface").unwrap();
        let initial_interfaces = create_sample_interfaces_data("org.test.Iface1", "P1", "V1");
        om.add_object(obj_path.clone(), initial_interfaces).await.unwrap();

        let result = om.remove_interface(obj_path.clone(), "org.test.NonExistingIface".to_string()).await;
        assert!(result.is_err());
        if let Err(zbus::fdo::Error::UnknownInterface(name)) = result {
            assert_eq!(name, "org.test.NonExistingIface");
        } else {
            panic!("Expected UnknownInterface error, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_remove_interface_from_non_existing_object() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path = ObjectPath::try_from("/test/object_not_there_for_iface_remove").unwrap();

        let result = om.remove_interface(obj_path.clone(), "org.test.AnyIface".to_string()).await;
        assert!(result.is_err());
        if let Err(zbus::fdo::Error::UnknownObject(p)) = result {
            assert_eq!(p, obj_path.as_str());
        } else {
            panic!("Expected UnknownObject error, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_get_managed_objects_empty() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let managed_objects_map = om.get_managed_objects().await.unwrap();
        assert!(managed_objects_map.is_empty());
    }

    #[tokio::test]
    async fn test_get_managed_objects_with_data() {
        let (om, _conn) = new_test_object_manager_with_conn().await;
        let obj_path1 = ObjectPath::try_from("/test/objA").unwrap();
        let ifaces1 = create_sample_interfaces_data("org.test.IA", "PA", "VA");
        om.add_object(obj_path1.clone(), ifaces1.clone()).await.unwrap();

        let obj_path2 = ObjectPath::try_from("/test/objB").unwrap();
        let ifaces2 = create_sample_interfaces_data("org.test.IB", "PB", "VB");
        om.add_object(obj_path2.clone(), ifaces2.clone()).await.unwrap();

        let managed_objects_map = om.get_managed_objects().await.unwrap();
        assert_eq!(managed_objects_map.len(), 2);
        assert_eq!(managed_objects_map.get(&obj_path1).unwrap(), &ifaces1);
        assert_eq!(managed_objects_map.get(&obj_path2).unwrap(), &ifaces2);
    }
}
