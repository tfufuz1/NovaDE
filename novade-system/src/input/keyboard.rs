// src/input/keyboard.rs
use crate::input::config::KeyboardConfig;
use input::event::keyboard::{KeyboardKeyEvent, KeyState};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use xkbcommon::xkb;

// Libinput keycodes are offset by +8 compared to XKB/evdev keycodes.
const KEYCODE_OFFSET: u32 = 8;

#[derive(Debug, Clone, Copy, Default)]
pub struct ModifiersState {
    pub depressed: xkb::ModMask,
    pub latched: xkb::ModMask,
    pub locked: xkb::ModMask,
    pub effective: xkb::ModMask,
}

struct RepeatingKeyInfo {
    keycode: u32, // XKB keycode
    keysym: xkb::Keysym,
    first_repeat_at: Instant,
    next_repeat_at: Instant,
    repeat_interval: Duration,
}

pub struct Keyboard {
    xkb_context: xkb::Context,
    xkb_keymap: xkb::Keymap,
    pub xkb_state: xkb::State,
    modifiers_state: ModifiersState,

    // Key repeat handling
    repeating_key_info: Option<RepeatingKeyInfo>,
    repeat_delay_ms: u64,
    repeat_rate_hz: u32,

    // Compose key support
    compose_state: Option<xkb::ComposeState>,
    compose_pending: bool,
}

impl Keyboard {
    pub fn new(config: &KeyboardConfig) -> Result<Self, String> {
        info!("Keyboard: Initializing XKB context and keymap...");
        debug!("Keyboard: Using config: {:?}", config);

        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        if context.is_null() {
            return Err("Keyboard: Failed to create XKB context.".to_string());
        }
        info!("Keyboard: XKB context created successfully.");

        let keymap = xkb::Keymap::new_from_names(
            &context,
            "", // rules (empty for default)
            "", // model (empty for default)
            "", // layout (empty for default)
            "", // variant (empty for default)
            None, // options
            xkb::COMPILE_NO_FLAGS,
        );

        let xkb_keymap = match keymap {
            Some(km) => {
                info!("Keyboard: XKB keymap loaded successfully (using system defaults).");
                km
            }
            None => {
                return Err("Keyboard: Failed to load default XKB keymap.".to_string());
            }
        };

        let xkb_state = xkb::State::new(&xkb_keymap);
        info!("Keyboard: XKB state created from keymap.");
        let initial_modifiers = Self::current_xkb_modifiers_state(&xkb_state);

        // Initialize Compose State (Stub)
        let mut compose_state_opt: Option<xkb::ComposeState> = None;
        // TODO: Locale should ideally come from system settings or user configuration.
        let locale = "en_US.UTF-8";
        match xkb::ComposeTable::new_from_locale(&context, locale, xkb::COMPILE_NO_FLAGS) {
            Some(compose_table) => {
                info!("Keyboard: XKB compose table loaded successfully for locale '{}'.", locale);
                match xkb::ComposeState::new(&compose_table, xkb::COMPOSE_STATE_NO_FLAGS) {
                    Some(state) => {
                        info!("Keyboard: XKB compose state initialized successfully.");
                        compose_state_opt = Some(state);
                    }
                    None => {
                        warn!("Keyboard: Failed to create XKB compose state from table.");
                    }
                }
            }
            None => {
                warn!("Keyboard: Failed to load XKB compose table for locale '{}'. Compose key functionality will be unavailable.", locale);
            }
        }

        Ok(Self {
            xkb_context: context,
            xkb_keymap,
            xkb_state,
            modifiers_state: initial_modifiers,
            repeating_key_info: None,
            repeat_delay_ms: config.repeat_delay as u64,
            repeat_rate_hz: config.repeat_rate as u32,
            compose_state: compose_state_opt,
            compose_pending: false,
        })
    }

    fn current_xkb_modifiers_state(state: &xkb::State) -> ModifiersState {
        ModifiersState {
            depressed: state.serialize_mods(xkb::STATE_MODS_DEPRESSED),
            latched: state.serialize_mods(xkb::STATE_MODS_LATCHED),
            locked: state.serialize_mods(xkb::STATE_MODS_LOCKED),
            effective: state.serialize_mods(xkb::STATE_MODS_EFFECTIVE),
        }
    }

    pub fn get_effective_modifiers(&self) -> xkb::ModMask {
        self.modifiers_state.effective
    }

    pub fn get_modifiers_state(&self) -> ModifiersState {
        self.modifiers_state
    }

    pub fn handle_key_event(&mut self, event: &KeyboardKeyEvent) {
        let keycode = event.key();
        let xkb_keycode = keycode + KEYCODE_OFFSET;

        let direction = match event.key_state() {
            KeyState::Pressed => xkb::KeyDirection::Down,
            KeyState::Released => xkb::KeyDirection::Up,
        };

        // For compose, we are interested in Down events for feeding keysyms.
        // The actual key update to xkb_state should happen regardless,
        // as it affects modifiers which might be part of a compose sequence.
        let state_component_changed = self.xkb_state.update_key(xkb_keycode, direction);

        if state_component_changed.intersects(xkb::STATE_MODS_EFFECTIVE | xkb::STATE_MODS_DEPRESSED | xkb::STATE_MODS_LATCHED | xkb::STATE_MODS_LOCKED) {
            self.modifiers_state = Self::current_xkb_modifiers_state(&self.xkb_state);
            debug!("Keyboard: Modifier state changed: {:?}", self.modifiers_state);
        }

        let current_keysym = self.xkb_state.key_get_one_sym(xkb_keycode);
        let current_keysym_name = xkb::keysym_get_name(current_keysym);

        // --- Compose Key Handling (Stub) ---
        let mut event_consumed_by_compose = false;
        if direction == xkb::KeyDirection::Down { // Only feed key presses to compose state
            if let Some(ref mut compose_state) = self.compose_state {
                // Feed the keysym to the compose state.
                // Note: According to libxkbcommon docs, feeding a keysym that didn't result from
                // a key press (e.g. from a key repeat) might be problematic or ignored.
                // For now, we feed all keysyms from Down events.
                compose_state.feed(current_keysym);
                let status = compose_state.status();

                match status {
                    xkb::ComposeStatus::Composing => {
                        self.compose_pending = true;
                        event_consumed_by_compose = true; // Event is part of compose sequence
                        info!("Keyboard: Compose sequence active (composing). Fed keysym: '{}'.", current_keysym_name);
                    }
                    xkb::ComposeStatus::Composed => {
                        let composed_sym = compose_state.keysym();
                        let composed_name = xkb::keysym_get_name(composed_sym);
                        info!("Keyboard: Compose sequence COMPLETED. Composed keysym: '{}' ({:#0X}).", composed_name, composed_sym);
                        // TODO: This composed_sym should be sent to the client.
                        // For now, just log. The original event is consumed.
                        compose_state.reset();
                        self.compose_pending = false;
                        event_consumed_by_compose = true;
                    }
                    xkb::ComposeStatus::Cancelled => {
                        if self.compose_pending {
                            info!("Keyboard: Compose sequence CANCELLED. Fed keysym: '{}'.", current_keysym_name);
                        }
                        compose_state.reset(); // Reset on cancel
                        self.compose_pending = false;
                        // Event is NOT consumed by compose; process normally.
                    }
                    xkb::ComposeStatus::Nothing => {
                        if self.compose_pending {
                            // This means the sequence ended without a valid composition.
                            info!("Keyboard: Compose sequence ended with NO result after fed keysym: '{}'.", current_keysym_name);
                            self.compose_pending = false;
                        }
                        // Event is NOT consumed by compose; process normally.
                        // (No need to reset here as per typical state machine, it's already "nothing")
                    }
                }
            }
        }

        // If event was consumed by compose, we might not want to process repeats or other default actions.
        if event_consumed_by_compose {
            // If a key press was consumed by compose, cancel any repeats for previous keys.
            // And do not initiate repeat for the current key.
            if self.repeating_key_info.is_some() {
                 self.repeating_key_info = None;
                 debug!("Keyboard: Key repeat cancelled due to compose sequence consuming key press.");
            }
            debug!("Keyboard: Key event for '{}' ({:?}) consumed by compose sequence.", current_keysym_name, event.key_state());
            return; // Early return for consumed event
        }

        // --- Normal Key Processing (including Repeat) ---
        // (This part is skipped if event_consumed_by_compose is true)
        debug!(
            "Keyboard: Key event (post-compose check): raw_kc={}, xkb_kc={}, state={:?}, keysym='{}' ({:#0X}), mods_eff={:?}",
            keycode, xkb_keycode, event.key_state(), current_keysym_name, current_keysym, self.modifiers_state.effective
        );

        match direction {
            xkb::KeyDirection::Down => {
                if self.repeat_rate_hz > 0 && self.xkb_keymap.key_repeats(xkb_keycode) {
                    let repeat_interval_ms = if self.repeat_rate_hz > 0 { 1000 / self.repeat_rate_hz as u64 } else { 0 };
                    if repeat_interval_ms > 0 {
                        let now = Instant::now();
                        let first_repeat_at = now + Duration::from_millis(self.repeat_delay_ms);
                        self.repeating_key_info = Some(RepeatingKeyInfo {
                            keycode: xkb_keycode,
                            keysym: current_keysym,
                            first_repeat_at,
                            next_repeat_at: first_repeat_at,
                            repeat_interval: Duration::from_millis(repeat_interval_ms),
                        });
                        debug!("Keyboard: Key repeat initiated for keycode {} ('{}'), delay: {}ms, rate: {}hz (interval: {}ms)",
                               xkb_keycode, current_keysym_name, self.repeat_delay_ms, self.repeat_rate_hz, repeat_interval_ms);
                    } else {
                         if self.repeating_key_info.is_some() {
                            self.repeating_key_info = None;
                            debug!("Keyboard: Key repeat cancelled for keycode {} due to repeat_rate_hz <= 0.", xkb_keycode);
                        }
                    }
                } else {
                    if let Some(info) = &self.repeating_key_info {
                        if info.keycode == xkb_keycode {
                            self.repeating_key_info = None;
                             debug!("Keyboard: Key does not repeat, cancelling any existing repeat for keycode {}.", xkb_keycode);
                        }
                    }
                }
            }
            xkb::KeyDirection::Up => {
                if let Some(info) = &self.repeating_key_info {
                    if info.keycode == xkb_keycode {
                        self.repeating_key_info = None;
                        debug!("Keyboard: Key repeat cancelled for keycode {} ('{}') due to key release.", xkb_keycode, current_keysym_name);
                    }
                }
            }
        }
    }

    pub fn process_repeat(&mut self, now: Instant) {
        let mut should_repeat_info: Option<(u32, xkb::Keysym)> = None;

        if let Some(info) = &self.repeating_key_info {
            if now >= info.next_repeat_at {
                should_repeat_info = Some((info.keycode, info.keysym));
            }
        }

        if let Some((keycode_to_repeat, keysym_to_repeat)) = should_repeat_info {
            // Take and update, then put back to avoid borrowing issues if we called handle_key_event
            if let Some(mut info) = self.repeating_key_info.take() {
                let keysym_name = xkb::keysym_get_name(keysym_to_repeat);
                info!(
                    "Keyboard: Repeating key: xkb_keycode={}, keysym='{}' ({:#0X}), mods_eff={:?}",
                    keycode_to_repeat, keysym_name, keysym_to_repeat, self.modifiers_state.effective
                );

                // TODO: Generate a "repeated" key event to be sent to the client.
                // This might involve constructing a synthetic KeyboardKeyEvent or similar.
                // For now, we just log.
                // Crucially, a repeated key should NOT re-feed the compose state.

                info.next_repeat_at += info.repeat_interval;
                if info.next_repeat_at < now { // Ensure time moves forward
                    info.next_repeat_at = now + info.repeat_interval;
                }
                self.repeating_key_info = Some(info); // Put it back
            }
        }
    }
}

impl std::fmt::Debug for Keyboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keyboard")
            .field("modifiers_state", &self.modifiers_state)
            .field("repeat_delay_ms", &self.repeat_delay_ms)
            .field("repeat_rate_hz", &self.repeat_rate_hz)
            .field("repeating_key_info", &self.repeating_key_info)
            .field("compose_pending", &self.compose_pending)
            .field("compose_state_is_some", &self.compose_state.is_some())
            .field("xkb_context", &"Opaque xkb::Context")
            .field("xkb_keymap", &"Opaque xkb::Keymap")
            // xkb_state is not easily printable without its internal raw pointers
            .field("xkb_state", &"Opaque xkb::State")
            .finish()
    }
}

impl std::fmt::Debug for RepeatingKeyInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RepeatingKeyInfo")
            .field("keycode", &self.keycode)
            .field("keysym", &format_args!("{:#0X} ({})", self.keysym, xkb::keysym_get_name(self.keysym)))
            .field("first_repeat_at", &self.first_repeat_at)
            .field("next_repeat_at", &self.next_repeat_at)
            .field("repeat_interval", &self.repeat_interval)
            .finish()
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
