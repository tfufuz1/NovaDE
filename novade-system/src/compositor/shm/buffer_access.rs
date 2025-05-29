use smithay::reexports::wayland_server::protocol::wl_buffer;
use smithay::wayland::shm;
use super::errors::ShmError; // Assuming errors.rs is in the same shm module

/// Provides safe access to the contents of an SHM buffer.
///
/// This function wraps `smithay::wayland::shm::with_buffer_contents`, mapping errors
/// to the module-specific `ShmError` type.
///
/// Type Parameters:
/// - `F`: The callback function type.
/// - `T`: The success type returned by the callback.
/// - `E`: The error type returned by the callback, which must be convertible into `ShmError`.
///
/// Parameters:
/// - `buffer`: The Wayland buffer to access.
/// - `callback`: A function that takes a pointer to the buffer data, its length,
///               and metadata, and returns a `Result<T, E>`.
///
/// Returns:
/// - `Ok(T)` if the buffer access and callback are successful.
/// - `Err(ShmError)` if buffer access fails or the callback returns an error.
pub fn with_shm_buffer_contents<F, T, E>(
    buffer: &wl_buffer::WlBuffer,
    callback: F,
) -> Result<T, ShmError>
where
    F: FnOnce(*const u8, usize, &shm::BufferData) -> Result<T, E>,
    E: Into<ShmError>,
{
    shm::with_buffer_contents(buffer, |ptr, len, data| {
        callback(ptr, len, data).map_err(Into::into)
    })
    .map_err(ShmError::from) // This will use `impl From<BufferAccessError> for ShmError`
}
