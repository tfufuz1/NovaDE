use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{warn, error, info}; // For logging
use serde_json::Value as JsonValue;

use super::types::{GlobalDesktopSettings, SettingChangedEvent, SettingsLoadedEvent, SettingsSavedEvent, GlobalSettingsEvent};
use super::paths::SettingPath;
use super::errors::GlobalSettingsError;
use super::persistence_iface::SettingsPersistenceProvider;

// For path navigation helpers
use super::paths::{
    SettingPath, ApplicationSettingPath, AppearanceSettingPath, FontSettingPath, WorkspaceSettingPath,
    InputBehaviorSettingPath, PowerManagementPolicySettingPath, DefaultApplicationsSettingPath,
};


const DEFAULT_BROADCAST_CAPACITY: usize = 32; // Default capacity for the event channel

#[async_trait::async_trait]
pub trait GlobalSettingsService: Send + Sync {
    async fn load_settings(&self) -> Result<(), GlobalSettingsError>;
    async fn save_settings(&self) -> Result<(), GlobalSettingsError>;
    fn get_current_settings(&self) -> Result<GlobalDesktopSettings, GlobalSettingsError>;
    async fn update_setting(&self, path: SettingPath, value: JsonValue) -> Result<(), GlobalSettingsError>;
    fn get_setting(&self, path: &SettingPath) -> Result<JsonValue, GlobalSettingsError>;
    async fn reset_to_defaults(&self) -> Result<(), GlobalSettingsError>;
    fn subscribe_to_events(&self) -> broadcast::Receiver<GlobalSettingsEvent>;
}

pub struct DefaultGlobalSettingsService {
    settings: Arc<RwLock<GlobalDesktopSettings>>,
    persistence_provider: Arc<dyn SettingsPersistenceProvider>,
    event_sender: broadcast::Sender<GlobalSettingsEvent>,
    broadcast_capacity: usize,
}

impl DefaultGlobalSettingsService {
    pub fn new(
        persistence_provider: Arc<dyn SettingsPersistenceProvider>,
        broadcast_capacity: Option<usize>,
    ) -> Self {
        let capacity = broadcast_capacity.unwrap_or(DEFAULT_BROADCAST_CAPACITY);
        let (event_sender, _) = broadcast::channel::<GlobalSettingsEvent>(capacity);
        Self {
            settings: Arc::new(RwLock::new(GlobalDesktopSettings::default())),
            persistence_provider,
            event_sender,
            broadcast_capacity: capacity,
        }
    }
}

#[async_trait::async_trait]
impl GlobalSettingsService for DefaultGlobalSettingsService {
    async fn load_settings(&self) -> Result<(), GlobalSettingsError> {
        info!("Lade globale Einstellungen...");
        let loaded_settings = self.persistence_provider.load_global_settings().await?;
        
        // Validate loaded settings before applying them
        loaded_settings.validate_recursive().map_err(|reason| {
            error!("Validierung der geladenen Einstellungen fehlgeschlagen: {}. Standardeinstellungen werden verwendet.", reason);
            // Decide on behavior: return error, or load defaults?
            // Current persistence provider returns defaults on not found, but this is for corrupted/invalid.
            // For now, let's return an error if loaded settings are invalid.
            GlobalSettingsError::GlobalValidationFailed { reason }
        })?;

        let mut settings_guard = self.settings.write().await;
        *settings_guard = loaded_settings.clone();
        info!("Globale Einstellungen erfolgreich geladen und angewendet.");

        if self.event_sender.receiver_count() > 0 {
            let event = GlobalSettingsEvent::SettingsLoaded(SettingsLoadedEvent { settings: loaded_settings.clone() });
            if let Err(e) = self.event_sender.send(event) {
                warn!("Fehler beim Senden des SettingsLoadedEvent: {}", e);
            }
        }
        Ok(())
    }

    async fn save_settings(&self) -> Result<(), GlobalSettingsError> {
        info!("Speichere globale Einstellungen...");
        let settings_guard = self.settings.read().await;
        self.persistence_provider.save_global_settings(&*settings_guard).await?;
        info!("Globale Einstellungen erfolgreich gespeichert.");

        if self.event_sender.receiver_count() > 0 {
            let event = GlobalSettingsEvent::SettingsSaved(SettingsSavedEvent::default());
            if let Err(e) = self.event_sender.send(event) {
                warn!("Fehler beim Senden des SettingsSavedEvent: {}", e);
            }
        }
        Ok(())
    }

    fn get_current_settings(&self) -> Result<GlobalDesktopSettings, GlobalSettingsError> {
        match self.settings.try_read() {
            Ok(guard) => Ok(guard.clone()),
            Err(_) => {
                error!("Sperrfehler beim Lesen der aktuellen Einstellungen.");
                Err(GlobalSettingsError::InternalError(
                    "Fehler beim Erhalten des Read-Locks für Einstellungen. Möglicher Deadlock oder inkonsistenter Zustand.".to_string(),
                ))
            }
        }
    }

    async fn update_setting(&self, path: SettingPath, value: JsonValue) -> Result<(), GlobalSettingsError> {
        info!("Aktualisiere Einstellung unter Pfad '{}'", path);
        let mut settings_guard = self.settings.write().await;
        let mut new_settings_clone = (*settings_guard).clone();

        // Use helper to update the field
        update_field_in_settings(&mut new_settings_clone, &path, value.clone())?;
        
        // Validate the entire settings object after change
        new_settings_clone.validate_recursive().map_err(|err_msg| {
            error!("Validierung nach Update für Pfad '{}' fehlgeschlagen: {}", path, err_msg);
            GlobalSettingsError::ValidationError { path: path.clone(), reason: err_msg }
        })?;

        *settings_guard = new_settings_clone;
        
        if self.event_sender.receiver_count() > 0 {
            let event = GlobalSettingsEvent::SettingChanged(SettingChangedEvent { path: path.clone(), new_value: value });
            if let Err(e) = self.event_sender.send(event) {
                warn!("Fehler beim Senden des GlobalSettingsEvent::SettingChanged: {}. Empfängeranzahl: {}", e, self.event_sender.receiver_count());
            } else {
                info!("GlobalSettingsEvent::SettingChanged erfolgreich gesendet.");
            }
        } else {
            info!("Keine Empfänger für GlobalSettingsEvent::SettingChanged registriert.");
        }
        
        // Release lock before saving
        drop(settings_guard);
        
        // Persist changes
        // Note: save_settings itself might re-acquire a read lock.
        self.save_settings().await?;
        
        Ok(())
    }

    fn get_setting(&self, path: &SettingPath) -> Result<JsonValue, GlobalSettingsError> {
        let settings_guard = self.settings.try_read().map_err(|_| {
            error!("Sperrfehler beim Lesen der Einstellung für Pfad '{}'.", path);
            GlobalSettingsError::InternalError(format!("Fehler beim Erhalten des Read-Locks für Einstellungen beim Abrufen von Pfad '{}'.", path))
        })?;
        get_field_from_settings(&*settings_guard, path)
    }

    async fn reset_to_defaults(&self) -> Result<(), GlobalSettingsError> {
        info!("Setze globale Einstellungen auf Standardwerte zurück...");
        let default_settings = GlobalDesktopSettings::default();
        
        // Validate defaults before applying (should always pass if Default is well-behaved)
        default_settings.validate_recursive().map_err(|reason| {
            error!("Validierung der Standardeinstellungen fehlgeschlagen: {}. Dies sollte nicht passieren.", reason);
            GlobalSettingsError::InternalError(format!("Standardeinstellungen sind ungültig: {}", reason))
        })?;

        let mut settings_guard = self.settings.write().await;
        *settings_guard = default_settings.clone();
        info!("Globale Einstellungen auf Standardwerte zurückgesetzt und angewendet.");

        if self.event_sender.receiver_count() > 0 {
            let event = GlobalSettingsEvent::SettingsLoaded(SettingsLoadedEvent { settings: default_settings.clone() });
            if let Err(e) = self.event_sender.send(event) {
                warn!("Fehler beim Senden des SettingsLoadedEvent nach Reset: {}", e);
            }
        }
        
        drop(settings_guard);
        self.save_settings().await?; // This will also send a SettingsSaved event
        Ok(())
    }

    fn subscribe_to_events(&self) -> broadcast::Receiver<GlobalSettingsEvent> {
        self.event_sender.subscribe()
    }
}


// --- Path Navigation Helper Functions ---
// These are quite verbose due to the nested structure and strong typing.
// Macros could shorten this but might reduce clarity.

fn update_field_in_settings(
    settings: &mut GlobalDesktopSettings,
    path: &SettingPath,
    value: JsonValue,
) -> Result<(), GlobalSettingsError> {
    match path {
        SettingPath::Appearance(ap_path) => match ap_path {
            AppearanceSettingPath::ActiveThemeName => settings.appearance.active_theme_name = deserialize_field(path, value)?,
            AppearanceSettingPath::ColorScheme => settings.appearance.color_scheme = deserialize_field(path, value)?,
            AppearanceSettingPath::AccentColorToken => settings.appearance.accent_color_token = deserialize_field(path, value)?,
            AppearanceSettingPath::IconThemeName => settings.appearance.icon_theme_name = deserialize_field(path, value)?,
            AppearanceSettingPath::CursorThemeName => settings.appearance.cursor_theme_name = deserialize_field(path, value)?,
            AppearanceSettingPath::EnableAnimations => settings.appearance.enable_animations = deserialize_field(path, value)?,
            AppearanceSettingPath::InterfaceScalingFactor => settings.appearance.interface_scaling_factor = deserialize_field(path, value)?,
            AppearanceSettingPath::FontSettings(f_path) => match f_path {
                FontSettingPath::DefaultFontFamily => settings.appearance.font_settings.default_font_family = deserialize_field(path, value)?,
                FontSettingPath::DefaultFontSize => settings.appearance.font_settings.default_font_size = deserialize_field(path, value)?,
                FontSettingPath::MonospaceFontFamily => settings.appearance.font_settings.monospace_font_family = deserialize_field(path, value)?,
                FontSettingPath::DocumentFontFamily => settings.appearance.font_settings.document_font_family = deserialize_field(path, value)?,
                FontSettingPath::Hinting => settings.appearance.font_settings.hinting = deserialize_field(path, value)?,
                FontSettingPath::Antialiasing => settings.appearance.font_settings.antialiasing = deserialize_field(path, value)?,
            },
        },
        SettingPath::Workspace(ws_path) => match ws_path {
            WorkspaceSettingPath::DynamicWorkspaces => settings.workspace.dynamic_workspaces = deserialize_field(path, value)?,
            WorkspaceSettingPath::DefaultWorkspaceCount => settings.workspace.default_workspace_count = deserialize_field(path, value)?,
            WorkspaceSettingPath::WorkspaceSwitchingBehavior => settings.workspace.workspace_switching_behavior = deserialize_field(path, value)?,
            WorkspaceSettingPath::ShowWorkspaceIndicator => settings.workspace.show_workspace_indicator = deserialize_field(path, value)?,
        },
        SettingPath::InputBehavior(ib_path) => match ib_path {
            InputBehaviorSettingPath::MouseAccelerationProfile => settings.input_behavior.mouse_acceleration_profile = deserialize_field(path, value)?,
            InputBehaviorSettingPath::CustomMouseAccelerationFactor => settings.input_behavior.custom_mouse_acceleration_factor = deserialize_field(path, value)?,
            InputBehaviorSettingPath::MouseSensitivity => settings.input_behavior.mouse_sensitivity = deserialize_field(path, value)?,
            InputBehaviorSettingPath::NaturalScrollingMouse => settings.input_behavior.natural_scrolling_mouse = deserialize_field(path, value)?,
            InputBehaviorSettingPath::NaturalScrollingTouchpad => settings.input_behavior.natural_scrolling_touchpad = deserialize_field(path, value)?,
            InputBehaviorSettingPath::TapToClickTouchpad => settings.input_behavior.tap_to_click_touchpad = deserialize_field(path, value)?,
            InputBehaviorSettingPath::TouchpadPointerSpeed => settings.input_behavior.touchpad_pointer_speed = deserialize_field(path, value)?,
            InputBehaviorSettingPath::KeyboardRepeatDelayMs => settings.input_behavior.keyboard_repeat_delay_ms = deserialize_field(path, value)?,
            InputBehaviorSettingPath::KeyboardRepeatRateCps => settings.input_behavior.keyboard_repeat_rate_cps = deserialize_field(path, value)?,
        },
        SettingPath::PowerManagementPolicy(pm_path) => match pm_path {
            PowerManagementPolicySettingPath::ScreenBlankTimeoutAcSecs => settings.power_management_policy.screen_blank_timeout_ac_secs = deserialize_field(path, value)?,
            PowerManagementPolicySettingPath::ScreenBlankTimeoutBatterySecs => settings.power_management_policy.screen_blank_timeout_battery_secs = deserialize_field(path, value)?,
            PowerManagementPolicySettingPath::SuspendActionOnLidCloseAc => settings.power_management_policy.suspend_action_on_lid_close_ac = deserialize_field(path, value)?,
            PowerManagementPolicySettingPath::SuspendActionOnLidCloseBattery => settings.power_management_policy.suspend_action_on_lid_close_battery = deserialize_field(path, value)?,
            PowerManagementPolicySettingPath::AutomaticSuspendDelayAcSecs => settings.power_management_policy.automatic_suspend_delay_ac_secs = deserialize_field(path, value)?,
            PowerManagementPolicySettingPath::AutomaticSuspendDelayBatterySecs => settings.power_management_policy.automatic_suspend_delay_battery_secs = deserialize_field(path, value)?,
            PowerManagementPolicySettingPath::ShowBatteryPercentage => settings.power_management_policy.show_battery_percentage = deserialize_field(path, value)?,
        },
        SettingPath::DefaultApplications(da_path) => match da_path {
            DefaultApplicationsSettingPath::WebBrowserDesktopFile => settings.default_applications.web_browser_desktop_file = deserialize_field(path, value)?,
            DefaultApplicationsSettingPath::EmailClientDesktopFile => settings.default_applications.email_client_desktop_file = deserialize_field(path, value)?,
            DefaultApplicationsSettingPath::TerminalEmulatorDesktopFile => settings.default_applications.terminal_emulator_desktop_file = deserialize_field(path, value)?,
            DefaultApplicationsSettingPath::FileManagerDesktopFile => settings.default_applications.file_manager_desktop_file = deserialize_field(path, value)?,
            DefaultApplicationsSettingPath::MusicPlayerDesktopFile => settings.default_applications.music_player_desktop_file = deserialize_field(path, value)?,
            DefaultApplicationsSettingPath::VideoPlayerDesktopFile => settings.default_applications.video_player_desktop_file = deserialize_field(path, value)?,
            DefaultApplicationsSettingPath::ImageViewerDesktopFile => settings.default_applications.image_viewer_desktop_file = deserialize_field(path, value)?,
            DefaultApplicationsSettingPath::TextEditorDesktopFile => settings.default_applications.text_editor_desktop_file = deserialize_field(path, value)?,
        },
        SettingPath::Application(app_path) => {
            let app_settings_group = settings
                .application_settings
                .entry(app_path.app_id.clone())
                .or_default();
            app_settings_group
                .settings
                .insert(app_path.key.clone(), value);
        } // Add future top-level categories here
    }
    Ok(())
}

fn get_field_from_settings(
    settings: &GlobalDesktopSettings,
    path: &SettingPath,
) -> Result<JsonValue, GlobalSettingsError> {
    let value = match path {
        SettingPath::Appearance(ap_path) => match ap_path {
            AppearanceSettingPath::ActiveThemeName => serde_json::to_value(&settings.appearance.active_theme_name),
            AppearanceSettingPath::ColorScheme => serde_json::to_value(&settings.appearance.color_scheme),
            AppearanceSettingPath::AccentColorToken => serde_json::to_value(&settings.appearance.accent_color_token),
            AppearanceSettingPath::IconThemeName => serde_json::to_value(&settings.appearance.icon_theme_name),
            AppearanceSettingPath::CursorThemeName => serde_json::to_value(&settings.appearance.cursor_theme_name),
            AppearanceSettingPath::EnableAnimations => serde_json::to_value(&settings.appearance.enable_animations),
            AppearanceSettingPath::InterfaceScalingFactor => serde_json::to_value(&settings.appearance.interface_scaling_factor),
            AppearanceSettingPath::FontSettings(f_path) => match f_path {
                FontSettingPath::DefaultFontFamily => serde_json::to_value(&settings.appearance.font_settings.default_font_family),
                FontSettingPath::DefaultFontSize => serde_json::to_value(&settings.appearance.font_settings.default_font_size),
                FontSettingPath::MonospaceFontFamily => serde_json::to_value(&settings.appearance.font_settings.monospace_font_family),
                FontSettingPath::DocumentFontFamily => serde_json::to_value(&settings.appearance.font_settings.document_font_family),
                FontSettingPath::Hinting => serde_json::to_value(&settings.appearance.font_settings.hinting),
                FontSettingPath::Antialiasing => serde_json::to_value(&settings.appearance.font_settings.antialiasing),
            },
        },
        SettingPath::Workspace(ws_path) => match ws_path {
            WorkspaceSettingPath::DynamicWorkspaces => serde_json::to_value(&settings.workspace.dynamic_workspaces),
            WorkspaceSettingPath::DefaultWorkspaceCount => serde_json::to_value(&settings.workspace.default_workspace_count),
            WorkspaceSettingPath::WorkspaceSwitchingBehavior => serde_json::to_value(&settings.workspace.workspace_switching_behavior),
            WorkspaceSettingPath::ShowWorkspaceIndicator => serde_json::to_value(&settings.workspace.show_workspace_indicator),
        },
        SettingPath::InputBehavior(ib_path) => match ib_path {
            InputBehaviorSettingPath::MouseAccelerationProfile => serde_json::to_value(&settings.input_behavior.mouse_acceleration_profile),
            InputBehaviorSettingPath::CustomMouseAccelerationFactor => serde_json::to_value(&settings.input_behavior.custom_mouse_acceleration_factor),
            InputBehaviorSettingPath::MouseSensitivity => serde_json::to_value(&settings.input_behavior.mouse_sensitivity),
            InputBehaviorSettingPath::NaturalScrollingMouse => serde_json::to_value(&settings.input_behavior.natural_scrolling_mouse),
            InputBehaviorSettingPath::NaturalScrollingTouchpad => serde_json::to_value(&settings.input_behavior.natural_scrolling_touchpad),
            InputBehaviorSettingPath::TapToClickTouchpad => serde_json::to_value(&settings.input_behavior.tap_to_click_touchpad),
            InputBehaviorSettingPath::TouchpadPointerSpeed => serde_json::to_value(&settings.input_behavior.touchpad_pointer_speed),
            InputBehaviorSettingPath::KeyboardRepeatDelayMs => serde_json::to_value(&settings.input_behavior.keyboard_repeat_delay_ms),
            InputBehaviorSettingPath::KeyboardRepeatRateCps => serde_json::to_value(&settings.input_behavior.keyboard_repeat_rate_cps),
        },
        SettingPath::PowerManagementPolicy(pm_path) => match pm_path {
            PowerManagementPolicySettingPath::ScreenBlankTimeoutAcSecs => serde_json::to_value(&settings.power_management_policy.screen_blank_timeout_ac_secs),
            PowerManagementPolicySettingPath::ScreenBlankTimeoutBatterySecs => serde_json::to_value(&settings.power_management_policy.screen_blank_timeout_battery_secs),
            PowerManagementPolicySettingPath::SuspendActionOnLidCloseAc => serde_json::to_value(&settings.power_management_policy.suspend_action_on_lid_close_ac),
            PowerManagementPolicySettingPath::SuspendActionOnLidCloseBattery => serde_json::to_value(&settings.power_management_policy.suspend_action_on_lid_close_battery),
            PowerManagementPolicySettingPath::AutomaticSuspendDelayAcSecs => serde_json::to_value(&settings.power_management_policy.automatic_suspend_delay_ac_secs),
            PowerManagementPolicySettingPath::AutomaticSuspendDelayBatterySecs => serde_json::to_value(&settings.power_management_policy.automatic_suspend_delay_battery_secs),
            PowerManagementPolicySettingPath::ShowBatteryPercentage => serde_json::to_value(&settings.power_management_policy.show_battery_percentage),
        },
        SettingPath::DefaultApplications(da_path) => match da_path {
            DefaultApplicationsSettingPath::WebBrowserDesktopFile => serde_json::to_value(&settings.default_applications.web_browser_desktop_file),
            DefaultApplicationsSettingPath::EmailClientDesktopFile => serde_json::to_value(&settings.default_applications.email_client_desktop_file),
            DefaultApplicationsSettingPath::TerminalEmulatorDesktopFile => serde_json::to_value(&settings.default_applications.terminal_emulator_desktop_file),
            DefaultApplicationsSettingPath::FileManagerDesktopFile => serde_json::to_value(&settings.default_applications.file_manager_desktop_file),
            DefaultApplicationsSettingPath::MusicPlayerDesktopFile => serde_json::to_value(&settings.default_applications.music_player_desktop_file),
            DefaultApplicationsSettingPath::VideoPlayerDesktopFile => serde_json::to_value(&settings.default_applications.video_player_desktop_file),
            DefaultApplicationsSettingPath::ImageViewerDesktopFile => serde_json::to_value(&settings.default_applications.image_viewer_desktop_file),
            DefaultApplicationsSettingPath::TextEditorDesktopFile => serde_json::to_value(&settings.default_applications.text_editor_desktop_file),
        },
        SettingPath::Application(app_path) => {
            return settings
                .application_settings
                .get(&app_path.app_id)
                .ok_or_else(|| GlobalSettingsError::PathNotFound(path.clone()))
                .and_then(|app_group| {
                    app_group
                        .settings
                        .get(&app_path.key)
                        .cloned() // Clone the JsonValue
                        .ok_or_else(|| GlobalSettingsError::PathNotFound(path.clone()))
                });
            // This does not go through the final map_err, so it's slightly different.
            // However, the error types are compatible.
            // For direct JsonValue, no further serde_json::to_value is needed.
        }
    };
    value.map_err(|e| GlobalSettingsError::serialization_error(Some(path.to_string()), e))
}

// Helper for deserializing a JsonValue into a specific type for a field.
fn deserialize_field<T: serde::de::DeserializeOwned>(path: &SettingPath, value: JsonValue) -> Result<T, GlobalSettingsError> {
    serde_json::from_value(value.clone()).map_err(|e| {
        // Provide a preview of the value in the error to help diagnose.
        let value_preview = value.to_string();
        let preview_truncated = if value_preview.len() > 50 {
            format!("{}...", &value_preview[..50])
        } else {
            value_preview
        };
        error!("Fehler beim Deserialisieren des Feldes '{}': Wert '{}', Fehler: {}", path, preview_truncated, e);
        GlobalSettingsError::FieldDeserializationError {
            path: path.clone(),
            source_message: e.to_string(), // Include the source error message
        }
    })
}
