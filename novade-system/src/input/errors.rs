use thiserror::Error;
use std::io::Error as StdIoError;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Failed to create seat: {0}")]
    SeatCreationFailed(String),

    #[error("Failed to add capability '{capability}' to seat '{seat_name}': {source}")]
    CapabilityAdditionFailed {
        seat_name: String,
        capability: String,
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    #[error("XKB configuration error for seat '{seat_name}': {message}")]
    XkbConfigError {
        seat_name: String,
        message: String,
    },

    #[error("Libinput error: {0}")]
    LibinputError(String),

    #[error("Libinput session error: {0}")]
    LibinputSessionError(#[from] StdIoError),

    #[error("Seat not found: {0}")]
    SeatNotFound(String),

    #[error("Keyboard handle not found: {0}")]
    KeyboardHandleNotFound(String),

    #[error("Pointer handle not found: {0}")]
    PointerHandleNotFound(String),

    #[error("Touch handle not found: {0}")]
    TouchHandleNotFound(String),

    #[error("Event source setup error: {0}")]
    EventSourceSetupError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    // Added for this subtask
    #[error("Configuration file error: {0}")]
    ConfigFileError(String),

    #[error("Event processing error: {0}")]
    EventProcessingError(String),

    #[error("Focus management error: {0}")]
    FocusError(String),
    // Note: LibinputError and LibinputSessionError might be less relevant for pure stubs
    // but are kept as they exist. DeviceInitializationFailed can be a new variant if needed.
}
