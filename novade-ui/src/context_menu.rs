// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Context Menu Module
//!
//! This module provides context menu functionality for the NovaDE desktop environment.
//! It handles the creation, display, and interaction with context menus throughout the UI.

use gtk4 as gtk;
use gtk::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;

use crate::error::UiError;
use crate::common::{UiResult, UiComponent};
use crate::styles::StyleManager;

/// Context menu manager that handles all context menus in the UI
pub struct ContextMenuManager {
    /// The GTK application
    app: gtk::Application,
    
    /// Style manager for theming
    style_manager: Arc<StyleManager>,
    
    /// Active context menus
    active_menus: Arc<Mutex<HashMap<String, ContextMenu>>>,
    
    /// Context menu templates
    templates: Arc<RwLock<HashMap<String, ContextMenuTemplate>>>,
}

/// Context menu template for creating context menus
pub struct ContextMenuTemplate {
    /// Template ID
    id: String,
    
    /// Template name
    name: String,
    
    /// Menu items
    items: Vec<ContextMenuItem>,
}

/// Context menu item
pub struct ContextMenuItem {
    /// Item ID
    id: String,
    
    /// Item label
    label: String,
    
    /// Item icon
    icon: Option<String>,
    
    /// Item action
    action: ContextMenuAction,
    
    /// Is the item enabled
    enabled: bool,
    
    /// Is the item visible
    visible: bool,
    
    /// Submenu items
    submenu: Option<Vec<ContextMenuItem>>,
}

/// Context menu action
pub enum ContextMenuAction {
    /// Execute a command
    Command(String),
    
    /// Execute a callback function
    Callback(Box<dyn Fn() -> UiResult<()> + Send + Sync>),
    
    /// Open a submenu
    Submenu,
    
    /// Separator (no action)
    Separator,
}

/// Active context menu
pub struct ContextMenu {
    /// Menu ID
    id: String,
    
    /// GTK popup menu
    menu: gtk::PopoverMenu,
    
    /// Menu position
    position: (i32, i32),
    
    /// Is the menu visible
    visible: bool,
    
    /// Menu items
    items: Vec<ContextMenuItem>,
}

impl ContextMenuManager {
    /// Creates a new context menu manager
    pub fn new(app: gtk::Application, style_manager: Arc<StyleManager>) -> Self {
        Self {
            app,
            style_manager,
            active_menus: Arc::new(Mutex::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Registers a context menu template
    pub fn register_template(&self, template: ContextMenuTemplate) -> UiResult<()> {
        let mut templates = self.templates.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on templates".to_string())
        })?;
        
        templates.insert(template.id.clone(), template);
        
        Ok(())
    }
    
    /// Creates a context menu from a template
    pub fn create_menu_from_template(&self, template_id: &str, position: (i32, i32)) -> UiResult<ContextMenu> {
        let templates = self.templates.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on templates".to_string())
        })?;
        
        let template = templates.get(template_id).ok_or_else(|| {
            UiError::NotFound(format!("Context menu template not found: {}", template_id))
        })?;
        
        let menu_id = format!("{}_{}", template_id, uuid::Uuid::new_v4());
        
        // Create the GTK menu
        let menu_model = self.build_menu_model(template)?;
        let menu = gtk::PopoverMenu::from_model(Some(&menu_model));
        
        // Set up the menu
        menu.set_has_arrow(false);
        menu.set_position(gtk::PositionType::Bottom);
        
        // Apply styles
        self.style_manager.apply_styles_to_widget(&menu, "context-menu")?;
        
        let context_menu = ContextMenu {
            id: menu_id.clone(),
            menu,
            position,
            visible: false,
            items: template.items.clone(),
        };
        
        // Store the menu
        let mut active_menus = self.active_menus.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on active menus".to_string())
        })?;
        
        active_menus.insert(menu_id, context_menu.clone());
        
        Ok(context_menu)
    }
    
    /// Builds a GTK menu model from a template
    fn build_menu_model(&self, template: &ContextMenuTemplate) -> UiResult<gio::MenuModel> {
        let menu = gio::Menu::new();
        
        for item in &template.items {
            match &item.action {
                ContextMenuAction::Separator => {
                    menu.append_section(None, &gio::Menu::new());
                }
                ContextMenuAction::Submenu => {
                    if let Some(submenu_items) = &item.submenu {
                        let submenu = gio::Menu::new();
                        
                        for submenu_item in submenu_items {
                            self.add_menu_item(&submenu, submenu_item)?;
                        }
                        
                        menu.append_submenu(Some(&item.label), &submenu);
                    }
                }
                _ => {
                    self.add_menu_item(&menu, item)?;
                }
            }
        }
        
        Ok(menu.upcast())
    }
    
    /// Adds a menu item to a GTK menu
    fn add_menu_item(&self, menu: &gio::Menu, item: &ContextMenuItem) -> UiResult<()> {
        if !item.visible {
            return Ok(());
        }
        
        let menu_item = gio::MenuItem::new(Some(&item.label), None);
        
        // Set the action
        match &item.action {
            ContextMenuAction::Command(command) => {
                let action_name = format!("app.{}", item.id);
                menu_item.set_action_and_target_value(Some(&action_name), None);
                
                // Register the action with the application
                let action = gio::SimpleAction::new(&item.id, None);
                let command = command.clone();
                
                action.connect_activate(move |_, _| {
                    // Execute the command
                    if let Err(e) = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&command)
                        .spawn() {
                        eprintln!("Failed to execute command: {}", e);
                    }
                });
                
                self.app.add_action(&action);
            }
            ContextMenuAction::Callback(callback) => {
                let action_name = format!("app.{}", item.id);
                menu_item.set_action_and_target_value(Some(&action_name), None);
                
                // Register the action with the application
                let action = gio::SimpleAction::new(&item.id, None);
                let callback = callback.clone();
                
                action.connect_activate(move |_, _| {
                    // Execute the callback
                    if let Err(e) = callback() {
                        eprintln!("Failed to execute callback: {}", e);
                    }
                });
                
                self.app.add_action(&action);
            }
            _ => {}
        }
        
        // Set the icon
        if let Some(icon) = &item.icon {
            menu_item.set_icon(&gio::Icon::for_string(icon).unwrap());
        }
        
        // Add the item to the menu
        menu.append_item(&menu_item);
        
        Ok(())
    }
    
    /// Shows a context menu at the specified position
    pub fn show_menu(&self, menu_id: &str, position: Option<(i32, i32)>) -> UiResult<()> {
        let mut active_menus = self.active_menus.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on active menus".to_string())
        })?;
        
        let menu = active_menus.get_mut(menu_id).ok_or_else(|| {
            UiError::NotFound(format!("Context menu not found: {}", menu_id))
        })?;
        
        // Update the position if provided
        if let Some(pos) = position {
            menu.position = pos;
        }
        
        // Show the menu
        menu.menu.set_pointing_to(&gdk::Rectangle::new(
            menu.position.0,
            menu.position.1,
            1,
            1,
        ));
        
        menu.menu.popup();
        menu.visible = true;
        
        Ok(())
    }
    
    /// Hides a context menu
    pub fn hide_menu(&self, menu_id: &str) -> UiResult<()> {
        let mut active_menus = self.active_menus.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on active menus".to_string())
        })?;
        
        let menu = active_menus.get_mut(menu_id).ok_or_else(|| {
            UiError::NotFound(format!("Context menu not found: {}", menu_id))
        })?;
        
        // Hide the menu
        menu.menu.popdown();
        menu.visible = false;
        
        Ok(())
    }
    
    /// Destroys a context menu
    pub fn destroy_menu(&self, menu_id: &str) -> UiResult<()> {
        let mut active_menus = self.active_menus.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on active menus".to_string())
        })?;
        
        if let Some(menu) = active_menus.remove(menu_id) {
            // Hide the menu
            menu.menu.popdown();
            
            // Destroy the menu
            menu.menu.unparent();
        }
        
        Ok(())
    }
    
    /// Creates a desktop context menu
    pub fn create_desktop_context_menu(&self, position: (i32, i32)) -> UiResult<ContextMenu> {
        // Create the template if it doesn't exist
        let templates = self.templates.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on templates".to_string())
        })?;
        
        if !templates.contains_key("desktop") {
            drop(templates);
            
            let template = ContextMenuTemplate {
                id: "desktop".to_string(),
                name: "Desktop Context Menu".to_string(),
                items: vec![
                    ContextMenuItem {
                        id: "new_folder".to_string(),
                        label: "New Folder".to_string(),
                        icon: Some("folder-new-symbolic".to_string()),
                        action: ContextMenuAction::Command("mkdir -p ~/Desktop/New\\ Folder".to_string()),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "new_file".to_string(),
                        label: "New File".to_string(),
                        icon: Some("document-new-symbolic".to_string()),
                        action: ContextMenuAction::Command("touch ~/Desktop/New\\ File".to_string()),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "separator1".to_string(),
                        label: "".to_string(),
                        icon: None,
                        action: ContextMenuAction::Separator,
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "paste".to_string(),
                        label: "Paste".to_string(),
                        icon: Some("edit-paste-symbolic".to_string()),
                        action: ContextMenuAction::Command("xclip -o > ~/Desktop/pasted_file".to_string()),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "separator2".to_string(),
                        label: "".to_string(),
                        icon: None,
                        action: ContextMenuAction::Separator,
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "display_settings".to_string(),
                        label: "Display Settings".to_string(),
                        icon: Some("preferences-desktop-display-symbolic".to_string()),
                        action: ContextMenuAction::Command("gnome-control-center display".to_string()),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "change_background".to_string(),
                        label: "Change Background".to_string(),
                        icon: Some("preferences-desktop-wallpaper-symbolic".to_string()),
                        action: ContextMenuAction::Command("gnome-control-center background".to_string()),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                ],
            };
            
            self.register_template(template)?;
        }
        
        // Create the menu
        self.create_menu_from_template("desktop", position)
    }
    
    /// Creates a window context menu
    pub fn create_window_context_menu(&self, position: (i32, i32)) -> UiResult<ContextMenu> {
        // Create the template if it doesn't exist
        let templates = self.templates.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on templates".to_string())
        })?;
        
        if !templates.contains_key("window") {
            drop(templates);
            
            let template = ContextMenuTemplate {
                id: "window".to_string(),
                name: "Window Context Menu".to_string(),
                items: vec![
                    ContextMenuItem {
                        id: "minimize".to_string(),
                        label: "Minimize".to_string(),
                        icon: Some("window-minimize-symbolic".to_string()),
                        action: ContextMenuAction::Callback(Box::new(|| {
                            // This would be implemented to minimize the window
                            Ok(())
                        })),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "maximize".to_string(),
                        label: "Maximize".to_string(),
                        icon: Some("window-maximize-symbolic".to_string()),
                        action: ContextMenuAction::Callback(Box::new(|| {
                            // This would be implemented to maximize the window
                            Ok(())
                        })),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "move".to_string(),
                        label: "Move".to_string(),
                        icon: None,
                        action: ContextMenuAction::Callback(Box::new(|| {
                            // This would be implemented to move the window
                            Ok(())
                        })),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "resize".to_string(),
                        label: "Resize".to_string(),
                        icon: None,
                        action: ContextMenuAction::Callback(Box::new(|| {
                            // This would be implemented to resize the window
                            Ok(())
                        })),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "separator1".to_string(),
                        label: "".to_string(),
                        icon: None,
                        action: ContextMenuAction::Separator,
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "always_on_top".to_string(),
                        label: "Always on Top".to_string(),
                        icon: None,
                        action: ContextMenuAction::Callback(Box::new(|| {
                            // This would be implemented to toggle always on top
                            Ok(())
                        })),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "separator2".to_string(),
                        label: "".to_string(),
                        icon: None,
                        action: ContextMenuAction::Separator,
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                    ContextMenuItem {
                        id: "close".to_string(),
                        label: "Close".to_string(),
                        icon: Some("window-close-symbolic".to_string()),
                        action: ContextMenuAction::Callback(Box::new(|| {
                            // This would be implemented to close the window
                            Ok(())
                        })),
                        enabled: true,
                        visible: true,
                        submenu: None,
                    },
                ],
            };
            
            self.register_template(template)?;
        }
        
        // Create the menu
        self.create_menu_from_template("window", position)
    }
}

impl Clone for ContextMenu {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            menu: self.menu.clone(),
            position: self.position,
            visible: self.visible,
            items: self.items.clone(),
        }
    }
}

impl Clone for ContextMenuItem {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            label: self.label.clone(),
            icon: self.icon.clone(),
            action: match &self.action {
                ContextMenuAction::Command(cmd) => ContextMenuAction::Command(cmd.clone()),
                ContextMenuAction::Callback(_) => ContextMenuAction::Callback(Box::new(|| Ok(()))),
                ContextMenuAction::Submenu => ContextMenuAction::Submenu,
                ContextMenuAction::Separator => ContextMenuAction::Separator,
            },
            enabled: self.enabled,
            visible: self.visible,
            submenu: self.submenu.clone(),
        }
    }
}

impl UiComponent for ContextMenuManager {
    fn init(&self) -> UiResult<()> {
        // Initialize default templates
        let desktop_template = ContextMenuTemplate {
            id: "desktop".to_string(),
            name: "Desktop Context Menu".to_string(),
            items: vec![
                ContextMenuItem {
                    id: "new_folder".to_string(),
                    label: "New Folder".to_string(),
                    icon: Some("folder-new-symbolic".to_string()),
                    action: ContextMenuAction::Command("mkdir -p ~/Desktop/New\\ Folder".to_string()),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "new_file".to_string(),
                    label: "New File".to_string(),
                    icon: Some("document-new-symbolic".to_string()),
                    action: ContextMenuAction::Command("touch ~/Desktop/New\\ File".to_string()),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "separator1".to_string(),
                    label: "".to_string(),
                    icon: None,
                    action: ContextMenuAction::Separator,
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "paste".to_string(),
                    label: "Paste".to_string(),
                    icon: Some("edit-paste-symbolic".to_string()),
                    action: ContextMenuAction::Command("xclip -o > ~/Desktop/pasted_file".to_string()),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "separator2".to_string(),
                    label: "".to_string(),
                    icon: None,
                    action: ContextMenuAction::Separator,
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "display_settings".to_string(),
                    label: "Display Settings".to_string(),
                    icon: Some("preferences-desktop-display-symbolic".to_string()),
                    action: ContextMenuAction::Command("gnome-control-center display".to_string()),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "change_background".to_string(),
                    label: "Change Background".to_string(),
                    icon: Some("preferences-desktop-wallpaper-symbolic".to_string()),
                    action: ContextMenuAction::Command("gnome-control-center background".to_string()),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
            ],
        };
        
        self.register_template(desktop_template)?;
        
        let window_template = ContextMenuTemplate {
            id: "window".to_string(),
            name: "Window Context Menu".to_string(),
            items: vec![
                ContextMenuItem {
                    id: "minimize".to_string(),
                    label: "Minimize".to_string(),
                    icon: Some("window-minimize-symbolic".to_string()),
                    action: ContextMenuAction::Callback(Box::new(|| {
                        // This would be implemented to minimize the window
                        Ok(())
                    })),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "maximize".to_string(),
                    label: "Maximize".to_string(),
                    icon: Some("window-maximize-symbolic".to_string()),
                    action: ContextMenuAction::Callback(Box::new(|| {
                        // This would be implemented to maximize the window
                        Ok(())
                    })),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "move".to_string(),
                    label: "Move".to_string(),
                    icon: None,
                    action: ContextMenuAction::Callback(Box::new(|| {
                        // This would be implemented to move the window
                        Ok(())
                    })),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "resize".to_string(),
                    label: "Resize".to_string(),
                    icon: None,
                    action: ContextMenuAction::Callback(Box::new(|| {
                        // This would be implemented to resize the window
                        Ok(())
                    })),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "separator1".to_string(),
                    label: "".to_string(),
                    icon: None,
                    action: ContextMenuAction::Separator,
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "always_on_top".to_string(),
                    label: "Always on Top".to_string(),
                    icon: None,
                    action: ContextMenuAction::Callback(Box::new(|| {
                        // This would be implemented to toggle always on top
                        Ok(())
                    })),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "separator2".to_string(),
                    label: "".to_string(),
                    icon: None,
                    action: ContextMenuAction::Separator,
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
                ContextMenuItem {
                    id: "close".to_string(),
                    label: "Close".to_string(),
                    icon: Some("window-close-symbolic".to_string()),
                    action: ContextMenuAction::Callback(Box::new(|| {
                        // This would be implemented to close the window
                        Ok(())
                    })),
                    enabled: true,
                    visible: true,
                    submenu: None,
                },
            ],
        };
        
        self.register_template(window_template)?;
        
        Ok(())
    }
    
    fn shutdown(&self) -> UiResult<()> {
        // Hide and destroy all active menus
        let active_menus = self.active_menus.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on active menus".to_string())
        })?;
        
        let menu_ids: Vec<String> = active_menus.keys().cloned().collect();
        
        drop(active_menus);
        
        for menu_id in menu_ids {
            self.destroy_menu(&menu_id)?;
        }
        
        Ok(())
    }
}
