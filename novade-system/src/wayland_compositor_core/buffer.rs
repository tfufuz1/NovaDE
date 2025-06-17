use super::object::{WaylandObject, ObjectId, ProtocolError, RequestContext}; // Changed
use super::shm::{ShmFormat, ShmPoolSharedData}; // Changed
use super::wire::WlArgument; // Changed
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

pub type BufferId = ObjectId;

// Represents the data source of a buffer.
// For now, only SHM. Later could include DMABUF, etc.
#[derive(Debug)]
enum BufferDataSource {
    Shm {
        pool_data: Arc<ShmPoolSharedData>, // Shared mmap from the pool
        offset: usize,                     // Offset into the mmap
    },
    // DmaBuf { ... }
    // SolidColor { ... } // Example for a compositor-internal buffer type
}

#[derive(Debug)]
pub struct WlBuffer {
    id: BufferId,
    version: u32,
    width: i32,
    height: i32,
    stride: i32,
    format: ShmFormat, // For SHM buffers, this is ShmFormat. Could be a generic BufferFormat later.
    data_source: BufferDataSource,
    client_id: u32, // Client that created/owns this buffer resource

    // is_released: bool. Needs to be atomic if mutated by compositor (another thread)
    // and read by client handling (WaylandObject methods).
    // Or, if release event is sent via main event loop, then simple bool + Mutex is fine.
    // For now, let's use AtomicBool as release can be triggered by compositor logic.
    is_released: AtomicBool,
    // Could also have a usage_count or attached_surface_id to track if busy.
}

impl WlBuffer {
    // Constructor for SHM-based WlBuffer
    pub fn new_shm(
        id: BufferId,
        version: u32,
        width: i32,
        height: i32,
        stride: i32,
        format: ShmFormat,
        pool_data: Arc<ShmPoolSharedData>,
        offset: usize,
        client_id: u32,
    ) -> Self {
        Self {
            id,
            version,
            width,
            height,
            stride,
            format,
            data_source: BufferDataSource::Shm { pool_data, offset },
            client_id,
            is_released: AtomicBool::new(true), // A new buffer is initially considered "released"
                                                // until attached to a surface.
        }
    }

    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }
    pub fn stride(&self) -> i32 { self.stride }
    pub fn format(&self) -> ShmFormat { self.format } // Later generic BufferFormat

    pub fn is_released(&self) -> bool {
        self.is_released.load(Ordering::SeqCst)
    }

    // Called by Surface when it no longer uses this buffer.
    pub fn mark_as_released(&self) {
        self.is_released.store(true, Ordering::SeqCst);
        // TODO: This should queue a wl_buffer.release event to the client.
        // This requires access to an event queue or similar mechanism, possibly via client_id.
        // For now, we just set the flag. The event sending part is pending event loop integration.
        println!("WlBuffer {}: Marked as released. (Event sending to client not implemented yet)", self.id);
    }

    // Called by Surface when it attaches this buffer.
    pub fn mark_as_used(&self) {
        self.is_released.store(false, Ordering::SeqCst);
    }

    // Provides access to the buffer's memory slice if it's an SHM buffer.
    // This is unsafe because the lifetime of the slice is not tied to the mmap guard.
    // A safer version would return an Arc<Mmap> and offset, or a guarded slice.
    // For rendering, one might pass the Arc<ShmPoolSharedData> and offset directly.
    pub fn get_shm_data_slice(&self) -> Option<&[u8]> {
        match &self.data_source {
            BufferDataSource::Shm { pool_data, offset } => {
                let mmap_len = pool_data.mmap.len();
                let required_len = self.stride as usize * self.height as usize;
                if *offset + required_len > mmap_len {
                    // This should have been caught at buffer creation, but double check.
                    eprintln!("WlBuffer {}: SHM data slice out of bounds.", self.id);
                    return None;
                }
                Some(&pool_data.mmap[*offset..*offset + required_len])
            }
            // _ => None, // Other buffer types
        }
    }
}

impl WaylandObject for WlBuffer {
    fn id(&self) -> ObjectId { self.id }
    fn version(&self) -> u32 { self.version }
    fn interface_name(&self) -> &'static str { "wl_buffer" }

    fn handle_request(
        &self,
        opcode: u16,
        _args: Vec<WlArgument>, // destroy takes no args
        context: &mut RequestContext,
    ) -> Result<(), ProtocolError> {
        match opcode {
            0 => { // destroy
                // When a client destroys a wl_buffer, it can no longer be used, even if released.
                // The server should release it if it was attached.
                // The actual memory (if SHM) is managed by its WlShmPool.
                // This object is just a representation.
                self.mark_as_released(); // Ensure it's considered released by compositor logic too.

                // Remove from ObjectManager.
                // The Arc for this WlBuffer will be dropped. If it was the last Arc,
                // WlBuffer::drop is called. If this WlBuffer held an Arc to ShmPoolSharedData,
                // that Arc's count decreases.
                context.object_manager.destroy_object(self.id);
                Ok(())
            }
            _ => Err(ProtocolError::InvalidOpcode(opcode)),
        }
    }
}

// Drop behavior for WlBuffer:
// If it's an SHM buffer, its drop will decrement the Arc count on ShmPoolSharedData.
// If that was the last Arc to ShmPoolSharedData (meaning the pool itself might also be destroyed
// and all its other buffers are destroyed), then ShmPoolSharedData::drop will run,
// unmapping memory and closing the FD.
impl Drop for WlBuffer {
    fn drop(&mut self) {
        // If the buffer was associated with a pool, we might want to notify the pool
        // that this buffer is gone, for resource tracking within the pool.
        // However, with Arc<ShmPoolSharedData>, this is mostly automatic.
        // println!("WlBuffer {} dropped.", self.id);
    }
}
