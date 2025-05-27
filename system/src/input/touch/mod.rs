// system/src/input/touch/mod.rs
pub mod touch_event_translator;
// pub mod focus; // If more complex, separate touch focus logic is needed

pub use self::touch_event_translator::{
    handle_touch_down_event,
    handle_touch_up_event,
    handle_touch_motion_event,
    handle_touch_frame_event,
    handle_touch_cancel_event
};
