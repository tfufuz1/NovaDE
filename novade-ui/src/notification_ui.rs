// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Notification UI Module
//!
//! This module provides UI components for displaying notifications in the NovaDE desktop environment.
//! It handles the rendering, interaction, and management of notification popups using the
//! new NotificationPopup widget.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib; // Required for glib::timeout_add_local and glib::Sender (if used directly here)
use std::sync::{Arc, Mutex as StdMutex}; // Using StdMutex for active_notifications
use std::collections::HashMap;
use std::time::Duration; // For glib::timeout_add_local
use tracing::{debug, error, info, warn};

use crate::error::UiError; // Assuming UiError is defined
use crate::common::{UiResult, UiComponent}; // Assuming these are defined
use crate::styles::StyleManager;
use crate::compositor_integration::{CompositorIntegration, SurfaceType};

// Use the new NotificationPopup widget and its related data structures
use crate::widgets::notification_popup::{
    NotificationPopup, 
    NotificationPopupData, 
    PopupAction, // This is crate::widgets::notification_popup::PopupAction
    NotificationPopupEvent
};
// For interacting with the D-Bus service layer
use crate::shell::ui_notification_service::UINotificationService;


// NotificationSettings and NotificationPosition enums can remain as they are.
#[derive(Clone, Debug)]
pub enum NotificationPosition { TopRight, TopCenter, TopLeft, BottomRight, BottomCenter, BottomLeft }

#[derive(Debug, Clone)] // Added Clone for settings
pub struct NotificationSettings {
    pub position: NotificationPosition,
    pub timeout_secs: u32, // Default timeout for non-resident notifications in seconds
    pub width: i32,
    pub spacing: i32,
    pub max_popups: usize, // Renamed from max_notifications for clarity (popups on screen)
    pub show_icons: bool, // This would be used by NotificationPopupData construction
    pub show_actions: bool, // This would be used by NotificationPopupData construction
    // pub enable_sounds: bool, // Sounds are not handled in this iteration
    // pub enable_animations: bool, // Animations are GTK theme dependent or custom widget work
}

impl Default for NotificationSettings {
    fn default() -> Self {
        NotificationSettings {
            position: NotificationPosition::TopRight,
            timeout_secs: 5, // seconds
            width: 350,    // pixels
            spacing: 10,   // pixels
            max_popups: 5,
            show_icons: true,
            show_actions: true,
        }
    }
}

/// Notification UI manager
pub struct NotificationUi {
    app: gtk::Application,
    style_manager: Arc<StyleManager>,
    compositor: Arc<CompositorIntegration>,
    // Key is D-Bus notification ID (u32). Value is the GTK Window hosting the popup.
    active_popups: Arc<StdMutex<HashMap<u32, gtk::Window>>>,
    settings: Arc<StdMutex<NotificationSettings>>,
    // To call back for actions/close, this needs to be set after UINotificationService is created.
    ui_notification_service: Arc<StdMutex<Option<Arc<UINotificationService>>>>,
    // Monitor where popups are placed. This is a simplified representation.
    // A real implementation would need detailed screen geometry.
    screen_width: i32, 
    screen_height: i32,
}


impl NotificationUi {
    pub fn new(
        app: gtk::Application,
        style_manager: Arc<StyleManager>,
        compositor: Arc<CompositorIntegration>,
    ) -> Self {
        // Get primary monitor dimensions (simplified)
        let (screen_width, screen_height) = match app.primary_monitor() {
            Some(monitor) => (monitor.geometry().width(), monitor.geometry().height()),
            None => {
                warn!("Could not get primary monitor, defaulting to 1920x1080 for layout.");
                (1920, 1080)
            }
        };

        Self {
            app,
            style_manager,
            compositor,
            active_popups: Arc::new(StdMutex::new(HashMap::new())),
            settings: Arc::new(StdMutex::new(NotificationSettings::default())),
            ui_notification_service: Arc::new(StdMutex::new(None)),
            screen_width,
            screen_height,
        }
    }

    pub fn set_ui_notification_service(&self, service: Arc<UINotificationService>) {
        let mut service_guard = self.ui_notification_service.lock().unwrap();
        *service_guard = Some(service);
        info!("UINotificationService linked to NotificationUi.");
    }
    
    // Simplified data structure for showing a notification via NotificationUi
    // This is constructed by UINotificationService from domain Notification or D-Bus data
    pub struct ShowNotificationParams {
        pub dbus_id: u32,
        pub app_name: String, // For context, not directly displayed by PopupWidget unless in title/body
        pub replaces_id: u32, // D-Bus ID of notification to replace (0 if new)
        pub title: String,
        pub body: String,
        pub icon_name: Option<String>,
        pub actions: Vec<PopupAction>, // Uses PopupAction from widgets::notification_popup
        pub resident: bool, // If true, doesn't auto-expire based on default timeout
        pub default_timeout_ms: i32, // Server provided timeout (-1 for server default, 0 for persistent)
        // pub urgency: UIPriority, // Could be used for styling or priority in queue
    }


    pub fn show_notification(&self, params: ShowNotificationParams) -> Result<(), String> {
        let dbus_id = params.dbus_id;
        debug!("NotificationUi: show_notification called for D-Bus ID {}", dbus_id);
        
        let mut popups_guard = self.active_popups.lock().unwrap();
        let settings = self.settings.lock().unwrap().clone(); // Clone settings for this operation

        // Handle replaces_id: if it's non-zero, dismiss the old one first.
        if params.replaces_id != 0 {
            if let Some(window_to_replace) = popups_guard.remove(&params.replaces_id) {
                info!("Replacing notification with D-Bus ID: {}", params.replaces_id);
                window_to_replace.destroy(); // Destroy the old window
                // Compositor surface destruction would happen here too if tracked per window
            }
        }

        // Limit number of popups
        if popups_guard.len() >= settings.max_popups {
            warn!("Max popups ({}) reached. New notification D-Bus ID {} not shown.", settings.max_popups, dbus_id);
            // Optionally, implement a queue or replace the oldest non-resident popup.
            // For now, just reject.
            return Err(format!("Max popups ({}) reached", settings.max_popups));
        }

        let popup_data = NotificationPopupData {
            notification_dbus_id: dbus_id,
            title: params.title,
            body: params.body,
            icon_name: if settings.show_icons { params.icon_name } else { None },
            actions: if settings.show_actions { params.actions } else { vec![] },
        };

        let service_opt_clone = self.ui_notification_service.lock().unwrap().clone();
        // Important: Clone Arcs for the callback
        let active_popups_cb_clone = self.active_popups.clone();
        let compositor_cb_clone = self.compositor.clone();


        let callback_fn = Arc::new(Box::new(move |event: NotificationPopupEvent| {
            match event {
                NotificationPopupEvent::Closed(id) => {
                    info!("Popup event: Closed for D-Bus ID {}", id);
                    if let Some(service) = &service_opt_clone {
                        let service_clone_for_close = service.clone();
                        // Offload to tokio runtime from UINotificationService
                        service.tokio_handle().spawn(async move {
                            if let Err(e) = service_clone_for_close.close_ui_notification(id).await {
                                error!("NotificationUi: Failed to request D-Bus close for ID {}: {}", id, e);
                            }
                        });
                    }
                    // Hide immediately. Actual removal from active_popups and destruction
                    // should happen in `notification_confirmed_closed` after server confirms.
                    if let Some(window) = active_popups_cb_clone.lock().unwrap().get(&id) {
                        window.hide(); 
                    }
                }
                NotificationPopupEvent::ActionInvoked(id, action_id) => {
                    info!("Popup event: ActionInvoked for D-Bus ID {}, action_id {}", id, action_id);
                    if let Some(service) = &service_opt_clone {
                        let service_clone_for_action = service.clone();
                        service.tokio_handle().spawn(async move {
                            service_clone_for_action.handle_ui_invoked_action(id, &action_id).await;
                        });
                    }
                }
            }
        }) as crate::widgets::notification_popup::PopupCallback);

        let notification_widget = NotificationPopup::new(popup_data, callback_fn);
        
        let window = gtk::Window::builder()
            .application(&self.app)
            // .title("NovaDE Notification") // Generally not needed for popups
            .child(notification_widget.widget())
            .decorated(false)
            .resizable(false)
            .can_focus(false) // Popups usually don't take focus
            .default_width(settings.width)
            // .css_name("notification-window") // Add a CSS name for the window itself if needed
            .build();
        
        // Style the window or widget if necessary (StyleManager can be used here)
        // self.style_manager.apply_styles_to_widget(&window, "notification-window-style-class");
        // self.style_manager.apply_styles_to_widget(notification_widget.widget(), "notification-popup-style-class");

        match self.compositor.create_surface(&window, SurfaceType::Notification) {
            Ok(_surface_data) => {
                info!("Compositor surface created for notification D-Bus ID {}", dbus_id);
            }
            Err(e) => {
                error!("Failed to create compositor surface for notification D-Bus ID {}: {}. Showing window directly.", dbus_id, e);
            }
        }
        
        window.present();
        popups_guard.insert(dbus_id, window);
        
        // Drop the main lock before calling reposition
        drop(popups_guard);
        self.reposition_all_notifications();


        // Handle auto-expiration
        let effective_timeout_ms = match params.default_timeout_ms {
            -1 => settings.timeout_secs * 1000, // Server default means use our configured default
            0 => 0, // Persistent, don't auto-close with this timer
            val => val as u32, // Use server-provided timeout
        };

        if !params.resident && effective_timeout_ms > 0 {
            let service_opt_clone_timeout = self.ui_notification_service.lock().unwrap().clone();
            let active_popups_timeout_clone = self.active_popups.clone();

            glib::timeout_add_local(Duration::from_millis(effective_timeout_ms.into()), move || {
                // Check if popup still exists and is managed by us
                if active_popups_timeout_clone.lock().unwrap().contains_key(&dbus_id) {
                    if let Some(service) = &service_opt_clone_timeout {
                        info!("Notification D-Bus ID {} timed out. Requesting close.", dbus_id);
                        let service_clone_for_timeout = service.clone();
                        service.tokio_handle().spawn(async move {
                            if let Err(e) = service_clone_for_timeout.close_ui_notification(dbus_id).await {
                                error!("NotificationUi: Failed to D-Bus close timed-out ID {}: {}", dbus_id, e);
                            }
                        });
                    } else {
                        // Fallback if service is not available: try to hide it directly.
                        // This path should ideally not be taken if UINotificationService is always set.
                        warn!("UINotificationService not available for timeout of ID {}. Hiding locally.", dbus_id);
                         if let Some(window) = active_popups_timeout_clone.lock().unwrap().remove(&dbus_id) {
                            window.destroy(); // Or just hide and let confirmed_closed handle full removal
                        }
                    }
                }
                glib::Continue(false)
            });
        }
        Ok(())
    }

    // Called by UINotificationService when the D-Bus server confirms a notification is closed
    // (e.g., after CloseNotification D-Bus call completes, or server emits NotificationClosed signal)
    pub fn notification_confirmed_closed(&self, dbus_id: u32) {
        debug!("NotificationUi: Confirmed closed for D-Bus ID {}", dbus_id);
        let mut popups_guard = self.active_popups.lock().unwrap();
        if let Some(window) = popups_guard.remove(&dbus_id) {
            // TODO: self.compositor.destroy_surface(...); needs surface ID to be tracked.
            // For now, just destroy the GTK window.
            window.destroy();
            info!("Removed and destroyed window for D-Bus ID {}", dbus_id);
            
            // Drop lock before calling reposition
            drop(popups_guard);
            self.reposition_all_notifications();
        } else {
            warn!("NotificationUi: Tried to confirm close for D-Bus ID {}, but it was not found in active_popups.", dbus_id);
        }
    }
    
    // Renamed from dismiss_notification. This is for programmatic dismissal from UI itself,
    // if needed, separate from server-side close.
    pub fn dismiss_ui_managed_popup(&self, dbus_id: u32) {
        info!("NotificationUi: Dismissing UI managed popup for D-Bus ID {}", dbus_id);
        // This would typically also involve calling close_ui_notification on UINotificationService
        if let Some(service) = self.ui_notification_service.lock().unwrap().clone() {
            let service_clone = service.clone();
            service.tokio_handle().spawn(async move {
                if let Err(e) = service_clone.close_ui_notification(dbus_id).await {
                    error!("Failed to D-Bus close notification ID {} during UI dismissal: {}", dbus_id, e);
                }
            });
        }
        // The actual removal and destruction will be handled by notification_confirmed_closed
        // when the D-Bus roundtrip completes (server confirms via signal or method reply).
        // For immediate visual feedback, we can hide it.
        if let Some(window) = self.active_popups.lock().unwrap().get(&dbus_id) {
            window.hide();
        }
    }

    fn reposition_all_notifications(&self) {
        let popups_guard = self.active_popups.lock().unwrap();
        let settings = self.settings.lock().unwrap().clone(); // Clone for this operation
        info!("Repositioning {} active popups...", popups_guard.len());

        let mut y_offset = settings.spacing; // Start with spacing from the edge
        // For bottom alignment, initial y_offset would be screen_height - settings.spacing

        // TODO: Iterate popups in desired order (e.g., by D-Bus ID or creation time if stored)
        // For now, iterating HashMap order is undefined, but sufficient for basic stacking.
        for (_id, window) in popups_guard.iter() {
            if !window.is_visible() { continue; }

            let window_width = window.width(); // This might be settings.width or actual allocated
            let window_height = window.height(); // Actual allocated height

            let x = match settings.position {
                NotificationPosition::TopLeft | NotificationPosition::BottomLeft => settings.spacing,
                NotificationPosition::TopCenter | NotificationPosition::BottomCenter => (self.screen_width - window_width) / 2,
                NotificationPosition::TopRight | NotificationPosition::BottomRight => self.screen_width - window_width - settings.spacing,
            };
            
            let current_y = match settings.position {
                NotificationPosition::TopRight | NotificationPosition::TopCenter | NotificationPosition::TopLeft => y_offset,
                NotificationPosition::BottomRight | NotificationPosition::BottomCenter | NotificationPosition::BottomLeft => self.screen_height - y_offset - window_height,
            };

            // self.compositor.move_surface(surface_id, x, current_y);
            // For GTK window directly:
            window.move_(x, current_y); // Note: window.move_ is not standard GTK4 for positioning on screen for layered shells.
                                        // Proper positioning requires compositor interaction (e.g. layer-shell protocol).
                                        // This .move_ might work for traditional WMs or if the window is a child of a layer surface.
                                        // For now, this is a placeholder for actual compositor-based positioning.
            debug!("Attempting to position popup at x={}, y={}", x, current_y);

            y_offset += window_height + settings.spacing;
        }
    }
    
    pub fn update_settings(&self, settings_update: impl FnOnce(&mut NotificationSettings)) -> UiResult<()> {
        let mut settings_guard = self.settings.lock().unwrap();
        settings_update(&mut *settings_guard);
        drop(settings_guard); // Release lock before calling reposition
        
        self.reposition_all_notifications();
        Ok(())
    }
}


impl UiComponent for NotificationUi {
    fn init(&self) -> UiResult<()> {
        info!("NotificationUi initialized.");
        // The old timer logic for expiration was per-popup and handled via glib::timeout_add_seconds_local
        // in the old `init`. Now, timeouts are set when a popup is shown.
        // Any global periodic checks could go here if needed.
        Ok(())
    }
    
    fn shutdown(&self) -> UiResult<()> {
        info!("NotificationUi shutting down...");
        let popups_guard = self.active_popups.lock().unwrap();
        for (_id, window) in popups_guard.iter() {
            // self.compositor.destroy_surface(...);
            window.destroy();
        }
        drop(popups_guard);
        self.active_popups.lock().unwrap().clear();
        info!("All active notification popups destroyed.");
        Ok(())
    }
}

// Clone implementation needs to be careful with Arc<Mutex<Option<...>>>
impl Clone for NotificationUi {
    fn clone(&self) -> Self {
        Self {
            app: self.app.clone(),
            style_manager: self.style_manager.clone(),
            compositor: self.compositor.clone(),
            active_popups: self.active_popups.clone(),
            settings: self.settings.clone(),
            ui_notification_service: self.ui_notification_service.clone(),
            screen_width: self.screen_width,
            screen_height: self.screen_height,
        }
    }
}
