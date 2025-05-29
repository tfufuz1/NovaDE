use smithay::reexports::wayland_server::protocol::wl_shm;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShmError {
    #[error("SHM Pool creation failed: {0}")]
    PoolCreationFailed(String),

    #[error("SHM Buffer creation failed: {0}")]
    BufferCreationFailed(String),

    #[error("Invalid SHM buffer format: {0:?}")]
    InvalidFormat(wl_shm::Format),

    #[error("SHM Buffer access error: {0}")]
    AccessError(#[from] smithay::wayland::shm::BufferAccessError),
    // CallbackError removed as per current subtask's ShmError definition
}
