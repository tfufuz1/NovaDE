use crate::input::errors::InputError; // Adjusted path
use smithay::input::keyboard::{KeyboardConfig, ModifiersState as SmithayModifiersState};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::Weak; // For Weak<WlSurface>
use smithay::utils::Serial;
use xkbcommon::xkb::{self, LedMask}; // Ensure LedMask is imported
use calloop::TimerHandle; // For key repeat

#[derive(Debug)] // Note: TimerHandle is not Debug. Consider custom Debug impl or removing if not needed for XkbKeyboardData directly.
pub struct XkbKeyboardData {
    pub context: xkb::Context,
    pub keymap: xkb::Keymap,
    pub state: xkb::State,
    pub active_leds: LedMask, // ADDED: To track current LED state
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
                    "XKB keymap compilation failed for (rules='{}', model='{}', layout='{}', variant='{:?}', options='{:?}'): {:?}. Attempting fallback.",
                    rules, model, layout, variant, options, e
                );
                tracing::warn!("{}", error_message); // Changed to warn as it's not fatal yet
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
            active_leds: LedMask::empty(), // Initialize with no active LEDs
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

use smithay::input::KeyboardHandle; // For KeyboardHandle
use crate::compositor::core::state::DesktopState; // Path to DesktopState
use std::sync::{Arc, Mutex}; // For accessing XkbKeyboardData in DesktopState

#[allow(dead_code)] // This function will be called by a higher-level service (e.g., settings via D-Bus)
pub fn update_keyboard_keymap(
    desktop_state: &mut DesktopState,
    seat_name: &str,
    new_config: KeyboardConfig<'_>, // Lifetime might need to be 'static if stored long-term, or clone data
) -> Result<(), InputError> {
    tracing::info!(
        "Attempting to update keymap for seat '{}' with new config: {:?}",
        seat_name,
        new_config
    );

    let seat = desktop_state
        .seat_state
        .seats()
        .find(|s| s.name() == seat_name)
        .cloned()
        .ok_or_else(|| {
            tracing::error!("Seat '{}' not found for keymap update.", seat_name);
            InputError::SeatNotFound(seat_name.to_string())
        })?;

    let keyboard_handle: KeyboardHandle<DesktopState> = seat.get_keyboard().ok_or_else(|| {
        tracing::warn!(
            "Kein Keyboard-Handle für Seat '{}' beim Versuch, Keymap zu aktualisieren.",
            seat_name
        );
        InputError::KeyboardHandleNotFound(seat_name.to_string())
    })?;

    let xkb_data_arc_mutex = desktop_state
        .keyboard_data_map
        .get(seat_name)
        .cloned()
        .ok_or_else(|| {
            tracing::error!("Keine XKB-Daten für Seat '{}' beim Keymap-Update.", seat_name);
            InputError::XkbConfigError {
                seat_name: seat_name.to_string(),
                message: "XKB data not found for seat.".to_string(),
            }
        })?;
    
    // Create new XKB keymap and state from the new_config
    // This part is similar to XkbKeyboardData::new()
    let new_context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS); // Can reuse context if it's cheap, or create new
    
    let rules = new_config.rules.as_deref().unwrap_or("evdev");
    let model = new_config.model.as_deref().unwrap_or("pc105");
    let layout = new_config.layout.as_deref().unwrap_or("us");
    let variant = new_config.variant.as_deref();
    let options = new_config.options.as_deref();

    let mut compile_args = xkb::KeymapCompileArgs::new();
    compile_args.rules(rules);
    compile_args.model(model);
    compile_args.layout(layout);
    if let Some(v) = variant { compile_args.variant(v); }
    if let Some(o) = options { compile_args.options(o); }

    let new_keymap = match xkb::Keymap::new_from_names(&new_context, &compile_args, xkb::KEYMAP_COMPILE_NO_FLAGS) {
        Ok(km) => km,
        Err(e) => {
            let error_message = format!("Failed to compile new XKB keymap: {:?}. Error: {:?}", compile_args, e);
            tracing::error!("{}", error_message);
            return Err(InputError::XkbKeymapCompilationFailed(error_message));
        }
    };
    let new_xkb_state = xkb::State::new(&new_keymap);

    // Update Smithay's KeyboardHandle with the new keymap.
    // This will trigger sending the new keymap to clients.
    // Smithay's set_keymap_from_rules is convenient here.
    if let Err(smithay_err) = keyboard_handle.set_keymap_from_rules(rules, model, layout, variant, options) {
        let error_message = format!("Failed to set new keymap in Smithay's KeyboardHandle: {}. Config: {:?}", smithay_err, new_config);
        tracing::error!("{}", error_message);
        // Decide if this is a fatal error for our internal state update too.
        // If Smithay fails, clients won't get the new map, so our internal state might diverge.
        // It might be better to return error here and not update our XkbKeyboardData.
        return Err(InputError::XkbConfigError {
            seat_name: seat_name.to_string(),
            message: error_message,
        });
    }

    // If Smithay's update was successful, update our internal XkbKeyboardData
    {
        let mut xkb_data_guard = xkb_data_arc_mutex.lock().unwrap();
        xkb_data_guard.context = new_context; // If context is per-keymap
        xkb_data_guard.keymap = new_keymap;
        xkb_data_guard.state = new_xkb_state;
        // Reset LED state as it might change with new keymap/state
        xkb_data_guard.active_leds = LedMask::empty(); 
        // KeyboardHandle::update_led_state will be called on next key event if LEDs differ.
        // Or, we can query new_xkb_state.leds() and call update_led_state here.
        let current_leds_mask = xkb_data_guard.state.leds();
        xkb_data_guard.active_leds = current_leds_mask;
        keyboard_handle.update_led_state(current_leds_mask);

        tracing::info!("Keymap für Seat '{}' erfolgreich aktualisiert.", seat_name);
    }
    
    // The KeyboardHandle, after set_keymap_from_rules, should automatically send
    // the wl_keyboard.keymap event to clients.

    Ok(())
}
