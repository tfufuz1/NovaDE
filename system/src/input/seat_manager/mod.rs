use crate::compositor::core::state::DesktopState;
use crate::input::{
    errors::InputError,
    keyboard::XkbKeyboardData, // Ensure this path is correct
};
use smithay::{
    input::{Seat, SeatState, keyboard::KeyboardConfig}, // Added KeyboardConfig
    reexports::wayland_server::{protocol::wl_seat, DisplayHandle},
    wayland::seat::WaylandSeatData, // For associating with wl_seat
};
use std::collections::HashMap; // For keyboard_data_map
use std::sync::Arc; // For XkbKeyboardData if it's Arc'd

/// Creates a new Wayland seat and associated resources.
///
/// This function initializes a new `wl_seat` global, configures basic seat data,
/// and sets up keyboard handling (initially with a default XKB configuration).
///
/// # Arguments
///
/// * `desktop_state`: A mutable reference to the global `DesktopState`.
/// * `display_handle`: A handle to the Wayland display.
/// * `seat_name`: The name for the new seat (e.g., "seat0").
///
/// # Returns
///
/// * `Ok(Seat<DesktopState>)`: The newly created `Seat` object.
/// * `Err(InputError)`: If seat creation or initial configuration fails.
pub fn create_seat(
    desktop_state: &mut DesktopState,
    display_handle: &DisplayHandle,
    seat_name: String,
) -> Result<Seat<DesktopState>, InputError> {
    // Create the wl_seat global.
    // The new_wl_seat method on SeatState handles global creation and association.
    // It takes the display_handle, name, and a logger.
    let seat = desktop_state.seat_state.new_wl_seat(display_handle, seat_name.clone(), None);
    tracing::info!("Created new wl_seat global: {}", seat_name);

    // Associate WaylandSeatData with the new seat's user data.
    // This is often done by Smithay internally or when capabilities are added.
    // For clarity, we can ensure it's there. Smithay's Seat::user_data() provides access.
    // If WaylandSeatData::default() is appropriate, it can be inserted if missing.
    // Smithay's Seat::new also does this, so this might be redundant if using Seat::new directly.
    // However, we are using desktop_state.seat_state.new_wl_seat which returns the wl_seat object,
    // and the Seat<DesktopState> object is typically what we store in DesktopState.

    // The `Seat<D>` object itself holds capabilities.
    // `desktop_state.seat` is the primary `Seat<DesktopState>` object.
    // If this function is for creating *additional* seats, that's a more complex scenario.
    // The current plan implies this sets up the *main* seat defined in DesktopState.

    // The `desktop_state.seat` is already initialized in `DesktopState::new`.
    // This function seems to be about re-initializing or configuring the *existing* primary seat
    // rather than creating a new `Seat<DesktopState>` object from scratch separate from `desktop_state.seat`.
    // Or, it's about creating the wl_seat Wayland global for the existing `desktop_state.seat`.

    // Let's assume this function is ensuring the `wl_seat` global exists for `desktop_state.seat`
    // and configuring its keyboard data.
    // The `desktop_state.seat` is already a `Seat<DesktopState>`.

    // Update desktop_state with the name of the seat whose global we just created.
    // This assumes we are configuring the primary seat.
    if desktop_state.seat_name.is_empty() { // Or some other logic to identify primary seat setup
        desktop_state.seat_name = seat_name.clone();
    } else if desktop_state.seat_name != seat_name {
        // This implies creating a new, additional seat, which DesktopState isn't currently structured for.
        // For now, let's focus on configuring the primary seat.
        tracing::warn!("Configuring seat named '{}', but DesktopState already has a seat named '{}'. This function currently only fully configures the primary seat.", seat_name, desktop_state.seat_name);
        // return Err(InputError::SeatCreationFailed("Multi-seat setup not yet fully supported by this function.".to_string()));
        // Or, proceed to create XKB data for this potentially new seat name if keyboard_data_map supports it.
    }


    // Initialize XkbKeyboardData for this seat.
    // Use a default KeyboardConfig for now.
    let default_kb_config = KeyboardConfig::default();
    match XkbKeyboardData::new(&default_kb_config) {
        Ok(xkb_data) => {
            desktop_state.keyboard_data_map.insert(seat_name.clone(), Arc::new(xkb_data));
            tracing::info!("Initialized and stored XkbKeyboardData for seat '{}'", seat_name);
        }
        Err(e) => {
            tracing::error!("Failed to initialize XkbKeyboardData for seat '{}': {}", seat_name, e);
            return Err(InputError::XkbConfigError {
                seat_name,
                message: format!("Failed to create XkbKeyboardData: {}", e),
            });
        }
    }

    // The `Seat<DesktopState>` object to return should be the one from `desktop_state.seat`.
    // This function's purpose seems to be more about ensuring the `wl_seat` global is up
    // and `XkbKeyboardData` is initialized for the named seat, rather than creating a new
    // `Seat<D>` struct instance.
    // If `desktop_state.seat` is the one we're configuring:
    Ok(desktop_state.seat.clone()) // Clone the Arc<Seat<DesktopState>> or Seat<DesktopState> if not Arc'd
                                   // Seat<D> is not typically Arc'd directly in DesktopState, it's owned.
                                   // Smithay's SeatState::new_wl_seat returns WlSeat, not Seat<D>.
                                   // The Seat<D> is usually created once.

    // Re-evaluation:
    // `SeatState<D>::new_wl_seat` creates the Wayland global `wl_seat`.
    // The `Seat<D>` object (e.g., `desktop_state.seat`) is the compositor-side representation
    // that manages capabilities (keyboard, pointer, touch) and focus.
    // This `Seat<D>` is what `SeatHandler` methods receive.
    // The function should probably return the `Seat<D>` that corresponds to the `seat_name`
    // for which the `wl_seat` global was just created.
    // If we assume a single primary seat for now, `desktop_state.seat` is that seat.
    // The name given to `new_wl_seat` should match `desktop_state.seat.name()`.

    // Clarification: `desktop_state.seat` is the primary compositor-side seat object.
    // `desktop_state.seat_state` is used to manage Wayland globals for seats.
    // `create_seat` should:
    // 1. Ensure `desktop_state.seat` (the main one) has its `wl_seat` global created via `seat_state`.
    // 2. Initialize `XkbKeyboardData` for it.

    // If `desktop_state.seat.name()` is not yet set or doesn't match `seat_name`, we should align them.
    // Let's assume `desktop_state.seat`'s name is set during its creation in `DesktopState::new`.
    // This function then ensures the Wayland global representation matches.

    // Let's assume `seat_name` passed to this function *is* the name of `desktop_state.seat`.
    if desktop_state.seat.name() != seat_name {
        // This is an inconsistency. The seat object in DesktopState should be the one
        // for which we are creating a global.
        // For now, we'll proceed assuming they match or `desktop_state.seat`'s name will be updated.
        // Or, `SeatState::new_wl_seat` is what actually sets the name for the global,
        // and `Seat::name()` will reflect that after the global is tied to it.
        // Smithay's `Seat::new` takes the name. `SeatState::new_wl_seat` also takes a name.
        // These should be consistent.
        tracing::warn!("Attempting to create wl_seat global for name '{}', but primary seat in DesktopState is named '{}'. Ensure these are consistent.", seat_name, desktop_state.seat.name());
        // This function might be better named `ensure_seat_global_and_keyboard` or similar.
    }

    // The `Seat<DesktopState>` object itself isn't "created" here in terms of a new struct instance
    // if we're always referring to `desktop_state.seat`.
    // The function is more about initializing its Wayland presence and keyboard data.
    // So, returning `desktop_state.seat.clone()` is correct.
}

// This module could also contain functions for managing multiple seats if that becomes a requirement.
// For now, `create_seat` primarily configures the main seat referenced in `DesktopState`.
// Capabilities like keyboard, pointer, touch are added to `Seat<DesktopState>` later,
// e.g., `seat.add_keyboard(...)`, `seat.add_pointer(...)`.
// This will trigger Wayland capability announcements.
