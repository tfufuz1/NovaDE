use crate::compositor::core::state::DesktopState;
use smithay::{
    backend::input::{
        Axis, ButtonState, Device, DeviceCapability, GestureHoldBeginEvent, GestureHoldEndEvent,
        GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent,
        GestureSwipeEndEvent, GestureSwipeUpdateEvent, InputEvent, KeyboardKeyEvent,
        LibinputInputBackend, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent,
        PointerMotionAbsoluteEvent, TouchCancelEvent, TouchDownEvent, TouchFrameEvent,
        TouchMotionEvent, TouchUpEvent, // Added missing gesture events for completeness
    },
    input::{
        keyboard::KeyboardConfig, pointer::PointerHandle, touch::TouchHandle, Seat, // Added Seat
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};
use crate::input::{
    keyboard::{handle_keyboard_key_event, XkbKeyboardData},
    pointer::{
        handle_pointer_axis_event, handle_pointer_button_event,
        handle_pointer_motion_absolute_event, handle_pointer_motion_event,
    },
    touch::{
        handle_touch_cancel_event, handle_touch_down_event_corrected as handle_touch_down_event, // Use corrected version
        handle_touch_frame_event, handle_touch_motion_event, handle_touch_up_event,
    },
};
use std::sync::Arc;

/// Processes an input event from the libinput backend.
///
/// This function dispatches various types of input events to the appropriate handlers
/// or logs them if no specific handler is yet implemented. It manages device
/// capabilities (keyboard, pointer, touch) for a given seat.
///
/// # Arguments
///
/// * `desktop_state`: A mutable reference to the global `DesktopState`.
/// * `event`: The `InputEvent` received from the `LibinputInputBackend`.
/// * `seat_name`: The name of the seat to which this event pertains.
pub fn process_input_event(
    desktop_state: &mut DesktopState,
    event: InputEvent<LibinputInputBackend>,
    seat_name: &str,
) {
    // Retrieve the Seat<DesktopState> based on seat_name.
    // Assuming the seat_name passed matches the name of desktop_state.seat for now.
    // If multiple seats were supported, we'd look up the seat in SeatState by name.
    if desktop_state.seat.name() != seat_name {
        tracing::error!(
            "process_input_event called for seat_name '{}', but DesktopState's primary seat is named '{}'. Event dropped.",
            seat_name,
            desktop_state.seat.name()
        );
        return;
    }
    let seat = desktop_state.seat.clone(); // Clone the Arc or reference to the seat

    match event {
        InputEvent::DeviceAdded { device } => {
            tracing::info!(device_name = %device.name(), "Input device added");

            // Keyboard capability
            if device.has_capability(DeviceCapability::Keyboard) {
                if seat.get_keyboard().is_none() {
                    tracing::info!("Device {:?} has Keyboard capability. Adding keyboard to seat '{}'.", device.name(), seat_name);
                    // TODO: Load KeyboardConfig from settings if available.
                    let kb_config = KeyboardConfig::default(); // Use default for now
                    match seat.add_keyboard(kb_config.clone(), 200, 25) { // 200ms delay, 25ms rate
                        Ok(keyboard_handle) => {
                            tracing::info!("Keyboard added successfully to seat '{}'.", seat_name);
                            // Ensure XkbKeyboardData is initialized/updated for this seat and config.
                            // This might involve re-creating XkbKeyboardData if config changed,
                            // or if it wasn't created during create_seat.
                            match XkbKeyboardData::new(&kb_config) {
                                Ok(xkb_data) => {
                                    desktop_state.keyboard_data_map.insert(seat_name.to_string(), Arc::new(xkb_data));
                                    tracing::info!("XkbKeyboardData initialized/updated for seat '{}'.", seat_name);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to initialize XkbKeyboardData for seat '{}': {}. Keyboard may not function correctly.", seat_name, e);
                                    // Keyboard handle was added, but XKB data failed. This is problematic.
                                    // Consider removing the keyboard capability if XKB init fails critically.
                                    // seat.remove_keyboard(); // Or handle error more gracefully.
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to add keyboard to seat '{}': {}", seat_name, e);
                        }
                    }
                } else {
                    tracing::info!("Seat '{}' already has a keyboard. Ignoring keyboard capability for device {:?}.", seat_name, device.name());
                }
            }

            // Pointer capability
            if device.has_capability(DeviceCapability::Pointer) {
                if seat.get_pointer().is_none() {
                    tracing::info!("Device {:?} has Pointer capability. Adding pointer to seat '{}'.", device.name(), seat_name);
                    if let Err(e) = seat.add_pointer(Some(tracing::Span::current())) {
                        tracing::error!("Failed to add pointer to seat '{}': {}", seat_name, e);
                    } else {
                        tracing::info!("Pointer added successfully to seat '{}'.", seat_name);
                    }
                } else {
                    tracing::info!("Seat '{}' already has a pointer. Ignoring pointer capability for device {:?}.", seat_name, device.name());
                }
            }

            // Touch capability
            if device.has_capability(DeviceCapability::Touch) {
                if seat.get_touch().is_none() {
                    tracing::info!("Device {:?} has Touch capability. Adding touch to seat '{}'.", device.name(), seat_name);
                     if let Err(e) = seat.add_touch(Some(tracing::Span::current())) {
                        tracing::error!("Failed to add touch to seat '{}': {}", seat_name, e);
                    } else {
                        tracing::info!("Touch added successfully to seat '{}'.", seat_name);
                    }
                } else {
                    tracing::info!("Seat '{}' already has touch. Ignoring touch capability for device {:?}.", seat_name, device.name());
                }
            }
        }
        InputEvent::DeviceRemoved { device } => {
            tracing::info!(device_name = %device.name(), "Input device removed");
            // If the removed device was providing a capability that no other device provides,
            // then the capability should be removed from the seat.
            // Smithay's Seat::remove_keyboard/pointer/touch handles this gracefully.
            // It checks if the capability is still provided by other devices.

            // This logic is simplified: it assumes removal of *any* device with a capability
            // means removing that capability from the seat. A more robust implementation would
            // track which device provides which capability and only remove the capability if
            // no other device provides it. However, Smithay's Seat often handles this internally
            // by checking if other devices still support the capability.
            // For now, directly calling remove_* is okay as Smithay's Seat should manage it.

            if device.has_capability(DeviceCapability::Keyboard) {
                tracing::info!("Device {:?} had Keyboard capability. Removing keyboard from seat '{}' if no other device provides it.", device.name(), seat_name);
                seat.remove_keyboard();
                // Optionally, clean up XkbKeyboardData from desktop_state.keyboard_data_map
                // if this seat no longer has a keyboard.
                // desktop_state.keyboard_data_map.remove(seat_name);
                tracing::info!("Keyboard removed from seat '{}' (if it was the last one).", seat_name);
            }
            if device.has_capability(DeviceCapability::Pointer) {
                tracing::info!("Device {:?} had Pointer capability. Removing pointer from seat '{}' if no other device provides it.", device.name(), seat_name);
                seat.remove_pointer();
                tracing::info!("Pointer removed from seat '{}' (if it was the last one).", seat_name);
            }
            if device.has_capability(DeviceCapability::Touch) {
                tracing::info!("Device {:?} had Touch capability. Removing touch from seat '{}' if no other device provides it.", device.name(), seat_name);
                seat.remove_touch();
                tracing::info!("Touch removed from seat '{}' (if it was the last one).", seat_name);
            }
        }
        InputEvent::Keyboard { event } => {
            handle_keyboard_key_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::PointerMotion { event } => {
            handle_pointer_motion_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::PointerMotionAbsolute { event } => {
            handle_pointer_motion_absolute_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::PointerButton { event } => {
            handle_pointer_button_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::PointerAxis { event } => {
            handle_pointer_axis_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::TouchDown { event } => {
            handle_touch_down_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::TouchUp { event } => {
            handle_touch_up_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::TouchMotion { event } => {
            handle_touch_motion_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::TouchFrame { event } => {
            handle_touch_frame_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::TouchCancel { event } => {
            handle_touch_cancel_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::GestureSwipeBegin { event: _ } => { // Added event bindings
            tracing::debug!(seat_name = %seat_name, "Gesture swipe begin event (unhandled)");
        }
        InputEvent::GestureSwipeUpdate { event: _ } => {
            tracing::trace!(seat_name = %seat_name, "Gesture swipe update event (unhandled)");
        }
        InputEvent::GestureSwipeEnd { event: _ } => {
            tracing::debug!(seat_name = %seat_name, "Gesture swipe end event (unhandled)");
        }
        InputEvent::GesturePinchBegin { event: _ } => {
            tracing::debug!(seat_name = %seat_name, "Gesture pinch begin event (unhandled)");
        }
        InputEvent::GesturePinchUpdate { event: _ } => {
            tracing::trace!(seat_name = %seat_name, "Gesture pinch update event (unhandled)");
        }
        InputEvent::GesturePinchEnd { event: _ } => {
            tracing::debug!(seat_name = %seat_name, "Gesture pinch end event (unhandled)");
        }
        InputEvent::GestureHoldBegin { event: _ } => {
            tracing::debug!(seat_name = %seat_name, "Gesture hold begin event (unhandled)");
        }
        InputEvent::GestureHoldEnd { event: _ } => {
            tracing::debug!(seat_name = %seat_name, "Gesture hold end event (unhandled)");
        }
        // Other events like TabletTool related events can be added here if needed.
        _ => { // Generic catch-all for any other InputEvent variants
            tracing::trace!(seat_name = %seat_name, "Other unhandled input event: {:?}", event);
        }
    }
}
