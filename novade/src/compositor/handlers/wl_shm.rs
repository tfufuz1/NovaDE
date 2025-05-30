//! Handler for the `wl_shm` (Wayland Shared Memory) global.
//!
//! This module implements the `smithay::wayland::shm::ShmHandler` trait for
//! `NovaCompositorState`. It is responsible for managing client requests related
//! to shared memory, which is a primary mechanism for clients to provide pixel data
//! for their surfaces to the compositor. This includes creating shared memory pools
//! (`wl_shm_pool`) and buffers (`wl_buffer`) from these pools.

#![allow(unused_variables)]
#![allow(dead_code)]

use crate::compositor::state::NovaCompositorState;
use smithay::{
    delegate_shm,
    reexports::wayland_server::protocol::{wl_shm::WlShm, wl_shm_pool::WlShmPool, wl_buffer::WlBuffer}, // Added pool and buffer for docs
    wayland::shm::{ShmHandler, ShmState},
};

// The actual `smithay::wayland::shm::ShmState` is stored in `NovaCompositorState`.
// This handler provides access to it and defines callbacks for SHM-related events.

impl ShmHandler for NovaCompositorState {
    /// Provides access to Smithay's `ShmState`, which tracks shared memory pools
    /// and buffers.
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }

    /// Called when a new `wl_shm_pool` is created by a client.
    ///
    /// This callback can be used for logging or for any custom tracking of SHM pools,
    /// though Smithay's `ShmState` handles the core logic.
    fn new_pool(&mut self, pool: &WlShmPool) {
        slog::debug!(self.logger, "New SHM pool created: {:?}", pool.id());
        // Additional compositor logic related to pool creation can go here.
    }

    /// Called after a new `wl_buffer` backed by shared memory is created by a client.
    ///
    /// This is a notification that a buffer is now available. The buffer's validity
    /// and properties are managed by `ShmState`.
    fn created_buffer(&mut self, buffer: &WlBuffer) {
        slog::debug!(self.logger, "SHM buffer created: {:?}", buffer.id());
        // Additional compositor logic related to buffer creation can go here.
    }

    /// Called when a `wl_shm_pool` is destroyed by a client.
    ///
    /// Smithay handles the actual resource release. This callback is for any additional
    /// cleanup or state updates the compositor might need to perform.
    fn destroyed_pool(&mut self, pool: &WlShmPool) {
        slog::debug!(self.logger, "SHM pool destroyed: {:?}", pool.id());
        // Additional compositor logic related to pool destruction can go here.
    }

    /// Called when a `wl_buffer` backed by shared memory is destroyed by a client.
    ///
    /// Smithay handles the resource release. This callback allows for custom cleanup.
    fn destroyed_buffer(&mut self, buffer: &WlBuffer) {
        slog::debug!(self.logger, "SHM buffer destroyed: {:?}", buffer.id());
        // Additional compositor logic related to buffer destruction can go here.
    }
}

// Delegate wl_shm requests to NovaCompositorState.
// This macro implements `GlobalDispatch<WlShm, ShmData<D>>` and the necessary
// `Dispatch` trait for `NovaCompositorState`.
// `ShmData` is a simple marker struct provided by Smithay for `wl_shm` globals.
delegate_shm!(NovaCompositorState);
```
