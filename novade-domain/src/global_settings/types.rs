use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    Light,
    Dark,
    AutoSystem,
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme::AutoSystem
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub active_theme_name: String,
    pub color_scheme: ColorScheme,
    pub enable_animations: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            active_theme_name: "default_light_theme".to_string(),
            color_scheme: ColorScheme::default(),
            enable_animations: true,
        }
    }
}

impl AppearanceSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.active_theme_name.is_empty() {
            return Err("Active theme name cannot be empty.".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputBehaviorSettings {
    pub mouse_sensitivity: f32,
    pub natural_scrolling_touchpad: bool,
}

impl Default for InputBehaviorSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            natural_scrolling_touchpad: true,
        }
    }
}

impl InputBehaviorSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.mouse_sensitivity <= 0.0 {
            return Err("Mouse sensitivity must be greater than 0.".to_string());
        }
        if self.mouse_sensitivity > 10.0 {
            return Err("Mouse sensitivity is too high (max 10.0).".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontHinting {
    None,
    Slight,
    Medium,
    Full,
}

impl Default for FontHinting {
    fn default() -> Self {
        FontHinting::Medium
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontAntialiasing {
    None,
    Grayscale,
    SubpixelRgba, // (e.g. LCD screens)
}

impl Default for FontAntialiasing {
    fn default() -> Self {
        FontAntialiasing::Grayscale
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            default_font_family: "system-ui".to_string(),
            default_font_size: 10, // Typically points
            monospace_font_family: "monospace".to_string(),
            document_font_family: "sans-serif".to_string(),
            hinting: FontHinting::default(),
            antialiasing: FontAntialiasing::default(),
        }
    }
}

impl FontSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.default_font_family.is_empty() {
            return Err("Default font family cannot be empty.".to_string());
        }
        if self.default_font_size < 6 || self.default_font_size > 36 {
            return Err("Default font size must be between 6 and 36.".to_string());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceSwitchingBehavior {
    FollowMouse, // Switch workspace when mouse reaches screen edge
    RequiresClick, // Click on workspace indicator or use shortcut
    ScrollOnDesktop, // Scroll wheel on desktop background
}

impl Default for WorkspaceSwitchingBehavior {
    fn default() -> Self {
        WorkspaceSwitchingBehavior::RequiresClick
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            default_workspace_count: 4,
            workspace_switching_behavior: WorkspaceSwitchingBehavior::default(),
            show_workspace_indicator: true,
        }
    }
}

impl WorkspaceSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.dynamic_workspaces && (self.default_workspace_count < 1 || self.default_workspace_count > 32) {
            return Err("Default workspace count must be between 1 and 32 for static workspaces.".to_string());
        }
        if self.dynamic_workspaces && self.default_workspace_count != 0 { // Allow 0 for truly dynamic initial state
             // Or some other logic if default_workspace_count means "initial count for dynamic"
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LidCloseAction {
    Suspend,
    Hibernate,
    Shutdown,
    DoNothing,
    LockScreen,
}

impl Default for LidCloseAction {
    fn default() -> Self {
        LidCloseAction::Suspend
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerManagementPolicySettings {
    pub screen_blank_timeout_ac_secs: u32,      // 0 means never
    pub screen_blank_timeout_battery_secs: u32, // 0 means never
    pub suspend_action_on_lid_close_ac: LidCloseAction,
    pub suspend_action_on_lid_close_battery: LidCloseAction,
    pub automatic_suspend_delay_ac_secs: u32,      // 0 means never
    pub automatic_suspend_delay_battery_secs: u32, // 0 means never
    pub show_battery_percentage: bool,
}

impl Default for PowerManagementPolicySettings {
    fn default() -> Self {
        Self {
            screen_blank_timeout_ac_secs: 300,      // 5 minutes
            screen_blank_timeout_battery_secs: 180, // 3 minutes
            suspend_action_on_lid_close_ac: LidCloseAction::DoNothing,
            suspend_action_on_lid_close_battery: LidCloseAction::Suspend,
            automatic_suspend_delay_ac_secs: 1800, // 30 minutes
            automatic_suspend_delay_battery_secs: 600, // 10 minutes
            show_battery_percentage: true,
        }
    }
}

impl PowerManagementPolicySettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.screen_blank_timeout_ac_secs > 3600 * 2 { // Max 2 hours
            return Err("Screen blank timeout (AC) cannot exceed 2 hours.".to_string());
        }
        if self.screen_blank_timeout_battery_secs > 3600 { // Max 1 hour
            return Err("Screen blank timeout (Battery) cannot exceed 1 hour.".to_string());
        }
        if self.automatic_suspend_delay_ac_secs > 3600 * 8 { // Max 8 hours
            return Err("Automatic suspend delay (AC) cannot exceed 8 hours.".to_string());
        }
        if self.automatic_suspend_delay_battery_secs > 3600 * 4 { // Max 4 hours
            return Err("Automatic suspend delay (Battery) cannot exceed 4 hours.".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultApplicationsSettings {
    pub web_browser_desktop_file: String,
    pub email_client_desktop_file: String,
    pub terminal_emulator_desktop_file: String,
    pub file_manager_desktop_file: String,
    pub text_editor_desktop_file: String,
}

impl Default for DefaultApplicationsSettings {
    fn default() -> Self {
        Self {
            web_browser_desktop_file: "firefox.desktop".to_string(),
            email_client_desktop_file: "thunderbird.desktop".to_string(),
            terminal_emulator_desktop_file: "gnome-terminal.desktop".to_string(),
            file_manager_desktop_file: "nautilus.desktop".to_string(),
            text_editor_desktop_file: "gedit.desktop".to_string(),
        }
    }
}

impl DefaultApplicationsSettings {
    pub fn validate(&self) -> Result<(), String> {
        let check_desktop_file = |file: &str, app_type: &str| {
            if file.is_empty() {
                // Allow empty if user wants no default or system handles it
                return Ok(());
            }
            if !file.ends_with(".desktop") {
                return Err(format!("{} (value: '{}') must end with .desktop or be empty.", app_type, file));
            }
            // Further validation could involve checking if the file exists via ConfigService,
            // but that's beyond simple struct validation here.
            Ok(())
        };

        check_desktop_file(&self.web_browser_desktop_file, "Web browser")?;
        check_desktop_file(&self.email_client_desktop_file, "Email client")?;
        check_desktop_file(&self.terminal_emulator_desktop_file, "Terminal emulator")?;
        check_desktop_file(&self.file_manager_desktop_file, "File manager")?;
        check_desktop_file(&self.text_editor_desktop_file, "Text editor")?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GlobalDesktopSettings {
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub input_behavior: InputBehaviorSettings,
    #[serde(default)]
    pub font_settings: FontSettings,
    #[serde(default)]
    pub workspace_config: WorkspaceSettings,
    #[serde(default)]
    pub power_management_policy: PowerManagementPolicySettings,
    #[serde(default)]
    pub default_applications: DefaultApplicationsSettings,
}

impl GlobalDesktopSettings {
    pub fn validate_recursive(&self) -> Result<(), String> {
        self.appearance.validate().map_err(|e| format!("Appearance settings error: {}", e))?;
        self.input_behavior.validate().map_err(|e| format!("Input behavior settings error: {}", e))?;
        self.font_settings.validate().map_err(|e| format!("Font settings error: {}", e))?;
        self.workspace_config.validate().map_err(|e| format!("Workspace config error: {}", e))?;
        self.power_management_policy.validate().map_err(|e| format!("Power management policy error: {}", e))?;
        self.default_applications.validate().map_err(|e| format!("Default applications error: {}", e))?;
        Ok(())
    }
}
