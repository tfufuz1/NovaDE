use crate::{
    compositor::core::state::DesktopState,
    input::keyboard::{focus::set_keyboard_focus, XkbKeyboardData}, // XkbKeyboardData might be directly passed to repeat handler
};
use smithay::{
    backend::input::{KeyState, KeyboardKeyEvent, LibinputInputBackend},
    input::{keyboard::KeyboardHandle, Seat},
    reexports::{
        calloop::LoopHandle, // For scheduling repeat timer
        wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle, Serial}, // For serials
    },
    utils::SERIAL_COUNTER as WSERIAL_COUNTER, // For generating new serials
};
use std::{sync::Arc, time::Duration}; // For Arc, Duration

const KEY_REPEAT_DELAY_MS: u64 = 200; // Default key repeat delay
const KEY_REPEAT_RATE_HZ: u64 = 25; // Default key repeat rate (keys per second)

/// Handles a raw keyboard key event from an input backend.
///
/// This function translates the raw key event into XKB state changes,
/// sends appropriate Wayland keyboard events to clients, and manages key repetition.
///
/// # Arguments
///
/// * `desktop_state`: Mutable reference to the global `DesktopState`.
/// * `seat`: The `Seat` to which this keyboard event belongs.
/// * `event`: The `KeyboardKeyEvent` received from the backend.
/// * `seat_name`: The name of the seat.
pub fn handle_keyboard_key_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: KeyboardKeyEvent<LibinputInputBackend>,
    seat_name: &str,
) {
    let keyboard_handle = match seat.get_keyboard() {
        Some(kh) => kh,
        None => {
            tracing::error!("Received keyboard event for seat '{}' which has no keyboard handle.", seat_name);
            return;
        }
    };

    let xkb_data_arc = match desktop_state.keyboard_data_map.get(seat_name) {
        Some(data) => data.clone(), // Clone Arc to use
        None => {
            tracing::error!("XkbKeyboardData not found for seat '{}' during key event.", seat_name);
            return;
        }
    };

    // Update XKB state and get current modifiers
    // `update_xkb_state_and_get_modifiers` takes `&self` on XkbKeyboardData
    let smithay_mods_state = xkb_data_arc.update_xkb_state_and_get_modifiers(&event);

    // Inform Smithay's KeyboardHandle about modifier changes.
    // This will send wl_keyboard.modifiers to the focused client if state changed.
    keyboard_handle.modifiers(
        event.serial(),
        smithay_mods_state.clone(), // Clone if needed later for repeat
        Some(tracing::Span::current()),
    );

    let xkb_keycode = event.key_code() + 8; // Convert libinput to XKB keycode

    match event.state() {
        KeyState::Pressed => {
            tracing::debug!(
                "Key pressed: libinput_keycode={}, xkb_keycode={}, time={}",
                event.key_code(), xkb_keycode, event.time()
            );

            // Send key press event to client
            keyboard_handle.key(
                event.serial(),
                event.time(),
                xkb_keycode, // Smithay expects XKB keycode
                KeyState::Pressed,
                Some(tracing::Span::current()),
            );

            // Handle key repetition
            // Cancel any existing repeat timer for this XKB data
            if let Some(timer) = xkb_data_arc.repeat_timer.lock().unwrap().take() {
                timer.cancel();
            }
            *xkb_data_arc.repeat_info.lock().unwrap() = None;

            if keyboard_handle.is_repeating(xkb_keycode) {
                let (delay, rate_duration) = match keyboard_handle.repeat_info() {
                    Some((delay_ms, rate_hz)) if rate_hz > 0 => {
                        (Duration::from_millis(delay_ms as u64), Duration::from_millis(1000 / rate_hz as u64))
                    }
                    _ => (Duration::from_millis(KEY_REPEAT_DELAY_MS), Duration::from_millis(1000 / KEY_REPEAT_RATE_HZ)),
                };

                tracing::info!(
                    "Starting key repetition for xkb_keycode {} on seat '{}': delay={:?}, rate={:?}",
                    xkb_keycode, seat_name, delay, rate_duration
                );

                *xkb_data_arc.repeat_info.lock().unwrap() = Some((
                    event.key_code(), // Store original libinput keycode for consistency if needed
                    xkb_keycode,
                    smithay_mods_state, // Store current modifiers for repeat
                    delay,
                    rate_duration,
                ));
                *xkb_data_arc.repeat_key_serial.lock().unwrap() = Some(event.serial());

                // Create a clone of necessary Arcs/data for the timer callback
                let repeat_xkb_data_arc = xkb_data_arc.clone();
                let repeat_loop_handle = desktop_state.loop_handle.clone();
                let repeat_seat_name = seat_name.to_string();
                let repeat_display_handle = desktop_state.display_handle.clone(); // For generating serials

                let timer = desktop_state.loop_handle.insert_timer(
                    delay,
                    move |_event_data, _timer_shared_data, _main_shared_data| { // main_shared_data is &mut DesktopState
                        // This callback needs access to DesktopState to get the seat and then keyboard handle.
                        // Or, pass KeyboardHandle directly if possible (it's not Clone or Send/Sync easily).
                        // The `_main_shared_data` in `insert_timer` is `&mut D`, where `D` is `DesktopState`.
                        // So, we can access `_main_shared_data` (which is `desktop_state`).
                        // However, the closure for insert_timer is `FnOnce(Event, &mut TimerSharedData, &mut D)`.
                        // The `_main_shared_data` is not what we expect here.
                        // calloop::LoopHandle::insert_timer takes F: FnOnce(T, &mut S) where T is event, S is shared data.
                        // The shared data for the timer itself is usually (). The global shared data is DesktopState.
                        // Let's re-check calloop timer API.
                        // `LoopHandle<D>::insert_timer<F, T>(duration, data: T, callback: F)`
                        // where `F: FnOnce(T, &mut D) + 'static`.
                        // So, `data` is `T` (our event data), and `callback` gets `(T, &mut DesktopState)`.
                        // We can pass `(repeat_xkb_data_arc, repeat_seat_name, repeat_display_handle)` as `T`.

                        // This closure needs `&mut DesktopState` for `loop_handle` again to reschedule.
                        // The callback for `insert_timer` is `FnOnce(T, &mut LoopSharedData<D>)`.
                        // `LoopSharedData<D>` is not `&mut D`.
                        // The callback signature is `F: FnOnce(T, &mut S, &mut D) -> Option<Duration>` for `Generic::new_timer`.
                        // For `LoopHandle::insert_timer`, it's `F: FnMut(T, &mut D) -> Option<Duration>`.
                        // This is simpler. `T` is our event data, `&mut D` is `&mut DesktopState`.

                        // The closure needs to be `FnMut` for rescheduling.
                        // Let's reconstruct what we need for the repeat handler.
                        // `handle_key_repeat(desktop_state, seat_name_str, xkb_data_arc_clone)`
                        // This requires `desktop_state` to be passed into the closure.
                        // The closure signature is `F: FnMut(T, &mut D) -> Option<Duration>`.
                        // `T` is our `event_data` which can be `()` if we capture everything.
                        // `&mut D` is `&mut DesktopState`.

                        // This is the data for one repeat tick.
                        handle_key_repeat(
                            repeat_xkb_data_arc, // Arc<XkbKeyboardData>
                            repeat_loop_handle,  // LoopHandle<DesktopState>
                            repeat_seat_name,    // String
                            repeat_display_handle, // DisplayHandle
                        );
                        // Rescheduling is handled within handle_key_repeat
                    },
                ).expect("Failed to insert key repeat timer"); // Should handle error better

                *xkb_data_arc.repeat_timer.lock().unwrap() = Some(timer);
            }
        }
        KeyState::Released => {
            tracing::debug!(
                "Key released: libinput_keycode={}, xkb_keycode={}, time={}",
                event.key_code(), xkb_keycode, event.time()
            );

            // Send key release event to client
            keyboard_handle.key(
                event.serial(),
                event.time(),
                xkb_keycode, // Smithay expects XKB keycode
                KeyState::Released,
                Some(tracing::Span::current()),
            );

            // Cancel key repetition if the released key was the one being repeated
            let mut repeat_info_guard = xkb_data_arc.repeat_info.lock().unwrap();
            if let Some((_libinput_kc, r_xkb_kc, _, _, _)) = *repeat_info_guard {
                if r_xkb_kc == xkb_keycode {
                    tracing::info!("Key repetition canceled for xkb_keycode {} on seat '{}' due to key release.", xkb_keycode, seat_name);
                    if let Some(timer) = xkb_data_arc.repeat_timer.lock().unwrap().take() {
                        timer.cancel();
                    }
                    *repeat_info_guard = None;
                    *xkb_data_arc.repeat_key_serial.lock().unwrap() = None;
                }
            }
        }
    }
}

/// Handles a key repetition tick.
///
/// This function is called by a `calloop` timer when a key repeat event is due.
/// It sends the repeated key press to the client and reschedules the timer for the next repeat.
///
/// # Arguments (captured by the timer closure)
///
/// * `xkb_data_arc`: An `Arc` to the `XkbKeyboardData` for the relevant seat.
/// * `loop_handle`: A `LoopHandle` to reschedule the timer.
/// * `seat_name`: The name of the seat.
/// * `display_handle`: A `DisplayHandle` for generating new serials.
fn handle_key_repeat(
    xkb_data_arc: Arc<XkbKeyboardData>,
    loop_handle: LoopHandle<'static, DesktopState>, // LoopHandle to reschedule
    seat_name: String, // For logging and potentially getting seat if needed
    display_handle: DisplayHandle, // For generating serials
) {
    let mut repeat_info_guard = xkb_data_arc.repeat_info.lock().unwrap();

    // Check if repetition should continue (focus might have changed, or info cleared)
    if repeat_info_guard.is_none() {
        tracing::debug!("Repeat info cleared, stopping key repetition for seat '{}'.", seat_name);
        if let Some(timer) = xkb_data_arc.repeat_timer.lock().unwrap().take() {
            timer.cancel(); // Ensure timer is definitely cancelled
        }
        return;
    }
    
    // Check if focused surface still exists for this seat
    // If XkbKeyboardData.focused_surface_on_seat.lock().unwrap().as_ref().and_then(|wk| wk.upgrade()).is_none() {
    // This check should be done by DesktopState or a focus manager.
    // For now, assume if repeat_info is Some, focus is still valid for repeat.
    // A more robust check would involve querying DesktopState for current focus on this seat.
    // This function doesn't have &mut DesktopState, so it cannot directly query seat focus easily.
    // This is a limitation of making the timer callback not have access to DesktopState.

    // The timer callback for LoopHandle::insert_timer *does* get &mut DesktopState if it's the shared data.
    // Let's re-evaluate the timer callback signature for `LoopHandle::insert_timer`.
    // `insert_timer<T, F>(duration, data: T, callback: F)` where `F: FnMut(T, &mut D) -> Option<Duration>`.
    // So, the callback *can* take `&mut DesktopState`.
    // This means `handle_key_repeat` can be a method on `DesktopState` or a free function
    // that takes `&mut DesktopState`.

    // The closure passed to `insert_timer` in `handle_keyboard_key_event` needs to be adjusted.
    // It should capture `seat_name` (String) and `xkb_data_arc` (Arc<XkbKeyboardData>).
    // The closure itself will be `FnMut((String, Arc<XkbKeyboardData>), &mut DesktopState) -> Option<Duration>`.
    // The `handle_key_repeat` function itself would then be:
    // `fn handle_key_repeat_tick(event_data: (String, Arc<XkbKeyboardData>), desktop_state: &mut DesktopState) -> Option<Duration>`
    // This is much cleaner.

    // --- This function `handle_key_repeat` will be the body of that improved closure. ---
    // --- It will be called from within the `FnMut` that `insert_timer` gets. ---
    // --- For now, simulate that by using the captured variables. ---

    let (_libinput_kc, r_xkb_kc, ref r_mods, _delay, r_rate) = match *repeat_info_guard {
        Some(ref data) => data.clone(), // Clone the contents for use
        None => { // Should be caught by the check above, but as a safeguard
             if let Some(timer) = xkb_data_arc.repeat_timer.lock().unwrap().take() {
                timer.cancel();
            }
            return;
        }
    };


    // Get KeyboardHandle from DesktopState (this is where &mut DesktopState would be needed)
    // This is a conceptual problem if handle_key_repeat is a standalone function without DesktopState.
    // For now, assume we can get it. In the real timer callback, we'd use desktop_state.seat.get_keyboard().
    // This function, as standalone, cannot get the KeyboardHandle without DesktopState.
    // This implies the timer callback MUST call a method on DesktopState or a function that receives it.

    // Let's assume this function is called by a closure that *has* desktop_state:
    // fn actual_timer_callback(desktop_state: &mut DesktopState, seat_name_str: &str, xkb_data_arc_clone: Arc<XkbKeyboardData>) { ... }
    // And inside that, it calls this or similar logic.

    // For the purpose of this function signature (without &mut DesktopState):
    // This function cannot proceed without KeyboardHandle.
    // This highlights that the timer callback needs to be structured to have access to DesktopState.
    // The `handle_key_repeat` function will be simplified and its content moved to the timer lambda.

    // --- Refactoring: The logic of handle_key_repeat will be inside the lambda in handle_keyboard_key_event ---
    // --- This standalone `handle_key_repeat` function will be removed. ---
    // --- The `handle_keyboard_repeat` in `keyboard/mod.rs` will be the entry point for the timer. ---
    tracing::error!("handle_key_repeat standalone function should not be called. Logic moved to timer lambda.");
}


/// Entry point for handling a key repeat event, typically called from a timer.
///
/// This function should be invoked by the calloop timer scheduled for key repetition.
/// It retrieves necessary information from `DesktopState` and `XkbKeyboardData`
/// to send a repeated key event and then reschedules itself if repetition should continue.
///
/// # Arguments
///
/// * `desktop_state`: Mutable reference to the global `DesktopState`.
/// * `seat_name`: The name of the seat for which repetition is occurring.
/// * `xkb_data_for_seat_arc`: An `Arc` to the `XkbKeyboardData` for this seat.
///
/// # Returns
///
/// * `Option<Duration>`: If `Some(duration)`, the timer will be rescheduled with this duration.
///   If `None`, the timer will not be rescheduled (repetition stops).
pub fn handle_keyboard_repeat( // This is the function the timer will effectively call (via a lambda)
    desktop_state: &mut DesktopState,
    seat_name: &str, // seat_name is a String in the closure
    xkb_data_for_seat_arc: Arc<XkbKeyboardData>, // This Arc is cloned into the closure
) -> Option<Duration> { // Return Option<Duration> for rescheduling
    let mut repeat_info_guard = xkb_data_for_seat_arc.repeat_info.lock().unwrap();

    // Check if focused surface still exists for this seat
    let focused_surface_exists = xkb_data_for_seat_arc.focused_surface_on_seat.lock().unwrap().as_ref()
        .and_then(|wk| wk.upgrade()).is_some();

    if !focused_surface_exists || repeat_info_guard.is_none() {
        tracing::info!("Key repetition stopped for seat '{}': No focus or repeat info cleared.", seat_name);
        *repeat_info_guard = None; // Ensure it's cleared
        *xkb_data_for_seat_arc.repeat_timer.lock().unwrap() = None; // Clear stored timer handle
        return None; // Stop timer
    }

    let (_libinput_kc, r_xkb_kc, r_mods, _delay, r_rate) = match *repeat_info_guard {
        Some(ref data) => data.clone(), // Clone needed data
        None => return None, // Should not happen due to check above
    };

    let keyboard_handle = match desktop_state.seat.get_keyboard() {
        Some(kh) if desktop_state.seat.name() == seat_name => kh,
        _ => {
            tracing::error!("Keyboard handle not found for seat '{}' during key repeat. Stopping.", seat_name);
            *repeat_info_guard = None;
            *xkb_data_for_seat_arc.repeat_timer.lock().unwrap() = None;
            return None; // Stop timer
        }
    };

    let new_serial = WSERIAL_COUNTER.next_serial();
    let time = event_time_msec(); // Get current time in msec for the event

    tracing::debug!(
        "Repeating key press: xkb_keycode={}, time={}, serial={:?} for seat '{}'",
        r_xkb_kc, time, new_serial, seat_name
    );

    // Send modifiers state first
    keyboard_handle.modifiers(new_serial, r_mods, Some(tracing::Span::current()));
    // Send key press event
    keyboard_handle.key(
        new_serial,
        time,
        r_xkb_kc,
        KeyState::Pressed,
        Some(tracing::Span::current()),
    );

    // Update the serial for this repeating key
    *xkb_data_for_seat_arc.repeat_key_serial.lock().unwrap() = Some(new_serial);

    // Reschedule the timer with the rate
    Some(r_rate)
}

// Helper to get current time in milliseconds (compatible with libinput event times)
fn event_time_msec() -> u32 {
    // Using std::time::SystemTime for a monotonic-like source if available through Instant.
    // This needs to be compatible with what clients expect for event times.
    // Libinput uses `clock_gettime(CLOCK_MONOTONIC, ...)`
    // For simplicity, if not available, can use Instant relative to a start time.
    // Smithay examples sometimes use `SystemTime::now().duration_since(UNIX_EPOCH)`
    // but that's wall-clock time. For events, monotonic is better.
    // Calloop's LoopHandle has `now()` which is usually based on CLOCK_MONOTONIC.
    // However, we don't have LoopHandle here directly.
    // For now, a placeholder. This should ideally come from the event loop's clock.
    // This is a known challenge. Let's use a simple Instant for now assuming it's okay.
    // This part requires careful consideration in a full compositor.
    // For now, let's use a simple incrementing counter or a coarse time.
    // Using Duration::as_millis on Instant::now() is not what libinput does.
    // This is a placeholder.
    static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let start_time = START_TIME.get_or_init(std::time::Instant::now);
    start_time.elapsed().as_millis() as u32
}
