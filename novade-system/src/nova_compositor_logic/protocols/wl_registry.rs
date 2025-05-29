//! Implements the `wl_registry` Wayland global and interface.
//!
//! `wl_registry` is a special Wayland object that clients use to discover
//! and bind to other available Wayland globals (e.g., `wl_compositor`, `wl_shm`).
//! When a client connects, it first binds to `wl_registry` to get a list of
//! available globals and their capabilities.

use wayland_server::{
    Dispatch, DelegateDispatch, DisplayHandle, GlobalDispatch, Main, Resource,
    protocol::wl_registry::{self, WlRegistry}, // Import WlRegistry interface
    Client, GlobalData, DataInit, implement_dispatch, // Added DataInit and implement_dispatch
};
use crate::state::CompositorState;

/// Data associated with a client's `wl_registry` resource instance.
///
/// `wl_registry` itself doesn't have much client-specific state beyond the
/// list of globals it has advertised to that client. Smithay's `GlobalList`
/// and `Display` handle much of this tracking implicitly.
#[derive(Debug, Default)]
pub struct RegistryState; // This can be the resource data for wl_registry instances.

// GlobalDispatch for WlRegistry on CompositorState.
// This handles the initial binding to the wl_registry global by a client.
// Smithay's GlobalList typically handles the creation and advertisement of wl_registry.
// If we are using GlobalList::new(), it usually registers its own wl_registry global.
// This manual GlobalDispatch might be for a custom setup or if not relying on GlobalList's auto-registry.
// Assuming CompositorState is the intended dispatcher for the global wl_registry resource itself.
impl GlobalDispatch<WlRegistry, GlobalData, CompositorState> for CompositorState {
    /// Called when a client binds to the `wl_registry` global.
    ///
    /// This initializes the `WlRegistry` resource for the client. Smithay's `GlobalList`
    /// will then typically send the list of available globals as events on this resource.
    fn bind(
        _state: &mut CompositorState, // Global compositor state.
        _handle: &DisplayHandle,     // Handle to the display.
        _client: &Client,            // Client that bound to the global.
        resource: Main<WlRegistry>,  // The WlRegistry resource to initialize for the client.
        _global_data: &GlobalData,   // UserData for this global (GlobalData from make_global).
        data_init: &mut DataInit<'_, CompositorState> // Utility to initialize resource data.
    ) {
        println!("Client bound to wl_registry global. Initializing wl_registry resource {:?}.", resource.id());
        // Initialize the client's WlRegistry resource with RegistryState.
        let registry_resource = data_init.init_resource(resource, RegistryState::default());

        // At this point, Smithay's GlobalList (if used and populated in CompositorState)
        // would typically take over and send `global` events for each registered global
        // to this `registry_resource`.
        // The `resource.init_global()` call mentioned in the original code might be part of
        // older Smithay patterns or specific resource initialization if not using DataInit.
        // With DataInit, the resource is initialized, and then events are sent.
        // If GlobalList is managing the registry, it handles advertising other globals.
        // No explicit `global` events need to be sent here manually if GlobalList is active.
        println!("wl_registry resource {:?} initialized. GlobalList will advertise other globals.", registry_resource.id());
    }

    /// Checks if the requested version of `WlRegistry` is supported.
    fn check_versions(&self, _main: Main<WlRegistry>, _versions: &[u32]) -> bool {
        true // WlRegistry is typically version 1 and quite stable.
    }
}

// DelegateDispatch for requests made on a client's WlRegistry resource instance.
// This is dispatched to RegistryState as per implement_dispatch below.
impl DelegateDispatch<WlRegistry, GlobalData, CompositorState> for RegistryState {
    /// Handles requests from a client to a specific `WlRegistry` resource.
    ///
    /// The primary request for `WlRegistry` is `bind`.
    fn request(
        &mut self, // State for this specific WlRegistry resource (&mut RegistryState).
        client: &Client,
        resource: &WlRegistry, // The WlRegistry resource this request is for.
        request: wl_registry::Request,
        _data: &GlobalData, // UserData for WlRegistry dispatch (GlobalData from implement_dispatch).
        dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, CompositorState>,
    ) {
        match request {
            wl_registry::Request::Bind { name, id } => {
                // Client requests to bind to a global advertised by `name`.
                // `id` is the `New<T: Interface>` object for the new resource.
                println!(
                    "wl_registry {:?}: Client requested Bind to global '{}' (new id: {:?})",
                    resource.id(), name, id.id()
                );
                // The actual binding is handled by the GlobalList in CompositorState.
                // It finds the global by `name` and calls its `GlobalDispatch::bind` method.
                // We need to access the GlobalList from CompositorState.
                // This requires `data_init` to provide access to `CompositorState` if `RegistryState`
                // doesn't have it, or `RegistryState` needs a reference to `CompositorState` or `GlobalList`.

                // This is tricky: RegistryState::request is called, but the binding action needs
                // access to the global CompositorState.data.global_list.
                // `data_init` gives `&mut DataInit<'_, CompositorState>`, from which we can get `&mut CompositorState`.
                let main_state: &mut CompositorState = data_init.get_global_data();
                
                match main_state.global_list.bind(dhandle, client, name, id) {
                    Ok(_) => {
                        println!("wl_registry: Successfully bound global '{}'.", name);
                    }
                    Err(_) => {
                        // TODO: This should ideally send a protocol error or log more robustly.
                        // For now, using wl_display::error.
                        // Client might disconnect if it can't bind a required global.
                        // resource.post_error(wl_display::Error::InvalidObject, "global not found");
                        eprintln!("wl_registry: Failed to bind global '{}'. Global might not exist or version mismatch.", name);
                        // It's also possible the client requested a version not supported by the global.
                        // The resource `id` (New<I>) might be dead now.
                        // A client error could be sent on the display object.
                        // client.post_error(resource.id(), wl_display::Error::InvalidObject, "..."); // Needs Display resource
                    }
                }
            }
            _ => {
                // WlRegistry only has the `bind` request.
                eprintln!("wl_registry: Unknown request received for {:?}: {:?}", resource.id(), request);
            }
        }
    }
}

// This Dispatch implementation is also for RegistryState.
// It handles the lifecycle (destruction) of the WlRegistry resource.
impl Dispatch<WlRegistry, GlobalData, CompositorState> for RegistryState {
    /// Handles requests if DelegateDispatch was not used or did not handle them.
    /// For `WlRegistry`, `DelegateDispatch::request` above handles the `bind` request.
    fn request(
        &mut self,
        client: &Client,
        resource: &WlRegistry,
        request: wl_registry::Request,
        data: &GlobalData,
        dhandle: &DisplayHandle,
        data_init: &mut wayland_server::DataInit<'_, CompositorState>,
    ) {
        // Forward to DelegateDispatch for consistent handling.
        self.request(client, resource, request, data, dhandle, data_init);
    }

    /// Called when the `WlRegistry` resource is destroyed.
    fn destroyed(
        &mut self,
        _client_id: wayland_server::backend::ClientId,
        resource_id: wayland_server::backend::ObjectId,
        _data: &GlobalData,
    ) {
        println!("wl_registry: Resource {:?} destroyed.", resource_id);
        // Cleanup for RegistryState if any was needed.
    }
}

// Connects WlRegistry requests to RegistryState's Dispatch/DelegateDispatch implementations.
// - `RegistryState` is the struct handling the dispatch.
// - `wl_registry::WlRegistry` is the interface.
// - `GlobalData` is specified as the UserData for dispatch (can be `()` if not needed).
// - `CompositorState` is the global application data.
implement_dispatch!(RegistryState => [wl_registry::WlRegistry: GlobalData], CompositorState);
