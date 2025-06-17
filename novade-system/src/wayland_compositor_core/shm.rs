use super::object::{WaylandObject, ObjectId, ProtocolError, RequestContext}; // Changed
use super::wire::WlArgument; // Changed
use super::buffer::WlBuffer; // Changed
use memmap2::{MmapMut, MmapOptions};
use nix::unistd::close;
use std::fs::File;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::sync::{Arc, Mutex}; // Mutex for MmapMut if resize needs exclusive access. Or Arc<MmapMut>

// wl_shm_pool.format enum (from wayland.xml)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ShmFormat {
    Argb8888 = 0,
    Xrgb8888 = 1,
    C8 = 0x20384343,
    Rgb332 = 0x38424752,
    Bgr233 = 0x38524742,
    Xrgb4444 = 0x32315258,
    Xbgr4444 = 0x32314258,
    Rgbx4444 = 0x32315852,
    Bgrx4444 = 0x32315842,
    Argb4444 = 0x32315241,
    Abgr4444 = 0x32314241,
    Rgb565 = 0x36314752,
    Bgr565 = 0x36314742,
    Rgb888 = 0x34324752,
    Bgr888 = 0x34324742,
    Xbgr8888 = 0x34324258,
    Rgbx8888 = 0x34325852,
    Abgr8888 = 0x34324241,
    Bgra8888 = 0x34324142,
    Xrgb2101010 = 0x30335258,
    Xbgr2101010 = 0x30334258,
    Rgbx1010102 = 0x30335852,
    Bgrx1010102 = 0x30335842,
    Argb2101010 = 0x30335241,
    Abgr2101010 = 0x30334241,
    Rgba1010102 = 0x30334152,
    Bgra1010102 = 0x30334142,
    Yuyv = 0x56595559,
    Yvyu = 0x55595659,
    Uyvy = 0x59565955,
    Vyuy = 0x59555956,
    Ayuv = 0x56555941,
    Nv12 = 0x3231564e,
    Nv21 = 0x3132564e,
    Nv16 = 0x3631564e,
    Nv61 = 0x3136564e,
    Yuv410 = 0x39565559,
    Yvu410 = 0x39555659,
    Yuv411 = 0x31315559,
    Yvu411 = 0x31315659,
    Yuv420 = 0x32315559,
    Yvu420 = 0x32315659,
    Yuv422 = 0x36315559,
    Yvu422 = 0x36315659,
    Yuv444 = 0x34325559,
    Yvu444 = 0x34325659,
    R8 = 0x20382052,
    R16 = 0x20363152,
    Rg88 = 0x38384752,
    Gr88 = 0x38385247, // Way oficiales GR88
    Rg1616 = 0x32334752,
    Gr1616 = 0x32335247, // Way oficiales GR1616
    Xrgb16161616f = 0x48345258,
    Xbgr16161616f = 0x48344258,
    Argb16161616f = 0x48345241,
    Abgr16161616f = 0x48344241,
    Xyuv8888 = 0x56555958, // wl_drm Xr30, Xb30, etc.
    Vuy888 = 0x34325556, // MediaTek specific
    // ... and many more, this is a selection
}

impl ShmFormat {
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            0 => Some(ShmFormat::Argb8888),
            1 => Some(ShmFormat::Xrgb8888),
            // ... add all other variants
            _ => None,
        }
    }
}


// --- WlShm ---
#[derive(Debug)]
pub struct WlShm {
    id: ObjectId,
    version: u32,
}

impl WlShm {
    pub fn new(id: ObjectId, version: u32) -> Self {
        Self { id, version }
    }
}

impl WaylandObject for WlShm {
    fn id(&self) -> ObjectId { self.id }
    fn version(&self) -> u32 { self.version }
    fn interface_name(&self) -> &'static str { "wl_shm" }

    fn handle_request(
        &self,
        opcode: u16,
        args: Vec<WlArgument>,
        context: &mut RequestContext,
    ) -> Result<(), ProtocolError> {
        match opcode {
            0 => { // create_pool(id: new_id, fd: fd, size: int)
                if args.len() < 3 { return Err(ProtocolError::InvalidArguments); }
                let new_pool_id = match args[0] { WlArgument::NewId(id) => id, _ => return Err(ProtocolError::InvalidArguments) };
                let fd = match args[1] { WlArgument::Fd(fd) => fd, _ => return Err(ProtocolError::InvalidArguments) };
                let size = match args[2] { WlArgument::Int(s) => s, _ => return Err(ProtocolError::InvalidArguments) };

                if size <= 0 {
                    // TODO: Send wl_shm.error (invalid_size or something)
                    // For now, protocol error. Should close FD.
                    close(fd).ok();
                    return Err(ProtocolError::InvalidArguments); // Or a more specific SHM error
                }

                let pool = WlShmPool::new(new_pool_id, self.version, fd, size as usize, context.client_id)?;
                context.object_manager.register_new_object(new_pool_id, pool)?;
                Ok(())
            }
            _ => Err(ProtocolError::InvalidOpcode(opcode)),
        }
    }
}

// --- WlShmPool ---
// Need to store MmapMut in an Arc if WlBuffers are to share it and outlive the pool destruction request
// or if WlBuffers are Arcs themselves.
// For now, let's assume WlBuffer will hold Arc<ShmPoolSharedData>
#[derive(Debug)]
pub struct ShmPoolSharedData {
    pub mmap: MmapMut, // The memory map
    pub fd: RawFd,     // Original FD, kept for resizing/unmapping
    pub size: usize,   // Current size of the mmap
    // Could add a list of active buffer IDs created from this pool for validation
    // active_buffers: Mutex<HashSet<ObjectId>>,
}

impl Drop for ShmPoolSharedData {
    fn drop(&mut self) {
        // MmapMut handles unmapping on drop.
        // We need to close the original FD.
        close(self.fd).ok(); // Best effort close
        // println!("ShmPoolSharedData dropped, fd {} closed.", self.fd);
    }
}


#[derive(Debug)]
pub struct WlShmPool {
    id: ObjectId,
    version: u32,
    shared_data: Arc<ShmPoolSharedData>, // Arc allows buffers to share this mmap
    client_id: u32, // Client that owns this pool
                     // internal_state: Mutex<PoolState>, // e.g. Alive, Destroyed
}

impl WlShmPool {
    pub fn new(id: ObjectId, version: u32, fd: RawFd, size: usize, client_id: u32) -> Result<Self, ProtocolError> {
        // Mmap the fd. The fd is owned by the pool (or its shared_data).
        // Safety: MmapOptions::map_mut is unsafe. Caller must ensure fd is valid and size is correct.
        // We assume client sent a valid FD. Compositor might validate FD type/flags.
        let mmap = unsafe {
            MmapOptions::new()
                .len(size)
                .map_mut(&unsafe { File::from_raw_fd(fd) }) // MmapMut needs a &File, File::from_raw_fd takes ownership of fd
                                                            // This is problematic if fd is also stored in ShmPoolSharedData for close.
                                                            // A better way: dup the fd for Mmap, store original for close.
        };

        // Let's dup the fd to avoid double close issues or File taking ownership before we want.
        let owned_fd_for_mmap = match nix::unistd::dup(fd) {
            Ok(duped_fd) => duped_fd,
            Err(_) => {
                close(fd).ok(); // Close original fd if dup fails
                return Err(ProtocolError::ImplementationError); // Cannot dup fd
            }
        };

        let mmap_result = unsafe {
            MmapOptions::new()
                .len(size)
                .map_mut(&File::from_raw_fd(owned_fd_for_mmap))
        };

        match mmap_result {
            Ok(mmap) => {
                let shared_data = Arc::new(ShmPoolSharedData { mmap, fd, size });
                Ok(Self { id, version, shared_data, client_id })
            }
            Err(e) => {
                close(fd).ok(); // Close original fd if mmap failed
                close(owned_fd_for_mmap).ok(); // Close duped fd
                eprintln!("Mmap error: {}", e);
                Err(ProtocolError::NoMemory) // Or map to a specific SHM error
            }
        }
    }
}

impl WaylandObject for WlShmPool {
    fn id(&self) -> ObjectId { self.id }
    fn version(&self) -> u32 { self.version }
    fn interface_name(&self) -> &'static str { "wl_shm_pool" }

    fn handle_request(
        &self, // Takes &self, so need internal mutability for state changes like active_buffers
        opcode: u16,
        args: Vec<WlArgument>,
        context: &mut RequestContext,
    ) -> Result<(), ProtocolError> {
        // TODO: Check if pool is destroyed before handling requests other than destroy.
        // This would require WlShmPool to have its own internal_state like Surface.

        match opcode {
            0 => { // create_buffer(id: new_id, offset: int, width: int, height: int, stride: int, format: uint)
                if args.len() < 6 { return Err(ProtocolError::InvalidArguments); }
                let new_buffer_id = match args[0] { WlArgument::NewId(id) => id, _ => return Err(ProtocolError::InvalidArguments) };
                let offset = match args[1] { WlArgument::Int(o) => o as usize, _ => return Err(ProtocolError::InvalidArguments) };
                let width = match args[2] { WlArgument::Int(w) => w, _ => return Err(ProtocolError::InvalidArguments) };
                let height = match args[3] { WlArgument::Int(h) => h, _ => return Err(ProtocolError::InvalidArguments) };
                let stride = match args[4] { WlArgument::Int(s) => s, _ => return Err(ProtocolError::InvalidArguments) };
                let format_u32 = match args[5] { WlArgument::Uint(f) => f, _ => return Err(ProtocolError::InvalidArguments) };

                if width <= 0 || height <= 0 || stride <= 0 { return Err(ProtocolError::InvalidArguments); /* Or specific shm error */ }
                if stride < width * 4 { /* Assuming 4 bytes per pixel for basic check */ return Err(ProtocolError::InvalidArguments); }


                let format = ShmFormat::from_u32(format_u32).ok_or(ProtocolError::InvalidArguments)?; // wl_shm.error: invalid_format

                // Validate offset and size against pool
                let required_size = offset + (stride as usize * height as usize);
                if required_size > self.shared_data.size {
                    return Err(ProtocolError::InvalidArguments); // wl_shm.error: invalid_stride or size too small
                }

                // WlBuffer needs to hold Arc<ShmPoolSharedData>
                let buffer = WlBuffer::new_shm(
                    new_buffer_id,
                    self.version(), // Buffer version can be inherited from pool or shm global
                    width, height, stride, format,
                    self.shared_data.clone(), // Pass Arc to buffer
                    offset,
                    self.client_id,
                );
                context.object_manager.register_new_object(new_buffer_id, buffer)?;
                // TODO: Track new_buffer_id as active in this pool if needed for resize validation.
                Ok(())
            }
            1 => { // destroy
                // Mark this pool as destroyed.
                // The actual memory unmap and FD close happens when all Arcs to ShmPoolSharedData are dropped.
                // This means all WlBuffers created from this pool must also be destroyed/released.
                // For now, just remove from ObjectManager. ObjectManager::destroy_object will drop our Arc<WlShmPool>.
                // If other Arcs to ShmPoolSharedData exist (in WlBuffers), data remains mapped.
                // This is "deferred destruction" by nature of Arc.
                context.object_manager.destroy_object(self.id); // Request removal
                Ok(())
            }
            2 => { // resize(size: int)
                // This is complex. Mremap or unmap/remap.
                // All existing buffers become invalid. Wayland spec says:
                // "Existing wl_buffer objects thus become invalid and should not be used anymore."
                // Compositor should send wl_buffer.release to all buffers from this pool.
                // Then, client should destroy them.
                // For now, this is a placeholder or error.
                eprintln!("WlShmPool {}: resize (opcode 2) - Unimplemented or requires careful handling.", self.id);
                // To implement correctly:
                // 1. Lock shared_data if it's behind a Mutex for modification.
                // 2. If using mremap, that's OS-specific (nix::sys::mman::mremap).
                // 3. Update shared_data.size and possibly shared_data.mmap.
                // 4. Release all existing buffers from this pool.
                Err(ProtocolError::ImplementationError) // Mark as unimplemented for now
            }
            _ => Err(ProtocolError::InvalidOpcode(opcode)),
        }
    }
}
