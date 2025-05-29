use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

// --- Error for FromStr ---
#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
pub enum SettingPathParseError {
    #[error("Invalid path string format: '{0}'")]
    InvalidFormat(String),
    #[error("Unknown path segment: '{segment}' in path '{path_str}'")]
    UnknownSegment { segment: String, path_str: String },
    #[error("Incomplete path: '{0}'")]
    IncompletePath(String),
}

// --- Sub-Path Enums ---

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
        write!(f, "{}", match self {
            FontSettingPath::DefaultFontFamily => "default-font-family",
            FontSettingPath::DefaultFontSize => "default-font-size",
            FontSettingPath::MonospaceFontFamily => "monospace-font-family",
            FontSettingPath::DocumentFontFamily => "document-font-family",
            FontSettingPath::Hinting => "hinting",
            FontSettingPath::Antialiasing => "antialiasing",
        })
    }
}

impl FromStr for FontSettingPath {
    type Err = SettingPathParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default-font-family" => Ok(FontSettingPath::DefaultFontFamily),
            "default-font-size" => Ok(FontSettingPath::DefaultFontSize),
            "monospace-font-family" => Ok(FontSettingPath::MonospaceFontFamily),
            "document-font-family" => Ok(FontSettingPath::DocumentFontFamily),
            "hinting" => Ok(FontSettingPath::Hinting),
            "antialiasing" => Ok(FontSettingPath::Antialiasing),
            _ => Err(SettingPathParseError::UnknownSegment { segment: s.to_string(), path_str: s.to_string() }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppearanceSettingPath {
    ActiveThemeName,
    ColorScheme,
    AccentColorToken,
    FontSettings(FontSettingPath),
    IconThemeName,
    CursorThemeName,
    EnableAnimations,
    InterfaceScalingFactor,
}

impl fmt::Display for AppearanceSettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppearanceSettingPath::ActiveThemeName => write!(f, "active-theme-name"),
            AppearanceSettingPath::ColorScheme => write!(f, "color-scheme"),
            AppearanceSettingPath::AccentColorToken => write!(f, "accent-color-token"),
            AppearanceSettingPath::FontSettings(sub_path) => write!(f, "font-settings.{}", sub_path),
            AppearanceSettingPath::IconThemeName => write!(f, "icon-theme-name"),
            AppearanceSettingPath::CursorThemeName => write!(f, "cursor-theme-name"),
            AppearanceSettingPath::EnableAnimations => write!(f, "enable-animations"),
            AppearanceSettingPath::InterfaceScalingFactor => write!(f, "interface-scaling-factor"),
        }
    }
}

impl FromStr for AppearanceSettingPath {
    type Err = SettingPathParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '.');
        let current_segment = parts.next().unwrap_or(""); 
        let rest = parts.next();

        match current_segment {
            "active-theme-name" if rest.is_none() => Ok(AppearanceSettingPath::ActiveThemeName),
            "color-scheme" if rest.is_none() => Ok(AppearanceSettingPath::ColorScheme),
            "accent-color-token" if rest.is_none() => Ok(AppearanceSettingPath::AccentColorToken),
            "font-settings" => {
                let sub_path_str = rest.ok_or_else(|| SettingPathParseError::IncompletePath(s.to_string()))?;
                Ok(AppearanceSettingPath::FontSettings(FontSettingPath::from_str(sub_path_str).map_err(|e| match e {
                    SettingPathParseError::UnknownSegment { segment, .. } => SettingPathParseError::UnknownSegment { segment, path_str: s.to_string() },
                    _ => SettingPathParseError::IncompletePath(s.to_string()), 
                })?))
            }
            "icon-theme-name" if rest.is_none() => Ok(AppearanceSettingPath::IconThemeName),
            "cursor-theme-name" if rest.is_none() => Ok(AppearanceSettingPath::CursorThemeName),
            "enable-animations" if rest.is_none() => Ok(AppearanceSettingPath::EnableAnimations),
            "interface-scaling-factor" if rest.is_none() => Ok(AppearanceSettingPath::InterfaceScalingFactor),
            _ => Err(SettingPathParseError::UnknownSegment { segment: current_segment.to_string(), path_str: s.to_string() }),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceSettingPath {
    DynamicWorkspaces,
    DefaultWorkspaceCount,
    WorkspaceSwitchingBehavior,
    ShowWorkspaceIndicator,
}

impl fmt::Display for WorkspaceSettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            WorkspaceSettingPath::DynamicWorkspaces => "dynamic-workspaces",
            WorkspaceSettingPath::DefaultWorkspaceCount => "default-workspace-count",
            WorkspaceSettingPath::WorkspaceSwitchingBehavior => "workspace-switching-behavior",
            WorkspaceSettingPath::ShowWorkspaceIndicator => "show-workspace-indicator",
        })
    }
}

impl FromStr for WorkspaceSettingPath {
    type Err = SettingPathParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dynamic-workspaces" => Ok(WorkspaceSettingPath::DynamicWorkspaces),
            "default-workspace-count" => Ok(WorkspaceSettingPath::DefaultWorkspaceCount),
            "workspace-switching-behavior" => Ok(WorkspaceSettingPath::WorkspaceSwitchingBehavior),
            "show-workspace-indicator" => Ok(WorkspaceSettingPath::ShowWorkspaceIndicator),
            _ => Err(SettingPathParseError::UnknownSegment { segment: s.to_string(), path_str: s.to_string() }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputBehaviorSettingPath {
    MouseAccelerationProfile,
    CustomMouseAccelerationFactor,
    MouseSensitivity,
    NaturalScrollingMouse,
    NaturalScrollingTouchpad,
    TapToClickTouchpad,
    TouchpadPointerSpeed,
    KeyboardRepeatDelayMs,
    KeyboardRepeatRateCps,
}

impl fmt::Display for InputBehaviorSettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            InputBehaviorSettingPath::MouseAccelerationProfile => "mouse-acceleration-profile",
            InputBehaviorSettingPath::CustomMouseAccelerationFactor => "custom-mouse-acceleration-factor",
            InputBehaviorSettingPath::MouseSensitivity => "mouse-sensitivity",
            InputBehaviorSettingPath::NaturalScrollingMouse => "natural-scrolling-mouse",
            InputBehaviorSettingPath::NaturalScrollingTouchpad => "natural-scrolling-touchpad",
            InputBehaviorSettingPath::TapToClickTouchpad => "tap-to-click-touchpad",
            InputBehaviorSettingPath::TouchpadPointerSpeed => "touchpad-pointer-speed",
            InputBehaviorSettingPath::KeyboardRepeatDelayMs => "keyboard-repeat-delay-ms",
            InputBehaviorSettingPath::KeyboardRepeatRateCps => "keyboard-repeat-rate-cps",
        })
    }
}

impl FromStr for InputBehaviorSettingPath {
    type Err = SettingPathParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mouse-acceleration-profile" => Ok(InputBehaviorSettingPath::MouseAccelerationProfile),
            "custom-mouse-acceleration-factor" => Ok(InputBehaviorSettingPath::CustomMouseAccelerationFactor),
            "mouse-sensitivity" => Ok(InputBehaviorSettingPath::MouseSensitivity),
            "natural-scrolling-mouse" => Ok(InputBehaviorSettingPath::NaturalScrollingMouse),
            "natural-scrolling-touchpad" => Ok(InputBehaviorSettingPath::NaturalScrollingTouchpad),
            "tap-to-click-touchpad" => Ok(InputBehaviorSettingPath::TapToClickTouchpad),
            "touchpad-pointer-speed" => Ok(InputBehaviorSettingPath::TouchpadPointerSpeed),
            "keyboard-repeat-delay-ms" => Ok(InputBehaviorSettingPath::KeyboardRepeatDelayMs),
            "keyboard-repeat-rate-cps" => Ok(InputBehaviorSettingPath::KeyboardRepeatRateCps),
            _ => Err(SettingPathParseError::UnknownSegment { segment: s.to_string(), path_str: s.to_string() }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PowerManagementPolicySettingPath {
    ScreenBlankTimeoutAcSecs,
    ScreenBlankTimeoutBatterySecs,
    SuspendActionOnLidCloseAc,
    SuspendActionOnLidCloseBattery,
    AutomaticSuspendDelayAcSecs,
    AutomaticSuspendDelayBatterySecs,
    ShowBatteryPercentage,
}

impl fmt::Display for PowerManagementPolicySettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            PowerManagementPolicySettingPath::ScreenBlankTimeoutAcSecs => "screen-blank-timeout-ac-secs",
            PowerManagementPolicySettingPath::ScreenBlankTimeoutBatterySecs => "screen-blank-timeout-battery-secs",
            PowerManagementPolicySettingPath::SuspendActionOnLidCloseAc => "suspend-action-on-lid-close-ac",
            PowerManagementPolicySettingPath::SuspendActionOnLidCloseBattery => "suspend-action-on-lid-close-battery",
            PowerManagementPolicySettingPath::AutomaticSuspendDelayAcSecs => "automatic-suspend-delay-ac-secs",
            PowerManagementPolicySettingPath::AutomaticSuspendDelayBatterySecs => "automatic-suspend-delay-battery-secs",
            PowerManagementPolicySettingPath::ShowBatteryPercentage => "show-battery-percentage",
        })
    }
}

impl FromStr for PowerManagementPolicySettingPath {
    type Err = SettingPathParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "screen-blank-timeout-ac-secs" => Ok(PowerManagementPolicySettingPath::ScreenBlankTimeoutAcSecs),
            "screen-blank-timeout-battery-secs" => Ok(PowerManagementPolicySettingPath::ScreenBlankTimeoutBatterySecs),
            "suspend-action-on-lid-close-ac" => Ok(PowerManagementPolicySettingPath::SuspendActionOnLidCloseAc),
            "suspend-action-on-lid-close-battery" => Ok(PowerManagementPolicySettingPath::SuspendActionOnLidCloseBattery),
            "automatic-suspend-delay-ac-secs" => Ok(PowerManagementPolicySettingPath::AutomaticSuspendDelayAcSecs),
            "automatic-suspend-delay-battery-secs" => Ok(PowerManagementPolicySettingPath::AutomaticSuspendDelayBatterySecs),
            "show-battery-percentage" => Ok(PowerManagementPolicySettingPath::ShowBatteryPercentage),
            _ => Err(SettingPathParseError::UnknownSegment { segment: s.to_string(), path_str: s.to_string() }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DefaultApplicationsSettingPath {
    WebBrowser,
    EmailClient,
    TerminalEmulator,
    FileManager,
    MusicPlayer,
    VideoPlayer,
    ImageViewer,
    TextEditor,
}

impl fmt::Display for DefaultApplicationsSettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            DefaultApplicationsSettingPath::WebBrowser => "web-browser",
            DefaultApplicationsSettingPath::EmailClient => "email-client",
            DefaultApplicationsSettingPath::TerminalEmulator => "terminal-emulator",
            DefaultApplicationsSettingPath::FileManager => "file-manager",
            DefaultApplicationsSettingPath::MusicPlayer => "music-player",
            DefaultApplicationsSettingPath::VideoPlayer => "video-player",
            DefaultApplicationsSettingPath::ImageViewer => "image-viewer",
            DefaultApplicationsSettingPath::TextEditor => "text-editor",
        })
    }
}

impl FromStr for DefaultApplicationsSettingPath {
    type Err = SettingPathParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "web-browser" => Ok(DefaultApplicationsSettingPath::WebBrowser),
            "email-client" => Ok(DefaultApplicationsSettingPath::EmailClient),
            "terminal-emulator" => Ok(DefaultApplicationsSettingPath::TerminalEmulator),
            "file-manager" => Ok(DefaultApplicationsSettingPath::FileManager),
            "music-player" => Ok(DefaultApplicationsSettingPath::MusicPlayer),
            "video-player" => Ok(DefaultApplicationsSettingPath::VideoPlayer),
            "image-viewer" => Ok(DefaultApplicationsSettingPath::ImageViewer),
            "text-editor" => Ok(DefaultApplicationsSettingPath::TextEditor),
            _ => Err(SettingPathParseError::UnknownSegment { segment: s.to_string(), path_str: s.to_string() }),
        }
    }
}


// --- Main SettingPath Enum ---

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SettingPath {
    Root, // For errors related to the entire settings object
    AppearanceRoot,
    WorkspacesRoot,
    InputBehaviorRoot,
    PowerManagementPolicyRoot,
    DefaultApplicationsRoot,
    Appearance(AppearanceSettingPath),
    Workspaces(WorkspaceSettingPath),
    InputBehavior(InputBehaviorSettingPath),
    PowerManagementPolicy(PowerManagementPolicySettingPath),
    DefaultApplications(DefaultApplicationsSettingPath),
}

impl fmt::Display for SettingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingPath::Root => write!(f, "root"),
            SettingPath::AppearanceRoot => write!(f, "appearance"),
            SettingPath::WorkspacesRoot => write!(f, "workspaces"),
            SettingPath::InputBehaviorRoot => write!(f, "input-behavior"),
            SettingPath::PowerManagementPolicyRoot => write!(f, "power-management-policy"),
            SettingPath::DefaultApplicationsRoot => write!(f, "default-applications"),
            SettingPath::Appearance(sub_path) => write!(f, "appearance.{}", sub_path),
            SettingPath::Workspaces(sub_path) => write!(f, "workspaces.{}", sub_path),
            SettingPath::InputBehavior(sub_path) => write!(f, "input-behavior.{}", sub_path),
            SettingPath::PowerManagementPolicy(sub_path) => write!(f, "power-management-policy.{}", sub_path),
            SettingPath::DefaultApplications(sub_path) => write!(f, "default-applications.{}", sub_path),
        }
    }
}

impl FromStr for SettingPath {
    type Err = SettingPathParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "root" { return Ok(SettingPath::Root); }
        if s == "appearance" { return Ok(SettingPath::AppearanceRoot); }
        if s == "workspaces" { return Ok(SettingPath::WorkspacesRoot); }
        if s == "input-behavior" { return Ok(SettingPath::InputBehaviorRoot); }
        if s == "power-management-policy" { return Ok(SettingPath::PowerManagementPolicyRoot); }
        if s == "default-applications" { return Ok(SettingPath::DefaultApplicationsRoot); }
        
        let mut parts = s.splitn(2, '.');
        let top_level_segment = parts.next().ok_or_else(|| SettingPathParseError::InvalidFormat(s.to_string()))?;
        let rest = parts.next().ok_or_else(|| SettingPathParseError::IncompletePath(s.to_string()))?;

        match top_level_segment {
            "appearance" => Ok(SettingPath::Appearance(AppearanceSettingPath::from_str(rest).map_err(|e| match e {
                SettingPathParseError::UnknownSegment { segment, .. } => SettingPathParseError::UnknownSegment { segment, path_str: s.to_string() },
                _ => SettingPathParseError::IncompletePath(s.to_string()),
            })?)),
            "workspaces" => Ok(SettingPath::Workspaces(WorkspaceSettingPath::from_str(rest).map_err(|e| match e {
                 SettingPathParseError::UnknownSegment { segment, .. } => SettingPathParseError::UnknownSegment { segment, path_str: s.to_string() },
                _ => SettingPathParseError::IncompletePath(s.to_string()),
            })?)),
            "input-behavior" => Ok(SettingPath::InputBehavior(InputBehaviorSettingPath::from_str(rest).map_err(|e| match e {
                 SettingPathParseError::UnknownSegment { segment, .. } => SettingPathParseError::UnknownSegment { segment, path_str: s.to_string() },
                _ => SettingPathParseError::IncompletePath(s.to_string()),
            })?)),
            "power-management-policy" => Ok(SettingPath::PowerManagementPolicy(PowerManagementPolicySettingPath::from_str(rest).map_err(|e| match e {
                 SettingPathParseError::UnknownSegment { segment, .. } => SettingPathParseError::UnknownSegment { segment, path_str: s.to_string() },
                _ => SettingPathParseError::IncompletePath(s.to_string()),
            })?)),
            "default-applications" => Ok(SettingPath::DefaultApplications(DefaultApplicationsSettingPath::from_str(rest).map_err(|e| match e {
                 SettingPathParseError::UnknownSegment { segment, .. } => SettingPathParseError::UnknownSegment { segment, path_str: s.to_string() },
                _ => SettingPathParseError::IncompletePath(s.to_string()),
            })?)),
            _ => Err(SettingPathParseError::UnknownSegment { segment: top_level_segment.to_string(), path_str: s.to_string() }),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_setting_path_display() {
        assert_eq!(FontSettingPath::DefaultFontSize.to_string(), "default-font-size");
    }

    #[test]
    fn test_font_setting_path_from_str() {
        assert_eq!("default-font-family".parse::<FontSettingPath>().unwrap(), FontSettingPath::DefaultFontFamily);
    }

    #[test]
    fn test_appearance_setting_path_display() {
        let font_path = AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize);
        assert_eq!(font_path.to_string(), "font-settings.default-font-size");
    }

    #[test]
    fn test_appearance_setting_path_from_str() {
        let parsed_font_path: AppearanceSettingPath = "font-settings.hinting".parse().unwrap();
        assert_eq!(parsed_font_path, AppearanceSettingPath::FontSettings(FontSettingPath::Hinting));
    }
    
    #[test]
    fn test_setting_path_root_variants_display_and_from_str() {
        assert_eq!(SettingPath::Root.to_string(), "root");
        assert_eq!("root".parse::<SettingPath>().unwrap(), SettingPath::Root);

        assert_eq!(SettingPath::AppearanceRoot.to_string(), "appearance");
        assert_eq!("appearance".parse::<SettingPath>().unwrap(), SettingPath::AppearanceRoot);
        
        assert_eq!(SettingPath::WorkspacesRoot.to_string(), "workspaces");
        assert_eq!("workspaces".parse::<SettingPath>().unwrap(), SettingPath::WorkspacesRoot);
    }


    #[test]
    fn test_setting_path_display() {
        let path = SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontFamily));
        assert_eq!(path.to_string(), "appearance.font-settings.default-font-family");
    }

    #[test]
    fn test_setting_path_from_str_valid() {
        let path_str = "appearance.font-settings.default-font-size";
        let expected = SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize));
        assert_eq!(path_str.parse::<SettingPath>().unwrap(), expected);
    }

    #[test]
    fn test_setting_path_from_str_invalid() {
        assert!("appearance.font-settings".parse::<SettingPath>().is_err()); // Incomplete if sub-path not provided
        assert!("invalid-top-level.some-setting".parse::<SettingPath>().is_err());
    }

    #[test]
    fn test_path_serde_roundtrip_detailed() {
        let path = SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontFamily));
        let serialized = serde_json::to_string(&path).unwrap();
        let expected_json = r#"{"appearance":{"font-settings":"default-font-family"}}"#;
        assert_eq!(serialized, expected_json);
        let deserialized: SettingPath = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, path);
    }
    
    #[test]
    fn test_path_serde_roundtrip_root() {
        let path_root = SettingPath::Root;
        let ser_root = serde_json::to_string(&path_root).unwrap();
        assert_eq!(ser_root, r#""root""#); // Unit variant with rename_all
        let de_root: SettingPath = serde_json::from_str(&ser_root).unwrap();
        assert_eq!(de_root, path_root);

        let path_app_root = SettingPath::AppearanceRoot;
        let ser_app_root = serde_json::to_string(&path_app_root).unwrap();
        assert_eq!(ser_app_root, r#""appearance-root""#); // kebab-case
        let de_app_root: SettingPath = serde_json::from_str(&ser_app_root).unwrap();
        assert_eq!(de_app_root, path_app_root);
    }
}
