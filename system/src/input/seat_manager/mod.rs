// system/src/input/seat_manager/mod.rs
use crate::compositor::core::state::DesktopState; // Path to DesktopState
use crate::input::errors::InputError;
use crate::input::keyboard::xkb_config::XkbKeyboardData; // Path to XkbKeyboardData
use smithay::input::{Seat, SeatState, keyboard::KeyboardConfig}; // Add KeyboardConfig
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::reexports::wayland_server::protocol::wl_seat::WlSeat; // For UserData
use smithay::wayland::seat::WaylandSeatData; // Smithay's UserData for wl_seat
use std::sync::{Arc, Mutex}; // Ensure Arc and Mutex are used

pub fn create_seat(
    desktop_state: &mut DesktopState,
    display_handle: &DisplayHandle,
    seat_name: String,
) -> Result<Seat<DesktopState>, InputError> {
    tracing::info!("Erstelle neuen Seat: {}", seat_name);

    if desktop_state.seat_state.seats().any(|s| s.name() == seat_name) {
        tracing::warn!("Seat '{}' existiert bereits.", seat_name);
        // Depending on desired behavior, either return the existing seat
        // or an error. The spec implies creation, so an error for existing might be appropriate.
        // However, the original spec's create_seat in the document doesn't show error handling for this.
        // For now, let's assume we want to proceed and get the new_wl_seat,
        // smithay's SeatState handles internal seat uniqueness by name for wl_seat.
        // Smithay 0.3.0 `new_wl_seat` will actually return the existing seat if the name matches.
    }

    let seat = desktop_state.seat_state.new_wl_seat(
        display_handle,
        seat_name.clone(),
        Some(tracing::Span::current()) // For logging context
    );
    
    // Set default UserData for the wl_seat global, if not already set by new_wl_seat
    // This is good practice for Smithay's handler patterns.
    // Note: Smithay's SeatState::new_wl_seat already initializes WaylandSeatData for the WlSeat.
    // So, this explicit insertion might be redundant but harmless if it checks existence.
    let seat_resource: &WlSeat = seat.global().resource(); // Get the WlSeat resource associated with the global
    seat_resource.user_data().insert_if_missing(WaylandSeatData::default);


    // Initialize XkbKeyboardData for this seat.
    // The actual keyboard capability will be added when a keyboard device is detected.
    // We use a default configuration for now.
    let default_xkb_config = KeyboardConfig::default();
    match XkbKeyboardData::new(&default_xkb_config) {
        Ok(xkb_data) => {
            // The document specifies desktop_state.keyboards.
            // The DesktopState was defined with keyboard_data_map.
            desktop_state.keyboard_data_map.insert(seat_name.clone(), Arc::new(Mutex::new(xkb_data)));
            tracing::debug!("XkbKeyboardData für Seat '{}' initialisiert und in Mutex verpackt.", seat_name);
        }
        Err(e) => {
            tracing::error!("Fehler beim Initialisieren von XkbKeyboardData für Seat '{}': {:?}", seat_name, e);
            // Return an error, as XKB data is crucial for keyboard input.
            return Err(InputError::XkbConfigError {
                seat_name, // seat_name is moved if we return here, clone if needed later
                message: format!("Failed to initialize XKB keyboard data: {}", e),
            });
        }
    }

    // Store the primary seat information if this is the one being created
    // (e.g. if seat_name matches a configured primary seat name like "seat0")
    // This logic might be better placed where DesktopState initializes its own `seat` field.
    // For now, we assume DesktopState.seat is already the one returned by new_wl_seat
    // if it's the first/primary one. The original DesktopState initialization already calls new_wl_seat.
    // This function could be used to create *additional* seats if needed,
    // or to ensure the primary seat is fully configured.

    tracing::info!("Seat '{}' erfolgreich erstellt und konfiguriert. Capabilities (Tastatur, Zeiger, Touch) werden beim Hinzufügen von Geräten dynamisch gesetzt.", seat_name);

    Ok(seat)
}
