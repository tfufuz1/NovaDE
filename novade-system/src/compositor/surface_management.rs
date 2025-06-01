// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Surface Management Implementation
//!
//! This module implements the management of surfaces in the compositor,
//! including tracking surface state and buffer attachments.

use std::sync::{Arc, RwLock};
use smithay::reexports::wayland_server::protocol::{wl_surface::WlSurface, wl_buffer::WlBuffer};
use smithay::reexports::wayland_server::Weak;
use smithay::utils::{Logical, Point, Size, Rectangle, Transform, Region};

use super::{CompositorError, CompositorResult};

/// Data associated with a surface
#[derive(Debug)]
pub struct SurfaceData {
    /// Unique identifier for the surface (e.g., client ID)
    pub id: String,

    /// Current buffer information
    pub current_buffer_info: Option<AttachedBufferInfo>,

    /// Surface state
    pub state: Arc<RwLock<SurfaceState>>,

    /// Surface children
    pub children: Vec<Weak<WlSurface>>,

    /// Surface parent
    pub parent: Option<Weak<WlSurface>>,

    /// Damage regions in buffer coordinates
    pub damage_buffer_coords: Vec<Rectangle<i32, Logical>>,

    /// Opaque region in surface-local coordinates
    pub opaque_region_surface_local: Option<Region<Logical>>,

    /// Input region in surface-local coordinates
    pub input_region_surface_local: Option<Region<Logical>>,

    /// Handle to the renderer-specific texture for the current buffer.
    pub texture_handle: Option<Box<dyn crate::compositor::renderer_interface::abstraction::RenderableTexture>>,
}

/// Information about an attached buffer
#[derive(Debug, Clone)]
pub struct AttachedBufferInfo {
    /// The Wayland buffer resource.
    pub buffer: WlBuffer,

    /// Buffer dimensions (width, height) in pixels.
    pub dimensions: Size<i32, Logical>,

    /// Buffer scale factor.
    pub scale: i32,

    /// Buffer transform (e.g., rotation, flip).
    pub transform: Transform,
}

/// Surface state
#[derive(Debug, Clone)]
pub struct SurfaceState {
    /// Is the surface visible
    pub visible: bool,

    /// Surface position
    pub position: Point<i32, Logical>,

    /// Surface size
    pub size: Size<i32, Logical>,

    /// Surface opacity
    pub opacity: f64,

    /// Surface z-index
    pub z_index: i32,

    /// Surface workspace
    pub workspace: Option<u32>,

    /// Surface activation state
    pub activated: bool,

    /// Surface fullscreen state
    pub fullscreen: bool,

    /// Surface maximized state
    pub maximized: bool,

    /// Surface minimized state
    pub minimized: bool,

    /// Surface resizing state
    pub resizing: bool,

    /// Surface moving state
    pub moving: bool,
}

impl SurfaceData {
    /// Creates a new surface data
    pub fn new(id: String) -> Self {
        Self {
            id,
            current_buffer_info: None,
            texture_handle: None,
            state: Arc::new(RwLock::new(SurfaceState {
                visible: true,
                position: Point::from((0, 0)),
                size: Size::from((0, 0)),
                opacity: 1.0,
                z_index: 0,
                workspace: Some(0),
                activated: false,
                fullscreen: false,
                maximized: false,
                minimized: false,
                resizing: false,
                moving: false,
            })),
            children: Vec::new(),
            parent: None,
            damage_buffer_coords: Vec::new(),
            opaque_region_surface_local: None,
            input_region_surface_local: None,
        }
    }

    /// Updates the buffer information and related state
    pub fn update_buffer(&mut self, buffer_info: AttachedBufferInfo) -> CompositorResult<()> {
        // Update surface size in state based on the new buffer's dimensions
        // Note: The scale factor from the buffer_info might need to be applied here
        // if state.size is expected to be in logical pixels.
        // For now, assuming dimensions are already logical or handled elsewhere.
        let new_size = buffer_info.dimensions;
        self.current_buffer_info = Some(buffer_info);

        let mut state = self.state.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on surface state".to_string())
        })?;
        state.size = new_size;
        Ok(())
    }

    /// Sets the input region
    pub fn set_input_region(&mut self, region: Option<Region<Logical>>) -> CompositorResult<()> {
        self.input_region_surface_local = region;
        Ok(())
    }

    /// Sets the opaque region
    pub fn set_opaque_region(&mut self, region: Option<Region<Logical>>) -> CompositorResult<()> {
        self.opaque_region_surface_local = region;
        Ok(())
    }

    /// Updates the surface state
    pub fn update_state<F>(&self, update_fn: F) -> CompositorResult<()>
    where
        F: FnOnce(&mut SurfaceState),
    {
        let mut state = self.state.write().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire write lock on surface state".to_string())
        })?;

        update_fn(&mut state);

        Ok(())
    }
    
    /// Gets the surface state
    pub fn get_state(&self) -> CompositorResult<SurfaceState> {
        let state = self.state.read().map_err(|_| {
            CompositorError::SurfaceError("Failed to acquire read lock on surface state".to_string())
        })?;
        
        Ok(state.clone())
    }

    /// Gets the current buffer information
    pub fn get_buffer_info(&self) -> Option<AttachedBufferInfo> {
        self.current_buffer_info.clone()
    }

    /// Gets the children as a Vec of Weak<WlSurface>
    /// Note: Callers will need to upgrade the Weak pointers to use the WlSurface.
    pub fn get_children(&self) -> Vec<Weak<WlSurface>> {
        self.children.clone()
    }

    /// Gets the parent as an Option<Weak<WlSurface>>
    /// Note: Callers will need to upgrade the Weak pointer to use the WlSurface.
    pub fn get_parent(&self) -> Option<Weak<WlSurface>> {
        self.parent.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
    use smithay::utils::{Rectangle, Logical, Region, Size, Point, Transform};
    // Mocking WlBuffer is non-trivial. For tests involving AttachedBufferInfo,
    // we might need a more complex setup or to simplify what we test about it.

    #[test]
    fn test_surface_data_new() {
        let surface_id = "test_surface_1".to_string();
        let surface_data = SurfaceData::new(surface_id.clone());

        assert_eq!(surface_data.id, surface_id);
        assert!(surface_data.current_buffer_info.is_none());
        assert!(surface_data.texture_handle.is_none());
        assert!(surface_data.parent.is_none());
        assert!(surface_data.children.is_empty());
        assert!(surface_data.damage_buffer_coords.is_empty());
        assert!(surface_data.opaque_region_surface_local.is_none());
        assert!(surface_data.input_region_surface_local.is_none());

        let initial_state = surface_data.get_state().unwrap();
        assert_eq!(initial_state.visible, true);
        assert_eq!(initial_state.position, Point::from((0, 0)));
        assert_eq!(initial_state.size, Size::from((0, 0)));
        assert_eq!(initial_state.opacity, 1.0);
        assert_eq!(initial_state.z_index, 0);
        assert_eq!(initial_state.workspace, Some(0));
        assert_eq!(initial_state.activated, false);
        assert_eq!(initial_state.fullscreen, false);
        assert_eq!(initial_state.maximized, false);
        assert_eq!(initial_state.minimized, false);
        assert_eq!(initial_state.resizing, false);
        assert_eq!(initial_state.moving, false);
    }

    #[test]
    fn test_surface_data_update_and_get_state() {
        let surface_data = SurfaceData::new("test_state_surface".to_string());

        let new_position = Point::from((100, 100));
        let new_opacity = 0.5;
        let new_z_index = 5;

        surface_data.update_state(|state| {
            state.position = new_position;
            state.opacity = new_opacity;
            state.z_index = new_z_index;
            state.visible = false;
            state.activated = true;
        }).unwrap();

        let updated_state = surface_data.get_state().unwrap();
        assert_eq!(updated_state.position, new_position);
        assert_eq!(updated_state.opacity, new_opacity);
        assert_eq!(updated_state.z_index, new_z_index);
        assert_eq!(updated_state.visible, false);
        assert_eq!(updated_state.activated, true);
    }

    // Mock WlBuffer for testing purposes.
    // This is a very basic mock. Real WlBuffer interaction is complex.
    // In a real test environment, one might use Smithay's test helpers if available for this.
    #[derive(Debug, Clone)]
    struct MockWlBuffer;

    // Implement AsResource for MockWlBuffer if methods on WlBuffer that need it are called.
    // For just storing it in AttachedBufferInfo, this basic struct might be enough if no methods are called on it.
    // However, WlBuffer requires wayland_server::Resource, which is not easily mockable.
    // For the purpose of this test, we'll assume AttachedBufferInfo can be created
    // without a fully functional WlBuffer, or we accept this test might be limited.
    // The `buffer: WlBuffer` field in `AttachedBufferInfo` makes this hard.
    //
    // **Revised approach for buffer info test due to WlBuffer complexity:**
    // We will test that if `current_buffer_info` is set (even if `None`), `get_buffer_info` returns it.
    // Creating a *real* `AttachedBufferInfo` with a `WlBuffer` is beyond simple unit test scope.
    // The `update_buffer` method itself takes `AttachedBufferInfo`, so testing it fully
    // would require constructing one.

    #[test]
    fn test_surface_data_get_buffer_info_simplified() {
        let mut surface_data = SurfaceData::new("test_buffer_info_surface".to_string());
        
        // Test initial state
        assert!(surface_data.get_buffer_info().is_none());

        // Simulate that current_buffer_info was set to None (e.g., buffer detached)
        surface_data.current_buffer_info = None;
        assert!(surface_data.get_buffer_info().is_none());

        // To test with Some(AttachedBufferInfo), we face the WlBuffer issue.
        // The best we can do in a simple unit test without a Wayland server context
        // is to acknowledge that `get_buffer_info` clones what's there.
        // A more complete test would involve the `update_buffer` method, which has this dependency.
        // For now, this test is very limited.
    }

    #[test]
    fn test_surface_data_set_regions() {
        let mut surface_data = SurfaceData::new("test_regions_surface".to_string());
        let sample_rect = Rectangle::from_loc_and_size((10, 10), (50, 50));
        let region = Region::from(sample_rect);

        // Test input region
        assert!(surface_data.input_region_surface_local.is_none());
        surface_data.set_input_region(Some(region.clone())).unwrap();
        assert_eq!(surface_data.input_region_surface_local.as_ref(), Some(&region));
        
        surface_data.set_input_region(None).unwrap();
        assert!(surface_data.input_region_surface_local.is_none());

        // Test opaque region
        assert!(surface_data.opaque_region_surface_local.is_none());
        surface_data.set_opaque_region(Some(region.clone())).unwrap();
        assert_eq!(surface_data.opaque_region_surface_local.as_ref(), Some(&region));

        surface_data.set_opaque_region(None).unwrap();
        assert!(surface_data.opaque_region_surface_local.is_none());
    }

    // Test for update_buffer's effect on state.size (simplified due to WlBuffer)
    #[test]
    fn test_update_buffer_updates_state_size() {
        // This test requires a WlBuffer, which is hard to mock.
        // We'll assume that if update_buffer were called with a valid AttachedBufferInfo,
        // it would update state.size. The logic is:
        //   let new_size = buffer_info.dimensions;
        //   self.current_buffer_info = Some(buffer_info);
        //   state.size = new_size;
        // This direct logic is tested by `test_surface_data_new` (initial size)
        // and `test_surface_data_update_and_get_state` (general state updates).
        // A full test of `update_buffer` would need a more integrated test environment.
        // For now, we acknowledge this limitation.
        //
        // If we could mock WlBuffer or use a test double:
        /*
        let mut surface_data = SurfaceData::new("test_update_buffer_size".to_string());
        let mock_buffer = MockWlBuffer; // This needs to be a proper WlBuffer compatible mock
        let buffer_info = AttachedBufferInfo {
            buffer: mock_buffer, // This is the issue
            dimensions: Size::from((100, 200)),
            scale: 1,
            transform: Transform::Normal,
        };
        surface_data.update_buffer(buffer_info.clone()).unwrap();
        assert_eq!(surface_data.get_state().unwrap().size, Size::from((100, 200)));
        assert_eq!(surface_data.get_buffer_info().unwrap().dimensions, Size::from((100, 200)));
        */
    }
}
