// This is novade-system/src/compositor/input.rs
// Integration with input backends and mapping Smithay input events.

use smithay::{
    backend::input::{
        self as backend_input, // Alias to avoid confusion with smithay::input
        Axis, AxisSource as BackendAxisSource, Event as BackendInputEvent, InputEvent,
        KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent, PointerMotionAbsoluteEvent,
        TouchDownEvent, TouchMotionEvent, TouchUpEvent, TabletToolAxisEvent, TabletToolButtonEvent,
        TabletToolProximityEvent, TabletToolTipEvent,
    },
    desktop::{Space, Window, WindowSurfaceType},
    input::{
        Seat, SeatHandler, SeatState as SmithaySeatState, SeatData, // Smithay's Seat structures
        pointer::{
            AxisFrame, CursorImageStatus, GrabStartData as PointerGrabStartData, PointerHandle,
            PointerInnerHandle, RelativePointerHandle, LockedPointerHandle, MotionEvent as SmithayPointerMotion,
            ButtonEvent as SmithayPointerButton, AxisEvent as SmithayPointerAxis,
        },
        keyboard::{
            KeyboardHandle, Keysym, ModifiersState, XkbConfig, LedState, FilterResult as XkbFilterResult,
            Error as XkbError, COMPOSITOR_MODIFIERS, MODIFIER_CAPS_LOCK, MODIFIER_NUM_LOCK, // For LED state
        },
        touch::{TouchHandle, TouchDownEvent as SmithayTouchDown, TouchUpEvent as SmithayTouchUp, TouchMotionEvent as SmithayTouchMotion, TouchShape, TouchSlot}, // Smithay 0.30 specific touch events
        SeatFocus, GrabStartData,
    },
    reexports::{
        calloop::LoopHandle,
        wayland_server::{
            protocol::{wl_surface::WlSurface, wl_seat::WlSeat},
            DisplayHandle, Client,
        },
        input::LibinputInputBackend, // Example backend
    },
    utils::{Clock, Logical, Point, Serial, SERIAL_COUNTER, Transform, Physical, Size},
    wayland::{
        relative_pointer::RelativePointerManagerState,
        pointer_constraints::PointerConstraintsState,
        input_method::InputMethodManagerState,
        text_input::TextInputManagerState,
    },
};
use tracing::{info, warn, debug, error};
use xkbcommon::xkb; // For keysym definitions

use crate::compositor::state::{DesktopState, NovaSeatState}; // Assuming NovaSeatState wraps SmithaySeatState
use crate::compositor::errors::CompositorError;


// --- Input Event Processing ---

/// Processes a generic `InputEvent` from a backend and dispatches it to the appropriate Smithay `Seat` handler.
pub fn process_input_event<B: backend_input::InputBackend>(
    state: &mut DesktopState,
    event: BackendInputEvent<B>,
    // output_name: &str, // Name of the output where the event originated, for coordinate transformation
) {
    let serial = SERIAL_COUNTER.next_serial();
    let time = state.clock.now().as_millis() as u32; // Use DesktopState's clock

    let seat = &state.primary_seat; // Get the primary seat from DesktopState

    match event {
        BackendInputEvent::Keyboard { event, .. } => {
            if let Some(keyboard) = seat.get_keyboard() {
                keyboard.input(
                    state, // &mut DesktopState which implements SeatHandler
                    event.key_code(),
                    event.state(),
                    serial,
                    time,
                    |d_state, modifiers, handle| {
                        // This is the key filter callback.
                        // `handle` is a KeyboardHandle. `modifiers` is the current ModifiersState.
                        // Check for compositor keybindings first.
                        if d_state.handle_compositor_keybinding(modifiers, handle.modified_sym()) {
                            return XkbFilterResult::HandledByCompositor;
                        }
                        // If not handled, let it pass to the client.
                        XkbFilterResult::ForwardToClient
                    },
                );
                // Update LED state if needed (e.g. Caps Lock, Num Lock)
                // This might be better handled inside DesktopState::handle_compositor_keybinding or a post-input hook.
                // let kbd_led_state = keyboard.led_state();
                // update_leds_on_physical_keyboard(kbd_led_state);
            }
        }
        BackendInputEvent::PointerMotion { event, .. } => {
            if let Some(pointer) = seat.get_pointer() {
                // Relative motion: update pointer_location based on delta
                let delta = event.delta(); // This is often in screen coordinates or device coordinates
                // We need to ensure pointer_location is updated correctly.
                // If delta is already logical:
                state.pointer_location += delta.to_f64();
                // If delta is physical and needs scaling:
                // let output_under_pointer = state.space.lock().unwrap().output_under(state.pointer_location).cloned();
                // let scale = output_under_pointer.map_or(Scale::from(1.0), |o| o.current_scale());
                // state.pointer_location += delta.to_f64().to_logical(scale);

                pointer.motion(state, state.pointer_location, serial, time);
            }
        }
        BackendInputEvent::PointerMotionAbsolute { event, .. } => {
            if let Some(pointer) = seat.get_pointer() {
                // Absolute motion: event provides new absolute position.
                // This position might be specific to an output or screen.
                // It needs to be transformed to the global logical coordinate space.
                let space_lock = state.space.lock().unwrap();
                let output_name_from_event = event.output_name().unwrap_or_default(); // Backend might provide output name

                let new_logical_pos = space_lock.outputs()
                    .find(|o| o.name() == output_name_from_event)
                    .map(|o| {
                        let output_geo = space_lock.output_geometry(o).unwrap();
                        let output_transform = o.current_transform();
                        // position_transformed takes physical size of output, not event size
                        event.position_transformed(output_geo.size, output_transform) + output_geo.loc.to_f64()
                    })
                    .unwrap_or_else(|| {
                        // Fallback if output not found or event is not tied to a specific output
                        // This might mean treating it as a global coordinate if backend guarantees it
                        // or a position relative to the primary screen.
                        // For now, assume it's a position that can be directly used or needs simple mapping.
                        warn!("Absolute pointer motion on unknown output or unmapped. Using event position directly.");
                        event.position() // This is usually physical, needs careful handling
                    });

                state.pointer_location = new_logical_pos;
                pointer.motion(state, state.pointer_location, serial, time);
            }
        }
        BackendInputEvent::PointerButton { event, .. } => {
            if let Some(pointer) = seat.get_pointer() {
                pointer.button(state, event.button_code(), event.state(), serial, time);
            }
        }
        BackendInputEvent::PointerAxis { event, .. } => {
            if let Some(pointer) = seat.get_pointer() {
                let source = map_backend_axis_source(event.source());
                let mut frame = AxisFrame::new(time).source(source);
                if event.axis() == Axis::Horizontal {
                    if let Some(discrete) = event.amount_discrete() {
                        frame = frame.discrete(smithay::input::pointer::Axis::Horizontal, (discrete * 120.0) as i32);
                    }
                    frame = frame.value(smithay::input::pointer::Axis::Horizontal, event.amount_continuous() * 10.0); // Example scaling
                } else if event.axis() == Axis::Vertical {
                     if let Some(discrete) = event.amount_discrete() {
                        frame = frame.discrete(smithay::input::pointer::Axis::Vertical, (discrete * 120.0) as i32);
                    }
                    frame = frame.value(smithay::input::pointer::Axis::Vertical, event.amount_continuous() * 10.0);
                }
                pointer.axis(state, frame);
            }
        }
        BackendInputEvent::TouchDown { event, .. } => {
            if let Some(touch) = seat.get_touch() {
                // Similar to PointerMotionAbsolute, transform coordinates if needed.
                // let logical_pos = event.position_transformed(...);
                // state.pointer_location = logical_pos; // Update pointer focus for new touch
                // touch.down(state, serial, time, event.slot(), logical_pos);
                 warn!("TouchDown event processing needs coordinate transformation and focus update logic.");
                 // Placeholder: use raw position, assuming it's logical for now
                 let pos = event.position();
                 state.pointer_location = pos; // Update pointer location for focus purposes
                 touch.down(state, serial, time, event.slot_id(), pos, Some(state.primary_seat.clone())).unwrap_or_else(|e| warn!("Touch down failed: {}",e));
            }
        }
        BackendInputEvent::TouchUp { event, .. } => {
            if let Some(touch) = seat.get_touch() {
                touch.up(state, serial, time, event.slot_id()).unwrap_or_else(|e| warn!("Touch up failed: {}",e));
            }
        }
        BackendInputEvent::TouchMotion { event, .. } => {
            if let Some(touch) = seat.get_touch() {
                // let logical_pos = event.position_transformed(...);
                // state.pointer_location = logical_pos;
                // touch.motion(state, serial, time, event.slot(), logical_pos);
                warn!("TouchMotion event processing needs coordinate transformation.");
                let pos = event.position();
                state.pointer_location = pos;
                touch.motion(state, serial, time, event.slot_id(), pos).unwrap_or_else(|e| warn!("Touch motion failed: {}",e));
            }
        }
        BackendInputEvent::TouchFrame { .. } => {
            if let Some(touch) = seat.get_touch() {
                touch.frame(state).unwrap_or_else(|e| warn!("Touch frame failed: {}",e));
            }
        }
        BackendInputEvent::TouchCancel { .. } => {
            if let Some(touch) = seat.get_touch() {
                touch.cancel(state).unwrap_or_else(|e| warn!("Touch cancel failed: {}",e));
            }
        }
        BackendInputEvent::DeviceAdded { device } => {
            info!("Input device added: {} (Backend notified)", device.name());
            // Backend (e.g. UdevBackend) usually handles adding device to Seat.
            // If manual association is needed: state.primary_seat.add_device(&device);
        }
        BackendInputEvent::DeviceRemoved { device } => {
            info!("Input device removed: {} (Backend notified)", device.name());
            // Backend usually handles removing device from Seat.
            // If manual: state.primary_seat.remove_device(&device);
        }
        // TODO: Handle TabletTool events if tablet support is desired.
        _ => {
            // debug!("Unhandled backend input event: {:?}", event);
        }
    }
}

impl DesktopState {
    /// Handles compositor-level keybindings.
    /// Returns `true` if the keybinding was handled, `false` otherwise.
    fn handle_compositor_keybinding(&mut self, modifiers: ModifiersState, keysym: Keysym) -> bool {
        // Example: Alt + F4 to close focused window
        if modifiers.alt && keysym == xkb::KEY_F4 {
            info!("Alt+F4 pressed. Attempting to close focused window.");
            // Find focused window (XDG Toplevel or XWayland) and send close request.
            // This requires access to the focused window, e.g., via seat.keyboard_focus_element().
            if let Some(keyboard) = self.primary_seat.get_keyboard() {
                if let Some(focused_surface) = keyboard.focused_surface() {
                     if let Some(window) = self.space.lock().unwrap().window_for_surface(&focused_surface).cloned() {
                        match window.toplevel() {
                            Some(WindowSurfaceType::Xdg(xdg_toplevel)) => {
                                xdg_toplevel.send_close();
                                return true;
                            }
                            Some(WindowSurfaceType::X11(x11_surface)) => {
                                x11_surface.close().ok(); // Attempt to close X11 window
                                return true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        // Add more keybindings here
        // Example: Ctrl + Alt + T to launch terminal
        // if modifiers.ctrl && modifiers.alt && keysym == xkb::KEY_T {
        //     info!("Ctrl+Alt+T pressed. Launching terminal (TODO).");
        //     // Command::new("foot").spawn().ok(); // Example, needs error handling
        //     return true;
        // }
        false // Not handled by compositor
    }
}


// Helper to map backend AxisSource to Smithay's AxisSource
fn map_backend_axis_source(backend_source: BackendAxisSource) -> smithay::input::pointer::AxisSource {
    match backend_source {
        BackendAxisSource::Wheel => smithay::input::pointer::AxisSource::Wheel,
        BackendAxisSource::Finger => smithay::input::pointer::AxisSource::Finger,
        BackendAxisSource::Continuous => smithay::input::pointer::AxisSource::Continuous,
        BackendAxisSource::WheelTilt => smithay::input::pointer::AxisSource::WheelTilt,
    }
}
