// system/src/input/libinput_handler/mod.rs
pub mod session_interface;

// Re-export for easier access if needed by other parts of the input system
pub use session_interface::LibinputSessionManager;

use crate::compositor::core::state::DesktopState;
use crate::input::errors::InputError;
use smithay::backend::input::{
    InputEvent, LibinputInputBackend, LibinputInterface, DeviceCapability, DeviceEvent,
    KeyboardKeyEvent, PointerMotionAbsoluteEvent, PointerButtonEvent, PointerAxisEvent,
    PointerMotionEvent, TouchDownEvent, TouchUpEvent, TouchMotionEvent, TouchFrameEvent,
    TouchCancelEvent,
};
use smithay::reexports::input::Libinput;
use smithay::reexports::input::event::EventTrait; // For event.time()
use smithay::reexports::calloop::{LoopHandle, RegistrationToken}; // Add RegistrationToken
use std::rc::Rc;
use std::cell::RefCell;
use tracing::{error, info, warn, trace}; // For logging

pub fn init_libinput_backend<I: LibinputInterface + 'static>(
    session_interface: Rc<RefCell<I>>,
) -> Result<LibinputInputBackend, InputError> {
    info!("Initialisiere Libinput-Backend...");

    let mut libinput_context = Libinput::new_from_path(session_interface);

    if let Err(e) = libinput_context.udev_assign_seat("seat0") {
        warn!("Zuweisung zu udev seat0 fehlgeschlagen (non-fatal): {:?}", e);
    } else {
        info!("Libinput context erfolgreich an seat0 zugewiesen.");
    }
    
    let libinput_backend = LibinputInputBackend::new(libinput_context.into(), Some(tracing::Span::current()));
    info!("Libinput-Backend erfolgreich initialisiert.");
    Ok(libinput_backend)
}

#[allow(dead_code)] // Remove when all event types are handled
pub fn process_input_event(
    desktop_state: &mut DesktopState,
    event: InputEvent<LibinputInputBackend>,
    seat_name: &str,
) {
    let seat = match desktop_state.seat_state.seats().find(|s| s.name() == seat_name) {
        Some(s) => s.clone(),
        None => {
            error!("Seat '{}' nicht gefunden für Input-Event: {:?}", seat_name, event);
            return;
        }
    };

    match event {
        InputEvent::DeviceAdded { device } => {
            info!("Eingabegerät hinzugefügt: {} (Sys: {}) auf Seat '{}'", device.name(), device.sysname(), seat_name);
            if device.has_capability(DeviceCapability::Keyboard) {
                if seat.get_keyboard().is_none() {
                    match seat.add_keyboard(Default::default(), 200, 25) {
                        Ok(_) => info!("Tastatur-Capability zu Seat '{}' hinzugefügt.", seat_name),
                        Err(e) => error!("Fehler beim Hinzufügen der Tastatur-Capability zu Seat '{}': {}", seat_name, e),
                    }
                }
            }
            if device.has_capability(DeviceCapability::Pointer) {
                if seat.get_pointer().is_none() {
                    if let Err(e) = seat.add_pointer() {
                        error!("Fehler beim Hinzufügen der Zeiger-Capability zu Seat '{}': {}", seat_name, e);
                    } else {
                        info!("Zeiger-Capability zu Seat '{}' hinzugefügt.", seat_name);
                    }
                }
            }
            if device.has_capability(DeviceCapability::Touch) {
                if seat.get_touch().is_none() {
                    if let Err(e) = seat.add_touch() {
                        error!("Fehler beim Hinzufügen der Touch-Capability zu Seat '{}': {}", seat_name, e);
                    } else {
                        info!("Touch-Capability zu Seat '{}' hinzugefügt.", seat_name);
                    }
                }
            }
        }
        InputEvent::DeviceRemoved { device } => {
            info!("Eingabegerät entfernt: {} auf Seat '{}'", device.name(), seat_name);
            if device.has_capability(DeviceCapability::Keyboard) && seat.get_keyboard().is_some() {
                seat.remove_keyboard();
                info!("Tastatur-Capability von Seat '{}' entfernt.", seat_name);
            }
            if device.has_capability(DeviceCapability::Pointer) && seat.get_pointer().is_some() {
                seat.remove_pointer();
                info!("Zeiger-Capability von Seat '{}' entfernt.", seat_name);
            }
            if device.has_capability(DeviceCapability::Touch) && seat.get_touch().is_some() {
                seat.remove_touch();
                info!("Touch-Capability von Seat '{}' entfernt.", seat_name);
            }
        }
        InputEvent::Keyboard { event } => {
            trace!("Keyboard event: {:?} on seat '{}'", event.key_code(), seat_name);
            crate::input::keyboard::key_event_translator::handle_keyboard_key_event(desktop_state, &seat, event, seat_name);
        }
        InputEvent::PointerMotion { event } => {
            trace!("Pointer motion event on seat '{}': delta ({},{})", seat_name, event.delta_x(), event.delta_y());
            crate::input::pointer::pointer_event_translator::handle_pointer_motion_event(desktop_state, &seat, event);
        }
        InputEvent::PointerMotionAbsolute { event } => {
            trace!("Pointer motion absolute event on seat '{}': ({},{})", seat_name, event.absolute_x_transformed(0), event.absolute_y_transformed(0));
            crate::input::pointer::pointer_event_translator::handle_pointer_motion_absolute_event(desktop_state, &seat, event);
        }
        InputEvent::PointerButton { event } => {
            trace!("Pointer button event on seat '{}': button {}, state {:?}", seat_name, event.button(), event.button_state());
            crate::input::pointer::pointer_event_translator::handle_pointer_button_event(desktop_state, &seat, event);
        }
        InputEvent::PointerAxis { event } => {
            trace!("Pointer axis event on seat '{}': {:?}, source {:?}", seat_name, event.axis(), event.axis_source());
            crate::input::pointer::pointer_event_translator::handle_pointer_axis_event(desktop_state, &seat, event);
        }
        InputEvent::TouchDown { event } => {
            trace!("Touch down event on seat '{}': slot {:?}", seat_name, event.slot());
            crate::input::touch::touch_event_translator::handle_touch_down_event(desktop_state, &seat, event);
        }
        InputEvent::TouchUp { event } => {
            trace!("Touch up event on seat '{}': slot {:?}", seat_name, event.slot());
            crate::input::touch::touch_event_translator::handle_touch_up_event(desktop_state, &seat, event);
        }
        InputEvent::TouchMotion { event } => {
            trace!("Touch motion event on seat '{}': slot {:?}", seat_name, event.slot());
            crate::input::touch::touch_event_translator::handle_touch_motion_event(desktop_state, &seat, event);
        }
        InputEvent::TouchFrame { event: _ } => {
            trace!("Touch frame event on seat '{}'", seat_name);
            crate::input::touch::touch_event_translator::handle_touch_frame_event(desktop_state, &seat);
        }
        InputEvent::TouchCancel { event: _ } => {
            trace!("Touch cancel event on seat '{}'", seat_name);
            crate::input::touch::touch_event_translator::handle_touch_cancel_event(desktop_state, &seat);
        }
        InputEvent::GesturePinchBegin { event } => {
            trace!("Gesture Pinch Begin on seat '{}': fingers {}", seat_name, event.finger_count());
            // crate::input::gestures::handle_gesture_pinch_begin(desktop_state, &seat, event);
        }
        InputEvent::GesturePinchUpdate { event } => {
            trace!("Gesture Pinch Update on seat '{}': scale {}, delta {}", seat_name, event.scale(), event.delta_unaccelerated());
            // crate::input::gestures::handle_gesture_pinch_update(desktop_state, &seat, event);
        }
        InputEvent::GesturePinchEnd { event } => {
            trace!("Gesture Pinch End on seat '{}': cancelled {}", seat_name, event.is_cancelled());
            // crate::input::gestures::handle_gesture_pinch_end(desktop_state, &seat, event);
        }
        InputEvent::GestureSwipeBegin { event } => {
            trace!("Gesture Swipe Begin on seat '{}': fingers {}", seat_name, event.finger_count());
            // crate::input::gestures::handle_gesture_swipe_begin(desktop_state, &seat, event);
        }
        InputEvent::GestureSwipeUpdate { event } => {
            trace!("Gesture Swipe Update on seat '{}': delta ({}, {})", seat_name, event.delta_x_unaccelerated(), event.delta_y_unaccelerated());
            // crate::input::gestures::handle_gesture_swipe_update(desktop_state, &seat, event);
        }
        InputEvent::GestureSwipeEnd { event } => {
            trace!("Gesture Swipe End on seat '{}': cancelled {}", seat_name, event.is_cancelled());
            // crate::input::gestures::handle_gesture_swipe_end(desktop_state, &seat, event);
        }
        other_event => {
            trace!("Unhandled input event on seat '{}': {:?}", seat_name, other_event);
        }
    }
}

// This function will register the libinput backend with the calloop event loop.
// It's designed to be called after init_libinput_backend.
pub fn register_libinput_event_source(
    loop_handle: &LoopHandle<'static, DesktopState>,
    libinput_backend: LibinputInputBackend,
    seat_name: String, // seat_name is moved into the closure
) -> Result<RegistrationToken, InputError> {
    tracing::info!("Registriere Libinput-Ereignisquelle für Seat '{}' in der Event-Loop...", seat_name);

    let registration_token = loop_handle.insert_source(
        libinput_backend,
        move |event: InputEvent<LibinputInputBackend>, _metadata, desktop_state: &mut DesktopState| {
            // Dispatch the event to the central handler
            process_input_event(desktop_state, event, &seat_name);
        }
    )
    .map_err(|e| {
        tracing::error!("Fehler beim Einfügen der Libinput-Ereignisquelle in die Event-Loop: {}", e);
        InputError::EventSourceSetupError(e.to_string())
    })?;

    tracing::info!("Libinput-Ereignisquelle erfolgreich für Seat '{}' registriert.", seat_name);
    Ok(registration_token)
}
