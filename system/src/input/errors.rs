use thiserror::Error;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Seat creation failed: {0}")]
    SeatCreationFailed(String),
    #[error("Failed to add capability '{capability}' to seat '{seat_name}': {source}")]
    CapabilityAdditionFailed {
        seat_name: String,
        capability: String,
        // Box the source to keep InputError Sized if the source is dyn Error
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    #[error("XKB configuration error for seat '{seat_name}': {message}")]
    XkbConfigError { seat_name: String, message: String },
    #[error("Libinput context creation or configuration error: {0}")]
    LibinputError(String),
    #[error("Libinput session management error: {0}")]
    LibinputSessionError(#[from] std::io::Error), // For open_restricted/close_restricted errors
    #[error("Seat '{0}' not found")]
    SeatNotFound(String),
    #[error("Keyboard handle not found for seat '{0}'")]
    KeyboardHandleNotFound(String),
    #[error("Pointer handle not found for seat '{0}'")]
    PointerHandleNotFound(String),
    #[error("Touch handle not found for seat '{0}'")]
    TouchHandleNotFound(String),
    #[error("Input event source setup error: {0}")]
    EventSourceSetupError(String),
    #[error("Internal input system error: {0}")]
    InternalError(String),
}

// Helper for CapabilityAdditionFailed
impl InputError {
    pub fn capability_addition_failed<E>(
        seat_name: impl Into<String>,
        capability: impl Into<String>,
        source: E,
    ) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        InputError::CapabilityAdditionFailed {
            seat_name: seat_name.into(),
            capability: capability.into(),
            source: Box::new(source),
        }
    }
}
