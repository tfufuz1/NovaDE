#[cfg(test)]
mod tests {
    use crate::global_settings_management::paths::*;
    use std::str::FromStr;

    // --- Display Tests ---

    #[test]
    fn test_font_setting_path_display() {
        assert_eq!(FontSettingPath::DefaultFontFamily.to_string(), "default-font-family");
        assert_eq!(FontSettingPath::DefaultFontSize.to_string(), "default-font-size");
    }

    #[test]
    fn test_appearance_setting_path_display() {
        assert_eq!(AppearanceSettingPath::ActiveThemeName.to_string(), "active-theme-name");
        assert_eq!(
            AppearanceSettingPath::FontSettings(FontSettingPath::Hinting).to_string(),
            "font-settings.hinting"
        );
        assert_eq!(AppearanceSettingPath::InterfaceScalingFactor.to_string(), "interface-scaling-factor");
    }

    #[test]
    fn test_workspace_setting_path_display() {
        assert_eq!(WorkspaceSettingPath::DynamicWorkspaces.to_string(), "dynamic-workspaces");
    }

    #[test]
    fn test_input_behavior_setting_path_display() {
        assert_eq!(InputBehaviorSettingPath::MouseSensitivity.to_string(), "mouse-sensitivity");
    }

    #[test]
    fn test_power_management_policy_setting_path_display() {
        assert_eq!(PowerManagementPolicySettingPath::ScreenBlankTimeoutAcSecs.to_string(), "screen-blank-timeout-ac-secs");
    }

    #[test]
    fn test_default_applications_setting_path_display() {
        assert_eq!(DefaultApplicationsSettingPath::WebBrowserDesktopFile.to_string(), "web-browser-desktop-file");
    }

    #[test]
    fn test_setting_path_display() {
        assert_eq!(
            SettingPath::Appearance(AppearanceSettingPath::ActiveThemeName).to_string(),
            "appearance.active-theme-name"
        );
        assert_eq!(
            SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize)).to_string(),
            "appearance.font-settings.default-font-size"
        );
        assert_eq!(
            SettingPath::Workspace(WorkspaceSettingPath::DefaultWorkspaceCount).to_string(),
            "workspace.default-workspace-count"
        );
        assert_eq!(
            SettingPath::InputBehavior(InputBehaviorSettingPath::KeyboardRepeatDelayMs).to_string(),
            "input-behavior.keyboard-repeat-delay-ms"
        );
        assert_eq!(
            SettingPath::PowerManagementPolicy(PowerManagementPolicySettingPath::ShowBatteryPercentage).to_string(),
            "power-management-policy.show-battery-percentage"
        );
        assert_eq!(
            SettingPath::DefaultApplications(DefaultApplicationsSettingPath::TerminalEmulatorDesktopFile).to_string(),
            "default-applications.terminal-emulator-desktop-file"
        );
    }

    // --- FromStr Tests ---

    #[test]
    fn test_font_setting_path_from_str() {
        assert_eq!(FontSettingPath::from_str("default-font-family").unwrap(), FontSettingPath::DefaultFontFamily);
        assert!(FontSettingPath::from_str("invalid-path").is_err());
    }

    #[test]
    fn test_appearance_setting_path_from_str() {
        assert_eq!(AppearanceSettingPath::from_str("active-theme-name").unwrap(), AppearanceSettingPath::ActiveThemeName);
        assert_eq!(
            AppearanceSettingPath::from_str("font-settings.hinting").unwrap(),
            AppearanceSettingPath::FontSettings(FontSettingPath::Hinting)
        );
        assert_eq!(
            AppearanceSettingPath::from_str("font-settings.default-font-size").unwrap(),
            AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize)
        );
        assert!(AppearanceSettingPath::from_str("font-settings.invalid-sub").is_err());
        assert!(AppearanceSettingPath::from_str("invalid-path").is_err());
        assert!(AppearanceSettingPath::from_str("font-settings").is_err()); // Missing sub-path
    }
    
    #[test]
    fn test_setting_path_from_str_valid_paths() {
        // Top-level direct paths
        assert_eq!(
            SettingPath::from_str("appearance.active-theme-name").unwrap(),
            SettingPath::Appearance(AppearanceSettingPath::ActiveThemeName)
        );
        assert_eq!(
            SettingPath::from_str("workspace.dynamic-workspaces").unwrap(),
            SettingPath::Workspace(WorkspaceSettingPath::DynamicWorkspaces)
        );
        assert_eq!(
            SettingPath::from_str("input-behavior.mouse-sensitivity").unwrap(),
            SettingPath::InputBehavior(InputBehaviorSettingPath::MouseSensitivity)
        );
        assert_eq!(
            SettingPath::from_str("power-management-policy.show-battery-percentage").unwrap(),
            SettingPath::PowerManagementPolicy(PowerManagementPolicySettingPath::ShowBatteryPercentage)
        );
        assert_eq!(
            SettingPath::from_str("default-applications.web-browser-desktop-file").unwrap(),
            SettingPath::DefaultApplications(DefaultApplicationsSettingPath::WebBrowserDesktopFile)
        );

        // Nested paths (e.g., font settings)
        assert_eq!(
            SettingPath::from_str("appearance.font-settings.default-font-family").unwrap(),
            SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontFamily))
        );
        assert_eq!(
            SettingPath::from_str("appearance.font-settings.hinting").unwrap(),
            SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::Hinting))
        );
    }

    #[test]
    fn test_setting_path_from_str_invalid_paths() {
        // Invalid top-level
        assert!(SettingPath::from_str("nonexistent.path").is_err());
        // Valid top-level but invalid sub-path
        assert!(SettingPath::from_str("appearance.nonexistent-sub-path").is_err());
        // Valid top-level and mid-level, but invalid deepest sub-path
        assert!(SettingPath::from_str("appearance.font-settings.nonexistent-font-setting").is_err());
        // Incomplete path (missing sub-path)
        assert!(SettingPath::from_str("appearance").is_err());
        assert!(SettingPath::from_str("appearance.font-settings").is_err());
        // Empty string
        assert!(SettingPath::from_str("").is_err());
        // Only a dot
        assert!(SettingPath::from_str(".").is_err());
    }

    #[test]
    fn test_setting_path_parse_error_display() {
        let error = SettingPathParseError("Test error message".to_string());
        assert_eq!(error.to_string(), "Ungültiger Einstellungs-Pfad: Test error message");
    }

    #[test]
    fn test_path_round_trip() {
        let paths_to_test = vec![
            SettingPath::Appearance(AppearanceSettingPath::ActiveThemeName),
            SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize)),
            SettingPath::Workspace(WorkspaceSettingPath::DefaultWorkspaceCount),
            SettingPath::InputBehavior(InputBehaviorSettingPath::KeyboardRepeatDelayMs),
            SettingPath::PowerManagementPolicy(PowerManagementPolicySettingPath::ShowBatteryPercentage),
            SettingPath::DefaultApplications(DefaultApplicationsSettingPath::TerminalEmulatorDesktopFile),
        ];

        for original_path in paths_to_test {
            let path_str = original_path.to_string();
            let parsed_path = SettingPath::from_str(&path_str).unwrap();
            assert_eq!(original_path, parsed_path, "Round trip failed for: {}", path_str);
        }
    }
}
