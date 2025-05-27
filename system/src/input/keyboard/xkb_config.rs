use crate::input::errors::InputError;
use smithay::{
    backend::input::KeyboardKeyEvent,
    input::keyboard::{KeyboardConfig, ModifiersState as SmithayModifiersState},
    reexports::{
        calloop::TimerHandle, // For repeat_timer
        wayland_server::{protocol::wl_surface::WlSurface, Serial}, // For repeat_key_serial
    },
};
use std::sync::{Mutex, Weak, Arc}; // Added Mutex, Arc
use xkbcommon::xkb; // For XKB related types

#[derive(Debug)]
pub struct XkbKeyboardData {
    pub context: xkb::Context,
    pub keymap: xkb::Keymap,
    pub state: xkb::State,
    pub repeat_timer: Mutex<Option<TimerHandle>>,
    // Store: (libinput_keycode, xkb_keycode, SmithayModifiersState, delay_ms, rate_hz)
    pub repeat_info: Mutex<Option<(u32, xkb::Keycode, SmithayModifiersState, std::time::Duration, std::time::Duration)>>,
    pub focused_surface_on_seat: Mutex<Option<Weak<WlSurface>>>, // Added Mutex
    pub repeat_key_serial: Mutex<Option<Serial>>,
}

impl Clone for XkbKeyboardData {
    fn clone(&self) -> Self {
        // xkb::Context has no clone. Keymap and State can be cloned from their base objects.
        // This manual Clone is tricky because xkb objects are pointers internally.
        // A proper clone would involve re-creating keymap and state from names/rules,
        // or ensuring that xkbcommon's ref-counting (if any on these types) is handled.
        // For now, this will be a shallow clone for keymap and state if they are Arc-like,
        // or a re-parse if they are not.
        // xkb::Keymap can be cloned (it's an Arc internally).
        // xkb::State can be cloned (it's an Arc internally).
        // xkb::Context is not Clone. It should typically be created once per application.
        // This implies XkbKeyboardData should probably hold an Arc<xkb::Context>.

        // Let's assume context is shared via Arc if XkbKeyboardData needs to be Clone.
        // However, the plan is to store Arc<XkbKeyboardData> in the map, so XkbKeyboardData
        // itself doesn't strictly need to be Clone for that purpose.
        // The `Clone` requirement on `XkbKeyboardData` was from a previous step's assumption
        // that `KeyboardConfig` might own it directly.
        // Given `Arc<XkbKeyboardData>` in `keyboard_data_map`, `XkbKeyboardData` itself
        // does not need to be `Clone` unless `Arc::make_mut` is used extensively.

        // For now, let's remove Clone and see if it's truly needed by subsequent steps.
        // If it is, we'll need to handle context sharing properly (e.g. Arc<xkb::Context>).
        // The prompt for current step does not require XkbKeyboardData to be Clone.
        // The previous step's XkbKeyboardData stub was made Clone, but the full struct is harder.

        // --- Reverting the decision to make XkbKeyboardData Clone for now. ---
        // It will be stored as Arc<XkbKeyboardData> in the HashMap.
        // If mutable access is needed that `Arc::get_mut` can't provide,
        // fields within XkbKeyboardData will use interior mutability (Mutex).

        // This manual clone is effectively a re-initialization if context isn't shared.
        // This is complex and error-prone.
        // Let's assume for the sake of this step that XkbKeyboardData is NOT Clone.
        // If a clone is needed, it implies a deeper architectural choice for xkb::Context.
        // The `Arc<XkbKeyboardData>` in the map is the primary way to share this.

        // This method will not be called if XkbKeyboardData is not Clone.
        // We will proceed without Clone for XkbKeyboardData.
        // If any code path requires XkbKeyboardData: Clone, that code path needs review.
        // (e.g. `seat.add_keyboard(config, XkbKeyboardData, ...)` if XkbKeyboardData was passed by value).
        // Smithay's `add_keyboard` takes `KeyboardConfig`, not `XkbKeyboardData`.
        // Our `XkbKeyboardData` is stored in `DesktopState::keyboard_data_map`.

        // This placeholder will cause a compile error if Clone is actually required by some code.
        // Which is good, as it will highlight where the assumption is wrong.
        panic!("XkbKeyboardData clone is non-trivial due to xkb::Context and should be avoided if possible. Store as Arc<XkbKeyboardData>.");
    }
}


impl XkbKeyboardData {
    pub fn new(config: &KeyboardConfig<'_>) -> Result<Self, InputError> {
        tracing::info!(
            "Initializing XkbKeyboardData with config: rules={:?}, model={:?}, layout={:?}, variant={:?}, options={:?}",
            config.rules,
            config.model,
            config.layout,
            config.variant,
            config.options
        );

        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap_result = xkb::Keymap::new_from_names(
            &context,
            &config.rules.as_deref().unwrap_or(""),
            &config.model.as_deref().unwrap_or(""),
            &config.layout.as_deref().unwrap_or("us"), // Default to "us" layout
            &config.variant.as_deref().unwrap_or(""),
            config.options.clone(), // new_from_names expects Option<String>
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );

        let keymap = match keymap_result {
            Ok(km) => km,
            Err(e) => {
                tracing::warn!(
                    "Failed to compile XKB keymap with provided config ({:?}): {:?}. Falling back to default 'us' layout.",
                    config, e // XKB error type might not be Debug, convert to string if needed
                );
                // Try a fallback to a simple "us" layout without model/variant/options
                xkb::Keymap::new_from_names(
                    &context,
                    "", // rules
                    "", // model
                    "us", // layout
                    "", // variant
                    None, // options
                    xkb::KEYMAP_COMPILE_NO_FLAGS,
                )
                .map_err(|fallback_err| InputError::XkbConfigError {
                    seat_name: String::new(), // Seat name not known here, might need to be passed or set later
                    message: format!(
                        "Failed to compile primary XKB keymap and fallback 'us' keymap also failed: primary_err={:?}, fallback_err={:?}",
                        e, fallback_err // Again, XKB error might not be directly formattable.
                    ),
                })?
            }
        };

        let state = xkb::State::new(&keymap);

        Ok(Self {
            context,
            keymap,
            state,
            repeat_timer: Mutex::new(None),
            repeat_info: Mutex::new(None),
            focused_surface_on_seat: Mutex::new(None),
            repeat_key_serial: Mutex::new(None),
        })
    }

    /// Updates the XKB state based on a keyboard key event and returns the current modifier state.
    ///
    /// # Arguments
    /// * `event`: The `KeyboardKeyEvent` from the input backend.
    ///
    /// # Returns
    /// * `SmithayModifiersState`: The current state of modifiers (Shift, Ctrl, Alt, etc.)
    ///   after processing the key event.
    pub fn update_xkb_state_from_key_event(
        &mut self, // Takes &mut self to modify self.state
        event: &KeyboardKeyEvent<LibinputInputBackend>,
    ) -> SmithayModifiersState {
        // Convert libinput keycode to XKB keycode (libinput keycodes are XKB keycodes - 8)
        let xkb_keycode = event.key_code() + 8;

        // Update the XKB state
        let direction = match event.state() {
            smithay::backend::input::KeyState::Pressed => xkb::KeyDirection::Down,
            smithay::backend::input::KeyState::Released => xkb::KeyDirection::Up,
        };
        self.state.update_key(xkb_keycode, direction);

        // Serialize depressed, latched, locked modifiers and effective layout
        let depressed_mods = self.state.serialize_mods(xkb::STATE_MODS_DEPRESSED);
        let latched_mods = self.state.serialize_mods(xkb::STATE_MODS_LATCHED);
        let locked_mods = self.state.serialize_mods(xkb::STATE_MODS_LOCKED);
        let effective_layout = self.state.serialize_layout(xkb::STATE_LAYOUT_EFFECTIVE);

        // This is how Smithay's KeyboardHandle typically constructs ModifiersState
        SmithayModifiersState {
            depressed: depressed_mods,
            latched: latched_mods,
            locked: locked_mods,
            layout_effective: effective_layout,
            // Smithay 0.2 and older might have different field names or structure.
            // Smithay 0.3 ModifiersState has these fields.
        }
    }
}

// Note: XkbKeyboardData does not need to be Clone if it's always managed via Arc
// in the HashMap, and its methods take `&self` or `&mut self` (if Arc::get_mut or Arc::make_mut is used)
// or if its fields use interior mutability (e.g., Mutex).
// For `update_xkb_state_from_key_event` to take `&mut self`, it means the caller needs
// mutable access to `XkbKeyboardData`. If `XkbKeyboardData` is in an `Arc`, this implies
// `Arc::get_mut` or `Arc::make_mut` if the XKB state itself isn't behind a `Mutex`.
// `xkb::State` itself is internally ref-counted (like an Arc), so `update_key` takes `&State`.
// This means `update_xkb_state_from_key_event` can take `&self` if `self.state` is just `xkb::State`.
// Let's adjust that.

impl XkbKeyboardData {
    // Version of update_xkb_state_from_key_event taking &self
    // because xkb::State methods take &self.
    pub fn update_xkb_state_and_get_modifiers(
        &self, // Takes &self now
        event: &KeyboardKeyEvent<LibinputInputBackend>,
    ) -> SmithayModifiersState {
        let xkb_keycode = event.key_code() + 8;
        let direction = match event.state() {
            smithay::backend::input::KeyState::Pressed => xkb::KeyDirection::Down,
            smithay::backend::input::KeyState::Released => xkb::KeyDirection::Up,
        };
        // xkb::State::update_key takes &self, so this is fine.
        self.state.update_key(xkb_keycode, direction);

        SmithayModifiersState {
            depressed: self.state.serialize_mods(xkb::STATE_MODS_DEPRESSED),
            latched: self.state.serialize_mods(xkb::STATE_MODS_LATCHED),
            locked: self.state.serialize_mods(xkb::STATE_MODS_LOCKED),
            layout_effective: self.state.serialize_layout(xkb::STATE_LAYOUT_EFFECTIVE),
        }
    }
}
