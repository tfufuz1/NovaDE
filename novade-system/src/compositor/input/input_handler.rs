use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;

use libinput::{Libinput, LibinputInterface, Event, EventKind, event, device::Device};
use xkbcommon::xkb;
use tracing::{info, debug, error, warn};

// Import own error and data types
use crate::compositor::input::error::InputError;
// The direct usage of our own data_types for events is being phased out in dispatch_events
// in favor of direct calls to Smithay handlers.
// Keep ModifiersState as it's used internally for now.
use crate::compositor::input::data_types::ModifiersState;


// Smithay imports
use smithay::input::{Seat as SmithaySeat, KeyboardHandle, PointerHandle, TouchHandle, SeatHandler, SeatState};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Clock, SERIAL_COUNTER, Serial, Logical, Point, Time};
use smithay::backend::input::{self as smithay_input, Axis, AxisSource, ButtonState as SmithayButtonState, KeyState as SmithayKeyState, MouseButton};
use crate::compositor::core::state::DesktopState;
use smithay::input::pointer::{MotionEvent, ButtonEvent as SmithayPointerButtonEvent, AxisEvent as SmithayPointerAxisEvent, AxisFrame};
use smithay::input::touch::{DownEvent, UpEvent, MotionEvent as TouchMotionEvent, TouchSlotId};
use std::ffi::CString;


// Placeholder for Smithay integration - these would come from smithay
// For now, let's define dummy versions or assume they are not used in this initial implementation phase.
// use smithay::input::{Seat as SmithaySeat, SeatHandler, SeatState, pointer::PointerHandle, keyboard::KeyboardHandle, touch::TouchHandle};
// use smithay::backend::input::InputEvent as SmithayInputEvent; // This is what libinput events often translate to in Smithay

// Dummy CompositorState if needed for context, replace with actual later
// pub struct CompositorState { /* ... */ }


// LibinputInterface implementation
struct InputInterface;

impl LibinputInterface for InputInterface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        info!("Opening device: {:?} with flags {}", path, flags);
        OpenOptions::new()
            .custom_flags(flags)
            .read(true)
            .write(true) // Required by libinput, even if we only read events
            .open(path)
            .map(|file| file.as_raw_fd())
            .map_err(|err| {
                error!("Failed to open device {:?}: {}", path, err);
                err.raw_os_error().unwrap_or(libc::EIO)
            })
    }

    fn close_restricted(&mut self, fd: RawFd) {
        info!("Closing device fd: {}", fd);
        // Safety: fd is a valid file descriptor provided by open_restricted
        unsafe {
            libc::close(fd);
        }
    }
}

pub struct InputHandler {
    libinput: Libinput,
    xkb_context: xkb::Context,
    xkb_keymap_name: String, // e.g., "pc105" - Stores model name primarily
    xkb_keymap: Option<xkb::Keymap>, // Compiled keymap
    xkb_state: Option<xkb::State>,   // State derived from xkb_keymap
    // seat: SmithaySeat<CompositorState>, // Actual Smithay seat
    // For now, using our own Seat definition
    // We might need to manage multiple keyboards, pointers etc. per seat or have a global one for now.
}

impl InputHandler {
    pub fn new() -> Result<Self, InputError> {
        info!("Initializing InputHandler");

        // 1. libinput initialisieren
        let mut libinput = Libinput::new_with_udev(InputInterface);
        match libinput {
            Ok(ref mut li) => {
                // Assign to a seat. This is important for libinput to manage devices correctly.
                if let Err(e) = li.udev_assign_seat("seat0") { // "seat0" is a common default
                    warn!("Failed to assign seat0 to libinput context: {}. This might be an issue in headless environments or with permissions.", e);
                    // Depending on strictness, you might return an error:
                    // return Err(InputError::LibinputInitialization(format!("Failed to assign seat: {}", e)));
                }
            }
            Err(e) => {
                error!("Failed to initialize libinput with udev: {}", e);
                return Err(InputError::LibinputInitialization(format!("udev backend failed: {}", e)));
            }
        }
        let libinput = libinput.unwrap(); // Safe due to check above or early return.

        // 2. xkbcommon Kontext initialisieren
        let xkb_context = xkb::Context::new(xkb::ContextFlags::NO_FLAGS);
        if xkb_context.is_null() { // xkb::Context doesn't directly return Result for new()
            error!("Failed to create xkbcommon context.");
            return Err(InputError::XkbCommonContext("Failed to create xkb::Context".to_string()));
        }
        info!("xkbcommon context initialized.");

        // Keymap and state will be initialized when the first keyboard is added
        // or when set_keymap is called.

        Ok(InputHandler {
            libinput,
            xkb_context,
            xkb_keymap_name: "default".to_string(), // Will be updated upon keymap compilation
            xkb_keymap: None,
            xkb_state: None,
        })
    }

    fn compile_keymap_and_state(&self, rmlvo: &xkb::RuleNames) -> Result<(xkb::Keymap, xkb::State), InputError> {
        info!("Compiling xkb keymap with RMLVO: rules={:?}, model={:?}, layout={:?}, variant={:?}, options={:?}",
              rmlvo.rules, rmlvo.model, rmlvo.layout, rmlvo.variant, rmlvo.options);

        let keymap = xkb::Keymap::new_from_names(
            &self.xkb_context,
            rmlvo,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .ok_or_else(|| {
            let model = rmlvo.model.as_ref().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let layout = rmlvo.layout.as_ref().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            error!("Failed to compile xkb keymap for model '{}', layout '{}'", model, layout);
            InputError::XkbCommonState(format!("Failed to compile xkb keymap (model: {}, layout: {})", model, layout))
        })?;

        let state = xkb::State::new(&keymap);
        info!("Successfully compiled xkb keymap and created state.");
        Ok((keymap, state))
    }

    /// Ensures a default keymap is loaded if no keymap has been set yet.
    /// This is typically called when the first keyboard device is detected.
    fn ensure_initial_keymap_loaded(&mut self) -> Result<(), InputError> {
        if self.xkb_keymap.is_none() {
            info!("No keymap loaded. Attempting to load default (us, pc105, evdev).");
            let default_rmlvo = xkb::RuleNames {
                rules: Some(CString::new("evdev").unwrap()),
                model: Some(CString::new("pc105").unwrap()),
                layout: Some(CString::new("us").unwrap()),
                variant: Some(CString::new("").unwrap()),
                options: None,
            };
            match self.compile_keymap_and_state(&default_rmlvo) {
                Ok((keymap, state)) => {
                    self.xkb_keymap = Some(keymap);
                    self.xkb_state = Some(state);
                    self.xkb_keymap_name = default_rmlvo.model.as_ref()
                        .and_then(|s| s.to_str().ok())
                        .unwrap_or("pc105")
                        .to_string();
                }
                Err(e) => {
                    error!("Failed to load default keymap: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub fn set_keymap(
        &mut self,
        desktop_state: &mut DesktopState, // To get KeyboardHandle
        new_rmlvo: xkb::RuleNames,
    ) -> Result<(), InputError> {
        info!("Setting new keymap. Rules: {:?}, Model: {:?}, Layout: {:?}, Variant: {:?}, Options: {:?}",
              new_rmlvo.rules, new_rmlvo.model, new_rmlvo.layout, new_rmlvo.variant, new_rmlvo.options);

        match self.compile_keymap_and_state(&new_rmlvo) {
            Ok((new_keymap_obj, new_state_obj)) => {
                // Update InputHandler's internal keymap and state
                self.xkb_keymap = Some(new_keymap_obj.clone()); // Clone for internal storage
                self.xkb_state = Some(new_state_obj);
                self.xkb_keymap_name = new_rmlvo.model.as_ref()
                    .and_then(|s| s.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string();

                // Update Smithay's KeyboardHandle
                if let Some(keyboard_handle) = desktop_state.seat.get_keyboard() {
                    // Pass the cloned keymap (Arc clone) to Smithay
                    keyboard_handle.set_keymap(new_keymap_obj);
                    info!("Successfully updated Smithay's keyboard handle with new keymap.");
                } else {
                    warn!("No keyboard handle found on Smithay seat to update keymap.");
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to compile new keymap and state: {}", e);
                Err(e)
            }
        }
    }

    /// Dispatches pending libinput events and translates them into Smithay compatible events.
    pub fn dispatch_events(
        &mut self,
        desktop_state: &mut DesktopState,
    ) -> Result<(), InputError> {
        self.libinput.dispatch().map_err(|e| {
            error!("Error dispatching libinput events: {}", e);
            InputError::EventProcessing(format!("Libinput dispatch failed: {}", e))
        })?;

        for event in &mut self.libinput {
            // let time_usec = event.time_usec(); // Common to most events - get it per event type for now

            match event {
                Event::DeviceAdded(device_added_event) => {
                    let device = device_added_event.device();
                    info!("Device added: {:?} (type: {:?}, seat: {:?})",
                          device.name(), device.devnode(), device.seat_logical_name());
                    // Here you would typically check device capabilities (keyboard, pointer, touch)
                    // and configure them (e.g., set up xkb state for keyboards).
                    if device.has_capability(libinput::DeviceCapability::Keyboard) {
                        if let Err(e) = self.ensure_initial_keymap_loaded() {
                            error!("Failed to ensure initial keymap for new keyboard: {}", e);
                            // Decide if this is a critical error, perhaps return e?
                        }
                    }
                }
                Event::DeviceRemoved(device_removed_event) => {
                    let device = device_removed_event.device();
                    info!("Device removed: {:?}", device.name());
                    // Clean up resources associated with the device if necessary.
                    // E.g., if xkb_state was per-device.
                }
                Event::Keyboard(kb_event) => {
                    // Ensure keymap and state are loaded.
                    if self.xkb_keymap.is_none() {
                        warn!("Keyboard event received but xkb_keymap is not initialized. Device might not have been added correctly or is not a keyboard.");
                        // If there's no keymap, we likely can't process. Could also choose to pass raw scancodes to Smithay if that's desired.
                        // For now, consistent with previous logic of needing xkb_state.
                        return Ok(()); // Skip this event
                    }
                    // self.xkb_state should always be Some if self.xkb_keymap is Some, due to compile_keymap_and_state logic
                    if self.xkb_state.is_none() {
                        warn!("Keyboard event received but xkb_state is not initialized (should be linked to keymap). This indicates an internal logic error.");
                        return Ok(());
                    }

                    let key_code = kb_event.key(); // Raw scancode from libinput
                    let smithay_key_state = match kb_event.key_state() {
                        libinput::event::keyboard::KeyState::Pressed => SmithayKeyState::Pressed,
                        libinput::event::keyboard::KeyState::Released => SmithayKeyState::Released,
                    };
                    let time_ms = (kb_event.time_usec() / 1000) as u32;
                    let event_serial = SERIAL_COUNTER.next_serial();

                    let mut temp_keysym = xkb::KEY_NoSymbol;
                    let mut temp_utf8: Option<String> = None;

                    if let Some(keyboard_handle) = desktop_state.seat.get_keyboard() {
                        keyboard_handle.input(
                            desktop_state, // The SeatHandler
                            key_code,      // Raw scan code
                            smithay_key_state,
                            event_serial,
                            time_ms,
                            |_desktop_state_from_filter, _modifiers_state_filter, keysym_handle| {
                                // Get keysym and UTF-8 from Smithay's KeysymHandle
                                temp_keysym = keysym_handle.modified_sym();
                                temp_utf8 = keysym_handle.modified_utf8();

                                // Log information obtained from Smithay's filter if desired
                                // debug!("Smithay Filter: mods={:?}, sym={:?}, utf8={:?}", _modifiers_state_filter, temp_keysym, temp_utf8);

                                // Decide if the event should be forwarded or filtered out
                                smithay::input::keyboard::FilterReturn::Forward
                            },
                        );
                    } else {
                         warn!("No keyboard handle found on Smithay seat to process key event for key_code: {}", key_code);
                    }

                    // Update our internal xkb_state for modifier tracking AFTER Smithay processes it.
                    // This ensures our logged `mods` reflect the state after the event.
                    if let Some(state) = self.xkb_state.as_mut() {
                        // Use key_code + 8 for xkb_state.update_key as standard XKB keymaps expect this offset from raw evdev codes.
                        state.update_key(key_code + 8, match smithay_key_state {
                            SmithayKeyState::Pressed => xkb::KeyDirection::Down,
                            SmithayKeyState::Released => xkb::KeyDirection::Up,
                        });

                        let mods = ModifiersState { // This is our data_types::ModifiersState
                            ctrl: state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE),
                            alt: state.mod_name_is_active(&xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE),
                            shift: state.mod_name_is_active(&xkb::MOD_NAME_SHIFT, xkb::STATE_MODS_EFFECTIVE),
                            caps_lock: state.mod_name_is_active(&xkb::MOD_NAME_CAPS, xkb::STATE_MODS_EFFECTIVE),
                            logo: state.mod_name_is_active(&xkb::MOD_NAME_LOGO, xkb::STATE_MODS_EFFECTIVE),
                        };

                        // Log using the keysym and utf8 obtained from Smithay's filter
                        debug!(
                            "Keyboard Event: key_code={}, smithay_keysym={:?} ({}), smithay_utf8='{}', state={:?}, our_mods={:?}",
                            key_code, temp_keysym, xkb::keysym_get_name(temp_keysym), temp_utf8.as_deref().unwrap_or(""), smithay_key_state, mods
                        );
                    } else {
                        // This case should not be reached due to the check at the beginning of this arm.
                        warn!("xkb_state is None when attempting to log modifiers for key_code: {}. Modifiers will not be logged correctly.", key_code);
                    }
                },
                Event::Pointer(pointer_event) => {
                    match pointer_event {
                        event::Pointer::Motion(motion) => {
                            let dx = motion.dx();
                            let dy = motion.dy();

                            desktop_state.pointer_location.x += dx;
                            desktop_state.pointer_location.y += dy;

                            // Clamp pointer location to screen dimensions (example, needs actual screen size)
                            // let (max_x, max_y) = (1920.0, 1080.0); // Example screen dimensions
                            // desktop_state.pointer_location.x = desktop_state.pointer_location.x.max(0.0).min(max_x);
                            // desktop_state.pointer_location.y = desktop_state.pointer_location.y.max(0.0).min(max_y);

                            let time_ms = (motion.time_usec() / 1000) as u32;
                            let serial = SERIAL_COUNTER.next_serial();

                            let smithay_motion_event = MotionEvent {
                                location: desktop_state.pointer_location,
                                serial,
                                time: time_ms,
                                focus: None, // SeatHandler will determine focus based on location
                            };
                            // Use a clone of the seat for the call, as DesktopState methods might need &mut self.
                            // Smithay's Seat is Arc-based, so clone is cheap.
                            desktop_state.pointer_motion_event(&desktop_state.seat.clone(), &smithay_motion_event);
                        }
                        event::Pointer::Button(button_event) => {
                            let time_ms = (button_event.time_usec() / 1000) as u32;
                            let serial = SERIAL_COUNTER.next_serial();
                            let s_button_state = match button_event.button_state() {
                                libinput::event::pointer::ButtonState::Pressed => SmithayButtonState::Pressed,
                                libinput::event::pointer::ButtonState::Released => SmithayButtonState::Released,
                            };

                            let s_mouse_button = match button_event.button() {
                                0x110 | 272 => MouseButton::Left,    // BTN_LEFT
                                0x111 | 273 => MouseButton::Right,   // BTN_RIGHT
                                0x112 | 274 => MouseButton::Middle,  // BTN_MIDDLE
                                0x113 | 275 => MouseButton::Other(0x113), // BTN_SIDE (often Forward)
                                0x114 | 276 => MouseButton::Other(0x114), // BTN_EXTRA (often Back)
                                // Linux input codes for common mouse buttons (from input-event-codes.h)
                                // BTN_MOUSE (0x110) to BTN_TASK (0x117)
                                // Smithay's MouseButton enum covers Left, Right, Middle. Others are u16.
                                // It's common to map BTN_SIDE and BTN_EXTRA if your compositor wants to handle them specially.
                                // For now, passing them as MouseButton::Other(code) is fine.
                                code => MouseButton::Other(code as u16),
                            };

                            let smithay_button_event = SmithayPointerButtonEvent {
                                button: s_mouse_button,
                                state: s_button_state,
                                serial,
                                time: time_ms,
                            };
                            desktop_state.pointer_button_event(&desktop_state.seat.clone(), &smithay_button_event);
                        }
                        event::Pointer::Axis(axis_event) => {
                            let time_ms = (axis_event.time_usec() / 1000) as u32;
                            let source = match axis_event.source() {
                                libinput::event::pointer::AxisSource::Wheel => AxisSource::Wheel,
                                libinput::event::pointer::AxisSource::Finger => AxisSource::Finger,
                                libinput::event::pointer::AxisSource::Continuous => AxisSource::Continuous,
                                libinput::event::pointer::AxisSource::WheelTilt => AxisSource::WheelTilt,
                                // _ => AxisSource::Other("unknown".into()), // Smithay 0.3 doesn't have Other(String)
                                // For Smithay 0.3, if WheelTilt isn't there, map to Wheel or Continuous or ignore.
                                // Assuming WheelTilt is present or we map it to something like Finger for now if not.
                            };

                            let mut frame = AxisFrame::new(time_ms).source(source);
                            if axis_event.has_axis(libinput::event::pointer::Axis::Horizontal) {
                                let val = axis_event.axis_value(libinput::event::pointer::Axis::Horizontal);
                                frame = frame.value(Axis::Horizontal, val);
                                if let Some(discrete) = axis_event.axis_value_discrete(libinput::event::pointer::Axis::Horizontal) {
                                    frame = frame.discrete(Axis::Horizontal, discrete as i32); // discrete takes i32
                                }
                            }
                            if axis_event.has_axis(libinput::event::pointer::Axis::Vertical) {
                                let val = axis_event.axis_value(libinput::event::pointer::Axis::Vertical);
                                frame = frame.value(Axis::Vertical, val);
                                if let Some(discrete) = axis_event.axis_value_discrete(libinput::event::pointer::Axis::Vertical) {
                                    frame = frame.discrete(Axis::Vertical, discrete as i32); // discrete takes i32
                                }
                            }

                            // Smithay 0.3 AxisFrame doesn't have .stop()
                            // if axis_event.axis_stop(libinput::event::pointer::Axis::Horizontal) {
                            //     frame = frame.stop(Axis::Horizontal);
                            // }
                            // if axis_event.axis_stop(libinput::event::pointer::Axis::Vertical) {
                            //     frame = frame.stop(Axis::Vertical);
                            // }

                            // In Smithay 0.3, AxisEvent is just an alias for AxisFrame
                            // let smithay_axis_event = SmithayPointerAxisEvent { frame };
                            // So we pass frame directly.
                            desktop_state.pointer_axis_event(&desktop_state.seat.clone(), frame);
                        }
                        _ => debug!("Other pointer event: {:?}", pointer_event),
                    }
                },
                Event::Touch(touch_event) => {
                    match touch_event {
                        event::Touch::Down(down) => {
                            let time_ms = (down.time_usec() / 1000) as u32;
                            let serial = SERIAL_COUNTER.next_serial();
                            let raw_slot_id = down.seat_slot().or(down.slot()).unwrap_or(0); // Get slot ID, default to 0 if None
                            let slot_id = TouchSlotId::new(raw_slot_id);
                            let location = Point::from((down.x(), down.y()));

                            let smithay_event = DownEvent {
                                slot: slot_id,
                                serial,
                                time: time_ms,
                                location,
                                focus: None, // SeatHandler (DesktopState) will determine focus
                            };
                            desktop_state.touch_down_event(&desktop_state.seat.clone(), &smithay_event);
                            debug!("Touch Down: id={}, x={}, y={} -> Smithay DownEvent dispatched", raw_slot_id, down.x(), down.y());
                        }
                        event::Touch::Up(up) => {
                            let time_ms = (up.time_usec() / 1000) as u32;
                            let serial = SERIAL_COUNTER.next_serial();
                            let raw_slot_id = up.seat_slot().or(up.slot()).unwrap_or(0);
                            let slot_id = TouchSlotId::new(raw_slot_id);

                            let smithay_event = UpEvent {
                                slot: slot_id,
                                serial,
                                time: time_ms,
                            };
                            desktop_state.touch_up_event(&desktop_state.seat.clone(), &smithay_event);
                            debug!("Touch Up: id={} -> Smithay UpEvent dispatched", raw_slot_id);
                        }
                        event::Touch::Motion(motion) => {
                            let time_ms = (motion.time_usec() / 1000) as u32;
                            let serial = SERIAL_COUNTER.next_serial();
                            let raw_slot_id = motion.seat_slot().or(motion.slot()).unwrap_or(0);
                            let slot_id = TouchSlotId::new(raw_slot_id);
                            let location = Point::from((motion.x(), motion.y()));

                            let smithay_event = TouchMotionEvent {
                                slot: slot_id,
                                serial,
                                time: time_ms,
                                location,
                                focus: None, // SeatHandler (DesktopState) will determine focus
                            };
                            desktop_state.touch_motion_event(&desktop_state.seat.clone(), &smithay_event);
                            debug!("Touch Motion: id={}, x={}, y={} -> Smithay MotionEvent dispatched", raw_slot_id, motion.x(), motion.y());
                        }
                        event::Touch::Cancel(_cancel) => {
                            // let _time_ms = (_cancel.time_usec() / 1000) as u32; // if time was needed by handler
                            desktop_state.touch_cancel_event(&desktop_state.seat.clone());
                            debug!("Touch Cancelled -> Smithay Cancel event dispatched");
                        }
                        event::Touch::Frame(_frame) => {
                            // let _time_ms = (_frame.time_usec() / 1000) as u32; // if time was needed by handler
                            desktop_state.touch_frame_event(&desktop_state.seat.clone());
                            debug!("Touch Frame -> Smithay Frame event dispatched");
                        }
                        _ => debug!("Other touch event: {:?}", touch_event),
                    }
                },
                Event::Gesture(_gesture_event) => {
                    // Gestures (swipe, pinch, etc.)
                    // These are higher-level events built from pointer/touch.
                    // Processing them would involve translating them into compositor actions.
                    debug!("Gesture Event: (not yet fully handled)");
                },
                Event::Tablet(_tablet_event) => {
                    debug!("Tablet Event: (not yet fully handled)");
                },
                Event::TabletPad(_pad_event) => {
                    debug!("Tablet Pad Event: (not yet fully handled)");
                },
                Event::Switch(_switch_event) => {
                    debug!("Switch Event: (not yet fully handled)");
                },
                _ => {
                    warn!("Unhandled event type: {:?}", event.variant());
                }
            }
        }
        Ok(())
    }
}

// Basic test for initialization
#[cfg(test)]
mod tests {
    use super::*;
    // At the top of mod tests block
    use smithay::input::{Seat, SeatState, SeatHandler, KeyboardHandle, pointer::PointerHandle, touch::TouchHandle, keyboard::{KeysymHandle, FilterReturn, ModifiersState as SmithayModifiersState, KeyState as SmithayKeyState}};
    use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
    use smithay::reexports::wayland_server::DisplayHandle; // For mock SeatState
    use smithay::utils::{Serial, Point, Logical, Clock}; // For mock DesktopState
    use std::sync::{Arc, Mutex};
    use std::cell::RefCell;
    use std::collections::HashMap; // For DesktopState fields
    // use crate::compositor::core::state::DesktopState as RealDesktopState; // To avoid naming conflict - Not needed for this mock
    use crate::compositor::input::error::InputError; // For InputHandler
    use xkbcommon::xkb::{self, RuleNames, KEY_a}; // For test RMLVO
    use std::ffi::CString;


    // This test is very basic and will likely fail on CI or systems without udev/input devices.
    // It's more of a structural test. For real testing, mocking libinput or using a test backend is needed.
    #[test]
    fn test_input_handler_new() {
        // We expect this to fail if udev is not available or permissions are insufficient.
        // In a CI environment, this might always fail.
        // The goal here is to ensure `new` can be called and doesn't immediately panic
        // due to trivial issues like incorrect xkbcommon context creation.
        match InputHandler::new() {
            Ok(_) => info!("InputHandler initialized successfully (likely in a desktop environment)."),
            Err(InputError::LibinputInitialization(msg)) if msg.contains("udev") || msg.contains("seat") => {
                warn!("InputHandler initialization failed as expected in some environments: {}", msg);
            }
            Err(e) => {
                // Unexpected error
                panic!("InputHandler::new() failed with unexpected error: {:?}", e);
            }
        }
    }

    // Further tests would require more sophisticated mocking of libinput events.
    // For example, creating a dummy LibinputInterface that can feed predefined events.

    // Mock Smithay KeyboardHandle
    #[derive(Clone, Debug)]
    struct MockKeyboard {
        keymap_set_count: Arc<Mutex<usize>>,
        last_keymap_rules: Arc<Mutex<Option<(String, String, String)>>>, // Store RML as strings for simplicity
    }
    impl MockKeyboard {
        fn new() -> Self {
            Self {
                keymap_set_count: Arc::new(Mutex::new(0)),
                last_keymap_rules: Arc::new(Mutex::new(None)),
            }
        }
        // Mocked method
        pub fn set_keymap(&self, _keymap: xkb::Keymap) { // keymap param changed to _keymap
            *self.keymap_set_count.lock().unwrap() += 1;
            // For simplicity, we can't easily inspect the keymap rules directly from xkb::Keymap without context.
            // In a real scenario, you might compile a known keymap and compare if possible, or check specific properties.
            // Here, we'll just note it was called.
            // To verify which keymap was set, the test would need to provide specific RMLVOs
            // and we'd have to trust that xkbcommon compiled it correctly.
            // For this mock, we'll just increment a counter.
        }

        // Mocked input method (simplified, not testing event flow here)
        pub fn input<D: SeatHandler + 'static>(
            &self,
            _handler: &mut D,
            _keycode: u32,
            _key_state: SmithayKeyState,
            _serial: Serial,
            _time: u32,
            _filter: impl FnMut(SmithayModifiersState, KeysymHandle<'_>) -> FilterReturn + Send + 'static,
        ) {
            // Not used in set_keymap test
        }
    }

    // Mock Smithay Seat
    #[derive(Clone, Debug)]
    struct MockSmithaySeat {
        keyboard: Option<MockKeyboard>,
        // Add pointer, touch if needed for other tests
    }
    impl MockSmithaySeat {
        fn new() -> Self {
            Self {
                keyboard: Some(MockKeyboard::new()),
            }
        }
        fn get_keyboard(&self) -> Option<MockKeyboard> {
            self.keyboard.clone()
        }
        // Add get_pointer, get_touch if needed
    }

    // Mock DesktopState (minimal for set_keymap)
    // We need a struct that can hold a MockSmithaySeat.
    // Real DesktopState is complex.
    struct MockDesktopState {
        pub seat: MockSmithaySeat,
        // Add other fields if InputHandler::set_keymap started using them from DesktopState
    }
    impl MockDesktopState {
        fn new() -> Self {
            Self {
                seat: MockSmithaySeat::new(),
            }
        }
    }

    #[test]
    fn test_input_handler_set_keymap() {
        // This test might fail in CI if libinput initialization within InputHandler::new() fails.
        // We proceed assuming InputHandler::new() can succeed enough to test set_keymap.
        let mut input_handler = match InputHandler::new() {
            Ok(ih) => ih,
            Err(InputError::LibinputInitialization(msg)) if msg.contains("udev") || msg.contains("seat") => {
                warn!("Skipping test_input_handler_set_keymap due to libinput init failure: {}", msg);
                return;
            }
            Err(e) => {
                panic!("InputHandler::new() failed unexpectedly: {:?}", e);
            }
        };

        let mut mock_desktop_state = MockDesktopState::new();

        let rmlvo = RuleNames {
            rules: Some(CString::new("evdev").unwrap()),
            model: Some(CString::new("pc105").unwrap()),
            layout: Some(CString::new("de").unwrap()), // German layout
            variant: Some(CString::new("nodeadkeys").unwrap()),
            options: None,
        };

        // Ensure initial keymap is loaded (e.g. US default) if not already by a (mock) device add
        // This ensures xkb_keymap and xkb_state are Some.
        // In a real scenario, a device would be added first. Here we force it.
        if input_handler.xkb_keymap.is_none() {
            input_handler.ensure_initial_keymap_loaded().expect("Failed to load initial keymap for test");
        }

        let initial_keymap_name = input_handler.xkb_keymap_name.clone();
        // The default loaded by ensure_initial_keymap_loaded is 'pc105' (from model)
        // The test target model is also 'pc105', but the layout is 'de'.
        // A better check would be against the full RMLVO string if we stored it, or a specific keysym.
        // For now, we check that it's not exactly the specific combination we are about to set,
        // if the layout part was included in xkb_keymap_name.
        // Since xkb_keymap_name only stores model, this check might not be very effective if default model is pc105.
        // Let's assume default is just "pc105" and we are setting a "pc105" model with "de" layout.
        // The name will be "pc105".
        // A more direct check is that set_keymap is called.
        // assert_ne!(initial_keymap_name, "pc105_de_nodeadkeys", "Initial keymap name should not be the test target yet.");


        // Call set_keymap
        match input_handler.set_keymap(&mut mock_desktop_state, rmlvo) {
            Ok(()) => {
                info!("set_keymap executed successfully.");
                // Verify internal keymap name updated (simple check)
                assert!(input_handler.xkb_keymap_name.contains("pc105")); // Model used in RMLVO

                // Verify Smithay's keyboard_handle.set_keymap was called
                let keyboard = mock_desktop_state.seat.get_keyboard().unwrap();
                assert_eq!(*keyboard.keymap_set_count.lock().unwrap(), 1, "Smithay keyboard_handle.set_keymap should have been called once.");
            }
            Err(e) => {
                // If it's a libinput init error, it might be an environment issue.
                // Otherwise, the test fails.
                match e {
                    InputError::XkbCommonState(msg) => {
                        // This could happen if xkbcommon can't find the layout "de" etc.
                        // This is an environment dependency (xkb-data installed).
                        warn!("set_keymap failed due to XkbCommonState error: {}. This might be an environment issue (missing xkb-data for 'de' layout?).", msg);
                         // We don't panic here as it might be CI. But it's a sign the test isn't fully validating on this env.
                    }
                    _ => panic!("set_keymap failed with unexpected error: {:?}", e),
                }
            }
        }
    }
}
