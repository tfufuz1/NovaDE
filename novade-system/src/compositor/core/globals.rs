//! Manages the registration and dispatching of Wayland global objects.
//!
//! This module includes `GlobalDispatch` implementations for various core Wayland
//! protocols. These implementations define how clients bind to global objects
//! advertised by the compositor. Many core globals (like `wl_compositor`, `wl_shm`,
//! `xdg_wm_base`) are registered implicitly when their respective Smithay state
//! objects (e.g., `CompositorState`, `ShmState`, `XdgShellState`) are initialized
//! in `DesktopState::new()`.
//!
//! This file might also contain functions for explicit global creation if needed,
//! though that pattern is less common with newer Smithay versions where states manage
//! their own globals.

use smithay::{
    reexports::wayland_server::{
        Client, DataInit, DisplayHandle, GlobalDispatch, New, Resource,
        protocol::wl_output::WlOutput as WlOutputResource, // Specific resource type
    },
    wayland::compositor::{WlCompositor, WlSubcompositor},
    wayland::shell::xdg::{XdgWmBase, XdgWmBaseClientData},
    wayland::shm::WlShm,
    output::Output as SmithayOutput, // Server-side Output representation
    output::Mode as SmithayMode,     // Server-side Mode representation
    output::PhysicalProperties as SmithayPhysicalProperties, // Server-side PhysicalProperties
    utils::Transform as SmithayTransform, // Server-side Transform
};

use super::state::{/* ClientCompositorData (if used), */ DesktopState}; // ClientCompositorData example removed as not directly used
use crate::compositor::foreign_toplevel::ForeignToplevelManagerClientData;
use smithay::reexports::wayland_protocols::unstable::foreign_toplevel_management::v1::server::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1;


// --- WlCompositor Global Dispatch ---
// The WlCompositor global is typically registered by CompositorState::new().
// This GlobalDispatch handles client binding requests to it.
impl GlobalDispatch<WlCompositor, ()> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client, // Client isn't directly used in this simple bind
        resource: New<WlCompositor>,
        _global_data: &(), // No user data associated with the WlCompositor global itself
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!(client_id = ?_client.id(), resource_id = ?resource.id(), "Client binding to WlCompositor");
        // Smithay's CompositorState requires CompositorClientState to be associated with the client.
        // This is typically done when the client first connects or when DesktopState is initialized for the client.
        // For WlCompositor binding, we just initialize the resource.
        data_init.init(resource, ());
    }
}

// --- WlSubcompositor Global Dispatch ---
// The WlSubcompositor global is typically registered by SubcompositorState::new().
impl GlobalDispatch<WlSubcompositor, ()> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<WlSubcompositor>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!(client_id = ?_client.id(), resource_id = ?resource.id(), "Client binding to WlSubcompositor");
        data_init.init(resource, ());
    }
}

// --- WlShm Global Dispatch ---
// The WlShm global is typically registered by ShmState::new().
impl GlobalDispatch<WlShm, ()> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        client: &Client,
        resource: New<WlShm>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!(client_id = ?client.id(), resource_id = ?resource.id(), "Client binding to WlShm");
        // ShmState handles sending supported formats upon binding.
        // We initialize the client's WlShm resource.
        data_init.init(resource, ());
    }
}

// --- XDG WM Base Global Dispatch ---
// The XdgWmBase global is typically registered by XdgShellState::new().
impl GlobalDispatch<XdgWmBase, XdgWmBaseClientData> for DesktopState {
    fn bind(
        state: &mut Self, // DesktopState instance
        _handle: &DisplayHandle,
        client: &Client,
        resource: New<XdgWmBase>,
        _global_data: &XdgWmBaseClientData,
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!(client_id = ?client.id(), resource_id = ?resource.id(), "Client binding to XdgWmBase");
        // XdgShellState provides client-specific data for XdgWmBase.
        let client_xdg_shell_data = state.xdg_shell_state.new_client_data(client);
        data_init.init(resource, client_xdg_shell_data);
    }
}

// --- Foreign Toplevel Manager Global Dispatch ---
// This global is explicitly created in DesktopState::new().
impl GlobalDispatch<ZwlrForeignToplevelManagerV1, ()> for DesktopState {
    fn bind(
        state: &mut Self,
        _handle: &DisplayHandle,
        client: &Client,
        resource: New<ZwlrForeignToplevelManagerV1>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        let client_id = client.id();
        tracing::info!(client_id = ?client_id, resource_id = ?resource.id(), "Client binding to ZwlrForeignToplevelManagerV1");

        // Initialize the manager resource for the client.
        let manager_resource_main = data_init.init(resource, ForeignToplevelManagerClientData::default());
        
        // Add this manager to our state to inform it about existing and new toplevels.
        state.foreign_toplevel_manager_state
            .lock()
            .unwrap()
            .add_manager(manager_resource_main, state);
    }
}


// --- Output Global Management & Creation ---

/// Creates and registers an initial "virtual" output for the compositor.
///
/// This function is typically called once during compositor initialization to ensure
/// that clients have at least one `wl_output` to interact with, especially in
/// headless or Winit-based development setups. The created `smithay::output::Output`
/// is mapped into the `DesktopState.space`.
pub fn create_initial_output(state: &mut DesktopState) {
    tracing::info!("Creating initial virtual output 'NOVADE-VIRT-1'");

    let mode = SmithayMode {
        size: (1920, 1080).into(),
        refresh: 60_000,
    };

    let physical_properties = SmithayPhysicalProperties {
        size: (0, 0).into(),
        subpixel: smithay::output::Subpixel::Unknown,
        make: "NovaDE".to_string(),
        model: "Virtual Output".to_string(),
    };

    // Output::new registers the wl_output global with the display.
    let new_output = SmithayOutput::new(
        "NOVADE-VIRT-1".to_string(), // Output name
        physical_properties,
        Some(tracing::Span::current().into()),
    );

    new_output.add_mode(mode);
    new_output.set_preferred_mode(mode);
    if !new_output.set_current_mode(mode) { // This also sets the size property of the output
        tracing::error!("Failed to set current mode for initial output. This should not happen.");
    }
    new_output.set_transform(SmithayTransform::Normal);
    new_output.set_scale_factor(1.0); // Default scale factor

    let output_name = new_output.name().to_string();

    // Map the output into the compositor's space.
    // This makes the output known to rendering and window management.
    // OutputManagerState will discover outputs from the space (or other means)
    // to provide zxdg_output_v1 interfaces for them.
    state.space.lock().unwrap().map_output(&new_output, (0,0).into());

    tracing::info!("Successfully created and mapped output '{}' to space with mode {:?} and refresh {} mHz.", output_name, mode.size, mode.refresh);
    // Note: The wl_output global is created by `Output::new` itself.
    // OutputManagerState, initialized with `new_with_xdg_output`, will handle zxdg_output_v1 for this.
}
