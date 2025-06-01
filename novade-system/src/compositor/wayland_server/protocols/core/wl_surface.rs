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


// TODO: Define handlers and event/request argument structures
