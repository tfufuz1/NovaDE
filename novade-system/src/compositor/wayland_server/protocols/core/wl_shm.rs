// In novade-system/src/compositor/wayland_server/protocols/core/wl_shm.rs
use crate::compositor::wayland_server::client::ClientId;
use crate::compositor::wayland_server::error::WaylandServerError;
use crate::compositor::wayland_server::objects::{ClientObjectSpace, Interface, ObjectId, ObjectEntry};
use crate::compositor::wayland_server::protocol::ArgumentValue;
use crate::compositor::wayland_server::protocols::core::wl_shm_pool::{wl_shm_pool_interface, WL_SHM_POOL_VERSION, ShmPoolState};
// use crate::compositor::wayland_server::event_sender::EventSender; // For sending format events
use std::sync::Arc;
use tracing::{debug, info, error, warn};
use std::os::unix::io::RawFd;

pub fn wl_shm_interface() -> Interface { Interface::new("wl_shm") }
pub const WL_SHM_VERSION: u32 = 1;

// Event Opcodes for wl_shm
pub const EVT_FORMAT_OPCODE: u16 = 0;

// Request Opcodes for wl_shm
pub const REQ_CREATE_POOL_OPCODE: u16 = 0;

// wl_shm.format event data (sent by server)
// These are Wayland protocol values for pixel formats.
// See enum wl_shm_format in wayland.xml
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WlShmFormat {
    Argb8888 = 0,
    Xrgb8888 = 1,
    C8 = 0x20384343,
    Rgb332 = 0x38424752,
    Bgr233 = 0x38524742,
    // ... many other formats omitted for brevity
    Xbgr8888 = 0x34324758, // XBGR8888
    Abgr8888 = 0x34324241, // ABGR8888
}

#[derive(Debug)]
pub struct ShmFormatEvent {
    pub format_code: WlShmFormat, // The wl_shm_format enum value
}

// This function would be called once after wl_shm global is bound by a client.
pub async fn send_initial_format_events(
    client_id: ClientId,
    shm_object_id: ObjectId, // The ID of the client's wl_shm object
    // event_sender: &EventSender,
) -> Result<(), WaylandServerError> {
    info!("Client {}: Preparing to send initial wl_shm.format events for shm object {}", client_id, shm_object_id.value());
    let supported_formats = vec![
        WlShmFormat::Argb8888,
        WlShmFormat::Xrgb8888,
        WlShmFormat::Xbgr8888,
        WlShmFormat::Abgr8888,
    ];

    for format_enum in supported_formats {
        let _event_data = ShmFormatEvent { format_code: format_enum }; // Mark as unused for now
        debug!("Client {}: Preparing wl_shm.format event ({:?}) for shm object {}", client_id, _event_data, shm_object_id.value());
        // TODO: Use EventSender to serialize and send this event:
        // event_sender.send_event(client_id, shm_object_id, EVT_FORMAT_OPCODE, event_data).await?;
    }
    Ok(())
}


// Handler for wl_shm.create_pool (opcode 0)
// Arguments: id (new_id<wl_shm_pool>), fd (fd), size (int32)
pub async fn handle_create_pool(
    client_id: ClientId,
    _shm_object_id: ObjectId, // The wl_shm object that received the request
    _shm_object_version: u32,
    args: Vec<ArgumentValue>,
    client_space: Arc<ClientObjectSpace>,
) -> Result<(), WaylandServerError> {
    info!("Client {}: Handling wl_shm.create_pool", client_id);

    let new_pool_id_val = match args.get(0) {
        Some(ArgumentValue::NewId(id)) => id.value(),
        _ => return Err(WaylandServerError::Protocol("wl_shm.create_pool: Arg 0 (new_id) must be NewId".to_string())),
    };
    let pool_fd = match args.get(1) {
        Some(ArgumentValue::Fd(fd)) => *fd,
        _ => return Err(WaylandServerError::Protocol("wl_shm.create_pool: Arg 1 (fd) must be Fd".to_string())),
    };
    let pool_size = match args.get(2) {
        Some(ArgumentValue::Int(val)) => *val,
        _ => return Err(WaylandServerError::Protocol("wl_shm.create_pool: Arg 2 (size) must be Int".to_string())),
    };

    if pool_size <= 0 {
        error!("Client {}: wl_shm.create_pool size must be positive, got {}. Closing FD {}.", client_id, pool_size, pool_fd);
        nix::unistd::close(pool_fd).ok();
        // TODO: Send wl_display.error(object_id=WL_DISPLAY_ID, code=2 (INVALID_FD), message="shm pool size must be positive")
        return Err(WaylandServerError::Protocol(format!("wl_shm.create_pool: size must be positive, got {}", pool_size)));
    }

    let new_pool_id = ObjectId::new(new_pool_id_val);

    if client_space.get_object(new_pool_id).await.is_some() {
        error!("Client {}: Attempted to create shm_pool with already existing object ID {}", client_id, new_pool_id.value());
        nix::unistd::close(pool_fd).ok();
        return Err(WaylandServerError::Protocol(format!(
            "Client attempted to create shm_pool with existing ID {}", new_pool_id.value()
        )));
    }

    let _pool_state = ShmPoolState { fd: pool_fd, size: pool_size };
    debug!("Client {}: SHM Pool details: FD={}, Size={}", client_id, pool_fd, pool_size);

    let pool_bind_version = WL_SHM_POOL_VERSION;

    client_space.register_object(
        new_pool_id,
        wl_shm_pool_interface(),
        pool_bind_version,
    ).await?;

    info!(
        "Client {}: Created new wl_shm_pool with ID {} (bound at version {}), FD: {}, Size: {}. IMPORTANT: FD not yet stored with object, will be closed if not managed!",
        client_id, new_pool_id.value(), pool_bind_version, pool_fd, pool_size
    );
    // The FD (pool_fd) needs to be managed.
    // For now, we are not storing ShmPoolState with ObjectEntry, so the FD might leak or be closed prematurely.
    // A real implementation would store `_pool_state` (likely an Arc<Mutex<ShmPoolState>>) with the ObjectEntry.
    // The `close(pool_fd)` that was here is removed as the FD should be owned by the ShmPoolState,
    // which in turn should be owned by (or associated with) the ObjectEntry for the wl_shm_pool.
    // The Drop impl of ShmPoolState (or a wrapper around it) would close the FD.

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::wayland_server::protocol::NewId;
    use crate::compositor::wayland_server::objects::ClientObjectSpace;
    use crate::compositor::wayland_server::client::ClientId;
    // use std::os::unix::io::FromRawFd; // Not needed for create_dummy_fd logic

    fn create_dummy_fd() -> RawFd {
        let mut fds = [-1; 2];
        nix::unistd::pipe(&mut fds).expect("Failed to create pipe for dummy FD");
        let fd_to_use = fds[0];
        nix::unistd::close(fds[1]).ok();
        fd_to_use
    }

    #[test]
    fn test_shm_format_enum_values() {
        assert_eq!(WlShmFormat::Argb8888 as u32, 0);
        assert_eq!(WlShmFormat::Xrgb8888 as u32, 1);
    }

    #[tokio::test]
    async fn test_handle_create_pool_success() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let shm_id = ObjectId::new(3);
        let new_pool_client_chosen_id = 200u32;
        let dummy_fd = create_dummy_fd();
        let pool_size = 4096;

        let args = vec![
            ArgumentValue::NewId(NewId::new(new_pool_client_chosen_id)),
            ArgumentValue::Fd(dummy_fd),
            ArgumentValue::Int(pool_size),
        ];

        let result = handle_create_pool(
            client_id,
            shm_id,
            WL_SHM_VERSION,
            args,
            Arc::clone(&client_space)
        ).await;

        assert!(result.is_ok(), "handle_create_pool failed: {:?}", result.err());

        let pool_object = client_space.get_object(ObjectId::new(new_pool_client_chosen_id)).await;
        assert!(pool_object.is_some(), "wl_shm_pool object was not registered");
        let entry = pool_object.unwrap();
        assert_eq!(entry.interface.as_str(), "wl_shm_pool");
        assert_eq!(entry.version, WL_SHM_POOL_VERSION);

        // The dummy_fd is now conceptually owned by the ShmPoolState that was created (though not stored globally).
        // In a real scenario, the ShmPoolState's Drop impl would close it when the wl_shm_pool object is destroyed.
        // For this test, since we don't have that full lifecycle, we manually close it IF the handler didn't error out and close it.
        // The handler currently doesn't close on success, assuming ShmPoolState's owner will.
        nix::unistd::close(dummy_fd).ok();
    }

    #[tokio::test]
    async fn test_handle_create_pool_invalid_size() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let shm_id = ObjectId::new(3);
        let new_pool_id = 201u32;
        let dummy_fd = create_dummy_fd(); // This FD will be closed by handle_create_pool on error

        let args = vec![
            ArgumentValue::NewId(NewId::new(new_pool_id)),
            ArgumentValue::Fd(dummy_fd),
            ArgumentValue::Int(0),
        ];

        let result = handle_create_pool(client_id, shm_id, WL_SHM_VERSION, args, client_space).await;
        assert!(result.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("size must be positive"));
        } else {
            panic!("Expected Protocol error for invalid size, got {:?}", result);
        }
        // FD should have been closed by the handler. Trying to close again might error or close a reused FD.
        // assert!(nix::unistd::close(dummy_fd).is_err(), "FD should be closed by handler on error");
    }

     #[tokio::test]
    async fn test_handle_create_pool_id_already_exists() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let shm_id = ObjectId::new(3);
        let existing_pool_id_val = 202u32;
        let dummy_fd = create_dummy_fd(); // This FD will be closed by handle_create_pool on error

        client_space.register_object(
            ObjectId::new(existing_pool_id_val),
            wl_shm_pool_interface(), 1
        ).await.unwrap();

        let args = vec![
            ArgumentValue::NewId(NewId::new(existing_pool_id_val)),
            ArgumentValue::Fd(dummy_fd),
            ArgumentValue::Int(4096),
        ];
        let result = handle_create_pool(client_id, shm_id, WL_SHM_VERSION, args, client_space).await;
        assert!(result.is_err());
         if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("create shm_pool with existing ID"));
        } else {
            panic!("Expected Protocol error for existing ID, got {:?}", result);
        }
        // FD should have been closed by the handler.
        // assert!(nix::unistd::close(dummy_fd).is_err(), "FD should be closed by handler on error");
    }
}
