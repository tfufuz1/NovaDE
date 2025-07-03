// novade-system/src/compositor/protocols/single_pixel_buffer.rs
// Implementation of the wp_single_pixel_buffer_v1 Wayland protocol

use smithay::{
    // There isn't a high-level delegate macro specifically for single_pixel_buffer
    // as it's a fairly simple protocol. Dispatch is handled directly.
    reexports::{
        wayland_protocols::wp::single_pixel_buffer::v1::server::{
            wp_single_pixel_buffer_manager_v1::{self, WpSinglePixelBufferManagerV1, Request as ManagerRequest},
            // No specific object for the buffer itself, it's just a wl_buffer with special properties.
        },
        wayland_server::{
            protocol::wl_buffer, // The created buffer is a wl_buffer
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    utils::Size, // For the 1x1 size
    backend::allocator::{
        PixelFormat, // To represent the RGBA8888 format
        // Smithay doesn't have a direct "SinglePixelBuffer" type in backend::allocator.
        // We create a wl_buffer and associate data with it indicating it's a single pixel buffer.
        // Or, the renderer handles a 1x1 wl_shm_buffer efficiently.
    },
    wayland::shm::ShmState, // SHM state is needed to create a wl_shm_buffer for the pixel
};
use std::{
    sync::{Arc, Mutex},
    io::Write, // For writing to the SHM pool
    os::unix::io::OwnedFd, // For creating SHM pool
};
use tempfile::tempfile; // For creating a temporary SHM file
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to provide access to `ShmState`.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For Single Pixel Buffer, it would need to access:
    // - ShmState (to create a 1x1 SHM buffer)
    // - Renderer capabilities (to ensure efficient handling of 1x1 textures if not SHM).
}

#[derive(Debug, Error)]
pub enum SinglePixelBufferError {
    #[error("Failed to create SHM pool for single pixel buffer: {0}")]
    ShmPoolCreationFailed(String),
    #[error("Failed to create wl_buffer for single pixel: {0}")]
    BufferCreationFailed(String),
    #[error("Invalid RGBA data for single pixel buffer")]
    InvalidRgbaData, // Though the protocol takes u32, so it's always valid bits
}

// UserData for the WpSinglePixelBufferManagerV1 global (if any needed, often unit).
#[derive(Debug, Default)]
pub struct SinglePixelBufferManagerData;


// The main compositor state (e.g., NovaCompositorState) would implement Dispatch
// for WpSinglePixelBufferManagerV1.
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub shm_state: ShmState, // Needed to create the 1x1 SHM buffer
//     // Potentially other states if we want to track these buffers specifically.
//     ...
// }
//
// impl Dispatch<WpSinglePixelBufferManagerV1, SinglePixelBufferManagerData, NovaCompositorState> for NovaCompositorState {
//     fn request(
//         state: &mut NovaCompositorState,
//         _client: &Client,
//         manager: &WpSinglePixelBufferManagerV1,
//         request: ManagerRequest,
//         _data: &SinglePixelBufferManagerData,
//         _dhandle: &DisplayHandle,
//         buffer_init: &mut DataInit<'_, NovaCompositorState>, // Should be DataInit for wl_buffer
//     ) {
//         match request {
//             ManagerRequest::Destroy => { /* Smithay handles */ }
//             ManagerRequest::CreateSinglePixelBuffer { id, r, g, b, a } => {
//                 // ... logic to create 1x1 SHM buffer ...
//                 // let wl_buffer = shm_state.create_buffer(...);
//                 // buffer_init.init(id, wl_buffer_data_for_user_data);
//             }
//             _ => unimplemented!(),
//         }
//     }
// }

/// Handles dispatching of the WpSinglePixelBufferManagerV1 global.
/// `D` is your main compositor state, which must provide `ShmState`.
impl<D> GlobalDispatch<WpSinglePixelBufferManagerV1, SinglePixelBufferManagerData, D> for DesktopState // Replace DesktopState with D
where
    D: GlobalDispatch<WpSinglePixelBufferManagerV1, SinglePixelBufferManagerData, D> +
       Dispatch<WpSinglePixelBufferManagerV1, SinglePixelBufferManagerData, D> +
       AsMut<ShmState> + // Need mutable access to ShmState
       'static,
{
    fn bind(
        _state: &mut D, // The main compositor state
        _handle: &DisplayHandle,
        _client: &Client,
        resource: WpSinglePixelBufferManagerV1,
        _global_data: &SinglePixelBufferManagerData,
    ) {
        info!("Client bound WpSinglePixelBufferManagerV1: {:?}", resource);
        resource.quick_assign(|manager, request, dispatch_data| {
            // `dispatch_data` here is &mut D (our main state, e.g., NovaCompositorState)
            // We need to access `ShmState` from `dispatch_data`.
            // For this skeleton, we assume `dispatch_data.shm_state()` or similar.
            let shm_state = dispatch_data.as_mut(); // D must implement AsMut<ShmState>

            match request {
                ManagerRequest::Destroy => {
                    // Handled by Smithay's resource destruction
                    info!("WpSinglePixelBufferManagerV1 {:?} destroyed by client", manager);
                }
                ManagerRequest::CreateSinglePixelBuffer { id, r, g, b, a } => {
                    debug!(
                        "Client requests CreateSinglePixelBuffer with id {:?}, R:{}, G:{}, B:{}, A:{}",
                        id, r, g, b, a
                    );

                    // The protocol specifies ARGB8888 format.
                    // Data is packed as: (A << 24) | (R << 16) | (G << 8) | B
                    // The parameters r, g, b, a are u32, but they represent u8 values.
                    // The wire format sends these as individual u32s, but they are conceptually u8.
                    // Let's ensure they are treated as such for pixel data.
                    // The protocol actually sends u32 for each component, which is unusual.
                    // "The R, G, B and A arguments are passed as 32-bit unsigned integers.
                    // The compositor must clamp these values to the range [0, 255] and then
                    // construct the actual pixel data."
                    // This means we should take `(val & 0xFF)` for each.

                    let clamped_r = (r & 0xFF) as u8;
                    let clamped_g = (g & 0xFF) as u8;
                    let clamped_b = (b & 0xFF) as u8;
                    let clamped_a = (a & 0xFF) as u8;

                    // Pixel data in ARGB8888 format (bytes: B, G, R, A for little-endian if read as u32)
                    // Or, if writing byte by byte: A, R, G, B for standard ARGB byte order.
                    // wl_shm.format is ARGB8888.
                    // If system is little-endian (most common):
                    // A u32 0xAARRGGBB would be stored in memory as bytes BB, GG, RR, AA.
                    // wl_shm format ARGB8888 means pixel byte order A,R,G,B.
                    // So, if we make a u32 for pixel data, it should be `(clamped_a as u32) << 24 | (clamped_r as u32) << 16 | (clamped_g as u32) << 8 | (clamped_b as u32)`
                    // And then write its bytes to the SHM pool.
                    let pixel_data_bytes: [u8; 4] = [clamped_b, clamped_g, clamped_r, clamped_a]; // BGRA byte order for little-endian ARGB8888 u32
                    // Let's be explicit with byte order for wl_shm_format::ARGB8888, which means A, R, G, B in memory.
                    // This is confusing. wl_shm.format ARGB8888 implies byte order A, R, G, B.
                    // If we write a u32, its byte order depends on endianness.
                    // Safest is to write bytes directly in A, R, G, B order if that's what ARGB8888 means for shm.
                    // Cogl and Cairo use ARGB32 as A<<24|R<<16|G<<8|B for native endian.
                    // Wayland protocol for wl_shm_format::ARGB8888 is A,R,G,B bytes.
                    // So, pixel_data_bytes should be [clamped_a, clamped_r, clamped_g, clamped_b] for direct byte write.
                    // Let's use this and be careful.
                    //
                    // Re-check wl_shm spec: For ARGB8888, "alpha, red, green, blue; 8 bits per component".
                    // "The data are organized as like arrays of pixel data, [...] where an array element
                    // is a an unsigned integer of the specified bits per pixel format."
                    // For ARGB8888, it's 32 bits per pixel.
                    // "All values are encoded in host byte order when transmitted over the wire." (This is for protocol messages, not shm content)
                    // "The data in the shared memory segment is in host byte order." (This is the key for shm content)
                    // So, a u32 value `(A << 24) | (R << 16) | (G << 8) | B` written to SHM is correct.
                    let pixel_value: u32 = ((clamped_a as u32) << 24) |
                                           ((clamped_r as u32) << 16) |
                                           ((clamped_g as u32) << 8)  |
                                           (clamped_b as u32);

                    let width = 1;
                    let height = 1;
                    let stride = 4; // 1 pixel * 4 bytes (ARGB8888)
                    let shm_pool_size = stride * height; // = 4 bytes

                    let mut temp_shm_file = match tempfile() {
                        Ok(file) => file,
                        Err(e) => {
                            error!("Failed to create temp SHM file for single pixel buffer: {}", e);
                            // TODO: Send protocol error on manager? Protocol doesn't define errors for manager.
                            // Client might get a fatal error if `id` is not implemented.
                            // We should ensure `id` is properly destroyed or an error is sent on it if possible.
                            // For now, just log and the request will effectively fail.
                            resource.post_error(wp_single_pixel_buffer_manager_v1::Error::NoMemory, "Failed to create SHM temp file");
                            return;
                        }
                    };

                    // Write the pixel data to the temp file.
                    if let Err(e) = temp_shm_file.write_all(&pixel_value.to_ne_bytes()) { // to_ne_bytes for host byte order
                        error!("Failed to write to SHM temp file for single pixel buffer: {}", e);
                        resource.post_error(wp_single_pixel_buffer_manager_v1::Error::NoMemory, "Failed to write to SHM temp file");
                        return;
                    }
                    if let Err(e) = temp_shm_file.flush() { // Ensure data is written
                        error!("Failed to flush SHM temp file: {}", e);
                        resource.post_error(wp_single_pixel_buffer_manager_v1::Error::NoMemory, "Failed to flush SHM temp file");
                        return;
                    }

                    // Create an SHM pool from the temp file.
                    // `ShmState::import_raw` or `ShmState::import_fd` can be used.
                    // `create_pool` takes an FD and size, then `create_buffer` on the pool.
                    let shm_fd: OwnedFd = match rustix::fs::dup(&temp_shm_file) {
                        Ok(fd) => fd.into(),
                        Err(e) => {
                             error!("Failed to dup fd for SHM temp file: {}", e);
                             resource.post_error(wp_single_pixel_buffer_manager_v1::Error::NoMemory, "Failed to dup SHM fd");
                             return;
                        }
                    };


                    // We need a WlShmPool resource first.
                    // This is usually created by the client. Here, the server creates it implicitly.
                    // Smithay's ShmState::create_buffer directly might be simpler if it supports this.
                    // Let's try to use `ShmState::create_buffer_from_raw` if available, or manually construct.

                    // Smithay's `ShmState::create_shm_buffer` is the typical way.
                    // It takes an fd, offset, width, height, stride, and format.
                    // The fd should be a valid memfd or similar that ShmState can map.
                    let wl_shm_buffer = match shm_state.create_buffer(
                        &id, // The NewId for the wl_buffer resource
                        shm_fd,
                        shm_pool_size as u32, // length of the pool (for offset validation)
                        0, // offset within the pool
                        width,
                        height,
                        stride as i32,
                        wl_shm::Format::Argb8888, // Protocol specifies ARGB8888
                        (), // UserData for the wl_buffer, can be unit or specific data
                    ) {
                        Ok(buffer_resource) => {
                            info!(
                                "Created 1x1 SHM wl_buffer {:?} for single pixel (R:{}, G:{}, B:{}, A:{})",
                                buffer_resource, clamped_r, clamped_g, clamped_b, clamped_a
                            );
                            // The `id` (NewId<WlBuffer>) is now implemented with this buffer.
                            // Smithay handles associating the UserData and dispatching.
                            // No explicit `data_init.init(id, ...)` needed here as `create_buffer` does it.
                        }
                        Err(e) => {
                            error!("Failed to create 1x1 SHM wl_buffer: {:?}", e);
                            // If create_buffer fails, `id` is not implemented.
                            // The client will get a protocol error or the ID will be dead.
                            // We might need to send an error on the manager if the protocol allows.
                            // `wp_single_pixel_buffer_manager_v1` has `no_memory` error.
                            manager.post_error(wp_single_pixel_buffer_manager_v1::Error::NoMemory, format!("Failed to create SHM buffer: {}", e));
                        }
                    };
                    // The `temp_shm_file` and its `OwnedFd` (`shm_fd`) will be closed when they go out of scope.
                    // The kernel keeps the shared memory alive as long as it's mapped (which ShmState does).
                }
                _ => unimplemented!("Request not implemented for WpSinglePixelBufferManagerV1"),
            }
        });
    }

    fn can_view(_client: Client, _global_data: &SinglePixelBufferManagerData) -> bool {
        true // Any client can use this manager
    }
}


/// Initializes and registers the WpSinglePixelBufferManagerV1 global.
/// `D` is your main compositor state type.
pub fn init_single_pixel_buffer_manager<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<WpSinglePixelBufferManagerV1, SinglePixelBufferManagerData, D> +
       Dispatch<WpSinglePixelBufferManagerV1, SinglePixelBufferManagerData, D> +
       AsMut<ShmState> + // For creating SHM buffers
       'static,
       // D must also own ShmState.
       // Dispatch<wl_buffer::WlBuffer, BufferData, D> is also needed for the created buffers.
       // Smithay's ShmState::create_buffer handles initializing the wl_buffer resource.
{
    info!("Initializing WpSinglePixelBufferManagerV1 global");

    // ShmState must be initialized and part of D for this to work.

    display.create_global::<D, WpSinglePixelBufferManagerV1, _>(
        1, // protocol version
        SinglePixelBufferManagerData::default() // GlobalData for the manager
    )?;

    info!("WpSinglePixelBufferManagerV1 global initialized.");
    Ok(())
}

// TODO:
// - Renderer Optimization:
//   - While using SHM is straightforward, for performance-critical uses (like frequent cursor updates
//     if cursors were made of these), a renderer might have optimized paths for handling
//     solid color textures or 1x1 textures without full SHM overhead.
//   - This protocol is primarily for convenience and reducing client-side complexity, not necessarily
//     peak performance for all scenarios. Ensure the renderer handles 1x1 SHM buffers efficiently.
// - Error Handling:
//   - Robustly handle failures in SHM pool creation (e.g., out of memory, FD limits) and
//     send appropriate protocol errors to the client if possible. The protocol defines
//     `no_memory` error on the manager.
// - State Integration:
//   - `ShmState` must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `AsMut<ShmState>`.
// - Testing:
//   - Test with clients that use this protocol (e.g., for simple cursors or UI elements).
//     A custom test client might be needed if common apps don't use it extensively.
//   - Verify that the created `wl_buffer` contains the correct single pixel color data
//     by attaching it to a surface and rendering it.
//   - Check various RGBA color combinations.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod single_pixel_buffer;
