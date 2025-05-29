use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, warn};

use super::types::GlobalDesktopSettings;
use super::paths::{SettingPath, AppearanceSettingPath, FontSettingPath, WorkspaceSettingPath, InputBehaviorSettingPath, PowerManagementPolicySettingPath, DefaultApplicationsSettingPath};
use super::errors::GlobalSettingsError;
use super::events::{SettingChangedEvent, SettingsLoadedEvent, SettingsSavedEvent};
use super::persistence_iface::SettingsPersistenceProvider;

// --- GlobalSettingsService Trait ---

#[async_trait]
pub trait GlobalSettingsService: Send + Sync {
    async fn load_settings(&self) -> Result<(), GlobalSettingsError>;
    async fn save_settings(&self) -> Result<(), GlobalSettingsError>;
    fn get_current_settings(&self) -> GlobalDesktopSettings;
    async fn update_setting(&self, path: SettingPath, value: JsonValue) -> Result<(), GlobalSettingsError>;
    fn get_setting(&self, path: &SettingPath) -> Result<JsonValue, GlobalSettingsError>;
    async fn reset_to_defaults(&self) -> Result<(), GlobalSettingsError>;
    fn subscribe_to_setting_changes(&self) -> broadcast::Receiver<SettingChangedEvent>;
    fn subscribe_to_settings_loaded(&self) -> broadcast::Receiver<SettingsLoadedEvent>;
    fn subscribe_to_settings_saved(&self) -> broadcast::Receiver<SettingsSavedEvent>;
    // async fn get_typed_setting<T: serde::de::DeserializeOwned + Send>(&self, path: &SettingPath) -> Result<T, GlobalSettingsError>;
}

// --- DefaultGlobalSettingsService Implementation ---

pub struct DefaultGlobalSettingsService {
    settings: Arc<RwLock<GlobalDesktopSettings>>,
    persistence_provider: Arc<dyn SettingsPersistenceProvider>,
    event_sender: broadcast::Sender<SettingChangedEvent>,
    loaded_event_sender: broadcast::Sender<SettingsLoadedEvent>,
    saved_event_sender: broadcast::Sender<SettingsSavedEvent>,
}

impl DefaultGlobalSettingsService {
    pub fn new(persistence_provider: Arc<dyn SettingsPersistenceProvider>, broadcast_capacity: usize) -> Self {
        let (event_sender, _) = broadcast::channel(broadcast_capacity);
        let (loaded_event_sender, _) = broadcast::channel(broadcast_capacity);
        let (saved_event_sender, _) = broadcast::channel(broadcast_capacity);
        Self {
            settings: Arc::new(RwLock::new(GlobalDesktopSettings::default())),
            persistence_provider,
            event_sender,
            loaded_event_sender,
            saved_event_sender,
        }
    }
}

#[async_trait]
impl GlobalSettingsService for DefaultGlobalSettingsService {
    async fn load_settings(&self) -> Result<(), GlobalSettingsError> {
        debug!("Service: Attempting to load settings from persistence provider.");
        let loaded_settings = self.persistence_provider.load_global_settings().await?;
        
        let mut settings_guard = self.settings.write().await;
        *settings_guard = loaded_settings.clone();
        debug!("Service: Settings loaded and updated internally.");

        if let Err(e) = self.loaded_event_sender.send(SettingsLoadedEvent::new(loaded_settings)) {
            error!("Failed to send SettingsLoadedEvent: {}", e);
        }
        Ok(())
    }

    async fn save_settings(&self) -> Result<(), GlobalSettingsError> {
        debug!("Service: Attempting to save settings using persistence provider.");
        let settings_clone = self.settings.read().await.clone();
        self.persistence_provider.save_global_settings(&settings_clone).await?;
        debug!("Service: Settings saved successfully by persistence provider.");

        if let Err(e) = self.saved_event_sender.send(SettingsSavedEvent::new()) {
            error!("Failed to send SettingsSavedEvent: {}", e);
        }
        Ok(())
    }

    fn get_current_settings(&self) -> GlobalDesktopSettings {
        self.settings.blocking_read().clone()
    }

    async fn update_setting(&self, path: SettingPath, value: JsonValue) -> Result<(), GlobalSettingsError> {
        debug!("Service: Attempting to update setting at path: {:?}, with value: {:?}", path, value);
        let mut settings_guard = self.settings.write().await;
        let mut new_settings = (*settings_guard).clone();

        macro_rules! update_field {
            ($target_struct:expr, $field_name:ident, $json_value:expr, $error_path:expr, $expected_type_name:expr) => {
                match serde_json::from_value($json_value.clone()) {
                    Ok(val) => $target_struct.$field_name = val,
                    Err(_e) => return Err(GlobalSettingsError::InvalidValueType {
                        path: $error_path,
                        expected_type: $expected_type_name.to_string(),
                        actual_value_preview: format!("{:.50}", $json_value.to_string()),
                    }),
                }
            };
        }
        
        match path.clone() {
            SettingPath::Appearance(ref ap_path) => match ap_path {
                AppearanceSettingPath::ActiveThemeName => update_field!(new_settings.appearance, active_theme_name, value, path, "String"),
                AppearanceSettingPath::ColorScheme => update_field!(new_settings.appearance, color_scheme, value, path, "ColorScheme"),
                AppearanceSettingPath::AccentColorToken => update_field!(new_settings.appearance, accent_color_token, value, path, "String"),
                AppearanceSettingPath::IconThemeName => update_field!(new_settings.appearance, icon_theme_name, value, path, "String"),
                AppearanceSettingPath::CursorThemeName => update_field!(new_settings.appearance, cursor_theme_name, value, path, "String"),
                AppearanceSettingPath::EnableAnimations => update_field!(new_settings.appearance, enable_animations, value, path, "bool"),
                AppearanceSettingPath::InterfaceScalingFactor => update_field!(new_settings.appearance, interface_scaling_factor, value, path, "f64"),
                AppearanceSettingPath::FontSettings(ref fs_path) => match fs_path {
                    FontSettingPath::DefaultFontFamily => update_field!(new_settings.appearance.font_settings, default_font_family, value, path, "String"),
                    FontSettingPath::DefaultFontSize => update_field!(new_settings.appearance.font_settings, default_font_size, value, path, "u8"),
                    FontSettingPath::MonospaceFontFamily => update_field!(new_settings.appearance.font_settings, monospace_font_family, value, path, "String"),
                    FontSettingPath::DocumentFontFamily => update_field!(new_settings.appearance.font_settings, document_font_family, value, path, "String"),
                    FontSettingPath::Hinting => update_field!(new_settings.appearance.font_settings, hinting, value, path, "FontHinting"),
                    FontSettingPath::Antialiasing => update_field!(new_settings.appearance.font_settings, antialiasing, value, path, "FontAntialiasing"),
                }
            },
            SettingPath::Workspaces(ref ws_path) => match ws_path {
                WorkspaceSettingPath::DynamicWorkspaces => update_field!(new_settings.workspaces, dynamic_workspaces, value, path, "bool"),
                WorkspaceSettingPath::DefaultWorkspaceCount => update_field!(new_settings.workspaces, default_workspace_count, value, path, "u8"),
                WorkspaceSettingPath::WorkspaceSwitchingBehavior => update_field!(new_settings.workspaces, workspace_switching_behavior, value, path, "WorkspaceSwitchingBehavior"),
                WorkspaceSettingPath::ShowWorkspaceIndicator => update_field!(new_settings.workspaces, show_workspace_indicator, value, path, "bool"),
            },
            SettingPath::InputBehavior(ref ib_path) => match ib_path {
                InputBehaviorSettingPath::MouseAccelerationProfile => update_field!(new_settings.input_behavior, mouse_acceleration_profile, value, path, "MouseAccelerationProfile"),
                InputBehaviorSettingPath::CustomMouseAccelerationFactor => update_field!(new_settings.input_behavior, custom_mouse_acceleration_factor, value, path, "Option<f32>"),
                InputBehaviorSettingPath::MouseSensitivity => update_field!(new_settings.input_behavior, mouse_sensitivity, value, path, "f32"),
                InputBehaviorSettingPath::NaturalScrollingMouse => update_field!(new_settings.input_behavior, natural_scrolling_mouse, value, path, "bool"),
                InputBehaviorSettingPath::NaturalScrollingTouchpad => update_field!(new_settings.input_behavior, natural_scrolling_touchpad, value, path, "bool"),
                InputBehaviorSettingPath::TapToClickTouchpad => update_field!(new_settings.input_behavior, tap_to_click_touchpad, value, path, "bool"),
                InputBehaviorSettingPath::TouchpadPointerSpeed => update_field!(new_settings.input_behavior, touchpad_pointer_speed, value, path, "f32"),
                InputBehaviorSettingPath::KeyboardRepeatDelayMs => update_field!(new_settings.input_behavior, keyboard_repeat_delay_ms, value, path, "u32"),
                InputBehaviorSettingPath::KeyboardRepeatRateCps => update_field!(new_settings.input_behavior, keyboard_repeat_rate_cps, value, path, "u32"),
            },
            SettingPath::PowerManagementPolicy(ref pmp_path) => match pmp_path {
                PowerManagementPolicySettingPath::ScreenBlankTimeoutAcSecs => update_field!(new_settings.power_management_policy, screen_blank_timeout_ac_secs, value, path, "u32"),
                PowerManagementPolicySettingPath::ScreenBlankTimeoutBatterySecs => update_field!(new_settings.power_management_policy, screen_blank_timeout_battery_secs, value, path, "u32"),
                PowerManagementPolicySettingPath::SuspendActionOnLidCloseAc => update_field!(new_settings.power_management_policy, suspend_action_on_lid_close_ac, value, path, "LidCloseAction"),
                PowerManagementPolicySettingPath::SuspendActionOnLidCloseBattery => update_field!(new_settings.power_management_policy, suspend_action_on_lid_close_battery, value, path, "LidCloseAction"),
                PowerManagementPolicySettingPath::AutomaticSuspendDelayAcSecs => update_field!(new_settings.power_management_policy, automatic_suspend_delay_ac_secs, value, path, "u32"),
                PowerManagementPolicySettingPath::AutomaticSuspendDelayBatterySecs => update_field!(new_settings.power_management_policy, automatic_suspend_delay_battery_secs, value, path, "u32"),
                PowerManagementPolicySettingPath::ShowBatteryPercentage => update_field!(new_settings.power_management_policy, show_battery_percentage, value, path, "bool"),
            },
            SettingPath::DefaultApplications(ref da_path) => match da_path {
                DefaultApplicationsSettingPath::WebBrowser => update_field!(new_settings.default_applications, web_browser, value, path, "String"),
                DefaultApplicationsSettingPath::EmailClient => update_field!(new_settings.default_applications, email_client, value, path, "String"),
                DefaultApplicationsSettingPath::TerminalEmulator => update_field!(new_settings.default_applications, terminal_emulator, value, path, "String"),
                DefaultApplicationsSettingPath::FileManager => update_field!(new_settings.default_applications, file_manager, value, path, "String"),
                DefaultApplicationsSettingPath::MusicPlayer => update_field!(new_settings.default_applications, music_player, value, path, "String"),
                DefaultApplicationsSettingPath::VideoPlayer => update_field!(new_settings.default_applications, video_player, value, path, "String"),
                DefaultApplicationsSettingPath::ImageViewer => update_field!(new_settings.default_applications, image_viewer, value, path, "String"),
                DefaultApplicationsSettingPath::TextEditor => update_field!(new_settings.default_applications, text_editor, value, path, "String"),
            },
            SettingPath::Root | SettingPath::AppearanceRoot | SettingPath::WorkspacesRoot | 
            SettingPath::InputBehaviorRoot | SettingPath::PowerManagementPolicyRoot | SettingPath::DefaultApplicationsRoot => {
                return Err(GlobalSettingsError::InvalidValueType {
                    path: path.clone(),
                    expected_type: "Specific setting path".to_string(),
                    actual_value_preview: "Attempted to update a root/category path.".to_string(),
                });
            }
        }

        new_settings.validate_recursive()?;
        
        *settings_guard = new_settings;
        debug!("Service: Setting updated and validated successfully for path: {:?}", path);

        if let Err(e) = self.event_sender.send(SettingChangedEvent { path, new_value: value }) {
            error!("Failed to send SettingChangedEvent: {}", e);
        }
        
        self.save_settings().await?;
        
        Ok(())
    }

    fn get_setting(&self, path: &SettingPath) -> Result<JsonValue, GlobalSettingsError> {
        let settings_guard = self.settings.blocking_read();
        
        macro_rules! get_json_value {
            ($field_val:expr) => {
                serde_json::to_value($field_val).map_err(|e| GlobalSettingsError::SerializationError {
                    path: path.clone(),
                    source: e,
                })
            };
        }

        match path {
            SettingPath::Appearance(ap_path) => match ap_path {
                AppearanceSettingPath::ActiveThemeName => get_json_value!(&settings_guard.appearance.active_theme_name),
                AppearanceSettingPath::ColorScheme => get_json_value!(&settings_guard.appearance.color_scheme),
                AppearanceSettingPath::AccentColorToken => get_json_value!(&settings_guard.appearance.accent_color_token),
                AppearanceSettingPath::IconThemeName => get_json_value!(&settings_guard.appearance.icon_theme_name),
                AppearanceSettingPath::CursorThemeName => get_json_value!(&settings_guard.appearance.cursor_theme_name),
                AppearanceSettingPath::EnableAnimations => get_json_value!(&settings_guard.appearance.enable_animations),
                AppearanceSettingPath::InterfaceScalingFactor => get_json_value!(&settings_guard.appearance.interface_scaling_factor),
                AppearanceSettingPath::FontSettings(fs_path) => match fs_path {
                    FontSettingPath::DefaultFontFamily => get_json_value!(&settings_guard.appearance.font_settings.default_font_family),
                    FontSettingPath::DefaultFontSize => get_json_value!(&settings_guard.appearance.font_settings.default_font_size),
                    FontSettingPath::MonospaceFontFamily => get_json_value!(&settings_guard.appearance.font_settings.monospace_font_family),
                    FontSettingPath::DocumentFontFamily => get_json_value!(&settings_guard.appearance.font_settings.document_font_family),
                    FontSettingPath::Hinting => get_json_value!(&settings_guard.appearance.font_settings.hinting),
                    FontSettingPath::Antialiasing => get_json_value!(&settings_guard.appearance.font_settings.antialiasing),
                }
            },
            SettingPath::Workspaces(ws_path) => match ws_path {
                WorkspaceSettingPath::DynamicWorkspaces => get_json_value!(&settings_guard.workspaces.dynamic_workspaces),
                WorkspaceSettingPath::DefaultWorkspaceCount => get_json_value!(&settings_guard.workspaces.default_workspace_count),
                WorkspaceSettingPath::WorkspaceSwitchingBehavior => get_json_value!(&settings_guard.workspaces.workspace_switching_behavior),
                WorkspaceSettingPath::ShowWorkspaceIndicator => get_json_value!(&settings_guard.workspaces.show_workspace_indicator),
            },
            SettingPath::InputBehavior(ib_path) => match ib_path {
                InputBehaviorSettingPath::MouseAccelerationProfile => get_json_value!(&settings_guard.input_behavior.mouse_acceleration_profile),
                InputBehaviorSettingPath::CustomMouseAccelerationFactor => get_json_value!(&settings_guard.input_behavior.custom_mouse_acceleration_factor),
                InputBehaviorSettingPath::MouseSensitivity => get_json_value!(&settings_guard.input_behavior.mouse_sensitivity),
                InputBehaviorSettingPath::NaturalScrollingMouse => get_json_value!(&settings_guard.input_behavior.natural_scrolling_mouse),
                InputBehaviorSettingPath::NaturalScrollingTouchpad => get_json_value!(&settings_guard.input_behavior.natural_scrolling_touchpad),
                InputBehaviorSettingPath::TapToClickTouchpad => get_json_value!(&settings_guard.input_behavior.tap_to_click_touchpad),
                InputBehaviorSettingPath::TouchpadPointerSpeed => get_json_value!(&settings_guard.input_behavior.touchpad_pointer_speed),
                InputBehaviorSettingPath::KeyboardRepeatDelayMs => get_json_value!(&settings_guard.input_behavior.keyboard_repeat_delay_ms),
                InputBehaviorSettingPath::KeyboardRepeatRateCps => get_json_value!(&settings_guard.input_behavior.keyboard_repeat_rate_cps),
            },
            SettingPath::PowerManagementPolicy(pmp_path) => match pmp_path {
                PowerManagementPolicySettingPath::ScreenBlankTimeoutAcSecs => get_json_value!(&settings_guard.power_management_policy.screen_blank_timeout_ac_secs),
                PowerManagementPolicySettingPath::ScreenBlankTimeoutBatterySecs => get_json_value!(&settings_guard.power_management_policy.screen_blank_timeout_battery_secs),
                PowerManagementPolicySettingPath::SuspendActionOnLidCloseAc => get_json_value!(&settings_guard.power_management_policy.suspend_action_on_lid_close_ac),
                PowerManagementPolicySettingPath::SuspendActionOnLidCloseBattery => get_json_value!(&settings_guard.power_management_policy.suspend_action_on_lid_close_battery),
                PowerManagementPolicySettingPath::AutomaticSuspendDelayAcSecs => get_json_value!(&settings_guard.power_management_policy.automatic_suspend_delay_ac_secs),
                PowerManagementPolicySettingPath::AutomaticSuspendDelayBatterySecs => get_json_value!(&settings_guard.power_management_policy.automatic_suspend_delay_battery_secs),
                PowerManagementPolicySettingPath::ShowBatteryPercentage => get_json_value!(&settings_guard.power_management_policy.show_battery_percentage),
            },
            SettingPath::DefaultApplications(da_path) => match da_path {
                DefaultApplicationsSettingPath::WebBrowser => get_json_value!(&settings_guard.default_applications.web_browser),
                DefaultApplicationsSettingPath::EmailClient => get_json_value!(&settings_guard.default_applications.email_client),
                DefaultApplicationsSettingPath::TerminalEmulator => get_json_value!(&settings_guard.default_applications.terminal_emulator),
                DefaultApplicationsSettingPath::FileManager => get_json_value!(&settings_guard.default_applications.file_manager),
                DefaultApplicationsSettingPath::MusicPlayer => get_json_value!(&settings_guard.default_applications.music_player),
                DefaultApplicationsSettingPath::VideoPlayer => get_json_value!(&settings_guard.default_applications.video_player),
                DefaultApplicationsSettingPath::ImageViewer => get_json_value!(&settings_guard.default_applications.image_viewer),
                DefaultApplicationsSettingPath::TextEditor => get_json_value!(&settings_guard.default_applications.text_editor),
            },
            SettingPath::AppearanceRoot => get_json_value!(&settings_guard.appearance),
            SettingPath::WorkspacesRoot => get_json_value!(&settings_guard.workspaces),
            SettingPath::InputBehaviorRoot => get_json_value!(&settings_guard.input_behavior),
            SettingPath::PowerManagementPolicyRoot => get_json_value!(&settings_guard.power_management_policy),
            SettingPath::DefaultApplicationsRoot => get_json_value!(&settings_guard.default_applications),
            SettingPath::Root => get_json_value!(&*settings_guard),
        }
    }

    async fn reset_to_defaults(&self) -> Result<(), GlobalSettingsError> {
        debug!("Service: Resetting settings to defaults.");
        let mut settings_guard = self.settings.write().await;
        let defaults = GlobalDesktopSettings::default();
        
        *settings_guard = defaults.clone();

        let paths_to_notify = [
            (SettingPath::AppearanceRoot, serde_json::to_value(&defaults.appearance).unwrap_or(JsonValue::Null)),
            (SettingPath::WorkspacesRoot, serde_json::to_value(&defaults.workspaces).unwrap_or(JsonValue::Null)),
            (SettingPath::InputBehaviorRoot, serde_json::to_value(&defaults.input_behavior).unwrap_or(JsonValue::Null)),
            (SettingPath::PowerManagementPolicyRoot, serde_json::to_value(&defaults.power_management_policy).unwrap_or(JsonValue::Null)),
            (SettingPath::DefaultApplicationsRoot, serde_json::to_value(&defaults.default_applications).unwrap_or(JsonValue::Null)),
        ];

        for (path, new_value) in paths_to_notify {
            if let Err(e) = self.event_sender.send(SettingChangedEvent { path, new_value }) {
                error!("Failed to send SettingChangedEvent during reset: {}", e);
            }
        }
        
        self.save_settings().await?;
        debug!("Service: Settings reset to defaults and saved.");
        Ok(())
    }

    fn subscribe_to_setting_changes(&self) -> broadcast::Receiver<SettingChangedEvent> {
        self.event_sender.subscribe()
    }
    
    fn subscribe_to_settings_loaded(&self) -> broadcast::Receiver<SettingsLoadedEvent> {
        self.loaded_event_sender.subscribe()
    }

    fn subscribe_to_settings_saved(&self) -> broadcast::Receiver<SettingsSavedEvent> {
        self.saved_event_sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_settings::persistence_iface::MockSettingsPersistenceProvider;
    use crate::global_settings::types::{ColorScheme, FontAntialiasing}; // Import necessary types for tests
    use tokio::sync::broadcast::error::RecvError;

    #[tokio::test]
    async fn test_new_and_initial_load_success() {
        let mut mock_persistence = MockSettingsPersistenceProvider::new();
        let default_settings = GlobalDesktopSettings::default();
        
        mock_persistence.expect_load_global_settings()
            .times(1)
            .returning(move || Ok(default_settings.clone()));

        let service = DefaultGlobalSettingsService::new(Arc::new(mock_persistence), 5);
        let initial_load_result = service.load_settings().await;
        assert!(initial_load_result.is_ok());
        
        let current_settings = service.get_current_settings();
        assert_eq!(current_settings, GlobalDesktopSettings::default());
    }
    
    #[tokio::test]
    async fn test_initial_load_persistence_error() {
        let mut mock_persistence = MockSettingsPersistenceProvider::new();
        mock_persistence.expect_load_global_settings()
            .times(1)
            .returning(|| Err(GlobalSettingsError::persistence_error_no_source("load", "Failed to read")));

        let service = DefaultGlobalSettingsService::new(Arc::new(mock_persistence), 5);
        let result = service.load_settings().await;
        assert!(result.is_err());
        if let Err(GlobalSettingsError::PersistenceError{operation, ..}) = result {
            assert_eq!(operation, "load");
        } else {
            panic!("Expected PersistenceError, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_update_setting_successful() {
        let mut mock_persistence = MockSettingsPersistenceProvider::new();
        mock_persistence.expect_load_global_settings().returning(|| Ok(GlobalDesktopSettings::default()));
        mock_persistence.expect_save_global_settings().times(1).returning(|_| Ok(())); // Expect save

        let service = DefaultGlobalSettingsService::new(Arc::new(mock_persistence), 5);
        service.load_settings().await.unwrap();

        let path = SettingPath::Appearance(AppearanceSettingPath::ColorScheme);
        let new_value = JsonValue::String("dark".to_string());
        
        let mut event_rx = service.subscribe_to_setting_changes();

        let update_result = service.update_setting(path.clone(), new_value.clone()).await;
        assert!(update_result.is_ok());

        let current_settings = service.get_current_settings();
        assert_eq!(current_settings.appearance.color_scheme, ColorScheme::Dark);
        
        match tokio::time::timeout(std::time::Duration::from_millis(10), event_rx.recv()).await {
            Ok(Ok(event)) => {
                assert_eq!(event.path, path);
                assert_eq!(event.new_value, new_value);
            }
            res => panic!("SettingChangedEvent not received or incorrect: {:?}", res),
        }
    }
    
    #[tokio::test]
    async fn test_update_setting_invalid_value_type() {
        let mut mock_persistence = MockSettingsPersistenceProvider::new();
        mock_persistence.expect_load_global_settings().returning(|| Ok(GlobalDesktopSettings::default()));
        mock_persistence.expect_save_global_settings().times(0).returning(|_| Ok(()));

        let service = DefaultGlobalSettingsService::new(Arc::new(mock_persistence), 5);
        service.load_settings().await.unwrap();

        let path = SettingPath::Appearance(AppearanceSettingPath::ColorScheme);
        let new_value = JsonValue::Number(123.into());

        let update_result = service.update_setting(path.clone(), new_value.clone()).await;
        assert!(update_result.is_err());
        assert!(matches!(update_result.unwrap_err(), GlobalSettingsError::InvalidValueType { .. }));
    }

    #[tokio::test]
    async fn test_update_setting_validation_failure() {
        let mut mock_persistence = MockSettingsPersistenceProvider::new();
        mock_persistence.expect_load_global_settings().returning(|| Ok(GlobalDesktopSettings::default()));
        mock_persistence.expect_save_global_settings().times(0).returning(|_| Ok(()));

        let service = DefaultGlobalSettingsService::new(Arc::new(mock_persistence), 5);
        service.load_settings().await.unwrap();

        let path = SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize));
        let new_value = JsonValue::Number(serde_json::Number::from(3)); // Invalid: too small

        let update_result = service.update_setting(path.clone(), new_value.clone()).await;
        assert!(update_result.is_err());
        // validate_recursive error path should be AppearanceRoot if FontSettings validation fails
        assert!(matches!(update_result.unwrap_err(), GlobalSettingsError::ValidationError { path: SettingPath::AppearanceRoot, .. }));
    }

    #[tokio::test]
    async fn test_get_setting_successful() {
        let mock_persistence = MockSettingsPersistenceProvider::new(); 
        let service = DefaultGlobalSettingsService::new(Arc::new(mock_persistence), 5);
        // No load_settings called, so it uses default settings

        let path = SettingPath::Appearance(AppearanceSettingPath::ColorScheme);
        let value = service.get_setting(&path).unwrap();
        assert_eq!(value, JsonValue::String("system-preference".to_string())); // Default value

        let path_font_size = SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize));
        let value_font_size = service.get_setting(&path_font_size).unwrap();
        assert_eq!(value_font_size, JsonValue::Number(11.into())); // Default font size
    }

    #[tokio::test]
    async fn test_reset_to_defaults_successful() {
        let mut mock_persistence = MockSettingsPersistenceProvider::new();
        // Initial load
        mock_persistence.expect_load_global_settings().times(1).returning(|| Ok(GlobalDesktopSettings::default()));
        // Save after update
        mock_persistence.expect_save_global_settings().times(1).returning(|_| Ok(())); 
        // Save after reset
        mock_persistence.expect_save_global_settings().times(1).returning(|settings| {
            assert_eq!(*settings, GlobalDesktopSettings::default()); 
            Ok(())
        });

        let service = DefaultGlobalSettingsService::new(Arc::new(mock_persistence), 5);
        service.load_settings().await.unwrap();

        // Change a setting first
        let path_to_change = SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::Antialiasing));
        let changed_value = JsonValue::String("rgba".to_string()); // FontAntialiasing::Rgba
        service.update_setting(path_to_change.clone(), changed_value.clone()).await.unwrap();
        
        assert_ne!(service.get_current_settings().appearance.font_settings.antialiasing, FontAntialiasing::default());

        let mut event_rx = service.subscribe_to_setting_changes();
        
        let reset_result = service.reset_to_defaults().await;
        assert!(reset_result.is_ok());

        let current_settings = service.get_current_settings();
        assert_eq!(current_settings, GlobalDesktopSettings::default());

        let mut events_received = 0;
        for _ in 0..5 { 
            match tokio::time::timeout(std::time::Duration::from_millis(10), event_rx.recv()).await {
                Ok(Ok(event)) => {
                    events_received += 1;
                    // Check if the event corresponds to one of the reset root paths
                    assert!(matches!(event.path, SettingPath::AppearanceRoot | SettingPath::WorkspacesRoot | SettingPath::InputBehaviorRoot | SettingPath::PowerManagementPolicyRoot | SettingPath::DefaultApplicationsRoot));
                }
                Ok(Err(RecvError::Lagged(_))) => { /* ignore lagged */ continue; }
                Ok(Err(RecvError::Closed)) => break, 
                Err(_) => break, // Timeout
            }
        }
        assert_eq!(events_received, 5, "Expected 5 events for reset categories");
    }
    
    #[tokio::test]
    async fn test_loaded_saved_events() {
        let mut mock_persistence = MockSettingsPersistenceProvider::new();
        let default_settings = GlobalDesktopSettings::default();

        mock_persistence.expect_load_global_settings().returning(move || Ok(default_settings.clone()));
        mock_persistence.expect_save_global_settings().returning(|_| Ok(()));

        let service = DefaultGlobalSettingsService::new(Arc::new(mock_persistence), 5);
        
        let mut loaded_rx = service.subscribe_to_settings_loaded();
        let mut saved_rx = service.subscribe_to_settings_saved();

        service.load_settings().await.unwrap();
        match tokio::time::timeout(std::time::Duration::from_millis(10), loaded_rx.recv()).await {
            Ok(Ok(event)) => assert_eq!(event.settings, GlobalDesktopSettings::default()),
            res => panic!("SettingsLoadedEvent not received or incorrect: {:?}", res),
        }

        service.save_settings().await.unwrap();
        match tokio::time::timeout(std::time::Duration::from_millis(10), saved_rx.recv()).await {
            Ok(Ok(_)) => { /* SettingsSavedEvent received */ },
            res => panic!("SettingsSavedEvent not received or incorrect: {:?}", res),
        }
    }
}
