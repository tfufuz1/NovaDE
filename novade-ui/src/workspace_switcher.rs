// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Workspace Switcher UI Module
//!
//! This module provides UI components for switching between workspaces in the NovaDE desktop environment.
//! It handles the visualization and interaction with virtual workspaces.

use gtk4 as gtk;
use gtk::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;

use crate::error::UiError;
use crate::common::{UiResult, UiComponent};
use crate::styles::StyleManager;
use crate::compositor_integration::{CompositorIntegration, SurfaceType};

/// Workspace switcher UI manager
pub struct WorkspaceSwitcherUi {
    /// The GTK application
    app: gtk::Application,
    
    /// Style manager for theming
    style_manager: Arc<StyleManager>,
    
    /// Compositor integration
    compositor: Arc<CompositorIntegration>,
    
    /// Workspace data
    workspaces: Arc<RwLock<HashMap<u32, WorkspaceData>>>,
    
    /// Current active workspace
    active_workspace: Arc<RwLock<u32>>,
    
    /// Workspace switcher widget
    widget: Arc<Mutex<Option<gtk::Widget>>>,
    
    /// Workspace switcher settings
    settings: Arc<RwLock<WorkspaceSwitcherSettings>>,
}

/// Workspace data
pub struct WorkspaceData {
    /// Workspace ID
    id: u32,
    
    /// Workspace name
    name: String,
    
    /// Workspace thumbnail
    thumbnail: Option<gdk::Texture>,
    
    /// Workspace windows
    windows: Vec<WindowThumbnail>,
    
    /// Is the workspace active
    active: bool,
    
    /// Workspace position (row, column)
    position: (u32, u32),
}

/// Window thumbnail
pub struct WindowThumbnail {
    /// Window ID
    id: String,
    
    /// Window title
    title: String,
    
    /// Window icon
    icon: Option<gdk::Texture>,
    
    /// Window thumbnail
    thumbnail: Option<gdk::Texture>,
    
    /// Window position
    position: (i32, i32),
    
    /// Window size
    size: (i32, i32),
}

/// Workspace switcher settings
pub struct WorkspaceSwitcherSettings {
    /// Workspace layout (rows, columns)
    pub layout: (u32, u32),
    
    /// Show workspace names
    pub show_names: bool,
    
    /// Show window thumbnails
    pub show_thumbnails: bool,
    
    /// Enable workspace animations
    pub enable_animations: bool,
    
    /// Workspace switcher position
    pub position: WorkspaceSwitcherPosition,
    
    /// Workspace switcher size
    pub size: WorkspaceSwitcherSize,
}

/// Workspace switcher position
pub enum WorkspaceSwitcherPosition {
    /// Top
    Top,
    
    /// Bottom
    Bottom,
    
    /// Left
    Left,
    
    /// Right
    Right,
    
    /// Center (overlay)
    Center,
}

/// Workspace switcher size
pub enum WorkspaceSwitcherSize {
    /// Small
    Small,
    
    /// Medium
    Medium,
    
    /// Large
    Large,
    
    /// Custom size
    Custom(i32, i32),
}

impl WorkspaceSwitcherUi {
    /// Creates a new workspace switcher UI manager
    pub fn new(
        app: gtk::Application,
        style_manager: Arc<StyleManager>,
        compositor: Arc<CompositorIntegration>,
    ) -> Self {
        Self {
            app,
            style_manager,
            compositor,
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            active_workspace: Arc::new(RwLock::new(0)),
            widget: Arc::new(Mutex::new(None)),
            settings: Arc::new(RwLock::new(WorkspaceSwitcherSettings {
                layout: (2, 2),
                show_names: true,
                show_thumbnails: true,
                enable_animations: true,
                position: WorkspaceSwitcherPosition::Bottom,
                size: WorkspaceSwitcherSize::Medium,
            })),
        }
    }
    
    /// Creates the workspace switcher widget
    pub fn create_widget(&self) -> UiResult<gtk::Widget> {
        // Create the main container
        let container = gtk::Box::new(gtk::Orientation::Vertical, 6);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        
        // Create the workspace grid
        let grid = gtk::Grid::new();
        grid.set_row_spacing(6);
        grid.set_column_spacing(6);
        grid.set_row_homogeneous(true);
        grid.set_column_homogeneous(true);
        
        // Add workspaces to the grid
        let workspaces = self.workspaces.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on workspaces".to_string())
        })?;
        
        let active_workspace = self.active_workspace.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on active workspace".to_string())
        })?;
        
        let settings = self.settings.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on settings".to_string())
        })?;
        
        for workspace in workspaces.values() {
            // Create the workspace widget
            let workspace_widget = self.create_workspace_widget(workspace, *active_workspace == workspace.id, &settings)?;
            
            // Add to the grid
            grid.attach(&workspace_widget, workspace.position.1 as i32, workspace.position.0 as i32, 1, 1);
        }
        
        container.append(&grid);
        
        // Store the widget
        let mut widget = self.widget.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on widget".to_string())
        })?;
        
        *widget = Some(container.clone());
        
        Ok(container.upcast())
    }
    
    /// Creates a workspace widget
    fn create_workspace_widget(
        &self,
        workspace: &WorkspaceData,
        active: bool,
        settings: &WorkspaceSwitcherSettings,
    ) -> UiResult<gtk::Widget> {
        // Create the workspace container
        let container = gtk::Box::new(gtk::Orientation::Vertical, 3);
        container.set_hexpand(true);
        container.set_vexpand(true);
        
        // Add CSS classes
        container.add_css_class("workspace");
        if active {
            container.add_css_class("active");
        }
        
        // Create the workspace thumbnail
        let thumbnail_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        thumbnail_container.set_hexpand(true);
        thumbnail_container.set_vexpand(true);
        
        // Set the size based on settings
        let (width, height) = match settings.size {
            WorkspaceSwitcherSize::Small => (100, 75),
            WorkspaceSwitcherSize::Medium => (150, 112),
            WorkspaceSwitcherSize::Large => (200, 150),
            WorkspaceSwitcherSize::Custom(w, h) => (w, h),
        };
        
        thumbnail_container.set_size_request(width, height);
        
        // Add window thumbnails if enabled
        if settings.show_thumbnails {
            for window in &workspace.windows {
                let window_widget = self.create_window_thumbnail(window)?;
                thumbnail_container.append(&window_widget);
            }
        }
        
        container.append(&thumbnail_container);
        
        // Add workspace name if enabled
        if settings.show_names {
            let name_label = gtk::Label::new(Some(&workspace.name));
            name_label.add_css_class("workspace-name");
            container.append(&name_label);
        }
        
        // Add click handler to switch to this workspace
        let workspace_id = workspace.id;
        let self_clone = self.clone();
        container.connect_button_press_event(move |_, _| {
            if let Err(e) = self_clone.switch_to_workspace(workspace_id) {
                eprintln!("Failed to switch to workspace: {}", e);
            }
            gtk::Inhibit(false)
        });
        
        Ok(container.upcast())
    }
    
    /// Creates a window thumbnail widget
    fn create_window_thumbnail(&self, window: &WindowThumbnail) -> UiResult<gtk::Widget> {
        // Create the window container
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        
        // Position the window
        container.set_margin_start(window.position.0);
        container.set_margin_top(window.position.1);
        container.set_size_request(window.size.0, window.size.1);
        
        // Add CSS classes
        container.add_css_class("window-thumbnail");
        
        // Add the thumbnail if available
        if let Some(thumbnail) = &window.thumbnail {
            let picture = gtk::Picture::new();
            picture.set_paintable(Some(thumbnail));
            picture.set_can_shrink(true);
            picture.set_keep_aspect_ratio(true);
            container.append(&picture);
        } else {
            // Fallback to a placeholder
            let placeholder = gtk::Box::new(gtk::Orientation::Vertical, 0);
            placeholder.set_size_request(window.size.0, window.size.1);
            placeholder.add_css_class("window-placeholder");
            container.append(&placeholder);
        }
        
        Ok(container.upcast())
    }
    
    /// Switches to a workspace
    pub fn switch_to_workspace(&self, workspace_id: u32) -> UiResult<()> {
        // Check if the workspace exists
        let workspaces = self.workspaces.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on workspaces".to_string())
        })?;
        
        if !workspaces.contains_key(&workspace_id) {
            return Err(UiError::NotFound(format!("Workspace not found: {}", workspace_id)));
        }
        
        // Update the active workspace
        let mut active_workspace = self.active_workspace.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on active workspace".to_string())
        })?;
        
        *active_workspace = workspace_id;
        
        // Notify the compositor of the workspace change
        // This would be implemented to communicate with the compositor
        
        // Update the UI
        self.update_ui()?;
        
        Ok(())
    }
    
    /// Updates the UI
    fn update_ui(&self) -> UiResult<()> {
        // Get the widget
        let widget = self.widget.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on widget".to_string())
        })?;
        
        if let Some(widget) = &*widget {
            // Remove the old widget
            if let Some(parent) = widget.parent() {
                parent.remove(widget);
            }
        }
        
        // Create a new widget
        self.create_widget()?;
        
        Ok(())
    }
    
    /// Adds a workspace
    pub fn add_workspace(&self, name: &str, position: (u32, u32)) -> UiResult<u32> {
        let mut workspaces = self.workspaces.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on workspaces".to_string())
        })?;
        
        // Generate a new workspace ID
        let id = workspaces.len() as u32;
        
        // Create the workspace
        let workspace = WorkspaceData {
            id,
            name: name.to_string(),
            thumbnail: None,
            windows: Vec::new(),
            active: false,
            position,
        };
        
        // Add the workspace
        workspaces.insert(id, workspace);
        
        // Update the UI
        self.update_ui()?;
        
        Ok(id)
    }
    
    /// Removes a workspace
    pub fn remove_workspace(&self, workspace_id: u32) -> UiResult<()> {
        let mut workspaces = self.workspaces.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on workspaces".to_string())
        })?;
        
        // Check if the workspace exists
        if !workspaces.contains_key(&workspace_id) {
            return Err(UiError::NotFound(format!("Workspace not found: {}", workspace_id)));
        }
        
        // Remove the workspace
        workspaces.remove(&workspace_id);
        
        // Update the active workspace if needed
        let mut active_workspace = self.active_workspace.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on active workspace".to_string())
        })?;
        
        if *active_workspace == workspace_id {
            // Switch to the first available workspace
            *active_workspace = workspaces.keys().next().copied().unwrap_or(0);
        }
        
        // Update the UI
        self.update_ui()?;
        
        Ok(())
    }
    
    /// Adds a window to a workspace
    pub fn add_window_to_workspace(
        &self,
        workspace_id: u32,
        window_id: &str,
        title: &str,
        position: (i32, i32),
        size: (i32, i32),
    ) -> UiResult<()> {
        let mut workspaces = self.workspaces.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on workspaces".to_string())
        })?;
        
        // Check if the workspace exists
        let workspace = workspaces.get_mut(&workspace_id).ok_or_else(|| {
            UiError::NotFound(format!("Workspace not found: {}", workspace_id))
        })?;
        
        // Create the window thumbnail
        let window = WindowThumbnail {
            id: window_id.to_string(),
            title: title.to_string(),
            icon: None,
            thumbnail: None,
            position,
            size,
        };
        
        // Add the window
        workspace.windows.push(window);
        
        // Update the UI
        self.update_ui()?;
        
        Ok(())
    }
    
    /// Removes a window from a workspace
    pub fn remove_window_from_workspace(&self, workspace_id: u32, window_id: &str) -> UiResult<()> {
        let mut workspaces = self.workspaces.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on workspaces".to_string())
        })?;
        
        // Check if the workspace exists
        let workspace = workspaces.get_mut(&workspace_id).ok_or_else(|| {
            UiError::NotFound(format!("Workspace not found: {}", workspace_id))
        })?;
        
        // Remove the window
        workspace.windows.retain(|w| w.id != window_id);
        
        // Update the UI
        self.update_ui()?;
        
        Ok(())
    }
    
    /// Updates the workspace switcher settings
    pub fn update_settings(&self, settings_update: impl FnOnce(&mut WorkspaceSwitcherSettings)) -> UiResult<()> {
        let mut settings = self.settings.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on settings".to_string())
        })?;
        
        settings_update(&mut settings);
        
        // Update the UI
        self.update_ui()?;
        
        Ok(())
    }
}

impl Clone for WorkspaceSwitcherUi {
    fn clone(&self) -> Self {
        Self {
            app: self.app.clone(),
            style_manager: self.style_manager.clone(),
            compositor: self.compositor.clone(),
            workspaces: self.workspaces.clone(),
            active_workspace: self.active_workspace.clone(),
            widget: self.widget.clone(),
            settings: self.settings.clone(),
        }
    }
}

impl Clone for WorkspaceSwitcherPosition {
    fn clone(&self) -> Self {
        match self {
            Self::Top => Self::Top,
            Self::Bottom => Self::Bottom,
            Self::Left => Self::Left,
            Self::Right => Self::Right,
            Self::Center => Self::Center,
        }
    }
}

impl Clone for WorkspaceSwitcherSize {
    fn clone(&self) -> Self {
        match self {
            Self::Small => Self::Small,
            Self::Medium => Self::Medium,
            Self::Large => Self::Large,
            Self::Custom(w, h) => Self::Custom(*w, *h),
        }
    }
}

impl UiComponent for WorkspaceSwitcherUi {
    fn init(&self) -> UiResult<()> {
        // Create default workspaces
        let mut workspaces = self.workspaces.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on workspaces".to_string())
        })?;
        
        // Add default workspaces
        workspaces.insert(0, WorkspaceData {
            id: 0,
            name: "Workspace 1".to_string(),
            thumbnail: None,
            windows: Vec::new(),
            active: true,
            position: (0, 0),
        });
        
        workspaces.insert(1, WorkspaceData {
            id: 1,
            name: "Workspace 2".to_string(),
            thumbnail: None,
            windows: Vec::new(),
            active: false,
            position: (0, 1),
        });
        
        workspaces.insert(2, WorkspaceData {
            id: 2,
            name: "Workspace 3".to_string(),
            thumbnail: None,
            windows: Vec::new(),
            active: false,
            position: (1, 0),
        });
        
        workspaces.insert(3, WorkspaceData {
            id: 3,
            name: "Workspace 4".to_string(),
            thumbnail: None,
            windows: Vec::new(),
            active: false,
            position: (1, 1),
        });
        
        // Set the active workspace
        let mut active_workspace = self.active_workspace.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on active workspace".to_string())
        })?;
        
        *active_workspace = 0;
        
        Ok(())
    }
    
    fn shutdown(&self) -> UiResult<()> {
        // Clean up resources
        let mut widget = self.widget.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on widget".to_string())
        })?;
        
        if let Some(widget) = &*widget {
            if let Some(parent) = widget.parent() {
                parent.remove(widget);
            }
        }
        
        *widget = None;
        
        Ok(())
    }
}
