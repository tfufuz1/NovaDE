use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Seat creation failed or seat '{seat_name}' already exists.")]
    SeatCreationFailed { seat_name: String },

    #[error("Failed to add capability '{capability}' to seat '{seat_name}': {source_message}")]
    CapabilityAdditionFailed {
        seat_name: String,
        capability: String,
        source_message: String, // Simplified to string to avoid complex Box<dyn Error> in enum for now
    },

    #[error("XKB configuration error for seat '{seat_name}': {message}")]
    XkbConfigError { seat_name: String, message: String },

    #[error("Libinput backend initialization or processing error: {0}")]
    LibinputError(String),

    #[error("Libinput session error: {0}")]
    LibinputSessionError(#[from] io::Error),

    #[error("Seat '{0}' not found.")]
    SeatNotFound(String),

    #[error("Keyboard handle not found for seat '{0}'.")]
    KeyboardHandleNotFound(String),

    #[error("Pointer handle not found for seat '{0}'.")]
    PointerHandleNotFound(String),

    #[error("Touch handle not found for seat '{0}'.")]
    TouchHandleNotFound(String),

    #[error("Failed to initialize input event source in event loop: {0}")]
    EventSourceSetupError(String),

    #[error("Internal error in input system: {0}")]
    InternalError(String),

    #[error("Device has no slot ID for a touch event.")]
    TouchNoSlotId,

    #[error("XKB Keymap compilation failed: {0}")]
    XkbKeymapCompilationFailed(String),
}
