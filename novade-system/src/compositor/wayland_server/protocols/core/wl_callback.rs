// In novade-system/src/compositor/wayland_server/protocols/core/wl_callback.rs
use crate::compositor::wayland_server::objects::Interface;

// Using Interface::new directly as it's const-compatible with nightly features,
// but for stable, make it a lazy_static or a fn. For now, assuming it's fine.
// Let's define them as functions returning Interface for broader compatibility.
pub fn wl_callback_interface() -> Interface { Interface::new("wl_callback") }
pub const WL_CALLBACK_VERSION: u32 = 1;

// Event opcodes for wl_callback
pub const EVT_DONE_OPCODE: u16 = 0;

#[derive(Debug)]
pub struct CallbackDoneEvent {
    pub callback_data: u32,
}
