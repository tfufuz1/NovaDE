use crate::compositor::core::state::DesktopState;
// use crate::input::errors::InputError; // Not directly used in this file's functions
use crate::input::keyboard::xkb_config::XkbKeyboardData; // For type reference
use smithay::backend::input::{KeyState, KeyboardKeyEvent, LibinputInputBackend};
use smithay::input::{keyboard::{FilterResult, KeyboardHandle}, Seat};
use smithay::utils::{Serial, Clock}; // Clock for current time
use xkbcommon::xkb; // For xkb::Keycode type
use std::time::Duration; // For timer
use std::sync::{Arc, Mutex}; // For Arc<Mutex<XkbKeyboardData>>

const RAW_KEYCODE_OFFSET: u32 = 8; // libinput to XKB offset

pub fn handle_keyboard_key_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: KeyboardKeyEvent<LibinputInputBackend>,
    seat_name: &str,
) {
    let raw_keycode = event.key_code();
    let xkb_keycode: xkb::Keycode = (raw_keycode + RAW_KEYCODE_OFFSET).into(); // Convert to xkb::Keycode type
    let key_state = event.state();
    let serial = event.serial();
    let time = event.time();

    let keyboard_handle = match seat.get_keyboard() {
        Some(h) => h,
        None => {
            tracing::warn!("Kein Keyboard-Handle für Seat '{}' bei Key-Event.", seat_name);
            return;
        }
    };

    let xkb_data_arc_mutex = match desktop_state.keyboard_data_map.get(seat_name) {
        Some(data_mutex) => data_mutex.clone(), // Clone Arc<Mutex<XkbKeyboardData>>
        None => {
            tracing::error!("Keine XKB-Daten für Seat '{}' bei Key-Event.", seat_name);
            return;
        }
    };
    
    // This lock should be short-lived
    let mut xkb_data_guard = match xkb_data_arc_mutex.lock() {
        Ok(guard) => guard,
        Err(poison_err) => {
            tracing::error!("XkbKeyboardData Mutex vergiftet für Seat '{}': {}. Stoppe Verarbeitung.", seat_name, poison_err);
            // Depending on policy, might try to reinitialize XkbKeyboardData or clear the poisoned state.
            return; 
        }
    };

    let xkb_key_direction = match key_state {
        KeyState::Pressed => xkb::KeyDirection::Down,
        KeyState::Released => xkb::KeyDirection::Up,
    };
    xkb_data_guard.state.update_key(xkb_keycode, xkb_key_direction);

    let current_smithay_mods: smithay::input::keyboard::ModifiersState = (&xkb_data_guard.state).into(); // From<&xkb::State> for ModifiersState
    
    // Notify Smithay's KeyboardHandle about the modifier update.
    // The KeyboardHandle will then send wl_keyboard.modifiers to the client if changed.
    keyboard_handle.modifiers(serial, current_smithay_mods.clone(), Some(tracing::Span::current()));

    // Let Smithay's KeyboardHandle process the key event.
    // It will use its own XKB state (derived from the keymap provided when keyboard capability was added)
    // to translate the raw_keycode into keysyms and utf8 for the client.
    let filter_result = keyboard_handle.input(raw_keycode, key_state, serial, time, |filter_desktop_state, _filter_xkb_state, _filter_handle| {
        // This is the filter closure. It's called by Smithay *before* the event is sent to the client.
        // It allows us to intercept and potentially consume the event.
        // 'filter_xkb_state' here is &xkb::State from KeyboardHandle's internal view.
        // 'filter_desktop_state' is &mut DesktopState.
        // For now, we don't consume any keys, so return FilterResult::Forward.
        // This is where one might implement compositor-level keybindings.
        // Example: if check_compositor_binding(filter_desktop_state, filter_xkb_state, raw_keycode, key_state) { FilterResult::Consumed } else { FilterResult::Forward }
        
        // The prompt mentions updating XkbKeyboardData's modifiers if Smithay's internal state changed them.
        // This is tricky because our xkb_data_guard is locked here.
        // Smithay's KeyboardHandle has its own xkb::State. If it diverges from ours,
        // and we want ours to follow Smithay's, we'd need to update it.
        // However, our xkb_data_guard.state.update_key was just called, making it the most current.
        // The ModifiersState sent to keyboard_handle.modifiers() was derived from this.
        // So, they should be in sync unless the filter_closure itself modifies filter_xkb_state
        // in a way that should reflect back to our XkbKeyboardData. This is not typical.
        // For now, assume our XkbKeyboardData.state is the source of truth for modifiers we track.
        FilterResult::Forward
    });

    if filter_result == FilterResult::Consumed {
        tracing::debug!("Key event {:?} consumed by compositor filter.", raw_keycode);
        // If consumed, typically no key repeat.
        if let Some(timer) = xkb_data_guard.repeat_timer.take() {
            timer.cancel();
        }
        xkb_data_guard.repeat_info = None;
        xkb_data_guard.repeat_key_serial = None;
        return;
    }

    // Key repeat handling
    if key_state == KeyState::Pressed {
        // Cancel any existing timer (e.g., if another key was being repeated or this key was pressed again)
        if let Some(timer) = xkb_data_guard.repeat_timer.take() {
            timer.cancel();
        }
        xkb_data_guard.repeat_info = None; // Clear previous repeat info

        // Check if the pressed key should repeat
        // keyboard_handle.is_repeating(xkb_keycode) uses the XKB keycode.
        if keyboard_handle.is_repeating(xkb_keycode) {
            let (delay, rate) = keyboard_handle.repeat_info(); // Returns (Duration, Duration)
            
            if rate.as_millis() > 0 { // Ensure rate is valid for repetition
                xkb_data_guard.repeat_info = Some((
                    raw_keycode,
                    xkb_keycode, // Store for reference, not strictly needed for repeat logic using raw_keycode
                    current_smithay_mods, // Store the modifiers active at the time of the press
                    delay,
                    rate,
                ));
                xkb_data_guard.repeat_key_serial = Some(serial);

                let timer_seat_name = seat_name.to_string();
                // loop_handle is taken from desktop_state within the closure
                
                let timer_result = desktop_state.loop_handle.insert_timer(
                    delay, // Initial delay
                    // The closure receives &mut DesktopState as its argument directly from calloop
                    // This allows access to the most current DesktopState, including loop_handle for rescheduling.
                    move |ds: &mut DesktopState| { 
                        handle_keyboard_repeat(ds, &timer_seat_name);
                    },
                );

                match timer_result {
                    Ok(handle) => {
                        xkb_data_guard.repeat_timer = Some(handle);
                        tracing::debug!("Key repeat timer started for raw_keycode {}, delay {:?}, rate {:?}", raw_keycode, delay, rate);
                    }
                    Err(e) => {
                        tracing::error!("Fehler beim Erstellen des Key-Repeat-Timers: {}", e);
                        xkb_data_guard.repeat_info = None; // Clear info if timer failed
                    }
                }
            } else {
                tracing::trace!("Taste (XKB: {}) wiederholt nicht (Rate ist 0).", xkb_keycode);
            }
        } else {
             tracing::trace!("Taste (XKB: {}) wiederholt nicht (is_repeating ist false).", xkb_keycode);
        }
    } else { // KeyState::Released
        // If the released key is the one being repeated, cancel the timer
        if let Some((repeated_raw_kc, _, _, _, _)) = xkb_data_guard.repeat_info {
            if repeated_raw_kc == raw_keycode {
                if let Some(timer) = xkb_data_guard.repeat_timer.take() {
                    timer.cancel();
                    tracing::debug!("Key repeat timer cancelled for raw_keycode {}.", raw_keycode);
                }
                xkb_data_guard.repeat_info = None;
                xkb_data_guard.repeat_key_serial = None;
            }
        }
    }
}

// This function is called by the timer closure from handle_keyboard_key_event
fn handle_keyboard_repeat(desktop_state: &mut DesktopState, seat_name: &str) {
    let keyboard_handle = match desktop_state.seat_state.seats().find(|s| s.name() == seat_name).and_then(|s| s.get_keyboard()) {
        Some(h) => h,
        None => {
            tracing::warn!("Wiederholung: Keyboard-Handle für Seat '{}' nicht mehr verfügbar. Stoppe Wiederholung.", seat_name);
            if let Some(xkb_data_arc_mutex) = desktop_state.keyboard_data_map.get(seat_name) {
                if let Ok(mut xkb_data_guard) = xkb_data_arc_mutex.lock() {
                    if let Some(timer) = xkb_data_guard.repeat_timer.take() { timer.cancel(); }
                    xkb_data_guard.repeat_info = None;
                }
            }
            return;
        }
    };

    let xkb_data_arc_mutex = match desktop_state.keyboard_data_map.get(seat_name) {
        Some(data_mutex) => data_mutex.clone(),
        None => {
            tracing::error!("Wiederholung: Keine XKB-Daten für Seat '{}'. Stoppe Wiederholung.", seat_name);
            return;
        }
    };
    
    let mut xkb_data_guard = match xkb_data_arc_mutex.lock() {
        Ok(guard) => guard,
        Err(poison_err) => {
            tracing::error!("Wiederholung: XkbKeyboardData Mutex vergiftet für Seat '{}': {}. Stoppe Wiederholung.", seat_name, poison_err);
            return;
        }
    };

    if xkb_data_guard.repeat_info.is_none() {
        tracing::debug!("Wiederholung: Keine Wiederholungsinformationen für Seat '{}', Timer wird möglicherweise fälschlicherweise ausgelöst oder bereits abgebrochen.", seat_name);
        if let Some(timer) = xkb_data_guard.repeat_timer.take() { timer.cancel(); } // Ensure it's cancelled
        return;
    }

    // Check if focus is still valid for this keyboard
    // Compare the surface ID stored in XkbKeyboardData with the current focus of KeyboardHandle
    let current_kbd_focus_surface_ref = keyboard_handle.current_focus(); // Option<&WlSurface>
    let expected_focus_surface_weak_opt = xkb_data_guard.focused_surface_on_seat.clone(); // Option<Weak<WlSurface>>
    
    let focus_matches = match (current_kbd_focus_surface_ref, expected_focus_surface_weak_opt.as_ref().and_then(|wk| wk.upgrade())) {
        (Some(focused_surf), Some(expected_surf)) => focused_surf.id() == expected_surf.id(),
        (None, None) => true, // Both are None, focus is consistent (cleared)
        _ => false, // One is Some, the other is None, or different surfaces
    };

    if !focus_matches {
        tracing::debug!("Wiederholung: Keyboard-Fokus hat sich geändert oder ist verloren gegangen. Breche Wiederholung für Seat '{}' ab. Erwartet: {:?}, Aktuell: {:?}", 
            seat_name,
            expected_focus_surface_weak_opt.as_ref().and_then(|wk| wk.upgrade().map(|s| s.id())), 
            current_kbd_focus_surface_ref.map(|s| s.id())
        );
        if let Some(timer) = xkb_data_guard.repeat_timer.take() { timer.cancel(); }
        xkb_data_guard.repeat_info = None;
        xkb_data_guard.repeat_key_serial = None;
        return;
    }
    
    // Clone repeat_info for use.
    let (raw_keycode, _xkb_keycode, stored_mods, _delay, rate) = match xkb_data_guard.repeat_info.clone() {
        Some(info) => info,
        None => { // Should have been caught by the earlier check
            if let Some(timer) = xkb_data_guard.repeat_timer.take() { timer.cancel(); }
            return;
        }
    };
    
    let new_serial = Serial::now(); // Generate a new serial for the repeated event
    let time = desktop_state.clock.now().as_millis() as u32; // Current time for the event

    // Drop the guard before calling keyboard_handle.input if the filter closure might re-lock.
    // However, the filter is simple (FilterResult::Forward).
    // For this specific case, holding the lock might be fine.
    // Let's keep it locked to update repeat_key_serial and timer handle.

    // Send modifiers first, then the key
    keyboard_handle.modifiers(new_serial, stored_mods, Some(tracing::Span::current()));
    
    let _filter_result = keyboard_handle.input(raw_keycode, KeyState::Pressed, new_serial, time, |_ds, _filter_xkb_state, _filter_handle| {
        FilterResult::Forward // Repeated keys are generally not consumed by compositor bindings
    });
    
    xkb_data_guard.repeat_key_serial = Some(new_serial); // Update with the latest serial

    // Reschedule the timer
    let timer_seat_name_clone = seat_name.to_string();
    // desktop_state.loop_handle is available from the &mut DesktopState parameter
    let next_timer_result = desktop_state.loop_handle.insert_timer(
        rate, // Use the repeat rate for subsequent calls
        move |ds: &mut DesktopState| { // ds is &mut DesktopState
            handle_keyboard_repeat(ds, &timer_seat_name_clone);
        },
    );

    match next_timer_result {
        Ok(handle) => {
            xkb_data_guard.repeat_timer = Some(handle); // Store the new timer handle
            tracing::trace!("Key repeat timer rescheduled for raw_keycode {}, rate {:?}", raw_keycode, rate);
        }
        Err(e) => {
            tracing::error!("Fehler beim Neusetzen des Key-Repeat-Timers: {}", e);
            xkb_data_guard.repeat_timer = None; // Stop repeating if timer fails
            xkb_data_guard.repeat_info = None;
            xkb_data_guard.repeat_key_serial = None;
        }
    }
}
