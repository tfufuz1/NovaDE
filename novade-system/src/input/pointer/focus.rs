use smithay::{
    input::{Seat, pointer::PointerHandle, pointer::Focus, SeatHandler}, // Added SeatHandler
    reexports::wayland_server::protocol::wl_surface::{WlSurface, Weak},
    utils::{Logical, Point, Serial},
};
use crate::{
    compositor::core::state::DesktopState,
    input::errors::InputError,
};

// This function is marked pub(super) as it's intended for use by event_translator.rs
pub(super) fn update_pointer_focus_and_send_motion(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    pointer_handle: &PointerHandle<DesktopState>,
    new_focus_surface: Option<WlSurface>, // This is Option<WlSurface>, not Option<&WlSurface> for ownership matching with current_focus
    surface_local_coords: Point<f64, Logical>,
    time: u32,
    serial: Serial,
) -> Result<(), InputError> {
    // Get the old focus from the pointer_handle. Smithay's PointerHandle::current_focus()
    // returns an Option<F>, where F is Self::Focus, which is WlSurface for DesktopState.
    let old_focus_wl_surface: Option<WlSurface> = pointer_handle.current_focus();

    // Smithay's PointerHandle::motion will internally handle sending wl_pointer.enter
    // and wl_pointer.leave events if the focus changes between surfaces.
    // It also sends wl_pointer.motion.
    pointer_handle.motion(
        time,
        new_focus_surface.as_ref(), // motion() expects Option<&WlSurface>
        serial,
        surface_local_coords,
        Some(tracing::Span::current()),
    );

    // After calling pointer_handle.motion(), Smithay itself would have called
    // SeatHandler::focus_changed if the *keyboard* focus changed due to a click.
    // However, for pointer focus, Smithay updates its internal state.
    // We need to update our DesktopState's active_input_surface if the *pointer* focus changed.
    // This is distinct from keyboard focus. DesktopState.active_input_surface is used by keyboard repeat.
    // It might be better to have a separate desktop_state.active_pointer_focus or similar if they can diverge.
    // For now, let's assume active_input_surface tracks the surface under the pointer for general input.

    let new_focus_has_changed = match (&old_focus_wl_surface, &new_focus_surface) {
        (Some(old_s), Some(new_s)) => old_s.id() != new_s.id(), // Compare by WlSurface ID
        (Some(_), None) => true, // Focus lost
        (None, Some(_)) => true, // Focus gained
        (None, None) => false,   // Still no focus
    };

    if new_focus_has_changed {
        // This update is for our compositor's internal tracking, potentially for keyboard focus on click,
        // or if other parts of the system need to know what's under the pointer.
        // The actual wl_pointer.enter/leave events are handled by pointer_handle.motion().
        // If active_input_surface is strictly for keyboard, this line might be conditional or removed.
        // Based on previous use (keyboard repeat), active_input_surface seems tied to keyboard.
        // Let's not update active_input_surface here directly, as set_keyboard_focus handles that.
        // This function's job is to ensure pointer events (enter, leave, motion) are sent.
        // The `SeatHandler::focus_changed` in DesktopState is for *keyboard* focus.

        tracing::debug!(
            "Pointer focus changed for seat '{}'. Old: {:?}, New: {:?}. wl_pointer.enter/leave sent by Smithay.",
            seat.name(),
            old_focus_wl_surface.as_ref().map(|s| s.id()),
            new_focus_surface.as_ref().map(|s| s.id())
        );
    }
    Ok(())
}
