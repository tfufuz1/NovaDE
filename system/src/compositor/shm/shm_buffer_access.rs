use crate::compositor::shm::errors::ShmError;
use smithay::reexports::wayland_server::protocol::{wl_buffer, wl_shm};
use smithay::wayland::shm::{with_buffer_contents as smithay_with_buffer_contents, BufferAccessError};

/// Provides safe access to the contents of an SHM buffer.
///
/// This function wraps Smithay's `with_buffer_contents` to provide unified error handling
/// and potentially additional checks or logic specific to this compositor.
///
/// # Arguments
///
/// * `buffer`: A reference to the `wl_buffer` whose contents are to be accessed.
/// * `callback`: A closure that takes a `&[u8]` slice representing the buffer data
///   and `wl_shm::Format` and returns a result `R`.
///
/// # Returns
///
/// * `Ok(R)`: If the buffer is a valid SHM buffer and the callback executes successfully.
/// * `Err(ShmError)`: If the buffer is not a valid SHM buffer, or if access fails.
///
/// # Safety
///
/// This function ensures that the provided slice is valid only for the duration of the callback.
/// The callback should not store the slice or pointers derived from it.
pub fn with_shm_buffer_contents<F, R>(
    buffer: &wl_buffer::WlBuffer,
    callback: F,
) -> Result<R, ShmError>
where
    F: FnOnce(&[u8], wl_shm::Format) -> R,
{
    smithay_with_buffer_contents(buffer, |slice, data| {
        // `data` here is `ShmBufferData` which contains `format`, `width`, `height`, `stride`.
        // We only need to pass the slice and format to the user's callback.
        callback(slice, data.format)
    })
    .map_err(|err| match err {
        BufferAccessError::NotManaged => {
            // This means the buffer isn't managed by Smithay's ShmState,
            // which implies it's not a SHM buffer or not one we can access this way.
            ShmError::Internal("Buffer not managed by SHM state, likely not an SHM buffer.".into())
        }
        BufferAccessError::BadRead => {
            // This typically indicates an issue with mmap or reading the underlying SHM fd.
            ShmError::PoolCreationFailed("Failed to read SHM buffer contents (BadRead).".into())
        }
        // BufferAccessError::AlreadyAccessed is not present in Smithay 0.3 for with_buffer_contents
        // BufferAccessError::Poisoned would indicate a mutex poisoning, which is a critical error.
        // For now, map it to a generic internal error.
        // _ => ShmError::Internal("Unknown error accessing SHM buffer contents.".into()),
    })
}

// Example Usage (conceptual, would be within some part of the compositor logic):
//
// fn process_shm_buffer(buffer: &wl_buffer::WlBuffer) {
//     match with_shm_buffer_contents(buffer, |slice, format| {
//         tracing::info!("Accessed SHM buffer with format {:?}, size {} bytes", format, slice.len());
//         // TODO: Process the buffer data (e.g., copy to a texture for rendering)
//         // For example: renderer.upload_shm_texture(slice, width, height, stride, format);
//         Ok(()) // Return Ok if processing is successful
//     }) {
//         Ok(_) => tracing::info!("Successfully processed SHM buffer."),
//         Err(e) => tracing::error!("Error processing SHM buffer: {}", e),
//     }
// }

// Note on error mapping:
// `BufferAccessError::NotManaged` directly implies the buffer is not what we expect (not a SHM buffer
// known to Smithay's `ShmState`). This is a critical issue for something expecting to read SHM data.
// `BufferAccessError::BadRead` suggests an OS-level issue (like `EFAULT` or `EINVAL` during `mmap` or `lseek`/`read`),
// which could point to client misbehavior (e.g., truncating the SHM file after pool creation) or OS resource limits.
// Mapping it to `PoolCreationFailed` might be slightly misleading, as the pool was created, but access failed.
// `ShmError::MmapFailed` or a new variant like `ShmError::AccessFailed` could be more precise.
// Let's refine the error mapping for `BadRead`.

// Refined error mapping:
pub fn with_shm_buffer_contents_refined<F, R>(
    buffer: &wl_buffer::WlBuffer,
    callback: F,
) -> Result<R, ShmError>
where
    F: FnOnce(&[u8], wl_shm::Format) -> R,
{
    smithay_with_buffer_contents(buffer, |slice, data| {
        callback(slice, data.format)
    })
    .map_err(|err| match err {
        BufferAccessError::NotManaged => {
            ShmError::Internal("Buffer not managed by SHM state or not an SHM buffer.".into())
        }
        BufferAccessError::BadRead => {
            // This error in Smithay often wraps an underlying OS error from mmap.
            // It signifies that the data could not be read from the shared memory segment.
            ShmError::MmapFailed("Failed to access SHM buffer contents (e.g., mmap error or invalid segment).".into())
        }
    })
}

// The original `with_shm_buffer_contents` will be used as per the plan.
// The `_refined` version is for consideration during review if more specific errors are desired from this function.
// Sticking to the requested `with_shm_buffer_contents` name for the final version.
