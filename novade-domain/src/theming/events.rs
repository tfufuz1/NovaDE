use serde::{Deserialize, Serialize};
use crate::theming::types::AppliedThemeState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeChangedEvent {
    pub new_state: AppliedThemeState,
}

impl ThemeChangedEvent {
    pub fn new(new_state: AppliedThemeState) -> Self {
        Self { new_state }
    }
}
