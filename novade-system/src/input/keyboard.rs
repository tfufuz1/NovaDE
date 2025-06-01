// src/input/keyboard.rs

use std::collections::HashSet;
use super::config::KeyboardConfig;
// ProcessedKeyEvent is defined in focus.rs, Keyboard shouldn't know about FocusManager directly.
use super::focus::ProcessedKeyEvent;

// --- Stubbed XKB Structures ---
#[derive(Debug, Clone)]
pub struct StubXkbKeymap;

impl StubXkbKeymap {
    pub fn new(name: &str, options: &str) -> Option<Self> {
        tracing::debug!("StubXkbKeymap: new(name: \"{}\", options: \"{}\") called.", name, options);
        Some(Self)
    }
}

#[derive(Debug, Clone)]
pub struct StubXkbState;

impl StubXkbState {
    pub fn new(_keymap: &StubXkbKeymap) -> Option<Self> {
        tracing::debug!("StubXkbState: new() called.");
        Some(Self)
    }

    pub fn update_key(&mut self, keycode: u32, direction: KeyDirection) {
        tracing::trace!("StubXkbState: update_key(keycode: {}, direction: {:?}) called.", keycode, direction);
    }

    pub fn get_sym(&self, keycode: u32) -> u32 {
        let stub_sym = keycode + 1000;
        tracing::trace!("StubXkbState: get_sym(keycode: {}) -> returning stub_sym: {}.", keycode, stub_sym);
        stub_sym
    }

    pub fn serialize_mods(&self) -> (u32, u32, u32, u32) {
        tracing::trace!("StubXkbState: serialize_mods() -> returning (0,0,0,0).");
        (0, 0, 0, 0)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KeyDirection { Down, Up }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState { Pressed, Released }


#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)] // Added PartialEq, Eq for comparison
pub struct ModifiersState {
    pub depressed: u32,
    pub latched: u32,
    pub locked: u32,
    pub group: u32,
}

pub struct Keyboard {
    modifier_state: ModifiersState,
    pressed_keys: HashSet<u32>,
    xkb_keymap: Option<StubXkbKeymap>,
    xkb_state: Option<StubXkbState>,
    repeat_config: Option<KeyboardConfig>,
    repeating_key: Option<u32>,
    // focus_manager field removed
}

impl Keyboard {
    // `focus_manager` parameter removed from new
    pub fn new(config: Option<KeyboardConfig>) -> Self {
        tracing::info!("Keyboard: Initializing (refactored)... Config: {:?}", config);
        let keymap = StubXkbKeymap::new("default", "");
        let state = keymap.as_ref().and_then(StubXkbState::new);

        if keymap.is_none() || state.is_none() {
            tracing::warn!("Keyboard: Failed to initialize stubbed XKB keymap/state.");
        }

        Self {
            modifier_state: ModifiersState::default(),
            pressed_keys: HashSet::new(),
            xkb_keymap: keymap,
            xkb_state: state,
            repeat_config: config,
            repeating_key: None,
        }
    }

    // handle_key_event now returns Option<ProcessedKeyEvent>
    // Serial will be added by FocusManager.
    pub fn handle_key_event(&mut self, raw_keycode: u32, key_state: KeyState, time: u32) -> Option<ProcessedKeyEvent> {
        tracing::debug!("Keyboard: handle_key_event(raw_keycode: {}, state: {:?}, time: {})", raw_keycode, key_state, time);

        let direction = match key_state {
            KeyState::Pressed => {
                self.pressed_keys.insert(raw_keycode);
                KeyDirection::Down
            }
            KeyState::Released => {
                self.pressed_keys.remove(&raw_keycode);
                KeyDirection::Up
            }
        };

        let mut sym = 0;
        if let Some(state) = &mut self.xkb_state {
            state.update_key(raw_keycode, direction);
            sym = state.get_sym(raw_keycode);
        }

        // update_modifier_state now returns Option<ModifiersState>
        let new_mods_opt = self.update_modifier_state();
        // Use the new modifier state if available, otherwise keep the current one for the event.
        let event_modifiers = new_mods_opt.unwrap_or(self.modifier_state);

        // Key Repeat Logic (Conceptual for now)
        if let Some(config) = &self.repeat_config {
            if config.repeat_rate.is_some() && config.repeat_delay.is_some() {
                match key_state {
                    KeyState::Pressed => {
                        tracing::debug!("Keyboard: Key pressed. Raw: {}. Starting/resetting repeat timer (conceptual).", raw_keycode);
                        self.repeating_key = Some(raw_keycode);
                    }
                    KeyState::Released => {
                        if self.repeating_key == Some(raw_keycode) {
                            tracing::debug!("Keyboard: Key released. Raw: {}. Cancelling repeat timer (conceptual).", raw_keycode);
                            self.repeating_key = None;
                        }
                    }
                }
            }
        }

        // Prepare and return event. FocusManager will add serial.
        let processed_event = ProcessedKeyEvent {
            keysym: sym,
            raw_keycode,
            state: key_state,
            modifiers: event_modifiers,
            time,
            serial: 0, // Placeholder, FocusManager should fill this
        };

        tracing::trace!("Keyboard: Returning ProcessedKeyEvent: {:?}", processed_event);
        Some(processed_event)
        // TODO: Implement Compose Key Support
    }

    // update_modifier_state now returns Option<ModifiersState>
    pub fn update_modifier_state(&mut self) -> Option<ModifiersState> {
        if let Some(state) = &self.xkb_state {
            let (depressed, latched, locked, group) = state.serialize_mods();
            let new_state = ModifiersState { depressed, latched, locked, group };
            if self.modifier_state != new_state { // Only update and return if changed
                self.modifier_state = new_state;
                tracing::debug!("Keyboard: Updated modifier state: {:?}", self.modifier_state);
                Some(self.modifier_state)
            } else {
                tracing::trace!("Keyboard: Modifier state unchanged: {:?}", self.modifier_state);
                None // No change
            }
        } else {
            tracing::warn!("Keyboard: Cannot update modifier state, XKB state not available.");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::config::KeyboardConfig;
    use crate::input::focus::ProcessedKeyEvent; // For type matching

    #[test]
    fn test_keyboard_new() {
        let kb_config = KeyboardConfig { repeat_rate: Some(30), repeat_delay: Some(500) };
        let keyboard = Keyboard::new(Some(kb_config));
        assert!(keyboard.pressed_keys.is_empty());
        assert_eq!(keyboard.modifier_state, ModifiersState::default());
        assert!(keyboard.xkb_keymap.is_some()); // StubXkbKeymap::new returns Some
        assert!(keyboard.xkb_state.is_some());  // StubXkbState::new returns Some
        assert_eq!(keyboard.repeat_config.as_ref().unwrap().repeat_rate, Some(30));
    }

    #[test]
    fn test_handle_key_event_press_release() {
        let mut keyboard = Keyboard::new(None);
        let keycode_a = 30; // Example keycode
        let time = 1000;

        // Press event
        let event_opt = keyboard.handle_key_event(keycode_a, KeyState::Pressed, time);
        assert!(event_opt.is_some());
        let processed_event = event_opt.unwrap();

        assert!(keyboard.pressed_keys.contains(&keycode_a));
        assert_eq!(processed_event.raw_keycode, keycode_a);
        assert_eq!(processed_event.state, KeyState::Pressed);
        assert_eq!(processed_event.keysym, keycode_a + 1000); // StubXkbState logic

        // Release event
        let event_opt_rel = keyboard.handle_key_event(keycode_a, KeyState::Released, time + 10);
        assert!(event_opt_rel.is_some());
        let processed_event_rel = event_opt_rel.unwrap();

        assert!(!keyboard.pressed_keys.contains(&keycode_a));
        assert_eq!(processed_event_rel.state, KeyState::Released);
    }

    #[test]
    fn test_modifier_state_update_on_key_event() {
        // This test is limited because StubXkbState.serialize_mods always returns (0,0,0,0)
        // and StubXkbState.update_key is a no-op regarding actual modifier calculation.
        // A real test would need a mockable XKB state or more complex stubs.
        let mut keyboard = Keyboard::new(None);
        let keycode_shift = 42; // Example shift keycode

        // Simulate Shift press - this should internally call update_modifier_state
        keyboard.handle_key_event(keycode_shift, KeyState::Pressed, 100);

        // In our current stub, serialize_mods() always returns default.
        // So, this test mainly verifies that update_modifier_state is called and doesn't panic.
        // The returned ModifiersState from update_modifier_state() will be the default.
        let mod_state_opt = keyboard.update_modifier_state();
        // If it was already default and no actual change happened in stub, it returns None
        // If it was different and changed to default, it returns Some(default)
        // Our initial state is default, stub always returns default, so expect None (no change from default)
        // OR, if we want to ensure it *became* default:
        if mod_state_opt.is_some() { // if it did change
             assert_eq!(mod_state_opt.unwrap(), ModifiersState::default());
        } else { // if it didn't change from initial default
             assert_eq!(keyboard.modifier_state, ModifiersState::default());
        }


        // The ProcessedKeyEvent from handle_key_event would contain the modifier state
        let event_opt = keyboard.handle_key_event(keycode_shift, KeyState::Pressed, 100);
        let processed_event = event_opt.unwrap();
        // This reflects the state *after* the key press, based on stubbed serialize_mods
        assert_eq!(processed_event.modifiers, ModifiersState { depressed: 0, latched: 0, locked: 0, group: 0 });
    }

    #[test]
    fn test_key_repeat_logic_conceptual() {
        let kb_config = KeyboardConfig { repeat_rate: Some(25), repeat_delay: Some(600) };
        let mut keyboard = Keyboard::new(Some(kb_config));
        let keycode = 30;

        assert!(keyboard.repeating_key.is_none());
        keyboard.handle_key_event(keycode, KeyState::Pressed, 100);
        assert_eq!(keyboard.repeating_key, Some(keycode));

        // Press another key - repeat should switch
        keyboard.handle_key_event(31, KeyState::Pressed, 110);
        assert_eq!(keyboard.repeating_key, Some(31));

        keyboard.handle_key_event(31, KeyState::Released, 120);
        assert!(keyboard.repeating_key.is_none());

        // Release a non-repeating key
        keyboard.handle_key_event(keycode, KeyState::Released, 130); // keycode was not the one repeating
        assert!(keyboard.repeating_key.is_none()); // Should still be none
    }
}
