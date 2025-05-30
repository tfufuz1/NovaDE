// novade-system/src/input/mod.rs

pub mod errors;
pub use errors::InputError;

pub mod libinput_handler;
pub mod input_dispatcher;
pub mod keyboard_layout; // Added this line

pub mod keyboard;
pub mod seat_manager;
pub mod event_dispatcher; // Added
pub mod pointer; // Added
pub mod touch; // Added

// Optional: Re-export NovadeLibinputManager if it's part of the input module's public API
// pub use libinput_handler::NovadeLibinputManager;

// Optional: Re-export InputDispatcher if it's part of the input module's public API
// pub use input_dispatcher::InputDispatcher;

// Optional: Re-export KeyboardLayoutManager if it's part of the input module's public API
// pub use keyboard_layout::KeyboardLayoutManager;
