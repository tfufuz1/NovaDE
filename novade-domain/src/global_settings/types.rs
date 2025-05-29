use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::paths::SettingPath; // For validate_recursive
use super::errors::GlobalSettingsError; // For validate_recursive

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    Light,
    Dark,
    SystemPreference,
}

impl Default for ColorScheme {
    fn default() -> Self { ColorScheme::SystemPreference }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FontHinting {
    None,
    Slight,
    Medium,
    Full,
}

impl Default for FontHinting {
    fn default() -> Self { FontHinting::Slight }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FontAntialiasing {
    None,
    Grayscale,
    Rgba,
}

impl Default for FontAntialiasing {
    fn default() -> Self { FontAntialiasing::Grayscale }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceSwitchingBehavior {
    FollowMouse,
    CurrentScreen,
    PrimaryScreen,
}

impl Default for WorkspaceSwitchingBehavior {
    fn default() -> Self { WorkspaceSwitchingBehavior::FollowMouse }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MouseAccelerationProfile {
    Adaptive,
    Flat,
    Custom,
}

impl Default for MouseAccelerationProfile {
    fn default() -> Self { MouseAccelerationProfile::Adaptive }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LidCloseAction {
    Suspend,
    Hibernate,
    Shutdown,
    DoNothing,
}

impl Default for LidCloseAction {
    fn default() -> Self { LidCloseAction::Suspend }
}

// --- Nested Settings Structs ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FontSettings {
    pub default_font_family: String,
    pub default_font_size: u8,
    pub monospace_font_family: String,
    pub document_font_family: String,
    pub hinting: FontHinting,
    pub antialiasing: FontAntialiasing,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            default_font_family: "Cantarell".to_string(),
            default_font_size: 11,
            monospace_font_family: "Monospace".to_string(),
            document_font_family: "Sans-Serif".to_string(),
            hinting: FontHinting::default(),
            antialiasing: FontAntialiasing::default(),
        }
    }
}

impl FontSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.default_font_size < 6 || self.default_font_size > 24 {
            return Err(format!("Default font size {} is out of range (6-24).", self.default_font_size));
        }
        if self.default_font_family.is_empty() {
            return Err("Default font family cannot be empty.".to_string());
        }
        if self.monospace_font_family.is_empty() {
            return Err("Monospace font family cannot be empty.".to_string());
        }
        if self.document_font_family.is_empty() {
            return Err("Document font family cannot be empty.".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppearanceSettings {
    pub active_theme_name: String,
    pub color_scheme: ColorScheme,
    pub accent_color_token: String,
    #[serde(default)]
    pub font_settings: FontSettings,
    pub icon_theme_name: String,
    pub cursor_theme_name: String,
    pub enable_animations: bool,
    pub interface_scaling_factor: f64,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            active_theme_name: "novade-default".to_string(),
            color_scheme: ColorScheme::default(),
            accent_color_token: "blue.500".to_string(),
            font_settings: FontSettings::default(),
            icon_theme_name: "Adwaita".to_string(),
            cursor_theme_name: "Adwaita".to_string(),
            enable_animations: true,
            interface_scaling_factor: 1.0,
        }
    }
}

impl AppearanceSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.active_theme_name.is_empty() {
            return Err("Active theme name cannot be empty.".to_string());
        }
        if self.accent_color_token.is_empty() {
            return Err("Accent color token cannot be empty.".to_string());
        }
        self.font_settings.validate().map_err(|e| format!("Font settings validation failed: {}", e))?;
        if self.icon_theme_name.is_empty() {
            return Err("Icon theme name cannot be empty.".to_string());
        }
        if self.cursor_theme_name.is_empty() {
            return Err("Cursor theme name cannot be empty.".to_string());
        }
        if !(0.5..=3.0).contains(&self.interface_scaling_factor) {
            return Err(format!("Interface scaling factor {} is out of range (0.5-3.0).", self.interface_scaling_factor));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct WorkspaceSettings {
    pub dynamic_workspaces: bool,
    pub default_workspace_count: u8,
    pub workspace_switching_behavior: WorkspaceSwitchingBehavior,
    pub show_workspace_indicator: bool,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            dynamic_workspaces: true,
            default_workspace_count: 1,
            workspace_switching_behavior: WorkspaceSwitchingBehavior::default(),
            show_workspace_indicator: true,
        }
    }
}

impl WorkspaceSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.dynamic_workspaces && (self.default_workspace_count == 0 || self.default_workspace_count > 32) {
            return Err(format!("Default workspace count {} is out of range (1-32) for fixed workspaces.", self.default_workspace_count));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InputBehaviorSettings {
    pub mouse_acceleration_profile: MouseAccelerationProfile,
    pub custom_mouse_acceleration_factor: Option<f32>,
    pub mouse_sensitivity: f32,
    pub natural_scrolling_mouse: bool,
    pub natural_scrolling_touchpad: bool,
    pub tap_to_click_touchpad: bool,
    pub touchpad_pointer_speed: f32,
    pub keyboard_repeat_delay_ms: u32,
    pub keyboard_repeat_rate_cps: u32,
}

impl Default for InputBehaviorSettings {
    fn default() -> Self {
        Self {
            mouse_acceleration_profile: MouseAccelerationProfile::default(),
            custom_mouse_acceleration_factor: None,
            mouse_sensitivity: 0.0,
            natural_scrolling_mouse: false,
            natural_scrolling_touchpad: true,
            tap_to_click_touchpad: true,
            touchpad_pointer_speed: 0.0,
            keyboard_repeat_delay_ms: 500,
            keyboard_repeat_rate_cps: 30,
        }
    }
}

impl InputBehaviorSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.mouse_acceleration_profile == MouseAccelerationProfile::Custom {
            if self.custom_mouse_acceleration_factor.is_none() {
                return Err("Custom mouse acceleration factor must be set when profile is Custom.".to_string());
            }
            if let Some(factor) = self.custom_mouse_acceleration_factor {
                if !(0.0..=1.0).contains(&factor) {
                     return Err(format!("Custom mouse acceleration factor {} is out of range (0.0-1.0).", factor));
                }
            }
        } else if self.custom_mouse_acceleration_factor.is_some() {
            return Err("Custom mouse acceleration factor should only be set when profile is Custom.".to_string());
        }

        if !(-1.0..=1.0).contains(&self.mouse_sensitivity) {
            return Err(format!("Mouse sensitivity {} is out of range (-1.0 to 1.0).", self.mouse_sensitivity));
        }
        if !(-1.0..=1.0).contains(&self.touchpad_pointer_speed) {
            return Err(format!("Touchpad pointer speed {} is out of range (-1.0 to 1.0).", self.touchpad_pointer_speed));
        }
        if !(100..=2000).contains(&self.keyboard_repeat_delay_ms) {
            return Err(format!("Keyboard repeat delay {}ms is out of range (100-2000).", self.keyboard_repeat_delay_ms));
        }
        if !(5..=100).contains(&self.keyboard_repeat_rate_cps) {
            return Err(format!("Keyboard repeat rate {}cps is out of range (5-100).", self.keyboard_repeat_rate_cps));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PowerManagementPolicySettings {
    pub screen_blank_timeout_ac_secs: u32,
    pub screen_blank_timeout_battery_secs: u32,
    pub suspend_action_on_lid_close_ac: LidCloseAction,
    pub suspend_action_on_lid_close_battery: LidCloseAction,
    pub automatic_suspend_delay_ac_secs: u32,
    pub automatic_suspend_delay_battery_secs: u32,
    pub show_battery_percentage: bool,
}

impl Default for PowerManagementPolicySettings {
    fn default() -> Self {
        Self {
            screen_blank_timeout_ac_secs: 300,
            screen_blank_timeout_battery_secs: 120,
            suspend_action_on_lid_close_ac: LidCloseAction::DoNothing,
            suspend_action_on_lid_close_battery: LidCloseAction::Suspend,
            automatic_suspend_delay_ac_secs: 1800,
            automatic_suspend_delay_battery_secs: 600,
            show_battery_percentage: true,
        }
    }
}

impl PowerManagementPolicySettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.screen_blank_timeout_ac_secs > 7200 && self.screen_blank_timeout_ac_secs != 0 {
            return Err("AC screen blank timeout is too high (> 2 hours).".to_string());
        }
        if self.screen_blank_timeout_battery_secs > 7200 && self.screen_blank_timeout_battery_secs != 0 {
            return Err("Battery screen blank timeout is too high (> 2 hours).".to_string());
        }
        if self.automatic_suspend_delay_ac_secs > 86400 && self.automatic_suspend_delay_ac_secs != 0 {
            return Err("AC automatic suspend delay is too high (> 24 hours).".to_string());
        }
        if self.automatic_suspend_delay_battery_secs > 86400 && self.automatic_suspend_delay_battery_secs != 0 {
            return Err("Battery automatic suspend delay is too high (> 24 hours).".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DefaultApplicationsSettings {
    pub web_browser: String,
    pub email_client: String,
    pub terminal_emulator: String,
    pub file_manager: String,
    pub music_player: String,
    pub video_player: String,
    pub image_viewer: String,
    pub text_editor: String,
}

impl Default for DefaultApplicationsSettings {
    fn default() -> Self {
        Self {
            web_browser: "firefox.desktop".to_string(),
            email_client: "thunderbird.desktop".to_string(),
            terminal_emulator: "xterm.desktop".to_string(),
            file_manager: "xdg-open".to_string(), 
            music_player: "".to_string(),
            video_player: "".to_string(),
            image_viewer: "".to_string(),
            text_editor: "gedit.desktop".to_string(),
        }
    }
}

impl DefaultApplicationsSettings {
    pub fn validate(&self) -> Result<(), String> {
        let check_desktop_file = |app_name: &str, field_name: &str| {
            if !app_name.is_empty() && !app_name.ends_with(".desktop") && app_name != "xdg-open" {
                Err(format!("{} '{}' should be a .desktop file name, 'xdg-open', or empty.", field_name, app_name))
            } else {
                Ok(())
            }
        };
        check_desktop_file(&self.web_browser, "Web browser")?;
        check_desktop_file(&self.email_client, "Email client")?;
        check_desktop_file(&self.terminal_emulator, "Terminal emulator")?;
        check_desktop_file(&self.file_manager, "File manager")?;
        check_desktop_file(&self.music_player, "Music player")?;
        check_desktop_file(&self.video_player, "Video player")?;
        check_desktop_file(&self.image_viewer, "Image viewer")?;
        check_desktop_file(&self.text_editor, "Text editor")?;
        Ok(())
    }
}


// --- Main GlobalDesktopSettings Struct ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalDesktopSettings {
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub workspaces: WorkspaceSettings, // Corrected field name from workspace_config
    #[serde(default)]
    pub input_behavior: InputBehaviorSettings,
    #[serde(default)]
    pub power_management_policy: PowerManagementPolicySettings,
    #[serde(default)]
    pub default_applications: DefaultApplicationsSettings,
}

impl GlobalDesktopSettings {
    pub fn validate(&self) -> Result<(), String> { // General validation, not recursive
        self.appearance.validate().map_err(|e| format!("Appearance settings: {}", e))?;
        self.workspaces.validate().map_err(|e| format!("Workspace settings: {}", e))?;
        self.input_behavior.validate().map_err(|e| format!("Input behavior settings: {}", e))?;
        self.power_management_policy.validate().map_err(|e| format!("Power management policy settings: {}", e))?;
        self.default_applications.validate().map_err(|e| format!("Default applications settings: {}", e))?;
        Ok(())
    }

    // New method as per subtask description
    pub fn validate_recursive(&self) -> Result<(), GlobalSettingsError> {
        self.appearance.validate().map_err(|e| GlobalSettingsError::ValidationError { path: SettingPath::AppearanceRoot, reason: e })?;
        self.workspaces.validate().map_err(|e| GlobalSettingsError::ValidationError { path: SettingPath::WorkspacesRoot, reason: e })?; // Corrected field name
        self.input_behavior.validate().map_err(|e| GlobalSettingsError::ValidationError { path: SettingPath::InputBehaviorRoot, reason: e })?;
        self.power_management_policy.validate().map_err(|e| GlobalSettingsError::ValidationError { path: SettingPath::PowerManagementPolicyRoot, reason: e })?;
        self.default_applications.validate().map_err(|e| GlobalSettingsError::ValidationError { path: SettingPath::DefaultApplicationsRoot, reason: e })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Import specific paths for testing validate_recursive if needed, though not directly tested here.
    // use crate::global_settings::paths::{AppearanceSettingPath, FontSettingPath}; 

    #[test]
    fn default_global_settings_are_valid() {
        let settings = GlobalDesktopSettings::default();
        assert!(settings.validate().is_ok(), "Default settings validation failed: {:?}", settings.validate().err());
        assert!(settings.validate_recursive().is_ok(), "Default settings recursive validation failed: {:?}", settings.validate_recursive().err());
    }

    #[test]
    fn font_settings_validation() {
        let mut fs = FontSettings::default();
        fs.default_font_size = 5; 
        assert!(fs.validate().is_err());
        fs.default_font_size = 25; 
        assert!(fs.validate().is_err());
        fs.default_font_size = 10; 
        assert!(fs.validate().is_ok());
        fs.default_font_family = "".to_string(); 
        assert!(fs.validate().is_err());
    }

    #[test]
    fn appearance_settings_validation() {
        let mut apps = AppearanceSettings::default();
        apps.interface_scaling_factor = 0.4; 
        assert!(apps.validate().is_err());
        apps.interface_scaling_factor = 3.1; 
        assert!(apps.validate().is_err());
        apps.interface_scaling_factor = 1.5; 
        assert!(apps.validate().is_ok());
        apps.active_theme_name = "".to_string();
        assert!(apps.validate().is_err());
    }
    
    #[test]
    fn input_behavior_settings_validation() {
        let mut ibs = InputBehaviorSettings::default();
        ibs.mouse_acceleration_profile = MouseAccelerationProfile::Custom;
        ibs.custom_mouse_acceleration_factor = None; 
        assert!(ibs.validate().is_err(), "Should fail if Custom profile has no factor");

        ibs.custom_mouse_acceleration_factor = Some(0.5); 
        assert!(ibs.validate().is_ok());
        
        ibs.custom_mouse_acceleration_factor = Some(1.5); 
        assert!(ibs.validate().is_err(), "Should fail if factor is out of range");

        ibs.mouse_acceleration_profile = MouseAccelerationProfile::Flat;
        ibs.custom_mouse_acceleration_factor = Some(0.5); 
        assert!(ibs.validate().is_err(), "Should fail if factor is set for non-Custom profile");

        ibs.custom_mouse_acceleration_factor = None; 
        assert!(ibs.validate().is_ok());

        ibs.keyboard_repeat_delay_ms = 50; 
        assert!(ibs.validate().is_err());
        ibs.keyboard_repeat_delay_ms = 1000; 
        assert!(ibs.validate().is_ok());
    }

    #[test]
    fn default_apps_settings_validation() {
        let mut das = DefaultApplicationsSettings::default();
        das.web_browser = "firefox".to_string(); 
        assert!(das.validate().is_err());
        das.web_browser = "firefox.desktop".to_string(); 
        assert!(das.validate().is_ok());
        das.music_player = "".to_string(); 
        assert!(das.validate().is_ok());
        das.file_manager = "xdg-open".to_string();
        assert!(das.validate().is_ok());
    }

    #[test]
    fn test_color_scheme_serde() {
        let cs = ColorScheme::Dark;
        let serialized = serde_json::to_string(&cs).unwrap();
        assert_eq!(serialized, "\"dark\"");
        let deserialized: ColorScheme = serde_json::from_str("\"light\"").unwrap();
        assert_eq!(deserialized, ColorScheme::Light);
    }

    #[test]
    fn test_font_settings_serde_default() {
        let fs = FontSettings::default();
        let serialized = serde_json::to_string(&fs).unwrap();
        assert!(serialized.contains("\"default_font_size\":11"));
        let deserialized: FontSettings = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, fs);
    }
    
    #[test]
    fn test_global_desktop_settings_serde_default() {
        let gds = GlobalDesktopSettings::default();
        let serialized = serde_json::to_string(&gds).unwrap();
        assert!(serialized.contains("\"appearance\":"));
        assert!(serialized.contains("\"active_theme_name\":\"novade-default\""));
        let deserialized: GlobalDesktopSettings = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, gds);
    }

    #[test]
    fn test_validate_recursive_appearance_error() {
        let mut settings = GlobalDesktopSettings::default();
        settings.appearance.active_theme_name = "".to_string(); // Invalid
        let result = settings.validate_recursive();
        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::ValidationError { path, reason } => {
                assert_eq!(path, SettingPath::AppearanceRoot);
                assert_eq!(reason, "Active theme name cannot be empty.");
            }
            _ => panic!("Incorrect error type"),
        }
    }

    #[test]
    fn test_validate_recursive_workspace_error() { // Corrected test name
        let mut settings = GlobalDesktopSettings::default();
        settings.workspaces.dynamic_workspaces = false;
        settings.workspaces.default_workspace_count = 0; // Invalid
        let result = settings.validate_recursive();
        assert!(result.is_err());
        match result.err().unwrap() {
            GlobalSettingsError::ValidationError { path, reason } => {
                assert_eq!(path, SettingPath::WorkspacesRoot); // Corrected path
                assert!(reason.contains("Default workspace count"));
            }
            _ => panic!("Incorrect error type"),
        }
    }
}
