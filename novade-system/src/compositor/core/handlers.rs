use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use smithay::wayland::output::{OutputHandler, OutputManagerState, OutputData};
// Adjust path to where DesktopState is defined, assuming it's in a sibling 'state' module
use super::state::DesktopState;
use smithay::wayland::compositor::{CompositorHandler, CompositorState, SurfaceData as SmithayCoreSurfaceData, SurfaceAttributes};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use crate::compositor::surface_management::SurfaceData; // Your existing SurfaceData
use std::sync::{Arc, Mutex};
use smithay::wayland::foreign_toplevel::{ForeignToplevelHandler, ForeignToplevelManagerState};

impl OutputHandler for DesktopState {
    fn output_state(&mut self) -> &mut OutputManagerState {
        &mut self.output_manager_state
    }

    fn new_output(&mut self, _output: &WlOutput, _output_data: &OutputData) {
        // This is called when a client binds to an existing wl_output global.
        // The Output itself (the physical output concept) is created when the backend
        // (or in our case, placeholder logic in main.rs) detects/creates a physical output
        // and calls Output::create_global().
        tracing::info!(
            output_resource_id = ?_output.id(),
            output_name = %_output_data.name(),
            output_description = %_output_data.description(),
            "Client bound to WlOutput"
        );
    }

    fn output_destroyed(&mut self, _output: &WlOutput, _output_data: &OutputData) {
        // This is called if a client no longer needs a WlOutput (e.g., client disconnects).
        // The physical Output and its global might still exist if other clients are bound
        // or if the compositor intends to keep the output available.
        tracing::info!(
            output_resource_id = ?_output.id(),
            output_name = %_output_data.name(),
            output_description = %_output_data.description(),
            "Client destroyed WlOutput resource"
        );
    }
}

// --- SeatHandler Implementation ---
use smithay::input::{SeatHandler, SeatState, Seat, pointer::CursorImageStatus};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
// DesktopState is already imported via `use super::state::DesktopState;`

impl SeatHandler for DesktopState {
    type KeyboardFocus = WlSurface; // Type for keyboard focus target
    type PointerFocus = WlSurface;  // Type for pointer focus target
    type TouchFocus = WlSurface;    // Type for touch focus target (can be WlSurface or a custom type)

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        // This callback is primarily for keyboard focus changes.
        // Smithay's SeatState manages the actual focus state. This is a notification.
        tracing::info!(
            seat_name = %seat.name(),
            new_focus_surface_id = ?focused.map(WlSurface::id),
            "Keyboard focus changed (SeatHandler::focus_changed)"
        );
        // TODO: Update internal compositor state if needed based on focus change.
        // For example, signal the previously focused window it lost focus,
        // and the newly focused window it gained focus (e.g., for drawing active decorations).
        // This might involve calling methods on ManagedWindow or sending domain events.
    }

    fn cursor_image(&mut self, seat: &Seat<Self>, image: CursorImageStatus) {
        // This callback is triggered when a client requests a cursor change
        // (e.g., via wl_pointer.set_cursor).
        tracing::debug!(
            seat_name = %seat.name(),
            cursor_status = ?image,
            "Cursor image status updated by client"
        );
        
        // Update the shared cursor status. The rendering logic will use this
        // to decide which cursor image (or none) to draw.
        let mut current_status_guard = self.current_cursor_status.lock().unwrap();
        *current_status_guard = image;
        
        // No direct rendering here. The main render loop will query `current_cursor_status`.
    }
}

// --- DmabufHandler Implementation ---
use smithay::wayland::dmabuf::{DmabufHandler, DmabufState, ImportNotifier, DmabufGlobalData};

impl DmabufHandler for DesktopState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }

    /// Handles the initial proposal of a DMABUF by a client for import.
    ///
    /// This function is called by Smithay as part of the `wl_drm` (or more generally,
    /// `zwp_linux_dmabuf_v1`) protocol when a client requests to create a `wl_buffer`
    /// from a DMABUF.
    ///
    /// # Role in DMABUF Protocol Flow
    ///
    /// 1.  Client calls `zwp_linux_dmabuf_v1.create_params` to get a `zwp_linux_buffer_params_v1` object.
    /// 2.  Client adds DMABUF file descriptors, format, dimensions, modifiers, etc., to the params object.
    /// 3.  Client calls `zwp_linux_buffer_params_v1.create_immed` (or `.create`) to request `wl_buffer` creation.
    /// 4.  This `dmabuf_imported` callback is invoked on the compositor side.
    ///
    /// # Current Implementation
    ///
    /// This implementation immediately notifies the client of success by calling `notifier.successful()`.
    /// This means the compositor acknowledges the parameters and is willing to *attempt* an import.
    /// The *actual* import into the renderer (e.g., GLES2 creating an EGLImage) and validation
    /// against renderer capabilities happens later, typically when the client attaches the
    /// `wl_buffer` to a `wl_surface` and commits the surface (see `CompositorHandler::commit`).
    ///
    /// If the actual import fails during the commit phase, the buffer might be rejected at that point,
    /// or rendering might fail for that surface.
    ///
    /// # Parameters
    ///
    /// - `_global`: Data associated with the `DmabufGlobal` itself (e.g., supported formats/modifiers
    ///   advertised by the compositor). Currently unused in this specific logging logic but available.
    /// - `notifier`: An [`ImportNotifier`] used to signal the client whether the DMABUF parameters
    ///   are accepted (`.successful()`) or rejected (`.failed()`).
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If the notification was sent successfully.
    /// - `Err(std::io::Error)`: If sending the notification to the client failed.
    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobalData, // Data associated with the DmabufGlobal itself
        notifier: ImportNotifier,   // Used to signal success or failure of the import operation
    ) -> Result<(), std::io::Error> { // Smithay 0.10.x: dmabuf_imported returns Result<(), std::io::Error>
        // This function is called when a client creates a wl_buffer from a DMABUF fd.
        // It doesn't mean the buffer is ready for rendering yet, or that the import
        // to a specific renderer (like GLES2) has happened. That typically occurs
        // when the buffer is committed to a surface and the renderer attempts to use it.

        // For now, we just log the event and immediately notify success.
        // In a real implementation, you might:
        // - Validate the DMABUF properties (e.g., against supported formats/modifiers if known early).
        // - Store some metadata about the DMABUF if needed before it's committed.
        // - Defer the success notification until the renderer has actually tried to import it
        //   (though this makes the Wayland protocol handling more complex, often success is signaled here).
        let client_id = notifier.client_id();
        tracing::debug!(
            client_info = ?client_id,
            dmabuf_global_data = ?_global, // DmabufGlobalData has a Debug impl
            "DMABUF import proposed by client. Notifying success immediately. \
            Actual import into renderer will occur on surface commit."
        );

        // Notify the client that the DMABUF parameters are acceptable from the compositor's perspective.
        // This does *not* mean the GPU has accepted it yet.
        notifier.successful();
        
        Ok(())
    }
}

impl CompositorHandler for DesktopState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        // Call Smithay's processing for buffer attachment, role checks etc.
        // This will update the SurfaceAttributes in WlSurface::data_map if a buffer is attached.
        if let Err(e) = smithay::wayland::compositor::on_commit_buffer_handler(surface) {
            tracing::warn!(surface_id = ?surface.id(), "Error in on_commit_buffer_handler: {}", e);
            // Potentially send a protocol error to the client if appropriate
        }

        // Ensure our SurfaceData is present and update it.
        let surface_data_exists = surface.data_map().has::<Arc<Mutex<SurfaceData>>>();

        if !surface_data_exists {
            // If it's the first commit and our data isn't there, create and insert it.
            // This might also be done when the wl_surface is first created via wl_compositor global,
            // but doing it on first commit is also a common pattern.
            let new_surface_data = Arc::new(Mutex::new(SurfaceData::new(
                format!("surface-{}", surface.id().protocol_id()) // Example ID
            )));
            surface.data_map().insert_if_missing_threadsafe(|| new_surface_data);
            tracing::debug!(surface_id = ?surface.id(), "New SurfaceData created and associated on commit.");
        }

        // Retrieve our SurfaceData (it should exist now)
        let surface_data_arc_mutex = surface.data_map().get::<Arc<Mutex<SurfaceData>>>().cloned();

        if let Some(arc_mutex) = surface_data_arc_mutex {
            let mut surface_data_guard = arc_mutex.lock().unwrap();

            // Check for an attached buffer using Smithay's core surface data
            let smithay_core_surface_data = surface.data_map().get::<SmithayCoreSurfaceData>().unwrap(); // Smithay ensures this exists
            let surface_attributes = smithay_core_surface_data.current_state(); // Get SurfaceAttributes from current_state

            if let Some(wl_buffer) = surface_attributes.buffer.as_ref() {
                // A buffer is attached, update our SurfaceData.
                // The texture_handle would be created/updated by the renderer when it processes this commit.
                // For now, we can update other buffer-related info if necessary.
                // Example: if SurfaceData tracks buffer size or damage.
                // This part depends on how much SurfaceData duplicates SmithayCoreSurfaceData vs extends it.
                // For now, mostly log. The renderer will handle texture creation from this buffer.
                tracing::debug!(surface_id = ?surface.id(), buffer_id = ?wl_buffer.id(), "Surface committed with a buffer.");

                // If your SurfaceData needs to store buffer dimensions or other attributes from SmithayCoreSurfaceData:
                // surface_data_guard.current_buffer_info = Some(AttachedBufferInfo {
                //     buffer: wl_buffer.clone(),
                //     dimensions: surface_attributes.buffer_dimensions.unwrap_or_default(),
                //     scale: surface_attributes.buffer_scale,
                //     transform: surface_attributes.buffer_transform,
                // });
            } else {
                // No buffer attached, or buffer was detached.
                // Clear relevant parts of your SurfaceData, e.g., texture handle.
                surface_data_guard.current_buffer_info = None;
                surface_data_guard.texture_handle = None;
                tracing::debug!(surface_id = ?surface.id(), "Surface committed without a buffer (or buffer detached).");
            }

            // Handle damage. Smithay's on_commit_buffer_handler usually clears pending damage and moves it to current.
            // We might need to accumulate it in our SurfaceData if the renderer needs it in a specific format.
            // For example:
            // surface_data_guard.damage_buffer_coords.clear();
            // surface_data_guard.damage_buffer_coords.extend_from_slice(&surface_attributes.damage_buffer);

        } else {
            tracing::error!(surface_id = ?surface.id(), "SurfaceData Arc<Mutex<_>> not found after attempting to ensure its presence!");
        }

        // Further processing for XDG surfaces or other roles, often done by calling methods on ManagedWindow
        // which would then access this SurfaceData.
        // Example:
        // if let Some(managed_window_arc) = find_managed_window_by_wl_surface(self, surface) {
        //     managed_window_arc.on_commit(); // Let ManagedWindow update its internal state from WlSurface
        // }
    }
    // Add other CompositorHandler methods if needed, e.g., new_surface, surface_destroyed
}

impl ForeignToplevelHandler for DesktopState {
    fn foreign_toplevel_state(&mut self) -> &mut ForeignToplevelManagerState {
        &mut self.foreign_toplevel_state
    }
}
