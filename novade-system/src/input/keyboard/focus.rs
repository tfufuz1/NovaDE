use smithay::{
    input::{Seat, keyboard::KeyboardHandle},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    reexports::wayland_server::Weak, // Added Weak
    utils::Serial,
};
use crate::{
    compositor::core::state::DesktopState, // Pfad anpassen
    input::errors::InputError,
};

pub fn set_keyboard_focus(
    desktop_state: &mut DesktopState,
    seat_name: &str,
    surface: Option<&WlSurface>, // This is &WlSurface, not Weak<WlSurface>
    serial: Serial,
) -> Result<(), InputError> {
    tracing::debug!(seat = %seat_name, new_focus_surface_id = ?surface.map(|s| s.id()), ?serial, "Setting keyboard focus");

    // Find the seat. We need a reference to it, not a cloned one, to affect the original state.
    // However, SeatState::seats() returns an iterator of &Seat<D>.
    // Methods on KeyboardHandle take &self, so getting a reference to KeyboardHandle is fine.
    let seat = desktop_state.seat_state.seats()
        .find(|s| s.name() == seat_name)
        // We need to clone the KeyboardHandle from the seat, not the seat itself,
        // as set_focus is a method on KeyboardHandle.
        // Or, get the seat and then its keyboard handle.
        .ok_or_else(|| InputError::SeatNotFound(seat_name.to_string()))?;

    let keyboard_handle = seat.get_keyboard()
        .ok_or_else(|| InputError::KeyboardHandleNotFound(seat_name.to_string()))?;

    // Let Smithay's KeyboardHandle manage sending enter/leave events.
    // The SeatHandler::focus_changed in DesktopState will be called by this action.
    keyboard_handle.set_focus(surface, serial, Some(tracing::Span::current()));

    // Update XkbKeyboardData's focused_surface_on_seat, which is used by key repeat logic.
    // This needs to be done after keyboard_handle.set_focus() because focus_changed (which also updates active_input_surface)
    // is called synchronously within set_focus.
    if let Some(xkb_data) = desktop_state.keyboard_data_map.get_mut(seat_name) {
        xkb_data.focused_surface_on_seat = surface.map(|s| s.downgrade()); // Store as Weak<WlSurface>

        if surface.is_none() { // If focus is lost (surface is None)
            // Cancel any ongoing key repeat for this seat.
            if let Some(timer) = xkb_data.repeat_timer.take() {
                timer.cancel();
                tracing::debug!("Keyboard focus lost on seat '{}', key repeat timer cancelled.", seat_name);
            }
            xkb_data.repeat_info = None;
            xkb_data.repeat_key_serial = None;
        }
    } else {
        // This case might be problematic if XkbKeyboardData is essential for keyboard operation.
        // However, focus setting itself might still be valid.
        tracing::warn!("Could not find XkbKeyboardData for seat '{}' while setting keyboard focus. Key repeat might be affected.", seat_name);
    }
    Ok(())
}
