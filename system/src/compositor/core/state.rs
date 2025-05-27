use smithay::{
    delegate_compositor, delegate_shm,
    desktop::{Space, Window, WindowSurfaceType, PopupManager}, // Added PopupManager
    reexports::{
        calloop::{generic::Generic, Interest, LoopHandle, LoopSignal, Mode},
        wayland_server::{
            protocol::{wl_compositor, wl_shm, wl_subcompositor, wl_surface::WlSurface}, // Added WlSurface
            Client, DisplayHandle, GlobalDispatch, GlobalId, // Added GlobalId
            ClientHandler as WaylandClientHandler, ClientId,
        },
    },
    wayland::{
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        shm::{ShmHandler, ShmState},
        buffer::BufferHandler,
        shell::xdg::{XdgShellState, XdgShellHandler, XdgToplevelSurfaceData, XdgPopupSurfaceData, ToplevelSurface, PopupSurface, XdgWmBaseClientData, XdgPositionerUserData, XdgSurfaceUserData, XdgActivationState, XdgActivationHandler},
        seat::Seat,
    },
    input::{SeatHandler, SeatState, pointer::CursorImageStatus, keyboard::KeyboardHandle, touch::TouchHandle, TouchSlot}, // Use SeatHandler directly, added TouchHandle and TouchSlot
    utils::{Point, Size, Rectangle, Logical, Physical, Serial, Transform, Clock}, // Added Clock
};
use std::{time::Duration, collections::HashMap, sync::{Arc, Mutex, Weak}}; // Mutex is already here
// Ensure wl_surface is specifically available if not covered by wildcard
use smithay::reexports::wayland_server::protocol::wl_surface;
use uuid::Uuid;
use crate::{
    compositor::{
        shm::ShmError,
        xdg_shell::types::ManagedWindow,
        display_loop::client_data::ClientData,
    },
    input::{keyboard::XkbKeyboardData}, // Removed TouchFocusData
    outputs::manager::OutputManager,
};
use smithay::{
    reexports::wayland_protocols::xdg::xdg_output::server::zxdg_output_manager_v1::ZxdgOutputManagerV1, // For GlobalDispatch
    wayland::output::{OutputManagerState, OutputHandler, OutputData}, // For OutputHandler
    reexports::wayland_server::protocol::wl_output, // For OutputHandler
};

/// Data associated with each client's compositor state.
#[derive(Debug, Clone)]
pub struct ClientCompositorData {
    // Smithay examples often use `ClientState` which is a more general store.
    // For now, let's keep it simple or use Smithay's CompositorClientState directly if sufficient.
    // If we need custom client-specific compositor data, we can add fields here.
    // For example, if a client could have specific capabilities or restrictions.
    _placeholder: (), // Replace with actual fields if needed
}

impl Default for ClientCompositorData {
    fn default() -> Self {
        Self { _placeholder: () }
    }
}

/// The global state for the compositor.
#[derive(Debug)]
pub struct DesktopState {
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, Self>, // 'static might need adjustment based on main loop
    pub loop_signal: LoopSignal,
    pub clock: Clock<u64>, // ADDED/UNCOMMENTED

    // Smithay states
    pub compositor_state: CompositorState,
    pub shm_state: ShmState,
    pub xdg_shell_state: XdgShellState, // Added XDG Shell State
    pub xdg_activation_state: XdgActivationState,

    pub space: Space<ManagedWindow>,
    pub windows: HashMap<crate::compositor::xdg_shell::types::DomainWindowIdentifier, Arc<ManagedWindow>>,

    pub shm_global: Option<GlobalId>,
    pub xdg_shell_global: Option<GlobalId>,
    pub xdg_activation_global: Option<GlobalId>,

    // --- Input Related State ---
    pub seat_state: SeatState<Self>, // Smithay's manager for seat Wayland objects
    pub seat: Seat<Self>,            // Smithay's core Seat object
    pub seat_name: String,           // Name of the primary seat, e.g., "seat0"
    pub keyboard_data_map: HashMap<String, Arc<Mutex<XkbKeyboardData>>>, // MODIFIED type
    pub current_cursor_status: Arc<Mutex<CursorImageStatus>>, // For renderer to observe
    pub pointer_location: Point<f64, Logical>, // Global pointer coordinates
    pub active_input_surface: Option<Weak<WlSurface>>, // General purpose, might be kb focus
    // pub touch_focus_data: TouchFocusData, // REMOVED
    pub active_touch_targets: HashMap<TouchSlot, Weak<wl_surface::WlSurface>>, // Per-slot touch targets

    // --- Output Related State ---
    pub output_manager_state: OutputManagerState, // Smithay's manager for output Wayland objects
    pub output_manager: Arc<Mutex<OutputManager>>, // Our manager for OutputDevice instances

    // Other fields like output_manager will be added later (this was a note for other fields, output_manager is here)
}

impl DesktopState {
    pub fn new(
        display_handle: DisplayHandle,
        loop_handle: LoopHandle<'static, Self>,
        loop_signal: LoopSignal,
    ) -> Self {
        let seat_name = "seat0".to_string(); // Default seat name

        // Initialize SeatState for managing seat-related Wayland globals
        let mut seat_state = SeatState::new();

        // Create the primary compositor Seat object
        // The seat needs to be created via SeatState to correctly associate with Wayland globals.
        // However, Seat::new is also a valid way if not using SeatState for global creation directly.
        // Let's ensure consistency: Smithay examples often create Seat via SeatState.
        // SeatState::new_seat is not a method. Seat::new is typical.
        // The wl_seat global is created later via seat_state.new_wl_seat().
        let mut seat = Seat::new(&display_handle, seat_name.clone(), None); // Logger can be added
        let clock = Clock::new(None); // ADDED/UNCOMMENTED initialization, using None for span for now

        // Store SeatState in the seat's user data if SeatState itself needs to be accessed via Seat later.
        // Or, more commonly, SeatState is owned by DesktopState directly.
        // The SeatHandler methods get `&mut SeatState<Self>` via `self.seat_state`.
        // seat.user_data().insert_if_missing(|| seat_state.clone()); // SeatState is not Clone

        Self {
            display_handle: display_handle.clone(),
            loop_handle,
            loop_signal,
            clock, // ADDED/UNCOMMENTED
            compositor_state: CompositorState::new(),
            shm_state: ShmState::new(Vec::new(), None),
            xdg_shell_state: XdgShellState::new(&display_handle, PopupManager::new(None), None),
            xdg_activation_state: XdgActivationState::new(&display_handle, seat.clone(), None), // XdgActivationState needs the Seat
            space: Space::new(None),
            windows: HashMap::new(),
            shm_global: None,
            xdg_shell_global: None,
            xdg_activation_global: None,

            seat_state, // Initialize SeatState
            seat,       // Initialize Seat
            seat_name,  // Initialize seat_name
            keyboard_data_map: HashMap::new(), // Initialization is fine, type already changed
            current_cursor_status: Arc::new(Mutex::new(CursorImageStatus::Default)),
            pointer_location: Point::from((0.0, 0.0)),
            active_input_surface: None,
            // touch_focus_data: TouchFocusData::default(), // REMOVED
            active_touch_targets: HashMap::new(), // Initialize new field

            output_manager_state: OutputManagerState::new_with_xdg_output::<Self>(&display_handle), // Initialize with XDG support
            output_manager: Arc::new(Mutex::new(OutputManager::new())),
        }
    }
}

smithay::delegate_client_handler!(DesktopState);

// --- SeatHandler Implementation ---
impl SeatHandler for DesktopState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface; // Or WlSurface if touch focus is on a surface

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&Self::KeyboardFocus>) {
        tracing::debug!(
            seat_name = %seat.name(),
            new_focus_surface_id = ?focused.map(|s| s.id()),
            "SeatHandler::focus_changed (keyboard) triggered by Smithay."
        );

        // This method is called when Smithay's KeyboardHandle::set_focus changes the focus.
        // Its primary role here is to update any internal compositor state that depends on the keyboard focus.
        // For example, if we have a specific field in DesktopState to track the focused window's domain ID.

        let new_focused_wl_surface_weak = focused.map(|s| s.downgrade());
        self.active_input_surface = new_focused_wl_surface_weak;

        // Example of notifying a domain service (conceptual, actual call might differ)
        // if let Some(surface) = focused {
        //     if let Some(window) = self.space.window_for_surface(surface, WindowSurfaceType::TOPLEVEL) {
        //         let domain_id = window.user_data().get::<crate::compositor::xdg_shell::types::DomainWindowIdentifierWrapper>().map(|w| w.0.clone());
        //         if let Some(id) = domain_id {
        //             tracing::info!("Keyboard focus changed to domain window: {:?}", id);
        //             // self.workspace_manager_service.notify_keyboard_focus_changed(Some(id)); // Assuming async call handled elsewhere or made sync
        //         }
        //     }
        // } else {
        //     tracing::info!("Keyboard focus lost.");
        //     // self.workspace_manager_service.notify_keyboard_focus_changed(None);
        // }
        
        // The actual wl_keyboard.enter/leave events are sent by Smithay's KeyboardHandle.
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        // tracing::trace!(seat_name = %seat.name(), status = ?image, "SeatHandler::cursor_image request received.");
        // The cursor image status is stored in DesktopState and read by the renderer.
        // The renderer also needs the pointer_location from DesktopState.
        *self.current_cursor_status.lock().unwrap() = image;
        // No direct action needed here other than updating the state.
        // The renderer will pick this up in its drawing cycle.
    }
}
smithay::delegate_seat_handler!(DesktopState); // Ensures DesktopState delegates SeatHandler calls correctly

// --- OutputHandler Implementation ---
impl OutputHandler for DesktopState {
    fn output_state(&mut self) -> &mut OutputManagerState {
        &mut self.output_manager_state
    }

    fn new_output(&mut self, output: &wl_output::WlOutput, _output_data: &OutputData) {
        tracing::info!(output_name = %output.name(), output_description = %output.description(), "New wl_output global created by Smithay: {:?}", output.id());
        // Smithay's OutputManagerState handles the creation. We might log or perform
        // additional setup if needed, but usually, Smithay takes care of it.
        // The OutputDevice and its globals are managed via OutputManager::handle_hotplug_event
        // and output_device_created_notifications.
    }

    fn output_destroyed(&mut self, output: &wl_output::WlOutput, _output_data: &OutputData) {
        tracing::info!(output_name = %output.name(), "wl_output global destroyed by Smithay: {:?}", output.id());
        // Smithay handles the destruction. If we need to clean up anything in DesktopState
        // directly related to this specific wl_output global (not the OutputDevice itself,
        // which is handled by OutputManager), it would go here.
    }
}
smithay::delegate_output!(DesktopState); // Delegate wl_output handling to DesktopState

// --- GlobalDispatch for ZxdgOutputManagerV1 ---
// This ensures that clients can bind to the XDG Output Manager.
impl GlobalDispatch<ZxdgOutputManagerV1, ()> for DesktopState {
    fn bind(
        &mut self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<ZxdgOutputManagerV1>,
        _global_data: &(), // No specific global data for ZxdgOutputManagerV1 itself
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!("Client binding ZxdgOutputManagerV1");
        // Use Smithay's OutputManagerState to handle the binding and further dispatch.
        // This correctly sets up the ZxdgOutputManagerV1 resource to use Smithay's internal
        // dispatching logic, which will handle get_xdg_output requests based on existing
        // wl_output globals and their associated XDG output data.
        self.output_manager_state.xdg_output_manager_bind_dispatch(resource, data_init);
    }
}

// Smithay's OutputManagerState, when initialized with new_with_xdg_output,
// internally handles the Dispatch implementations for ZxdgOutputManagerV1 and ZxdgOutputV1.
// We just need to ensure the delegation macro for xdg_output is present.
smithay::delegate_xdg_output!(DesktopState); // Delegate zxdg_output_v1 and zxdg_output_manager_v1 requests


// Implement CompositorHandler for DesktopState
impl CompositorHandler for DesktopState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        // Smithay's CompositorHandler trait expects a CompositorClientState.
        // We can store this in the client's UserDataMap or use a dedicated field if ClientCompositorData grows.
        // For now, relying on Smithay's default implementation or storing it via UserData.
        // This part might need adjustment based on how ClientCompositorData is actually used.
        client
            .get_data::<CompositorClientState>()
            .unwrap() // Or handle error appropriately
    }

    fn commit(&mut self, surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface) {
        // Handle surface commit logic
        // This will involve accessing surface data, applying new state, damage tracking, etc.
        tracing::info!("CompositorHandler: Surface commit for {:?}", surface);
        // TODO: Implement detailed commit logic using surface_management functions
    }
}

// Implement GlobalDispatch for WlCompositor and WlSubcompositor (existing)
impl GlobalDispatch<wl_compositor::WlCompositor, ()> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: smithay::reexports::wayland_server::New<wl_compositor::WlCompositor>,
        _global_data: &(),
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, Self>,
    ) {
        tracing::info!("Binding WlCompositor");
        data_init.init(resource, ());
    }
}

impl GlobalDispatch<wl_subcompositor::WlSubcompositor, ()> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: smithay::reexports::wayland_server::New<wl_subcompositor::WlSubcompositor>,
        _global_data: &(),
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, Self>,
    ) {
        tracing::info!("Binding WlSubcompositor");
        data_init.init(resource, ());
    }
}


// Delegate Smithay traits (existing)
delegate_compositor!(DesktopState);
delegate_shm!(DesktopState);

// BufferHandler (existing)
impl BufferHandler for DesktopState {
    fn buffer_destroyed(
        &mut self,
        _buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) {
        tracing::info!("Buffer destroyed: {:?}", _buffer);
        // TODO: Notify renderer about texture invalidation.
    }
}

// ShmHandler (existing, but ensure it's correct for DesktopState)
impl ShmHandler for DesktopState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
    // shm_formats, shm_client_data can be added if needed
}

// XdgShellHandler (minimal for now, will be expanded in xdg_shell/handlers.rs)
// This is a preliminary stub. The full implementation will be in its own module.
impl XdgShellHandler for DesktopState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        tracing::info!("New toplevel surface created: {:?}", surface.wl_surface());
        // TODO: Full ManagedWindow creation and mapping will be in xdg_shell/handlers.rs
        // For now, just acknowledge to prevent client errors.
        let data = surface.wl_surface().get_data::<XdgToplevelSurfaceData>().unwrap(); // Smithay attaches this
        data.send_configure(); // Send initial configure
    }

    fn new_popup(&mut self, surface: PopupSurface, _client_data: &XdgWmBaseClientData) {
        tracing::info!("New popup surface created: {:?}", surface.wl_surface());
        // TODO: Full ManagedWindow (popup variant) creation and mapping.
        // For now, just acknowledge.
        let data = surface.wl_surface().get_data::<XdgPopupSurfaceData>().unwrap(); // Smithay attaches this
        data.send_configure(surface.get_parent_surface().unwrap()); // Example configure
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        tracing::info!("Toplevel surface destroyed: {:?}", surface.wl_surface());
        // TODO: Cleanup ManagedWindow from space and windows map.
    }

    fn popup_destroyed(&mut self, surface: PopupSurface) {
        tracing::info!("Popup surface destroyed: {:?}", surface.wl_surface());
        // TODO: Cleanup ManagedWindow from space and windows map.
    }

    fn ack_configure(&mut self, surface: smithay::reexports::wayland_server::protocol::wl_surface::WlSurface, configure: smithay::wayland::shell::xdg::XdgSurfaceConfigure) {
        tracing::info!("Client acknowledged configure for surface: {:?}", surface);
        let _ = configure; // Data from configure can be used if needed.
        // TODO: Update ManagedWindow state based on ack if necessary.
    }

    // ... other XdgShellHandler methods will be implemented in xdg_shell/handlers.rs
    // For now, these stubs prevent the program from crashing if these events occur.
    fn grab(&mut self, _surface: PopupSurface, _seat: Seat<Self>, _serial: Serial) {
        tracing::warn!("XdgShellHandler::grab not yet implemented.");
    }

}

// XdgActivationHandler (minimal for now)
impl smithay::wayland::shell::xdg::activation::XdgActivationHandler for DesktopState {
    fn activation_state(&mut self) -> &mut smithay::wayland::shell::xdg::activation::XdgActivationState {
        &mut self.xdg_activation_state
    }

    fn request_new_token(&mut self, _token_data: smithay::wayland::shell::xdg::activation::TokenData, client: &Client) { // Added client
        tracing::info!("Client {:?} requested new XDG activation token.", client.id());
        // Smithay's XdgActivationState handles token creation and storage.
        // We might add custom logic here if needed (e.g. logging, policy checks).
    }

    fn token_activated(&mut self, token: String, activated_surface_role: smithay::wayland::shell::xdg::activation::ActivatedSurfaceRole) { // Corrected params
        tracing::info!("XDG activation token {} activated for role {:?}.", token, activated_surface_role);
        // TODO: Handle window activation logic (e.g., bring window to front, focus).
        // This would involve finding the ManagedWindow associated with the surface
        // that corresponds to this token and role, then calling set_activated(true) on it,
        // and potentially focusing the seat on it.
    }
}

// ClientHandler implementation for DesktopState
impl WaylandClientHandler for DesktopState {
    fn client_created(&mut self, client_id: ClientId, client: Arc<Client>) {
        tracing::info!("New client connected: ID {:?}", client_id);

        // Create our custom ClientData
        let client_data_arc = Arc::new(ClientData::new());
        tracing::info!("Created ClientData with internal UUID: {}", client_data_arc.id);

        // Store our ClientData in the client's global UserDataMap
        client.user_data().insert_if_missing_threadsafe(|| client_data_arc.clone());

        // Initialize Smithay's per-client states and store them in our ClientData's UserDataMap.
        // Note: Smithay's states often expect to be stored directly in the wayland_server::Client's UserDataMap.
        // The plan implies storing them within *our* ClientData's UserDataMap. Let's adjust if Smithay's
        // `new_client` methods directly manipulate `client.user_data()` vs returning data for us to store.

        // CompositorState per-client data
        // Smithay's CompositorState::new_client takes `&Client` and stores CompositorClientState in its UserDataMap.
        // It does not return the state for us to store elsewhere.
        self.compositor_state.new_client(&client);
        // To verify, we could check if client.user_data().get::<CompositorClientState>().is_some() here.

        // XdgShellState per-client data
        // XdgShellState::new_client returns XdgWmBaseClientData, which we *are* supposed to store.
        let xdg_wm_base_client_data = self.xdg_shell_state.new_client(&self.display_handle, &client);
        client_data_arc.user_data_map.insert_if_missing_threadsafe(|| xdg_wm_base_client_data);
        tracing::debug!("Stored XdgWmBaseClientData in our ClientData for client {:?}", client_id);

        // ShmState per-client data (if any)
        // ShmState::new_client is similar to CompositorState, it manages its own client data.
        self.shm_state.new_client(&client);

        // TODO: Potentially other per-client states (e.g., for custom protocols).
        // Example: If we had ClientCompositorData as a separate struct we manage:
        // let our_compositor_data = ClientCompositorData::default();
        // client_data_arc.user_data_map.insert_if_missing_threadsafe(|| our_compositor_data);

        tracing::info!("Client {:?} fully initialized with its data maps.", client_id);
    }

    fn client_disconnected(&mut self, client_id: ClientId, client: Client) {
        tracing::info!("Client disconnected: ID {:?}", client_id);
        // Smithay's states (CompositorState, XdgShellState, ShmState) have `client_destroyed`
        // methods that are typically called by Display::dispatch_clients when a client disconnects.
        // These methods clean up their internal per-client data.
        // Our `ClientData` (and anything in its `user_data_map`) stored in `client.user_data()`
        // will be dropped automatically when the `Arc<Client>` is dropped, as Smithay
        // removes the client from its internal list.

        // If we needed to do explicit cleanup beyond what RAII provides for ClientData:
        if let Some(client_data_arc) = client.user_data().get::<Arc<ClientData>>() {
            tracing::info!("Cleaning up ClientData with internal UUID: {}", client_data_arc.id);
            // Any specific cleanup related to client_data_arc.user_data_map contents could go here,
            // but typically UserDataMap handles drops of its stored Arcs correctly.
        } else {
            tracing::warn!("Could not find our ClientData for disconnected client {:?}", client_id);
        }

        // Call Smithay's client_destroyed methods
        self.compositor_state.client_destroyed(&client_id);
        self.xdg_shell_state.client_destroyed(&client); // Note: XdgShellState takes &Client
        self.shm_state.client_destroyed(&client_id);
        self.xdg_activation_state.client_destroyed(&client_id);


        // Any other state cleanup related to this client.
        // E.g., explicitly unmapping/destroying windows owned by this client.
        // Smithay's shell integrations often handle this when the surface roles are destroyed.
        // We also have toplevel_destroyed / popup_destroyed in XdgShellHandler.
        // Let's ensure all windows owned by this client are cleaned up.
        let client_windows: Vec<_> = self.windows.iter()
            .filter(|(_, managed_window)| {
                managed_window.wl_surface().client().map_or(false, |c| c.id() == client_id)
            })
            .map(|(id, _)| *id)
            .collect();

        for window_domain_id in client_windows {
            tracing::info!("Performing cleanup for window {:?} due to client {:?} disconnect.", window_domain_id, client_id);
            if let Some(managed_window_arc) = self.windows.remove(&window_domain_id) {
                 if managed_window_arc.is_mapped_by_compositor() {
                    self.space.unmap_window(&managed_window_arc);
                 }
                 // Other cleanup related to ManagedWindow, like notifying domain layer.
                 tracing::info!("Removed and unmapped window {:?} for disconnected client.", window_domain_id);
            }
        }
        if !self.windows.is_empty() { // Damage only if necessary
            self.space.damage_all();
            self.loop_signal.wakeup();
        }
    }
}
