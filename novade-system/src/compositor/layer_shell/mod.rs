// This file will export the public interface of the layer_shell module.

// Re-export key components that might be needed by the main compositor logic
pub use self::state::LayerSurfaceData;
// The handler itself is usually not directly called from outside,
// but the trait implementation on DesktopState makes it active.
// So, no need to pub use handler typically, unless there are specific functions to expose.

// Define module structure
pub mod handler;
pub mod state;

// Define actual error types
pub mod error {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum LayerShellError {
        #[error("Invalid layer surface state for operation: {0}")]
        InvalidState(String),
        #[error("A Wayland protocol error occurred: {0}")]
        Protocol(String),
        #[error("The requested output was not found or is not suitable for the layer surface")]
        OutputUnavailable,
        // Add more specific errors as they become identified
    }
}

// Optional: Re-export the error type for easier access if needed elsewhere
pub use self::error::LayerShellError;
