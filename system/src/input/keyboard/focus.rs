use crate::compositor::core::state::DesktopState;
use crate::input::errors::InputError;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::Serial;
use smithay::input::Seat; // For type annotation on seat

pub fn set_keyboard_focus(
    desktop_state: &mut DesktopState,
    seat_name: &str,
    surface_to_focus: Option<&WlSurface>, // The surface to focus, or None to clear focus
    serial: Serial,
) -> Result<(), InputError> {
    tracing::debug!(
        seat_name = %seat_name,
        new_focus_surface_id = ?surface_to_focus.map(|s| s.id()),
        serial = ?serial,
        "Aufruf zum Setzen des Tastaturfokus."
    );

    let seat: Seat<DesktopState> = desktop_state
        .seat_state
        .seats()
        .find(|s| s.name() == seat_name)
        .cloned() // Clone the Arc<SeatInner<D>>
        .ok_or_else(|| {
            tracing::error!("Seat '{}' nicht gefunden für Fokusänderung.", seat_name);
            InputError::SeatNotFound(seat_name.to_string())
        })?;

    let keyboard_handle = seat.get_keyboard().ok_or_else(|| {
        tracing::warn!(
            "Kein Keyboard-Handle für Seat '{}' beim Versuch, Fokus zu setzen.",
            seat_name
        );
        InputError::KeyboardHandleNotFound(seat_name.to_string())
    })?;
    
    // Access XkbKeyboardData, assuming it's stored in an Arc<Mutex<XkbKeyboardData>>
    let xkb_data_arc = desktop_state.keyboard_data_map.get(seat_name).cloned().ok_or_else(|| {
        tracing::error!("Keine XKB-Daten für Seat '{}' beim Fokussetzen.", seat_name);
        InputError::XkbConfigError {
            seat_name: seat_name.to_string(),
            message: "XKB data not found for seat.".to_string(),
        }
    })?;

    let mut xkb_data_guard = xkb_data_arc.lock().unwrap(); // Acquire lock

    let old_focused_surface_weak = xkb_data_guard.focused_surface_on_seat.clone();
    let old_focused_surface_option = old_focused_surface_weak.as_ref().and_then(|wk| wk.upgrade());

    // Check if focus is actually changing
    if old_focused_surface_option.as_ref().map(|s| s.id()) == surface_to_focus.map(|s| s.id()) {
        tracing::trace!("Tastaturfokus für Seat '{}' ist bereits auf {:?}, keine Änderung.", seat_name, surface_to_focus.map(|s|s.id()));
        return Ok(());
    }

    tracing::info!(
        "Tastaturfokus für Seat '{}' wird geändert von {:?} zu {:?}.",
        seat_name,
        old_focused_surface_option.as_ref().map(|s|s.id()),
        surface_to_focus.map(|s|s.id())
    );

    // Call Smithay's method to set focus. This will trigger wl_keyboard.enter/leave.
    // It will also trigger DesktopState::SeatHandler::focus_changed callback.
    keyboard_handle.set_focus(surface_to_focus, serial, Some(tracing::Span::current()));

    // Update our internal tracking of the focused surface
    xkb_data_guard.focused_surface_on_seat = surface_to_focus.map(|s| s.downgrade());

    // If focus is lost or changed, cancel any ongoing key repeat for this keyboard.
    // This check is important because the timer callback for repeat also checks this,
    // but cancelling here is more immediate if focus changes explicitly.
    if xkb_data_guard.repeat_info.is_some() {
        if let Some(timer) = xkb_data_guard.repeat_timer.take() {
            timer.cancel();
            tracing::debug!("Key repeat timer cancelled für Seat '{}' aufgrund von Fokusänderung.", seat_name);
        }
        xkb_data_guard.repeat_info = None;
        xkb_data_guard.repeat_key_serial = None;
    }
    
    // The SeatHandler::focus_changed method in DesktopState will be called by keyboard_handle.set_focus()
    // and is responsible for any further high-level focus change notifications (e.g., to domain layer).

    Ok(())
}
