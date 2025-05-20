// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # XDG Shell Implementation
//!
//! This module implements the XDG shell protocol for the compositor.

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use smithay::wayland::shell::xdg::{XdgShellHandler, XdgShellState, ToplevelSurface, PopupSurface, PositionerState, ResizeEdge};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::backend::ClientId;
use smithay::utils::{Logical, Point, Size, Rectangle};
use smithay::input::Seat;

use super::{CompositorError, CompositorResult};
use super::core::DesktopState;
use super::surface_management::{SurfaceData, SurfaceRole, SurfaceManager};

/// Managed window in the XDG shell
pub struct ManagedWindow {
    /// The toplevel surface
    pub toplevel: ToplevelSurface,
    
    /// Surface data
    pub surface_data: Arc<SurfaceData>,
    
    /// Window title
    pub title: Arc<RwLock<Option<String>>>,
    
    /// Window app ID
    pub app_id: Arc<RwLock<Option<String>>>,
    
    /// Window state
    pub state: Arc<RwLock<WindowState>>,
    
    /// Window manager data
    pub manager_data: Arc<RwLock<WindowManagerData>>,
}

/// Window state
#[derive(Debug, Clone)]
pub struct WindowState {
    /// Is the window maximized
    pub maximized: bool,
    
    /// Is the window fullscreen
    pub fullscreen: bool,
    
    /// Is the window minimized
    pub minimized: bool,
    
    /// Is the window activated (has focus)
    pub activated: bool,
    
    /// Window geometry
    pub geometry: Option<Rectangle<i32, Logical>>,
    
    /// Window position
    pub position: Point<i32, Logical>,
    
    /// Window size
    pub size: Size<i32, Logical>,
    
    /// Window minimum size
    pub min_size: Size<i32, Logical>,
    
    /// Window maximum size
    pub max_size: Size<i32, Logical>,
}

/// Window manager data
#[derive(Debug, Clone)]
pub struct WindowManagerData {
    /// Is the window being moved
    pub moving: bool,
    
    /// Is the window being resized
    pub resizing: bool,
    
    /// Resize edges
    pub resize_edges: Option<ResizeEdge>,
    
    /// Window workspace
    pub workspace: u32,
    
    /// Window layer
    pub layer: WindowLayer,
    
    /// Window opacity
    pub opacity: f64,
    
    /// Window z-index
    pub z_index: i32,
    
    /// Window decorations
    pub decorations: bool,
}

/// Window layer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowLayer {
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

impl ManagedWindow {
    /// Creates a new managed window
    pub fn new(toplevel: ToplevelSurface, surface_data: Arc<SurfaceData>) -> Self {
        Self {
            toplevel,
            surface_data,
            title: Arc::new(RwLock::new(None)),
            app_id: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(WindowState {
                maximized: false,
                fullscreen: false,
                minimized: false,
                activated: false,
                geometry: None,
                position: Point::from((0, 0)),
                size: Size::from((800, 600)),
                min_size: Size::from((1, 1)),
                max_size: Size::from((0, 0)), // 0 means unlimited
            })),
            manager_data: Arc::new(RwLock::new(WindowManagerData {
                moving: false,
                resizing: false,
                resize_edges: None,
                workspace: 0,
                layer: WindowLayer::Normal,
                opacity: 1.0,
                z_index: 0,
                decorations: true,
            })),
        }
    }
    
    /// Sets the window title
    pub fn set_title(&self, title: String) -> CompositorResult<()> {
        let mut title_lock = self.title.write().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire write lock on window title".to_string())
        })?;
        
        *title_lock = Some(title);
        Ok(())
    }
    
    /// Gets the window title
    pub fn get_title(&self) -> CompositorResult<Option<String>> {
        let title_lock = self.title.read().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire read lock on window title".to_string())
        })?;
        
        Ok(title_lock.clone())
    }
    
    /// Sets the window app ID
    pub fn set_app_id(&self, app_id: String) -> CompositorResult<()> {
        let mut app_id_lock = self.app_id.write().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire write lock on window app ID".to_string())
        })?;
        
        *app_id_lock = Some(app_id);
        Ok(())
    }
    
    /// Gets the window app ID
    pub fn get_app_id(&self) -> CompositorResult<Option<String>> {
        let app_id_lock = self.app_id.read().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire read lock on window app ID".to_string())
        })?;
        
        Ok(app_id_lock.clone())
    }
    
    /// Updates the window state
    pub fn update_state<F>(&self, state_update: F) -> CompositorResult<()>
    where
        F: FnOnce(&mut WindowState),
    {
        let mut state_lock = self.state.write().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire write lock on window state".to_string())
        })?;
        
        state_update(&mut state_lock);
        Ok(())
    }
    
    /// Gets the window state
    pub fn get_state(&self) -> CompositorResult<WindowState> {
        let state_lock = self.state.read().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire read lock on window state".to_string())
        })?;
        
        Ok(state_lock.clone())
    }
    
    /// Updates the window manager data
    pub fn update_manager_data<F>(&self, data_update: F) -> CompositorResult<()>
    where
        F: FnOnce(&mut WindowManagerData),
    {
        let mut data_lock = self.manager_data.write().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire write lock on window manager data".to_string())
        })?;
        
        data_update(&mut data_lock);
        Ok(())
    }
    
    /// Gets the window manager data
    pub fn get_manager_data(&self) -> CompositorResult<WindowManagerData> {
        let data_lock = self.manager_data.read().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire read lock on window manager data".to_string())
        })?;
        
        Ok(data_lock.clone())
    }
    
    /// Configures the window
    pub fn configure(&self, state: &mut DesktopState) -> CompositorResult<()> {
        let window_state = self.get_state()?;
        
        let mut states = Vec::new();
        if window_state.maximized {
            states.push(smithay::wayland::shell::xdg::ToplevelState::Maximized);
        }
        if window_state.fullscreen {
            states.push(smithay::wayland::shell::xdg::ToplevelState::Fullscreen);
        }
        if window_state.activated {
            states.push(smithay::wayland::shell::xdg::ToplevelState::Activated);
        }
        
        let size = if window_state.maximized || window_state.fullscreen {
            // Use output size for maximized or fullscreen windows
            let output_config = state.get_output_configuration()?;
            Some(output_config.size)
        } else {
            // Use window size for normal windows
            Some(window_state.size)
        };
        
        self.toplevel.with_pending_state(|state| {
            state.states = states;
            state.size = size;
        });
        
        self.toplevel.send_configure();
        
        Ok(())
    }
}

/// XDG shell manager for tracking and managing XDG shell surfaces
pub struct XdgShellManager {
    /// Toplevel windows
    toplevels: Arc<RwLock<HashMap<ToplevelSurface, Arc<ManagedWindow>>>>,
    
    /// Popup windows
    popups: Arc<RwLock<HashMap<PopupSurface, WlSurface>>>,
    
    /// Surface manager
    surface_manager: Arc<SurfaceManager>,
}

impl XdgShellManager {
    /// Creates a new XDG shell manager
    pub fn new(surface_manager: Arc<SurfaceManager>) -> Self {
        Self {
            toplevels: Arc::new(RwLock::new(HashMap::new())),
            popups: Arc::new(RwLock::new(HashMap::new())),
            surface_manager,
        }
    }
    
    /// Registers a toplevel window
    pub fn register_toplevel(&self, toplevel: ToplevelSurface, surface_data: Arc<SurfaceData>) -> CompositorResult<Arc<ManagedWindow>> {
        let mut toplevels = self.toplevels.write().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire write lock on toplevels".to_string())
        })?;
        
        let window = Arc::new(ManagedWindow::new(toplevel.clone(), surface_data));
        toplevels.insert(toplevel, window.clone());
        
        Ok(window)
    }
    
    /// Gets a toplevel window
    pub fn get_toplevel(&self, toplevel: &ToplevelSurface) -> CompositorResult<Option<Arc<ManagedWindow>>> {
        let toplevels = self.toplevels.read().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire read lock on toplevels".to_string())
        })?;
        
        Ok(toplevels.get(toplevel).cloned())
    }
    
    /// Removes a toplevel window
    pub fn remove_toplevel(&self, toplevel: &ToplevelSurface) -> CompositorResult<()> {
        let mut toplevels = self.toplevels.write().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire write lock on toplevels".to_string())
        })?;
        
        toplevels.remove(toplevel);
        
        Ok(())
    }
    
    /// Registers a popup window
    pub fn register_popup(&self, popup: PopupSurface, parent: WlSurface) -> CompositorResult<()> {
        let mut popups = self.popups.write().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire write lock on popups".to_string())
        })?;
        
        popups.insert(popup, parent);
        
        Ok(())
    }
    
    /// Gets a popup window's parent
    pub fn get_popup_parent(&self, popup: &PopupSurface) -> CompositorResult<Option<WlSurface>> {
        let popups = self.popups.read().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire read lock on popups".to_string())
        })?;
        
        Ok(popups.get(popup).cloned())
    }
    
    /// Removes a popup window
    pub fn remove_popup(&self, popup: &PopupSurface) -> CompositorResult<()> {
        let mut popups = self.popups.write().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire write lock on popups".to_string())
        })?;
        
        popups.remove(popup);
        
        Ok(())
    }
    
    /// Gets all toplevel windows
    pub fn get_all_toplevels(&self) -> CompositorResult<Vec<Arc<ManagedWindow>>> {
        let toplevels = self.toplevels.read().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire read lock on toplevels".to_string())
        })?;
        
        Ok(toplevels.values().cloned().collect())
    }
    
    /// Gets toplevel windows by workspace
    pub fn get_toplevels_by_workspace(&self, workspace: u32) -> CompositorResult<Vec<Arc<ManagedWindow>>> {
        let toplevels = self.toplevels.read().map_err(|_| {
            CompositorError::XdgShellError("Failed to acquire read lock on toplevels".to_string())
        })?;
        
        let mut result = Vec::new();
        
        for window in toplevels.values() {
            if let Ok(manager_data) = window.get_manager_data() {
                if manager_data.workspace == workspace {
                    result.push(window.clone());
                }
            }
        }
        
        Ok(result)
    }
}

/// Implementation of the XDG shell handler for the desktop state
impl XdgShellHandler for DesktopState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }
    
    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        // Get the underlying wl_surface
        let wl_surface = surface.wl_surface().clone();
        
        // Create surface data for the toplevel
        let attributes = smithay::wayland::compositor::get_surface_attributes(&wl_surface).unwrap();
        
        // Register the surface with the surface manager
        let surface_manager = SurfaceManager::new();
        let surface_data = match surface_manager.register_surface(wl_surface.clone(), attributes, SurfaceRole::XdgToplevel) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to register toplevel surface: {}", e);
                return;
            }
        };
        
        // Create a managed window
        let xdg_shell_manager = XdgShellManager::new(Arc::new(surface_manager));
        let window = match xdg_shell_manager.register_toplevel(surface.clone(), surface_data) {
            Ok(window) => window,
            Err(e) => {
                tracing::error!("Failed to register toplevel window: {}", e);
                return;
            }
        };
        
        // Configure the window
        if let Err(e) = window.configure(self) {
            tracing::error!("Failed to configure toplevel window: {}", e);
        }
    }
    
    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        // Get the underlying wl_surface
        let wl_surface = surface.wl_surface().clone();
        
        // Get the parent surface
        let parent = surface.get_parent_surface().unwrap();
        
        // Create surface data for the popup
        let attributes = smithay::wayland::compositor::get_surface_attributes(&wl_surface).unwrap();
        
        // Register the surface with the surface manager
        let surface_manager = SurfaceManager::new();
        let surface_data = match surface_manager.register_surface(wl_surface.clone(), attributes, SurfaceRole::XdgPopup) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to register popup surface: {}", e);
                return;
            }
        };
        
        // Register the popup with the XDG shell manager
        let xdg_shell_manager = XdgShellManager::new(Arc::new(surface_manager));
        if let Err(e) = xdg_shell_manager.register_popup(surface.clone(), parent) {
            tracing::error!("Failed to register popup: {}", e);
            return;
        }
        
        // Configure the popup
        let geometry = positioner.get_geometry();
        surface.with_pending_state(|state| {
            state.geometry = geometry;
        });
        surface.send_configure();
    }
    
    fn move_request(&mut self, surface: ToplevelSurface, seat: Seat<Self>, serial: ClientId) {
        // Get the managed window
        let xdg_shell_manager = XdgShellManager::new(Arc::new(SurfaceManager::new()));
        let window = match xdg_shell_manager.get_toplevel(&surface) {
            Ok(Some(window)) => window,
            Ok(None) => {
                tracing::error!("Move request for unknown toplevel surface");
                return;
            }
            Err(e) => {
                tracing::error!("Failed to get toplevel window: {}", e);
                return;
            }
        };
        
        // Update the window state
        if let Err(e) = window.update_manager_data(|data| {
            data.moving = true;
        }) {
            tracing::error!("Failed to update window manager data: {}", e);
            return;
        }
        
        // Implementation would handle the actual move operation
        // This would typically involve grabbing the pointer and updating
        // the window position as the pointer moves
    }
    
    fn resize_request(&mut self, surface: ToplevelSurface, seat: Seat<Self>, serial: ClientId, edges: ResizeEdge) {
        // Get the managed window
        let xdg_shell_manager = XdgShellManager::new(Arc::new(SurfaceManager::new()));
        let window = match xdg_shell_manager.get_toplevel(&surface) {
            Ok(Some(window)) => window,
            Ok(None) => {
                tracing::error!("Resize request for unknown toplevel surface");
                return;
            }
            Err(e) => {
                tracing::error!("Failed to get toplevel window: {}", e);
                return;
            }
        };
        
        // Update the window state
        if let Err(e) = window.update_manager_data(|data| {
            data.resizing = true;
            data.resize_edges = Some(edges);
        }) {
            tracing::error!("Failed to update window manager data: {}", e);
            return;
        }
        
        // Implementation would handle the actual resize operation
        // This would typically involve grabbing the pointer and updating
        // the window size as the pointer moves
    }
    
    fn maximize_request(&mut self, surface: ToplevelSurface) {
        // Get the managed window
        let xdg_shell_manager = XdgShellManager::new(Arc::new(SurfaceManager::new()));
        let window = match xdg_shell_manager.get_toplevel(&surface) {
            Ok(Some(window)) => window,
            Ok(None) => {
                tracing::error!("Maximize request for unknown toplevel surface");
                return;
            }
            Err(e) => {
                tracing::error!("Failed to get toplevel window: {}", e);
                return;
            }
        };
        
        // Update the window state
        if let Err(e) = window.update_state(|state| {
            state.maximized = true;
        }) {
            tracing::error!("Failed to update window state: {}", e);
            return;
        }
        
        // Configure the window
        if let Err(e) = window.configure(self) {
            tracing::error!("Failed to configure window: {}", e);
            return;
        }
    }
    
    fn unmaximize_request(&mut self, surface: ToplevelSurface) {
        // Get the managed window
        let xdg_shell_manager = XdgShellManager::new(Arc::new(SurfaceManager::new()));
        let window = match xdg_shell_manager.get_toplevel(&surface) {
            Ok(Some(window)) => window,
            Ok(None) => {
                tracing::error!("Unmaximize request for unknown toplevel surface");
                return;
            }
            Err(e) => {
                tracing::error!("Failed to get toplevel window: {}", e);
                return;
            }
        };
        
        // Update the window state
        if let Err(e) = window.update_state(|state| {
            state.maximized = false;
        }) {
            tracing::error!("Failed to update window state: {}", e);
            return;
        }
        
        // Configure the window
        if let Err(e) = window.configure(self) {
            tracing::error!("Failed to configure window: {}", e);
            return;
        }
    }
    
    fn fullscreen_request(&mut self, surface: ToplevelSurface, output: Option<smithay::wayland::output::Output>) {
        // Get the managed window
        let xdg_shell_manager = XdgShellManager::new(Arc::new(SurfaceManager::new()));
        let window = match xdg_shell_manager.get_toplevel(&surface) {
            Ok(Some(window)) => window,
            Ok(None) => {
                tracing::error!("Fullscreen request for unknown toplevel surface");
                return;
            }
            Err(e) => {
                tracing::error!("Failed to get toplevel window: {}", e);
                return;
            }
        };
        
        // Update the window state
        if let Err(e) = window.update_state(|state| {
            state.fullscreen = true;
        }) {
            tracing::error!("Failed to update window state: {}", e);
            return;
        }
        
        // Configure the window
        if let Err(e) = window.configure(self) {
            tracing::error!("Failed to configure window: {}", e);
            return;
        }
    }
    
    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        // Get the managed window
        let xdg_shell_manager = XdgShellManager::new(Arc::new(SurfaceManager::new()));
        let window = match xdg_shell_manager.get_toplevel(&surface) {
            Ok(Some(window)) => window,
            Ok(None) => {
                tracing::error!("Unfullscreen request for unknown toplevel surface");
                return;
            }
            Err(e) => {
                tracing::error!("Failed to get toplevel window: {}", e);
                return;
            }
        };
        
        // Update the window state
        if let Err(e) = window.update_state(|state| {
            state.fullscreen = false;
        }) {
            tracing::error!("Failed to update window state: {}", e);
            return;
        }
        
        // Configure the window
        if let Err(e) = window.configure(self) {
            tracing::error!("Failed to configure window: {}", e);
            return;
        }
    }
    
    fn minimize_request(&mut self, surface: ToplevelSurface) {
        // Get the managed window
        let xdg_shell_manager = XdgShellManager::new(Arc::new(SurfaceManager::new()));
        let window = match xdg_shell_manager.get_toplevel(&surface) {
            Ok(Some(window)) => window,
            Ok(None) => {
                tracing::error!("Minimize request for unknown toplevel surface");
                return;
            }
            Err(e) => {
                tracing::error!("Failed to get toplevel window: {}", e);
                return;
            }
        };
        
        // Update the window state
        if let Err(e) = window.update_state(|state| {
            state.minimized = true;
        }) {
            tracing::error!("Failed to update window state: {}", e);
            return;
        }
        
        // No need to configure the window for minimize
        // The client will stop rendering, and we'll stop displaying it
    }
}
