use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;

use libinput::{Libinput, LibinputInterface, Event, EventKind, event, device::Device};
use xkbcommon::xkb;
use tracing::{info, debug, error, warn};

// Import own error and data types
use crate::compositor::input::error::InputError;
use crate::compositor::input::data_types::{
    KeyboardState, PointerState, TouchState, InputEvent,
    KeyboardEventInfo, KeyState, ModifiersState,
    PointerMotionInfo, PointerButtonInfo, ButtonState, PointerAxisInfo, AxisMovement,
    TouchDownInfo, TouchUpInfo, TouchMotionInfo, Seat,
};

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
    xkb_keymap_name: String, // e.g., "pc105"
    xkb_state: Option<xkb::State>, // Will be initialized per keyboard device
    // seat: SmithaySeat<CompositorState>, // Actual Smithay seat
    // For now, using our own Seat definition
    pub seat: Seat, // Public to allow access to its state if needed from outside
    // We might need to manage multiple keyboards, pointers etc. per seat or have a global one for now.
}

impl InputHandler {
    pub fn new(seat_name: String) -> Result<Self, InputError> {
        info!("Initializing InputHandler for seat: {}", seat_name);

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

        // Keymap configuration (can be made configurable later)
        // These are common defaults.
        let rmlvo = xkb::RuleNames {
            rules: Some(std::ffi::CString::new("evdev").unwrap()),
            model: Some(std::ffi::CString::new("pc105").unwrap()),
            layout: Some(std::ffi::CString::new("us").unwrap()),
            variant: Some(std::ffi::CString::new("").unwrap()), // Empty for default variant
            options: None,
        };

        // We will compile a keymap when a keyboard is added.
        // The xkb_state will also be initialized then.

        Ok(InputHandler {
            libinput,
            xkb_context,
            xkb_keymap_name: "pc105".to_string(), // Store for potential re-use or logging
            xkb_state: None,
            seat: Seat::new(seat_name),
        })
    }

    /// Initializes or updates the XKB state for a given keyboard device.
    /// This should be called when a keyboard is detected or its configuration changes.
    fn ensure_xkb_state_for_device(&mut self, _device: &Device) -> Result<(), InputError> {
        // For now, we assume a single keyboard setup and initialize xkb_state once.
        // In a multi-keyboard setup, this would be more complex, possibly storing
        // xkb::State per device or ensuring the "focused" keyboard's state is active.
        if self.xkb_state.is_none() {
            info!("Initializing xkbcommon keymap and state.");
            // These are common defaults.
            let rmlvo = xkb::RuleNames {
                rules: Some(std::ffi::CString::new("evdev").unwrap()), // rules typically 'evdev' for libinput
                model: Some(std::ffi::CString::new("pc105").unwrap()), // model, e.g. "pc105", "apple_laptop"
                layout: Some(std::ffi::CString::new("us").unwrap()),   // layout, e.g. "us", "de"
                variant: Some(std::ffi::CString::new("").unwrap()), // variant, e.g. "dvorak", "" for default
                options: None, // options, e.g. "caps:escape"
            };

            let keymap = xkb::Keymap::new_from_names(
                &self.xkb_context,
                &rmlvo,
                xkb::KEYMAP_COMPILE_NO_FLAGS,
            )
            .ok_or_else(|| InputError::XkbCommonState("Failed to compile xkb keymap".to_string()))?;

            info!("xkb keymap compiled successfully. Model: {:?}, Layout: {:?}, Variant: {:?}",
                  rmlvo.model, rmlvo.layout, rmlvo.variant);

            let state = xkb::State::new(&keymap);
            self.xkb_state = Some(state);
            info!("xkb state initialized.");
        }
        Ok(())
    }


    /// Dispatches pending libinput events and translates them into `InputEvent`s.
    /// These events would then be further processed or sent to the compositor (e.g., Smithay).
    pub fn dispatch_events(&mut self /*, smithay_seat_handler: &mut SmithaySeatHandler */) -> Result<Vec<InputEvent>, InputError> {
        self.libinput.dispatch().map_err(|e| {
            error!("Error dispatching libinput events: {}", e);
            InputError::EventProcessing(format!("Libinput dispatch failed: {}", e))
        })?;

        let mut internal_events = Vec::new();

        for event in &mut self.libinput {
            let time_usec = event.time_usec(); // Common to most events

            match event {
                Event::DeviceAdded(device_added_event) => {
                    let device = device_added_event.device();
                    info!("Device added: {:?} (type: {:?}, seat: {:?})",
                          device.name(), device.devnode(), device.seat_logical_name());
                    // Here you would typically check device capabilities (keyboard, pointer, touch)
                    // and configure them (e.g., set up xkb state for keyboards).
                    if device.has_capability(libinput::DeviceCapability::Keyboard) {
                        if let Err(e) = self.ensure_xkb_state_for_device(&device) {
                            error!("Failed to ensure xkb_state for new keyboard: {}", e);
                            // Decide if this is a critical error
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
                    if let Some(ref mut xkb_state) = self.xkb_state {
                        let key_code = kb_event.key(); // This is the raw scancode
                        let key_state = match kb_event.key_state() {
                            event::keyboard::KeyState::Pressed => KeyState::Pressed,
                            event::keyboard::KeyState::Released => KeyState::Released,
                        };

                        // Update xkb state for modifiers, LEDs etc.
                        // The order of operations for xkb_state updates can be subtle.
                        // Typically, for a press, you get the keysym *before* updating for that press.
                        // For a release, you might update then get keysym, or get keysym then update.
                        // xkbcommon-rs documentation or examples should clarify this.

                        let keysym = xkb_state.key_get_one_sym(key_code + 8); // libinput scancodes are often offset by 8 for xkb
                        let utf8 = xkb_state.key_get_utf8(key_code + 8);

                        // Update the state of modifiers in xkb_state based on the key event
                        // This also updates locked modifiers like CapsLock.
                        xkb_state.update_key(key_code + 8, match key_state {
                            KeyState::Pressed => xkb::KeyDirection::Down,
                            KeyState::Released => xkb::KeyDirection::Up,
                        });

                        let mods = ModifiersState {
                            ctrl: xkb_state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE),
                            alt: xkb_state.mod_name_is_active(&xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE),
                            shift: xkb_state.mod_name_is_active(&xkb::MOD_NAME_SHIFT, xkb::STATE_MODS_EFFECTIVE),
                            caps_lock: xkb_state.mod_name_is_active(&xkb::MOD_NAME_CAPS, xkb::STATE_MODS_EFFECTIVE),
                            logo: xkb_state.mod_name_is_active(&xkb::MOD_NAME_LOGO, xkb::STATE_MODS_EFFECTIVE),
                        };
                        self.seat.keyboard_state.modifiers = mods.clone();

                        debug!(
                            "Keyboard Event: key_code={}, keysym={:?} ({}), utf8='{}', state={:?}, mods={:?}",
                            key_code, keysym, xkb::keysym_get_name(keysym), utf8, key_state, mods
                        );

                        internal_events.push(InputEvent::Keyboard(KeyboardEventInfo {
                            time_usec,
                            key_code,
                            key_state,
                            keysym,
                            utf8,
                            modifiers: mods,
                        }));

                        // Example of how one might integrate with Smithay's keyboard handler
                        // if let Some(keyboard) = self.seat.get_keyboard() {
                        //     keyboard.input(
                        //         compositor_state, // The global compositor state
                        //         key_code,
                        //         smithay_key_state, // Smithay's KeyState enum
                        //         serial, // Event serial
                        //         time, // Event time in ms
                        //         |modifiers, handle| { /* callback for smithay to update its internal state and notify clients */ }
                        //     );
                        // }

                    } else {
                        warn!("Keyboard event received but xkb_state is not initialized. Device might not have been added correctly or is not a keyboard.");
                    }
                },
                Event::Pointer(pointer_event) => {
                    match pointer_event {
                        event::Pointer::Motion(motion) => {
                            let (dx, dy) = (motion.dx(), motion.dy());
                            self.seat.pointer_state.position_relative = (dx, dy);
                            // Absolute position update would depend on how you track it.
                            // If your compositor maintains an absolute position, update it here.
                            // self.seat.pointer_state.position_absolute.0 += dx;
                            // self.seat.pointer_state.position_absolute.1 += dy;
                            debug!("Pointer Motion: dx={}, dy={}", dx, dy);
                            internal_events.push(InputEvent::PointerMotion(PointerMotionInfo {
                                time_usec, dx, dy
                            }));
                        }
                        event::Pointer::Button(button_event) => {
                            let button_code = button_event.button(); // e.g., BTN_LEFT (0x110 or 272)
                            let button_state = match button_event.button_state() {
                                event::pointer::ButtonState::Pressed => {
                                    self.seat.pointer_state.pressed_buttons.insert(button_code);
                                    ButtonState::Pressed
                                }
                                event::pointer::ButtonState::Released => {
                                    self.seat.pointer_state.pressed_buttons.remove(&button_code);
                                    ButtonState::Released
                                }
                            };
                            debug!("Pointer Button: button_code={}, state={:?}", button_code, button_state);
                            internal_events.push(InputEvent::PointerButton(PointerButtonInfo {
                                time_usec, button_code, button_state
                            }));
                        }
                        event::Pointer::Axis(axis_event) => {
                            let mut horizontal = AxisMovement::default();
                            let mut vertical = AxisMovement::default();

                            if axis_event.has_axis(event::pointer::Axis::Horizontal) {
                                horizontal.continuous_value = axis_event.axis_value(event::pointer::Axis::Horizontal);
                                if let Some(discrete) = axis_event.axis_value_discrete(event::pointer::Axis::Horizontal) {
                                    horizontal.discrete_steps = discrete as i32;
                                }
                            }
                            if axis_event.has_axis(event::pointer::Axis::Vertical) {
                                vertical.continuous_value = axis_event.axis_value(event::pointer::Axis::Vertical);
                                if let Some(discrete) = axis_event.axis_value_discrete(event::pointer::Axis::Vertical) {
                                    vertical.discrete_steps = discrete as i32;
                                }
                            }
                            debug!("Pointer Axis: horizontal={:?}, vertical={:?}", horizontal, vertical);
                            internal_events.push(InputEvent::PointerAxis(PointerAxisInfo {
                                time_usec, horizontal, vertical
                            }));
                        }
                        _ => debug!("Other pointer event: {:?}", pointer_event),
                    }
                },
                Event::Touch(touch_event) => {
                    match touch_event {
                        event::Touch::Down(down) => {
                            let (id, x, y) = (down.seat_slot().unwrap_or_else(|| down.slot()), down.x(), down.y());
                            debug!("Touch Down: id={}, x={}, y={}", id, x, y);
                            let point = crate::compositor::input::data_types::TouchPoint { id, position: (x,y) };
                            self.seat.touch_state.active_points.insert(id, point.clone());
                            internal_events.push(InputEvent::TouchDown(TouchDownInfo {
                                time_usec, slot_id: id, x, y
                            }));
                        }
                        event::Touch::Up(up) => {
                            let id = up.seat_slot().unwrap_or_else(|| up.slot());
                            debug!("Touch Up: id={}", id);
                            self.seat.touch_state.active_points.remove(&id);
                            internal_events.push(InputEvent::TouchUp(TouchUpInfo {
                                time_usec, slot_id: id
                            }));
                        }
                        event::Touch::Motion(motion) => {
                            let (id, x, y) = (motion.seat_slot().unwrap_or_else(|| motion.slot()), motion.x(), motion.y());
                            debug!("Touch Motion: id={}, x={}, y={}", id, x, y);
                            if let Some(point) = self.seat.touch_state.active_points.get_mut(&id) {
                                point.position = (x,y);
                            }
                            internal_events.push(InputEvent::TouchMotion(TouchMotionInfo {
                                time_usec, slot_id: id, x, y
                            }));
                        }
                        event::Touch::Cancel(_cancel) => {
                            // Treat cancel as removing all active points for now, or handle specific IDs if provided
                            debug!("Touch Cancelled");
                            self.seat.touch_state.active_points.clear();
                            // It might be better to send TouchUp for each active point.
                        }
                        event::Touch::Frame(_frame) => {
                            // Frame events signify end of a batch of touch updates.
                            // Useful for synchronization if needed.
                            debug!("Touch Frame");
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
        Ok(internal_events)
    }
}

// Basic test for initialization
#[cfg(test)]
mod tests {
    use super::*;

    // This test is very basic and will likely fail on CI or systems without udev/input devices.
    // It's more of a structural test. For real testing, mocking libinput or using a test backend is needed.
    #[test]
    fn test_input_handler_new() {
        // We expect this to fail if udev is not available or permissions are insufficient.
        // In a CI environment, this might always fail.
        // The goal here is to ensure `new` can be called and doesn't immediately panic
        // due to trivial issues like incorrect xkbcommon context creation.
        match InputHandler::new("seat_test".to_string()) {
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
}
