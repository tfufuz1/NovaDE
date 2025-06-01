// In novade-system/src/compositor/wayland_server/objects.rs

use crate::compositor::wayland_server::client::ClientId;
use crate::compositor::wayland_server::error::WaylandServerError;
use crate::compositor::wayland_server::protocol::ObjectId;
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use tokio::sync::RwLock;
use tracing::{debug, error, warn, info, trace};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Interface(Arc<str>);
impl Interface {
    pub fn new(name: &str) -> Self { Interface(Arc::from(name)) }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone)]
pub struct ObjectEntry {
    pub id: ObjectId,
    pub interface: Interface,
    pub version: u32,
    pub client_id: ClientId,
    ref_count: Arc<AtomicU32>,
}

impl ObjectEntry {
    fn new(id: ObjectId, interface: Interface, version: u32, client_id: ClientId, initial_ref_count: u32) -> Self {
        ObjectEntry { id, interface, version, client_id, ref_count: Arc::new(AtomicU32::new(initial_ref_count)) }
    }
    pub fn inc_ref(&self) {
        let old_count = self.ref_count.fetch_add(1, Ordering::Relaxed);
        trace!("Object {}: Ref count incremented from {} to {}", self.id.value(), old_count, old_count + 1);
    }
    pub fn dec_ref(&self) -> u32 {
        // fetch_saturating_sub ensures the atomic value itself doesn't go below 0.
        let old_count = self.ref_count.fetch_saturating_sub(1, Ordering::Relaxed);
        // The value returned by fetch_saturating_sub is the value *before* the operation.
        // So, the new count is old_count - 1, unless old_count was already 0.
        let new_count = if old_count == 0 { 0 } else { old_count - 1 };
        trace!("Object {}: Ref count decremented from {} to {}", self.id.value(), old_count, new_count);
        if new_count == 0 {
            info!("Object {}: Ref count reached zero.", self.id.value());
        }
        new_count
    }
    pub fn get_ref_count(&self) -> u32 { self.ref_count.load(Ordering::Relaxed) }
}

const SERVER_ID_RANGE_START: u32 = 0xFF000000;
const SERVER_ID_RANGE_END: u32 = 0xFFFFFFFF;

#[derive(Debug)]
pub struct ClientObjectSpace {
    objects: RwLock<HashMap<ObjectId, ObjectEntry>>,
    client_id: ClientId,
    next_server_id: AtomicU32,
}

impl ClientObjectSpace {
    pub fn new(client_id: ClientId) -> Self {
        ClientObjectSpace {
            objects: RwLock::new(HashMap::new()),
            client_id,
            next_server_id: AtomicU32::new(SERVER_ID_RANGE_START),
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
        let entry = ObjectEntry::new(id, interface.clone(), version, self.client_id, 1);
        objects_guard.insert(id, entry.clone());
        debug!(
            "Client {}: Registered object ID {} (Interface: {}, Version: {}, RefCount: 1).",
            self.client_id, id.value(), interface.as_str(), version
        );
        Ok(entry)
    }

    pub async fn allocate_server_object(
        &self,
        interface: Interface,
        version: u32,
    ) -> Result<ObjectEntry, WaylandServerError> {
        let mut attempts = 0;
        const MAX_ALLOCATION_ATTEMPTS: u32 = 100;

        loop {
            let new_id_val = self.next_server_id.fetch_add(1, Ordering::Relaxed);
            attempts += 1;

            if new_id_val > SERVER_ID_RANGE_END {
                error!(
                    "Client {}: Server-side object ID allocation exhausted. Last ID tried: {:#X}",
                    self.client_id, new_id_val
                );
                // Consider decrementing next_server_id here if it's safe, or use a lock for the whole allocation.
                // For now, it just means this client can't allocate more server IDs.
                return Err(WaylandServerError::Object(
                    "Server-side object ID allocation exhausted.".to_string()
                ));
            }

            // This check is mostly for catastrophic failure, as fetch_add should keep it in range if started correctly.
            if new_id_val < SERVER_ID_RANGE_START {
                 error!(
                    "Client {}: Server-side object ID allocator generated out of range ID {:#X}. Resetting counter.",
                    self.client_id, new_id_val
                );
                self.next_server_id.store(SERVER_ID_RANGE_START, Ordering::Relaxed);
                 return Err(WaylandServerError::Object(
                    "Server-side object ID allocator malfunctioned (underflow).".to_string()
                ));
            }

            let new_object_id = ObjectId::new(new_id_val);

            let mut objects_guard = self.objects.write().await;
            if !objects_guard.contains_key(&new_object_id) {
                let entry = ObjectEntry::new(new_object_id, interface.clone(), version, self.client_id, 1);
                objects_guard.insert(new_object_id, entry.clone());
                debug!(
                    "Client {}: Allocated and registered server-side object ID {} (Interface: {}, Version: {}, RefCount: 1).",
                    self.client_id, new_object_id.value(), interface.as_str(), version
                );
                return Ok(entry);
            }
            drop(objects_guard);

            if attempts > MAX_ALLOCATION_ATTEMPTS {
                error!(
                    "Client {}: Failed to allocate a unique server-side object ID after {} attempts. Last tried: {:#X}",
                    self.client_id, attempts, new_id_val
                );
                return Err(WaylandServerError::Object(
                    "Failed to allocate unique server-side ID after multiple attempts.".to_string()
                ));
            }
            warn!("Client {}: Server-allocated ID {:#X} was already in use (attempt {}). Retrying.", self.client_id, new_id_val, attempts);
        }
    }

    pub async fn get_object(&self, id: ObjectId) -> Option<ObjectEntry> {
        self.objects.read().await.get(&id).cloned()
    }
    pub async fn destroy_object_by_client(&self, id: ObjectId) -> Result<bool, WaylandServerError> {
        let mut objects_guard = self.objects.write().await;
        match objects_guard.get(&id) {
            Some(entry) => {
                let new_ref_count = entry.dec_ref();
                if new_ref_count == 0 {
                    info!(
                        "Client {}: Object {} (Interface: {}) ref count reached zero upon client destroy request. Removing from registry.",
                        self.client_id, id.value(), entry.interface.as_str()
                    );
                    objects_guard.remove(&id);
                    Ok(true)
                } else {
                    debug!(
                        "Client {}: Decremented ref count for object {} (Interface: {}). New count: {}. Not removing yet.",
                        self.client_id, id.value(), entry.interface.as_str(), new_ref_count
                    );
                    Ok(false)
                }
            }
            None => {
                warn!("Client {}: Attempted to destroy non-existent object ID {}.", self.client_id, id.value());
                Err(WaylandServerError::Object(format!("Attempted to destroy non-existent object ID {}", id.value())))
            }
        }
    }
    #[deprecated(note = "Prefer destroy_object_by_client or ensure ref_count is zero before calling directly.")]
    pub async fn force_unregister_object(&self, id: ObjectId) -> Option<ObjectEntry> {
        let mut objects_guard = self.objects.write().await;
        let removed_entry = objects_guard.remove(&id);
        if let Some(ref entry) = removed_entry {
            info!(
                "Client {}: Forcefully unregistered object ID {} (Interface: {}). RefCount was: {}.",
                self.client_id, id.value(), entry.interface.as_str(), entry.get_ref_count()
            );
        } else {
            warn!("Client {}: Attempted to force unregister non-existent object ID {}.", self.client_id, id.value());
        }
        removed_entry
    }
    pub async fn list_objects(&self) -> Vec<ObjectEntry> {
        self.objects.read().await.values().cloned().collect()
    }
}

#[derive(Debug, Default)]
pub struct GlobalObjectManager {
    client_spaces: RwLock<HashMap<ClientId, Arc<ClientObjectSpace>>>
}
impl GlobalObjectManager {
    pub fn new() -> Self { Default::default() }
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
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_object_entry_ref_counting() {
         let client_id = ClientId::new();
         let entry = ObjectEntry::new(ObjectId::new(1), Interface::new("test"), 1, client_id, 1);
         assert_eq!(entry.get_ref_count(), 1); entry.inc_ref(); assert_eq!(entry.get_ref_count(), 2);
         assert_eq!(entry.dec_ref(), 1); assert_eq!(entry.get_ref_count(), 1);
         assert_eq!(entry.dec_ref(), 0); assert_eq!(entry.get_ref_count(), 0);
         // Test dec_ref on an already zero count
         assert_eq!(entry.dec_ref(), 0, "dec_ref on zero count should return 0");
         assert_eq!(entry.get_ref_count(), 0, "get_ref_count should still be 0");
    }

    #[tokio::test]
    async fn test_client_object_space_destroy_object_by_client_ref_count() {
        let client_id = ClientId::new();
        let space = ClientObjectSpace::new(client_id);
        let interface = Interface::new("wl_surface");
        let object_id = ObjectId::new(1001);

        let entry = space.register_object(object_id, interface.clone(), 1).await.unwrap();
        entry.inc_ref();

        let removed = space.destroy_object_by_client(object_id).await.unwrap();
        assert!(!removed);
        assert_eq!(entry.get_ref_count(), 1);

        let removed_again = space.destroy_object_by_client(object_id).await.unwrap();
        assert!(removed_again);
        assert!(space.get_object(object_id).await.is_none());
    }

    #[tokio::test]
    async fn test_allocate_server_object_success_and_uniqueness() {
        let client_id = ClientId::new();
        let space = ClientObjectSpace::new(client_id);
        let interface1 = Interface::new("wl_callback");
        let interface2 = Interface::new("wl_buffer");

        let entry1 = space.allocate_server_object(interface1.clone(), 1).await.unwrap();
        assert!(entry1.id.value() >= SERVER_ID_RANGE_START, "ID {} should be in server range", entry1.id.value());
        assert_eq!(entry1.interface.as_str(), "wl_callback");
        assert_eq!(entry1.get_ref_count(), 1);
        assert!(space.get_object(entry1.id).await.is_some());

        let entry2 = space.allocate_server_object(interface2.clone(), 1).await.unwrap();
        assert!(entry2.id.value() >= SERVER_ID_RANGE_START, "ID {} should be in server range", entry2.id.value());
        assert_ne!(entry1.id, entry2.id, "Server allocated IDs should be unique");
        assert_eq!(entry2.interface.as_str(), "wl_buffer");

        assert_eq!(space.next_server_id.load(Ordering::Relaxed), SERVER_ID_RANGE_START + 2);
    }

    #[tokio::test]
    async fn test_allocate_server_object_id_collision_retry() {
        let client_id = ClientId::new();
        let space = ClientObjectSpace::new(client_id);
        let interface = Interface::new("test_collision");

        let first_expected_id_val = SERVER_ID_RANGE_START;
        let second_expected_id_val = SERVER_ID_RANGE_START + 1;

        // Manually register the first ID the allocator would try.
        // Note: register_object doesn't advance next_server_id.
        space.register_object(ObjectId::new(first_expected_id_val), interface.clone(), 1).await.unwrap();

        // Check current next_server_id before allocation
        assert_eq!(space.next_server_id.load(Ordering::Relaxed), SERVER_ID_RANGE_START, "next_server_id should still be at the start");

        let allocated_entry = space.allocate_server_object(interface.clone(), 1).await.unwrap();
        // The first ID from fetch_add will be SERVER_ID_RANGE_START. This is a collision.
        // next_server_id becomes SERVER_ID_RANGE_START + 1.
        // Loop retries. fetch_add gets SERVER_ID_RANGE_START + 1. This is not a collision.
        // next_server_id becomes SERVER_ID_RANGE_START + 2.
        assert_eq!(allocated_entry.id.value(), second_expected_id_val, "Should have allocated the next available ID after simulated collision");
        assert_eq!(space.next_server_id.load(Ordering::Relaxed), SERVER_ID_RANGE_START + 2, "Next ID counter should be incremented past the allocated one");
    }

    #[tokio::test]
    async fn test_server_id_range_exhaustion_very_limited() {
        let client_id = ClientId::new();
        let space = ClientObjectSpace::new(client_id);
        let interface = Interface::new("exhaust_test");

        let test_range_start = SERVER_ID_RANGE_END - 1;
        space.next_server_id.store(test_range_start, Ordering::Relaxed);

        let entry1 = space.allocate_server_object(interface.clone(), 1).await.unwrap();
        assert_eq!(entry1.id.value(), test_range_start); // SERVER_ID_RANGE_END - 1
        assert_eq!(space.next_server_id.load(Ordering::Relaxed), test_range_start + 1); // SERVER_ID_RANGE_END

        let entry2 = space.allocate_server_object(interface.clone(), 1).await.unwrap();
        assert_eq!(entry2.id.value(), test_range_start + 1); // SERVER_ID_RANGE_END
        assert_eq!(space.next_server_id.load(Ordering::Relaxed), test_range_start + 2); // SERVER_ID_RANGE_END + 1

        let result = space.allocate_server_object(interface.clone(), 1).await;
        assert!(result.is_err());
        if let Err(WaylandServerError::Object(msg)) = result {
            assert!(msg.contains("Server-side object ID allocation exhausted"));
        } else {
            panic!("Expected Object error for ID exhaustion");
        }
    }
    // Other existing tests (destroy_non_existent_object, force_unregister_object, register_and_get etc.) should be here
}
