use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppearanceSettingPath {
    ActiveThemeName,
    ColorScheme,
    EnableAnimations,
}

impl fmt::Display for AppearanceSettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppearanceSettingPath::ActiveThemeName => write!(f, "active_theme_name"),
            AppearanceSettingPath::ColorScheme => write!(f, "color_scheme"),
            AppearanceSettingPath::EnableAnimations => write!(f, "enable_animations"),
        }
    }
}

impl FromStr for AppearanceSettingPath {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active_theme_name" => Ok(AppearanceSettingPath::ActiveThemeName),
            "color_scheme" => Ok(AppearanceSettingPath::ColorScheme),
            "enable_animations" => Ok(AppearanceSettingPath::EnableAnimations),
            _ => Err(format!("Unknown AppearanceSettingPath: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputBehaviorSettingPath {
    MouseSensitivity,
    NaturalScrollingTouchpad,
}

impl fmt::Display for InputBehaviorSettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputBehaviorSettingPath::MouseSensitivity => write!(f, "mouse_sensitivity"),
            InputBehaviorSettingPath::NaturalScrollingTouchpad => write!(f, "natural_scrolling_touchpad"),
        }
    }
}

impl FromStr for InputBehaviorSettingPath {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mouse_sensitivity" => Ok(InputBehaviorSettingPath::MouseSensitivity),
            "natural_scrolling_touchpad" => Ok(InputBehaviorSettingPath::NaturalScrollingTouchpad),
            _ => Err(format!("Unknown InputBehaviorSettingPath: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontSettingPath {
    DefaultFontFamily,
    DefaultFontSize,
    MonospaceFontFamily,
    DocumentFontFamily,
    Hinting,
    Antialiasing,
}

impl fmt::Display for FontSettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontSettingPath::DefaultFontFamily => write!(f, "default_font_family"),
            FontSettingPath::DefaultFontSize => write!(f, "default_font_size"),
            FontSettingPath::MonospaceFontFamily => write!(f, "monospace_font_family"),
            FontSettingPath::DocumentFontFamily => write!(f, "document_font_family"),
            FontSettingPath::Hinting => write!(f, "hinting"),
            FontSettingPath::Antialiasing => write!(f, "antialiasing"),
        }
    }
}

impl FromStr for FontSettingPath {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default_font_family" => Ok(FontSettingPath::DefaultFontFamily),
            "default_font_size" => Ok(FontSettingPath::DefaultFontSize),
            "monospace_font_family" => Ok(FontSettingPath::MonospaceFontFamily),
            "document_font_family" => Ok(FontSettingPath::DocumentFontFamily),
            "hinting" => Ok(FontSettingPath::Hinting),
            "antialiasing" => Ok(FontSettingPath::Antialiasing),
            _ => Err(format!("Unknown FontSettingPath: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkspaceSettingPath {
    DynamicWorkspaces,
    DefaultWorkspaceCount,
    WorkspaceSwitchingBehavior,
    ShowWorkspaceIndicator,
}

impl fmt::Display for WorkspaceSettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkspaceSettingPath::DynamicWorkspaces => write!(f, "dynamic_workspaces"),
            WorkspaceSettingPath::DefaultWorkspaceCount => write!(f, "default_workspace_count"),
            WorkspaceSettingPath::WorkspaceSwitchingBehavior => write!(f, "workspace_switching_behavior"),
            WorkspaceSettingPath::ShowWorkspaceIndicator => write!(f, "show_workspace_indicator"),
        }
    }
}

impl FromStr for WorkspaceSettingPath {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dynamic_workspaces" => Ok(WorkspaceSettingPath::DynamicWorkspaces),
            "default_workspace_count" => Ok(WorkspaceSettingPath::DefaultWorkspaceCount),
            "workspace_switching_behavior" => Ok(WorkspaceSettingPath::WorkspaceSwitchingBehavior),
            "show_workspace_indicator" => Ok(WorkspaceSettingPath::ShowWorkspaceIndicator),
            _ => Err(format!("Unknown WorkspaceSettingPath: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerManagementSettingPath {
    ScreenBlankTimeoutAcSecs,
    ScreenBlankTimeoutBatterySecs,
    SuspendActionOnLidCloseAc,
    SuspendActionOnLidCloseBattery,
    AutomaticSuspendDelayAcSecs,
    AutomaticSuspendDelayBatterySecs,
    ShowBatteryPercentage,
}

impl fmt::Display for PowerManagementSettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PowerManagementSettingPath::ScreenBlankTimeoutAcSecs => write!(f, "screen_blank_timeout_ac_secs"),
            PowerManagementSettingPath::ScreenBlankTimeoutBatterySecs => write!(f, "screen_blank_timeout_battery_secs"),
            PowerManagementSettingPath::SuspendActionOnLidCloseAc => write!(f, "suspend_action_on_lid_close_ac"),
            PowerManagementSettingPath::SuspendActionOnLidCloseBattery => write!(f, "suspend_action_on_lid_close_battery"),
            PowerManagementSettingPath::AutomaticSuspendDelayAcSecs => write!(f, "automatic_suspend_delay_ac_secs"),
            PowerManagementSettingPath::AutomaticSuspendDelayBatterySecs => write!(f, "automatic_suspend_delay_battery_secs"),
            PowerManagementSettingPath::ShowBatteryPercentage => write!(f, "show_battery_percentage"),
        }
    }
}

impl FromStr for PowerManagementSettingPath {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "screen_blank_timeout_ac_secs" => Ok(PowerManagementSettingPath::ScreenBlankTimeoutAcSecs),
            "screen_blank_timeout_battery_secs" => Ok(PowerManagementSettingPath::ScreenBlankTimeoutBatterySecs),
            "suspend_action_on_lid_close_ac" => Ok(PowerManagementSettingPath::SuspendActionOnLidCloseAc),
            "suspend_action_on_lid_close_battery" => Ok(PowerManagementSettingPath::SuspendActionOnLidCloseBattery),
            "automatic_suspend_delay_ac_secs" => Ok(PowerManagementSettingPath::AutomaticSuspendDelayAcSecs),
            "automatic_suspend_delay_battery_secs" => Ok(PowerManagementSettingPath::AutomaticSuspendDelayBatterySecs),
            "show_battery_percentage" => Ok(PowerManagementSettingPath::ShowBatteryPercentage),
            _ => Err(format!("Unknown PowerManagementSettingPath: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefaultApplicationSettingPath {
    WebBrowserDesktopFile,
    EmailClientDesktopFile,
    TerminalEmulatorDesktopFile,
    FileManagerDesktopFile,
    TextEditorDesktopFile,
}

impl fmt::Display for DefaultApplicationSettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DefaultApplicationSettingPath::WebBrowserDesktopFile => write!(f, "web_browser_desktop_file"),
            DefaultApplicationSettingPath::EmailClientDesktopFile => write!(f, "email_client_desktop_file"),
            DefaultApplicationSettingPath::TerminalEmulatorDesktopFile => write!(f, "terminal_emulator_desktop_file"),
            DefaultApplicationSettingPath::FileManagerDesktopFile => write!(f, "file_manager_desktop_file"),
            DefaultApplicationSettingPath::TextEditorDesktopFile => write!(f, "text_editor_desktop_file"),
        }
    }
}

impl FromStr for DefaultApplicationSettingPath {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "web_browser_desktop_file" => Ok(DefaultApplicationSettingPath::WebBrowserDesktopFile),
            "email_client_desktop_file" => Ok(DefaultApplicationSettingPath::EmailClientDesktopFile),
            "terminal_emulator_desktop_file" => Ok(DefaultApplicationSettingPath::TerminalEmulatorDesktopFile),
            "file_manager_desktop_file" => Ok(DefaultApplicationSettingPath::FileManagerDesktopFile),
            "text_editor_desktop_file" => Ok(DefaultApplicationSettingPath::TextEditorDesktopFile),
            _ => Err(format!("Unknown DefaultApplicationSettingPath: {}", s)),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingPath {
    Appearance(AppearanceSettingPath),
    InputBehavior(InputBehaviorSettingPath),
    Font(FontSettingPath),
    Workspace(WorkspaceSettingPath),
    PowerManagement(PowerManagementSettingPath),
    DefaultApplications(DefaultApplicationSettingPath),
}

impl fmt::Display for SettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingPath::Appearance(path) => write!(f, "appearance.{}", path),
            SettingPath::InputBehavior(path) => write!(f, "input_behavior.{}", path),
            SettingPath::Font(path) => write!(f, "font.{}", path),
            SettingPath::Workspace(path) => write!(f, "workspace.{}", path),
            SettingPath::PowerManagement(path) => write!(f, "power_management.{}", path),
            SettingPath::DefaultApplications(path) => write!(f, "default_applications.{}", path),
        }
    }
}

impl FromStr for SettingPath {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, '.').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid SettingPath format: {}. Expected 'group.setting_name'", s));
        }
        match parts[0] {
            "appearance" => AppearanceSettingPath::from_str(parts[1]).map(SettingPath::Appearance),
            "input_behavior" => InputBehaviorSettingPath::from_str(parts[1]).map(SettingPath::InputBehavior),
            "font" => FontSettingPath::from_str(parts[1]).map(SettingPath::Font),
            "workspace" => WorkspaceSettingPath::from_str(parts[1]).map(SettingPath::Workspace),
            "power_management" => PowerManagementSettingPath::from_str(parts[1]).map(SettingPath::PowerManagement),
            "default_applications" => DefaultApplicationSettingPath::from_str(parts[1]).map(SettingPath::DefaultApplications),
            _ => Err(format!("Unknown setting group: {}", parts[0])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_setting_path_display_from_str() {
        assert_eq!(FontSettingPath::DefaultFontFamily.to_string(), "default_font_family");
        assert_eq!("default_font_family".parse::<FontSettingPath>().unwrap(), FontSettingPath::DefaultFontFamily);
        assert_eq!(FontSettingPath::Antialiasing.to_string(), "antialiasing");
        assert_eq!("antialiasing".parse::<FontSettingPath>().unwrap(), FontSettingPath::Antialiasing);
        assert!("invalid".parse::<FontSettingPath>().is_err());
    }

    #[test]
    fn test_workspace_setting_path_display_from_str() {
        assert_eq!(WorkspaceSettingPath::DynamicWorkspaces.to_string(), "dynamic_workspaces");
        assert_eq!("dynamic_workspaces".parse::<WorkspaceSettingPath>().unwrap(), WorkspaceSettingPath::DynamicWorkspaces);
        assert!("invalid".parse::<WorkspaceSettingPath>().is_err());
    }

    #[test]
    fn test_power_management_setting_path_display_from_str() {
        assert_eq!(PowerManagementSettingPath::ScreenBlankTimeoutAcSecs.to_string(), "screen_blank_timeout_ac_secs");
        assert_eq!("screen_blank_timeout_ac_secs".parse::<PowerManagementSettingPath>().unwrap(), PowerManagementSettingPath::ScreenBlankTimeoutAcSecs);
        assert!("invalid".parse::<PowerManagementSettingPath>().is_err());
    }

    #[test]
    fn test_default_application_setting_path_display_from_str() {
        assert_eq!(DefaultApplicationSettingPath::WebBrowserDesktopFile.to_string(), "web_browser_desktop_file");
        assert_eq!("web_browser_desktop_file".parse::<DefaultApplicationSettingPath>().unwrap(), DefaultApplicationSettingPath::WebBrowserDesktopFile);
        assert!("invalid".parse::<DefaultApplicationSettingPath>().is_err());
    }
    
    #[test]
    fn test_main_setting_path_display_from_str() {
        assert_eq!(SettingPath::Font(FontSettingPath::DefaultFontSize).to_string(), "font.default_font_size");
        assert_eq!("font.default_font_size".parse::<SettingPath>().unwrap(), SettingPath::Font(FontSettingPath::DefaultFontSize));

        assert_eq!(SettingPath::Workspace(WorkspaceSettingPath::DefaultWorkspaceCount).to_string(), "workspace.default_workspace_count");
        assert_eq!("workspace.default_workspace_count".parse::<SettingPath>().unwrap(), SettingPath::Workspace(WorkspaceSettingPath::DefaultWorkspaceCount));
        
        assert_eq!(SettingPath::PowerManagement(PowerManagementSettingPath::ShowBatteryPercentage).to_string(), "power_management.show_battery_percentage");
        assert_eq!("power_management.show_battery_percentage".parse::<SettingPath>().unwrap(), SettingPath::PowerManagement(PowerManagementSettingPath::ShowBatteryPercentage));

        assert_eq!(SettingPath::DefaultApplications(DefaultApplicationSettingPath::TerminalEmulatorDesktopFile).to_string(), "default_applications.terminal_emulator_desktop_file");
        assert_eq!("default_applications.terminal_emulator_desktop_file".parse::<SettingPath>().unwrap(), SettingPath::DefaultApplications(DefaultApplicationSettingPath::TerminalEmulatorDesktopFile));
        
        assert!("font.invalid_setting".parse::<SettingPath>().is_err());
        assert!("unknown_group.setting".parse::<SettingPath>().is_err());
    }
}
