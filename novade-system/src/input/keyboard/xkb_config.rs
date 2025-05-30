use xkbcommon::xkb;
use smithay::input::keyboard::{KeyboardConfig, ModifiersState as SmithayModifiersState};
use calloop::TimerHandle;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use wayland_server::Weak;
use smithay::utils::Serial;
use crate::input::errors::InputError; // Assuming InputError is in crate::input::errors

// #[derive(Debug)] // TimerHandle might not be Debug
pub struct XkbKeyboardData {
    pub context: xkb::Context,
    pub keymap: xkb::Keymap,
    pub state: xkb::State,
    pub repeat_timer: Option<TimerHandle>,
    pub repeat_info: Option<(u32 /* libinput keycode */, xkb::Keycode /* xkb keycode */, SmithayModifiersState, std::time::Duration /* delay */, std::time::Duration /* rate */)>,
    pub focused_surface_on_seat: Option<Weak<WlSurface>>,
    pub repeat_key_serial: Option<Serial>,
}

impl XkbKeyboardData {
    pub fn new(config: &KeyboardConfig<'_>) -> Result<Self, InputError> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        // Default to "evdev", "pc105", "us" if not provided in config, as per spec.
        let rules = config.rules.as_deref().unwrap_or("evdev");
        let model = config.model.as_deref().unwrap_or("pc105");
        let layout = config.layout.as_deref().unwrap_or("us");
        let variant = config.variant.as_deref();
        let options = config.options.as_deref();

        tracing::debug!(
            "Loading XKB Keymap: rules='{}', model='{}', layout='{}', variant='{:?}', options='{:?}'",
            rules, model, layout, variant, options
        );

        let mut keymap_builder = xkb::KeymapCompileArgsBuilder::new();
        // Set non-empty values only
        if !rules.is_empty() { keymap_builder.rules(rules); }
        if !model.is_empty() { keymap_builder.model(model); }
        if !layout.is_empty() { keymap_builder.layout(layout); }
        if let Some(v) = variant { if !v.is_empty() { keymap_builder.variant(v); } }
        if let Some(o) = options { if !o.is_empty() { keymap_builder.options(o); } }


        let keymap = match xkb::Keymap::new_from_names(
            &context,
            &keymap_builder.build(),
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        ) {
            Ok(km) => km,
            Err(_) => {
                tracing::warn!(
                    "Complex XKB keymap (rules='{}', model='{}', layout='{}', variant='{:?}', options='{:?}') could not be loaded. Attempting fallback to 'us' layout only.",
                    rules, model, layout, variant, options
                );
                let fallback_args = xkb::KeymapCompileArgsBuilder::new().layout("us").build();
                xkb::Keymap::new_from_names(&context, &fallback_args, xkb::KEYMAP_COMPILE_NO_FLAGS)
                    .map_err(|_| InputError::XkbConfigError {
                        seat_name: "unknown".to_string(), // Seat name not available here, might need to pass or use a generic
                        message: "Fallback XKB Keymap (us) could not be compiled".into()
                    })?
            }
        };

        let state = xkb::State::new(&keymap);
        Ok(Self {
            context, keymap, state,
            repeat_timer: None,
            repeat_info: None,
            focused_surface_on_seat: None,
            repeat_key_serial: None,
        })
    }
}

// Add a simple Debug impl manually if TimerHandle is the issue
impl std::fmt::Debug for XkbKeyboardData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XkbKeyboardData")
            // .field("context", &self.context) // xkb::Context might not be Debug
            // .field("keymap", &self.keymap)   // xkb::Keymap might not be Debug
            // .field("state", &self.state)     // xkb::State might not be Debug
            .field("repeat_timer_is_some", &self.repeat_timer.is_some())
            .field("repeat_info", &self.repeat_info)
            .field("focused_surface_on_seat_is_some", &self.focused_surface_on_seat.is_some())
            .field("repeat_key_serial", &self.repeat_key_serial)
            .finish()
    }
}

use xkbcommon::xkb as xkb_common_xkb; // Alias to avoid conflict if xkb is already in scope
use smithay::input::keyboard::ModifiersState as SmithayModifiersState;

pub fn update_xkb_state_with_smithay_modifiers(
    xkb_state: &mut xkb_common_xkb::State,
    modifiers_state: &SmithayModifiersState
) -> bool {
    xkb_state.update_mask(
        modifiers_state.depressed,
        modifiers_state.latched,
        modifiers_state.locked,
        modifiers_state.layout_effective, // effective layout index for group
        modifiers_state.layout_locked,    // locked layout index
        modifiers_state.layout_latched,   // latched layout index
    )
}
