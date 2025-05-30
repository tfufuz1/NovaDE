use smithay::{
    backend::input::{Event as BackendEvent, InputEvent as BackendInputEvent, KeyboardKeyEvent, KeyState, LibinputInputBackend},
    input::{Seat, keyboard::{KeyboardHandle, FilterResult, ModifiersState as SmithayModifiersState, KeysymHandle, KeyText}, SeatHandler}, // Added SeatHandler for DesktopState
    reexports::{
        calloop::{LoopHandle, Timer},
        input::event::keyboard::KeyboardEventTrait, // For common methods on KeyboardKeyEvent
        wayland_server::protocol::wl_surface::WlSurface,
        wayland_server::Weak, // Added Weak
    },
    utils::{Serial, TimeSpan, current_time},
};
use xkbcommon::xkb;
use crate::{
    compositor::core::state::DesktopState,
    input::{
        errors::InputError,
        keyboard::xkb_config::XkbKeyboardData,
        // No, this function is in xkb_config, not a free function here
        // keyboard::xkb_config::update_xkb_state_with_smithay_modifiers,
    },
};
use std::time::Duration;

const LIBINPUT_XKB_KEYCODE_OFFSET: u32 = 8;

// This function is intended to be called by the timer
fn repeat_timer_handler(desktop_state: &mut DesktopState, seat_name_str: &str) {
    let seat_name = seat_name_str.to_string(); // Own the string for safety with potential re-scheduling

    // It's crucial to re-fetch the seat and keyboard handle every time,
    // as they might have been removed or changed since the timer was set.
    let seat = match desktop_state.seat_state.seats().find(|s| s.name() == seat_name).cloned() {
        Some(s) => s,
        None => {
            tracing::warn!("[Repeat] Seat '{}' not found during repeat handling. Cancelling further repeats.", seat_name);
            if let Some(xkb_data) = desktop_state.keyboard_data_map.get_mut(&seat_name) {
                if let Some(timer) = xkb_data.repeat_timer.take() {
                    timer.cancel();
                }
                xkb_data.repeat_info = None;
            }
            return;
        }
    };

    let keyboard_handle = match seat.get_keyboard().cloned() {
        Some(kh) => kh,
        None => {
            tracing::warn!("[Repeat] No keyboard handle for seat '{}' during repeat. Cancelling.", seat_name);
            if let Some(xkb_data) = desktop_state.keyboard_data_map.get_mut(&seat_name) {
                if let Some(timer) = xkb_data.repeat_timer.take() {
                    timer.cancel();
                }
                xkb_data.repeat_info = None;
            }
            return;
        }
    };

    let xkb_data_option = desktop_state.keyboard_data_map.get_mut(&seat_name);
    let (libinput_keycode_to_repeat, xkb_keycode_to_repeat, rate_duration) = match xkb_data_option {
        Some(ref mut data) => {
            // Check if focus is still valid for repeat
            // focused_surface_on_seat is Option<Weak<WlSurface>>
            let focus_still_valid = data.focused_surface_on_seat.as_ref().map_or(false, |weak_surf| weak_surf.upgrade().is_some());
            if !focus_still_valid {
                 tracing::debug!("[Repeat] Focus lost on seat '{}', cancelling repeat.", seat_name);
                if let Some(timer) = data.repeat_timer.take() {
                    timer.cancel();
                }
                data.repeat_info = None;
                return;
            }

            if let Some((lib_kc, xkb_kc, _mods, _delay, rate)) = data.repeat_info {
                (lib_kc, xkb_kc, rate) // Return the necessary info
            } else {
                tracing::warn!("[Repeat] Repeat info missing for seat '{}' despite active timer. Cancelling.", seat_name);
                if let Some(timer) = data.repeat_timer.take() { // data is already mut xkb_data
                    timer.cancel();
                }
                return;
            }
        }
        None => {
            tracing::error!("[Repeat] XkbKeyboardData not found for seat '{}' during repeat. Should not happen if timer was set.", seat_name);
            return;
        }
    };

    let new_serial = Serial::now();
    let current_event_time = current_time().as_millis() as u32;

    // Use the original libinput_keycode for the input method
    keyboard_handle.input(libinput_keycode_to_repeat, KeyState::Pressed, new_serial, current_event_time, Some(tracing::Span::current()));

    // Re-fetch xkb_data for rescheduling, as the previous borrow might have ended or to be safe
    let xkb_data_for_reschedule = desktop_state.keyboard_data_map.get_mut(&seat_name)
        .expect("[Repeat] XkbKeyboardData vanished while trying to reschedule repeat timer.");

    xkb_data_for_reschedule.repeat_key_serial = Some(new_serial);

    // Reschedule the timer for the next repeat
    // Need to ensure the old timer is properly handled. It's taken if we got here.
    let seat_name_clone_for_next_timer = seat_name.to_string();
    if let Some(old_timer) = xkb_data_for_reschedule.repeat_timer.take() {
        old_timer.cancel(); // Explicitly cancel, though TimerHandle::drop also cancels.
    }

    match desktop_state.loop_handle.insert_timer(rate_duration, move |ds: &mut DesktopState| {
        repeat_timer_handler(ds, &seat_name_clone_for_next_timer);
    }) {
        Ok(new_timer_handle) => {
            xkb_data_for_reschedule.repeat_timer = Some(new_timer_handle);
            tracing::trace!("[Repeat] Key {} (XKB {}) repeated for seat '{}', next in {:?}.", libinput_keycode_to_repeat, xkb_keycode_to_repeat.raw(), seat_name, rate_duration);
        }
        Err(e) => {
            tracing::error!("[Repeat] Failed to insert key repeat timer for seat {}: {:?}", seat_name, e);
            // Clear repeat info if we can't schedule the next event
            xkb_data_for_reschedule.repeat_info = None;
            xkb_data_for_reschedule.repeat_timer = None;
        }
    }
}


pub fn handle_keyboard_key_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>, // seat is &Seat<DesktopState>
    event: KeyboardKeyEvent<LibinputInputBackend>, // event is KeyboardKeyEvent<LibinputInputBackend>
    seat_name: &str,
) -> Result<(), InputError> {
    let keyboard_handle = seat.get_keyboard().cloned() // Cloned to satisfy lifetime requirements if methods needed owned handle
        .ok_or_else(|| InputError::KeyboardHandleNotFound(seat_name.to_string()))?;

    // We need mutable access to XkbKeyboardData for updating state and repeat_info.
    let xkb_data = desktop_state.keyboard_data_map.get_mut(seat_name)
        .ok_or_else(|| InputError::XkbConfigError {
            seat_name: seat_name.to_string(),
            message: "XkbKeyboardData not found for seat".to_string(),
        })?;

    let libinput_keycode = event.key_code();
    // The xkb_keycode is used for local XKB state processing if needed, and for repeat tracking.
    let xkb_keycode_for_tracking = xkb::Keycode::new(libinput_keycode + LIBINPUT_XKB_KEYCODE_OFFSET);
    let key_state = event.state();

    // Smithay's KeyboardHandle::input takes the raw libinput keycode.
    // It internally translates this to an XKB keycode using its configured keymap,
    // updates its internal XKB state, and sends appropriate wl_keyboard.key and wl_keyboard.modifiers events.
    let _filter_result = keyboard_handle.input(
        libinput_keycode,
        key_state,
        event.serial(),
        event.time(),
        Some(tracing::Span::current()),
    );

    // The KeyboardHandle has its own xkb::State. We are also keeping one in XkbKeyboardData.
    // This is for server-side decision making (like key repeat, compositor shortcuts).
    // The one in KeyboardHandle is for client communication.
    // We should update our server-side XKB state.
    let xkb_direction = match key_state {
        KeyState::Pressed => xkb::KeyDirection::Down,
        KeyState::Released => xkb::KeyDirection::Up,
    };
    xkb_data.state.update_key(xkb_keycode_for_tracking, xkb_direction);

    // Optional: Log keysym and UTF-8 for debugging server-side interpretation
    if key_state == KeyState::Pressed {
        let keysym = xkb_data.state.key_get_one_sym(xkb_keycode_for_tracking);
        let utf8_char_string = xkb_data.state.key_get_utf8(xkb_keycode_for_tracking);
        tracing::debug!(
            "Seat '{}': Key Press (libinput: {}, xkb: {}). Keysym: {:?} ({}), UTF-8: '{}'",
            seat_name, libinput_keycode, xkb_keycode_for_tracking.raw(),
            keysym, xkb::keysym_get_name(keysym), utf8_char_string
        );
    }

    // Key Repeat Logic
    if key_state == KeyState::Pressed {
        if let Some(timer) = xkb_data.repeat_timer.take() {
            timer.cancel();
        }
        xkb_data.repeat_info = None;

        // is_repeating checks against the keymap in KeyboardHandle.
        // It needs the XKB keycode (offset applied).
        if keyboard_handle.is_repeating(xkb_keycode_for_tracking) {
            let (delay, rate) = keyboard_handle.repeat_info();
            if rate.as_millis() > 0 {
                xkb_data.repeat_info = Some((
                    libinput_keycode, // Store original libinput keycode for re-sending
                    xkb_keycode_for_tracking, // Store XKB keycode for matching on release
                    keyboard_handle.modifiers_state(),
                    delay,
                    rate,
                ));
                xkb_data.repeat_key_serial = Some(event.serial());

                // Store the current focused surface for the repeat handler to check
                // The active_input_surface is on DesktopState, SeatHandler::focus_changed updates it.
                // XkbKeyboardData also has focused_surface_on_seat, updated by set_keyboard_focus.
                // We should ensure focused_surface_on_seat is up-to-date before starting repeat.
                xkb_data.focused_surface_on_seat = desktop_state.active_input_surface.clone();


                let seat_name_clone = seat_name.to_string();
                match desktop_state.loop_handle.insert_timer(
                    delay,
                    move |ds: &mut DesktopState| { // ds is &mut DesktopState
                        repeat_timer_handler(ds, &seat_name_clone);
                    }
                ) {
                    Ok(timer_handle) => {
                        xkb_data.repeat_timer = Some(timer_handle);
                        tracing::debug!("Key repeat started for keycode {} (XKB {}), seat '{}'. Delay: {:?}, Rate: {:?}", libinput_keycode, xkb_keycode_for_tracking.raw(), seat_name, delay, rate);
                    }
                    Err(e) => {
                        tracing::error!("Failed to insert key repeat timer for seat {}: {:?}", seat_name, e);
                        xkb_data.repeat_info = None; // Clear info if timer fails
                    }
                }
            }
        }
    } else { // KeyState::Released
        if let Some((_stored_lib_kc, stored_xkb_kc, ..)) = xkb_data.repeat_info {
            if stored_xkb_kc == xkb_keycode_for_tracking {
                if let Some(timer) = xkb_data.repeat_timer.take() {
                    timer.cancel();
                }
                xkb_data.repeat_info = None;
                xkb_data.repeat_key_serial = None;
                tracing::debug!("Key repeat stopped for keycode {} (XKB {}), seat '{}'.", libinput_keycode, xkb_keycode_for_tracking.raw(), seat_name);
            }
        }
    }
    Ok(())
}
