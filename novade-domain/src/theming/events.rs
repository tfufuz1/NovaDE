//! Defines events related to the theming system.
//!
//! These events are used to communicate changes in the theme state, allowing different
//! parts of the application (especially the UI) to react accordingly.

use serde::{Deserialize, Serialize};
use crate::theming::types::AppliedThemeState;

/// Event broadcast when the applied theme has changed.
///
/// This event is published by the `ThemingEngine` whenever the active theme,
/// color scheme, accent color, or user token overrides result in a new
/// `AppliedThemeState`. UI components and other services can subscribe to
/// these events to update their appearance or behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeChangedEvent {
    /// The new `AppliedThemeState` that has been activated.
    /// This state contains all the resolved tokens and theme properties
    /// necessary for the UI to render the current theme.
    pub new_state: AppliedThemeState,
}

impl ThemeChangedEvent {
    /// Creates a new `ThemeChangedEvent`.
    ///
    /// # Arguments
    /// * `new_state`: The `AppliedThemeState` that represents the new theme configuration.
    pub fn new(new_state: AppliedThemeState) -> Self {
        Self { new_state }
    }
}
