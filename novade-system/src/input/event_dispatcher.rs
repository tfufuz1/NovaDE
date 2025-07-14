use smithay::{
    backend::input::{InputEvent, LibinputInputBackend, Device, DeviceCapability},
    input::Seat,
    reexports::libinput, // To access libinput::DeviceCapability more directly if needed
};
use crate::compositor::state::DesktopState;

pub fn process_input_event(
    desktop_state: &mut DesktopState,
    event: InputEvent<LibinputInputBackend>,
    seat_name: &str,
) {
    // Retrieve the specific seat by name. Cloned because `Seat` itself doesn't implement `Copy`,
    // and we might need to pass `desktop_state` mutably to seat methods later.
    // However, `Seat` methods typically take `&self` or `&mut self` on the `Seat` itself,
    // and `desktop_state` (the `D` in `Seat<D>`) as `&mut D`.
    // Smithay operations often involve finding the seat and then calling methods on it
    // which then internally access `desktop_state.seat_state` or other parts of `D`.
    // For operations like `seat.add_keyboard()`, we need `&Seat<DesktopState>`.
    // `seat_state.seats()` returns an iterator of `&Seat<D>`.
    // Let's find the seat first and then operate.

    let seat_handle = match desktop_state.seat_state.seats().find(|s| s.name() == seat_name) {
        Some(s) => s.clone(), // .clone() is important here if seat methods need ownership or if we modify seat_state
        None => {
            tracing::error!("Seat '{}' not found for input event: {:?}", seat_name, event);
            return;
        }
    };
    // After cloning, seat_handle is a Seat<DesktopState>.
    // Methods on seat_handle like seat_handle.add_keyboard(...) will correctly use the original DesktopState.

    match event {
        InputEvent::DeviceAdded { device } => {
            tracing::info!("Input device added: {} (Sys: {}) on seat {}", device.name(), device.sysname(), seat_name);

            if device.has_capability(libinput::DeviceCapability::Keyboard) {
                if seat_handle.get_keyboard().is_none() {
                    // Use the cloned config from KeyboardLayoutManager, similar to DesktopState::new
                    let xkb_config = desktop_state.keyboard_layout_manager.xkb_config_cloned();
                    match seat_handle.add_keyboard(xkb_config, 200, 25) {
                        Ok(_) => tracing::info!("Keyboard capability added to seat '{}'.", seat_name),
                        Err(e) => tracing::error!("Failed to add keyboard to seat '{}': {}", seat_name, e),
                    }
                } else {
                     tracing::info!("Seat '{}' already has keyboard capability.", seat_name);
                }
            }

            if device.has_capability(libinput::DeviceCapability::Pointer) {
                if seat_handle.get_pointer().is_none() {
                    if let Err(e) = seat_handle.add_pointer() {
                        tracing::error!("Failed to add pointer to seat '{}': {}", seat_name, e);
                    } else {
                        tracing::info!("Pointer capability added to seat '{}'.", seat_name);
                    }
                } else {
                    tracing::info!("Seat '{}' already has pointer capability.", seat_name);
                }
            }

            if device.has_capability(libinput::DeviceCapability::Touch) {
                if seat_handle.get_touch().is_none() {
                    if let Err(e) = seat_handle.add_touch() {
                        tracing::error!("Failed to add touch to seat '{}': {}", seat_name, e);
                    } else {
                        tracing::info!("Touch capability added to seat '{}'.", seat_name);
                    }
                } else {
                    tracing::info!("Seat '{}' already has touch capability.", seat_name);
                }
            }
        }
        InputEvent::DeviceRemoved { device } => {
            tracing::info!("Input device removed: {} (Sys: {}) on seat {}", device.name(), device.sysname(), seat_name);
            if device.has_capability(libinput::DeviceCapability::Keyboard) && seat_handle.get_keyboard().is_some() {
                seat_handle.remove_keyboard();
                tracing::info!("Keyboard capability removed from seat '{}'.", seat_name);
            }
            if device.has_capability(libinput::DeviceCapability::Pointer) && seat_handle.get_pointer().is_some() {
                seat_handle.remove_pointer();
                tracing::info!("Pointer capability removed from seat '{}'.", seat_name);
            }
            if device.has_capability(libinput::DeviceCapability::Touch) && seat_handle.get_touch().is_some() {
                seat_handle.remove_touch();
                tracing::info!("Touch capability removed from seat '{}'.", seat_name);
            }
        }
        InputEvent::Keyboard { event, .. } => {
            // The 'event' variable here is of type smithay::backend::input::KeyboardKeyEvent<LibinputInputBackend>
            tracing::debug!("Keyboard event received on seat {}: Raw libinput event: {:?}", seat_name, event);
            // Pass &seat_handle because handle_keyboard_key_event expects &Seat<DesktopState>
            if let Err(e) = crate::input::keyboard::handle_keyboard_key_event(desktop_state, &seat_handle, event, seat_name) {
                tracing::error!("Error handling keyboard event for seat {}: {:?}", seat_name, e);
            }
        }
        InputEvent::PointerMotion { event, .. } => {
            // tracing::debug!("PointerMotion event received on seat {}: {:?}", seat_name, event);
            if let Err(e) = crate::input::pointer::handle_pointer_motion_event(desktop_state, &seat_handle, event) {
                tracing::error!("Error handling pointer motion event for seat {}: {:?}", seat_name, e);
            }
        }
        InputEvent::PointerMotionAbsolute { event, .. } => {
            // tracing::debug!("PointerMotionAbsolute event received on seat {}: {:?}", seat_name, event);
            if let Err(e) = crate::input::pointer::handle_pointer_motion_absolute_event(desktop_state, &seat_handle, event) {
                tracing::error!("Error handling pointer motion absolute event for seat {}: {:?}", seat_name, e);
            }
        }
        InputEvent::PointerButton { event, .. } => {
            // tracing::debug!("PointerButton event received on seat {}: {:?}", seat_name, event);
            if let Err(e) = crate::input::pointer::handle_pointer_button_event(desktop_state, &seat_handle, event) {
                tracing::error!("Error handling pointer button event for seat {}: {:?}", seat_name, e);
            }
        }
        InputEvent::PointerAxis { event, .. } => {
            // tracing::debug!("PointerAxis event received on seat {}: {:?}", seat_name, event);
            if let Err(e) = crate::input::pointer::handle_pointer_axis_event(desktop_state, &seat_handle, event) {
                tracing::error!("Error handling pointer axis event for seat {}: {:?}", seat_name, e);
            }
        }
        InputEvent::TouchDown { event, .. } => {
            // tracing::debug!("TouchDown event received on seat {}: {:?}", seat_name, event);
            if let Err(e) = crate::input::touch::handle_touch_down_event(desktop_state, &seat_handle, event) {
                tracing::error!("Error handling touch down event for seat {}: {:?}", seat_name, e);
            }
        }
        InputEvent::TouchUp { event, .. } => {
            // tracing::debug!("TouchUp event received on seat {}: {:?}", seat_name, event);
            if let Err(e) = crate::input::touch::handle_touch_up_event(desktop_state, &seat_handle, event) {
                tracing::error!("Error handling touch up event for seat {}: {:?}", seat_name, e);
            }
        }
        InputEvent::TouchMotion { event, .. } => {
            // tracing::debug!("TouchMotion event received on seat {}: {:?}", seat_name, event);
            if let Err(e) = crate::input::touch::handle_touch_motion_event(desktop_state, &seat_handle, event) {
                tracing::error!("Error handling touch motion event for seat {}: {:?}", seat_name, e);
            }
        }
        InputEvent::TouchFrame { event, .. } => {
            // tracing::debug!("TouchFrame event received on seat {}", seat_name);
            if let Err(e) = crate::input::touch::handle_touch_frame_event(desktop_state, &seat_handle, event) {
                tracing::error!("Error handling touch frame event for seat {}: {:?}", seat_name, e);
            }
        }
        InputEvent::TouchCancel { event, .. } => {
            // tracing::debug!("TouchCancel event received on seat {}", seat_name);
            if let Err(e) = crate::input::touch::handle_touch_cancel_event(desktop_state, &seat_handle, event) {
                tracing::error!("Error handling touch cancel event for seat {}: {:?}", seat_name, e);
            }
        }
        InputEvent::GestureSwipeBegin { event, .. } => {
             tracing::debug!("GestureSwipeBegin event on seat {}: {:?}", seat_name, event);
        }
        InputEvent::GestureSwipeUpdate { event, .. } => {
             tracing::debug!("GestureSwipeUpdate event on seat {}: {:?}", seat_name, event);
        }
        InputEvent::GestureSwipeEnd { event, .. } => {
             tracing::debug!("GestureSwipeEnd event on seat {}: {:?}", seat_name, event);
        }
        InputEvent::GesturePinchBegin { event, .. } => {
             tracing::debug!("GesturePinchBegin event on seat {}: {:?}", seat_name, event);
        }
        InputEvent::GesturePinchUpdate { event, .. } => {
             tracing::debug!("GesturePinchUpdate event on seat {}: {:?}", seat_name, event);
        }
        InputEvent::GesturePinchEnd { event, .. } => {
             tracing::debug!("GesturePinchEnd event on seat {}: {:?}", seat_name, event);
        }
        // Explicitly handle other InputEvent variants Smithay might have added
        InputEvent::GestureHoldBegin { event, .. } => {
            tracing::debug!("GestureHoldBegin event on seat {}: {:?}", seat_name, event);
        }
        InputEvent::GestureHoldEnd { event, .. } => {
            tracing::debug!("GestureHoldEnd event on seat {}: {:?}", seat_name, event);
        }
        InputEvent::TabletToolAxis { event, .. } => {
            tracing::debug!("TabletToolAxis event on seat {}: {:?}", seat_name, event);
        }
        InputEvent::TabletToolProximity { event, .. } => {
            tracing::debug!("TabletToolProximity event on seat {}: {:?}", seat_name, event);
        }
        InputEvent::TabletToolTip { event, .. } => {
            tracing::debug!("TabletToolTip event on seat {}: {:?}", seat_name, event);
        }
        InputEvent::TabletToolButton { event, .. } => {
            tracing::debug!("TabletToolButton event on seat {}: {:?}", seat_name, event);
        }
        _ => { // Fallback for any other event variants not explicitly listed
            tracing::debug!("Unhandled input event variant on seat {}: {:?}", seat_name, event);
        }
    }
}
