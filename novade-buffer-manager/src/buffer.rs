//! Manages buffer objects and their properties.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Represents a unique identifier for a Wayland client.
///
/// This is a placeholder and might be replaced with a more robust client tracking mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(u64);

impl ClientId {
    /// Creates a new client ID.
    ///
    /// # Arguments
    /// * `id`: The raw `u64` value for this client ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Represents a unique identifier for a buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId(u64);

impl BufferId {
    /// Creates a new, unique `BufferId`.
    ///
    /// Note: Current implementation uses a simple atomic counter. This might be
    /// extended or replaced by a more sophisticated ID generation scheme if needed,
    /// for example, one that reuses IDs or is specific to a Wayland display server context.
    fn new_unique() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
        BufferId(NEXT_ID.fetch_add(1, Ordering::Relaxed) as u64)
    }
}

/// Specifies the underlying type or source of a buffer's memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    /// Buffer memory is managed via a shared memory mechanism (e.g., `wl_shm`).
    Shm,
    /// Buffer memory is represented by a DMA buffer file descriptor.
    DmaBuf,
    /// Buffer memory is an opaque GPU texture, managed by the rendering backend.
    GpuTexture,
}

/// Enumerates common pixel formats for buffers.
///
/// These formats typically align with Wayland's `wl_shm.format` and DRM formats.
/// This list can be extended as more formats are supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferFormat {
    /// 32-bit ARGB format, 8 bits per channel, alpha first.
    Argb8888,
    /// 32-bit XRGB format, 8 bits per channel, alpha ignored (X).
    Xrgb8888,
    /// YUV format, NV12 (2-plane Y followed by interleaved UV).
    Nv12,
    // Add other formats as needed, e.g., Yuyv, Rgb565, etc.
}

/// Holds detailed information about a specific buffer.
///
/// This includes its dimensions, format, type, and ownership details.
/// It also tracks the reference count for managing the buffer's lifetime.
#[derive(Debug)]
pub struct BufferDetails {
    /// Unique identifier for this buffer.
    pub id: BufferId,
    /// The type of the buffer (e.g., SHM, DMA-BUF).
    pub buffer_type: BufferType,
    /// Width of the buffer in pixels.
    pub width: u32,
    /// Height of the buffer in pixels.
    pub height: u32,
    /// Stride of the buffer in bytes (bytes per row).
    pub stride: u32,
    /// Pixel format of the buffer.
    pub format: BufferFormat,
    /// Atomic reference counter for this buffer.
    /// When this count reaches zero, the buffer can be considered for deallocation
    /// or release back to the client.
    pub ref_count: AtomicUsize,
    /// Optional ID of the client that originally created or owns this buffer.
    /// This can be used for validation purposes (e.g., ensuring a client
    /// only attaches buffers it owns).
    pub client_owner_id: Option<ClientId>,
    // data: Vec<u8>, // Placeholder for actual buffer data (e.g., for SHM this might be a MappedRegion).
    //                 // For DMA-BUF, it would be one or more file descriptors and offsets/strides per plane.
}

impl BufferDetails {
    /// Creates a new `BufferDetails` instance.
    ///
    /// A unique `BufferId` is generated automatically. The initial reference count is set to 1,
    /// representing the "owner" or the entity that registered it (typically the `BufferManager` itself,
    /// or the client that created it via a Wayland request).
    ///
    /// # Arguments
    /// * `buffer_type`: The type of the buffer.
    /// * `width`: Width of the buffer in pixels. Must be positive.
    /// * `height`: Height of the buffer in pixels. Must be positive.
    /// * `stride`: Stride of the buffer in bytes. Must be sufficient for `width` and format.
    /// * `format`: Pixel format of the buffer.
    /// * `client_owner_id`: Optional `ClientId` of the buffer's owner.
    pub fn new(
        buffer_type: BufferType,
        width: u32,
        height: u32,
        stride: u32,
        format: BufferFormat,
        client_owner_id: Option<ClientId>,
    ) -> Self {
        // Basic validation for dimensions, though more strict checks might apply
        // depending on buffer type or usage context (e.g., in BufferManager::register_buffer).
        debug_assert!(width > 0, "Buffer width must be positive.");
        debug_assert!(height > 0, "Buffer height must be positive.");

        Self {
            id: BufferId::new_unique(),
            buffer_type,
            width,
            height,
            stride,
            format,
            ref_count: AtomicUsize::new(1), // Starts with a ref count of 1 (the owner)
            client_owner_id,
        }
    }

    /// Increments the atomic reference count of the buffer.
    pub fn increment_ref_count(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements the atomic reference count of the buffer.
    ///
    /// # Returns
    /// `true` if the reference count reached zero after decrementing, `false` otherwise.
    pub fn decrement_ref_count(&self) -> bool {
        self.ref_count.fetch_sub(1, Ordering::Relaxed) == 1
    }
}

/// Manages a collection of `BufferDetails` shared across the compositor.
///
/// It allows registering new buffers, retrieving their details, and managing their
/// lifecycle through reference counting.
#[derive(Default)]
pub struct BufferManager {
    buffers: HashMap<BufferId, Arc<Mutex<BufferDetails>>>,
}

impl BufferManager {
    /// Creates a new, empty `BufferManager`.
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    /// Registers a new buffer with the specified properties.
    ///
    /// This function typically corresponds to a client request to create a buffer
    /// (e.g., `wl_shm_pool.create_buffer` or importing a DMA-BUF).
    /// The `BufferManager` takes ownership of tracking this buffer.
    ///
    /// # Arguments
    /// * `buffer_type`: The type of the buffer.
    /// * `width`: Width in pixels.
    /// * `height`: Height in pixels.
    /// * `stride`: Stride in bytes.
    /// * `format`: Pixel format.
    /// * `client_owner_id`: Optional `ClientId` of the owner.
    ///
    /// # Returns
    /// An `Arc<Mutex<BufferDetails>>` for the newly registered buffer. The buffer
    /// will have an initial reference count of 1.
    ///
    /// # Notes
    /// - If this function were handling raw Wayland requests (e.g., from `wl_shm`),
    ///   it would perform more extensive validation (FD validity, offset/stride against pool size,
    ///   format support) and memory mapping. Errors like `InvalidFormat`, `InvalidStride`,
    ///   `InvalidFd`, or memory mapping failures (leading to `wl_display.error(no_memory)`)
    ///   would originate from such a process.
    /// - Robust `no_memory` handling for `BufferDetails` allocation itself (and the `Arc<Mutex<>>`)
    ///   would involve fallible allocation and error propagation. Currently, Rust's default
    ///   allocators will panic on OOM.
    pub fn register_buffer(
        &mut self,
        buffer_type: BufferType,
        width: u32,
        height: u32,
        stride: u32,
        format: BufferFormat,
        client_owner_id: Option<ClientId>,
    ) -> Arc<Mutex<BufferDetails>> {
        // Note on Wayland SHM buffer creation: (This comment was actually good and relevant here)
        // If this function were creating a buffer from a file descriptor (e.g., for wl_shm_pool_create_buffer),
        // it would perform validations like:
        // - Checking if the FD is valid.
        // - Checking if offset and stride are valid for the given FD size and buffer dimensions.
        //   (e.g., offset + stride * height <= pool_size).
        // - Validating format against supported wl_shm.format enums.
        // Errors like InvalidFormat, InvalidStride, InvalidFd would originate here.
        // It would also handle memory mapping (mmap) which could fail (e.g., no_memory).
        // If mmap fails, a wl_display.error(no_memory) should be sent.
        let details = BufferDetails::new(
            buffer_type,
            width,
            height,
            stride,
            format,
            client_owner_id,
        );
        let id = details.id;
        let arc_details = Arc::new(Mutex::new(details));
        self.buffers.insert(id, arc_details.clone());
        arc_details
    }

    /// Retrieves the shared `BufferDetails` for a given `BufferId`.
    ///
    /// # Arguments
    /// * `id`: The `BufferId` of the buffer to retrieve.
    ///
    /// # Returns
    /// An `Option<Arc<Mutex<BufferDetails>>>`. Returns `Some` if the buffer is found,
    /// `None` otherwise.
    pub fn get_buffer_details(&self, id: BufferId) -> Option<Arc<Mutex<BufferDetails>>> {
        self.buffers.get(&id).cloned()
    }

    /// Releases a reference to a buffer.
    ///
    /// This should be called when a component (e.g., a surface) no longer needs to use
    /// the buffer. It decrements the buffer's reference count. If the reference count
    /// drops to zero, the buffer is removed from the manager and can be considered
    /// fully released (e.g., the client can be notified via `wl_buffer.release`).
    ///
    /// # Arguments
    /// * `id`: The `BufferId` of the buffer to release.
    ///
    /// # Returns
    /// An `Option<Arc<Mutex<BufferDetails>>>`.
    /// - If the buffer was found and its reference count was decremented but is still greater than zero,
    ///   it returns `Some(Arc<Mutex<BufferDetails>>)` (cloned).
    /// - If the buffer was found and its reference count reached zero (and thus it was removed from the manager),
    ///   it returns `Some(Arc<Mutex<BufferDetails>>)` containing the buffer details just before removal.
    /// - If the buffer was not found in the manager, it returns `None`.
    pub fn release_buffer(&mut self, id: BufferId) -> Option<Arc<Mutex<BufferDetails>>> {
        let buffer_arc = self.buffers.get(&id)?;
        let should_remove = {
            let buffer_details = buffer_arc.lock().unwrap(); // Handle potential poison
            buffer_details.decrement_ref_count()
        };

        if should_remove {
            // If ref_count is zero, remove from manager.
            // Actual wl_buffer.release would be triggered here or by the caller.
            self.buffers.remove(&id)
        } else {
            // Still referenced elsewhere
            Some(buffer_arc.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread; // For testing unique IDs across threads, though basic test won't need this complexity yet.

    #[test]
    fn test_unique_buffer_ids() {
        let id1 = BufferId::new_unique();
        let id2 = BufferId::new_unique();
        assert_ne!(id1, id2, "BufferId::new_unique should generate unique IDs.");
    }

    #[test]
    fn test_register_buffer() {
        let mut manager = BufferManager::new();
        let client_id = Some(ClientId::new(1));
        let buffer_arc = manager.register_buffer(
            BufferType::Shm,
            640,
            480,
            640 * 4,
            BufferFormat::Argb8888,
            client_id,
        );

        let id = buffer_arc.lock().unwrap().id;
        assert!(manager.buffers.contains_key(&id), "Buffer should be registered.");

        let details = manager.get_buffer_details(id).unwrap();
        let locked_details = details.lock().unwrap();
        assert_eq!(locked_details.width, 640);
        assert_eq!(locked_details.height, 480);
        assert_eq!(locked_details.stride, 640 * 4);
        assert_eq!(locked_details.format, BufferFormat::Argb8888);
        assert_eq!(locked_details.client_owner_id, client_id);
        assert_eq!(locked_details.ref_count.load(Ordering::SeqCst), 1, "Initial ref count should be 1.");
    }

    #[test]
    fn test_get_buffer_details() {
        let mut manager = BufferManager::new();
        let client_id = Some(ClientId::new(1));
        let buffer_arc = manager.register_buffer(
            BufferType::Shm, 100, 200, 400, BufferFormat::Xrgb8888, client_id
        );
        let id = buffer_arc.lock().unwrap().id;

        // Test getting existing buffer
        let retrieved_arc = manager.get_buffer_details(id);
        assert!(retrieved_arc.is_some(), "Should retrieve registered buffer.");
        let details = retrieved_arc.unwrap().lock().unwrap();
        assert_eq!(details.id, id);
        assert_eq!(details.width, 100);

        // Test getting non-existent buffer
        let non_existent_id = BufferId::new_unique(); // Ensure it's different
        let non_existent_retrieved = manager.get_buffer_details(non_existent_id);
        assert!(non_existent_retrieved.is_none(), "Should not retrieve non-existent buffer.");
    }

    #[test]
    fn test_buffer_reference_counting() {
        let mut manager = BufferManager::new();
        let client_id = Some(ClientId::new(1));
        let buffer_arc = manager.register_buffer(
            BufferType::Shm, 32, 32, 128, BufferFormat::Argb8888, client_id
        );
        let id = buffer_arc.lock().unwrap().id;

        // Initial ref count is 1 (from registration by manager)
        assert_eq!(buffer_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 1);

        // Simulate external references
        buffer_arc.lock().unwrap().increment_ref_count(); // Ref count becomes 2
        assert_eq!(buffer_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2);

        buffer_arc.lock().unwrap().increment_ref_count(); // Ref count becomes 3
        assert_eq!(buffer_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 3);

        // First release (e.g., one user drops its reference)
        // This is simulated by BufferDetails::decrement_ref_count directly for simplicity,
        // as BufferManager::release_buffer also checks if it should remove.
        let was_last_ref1 = buffer_arc.lock().unwrap().decrement_ref_count(); // Count becomes 2
        assert!(!was_last_ref1, "Should not be the last reference yet.");
        assert_eq!(buffer_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2);
        assert!(manager.get_buffer_details(id).is_some(), "Buffer should still be in manager.");

        // Second release by BufferManager (simulates a surface releasing it)
        // BufferManager::release_buffer will call decrement_ref_count.
        let released_arc1 = manager.release_buffer(id); // Count becomes 1
        assert!(released_arc1.is_some(), "release_buffer should return the Arc if not last ref.");
        assert_eq!(buffer_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 1);
        assert!(manager.get_buffer_details(id).is_some(), "Buffer should still be in manager after one release by manager.");

        // Third release by BufferManager (simulates the original owner/manager releasing it)
        // This should be the final release.
        let released_arc2 = manager.release_buffer(id); // Count becomes 0
        assert!(released_arc2.is_some(), "release_buffer should return the Arc on final release before removal.");
         // The ref_count on `buffer_arc` itself is now 0, but the Arc still exists via `released_arc2`.
         // The manager should have removed its own tracking Arc.
        assert_eq!(buffer_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 0, "Ref count should be 0 before manager removes it.");
        assert!(manager.get_buffer_details(id).is_none(), "Buffer should be removed from manager after final release.");

        // Further releases should do nothing or error (current impl returns None)
        assert!(manager.release_buffer(id).is_none(), "Releasing already removed buffer should return None.");
    }
}
