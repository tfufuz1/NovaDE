//! Bridges the domain-layer theming system with the GTK UI.
//!
//! This module provides `GtkThemeManager`, which is responsible for:
//! - Listening to theme changes from `novade_domain::theming::ThemingEngine`.
//! - Generating CSS from the `AppliedThemeState`.
//! - Applying this CSS to the GTK application using a `gtk::CssProvider`.

use gtk::{gdk, glib, prelude::*, CssProvider, StyleContext};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use novade_domain::theming::{
    events::ThemeChangedEvent as DomainThemeChangedEvent, // Alias to avoid conflict if we had a local one
    types::{AppliedThemeState, ColorSchemeType, TokenIdentifier}, // Added TokenIdentifier for tests
    ThemingEngine,
};
// Added for tests
use novade_core::types::Color as CoreColor;
use std::collections::BTreeMap;


/// Generates a CSS string from a given `AppliedThemeState`.
///
/// The generated CSS defines CSS custom properties (variables) under the `:root` selector,
/// allowing them to be used throughout the GTK application's CSS stylesheets.
/// For example, a token `color.background.primary` with value `#ffffff` becomes
/// `--color-background-primary: #ffffff;`.
///
/// If an `active_accent_color` is present in the `new_state`, a `--accent-color`
/// CSS variable is also defined with its hex value.
///
/// This function is public primarily for testability but is used internally by `GtkThemeManager`.
///
/// # Arguments
/// * `new_state`: The `AppliedThemeState` containing resolved tokens and theme properties.
///
/// # Returns
/// A `String` containing the generated CSS.
pub fn generate_css_from_theme_state(new_state: &AppliedThemeState) -> String {
    let mut css_vars = String::new();
    css_vars.push_str(":root {\n");

    for (token_id, value) in &new_state.resolved_tokens {
        let css_var_name = token_id.as_str().replace(".", "-");
        css_vars.push_str(&format!("    --{}: {};\n", css_var_name, value));
    }

    if let Some(accent_color) = &new_state.active_accent_color {
        css_vars.push_str(&format!(
            "    --accent-color: {};\n",
            accent_color.value.to_hex_string()
        ));
    }

    // It might be useful to expose the scheme as a variable too, or a class on :root
    // For example:
    // css_vars.push_str(&format!("    --color-scheme: {};\n",
    //     match new_state.color_scheme {
    //         ColorSchemeType::Light => "light",
    //         ColorSchemeType::Dark => "dark",
    //     }
    // ));

    css_vars.push_str("}\n");

    // Optionally, add classes to :root or other global styles based on scheme
    // Example:
    // let scheme_class = match new_state.color_scheme {
    //     ColorSchemeType::Light => ".theme-light",
    //     ColorSchemeType::Dark => ".theme-dark",
    // };
    // css_vars.push_str(&format!("{} {{\n /* Scheme specific global styles if any */ \n}}\n", scheme_class));
    // This is generally better handled by the application's main CSS file using the variables from :root.

    css_vars
}


/// Manages the application of themes to the GTK UI.
///
/// `GtkThemeManager` acts as a bridge between the domain-layer `ThemingEngine` and
/// GTK's styling system. It performs the following key functions:
///
/// - **Holds a reference** to the `novade_domain::theming::ThemingEngine` to access
///   the current theme state and subscribe to theme updates.
/// - **Manages a `gtk::CssProvider`**: This provider is used to apply the generated
///   theme styles (primarily CSS custom properties) to the entire GTK application.
/// - **Initial Theme Application**: When initialized (via `initialize_for_display`),
///   it fetches the current theme state from the `ThemingEngine` and applies it.
/// - **Dynamic Updates**: It subscribes to `ThemeChangedEvent`s from the `ThemingEngine`.
///   Upon receiving an event, it regenerates the CSS from the new `AppliedThemeState`
///   and updates the `CssProvider`, causing the GTK UI to reflect the changes.
///
/// The generated CSS primarily consists of CSS custom properties (e.g., `--token-name: value;`)
/// defined under `:root`, making them globally available in the application's stylesheets.
pub struct GtkThemeManager {
    theming_engine: Arc<ThemingEngine>,
    css_provider: CssProvider,
}

impl GtkThemeManager {
    /// Creates a new `GtkThemeManager`.
    ///
    /// # Arguments
    /// * `theming_engine`: An `Arc` pointing to the application's `ThemingEngine` instance.
    ///
    /// # Returns
    /// A new `GtkThemeManager` instance.
    pub fn new(theming_engine: Arc<ThemingEngine>) -> Self {
        let css_provider = CssProvider::new();
        Self {
            theming_engine,
            css_provider,
        }
    }

    /// Initializes the `GtkThemeManager` for a given `gdk::Display`.
    ///
    /// This method should be called once when the GTK application starts up.
    /// It performs the following actions:
    /// 1. Adds the manager's internal `gtk::CssProvider` to the specified `display`.
    ///    This makes the CSS properties defined by this provider available to all widgets
    ///    on that display.
    /// 2. Spawns a local task to load the initial theme from the `ThemingEngine` and
    ///    apply it using the `CssProvider`.
    /// 3. Spawns another local task that subscribes to `ThemeChangedEvent`s from the
    ///    `ThemingEngine`. When new events are received, it updates the `CssProvider`
    ///    with the new theme's CSS.
    ///
    /// # Arguments
    /// * `display`: The `gdk::Display` to which the theme's CSS provider should be attached.
    ///   This is typically obtained from the main application window or `gdk::Display::default()`.
    pub fn initialize_for_display(&self, display: &gdk::Display) {
        info!("Initializing GtkThemeManager for display.");
        StyleContext::add_provider_for_display(
            display,
            &self.css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let engine_clone = Arc::clone(&self.theming_engine);
        let css_provider_clone = self.css_provider.clone();
        glib::MainContext::default().spawn_local(async move {
            Self::load_and_apply_current_theme_internal(engine_clone, css_provider_clone).await;
        });

        let mut theme_event_rx = self.theming_engine.subscribe_to_theme_changes();
        let css_provider_clone_for_updates = self.css_provider.clone();

        glib::MainContext::default().spawn_local(async move {
            info!("Listening for theme changes from domain ThemingEngine.");
            loop {
                match theme_event_rx.recv().await {
                    Ok(domain_event) => {
                        info!("Received ThemeChangedEvent from domain.");
                        Self::apply_theme_state_to_css_internal(
                            &css_provider_clone_for_updates,
                            domain_event.new_state,
                        );
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("ThemingEngine event channel closed. No longer listening for theme updates.");
                        break;
                    }
                    Err(e) => {
                        error!("Error receiving theme change event: {:?}. May have missed updates.", e);
                    }
                }
            }
        });
    }

    /// Asynchronously loads the current theme state from the `ThemingEngine`
    /// and applies it to the provided `CssProvider`.
    ///
    /// This is an internal helper typically called during initialization.
    async fn load_and_apply_current_theme_internal(
        theming_engine: Arc<ThemingEngine>,
        css_provider: CssProvider,
    ) {
        debug!("Loading and applying current theme internally.");
        let applied_state = theming_engine.get_current_theme_state().await;
        Self::apply_theme_state_to_css_internal(&css_provider, applied_state);
    }

    /// Applies the CSS generated from `new_state` to the given `CssProvider`.
    ///
    /// This is an internal helper called during initial load and when theme updates are received.
    /// It uses `generate_css_from_theme_state` to create the CSS string.
    fn apply_theme_state_to_css_internal(
        css_provider: &CssProvider,
        new_state: AppliedThemeState,
    ) {
        debug!("Applying new theme state to CSS. Theme ID: {}, Scheme: {:?}",
               new_state.theme_id, new_state.color_scheme);

        let css_string = generate_css_from_theme_state(&new_state);

        debug!("Generated CSS variables:\n{}", css_string);
        css_provider.load_from_string(&css_string);
        info!("New theme CSS loaded into GtkCssProvider. Theme: {}, Scheme: {:?}", new_state.theme_id, new_state.color_scheme);
    }
}

use tokio::sync::broadcast;


#[cfg(test)]
mod tests {
    use super::*;
    use novade_domain::theming::types::{ThemeIdentifier, AccentColor};
    use std::collections::BTreeMap;

    #[test]
    fn test_generate_css_from_theme_state_basic() {
        let mut resolved_tokens = BTreeMap::new();
        resolved_tokens.insert(TokenIdentifier::new("color.background"), "#111111".to_string());
        resolved_tokens.insert(TokenIdentifier::new("font.main"), "Sans Serif".to_string());
        resolved_tokens.insert(TokenIdentifier::new("spacing.medium.size"), "10px".to_string());

        let accent_val = CoreColor::from_hex("#FF0000").unwrap(); // Red
        let active_accent = AccentColor { name: Some("Test Red".to_string()), value: accent_val };

        let state = AppliedThemeState {
            theme_id: ThemeIdentifier::new("test-theme"),
            color_scheme: ColorSchemeType::Dark,
            active_accent_color: Some(active_accent),
            resolved_tokens,
        };

        let css = generate_css_from_theme_state(&state);

        assert!(css.contains(":root {"));
        assert!(css.contains("--color-background: #111111;"));
        assert!(css.contains("--font-main: Sans Serif;"));
        assert!(css.contains("--spacing-medium-size: 10px;")); // Dots replaced with hyphens
        assert!(css.contains("--accent-color: #ff0000;")); // Hex should be lowercase
        assert!(css.ends_with("}\n"));

        println!("Generated CSS for basic test:\n{}", css);
    }

    #[test]
    fn test_generate_css_from_theme_state_no_accent() {
        let mut resolved_tokens = BTreeMap::new();
        resolved_tokens.insert(TokenIdentifier::new("color.text"), "#eeeeee".to_string());

        let state = AppliedThemeState {
            theme_id: ThemeIdentifier::new("no-accent-theme"),
            color_scheme: ColorSchemeType::Light,
            active_accent_color: None,
            resolved_tokens,
        };

        let css = generate_css_from_theme_state(&state);
        assert!(css.contains("--color-text: #eeeeee;"));
        assert!(!css.contains("--accent-color:"));
        println!("Generated CSS for no accent test:\n{}", css);
    }

    #[test]
    fn test_css_provider_loading_integration() {
        // Ensure GTK is initialized for tests that use GTK components like CssProvider
        gtk::test_init();

        let mut resolved_tokens = BTreeMap::new();
        resolved_tokens.insert(TokenIdentifier::new("color.primary"), "blue".to_string());
        resolved_tokens.insert(TokenIdentifier::new("size.padding"), "5px".to_string());

        let accent_val = CoreColor::from_hex("#00FF00").unwrap(); // Green
        let active_accent = AccentColor { name: Some("Test Green".to_string()), value: accent_val };

        let sample_state = AppliedThemeState {
            theme_id: ThemeIdentifier::new("provider-test-theme"),
            color_scheme: ColorSchemeType::Dark,
            active_accent_color: Some(active_accent),
            resolved_tokens,
        };

        let css_provider = CssProvider::new();
        // Directly call the static function for CSS generation
        let css_to_load = generate_css_from_theme_state(&sample_state);

        css_provider.load_from_string(&css_to_load);

        let provider_css_output = css_provider.to_string();

        // GTK's CssProvider might reformat the CSS slightly (e.g., newlines, spacing),
        // so we check for containment of key properties rather than exact string match.
        assert!(provider_css_output.contains("--color-primary: blue;"));
        assert!(provider_css_output.contains("--size-padding: 5px;"));
        assert!(provider_css_output.contains("--accent-color: #00ff00;"));

        // Check if :root is present (GTK might make it more specific, e.g. `*:root`)
        assert!(provider_css_output.contains("root {")); // GTK might not show :root but just the rules

        println!("CssProvider output:\n{}", provider_css_output);
        // Example of what GTK might output (it's often more compact or specific):
        // "* {\n --color-primary: blue;\n --size-padding: 5px;\n --accent-color: #00ff00;\n}\n"
        // Or it might be very close to the input. The key is that the properties are there.
    }
}
