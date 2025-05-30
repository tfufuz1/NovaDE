//! Handler for the `wl_seat` global and input event processing.
//!
//! This module implements `SeatHandler` for `NovaCompositorState`, providing access
//! to Smithay's `SeatState`. It also implements `KeyboardHandle` and `PointerHandle`
//! to define how the compositor reacts to keyboard and pointer input events after
//! they have been processed by Smithay's core seat logic. This includes managing
//! keyboard focus, pointer focus, and implementing features like click-to-focus.

#![allow(unused_variables)]
#![allow(dead_code)]

use crate::compositor::state::NovaCompositorState;
use smithay::{
    backend::input::{AbsolutePositionEvent, Axis, AxisSource, ButtonState, KeyState, KeyboardKeyEvent,
                     PointerAxisEvent, PointerButtonEvent, PointerMotionEvent,BTN_LEFT},
    delegate_seat,
    desktop::Window,
    input::{
        keyboard::{KeyboardHandle, KeysymHandle, ModifiersState, XkbConfig, Error as XkbError, KEYMAP_FORMAT_TEXT_V1},
        pointer::{AxisFrame, ButtonEvent, MotionEvent, PointerHandle, PointerInnerHandle, RelativeMotionEvent},
        Seat, SeatHandler, SeatState,
    },
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Client},
    utils::{SERIAL_COUNTER, Point},
};
use xkbcommon::xkb; // For keymap

// The main `smithay::input::SeatState<NovaCompositorState>` is stored in `NovaCompositorState`.
// The `smithay::input::Seat` object, also stored in `NovaCompositorState` (as `Option<Seat<Self>>`),
// is the primary entry point for processing input via `Seat::process_input_event()`.
// The handler trait implementations below define callbacks that Smithay's seat logic
// invokes at various points during input processing.

impl SeatHandler for NovaCompositorState {
    /// Defines the type used for representing keyboard focus targets. Here, `WlSurface`.
    type KeyboardFocus = WlSurface;
    /// Defines the type used for representing pointer focus targets. Here, `WlSurface`.
    type PointerFocus = WlSurface;
    // TODO: type TouchFocus = WlSurface; // For touch input

    /// Provides access to Smithay's `SeatState`, which manages the set of
    /// capabilities (keyboard, pointer, touch) and active devices for the seat.
    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    // TODO: Implement `new_client_request` if clients should be able to create new seats.
    // TODO: Implement `new_seat_request` if dynamic seat creation is needed.
}

/// Handler for keyboard-related events and focus management.
impl KeyboardHandle for NovaCompositorState {
    /// Called when a key event occurs (press or release).
    ///
    /// Smithay's core seat logic (`Seat::process_input_event` which calls `Keyboard::input`)
    /// handles forwarding the key event to the currently focused client surface. This callback
    /// is invoked *after* that internal processing. It can be used for compositor-specific
    /// actions like global keyboard shortcuts or additional logging.
    ///
    /// # Arguments
    /// * `seat`: The `Seat` object that processed this event.
    /// * `event`: The raw `KeyboardKeyEvent`.
    /// * `modifiers`: Current effective keyboard modifiers.
    /// * `serial`: The serial number for this event.
    /// * `time`: Timestamp of the event.
    fn input(
        &mut self,
        seat: &Seat<Self>,
        event: KeyboardKeyEvent,
        modifiers: Option<ModifiersState>,
        serial: SERIAL_COUNTER,
        time: u32,
    ) {
        let keyboard = seat.get_keyboard().expect("Keyboard should exist on seat");

        slog::trace!(
            self.logger,
            "Keyboard input (post-process): keycode {:?}, state {:?}, serial {:?}, time {}, focused: {:?}",
            event.key_code(),
            event.state(),
            serial,
            time,
            keyboard.focused_surface().map(|s|s.id())
        );

        // TODO: Implement compositor-level keyboard shortcuts (e.g., Alt+Tab, workspace switching).
        // These would typically be checked before the event is forwarded to a client, or if no client has focus.
        // However, this handler is post-Smithay's internal client forwarding.
        // For pre-client shortcuts, `Seat::process_input_event` might need custom logic or a pre-filter.
    }

    /// Called when the keyboard modifier state changes.
    ///
    /// Smithay's `Keyboard` updates its internal state and sends the new modifier
    /// state to focused clients. This callback allows the compositor to react to
    /// these changes if needed.
    fn modifiers(&mut self, seat: &Seat<Self>, modifiers_state: ModifiersState, serial: SERIAL_COUNTER) {
        slog::debug!(
            self.logger,
            "Keyboard modifiers changed (post-process): new_state={:?}, serial={:?}",
            modifiers_state,
            serial
        );
    }

    /// Called when a client requests the keyboard map.
    ///
    /// Smithay's `Keyboard` handles providing the keymap (based on XKB configuration
    /// supplied when `Seat::add_keyboard` was called) to the client. This callback
    /// is informational.
    fn keymap(&mut self, seat: &Seat<Self>, format: u32, fd: std::os::unix::io::RawFd, size: usize) {
        slog::debug!(self.logger, "Keymap requested by client: format {:?}, fd {:?}, size {:?}", format, fd, size);
    }

    /// Called when the keyboard focus changes.
    ///
    /// This callback is triggered *after* Smithay's `Keyboard::set_focus()` successfully
    /// changes the focus (e.g., due to a click-to-focus action or an explicit compositor decision).
    /// It updates the compositor's internal focus tracking (`NovaCompositorState::set_keyboard_focus`)
    /// which can also handle visual cues like activating/deactivating window decorations.
    fn focus(
        &mut self,
        seat: &Seat<Self>,
        focused_surface: Option<&WlSurface>,
        serial: SERIAL_COUNTER,
    ) {
        slog::debug!(
            self.logger,
            "Keyboard focus changed (by Smithay): new_focus={:?}, serial={:?}",
            focused_surface.map(|s| s.id()),
            serial
        );
        // Update our compositor's explicit focus tracking and window activation state.
        self.set_keyboard_focus(focused_surface.cloned());
    }
}

/// Handler for pointer-related events, focus, and actions like click-to-focus.
impl PointerHandle for NovaCompositorState {
    /// Called when the pointer moves.
    ///
    /// Smithay's core seat logic (`Seat::process_input_event` which calls `Pointer::motion`)
    /// updates the pointer's internal focus based on `event.focus` (determined by checking
    /// what's under the cursor in the `Space`) and sends `wl_pointer.motion`, `wl_pointer.enter`,
    /// and `wl_pointer.leave` events to the appropriate client surfaces.
    /// This callback is invoked *after* that internal processing.
    ///
    /// # Arguments
    /// * `seat`: The `Seat` object that processed this event.
    /// * `event`: Contains the new pointer location (`event.location`) and the focus
    ///   determined by Smithay (`event.focus`).
    /// * `serial`: The serial number for this event.
    /// * `time`: Timestamp of the event.
    fn motion(
        &mut self,
        seat: &Seat<Self>,
        event: MotionEvent<Self::PointerFocus>,
        serial: SERIAL_COUNTER,
        time: u32,
    ) {
        slog::trace!(
            self.logger,
            "Pointer motion (post-process): new_location {:?}, new_focus_target {:?}, serial {:?}, time {}",
            event.location, // Absolute position in compositor space
            event.focus.as_ref().map(|(s, p)| (s.id(), p.clone())), // Surface and surface-local coords
            serial,
            time
        );

        // TODO: Update cursor icon based on the surface or compositor state.
        // Example:
        // let pointer = seat.get_pointer().expect("Pointer should exist on seat");
        // if let Some((surface, _)) = event.focus {
        //     // client_of_surface can be found using surface.client()
        //     // pointer.set_cursor(client_of_surface, Some(cursor_name_or_buffer));
        // } else {
        //     // Set a default cursor
        // }

        // TODO: Handle drag-and-drop logic initiation if a button is held during motion.
    }

    /// Called when a pointer button is pressed or released.
    ///
    /// Smithay's `Pointer::button` (called by `Seat::process_input_event`) sends the
    /// button event to the client surface that currently has pointer focus. This callback
    /// is invoked *after* that.
    ///
    /// This implementation includes click-to-focus logic:
    /// - On a left button press, it identifies the window under the pointer.
    /// - If a window is found, it sets keyboard focus to that window's surface
    ///   using `Keyboard::set_focus()` and raises the window to the top of the stack
    ///   using `Space::raise_element()`.
    fn button(&mut self, seat: &Seat<Self>, event: ButtonEvent, serial: SERIAL_COUNTER, time: u32) {
        let pointer = seat.get_pointer().expect("Pointer should exist on seat");
        let keyboard = seat.get_keyboard().expect("Keyboard should exist on seat");

        slog::debug!(
            self.logger,
            "Pointer button (post-process): button_code={:?}, state={:?}, serial={:?}, time={}",
            event.button, event.state, serial, time
        );

        // Implement click-to-focus for left button presses
        if event.state == ButtonState::Pressed && event.button == BTN_LEFT {
            // `pointer.current_focus()` gives the WlSurface that Smithay determined has pointer focus.
            // This is the surface that received the button press from Smithay's internal logic.
            if let Some((focused_surface, _surface_local_coords)) = pointer.current_focus() {
                // Find the `Window` in our `Space` that corresponds to this `WlSurface`.
                let window_clicked = self.space.elements()
                    .find(|win| win.wl_surface().as_ref() == Some(focused_surface))
                    .cloned(); // Clone to avoid borrowing `self.space` issues with `keyboard.set_focus`

                if let Some(window) = window_clicked {
                    slog::info!(self.logger, "Click-to-focus: Focusing window with surface {:?}", focused_surface.id());

                    // Set keyboard focus to this window's surface.
                    // This will trigger `KeyboardHandle::focus` where `self.set_keyboard_focus` is called.
                    keyboard.set_focus(Some(focused_surface), serial);

                    // Bring the clicked window to the top of the stacking order.
                    self.space.raise_element(&window, true);
                    // TODO: Trigger a redraw if necessary due to stacking change or focus decoration.
                } else {
                    slog::debug!(self.logger, "Click-to-focus: Clicked on a surface not managed as a toplevel window in space (e.g., a popup or layer). No focus change.");
                    // Optionally, if clicking on a surface not in `self.space` (e.g. a background or layer surface),
                    // you might want to clear keyboard focus from any previously focused toplevel.
                    // keyboard.set_focus(None, serial);
                }
            } else {
                // Clicked on empty space (no surface has pointer focus)
                slog::debug!(self.logger, "Click-to-focus: Clicked on empty space. Clearing keyboard focus.");
                keyboard.set_focus(None, serial);
            }
        }
    }

    /// Called for pointer axis events (e.g., mouse wheel scrolling).
    ///
    /// Smithay's `Pointer::axis` (called by `Seat::process_input_event`) sends the
    /// axis event to the client surface with current pointer focus. This callback is
    /// invoked *after* that.
    fn axis(&mut self, seat: &Seat<Self>, details: AxisFrame, serial: SERIAL_COUNTER, time: u32) {
        slog::trace!(
            self.logger,
            "Pointer axis (post-process): details={:?}, serial={:?}, time={}",
            details, serial, time
        );
        // TODO: Implement any compositor-specific scroll handling if needed (e.g., workspace scrolling).
    }

    // TODO: Implement other PointerHandle methods like `frame`, `gesture_swipe_begin`, etc., if needed.
}

// Delegate wl_seat requests to NovaCompositorState.
// This macro implements `GlobalDispatch<WlSeat, SeatData<D>>` and the necessary
// `Dispatch` trait for `NovaCompositorState`. It also ensures `NovaCompositorState`
// implements `SeatData` which provides the focus target types.
delegate_seat!(NovaCompositorState);
```
