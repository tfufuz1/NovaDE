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
