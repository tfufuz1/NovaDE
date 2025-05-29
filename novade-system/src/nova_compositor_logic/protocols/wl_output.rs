//! Implements the `wl_output` Wayland global and interface.
//!
//! `wl_output` represents a display or monitor. Clients use it to get information
//! about display properties like resolution, physical size, scale factor, and mode.
//! This module provides a basic implementation for a static virtual output.

use wayland_server::{
    protocol::wl_output,
    Dispatch, DelegateDispatch, GlobalDispatch, Main, Resource, Client, DisplayHandle, GlobalData, DataInit,
    implement_dispatch
};
use crate::state::CompositorState;

/// Data associated with a client's `wl_output` resource instance.
///
/// For `wl_output`, this struct is minimal as most output state is global
/// or sent as initial events. `wl_output` primarily serves information and
/// has few client-driven requests beyond `release`.
#[derive(Debug, Default)]
pub struct OutputData;

/// Data associated with the `wl_output` global itself in the `GlobalList`.
///
/// This can be used to store properties of the output that are global
/// and not client-specific, if any are needed beyond what's sent in `bind`.
#[derive(Debug, Default)]
pub struct OutputGlobalData; // This was defined in the subtask, so keeping it.

// GlobalDispatch for wl_output on CompositorState.
// Handles new client bindings to the wl_output global.
impl GlobalDispatch<wl_output::WlOutput, GlobalData, CompositorState> for CompositorState {
    /// Called when a client binds to the `wl_output` global.
    ///
    /// This function initializes the `wl_output` resource for the client and sends
    /// the initial sequence of events describing the output's properties.
    fn bind(
        _state: &mut CompositorState, // Global compositor state.
        _handle: &DisplayHandle,     // Handle to the display.
        _client: &Client,            // Client that bound to the global.
        resource: Main<wl_output::WlOutput>, // The wl_output resource to initialize for the client.
        _global_data: &GlobalData,   // UserData for this global, specified in make_global_with_data.
        data_init: &mut DataInit<'_, CompositorState>, // Utility to initialize resource data.
    ) {
        println!("Client bound to wl_output global. Initializing wl_output resource {:?}.", resource.id());

        // Initialize the client's wl_output resource with OutputData.
        let output_resource = data_init.init_resource(resource, OutputData::default());

        // Send static output information.
        // These values represent a virtual 1080p display.
        // TODO: Make these configurable or derive from a backend (e.g., DRM).

        // Geometry event: physical properties of the output.
        output_resource.send_event(wl_output::Event::Geometry {
            x: 0, // Position on the global compositor space (if applicable)
            y: 0,
            physical_width: 345,  // Example physical width in mm (e.g., for a 15.6" 16:9 display)
            physical_height: 194, // Example physical height in mm
            subpixel: wl_output::Subpixel::Unknown, // Subpixel layout
            make: "NovaCorp".to_string(),           // Manufacturer
            model: "VirtualDisplay-1".to_string(),  // Model name
            transform: wl_output::Transform::Normal, // Output transform (e.g., rotation)
        });
        println!("Sent Geometry for wl_output {:?}", output_resource.id());

        // Mode event: resolution, refresh rate, and flags.
        // Flags indicate if this mode is current and/or preferred.
        let flags = wl_output::Mode::Current | wl_output::Mode::Preferred;
        output_resource.send_event(wl_output::Event::Mode {
            flags,
            width: 1920,        // Width in pixels
            height: 1080,       // Height in pixels
            refresh: 60000,     // Refresh rate in mHz (e.g., 60Hz)
        });
        println!("Sent Mode for wl_output {:?}", output_resource.id());

        // Scale event (requires wl_output version 2+).
        // Used for HiDPI scaling. A factor of 1 means no scaling.
        output_resource.send_event(wl_output::Event::Scale { factor: 1 });
        println!("Sent Scale for wl_output {:?}", output_resource.id());
        
        // Done event (requires wl_output version 2+).
        // Signals the end of the initial burst of output properties.
        // For wl_output v3 (which includes scale), 'done' is part of the initial sequence.
        output_resource.send_event(wl_output::Event::Done);
        println!("Sent Done for wl_output {:?}", output_resource.id());
    }

    /// Checks if the requested version of `wl_output` is supported.
    fn check_versions(&self, _main: Main<wl_output::WlOutput>, _versions: &[u32]) -> bool {
        // We send events up to version 3 (scale, done).
        // Client must support at least this version if they want all info.
        // Allowing any version for now, client handles what it understands.
        true 
    }
}

// DelegateDispatch for requests made on a client's wl_output resource instance.
impl DelegateDispatch<wl_output::WlOutput, (), CompositorState> for OutputData {
    /// Handles requests from a client to a specific `wl_output` resource.
    ///
    /// The primary request for `wl_output` is `release`.
    fn request(
        &mut self, // State for this specific wl_output resource (&mut OutputData).
        _client: &Client,
        resource: &wl_output::WlOutput, // The wl_output resource this request is for.
        request: wl_output::Request,
        _data: &(), // UserData for wl_output dispatch (here, unit type `()`).
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, CompositorState>, // Not typically used for wl_output requests.
    ) {
        match request {
            wl_output::Request::Release => {
                // Client requests to release/destroy the wl_output resource.
                // Smithay handles the actual resource destruction when this request is made.
                // Our `destroyed` method below will be called as part of that process.
                println!("wl_output: Client requested Release for {:?}. (Handled by Smithay resource destruction)", resource.id());
            }
            _ => {
                // Should not happen for wl_output, as it has a very limited set of requests.
                eprintln!("wl_output: Unknown request received for {:?}: {:?}", resource.id(), request);
            }
        }
    }

    /// Called when the `wl_output` resource is destroyed.
    fn destroyed(
        &mut self, // State for this specific wl_output resource (&mut OutputData).
        _client_id: wayland_server::backend::ClientId, // ID of the client whose resource was destroyed.
        object_id: wayland_server::backend::ObjectId,  // ID of the wl_output resource.
        _data: &(), // UserData for wl_output dispatch.
    ) {
        println!("wl_output: Resource {:?} destroyed.", object_id);
        // Any cleanup specific to this OutputData instance can go here, though usually not needed for wl_output.
    }
}

// Connects WlOutput requests to the OutputData's Dispatch/DelegateDispatch implementations.
// - `OutputData` is the struct handling the dispatch.
// - `wl_output::WlOutput` is the interface.
// - `()` is the UserData associated with the resource for dispatch purposes.
// - `CompositorState` is the global application data.
implement_dispatch!(OutputData => [wl_output::WlOutput: ()], CompositorState);
```
