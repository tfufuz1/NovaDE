use crate::compositor::core::state::DesktopState;
use crate::input::errors::InputError;
use smithay::{
    input::Seat,
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Serial},
};
use std::sync::Weak; // For Weak<WlSurface>

/// Sets the keyboard focus for a given seat.
///
/// This function changes the currently focused surface for keyboard input. It informs
/// Smithay's `KeyboardHandle` about the focus change, which in turn handles sending
/// `wl_keyboard.enter` and `wl_keyboard.leave` events to the appropriate clients.
/// The focus state is also updated within the seat's `XkbKeyboardData`.
///
/// # Arguments
///
/// * `desktop_state`: A mutable reference to the global `DesktopState`.
/// * `seat_name`: The name of the seat whose focus is to be set.
/// * `surface`: An `Option<&WlSurface>` representing the surface to focus. If `None`,
///   focus is cleared.
/// * `serial`: The serial of the event that triggered the focus change.
///
/// # Returns
///
/// * `Ok(())`: If the focus was successfully set or if the focus did not change.
/// * `Err(InputError)`: If the seat or its keyboard/XKB data is not found.
pub fn set_keyboard_focus(
    desktop_state: &mut DesktopState,
    seat_name: &str,
    surface: Option<&WlSurface>,
    serial: Serial,
) -> Result<(), InputError> {
    // Get the primary seat.
    // TODO: Support multiple seats by looking up seat_name in SeatState.
    if desktop_state.seat.name() != seat_name {
        return Err(InputError::SeatNotFound(seat_name.to_string()));
    }
    let seat = &desktop_state.seat;

    // Get the KeyboardHandle from the seat.
    let keyboard_handle = seat.get_keyboard().ok_or_else(|| {
        tracing::error!("No keyboard handle found for seat '{}' to set focus.", seat_name);
        InputError::KeyboardHandleNotFound(seat_name.to_string())
    })?;

    // Get the mutable XkbKeyboardData for seat_name from desktop_state.keyboard_data_map.
    let xkb_data_arc = desktop_state
        .keyboard_data_map
        .get_mut(seat_name) // Get mutable reference to Arc<XkbKeyboardData>
        .ok_or_else(|| {
            tracing::error!("XkbKeyboardData not found for seat '{}'.", seat_name);
            InputError::XkbConfigError {
                seat_name: seat_name.to_string(),
                message: "XkbKeyboardData not found during focus change.".to_string(),
            }
        })?;
    
    // To modify XkbKeyboardData, we need mutable access. If it's Arc<Mutex<XkbKeyboardData>>,
    // we'd lock it. If it's Arc<XkbKeyboardData> and XkbKeyboardData has interior mutability (e.g. Mutex fields),
    // that's fine. Or, if XkbKeyboardData itself needs to be replaced (e.g. to update focused_surface_on_seat
    // which is not behind a Mutex), we might need `Arc::make_mut` or a different structure.
    // The current XkbKeyboardData has `focused_surface_on_seat: Option<Weak<WlSurface>>`.
    // This should be updatable if we have `&mut XkbKeyboardData`.
    // `HashMap::get_mut` on `Arc<XkbKeyboardData>` gives `&mut Arc<XkbKeyboardData>`.
    // This doesn't allow mutating `XkbKeyboardData` unless `Arc::get_mut` can be used,
    // which requires the Arc to be uniquely owned at that moment. This is unlikely here.

    // Assuming XkbKeyboardData's fields relevant to focus (like repeat_timer, focused_surface_on_seat)
    // are designed for interior mutability (e.g. wrapped in Mutex or Cell) or can be updated
    // via methods that take `&self`.
    // For `focused_surface_on_seat: Option<Weak<WlSurface>>`, direct mutation is needed.
    // This implies XkbKeyboardData itself should probably be wrapped in a Mutex inside the Arc,
    // i.e., `HashMap<String, Arc<Mutex<XkbKeyboardData>>>`.
    // Or, `XkbKeyboardData`'s fields are `Mutex<Option<TimerHandle>>`, etc.
    // The current plan for XkbKeyboardData doesn't specify interior mutability for all fields.
    // Let's assume for now `focused_surface_on_seat` is `Mutex<Option<Weak<WlSurface>>>` in XkbKeyboardData.
    // (This change would be needed in xkb_config.rs)

    // If `Arc::get_mut` is not viable, and `XkbKeyboardData` isn't `Mutex`-wrapped at the top level,
    // we might need to reconstruct `XkbKeyboardData` or use interior mutability for `focused_surface_on_seat`.
    // Let's proceed with the assumption that we can get a mutable reference to XkbKeyboardData or its relevant fields.
    // For now, we'll assume `Arc::make_mut` can be used, or that the structure of keyboard_data_map will be
    // `HashMap<String, XkbKeyboardData>` (if not shared) or `HashMap<String, Arc<Mutex<XkbKeyboardData>>>`.
    // Given the `Arc` usage, interior mutability for `XkbKeyboardData` fields is the most likely correct path.
    // I will assume `focused_surface_on_seat` is `Mutex<Option<Weak<WlSurface>>>` inside XkbKeyboardData.
    // (This change needs to be reflected in xkb_config.rs)

    let current_focused_weak_opt = xkb_data_arc.focused_surface_on_seat.lock().unwrap().clone(); // Clone Weak
    let new_surface_id = surface.map(|s| s.id());
    let current_surface_id = current_focused_weak_opt.as_ref().and_then(|wk| wk.upgrade()).map(|s| s.id());

    if new_surface_id == current_surface_id {
        tracing::trace!("Keyboard focus for seat '{}' unchanged (surface: {:?}).", seat_name, new_surface_id);
        return Ok(());
    }

    tracing::info!(
        "Setting keyboard focus for seat '{}' to surface {:?} (was {:?}) with serial {:?}",
        seat_name,
        new_surface_id,
        current_surface_id,
        serial
    );

    // Inform Smithay's KeyboardHandle. This sends wl_keyboard.enter/leave.
    // `set_focus` will also trigger `SeatHandler::focus_changed`.
    keyboard_handle.set_focus(surface, serial, Some(tracing::Span::current()));

    // Update our internal tracking of the focused surface in XkbKeyboardData.
    // This is now redundant if SeatHandler::focus_changed updates active_input_surface in DesktopState,
    // and XkbKeyboardData refers to that or gets updated by focus_changed.
    // However, XkbKeyboardData might need its own copy for repeat key logic.
    let mut focused_guard = xkb_data_arc.focused_surface_on_seat.lock().unwrap();
    *focused_guard = surface.map(|s| s.downgrade());


    // If focus is cleared, cancel key repetition.
    if surface.is_none() {
        if let Some(timer) = xkb_data_arc.repeat_timer.lock().unwrap().take() {
            timer.cancel();
            *xkb_data_arc.repeat_info.lock().unwrap() = None;
            tracing::info!("Key repetition canceled due to focus loss on seat '{}'.", seat_name);
        }
    }
    
    // Optional: Notify domain layer about focus change.
    // This is likely already handled by `SeatHandler::focus_changed` updating `DesktopState::active_input_surface`
    // and potentially a domain notifier there. If more specific data from XkbKeyboardData is needed
    // by the domain layer, it could be sent from here.

    Ok(())
}
