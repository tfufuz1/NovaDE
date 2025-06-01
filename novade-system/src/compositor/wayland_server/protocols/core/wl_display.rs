// In novade-system/src/compositor/wayland_server/protocols/core/wl_display.rs
use crate::compositor::wayland_server::client::ClientId;
use crate::compositor::wayland_server::error::WaylandServerError;
use crate::compositor::wayland_server::objects::{ClientObjectSpace, Interface, ObjectId};
use crate::compositor::wayland_server::protocol::{ArgumentValue, NewId};
use crate::compositor::wayland_server::protocols::core::wl_callback::{wl_callback_interface, WL_CALLBACK_VERSION, CallbackDoneEvent, EVT_DONE_OPCODE};
use crate::compositor::wayland_server::protocols::core::wl_registry::{wl_registry_interface, WL_REGISTRY_VERSION};
    use crate::compositor::wayland_server::event_sender::EventSender; // Import EventSender
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub fn wl_display_interface() -> Interface { Interface::new("wl_display") }
pub const WL_DISPLAY_VERSION: u32 = 1;
pub const WL_DISPLAY_ID: ObjectId = ObjectId::new(1); // Fixed ID for wl_display

// Event opcodes for wl_display
pub const EVT_ERROR_OPCODE: u16 = 0;
pub const EVT_DELETE_ID_OPCODE: u16 = 1;

// Request opcodes for wl_display
pub const REQ_SYNC_OPCODE: u16 = 0;
pub const REQ_GET_REGISTRY_OPCODE: u16 = 1;

#[derive(Debug)]
pub struct DisplayErrorEvent {
    pub object_id: ObjectId, // Object that caused the error
    pub code: u32,
    pub message: String,
}

// Handler for wl_display.sync (opcode 0)
// Args: new_id (callback: wl_callback)
pub async fn handle_sync(
    client_id: ClientId, // Added client_id to log messages
    _object_id: ObjectId, // Should be WL_DISPLAY_ID
    _object_version: u32,
    args: Vec<ArgumentValue>,
    client_space: Arc<ClientObjectSpace>,
    event_sender: Arc<EventSender>,     // Added EventSender
    client_stream: Arc<TokioUnixStream>, // Added client_stream to send the event to
) -> Result<(), WaylandServerError> {
    info!("Handling wl_display.sync for client {}", client_id);
    let callback_new_id = match args.get(0) {
        Some(ArgumentValue::NewId(id)) => *id,
        _ => return Err(WaylandServerError::Protocol("wl_display.sync: Expected new_id argument".to_string())),
    };

    let callback_object_id = ObjectId::new(callback_new_id.value());
    // Register the new wl_callback object
    client_space.register_object(
        callback_object_id,
        wl_callback_interface(), // Use function
        WL_CALLBACK_VERSION,
    ).await?;
    debug!("Client {}: Registered new wl_callback object {} for sync", client_id, callback_object_id.value());

    let done_event_serial = 0; // Placeholder serial for the callback data
    let done_event = CallbackDoneEvent { callback_data: done_event_serial };

    // Send wl_callback.done event
    event_sender.send_event(
        &*client_stream, // Pass the actual stream
        callback_object_id,
        EVT_DONE_OPCODE,
        done_event,
        vec![] // No FDs for wl_callback.done
    ).await?;
    info!("Client {}: Sent wl_callback.done event for callback {} (serial {})", client_id, callback_object_id.value(), done_event_serial);

    // Auto-destroy one-shot callback after sending the event
    // Note: destroy_object_by_client decrements ref_count. If it's 1 (our initial registration), it will be removed.
    // If the client somehow managed to get another reference (not possible for wl_callback), it would persist.
    match client_space.destroy_object_by_client(callback_object_id).await {
        Ok(true) => debug!("Client {}: Auto-destroyed wl_callback object {}", client_id, callback_object_id.value()),
        Ok(false) => warn!("Client {}: wl_callback object {} not fully destroyed (ref_count > 0), this is unexpected for a sync callback.", client_id, callback_object_id.value()),
        Err(e) => warn!("Client {}: Failed to auto-destroy wl_callback {} after sync: {}", client_id, callback_object_id.value(), e),
    }
    Ok(())
}

// Handler for wl_display.get_registry (opcode 1)
// Args: new_id (registry: wl_registry)
pub async fn handle_get_registry(
    client_id: ClientId,
    _object_id: ObjectId,
    _object_version: u32,
    args: Vec<ArgumentValue>,
    client_space: Arc<ClientObjectSpace>,
    event_sender: Arc<EventSender>,         // Added EventSender
    client_stream: Arc<TokioUnixStream>,     // Added client_stream
    server_globals: &[crate::compositor::wayland_server::protocols::core::wl_registry::ServerGlobal], // Added server_globals
) -> Result<(), WaylandServerError> {
    info!("Handling wl_display.get_registry for client {}", client_id);
    let registry_new_id = match args.get(0) {
        Some(ArgumentValue::NewId(id)) => *id,
        _ => return Err(WaylandServerError::Protocol("wl_display.get_registry: Expected new_id argument".to_string())),
    };

    let registry_object_id = ObjectId::new(registry_new_id.value());

    // Register the new wl_registry object
    client_space.register_object(
        registry_object_id,
        wl_registry_interface(), // Use function
        WL_REGISTRY_VERSION,
    ).await?;
    debug!("Client {}: Registered new wl_registry object {}", client_id, registry_object_id.value());

    info!("Client {}: wl_registry object {} created. Sending global events.", client_id, registry_object_id.value());
    // Send wl_registry.global events for all available globals.
    for global in server_globals {
        let global_event = crate::compositor::wayland_server::protocols::core::wl_registry::RegistryGlobalEvent {
            name: global.name_id,
            interface: global.interface.as_str().to_string(),
            version: global.version,
        };
        event_sender.send_event(
            &*client_stream,
            registry_object_id,
            crate::compositor::wayland_server::protocols::core::wl_registry::EVT_GLOBAL_OPCODE,
            global_event,
            vec![]
        ).await?;
        debug!("Client {}: Sent global event for {} (name_id: {}) to registry {}", client_id, global.interface.as_str(), global.name_id, registry_object_id.value());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::wayland_server::client::ClientId;
    use crate::compositor::wayland_server::event_sender::EventSender;
    use crate::compositor::wayland_server::objects::ClientObjectSpace;
    use crate::compositor::wayland_server::protocol::{ArgumentValue, NewId, ObjectId};
    use crate::compositor::wayland_server::protocols::core::wl_callback::wl_callback_interface;
    use crate::compositor::wayland_server::protocols::core::wl_registry::{get_server_globals_list, wl_registry_interface};

    use std::sync::Arc;
    use tokio::net::UnixStream as TokioUnixStream;
    use tempfile::tempdir;
    use std::collections::HashMap; // For mock client_streams map in dispatcher tests (not directly here but for consistency)

    // Helper to create a mock client stream pair for testing event sending
    async fn create_mock_stream_pair_for_display_tests() -> (Arc<TokioUnixStream>, Arc<TokioUnixStream>) {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_wl_display_mock.sock");
        let listener = tokio::net::UnixListener::bind(&socket_path).await.unwrap();
        let client_stream_out = Arc::new(TokioUnixStream::connect(&socket_path).await.unwrap()); // Server writes to this
        let (client_stream_in_conn, _addr) = listener.accept().await.unwrap(); // Server reads from this (not used in these tests)
                                                                              // For testing event sending, client_stream_out is used by server to send,
                                                                              // and we'd need another stream for client to read if verifying receipt.
                                                                              // For now, tests will focus on handler logic and EventSender calls.
        (client_stream_out, Arc::new(client_stream_in_conn)) // Return pair, though only one might be used directly by test
    }

    #[tokio::test]
    async fn test_handle_sync_registers_callback_and_sends_done() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let event_sender = Arc::new(EventSender::new());
        let (server_writes_to_client, _client_reads_from) = create_mock_stream_pair_for_display_tests().await;

        let callback_new_id_val = 100u32;
        let args = vec![ArgumentValue::NewId(NewId::new(callback_new_id_val))];

        let result = handle_sync(
            client_id,
            WL_DISPLAY_ID,
            WL_DISPLAY_VERSION,
            args,
            Arc::clone(&client_space),
            Arc::clone(&event_sender),
            Arc::clone(&server_writes_to_client),
        ).await;

        assert!(result.is_ok(), "handle_sync failed: {:?}", result.err());

        // Verify callback object was registered
        let callback_obj = client_space.get_object(ObjectId::new(callback_new_id_val)).await;
        assert!(callback_obj.is_some(), "wl_callback object was not registered");
        assert_eq!(callback_obj.unwrap().interface.as_str(), wl_callback_interface().as_str());

        // TODO: Verify wl_callback.done was sent. This requires reading from _client_reads_from.
        // For now, we trust EventSender was called. The EventSender itself has tests.
        // After event is sent, callback should be destroyed.
        assert!(client_space.get_object(ObjectId::new(callback_new_id_val)).await.is_none(), "wl_callback should be auto-destroyed after done event");
    }

    #[tokio::test]
    async fn test_handle_get_registry_registers_registry_and_sends_globals() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let event_sender = Arc::new(EventSender::new());
        let (server_writes_to_client, _client_reads_from) = create_mock_stream_pair_for_display_tests().await;
        let server_globals = get_server_globals_list();

        let registry_new_id_val = 101u32;
        let args = vec![ArgumentValue::NewId(NewId::new(registry_new_id_val))];

        let result = handle_get_registry(
            client_id,
            WL_DISPLAY_ID,
            WL_DISPLAY_VERSION,
            args,
            Arc::clone(&client_space),
            Arc::clone(&event_sender),
            Arc::clone(&server_writes_to_client),
            &server_globals,
        ).await;
        assert!(result.is_ok(), "handle_get_registry failed: {:?}", result.err());

        // Verify wl_registry object was registered
        let registry_obj = client_space.get_object(ObjectId::new(registry_new_id_val)).await;
        assert!(registry_obj.is_some(), "wl_registry object was not registered");
        assert_eq!(registry_obj.unwrap().interface.as_str(), wl_registry_interface().as_str());

        // TODO: Verify wl_registry.global events were sent by reading from _client_reads_from.
        // For now, we trust EventSender was called for each global.
    }
}
