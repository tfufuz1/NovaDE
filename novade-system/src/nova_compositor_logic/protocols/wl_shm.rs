//! Implements the `wl_shm` (Shared Memory) Wayland global and related interfaces.
//!
//! `wl_shm` allows clients to create shared memory pools (`wl_shm_pool`) and then
//! allocate buffers (`wl_buffer`) from these pools. These buffers are used to share
//! pixel data with the compositor for rendering surfaces.

use wayland_server::{
    protocol::{wl_shm, wl_shm_pool, wl_buffer}, // Added wl_buffer
    Dispatch, DelegateDispatch, GlobalDispatch, Main, Resource, Client, DisplayHandle, GlobalData, DataInit,
    implement_dispatch
};
use crate::state::CompositorState;
// TODO: Import Smithay's ShmHandler and related utilities if we want more robust SHM handling,
// including mmap and safety checks. For now, this is a basic stub.

/// Data associated with the `wl_shm` global instance when a client binds to it.
/// This struct is used as the resource data for the `wl_shm` resource.
#[derive(Debug, Default)]
pub struct ShmState; // Renamed from ShmGlobalData if it's resource data.

// GlobalDispatch for wl_shm on CompositorState.
// Handles new client bindings to the wl_shm global.
impl GlobalDispatch<wl_shm::WlShm, GlobalData, CompositorState> for CompositorState {
    /// Called when a client binds to the `wl_shm` global.
    ///
    /// Initializes the `wl_shm` resource for the client and advertises supported SHM formats.
    fn bind(
        _state: &mut CompositorState, // Global compositor state.
        _handle: &DisplayHandle,     // Handle to the display.
        _client: &Client,            // Client that bound to the global.
        resource: Main<wl_shm::WlShm>, // The wl_shm resource to initialize for the client.
        _global_data: &GlobalData,   // UserData for this global.
        data_init: &mut DataInit<'_, CompositorState> // Utility to initialize resource data.
    ) {
        println!("Client bound to wl_shm global. Initializing wl_shm resource {:?}.", resource.id());
        // Initialize the client's wl_shm resource with ShmState.
        let shm_resource = data_init.init_resource(resource, ShmState::default());

        // Advertise supported SHM buffer formats.
        // Argb8888 and Xrgb8888 are common and widely supported.
        shm_resource.format(wl_shm::Format::Argb8888);
        shm_resource.format(wl_shm::Format::Xrgb8888);
        // TODO: Consider adding more formats like Rgb888 if needed.
        println!("wl_shm {:?}: Advertised formats Argb8888, Xrgb8888.", shm_resource.id());
    }

    /// Checks if the requested version of `wl_shm` is supported.
    fn check_versions(&self, _main: Main<wl_shm::WlShm>, _versions: &[u32]) -> bool {
        true // wl_shm is typically version 1.
    }
}

// DelegateDispatch for requests made on a wl_shm resource.
// Since implement_dispatch!(CompositorState => [wl_shm::WlShm: GlobalData]); is used,
// CompositorState itself handles these requests.
impl DelegateDispatch<wl_shm::WlShm, GlobalData, CompositorState> for CompositorState {
    /// Handles requests from a client to a `wl_shm` resource.
    /// The primary request is `create_pool`.
    fn request(
        &mut self, // Global CompositorState.
        _client: &Client,
        shm_resource: &wl_shm::WlShm, // The wl_shm resource this request is for.
        request: wl_shm::Request,
        _data: &GlobalData, // UserData for this WlShm resource.
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, CompositorState>, // For creating the new wl_shm_pool.
    ) {
        match request {
            wl_shm::Request::CreatePool { id, fd, size } => {
                // Client requests to create a new wl_shm_pool.
                // `id` is the New<wl_shm_pool::WlShmPool> object.
                // `fd` is the file descriptor for the shared memory.
                // `size` is the size of the memory pool.
                println!(
                    "wl_shm {:?}: Client requested CreatePool (new id: {:?}, fd: {}, size: {})",
                    shm_resource.id(), id.id(), fd, size
                );

                // TODO: Implement proper SHM pool creation and management.
                // This involves:
                // 1. Validating the file descriptor and size.
                // 2. Memory-mapping the file descriptor. Smithay provides utilities for this
                //    which can handle safety concerns (e.g., in `smithay::wayland::shm::ShmState`).
                // 3. Storing the pool and its associated memory map.
                // For now, create a placeholder resource with basic dispatch.
                let pool_resource = data_init.init_resource(id, ShmPoolData::new(fd, size));
                println!("wl_shm_pool {:?} created.", pool_resource.id());

            }
            wl_shm::Request::Destroy => {
                // Handled by Dispatch::destroyed.
                println!("wl_shm {:?}: Client requested Destroy (handled by Dispatch::destroyed)", shm_resource.id());
            }
            _ => {
                eprintln!("wl_shm {:?}: Unknown request: {:?}", shm_resource.id(), request);
                unreachable!("Unknown request for WlShm: {:?}", request);
            }
        }
    }
}

// Dispatch implementation for CompositorState for wl_shm.
impl Dispatch<wl_shm::WlShm, GlobalData, CompositorState> for CompositorState {
    /// Handles requests if DelegateDispatch was not used or did not handle them.
    fn request(
        &mut self,
        _client: &Client,
        resource: &wl_shm::WlShm,
        request: wl_shm::Request,
        _data: &GlobalData,
        _dhandle: &DisplayHandle,
        _data_init: &mut wayland_server::DataInit<'_, Self>,
    ) {
        eprintln!(
            "wl_shm: Unexpected call to Dispatch::request for {:?} with request {:?}. Should be handled by DelegateDispatch.",
            resource.id(), request
        );
        unreachable!("Dispatch::request should not be called for WlShm when DelegateDispatch is implemented.");
    }

    /// Called when the `wl_shm` resource is destroyed.
    fn destroyed(
        &mut self,
        _client_id: wayland_server::backend::ClientId,
        resource_id: wayland_server::backend::ObjectId,
        _data: &GlobalData,
    ) {
        println!("wl_shm: Resource {:?} destroyed.", resource_id);
        // Cleanup for ShmState if any was needed.
    }
}

// Connects WlShm requests to CompositorState's Dispatch/DelegateDispatch.
implement_dispatch!(CompositorState => [wl_shm::WlShm: GlobalData]);


// --- wl_shm_pool ---

/// Data associated with a `wl_shm_pool` resource.
///
/// This should store information about the memory pool, such as the
/// file descriptor and its size, and potentially the memory map.
#[derive(Debug)]
pub struct ShmPoolData {
    // TODO: Store the memory map or relevant data from the fd.
    // For now, just storing fd and size for logging.
    #[allow(dead_code)] // Will be used when mmap is implemented
    fd: std::os::unix::io::RawFd,
    #[allow(dead_code)] // Will be used when mmap is implemented
    size: i32,
}

impl ShmPoolData {
    /// Creates new data for a `wl_shm_pool`.
    pub fn new(fd: std::os::unix::io::RawFd, size: i32) -> Self {
        // TODO: Here, or in the CreatePool handler, you would typically mmap the fd.
        // Closing the original fd after mmap is also common.
        Self { fd, size }
    }
}

// DelegateDispatch for wl_shm_pool requests, handled by ShmPoolData.
impl DelegateDispatch<wl_shm_pool::WlShmPool, (), CompositorState> for ShmPoolData {
    /// Handles requests from a client to a `wl_shm_pool` resource.
    fn request(
        &mut self, // State for this specific wl_shm_pool (&mut ShmPoolData).
        _client: &Client,
        pool_resource: &wl_shm_pool::WlShmPool, // The wl_shm_pool resource.
        request: wl_shm_pool::Request,
        _data: &(), // UserData for wl_shm_pool dispatch (unit type `()`).
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, CompositorState>, // For creating wl_buffer.
    ) {
        match request {
            wl_shm_pool::Request::CreateBuffer { id, offset, width, height, stride, format } => {
                // Client requests to create a wl_buffer from this pool.
                println!(
                    "wl_shm_pool {:?}: CreateBuffer request: new_id={:?}, offset={}, width={}, height={}, stride={}, format={:?}",
                    pool_resource.id(), id.id(), offset, width, height, stride, format
                );
                // TODO: Implement actual buffer creation:
                // 1. Validate parameters (offset, width, height, stride, format) against pool size and SHM rules.
                // 2. Create ShmBufferData (see below) and associate it with the new wl_buffer resource.
                //    This data might include a reference to the pool's memory map and the buffer's specific region.
                // 3. Initialize the wl_buffer resource.
                let buffer_resource = data_init.init_resource(id, ShmBufferData::new(offset, width, height, stride, format));
                println!("wl_buffer {:?} created from pool {:?}.", buffer_resource.id(), pool_resource.id());

            }
            wl_shm_pool::Request::Destroy => {
                // Client requests to destroy the wl_shm_pool.
                // Handled by Dispatch::destroyed for ShmPoolData.
                println!("wl_shm_pool {:?}: Client requested Destroy.", pool_resource.id());
            }
            wl_shm_pool::Request::Resize { size } => {
                // Client requests to resize the wl_shm_pool.
                // TODO: Implement pool resizing. This is complex and involves munmap/mmap if memory is already mapped.
                // Smithay's ShmState might handle this if used.
                // For now, just log. A real implementation might reject this or re-map.
                println!("wl_shm_pool {:?}: Resize request to size {}. (Not fully implemented)", pool_resource.id(), size);
                self.size = size; // Update internal size, though memory map isn't handled here.
            }
            _ => {
                eprintln!("wl_shm_pool {:?}: Unknown request: {:?}", pool_resource.id(), request);
            }
        }
    }
}

// Dispatch implementation for ShmPoolData.
impl Dispatch<wl_shm_pool::WlShmPool, (), CompositorState> for ShmPoolData {
     /// Handles requests if DelegateDispatch was not used or did not handle them.
    fn request(
        &mut self,
        client: &Client,
        resource: &wl_shm_pool::WlShmPool,
        request: wl_shm_pool::Request,
        data: &(),
        dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, CompositorState>,
    ) {
        // Forward to DelegateDispatch for consistent handling.
        self.request(client, resource, request, data, dhandle, data_init);
    }

    /// Called when the `wl_shm_pool` resource is destroyed.
    fn destroyed(
        &mut self,
        _client_id: wayland_server::backend::ClientId,
        object_id: wayland_server::backend::ObjectId,
        _data: &(),
    ) {
        println!("wl_shm_pool {:?}: Resource destroyed.", object_id);
        // TODO: If memory was mmap'd for this pool, it should be unmapped here.
        // Smithay's ShmPoolData/ShmState would handle this.
    }
}
// Connects WlShmPool requests to ShmPoolData's Dispatch/DelegateDispatch.
implement_dispatch!(ShmPoolData => [wl_shm_pool::WlShmPool: ()], CompositorState);


// --- wl_buffer ---

/// Data associated with a `wl_buffer` resource created from a SHM pool.
///
/// This should store information about the buffer, like its dimensions, format,
/// and a reference to the underlying shared memory.
#[derive(Debug)]
pub struct ShmBufferData {
    // TODO: Store reference to the SHM pool's memory map or a slice of it.
    // For now, storing properties for logging.
    #[allow(dead_code)] offset: i32,
    #[allow(dead_code)] width: i32,
    #[allow(dead_code)] height: i32,
    #[allow(dead_code)] stride: i32,
    #[allow(dead_code)] format: wl_shm::Format,
}

impl ShmBufferData {
    /// Creates new data for a `wl_buffer`.
    pub fn new(offset: i32, width: i32, height: i32, stride: i32, format: wl_shm::Format) -> Self {
        Self { offset, width, height, stride, format }
    }
}

// Dispatch for wl_buffer. wl_buffer only has one request: destroy.
impl Dispatch<wl_buffer::WlBuffer, (), CompositorState> for ShmBufferData {
    /// Handles requests to a `wl_buffer` resource. The only request is `destroy`.
    fn request(
        &mut self,
        _client: &Client,
        resource: &wl_buffer::WlBuffer,
        request: wl_buffer::Request,
        _data: &(), // UserData for wl_buffer dispatch (unit type `()`).
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, CompositorState>,
    ) {
        match request {
            wl_buffer::Request::Destroy => {
                // Client requests to destroy the wl_buffer.
                // Handled by Dispatch::destroyed for ShmBufferData.
                println!("wl_buffer {:?}: Client requested Destroy.", resource.id());
            }
            _ => {
                // wl_buffer should not receive other requests.
                eprintln!("wl_buffer {:?}: Unknown request: {:?}", resource.id(), request);
            }
        }
    }

    /// Called when the `wl_buffer` resource is destroyed.
    fn destroyed(
        &mut self,
        _client_id: wayland_server::backend::ClientId,
        object_id: wayland_server::backend::ObjectId,
        _data: &(),
    ) {
        println!("wl_buffer {:?}: Resource destroyed.", object_id);
        // TODO: Any cleanup specific to this buffer instance.
        // If this buffer held a reference to a mmap, it might be released here,
        // or when the parent pool is destroyed. Smithay's ShmBuffer often handles this.
    }
}
// Connects WlBuffer requests to ShmBufferData's Dispatch.
// No DelegateDispatch needed if Dispatch handles all (which it does for wl_buffer's single request).
implement_dispatch!(ShmBufferData => [wl_buffer::WlBuffer: ()], CompositorState);
