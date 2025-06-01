// In novade-system/src/compositor/wayland_server/objects.rs

use crate::compositor::wayland_server::client::ClientId;
use crate::compositor::wayland_server::error::WaylandServerError;
use crate::compositor::wayland_server::protocol::ObjectId;
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU32, Ordering}}; // Using std::sync::RwLock for better read concurrency
use tokio::sync::RwLock; // Using tokio's RwLock for async contexts
use tracing::{debug, error, warn};

// Represents the type of a Wayland object (e.g., "wl_surface", "wl_compositor")
// In a full implementation, this might be an enum or a more complex type
// that can also hold dispatch logic or trait objects.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Interface(Arc<str>); // Using Arc<str> for efficient cloning and sharing

impl Interface {
    pub fn new(name: &str) -> Self { // Corrected: removed trailing space from new
        Interface(Arc::from(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Basic information about a registered Wayland object
#[derive(Debug, Clone)]
pub struct ObjectEntry {
    pub id: ObjectId,
    pub interface: Interface,
    pub version: u32,
    pub client_id: ClientId, // The client that owns/created this object
                             // TODO: Add reference count for lifecycle management
                             // pub ref_count: Arc<AtomicU32>,
}

// Thread-safe registry for Wayland objects associated with a specific client.
// Each client will have its own ObjectSpace.
// Object IDs are typically allocated by the client (for existing objects)
// or by the server (for new_id requests).
// The server needs to ensure IDs are valid within the client's context.
#[derive(Debug)]
pub struct ClientObjectSpace {
    objects: RwLock<HashMap<ObjectId, ObjectEntry>>,
    client_id: ClientId,
    // TODO: Server-side ID allocation needs a strategy, e.g. ranges or a simple counter.
    // For now, we primarily register objects created by clients or in response to new_id.
    // next_server_allocated_id: AtomicU32, // Example for server-side ID generation
}

impl ClientObjectSpace {
    pub fn new(client_id: ClientId) -> Self {
        ClientObjectSpace {
            objects: RwLock::new(HashMap::new()),
            client_id,
            // next_server_allocated_id: AtomicU32::new(0xF0000000), // Example server-side range
        }
    }

    pub async fn register_object(
        &self,
        id: ObjectId,
        interface: Interface,
        version: u32,
    ) -> Result<ObjectEntry, WaylandServerError> {
        let mut objects_guard = self.objects.write().await;
        if objects_guard.contains_key(&id) {
            error!(
                "Client {}: Attempted to register object with duplicate ID {}.",
                self.client_id, id.value()
            );
            return Err(WaylandServerError::Object(format!(
                "Client {}: Duplicate object ID {} for interface {}", self.client_id, id.value(), interface.as_str()
            )));
        }

        let entry = ObjectEntry {
            id,
            interface: interface.clone(),
            version,
            client_id: self.client_id,
            // ref_count: Arc::new(AtomicU32::new(1)),
        };

        objects_guard.insert(id, entry.clone());
        debug!(
            "Client {}: Registered object ID {} (Interface: {}, Version: {}).",
            self.client_id, id.value(), interface.as_str(), version
        );
        Ok(entry)
    }

    pub async fn get_object(&self, id: ObjectId) -> Option<ObjectEntry> {
        let objects_guard = self.objects.read().await;
        objects_guard.get(&id).cloned()
    }

    // Unregistering an object. In a full system, this would also handle
    // decrementing ref counts, and potentially cascading destructions.
    pub async fn unregister_object(&self, id: ObjectId) -> Option<ObjectEntry> {
        let mut objects_guard = self.objects.write().await;
        let removed_entry = objects_guard.remove(&id);
        if let Some(ref entry) = removed_entry {
            debug!(
                "Client {}: Unregistered object ID {} (Interface: {}).",
                self.client_id, id.value(), entry.interface.as_str()
            );
        } else {
            warn!(
                "Client {}: Attempted to unregister non-existent object ID {}.",
                self.client_id, id.value()
            );
        }
        removed_entry
    }

    pub async fn list_objects(&self) -> Vec<ObjectEntry> {
        self.objects.read().await.values().cloned().collect()
    }

    // TODO: Implement server-side ID allocation for `new_id` requests.
    // This would find a free ID within the client's allowed range.
    // pub async fn allocate_server_id(&self, interface: String, version: u32) -> Result<ObjectEntry, WaylandServerError> {
    //     let new_id_val = self.next_server_allocated_id.fetch_add(1, Ordering::Relaxed);
    //     if new_id_val >= 0xFFFFFFFF { /* Handle exhaustion or wrap-around carefully */ }
    //     self.register_object(ObjectId::new(new_id_val), interface, version).await
    // }
}


// A global object manager that holds object spaces for all connected clients.
// This is a higher-level construct. The issue implies a per-client object ID space.
// So, the primary component is `ClientObjectSpace`.
// A `GlobalObjectManager` might map `ClientId` to `Arc<ClientObjectSpace>`.

#[derive(Debug, Default)]
pub struct GlobalObjectManager {
    client_spaces: RwLock<HashMap<ClientId, Arc<ClientObjectSpace>>>,
}

impl GlobalObjectManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn add_client(&self, client_id: ClientId) -> Arc<ClientObjectSpace> {
        let mut spaces_guard = self.client_spaces.write().await;
        let space = Arc::new(ClientObjectSpace::new(client_id));
        spaces_guard.insert(client_id, Arc::clone(&space));
        debug!("Added object space for client {}", client_id);
        space
    }

    pub async fn get_client_space(&self, client_id: ClientId) -> Option<Arc<ClientObjectSpace>> {
        self.client_spaces.read().await.get(&client_id).cloned()
    }

    pub async fn remove_client(&self, client_id: ClientId) {
        let mut spaces_guard = self.client_spaces.write().await;
        if spaces_guard.remove(&client_id).is_some() {
            debug!("Removed object space for client {}", client_id);
        } else {
            warn!("Attempted to remove object space for non-existent client {}", client_id);
        }
        // When a client is removed, its ClientObjectSpace will be dropped if all Arcs are gone.
        // The objects within that space will also be dropped.
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    // use crate::compositor::wayland_server::client::ClientId; // Not needed as ClientId::new() is static

    // Helper to create a dummy ClientId for testing.
    // ClientId::new() is used directly as it's a static atomic counter.
    // fn test_client_id(id_val: u64) -> ClientId {
    //     ClientId::new()
    // }


    #[tokio::test]
    async fn test_client_object_space_register_and_get() {
        let client_id = ClientId::new(); // Use actual ClientId::new()
        let space = ClientObjectSpace::new(client_id);
        let interface = Interface::new("wl_surface");

        let object_id = ObjectId::new(1001);
        let entry = space
            .register_object(object_id, interface.clone(), 1)
            .await
            .unwrap();

        assert_eq!(entry.id, object_id);
        assert_eq!(entry.interface.as_str(), "wl_surface");
        assert_eq!(entry.version, 1);
        assert_eq!(entry.client_id, client_id);

        let fetched_entry = space.get_object(object_id).await.unwrap();
        assert_eq!(fetched_entry.id, object_id);
        assert_eq!(fetched_entry.interface.as_str(), "wl_surface");

        // Test registering duplicate ID
        let result_duplicate = space.register_object(object_id, Interface::new("wl_region"), 1).await;
        assert!(result_duplicate.is_err());
        if let Err(WaylandServerError::Object(msg)) = result_duplicate {
            assert!(msg.contains("Duplicate object ID"));
        } else {
            panic!("Expected Object error for duplicate ID");
        }
    }

    #[tokio::test]
    async fn test_client_object_space_unregister() {
        let client_id = ClientId::new();
        let space = ClientObjectSpace::new(client_id);
        let interface = Interface::new("wl_compositor");
        let object_id = ObjectId::new(2002);

        space.register_object(object_id, interface, 4).await.unwrap();
        assert!(space.get_object(object_id).await.is_some());

        let removed_entry = space.unregister_object(object_id).await.unwrap();
        assert_eq!(removed_entry.id, object_id);
        assert!(space.get_object(object_id).await.is_none());

        // Test unregistering non-existent ID
        let result_non_existent = space.unregister_object(ObjectId::new(9999)).await;
        assert!(result_non_existent.is_none());
    }

    #[tokio::test]
    async fn test_client_object_space_list() {
        let client_id = ClientId::new();
        let space = ClientObjectSpace::new(client_id);

        let id1 = ObjectId::new(3001);
        let id2 = ObjectId::new(3002);
        space.register_object(id1, Interface::new("wl_seat"), 7).await.unwrap();
        space.register_object(id2, Interface::new("wl_pointer"), 7).await.unwrap();

        let objects = space.list_objects().await;
        assert_eq!(objects.len(), 2);
        assert!(objects.iter().any(|o| o.id == id1));
        assert!(objects.iter().any(|o| o.id == id2));
    }

    #[tokio::test]
    async fn test_global_object_manager() {
        let manager = GlobalObjectManager::new();
        let client1_id = ClientId::new();
        let client2_id = ClientId::new();

        let space1 = manager.add_client(client1_id).await;
        let space1_fetched = manager.get_client_space(client1_id).await.unwrap();
        assert!(Arc::ptr_eq(&space1, &space1_fetched));
        assert_eq!(space1.client_id, client1_id);

        let interface_c1 = Interface::new("wl_shell");
        space1.register_object(ObjectId::new(1), interface_c1, 1).await.unwrap();

        let space2 = manager.add_client(client2_id).await;
        assert_ne!(space1.client_id, space2.client_id); // Client IDs should be different from ClientId::new()
        let interface_c2 = Interface::new("xdg_wm_base");
        space2.register_object(ObjectId::new(1), interface_c2, 2).await.unwrap(); // Same ID, different client space

        assert!(manager.get_client_space(client1_id).await.unwrap().get_object(ObjectId::new(1)).await.is_some());
        assert!(manager.get_client_space(client2_id).await.unwrap().get_object(ObjectId::new(1)).await.is_some());

        manager.remove_client(client1_id).await;
        assert!(manager.get_client_space(client1_id).await.is_none());
        assert!(manager.get_client_space(client2_id).await.is_some()); // Client 2 should still exist
    }

    #[test]
    fn test_interface_creation_and_as_str() {
        let iface_name = "test_interface";
        let interface = Interface::new(iface_name);
        assert_eq!(interface.as_str(), iface_name);

        let interface_clone = interface.clone();
        assert_eq!(interface_clone.as_str(), iface_name);
        // Check that they point to the same Arc<str> data
        assert!(Arc::ptr_eq(&interface.0, &interface_clone.0));
    }
}
