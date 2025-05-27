use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum DisplayLoopError {
    #[error("Failed to register Wayland event source with calloop: {0}")]
    EventSourceRegistrationFailed(#[from] io::Error), // Covers issues with Generic::from_fd or loop_handle.insert_source
    #[error("Error dispatching Wayland client events: {0}")]
    DispatchError(#[from] smithay::reexports::wayland_server::DispatchError), // From display_handle.dispatch_clients
    #[error("Failed to flush Wayland clients: {0}")]
    FlushClientsError(#[from] smithay::reexports::wayland_server::FlushError), // From display_handle.flush_clients
    #[error("Wayland display FD not available or invalid.")]
    WaylandFdError,
    #[error("Internal error related to display loop: {0}")]
    Internal(String),
}
