// src/input/focus.rs

use std::collections::{VecDeque, HashSet, HashMap};
use crate::input::keyboard::{KeyState, ModifiersState, Keyboard};
use crate::input::pointer::{Pointer, ButtonState as PointerButtonState};
use crate::input::touch::Touch;
use crate::input::config::{InputConfig, PointerConfig, KeyboardConfig};
use crate::wayland_server_module_placeholder::{WaylandServerHandle, SurfaceManagerHandle, StubbedSurfaceInfo, Rect};

pub type SurfaceId = u32;

// --- Event Structs ---
#[derive(Debug, Clone)]
pub struct ProcessedKeyEvent {
    pub keysym: u32,
    pub state: KeyState,
    pub modifiers: ModifiersState,
    pub time: u32,
    pub raw_keycode: u32,
    pub serial: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessedPointerMotionEvent {
    pub abs_x: f64,
    pub abs_y: f64,
    pub rel_dx: f64,
    pub rel_dy: f64,
    pub time: u32,
    pub serial: u32,
}

pub use crate::input::pointer::ButtonState;

#[derive(Debug, Clone, Copy)]
pub struct ProcessedPointerButtonEvent {
    pub button_code: u32,
    pub state: ButtonState,
    pub abs_x: f64,
    pub abs_y: f64,
    pub time: u32,
    pub serial: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollSource {
    Wheel,
    Finger,
    Continuous,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessedPointerScrollEvent {
    pub delta_x: f64,
    pub delta_y: f64,
    pub source: ScrollSource,
    pub abs_x: f64,
    pub abs_y: f64,
    pub time: u32,
    pub serial: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessedTouchDownEvent {
    pub id: i32,
    pub x: f64,
    pub y: f64,
    pub time: u32,
    pub serial: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessedTouchMotionEvent {
    pub id: i32,
    pub x: f64,
    pub y: f64,
    pub time: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessedTouchUpEvent {
    pub id: i32,
    pub x: f64,
    pub y: f64,
    pub time: u32,
    pub serial: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessedTouchFrameEvent {
    pub time: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct GrabRequest {
    pub surface_id: SurfaceId,
    pub serial: u32,
}

pub struct FocusManager {
    keyboard_focus: Option<SurfaceId>,
    pointer_focus: Option<SurfaceId>,
    touch_focus: HashMap<i32, SurfaceId>,
    active_grabs: Vec<GrabRequest>,
    focus_history: VecDeque<SurfaceId>,

    wayland_server: WaylandServerHandle,
    surface_manager: SurfaceManagerHandle,

    keyboard_handler: Keyboard,
    pointer_handler: Pointer,
    touch_handler: Touch,

    // Storing current raw key state for wl_keyboard.enter
    current_pressed_keys: HashSet<u32>,
    current_modifiers_state: ModifiersState,
    next_serial: u32,
}

impl FocusManager {
    pub fn new(
        wayland_server: WaylandServerHandle,
        surface_manager: SurfaceManagerHandle,
        config: &InputConfig,
    ) -> Self {
        tracing::info!("FocusManager (Stub): Initializing (refactored)...");

        let keyboard_config = config.get_effective_keyboard_config("Stubbed Keyboard");
        let pointer_config = config.get_effective_pointer_config("Stubbed Mouse")
            .unwrap_or_else(|| PointerConfig {
                acceleration_factor: Some(0.0),
                sensitivity: Some(1.0),
                acceleration_curve: None,
                button_mapping: None,
            });

        let keyboard_handler = Keyboard::new(keyboard_config);
        let pointer_handler = Pointer::new(pointer_config);
        let touch_handler = Touch::new();

        Self {
            keyboard_focus: None,
            pointer_focus: None,
            touch_focus: HashMap::new(),
            active_grabs: Vec::new(),
            focus_history: VecDeque::with_capacity(10),
            wayland_server,
            surface_manager,
            keyboard_handler,
            pointer_handler,
            touch_handler,
            current_pressed_keys: HashSet::new(),
            current_modifiers_state: ModifiersState::default(),
            next_serial: 0,
        }
    }

    fn get_next_serial(&mut self) -> u32 {
        self.next_serial = self.next_serial.wrapping_add(1);
        tracing::trace!("FocusManager: Next serial: {}", self.next_serial);
        self.next_serial
    }

    // --- Raw Event Handlers ---
    pub fn handle_raw_keyboard_input(&mut self, raw_keycode: u32, state: KeyState, time: u32) {
        tracing::debug!("FocusManager: Handling raw keyboard input: keycode={}, state={:?}, time={}", raw_keycode, state, time);
        // Update internal raw key state tracking
        match state {
            KeyState::Pressed => self.current_pressed_keys.insert(raw_keycode),
            KeyState::Released => self.current_pressed_keys.remove(&raw_keycode),
        };

        if let Some(mut processed_event) = self.keyboard_handler.handle_key_event(raw_keycode, state, time) {
            processed_event.serial = self.get_next_serial();
            // Modifier state is part of ProcessedKeyEvent, update self.current_modifiers_state from it
            self.current_modifiers_state = processed_event.modifiers;
            self.handle_processed_key_event(processed_event);
        }
        // Handle discrete modifier updates from keyboard_handler.update_modifier_state()
        // This is called internally by keyboard_handler.handle_key_event and updates its internal state.
        // If a modifier-only event needs to be sent to clients (e.g. only Ctrl pressed),
        // keyboard_handler.update_modifier_state() returning Some(ModifiersState) is the trigger.
        if let Some(new_mods) = self.keyboard_handler.update_modifier_state() {
             self.current_modifiers_state = new_mods;
             self.handle_processed_modifier_update(new_mods, self.get_next_serial());
        }
    }

    pub fn handle_raw_pointer_motion(&mut self, dx: f64, dy: f64, time: u32) {
        tracing::debug!("FocusManager: Handling raw pointer motion: dx={}, dy={}, time={}", dx, dy, time);
        if let Some(mut processed_event) = self.pointer_handler.handle_motion_event(dx, dy, time) {
            processed_event.serial = self.get_next_serial();
            self.handle_processed_pointer_motion(processed_event);
        }
    }

    pub fn handle_raw_pointer_button(&mut self, raw_button_code: u32, state: ButtonState, time: u32) {
        tracing::debug!("FocusManager: Handling raw pointer button: button={}, state={:?}, time={}", raw_button_code, state, time);
        if let Some(mut processed_event) = self.pointer_handler.handle_button_event(raw_button_code, state, time) {
            processed_event.serial = self.get_next_serial();
            self.handle_processed_pointer_button(processed_event);
        }
    }

    pub fn handle_raw_pointer_scroll(&mut self, dx_discrete: f64, dy_discrete: f64, dx_continuous: f64, dy_continuous: f64, source: ScrollSource, time: u32) {
        tracing::debug!("FocusManager: Handling raw pointer scroll: source={:?}, time={}", source, time);
        if let Some(mut processed_event) = self.pointer_handler.handle_scroll_event(dx_discrete, dy_discrete, dx_continuous, dy_continuous, source, time) {
            processed_event.serial = self.get_next_serial();
            self.handle_processed_pointer_scroll(processed_event);
        }
    }

    pub fn handle_raw_touch_down(&mut self, id: i32, x: f64, y: f64, time: u32) {
        tracing::debug!("FocusManager: Handling raw touch down: id={}, x={}, y={}, time={}", id, x, y, time);
        if let Some(mut processed_event) = self.touch_handler.handle_touch_down_event(id, x, y, time) {
            processed_event.serial = self.get_next_serial();
            self.handle_processed_touch_down(processed_event);
        }
    }

    pub fn handle_raw_touch_motion(&mut self, id: i32, x: f64, y: f64, time: u32) {
        tracing::debug!("FocusManager: Handling raw touch motion: id={}, x={}, y={}, time={}", id, x, y, time);
        if let Some(processed_event) = self.touch_handler.handle_touch_motion_event(id, x, y, time) {
            self.handle_processed_touch_motion(processed_event);
        }
    }

    pub fn handle_raw_touch_up(&mut self, id: i32, time: u32) {
        tracing::debug!("FocusManager: Handling raw touch up: id={}, time={}", id, time);
        if let Some(mut processed_event) = self.touch_handler.handle_touch_up_event(id, time) {
            processed_event.serial = self.get_next_serial();
            self.handle_processed_touch_up(processed_event);
        }
    }

    pub fn handle_raw_touch_frame(&mut self, time: u32) {
        tracing::debug!("FocusManager: Handling raw touch frame: time={}", time);
        if let Some(processed_event) = self.touch_handler.handle_touch_frame_event(time) {
            self.handle_processed_touch_frame(processed_event);
        }
    }

    // --- Surface Focus Calculation ---
    fn calculate_surface_at_pointer(&self, x: f64, y: f64) -> Option<SurfaceId> {
        tracing::debug!("FocusManager: Calculating surface at pointer: x={}, y={}", x, y);
        let surfaces = self.surface_manager.get_surfaces_at_coords(x, y);
        if let Some(top_surface) = surfaces.first() {
            tracing::trace!("FocusManager: Surface found at ({}, {}): ID {}", x, y, top_surface.id);
            return Some(top_surface.id);
        }
        tracing::trace!("FocusManager: No surface found at ({}, {}).", x, y);
        None
    }

    // --- Focus Change Logic ---
    fn set_keyboard_focus(&mut self, new_focus_id: Option<SurfaceId>, serial: u32) {
        if self.keyboard_focus != new_focus_id {
            if let Some(old_focus_id) = self.keyboard_focus {
                tracing::info!("FocusManager: Keyboard focus leaving Surface ID: {}", old_focus_id);
                self.wayland_server.send_keyboard_leave(old_focus_id, serial);
            }
            self.keyboard_focus = new_focus_id;
            if let Some(id) = new_focus_id {
                tracing::info!("FocusManager: Keyboard focus entering Surface ID: {}", id);
                // current_pressed_keys and current_modifiers_state are updated by handle_raw_keyboard_input
                self.wayland_server.send_keyboard_enter(id, serial, &self.current_pressed_keys, &self.current_modifiers_state);
                if self.focus_history.contains(&id) { self.focus_history.retain(|&x| x != id); }
                self.focus_history.push_front(id);
                if self.focus_history.len() > 10 { self.focus_history.pop_back(); }
                tracing::debug!("FocusManager: Focus history: {:?}", self.focus_history);
            }
        } else {
            tracing::trace!("FocusManager: set_keyboard_focus called with current focus ID {:?}, no change.", new_focus_id);
        }
    }

    // --- Input Grabbing ---
    fn request_pointer_grab(&mut self, surface_id: SurfaceId, serial: u32) {
        tracing::info!("FocusManager: Pointer grab requested by Surface ID: {} with serial {}", surface_id, serial);
        self.active_grabs.clear();
        self.active_grabs.push(GrabRequest { surface_id, serial });
    }

    fn release_pointer_grab(&mut self, surface_id: SurfaceId, serial: u32) {
        tracing::info!("FocusManager: Pointer grab released by Surface ID: {} with serial {}", surface_id, serial);
        self.active_grabs.retain(|grab| grab.surface_id != surface_id || grab.serial != serial);
    }

    // --- Processed Event Handlers ---
    fn handle_processed_key_event(&mut self, event: ProcessedKeyEvent) {
        tracing::debug!("FocusManager: Handling processed key event: {:?}", event);
        // self.current_modifiers_state and self.current_pressed_keys already updated in raw handler
        let target_surface = if let Some(grab) = self.active_grabs.first() {
            Some(grab.surface_id)
        } else {
            self.keyboard_focus
        };

        if let Some(surface_id) = target_surface {
            tracing::trace!("FocusManager: Forwarding ProcessedKeyEvent to Surface ID: {}", surface_id);
            self.wayland_server.send_keyboard_event_to_surface(surface_id, &event, event.serial);
        } else {
            tracing::debug!("FocusManager: No keyboard focus or active grab; ProcessedKeyEvent dropped: {:?}", event);
        }
    }

    fn handle_processed_modifier_update(&mut self, modifiers: ModifiersState, serial: u32) {
        tracing::debug!("FocusManager: Handling processed modifier update: {:?}, serial: {}", modifiers, serial);
        self.current_modifiers_state = modifiers;
        if let Some(surface_id) = self.keyboard_focus {
            tracing::trace!("FocusManager: Forwarding modifier update to Surface ID: {}", surface_id);
            self.wayland_server.send_keyboard_modifiers(surface_id, &modifiers, serial);
        }
    }

    fn handle_processed_pointer_motion(&mut self, event: ProcessedPointerMotionEvent) {
        tracing::debug!("FocusManager: Handling processed pointer motion: {:?}", event);
        let current_grab = self.active_grabs.first().copied();
        if let Some(grab) = current_grab {
            tracing::trace!("FocusManager: Pointer motion grabbed by Surface ID: {}", grab.surface_id);
            self.wayland_server.send_pointer_motion(grab.surface_id, &event);
        } else {
            let new_focus_candidate = self.calculate_surface_at_pointer(event.abs_x, event.abs_y);
            if self.pointer_focus != new_focus_candidate {
                if let Some(old_focus_id) = self.pointer_focus {
                    tracing::info!("FocusManager: Pointer leaving Surface ID: {}", old_focus_id);
                    self.wayland_server.send_pointer_leave(old_focus_id, event.serial);
                }
                self.pointer_focus = new_focus_candidate;
                if let Some(new_focus_id) = self.pointer_focus {
                    tracing::info!("FocusManager: Pointer entering Surface ID: {}", new_focus_id);
                    self.wayland_server.send_pointer_enter(new_focus_id, event.abs_x, event.abs_y, event.serial);
                }
            }
            if let Some(focus_id) = self.pointer_focus {
                tracing::trace!("FocusManager: Forwarding pointer motion to Surface ID: {}", focus_id);
                self.wayland_server.send_pointer_motion(focus_id, &event);
            }
        }
        let frame_target = if let Some(grab) = current_grab { Some(grab.surface_id) } else { self.pointer_focus };
        if let Some(target_id) = frame_target { self.wayland_server.send_pointer_frame(target_id); }
    }

    fn handle_processed_pointer_button(&mut self, event: ProcessedPointerButtonEvent) {
        tracing::debug!("FocusManager: Handling processed pointer button: {:?}", event);
        let target_surface_id_opt = if let Some(grab) = self.active_grabs.first() {
            Some(grab.surface_id)
        } else {
            if event.state == ButtonState::Pressed {
                // If no explicit grab, determine focus on press
                let surface_under_pointer = self.calculate_surface_at_pointer(event.abs_x, event.abs_y);
                 if self.pointer_focus != surface_under_pointer { // Update pointer focus if needed
                    if let Some(old_focus_id) = self.pointer_focus {
                        self.wayland_server.send_pointer_leave(old_focus_id, event.serial);
                    }
                    self.pointer_focus = surface_under_pointer;
                    if let Some(new_focus_id) = self.pointer_focus {
                         self.wayland_server.send_pointer_enter(new_focus_id, event.abs_x, event.abs_y, event.serial);
                    }
                }
                if let Some(target_for_kbd_focus) = surface_under_pointer {
                    self.set_keyboard_focus(Some(target_for_kbd_focus), event.serial);
                } else { // Click on "nothing"
                    self.set_keyboard_focus(None, event.serial);
                }
                surface_under_pointer // Event goes to where pointer is now focused or was just focused
            } else { // Button release
                self.pointer_focus // Send release to wherever pointer currently is (if not grabbed)
            }
        };

        if let Some(target_surface_id) = target_surface_id_opt {
            tracing::trace!("FocusManager: Forwarding pointer button to Surface ID: {}", target_surface_id);
            self.wayland_server.send_pointer_button(target_surface_id, &event, event.serial);
            self.wayland_server.send_pointer_frame(target_surface_id);
        } else {
            tracing::debug!("FocusManager: Pointer button event on no focused surface and no grab. Dropped: {:?}", event);
        }
    }

    fn handle_processed_pointer_scroll(&self, event: ProcessedPointerScrollEvent) {
        tracing::debug!("FocusManager: Handling processed pointer scroll: {:?}", event);
        let target_surface = if let Some(grab) = self.active_grabs.first() { Some(grab.surface_id) } else { self.pointer_focus };
        if let Some(surface_id) = target_surface {
            tracing::trace!("FocusManager: Forwarding pointer scroll to Surface ID: {}", surface_id);
            self.wayland_server.send_pointer_axis(surface_id, &event);
            self.wayland_server.send_pointer_frame(surface_id);
        } else {
             tracing::debug!("FocusManager: Pointer scroll event on no focused surface and no grab. Dropped: {:?}", event);
        }
    }

    fn handle_processed_touch_down(&mut self, event: ProcessedTouchDownEvent) {
        tracing::debug!("FocusManager: Handling processed touch down: {:?}", event);
        let target_surface_id = self.calculate_surface_at_pointer(event.x, event.y);
        if let Some(surface_id) = target_surface_id {
            self.touch_focus.insert(event.id, surface_id);
            tracing::info!("FocusManager: Touch ID {} focused on Surface ID: {}", event.id, surface_id);
            self.set_keyboard_focus(Some(surface_id), event.serial);
            self.wayland_server.send_touch_down(surface_id, &event, event.serial);
        } else {
            tracing::debug!("FocusManager: Touch down event on no surface. Dropped. ID: {}", event.id);
        }
    }

    fn handle_processed_touch_motion(&self, event: ProcessedTouchMotionEvent) {
        tracing::debug!("FocusManager: Handling processed touch motion: {:?}", event);
        if let Some(&surface_id) = self.touch_focus.get(&event.id) {
            tracing::trace!("FocusManager: Forwarding touch motion for ID {} to Surface ID: {}", event.id, surface_id);
            self.wayland_server.send_touch_motion(surface_id, &event);
        } else {
            tracing::debug!("FocusManager: Touch motion for unknown/unfocused touch ID: {}. Dropped.", event.id);
        }
    }

    fn handle_processed_touch_up(&mut self, event: ProcessedTouchUpEvent) {
        tracing::debug!("FocusManager: Handling processed touch up: {:?}", event);
        if let Some(surface_id) = self.touch_focus.remove(&event.id) {
            tracing::info!("FocusManager: Touch ID {} unfocused from Surface ID: {}", event.id, surface_id);
            self.wayland_server.send_touch_up(surface_id, &event, event.serial);
        } else {
            tracing::debug!("FocusManager: Touch up for unknown/unfocused touch ID: {}. Dropped.", event.id);
        }
    }

    fn handle_processed_touch_frame(&self, _event: ProcessedTouchFrameEvent) {
        tracing::debug!("FocusManager: Handling processed touch frame.");
        let mut notified_surfaces = HashSet::new();
        for surface_id in self.touch_focus.values() {
            if !notified_surfaces.contains(surface_id) {
                 tracing::trace!("FocusManager: Sending touch frame to Surface ID: {}", surface_id);
                 self.wayland_server.send_touch_frame(*surface_id);
                 notified_surfaces.insert(*surface_id);
            }
        }
    }
}


// Public accessors for Keyboard state needed by FocusManager for wl_keyboard.enter
// These are okay for stubs, but in real code, Keyboard might provide more direct methods.
impl Keyboard {
    pub fn pressed_keys(&self) -> &HashSet<u32> { &self.pressed_keys }
    pub fn modifier_state(&self) -> ModifiersState { self.modifier_state }
    // Allow tests to see pressed keys
    #[cfg(test)]
    pub fn test_pressed_keys(&self) -> &HashSet<u32> { &self.pressed_keys }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::config::{InputConfig, PointerConfig as CfgPointer, KeyboardConfig as CfgKeyboard, DeviceSpecificConfigEntry};
    use crate::wayland_server_module_placeholder::{WaylandServerHandle, SurfaceManagerHandle, StubbedSurfaceInfo, Rect};
    use crate::input::keyboard::{Keyboard, KeyState, ModifiersState};
    use crate::input::pointer::{Pointer, ButtonState as PtrButtonState, ScrollSource as PtrScrollSource};
    use crate::input::touch::Touch;
    use std::collections::HashMap;

    // Helper to create a default InputConfig for tests
    fn default_test_input_config() -> InputConfig {
        InputConfig {
            default_pointer_config: Some(CfgPointer {
                acceleration_factor: Some(0.0), sensitivity: Some(1.0),
                acceleration_curve: None, button_mapping: None,
            }),
            default_keyboard_config: Some(CfgKeyboard {
                repeat_rate: Some(25), repeat_delay: Some(600),
            }),
            device_specific: None,
        }
    }

    // Helper to create a FocusManager with default stubs for testing
    fn create_test_focus_manager(config: &InputConfig) -> FocusManager {
        let sm_handle = SurfaceManagerHandle::new(); // Uses its own stubbed surfaces
        let wl_handle = WaylandServerHandle::new();
        FocusManager::new(wl_handle, sm_handle, config)
    }

    #[test]
    fn test_focus_manager_new() {
        let config = default_test_input_config();
        let fm = create_test_focus_manager(&config);
        assert!(fm.keyboard_focus.is_none());
        assert!(fm.pointer_focus.is_none());
        assert!(fm.active_grabs.is_empty());
    }

    #[test]
    fn test_calculate_surface_at_pointer() {
        let config = default_test_input_config();
        let fm = create_test_focus_manager(&config);
        // Based on SurfaceManagerHandle stub:
        // s1: id: 1, x: 0, y: 0, w: 800, h: 600, z:0
        // s2: id: 2, x: 50, y: 50, w: 100, h: 100, z:1 (top)

        assert_eq!(fm.calculate_surface_at_pointer(10.0, 10.0), Some(1)); // Only s1
        assert_eq!(fm.calculate_surface_at_pointer(60.0, 60.0), Some(2)); // s2 is on top of s1
        assert_eq!(fm.calculate_surface_at_pointer(700.0, 500.0), Some(1)); // Only s1
        assert_eq!(fm.calculate_surface_at_pointer(900.0, 700.0), None); // Outside both
    }

    #[test]
    fn test_set_keyboard_focus() {
        let config = default_test_input_config();
        let mut fm = create_test_focus_manager(&config);
        let test_serial = 123;

        fm.set_keyboard_focus(Some(1), test_serial);
        assert_eq!(fm.keyboard_focus, Some(1));
        assert_eq!(fm.focus_history.front(), Some(&1));
        // TODO: Verify WaylandServerHandle logs for send_keyboard_enter (needs mock or log capture)

        fm.set_keyboard_focus(Some(2), test_serial + 1);
        assert_eq!(fm.keyboard_focus, Some(2));
        assert_eq!(fm.focus_history.front(), Some(&2));
        assert_eq!(fm.focus_history.get(1), Some(&1));
        // TODO: Verify WaylandServerHandle logs for send_keyboard_leave (for s1) and send_keyboard_enter (for s2)

        fm.set_keyboard_focus(None, test_serial + 2);
        assert!(fm.keyboard_focus.is_none());
        // TODO: Verify WaylandServerHandle logs for send_keyboard_leave (for s2)
    }

    #[test]
    fn test_pointer_motion_focus_change() {
        let config = default_test_input_config();
        let mut fm = create_test_focus_manager(&config);

        // Motion into surface 1 (from outside)
        fm.handle_raw_pointer_motion(10.0, 10.0, 100); // x=10, y=10
        assert_eq!(fm.pointer_focus, Some(1));
        // TODO: Verify WaylandServerHandle logs: send_pointer_enter(1), send_pointer_motion(1), send_pointer_frame(1)

        // Motion into surface 2 (from surface 1)
        fm.handle_raw_pointer_motion(50.0, 50.0, 101); // dx=50, dy=50 -> total x=60, y=60
        assert_eq!(fm.pointer_focus, Some(2));
        // TODO: Verify WaylandServerHandle logs: send_pointer_leave(1), send_pointer_enter(2), send_pointer_motion(2), send_pointer_frame(2)

        // Motion out of all surfaces
        fm.handle_raw_pointer_motion(1000.0, 1000.0, 102); // x=1060, y=1060
        assert!(fm.pointer_focus.is_none());
        // TODO: Verify WaylandServerHandle logs: send_pointer_leave(2)
    }

    #[test]
    fn test_pointer_button_click_to_focus() {
        let config = default_test_input_config();
        let mut fm = create_test_focus_manager(&config);

        // Initial state: no focus
        assert!(fm.keyboard_focus.is_none());
        assert!(fm.pointer_focus.is_none());

        // Click on surface 1 (coords 10,10)
        fm.handle_raw_pointer_button(1, PtrButtonState::Pressed, 200);
        // This raw handler calls pointer_handler.handle_button_event, which generates a ProcessedPointerButtonEvent.
        // Then FocusManager.handle_processed_pointer_button is called.
        // Pointer coords for button event are from pointer_handler's internal state.
        // We need to simulate a motion first to position the pointer.
        fm.handle_raw_pointer_motion(10.0, 10.0, 199); // Move pointer to (10,10)
        assert_eq!(fm.pointer_focus, Some(1)); // Pointer focus updated by motion

        fm.handle_raw_pointer_button(1, PtrButtonState::Pressed, 200);
        assert_eq!(fm.keyboard_focus, Some(1)); // Keyboard focus should follow click
        // TODO: Verify WaylandServerHandle logs: send_keyboard_enter(1), send_pointer_button(1)
    }

    #[test]
    fn test_keyboard_event_routing_to_focus() {
        let config = default_test_input_config();
        let mut fm = create_test_focus_manager(&config);
        fm.set_keyboard_focus(Some(1), 1000); // Set focus to surface 1

        // Simulate 'a' key press
        fm.handle_raw_keyboard_input(30, KeyState::Pressed, 1001);
        // TODO: Verify WaylandServerHandle logs: send_keyboard_event_to_surface(1, ...)
        // Check that internal raw key state is updated
        assert!(fm.current_pressed_keys.contains(&30));

        fm.handle_raw_keyboard_input(30, KeyState::Released, 1002);
        assert!(!fm.current_pressed_keys.contains(&30));
    }

    #[test]
    fn test_pointer_grab() {
        let config = default_test_input_config();
        let mut fm = create_test_focus_manager(&config);
        let surface_grab_owner: SurfaceId = 5; // Assume this surface exists for grab
        let grab_serial = 300;

        fm.request_pointer_grab(surface_grab_owner, grab_serial);
        assert_eq!(fm.active_grabs.len(), 1);
        assert_eq!(fm.active_grabs[0].surface_id, surface_grab_owner);

        // Motion event while grabbed - should go to grab owner, not affect pointer_focus for enter/leave
        fm.handle_raw_pointer_motion(10.0, 10.0, 301); // Moves to (10,10) which is surface 1
        assert_eq!(fm.pointer_focus, None); // Pointer focus should not change due to enter/leave during grab
        // TODO: Verify WaylandServerHandle logs: send_pointer_motion(surface_grab_owner, ...)

        fm.release_pointer_grab(surface_grab_owner, grab_serial);
        assert!(fm.active_grabs.is_empty());
    }
     #[test]
    fn test_touch_down_sets_keyboard_focus() {
        let config = default_test_input_config();
        let mut fm = create_test_focus_manager(&config);

        // Touch down on surface 2 (coords 60,60)
        fm.handle_raw_touch_down(0, 60.0, 60.0, 400);
        assert_eq!(fm.keyboard_focus, Some(2));
        assert_eq!(fm.touch_focus.get(&0), Some(&2));
        // TODO: Verify WaylandServerHandle logs for touch_down and keyboard_enter
    }
}
