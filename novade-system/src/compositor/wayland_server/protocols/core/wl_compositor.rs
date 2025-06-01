// In novade-system/src/compositor/wayland_server/protocols/core/wl_compositor.rs
use crate::compositor::wayland_server::client::ClientId;
use crate::compositor::wayland_server::error::WaylandServerError;
use crate::compositor::wayland_server::objects::{ClientObjectSpace, Interface, ObjectId};
use crate::compositor::wayland_server::protocol::ArgumentValue;
use crate::compositor::wayland_server::protocols::core::wl_surface::{wl_surface_interface, WL_SURFACE_VERSION};
use std::sync::Arc;
use tracing::{debug, info, error};

pub fn wl_compositor_interface() -> Interface { Interface::new("wl_compositor") }
pub const WL_COMPOSITOR_VERSION: u32 = 4;

// Request Opcodes
pub const REQ_CREATE_SURFACE_OPCODE: u16 = 0;
// pub const REQ_CREATE_REGION_OPCODE: u16 = 1; // Not implemented in this step

// Handler for wl_compositor.create_surface (opcode 0)
// Arguments: id (new_id<wl_surface>)
pub async fn handle_create_surface(
    client_id: ClientId,
    _compositor_object_id: ObjectId, // The wl_compositor object that received the request
    compositor_version: u32,       // Version of the wl_compositor object
    args: Vec<ArgumentValue>,
    client_space: Arc<ClientObjectSpace>,
) -> Result<(), WaylandServerError> {
    info!("Client {}: Handling wl_compositor.create_surface", client_id);

    let new_surface_id_val = match args.get(0) {
        Some(ArgumentValue::NewId(id)) => id.value(),
        _ => return Err(WaylandServerError::Protocol("wl_compositor.create_surface: Expected NewId for the new surface (arg 0)".to_string())),
    };

    let new_surface_id = ObjectId::new(new_surface_id_val);

    // Version negotiation for the new surface:
    // The client requests a version when it creates the new_id placeholder.
    // The server should bind the new object using min(client_requested_version, server_advertised_max_version_for_interface).
    // Here, `compositor_version` is the version of the wl_compositor object itself, not the client's request for wl_surface.
    // This part is simplified: a full implementation needs the client's requested version for the new_id.
    // We use WL_SURFACE_VERSION as the effective version to bind, assuming client requested a compatible version.
    let surface_bind_version = WL_SURFACE_VERSION; // Simplified: In reality, min(client_req_version_for_new_id, WL_SURFACE_VERSION)


    if client_space.get_object(new_surface_id).await.is_some() {
        error!("Client {}: Attempted to create surface with already existing object ID {}", client_id, new_surface_id.value());
        // TODO: Send wl_display.error (INVALID_ID or similar)
        return Err(WaylandServerError::Protocol(format!(
            "Client attempted to create surface with existing ID {}", new_surface_id.value()
        )));
    }

    client_space.register_object(
        new_surface_id,
        wl_surface_interface(),
        surface_bind_version,
    ).await?;

    info!(
        "Client {}: Created new wl_surface with ID {} (bound at version {}).",
        client_id, new_surface_id.value(), surface_bind_version
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::wayland_server::protocol::NewId;
    // Corrected: ClientId is in super::super::client, not directly here.
    // For tests, it's fine to use ClientId::new() which is globally available.
    use crate::compositor::wayland_server::client::ClientId;


    #[tokio::test]
    async fn test_handle_create_surface_success() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let compositor_id = ObjectId::new(2);
        let new_surface_client_chosen_id = 100u32;

        let args = vec![
            ArgumentValue::NewId(NewId::new(new_surface_client_chosen_id)),
        ];

        let result = handle_create_surface(
            client_id,
            compositor_id,
            WL_COMPOSITOR_VERSION,
            args,
            Arc::clone(&client_space)
        ).await;

        assert!(result.is_ok(), "handle_create_surface failed: {:?}", result.err());

        let surface_object = client_space.get_object(ObjectId::new(new_surface_client_chosen_id)).await;
        assert!(surface_object.is_some(), "wl_surface object was not registered");
        let entry = surface_object.unwrap();
        assert_eq!(entry.interface.as_str(), "wl_surface");
        // Test against the simplified version logic in the handler
        assert_eq!(entry.version, WL_SURFACE_VERSION);
    }

    #[tokio::test]
    async fn test_handle_create_surface_id_already_exists() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let compositor_id = ObjectId::new(2);
        let existing_surface_id_val = 101u32;

        client_space.register_object(
            ObjectId::new(existing_surface_id_val),
            wl_surface_interface(),
            1
        ).await.unwrap();

        let args = vec![
            ArgumentValue::NewId(NewId::new(existing_surface_id_val)),
        ];

        let result = handle_create_surface(
            client_id,
            compositor_id,
            WL_COMPOSITOR_VERSION,
            args,
            Arc::clone(&client_space)
        ).await;

        assert!(result.is_err(), "handle_create_surface should fail if ID already exists");
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("create surface with existing ID"));
        } else {
            panic!("Expected Protocol error for existing ID, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_handle_create_surface_incorrect_arg_type() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let compositor_id = ObjectId::new(2);

        let args = vec![
            ArgumentValue::Uint(102),
        ];

        let result = handle_create_surface(
            client_id,
            compositor_id,
            WL_COMPOSITOR_VERSION,
            args,
            Arc::clone(&client_space)
        ).await;

        assert!(result.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("Expected NewId for the new surface"));
        } else {
            panic!("Expected Protocol error for incorrect argument type, got {:?}", result);
        }
    }
}
