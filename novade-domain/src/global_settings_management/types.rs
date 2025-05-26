use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    #[default]
    Light,
    Dark,
    System, // Follows system preference if detectable
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FontHinting {
    None,
    Slight,
    Medium,
    #[default]
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FontAntialiasing {
    None,
    #[default]
    Grayscale,
    Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MouseAccelerationProfile {
    #[default]
    Adaptive,
    Flat,
    Custom, // Note: custom_mouse_acceleration_factor is a separate field
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LidCloseAction {
    Suspend,
    Hibernate,
    Shutdown,
    #[default]
    DoNothing,
    LockScreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceSwitchingBehavior {
    #[default]
    FollowMouse,
    CurrentScreen,
    AllScreens,
}


// --- Nested Settings Structs ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct FontSettings {
    pub default_font_family: String,
    pub default_font_size: f64, // e.g., 10.0 for 10pt
    pub monospace_font_family: String,
    pub document_font_family: String,
    pub hinting: FontHinting,
    pub antialiasing: FontAntialiasing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct AppearanceSettings {
    pub active_theme_name: String, // e.g., "Adwaita-dark" or custom theme ID
    pub color_scheme: ColorScheme,
    pub accent_color_token: String, // e.g., "color-accent-blue", refers to a token in the theme
    pub font_settings: FontSettings,
    pub icon_theme_name: String,
    pub cursor_theme_name: String,
    pub enable_animations: bool,
    pub interface_scaling_factor: f64, // e.g., 1.0, 1.5, 2.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct WorkspaceSettings {
    pub dynamic_workspaces: bool,
    pub default_workspace_count: u8,
    pub workspace_switching_behavior: WorkspaceSwitchingBehavior,
    pub show_workspace_indicator: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct InputBehaviorSettings {
    pub mouse_acceleration_profile: MouseAccelerationProfile,
    pub custom_mouse_acceleration_factor: Option<f32>, // Only used if profile is Custom
    pub mouse_sensitivity: f32, // e.g., 0.0 to 1.0
    pub natural_scrolling_mouse: bool,
    pub natural_scrolling_touchpad: bool,
    pub tap_to_click_touchpad: bool,
    pub touchpad_pointer_speed: f32, // e.g., 0.0 to 1.0
    pub keyboard_repeat_delay_ms: u32,
    pub keyboard_repeat_rate_cps: u32, // Characters per second
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct PowerManagementPolicySettings {
    pub screen_blank_timeout_ac_secs: u32, // 0 for never
    pub screen_blank_timeout_battery_secs: u32, // 0 for never
    pub suspend_action_on_lid_close_ac: LidCloseAction,
    pub suspend_action_on_lid_close_battery: LidCloseAction,
    pub automatic_suspend_delay_ac_secs: u32, // 0 for never
    pub automatic_suspend_delay_battery_secs: u32, // 0 for never
    pub show_battery_percentage: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct DefaultApplicationsSettings {
    pub web_browser_desktop_file: String, // e.g., "firefox.desktop"
    pub email_client_desktop_file: String,
    pub terminal_emulator_desktop_file: String,
    pub file_manager_desktop_file: String,
    pub music_player_desktop_file: String,
    pub video_player_desktop_file: String,
    pub image_viewer_desktop_file: String,
    pub text_editor_desktop_file: String,
}


// --- Top-Level Settings Struct ---

/// Holds a collection of key-value settings for a specific application.
///
/// The keys are strings, and values are generic `serde_json::Value` types,
/// allowing applications to store diverse setting structures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct ApplicationSettingGroup {
    /// The actual settings for the application, stored as a map from setting key to JSON value.
    pub settings: HashMap<String, serde_json::Value>,
}

/// Represents all global desktop settings, including appearance, workspace configuration,
/// input behaviors, power management policies, default applications, and
/// configurations for individual applications.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct GlobalDesktopSettings {
    pub appearance: AppearanceSettings,
    pub workspace: WorkspaceSettings,
    pub input_behavior: InputBehaviorSettings,
    pub power_management_policy: PowerManagementPolicySettings,
    pub default_applications: DefaultApplicationsSettings,
    /// Stores settings for individual applications, keyed by a unique application identifier
    /// (e.g., "com.example.texteditor" or "org.novade.filemanager").
    pub application_settings: HashMap<String, ApplicationSettingGroup>,
    // pub notifications: NotificationSettings, // Example for future extension
    // pub privacy: PrivacySettings,           // Example for future extension
}

// --- Validation Methods ---

impl ApplicationSettingGroup {
    pub fn validate(&self) -> Result<(), String> {
        for key in self.settings.keys() {
            if key.is_empty() {
                return Err("Application setting key cannot be empty.".to_string());
            }
        }
        Ok(())
    }
}

impl FontSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.default_font_family.is_empty() {
            return Err("Default font family darf nicht leer sein.".to_string());
        }
        if self.default_font_size <= 0.0 {
            return Err("Default font size muss größer als 0 sein.".to_string());
        }
        if self.monospace_font_family.is_empty() {
            return Err("Monospace font family darf nicht leer sein.".to_string());
        }
        if self.document_font_family.is_empty() {
            return Err("Document font family darf nicht leer sein.".to_string());
        }
        Ok(())
    }
}

// --- Event Structs ---

/// Event dispatched when a specific setting has changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingChangedEvent {
    pub path: super::paths::SettingPath, // Use the SettingPath from the paths module
    pub new_value: serde_json::Value,
}

/// Event dispatched when a full set of settings has been loaded (e.g., on initial load or reset).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsLoadedEvent {
    pub settings: GlobalDesktopSettings,
}

/// Event dispatched when settings have been successfully saved to persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)] // Added Default
pub struct SettingsSavedEvent; // Empty struct, just a notification

// --- Global Event Enum ---

/// Represents any event that can occur within the global settings management system.
///
/// This enum aggregates specific event types, allowing subscribers to listen for a broader
/// category of setting-related changes or a specific one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GlobalSettingsEvent {
    /// Indicates that a specific setting value has changed.
    SettingChanged(SettingChangedEvent),
    /// Indicates that a complete set of settings has been loaded.
    /// This is often dispatched on application startup or when settings are reset.
    SettingsLoaded(SettingsLoadedEvent),
    /// Confirms that the current settings state has been successfully persisted.
    SettingsSaved(SettingsSavedEvent),
}

impl AppearanceSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.active_theme_name.is_empty() {
            return Err("Active theme name darf nicht leer sein.".to_string());
        }
        if self.accent_color_token.is_empty() {
            return Err("Accent color token darf nicht leer sein.".to_string());
        }
        self.font_settings.validate().map_err(|e| format!("Font settings: {}", e))?;
        if self.icon_theme_name.is_empty() {
            return Err("Icon theme name darf nicht leer sein.".to_string());
        }
        if self.cursor_theme_name.is_empty() {
            return Err("Cursor theme name darf nicht leer sein.".to_string());
        }
        if self.interface_scaling_factor <= 0.0 {
            return Err("Interface scaling factor muss größer als 0 sein.".to_string());
        }
        // Example: Allow common scaling factors like 1.0, 1.25, 1.5, 1.75, 2.0, etc.
        // Or a reasonable range e.g., 0.5 to 3.0
        if self.interface_scaling_factor < 0.5 || self.interface_scaling_factor > 3.0 {
             return Err("Interface scaling factor sollte zwischen 0.5 und 3.0 liegen.".to_string());
        }
        Ok(())
    }
}

impl WorkspaceSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.default_workspace_count == 0 {
            return Err("Default workspace count muss mindestens 1 sein.".to_string());
        }
        if self.default_workspace_count > 32 { // Arbitrary upper limit
            return Err("Default workspace count ist unrealistisch hoch.".to_string());
        }
        Ok(())
    }
}

impl InputBehaviorSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.mouse_acceleration_profile == MouseAccelerationProfile::Custom && self.custom_mouse_acceleration_factor.is_none() {
            return Err("Custom mouse acceleration factor muss angegeben werden, wenn das Profil 'Custom' ist.".to_string());
        }
        if self.mouse_acceleration_profile != MouseAccelerationProfile::Custom && self.custom_mouse_acceleration_factor.is_some() {
            return Err("Custom mouse acceleration factor sollte nur angegeben werden, wenn das Profil 'Custom' ist.".to_string());
        }
        if !(0.0..=2.0).contains(&self.mouse_sensitivity) { // Example range
            return Err("Mouse sensitivity muss zwischen 0.0 und 2.0 liegen.".to_string());
        }
        if !(0.0..=2.0).contains(&self.touchpad_pointer_speed) { // Example range
            return Err("Touchpad pointer speed muss zwischen 0.0 und 2.0 liegen.".to_string());
        }
        if self.keyboard_repeat_delay_ms < 100 || self.keyboard_repeat_delay_ms > 2000 {
            return Err("Keyboard repeat delay sollte zwischen 100ms und 2000ms liegen.".to_string());
        }
        if self.keyboard_repeat_rate_cps < 1 || self.keyboard_repeat_rate_cps > 100 {
            return Err("Keyboard repeat rate sollte zwischen 1cps und 100cps liegen.".to_string());
        }
        Ok(())
    }
}

impl PowerManagementPolicySettings {
    pub fn validate(&self) -> Result<(), String> {
        // Timeouts can be 0 (never), so no lower bound check beyond non-negative (implicit in u32).
        // Check for excessively large values if necessary.
        Ok(())
    }
}

fn validate_desktop_file(file_name: &str, field_name: &str) -> Result<(), String> {
    if file_name.is_empty() {
        return Err(format!("{} darf nicht leer sein.", field_name));
    }
    if !file_name.ends_with(".desktop") && !file_name.ends_with(".desktop") { // Small typo, fixed
         return Err(format!("{} ('{}') sollte auf '.desktop' enden.", field_name, file_name));
    }
    // A more thorough check might involve checking for invalid path characters,
    // but for now, non-empty and .desktop suffix is a good start.
    Ok(())
}

impl DefaultApplicationsSettings {
    pub fn validate(&self) -> Result<(), String> {
        validate_desktop_file(&self.web_browser_desktop_file, "Web browser desktop file")?;
        validate_desktop_file(&self.email_client_desktop_file, "Email client desktop file")?;
        validate_desktop_file(&self.terminal_emulator_desktop_file, "Terminal emulator desktop file")?;
        validate_desktop_file(&self.file_manager_desktop_file, "File manager desktop file")?;
        validate_desktop_file(&self.music_player_desktop_file, "Music player desktop file")?;
        validate_desktop_file(&self.video_player_desktop_file, "Video player desktop file")?;
        validate_desktop_file(&self.image_viewer_desktop_file, "Image viewer desktop file")?;
        validate_desktop_file(&self.text_editor_desktop_file, "Text editor desktop file")?;
        Ok(())
    }
}

impl GlobalDesktopSettings {
    /// Validates all nested settings structures.
    pub fn validate_recursive(&self) -> Result<(), String> {
        self.appearance.validate().map_err(|e| format!("Appearance settings: {}", e))?;
        self.workspace.validate().map_err(|e| format!("Workspace settings: {}", e))?;
        self.input_behavior.validate().map_err(|e| format!("Input behavior settings: {}", e))?;
        self.power_management_policy.validate().map_err(|e| format!("Power management policy settings: {}", e))?;
        self.default_applications.validate().map_err(|e| format!("Default applications settings: {}", e))?;
        for (app_id, app_setting_group) in &self.application_settings {
            app_setting_group.validate().map_err(|e| format!("Application settings for '{}': {}", app_id, e))?;
        }
        Ok(())
    }
}
