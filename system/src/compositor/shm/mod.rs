use crate::compositor::core::state::DesktopState;
use crate::compositor::shm::errors::ShmError; // Assuming this is where ShmError is
use smithay::{
    delegate_shm, // Already in state.rs, but good to note its relevance here
    reexports::wayland_server::{
        protocol::{wl_buffer, wl_shm},
        Client, DisplayHandle, GlobalDispatch,
    },
    wayland::{
        buffer::BufferHandler, // Already in state.rs
        shm::{ShmHandler, ShmState, WlShmGlobal}, // Import WlShmGlobal
    },
};

// ShmHandler is already implemented in core::state.rs via delegate_shm!
// and the required methods:
// impl ShmHandler for DesktopState {
//     fn shm_state(&self) -> &ShmState {
//         &self.shm_state
//     }
// }
// We might need to add `shm_formats` or `shm_client_data` if defaults aren't enough.
// For now, the existing implementation in state.rs should suffice for delegation.

// BufferHandler is already implemented in core::state.rs:
// impl BufferHandler for DesktopState {
//     fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {
//         tracing::info!("Buffer destroyed: {:?}", _buffer);
//         // TODO: Notify renderer about texture invalidation.
//         // This is the place to hook into for SHM buffer cleanup.
//         // We need to find which SurfaceData might be using this buffer
//         // and clear its texture_handle and current_buffer_info.
//         // This requires iterating through surfaces, which can be complex.
//         // A more direct approach is if the renderer can map wl_buffer to its textures.
//         // Or, when a surface commits a new buffer, the old one (if SHM) can be explicitly released by renderer.
//         // For now, logging is a placeholder.
//     }
// }
// The current BufferHandler in state.rs is a good starting point.
// We'll need to refine the TODO for renderer notification.

// GlobalDispatch for WlShm
impl GlobalDispatch<wl_shm::WlShm, ()> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: smithay::reexports::wayland_server::New<wl_shm::WlShm>,
        _global_data: &(),
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, Self>,
    ) {
        tracing::info!("Binding WlShm for client.");
        data_init.init(resource, ());
        // Additional client-specific SHM setup could go here if needed.
    }
}

/// Initializes the SHM state and registers the `wl_shm` global.
/// Stores the `GlobalId` in `DesktopState`.
pub fn create_shm_global(display_handle: &DisplayHandle, state: &mut DesktopState) {
    // Initialize ShmState with default formats (Argb8888, Xrgb8888) and no logger.
    // If specific formats are needed, they can be added to the Vec.
    // Example: state.shm_state = ShmState::new(vec![wl_shm::Format::Argb8888], None);
    // For now, using the default initialization from DesktopState::new is fine.

    // Create the wl_shm global. WlShmGlobal is a helper type from Smithay.
    let shm_global_id = display_handle.create_global::<DesktopState, WlShmGlobal, ()>(
        1, // version
        (), // global data - none for wl_shm
    );

    state.shm_global = Some(shm_global_id);
    tracing::info!("Created wl_shm global with ID: {:?}", shm_global_id);

    // Note: The ShmState itself is already initialized in DesktopState::new.
    // This function's main role is to register the global and store its ID.
    // If ShmState needed display-specific setup, it could be done here.
}

// Reminder: DesktopState already has `delegate_shm!(DesktopState);`
// and the basic `ShmHandler` impl.
// `BufferHandler` is also in `DesktopState`.
// This file primarily adds `GlobalDispatch<WlShm, ()>` and `create_shm_global`.

// Further considerations for BufferHandler:
// When a `wl_buffer` is destroyed, if it was an SHM buffer used for a texture,
// that texture is now invalid. The `BufferHandler::buffer_destroyed` callback
// is the place to handle this.
//
// A robust way to handle this:
// 1. When a buffer is turned into a texture, store a mapping: `wl_buffer.id() -> texture_id/surface_id`.
// 2. In `buffer_destroyed`, use this mapping to find the affected texture/surface.
// 3. Instruct the renderer to free the texture.
// 4. Update `SurfaceData` to remove `texture_handle` and `current_buffer_info`.
//
// This requires `DesktopState` to have access to this mapping, or for the renderer
// to manage it internally if `WlBuffer` objects are passed to it.
// Smithay's `ShmState` or `UserDataMap` on `WlBuffer` could be used to track this.
// For now, the existing `BufferHandler` logs. Detailed cleanup is a deeper integration task.
// The `ShmError` enum is in `crate::compositor::shm::errors::ShmError;`
// It's not directly used in this file but is part of the SHM module.
