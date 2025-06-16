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


// Imports needed for WaylandObject trait method signatures
use crate::compositor::wayland_server::client::Client;
use crate::compositor::wayland_server::message::Argument;
use crate::compositor::wayland_server::event_sender::EventSender;
// ObjectRegistry itself is needed for the method signature, but it's in the same file.
use super::protocol_spec::{WlDisplayError, WlShmError}; // Import error enums

// Constants for opcodes (event opcodes for wl_callback and wl_registry)
const WL_CALLBACK_DONE_OPCODE: u16 = 0;
const WL_REGISTRY_GLOBAL_EVENT_OPCODE: u16 = 0;
// const WL_REGISTRY_GLOBAL_REMOVE_EVENT_OPCODE: u16 = 1; // If needed later

/// A trait that all Wayland objects managed by the registry must implement.
pub trait WaylandObject: fmt::Debug + Send + Sync {
    /// Dispatches a request (a message from client to server) to this object.
    ///
    /// # Arguments
    /// * `request_opcode`: The opcode of the request specific to this object's interface.
    /// * `request_args`: The arguments accompanying the request.
    /// * `client_info`: Information about the client that sent the request.
    /// * `object_id`: The ID of this object instance.
    /// * `event_sender`: A mutable reference to an `EventSender` to queue events back to the client.
    /// * `object_registry`: A mutable reference to the `ObjectRegistry` for creating new objects, etc.
    ///
    /// # Returns
    /// * `Ok(())` if the request was dispatched successfully.
    /// * `Err(String)` if an error occurred (e.g., invalid opcode, bad arguments, internal error).
    ///   The error string may be used for logging or sending a protocol error to the client.
    fn dispatch_request(
        &mut self,
        request_opcode: u16,
        request_args: Vec<Argument>, // Passed by value as they are consumed
        client_info: &Client,
        object_id: u32,
        event_sender: &mut EventSender,
        object_registry: &mut ObjectRegistry, // Changed to &mut
    ) -> Result<(), String>;
}

// Allow `dyn WaylandObject` to be used in a way that it itself is Send + Sync.
// This is generally true if all implementors are Send + Sync.
// The `Box<dyn WaylandObject + Send + Sync>` already ensures this.

/// Represents an entry in the ObjectRegistry.
#[derive(Debug)]
pub struct RegistryEntry {
    pub object: Box<dyn WaylandObject>,
    pub client_id: u64, // ID of the client that "owns" or created this object
    pub interface_name: String, // Name of the Wayland interface (e.g., "wl_surface")
    pub version: u32,   // Version of the interface the object was created with
    pub parent_id: Option<u32>, // ID of the parent object, if any
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
impl WaylandObject for WlDisplay {
    fn dispatch_request(
        &mut self,
        request_opcode: u16,
        request_args: Vec<Argument>,
        client_info: &Client,
        self_object_id: u32, // ID of this WlDisplay instance (always 1)
        event_sender: &mut EventSender,
        object_registry: &mut ObjectRegistry,
    ) -> Result<(), String> {
        match request_opcode {
            0 => { // sync
                if request_args.len() != 1 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "sync request expects 1 argument".to_string())?;
                    return Err("wl_display.sync: expects 1 argument".to_string());
                }
                let callback_new_id = match request_args[0] {
                    Argument::NewId(id) if id != 0 => id,
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, "sync callback ID must be a valid new_id".to_string())?;
                        return Err("wl_display.sync: callback_id must be a valid new_id".to_string());
                    }
                };

                let callback_obj = WlCallback;
                let server_assigned_id = object_registry.new_server_object(
                    client_info.id,
                    callback_obj,
                    "wl_callback".to_string(),
                    1, // version
                    None, // parent
                )?;

                // Send wl_callback.done event immediately.
                // The spec for wl_callback.done has one argument: callback_data (uint32).
                // This data is often a serial that the client can use to match with its sync request.
                // For simplicity, we can send a 0 or a placeholder serial.
                let serial: u32 = 0; // Example serial. In a real compositor, this might be tracked.
                event_sender.send_event(
                    client_info.id,
                    server_assigned_id, // ID of the WlCallback object
                    WL_CALLBACK_DONE_OPCODE,
                    vec![Argument::Uint(serial)]
                )?;
                println!("[WlDisplay] Sync: created wl_callback (ID {}) and sent done event.", server_assigned_id);
                Ok(())
            }
            1 => { // get_registry
                if request_args.len() != 1 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "get_registry request expects 1 argument".to_string())?;
                    return Err("wl_display.get_registry: expects 1 argument".to_string());
                }
                let registry_new_id = match request_args[0] {
                    Argument::NewId(id) if id != 0 => id,
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, "get_registry ID must be a valid new_id".to_string())?;
                        return Err("wl_display.get_registry: registry_id must be a valid new_id".to_string());
                    }
                };

                let registry_obj = WlRegistry;
                object_registry.new_object(
                    client_info.id,
                    registry_new_id,
                    registry_obj,
                    "wl_registry".to_string(),
                    1, // version
                    Some(1), // Parent is wl_display (ID 1)
                )?;
                println!("[WlDisplay] GetRegistry: created wl_registry (ID {}) for client {}.", registry_new_id, client_info.id);

                // Advertise globals
                // TODO: Manage globals more dynamically. For now, hardcode a few.
                // Using placeholder global "names" (IDs for globals, distinct from object IDs).
                // These names are what clients use in wl_registry.bind requests.
                let globals_to_advertise: Vec<(u32, &str, u32)> = vec![
                    (1, "wl_compositor", 4), // global_name=1, interface="wl_compositor", version=4
                    (2, "wl_shm", 1),       // global_name=2, interface="wl_shm", version=1
                    // Add other globals like wl_seat, xdg_wm_base here later
                ];

                for (global_name_id, interface_name_str, version) in globals_to_advertise {
                    event_sender.send_event(
                        client_info.id,
                        registry_new_id, // The wl_registry object is the sender of .global events
                        WL_REGISTRY_GLOBAL_EVENT_OPCODE,
                        vec![
                            Argument::Uint(global_name_id),
                            Argument::String(interface_name_str.to_string()),
                            Argument::Uint(version),
                        ],
                    )?;
                    println!("[WlDisplay/Registry] Advertised global: name={}, interface='{}', version={}", global_name_id, interface_name_str, version);
                }
                Ok(())
            }
            _ => Err(format!("Unsupported opcode {} for wl_display", request_opcode)),
        }
    }
}

/// Placeholder for the wl_registry object.
#[derive(Debug)]
pub struct WlRegistry;
impl WaylandObject for WlRegistry {
    fn dispatch_request(
        &mut self,
        request_opcode: u16,
        request_args: Vec<Argument>,
        client_info: &Client,
        self_object_id: u32, // ID of this WlRegistry instance
        event_sender: &mut EventSender,
        object_registry: &mut ObjectRegistry,
    ) -> Result<(), String> {
        match request_opcode {
            0 => { // bind
                if request_args.len() != 4 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "bind request expects 4 arguments".to_string())?;
                    return Err("wl_registry.bind: expects 4 arguments".to_string());
                }

                let global_name_id = match request_args[0] {
                    Argument::Uint(id) => id,
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "bind 'name' (arg 0) must be uint".to_string())?;
                        return Err("wl_registry.bind: 'name' (arg 0) must be uint.".to_string());
                    }
                };
                let interface_to_bind = match &request_args[1] {
                    Argument::String(s) => s.clone(),
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "bind 'interface' (arg 1) must be string".to_string())?;
                        return Err("wl_registry.bind: 'interface' (arg 1) must be string.".to_string());
                    }
                };
                let requested_version = match request_args[2] {
                    Argument::Uint(v) => v,
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "bind 'version' (arg 2) must be uint".to_string())?;
                        return Err("wl_registry.bind: 'version' (arg 2) must be uint.".to_string());
                    }
                };
                let new_object_id = match request_args[3] {
                    Argument::NewId(id) if id != 0 => id,
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, "bind 'id' (arg 3) must be a valid new_id".to_string())?;
                        return Err("wl_registry.bind: 'id' (arg 3) must be a valid new_id.".to_string());
                    }
                };

                println!(
                    "[WlRegistry] Bind request: global_name_id={}, interface='{}', version={}, new_object_id={}",
                    global_name_id, interface_to_bind, requested_version, new_object_id
                );

                // Server-side list of advertised globals (name_id, interface_string, max_version)
                // This should ideally come from a central place or be derived from the list used in WlDisplay::get_registry
                let server_globals: Vec<(u32, &str, u32)> = vec![
                    (1, "wl_compositor", 4),
                    (2, "wl_shm", 1),
                ];

                let mut found_global = false;
                for (s_global_id, s_interface, s_version) in server_globals {
                    if s_global_id == global_name_id && s_interface == interface_to_bind {
                        if requested_version == 0 || requested_version > s_version {
                            // Client requested version 0 (invalid) or a version higher than server supports
                            let err_msg = format!(
                                "Client requested version {} for '{}' (global_id {}), but server supports version {}. Binding with server version.",
                                requested_version, interface_to_bind, global_name_id, s_version
                            );
                            eprintln!("[WlRegistry] Bind warning: {}", err_msg);
                            // Bind with the server's max version as per Wayland spec for wl_registry.bind
                            // The actual version used is min(client_requested, server_supported) but client shouldn't request > server.
                            // If client sends 0 or > server_version, server can choose its version or error.
                            // Sending error is safer if client requests too high. For 0, it's a client bug.
                            // Let's be strict for now if requested_version > s_version.
                             event_sender.send_protocol_error(
                                client_info.id,
                                self_object_id, // Error on the wl_registry object
                                0, // Using a generic error code, could be more specific
                                format!("Unsupported version {} for interface '{}', server supports up to {}", requested_version, interface_to_bind, s_version)
                            )?;
                            return Err(err_msg);
                        }

                        // Create and register the object
                        let actual_bound_version = std::cmp::min(requested_version, s_version);

                        match interface_to_bind.as_str() {
                            "wl_compositor" => {
                                object_registry.new_object(client_info.id, new_object_id, WlCompositorImpl, interface_to_bind, actual_bound_version, Some(self_object_id))?;
                                println!("[WlRegistry] Bound wl_compositor (ID {}) v{} for client {}.", new_object_id, actual_bound_version, client_info.id);
                            }
                            "wl_shm" => {
                                object_registry.new_object(client_info.id, new_object_id, WlShmImpl, interface_to_bind, actual_bound_version, Some(self_object_id))?;
                                println!("[WlRegistry] Bound wl_shm (ID {}) v{} for client {}.", new_object_id, actual_bound_version, client_info.id);
                            }
                            _ => {
                                // Should not happen if it matched s_interface
                                event_sender.send_protocol_error(client_info.id, self_object_id, 0, format!("Internal error: Matched global '{}' but no factory.", interface_to_bind))?;
                                return Err(format!("Internal error: No factory for interface '{}'", interface_to_bind));
                            }
                        }
                        found_global = true;
                        break;
                    }
                }

                if !found_global {
                    let err_msg = format!("Global with name_id {} and interface '{}' not found or not supported.", global_name_id, interface_to_bind);
                    eprintln!("[WlRegistry] Bind error: {}", err_msg);
                    event_sender.send_protocol_error(client_info.id, self_object_id, 0, err_msg.clone())?; // Using generic error code
                    return Err(err_msg);
                }
                Ok(())
            }
            _ => Err(format!("Unsupported opcode {} for wl_registry", request_opcode)),
        }
    }
}

// --- Concrete Implementations for other core objects ---

#[derive(Debug)]
pub struct WlCallback;
impl WaylandObject for WlCallback {
    fn dispatch_request(
        &mut self,
        request_opcode: u16,
        _request_args: Vec<Argument>,
        _client_info: &Client,
        object_id: u32,
        _event_sender: &mut EventSender,
        _object_registry: &mut ObjectRegistry,
    ) -> Result<(), String> {
        // wl_callback typically has no client requests.
        eprintln!("[WlCallback] dispatch_request: object_id={}, opcode={}. wl_callback has no requests.", object_id, request_opcode);
        Err(format!("wl_callback (ID {}) does not support requests (opcode {})", object_id, request_opcode))
    }
}

#[derive(Debug)]
pub struct WlCompositorImpl;
impl WaylandObject for WlCompositorImpl {
    fn dispatch_request(
        &mut self,
        request_opcode: u16,
        request_args: Vec<Argument>,
        client_info: &Client,
        self_object_id: u32, // ID of this WlCompositorImpl instance
        event_sender: &mut EventSender,
        object_registry: &mut ObjectRegistry,
    ) -> Result<(), String> {
        match request_opcode {
            0 => { // create_surface
                if request_args.len() != 1 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_surface expects 1 argument (new_id)".to_string())?;
                    return Err("wl_compositor.create_surface: expects 1 argument".to_string());
                }
                let new_surface_id = match request_args[0] {
                    Argument::NewId(id) if id != 0 => id,
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, "create_surface ID must be a valid new_id".to_string())?;
                        return Err("wl_compositor.create_surface: ID must be a valid new_id".to_string());
                    }
                };

                let surface_obj = WlSurfaceImpl::new();
                object_registry.new_object(
                    client_info.id,
                    new_surface_id,
                    surface_obj,
                    "wl_surface".to_string(),
                    4, // Example version from spec
                    Some(self_object_id) // Parent is this wl_compositor instance
                )?;
                println!("[WlCompositor] Created wl_surface (ID {}) for client {}.", new_surface_id, client_info.id);
                Ok(())
            }
            1 => { // create_region
                 if request_args.len() != 1 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_region expects 1 argument (new_id)".to_string())?;
                    return Err("wl_compositor.create_region: expects 1 argument".to_string());
                }
                let new_region_id = match request_args[0] {
                    Argument::NewId(id) if id != 0 => id,
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, "create_region ID must be a valid new_id".to_string())?;
                        return Err("wl_compositor.create_region: ID must be a valid new_id".to_string());
                    }
                };

                let region_obj = WlRegionImpl::new();
                object_registry.new_object(
                    client_info.id,
                    new_region_id,
                    region_obj,
                    "wl_region".to_string(),
                    1, // Example version from spec
                    Some(self_object_id) // Parent is this wl_compositor instance
                )?;
                println!("[WlCompositor] Created wl_region (ID {}) for client {}.", new_region_id, client_info.id);
                Ok(())
            }
            _ => {
                event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, format!("Unsupported opcode {} for wl_compositor", request_opcode))?;
                Err(format!("Unsupported opcode {} for wl_compositor", request_opcode))
            }
        }
    }
}

#[derive(Debug, Default)] // Default for easy construction
pub struct WlSurfaceImpl {
    pending_buffer: Option<u32>,
    pending_buffer_offset: (i32, i32),
    pending_damage_surface: Vec<(i32, i32, i32, i32)>, // x, y, width, height
    pending_damage_buffer: Vec<(i32, i32, i32, i32)>,  // x, y, width, height
    frame_callbacks: Vec<u32>,
    opaque_region: Option<u32>,
    input_region: Option<u32>,
    buffer_transform: i32, // wl_output.transform enum values
    buffer_scale: i32,

    // Current committed state - not strictly part of this subtask to fully manage, but good to have
    current_buffer: Option<u32>,
    current_buffer_offset: (i32, i32),
    // current_damage_surface: Vec<(i32, i32, i32, i32)>, // This would accumulate until next frame usually
}

impl WlSurfaceImpl {
    pub fn new() -> Self {
        Self {
            buffer_transform: 0, // Normal
            buffer_scale: 1,
            ..Default::default()
        }
    }
}

impl WaylandObject for WlSurfaceImpl {
    fn dispatch_request(
        &mut self,
        opcode: u16,
        args: Vec<Argument>,
        client_info: &Client,
        self_object_id: u32,
        event_sender: &mut EventSender,
        object_registry: &mut ObjectRegistry,
    ) -> Result<(), String> {
        match opcode {
            0 => { // destroy
                if !args.is_empty() {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "destroy request expects 0 arguments".to_string())?;
                    return Err("wl_surface.destroy: expects 0 arguments".to_string());
                }
                object_registry.destroy_object(self_object_id)?;
                Ok(())
            }
            1 => { // attach
                if args.len() != 3 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "attach request expects 3 arguments".to_string())?;
                    return Err("wl_surface.attach: expects 3 arguments".to_string());
                }
                let buffer_id_arg = &args[0]; // ref to avoid move
                let x = match args[1] { Argument::Int(v) => v, _ => {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "attach 'x' (arg 1) must be int".to_string())?;
                    return Err("attach: x must be int".to_string());
                }};
                let y = match args[2] { Argument::Int(v) => v, _ => {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "attach 'y' (arg 2) must be int".to_string())?;
                    return Err("attach: y must be int".to_string());
                }};

                let buffer_obj_id = match buffer_id_arg {
                    Argument::Object(0) => None,
                    Argument::Object(id) if *id == 0 => None, // Explicitly treat object ID 0 as null.
                    Argument::Object(id) => {
                        let entry = object_registry.get_entry(*id).ok_or_else(|| {
                            event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, format!("attach: wl_buffer ID {} does not exist.", id)).unwrap_or_default();
                            format!("attach: wl_buffer ID {} does not exist.", id)
                        })?;
                        if entry.interface_name != "wl_buffer" {
                             event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, format!("attach: object ID {} is not a wl_buffer.", id)).unwrap_or_default();
                            return Err(format!("attach: object ID {} is not a wl_buffer.", id));
                        }
                        Some(*id)
                    }
                     // Allow Argument::Null or similar if defined, or handle Object(0) as null if that's the convention used.
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "attach 'buffer' (arg 0) must be object or null".to_string())?;
                        return Err("attach: buffer argument must be object or null".to_string());
                    }
                };
                self.pending_buffer = buffer_obj_id;
                self.pending_buffer_offset = (x, y);
                Ok(())
            }
            2 => { // damage
                if args.len() != 4 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "damage request expects 4 arguments".to_string())?;
                    return Err("wl_surface.damage: expects 4 arguments".to_string());
                }
                let x = match args[0] { Argument::Int(v) => v, _ => return Err("damage: bad x".to_string())};
                let y = match args[1] { Argument::Int(v) => v, _ => return Err("damage: bad y".to_string())};
                let width = match args[2] { Argument::Int(v) => v, _ => return Err("damage: bad width".to_string())};
                let height = match args[3] { Argument::Int(v) => v, _ => return Err("damage: bad height".to_string())};

                if width <= 0 || height <= 0 { // Wayland spec implies w,h > 0 for damage. Some compositors might allow 0.
                    // Not sending protocol error for this, as it's not strictly forbidden by core for damage (unlike shm buffer dimensions)
                    // but compositor can choose to ignore it.
                    eprintln!("[WlSurface {}] Warning: received damage with non-positive width/height ({},{}). Ignoring.", self_object_id, width, height);
                    return Ok(());
                }
                self.pending_damage_surface.push((x,y,width,height));
                Ok(())
            }
            3 => { // frame
                if args.len() != 1 {
                     event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "frame request expects 1 argument".to_string())?;
                    return Err("wl_surface.frame: expects 1 argument".to_string());
                }
                let callback_id = match args[0] { Argument::NewId(id) if id != 0 => id, _ => {
                     event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, "frame callback ID must be a valid new_id".to_string())?;
                    return Err("frame: bad callback_id".to_string());
                }};
                let callback_obj = WlCallback;
                let server_assigned_id = object_registry.new_server_object(client_info.id, callback_obj, "wl_callback".to_string(), 1, Some(self_object_id))?;
                self.frame_callbacks.push(server_assigned_id);
                Ok(())
            }
            4 => { // set_opaque_region
                if args.len() != 1 {
                     event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "set_opaque_region expects 1 argument".to_string())?;
                    return Err("set_opaque_region: expects 1 argument".to_string());
                }
                let region_id_arg = &args[0];
                self.opaque_region = match region_id_arg {
                    Argument::Object(0) => None,
                    Argument::Object(id) if *id == 0 => None,
                    Argument::Object(id) => {
                        let entry = object_registry.get_entry(*id).ok_or_else(|| {
                             event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, format!("set_opaque_region: wl_region ID {} does not exist.", id)).unwrap_or_default();
                            format!("set_opaque_region: wl_region ID {} does not exist.", id)
                        })?;
                        if entry.interface_name != "wl_region" {
                            event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, format!("set_opaque_region: object ID {} is not a wl_region.", id)).unwrap_or_default();
                            return Err(format!("set_opaque_region: object ID {} is not a wl_region.", id));
                        }
                        Some(*id)
                    }
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "set_opaque_region 'region' (arg 0) must be object or null".to_string())?;
                        return Err("set_opaque_region: region argument must be object or null".to_string());
                    }
                };
                Ok(())
            }
            5 => { // set_input_region
                 if args.len() != 1 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "set_input_region expects 1 argument".to_string())?;
                    return Err("set_input_region: expects 1 argument".to_string());
                }
                let region_id_arg = &args[0];
                self.input_region = match region_id_arg {
                    Argument::Object(0) => None,
                    Argument::Object(id) if *id == 0 => None,
                    Argument::Object(id) => {
                         let entry = object_registry.get_entry(*id).ok_or_else(|| {
                            event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, format!("set_input_region: wl_region ID {} does not exist.", id)).unwrap_or_default();
                            format!("set_input_region: wl_region ID {} does not exist.", id)
                        })?;
                        if entry.interface_name != "wl_region" {
                             event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, format!("set_input_region: object ID {} is not a wl_region.", id)).unwrap_or_default();
                            return Err(format!("set_input_region: object ID {} is not a wl_region.", id));
                        }
                        Some(*id)
                    }
                     _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "set_input_region 'region' (arg 0) must be object or null".to_string())?;
                        return Err("set_input_region: region argument must be object or null".to_string());
                     }
                };
                Ok(())
            }
            6 => { // commit
                if !args.is_empty() {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "commit request expects 0 arguments".to_string())?;
                    return Err("wl_surface.commit: expects 0 arguments".to_string());
                }
                self.current_buffer = self.pending_buffer.take();
                self.current_buffer_offset = self.pending_buffer_offset;
                // Damage handling: typically, pending damage is accumulated into current damage.
                // For simplicity here, we just move it. A real compositor might union regions.
                self.current_damage_surface.append(&mut self.pending_damage_surface); // Clears pending_damage_surface
                // self.current_damage_buffer.append(&mut self.pending_damage_buffer); // If using this
                self.pending_damage_buffer.clear();

                // Frame callbacks are typically sent after rendering the committed state.
                // For now, we don't send them immediately on commit. This will be handled by a render loop.
                println!("[WlSurface {}] Committed state. Buffer: {:?}, Offset: {:?}, DamageSurface: {:?}, DamageBuffer: {:?}, FrameCallbacks: {:?}",
                         self_object_id, self.current_buffer, self.current_buffer_offset, self.current_damage_surface, self.pending_damage_buffer, self.frame_callbacks);
                Ok(())
            }
            7 => { // set_buffer_transform
                if args.len() != 1 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "set_buffer_transform expects 1 argument".to_string())?;
                    return Err("set_buffer_transform: expects 1 argument".to_string());
                }
                self.buffer_transform = match args[0] { Argument::Int(v) => v, _ => {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "set_buffer_transform 'transform' (arg 0) must be int".to_string())?;
                    return Err("set_buffer_transform: bad transform".to_string());
                }};
                Ok(())
            }
            8 => { // set_buffer_scale
                 if args.len() != 1 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "set_buffer_scale expects 1 argument".to_string())?;
                    return Err("set_buffer_scale: expects 1 argument".to_string());
                }
                self.buffer_scale = match args[0] { Argument::Int(v) => v, _ => {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "set_buffer_scale 'scale' (arg 0) must be int".to_string())?;
                    return Err("set_buffer_scale: bad scale".to_string());
                }};
                Ok(())
            }
            9 => { // damage_buffer
                if args.len() != 4 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "damage_buffer expects 4 arguments".to_string())?;
                    return Err("wl_surface.damage_buffer: expects 4 arguments".to_string());
                }
                let x = match args[0] { Argument::Int(v) => v, _ => return Err("damage_buffer: bad x".to_string())};
                let y = match args[1] { Argument::Int(v) => v, _ => return Err("damage_buffer: bad y".to_string())};
                let width = match args[2] { Argument::Int(v) => v, _ => return Err("damage_buffer: bad width".to_string())};
                let height = match args[3] { Argument::Int(v) => v, _ => return Err("damage_buffer: bad height".to_string())};

                if width <= 0 || height <= 0 { // Similar to damage, spec implies w,h > 0.
                    eprintln!("[WlSurface {}] Warning: received damage_buffer with non-positive width/height ({},{}). Ignoring.", self_object_id, width, height);
                    return Ok(());
                }
                self.pending_damage_buffer.push((x,y,width,height));
                Ok(())
            }
            _ => {
                event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, format!("Unsupported opcode {} for wl_surface", opcode))?;
                Err(format!("Unsupported opcode {} for wl_surface", opcode))
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct WlRegionImpl {
    rectangles: Vec<(i32, i32, i32, i32)>, // x, y, width, height
}
impl WlRegionImpl {
    pub fn new() -> Self { Self::default() }
}
impl WaylandObject for WlRegionImpl {
     fn dispatch_request(
        &mut self,
        opcode: u16,
        args: Vec<Argument>,
        _client_info: &Client,
        self_object_id: u32,
        _event_sender: &mut EventSender,
        object_registry: &mut ObjectRegistry
    ) -> Result<(), String> {
        match opcode {
            0 => { // destroy
                if !args.is_empty() {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "destroy request expects 0 arguments".to_string())?;
                    return Err("wl_region.destroy: expects 0 arguments".to_string());
                }
                object_registry.destroy_object(self_object_id)?;
                Ok(())
            }
            1 => { // add
                if args.len() != 4 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "add request expects 4 arguments".to_string())?;
                    return Err("wl_region.add: expects 4 arguments".to_string());
                }
                let x = match args[0] { Argument::Int(v) => v, _ => return Err("region.add: bad x".to_string())};
                let y = match args[1] { Argument::Int(v) => v, _ => return Err("region.add: bad y".to_string())};
                let width = match args[2] { Argument::Int(v) => v, _ => return Err("region.add: bad width".to_string())};
                let height = match args[3] { Argument::Int(v) => v, _ => return Err("region.add: bad height".to_string())};

                if width <= 0 || height <= 0 { // Typically regions are positive, though spec might allow empty via 0.
                     event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidInput as u32, "region.add: width and height must be positive.".to_string())?;
                    return Err("region.add: width and height must be positive.".to_string());
                }
                self.rectangles.push((x,y,width,height));
                Ok(())
            }
            2 => { // subtract
                if args.len() != 4 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "subtract request expects 4 arguments".to_string())?;
                    return Err("wl_region.subtract: expects 4 arguments".to_string());
                }
                let x = match args[0] { Argument::Int(v) => v, _ => return Err("region.subtract: bad x".to_string())};
                let y = match args[1] { Argument::Int(v) => v, _ => return Err("region.subtract: bad y".to_string())};
                let width = match args[2] { Argument::Int(v) => v, _ => return Err("region.subtract: bad width".to_string())};
                let height = match args[3] { Argument::Int(v) => v, _ => return Err("region.subtract: bad height".to_string())};

                if width <= 0 || height <= 0 {
                     event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidInput as u32, "region.subtract: width and height must be positive.".to_string())?;
                    return Err("region.subtract: width and height must be positive.".to_string());
                }
                eprintln!("[WlRegionImpl] Subtract request (obj_id={}, x={}, y={}, w={}, h={}) - Not fully implemented, region math is complex.", self_object_id, x, y, width, height);
                Ok(())
            }
            _ => {
                event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, format!("Unsupported opcode {} for wl_region", opcode))?;
                Err(format!("Unsupported opcode {} for wl_region", opcode))
            }
        }
    }
}

#[derive(Debug)]
pub struct WlShmImpl; // For wl_shm global object
// Imports for WlShmPoolImpl Drop
use std::os::unix::io::RawFd;
use nix::unistd; // For close

// Constants for wl_buffer.release event
const WL_BUFFER_EVENT_RELEASE_OPCODE: u16 = 0;


// WL_DISPLAY_ERROR_INVALID_FD could be a general error code for invalid FDs
const WL_DISPLAY_ERROR_INVALID_FD: u32 = 4; // Example, ensure this is defined or use a generic one.
const WL_DISPLAY_ERROR_INVALID_INPUT: u32 = 5; // Example

impl WaylandObject for WlShmImpl {
    fn dispatch_request(
        &mut self,
        request_opcode: u16,
        request_args: Vec<Argument>,
        client_info: &Client,
        self_object_id: u32,
        event_sender: &mut EventSender,
        object_registry: &mut ObjectRegistry,
    ) -> Result<(), String> {
        match request_opcode {
            0 => { // create_pool
                if request_args.len() != 3 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_pool expects 3 arguments".to_string())?;
                    return Err("wl_shm.create_pool: expects 3 arguments".to_string());
                }
                let new_pool_id = match request_args[0] {
                    Argument::NewId(id) if id != 0 => id,
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, "create_pool ID must be a valid new_id".to_string())?;
                        return Err("wl_shm.create_pool: ID must be a valid new_id".to_string());
                    }
                };
                let fd = match request_args[1] {
                    Argument::Fd(fd_val) => fd_val,
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_pool 'fd' (arg 1) must be fd type".to_string())?;
                        return Err("wl_shm.create_pool: 'fd' (arg 1) must be fd type".to_string());
                    }
                };
                let size = match request_args[2] {
                    Argument::Int(s) => s,
                    _ => {
                        event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_pool 'size' (arg 2) must be int32".to_string())?;
                        return Err("wl_shm.create_pool: 'size' (arg 2) must be int32".to_string());
                    }
                };

                if size <= 0 {
                    // Wayland spec for wl_shm.error says:
                    // invalid_fd: fd is not a valid file descriptor (e.g. for shm_open with size 0)
                    // So, using WlShmError::InvalidFd for size <= 0 seems appropriate.
                    event_sender.send_protocol_error(
                        client_info.id,
                        self_object_id,
                        WlShmError::InvalidFd as u32,
                        "SHM pool size must be positive.".to_string()
                    )?;
                    unsafe {unistd::close(fd).ok(); }
                    return Err("wl_shm.create_pool: size must be positive".to_string());
                }

                let pool_obj = WlShmPoolImpl::new(fd, size);
                object_registry.new_object(
                    client_info.id,
                    new_pool_id,
                    pool_obj,
                    "wl_shm_pool".to_string(),
                    1, // version
                    Some(self_object_id),
                )?;
                println!("[WlShm] Created wl_shm_pool (ID {}) for client {}, fd={}, size={}", new_pool_id, client_info.id, fd, size);
                Ok(())
            }
            _ => {
                event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, format!("Unsupported opcode {} for wl_shm", request_opcode))?;
                Err(format!("Unsupported opcode {} for wl_shm", request_opcode))
            }
        }
    }
}

#[derive(Debug)]
pub struct WlShmPoolImpl {
    fd: RawFd,
    size: i32,
    // TODO: mmap_ptr: Option<*mut u8>, // For actual memory mapping
}

impl WlShmPoolImpl {
    pub fn new(fd: RawFd, size: i32) -> Self {
        // TODO: mmap the fd here if server needs direct access.
        // For now, just store fd and size. Client maps it.
        Self { fd, size, /*mmap_ptr: None*/ }
    }
}

impl Drop for WlShmPoolImpl {
    fn drop(&mut self) {
        println!("[WlShmPoolImpl] Dropping pool, closing fd: {}", self.fd);
        // TODO: munmap if mmap_ptr is Some
        unsafe {
            unistd::close(self.fd).unwrap_or_else(|e| {
                eprintln!("[WlShmPoolImpl] Error closing fd {}: {}", self.fd, e);
            });
        }
    }
}

impl WaylandObject for WlShmPoolImpl {
    fn dispatch_request(
        &mut self,
        request_opcode: u16,
        request_args: Vec<Argument>,
        client_info: &Client,
        self_object_id: u32,
        event_sender: &mut EventSender,
        object_registry: &mut ObjectRegistry,
    ) -> Result<(), String> {
        match request_opcode {
            0 => { // create_buffer
                if request_args.len() != 6 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_buffer expects 6 arguments".to_string())?;
                    return Err("wl_shm_pool.create_buffer: expects 6 arguments".to_string());
                }
                let new_buffer_id = match request_args[0] { Argument::NewId(id) if id != 0 => id, _ => {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidObject as u32, "create_buffer ID must be a valid new_id".to_string())?;
                    return Err("create_buffer: bad new_id".into());
                }};
                let offset = match request_args[1] { Argument::Int(val) => val, _ => {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_buffer offset (arg 1) must be int".to_string())?;
                    return Err("create_buffer: bad offset".into());
                }};
                let width = match request_args[2] { Argument::Int(val) => val, _ => {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_buffer width (arg 2) must be int".to_string())?;
                    return Err("create_buffer: bad width".into());
                }};
                let height = match request_args[3] { Argument::Int(val) => val, _ => {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_buffer height (arg 3) must be int".to_string())?;
                    return Err("create_buffer: bad height".into());
                }};
                let stride = match request_args[4] { Argument::Int(val) => val, _ => {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_buffer stride (arg 4) must be int".to_string())?;
                    return Err("create_buffer: bad stride".into());
                }};
                let format = match request_args[5] { Argument::Uint(val) => val, _ => {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "create_buffer format (arg 5) must be uint".to_string())?;
                    return Err("create_buffer: bad format".into());
                }};

                if width <= 0 || height <= 0 || stride <= 0 || offset < 0 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlShmError::InvalidStride as u32, "create_buffer: width, height, stride must be positive and offset non-negative.".to_string())?;
                    return Err("create_buffer: width, height, stride must be positive and offset non-negative.".to_string());
                }
                // Stride must be >= width * bytes_per_pixel. For now, just check stride >= width.
                // A full check needs bytes_per_pixel from the format.
                if stride < width {
                     event_sender.send_protocol_error(client_info.id, self_object_id, WlShmError::InvalidStride as u32, "create_buffer: stride is too small for width.".to_string())?;
                    return Err("create_buffer: stride is too small for width.".to_string());
                }
                if offset as i64 + (height as i64 * stride as i64) > self.size as i64 {
                     event_sender.send_protocol_error(client_info.id, self_object_id, WlShmError::InvalidStride as u32, "create_buffer: buffer extends past end of pool.".to_string())?;
                    return Err("create_buffer: buffer extends past end of pool.".to_string());
                }
                // TODO: Validate format against those advertised by wl_shm.format event.

                let buffer_obj = WlBufferImpl::new(width, height, format);
                object_registry.new_object(
                    client_info.id,
                    new_buffer_id,
                    buffer_obj,
                    "wl_buffer".to_string(),
                    1, // version
                    Some(self_object_id), // parent
                )?;
                println!("[WlShmPool] Created wl_buffer (ID {}) for client {}.", new_buffer_id, client_info.id);
                Ok(())
            }
            1 => { // destroy
                println!("[WlShmPool] Destroy request for pool ID {}. Object and FD will be cleaned up.", self_object_id);
                object_registry.destroy_object(self_object_id)?;
                Ok(())
            }
            2 => { // resize
                if request_args.len() != 1 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "resize expects 1 argument (size)".to_string())?;
                    return Err("wl_shm_pool.resize: expects 1 argument".to_string());
                }
                let new_size = match request_args[0] { Argument::Int(s) => s, _ => {
                     event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "resize 'size' (arg 0) must be int".to_string())?;
                    return Err("resize: bad size".into());
                }};

                if new_size <= 0 {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidInput as u32, "shm_pool.resize: new size must be positive.".to_string())?;
                    return Err("wl_shm_pool.resize: new size must be positive".to_string());
                }
                // TODO: More robust resize logic (check existing buffers that might become invalid, mremap if server uses mmap_ptr)
                println!("[WlShmPool] Resizing pool ID {} from {} to {}. (Note: mmap not updated yet).", self_object_id, self.size, new_size);
                self.size = new_size;
                Ok(())
            }
            _ => {
                event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, format!("Unsupported opcode {} for wl_shm_pool", request_opcode))?;
                Err(format!("Unsupported opcode {} for wl_shm_pool", request_opcode))
            }
        }
    }
}

#[derive(Debug)]
pub struct WlBufferImpl {
    pub width: i32,
    pub height: i32,
    pub format: u32,
    // TODO: Add reference to pool (e.g. Arc<WlShmPoolImpl> or pool_fd + offset + size for validation)
    // For now, these are enough to identify the buffer conceptually.
}
impl WlBufferImpl {
    pub fn new(width: i32, height: i32, format: u32) -> Self {
        Self { width, height, format }
    }
}
impl WaylandObject for WlBufferImpl {
    fn dispatch_request(
        &mut self,
        request_opcode: u16,
        _request_args: Vec<Argument>,
        client_info: &Client,
        self_object_id: u32,
        event_sender: &mut EventSender,
        object_registry: &mut ObjectRegistry,
    ) -> Result<(), String> {
        match request_opcode {
            0 => { // destroy
                if !request_args.is_empty() {
                    event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, "wl_buffer.destroy expects 0 arguments".to_string())?;
                    return Err("wl_buffer.destroy: expects 0 arguments".to_string());
                }
                println!("[WlBuffer] Destroy request for buffer ID {}.", self_object_id);
                // The object_registry.destroy_object call will remove it.
                // The release event should be sent *before* the object is actually gone from the registry usually,
                // or the client might not receive it if the object_id becomes invalid too quickly.
                // However, sending it after ensures the server won't try to use a destroyed object.
                // For Wayland, wl_buffer.release means the server is done with it.
                // If a client destroys a buffer, it's asserting it won't use it anymore.
                // If the server is also done, it sends release. If client destroys, server should also release.
                event_sender.send_event(client_info.id, self_object_id, WL_BUFFER_EVENT_RELEASE_OPCODE, vec![])?;
                println!("[WlBuffer] Sent release event for buffer ID {}.", self_object_id);
                object_registry.destroy_object(self_object_id)?;
                Ok(())
            }
            _ => {
                event_sender.send_protocol_error(client_info.id, self_object_id, WlDisplayError::InvalidMethod as u32, format!("Unsupported opcode {} for wl_buffer", request_opcode))?;
                Err(format!("Unsupported opcode {} for wl_buffer", request_opcode))
            }
        }
    }
}


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
                interface_name: "wl_display".to_string(),
                version: 1,   // Typically version 1 for wl_display
                parent_id: None, // wl_display has no parent
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
        interface_name: String,
        version: u32,
        parent_id: Option<u32>, // New parameter
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
                interface_name,
                version,
                parent_id, // Store it
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
        interface_name: String,
        version: u32,
        parent_id: Option<u32>, // New parameter
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
                interface_name,
                version,
                parent_id, // Store it
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
    /// If the object has children, they are recursively destroyed first.
    /// Returns an error if the object ID is not found or if it's wl_display.
    pub fn destroy_object(&mut self, object_id_to_destroy: u32) -> Result<Box<dyn WaylandObject>, String> {
        if object_id_to_destroy == 1 {
            return Err("Cannot destroy wl_display (object ID 1).".to_string());
        }

        // Step 1: Find and collect children IDs.
        // We collect IDs first to avoid borrowing issues with self.entries while iterating and calling destroy_object.
        let mut children_ids: Vec<u32> = Vec::new();
        for (id, entry) in &self.entries {
            if entry.parent_id == Some(object_id_to_destroy) {
                children_ids.push(*id);
            }
        }

        // Step 2: Recursively destroy children.
        for child_id in children_ids {
            // Check if child still exists, as it might have been destroyed by a previous cascade.
            if self.entries.contains_key(&child_id) {
                match self.destroy_object(child_id) { // Recursive call
                    Ok(_) => println!("[ObjectRegistry] Cascaded delete of child object {}", child_id),
                    Err(e) => {
                        // Log error and continue, or propagate. For now, log and continue.
                        eprintln!("[ObjectRegistry] Error during cascaded delete of child {}: {}", child_id, e);
                        // Depending on desired strictness, one might want to stop or collect errors.
                    }
                }
            }
        }

        // Step 3: Remove the actual object after its children (if any) are handled.
        self.entries
            .remove(&object_id_to_destroy)
            .map(|entry| entry.object)
            .ok_or_else(|| format!("Object ID {} not found for destruction (it may have been destroyed as a child).", object_id_to_destroy))
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
    impl WaylandObject for TestObject {
        fn dispatch_request( &mut self, opcode: u16, _args: Vec<Argument>, _client: &Client, _obj_id: u32, _sender: &mut EventSender, _registry: &mut ObjectRegistry) -> Result<(), String> {
            println!("TestObject received request with opcode: {}", opcode);
            Ok(())
        }
    }

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
    impl WaylandObject for AnotherTestObject {
        fn dispatch_request( &mut self, opcode: u16, _args: Vec<Argument>, _client: &Client, _obj_id: u32, _sender: &mut EventSender, _registry: &mut ObjectRegistry) -> Result<(), String> {
            println!("AnotherTestObject received request with opcode: {}", opcode);
            Ok(())
        }
    }


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
        let interface_name = "test_object_interface".to_string();

        let result = registry.new_object(client_id, obj_id, test_obj, interface_name.clone(), 1, None);
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
        assert_eq!(entry.interface_name, interface_name);
        assert_eq!(entry.parent_id, None);
    }

    #[test]
    fn test_new_client_object_id_collision() {
        let mut registry = ObjectRegistry::new();
        registry.new_object(1, 100, TestObject::new(100, "first"), "iface".to_string(), 1, None).unwrap();
        let result = registry.new_object(1, 100, TestObject::new(100, "second"), "iface".to_string(), 1, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Object ID 100 already exists.");
    }

    #[test]
    fn test_new_client_object_id_zero() {
        let mut registry = ObjectRegistry::new();
        let result = registry.new_object(1, 0, TestObject::new(0, "id_zero"), "iface".to_string(), 1, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Object ID 0 is invalid (null object).");
    }

    #[test]
    fn test_new_client_object_id_out_of_range() {
        let mut registry = ObjectRegistry::new();
        let result = registry.new_object(1, SERVER_ID_MIN, TestObject::new(SERVER_ID_MIN, "server_range_obj"), "iface".to_string(), 1, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of the allowed client range"));
    }

    #[test]
    fn test_new_server_object_success() {
        let mut registry = ObjectRegistry::new();
        let client_assoc_id = 1; // Associated with client 1
        let server_obj = AnotherTestObject { name: "my_server_object".to_string() };
        let interface_name = "server_object_interface".to_string();

        let result = registry.new_server_object(client_assoc_id, server_obj, interface_name.clone(), 1, None);
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
        assert_eq!(entry.interface_name, interface_name);
        assert_eq!(entry.parent_id, None);
    }

    #[test]
    fn test_new_multiple_server_objects_increment_id() {
        let mut registry = ObjectRegistry::new();
        let id1 = registry.new_server_object(1, AnotherTestObject { name: "obj1".into() }, "iface1".to_string(), 1, None).unwrap();
        let id2 = registry.new_server_object(1, AnotherTestObject { name: "obj2".into() }, "iface2".to_string(), 1, None).unwrap();
        assert_eq!(id1, SERVER_ID_MIN);
        assert_eq!(id2, SERVER_ID_MIN + 1);
        assert_eq!(registry.next_server_object_id, SERVER_ID_MIN + 2);
    }

    #[test]
    fn test_get_object_mut() {
        let mut registry = ObjectRegistry::new();
        let obj_id = 200;
        registry.new_object(1, obj_id, TestObject::new(obj_id, "mutable"), "iface_mut".to_string(), 1, None).unwrap();

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
        registry.new_object(1, obj_id, TestObject::new(obj_id, "to_destroy"), "iface_destroy".to_string(), 1, None).unwrap();

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

    #[test]
    fn test_create_object_with_parent() {
        let mut registry = ObjectRegistry::new();
        let parent_id = 100; // wl_display can't be a parent in new_object, use a regular client object
        registry.new_object(1, parent_id, TestObject::new(parent_id, "parent"), "parent_iface".to_string(), 1, None).unwrap();

        let child_id = 101;
        registry.new_object(1, child_id, TestObject::new(child_id, "child"), "child_iface".to_string(), 1, Some(parent_id)).unwrap();

        let child_entry = registry.get_entry(child_id).unwrap();
        assert_eq!(child_entry.parent_id, Some(parent_id));
    }

    #[test]
    fn test_destroy_parent_cascades_to_children() {
        let mut registry = ObjectRegistry::new();
        let client_id_val = 1;

        // Parent
        let parent_id = 100;
        registry.new_object(client_id_val, parent_id, TestObject::new(parent_id, "parent"), "parent_iface".to_string(), 1, None).unwrap();

        // Children
        let child1_id = 101;
        registry.new_object(client_id_val, child1_id, TestObject::new(child1_id, "child1"), "child_iface".to_string(), 1, Some(parent_id)).unwrap();
        let child2_id = 102;
        registry.new_object(client_id_val, child2_id, TestObject::new(child2_id, "child2"), "child_iface".to_string(), 1, Some(parent_id)).unwrap();

        // Grandchild
        let grandchild_id = 201;
        registry.new_object(client_id_val, grandchild_id, TestObject::new(grandchild_id, "grandchild"), "grandchild_iface".to_string(), 1, Some(child1_id)).unwrap();

        assert!(registry.get_object(parent_id).is_some());
        assert!(registry.get_object(child1_id).is_some());
        assert!(registry.get_object(child2_id).is_some());
        assert!(registry.get_object(grandchild_id).is_some());

        // Destroy parent
        let destroy_result = registry.destroy_object(parent_id);
        assert!(destroy_result.is_ok(), "Destroying parent failed: {:?}", destroy_result.err());

        // Check that parent and all children/grandchildren are gone
        assert!(registry.get_object(parent_id).is_none(), "Parent should be destroyed.");
        assert!(registry.get_object(child1_id).is_none(), "Child 1 should be destroyed due to cascade.");
        assert!(registry.get_object(child2_id).is_none(), "Child 2 should be destroyed due to cascade.");
        assert!(registry.get_object(grandchild_id).is_none(), "Grandchild should be destroyed due to cascade.");
    }

    #[test]
    fn test_destroy_child_does_not_destroy_parent() {
        let mut registry = ObjectRegistry::new();
        let client_id_val = 1;
        let parent_id = 100;
        let child_id = 101;

        registry.new_object(client_id_val, parent_id, TestObject::new(parent_id, "parent"), "parent_iface".to_string(), 1, None).unwrap();
        registry.new_object(client_id_val, child_id, TestObject::new(child_id, "child"), "child_iface".to_string(), 1, Some(parent_id)).unwrap();

        assert!(registry.get_object(parent_id).is_some());
        assert!(registry.get_object(child_id).is_some());

        // Destroy child
        let destroy_result = registry.destroy_object(child_id);
        assert!(destroy_result.is_ok());

        // Check child is gone, parent remains
        assert!(registry.get_object(child_id).is_none(), "Child should be destroyed.");
        assert!(registry.get_object(parent_id).is_some(), "Parent should NOT be destroyed.");
    }
}
