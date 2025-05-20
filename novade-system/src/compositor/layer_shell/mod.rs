// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Layer Shell Implementation
//!
//! This module implements the layer shell protocol for the compositor.

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use smithay::wayland::shell::wlr_layer::{LayerShellHandler, LayerShellState, LayerSurface, LayerSurfaceConfigure};
use smithay::wayland::shell::wlr_layer::{Layer, Anchor, KeyboardInteractivity};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Size, Rectangle, Margins};

use super::{CompositorError, CompositorResult};
use super::core::DesktopState;
use super::surface_management::{SurfaceData, SurfaceRole, SurfaceManager};

/// Layer shell surface
pub struct ManagedLayerSurface {
    /// The layer surface
    pub layer_surface: LayerSurface,
    
    /// Surface data
    pub surface_data: Arc<SurfaceData>,
    
    /// Layer
    pub layer: Layer,
    
    /// Anchor
    pub anchor: Anchor,
    
    /// Exclusive zone
    pub exclusive_zone: i32,
    
    /// Margin
    pub margin: Logical<Margins>,
    
    /// Keyboard interactivity
    pub keyboard_interactivity: KeyboardInteractivity,
    
    /// Namespace
    pub namespace: String,
    
    /// Layer surface state
    pub state: Arc<RwLock<LayerSurfaceState>>,
}

/// Layer surface state
#[derive(Debug, Clone)]
pub struct LayerSurfaceState {
    /// Is the surface visible
    pub visible: bool,
    
    /// Surface position
    pub position: Point<i32, Logical>,
    
    /// Surface size
    pub size: Size<i32, Logical>,
    
    /// Surface opacity
    pub opacity: f64,
    
    /// Surface z-index within its layer
    pub z_index: i32,
    
    /// Is the surface mapped
    pub mapped: bool,
    
    /// Is the surface configured
    pub configured: bool,
    
    /// Surface output
    pub output: Option<String>,
}

impl ManagedLayerSurface {
    /// Creates a new managed layer surface
    pub fn new(
        layer_surface: LayerSurface,
        surface_data: Arc<SurfaceData>,
        layer: Layer,
        anchor: Anchor,
        exclusive_zone: i32,
        margin: Logical<Margins>,
        keyboard_interactivity: KeyboardInteractivity,
        namespace: String,
    ) -> Self {
        Self {
            layer_surface,
            surface_data,
            layer,
            anchor,
            exclusive_zone,
            margin,
            keyboard_interactivity,
            namespace,
            state: Arc::new(RwLock::new(LayerSurfaceState {
                visible: true,
                position: Point::from((0, 0)),
                size: Size::from((0, 0)),
                opacity: 1.0,
                z_index: 0,
                mapped: false,
                configured: false,
                output: None,
            })),
        }
    }
    
    /// Updates the layer surface state
    pub fn update_state<F>(&self, update_fn: F) -> CompositorResult<()>
    where
        F: FnOnce(&mut LayerSurfaceState),
    {
        let mut state = self.state.write().map_err(|_| {
            CompositorError::LayerShellError("Failed to acquire write lock on layer surface state".to_string())
        })?;
        
        update_fn(&mut state);
        
        Ok(())
    }
    
    /// Gets the layer surface state
    pub fn get_state(&self) -> CompositorResult<LayerSurfaceState> {
        let state = self.state.read().map_err(|_| {
            CompositorError::LayerShellError("Failed to acquire read lock on layer surface state".to_string())
        })?;
        
        Ok(state.clone())
    }
    
    /// Configures the layer surface
    pub fn configure(&self, state: &mut DesktopState, size: Option<Size<i32, Logical>>) -> CompositorResult<()> {
        let mut state_lock = self.state.write().map_err(|_| {
            CompositorError::LayerShellError("Failed to acquire write lock on layer surface state".to_string())
        })?;
        
        // Update the state
        if let Some(size) = size {
            state_lock.size = size;
        }
        state_lock.configured = true;
        
        // Send the configure event
        let configure = LayerSurfaceConfigure {
            serial: state.display.lock().unwrap().next_serial(),
            size: state_lock.size,
        };
        
        self.layer_surface.configure(configure);
        
        Ok(())
    }
    
    /// Arranges the layer surface based on its anchor and the output size
    pub fn arrange(&self, output_size: Size<i32, Logical>) -> CompositorResult<()> {
        let mut state = self.state.write().map_err(|_| {
            CompositorError::LayerShellError("Failed to acquire write lock on layer surface state".to_string())
        })?;
        
        // Calculate the position based on the anchor
        let mut position = Point::from((0, 0));
        
        if self.anchor.contains(Anchor::LEFT) {
            position.x = 0;
        } else if self.anchor.contains(Anchor::RIGHT) {
            position.x = output_size.w - state.size.w;
        } else {
            // Center horizontally
            position.x = (output_size.w - state.size.w) / 2;
        }
        
        if self.anchor.contains(Anchor::TOP) {
            position.y = 0;
        } else if self.anchor.contains(Anchor::BOTTOM) {
            position.y = output_size.h - state.size.h;
        } else {
            // Center vertically
            position.y = (output_size.h - state.size.h) / 2;
        }
        
        // Apply margin
        if self.anchor.contains(Anchor::LEFT) {
            position.x += self.margin.left;
        }
        if self.anchor.contains(Anchor::RIGHT) {
            position.x -= self.margin.right;
        }
        if self.anchor.contains(Anchor::TOP) {
            position.y += self.margin.top;
        }
        if self.anchor.contains(Anchor::BOTTOM) {
            position.y -= self.margin.bottom;
        }
        
        // Update the position
        state.position = position;
        
        Ok(())
    }
}

/// Layer shell manager for tracking and managing layer shell surfaces
pub struct LayerShellManager {
    /// Layer surfaces by layer
    layers: Arc<RwLock<HashMap<Layer, Vec<Arc<ManagedLayerSurface>>>>>,
    
    /// Surface manager
    surface_manager: Arc<SurfaceManager>,
}

impl LayerShellManager {
    /// Creates a new layer shell manager
    pub fn new(surface_manager: Arc<SurfaceManager>) -> Self {
        Self {
            layers: Arc::new(RwLock::new(HashMap::new())),
            surface_manager,
        }
    }
    
    /// Registers a layer surface
    pub fn register_layer_surface(
        &self,
        layer_surface: LayerSurface,
        surface_data: Arc<SurfaceData>,
        layer: Layer,
        anchor: Anchor,
        exclusive_zone: i32,
        margin: Logical<Margins>,
        keyboard_interactivity: KeyboardInteractivity,
        namespace: String,
    ) -> CompositorResult<Arc<ManagedLayerSurface>> {
        let mut layers = self.layers.write().map_err(|_| {
            CompositorError::LayerShellError("Failed to acquire write lock on layers".to_string())
        })?;
        
        let managed_surface = Arc::new(ManagedLayerSurface::new(
            layer_surface,
            surface_data,
            layer,
            anchor,
            exclusive_zone,
            margin,
            keyboard_interactivity,
            namespace,
        ));
        
        // Ensure the layer exists in the map
        if !layers.contains_key(&layer) {
            layers.insert(layer, Vec::new());
        }
        
        // Add the surface to the layer
        layers.get_mut(&layer).unwrap().push(managed_surface.clone());
        
        Ok(managed_surface)
    }
    
    /// Gets a layer surface by its Wayland surface
    pub fn get_layer_surface(&self, surface: &WlSurface) -> CompositorResult<Option<Arc<ManagedLayerSurface>>> {
        let layers = self.layers.read().map_err(|_| {
            CompositorError::LayerShellError("Failed to acquire read lock on layers".to_string())
        })?;
        
        for surfaces in layers.values() {
            for managed_surface in surfaces {
                if managed_surface.layer_surface.wl_surface() == surface {
                    return Ok(Some(managed_surface.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Removes a layer surface
    pub fn remove_layer_surface(&self, surface: &WlSurface) -> CompositorResult<()> {
        let mut layers = self.layers.write().map_err(|_| {
            CompositorError::LayerShellError("Failed to acquire write lock on layers".to_string())
        })?;
        
        for surfaces in layers.values_mut() {
            surfaces.retain(|managed_surface| managed_surface.layer_surface.wl_surface() != surface);
        }
        
        Ok(())
    }
    
    /// Gets all layer surfaces in a specific layer
    pub fn get_layer_surfaces(&self, layer: Layer) -> CompositorResult<Vec<Arc<ManagedLayerSurface>>> {
        let layers = self.layers.read().map_err(|_| {
            CompositorError::LayerShellError("Failed to acquire read lock on layers".to_string())
        })?;
        
        Ok(layers.get(&layer).cloned().unwrap_or_default())
    }
    
    /// Gets all layer surfaces
    pub fn get_all_layer_surfaces(&self) -> CompositorResult<Vec<Arc<ManagedLayerSurface>>> {
        let layers = self.layers.read().map_err(|_| {
            CompositorError::LayerShellError("Failed to acquire read lock on layers".to_string())
        })?;
        
        let mut result = Vec::new();
        for surfaces in layers.values() {
            result.extend(surfaces.iter().cloned());
        }
        
        Ok(result)
    }
    
    /// Arranges all layer surfaces for an output
    pub fn arrange_layers(&self, output_size: Size<i32, Logical>) -> CompositorResult<()> {
        // Get all layer surfaces
        let all_surfaces = self.get_all_layer_surfaces()?;
        
        // Arrange each surface
        for surface in all_surfaces {
            surface.arrange(output_size)?;
        }
        
        Ok(())
    }
}

/// Implementation of the layer shell handler for the desktop state
impl LayerShellHandler for DesktopState {
    fn layer_shell_state(&mut self) -> &mut LayerShellState {
        &mut self.layer_shell_state
    }
    
    fn new_layer_surface(
        &mut self,
        surface: LayerSurface,
        output: Option<smithay::wayland::output::Output>,
        layer: Layer,
        namespace: String,
    ) {
        // Get the underlying wl_surface
        let wl_surface = surface.wl_surface().clone();
        
        // Create surface data for the layer surface
        let attributes = smithay::wayland::compositor::get_surface_attributes(&wl_surface).unwrap();
        
        // Register the surface with the surface manager
        let surface_manager = SurfaceManager::new();
        let surface_data = match surface_manager.register_surface(wl_surface.clone(), attributes, SurfaceRole::LayerShell) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to register layer surface: {}", e);
                return;
            }
        };
        
        // Get the layer surface properties
        let anchor = surface.anchor();
        let exclusive_zone = surface.exclusive_zone();
        let margin = surface.margin();
        let keyboard_interactivity = surface.keyboard_interactivity();
        
        // Create a managed layer surface
        let layer_shell_manager = LayerShellManager::new(Arc::new(surface_manager));
        let managed_surface = match layer_shell_manager.register_layer_surface(
            surface.clone(),
            surface_data,
            layer,
            anchor,
            exclusive_zone,
            margin,
            keyboard_interactivity,
            namespace,
        ) {
            Ok(managed_surface) => managed_surface,
            Err(e) => {
                tracing::error!("Failed to register layer surface: {}", e);
                return;
            }
        };
        
        // Update the output
        if let Some(output) = output {
            if let Err(e) = managed_surface.update_state(|state| {
                state.output = Some(output.name());
            }) {
                tracing::error!("Failed to update layer surface state: {}", e);
            }
        }
        
        // Configure the layer surface with a default size
        let output_config = match self.get_output_configuration() {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("Failed to get output configuration: {}", e);
                return;
            }
        };
        
        let size = match layer {
            Layer::Background | Layer::Bottom | Layer::Top | Layer::Overlay => {
                // Full screen for these layers by default
                Some(output_config.size)
            }
            _ => {
                // Default size for other layers
                Some(Size::from((256, 256)))
            }
        };
        
        if let Err(e) = managed_surface.configure(self, size) {
            tracing::error!("Failed to configure layer surface: {}", e);
        }
        
        // Arrange the layer surface
        if let Err(e) = managed_surface.arrange(output_config.size) {
            tracing::error!("Failed to arrange layer surface: {}", e);
        }
    }
    
    fn layer_surface_commit(&mut self, surface: LayerSurface) {
        // Get the managed layer surface
        let layer_shell_manager = LayerShellManager::new(Arc::new(SurfaceManager::new()));
        let managed_surface = match layer_shell_manager.get_layer_surface(surface.wl_surface()) {
            Ok(Some(managed_surface)) => managed_surface,
            Ok(None) => {
                tracing::error!("Commit for unknown layer surface");
                return;
            }
            Err(e) => {
                tracing::error!("Failed to get layer surface: {}", e);
                return;
            }
        };
        
        // Update the surface state
        if let Err(e) = managed_surface.update_state(|state| {
            state.mapped = true;
        }) {
            tracing::error!("Failed to update layer surface state: {}", e);
        }
        
        // Re-arrange the layer surface if needed
        let output_config = match self.get_output_configuration() {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("Failed to get output configuration: {}", e);
                return;
            }
        };
        
        if let Err(e) = managed_surface.arrange(output_config.size) {
            tracing::error!("Failed to arrange layer surface: {}", e);
        }
    }
    
    fn configure_request(
        &mut self,
        surface: LayerSurface,
        configure: LayerSurfaceConfigure,
        initialized: bool,
    ) {
        // Get the managed layer surface
        let layer_shell_manager = LayerShellManager::new(Arc::new(SurfaceManager::new()));
        let managed_surface = match layer_shell_manager.get_layer_surface(surface.wl_surface()) {
            Ok(Some(managed_surface)) => managed_surface,
            Ok(None) => {
                tracing::error!("Configure request for unknown layer surface");
                return;
            }
            Err(e) => {
                tracing::error!("Failed to get layer surface: {}", e);
                return;
            }
        };
        
        // Update the surface state
        if let Err(e) = managed_surface.update_state(|state| {
            state.size = configure.size;
            state.configured = true;
        }) {
            tracing::error!("Failed to update layer surface state: {}", e);
        }
        
        // Re-arrange the layer surface
        let output_config = match self.get_output_configuration() {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("Failed to get output configuration: {}", e);
                return;
            }
        };
        
        if let Err(e) = managed_surface.arrange(output_config.size) {
            tracing::error!("Failed to arrange layer surface: {}", e);
        }
    }
}
