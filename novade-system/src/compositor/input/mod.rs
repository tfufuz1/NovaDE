//! Input handling subsystem for the Novade Wayland compositor.
//!
//! This module is responsible for initializing and managing input devices using `libinput`,
//! handling keyboard layouts and key mapping via `xkbcommon`, and translating raw input
//! events into a structured format suitable for consumption by the compositor logic.
//!
//! Key components:
//! - `InputHandler`: The main struct for managing input contexts and dispatching events.
//! - `InputError`: Defines specific errors that can occur during input processing.
//! - `data_types`: Contains structs and enums for various input events and states.

// Re-export public types for easier access from parent modules.
pub mod error;
pub mod data_types;
pub mod input_handler;

pub use error::InputError;
pub use data_types::{
    InputEvent, KeyboardEventInfo, KeyState, ModifiersState,
    PointerMotionInfo, PointerButtonInfo, ButtonState, PointerAxisInfo, AxisMovement,
    TouchDownInfo, TouchUpInfo, TouchMotionInfo,
    KeyboardState, PointerState, TouchState, Seat,
};
pub use input_handler::InputHandler;

use tracing::info;

/// Initializes the input subsystem.
///
/// This function creates and configures an `InputHandler` instance, which will
/// set up `libinput` for device discovery and event reading, and `xkbcommon` for
// keyboard map handling.
///
/// # Arguments
///
/// * `seat_name`: A string slice representing the name of the seat to initialize (e.g., "seat0").
///
/// # Returns
///
/// A `Result` containing the initialized `InputHandler` or an `InputError` if
/// initialization fails.
pub fn initialize_input_subsystem(seat_name: String) -> Result<InputHandler, InputError> {
    info!("Initializing Novade input subsystem for seat: {}", seat_name);
    InputHandler::new(seat_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test is primarily for ensuring the initialization function can be called.
    // It relies on the same conditions as InputHandler::new() regarding udev availability.
    #[test]
    fn test_initialize_input_subsystem() {
        match initialize_input_subsystem("seat_test_mod".to_string()) {
            Ok(_) => {
                info!("Input subsystem initialized successfully via mod function.");
            }
            Err(InputError::LibinputInitialization(msg)) if msg.contains("udev") || msg.contains("seat") => {
                tracing::warn!("Input subsystem initialization failed as expected in some environments: {}", msg);
            }
            Err(e) => {
                panic!("initialize_input_subsystem() failed with unexpected error: {:?}", e);
            }
        }
    }
}
