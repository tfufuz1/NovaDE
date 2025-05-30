// novade-system/src/input/keyboard_layout.rs

use smithay::input::keyboard::{XkbConfig, Context, Keymap, RuleNames};
use anyhow::{Result, Context as AnyhowContext};

/// Manages keyboard layouts and XKB configuration.
pub struct KeyboardLayoutManager {
    xkb_config: XkbConfig,
    // keymap: Keymap, // The keymap could be stored if needed for direct queries
}

impl KeyboardLayoutManager {
    /// Creates a new `KeyboardLayoutManager` with default XKB settings.
    ///
    /// Default settings typically include:
    /// - Rules: "evdev"
    /// - Model: "pc105"
    /// - Layout: "us"
    /// - Variant: ""
    /// - Options: None
    ///
    /// These can be customized by creating `XkbConfig` with different `RuleNames`.
    pub fn new() -> Result<Self> {
        // Define default XKB rule names. These are common defaults.
        // TODO: Make these configurable through settings in a later iteration.
        let rules = RuleNames {
            rules: Some("evdev".to_string()), // Or "base"
            model: Some("pc105".to_string()), // Or "pc104", "macbook79", etc.
            layout: Some("us".to_string()),   // e.g., "us", "de", "fr"
            variant: Some("".to_string()),    // e.g., "dvorak", "colemak"
            options: None,                    // e.g., "ctrl:nocaps"
        };

        tracing::info!(
            "Initializing KeyboardLayoutManager with XKB rules: rules='{}', model='{}', layout='{}', variant='{}', options='{:?}'",
            rules.rules.as_deref().unwrap_or("default"),
            rules.model.as_deref().unwrap_or("default"),
            rules.layout.as_deref().unwrap_or("default"),
            rules.variant.as_deref().unwrap_or("default"),
            rules.options.as_deref().unwrap_or_default()
        );

        // Create XkbConfig. Smithay's KeyboardHandle will use this.
        // XkbConfig itself doesn't compile the keymap immediately.
        // The keymap is compiled when a keyboard is added to a seat with this config.
        let xkb_config = XkbConfig {
            names: rules,
            // Other fields like `keymap_string` for pre-compiled keymaps are None by default.
            ..Default::default()
        };

        // One could try to compile the keymap here to validate, but it's often
        // done implicitly by Smithay when the keyboard is created.
        // let context = Context::new().context("Failed to create XKB context")?;
        // let keymap = Keymap::new_from_names(
        //     &context,
        //     &rules.rules.as_deref().unwrap_or(""),
        //     &rules.model.as_deref().unwrap_or(""),
        //     &rules.layout.as_deref().unwrap_or(""),
        //     &rules.variant.as_deref().unwrap_or(""),
        //     rules.options.clone(), // Assuming options is Option<String>
        //     KeymapFormat::TextV1
        // ).context("Failed to compile initial keymap from names")?;
        // tracing::info!("Successfully compiled initial keymap for validation.");

        Ok(Self {
            xkb_config,
            // keymap,
        })
    }

    /// Returns a reference to the XKB configuration.
    /// This config can be used when adding a keyboard to a Smithay seat.
    pub fn xkb_config(&self) -> &XkbConfig {
        &self.xkb_config
    }

    /// Returns a clone of the XKB configuration.
    /// Useful if the config needs to be owned by the keyboard handler.
    pub fn xkb_config_cloned(&self) -> XkbConfig {
        self.xkb_config.clone()
    }

    // TODO: Add methods to change layout at runtime if needed.
    // This would involve creating a new XkbConfig and instructing Smithay's
    // KeyboardHandle to use the new keymap.
    // pub fn set_layout(&mut self, layout_names: RuleNames) -> Result<()> { ... }
}

impl Default for KeyboardLayoutManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default KeyboardLayoutManager")
    }
}
