use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use smithay::wayland::output::{OutputHandler, OutputManagerState, OutputData};
// Adjust path to where DesktopState is defined, assuming it's in a sibling 'state' module
use super::state::DesktopState; 

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
        
        tracing::info!(
            "DMABUF import proposed by client (via ImportNotifier). Notifying success immediately. \
            Actual import into renderer will occur on surface commit."
        );

        // Notify the client that the DMABUF parameters are acceptable from the compositor's perspective.
        // This does *not* mean the GPU has accepted it yet.
        notifier.successful();
        
        Ok(())
    }
}
