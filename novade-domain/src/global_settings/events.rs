use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use super::paths::SettingPath;
use super::types::GlobalDesktopSettings;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingChangedEvent {
    pub path: SettingPath,
    pub new_value: JsonValue, // Represents the new value of the setting
}

impl SettingChangedEvent {
    pub fn new(path: SettingPath, new_value: JsonValue) -> Self {
        Self { path, new_value }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsLoadedEvent {
    pub settings: GlobalDesktopSettings,
}

impl SettingsLoadedEvent {
    pub fn new(settings: GlobalDesktopSettings) -> Self {
        Self { settings }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsSavedEvent {
    pub saved_settings: GlobalDesktopSettings,
}

impl SettingsSavedEvent {
    pub fn new(saved_settings: GlobalDesktopSettings) -> Self {
        Self { saved_settings }
    }
}
