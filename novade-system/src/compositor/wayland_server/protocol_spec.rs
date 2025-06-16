use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgumentType {
    Int,
    Uint,
    Fixed,
    String,
    Object,
    NewId,
    Array,
    Fd,
}

#[derive(Debug, Clone)]
pub struct ArgumentSpec {
    pub name: String,
    pub arg_type: ArgumentType,
    pub interface: Option<String>, // Name of the interface for 'Object' or 'NewId' types
    // pub allow_null: bool, // Wayland spec sometimes mentions "allow_null" for objects. Future enhancement.
}

#[derive(Debug, Clone)]
pub struct RequestSpec {
    pub name: String,
    pub opcode: u16,
    pub args: Vec<ArgumentSpec>,
    // pub since: u32, // Protocol version since this request is available. Future.
    // pub type: Option<String>, // For some requests like wl_registry.bind that create an object not via NewId arg. Future.
}

#[derive(Debug, Clone)]
pub struct EventSpec {
    pub name: String,
    pub opcode: u16,
    pub args: Vec<ArgumentSpec>,
    // pub since: u32, // Future.
}

#[derive(Debug, Clone)]
pub struct InterfaceSpec {
    pub name: String,
    pub version: u32,
    pub requests: Vec<RequestSpec>,
    pub events: Vec<EventSpec>,
}

#[derive(Debug, Default)]
pub struct ProtocolManager {
    interfaces: HashMap<String, Arc<InterfaceSpec>>,
}

impl ProtocolManager {
    pub fn new() -> Self {
        Self {
            interfaces: HashMap::new(),
        }
    }

    pub fn load_protocol(&mut self, spec: InterfaceSpec) {
        println!("[ProtocolManager] Loaded protocol specification for interface: {}", spec.name);
        self.interfaces.insert(spec.name.clone(), Arc::new(spec));
    }

    pub fn get_interface(&self, name: &str) -> Option<Arc<InterfaceSpec>> {
        self.interfaces.get(name).cloned()
    }

    /// Get the specification for a request.
    pub fn get_request_spec(&self, interface_name: &str, opcode: u16) -> Option<&RequestSpec> {
        self.interfaces.get(interface_name).and_then(|iface_spec| {
            iface_spec.requests.iter().find(|req| req.opcode == opcode)
        })
    }

    /// Get the specification for an event.
    #[allow(dead_code)] // May be used later by server for sending events
    pub fn get_event_spec(&self, interface_name: &str, opcode: u16) -> Option<&EventSpec> {
        self.interfaces.get(interface_name).and_then(|iface_spec| {
            iface_spec.events.iter().find(|evt| evt.opcode == opcode)
        })
    }
}

// Function to load core Wayland protocols manually
pub fn load_core_protocols(pm: &mut ProtocolManager) {
    // wl_display interface
    let wl_display_spec = InterfaceSpec {
        name: "wl_display".to_string(),
        version: 1,
        requests: vec![
            RequestSpec {
                name: "sync".to_string(),
                opcode: 0,
                args: vec![ArgumentSpec {
                    name: "callback".to_string(),
                    arg_type: ArgumentType::NewId,
                    interface: Some("wl_callback".to_string()),
                }],
            },
            RequestSpec {
                name: "get_registry".to_string(),
                opcode: 1,
                args: vec![ArgumentSpec {
                    name: "registry".to_string(),
                    arg_type: ArgumentType::NewId,
                    interface: Some("wl_registry".to_string()),
                }],
            },
        ],
        events: vec![
            EventSpec { // error event
                name: "error".to_string(), opcode: 0,
                args: vec![
                    ArgumentSpec { name: "object_id".to_string(), arg_type: ArgumentType::Object, interface: None }, // Interface of object_id is context-dependent
                    ArgumentSpec { name: "code".to_string(), arg_type: ArgumentType::Uint, interface: None },
                    ArgumentSpec { name: "message".to_string(), arg_type: ArgumentType::String, interface: None },
                ],
            },
            EventSpec { // delete_id event
                name: "delete_id".to_string(), opcode: 1,
                args: vec![
                    ArgumentSpec { name: "id".to_string(), arg_type: ArgumentType::Uint, interface: None },
                ],
            }
        ],
    };
    pm.load_protocol(wl_display_spec);

    // wl_callback interface
    let wl_callback_spec = InterfaceSpec {
        name: "wl_callback".to_string(),
        version: 1,
        requests: vec![], // No requests for wl_callback
        events: vec![EventSpec {
            name: "done".to_string(),
            opcode: 0,
            args: vec![ArgumentSpec {
                name: "callback_data".to_string(), // data associated with the callback
                arg_type: ArgumentType::Uint,
                interface: None,
            }],
        }],
    };
    pm.load_protocol(wl_callback_spec);

    // wl_registry interface
    let wl_registry_spec = InterfaceSpec {
        name: "wl_registry".to_string(),
        version: 1,
        requests: vec![RequestSpec {
            name: "bind".to_string(),
            opcode: 0,
            args: vec![
                ArgumentSpec {
                    name: "name".to_string(), // The numeric 'name' of the global (an ID)
                    arg_type: ArgumentType::Uint,
                    interface: None,
                },
                ArgumentSpec {
                    name: "interface".to_string(), // The actual interface name string client wants to bind
                    arg_type: ArgumentType::String,
                    interface: None,
                },
                ArgumentSpec {
                    name: "version".to_string(), // Version client wants
                    arg_type: ArgumentType::Uint,
                    interface: None,
                },
                ArgumentSpec {
                    name: "id".to_string(), // new_id for the bound object
                    arg_type: ArgumentType::NewId,
                    // The interface for this new_id is determined by the 'interface' string argument above.
                    // So, `interface: None` here is appropriate as it's not fixed.
                    interface: None,
                },
            ],
        }],
        events: vec![
            EventSpec { // global event
                name: "global".to_string(), opcode: 0,
                args: vec![
                    ArgumentSpec { name: "name".to_string(), arg_type: ArgumentType::Uint, interface: None },
                    ArgumentSpec { name: "interface".to_string(), arg_type: ArgumentType::String, interface: None },
                    ArgumentSpec { name: "version".to_string(), arg_type: ArgumentType::Uint, interface: None },
                ],
            },
            EventSpec { // global_remove event
                name: "global_remove".to_string(), opcode: 1,
                args: vec![
                    ArgumentSpec { name: "name".to_string(), arg_type: ArgumentType::Uint, interface: None },
                ],
            }
        ],
    };
    pm.load_protocol(wl_registry_spec);

    // wl_compositor interface
    let wl_compositor_spec = InterfaceSpec {
        name: "wl_compositor".to_string(),
        version: 4, // Example version, check spec
        requests: vec![
            RequestSpec {
                name: "create_surface".to_string(),
                opcode: 0,
                args: vec![ArgumentSpec {
                    name: "id".to_string(),
                    arg_type: ArgumentType::NewId,
                    interface: Some("wl_surface".to_string()),
                }],
            },
            RequestSpec {
                name: "create_region".to_string(),
                opcode: 1,
                args: vec![ArgumentSpec {
                    name: "id".to_string(),
                    arg_type: ArgumentType::NewId,
                    interface: Some("wl_region".to_string()),
                }],
            },
        ],
        events: vec![], // No events for wl_compositor
    };
    pm.load_protocol(wl_compositor_spec);

    // wl_surface interface
    let wl_surface_spec = InterfaceSpec {
        name: "wl_surface".to_string(),
        version: 4, // Example version
        requests: vec![
            RequestSpec { name: "destroy".to_string(), opcode: 0, args: vec![] },
            RequestSpec {
                name: "attach".to_string(), opcode: 1,
                args: vec![
                    ArgumentSpec { name: "buffer".to_string(), arg_type: ArgumentType::Object, interface: Some("wl_buffer".to_string()) }, // allow_null=true
                    ArgumentSpec { name: "x".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "y".to_string(), arg_type: ArgumentType::Int, interface: None },
                ],
            },
            RequestSpec {
                name: "damage".to_string(), opcode: 2,
                args: vec![
                    ArgumentSpec { name: "x".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "y".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "width".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "height".to_string(), arg_type: ArgumentType::Int, interface: None },
                ],
            },
            RequestSpec { // frame
                name: "frame".to_string(), opcode: 3,
                args: vec![ArgumentSpec { name: "callback".to_string(), arg_type: ArgumentType::NewId, interface: Some("wl_callback".to_string()) }],
            },
            RequestSpec { // set_opaque_region
                name: "set_opaque_region".to_string(), opcode: 4,
                args: vec![ArgumentSpec { name: "region".to_string(), arg_type: ArgumentType::Object, interface: Some("wl_region".to_string()) }], // allow_null=true
            },
            RequestSpec { // set_input_region
                name: "set_input_region".to_string(), opcode: 5,
                args: vec![ArgumentSpec { name: "region".to_string(), arg_type: ArgumentType::Object, interface: Some("wl_region".to_string()) }], // allow_null=true
            },
            RequestSpec { name: "commit".to_string(), opcode: 6, args: vec![] },
            RequestSpec { // set_buffer_transform (since v2)
                name: "set_buffer_transform".to_string(), opcode: 7,
                args: vec![ArgumentSpec { name: "transform".to_string(), arg_type: ArgumentType::Int, interface: None }],
            },
            RequestSpec { // set_buffer_scale (since v3)
                name: "set_buffer_scale".to_string(), opcode: 8,
                args: vec![ArgumentSpec { name: "scale".to_string(), arg_type: ArgumentType::Int, interface: None }],
            },
            RequestSpec { // damage_buffer (since v4)
                name: "damage_buffer".to_string(), opcode: 9,
                args: vec![
                    ArgumentSpec { name: "x".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "y".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "width".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "height".to_string(), arg_type: ArgumentType::Int, interface: None },
                ],
            },
        ],
        events: vec![
             EventSpec { name: "enter".to_string(), opcode: 0, args: vec![
                 ArgumentSpec { name: "output".to_string(), arg_type: ArgumentType::Object, interface: Some("wl_output".to_string()) }
             ]},
             EventSpec { name: "leave".to_string(), opcode: 1, args: vec![
                 ArgumentSpec { name: "output".to_string(), arg_type: ArgumentType::Object, interface: Some("wl_output".to_string()) }
             ]},
        ],
    };
    pm.load_protocol(wl_surface_spec);

    // wl_region interface
    let wl_region_spec = InterfaceSpec {
        name: "wl_region".to_string(),
        version: 1,
        requests: vec![
            RequestSpec { name: "destroy".to_string(), opcode: 0, args: vec![] },
            RequestSpec {
                name: "add".to_string(), opcode: 1,
                args: vec![
                    ArgumentSpec { name: "x".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "y".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "width".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "height".to_string(), arg_type: ArgumentType::Int, interface: None },
                ],
            },
            RequestSpec {
                name: "subtract".to_string(), opcode: 2,
                args: vec![
                    ArgumentSpec { name: "x".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "y".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "width".to_string(), arg_type: ArgumentType::Int, interface: None },
                    ArgumentSpec { name: "height".to_string(), arg_type: ArgumentType::Int, interface: None },
                ],
            },
        ],
        events: vec![], // No events for wl_region
    };
    pm.load_protocol(wl_region_spec);

    // wl_shm interface
    let wl_shm_spec = InterfaceSpec {
        name: "wl_shm".to_string(),
        version: 1,
        requests: vec![RequestSpec {
            name: "create_pool".to_string(),
            opcode: 0,
            args: vec![
                ArgumentSpec { name: "id".to_string(), arg_type: ArgumentType::NewId, interface: Some("wl_shm_pool".to_string()) },
                ArgumentSpec { name: "fd".to_string(), arg_type: ArgumentType::Fd, interface: None },
                ArgumentSpec { name: "size".to_string(), arg_type: ArgumentType::Int, interface: None }, // size is i32
            ],
        }],
        events: vec![EventSpec { // format event
            name: "format".to_string(), opcode: 0,
            args: vec![ArgumentSpec { name: "format".to_string(), arg_type: ArgumentType::Uint, interface: None }],
        }],
    };
    pm.load_protocol(wl_shm_spec);

    // wl_shm_pool interface
    let wl_shm_pool_spec = InterfaceSpec {
        name: "wl_shm_pool".to_string(),
        version: 1,
        requests: vec![
            RequestSpec {
                name: "create_buffer".to_string(),
                opcode: 0,
                args: vec![
                    ArgumentSpec { name: "id".to_string(), arg_type: ArgumentType::NewId, interface: Some("wl_buffer".to_string()) },
                    ArgumentSpec { name: "offset".to_string(), arg_type: ArgumentType::Int, interface: None }, // i32
                    ArgumentSpec { name: "width".to_string(), arg_type: ArgumentType::Int, interface: None },  // i32
                    ArgumentSpec { name: "height".to_string(), arg_type: ArgumentType::Int, interface: None }, // i32
                    ArgumentSpec { name: "stride".to_string(), arg_type: ArgumentType::Int, interface: None }, // i32
                    ArgumentSpec { name: "format".to_string(), arg_type: ArgumentType::Uint, interface: None }, // u32 (enum wl_shm.format)
                ],
            },
            RequestSpec { name: "destroy".to_string(), opcode: 1, args: vec![] },
            RequestSpec { // resize
                name: "resize".to_string(), opcode: 2,
                args: vec![ArgumentSpec { name: "size".to_string(), arg_type: ArgumentType::Int, interface: None }], // i32
            },
        ],
        events: vec![], // No events for wl_shm_pool
    };
    pm.load_protocol(wl_shm_pool_spec);

    // wl_buffer interface (events only, requests are via specific buffer types like from wl_shm_pool)
    let wl_buffer_spec = InterfaceSpec {
        name: "wl_buffer".to_string(),
        version: 1,
        requests: vec![
            // The 'destroy' request is standard for objects that can be destroyed.
             RequestSpec { name: "destroy".to_string(), opcode: 0, args: vec![] },
        ],
        events: vec![
            EventSpec { name: "release".to_string(), opcode: 0, args: vec![] }
        ],
    };
    pm.load_protocol(wl_buffer_spec);

}

// Add this to `novade-system/src/compositor/wayland_server/mod.rs` or a top-level module file
// pub mod protocol_spec;
// pub use protocol_spec::InterfaceSpec; // etc.
// For now, this file will be part of the wayland_server module.
// It will be `wayland_server::protocol_spec::...`

// --- Standard Wayland Error Enums ---

/// Errors for wl_display interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WlDisplayError {
    InvalidObject = 0,  // server couldn't find object
    InvalidMethod = 1,  // method doesn't exist on the object
    NoMemory = 2,       // server is out of memory
    Implementation = 3, // generic implementation error
    // Note: These are standard error codes. A compositor might define more specific ones.
}

/// Errors for wl_shm interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WlShmError {
    InvalidFormat = 0, // buffer format is not known
    InvalidStride = 1, // invalid buffer stride
    InvalidFd = 2,     // invalid file descriptor
}

// Add other interface-specific error enums as they become relevant.
// For example, for xdg_shell:
// pub enum XdgWmBaseError { Role = 0, DefunctSurfaces = 1, ... }
