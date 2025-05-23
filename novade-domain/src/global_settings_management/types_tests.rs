#[cfg(test)]
mod tests {
    use crate::global_settings_management::types::*;
    use serde_json;
    use toml;

    // Helper to test serde for both JSON and TOML, and Default
    fn test_serde_default_for_type<T: Serialize + for<'de> Deserialize<'de> + PartialEq + Debug + Default + Clone>() {
        // Test Default
        let default_val = T::default();
        let default_clone = default_val.clone(); // Also tests Clone
        assert_eq!(default_val, default_clone);


        // Test Serde JSON
        let json_val = serde_json::to_string_pretty(&default_val).expect("JSON serialization failed");
        // println!("JSON for {}:\n{}", std::any::type_name::<T>(), json_val); // For debugging
        let deserialized_json_val: T = serde_json::from_str(&json_val).expect("JSON deserialization failed");
        assert_eq!(default_val, deserialized_json_val, "JSON Serde mismatch for {}", std::any::type_name::<T>());

        // Test Serde TOML
        let toml_val = toml::to_string_pretty(&default_val).expect("TOML serialization failed");
        // println!("TOML for {}:\n{}", std::any::type_name::<T>(), toml_val); // For debugging
        let deserialized_toml_val: T = toml::from_str(&toml_val).expect("TOML deserialization failed");
        assert_eq!(default_val, deserialized_toml_val, "TOML Serde mismatch for {}", std::any::type_name::<T>());
    }

    #[test]
    fn test_all_enums_serde_default() {
        test_serde_default_for_type::<ColorScheme>();
        test_serde_default_for_type::<FontHinting>();
        test_serde_default_for_type::<FontAntialiasing>();
        test_serde_default_for_type::<MouseAccelerationProfile>();
        test_serde_default_for_type::<LidCloseAction>();
        test_serde_default_for_type::<WorkspaceSwitchingBehavior>();
    }

    #[test]
    fn test_all_settings_structs_serde_default() {
        test_serde_default_for_type::<FontSettings>();
        test_serde_default_for_type::<AppearanceSettings>();
        test_serde_default_for_type::<WorkspaceSettings>();
        test_serde_default_for_type::<InputBehaviorSettings>();
        test_serde_default_for_type::<PowerManagementPolicySettings>();
        test_serde_default_for_type::<DefaultApplicationsSettings>();
        test_serde_default_for_type::<GlobalDesktopSettings>();
    }

    // --- Validation Tests ---

    #[test]
    fn test_font_settings_validation() {
        let mut settings = FontSettings::default();
        // Default should be invalid if strings are empty by default from String::default()
        settings.default_font_family = "Test Sans".to_string();
        settings.monospace_font_family = "Test Mono".to_string();
        settings.document_font_family = "Test Serif".to_string();
        settings.default_font_size = 10.0;
        assert!(settings.validate().is_ok());

        settings.default_font_family = "".to_string();
        assert!(settings.validate().is_err());
        settings.default_font_family = "Test Sans".to_string();

        settings.default_font_size = 0.0;
        assert!(settings.validate().is_err());
        settings.default_font_size = -1.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_appearance_settings_validation() {
        let mut settings = AppearanceSettings::default();
        // Populate with valid defaults for testing
        settings.active_theme_name = "TestTheme".to_string();
        settings.accent_color_token = "test-accent-token".to_string();
        settings.font_settings.default_font_family = "Test Font".to_string();
        settings.font_settings.monospace_font_family = "Test Mono".to_string();
        settings.font_settings.document_font_family = "Test Serif".to_string();
        settings.font_settings.default_font_size = 10.0;
        settings.icon_theme_name = "TestIcons".to_string();
        settings.cursor_theme_name = "TestCursors".to_string();
        settings.interface_scaling_factor = 1.0;
        settings.enable_animations = true;
        assert!(settings.validate().is_ok());

        settings.active_theme_name = "".to_string();
        assert!(settings.validate().is_err());
        settings.active_theme_name = "TestTheme".to_string();

        settings.interface_scaling_factor = 0.0;
        assert!(settings.validate().is_err());
        settings.interface_scaling_factor = 0.4; // too low
        assert!(settings.validate().is_err());
         settings.interface_scaling_factor = 3.1; // too high
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_workspace_settings_validation() {
        let mut settings = WorkspaceSettings::default();
        settings.default_workspace_count = 4; // Valid
        assert!(settings.validate().is_ok());

        settings.default_workspace_count = 0; // Invalid
        assert!(settings.validate().is_err());
        settings.default_workspace_count = 33; // Invalid (too high)
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_input_behavior_settings_validation() {
        let mut settings = InputBehaviorSettings::default();
        settings.mouse_sensitivity = 0.5;
        settings.touchpad_pointer_speed = 0.5;
        settings.keyboard_repeat_delay_ms = 300;
        settings.keyboard_repeat_rate_cps = 30;
        assert!(settings.validate().is_ok());

        settings.mouse_acceleration_profile = MouseAccelerationProfile::Custom;
        settings.custom_mouse_acceleration_factor = None; // Invalid
        assert!(settings.validate().is_err());
        settings.custom_mouse_acceleration_factor = Some(0.5); // Valid
        assert!(settings.validate().is_ok());

        settings.mouse_acceleration_profile = MouseAccelerationProfile::Flat;
        settings.custom_mouse_acceleration_factor = Some(0.5); // Invalid
        assert!(settings.validate().is_err());
        settings.custom_mouse_acceleration_factor = None; // Valid
        assert!(settings.validate().is_ok());

        settings.mouse_sensitivity = -0.1; // Invalid
        assert!(settings.validate().is_err());
        settings.mouse_sensitivity = 2.1; // Invalid
        assert!(settings.validate().is_err());
        settings.mouse_sensitivity = 0.5; // Valid again

        settings.keyboard_repeat_delay_ms = 50; // Invalid
        assert!(settings.validate().is_err());
        settings.keyboard_repeat_delay_ms = 3000; // Invalid
        assert!(settings.validate().is_err());
    }
    
    #[test]
    fn test_power_management_policy_settings_validation() {
        let settings = PowerManagementPolicySettings::default();
        // Currently, no specific validation rules beyond type limits (u32 >= 0)
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_default_applications_settings_validation() {
        let mut settings = DefaultApplicationsSettings::default();
        // Defaults are empty strings, which should fail validation
        assert!(settings.validate().is_err());
        
        settings.web_browser_desktop_file = "firefox.desktop".to_string();
        settings.email_client_desktop_file = "thunderbird.desktop".to_string();
        settings.terminal_emulator_desktop_file = "gnome-terminal.desktop".to_string();
        settings.file_manager_desktop_file = "nautilus.desktop".to_string();
        settings.music_player_desktop_file = "rhythmbox.desktop".to_string();
        settings.video_player_desktop_file = "totem.desktop".to_string();
        settings.image_viewer_desktop_file = "eog.desktop".to_string();
        settings.text_editor_desktop_file = "gedit.desktop".to_string();
        assert!(settings.validate().is_ok());

        settings.web_browser_desktop_file = "firefox".to_string(); // Missing .desktop
        assert!(settings.validate().is_err());
        settings.web_browser_desktop_file = "".to_string(); // Empty
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_global_desktop_settings_validation_recursive() {
        let mut settings = GlobalDesktopSettings::default();
        // Populate with valid settings for all sub-structs
        settings.appearance.active_theme_name = "TestTheme".to_string();
        settings.appearance.accent_color_token = "test-accent".to_string();
        settings.appearance.font_settings.default_font_family = "Sans".to_string();
        settings.appearance.font_settings.monospace_font_family = "Mono".to_string();
        settings.appearance.font_settings.document_font_family = "Serif".to_string();
        settings.appearance.font_settings.default_font_size = 10.0;
        settings.appearance.icon_theme_name = "Icons".to_string();
        settings.appearance.cursor_theme_name = "Cursors".to_string();
        settings.appearance.interface_scaling_factor = 1.0;

        settings.workspace.default_workspace_count = 2;

        settings.input_behavior.mouse_sensitivity = 0.5;
        settings.input_behavior.touchpad_pointer_speed = 0.5;
        settings.input_behavior.keyboard_repeat_delay_ms = 500;
        settings.input_behavior.keyboard_repeat_rate_cps = 25;

        settings.default_applications.web_browser_desktop_file = "b.desktop".to_string();
        settings.default_applications.email_client_desktop_file = "e.desktop".to_string();
        settings.default_applications.terminal_emulator_desktop_file = "t.desktop".to_string();
        settings.default_applications.file_manager_desktop_file = "f.desktop".to_string();
        settings.default_applications.music_player_desktop_file = "m.desktop".to_string();
        settings.default_applications.video_player_desktop_file = "v.desktop".to_string();
        settings.default_applications.image_viewer_desktop_file = "i.desktop".to_string();
        settings.default_applications.text_editor_desktop_file = "te.desktop".to_string();

        assert!(settings.validate_recursive().is_ok());

        // Introduce an error in a nested struct
        settings.appearance.font_settings.default_font_size = 0.0;
        assert!(settings.validate_recursive().is_err());
        assert_eq!(
            settings.validate_recursive().err().unwrap(),
            "Appearance settings: Font settings: Default font size muss größer als 0 sein."
        );
    }
}
