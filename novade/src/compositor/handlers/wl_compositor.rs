//! Handler for the `wl_compositor` global.
//!
//! This module implements the `smithay::wayland::compositor::CompositorHandler` trait
//! for `NovaCompositorState`. It is responsible for managing the lifecycle of `wl_surface`
//! objects, including their creation, destruction, and achnowledging commits of their state
//! (e.g., attaching buffers, damage information).

#![allow(unused_variables)]
#![allow(dead_code)]

use crate::compositor::state::{NovaCompositorState, SurfaceData};
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor,
    desktop::WindowAutoTune,
    reexports::wayland_server::{
        protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
        DisplayHandle, UserDataMap, // Added UserDataMap for new_surface
    },
    wayland::compositor::{CompositorHandler, CompositorState, SurfaceAttributes, SurfaceDataExt},
};

// TODO: Define a specific state struct for wl_compositor if complex logic arises,
// e.g., `WlCompositorHandlerState`. For now, its responsibilities are integrated
// into `NovaCompositorState` which holds `smithay::wayland::compositor::CompositorState`.

impl CompositorHandler for NovaCompositorState {
    /// Provides access to Smithay's `CompositorState` which tracks `wl_surface`s.
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    /// Called when a client commits changes to a `wl_surface`.
    ///
    /// This typically means the client has attached a new buffer, specified damage,
    /// or updated other surface properties. Smithay's `on_commit_buffer_handler`
    /// utility is called to handle standard buffer attachment and damage.
    ///
    /// TODO: Implement detailed damage tracking and scheduling a render for this surface.
    fn commit(&mut self, surface: &WlSurface) {
        slog::debug!(self.logger, "Commit on surface {:?}", surface.id());

        // Use Smithay's helper to handle buffer attachment and damage.
        // This updates the surface's internal state regarding its current buffer.
        on_commit_buffer_handler::<Self>(surface);

        // TODO: Further actions after a commit:
        // 1. Retrieve custom `SurfaceData` if needed.
        // 2. Update internal damage tracking for the output(s) this surface is on.
        //    - This involves translating surface-local damage to global coordinates.
        // 3. Schedule a render pass for the affected outputs.
        // 4. If the surface is an XDG toplevel, its geometry or state might need updating.

        // Example: Accessing NovaDE's custom SurfaceData (currently a placeholder)
        let udata_map = surface.data::<UserDataMap>().unwrap();
        if let Some(nova_surface_data) = udata_map.get::<SurfaceData>() {
            // slog::trace!(self.logger, "Custom SurfaceData on commit: {:?}", nova_surface_data);
        }
    }

    /// Called when a client creates a new `wl_surface`.
    ///
    /// Initializes `SurfaceData` (currently a placeholder) for the new surface
    /// and stores it in the surface's `UserDataMap`.
    fn new_surface(&mut self, surface: &WlSurface) {
        slog::debug!(self.logger, "New surface created: {:?}", surface.id());
        // Smithay's `CompositorState` automatically adds `SurfaceAttributes` via `SurfaceDataExt`.
        // We add our own `SurfaceData` for any NovaDE-specific state.
        surface
            .data::<UserDataMap>()
            .unwrap()
            .insert_if_missing_thread_safe(SurfaceData::default);
        // TODO: Any additional setup for a newly created surface.
    }

    /// Called when a client creates a new `wl_subsurface`.
    ///
    /// Initializes `SurfaceData` for the new subsurface.
    /// TODO: Implement logic to link the subsurface to its parent and manage its lifecycle.
    fn new_subsurface(
        &mut self,
        surface: &WlSurface,
        parent: &WlSurface,
    ) {
        slog::debug!(self.logger, "New subsurface {:?} created for parent {:?}", surface.id(), parent.id());
         surface
            .data::<UserDataMap>()
            .unwrap()
            .insert_if_missing_thread_safe(SurfaceData::default);
        // TODO: Subsurface-specific logic:
        // - Add to parent's subsurface list.
        // - Consider parent's visibility and effective damage.
    }

    /// Called when a `wl_surface` is destroyed by a client.
    ///
    /// Smithay handles the actual destruction of the Wayland object. This callback
    /// can be used for any custom cleanup related to the surface in `NovaCompositorState`.
    fn destroyed(&mut self, surface: &WlSurface) {
        slog::debug!(self.logger, "Surface destroyed: {:?}", surface.id());
        // TODO: Perform any necessary cleanup if `SurfaceData` or other compositor state
        //       was specifically tied to this surface beyond what UserDataMap handles.
        //       For example, if this surface was a drag-and-drop icon, clear that state.
    }
}

// Delegate wl_compositor requests to NovaCompositorState.
// This macro implements `GlobalDispatch<WlCompositor, CompositorData<D>>` and the
// necessary `Dispatch` trait for `NovaCompositorState`.
// `CompositorData` is a simple marker struct provided by Smithay for `wl_compositor` globals.
delegate_compositor!(NovaCompositorState);

// Note on `smithay::desktop::WindowAutoTune`:
// While not directly part of `wl_compositor` handling, this trait is often implemented
// on the main compositor state (`NovaCompositorState` in this case). It's used by
// higher-level shell protocols like XDG Shell to provide hints and automated
// management for window sizing, positioning, and state changes. Its implementation
// is typically found in `state.rs` or a dedicated shell management module.
```
