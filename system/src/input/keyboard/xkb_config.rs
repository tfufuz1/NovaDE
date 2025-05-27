use crate::input::errors::InputError; // Adjusted path
use smithay::input::keyboard::{KeyboardConfig, ModifiersState as SmithayModifiersState};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::Weak; // For Weak<WlSurface>
use smithay::utils::Serial;
use xkbcommon::xkb;
use calloop::TimerHandle; // For key repeat

#[derive(Debug)] // Note: TimerHandle is not Debug. Consider custom Debug impl or removing if not needed for XkbKeyboardData directly.
pub struct XkbKeyboardData {
    pub context: xkb::Context,
    pub keymap: xkb::Keymap,
    pub state: xkb::State,
    pub repeat_timer: Option<TimerHandle>, // Timer for key repetition
    // Store necessary info for repeating a key:
    // (libinput keycode, xkb_keycode, current modifiers, initial delay, repeat rate)
    pub repeat_info: Option<(
        u32,                         // libinput_keycode
        xkb::Keycode,                // xkb_keycode
        SmithayModifiersState,       // Modifiers at the time of press
        std::time::Duration,         // Initial delay
        std::time::Duration,         // Repeat rate (interval)
    )>,
    pub focused_surface_on_seat: Option<Weak<WlSurface>>, // Keep track of focused surface for this keyboard
    pub repeat_key_serial: Option<Serial>, // Serial of the key event that triggered repeat
}

impl XkbKeyboardData {
    pub fn new(config: &KeyboardConfig<'_>) -> Result<Self, InputError> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        
        let rules = config.rules.as_deref().unwrap_or("evdev");
        let model = config.model.as_deref().unwrap_or("pc105");
        let layout = config.layout.as_deref().unwrap_or("us");
        let variant = config.variant.as_deref();
        let options = config.options.as_deref();

        tracing::debug!(
            "Lade XKB Keymap: rules='{}', model='{}', layout='{}', variant='{:?}', options='{:?}'",
            rules, model, layout, variant, options
        );
        
        // Use xkb::KeymapCompileArgs for more robust keymap creation
        let mut compile_args = xkb::KeymapCompileArgs::new();
        compile_args.rules(rules);
        compile_args.model(model);
        compile_args.layout(layout);
        if let Some(v) = variant { compile_args.variant(v); }
        if let Some(o) = options { compile_args.options(o); }

        let keymap = match xkb::Keymap::new_from_names(
            &context,
            &compile_args, // Pass the compiled args
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        ) {
            Ok(km) => km,
            Err(e) => {
                let error_message = format!(
                    "XKB keymap compilation failed for rules='{}', model='{}', layout='{}', variant='{:?}', options='{:?}': {:?}",
                    rules, model, layout, variant, options, e
                );
                tracing::error!("{}", error_message);
                // Try a fallback to a very basic "us" layout
                let mut fallback_args = xkb::KeymapCompileArgs::new();
                fallback_args.layout("us");
                match xkb::Keymap::new_from_names(&context, &fallback_args, xkb::KEYMAP_COMPILE_NO_FLAGS) {
                    Ok(fb_km) => {
                        tracing::warn!("Erfolgreich auf XKB Fallback-Keymap 'us' gewechselt.");
                        fb_km
                    },
                    Err(fb_e) => {
                        let fb_error_message = format!("XKB fallback keymap 'us' compilation also failed: {:?}", fb_e);
                        tracing::error!("{}", fb_error_message);
                        return Err(InputError::XkbKeymapCompilationFailed(fb_error_message));
                    }
                }
            }
        };

        let state = xkb::State::new(&keymap);

        Ok(Self {
            context,
            keymap,
            state,
            repeat_timer: None,
            repeat_info: None,
            focused_surface_on_seat: None,
            repeat_key_serial: None,
        })
    }

    // Method to update XKB state based on Smithay's ModifiersState
    // This might be needed if there are other sources of modifier changes.
    // Typically, xkb::State is updated directly from key events.
    #[allow(dead_code)]
    pub fn update_xkb_state_from_smithay_modifiers(&mut self, smithay_mods: &SmithayModifiersState) -> bool {
        self.state.update_mask(
            smithay_mods.depressed,
            smithay_mods.latched,
            smithay_mods.locked,
            0, // base_group (effective layout) - usually derived, not directly set from smithay_mods like this
            0, // latched_group
            0, // locked_group
        ) != xkb::STATE_UPDATE_NO_CHANGE
    }
}
