// novade-system/src/input/input_dispatcher.rs

use smithay::{
    backend::input::{InputEvent, Axis, KeyState, AxisSource as BackendAxisSource},
    input::{
        pointer::{PointerHandle, AxisFrame, AxisSource, ButtonEvent, MotionEvent},
        keyboard::{KeyboardHandle, FilterResult, KeysymHandle}, // Added KeysymHandle
        Seat,
    },
    utils::{SERIAL_COUNTER, Serial, Logical, Point}, // SERIAL_COUNTER for event serials
};
use crate::compositor::state::DesktopState;
// use crate::input::keyboard_layout::KeyboardLayoutManager; // Will be needed later

pub struct InputDispatcher;

impl InputDispatcher {
    pub fn new() -> Self {
        Self
    }

    /// Processes an input event and dispatches it to the appropriate handlers
    /// within the `DesktopState`.
    ///
    /// This function takes the event from the input backend (e.g., libinput)
    /// and uses the compositor's seat and its capabilities (keyboard, pointer, touch)
    /// to forward the event to the focused client or update compositor state.
    pub fn dispatch_event(&self, desktop_state: &mut DesktopState, event: InputEvent<LibinputInputEvent>) {
        match event {
            InputEvent::Keyboard { event, .. } => {
                // TODO: Integrate KeyboardLayoutManager for proper key mapping
                // For now, directly forward keycode and state
                if let Some(keyboard) = desktop_state.seat.get_keyboard() {
                    let serial = SERIAL_COUNTER.next_serial(); // Use global serial counter
                    let time = event.time_msec();

                    // This is a simplified key processing.
                    // Proper handling involves xkbcommon for layout/sym translation.
                    // The FilterResult is currently unused, but could be used for more complex logic.
                    keyboard.input(
                        desktop_state, // The &mut D for the handler
                        event.key_code(),
                        event.state(),
                        serial,
                        time,
                        |state, modifiers, handle: KeysymHandle<'_>| { // Adapted closure signature
                            // This closure is called by Smithay after xkbcommon processing (if configured).
                            // `handle` gives you the resolved keysym.
                            // `modifiers` gives the current modifier state.
                            // `state` is `&mut DesktopState`.
                            // We need to ensure xkbcommon is configured on the keyboard for this to be effective.
                            tracing::debug!(
                                "Keyboard event: keycode {}, state {:?}, keysym {:?}, modifiers {:?}",
                                event.key_code(), event.state(), handle.modified_sym(), modifiers
                            );
                            // For now, just allow all keys through.
                            // Later, this could be used for compositor keybindings.
                            FilterResult::Forward
                        }
                    );
                }
            }
            InputEvent::PointerMotion { event, .. } => {
                if let Some(pointer) = desktop_state.seat.get_pointer() {
                    let time = event.time_msec();
                    // Smithay pointer handles motion relative to the current pointer position.
                    // The delta is provided by the event.
                    // DesktopState's pointer_location should be updated.
                    let new_pointer_location = pointer.current_position() + event.delta();
                    desktop_state.pointer_location = new_pointer_location;

                    pointer.motion(
                        desktop_state,           // &mut D
                        // Some(window_under_pointer), // Optional: The surface under the pointer if known
                        // &desktop_state.pointer_location, // New absolute position
                        time,                    // Event time
                    );
                }
            }
            InputEvent::PointerMotionAbsolute { event, .. } => {
                if let Some(pointer) = desktop_state.seat.get_pointer() {
                    let time = event.time_msec();
                    // Update DesktopState's pointer_location with the absolute position.
                    // The event position needs to be mapped to logical coordinates if it's not already.
                    // Assuming event.position_transformed() gives logical coordinates.
                    let new_pointer_location = event.position_transformed(desktop_state.pointer_location.to_vector()); // Needs output size if not logical
                    desktop_state.pointer_location = new_pointer_location;

                    pointer.motion(
                        desktop_state,           // &mut D
                        time,                    // Event time
                    );
                }
            }
            InputEvent::PointerButton { event, .. } => {
                if let Some(pointer) = desktop_state.seat.get_pointer() {
                    let serial = SERIAL_COUNTER.next_serial();
                    let time = event.time_msec();
                    pointer.button(
                        desktop_state, // &mut D
                        event.button_code(),
                        event.state(),
                        serial,
                        time,
                    );
                }
            }
            InputEvent::PointerAxis { event, .. } => {
                if let Some(pointer) = desktop_state.seat.get_pointer() {
                    let time = event.time_msec();
                    let source = match event.source() {
                        BackendAxisSource::Wheel => AxisSource::Wheel,
                        BackendAxisSource::Finger => AxisSource::Finger,
                        BackendAxisSource::Continuous => AxisSource::Continuous,
                        BackendAxisSource::WheelTilt => AxisSource::WheelTilt, // Make sure this variant exists or map appropriately
                    };

                    let mut axis_frame = AxisFrame::new(time)
                        .source(source);

                    if let Some(dx) = event.amount_discrete(Axis::Horizontal) {
                        axis_frame = axis_frame.discrete(Axis::Horizontal, dx as i32);
                    }
                    if let Some(dy) = event.amount_discrete(Axis::Vertical) {
                        axis_frame = axis_frame.discrete(Axis::Vertical, dy as i32);
                    }
                    // For continuous scrolling (e.g., touchpad precision scrolling)
                    if let Some(cx) = event.amount(Axis::Horizontal) {
                         axis_frame = axis_frame.value(Axis::Horizontal, cx);
                    }
                    if let Some(cy) = event.amount(Axis::Vertical) {
                        axis_frame = axis_frame.value(Axis::Vertical, cy);
                    }

                    pointer.axis(desktop_state, axis_frame);
                }
            }
            InputEvent::TouchDown { event, .. } => {
                if let Some(touch) = desktop_state.seat.get_touch() {
                    let serial = SERIAL_COUNTER.next_serial();
                    let time = event.time_msec();
                    // The position needs to be mapped to the focused window's coordinate space
                    // or global space depending on how touch events are handled.
                    // Smithay's touch.down takes position relative to the focus.
                    // For now, using the event's position directly, assuming it's in logical space.
                    let position = event.position_transformed(desktop_state.outputs.first().map_or((0,0), |o| o.current_mode().unwrap().size).into());

                    touch.down(
                        desktop_state, // &mut D
                        serial,
                        time,
                        event.slot().map(|s| s.id()).unwrap_or(0), // Get SlotId if available
                        position, // Position of the touch point
                    );
                }
            }
            InputEvent::TouchUp { event, .. } => {
                if let Some(touch) = desktop_state.seat.get_touch() {
                    let serial = SERIAL_COUNTER.next_serial();
                    let time = event.time_msec();
                    touch.up(
                        desktop_state, // &mut D
                        serial,
                        time,
                        event.slot().map(|s| s.id()).unwrap_or(0),
                    );
                }
            }
            InputEvent::TouchMotion { event, .. } => {
                if let Some(touch) = desktop_state.seat.get_touch() {
                    let time = event.time_msec();
                    let position = event.position_transformed(desktop_state.outputs.first().map_or((0,0), |o| o.current_mode().unwrap().size).into());

                    touch.motion(
                        desktop_state, // &mut D
                        time,
                        event.slot().map(|s| s.id()).unwrap_or(0),
                        position,
                    );
                }
            }
            InputEvent::TouchFrame { .. } => {
                if let Some(touch) = desktop_state.seat.get_touch() {
                    touch.frame(desktop_state);
                }
            }
            InputEvent::TouchCancel { .. } => {
                if let Some(touch) = desktop_state.seat.get_touch() {
                    touch.cancel(desktop_state);
                }
            }
            // Other events like Gesture, TabletTool, etc., can be added here.
            _ => {
                tracing::trace!("Unhandled input event: {:?}", event);
            }
        }
    }
}

// Define LibinputInputEvent if it's not a public type from smithay::backend::input
// Usually, InputEvent is generic over E which is the raw event type from backend.
// For LibinputInputBackend, E is smithay::backend::input::libinput::LibinputInputEvent.
use smithay::backend::input::libinput::LibinputInputEvent;


impl Default for InputDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
