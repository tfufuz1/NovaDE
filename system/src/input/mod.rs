pub mod errors;
pub mod seat_manager;
pub mod libinput_handler;
pub mod keyboard;
pub mod pointer; // Added pointer module
pub mod touch;   // Added touch module

// Re-export key types or functions if needed at the `system::input` level.
pub use self::errors::InputError;
pub use self::seat_manager::create_seat;
pub use self::libinput_handler::{LibinputSessionManager, init_libinput_backend};
pub use self::libinput_handler::event_dispatcher::process_input_event;
pub use self::keyboard::{XkbKeyboardData, handle_keyboard_key_event, handle_keyboard_repeat, set_keyboard_focus}; // Added handle_keyboard_repeat from prev step
pub use self::pointer::{handle_pointer_motion_event, handle_pointer_motion_absolute_event, handle_pointer_button_event, handle_pointer_axis_event}; // Added pointer handlers
pub use self::touch::{handle_touch_down_event, handle_touch_up_event, handle_touch_motion_event, handle_touch_frame_event, handle_touch_cancel_event, TouchFocusData}; // Added touch handlers and TouchFocusData
