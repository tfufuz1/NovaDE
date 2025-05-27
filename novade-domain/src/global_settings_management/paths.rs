use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use thiserror::Error; // For FromStr error

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Ungültiger Einstellungs-Pfad: {0}")]
pub struct SettingPathParseError(String);

// --- Nested Path Enums ---

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

impl Display for FontSettingPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl Display for AppearanceSettingPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceSettingPath {
    DynamicWorkspaces,
    DefaultWorkspaceCount,
    WorkspaceSwitchingBehavior,
    ShowWorkspaceIndicator,
}

impl Display for WorkspaceSettingPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            WorkspaceSettingPath::DynamicWorkspaces => "dynamic-workspaces",
            WorkspaceSettingPath::DefaultWorkspaceCount => "default-workspace-count",
            WorkspaceSettingPath::WorkspaceSwitchingBehavior => "workspace-switching-behavior",
            WorkspaceSettingPath::ShowWorkspaceIndicator => "show-workspace-indicator",
        })
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

impl Display for InputBehaviorSettingPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl Display for PowerManagementPolicySettingPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DefaultApplicationsSettingPath {
    WebBrowserDesktopFile,
    EmailClientDesktopFile,
    TerminalEmulatorDesktopFile,
    FileManagerDesktopFile,
    MusicPlayerDesktopFile,
    VideoPlayerDesktopFile,
    ImageViewerDesktopFile,
    TextEditorDesktopFile,
}

impl Display for DefaultApplicationsSettingPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            DefaultApplicationsSettingPath::WebBrowserDesktopFile => "web-browser-desktop-file",
            DefaultApplicationsSettingPath::EmailClientDesktopFile => "email-client-desktop-file",
            DefaultApplicationsSettingPath::TerminalEmulatorDesktopFile => "terminal-emulator-desktop-file",
            DefaultApplicationsSettingPath::FileManagerDesktopFile => "file-manager-desktop-file",
            DefaultApplicationsSettingPath::MusicPlayerDesktopFile => "music-player-desktop-file",
            DefaultApplicationsSettingPath::VideoPlayerDesktopFile => "video-player-desktop-file",
            DefaultApplicationsSettingPath::ImageViewerDesktopFile => "image-viewer-desktop-file",
            DefaultApplicationsSettingPath::TextEditorDesktopFile => "text-editor-desktop-file",
        })
    }
}

// --- Window Management Path Enum ---

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WindowManagementSettingPath {
    TilingMode,
    PlacementStrategy,
    // Gaps
    GapsScreenOuterHorizontal,
    GapsScreenOuterVertical,
    GapsWindowInner,
    // Snapping
    SnappingSnapToScreenEdges,
    SnappingSnapToOtherWindows,
    SnappingSnapToWorkspaceGaps,
    SnappingSnapDistancePx,
    // Focus
    FocusFocusFollowsMouse,
    FocusClickToFocus,
    FocusNewWindowsOnCreation,
    FocusNewWindowsOnWorkspaceSwitch,
    FocusFocusStealingPrevention,
    // Grouping
    GroupingEnableManualGrouping,
}

impl Display for WindowManagementSettingPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            WindowManagementSettingPath::TilingMode => "tiling-mode",
            WindowManagementSettingPath::PlacementStrategy => "placement-strategy",
            WindowManagementSettingPath::GapsScreenOuterHorizontal => "gaps.screen-outer-horizontal",
            WindowManagementSettingPath::GapsScreenOuterVertical => "gaps.screen-outer-vertical",
            WindowManagementSettingPath::GapsWindowInner => "gaps.window-inner",
            WindowManagementSettingPath::SnappingSnapToScreenEdges => "snapping.snap-to-screen-edges",
            WindowManagementSettingPath::SnappingSnapToOtherWindows => "snapping.snap-to-other-windows",
            WindowManagementSettingPath::SnappingSnapToWorkspaceGaps => "snapping.snap-to-workspace-gaps",
            WindowManagementSettingPath::SnappingSnapDistancePx => "snapping.snap-distance-px",
            WindowManagementSettingPath::FocusFocusFollowsMouse => "focus.focus-follows-mouse",
            WindowManagementSettingPath::FocusClickToFocus => "focus.click-to-focus",
            WindowManagementSettingPath::FocusNewWindowsOnCreation => "focus.focus-new-windows-on-creation",
            WindowManagementSettingPath::FocusNewWindowsOnWorkspaceSwitch => "focus.focus-new-windows-on-workspace-switch",
            WindowManagementSettingPath::FocusFocusStealingPrevention => "focus.focus-stealing-prevention",
            WindowManagementSettingPath::GroupingEnableManualGrouping => "grouping.enable-manual-grouping",
        })
    }
}

impl FromStr for WindowManagementSettingPath {
    type Err = SettingPathParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tiling-mode" => Ok(WindowManagementSettingPath::TilingMode),
            "placement-strategy" => Ok(WindowManagementSettingPath::PlacementStrategy),
            "gaps.screen-outer-horizontal" => Ok(WindowManagementSettingPath::GapsScreenOuterHorizontal),
            "gaps.screen-outer-vertical" => Ok(WindowManagementSettingPath::GapsScreenOuterVertical),
            "gaps.window-inner" => Ok(WindowManagementSettingPath::GapsWindowInner),
            "snapping.snap-to-screen-edges" => Ok(WindowManagementSettingPath::SnappingSnapToScreenEdges),
            "snapping.snap-to-other-windows" => Ok(WindowManagementSettingPath::SnappingSnapToOtherWindows),
            "snapping.snap-to-workspace-gaps" => Ok(WindowManagementSettingPath::SnappingSnapToWorkspaceGaps),
            "snapping.snap-distance-px" => Ok(WindowManagementSettingPath::SnappingSnapDistancePx),
            "focus.focus-follows-mouse" => Ok(WindowManagementSettingPath::FocusFocusFollowsMouse),
            "focus.click-to-focus" => Ok(WindowManagementSettingPath::FocusClickToFocus),
            "focus.focus-new-windows-on-creation" => Ok(WindowManagementSettingPath::FocusNewWindowsOnCreation),
            "focus.focus-new-windows-on-workspace-switch" => Ok(WindowManagementSettingPath::FocusNewWindowsOnWorkspaceSwitch),
            "focus.focus-stealing-prevention" => Ok(WindowManagementSettingPath::FocusFocusStealingPrevention),
            "grouping.enable-manual-grouping" => Ok(WindowManagementSettingPath::GroupingEnableManualGrouping),
            _ => Err(SettingPathParseError(format!("Ungültiger WindowManagementSettingPath: {}", s))),
        }
    }
}

// --- Top-Level SettingPath Enum ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ApplicationSettingPath {
    pub app_id: String,
    pub key: String,
}

// Note: We cannot use #[serde(untagged)] for SettingPath easily if we want to keep
// the kebab-case for existing variants and also have a custom format for Application.
// The FromStr and Display implementations will handle the custom "application.<app_id>.<key>" format.
// For direct serialization/deserialization, ApplicationSettingPath itself is kebab-case for its fields if needed,
// but SettingPath::Application will be handled by serde as a variant "application" containing the struct.

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SettingPath {
    Appearance(AppearanceSettingPath),
    Workspace(WorkspaceSettingPath),
    InputBehavior(InputBehaviorSettingPath),
    PowerManagementPolicy(PowerManagementPolicySettingPath),
    DefaultApplications(DefaultApplicationsSettingPath),
    Application(ApplicationSettingPath),
    WindowManagement(WindowManagementSettingPath), // New Variant
}

impl Display for SettingPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SettingPath::Appearance(sub_path) => write!(f, "appearance.{}", sub_path),
            SettingPath::Workspace(sub_path) => write!(f, "workspace.{}", sub_path),
            SettingPath::InputBehavior(sub_path) => write!(f, "input-behavior.{}", sub_path),
            SettingPath::PowerManagementPolicy(sub_path) => write!(f, "power-management-policy.{}", sub_path),
            SettingPath::DefaultApplications(sub_path) => write!(f, "default-applications.{}", sub_path),
            SettingPath::Application(app_path) => write!(f, "application.{}.{}", app_path.app_id, app_path.key),
            SettingPath::WindowManagement(sub_path) => write!(f, "window-management.{}", sub_path),
        }
    }
}

// --- FromStr Implementations ---

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
            _ => Err(SettingPathParseError(format!("Ungültiger FontSettingPath: {}", s))),
        }
    }
}

impl FromStr for AppearanceSettingPath {
    type Err = SettingPathParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((prefix, suffix)) = s.split_once('.') {
            if prefix == "font-settings" {
                return Ok(AppearanceSettingPath::FontSettings(FontSettingPath::from_str(suffix)?));
            }
        }
        match s {
            "active-theme-name" => Ok(AppearanceSettingPath::ActiveThemeName),
            "color-scheme" => Ok(AppearanceSettingPath::ColorScheme),
            "accent-color-token" => Ok(AppearanceSettingPath::AccentColorToken),
            "icon-theme-name" => Ok(AppearanceSettingPath::IconThemeName),
            "cursor-theme-name" => Ok(AppearanceSettingPath::CursorThemeName),
            "enable-animations" => Ok(AppearanceSettingPath::EnableAnimations),
            "interface-scaling-factor" => Ok(AppearanceSettingPath::InterfaceScalingFactor),
            _ => Err(SettingPathParseError(format!("Ungültiger AppearanceSettingPath: {}", s))),
        }
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
            _ => Err(SettingPathParseError(format!("Ungültiger WorkspaceSettingPath: {}", s))),
        }
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
            _ => Err(SettingPathParseError(format!("Ungültiger InputBehaviorSettingPath: {}", s))),
        }
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
            _ => Err(SettingPathParseError(format!("Ungültiger PowerManagementPolicySettingPath: {}", s))),
        }
    }
}

impl FromStr for DefaultApplicationsSettingPath {
    type Err = SettingPathParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "web-browser-desktop-file" => Ok(DefaultApplicationsSettingPath::WebBrowserDesktopFile),
            "email-client-desktop-file" => Ok(DefaultApplicationsSettingPath::EmailClientDesktopFile),
            "terminal-emulator-desktop-file" => Ok(DefaultApplicationsSettingPath::TerminalEmulatorDesktopFile),
            "file-manager-desktop-file" => Ok(DefaultApplicationsSettingPath::FileManagerDesktopFile),
            "music-player-desktop-file" => Ok(DefaultApplicationsSettingPath::MusicPlayerDesktopFile),
            "video-player-desktop-file" => Ok(DefaultApplicationsSettingPath::VideoPlayerDesktopFile),
            "image-viewer-desktop-file" => Ok(DefaultApplicationsSettingPath::ImageViewerDesktopFile),
            "text-editor-desktop-file" => Ok(DefaultApplicationsSettingPath::TextEditorDesktopFile),
            _ => Err(SettingPathParseError(format!("Ungültiger DefaultApplicationsSettingPath: {}", s))),
        }
    }
}

impl FromStr for SettingPath {
    type Err = SettingPathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');
        let top_level = parts.next().ok_or_else(|| SettingPathParseError("Pfad darf nicht leer sein".to_string()))?;

        match top_level {
            "appearance" => {
                let sub_path_str = s.strip_prefix("appearance.").ok_or_else(|| SettingPathParseError("Unvollständiger Appearance-Pfad".to_string()))?;
                Ok(SettingPath::Appearance(AppearanceSettingPath::from_str(sub_path_str)?))
            }
            "workspace" => {
                let sub_path_str = s.strip_prefix("workspace.").ok_or_else(|| SettingPathParseError("Unvollständiger Workspace-Pfad".to_string()))?;
                Ok(SettingPath::Workspace(WorkspaceSettingPath::from_str(sub_path_str)?))
            }
            "input-behavior" => {
                let sub_path_str = s.strip_prefix("input-behavior.").ok_or_else(|| SettingPathParseError("Unvollständiger InputBehavior-Pfad".to_string()))?;
                Ok(SettingPath::InputBehavior(InputBehaviorSettingPath::from_str(sub_path_str)?))
            }
            "power-management-policy" => {
                let sub_path_str = s.strip_prefix("power-management-policy.").ok_or_else(|| SettingPathParseError("Unvollständiger PowerManagementPolicy-Pfad".to_string()))?;
                Ok(SettingPath::PowerManagementPolicy(PowerManagementPolicySettingPath::from_str(sub_path_str)?))
            }
            "default-applications" => {
                let sub_path_str = s.strip_prefix("default-applications.").ok_or_else(|| SettingPathParseError("Unvollständiger DefaultApplications-Pfad".to_string()))?;
                Ok(SettingPath::DefaultApplications(DefaultApplicationsSettingPath::from_str(sub_path_str)?))
            }
            "application" => {
                // Expecting application.<app_id>.<key>
                // We already consumed "application", so parts iterator is now at <app_id>
                let app_id = parts.next().ok_or_else(|| SettingPathParseError("Unvollständiger Application-Pfad: app_id fehlt".to_string()))?;
                if app_id.is_empty() {
                    return Err(SettingPathParseError("Application-Pfad: app_id darf nicht leer sein".to_string()));
                }

                // The rest of the string after "application.<app_id>." is the key
                // This handles keys that might themselves contain dots.
                let key_prefix = format!("application.{}.", app_id);
                let key = s.strip_prefix(&key_prefix).ok_or_else(|| SettingPathParseError("Unvollständiger Application-Pfad: key fehlt".to_string()))?;


                if key.is_empty() {
                    return Err(SettingPathParseError("Application-Pfad: key darf nicht leer sein".to_string()));
                }

                Ok(SettingPath::Application(ApplicationSettingPath {
                    app_id: app_id.to_string(),
                    key: key.to_string(),
                }))
            }
            "window-management" => {
                let sub_path_str = s.strip_prefix("window-management.").ok_or_else(|| SettingPathParseError("Unvollständiger WindowManagement-Pfad".to_string()))?;
                Ok(SettingPath::WindowManagement(WindowManagementSettingPath::from_str(sub_path_str)?))
            }
            _ => Err(SettingPathParseError(format!("Unbekannter Top-Level-Pfad: {}", top_level))),
        }
    }
}
