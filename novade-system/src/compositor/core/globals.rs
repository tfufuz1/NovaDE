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
    let _dmabuf_global = _desktop_state.dmabuf_state.create_global_with_default_feedback::<DesktopState>(
         _display_handle, // Use the passed display_handle
         &[], // No formats advertised yet, as per plan
         Some(tracing::Span::current()) // Pass a tracing span
    );
    // If _desktop_state.wayland_globals.dmabuf_global = Some(_dmabuf_global); was used.

    tracing::info!("DMABUF global (zwp_linux_dmabuf_v1) registered (no formats advertised yet).");
    
    // TODO: Add creation of other essential globals here as they are implemented, e.g.:
    // - Data Device Manager (wl_data_device_manager)

    Ok(())
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
