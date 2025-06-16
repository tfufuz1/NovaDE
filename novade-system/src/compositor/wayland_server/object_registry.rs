use std::collections::HashMap;
use std::fmt;

// Define client and server ID ranges
const CLIENT_ID_MIN: u32 = 1;
const CLIENT_ID_MAX: u32 = 0xFFFEFFFF; // Placeholder, actual max is more like 0xFEFFFFFF based on some sources. Using a slightly smaller max for now.
                                       // Smithay uses 0x01000000 to 0xFEFFFFFF for client, 0xFF000000 - 0xFFFFFFFF for server.
                                       // Let's use a simpler split for now and refine if needed.
                                       // Wayland spec: "Objects IDs are currently 32-bit integers.
                                       // IDs from 0 to 0xFEFFFFFF are normal client-side objects.
                                       // IDs from 0xFF000000 to 0xFFFFFFFF are server-side implementation objects."
                                       // So, let's adjust CLIENT_ID_MAX and SERVER_ID_MIN.
const ACTUAL_CLIENT_ID_MAX: u32 = 0xFEFFFFFF;
const SERVER_ID_MIN: u32 = 0xFF000000;
const SERVER_ID_MAX: u32 = 0xFFFFFFFF;


/// A trait that all Wayland objects managed by the registry must implement.
/// It can be expanded later with methods for dispatching messages, type information, etc.
pub trait WaylandObject: fmt::Debug + Send + Sync {
    // fn id(&self) -> u32; // Might be useful later
    // fn interface_name(&self) -> &'static str; // Might be useful later
    // fn version(&self) -> u32; // Might be useful later
    // fn dispatch_request(&mut self, opcode: u16, args: Vec<Argument>) -> Result<(), String>; // For later
}

// Allow `dyn WaylandObject` to be used in a way that it itself is Send + Sync.
// This is generally true if all implementors are Send + Sync.
// The `Box<dyn WaylandObject + Send + Sync>` already ensures this.

/// Represents an entry in the ObjectRegistry.
#[derive(Debug)]
pub struct RegistryEntry {
    pub object: Box<dyn WaylandObject>, // Removed Send + Sync here as it's on the trait itself
    pub client_id: u64,                 // ID of the client that "owns" or created this object
    pub version: u32,
    // pub interface: &'static str, // Could store interface string for debugging/type checking
}

/// Manages Wayland objects and their IDs.
#[derive(Debug)]
pub struct ObjectRegistry {
    entries: HashMap<u32, RegistryEntry>,
    next_server_object_id: u32,
}

/// Placeholder for the wl_display object.
#[derive(Debug)]
pub struct WlDisplay;
impl WaylandObject for WlDisplay {}

/// Placeholder for the wl_registry object.
#[derive(Debug)]
pub struct WlRegistry;
impl WaylandObject for WlRegistry {}


impl ObjectRegistry {
    /// Creates a new ObjectRegistry.
    /// Pre-allocates `wl_display` as object ID 1.
    pub fn new() -> Self {
        let mut registry = ObjectRegistry {
            entries: HashMap::new(),
            next_server_object_id: SERVER_ID_MIN,
        };

        // Create and register wl_display (object ID 1, version 0)
        // The client_id for server objects like wl_display can be a special value (e.g., 0)
        // or not strictly necessary if server objects are identified by their ID range.
        let display = WlDisplay;
        registry.entries.insert(
            1, // wl_display is always ID 1
            RegistryEntry {
                object: Box::new(display),
                client_id: 0, // Special client_id for server-owned global objects
                version: 0,   // Typically version 0 or 1 for globals
            },
        );
        // Note: next_server_object_id starts at SERVER_ID_MIN, so wl_display (ID 1) is not from this pool.
        registry
    }

    /// Registers a new object created by a client.
    /// Object IDs from clients should be in the range 1-0xFEFFFFFF.
    /// The special ID 0 is invalid (null object). ID 1 (wl_display) is server-owned.
    pub fn new_object<T: WaylandObject + 'static>(
        &mut self,
        client_id_assoc: u64, // The client session ID this object is associated with
        object_id: u32,
        object: T,
        version: u32,
    ) -> Result<(), String> {
        if object_id == 0 {
            return Err("Object ID 0 is invalid (null object).".to_string());
        }
        if object_id > ACTUAL_CLIENT_ID_MAX {
            return Err(format!(
                "Client object ID {} is out of the allowed client range (1-{}).",
                object_id, ACTUAL_CLIENT_ID_MAX
            ));
        }
        if self.entries.contains_key(&object_id) {
            return Err(format!("Object ID {} already exists.", object_id));
        }

        self.entries.insert(
            object_id,
            RegistryEntry {
                object: Box::new(object),
                client_id: client_id_assoc,
                version,
            },
        );
        Ok(())
    }

    /// Registers a server-created object (e.g., wl_registry, wl_callback).
    /// Assigns an ID from the server range (0xFF000000 - 0xFFFFFFFF).
    pub fn new_server_object<T: WaylandObject + 'static>(
        &mut self,
        client_id_assoc: u64, // Client this server object is primarily for (e.g. a wl_registry for a client)
        object: T,
        version: u32,
    ) -> Result<u32, String> {
        if self.next_server_object_id > SERVER_ID_MAX {
            // This check is mostly theoretical if we have 2^24 server objects.
            return Err("No more server object IDs available.".to_string());
        }

        let object_id = self.next_server_object_id;
        // Ensure the generated ID is not already taken (should not happen if logic is correct)
        // and advance to the next ID. This loop handles potential (but unlikely) collisions
        // if server IDs were ever manually inserted or if the range is small.
        // For a simple incrementing counter, collision is impossible if range is large enough.
        let mut current_id_to_try = object_id;
        loop {
            if current_id_to_try > SERVER_ID_MAX {
                 return Err("No more server object IDs available (overflow during search).".to_string());
            }
            if !self.entries.contains_key(&current_id_to_try) {
                self.next_server_object_id = current_id_to_try + 1; // Prepare for next call
                break;
            }
            current_id_to_try += 1;
        }

        self.entries.insert(
            current_id_to_try,
            RegistryEntry {
                object: Box::new(object),
                client_id: client_id_assoc,
                version,
            },
        );
        Ok(current_id_to_try)
    }

    /// Retrieves a reference to an object by its ID.
    pub fn get_object(&self, object_id: u32) -> Option<&dyn WaylandObject> {
        self.entries.get(&object_id).map(|entry| entry.object.as_ref())
    }

    /// Retrieves a mutable reference to an object by its ID.
    pub fn get_object_mut(&mut self, object_id: u32) -> Option<&mut dyn WaylandObject> {
        self.entries.get_mut(&object_id).map(|entry| entry.object.as_mut())
    }

    /// Retrieves a RegistryEntry by its ID.
    #[allow(dead_code)] // May be used later for getting version, client_id etc.
    pub fn get_entry(&self, object_id: u32) -> Option<&RegistryEntry> {
        self.entries.get(&object_id)
    }


    /// Removes an object from the registry and returns it.
    /// Returns an error if the object ID is not found.
    pub fn destroy_object(&mut self, object_id: u32) -> Result<Box<dyn WaylandObject>, String> {
        if object_id == 1 {
            return Err("Cannot destroy wl_display (object ID 1).".to_string());
        }
        self.entries
            .remove(&object_id)
            .map(|entry| entry.object)
            .ok_or_else(|| format!("Object ID {} not found for destruction.", object_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple test object
    #[derive(Debug)]
    struct TestObject {
        id: u32,
        data: String,
    }
    impl WaylandObject for TestObject {}

    impl TestObject {
        fn new(id: u32, data: &str) -> Self {
            TestObject { id, data: data.to_string() }
        }
    }

    // Another test object
    #[derive(Debug)]
    struct AnotherTestObject {
        name: String,
    }
    impl WaylandObject for AnotherTestObject {}


    #[test]
    fn test_registry_new() {
        let registry = ObjectRegistry::new();
        assert!(registry.get_object(1).is_some(), "wl_display (ID 1) should be pre-allocated");
        assert!(registry.get_object(1).unwrap().is::<WlDisplay>());
        assert_eq!(registry.next_server_object_id, SERVER_ID_MIN);
    }

    #[test]
    fn test_new_client_object_success() {
        let mut registry = ObjectRegistry::new();
        let obj_id = 100;
        let client_id = 1;
        let test_obj = TestObject::new(obj_id, "client_obj_100");

        let result = registry.new_object(client_id, obj_id, test_obj, 1);
        assert!(result.is_ok());

        let retrieved = registry.get_object(obj_id).expect("Object not found after creation");
        assert!(retrieved.is::<TestObject>());
        if let Some(specific_obj) = retrieved.downcast_ref::<TestObject>() {
            assert_eq!(specific_obj.id, obj_id);
            assert_eq!(specific_obj.data, "client_obj_100");
        } else {
            panic!("Could not downcast to TestObject");
        }

        let entry = registry.get_entry(obj_id).unwrap();
        assert_eq!(entry.client_id, client_id);
        assert_eq!(entry.version, 1);
    }

    #[test]
    fn test_new_client_object_id_collision() {
        let mut registry = ObjectRegistry::new();
        registry.new_object(1, 100, TestObject::new(100, "first"), 1).unwrap();
        let result = registry.new_object(1, 100, TestObject::new(100, "second"), 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Object ID 100 already exists.");
    }

    #[test]
    fn test_new_client_object_id_zero() {
        let mut registry = ObjectRegistry::new();
        let result = registry.new_object(1, 0, TestObject::new(0, "id_zero"), 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Object ID 0 is invalid (null object).");
    }

    #[test]
    fn test_new_client_object_id_out_of_range() {
        let mut registry = ObjectRegistry::new();
        let result = registry.new_object(1, SERVER_ID_MIN, TestObject::new(SERVER_ID_MIN, "server_range_obj"), 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of the allowed client range"));
    }

    #[test]
    fn test_new_server_object_success() {
        let mut registry = ObjectRegistry::new();
        let client_assoc_id = 1; // Associated with client 1
        let server_obj = AnotherTestObject { name: "my_server_object".to_string() };

        let result = registry.new_server_object(client_assoc_id, server_obj, 1);
        assert!(result.is_ok());
        let obj_id = result.unwrap();

        assert!(obj_id >= SERVER_ID_MIN && obj_id <= SERVER_ID_MAX);
        assert_eq!(registry.next_server_object_id, obj_id + 1);

        let retrieved = registry.get_object(obj_id).expect("Server object not found");
        assert!(retrieved.is::<AnotherTestObject>());
        if let Some(specific_obj) = retrieved.downcast_ref::<AnotherTestObject>() {
            assert_eq!(specific_obj.name, "my_server_object");
        } else {
            panic!("Could not downcast to AnotherTestObject");
        }

        let entry = registry.get_entry(obj_id).unwrap();
        assert_eq!(entry.client_id, client_assoc_id);
        assert_eq!(entry.version, 1);
    }

    #[test]
    fn test_new_multiple_server_objects_increment_id() {
        let mut registry = ObjectRegistry::new();
        let id1 = registry.new_server_object(1, AnotherTestObject { name: "obj1".into() }, 1).unwrap();
        let id2 = registry.new_server_object(1, AnotherTestObject { name: "obj2".into() }, 1).unwrap();
        assert_eq!(id1, SERVER_ID_MIN);
        assert_eq!(id2, SERVER_ID_MIN + 1);
        assert_eq!(registry.next_server_object_id, SERVER_ID_MIN + 2);
    }

    #[test]
    fn test_get_object_mut() {
        let mut registry = ObjectRegistry::new();
        let obj_id = 200;
        registry.new_object(1, obj_id, TestObject::new(obj_id, "mutable"), 1).unwrap();

        let retrieved_mut = registry.get_object_mut(obj_id).expect("Failed to get mutable object");
        if let Some(specific_obj_mut) = retrieved_mut.downcast_mut::<TestObject>() {
            specific_obj_mut.data = "modified".to_string();
        } else {
            panic!("Could not downcast mutable to TestObject");
        }

        let retrieved_immut = registry.get_object(obj_id).expect("Failed to get immutable object");
        if let Some(specific_obj_immut) = retrieved_immut.downcast_ref::<TestObject>() {
            assert_eq!(specific_obj_immut.data, "modified");
        } else {
            panic!("Could not downcast immutable to TestObject");
        }
    }

    #[test]
    fn test_destroy_object_success() {
        let mut registry = ObjectRegistry::new();
        let obj_id = 300;
        registry.new_object(1, obj_id, TestObject::new(obj_id, "to_destroy"), 1).unwrap();

        assert!(registry.get_object(obj_id).is_some());
        let destroy_result = registry.destroy_object(obj_id);
        assert!(destroy_result.is_ok());

        let destroyed_obj = destroy_result.unwrap();
        assert!(destroyed_obj.is::<TestObject>());
        if let Some(specific_obj) = destroyed_obj.downcast_ref::<TestObject>() {
            assert_eq!(specific_obj.data, "to_destroy");
        } else {
             // This path should not be taken if is::<TestObject>() passed.
             // Box::downcast needs the concrete type.
             // For Box<dyn Trait>, we can't directly downcast without knowing T.
             // The test `is::<TestObject>()` confirms its type.
             // To get data, we'd need to Box::downcast(destroyed_obj).unwrap().data
             // This requires destroyed_obj to be Box<TestObject> not Box<dyn WaylandObject>
             // This part of the test is more about checking if the object was removed.
        }

        assert!(registry.get_object(obj_id).is_none(), "Object should be gone after destruction");
        assert!(registry.entries.get(&obj_id).is_none());
    }

    #[test]
    fn test_destroy_object_not_found() {
        let mut registry = ObjectRegistry::new();
        let result = registry.destroy_object(999); // Non-existent ID
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Object ID 999 not found for destruction.");
    }

    #[test]
    fn test_destroy_wl_display_fails() {
        let mut registry = ObjectRegistry::new();
        let result = registry.destroy_object(1); // wl_display ID
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot destroy wl_display (object ID 1).");
        assert!(registry.get_object(1).is_some(), "wl_display should still exist.");
    }
}
