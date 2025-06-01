// In novade-system/src/compositor/wayland_server/protocols/core/wl_registry.rs
use crate::compositor::wayland_server::client::ClientId;
use crate::compositor::wayland_server::error::WaylandServerError;
use crate::compositor::wayland_server::objects::{ClientObjectSpace, Interface, ObjectId};
use crate::compositor::wayland_server::protocol::ArgumentValue;
// use crate::compositor::wayland_server::event_sender::EventSender; // For sending global events
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub fn wl_registry_interface() -> Interface { Interface::new("wl_registry") }
pub const WL_REGISTRY_VERSION: u32 = 1;

// Request Opcodes
pub const REQ_BIND_OPCODE: u16 = 0;

// Event Opcodes
pub const EVT_GLOBAL_OPCODE: u16 = 0;
pub const EVT_GLOBAL_REMOVE_OPCODE: u16 = 1;

#[derive(Debug, Clone)]
pub struct RegistryGlobalEvent {
    pub name: u32,
    pub interface: String,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct ServerGlobal {
    pub name_id: u32,
    pub interface: Interface,
    pub version: u32,
}

pub fn get_server_globals_list() -> Vec<ServerGlobal> {
    use super::wl_compositor::{wl_compositor_interface, WL_COMPOSITOR_VERSION};
    use super::wl_shm::{wl_shm_interface, WL_SHM_VERSION};

    vec![
        ServerGlobal {
            name_id: 1,
            interface: wl_compositor_interface(),
            version: WL_COMPOSITOR_VERSION,
        },
        ServerGlobal {
            name_id: 2,
            interface: wl_shm_interface(),
            version: WL_SHM_VERSION,
        },
    ]
}

pub async fn handle_bind(
    client_id: ClientId,
    _registry_object_id: ObjectId,
    _registry_object_version: u32,
    args: Vec<ArgumentValue>,
    client_space: Arc<ClientObjectSpace>,
    server_globals: &[ServerGlobal],
) -> Result<(), WaylandServerError> {
    info!("Client {}: Handling wl_registry.bind", client_id);

    let global_name_to_bind = match args.get(0) {
        Some(ArgumentValue::Uint(val)) => *val,
        _ => return Err(WaylandServerError::Protocol("wl_registry.bind: Arg 0 (global name) must be Uint".to_string())),
    };

    // Argument 1 is interface name (string), Argument 2 is version (uint) for new_id in actual spec
    // However, the `new_id` type in Wayland protocol messages implicitly carries the target ID chosen by client.
    // The `validate_and_parse_args` gives us `NewId(id_val)` for arg[1] if signature is `[Uint, NewId]`
    // The actual Wayland spec for wl_registry.bind(name: uint, id: new_id)
    // The `id` argument is `new_id<unknown>` - client provides interface string and version implicitly with new_id request.
    // The `validate_and_parse_args` will give us NewId(client_chosen_id_val)
    // The interface and version for the new_id are *not* part of the bind arguments themselves,
    // rather, they are part of the `new_id` request that the client makes.
    // So, the signature for bind should be: `[Uint (name), String (interface), Uint (version), NewId (id)]` NO, this is wrong.
    // The standard signature for wl_registry.bind is (name: uint, id: new_id).
    // The client *requests* to bind a `name` to a `new_id` of a certain `interface` and `version`.
    // The `interface` and `version` for the `new_id` are *part of the `new_id` itself* conceptually,
    // but not separate arguments to `bind`.
    // The `NewId` type argument in our system just carries the numeric ID.
    // The dispatcher needs to know which interface and version this new_id is for *before* calling bind.
    // This is usually implicit in the `new_id` opcode itself (e.g. `wl_compositor.create_surface(new_id<wl_surface>)`).
    // For `wl_registry.bind`, the `interface` and `version` are those of the *global* being bound.

    // Correct interpretation:
    // Arg 0: name (uint) - ID of the global to bind.
    // Arg 1: id (new_id) - The new object ID the client wants to create.
    // The client *also* specifies string interface and version for this new_id, but these are not direct args to bind.
    // Our current `validate_and_parse_args` doesn't give us interface string/version for a NewId type.
    // This means the `MessageSignature` for `wl_registry.bind` needs to be more specific or our `NewId` handling needs enhancement.
    // For now, let's assume the signature is `[Uint (name), NewId (id)]` as per `mock_spec_store` update needed later.

    let client_chosen_new_object_id_val = match args.get(1) { // This is the ID for the new object.
        Some(ArgumentValue::NewId(id)) => id.value(),
        // If the signature was (uint name, string interface, uint version, new_id id)
        // Some(ArgumentValue::String(s)) => s.clone(), // This would be arg1 if string interface
        // Some(ArgumentValue::Uint(v)) => *v, // This would be arg2 if version
        // Some(ArgumentValue::NewId(id)) => id.value(), // This would be arg3 if new_id
        _ => return Err(WaylandServerError::Protocol("wl_registry.bind: Arg 1 (new object id) must be NewId".to_string())),
    };

    let target_global = match server_globals.iter().find(|g| g.name_id == global_name_to_bind) {
        Some(g) => g,
        None => {
            error!("Client {}: Attempted to bind to unknown global name ID {}", client_id, global_name_to_bind);
            // TODO: Send wl_display.error event to client
            return Err(WaylandServerError::Protocol(format!("Bind attempt to unknown global name ID {}", global_name_to_bind)));
        }
    };

    // Version negotiation: Client requests a version, server provides one.
    // The client's requested version for the new_id is not directly an argument to bind.
    // It's implicit in how new_id is handled by client libraries.
    // For now, server binds with the global's advertised version.
    // A real implementation would get the client's desired version from the new_id request context.
    let version_to_bind = target_global.version;

    let new_object_id = ObjectId::new(client_chosen_new_object_id_val);

    // Check if the client-chosen ID for the new object already exists.
    if client_space.get_object(new_object_id).await.is_some() {
        error!("Client {}: Attempted to bind global to already existing object ID {}", client_id, new_object_id.value());
        // This is a client protocol error. wl_display.error should be sent.
        return Err(WaylandServerError::Protocol(format!(
            "Client attempted to bind global to existing object ID {}", new_object_id.value()
        )));
    }

    // Register the new object with the global's interface and negotiated version.
    client_space.register_object(
        new_object_id,
        target_global.interface.clone(),
        version_to_bind,
    ).await?;

    info!(
        "Client {}: Bound global name {} (Interface: {}, ServerMaxVersion: {}) to new object ID {} with bound version {}.",
        client_id, global_name_to_bind, target_global.interface.as_str(), target_global.version,
        new_object_id.value(), version_to_bind
    );

    // TODO: If the bound object has specific post-bind actions (e.g. wl_shm needs to send 'format' events immediately),
    // they would be triggered here.
    if target_global.interface.as_str() == "wl_shm" {
        // Call send_initial_format_events for the newly bound wl_shm object
        // The EventSender would be needed here, passed into handle_bind.
        // For now, we'll just log that it should happen.
        info!(
            "Client {}: wl_shm global bound to object {}. Triggering initial format events (TODO: needs EventSender).",
            client_id, new_object_id.value()
        );
        // Example of how it might be called if event_sender was available:
        // use super::wl_shm::send_initial_format_events;
        // if let Err(e) = send_initial_format_events(client_id, new_object_id, event_sender).await {
        //     warn!("Client {}: Failed to send initial format events for wl_shm object {}: {}", client_id, new_object_id.value(), e);
        // }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::wayland_server::protocol::NewId;
    use crate::compositor::wayland_server::objects::ClientObjectSpace; // Ensure this is imported

    #[test]
    fn test_get_server_globals_list_content() {
        let globals = get_server_globals_list();
        assert!(!globals.is_empty(), "Server globals list should not be empty");

        let compositor_global = globals.iter().find(|g| g.interface.as_str() == "wl_compositor");
        assert!(compositor_global.is_some());
        assert_eq!(compositor_global.unwrap().name_id, 1);
        assert_eq!(compositor_global.unwrap().version, crate::compositor::wayland_server::protocols::core::wl_compositor::WL_COMPOSITOR_VERSION);

        let shm_global = globals.iter().find(|g| g.interface.as_str() == "wl_shm");
        assert!(shm_global.is_some());
        assert_eq!(shm_global.unwrap().name_id, 2);
        assert_eq!(shm_global.unwrap().version, crate::compositor::wayland_server::protocols::core::wl_shm::WL_SHM_VERSION);
    }

    #[tokio::test]
    async fn test_handle_bind_success() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let server_globals = get_server_globals_list();

        let wl_compositor_global = server_globals.iter().find(|g| g.interface.as_str() == "wl_compositor").unwrap();
        let client_chosen_id_for_compositor = 100u32;

        // wl_registry.bind(name: uint, id: new_id)
        // The 'id' new_id argument also implies interface and version for the new object.
        // Our current signature parsing assumes these are handled by the dispatcher before calling this.
        // The signature for bind should be (uint name, new_id id)
        // The client also provides interface and version for the new_id, but not as direct args to bind.
        // The test here will provide the args as they'd be parsed if the signature was [Uint, NewId].
        let args = vec![
            ArgumentValue::Uint(wl_compositor_global.name_id),
            ArgumentValue::NewId(NewId::new(client_chosen_id_for_compositor)),
            // If signature was (name, interface_str, version_uint, new_id):
            // ArgumentValue::String(wl_compositor_global.interface.as_str().to_string()),
            // ArgumentValue::Uint(wl_compositor_global.version),
        ];

        let result = handle_bind(
            client_id,
            ObjectId::new(5), // ID of the wl_registry object itself
            1, // version of the wl_registry object
            args,
            Arc::clone(&client_space),
            &server_globals
        ).await;

        assert!(result.is_ok(), "handle_bind failed: {:?}", result.err());

        let bound_object = client_space.get_object(ObjectId::new(client_chosen_id_for_compositor)).await;
        assert!(bound_object.is_some(), "Object {} was not registered after bind", client_chosen_id_for_compositor);
        let entry = bound_object.unwrap();
        assert_eq!(entry.interface.as_str(), "wl_compositor");
        assert_eq!(entry.version, wl_compositor_global.version); // Bound with server's max version for now
    }

    #[tokio::test]
    async fn test_handle_bind_to_existing_new_id() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let server_globals = get_server_globals_list();

        let wl_compositor_global = server_globals.iter().find(|g| g.interface.as_str() == "wl_compositor").unwrap();
        let client_chosen_id = 101u32;

        // Pre-register the ID the client wants to use for the new object
        client_space.register_object(ObjectId::new(client_chosen_id), Interface::new("some_other_interface"), 1).await.unwrap();

        let args = vec![
            ArgumentValue::Uint(wl_compositor_global.name_id),
            ArgumentValue::NewId(NewId::new(client_chosen_id)),
        ];

        let result = handle_bind(client_id, ObjectId::new(5), 1, args, Arc::clone(&client_space), &server_globals).await;
        assert!(result.is_err(), "handle_bind should fail if client chosen new_id already exists");
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("bind global to existing object ID"));
        } else {
            panic!("Expected Protocol error for binding to existing new_id");
        }
    }

    #[tokio::test]
    async fn test_handle_bind_invalid_global_name_id() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let server_globals = get_server_globals_list();
        let invalid_global_name = 9999u32;

        let args = vec![
            ArgumentValue::Uint(invalid_global_name),
            ArgumentValue::NewId(NewId::new(102)),
        ];

        let result = handle_bind(client_id, ObjectId::new(5), 1, args, client_space, &server_globals).await;
        assert!(result.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains(&format!("Bind attempt to unknown global name ID {}", invalid_global_name)));
        } else {
            panic!("Expected Protocol error for unknown global name");
        }
    }

    #[tokio::test]
    async fn test_handle_bind_incorrect_arg_types() {
        let client_id = ClientId::new();
        let client_space = Arc::new(ClientObjectSpace::new(client_id));
        let server_globals = get_server_globals_list();

        // Case 1: global name is not Uint
        let args_case1 = vec![
            ArgumentValue::String("not_a_uint".to_string()), // Incorrect type
            ArgumentValue::NewId(NewId::new(103)),
        ];
        let result1 = handle_bind(client_id, ObjectId::new(5), 1, args_case1, Arc::clone(&client_space), &server_globals).await;
        assert!(result1.is_err());
         if let Err(WaylandServerError::Protocol(msg)) = result1 {
            assert!(msg.contains("Arg 0 (global name) must be Uint"));
        } else {
            panic!("Expected Protocol error for wrong type on arg 0");
        }

        // Case 2: new_id is not NewId type
        let wl_compositor_global = server_globals.iter().find(|g| g.interface.as_str() == "wl_compositor").unwrap();
        let args_case2 = vec![
            ArgumentValue::Uint(wl_compositor_global.name_id),
            ArgumentValue::Int(104), // Incorrect type, should be NewId
        ];
        let result2 = handle_bind(client_id, ObjectId::new(5), 1, args_case2, Arc::clone(&client_space), &server_globals).await;
        assert!(result2.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result2 {
            assert!(msg.contains("Arg 1 (new object id) must be NewId"));
        } else {
            panic!("Expected Protocol error for wrong type on arg 1");
        }
    }
}
