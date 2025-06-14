// novade-system/src/system_settings_service.rs
//! Provides an interface to query and modify system-level settings.
use crate::error::SystemError;

/// Information about a system setting.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemSettingInfo {
    pub group: String, // e.g., "audio", "display", "power"
    pub name: String, // e.g., "master_volume", "screen_brightness", "power_profile"
    pub current_value: Option<String>, // User-friendly representation
    pub value_type: String, // e.g., "percentage", "boolean", "enum", "string"
    pub possible_values: Option<Vec<String>>, // For enum types
    pub value_range: Option<(String, String)>, // For numerical types (min, max)
    pub unit: Option<String>, // e.g., "%", "seconds"
    pub is_readable: bool,
    pub is_writable: bool,
}

pub trait SystemSettingsService: Send + Sync {
    /// Gets a system setting value, returned as a user-friendly string.
    ///
    /// # Arguments
    /// * `setting_group` - The group the setting belongs to (e.g., "audio", "display").
    /// * `setting_name` - The specific name of the setting (e.g., "volume", "brightness").
    ///
    /// # Returns
    /// The current value of the setting as a String, or an error if not found or not readable.
    fn get_setting_value(&self, setting_group: &str, setting_name: &str) -> Result<String, SystemError>;

    /// Sets a system setting value. The input value is a string, which the service
    /// will attempt to parse and apply based on the setting's actual type.
    ///
    /// # Arguments
    /// * `setting_group` - The group of the setting.
    /// * `setting_name` - The name of the setting.
    /// * `value` - The desired value as a string (e.g., "50", "true", "power-saver").
    ///
    /// # Returns
    /// `Ok(())` if the setting was successfully changed, or an error.
    fn set_setting_value(&self, setting_group: &str, setting_name: &str, value: &str) -> Result<(), SystemError>;

    /// Lists available settings, optionally filtered by group.
    /// Provides information about each setting, like its type, current value, and if it's writable.
    ///
    /// # Arguments
    /// * `group_filter` - Optional filter to list settings only for a specific group.
    ///                  If None, lists settings from all relevant groups.
    ///
    /// # Returns
    /// A vector of `SystemSettingInfo` objects, or an error.
    fn list_configurable_settings(&self, group_filter: Option<&str>) -> Result<Vec<SystemSettingInfo>, SystemError>;

    // TODO: Consider more specific methods if generic string parsing is too broad, e.g.:
    // fn set_volume(&self, channel: Option<&str>, level_percent: u8) -> Result<(), SystemError>;
    // fn set_brightness(&self, display_id: Option<&str>, level_percent: u8) -> Result<(), SystemError>;
}

// TODO: Assistant Integration: The Smart Assistant will use this to handle commands like
// "Set volume to 50%" or "Increase brightness".
// TODO: This service would act as a facade, potentially interacting with various system D-Bus services
// (e.g., PulseAudio/PipeWire for audio, UPower for power, backend-specific display brightness controls)
// or direct OS calls where appropriate.
// TODO: Define SystemError variants for setting-specific errors (e.g., SettingNotFound, InvalidValueForSetting, WriteFailed).
