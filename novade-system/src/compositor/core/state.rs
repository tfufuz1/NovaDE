use std::marker::PhantomData;
use smithay::{
    delegate_compositor, delegate_data_device, delegate_output, delegate_presentation,
    delegate_seat, delegate_shm, delegate_viewporter, delegate_xdg_shell, delegate_xdg_activation,
    delegate_xdg_decoration, // Added for XdgDecorationState
    desktop::{Space, Window, WindowSurfaceType},
    input::{Seat, SeatState, pointer::CursorImageStatus},
    reexports::{
        calloop::{generic::Generic, Interest, LoopHandle, LoopSignal, Mode, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason, GlobalId},
            protocol::{
                wl_output, wl_seat, wl_shm, wl_surface::WlSurface,
            },
            Client, Display, DisplayHandle, Global, Listener,
        },
    },
    utils::{Clock, Logical, Point, Rectangle, Scale, Transform},
    wayland::{
        compositor::{CompositorClientState, CompositorState, CompositorGlobal},
        output::{OutputManagerState, Output},
        presentation::{PresentationState, PresentationGlobal},
        selection::data_device::DataDeviceState,
        shell::xdg::{
            XdgShellState, XdgShellGlobal, xdg_wm_base::XdgWmBase, // Added XdgWmBase for version logging
            decoration::XdgDecorationState, // Added for XdgDecorationState
        },
        activation::{XdgActivationState, XdgActivationGlobal}, // Added for XdgActivationState
        shm::{ShmState, ShmGlobal},
        socket::ListeningSocketSource,
        viewporter::ViewporterState,
    },
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex}; // Added Mutex here
use slog_scope::{info, warn, error}; // For logging

use smithay::wayland::compositor::{
    CompositorHandler, SurfaceAttributes as WlSurfaceAttributes, CompositorClientState as SmithayCompositorClientState, add_destruction_hook, with_states
};
use smithay::reexports::wayland_server::protocol::wl_buffer::{self, WlBuffer}; // For WlBuffer in AttachedBufferInfo & BufferHandler
use smithay::reexports::wayland_server::backend::UserDataMap; // For UserDataMap access
use smithay::wayland::shm::{ShmHandler, ShmState as SmithayShmState}; // For ShmHandler trait & ShmState type
use smithay::wayland::buffer::BufferHandler; // For BufferHandler trait
use smithay::desktop::WindowSurface; // To get WlSurface from Window
use smithay::reexports::wayland_server::{Dispatch, GlobalDispatch, New, DataInit, DisplayHandle as WaylandDisplayHandle, Client as WaylandClient};
use smithay::reexports::wayland_server::protocol::{
    wl_compositor::WlCompositor,
    wl_subcompositor::WlSubcompositor,
    wl_shm::WlShm,
};


// Assuming these are in the correct relative path
use crate::compositor::surface_management::{SurfaceData, AttachedBufferInfo};
use crate::compositor::core::errors::CompositorCoreError;

// Domain service traits
use novade_domain::window_management_policy::service::WindowManagementPolicyService;
use novade_domain::workspaces::manager::WorkspaceManagerService;


pub const CLOCK_ID: usize = 0;

#[derive(Debug)]
pub struct ClientCompositorData {
    // As per spec, CompositorClientState is part of this.
    // Smithay's CompositorClientState is a struct that can be directly stored.
    pub compositor_state: SmithayCompositorClientState,
    // Potentially other client-specific data for the compositor module can go here.
}

impl ClientData for ClientCompositorData {
    fn initialized(&self, client_id: ClientId) {
        info!(slog_scope::logger(), "ClientCompositorData initialized"; "client_id" => ?client_id);
    }

    fn disconnected(&self, client_id: ClientId, reason: DisconnectReason) {
        info!(slog_scope::logger(), "ClientCompositorData disconnected"; "client_id" => ?client_id, "reason" => ?reason);
    }
}


// Per spec clarification, client data (ClientCompositorData) might be wrapped in Arc<Mutex<ClientCompositorData>>
// if it needs to be shared and mutated across different handlers for the same client.
// However, CompositorHandler::client_compositor_state expects a direct reference.
// Smithay typically has the specific state (like CompositorClientState) directly in client's UserData.
// If ClientCompositorData is just a wrapper for CompositorClientState, it might be redundant
// unless other fields are added to ClientCompositorData.

// For now, let's assume ClientCompositorData is stored directly in the client's UserDataMap
// as some form of client-global state for our compositor module.
// If it needs to be Arc<Mutex<...>>, the retrieval in client_compositor_state will need adjustment.
// The trait signature `&'a CompositorClientState` implies direct access.

#[derive(Debug, Default)]
pub struct NovaDEWaylandState {
    pub shm_global: Option<GlobalId>,
    pub compositor_global: Option<GlobalId>, // Managed by CompositorState, ID not typically stored by user
    pub subcompositor_global: Option<GlobalId>, // Managed by CompositorState, ID not typically stored by user
    pub xdg_shell_global: Option<GlobalId>,
    pub xdg_activation_global: Option<GlobalId>,
    pub xdg_decoration_manager_global: Option<GlobalId>, // Added field
    pub output_manager_global: Option<GlobalId>,
    pub seat_global: Option<GlobalId>,
    pub presentation_global: Option<GlobalId>,
    pub viewporter_global: Option<GlobalId>,
    // Add other global IDs as needed
}

impl NovaDEWaylandState {
    /// Checks if all essential globals have been initialized.
    pub fn is_initialized(&self) -> bool {
        // Define "essential" based on your compositor's needs.
        self.shm_global.is_some() &&
        self.xdg_shell_global.is_some() && 
        self.xdg_activation_global.is_some() &&
        self.xdg_decoration_manager_global.is_some() // XDG Decoration is essential
        // && self.output_manager_global.is_some() // etc. for other important globals
    }
}

#[derive(Debug)]
pub struct DesktopState {
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, Self>,
    pub clock: Clock<u64>,
    pub compositor_state: CompositorState,
    pub shm_state: ShmState,
    pub presentation_state: PresentationState,
    pub viewporter_state: ViewporterState,
    pub seat_state: SeatState<Self>,
    pub seat: Seat<Self>,
    pub seat_name: String,
    pub output_manager_state: OutputManagerState,
    pub data_device_state: DataDeviceState,

    // XDG Shell, Activation, and Decoration states
    pub xdg_shell_state: XdgShellState,
    pub xdg_activation_state: XdgActivationState,
    pub xdg_decoration_state: XdgDecorationState,

    // Input related state
    pub pointer_location: Point<f64, Logical>, // Current pointer location on the global space

    pub space: Option<Space<Window>>, // Assuming Window is the generic type for Space
    pub windows: HashMap<usize, Window>, // Example: Using usize as a window ID

    // Domain service handles
    pub input_service_handle: PhantomData<()>, // Remains PhantomData for now
    pub output_service_handle: PhantomData<()>, // Remains PhantomData for now
    pub window_policy_service: Arc<dyn WindowManagementPolicyService>,
    pub workspace_manager_service: Arc<dyn WorkspaceManagerService>,

    pub wayland_globals: Option<NovaDEWaylandState>,
    // Add other fields like cursor_status, loop_signal, etc.
    pub cursor_status: Arc<std::sync::Mutex<CursorImageStatus>>,
    pub loop_signal: LoopSignal,

}

impl DesktopState {
    pub fn new(
        display_handle: DisplayHandle,
        loop_handle: LoopHandle<'static, Self>,
        // loop_signal: LoopSignal, // Removed as per task, DesktopState.loop_signal will be from loop_handle
        window_policy_service: Arc<dyn WindowManagementPolicyService>,
        workspace_manager_service: Arc<dyn WorkspaceManagerService>
    ) -> Self {
        let clock = Clock::new().unwrap(); // Or handle error appropriately
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]); // Add allowed formats if any
        let presentation_state = PresentationState::new::<Self>(&display_handle, clock.id() as _);
        let viewporter_state = ViewporterState::new::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let seat_name = "seat-0".to_string();
        let seat = seat_state.new_seat(&seat_name);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        
        // Initialize XDG Shell, Activation, and Decoration states
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle, Some(tracing::Span::current()));
        let xdg_activation_state = XdgActivationState::new::<Self>(&display_handle, Some(tracing::Span::current()));
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle, Some(tracing::Span::current()));

        Self {
            display_handle,
            loop_handle,
            clock,
            compositor_state,
            shm_state,
            presentation_state,
            viewporter_state,
            seat_state,
            seat,
            seat_name,
            output_manager_state,
            data_device_state,
            xdg_shell_state,
            xdg_activation_state,
            xdg_decoration_state,
            pointer_location: Point::from((0.0, 0.0)), // Initialize to default
            space: Some(Space::new(slog_scope::logger())), // Requires a logger
            windows: HashMap::new(),
            input_service_handle: PhantomData, // Stays PhantomData
            output_service_handle: PhantomData, // Stays PhantomData
            window_policy_service,
            workspace_manager_service,
            wayland_globals: Some(NovaDEWaylandState::default()), // Initialize with default (all None)
            cursor_status: Arc::new(std::sync::Mutex::new(CursorImageStatus::Default)),
            loop_signal: loop_handle.get_signal(), // Initialize from loop_handle
        }
    }

    /// Creates and registers initial Wayland globals.
    /// This should be called after DesktopState is created and essential Smithay states (ShmState, CompositorState)
    /// are initialized within it.
    pub fn create_initial_wayland_globals(&mut self, display_handle: &WaylandDisplayHandle) {
        info!(slog_scope::logger(), "Registering initial Wayland globals...");

        // Compositor and Subcompositor globals are managed by CompositorState upon its creation.
        // Smithay's CompositorState::new() handles their registration.
        // We log their conceptual creation here. Actual versions are defined by Smithay.
        // Typically, wl_compositor is v4-v6, wl_subcompositor is v1.
        info!(slog_scope::logger(), "wl_compositor (e.g., v6) and wl_subcompositor (e.g., v1) globals registered internally by CompositorState.");
        // We don't store their GlobalIds in NovaDEWaylandState as CompositorState does not expose them,
        // and they are not typically needed by the user of CompositorState.
        // So, self.wayland_globals.compositor_global and subcompositor_global remain None.

        // SHM Global
        // ShmState::new() also registers the wl_shm global. We retrieve its GlobalId.
        let shm_global_id = self.shm_state.global().clone();
        if let Some(globals) = self.wayland_globals.as_mut() {
            globals.shm_global = Some(shm_global_id);
            info!(slog_scope::logger(), "wl_shm global registered and its ID stored."; "global_id" => ?globals.shm_global);
        } else {
            error!(slog_scope::logger(), "Wayland globals state not initialized in DesktopState.");
        }

        // Note: Other globals like XDG Shell, Presentation, Output Manager, Seat, Viewporter
        // will be created in subsequent tasks when their respective handlers are implemented.
        // Their GlobalIds will be stored in NovaDEWaylandState at that point.
        
        // XDG Shell Global (xdg_wm_base)
        // XdgShellState::new() registers the xdg_wm_base global. We retrieve its GlobalId.
        let xdg_shell_global_id = self.xdg_shell_state.global().clone();
        if let Some(globals) = self.wayland_globals.as_mut() {
            globals.xdg_shell_global = Some(xdg_shell_global_id);
            info!(slog_scope::logger(), "xdg_wm_base global v{} registered and its ID stored.", XdgWmBase::VERSION; "global_id" => ?globals.xdg_shell_global);
        } else {
            error!(slog_scope::logger(), "Wayland globals state not initialized in DesktopState for XDG Shell.");
        }

        // XDG Activation Global (xdg_activation_v1)
        // XdgActivationState::new() registers the xdg_activation_v1 global.
        let xdg_activation_global_id = self.xdg_activation_state.global().clone();
        if let Some(globals) = self.wayland_globals.as_mut() {
            globals.xdg_activation_global = Some(xdg_activation_global_id);
            info!(slog_scope::logger(), "xdg_activation_v1 global registered and its ID stored."; "global_id" => ?globals.xdg_activation_global);
        } else {
            error!(slog_scope::logger(), "Wayland globals state not initialized in DesktopState for XDG Activation.");
        }

        // XDG Decoration Manager Global (xdg_decoration_manager_v1)
        let decoration_manager_global_id = self.xdg_decoration_state.global().clone();
        if let Some(globals) = self.wayland_globals.as_mut() {
            globals.xdg_decoration_manager_global = Some(decoration_manager_global_id);
            info!(slog_scope::logger(), "xdg_decoration_manager_v1 global registered and its ID stored."; "global_id" => ?globals.xdg_decoration_manager_global);
        } else {
            error!(slog_scope::logger(), "Wayland globals state not initialized in DesktopState for XDG Decoration Manager.");
        }
    }
}

// Implement necessary delegate traits for DesktopState
delegate_compositor!(@<Backend = DesktopState> DesktopState);
delegate_shm!(@<Backend = DesktopState> DesktopState);
delegate_seat!(@<Backend = DesktopState> DesktopState);
delegate_output!(@<Backend = DesktopState> DesktopState);
delegate_data_device!(@<Backend = DesktopState> DesktopState);
delegate_presentation!(@<Backend = DesktopState> DesktopState);
delegate_viewporter!(@<Backend = DesktopState> DesktopState);
delegate_xdg_shell!(@<Backend = DesktopState> DesktopState);
delegate_xdg_activation!(@<Backend = DesktopState> DesktopState);
delegate_xdg_decoration!(@<Backend = DesktopState> DesktopState); // Delegate for XDG Decoration

// For XdgShell, if it's optional, you might need a custom impl or ensure it's initialized before delegation.
// This is a simplified example; actual delegation might need more setup.

impl ShmHandler for DesktopState {
    fn shm_state(&self) -> &SmithayShmState { // Corrected to SmithayShmState
        &self.shm_state
    }
    // shm_client_data default is fine due to `impl smithay::wayland::shm::ShmClientData for ClientCompositorData {}`
}

impl GlobalDispatch<WlShm, ()> for DesktopState {
    fn bind(
        _state: &mut Self, // DesktopState itself
        _handle: &WaylandDisplayHandle,
        _client: &WaylandClient,
        resource: New<WlShm>,
        _global_data: &(), // No global data for WlShm
        data_init: &mut DataInit<'_, Self>,
    ) {
        info!(slog_scope::logger(), "Client bound to wl_shm global"; "client_id" => ?_client.id());
        // ShmState has its own bind/initialization logic, often handled by delegate_shm or by ShmState itself.
        // For WlShm, data_init.init is used to associate the resource with its user data (if any for the resource itself).
        // Smithay's ShmState uses the resource user data for ShmPool data.
        // The ShmHandler::shm_state() provides the ShmState that wl_shm objects interact with.
        // The actual binding logic for wl_shm (creating pools etc.) is managed by ShmState internally when requests arrive.
        // Our role here is just to initialize the resource.
        data_init.init(resource, ()); // Minimal user data for the WlShm object itself.
    }
}


impl BufferHandler for DesktopState {
    fn buffer_destroyed(&mut self, buffer: &wl_buffer::WlBuffer) {
        info!(slog_scope::logger(), "SHM buffer destroyed"; "buffer_id" => ?buffer.id());

        // The primary concern is to update any SurfaceData that might still hold a reference
        // to this buffer in its current_buffer_info or texture_handle.

        // We need to iterate over surfaces that might be using this buffer.
        // If using smithay::desktop::Space, iterate its elements.
        if let Some(space) = self.space.as_mut() { // Need mutable space for damage_window
            // Create a list of surfaces to update to avoid holding locks during iteration if possible,
            // though direct modification with locks inside iteration is also common.
            // We are iterating elements of the space, which are likely Arc<Window>.
            for window in space.elements() {
                // Check if the window's surface is using this buffer.
                // A Window in smithay::desktop::Space can be of various types.
                // We need to get its WlSurface.
                let surface = match window.wl_surface() {
                    Some(srf) => srf,
                    None => continue, // This window does not have a direct WlSurface (e.g., custom element)
                };

                if let Some(surface_data_arc) = surface.data_map().get::<Arc<SurfaceData>>() {
                    let surface_data = surface_data_arc.clone(); // Clone Arc for local use

                    // Check and update current_buffer_info
                    let mut buffer_info_lock = surface_data.current_buffer_info.lock().unwrap();
                    if let Some(current_bi) = &*buffer_info_lock {
                        if current_bi.buffer.id() == buffer.id() {
                            info!(slog_scope::logger(), "Buffer being destroyed was in use by surface"; "surface_id" => ?surface.id(), "buffer_id" => ?buffer.id());
                            *buffer_info_lock = None;

                            // Release associated texture handle
                            let mut texture_handle_lock = surface_data.texture_handle.lock().unwrap();
                            if texture_handle_lock.is_some() {
                                *texture_handle_lock = None;
                                info!(slog_scope::logger(), "Renderer texture handle released for surface due to buffer destruction"; "surface_id" => ?surface.id());
                            }
                            drop(texture_handle_lock); // Release lock early

                            // Mark the surface as damaged because its content is now invalid.
                            // This typically involves damaging the window in the rendering space.
                            // The exact method might depend on the renderer and window management.
                            // Using space.damage_window as suggested by the spec.
                            // The location and size of damage should ideally cover the whole window.
                            let surface_size = window.geometry().size; // Get window size
                            let damage_rect = Rectangle::from_loc_and_size(Point::from((0,0)), surface_size);
                            space.damage_window(window, damage_rect.to_logical(1), []); // Damage the whole window, scale 1
                            info!(slog_scope::logger(), "Surface damaged due to buffer destruction"; "surface_id" => ?surface.id());
                        }
                    }
                    drop(buffer_info_lock); // Release lock
                }
            }
        } else {
            warn!(slog_scope::logger(), "Space not available in DesktopState during buffer_destroyed. Cannot iterate windows.");
        }
    }
}


// Minimal implementation for ClientData for DesktopState (if DesktopState itself is used as ClientData)
// This is unusual; typically, a separate struct like ClientCompositorData is used.
// If DesktopState is not ClientData, this impl is not needed.
// impl ClientData for DesktopState {
//     fn initialized(&self, _client_id: ClientId) {}
//     fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
// }

// Required for ShmState if you don't want to use the default client data
impl smithay::wayland::shm::ShmClientData for ClientCompositorData {}

// Required for CompositorState if you don't want to use the default client data
impl smithay::wayland::compositor::CompositorClientData for ClientCompositorData {
    fn compositor_state(&self) -> &SmithayCompositorClientState { // Corrected type
        &self.compositor_state
    }
}

impl GlobalDispatch<WlCompositor, ()> for DesktopState {
    fn bind(
        _state: &mut Self, // DesktopState
        _handle: &WaylandDisplayHandle,
        client: &WaylandClient,
        resource: New<WlCompositor>,
        _global_data: &(), // No global data for WlCompositor itself
        data_init: &mut DataInit<'_, Self>,
    ) {
        info!(slog_scope::logger(), "Client bound to wl_compositor global"; "client_id" => ?client.id());

        // Ensure ClientCompositorData is initialized for the client.
        // This data is specific to our compositor logic for this client.
        // Smithay's CompositorState also has its own internal client data (CompositorClientState).
        // We added `impl smithay::wayland::compositor::CompositorClientData for ClientCompositorData`
        // which means our ClientCompositorData *is* the CompositorClientState for Smithay's systems.
        if client.get_data::<ClientCompositorData>().is_none() {
            // If ClientCompositorData wraps Smithay's CompositorClientState, initialize it.
            // SmithayCompositorClientState is Default.
            client.insert_user_data(|| ClientCompositorData {
                compositor_state: Default::default(),
            });
            info!(slog_scope::logger(), "Initialized ClientCompositorData for client"; "client_id" => ?client.id());
        } else {
            info!(slog_scope::logger(), "ClientCompositorData already exists for client"; "client_id" => ?client.id());
        }

        // Initialize the WlCompositor resource. It typically doesn't have its own user data.
        data_init.init(resource, ());
    }
    // `advertise` is handled by `CompositorState` internally.
}

impl GlobalDispatch<WlSubcompositor, ()> for DesktopState {
    fn bind(
        _state: &mut Self, // DesktopState
        _handle: &WaylandDisplayHandle,
        client: &WaylandClient,
        resource: New<WlSubcompositor>,
        _global_data: &(), // No global data for WlSubcompositor itself
        data_init: &mut DataInit<'_, Self>,
    ) {
        info!(slog_scope::logger(), "Client bound to wl_subcompositor global"; "client_id" => ?client.id());
        // WlSubcompositor also doesn't typically have its own user data.
        // Its functionality is handled by interactions with CompositorState.
        data_init.init(resource, ());
    }
    // `advertise` is handled by `CompositorState` internally.
}


impl CompositorHandler for DesktopState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a SmithayCompositorClientState {
        // As per spec, retrieve ClientCompositorData from client's UserDataMap.
        // The spec mentions Arc<Mutex<ClientCompositorData>> but the trait expects &'a CompositorClientState.
        // If ClientCompositorData *only* holds CompositorClientState, and ClientCompositorData itself
        // is what's stored per-client, then this is straightforward.

        // Option 1: ClientCompositorData is stored directly.
        match client.get_data::<ClientCompositorData>() {
            Some(client_data) => &client_data.compositor_state,
            None => {
                // This case should ideally not happen if ClientCompositorData is correctly initialized for each client.
                // The trait signature does not allow returning an error.
                // Smithay often panics in such cases or logs an error and returns a default/dummy state if possible.
                // For safety, we should ensure ClientCompositorData is always added when a client connects
                // (e.g., in a Display::new_client callback).
                error!(slog_scope::logger(), "ClientCompositorData not found for client. This is a bug."; "client" => ?client.id());
                // Panic is an option, but let's consider if there's a default/dummy state.
                // Smithay's own CompositorState::new_client inserts a CompositorClientState.
                // If we are overriding client_compositor_state, we must provide it.
                // This indicates ClientCompositorData (or at least CompositorClientState)
                // must be initialized when the client is created.
                // Let's assume it's a bug and panic is appropriate if it's truly missing.
                panic!("ClientCompositorData not found for client. Ensure it's initialized on client creation.");
            }
        }
    }

    fn new_surface(&mut self, surface: &WlSurface) {
        let client_id = match surface.client() {
            Some(client) => client.id(),
            None => {
                // This should not happen for a newly created surface by a client.
                error!(slog_scope::logger(), "New surface has no associated client"; "surface_id" => ?surface.id());
                // Decide on error handling: panic, or skip initialization.
                // Skipping might lead to issues later. Panicking might be too harsh.
                // For now, log and return. This surface will lack SurfaceData.
                return;
            }
        };

        info!(slog_scope::logger(), "New surface created"; "surface_id" => ?surface.id(), "client_id" => ?client_id);

        // Create SurfaceData, wrap in Arc<Mutex<SurfaceData>> as it contains Mutex fields
        // and might be accessed concurrently if hooks or other parts of the system need it.
        // Smithay's UserDataMap itself is thread-safe for insertions.
        // The internal Mutexes in SurfaceData are for its fields.
        // Storing Arc<SurfaceData> is also common if SurfaceData itself is designed for shared access
        // with interior mutability. The previous step created SurfaceData with Mutex fields.
        // So Arc<SurfaceData> is appropriate. If SurfaceData needed to be externally mutable
        // (e.g. &mut SurfaceData), then Arc<Mutex<SurfaceData>> would be used.
        // Given SurfaceData has internal Mutexes, Arc<SurfaceData> is fine.
        let surface_data = Arc::new(SurfaceData::new(client_id));

        // Insert SurfaceData into the surface's UserDataMap.
        // insert_if_missing_threadsafe ensures it's only inserted once.
        surface.data_map().insert_if_missing_threadsafe(|| surface_data.clone());

        // Add a destruction hook.
        // This hook is called when the WlSurface object is being destroyed.
        add_destruction_hook(surface, |surface_data_map: &UserDataMap| {
            // The primary cleanup of SurfaceData from the map is handled by Smithay
            // when the WlSurface is destroyed, as UserDataMap drops its contents.
            // This hook is for custom cleanup logic if DesktopState held direct references
            // or needed to react to the destruction beyond UserDataMap cleanup.

            // Example: If SurfaceData contained a reference that needs explicit cleanup,
            // or if this surface was tracked in a list in DesktopState.
            // For now, just log.
            if let Some(data) = surface_data_map.get::<Arc<SurfaceData>>() {
                info!(slog_scope::logger(), "Surface destruction hook called"; "surface_id_in_data" => %data.id);
                // If SurfaceData had a custom destruction_callback field (as defined in previous step):
                // if let Some(callback) = data.destruction_callback.lock().unwrap().take() {
                //     // This would require access to &mut DesktopState, which is not available in this hook.
                //     // Hooks are limited in what they can access.
                //     // This implies that `destruction_callback` in `SurfaceData` needs careful design
                //     // if it needs to interact with global state.
                //     // Typically, such callbacks would be invoked *before* this low-level hook,
                //     // e.g., by the role handler (XDG shell, etc.).
                // }
            } else {
                warn!(slog_scope::logger(), "SurfaceData not found in UserDataMap during destruction hook. This is unexpected.");
            }
            info!(slog_scope::logger(), "Surface destruction hook finished for a surface (ID unknown here without map access to WlSurface)");
        });
    }

    fn commit(&mut self, surface: &WlSurface) {
        info!(slog_scope::logger(), "Commit on surface"; "surface_id" => ?surface.id());

        // Process subsurfaces first as per Wayland spec (parent commits apply to children)
        // This is complex. Smithay's Space::commit_subsurfaces handles this if using Space.
        // For now, we'll focus on the individual surface commit.
        // TODO: Handle synchronized subsurfaces correctly. Smithay's XdgShellHandler often does this.

        if surface.is_sync_subsurface() {
            info!(slog_scope::logger(), "Commit on sync subsurface, potentially deferred"; "surface_id" => ?surface.id());
            // Actual deferral logic is handled by parent surface commit or shell-specific logic.
        }

        with_states(surface, |states| {
            let surface_attributes = states.cached_state.current::<WlSurfaceAttributes>();
            let surface_data_guard = states.data_map.get::<Arc<SurfaceData>>();

            if surface_data_guard.is_none() {
                error!(slog_scope::logger(), "SurfaceData not found on commit for surface"; "surface_id" => ?surface.id());
                return;
            }
            let surface_data = surface_data_guard.unwrap(); // We checked for none

            // Buffer Handling
            if surface_attributes.buffer.is_some() ||
               surface_attributes.buffer_transform != surface_data.current_buffer_info.lock().unwrap().as_ref().map_or(Transform::Normal, |bi| bi.transform) ||
               surface_attributes.buffer_scale != surface_data.current_buffer_info.lock().unwrap().as_ref().map_or(1, |bi| bi.scale)
            {
                info!(slog_scope::logger(), "New buffer or buffer properties changed"; "surface_id" => ?surface.id());
                let mut current_buffer_info = surface_data.current_buffer_info.lock().unwrap();

                if let Some(new_buffer) = &surface_attributes.buffer {
                     // Buffer dimensions are usually available from the buffer itself,
                     // but Smithay often expects them to be part of WlSurfaceAttributes or derived.
                     // For simplicity, we assume dimensions are obtainable or are part of a renderer's task.
                     // The `AttachedBufferInfo` needs dimensions.
                     // Smithay's `buffer_dimensions` utility can get this.
                     let dimensions = smithay::desktop::utils::buffer_dimensions(new_buffer);
                     if dimensions.is_none() {
                        error!(slog_scope::logger(), "Could not get dimensions for attached buffer"; "surface_id" => ?surface.id());
                        // Decide how to handle: clear buffer_info, or keep old one?
                        // For now, let's not update if dimensions are missing.
                        return;
                     }

                    *current_buffer_info = Some(AttachedBufferInfo {
                        buffer: new_buffer.clone(),
                        scale: surface_attributes.buffer_scale,
                        transform: surface_attributes.buffer_transform,
                        dimensions: dimensions.unwrap(), // Safe due to check
                    });
                    // TODO: Notify renderer about the new buffer. This will be a separate step.
                    info!(slog_scope::logger(), "Renderer notification placeholder: new buffer attached"; "surface_id" => ?surface.id());
                } else {
                    // Buffer was detached
                    *current_buffer_info = None;
                    info!(slog_scope::logger(), "Buffer detached"; "surface_id" => ?surface.id());
                }
            }

            // Damage Tracking
            // damage is in surface coordinates relative to the new buffer
            if !surface_attributes.damage.is_empty() {
                info!(slog_scope::logger(), "Surface damage received"; "surface_id" => ?surface.id(), "damage_count" => surface_attributes.damage.len());
                let mut damage_buffer_coords = surface_data.damage_buffer_coords.lock().unwrap();
                damage_buffer_coords.extend(surface_attributes.damage.clone());
                // TODO: Transform this damage to global coordinates if needed for rendering.
            }

            // Role-Specific Commit Logic (Placeholder)
            // This is typically handled by XDG shell handlers, LayerShell handlers, etc.
            // For example, an XDG surface commit would trigger `XdgShellHandler::commit`.
            // Smithay's `Space::map_element` and related functions often tie into this.
            if surface_data.role.lock().unwrap().is_some() {
                info!(slog_scope::logger(), "Role-specific commit logic placeholder"; "surface_id" => ?surface.id(), "role" => ?surface_data.role.lock().unwrap());
            }

            // TODO: Mark surface for redraw.
            // This usually involves damaging the window/element in the render tree or space.
            // E.g., if part of a `Space`, `space.damage_window(&window, ...)`
            // For now, just a log.
            info!(slog_scope::logger(), "Placeholder: Mark surface for redraw"; "surface_id" => ?surface.id());

        }); // end of with_states

        // After processing the main surface, handle subsurface order changes, etc.
        // This is also often part of shell-specific logic or Space management.
        // smithay::wayland::compositor::update_state_surface_commit(surface, &self.compositor_state);
        // The above is an internal detail usually handled by delegate_compositor or similar.
    }


    fn new_subsurface(&mut self, surface: &WlSurface, parent: &WlSurface) {
        info!(slog_scope::logger(), "New subsurface created"; "surface_id" => ?surface.id(), "parent_id" => ?parent.id());

        let surface_data_opt = surface.data_map().get::<Arc<SurfaceData>>();
        let parent_surface_data_opt = parent.data_map().get::<Arc<SurfaceData>>();

        if let (Some(surface_data), Some(parent_surface_data)) = (surface_data_opt, parent_surface_data_opt) {
            // Set parent for the new subsurface
            let mut surface_parent_weak = surface_data.parent.lock().unwrap();
            *surface_parent_weak = Some(parent.downgrade());

            // Add new subsurface to parent's children list
            let mut parent_children_weak = parent_surface_data.children.lock().unwrap();
            parent_children_weak.push(surface.downgrade());

            info!(slog_scope::logger(), "Subsurface relationships updated"; "surface_id" => ?surface.id(), "parent_id" => ?parent.id());
        } else {
            error!(slog_scope::logger(), "SurfaceData not found for surface or parent in new_subsurface";
                "surface_id" => ?surface.id(), "parent_id" => ?parent.id(),
                "surface_data_found" => surface_data_opt.is_some(),
                "parent_surface_data_found" => parent_surface_data_opt.is_some()
            );
        }
    }

    fn destroyed(&mut self, surface: &WlSurface) {
        // This method is called when a WlSurface is destroyed by the client.
        // The UserDataMap associated with the WlSurface (which holds our Arc<SurfaceData>)
        // will be dropped automatically by Smithay shortly after this handler returns.
        // When the Arc<SurfaceData> is dropped (i.e., its strong count reaches zero),
        // the SurfaceData itself will be dropped, and its Drop implementation (if any) will run.

        // The `add_destruction_hook` in `new_surface` provides a way to run code
        // *just before* the UserDataMap is cleared. This is useful if the data
        // in the map needs to be moved out or if some resource it holds needs
        // explicit cleanup that depends on other parts of the map.

        // For `DesktopState`, if it holds any *direct strong references* to this `WlSurface`
        // or its `Arc<SurfaceData>` outside of the surface's own `UserDataMap` (e.g., in a
        // `self.windows` list if that list stored `Arc<SurfaceData>` or `WlSurface` directly,
        // which is less common with modern Smithay patterns that favor querying `Space` or
        // iterating client surfaces), then those references would need to be cleaned up here
        // or in the destruction hook.

        // Given `SurfaceData` is in an `Arc` and primarily lives in `surface.data_map()`,
        // its memory will be managed correctly by `Arc` and `UserDataMap` dropping.
        // The main role of this `destroyed` callback and the destruction hook is to
        // manage application-level state consistency if the surface was part of a larger structure.

        // For now, just log. Any specific cleanup logic (e.g., removing from a focus list,
        // telling a layout manager the surface is gone) would go here or be triggered by
        // role-specific destruction handlers (e.g., XdgShellHandler::xdg_surface_destroyed).
        info!(slog_scope::logger(), "Surface destroyed by client"; "surface_id" => ?surface.id());

        // The destruction hook added in `new_surface` will also run.
        // No need to manually remove from `surface.data_map()` here, Smithay handles it.
    }
}
