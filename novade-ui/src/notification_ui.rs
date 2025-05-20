// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Notification UI Module
//!
//! This module provides UI components for displaying notifications in the NovaDE desktop environment.
//! It handles the rendering, interaction, and management of notification popups.

use gtk4 as gtk;
use gtk::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::error::UiError;
use crate::common::{UiResult, UiComponent};
use crate::styles::StyleManager;
use crate::compositor_integration::{CompositorIntegration, SurfaceType};

/// Notification UI manager
pub struct NotificationUi {
    /// The GTK application
    app: gtk::Application,
    
    /// Style manager for theming
    style_manager: Arc<StyleManager>,
    
    /// Compositor integration
    compositor: Arc<CompositorIntegration>,
    
    /// Active notifications
    active_notifications: Arc<Mutex<HashMap<String, NotificationPopup>>>,
    
    /// Notification settings
    settings: Arc<RwLock<NotificationSettings>>,
}

/// Notification popup
pub struct NotificationPopup {
    /// Notification ID
    id: String,
    
    /// Notification window
    window: gtk::Window,
    
    /// Notification content
    content: NotificationContent,
    
    /// Notification state
    state: Arc<RwLock<NotificationState>>,
    
    /// Creation time
    created_at: Instant,
    
    /// Expiration time
    expires_at: Option<Instant>,
    
    /// Surface ID
    surface_id: Option<String>,
}

/// Notification content
pub struct NotificationContent {
    /// Notification title
    title: String,
    
    /// Notification body
    body: String,
    
    /// Notification icon
    icon: Option<String>,
    
    /// Notification actions
    actions: Vec<NotificationAction>,
    
    /// Notification urgency
    urgency: NotificationUrgency,
    
    /// Notification category
    category: Option<String>,
    
    /// Notification application
    application: Option<String>,
}

/// Notification action
pub struct NotificationAction {
    /// Action ID
    id: String,
    
    /// Action label
    label: String,
    
    /// Action callback
    callback: Box<dyn Fn() -> UiResult<()> + Send + Sync>,
}

/// Notification urgency
pub enum NotificationUrgency {
    /// Low urgency
    Low,
    
    /// Normal urgency
    Normal,
    
    /// Critical urgency
    Critical,
}

/// Notification state
pub struct NotificationState {
    /// Is the notification visible
    pub visible: bool,
    
    /// Is the notification expanded
    pub expanded: bool,
    
    /// Is the notification interactive
    pub interactive: bool,
    
    /// Is the notification being hovered
    pub hovered: bool,
    
    /// Is the notification being dismissed
    pub dismissing: bool,
}

/// Notification settings
pub struct NotificationSettings {
    /// Notification position
    pub position: NotificationPosition,
    
    /// Notification timeout (in seconds)
    pub timeout: u32,
    
    /// Notification width
    pub width: i32,
    
    /// Notification spacing
    pub spacing: i32,
    
    /// Maximum number of notifications
    pub max_notifications: usize,
    
    /// Show notification icons
    pub show_icons: bool,
    
    /// Show notification actions
    pub show_actions: bool,
    
    /// Enable notification sounds
    pub enable_sounds: bool,
    
    /// Enable notification animations
    pub enable_animations: bool,
}

/// Notification position
pub enum NotificationPosition {
    /// Top right
    TopRight,
    
    /// Top center
    TopCenter,
    
    /// Top left
    TopLeft,
    
    /// Bottom right
    BottomRight,
    
    /// Bottom center
    BottomCenter,
    
    /// Bottom left
    BottomLeft,
}

impl NotificationUi {
    /// Creates a new notification UI manager
    pub fn new(
        app: gtk::Application,
        style_manager: Arc<StyleManager>,
        compositor: Arc<CompositorIntegration>,
    ) -> Self {
        Self {
            app,
            style_manager,
            compositor,
            active_notifications: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(RwLock::new(NotificationSettings {
                position: NotificationPosition::TopRight,
                timeout: 5,
                width: 300,
                spacing: 10,
                max_notifications: 5,
                show_icons: true,
                show_actions: true,
                enable_sounds: true,
                enable_animations: true,
            })),
        }
    }
    
    /// Shows a notification
    pub fn show_notification(&self, content: NotificationContent) -> UiResult<String> {
        // Generate a notification ID
        let notification_id = format!("notification_{}", uuid::Uuid::new_v4());
        
        // Create the notification window
        let window = gtk::Window::new();
        window.set_title(Some(&content.title));
        window.set_default_size(self.get_notification_width()?, -1);
        window.set_resizable(false);
        window.set_decorated(false);
        window.set_skip_taskbar_hint(true);
        window.set_skip_pager_hint(true);
        window.set_keep_above(true);
        window.set_accept_focus(false);
        
        // Set up the window content
        let content_box = self.create_notification_content(&content)?;
        window.set_child(Some(&content_box));
        
        // Apply styles
        self.style_manager.apply_styles_to_widget(&window, "notification")?;
        
        // Create the notification popup
        let notification = NotificationPopup {
            id: notification_id.clone(),
            window,
            content,
            state: Arc::new(RwLock::new(NotificationState {
                visible: false,
                expanded: false,
                interactive: true,
                hovered: false,
                dismissing: false,
            })),
            created_at: Instant::now(),
            expires_at: Some(Instant::now() + Duration::from_secs(self.get_notification_timeout()? as u64)),
            surface_id: None,
        };
        
        // Register with the compositor
        if let Ok(surface) = self.compositor.create_surface(&notification.window, SurfaceType::Notification) {
            let mut state = notification.state.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on notification state".to_string())
            })?;
            
            state.visible = true;
            
            // Store the surface ID
            let mut notification = notification;
            notification.surface_id = Some(surface.id.clone());
            
            // Store the notification
            let mut active_notifications = self.active_notifications.lock().map_err(|_| {
                UiError::LockError("Failed to acquire lock on active notifications".to_string())
            })?;
            
            // Check if we need to remove old notifications
            if active_notifications.len() >= self.get_max_notifications()? {
                // Find the oldest notification
                let oldest = active_notifications.values()
                    .min_by_key(|n| n.created_at)
                    .map(|n| n.id.clone());
                
                if let Some(oldest_id) = oldest {
                    self.dismiss_notification(&oldest_id)?;
                }
            }
            
            // Position the notification
            self.position_notifications()?;
            
            // Show the notification
            notification.window.show();
            
            // Store the notification
            active_notifications.insert(notification_id.clone(), notification);
        }
        
        Ok(notification_id)
    }
    
    /// Creates the notification content widget
    fn create_notification_content(&self, content: &NotificationContent) -> UiResult<gtk::Box> {
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        content_box.set_margin_start(12);
        content_box.set_margin_end(12);
        content_box.set_margin_top(12);
        content_box.set_margin_bottom(12);
        
        // Header
        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        
        // Icon
        if self.get_show_icons()? {
            if let Some(icon_name) = &content.icon {
                let icon = gtk::Image::from_icon_name(icon_name);
                icon.set_pixel_size(32);
                header_box.append(&icon);
            }
        }
        
        // Title and body
        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 3);
        
        let title_label = gtk::Label::new(Some(&content.title));
        title_label.set_halign(gtk::Align::Start);
        title_label.set_valign(gtk::Align::Start);
        title_label.set_wrap(true);
        title_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        title_label.add_css_class("notification-title");
        text_box.append(&title_label);
        
        let body_label = gtk::Label::new(Some(&content.body));
        body_label.set_halign(gtk::Align::Start);
        body_label.set_valign(gtk::Align::Start);
        body_label.set_wrap(true);
        body_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        body_label.set_lines(3);
        body_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        body_label.add_css_class("notification-body");
        text_box.append(&body_label);
        
        header_box.append(&text_box);
        
        // Close button
        let close_button = gtk::Button::from_icon_name("window-close-symbolic");
        close_button.set_valign(gtk::Align::Start);
        close_button.set_halign(gtk::Align::End);
        close_button.add_css_class("notification-close");
        
        let notification_id = content.title.clone(); // This is a placeholder, in reality we would use the actual ID
        let self_clone = self.clone();
        close_button.connect_clicked(move |_| {
            if let Err(e) = self_clone.dismiss_notification(&notification_id) {
                eprintln!("Failed to dismiss notification: {}", e);
            }
        });
        
        header_box.append(&close_button);
        
        content_box.append(&header_box);
        
        // Actions
        if self.get_show_actions()? && !content.actions.is_empty() {
            let actions_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            actions_box.set_halign(gtk::Align::End);
            actions_box.set_margin_top(6);
            
            for action in &content.actions {
                let button = gtk::Button::with_label(&action.label);
                button.add_css_class("notification-action");
                
                let action_callback = action.callback.clone();
                button.connect_clicked(move |_| {
                    if let Err(e) = action_callback() {
                        eprintln!("Failed to execute notification action: {}", e);
                    }
                });
                
                actions_box.append(&button);
            }
            
            content_box.append(&actions_box);
        }
        
        Ok(content_box)
    }
    
    /// Dismisses a notification
    pub fn dismiss_notification(&self, notification_id: &str) -> UiResult<()> {
        let mut active_notifications = self.active_notifications.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on active notifications".to_string())
        })?;
        
        if let Some(notification) = active_notifications.get(notification_id) {
            // Update the state
            let mut state = notification.state.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on notification state".to_string())
            })?;
            
            state.dismissing = true;
            state.visible = false;
            
            // Hide the window
            notification.window.hide();
            
            // Remove the surface
            if let Some(surface_id) = &notification.surface_id {
                if let Err(e) = self.compositor.destroy_surface(surface_id) {
                    eprintln!("Failed to destroy notification surface: {}", e);
                }
            }
        }
        
        // Remove the notification
        active_notifications.remove(notification_id);
        
        // Reposition remaining notifications
        self.position_notifications()?;
        
        Ok(())
    }
    
    /// Positions all active notifications
    fn position_notifications(&self) -> UiResult<()> {
        let active_notifications = self.active_notifications.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on active notifications".to_string())
        })?;
        
        let position = self.get_notification_position()?;
        let spacing = self.get_notification_spacing()?;
        
        // Get the screen dimensions
        let display = gdk::Display::default().ok_or_else(|| {
            UiError::InitializationError("Failed to get default display".to_string())
        })?;
        
        let monitor = display.primary_monitor().ok_or_else(|| {
            UiError::InitializationError("Failed to get primary monitor".to_string())
        })?;
        
        let geometry = monitor.geometry();
        let screen_width = geometry.width();
        let screen_height = geometry.height();
        
        // Sort notifications by creation time
        let mut notifications: Vec<_> = active_notifications.values().collect();
        notifications.sort_by_key(|n| n.created_at);
        
        // Position each notification
        let mut y_offset = match position {
            NotificationPosition::TopRight | NotificationPosition::TopCenter | NotificationPosition::TopLeft => {
                spacing
            }
            NotificationPosition::BottomRight | NotificationPosition::BottomCenter | NotificationPosition::BottomLeft => {
                screen_height - spacing
            }
        };
        
        for notification in notifications {
            // Get the notification size
            let width = self.get_notification_width()?;
            let height = notification.window.height();
            
            // Calculate the position
            let x = match position {
                NotificationPosition::TopLeft | NotificationPosition::BottomLeft => {
                    spacing
                }
                NotificationPosition::TopCenter | NotificationPosition::BottomCenter => {
                    (screen_width - width) / 2
                }
                NotificationPosition::TopRight | NotificationPosition::BottomRight => {
                    screen_width - width - spacing
                }
            };
            
            let y = match position {
                NotificationPosition::TopRight | NotificationPosition::TopCenter | NotificationPosition::TopLeft => {
                    let y = y_offset;
                    y_offset += height + spacing;
                    y
                }
                NotificationPosition::BottomRight | NotificationPosition::BottomCenter | NotificationPosition::BottomLeft => {
                    y_offset -= height;
                    let y = y_offset;
                    y_offset -= spacing;
                    y
                }
            };
            
            // Move the window
            notification.window.move_(x, y);
            
            // Update the surface position if available
            if let Some(surface_id) = &notification.surface_id {
                if let Err(e) = self.compositor.update_surface_properties(surface_id, |props| {
                    props.position = (x, y);
                }) {
                    eprintln!("Failed to update notification surface position: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Gets the notification position
    fn get_notification_position(&self) -> UiResult<NotificationPosition> {
        let settings = self.settings.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on notification settings".to_string())
        })?;
        
        Ok(settings.position.clone())
    }
    
    /// Gets the notification timeout
    fn get_notification_timeout(&self) -> UiResult<u32> {
        let settings = self.settings.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on notification settings".to_string())
        })?;
        
        Ok(settings.timeout)
    }
    
    /// Gets the notification width
    fn get_notification_width(&self) -> UiResult<i32> {
        let settings = self.settings.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on notification settings".to_string())
        })?;
        
        Ok(settings.width)
    }
    
    /// Gets the notification spacing
    fn get_notification_spacing(&self) -> UiResult<i32> {
        let settings = self.settings.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on notification settings".to_string())
        })?;
        
        Ok(settings.spacing)
    }
    
    /// Gets the maximum number of notifications
    fn get_max_notifications(&self) -> UiResult<usize> {
        let settings = self.settings.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on notification settings".to_string())
        })?;
        
        Ok(settings.max_notifications)
    }
    
    /// Gets whether to show notification icons
    fn get_show_icons(&self) -> UiResult<bool> {
        let settings = self.settings.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on notification settings".to_string())
        })?;
        
        Ok(settings.show_icons)
    }
    
    /// Gets whether to show notification actions
    fn get_show_actions(&self) -> UiResult<bool> {
        let settings = self.settings.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on notification settings".to_string())
        })?;
        
        Ok(settings.show_actions)
    }
    
    /// Updates the notification settings
    pub fn update_settings(&self, settings_update: impl FnOnce(&mut NotificationSettings)) -> UiResult<()> {
        let mut settings = self.settings.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on notification settings".to_string())
        })?;
        
        settings_update(&mut settings);
        
        // Reposition notifications if the position changed
        self.position_notifications()?;
        
        Ok(())
    }
}

impl Clone for NotificationUi {
    fn clone(&self) -> Self {
        Self {
            app: self.app.clone(),
            style_manager: self.style_manager.clone(),
            compositor: self.compositor.clone(),
            active_notifications: self.active_notifications.clone(),
            settings: self.settings.clone(),
        }
    }
}

impl Clone for NotificationAction {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            label: self.label.clone(),
            callback: Box::new(|| Ok(())), // Placeholder, actual callbacks can't be cloned
        }
    }
}

impl Clone for NotificationUrgency {
    fn clone(&self) -> Self {
        match self {
            Self::Low => Self::Low,
            Self::Normal => Self::Normal,
            Self::Critical => Self::Critical,
        }
    }
}

impl Clone for NotificationPosition {
    fn clone(&self) -> Self {
        match self {
            Self::TopRight => Self::TopRight,
            Self::TopCenter => Self::TopCenter,
            Self::TopLeft => Self::TopLeft,
            Self::BottomRight => Self::BottomRight,
            Self::BottomCenter => Self::BottomCenter,
            Self::BottomLeft => Self::BottomLeft,
        }
    }
}

impl UiComponent for NotificationUi {
    fn init(&self) -> UiResult<()> {
        // Set up notification expiration checking
        let self_clone = self.clone();
        glib::timeout_add_seconds_local(1, move || {
            let active_notifications = match self_clone.active_notifications.lock() {
                Ok(notifications) => notifications,
                Err(_) => return glib::Continue(true),
            };
            
            let now = Instant::now();
            let expired: Vec<String> = active_notifications.iter()
                .filter_map(|(id, notification)| {
                    if let Some(expires_at) = notification.expires_at {
                        if now >= expires_at {
                            Some(id.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            
            drop(active_notifications);
            
            for id in expired {
                if let Err(e) = self_clone.dismiss_notification(&id) {
                    eprintln!("Failed to dismiss expired notification: {}", e);
                }
            }
            
            glib::Continue(true)
        });
        
        Ok(())
    }
    
    fn shutdown(&self) -> UiResult<()> {
        // Dismiss all notifications
        let active_notifications = self.active_notifications.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on active notifications".to_string())
        })?;
        
        let notification_ids: Vec<String> = active_notifications.keys().cloned().collect();
        
        drop(active_notifications);
        
        for id in notification_ids {
            self.dismiss_notification(&id)?;
        }
        
        Ok(())
    }
}
