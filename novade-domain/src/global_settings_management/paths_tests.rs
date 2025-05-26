#[cfg(test)]
mod tests {
    use crate::global_settings_management::paths::*;
    use serde_json; // Added for serde tests
    use std::str::FromStr;

    // --- ApplicationSettingPath Serde Tests ---
    #[test]
    fn test_application_setting_path_serde() {
        let app_path = ApplicationSettingPath {
            app_id: "com.example.app".to_string(),
            key: "feature.toggle.enabled".to_string(),
        };

        let json_string = serde_json::to_string(&app_path).expect("Serialization of ApplicationSettingPath failed");
        // Expected: {"app-id":"com.example.app","key":"feature.toggle.enabled"} due to struct's kebab-case
        // Let's verify by deserializing
        
        let deserialized_app_path: ApplicationSettingPath = serde_json::from_str(&json_string).expect("Deserialization of ApplicationSettingPath failed");
        assert_eq!(app_path, deserialized_app_path);
    }
    
    // --- SettingPath::Application Serde Tests ---
    #[test]
    fn test_setting_path_application_variant_serde() {
        let app_setting_path = ApplicationSettingPath {
            app_id: "my.app.id".to_string(),
            key: "user.preference.color".to_string(),
        };
        let path = SettingPath::Application(app_setting_path.clone());

        let json_string = serde_json::to_string(&path).expect("Serialization of SettingPath::Application failed");
        // Expected: {"application":{"app-id":"my.app.id","key":"user.preference.color"}}
        // This depends on how serde handles enum variants with one field and struct fields rename_all attributes.
        // The `SettingPath` enum itself is `#[serde(rename_all = "kebab-case")]`
        // The `ApplicationSettingPath` struct is also `#[serde(rename_all = "kebab-case")]`
        // So we expect: {"application": {"app-id": "...", "key": "..."}}
        // Let's verify by deserializing.

        let deserialized_path: SettingPath = serde_json::from_str(&json_string).expect("Deserialization of SettingPath::Application failed");
        assert_eq!(path, deserialized_path);
        match deserialized_path {
            SettingPath::Application(deserialized_app_path) => {
                assert_eq!(deserialized_app_path, app_setting_path);
            }
            _ => panic!("Deserialized path is not SettingPath::Application"),
        }
    }


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
        // Test for ApplicationSettingPath
        assert_eq!(
            SettingPath::Application(ApplicationSettingPath {
                app_id: "com.example.App".to_string(),
                key: "ui.darkmode".to_string()
            }).to_string(),
            "application.com.example.App.ui.darkmode"
        );
        assert_eq!(
            SettingPath::Application(ApplicationSettingPath {
                app_id: "another.app".to_string(),
                key: "general.first-launch".to_string()
            }).to_string(),
            "application.another.app.general.first-launch"
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
    fn test_setting_path_from_str_application_paths() {
        // Valid application paths
        assert_eq!(
            SettingPath::from_str("application.com.app.setting1").unwrap(),
            SettingPath::Application(ApplicationSettingPath {
                app_id: "com.app".to_string(),
                key: "setting1".to_string(),
            })
        );
        assert_eq!(
            SettingPath::from_str("application.another.app.complex.key.name").unwrap(),
            SettingPath::Application(ApplicationSettingPath {
                app_id: "another.app".to_string(),
                key: "complex.key.name".to_string(),
            })
        );
        assert_eq!( // ensure app_id can have multiple dots
            SettingPath::from_str("application.com.example.sub.app.config.value").unwrap(),
            SettingPath::Application(ApplicationSettingPath {
                app_id: "com.example.sub.app".to_string(),
                key: "config.value".to_string(),
            })
        );

        // Invalid application paths
        let err1 = SettingPath::from_str("application.com.app.id").expect_err("Should fail: missing key");
        assert_eq!(err1.to_string(), "Ungültiger Einstellungs-Pfad: Unvollständiger Application-Pfad: key fehlt");

        let err2 = SettingPath::from_str("application..key").expect_err("Should fail: empty app_id");
        assert_eq!(err2.to_string(), "Ungültiger Einstellungs-Pfad: Application-Pfad: app_id darf nicht leer sein");

        let err3 = SettingPath::from_str("application.app.id.").expect_err("Should fail: empty key");
        assert_eq!(err3.to_string(), "Ungültiger Einstellungs-Pfad: Application-Pfad: key darf nicht leer sein");
        
        let err4 = SettingPath::from_str("application.app_id_only").expect_err("Should fail: key missing");
         assert_eq!(err4.to_string(), "Ungültiger Einstellungs-Pfad: Unvollständiger Application-Pfad: key fehlt");

        // Wrong prefix (should be caught by the main match in FromStr for SettingPath)
        let err5 = SettingPath::from_str("app.com.app.setting1").expect_err("Should fail: wrong prefix");
        assert_eq!(err5.to_string(), "Ungültiger Einstellungs-Pfad: Unbekannter Top-Level-Pfad: app");

        let err6 = SettingPath::from_str("application").expect_err("Should fail: incomplete, missing app_id and key");
        assert_eq!(err6.to_string(), "Ungültiger Einstellungs-Pfad: Unvollständiger Application-Pfad: app_id fehlt");
         
        let err7 = SettingPath::from_str("application.").expect_err("Should fail: incomplete, empty app_id and missing key");
        assert_eq!(err7.to_string(), "Ungültiger Einstellungs-Pfad: Application-Pfad: app_id darf nicht leer sein");
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
            SettingPath::Application(ApplicationSettingPath { // Added Test Case
                app_id: "com.test.app".to_string(),
                key: "feature.flag.xyz".to_string(),
            }),
            SettingPath::Application(ApplicationSettingPath { // Added Test Case with dots in key
                app_id: "another.vendor.app".to_string(),
                key: "user.profile.settings.editor.font".to_string(),
            }),
        ];

        for original_path in paths_to_test {
            let path_str = original_path.to_string();
            let parsed_path = SettingPath::from_str(&path_str)
                .unwrap_or_else(|e| panic!("Parsing failed for '{}': {}", path_str, e));
            assert_eq!(original_path, parsed_path, "Round trip failed for: {}", path_str);
        }
    }
}
