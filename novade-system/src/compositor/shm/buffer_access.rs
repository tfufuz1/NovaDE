use smithay::reexports::wayland_server::protocol::wl_buffer;
use smithay::wayland::shm::{BufferData as SmithayBufferData, with_buffer_contents as smithay_with_buffer_contents};
use super::errors::ShmError; // Ensure this path is correct if errors.rs is in the same shm module

/// Provides safe access to the contents of an SHM buffer.
///
/// The callback `F` receives a pointer to the raw buffer data, its length in bytes,
/// and metadata about the buffer (`SmithayBufferData`).
/// The pointer is only valid for the duration of the callback.
/// The callback `F` is expected to return a `Result<T, E>`, where `E` can be converted into `ShmError`.
pub fn with_shm_buffer_contents<F, T, E>(
    buffer: &wl_buffer::WlBuffer,
    callback: F,
) -> Result<T, ShmError>
where
    F: FnOnce(*const u8, usize, &SmithayBufferData) -> Result<T, E>,
    E: Into<ShmError>,
{
    // smithay::wayland::shm::with_buffer_contents calls the provided closure.
    // If the buffer is not a valid SHM buffer or access fails, it returns Err(BufferAccessError).
    // If access is successful, it returns Ok(result_of_closure).
    // In our case, the closure (callback) itself returns a Result<T, E>.
    match smithay_with_buffer_contents(buffer, |ptr, len, data| {
        // Execute the user's callback
        callback(ptr, len, data)
    }) {
        Ok(inner_result) => {
            // The callback was successfully called. Now, convert its result.
            // inner_result is Result<T, E>
            inner_result.map_err(Into::into) // Converts E to ShmError
        }
        Err(buffer_access_error) => {
            // Accessing the buffer itself failed (e.g., not an SHM buffer)
            Err(ShmError::from(buffer_access_error)) // Converts BufferAccessError to ShmError
        }
    }
}
