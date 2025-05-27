use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShmError {
    #[error("Failed to create SHM pool: {0}")]
    PoolCreationFailed(String), // Consider more specific error types from nix or OS errors
    #[error("Invalid file descriptor for SHM")]
    InvalidFd,
    #[error("SHM pool size is too small: {size}, minimum required: {min_size}")]
    PoolTooSmall { size: usize, min_size: usize },
    #[error("SHM buffer access out of bounds: offset {offset} + size {size} > pool size {pool_size}")]
    AccessOutOfBounds {
        offset: usize,
        size: usize,
        pool_size: usize,
    },
    #[error("Mmap failed: {0}")]
    MmapFailed(String), // Or nix::Error directly if appropriate
    #[error("Internal SHM error: {0}")]
    Internal(String),
}
