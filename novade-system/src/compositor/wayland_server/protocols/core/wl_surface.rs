// In novade-system/src/compositor/wayland_server/protocols/core/wl_surface.rs
use crate::compositor::wayland_server::objects::Interface;

pub fn wl_surface_interface() -> Interface { Interface::new("wl_surface") }
pub const WL_SURFACE_VERSION: u32 = 4; // Example max version server might support

// Request Opcodes for wl_surface
pub const REQ_DESTROY_OPCODE: u16 = 0;
pub const REQ_ATTACH_OPCODE: u16 = 1;
pub const REQ_DAMAGE_OPCODE: u16 = 2;
pub const REQ_FRAME_OPCODE: u16 = 3;
// ... other opcodes ...
pub const REQ_COMMIT_OPCODE: u16 = 6;


use crate::compositor::wayland_server::client::ClientId;
use crate::compositor::wayland_server::error::WaylandServerError;
use crate::compositor::wayland_server::objects::{ClientObjectSpace, ObjectId, ObjectRef};
use crate::compositor::wayland_server::protocol::ArgumentValue;
use std::sync::Arc;
use tracing::{info, warn};

// wl_surface.destroy (opcode 0)
// No arguments.
pub async fn handle_destroy(
    client_id: ClientId,
    surface_object_id: ObjectId,
    _surface_version: u32, // Version of the wl_surface object itself
    _args: Vec<ArgumentValue>, // Should be empty for destroy
    client_space: Arc<ClientObjectSpace>,
) -> Result<(), WaylandServerError> {
    info!("Client {}: Handling wl_surface.destroy for surface ID {}", client_id, surface_object_id.value());

    // Attempt to destroy the object. destroy_object_by_client handles ref counting.
    // If the object is not found, it's a client error (destroying a non-existent object).
    // If ref_count reaches zero, it will be removed from the map.
    match client_space.destroy_object_by_client(surface_object_id).await {
        Ok(removed) => {
            if removed {
                info!("Client {}: wl_surface object {} successfully destroyed and removed.", client_id, surface_object_id.value());
            } else {
                info!("Client {}: wl_surface object {} ref_count decremented. Not yet removed.", client_id, surface_object_id.value());
            }
            // In Wayland, destroying an object usually means the client will not use it anymore.
            // If it had child objects (e.g., frame callbacks), those should also be cleaned up.
            // This simple handler doesn't manage explicit child objects yet.
        }
        Err(e) => {
            // This typically means the object_id didn't exist, which is a protocol error.
            warn!("Client {}: Failed to destroy wl_surface object {}: {}. This might be a client error (e.g., double destroy).", client_id, surface_object_id.value(), e);
            // Depending on strictness, this could return Err(e) to disconnect the client.
            // For now, log and continue, as the object might have already been destroyed.
            // However, the Wayland spec often implies client error for such cases.
            // Let's return the error to be more conformant.
            return Err(e);
        }
    }
    Ok(())
}

// wl_surface.attach (opcode 1)
// Args: buffer (object, nullable), x (int), y (int)
pub async fn handle_attach(
    client_id: ClientId,
    surface_object_id: ObjectId,
    _surface_version: u32,
    args: Vec<ArgumentValue>,
    _client_space: Arc<ClientObjectSpace>, // Marked as unused for stub
) -> Result<(), WaylandServerError> {
    info!(
        "Client {}: Handling wl_surface.attach for surface ID {} with args {:?} - STUB",
        client_id,
        surface_object_id.value(),
        args
    );
    // TODO: Actual implementation:
    // 1. Validate args:
    //    - args[0]: Option<ObjectId> (buffer) - if Some, check if it's a valid wl_buffer object.
    //    - args[1]: i32 (x)
    //    - args[2]: i32 (y)
    // 2. Associate buffer with surface state (pending state).
    // 3. Store x, y offsets.
    Ok(())
}

// wl_surface.damage (opcode 2)
// Args: x (int), y (int), width (int), height (int)
pub async fn handle_damage(
    client_id: ClientId,
    surface_object_id: ObjectId,
    _surface_version: u32,
    args: Vec<ArgumentValue>,
    _client_space: Arc<ClientObjectSpace>, // Marked as unused for stub
) -> Result<(), WaylandServerError> {
    info!(
        "Client {}: Handling wl_surface.damage for surface ID {} with args {:?} - STUB",
        client_id,
        surface_object_id.value(),
        args
    );
    // TODO: Actual implementation:
    // 1. Validate args: x, y, width, height (all i32). Ensure width/height are positive.
    // 2. Add damage rectangle to surface state (pending state).
    Ok(())
}

// wl_surface.commit (opcode 6)
// No arguments.
pub async fn handle_commit(
    client_id: ClientId,
    surface_object_id: ObjectId,
    _surface_version: u32,
    _args: Vec<ArgumentValue>, // Should be empty
    _client_space: Arc<ClientObjectSpace>, // Marked as unused for stub
) -> Result<(), WaylandServerError> {
    info!(
        "Client {}: Handling wl_surface.commit for surface ID {} - STUB",
        client_id,
        surface_object_id.value()
    );
    // TODO: Actual implementation:
    // 1. Atomically apply all pending state (attach, damage, frame, etc.) to current state.
    // 2. If a frame callback was requested, send it now or schedule it.
    // 3. Trigger rendering/composition updates.
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::wayland_server::client::Client;
    use crate::compositor::wayland_server::objects::{GlobalObjectId, ObjectType, ObjectId}; // Added ObjectId
    use crate::compositor::wayland_server::test_utils::create_test_client_object_space;

    #[tokio::test]
    async fn test_handle_surface_destroy_success() {
        let client_id = ClientId::new(1);
        let (client_space, _client_receiver) = create_test_client_object_space(client_id);

        let surface_id = ObjectId::new(101); // Example ID for the surface
        let surface_global_id = GlobalObjectId::new(surface_id.value()); // Mock global ID

        // Register a wl_surface object
        let surface_object = ObjectRef::new(
            surface_id,
            surface_global_id,
            "wl_surface".to_string(),
            WL_SURFACE_VERSION,
            ObjectType::Regular, // Assuming it's a regular object
        );
        client_space.register_object(client_id, surface_object.clone(), None).await.unwrap();

        // Check initial ref_count (should be 1 from registration)
        let initial_ref_count = client_space.get_object_ref_count(surface_id).await.unwrap();
        assert_eq!(initial_ref_count, 1, "Initial ref_count should be 1");

        // Call handle_destroy
        let result = handle_destroy(
            client_id,
            surface_id,
            WL_SURFACE_VERSION,
            vec![],
            Arc::clone(&client_space),
        ).await;

        assert!(result.is_ok(), "handle_destroy failed: {:?}", result.err());

        // Verify object is removed (since initial ref_count was 1)
        assert!(client_space.get_object(surface_id).await.is_err(), "Object should be removed after destroy");

        // Test decrementing ref_count
        let surface_id_2 = ObjectId::new(102);
        let surface_global_id_2 = GlobalObjectId::new(surface_id_2.value());
        let surface_object_2 = ObjectRef::new(
            surface_id_2,
            surface_global_id_2,
            "wl_surface".to_string(),
            WL_SURFACE_VERSION,
            ObjectType::Regular,
        );
        client_space.register_object(client_id, surface_object_2.clone(), None).await.unwrap();
        client_space.increment_object_ref_count(surface_id_2).await.unwrap(); // Increment ref_count to 2

        let initial_ref_count_2 = client_space.get_object_ref_count(surface_id_2).await.unwrap();
        assert_eq!(initial_ref_count_2, 2, "Initial ref_count for surface_id_2 should be 2");

        let result_2 = handle_destroy(
            client_id,
            surface_id_2,
            WL_SURFACE_VERSION,
            vec![],
            Arc::clone(&client_space),
        ).await;
        assert!(result_2.is_ok(), "handle_destroy for surface_id_2 failed: {:?}", result_2.err());

        let final_ref_count_2 = client_space.get_object_ref_count(surface_id_2).await.unwrap();
        assert_eq!(final_ref_count_2, 1, "Ref_count for surface_id_2 should be 1 after destroy");
        assert!(client_space.get_object(surface_id_2).await.is_ok(), "Object surface_id_2 should still exist");
    }

    #[tokio::test]
    async fn test_handle_surface_destroy_non_existent() {
        let client_id = ClientId::new(1);
        let (client_space, _client_receiver) = create_test_client_object_space(client_id);

        let non_existent_surface_id = ObjectId::new(999);

        let result = handle_destroy(
            client_id,
            non_existent_surface_id,
            WL_SURFACE_VERSION,
            vec![],
            Arc::clone(&client_space),
        ).await;

        assert!(result.is_err(), "handle_destroy should have failed for non-existent object");
        match result.err().unwrap() {
            WaylandServerError::Object(msg) => {
                // This is the expected error from destroy_object_by_client if object not found
                assert!(msg.contains(&format!("Object with ID {} not found for client", non_existent_surface_id.value())));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_handle_surface_attach_stub_runs() {
        let client_id = ClientId::new(1);
        let (client_space, _cr) = create_test_client_object_space(client_id);
        let surface_id = ObjectId::new(101);
        // Args: buffer (nullable object), x (int), y (int)
        let args = vec![
            ArgumentValue::Object(ObjectId::new(0)), // Null buffer
            ArgumentValue::Int(10),
            ArgumentValue::Int(20),
        ];

        let result = handle_attach(
            client_id,
            surface_id,
            WL_SURFACE_VERSION,
            args,
            client_space,
        ).await;
        assert!(result.is_ok(), "handle_attach stub failed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_handle_surface_damage_stub_runs() {
        let client_id = ClientId::new(1);
        let (client_space, _cr) = create_test_client_object_space(client_id);
        let surface_id = ObjectId::new(101);
        // Args: x, y, width, height (all int)
        let args = vec![
            ArgumentValue::Int(0),
            ArgumentValue::Int(0),
            ArgumentValue::Int(100),
            ArgumentValue::Int(100),
        ];

        let result = handle_damage(
            client_id,
            surface_id,
            WL_SURFACE_VERSION,
            args,
            client_space,
        ).await;
        assert!(result.is_ok(), "handle_damage stub failed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_handle_surface_commit_stub_runs() {
        let client_id = ClientId::new(1);
        let (client_space, _cr) = create_test_client_object_space(client_id);
        let surface_id = ObjectId::new(101);
        let args = vec![]; // No arguments for commit

        let result = handle_commit(
            client_id,
            surface_id,
            WL_SURFACE_VERSION,
            args,
            client_space,
        ).await;
        assert!(result.is_ok(), "handle_commit stub failed: {:?}", result.err());
    }
}
