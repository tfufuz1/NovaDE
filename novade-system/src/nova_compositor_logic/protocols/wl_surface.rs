//! Implements the `wl_surface` Wayland interface and related objects like `wl_region` and `wl_callback`.
//!
//! `wl_surface` is the primary object clients use to display content. It represents a rectangular
//! area on the screen. Clients attach buffers (`wl_buffer`) to surfaces, damage them, and commit
//! these changes to make them visible.
//!
//! This module provides:
//! - `SurfaceData`: State for `wl_surface` resources.
//! - `RegionData`: State for `wl_region` resources (used for defining areas like damage or input regions).
//! - `CallbackData` (inlined): State for `wl_callback` resources (used for frame synchronization).

use wayland_server::{
    protocol::{wl_surface, wl_region, wl_callback},
    // Added DataInit as it's used in parameters. Main is for global binding, Resource for existing.
    Dispatch, DelegateDispatch, Resource, Main, Client, UserData, DestructionNotify, DisplayHandle, DataInit,
    implement_dispatch
};
use crate::state::CompositorState;
// use std::sync::{Arc, Mutex}; // Not currently used, can be removed if not planned.

/// Data associated with a `wl_surface` resource.
///
/// This struct holds state specific to a client's `wl_surface`. This includes:
/// - `user_data`: Smithay's general-purpose UserData store.
/// - TODO: Pending state for atomic commits (buffer, damage, input/opaque regions, etc.).
/// - TODO: Current state that has been committed.
#[derive(Default, Debug)]
pub struct SurfaceData {
    /// General-purpose UserData provided by Smithay. Can be used to store
    /// additional data associated with this surface by other parts of the compositor.
    pub user_data: UserData,
    // TODO: Add fields for pending state, e.g.:
    // pending_buffer: Option<wl_buffer::WlBuffer>,
    // pending_damage: Vec<Rectangle>, // Rectangle would be a custom geometry struct
    // current_buffer: Option<wl_buffer::WlBuffer>,
    // etc.
}

impl SurfaceData {
    /// Creates new data for a `wl_surface`.
    pub fn new() -> Self {
        Self {
            user_data: UserData::new(),
            // Initialize other fields here if added
        }
    }
}

// Dispatch implementation for wl_surface, handled by SurfaceData.
// This means requests on a wl_surface resource are dispatched to methods on its SurfaceData instance.
impl Dispatch<wl_surface::WlSurface, (), CompositorState> for SurfaceData {
    /// Handles requests from a client to a `wl_surface` resource.
    fn request(
        &mut self, // State for this specific wl_surface (&mut SurfaceData).
        _client: &Client,
        surface: &wl_surface::WlSurface, // The wl_surface resource this request is for.
        request: wl_surface::Request,
        _data: &(), // UserData for wl_surface dispatch (here, unit type `()`).
        _dhandle: &DisplayHandle, // DisplayHandle, needed for creating some sub-resources.
        data_init: &mut DataInit<'_, CompositorState>, // For creating wl_callback.
    ) {
        // TODO: Most of these handlers need to manipulate pending state, which is then
        //       applied atomically during `wl_surface::Commit`.
        match request {
            wl_surface::Request::Attach { buffer, x, y } => {
                // Client attaches a wl_buffer to the surface.
                // `buffer` can be None to detach the current buffer.
                // `x` and `y` are offsets for the buffer relative to the surface origin (rarely used).
                println!(
                    "wl_surface {:?}: Attach called. Buffer: {:?}, x: {}, y: {}",
                    surface.id(), buffer.as_ref().map(|b| b.id()), x, y
                );
                // TODO: Store `buffer`, `x`, `y` in `self.pending_buffer` etc.
                //       If buffer is Some, might need to add a destruction listener to it.
            }
            wl_surface::Request::Damage { x, y, width, height } => {
                // Client reports damage (area that needs redraw) to the surface.
                // Coordinates are surface-local.
                println!(
                    "wl_surface {:?}: Damage called. x: {}, y: {}, width: {}, height: {}",
                    surface.id(), x, y, width, height
                );
                // TODO: Add this rectangle to `self.pending_damage`.
            }
            wl_surface::Request::Frame { callback } => {
                // Client requests a wl_callback to be invoked when it's optimal to draw the next frame.
                // `callback` is the `New<wl_callback::WlCallback>` object to implement.
                println!("wl_surface {:?}: Frame callback requested (new id: {:?})", surface.id(), callback.id());
                // TODO: Store this callback. It should be invoked (send `done` event) after the current
                //       surface state is committed and presented, or when the compositor decides.
                //       For now, just create the resource. A real implementation might queue it.
                #[derive(Debug, Default)] struct CallbackData; // Placeholder for wl_callback state
                impl Dispatch<wl_callback::WlCallback, (), CompositorState> for CallbackData {
                    fn request( &mut self, _c: &Client, _r: &wl_callback::WlCallback, _req: wl_callback::Request, _d: &(), _dh: &DisplayHandle, _di: &mut DataInit<'_, CompositorState>) { /* wl_callback has no requests */ }
                }
                // DelegateDispatch for wl_callback is not strictly needed if Dispatch handles its (non-existent) requests.
                // However, Smithay examples sometimes include it for completeness or future-proofing.
                impl DelegateDispatch<wl_callback::WlCallback, (), CompositorState> for CallbackData {
                     fn request( &mut self, _c: &Client, _r: &wl_callback::WlCallback, _req: wl_callback::Request, _d: &(), _dh: &DisplayHandle, _di: &mut DataInit<'_, CompositorState>) { /* wl_callback has no requests */ }
                }
                implement_dispatch!(CallbackData => [wl_callback::WlCallback: ()], CompositorState);
                data_init.init_resource(callback, CallbackData::default());
            }
            wl_surface::Request::SetOpaqueRegion { region } => {
                // Client sets the opaque region of the surface.
                // `region` can be None (fully transparent) or a wl_region object.
                // This is a hint to the compositor for optimization (e.g., skip drawing occluded content).
                println!("wl_surface {:?}: SetOpaqueRegion called. Region: {:?}", surface.id(), region.as_ref().map(|r| r.id()));
                // TODO: Store this region in `self.pending_opaque_region`.
            }
            wl_surface::Request::SetInputRegion { region } => {
                // Client sets the input region (area that accepts input events).
                // `region` can be None (entire surface) or a wl_region object.
                println!("wl_surface {:?}: SetInputRegion called. Region: {:?}", surface.id(), region.as_ref().map(|r| r.id()));
                // TODO: Store this region in `self.pending_input_region`.
            }
            wl_surface::Request::Commit => {
                // Client commits pending state (buffer, damage, regions) to be applied atomically.
                println!("wl_surface {:?}: Commit called.", surface.id());
                // TODO: Atomically apply all pending state from `self.pending_*` to `self.current_*`.
                //       - This is where the surface content effectively updates.
                //       - If a frame callback was requested, it should be processed (e.g., send wl_callback.done
                //         event now or after the content is actually presented/rendered by the compositor).
                //       - Trigger re-rendering or scene graph update.
            }
            wl_surface::Request::SetBufferTransform { transform } => {
                // Client sets a transform for the attached buffer (e.g., rotation).
                // Requires wl_surface version 2+.
                println!("wl_surface {:?}: SetBufferTransform called. Transform: {:?}", surface.id(), transform);
                // TODO: Store `transform` in pending state.
            }
            wl_surface::Request::SetBufferScale { scale } => {
                // Client sets a scale factor for the attached buffer.
                // Requires wl_surface version 3+.
                println!("wl_surface {:?}: SetBufferScale called. Scale: {}", surface.id(), scale);
                // TODO: Store `scale` in pending state.
            }
            wl_surface::Request::DamageBuffer { x, y, width, height } => {
                // Client reports damage to a specific region of the attached buffer.
                // Coordinates are buffer-local. Requires wl_surface version 4+.
                println!(
                    "wl_surface {:?}: DamageBuffer called. x: {}, y: {}, width: {}, height: {}",
                    surface.id(), x, y, width, height
                );
                // TODO: Add this buffer damage to pending state. This is different from surface damage.
            }
            wl_surface::Request::Destroy => {
                // Client explicitly requests to destroy the wl_surface.
                // This is handled by Smithay, leading to `Dispatch::destroyed` being called.
                println!("wl_surface {:?}: Explicit Destroy request received.", surface.id());
            }
            _ => {
                // Should not happen with a well-behaved client.
                eprintln!("wl_surface {:?}: Unknown request: {:?}", surface.id(), request);
            }
        }
    }

    /// Called when the `wl_surface` resource is destroyed.
    fn destroyed(
        &mut self, // State for this specific wl_surface (&mut SurfaceData).
        _client_id: wayland_server::backend::ClientId, // ID of the client whose resource was destroyed.
        object_id: wayland_server::backend::ObjectId,  // ID of the wl_surface resource.
        _data: &(), // UserData for wl_surface dispatch.
    ) {
        println!("wl_surface {:?}: Resource destroyed (Dispatch::destroyed called).", object_id);
        // TODO: Any cleanup specific to this surface can go here.
        //       For example, if this surface was stored in some global list or scene graph, remove it.
        //       Release any strong references to buffers, regions, etc.
    }
}

// Connects WlSurface requests to SurfaceData's Dispatch implementation.
// - `SurfaceData` is the struct handling the dispatch.
// - `wl_surface::WlSurface` is the interface.
// - `()` is the UserData associated with the resource for dispatch purposes.
// - `CompositorState` is the global application data context.
implement_dispatch!(SurfaceData => [wl_surface::WlSurface: ()], CompositorState);

// DestructionNotify provides an alternative or additional way to handle resource destruction,
// potentially with access to the global CompositorState if needed for cleanup.
// Dispatch::destroyed is often sufficient for self-contained resource data.
impl DestructionNotify<CompositorState> for SurfaceData {
    /// Called when the object associated with this `SurfaceData` is destroyed.
    /// This provides access to the global `CompositorState` during cleanup if needed.
    fn object_destroyed(
        &mut self,
        _client_id: wayland_server::backend::ClientId,
        object_id: wayland_server::backend::ObjectId,
        _gdata: &mut CompositorState, // Global state, can be used for unregistering from global lists etc.
    ) {
        println!("wl_surface {:?}: Destroyed (DestructionNotify called).", object_id);
        // This is another place for cleanup, potentially interacting with global state.
    }
}


// --- wl_region ---

/// Data associated with a `wl_region` resource.
///
/// `wl_region` objects are used by clients to define areas on a surface,
/// typically for damage, input handling, or opaque regions.
/// A `wl_region` is a set of rectangles.
#[derive(Default, Debug)]
pub struct RegionData {
    // TODO: Store the actual region data (e.g., a list of rectangles or a Pixman region).
    // For now, it's a placeholder.
    // region: pixman::Region32, // Example using pixman for region operations.
}

// Dispatch implementation for wl_region, handled by RegionData.
impl Dispatch<wl_region::WlRegion, (), CompositorState> for RegionData {
    /// Handles requests from a client to a `wl_region` resource.
     fn request(
        &mut self, // State for this specific wl_region (&mut RegionData).
        _client: &Client,
        resource: &wl_region::WlRegion, // The wl_region resource.
        request: wl_region::Request,
        _data: &(), // UserData for wl_region dispatch.
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, CompositorState>, // Not typically used for wl_region requests.
    ) {
        match request {
            wl_region::Request::Add { x, y, width, height } => {
                // Client adds a rectangle to the region.
                println!("wl_region {:?}: Add rectangle ({}, {}, {}, {})", resource.id(), x, y, width, height);
                // TODO: Add the rectangle to `self.region`.
            }
            wl_region::Request::Subtract { x, y, width, height } => {
                // Client subtracts a rectangle from the region.
                 println!("wl_region {:?}: Subtract rectangle ({}, {}, {}, {})", resource.id(), x, y, width, height);
                // TODO: Subtract the rectangle from `self.region`.
            }
            wl_region::Request::Destroy => {
                // Client explicitly requests to destroy the wl_region.
                // Handled by Smithay, leading to `Dispatch::destroyed`.
                println!("wl_region {:?}: Explicit Destroy request received.", resource.id());
            }
            _ => {
                // Should not happen with a well-behaved client.
                eprintln!("wl_region {:?}: Unknown request: {:?}", resource.id(), request);
            }
        }
    }

    /// Called when the `wl_region` resource is destroyed.
    fn destroyed(
        &mut self, // State for this specific wl_region (&mut RegionData).
        _client_id: wayland_server::backend::ClientId,
        object_id: wayland_server::backend::ObjectId, // ID of the wl_region resource.
        _data: &(), // UserData for wl_region dispatch.
    ) {
        println!("wl_region {:?}: Resource destroyed.", object_id);
        // Cleanup for RegionData if any (e.g., free Pixman region).
    }
}
// Connects WlRegion requests to RegionData's Dispatch implementation.
implement_dispatch!(RegionData => [wl_region::WlRegion: ()], CompositorState);

// Note: `wl_compositor` needs to be able to create `WlRegion` objects.
// This is handled in `wl_compositor.rs` where `CreateRegion` initializes a new resource
// with `RegionData::default()`.

```
