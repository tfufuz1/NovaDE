// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Quick Settings UI Module
//!
//! This module provides UI components for quick settings in the NovaDE desktop environment.
//! It handles the display and interaction with commonly used settings.

use gtk4 as gtk;
use gtk::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;

use crate::error::UiError;
use crate::common::{UiResult, UiComponent};
use crate::styles::StyleManager;
use crate::compositor_integration::{CompositorIntegration, SurfaceType};

/// Quick settings UI manager
pub struct QuickSettingsUi {
    /// The GTK application
    app: gtk::Application,
    
    /// Style manager for theming
    style_manager: Arc<StyleManager>,
    
    /// Compositor integration
    compositor: Arc<CompositorIntegration>,
    
    /// Quick settings panel
    panel: Arc<Mutex<Option<gtk::Window>>>,
    
    /// Quick settings items
    items: Arc<RwLock<HashMap<String, QuickSettingsItem>>>,
    
    /// Is the panel visible
    visible: Arc<RwLock<bool>>,
    
    /// Panel surface ID
    surface_id: Arc<RwLock<Option<String>>>,
}

/// Quick settings item
pub struct QuickSettingsItem {
    /// Item ID
    id: String,
    
    /// Item name
    name: String,
    
    /// Item icon
    icon: String,
    
    /// Item widget
    widget: gtk::Widget,
    
    /// Item type
    item_type: QuickSettingsItemType,
    
    /// Item state
    state: Arc<RwLock<QuickSettingsItemState>>,
}

/// Quick settings item type
pub enum QuickSettingsItemType {
    /// Toggle switch
    Toggle,
    
    /// Slider
    Slider,
    
    /// Button
    Button,
    
    /// Menu
    Menu,
    
    /// Custom widget
    Custom,
}

/// Quick settings item state
pub struct QuickSettingsItemState {
    /// Is the item enabled
    pub enabled: bool,
    
    /// Is the item visible
    pub visible: bool,
    
    /// Toggle state (for Toggle type)
    pub toggle_state: bool,
    
    /// Slider value (for Slider type)
    pub slider_value: f64,
    
    /// Slider range (for Slider type)
    pub slider_range: (f64, f64),
    
    /// Menu items (for Menu type)
    pub menu_items: Vec<String>,
    
    /// Selected menu item (for Menu type)
    pub selected_menu_item: Option<String>,
}

impl QuickSettingsUi {
    /// Creates a new quick settings UI manager
    pub fn new(
        app: gtk::Application,
        style_manager: Arc<StyleManager>,
        compositor: Arc<CompositorIntegration>,
    ) -> Self {
        Self {
            app,
            style_manager,
            compositor,
            panel: Arc::new(Mutex::new(None)),
            items: Arc::new(RwLock::new(HashMap::new())),
            visible: Arc::new(RwLock::new(false)),
            surface_id: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Creates the quick settings panel
    pub fn create_panel(&self) -> UiResult<gtk::Window> {
        // Create the panel window
        let window = gtk::Window::new();
        window.set_title(Some("Quick Settings"));
        window.set_default_size(320, -1);
        window.set_resizable(false);
        window.set_decorated(false);
        window.set_skip_taskbar_hint(true);
        window.set_skip_pager_hint(true);
        window.set_type_hint(gtk::gdk::WindowTypeHint::Popup);
        
        // Create the main container
        let container = gtk::Box::new(gtk::Orientation::Vertical, 6);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        
        // Create the grid for items
        let grid = gtk::Grid::new();
        grid.set_row_spacing(12);
        grid.set_column_spacing(12);
        
        // Add items to the grid
        let items = self.items.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on items".to_string())
        })?;
        
        let mut row = 0;
        for item in items.values() {
            let item_state = item.state.read().map_err(|_| {
                UiError::LockError("Failed to acquire read lock on item state".to_string())
            })?;
            
            if !item_state.visible {
                continue;
            }
            
            // Create the item container
            let item_container = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            
            // Add the icon
            let icon = gtk::Image::from_icon_name(&item.icon);
            icon.set_pixel_size(24);
            item_container.append(&icon);
            
            // Add the label
            let label = gtk::Label::new(Some(&item.name));
            label.set_halign(gtk::Align::Start);
            label.set_hexpand(true);
            item_container.append(&label);
            
            // Add the control widget based on item type
            match item.item_type {
                QuickSettingsItemType::Toggle => {
                    let switch = gtk::Switch::new();
                    switch.set_active(item_state.toggle_state);
                    switch.set_sensitive(item_state.enabled);
                    
                    let item_id = item.id.clone();
                    let self_clone = self.clone();
                    switch.connect_state_set(move |_, state| {
                        if let Err(e) = self_clone.set_toggle_state(&item_id, state) {
                            eprintln!("Failed to set toggle state: {}", e);
                        }
                        gtk::Inhibit(false)
                    });
                    
                    item_container.append(&switch);
                }
                QuickSettingsItemType::Slider => {
                    let scale = gtk::Scale::with_range(
                        gtk::Orientation::Horizontal,
                        item_state.slider_range.0,
                        item_state.slider_range.1,
                        0.1,
                    );
                    scale.set_value(item_state.slider_value);
                    scale.set_sensitive(item_state.enabled);
                    scale.set_draw_value(true);
                    scale.set_hexpand(true);
                    
                    let item_id = item.id.clone();
                    let self_clone = self.clone();
                    scale.connect_value_changed(move |scale| {
                        let value = scale.value();
                        if let Err(e) = self_clone.set_slider_value(&item_id, value) {
                            eprintln!("Failed to set slider value: {}", e);
                        }
                    });
                    
                    item_container.append(&scale);
                }
                QuickSettingsItemType::Button => {
                    let button = gtk::Button::with_label("Open");
                    button.set_sensitive(item_state.enabled);
                    
                    let item_id = item.id.clone();
                    let self_clone = self.clone();
                    button.connect_clicked(move |_| {
                        if let Err(e) = self_clone.activate_button(&item_id) {
                            eprintln!("Failed to activate button: {}", e);
                        }
                    });
                    
                    item_container.append(&button);
                }
                QuickSettingsItemType::Menu => {
                    let dropdown = gtk::DropDown::new(None::<gtk::StringList>, None::<gtk::Expression>);
                    let model = gtk::StringList::new(&item_state.menu_items);
                    dropdown.set_model(Some(&model));
                    dropdown.set_sensitive(item_state.enabled);
                    
                    if let Some(selected) = &item_state.selected_menu_item {
                        for (i, item) in item_state.menu_items.iter().enumerate() {
                            if item == selected {
                                dropdown.set_selected(i as u32);
                                break;
                            }
                        }
                    }
                    
                    let item_id = item.id.clone();
                    let self_clone = self.clone();
                    dropdown.connect_selected_notify(move |dropdown| {
                        let selected = dropdown.selected();
                        if let Err(e) = self_clone.set_selected_menu_item(&item_id, selected) {
                            eprintln!("Failed to set selected menu item: {}", e);
                        }
                    });
                    
                    item_container.append(&dropdown);
                }
                QuickSettingsItemType::Custom => {
                    // Custom widgets are already created
                    item_container.append(&item.widget.clone());
                }
            }
            
            // Add the item to the grid
            grid.attach(&item_container, 0, row, 1, 1);
            row += 1;
        }
        
        container.append(&grid);
        
        // Add a separator
        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
        separator.set_margin_top(6);
        separator.set_margin_bottom(6);
        container.append(&separator);
        
        // Add buttons at the bottom
        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        button_box.set_halign(gtk::Align::End);
        
        let settings_button = gtk::Button::with_label("Settings");
        settings_button.connect_clicked(move |_| {
            // Launch settings application
            if let Err(e) = std::process::Command::new("gnome-control-center").spawn() {
                eprintln!("Failed to launch settings: {}", e);
            }
        });
        
        button_box.append(&settings_button);
        container.append(&button_box);
        
        // Set the window content
        window.set_child(Some(&container));
        
        // Apply styles
        self.style_manager.apply_styles_to_widget(&window, "quick-settings")?;
        
        // Store the panel
        let mut panel = self.panel.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on panel".to_string())
        })?;
        
        *panel = Some(window.clone());
        
        Ok(window)
    }
    
    /// Shows the quick settings panel
    pub fn show_panel(&self, position: (i32, i32)) -> UiResult<()> {
        // Get or create the panel
        let panel = {
            let panel_lock = self.panel.lock().map_err(|_| {
                UiError::LockError("Failed to acquire lock on panel".to_string())
            })?;
            
            match &*panel_lock {
                Some(panel) => panel.clone(),
                None => {
                    drop(panel_lock);
                    self.create_panel()?
                }
            }
        };
        
        // Register with the compositor if needed
        let mut surface_id = self.surface_id.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on surface ID".to_string())
        })?;
        
        if surface_id.is_none() {
            if let Ok(surface) = self.compositor.create_surface(&panel, SurfaceType::Popup) {
                *surface_id = Some(surface.id.clone());
            }
        }
        
        // Position the panel
        panel.move_(position.0, position.1);
        
        // Update the surface position if available
        if let Some(id) = &*surface_id {
            if let Err(e) = self.compositor.update_surface_properties(id, |props| {
                props.position = position;
            }) {
                eprintln!("Failed to update quick settings surface position: {}", e);
            }
        }
        
        // Show the panel
        panel.show();
        
        // Update visibility state
        let mut visible = self.visible.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on visible flag".to_string())
        })?;
        
        *visible = true;
        
        Ok(())
    }
    
    /// Hides the quick settings panel
    pub fn hide_panel(&self) -> UiResult<()> {
        // Get the panel
        let panel_lock = self.panel.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on panel".to_string())
        })?;
        
        if let Some(panel) = &*panel_lock {
            // Hide the panel
            panel.hide();
        }
        
        // Update visibility state
        let mut visible = self.visible.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on visible flag".to_string())
        })?;
        
        *visible = false;
        
        Ok(())
    }
    
    /// Toggles the quick settings panel
    pub fn toggle_panel(&self, position: (i32, i32)) -> UiResult<()> {
        let visible = self.visible.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on visible flag".to_string())
        })?;
        
        if *visible {
            self.hide_panel()?;
        } else {
            self.show_panel(position)?;
        }
        
        Ok(())
    }
    
    /// Adds a toggle item
    pub fn add_toggle_item(&self, id: &str, name: &str, icon: &str, initial_state: bool) -> UiResult<()> {
        let mut items = self.items.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on items".to_string())
        })?;
        
        // Create the toggle widget
        let switch = gtk::Switch::new();
        switch.set_active(initial_state);
        
        // Create the item
        let item = QuickSettingsItem {
            id: id.to_string(),
            name: name.to_string(),
            icon: icon.to_string(),
            widget: switch.upcast(),
            item_type: QuickSettingsItemType::Toggle,
            state: Arc::new(RwLock::new(QuickSettingsItemState {
                enabled: true,
                visible: true,
                toggle_state: initial_state,
                slider_value: 0.0,
                slider_range: (0.0, 1.0),
                menu_items: Vec::new(),
                selected_menu_item: None,
            })),
        };
        
        // Add the item
        items.insert(id.to_string(), item);
        
        Ok(())
    }
    
    /// Adds a slider item
    pub fn add_slider_item(&self, id: &str, name: &str, icon: &str, initial_value: f64, range: (f64, f64)) -> UiResult<()> {
        let mut items = self.items.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on items".to_string())
        })?;
        
        // Create the slider widget
        let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, range.0, range.1, 0.1);
        scale.set_value(initial_value);
        scale.set_draw_value(true);
        scale.set_hexpand(true);
        
        // Create the item
        let item = QuickSettingsItem {
            id: id.to_string(),
            name: name.to_string(),
            icon: icon.to_string(),
            widget: scale.upcast(),
            item_type: QuickSettingsItemType::Slider,
            state: Arc::new(RwLock::new(QuickSettingsItemState {
                enabled: true,
                visible: true,
                toggle_state: false,
                slider_value: initial_value,
                slider_range: range,
                menu_items: Vec::new(),
                selected_menu_item: None,
            })),
        };
        
        // Add the item
        items.insert(id.to_string(), item);
        
        Ok(())
    }
    
    /// Adds a button item
    pub fn add_button_item(&self, id: &str, name: &str, icon: &str) -> UiResult<()> {
        let mut items = self.items.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on items".to_string())
        })?;
        
        // Create the button widget
        let button = gtk::Button::with_label("Open");
        
        // Create the item
        let item = QuickSettingsItem {
            id: id.to_string(),
            name: name.to_string(),
            icon: icon.to_string(),
            widget: button.upcast(),
            item_type: QuickSettingsItemType::Button,
            state: Arc::new(RwLock::new(QuickSettingsItemState {
                enabled: true,
                visible: true,
                toggle_state: false,
                slider_value: 0.0,
                slider_range: (0.0, 1.0),
                menu_items: Vec::new(),
                selected_menu_item: None,
            })),
        };
        
        // Add the item
        items.insert(id.to_string(), item);
        
        Ok(())
    }
    
    /// Adds a menu item
    pub fn add_menu_item(&self, id: &str, name: &str, icon: &str, menu_items: &[String], selected_item: Option<&str>) -> UiResult<()> {
        let mut items = self.items.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on items".to_string())
        })?;
        
        // Create the dropdown widget
        let model = gtk::StringList::new(menu_items);
        let dropdown = gtk::DropDown::new(Some(&model), None::<gtk::Expression>);
        
        if let Some(selected) = selected_item {
            for (i, item) in menu_items.iter().enumerate() {
                if item == selected {
                    dropdown.set_selected(i as u32);
                    break;
                }
            }
        }
        
        // Create the item
        let item = QuickSettingsItem {
            id: id.to_string(),
            name: name.to_string(),
            icon: icon.to_string(),
            widget: dropdown.upcast(),
            item_type: QuickSettingsItemType::Menu,
            state: Arc::new(RwLock::new(QuickSettingsItemState {
                enabled: true,
                visible: true,
                toggle_state: false,
                slider_value: 0.0,
                slider_range: (0.0, 1.0),
                menu_items: menu_items.to_vec(),
                selected_menu_item: selected_item.map(|s| s.to_string()),
            })),
        };
        
        // Add the item
        items.insert(id.to_string(), item);
        
        Ok(())
    }
    
    /// Adds a custom item
    pub fn add_custom_item(&self, id: &str, name: &str, icon: &str, widget: gtk::Widget) -> UiResult<()> {
        let mut items = self.items.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on items".to_string())
        })?;
        
        // Create the item
        let item = QuickSettingsItem {
            id: id.to_string(),
            name: name.to_string(),
            icon: icon.to_string(),
            widget,
            item_type: QuickSettingsItemType::Custom,
            state: Arc::new(RwLock::new(QuickSettingsItemState {
                enabled: true,
                visible: true,
                toggle_state: false,
                slider_value: 0.0,
                slider_range: (0.0, 1.0),
                menu_items: Vec::new(),
                selected_menu_item: None,
            })),
        };
        
        // Add the item
        items.insert(id.to_string(), item);
        
        Ok(())
    }
    
    /// Removes an item
    pub fn remove_item(&self, id: &str) -> UiResult<()> {
        let mut items = self.items.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on items".to_string())
        })?;
        
        items.remove(id);
        
        Ok(())
    }
    
    /// Sets the toggle state of an item
    pub fn set_toggle_state(&self, id: &str, state: bool) -> UiResult<()> {
        let items = self.items.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on items".to_string())
        })?;
        
        let item = items.get(id).ok_or_else(|| {
            UiError::NotFound(format!("Item not found: {}", id))
        })?;
        
        if let QuickSettingsItemType::Toggle = item.item_type {
            let mut item_state = item.state.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on item state".to_string())
            })?;
            
            item_state.toggle_state = state;
            
            // Emit signal or callback here
        }
        
        Ok(())
    }
    
    /// Sets the slider value of an item
    pub fn set_slider_value(&self, id: &str, value: f64) -> UiResult<()> {
        let items = self.items.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on items".to_string())
        })?;
        
        let item = items.get(id).ok_or_else(|| {
            UiError::NotFound(format!("Item not found: {}", id))
        })?;
        
        if let QuickSettingsItemType::Slider = item.item_type {
            let mut item_state = item.state.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on item state".to_string())
            })?;
            
            item_state.slider_value = value;
            
            // Emit signal or callback here
        }
        
        Ok(())
    }
    
    /// Activates a button item
    pub fn activate_button(&self, id: &str) -> UiResult<()> {
        let items = self.items.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on items".to_string())
        })?;
        
        let item = items.get(id).ok_or_else(|| {
            UiError::NotFound(format!("Item not found: {}", id))
        })?;
        
        if let QuickSettingsItemType::Button = item.item_type {
            // Emit signal or callback here
        }
        
        Ok(())
    }
    
    /// Sets the selected menu item
    pub fn set_selected_menu_item(&self, id: &str, selected: u32) -> UiResult<()> {
        let items = self.items.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on items".to_string())
        })?;
        
        let item = items.get(id).ok_or_else(|| {
            UiError::NotFound(format!("Item not found: {}", id))
        })?;
        
        if let QuickSettingsItemType::Menu = item.item_type {
            let mut item_state = item.state.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on item state".to_string())
            })?;
            
            if (selected as usize) < item_state.menu_items.len() {
                item_state.selected_menu_item = Some(item_state.menu_items[selected as usize].clone());
                
                // Emit signal or callback here
            }
        }
        
        Ok(())
    }
}

impl Clone for QuickSettingsUi {
    fn clone(&self) -> Self {
        Self {
            app: self.app.clone(),
            style_manager: self.style_manager.clone(),
            compositor: self.compositor.clone(),
            panel: self.panel.clone(),
            items: self.items.clone(),
            visible: self.visible.clone(),
            surface_id: self.surface_id.clone(),
        }
    }
}

impl UiComponent for QuickSettingsUi {
    fn init(&self) -> UiResult<()> {
        // Add default items
        self.add_toggle_item("wifi", "Wi-Fi", "network-wireless-symbolic", true)?;
        self.add_toggle_item("bluetooth", "Bluetooth", "bluetooth-active-symbolic", false)?;
        self.add_toggle_item("airplane", "Airplane Mode", "airplane-mode-symbolic", false)?;
        self.add_toggle_item("night_light", "Night Light", "night-light-symbolic", false)?;
        
        self.add_slider_item("volume", "Volume", "audio-volume-high-symbolic", 0.8, (0.0, 1.0))?;
        self.add_slider_item("brightness", "Brightness", "display-brightness-symbolic", 0.7, (0.0, 1.0))?;
        
        self.add_button_item("settings", "Settings", "preferences-system-symbolic")?;
        
        self.add_menu_item(
            "power_mode",
            "Power Mode",
            "battery-good-symbolic",
            &["Balanced".to_string(), "Power Saver".to_string(), "Performance".to_string()],
            Some("Balanced"),
        )?;
        
        Ok(())
    }
    
    fn shutdown(&self) -> UiResult<()> {
        // Hide the panel
        self.hide_panel()?;
        
        // Destroy the surface
        let surface_id = self.surface_id.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on surface ID".to_string())
        })?;
        
        if let Some(id) = &*surface_id {
            if let Err(e) = self.compositor.destroy_surface(id) {
                eprintln!("Failed to destroy quick settings surface: {}", e);
            }
        }
        
        // Clear the panel
        let mut panel = self.panel.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on panel".to_string())
        })?;
        
        *panel = None;
        
        Ok(())
    }
}
