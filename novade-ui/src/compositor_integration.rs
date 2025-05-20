// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Compositor Integration Module
//!
//! This module provides integration between the UI layer and the compositor.
//! It handles the communication between the UI components and the underlying
//! compositor implementation.

use gtk4 as gtk;
use gtk::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;

use crate::error::UiError;
use crate::common::{UiResult, UiComponent};
use crate::styles::StyleManager;

/// Compositor integration manager
pub struct CompositorIntegration {
    /// The GTK application
    app: gtk::Application,
    
    /// Style manager for theming
    style_manager: Arc<StyleManager>,
    
    /// Window to surface mapping
    window_surfaces: Arc<RwLock<HashMap<String, CompositorSurface>>>,
    
    /// Is compositor integration enabled
    enabled: Arc<RwLock<bool>>,
    
    /// Compositor connection
    connection: Arc<Mutex<Option<CompositorConnection>>>,
}

/// Compositor connection
pub struct CompositorConnection {
    /// Connection type
    connection_type: CompositorConnectionType,
    
    /// Connection status
    status: ConnectionStatus,
    
    /// Compositor capabilities
    capabilities: CompositorCapabilities,
}

/// Compositor connection type
pub enum CompositorConnectionType {
    /// Wayland connection
    Wayland,
    
    /// X11 connection
    X11,
    
    /// Direct connection (embedded)
    Direct,
}

/// Connection status
pub enum ConnectionStatus {
    /// Connected
    Connected,
    
    /// Disconnected
    Disconnected,
    
    /// Error
    Error(String),
}

/// Compositor capabilities
pub struct CompositorCapabilities {
    /// Supports transparency
    pub transparency: bool,
    
    /// Supports blur
    pub blur: bool,
    
    /// Supports shadows
    pub shadows: bool,
    
    /// Supports rounded corners
    pub rounded_corners: bool,
    
    /// Supports window animations
    pub window_animations: bool,
    
    /// Supports window thumbnails
    pub window_thumbnails: bool,
    
    /// Supports window overview
    pub window_overview: bool,
    
    /// Supports window tiling
    pub window_tiling: bool,
}

/// Compositor surface
pub struct CompositorSurface {
    /// Surface ID
    id: String,
    
    /// Surface type
    surface_type: SurfaceType,
    
    /// Surface state
    state: Arc<RwLock<SurfaceState>>,
    
    /// Surface properties
    properties: Arc<RwLock<SurfaceProperties>>,
    
    /// Associated GTK widget
    widget: gtk::Widget,
}

/// Surface type
pub enum SurfaceType {
    /// Normal window
    Window,
    
    /// Popup window
    Popup,
    
    /// Dialog window
    Dialog,
    
    /// Panel
    Panel,
    
    /// Dock
    Dock,
    
    /// Desktop
    Desktop,
    
    /// Notification
    Notification,
}

/// Surface state
pub struct SurfaceState {
    /// Is the surface visible
    pub visible: bool,
    
    /// Is the surface active
    pub active: bool,
    
    /// Is the surface maximized
    pub maximized: bool,
    
    /// Is the surface fullscreen
    pub fullscreen: bool,
    
    /// Is the surface minimized
    pub minimized: bool,
    
    /// Is the surface above others
    pub above: bool,
    
    /// Is the surface below others
    pub below: bool,
    
    /// Is the surface sticky
    pub sticky: bool,
    
    /// Is the surface resizable
    pub resizable: bool,
    
    /// Is the surface movable
    pub movable: bool,
}

/// Surface properties
pub struct SurfaceProperties {
    /// Surface position
    pub position: (i32, i32),
    
    /// Surface size
    pub size: (i32, i32),
    
    /// Surface minimum size
    pub min_size: (i32, i32),
    
    /// Surface maximum size
    pub max_size: (i32, i32),
    
    /// Surface opacity
    pub opacity: f64,
    
    /// Surface corner radius
    pub corner_radius: i32,
    
    /// Surface shadow
    pub shadow: bool,
    
    /// Surface blur
    pub blur: bool,
    
    /// Surface layer
    pub layer: SurfaceLayer,
    
    /// Surface workspace
    pub workspace: i32,
}

/// Surface layer
pub enum SurfaceLayer {
    /// Background layer
    Background,
    
    /// Bottom layer
    Bottom,
    
    /// Normal layer
    Normal,
    
    /// Top layer
    Top,
    
    /// Overlay layer
    Overlay,
}

impl CompositorIntegration {
    /// Creates a new compositor integration manager
    pub fn new(app: gtk::Application, style_manager: Arc<StyleManager>) -> Self {
        Self {
            app,
            style_manager,
            window_surfaces: Arc::new(RwLock::new(HashMap::new())),
            enabled: Arc::new(RwLock::new(true)),
            connection: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Connects to the compositor
    pub fn connect(&self) -> UiResult<()> {
        let mut connection = self.connection.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on connection".to_string())
        })?;
        
        // Detect the compositor type
        let connection_type = self.detect_compositor_type()?;
        
        // Create the connection
        *connection = Some(CompositorConnection {
            connection_type,
            status: ConnectionStatus::Connected,
            capabilities: CompositorCapabilities {
                transparency: true,
                blur: true,
                shadows: true,
                rounded_corners: true,
                window_animations: true,
                window_thumbnails: true,
                window_overview: true,
                window_tiling: true,
            },
        });
        
        Ok(())
    }
    
    /// Detects the compositor type
    fn detect_compositor_type(&self) -> UiResult<CompositorConnectionType> {
        // Check if running under Wayland
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            return Ok(CompositorConnectionType::Wayland);
        }
        
        // Check if running under X11
        if std::env::var("DISPLAY").is_ok() {
            return Ok(CompositorConnectionType::X11);
        }
        
        // Default to direct connection
        Ok(CompositorConnectionType::Direct)
    }
    
    /// Disconnects from the compositor
    pub fn disconnect(&self) -> UiResult<()> {
        let mut connection = self.connection.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on connection".to_string())
        })?;
        
        // Update the connection status
        if let Some(conn) = connection.as_mut() {
            conn.status = ConnectionStatus::Disconnected;
        }
        
        // Clear the connection
        *connection = None;
        
        Ok(())
    }
    
    /// Gets the compositor capabilities
    pub fn get_capabilities(&self) -> UiResult<CompositorCapabilities> {
        let connection = self.connection.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on connection".to_string())
        })?;
        
        match &*connection {
            Some(conn) => Ok(conn.capabilities.clone()),
            None => Err(UiError::NotConnected("Compositor not connected".to_string())),
        }
    }
    
    /// Creates a new compositor surface
    pub fn create_surface(&self, widget: &gtk::Widget, surface_type: SurfaceType) -> UiResult<CompositorSurface> {
        // Check if connected
        let connection = self.connection.lock().map_err(|_| {
            UiError::LockError("Failed to acquire lock on connection".to_string())
        })?;
        
        if connection.is_none() {
            return Err(UiError::NotConnected("Compositor not connected".to_string()));
        }
        
        // Create a new surface ID
        let surface_id = format!("surface_{}", uuid::Uuid::new_v4());
        
        // Create the surface
        let surface = CompositorSurface {
            id: surface_id.clone(),
            surface_type,
            state: Arc::new(RwLock::new(SurfaceState {
                visible: true,
                active: false,
                maximized: false,
                fullscreen: false,
                minimized: false,
                above: false,
                below: false,
                sticky: false,
                resizable: true,
                movable: true,
            })),
            properties: Arc::new(RwLock::new(SurfaceProperties {
                position: (0, 0),
                size: (800, 600),
                min_size: (1, 1),
                max_size: (0, 0), // 0 means unlimited
                opacity: 1.0,
                corner_radius: 0,
                shadow: true,
                blur: false,
                layer: SurfaceLayer::Normal,
                workspace: 0,
            })),
            widget: widget.clone(),
        };
        
        // Store the surface
        let mut window_surfaces = self.window_surfaces.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on window surfaces".to_string())
        })?;
        
        window_surfaces.insert(surface_id, surface.clone());
        
        Ok(surface)
    }
    
    /// Destroys a compositor surface
    pub fn destroy_surface(&self, surface_id: &str) -> UiResult<()> {
        let mut window_surfaces = self.window_surfaces.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on window surfaces".to_string())
        })?;
        
        window_surfaces.remove(surface_id);
        
        Ok(())
    }
    
    /// Updates a surface state
    pub fn update_surface_state(&self, surface_id: &str, state_update: impl FnOnce(&mut SurfaceState)) -> UiResult<()> {
        let window_surfaces = self.window_surfaces.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on window surfaces".to_string())
        })?;
        
        let surface = window_surfaces.get(surface_id).ok_or_else(|| {
            UiError::NotFound(format!("Surface not found: {}", surface_id))
        })?;
        
        let mut state = surface.state.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on surface state".to_string())
        })?;
        
        state_update(&mut state);
        
        Ok(())
    }
    
    /// Updates surface properties
    pub fn update_surface_properties(&self, surface_id: &str, properties_update: impl FnOnce(&mut SurfaceProperties)) -> UiResult<()> {
        let window_surfaces = self.window_surfaces.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on window surfaces".to_string())
        })?;
        
        let surface = window_surfaces.get(surface_id).ok_or_else(|| {
            UiError::NotFound(format!("Surface not found: {}", surface_id))
        })?;
        
        let mut properties = surface.properties.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on surface properties".to_string())
        })?;
        
        properties_update(&mut properties);
        
        Ok(())
    }
    
    /// Gets a surface by ID
    pub fn get_surface(&self, surface_id: &str) -> UiResult<CompositorSurface> {
        let window_surfaces = self.window_surfaces.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on window surfaces".to_string())
        })?;
        
        let surface = window_surfaces.get(surface_id).ok_or_else(|| {
            UiError::NotFound(format!("Surface not found: {}", surface_id))
        })?;
        
        Ok(surface.clone())
    }
    
    /// Gets a surface by widget
    pub fn get_surface_by_widget(&self, widget: &gtk::Widget) -> UiResult<CompositorSurface> {
        let window_surfaces = self.window_surfaces.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on window surfaces".to_string())
        })?;
        
        for surface in window_surfaces.values() {
            if surface.widget == *widget {
                return Ok(surface.clone());
            }
        }
        
        Err(UiError::NotFound("Surface not found for widget".to_string()))
    }
    
    /// Enables compositor integration
    pub fn enable(&self) -> UiResult<()> {
        let mut enabled = self.enabled.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on enabled flag".to_string())
        })?;
        
        *enabled = true;
        
        Ok(())
    }
    
    /// Disables compositor integration
    pub fn disable(&self) -> UiResult<()> {
        let mut enabled = self.enabled.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on enabled flag".to_string())
        })?;
        
        *enabled = false;
        
        Ok(())
    }
    
    /// Checks if compositor integration is enabled
    pub fn is_enabled(&self) -> UiResult<bool> {
        let enabled = self.enabled.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on enabled flag".to_string())
        })?;
        
        Ok(*enabled)
    }
}

impl Clone for CompositorSurface {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            surface_type: self.surface_type.clone(),
            state: self.state.clone(),
            properties: self.properties.clone(),
            widget: self.widget.clone(),
        }
    }
}

impl Clone for SurfaceType {
    fn clone(&self) -> Self {
        match self {
            Self::Window => Self::Window,
            Self::Popup => Self::Popup,
            Self::Dialog => Self::Dialog,
            Self::Panel => Self::Panel,
            Self::Dock => Self::Dock,
            Self::Desktop => Self::Desktop,
            Self::Notification => Self::Notification,
        }
    }
}

impl Clone for SurfaceLayer {
    fn clone(&self) -> Self {
        match self {
            Self::Background => Self::Background,
            Self::Bottom => Self::Bottom,
            Self::Normal => Self::Normal,
            Self::Top => Self::Top,
            Self::Overlay => Self::Overlay,
        }
    }
}

impl Clone for CompositorCapabilities {
    fn clone(&self) -> Self {
        Self {
            transparency: self.transparency,
            blur: self.blur,
            shadows: self.shadows,
            rounded_corners: self.rounded_corners,
            window_animations: self.window_animations,
            window_thumbnails: self.window_thumbnails,
            window_overview: self.window_overview,
            window_tiling: self.window_tiling,
        }
    }
}

impl UiComponent for CompositorIntegration {
    fn init(&self) -> UiResult<()> {
        // Connect to the compositor
        self.connect()?;
        
        Ok(())
    }
    
    fn shutdown(&self) -> UiResult<()> {
        // Disconnect from the compositor
        self.disconnect()?;
        
        Ok(())
    }
}
