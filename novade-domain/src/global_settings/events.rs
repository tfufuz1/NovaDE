use crate::global_settings::paths::SettingPath;
use crate::global_settings::types::GlobalDesktopSettings;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingChangedEvent {
    pub path: SettingPath,
    pub new_value: JsonValue,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SettingsSavedEvent {
    // Empty marker struct
}

impl SettingsSavedEvent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for SettingsSavedEvent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_settings::paths::{AppearanceSettingPath, FontSettingPath};
    use serde_json;

    #[test]
    fn test_setting_changed_event_serialization() {
        let path = SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize));
        let value = JsonValue::from(12);
        let event = SettingChangedEvent::new(path.clone(), value.clone());

        let serialized = serde_json::to_string(&event).unwrap();
        // Expected: {"path":{"appearance":{"font-settings":"default-font-size"}},"new_value":12}
        assert!(serialized.contains(r#""path":{"appearance":{"font-settings":"default-font-size"}}"#));
        assert!(serialized.contains(r#""new_value":12"#));

        let deserialized: SettingChangedEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.path, path);
        assert_eq!(deserialized.new_value, value);
    }

    #[test]
    fn test_settings_loaded_event_serialization() {
        let settings = GlobalDesktopSettings::default();
        let event = SettingsLoadedEvent::new(settings.clone());

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains(r#""active_theme_name":"novade-default""#)); 
        
        let deserialized: SettingsLoadedEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.settings, settings);
    }

    #[test]
    fn test_settings_saved_event_serialization() {
        let event = SettingsSavedEvent::new();
        let serialized = serde_json::to_string(&event).unwrap();
        assert_eq!(serialized, "{}");

        let deserialized: SettingsSavedEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, event);
    }
}
