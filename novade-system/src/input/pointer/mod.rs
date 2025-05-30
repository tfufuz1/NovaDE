// novade-system/src/input/pointer/mod.rs
pub mod event_translator;
pub mod focus; // For update_pointer_focus_and_send_motion

pub use event_translator::{
    handle_pointer_motion_event,
    handle_pointer_motion_absolute_event,
    handle_pointer_button_event,
    handle_pointer_axis_event,
    find_surface_and_coords_at_global_point,
};
pub use focus::update_pointer_focus_and_send_motion;
