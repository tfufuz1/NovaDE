//! Implements the `wl_compositor` Wayland global and related interfaces.
//!
//! `wl_compositor` is a core Wayland global that clients use to create surfaces (`wl_surface`)
//! and regions (`wl_region`). This module provides the dispatch logic for these objects.

use wayland_server::{
    protocol::{wl_compositor, wl_surface},
    // Added DataInit explicitly as it's used in parameters.
    Dispatch, DelegateDispatch, GlobalDispatch, Main, Resource, Client, DisplayHandle, GlobalData, DataInit,
    implement_dispatch
};
use crate::state::CompositorState;

/// Data associated with the `wl_compositor` global instance when a client binds to it.
///
/// This struct is used as the resource data for the `wl_compositor` resource.
/// It doesn't hold much state as `wl_compositor` is primarily a factory object.
#[derive(Debug, Default)]
pub struct CompositorStateData;

// Dispatch logic for the wl_compositor global itself.
// When a client binds to wl_compositor, this GlobalDispatch implementation is invoked.
impl GlobalDispatch<wl_compositor::WlCompositor, GlobalData, CompositorState> for CompositorState {
    /// Called when a client binds to the `wl_compositor` global.
    ///
    /// This function initializes the `wl_compositor` resource for the client.
    /// The `resource` parameter is a `Main<wl_compositor::WlCompositor>` which needs to be
    /// initialized with its resource-specific data (`CompositorStateData`).
    fn bind(
        _state: &mut CompositorState, // Global compositor state, not directly used here but required by trait.
        _handle: &DisplayHandle,     // Handle to the display, not used here.
        _client: &Client,            // Client that bound to the global, not used here.
        resource: Main<wl_compositor::WlCompositor>, // The wl_compositor resource to initialize.
        _global_data: &GlobalData,   // Data associated with the global_list entry (GlobalData here).
        data_init: &mut DataInit<'_, CompositorState>, // Utility to initialize resource data.
    ) {
        // Initialize the resource with CompositorStateData.
        // The actual dispatch for requests on this resource will be handled by
        // CompositorState's DelegateDispatch/Dispatch impl for wl_compositor.
        // `init_resource` is preferred over `resource.init` if using DataInit.
        data_init.init_resource(resource, CompositorStateData::default());
        println!("Client bound to wl_compositor global. Resource initialized.");
        // No events need to be sent upon binding wl_compositor.
    }

    /// Checks if the requested version of `wl_compositor` is supported.
    fn check_versions(&self, _main: Main<wl_compositor::WlCompositor>, _versions: &[u32]) -> bool {
        true // Allow all versions for now (up to the one advertised by GlobalList).
    }
}

// DelegateDispatch for requests made on a wl_compositor resource.
// Since implement_dispatch!(CompositorState => [wl_compositor::WlCompositor: GlobalData]); is used,
// CompositorState itself handles these requests.
impl DelegateDispatch<wl_compositor::WlCompositor, GlobalData, CompositorState> for CompositorState {
    /// Handles requests from a client to a `wl_compositor` resource.
    fn request(
        &mut self, // Global CompositorState
        _client: &Client,
        _compositor_resource: &wl_compositor::WlCompositor, // The specific wl_compositor resource this request is for
        request: wl_compositor::Request,
        _data: &GlobalData, // UserData associated with this WlCompositor resource (GlobalData from implement_dispatch)
        _dhandle: &DisplayHandle, // DisplayHandle, useful for creating other resources if needed
        data_init: &mut DataInit<'_, CompositorState>, // Utility for initializing new Wayland objects
    ) {
        match request {
            wl_compositor::Request::CreateSurface { id } => {
                // Client requests to create a new wl_surface.
                // `id` is the `New<wl_surface::WlSurface>` object to be implemented.
                println!("wl_compositor: Client requested CreateSurface (new id: {:?})", id.id());
                // Use SurfaceData from wl_surface.rs.
                // init_resource correctly associates SurfaceData with the new wl_surface
                // and sets up its dispatch according to implement_dispatch! in wl_surface.rs.
                let _surface_resource = data_init.init_resource(id, crate::protocols::wl_surface::SurfaceData::new());
                println!("wl_surface {:?} created and initialized with SurfaceData.", _surface_resource.id());
            }
            wl_compositor::Request::CreateRegion { id } => {
                // Client requests to create a new wl_region.
                // `id` is the `New<wl_region::WlRegion>` object.
                println!("wl_compositor: Client requested CreateRegion (new id: {:?})", id.id());
                // Use RegionData from wl_surface.rs (as it's closely related to surface geometry).
                let _region_resource = data_init.init_resource(id, crate::protocols::wl_surface::RegionData::default());
                println!("wl_region {:?} created and initialized with RegionData.", _region_resource.id());
            }
            wl_compositor::Request::Destroy => {
                // This request is handled by Smithay internally by calling the destructor,
                // which in turn calls Dispatch::destroyed if that's how the resource was implemented.
                // No specific logic needed here for `wl_compositor::Destroy` itself.
                println!("wl_compositor: Client requested Destroy for {:?} (handled by Dispatch::destroyed)", _compositor_resource.id());
            }
            _ => {
                // Should not happen with a well-behaved client, as wl_compositor has a fixed set of requests.
                eprintln!("wl_compositor: Unknown request received for {:?}: {:?}", _compositor_resource.id(), request);
                // Optionally, send a protocol error to the client.
                // For now, using unreachable as this path indicates a bug or misbehaving client.
                unreachable!("Unknown request for WlCompositor: {:?}", request);
            }
        }
    }
}

// This Dispatch implementation is also for CompositorState because of implement_dispatch!
// It handles the lifecycle (destruction) of the wl_compositor resource.
impl Dispatch<wl_compositor::WlCompositor, GlobalData, CompositorState> for CompositorState {
    /// Handles requests if DelegateDispatch was not used or did not handle them.
    /// For `wl_compositor`, `DelegateDispatch::request` above handles all client requests.
    /// So, this method should ideally not be called for active requests.
    fn request(
        &mut self,
        _client: &Client,
        resource: &wl_compositor::WlCompositor,
        request: wl_compositor::Request,
        _data: &GlobalData,
        _dhandle: &DisplayHandle,
        _data_init: &mut wayland_server::DataInit<'_, Self>,
    ) {
        // This should not be reached if DelegateDispatch handles all requests.
        // Log a warning if it is.
        eprintln!(
            "wl_compositor: Unexpected call to Dispatch::request for {:?} with request {:?}. Should be handled by DelegateDispatch.",
            resource.id(),
            request
        );
        // To be safe, one might call the DelegateDispatch handler or handle as an error.
        // For now, assume DelegateDispatch is primary and this is an error condition.
        // The original code had unreachable! here, which is a valid approach.
        unreachable!("Dispatch::request should not be called for WlCompositor when DelegateDispatch is implemented.");
    }

    /// Called when the `wl_compositor` resource is destroyed.
    ///
    /// This can happen if the client explicitly sends a `destroy` request (handled by `DelegateDispatch`
    /// which then leads to destruction), or if the client disconnects.
    fn destroyed(
        &mut self,
        _client_id: wayland_server::backend::ClientId, // ID of the client whose resource was destroyed
        resource_id: wayland_server::backend::ObjectId, // ID of the wl_compositor resource
        _data: &GlobalData, // UserData associated with the resource
    ) {
        println!("wl_compositor: Resource {:?} destroyed.", resource_id);
        // Any cleanup specific to this wl_compositor instance for this client can go here.
        // For wl_compositor itself, there's usually no specific state to clean up other
        // than what Smithay handles for resource tracking.
    }
}

// This macro connects WlCompositor requests to the CompositorState's Dispatch/DelegateDispatch implementations.
// It specifies that for `wl_compositor::WlCompositor` interface, requests are dispatched to `CompositorState`,
// and the `UserData` associated with the `wl_compositor` resource for dispatch purposes is `GlobalData`.
implement_dispatch!(CompositorState => [wl_compositor::WlCompositor: GlobalData]);

// Note: The WlSurface and WlRegion creation logic relies on `wl_surface.rs`
// providing `SurfaceData` and `RegionData` respectively, along with their
// own `Dispatch` implementations.
// The previous placeholder `SurfaceState` and its `implement_dispatch` are correctly removed
// as `wl_surface.rs` provides the definitive implementation.
