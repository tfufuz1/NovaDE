use smithay::{
    reexports::wayland_server::{
        Client, DataInit, DisplayHandle, GlobalDispatch, New, Resource,
    },
    wayland::compositor::{WlCompositor, WlSubcompositor},
};

// Adjust path as necessary, assuming state.rs and globals.rs are in the same core module.
use super::state::{ClientCompositorData, DesktopState}; 

impl GlobalDispatch<WlCompositor, ()> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        client: &Client,
        resource: New<WlCompositor>, // New<T> is used for globals that init with specific version
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!(client_id = ?client.id(), resource_id = ?resource.id(), "Client binding to WlCompositor");
        
        // Ensure ClientCompositorData is initialized for the client.
        // This is crucial because CompositorHandler::client_compositor_state will expect it.
        if client.get_data::<ClientCompositorData>().is_none() {
            client.insert_user_data(|| ClientCompositorData::new());
            tracing::debug!(client_id = ?client.id(), "Initialized ClientCompositorData for new client binding to WlCompositor.");
        }
        
        // The actual resource binding is typically handled by Smithay when init is called.
        // For WlCompositor, it's usually version-agnostic from the handler's perspective,
        // but Smithay might handle version negotiation internally.
        data_init.init(resource, ()); // The second argument is the user_data for the WlCompositor global itself, typically ()
    }
}

impl GlobalDispatch<WlSubcompositor, ()> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        client: &Client,
        resource: New<WlSubcompositor>, // New<T> for versioned globals
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!(client_id = ?client.id(), resource_id = ?resource.id(), "Client binding to WlSubcompositor");
        // WlSubcompositor also doesn't typically require specific user_data for the global itself.
        data_init.init(resource, ());
    }
}

// --- Function to ensure/log global creation ---
use crate::compositor::core::errors::CompositorCoreError;
// If other globals like XDG Shell were being created here, their modules would be imported.
// e.g., use crate::compositor::xdg_shell;

pub fn create_all_wayland_globals(
    _desktop_state: &mut DesktopState, // Marked as unused for now, but could be used for complex setups
    _display_handle: &DisplayHandle,   // Marked as unused for now
) -> Result<(), CompositorCoreError> {
    // In Smithay 0.10.0, WlCompositor (and WlSubcompositor) and WlShm globals
    // are typically registered with the Wayland display when CompositorState::new::<Self>()
    // and ShmState::new::<Self>() are called, respectively. This usually happens
    // during the initialization of DesktopState.

    // This function can be used to:
    // 1. Log that these globals are expected to be registered.
    // 2. Explicitly create other globals (e.g., XDG Shell, Output Manager - though OutputManager is also state-driven).
    // 3. Store Global<T> handles if needed for later removal or management, though this is less common.

    // Example: If we were to get and store global handles (not strictly needed by the plan for core globals)
    // let comp_glob = desktop_state.compositor_state.global(); // If CompositorState provided direct global access
    // let shm_glob = desktop_state.shm_state.global();       // If ShmState provided direct global access
    // tracing::debug!("Compositor global ID: {:?}", comp_glob);
    // tracing::debug!("SHM global ID: {:?}", shm_glob);

    // For XDG Shell, a typical pattern would be:
    // let xdg_shell_global = XdgShellState::new::<DesktopState>(display_handle);
    // desktop_state.xdg_shell_state = Some(xdg_shell_global); // If DesktopState stores XdgShellState
    // Or, if XdgShellState is part of DesktopState already:
    // desktop_state.xdg_shell_state.create_global(display_handle); // Conceptual

    tracing::info!(
        "Core Wayland globals (wl_compositor, wl_subcompositor, wl_shm, xdg_wm_base) \
        are implicitly registered by their respective state initializations \
        (CompositorState, ShmState, XdgShellState) within DesktopState::new()."
    );
    tracing::info!(
        "Output-related globals (wl_output, zxdg_output_manager_v1, zxdg_output_v1) \
        are handled by OutputManagerState (for manager) and explicit Output::create_global calls (for individual outputs)."
    );
    tracing::info!(
        "Input-related globals (wl_seat) are implicitly registered by SeatState::new_wl_seat \
        during DesktopState initialization."
    );

    // Explicitly create the DMABUF global using the DmabufState from DesktopState.
    // This global will advertise no formats initially, as per the plan.
    // The DmabufGlobalData for create_global_with_default_feedback is typically an empty tuple ().
    // Smithay 0.10.0 DmabufState::create_global_with_default_feedback signature:
    // pub fn create_global_with_default_feedback<D>(
    //     &self,
    //     display: &DisplayHandle,
    //     formats: &[Format],
    //     logger: Option<L>,
    // ) -> Global<ZwpLinuxDmabufV1>
    // where D: GlobalDispatch<ZwpLinuxDmabufV1, DmabufGlobalData> + DmabufHandler + 'static, L: Into<Option<slog::Logger>>
    // We need to ensure DesktopState implements GlobalDispatch<ZwpLinuxDmabufV1, DmabufGlobalData> if not using a generic bind.
    // However, the global is usually auto-bound by the DmabufHandler if the delegate is set up.
    // For now, let's just create it. The delegate_dmabuf macro should handle the dispatch.
    
    // For Smithay 0.10.x, DmabufState itself doesn't directly store the global.
    // The global creation returns a Global<ZwpLinuxDmabufV1>, which we can store or drop.
    // The global is registered with the display upon creation.

    // Define preferred DMABUF formats to advertise.
    // ARGB8888 with LINEAR modifier is a widely supported baseline.
    use smithay::backend::allocator::Format;
    use smithay::reexports::drm_fourcc::{DrmFourcc, DrmFormatModifier};

    let preferred_dmabuf_formats = [
        Format { code: DrmFourcc::Argb8888, modifier: DrmFormatModifier::Linear },
        // Optionally, add Xrgb8888 if it's also desired as a preferred default.
        // Format { code: DrmFourcc::Xrgb8888, modifier: DrmFormatModifier::Linear },
    ];
    
    let _dmabuf_global = _desktop_state.dmabuf_state.create_global_with_default_feedback::<DesktopState>(
         _display_handle, 
         &preferred_dmabuf_formats, // Pass the defined preferred formats
         Some(tracing::Span::current()) 
    );

    tracing::info!(
        "DMABUF global (zwp_linux_dmabuf_v1) registered, advertising preferred formats: {:?}",
        preferred_dmabuf_formats
    );
    
    // TODO: Add creation of other essential globals here as they are implemented, e.g.:
    // - Data Device Manager (wl_data_device_manager)
    // - wlr-layer-shell-unstable-v1:
    //   If using Smithay 0.6+, this would be:
    //   _desktop_state.layer_shell_state.create_global::<DesktopState>(_display_handle);
    //   tracing::info!("Layer shell global (wlr_layer_shell_unstable_v1) registered.");
    //   For Smithay 0.3.0, this requires manual global creation:
    //   _display_handle.create_global::<DesktopState, YourLayerShellInterface, _>(
    //       1, // version
    //       your_custom_layer_shell_dispatcher_data // if needed
    //   );
    //   And DesktopState would need to implement GlobalDispatch for YourLayerShellInterface.

    Ok(())
}


// --- Output Global Management ---
use smithay::output::{Output, Mode, PhysicalProperties, OutputHandler}; // Added OutputHandler
use smithay::utils::Transform;

/// Creates and registers an initial "virtual" output for the compositor.
///
/// For the MVP, this output has fixed properties:
/// - Name: "HEADLESS-1"
/// - Mode: 1920x1080 @ 60Hz
/// - Physical properties: Default/unknown
/// - Transform: Normal
///
/// The `smithay::output::Output` object handles the creation of the `wl_output` global.
/// This function then integrates the output into the `DesktopState` using the `OutputHandler` trait.
///
/// It is expected to be called once during compositor initialization.
pub fn create_initial_output(state: &mut DesktopState) {
    tracing::info!("Creating initial headless output 'HEADLESS-1'");

    let mode = Mode {
        size: (1920, 1080).into(), // 1920x1080 resolution
        refresh: 60_000,           // 60Hz (in mHz)
    };

    let physical_properties = PhysicalProperties {
        size: (0, 0).into(), // Unknown physical size
        subpixel: smithay::output::Subpixel::Unknown,
        make: "NovaDE".to_string(),
        model: "Virtual Headless".to_string(),
    };

    // The Output::new method creates the wl_output global on the provided display handle.
    // The DesktopState must implement GlobalDispatch<WlOutput, OutputData>,
    // which is typically handled by smithay::output::Output itself if it's created
    // with the display handle where DesktopState is the primary dispatching state.
    // Smithay 0.10+ Output::new takes name, physical_properties, and an optional logger.
    // It automatically creates the global.
    let new_output = Output::new(
        "HEADLESS-1".to_string(),
        physical_properties,
        Some(tracing::Span::current().into()), // Or None if no specific logger
    );

    // Set preferred and current mode
    new_output.add_mode(mode);
    new_output.set_preferred_mode(mode);
    if !new_output.set_current_mode(mode) {
        tracing::error!("Failed to set current mode for initial output HEADLESS-1. This should not happen with a newly created output and mode.");
        // Depending on error handling strategy, might panic or try to recover.
        // For MVP, logging an error is sufficient.
    }
    new_output.set_transform(Transform::Normal); // Set default transform

    // Call the OutputHandler::new_output method on DesktopState to integrate it.
    // This will map it to the space and add it to state.outputs.
    // The OutputHandler trait is implemented for DesktopState in output_handlers.rs.
    state.new_output(new_output); // This calls the method from the OutputHandler trait

    tracing::info!("Successfully created and registered 'HEADLESS-1' output with mode {:?} and refresh {} mHz.", mode.size, mode.refresh);
    // Note: The global for wl_output is created by `Output::new` itself.
    // `OutputManagerState` (in DesktopState) will be aware of this output via the `OutputHandler::new_output` call.
    // Clients will see this output when they bind to wl_registry and then to specific outputs.
}



// --- XDG WM Base Global Dispatch ---
use smithay::wayland::shell::xdg::{XdgWmBase, XdgWmBaseClientData, XdgShellState}; // Added XdgShellState here for new_client

impl GlobalDispatch<XdgWmBase, XdgWmBaseClientData> for DesktopState {
    fn bind(
        _state: &mut Self, // DesktopState instance
        _handle: &DisplayHandle,
        client: &Client,
        resource: New<XdgWmBase>, // New<T> for versioned globals
        _global_data: &XdgWmBaseClientData, // Global data associated with XdgWmBase itself (usually empty)
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!(client_id = ?client.id(), resource_id = ?resource.id(), "Client binding to XdgWmBase");

        // XdgShellState::new_client correctly initializes the client data for XdgWmBase.
        // This data will be associated with the client and accessible by XdgShellHandler methods.
        let client_data = XdgShellState::new_client(client);
        
        // Initialize the XdgWmBase resource for the client with its specific client data.
        data_init.init(resource, client_data);
    }

    // Optional: `can_view` method can be implemented if access to XdgWmBase global needs to be restricted.
    // fn can_view(client: Client, global_data: &XdgWmBaseClientData) -> bool { true }
}


// Placeholder to remind that DesktopState::new() handles global registration for compositor & shm
// by calling CompositorState::new() and ShmState::new().
// No explicit `create_core_compositor_globals` function call is needed in main.rs if done there.
pub fn ensure_globals_are_initialized_in_desktop_state_new() {
    // This is a conceptual marker. Actual calls are in DesktopState::new()
    // e.g., self.compositor_state = CompositorState::new::<Self>(display_handle);
    // e.g., self.shm_state = ShmState::new::<Self>(display_handle, vec![]);
    // e.g., self.xdg_shell_state = XdgShellState::new::<Self>(display_handle);
}

// --- SHM Global Dispatch ---
use smithay::wayland::shm::WlShm; // ShmState is not directly needed here for dispatch

impl GlobalDispatch<WlShm, ()> for DesktopState {
    fn bind(
        _state: &mut Self, // Access to DesktopState if needed, e.g., to get ShmState config
        _handle: &DisplayHandle,
        client: &Client,
        resource: New<WlShm>, // New<T> for versioned globals
        _global_data: &(), // Data associated with the global itself, usually ()
        data_init: &mut DataInit<'_, Self>, // Used to initialize the resource for the client
    ) {
        tracing::info!(client_id = ?client.id(), resource_id = ?resource.id(), "Client binding to WlShm");

        // Smithay's ShmState handles the actual binding logic, including sending supported formats
        // which were configured when ShmState was created in DesktopState::new().
        // All we need to do here is initialize the resource for the client.
        // The ShmState within DesktopState must have been initialized with <Self> as the delegate type.
        // Example: ShmState::new::<Self>(&display_handle, supported_formats)
        
        // The `data_init.init(resource, user_data)` call binds the resource to the client.
        // The `user_data` here is for the WlShm resource instance itself, if any.
        // For WlShm, it's typically `()`, as the ShmState manages the actual state.
        data_init.init(resource, ());
    }
}

// --- Seat Global Dispatch ---
// Smithay's SeatState::new_wl_seat function handles the creation of the wl_seat global
// and its registration with the display. It also sets up the necessary dispatch logic
// internally, provided that DesktopState (or whatever state struct is used) implements
// SeatHandler and GlobalDispatch<WlSeat, SeatGlobalData>.
//
// The `GlobalDispatch<WlSeat, SeatGlobalData>` implementation for `DesktopState` would look like this:
// (This is often provided by smithay::delegate_seat! macro or similar mechanisms if
// `SeatState` is used as intended.)

/*
use smithay::wayland::seat::{SeatGlobalData, WlSeat}; // WlSeat might be from reexports::wayland_server::protocol::wl_seat

impl GlobalDispatch<WlSeat, SeatGlobalData> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<WlSeat>,
        _global_data: &SeatGlobalData, // Data associated with the wl_seat global itself
        data_init: &mut DataInit<'_, Self>,
    ) {
        // SeatState::new_wl_seat already created the global.
        // The binding of a client to this global instance is what this `bind` call achieves.
        // Smithay's SeatState will typically handle the initialization of the client's seat resource.
        // We might need to call a method on self.seat_state or self.seat to finalize client binding
        // if not automatically handled by just data_init.init.
        // However, for wl_seat, the SeatState often manages this transparently when the global is created
        // with DesktopState as the handler type. The delegate_seat! macro helps here.

        // This init call binds the wl_seat resource for the specific client.
        // The user_data for the wl_seat resource itself is typically managed by SeatState.
        // Smithay's `Seat::add_client` or similar internal logic handles this.
        // It's possible that `data_init.init` with appropriate user_data (often derived from Seat::new_client_data())
        // is all that's needed if not using a higher-level abstraction from SeatState for this.

        // Assuming `self.seat` is the main `Seat<Self>` instance.
        // When a client binds to `wl_seat`, a new `wl_seat` resource is created for that client.
        // This resource needs to be associated with your `Seat<Self>` logic.
        // Smithay's `SeatState` usually handles this association if the global was created via `new_wl_seat`.
        // The `delegate_seat!` macro ensures that requests to `wl_seat` resources are forwarded
        // to the `SeatData` and then to `SeatHandler` methods on `DesktopState`.

        // For a client binding to wl_seat, Smithay needs to associate the new wl_seat resource
        // with the server-side Seat object. This is often done by initializing the resource
        // with user data that links it to the server's Seat. Smithay's SeatState typically
        // provides a way to do this, or it's handled by the new_wl_seat mechanism.

        // If using `Seat::add_client` pattern (more manual):
        // let seat_data = self.seat.add_client(client_resource_id); // client_resource_id is resource.id()
        // data_init.init(resource, seat_data);

        // If relying on `SeatState` and `delegate_seat!`:
        // The init might just need default user data for the WlSeat resource if Smithay handles
        // the connection internally. Often, the resource itself doesn't need complex user data
        // because its state is derived from the main `Seat<Self>` object via `SeatData::seat()`.
        // Smithay's `Seat::new_client_data()` is often used here.
        let client_seat_data = smithay::wayland::seat::SeatData::new(); // This is a marker.
                                                                    // Actual user_data for wl_seat might be more complex
                                                                    // or managed by SeatState/Seat.
                                                                    // The `delegate_seat!` macro implies that requests to
                                                                    // this client's wl_seat resource will be handled by
                                                                    // methods on `DesktopState` via `SeatData`.
                                                                    // The important part is that the resource is initialized.
        data_init.init(resource, client_seat_data);

        tracing::info!("Client bound to WlSeat global. Resource ID: {:?}", resource.id());
    }

    // Optional: Implement can_view if access to wl_seat needs to be restricted.
    // fn can_view(client: Client, global_data: &SeatGlobalData) -> bool { true }
}
*/

// Note: The `delegate_seat!(DesktopState);` macro in `compositor/core/state.rs` effectively
// generates the necessary `GlobalDispatch` and other `Dispatch` implementations for `wl_seat`
// and its associated objects like `wl_pointer`, `wl_keyboard`, `wl_touch`, assuming `DesktopState`
// is configured as the handler for these.
// Thus, explicit `GlobalDispatch<WlSeat, ...>` might not be needed here if `delegate_seat`
// covers it, which it usually does for the main `wl_seat` global created by `SeatState::new_wl_seat`.
// The global itself is created in `DesktopState::new()`.
