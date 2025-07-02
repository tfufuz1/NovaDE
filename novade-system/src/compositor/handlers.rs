// This is novade-system/src/compositor/handlers.rs
// Implementations of Smithay's delegate traits and Dispatch/GlobalDispatch for Wayland protocols.

use smithay::{
    delegate_compositor, delegate_data_device, delegate_dmabuf, delegate_output,
    delegate_seat, delegate_shm, delegate_subcompositor, delegate_xdg_activation,
    delegate_xdg_decoration, delegate_xdg_shell, delegate_input_method_manager,
    delegate_text_input_manager, delegate_fractional_scale_manager, delegate_presentation_time,
    delegate_relative_pointer_manager, delegate_pointer_constraints, delegate_viewporter,
    delegate_primary_selection, // If primary selection is used with data_device
    delegate_wlr_layer_shell, // Smithay 0.30 provides this
    delegate_idle_notifier,
    delegate_single_pixel_buffer_manager,
    // delegate_foreign_toplevel_manager, // Needs Smithay helper or manual implementation

    reexports::{
        wayland_server::{
            DisplayHandle, Client, protocol::{wl_surface, wl_output, wl_seat, wl_buffer},
            Dispatch, GlobalDispatch, New, Resource, ClientData,
            backend::{GlobalId, ClientId, DisconnectReason},
        },
        wayland_protocols::{ // For manual Dispatch/GlobalDispatch if needed
            xdg::shell::server::{xdg_toplevel, xdg_popup, xdg_surface, xdg_wm_base},
            wlr::layer_shell::v1::server::zwlr_layer_surface_v1,
            // Example: unstable::foreign_toplevel_management::v1::server::zwlr_foreign_toplevel_manager_v1,
        }
    },
    input::{Seat, SeatHandler, SeatGrab, SeatFocus, pointer::PointerGrabStartData, keyboard::KeyboardHandle},
    output::OutputHandler,
    wayland::{
        compositor::{CompositorClientState, CompositorHandler, CompositorState, TraversalAction, SurfaceAttributes as WlSurfaceAttributes, SubsurfaceCachedState},
        data_device::{ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler, DataDeviceUserData},
        dmabuf::{DmabufData, DmabufHandler, DmabufState, DmabufGlobalData},
        output::{OutputManagerState, OutputData, XdgOutputUserData},
        seat::{SeatState as SmithaySeatState, FilterResult as SeatFilterResult, SeatUserApi},
        shell::{
            xdg::{XdgShellState, XdgShellHandler, XdgPopupSurfaceData, XdgToplevelSurfaceData, XdgSurfaceUserData, Configure},
            wlr_layer::{WlrLayerShellState, WlrLayerShellHandler, LayerSurfaceData, Layer},
            xdg::decoration::{XdgDecorationHandler, XdgDecorationState},
        },
        shm::{ShmHandler, ShmState, ShmClientData},
        subcompositor::{SubcompositorHandler, SubcompositorState},
        xdg_activation::{XdgActivationHandler, XdgActivationState, XdgActivationTokenData, XdgActivationTokenSurfaceData},
        presentation::{PresentationHandler, PresentationState, PresentationFeedbackData},
        fractional_scale::{FractionalScaleManagerState, FractionalScaleHandler, FractionalScaleManagerUserData, PreferredScale, Scale},
        viewporter::{ViewporterState, ViewporterHandler},
        single_pixel_buffer::{SinglePixelBufferState, SinglePixelBufferHandler},
        relative_pointer::{RelativePointerManagerState, RelativePointerManagerHandler},
        pointer_constraints::{PointerConstraintsState, PointerConstraintsHandler, PointerConstraintData, PointerConstraint, LockedPointerData, ConfinedPointerData},
        input_method::{InputMethodManagerState, InputMethodHandler, InputMethodKeyboardGrabCreator, InputMethodPopupSurfaceCreator, InputMethodSeatUserData},
        text_input::{TextInputManagerState, TextInputHandler, TextInputSeatUserData},
        idle_notify::{IdleNotifierState, IdleNotifierHandler, IdleNotifySeatUserData},
        // foreign_toplevel::{ForeignToplevelManagerState, ForeignToplevelManagerHandler}, // Needs Smithay helper or manual
        selection::SelectionHandler, // For clipboard
    },
    desktop::{Space, Window, PopupManager, WindowSurfaceType, layer_map_for_output},
    utils::{Point, Logical, Serial, Rectangle, Size, Physical, Transform, SERIAL_COUNTER},
};
use tracing::{info, warn, debug, error};

use crate::compositor::state::{DesktopState, ClientState as NovaClientState, NovaSeatState}; // NovaClientState is our empty ClientData impl
use crate::compositor::xdg_shell as xdg_shell_impl; // For custom logic
use crate::compositor::layer_shell as layer_shell_impl; // For custom logic

// --- Core Protocol Handlers (delegated) ---
delegate_compositor!(DesktopState);
delegate_subcompositor!(DesktopState);
delegate_shm!(DesktopState);
delegate_output!(DesktopState); // Handles WlOutput and XdgOutput via OutputManagerState
delegate_seat!(DesktopState);   // Handles WlSeat and related input events

// --- Data Device & Selection (delegated, but might need custom SelectionHandler) ---
delegate_data_device!(DesktopState);
// DesktopState needs to implement SelectionHandler for clipboard/DND to work fully.
impl SelectionHandler for DesktopState {
    type SelectionUserData = (); // No specific user data for selection operations themselves for now
    fn new_selection(&mut self, _type: smithay::wayland::selection::SelectionTarget, _source: Option<smithay::wayland::selection::SelectionSource>, _seat: Seat<Self>) {
        // A new selection is available (e.g., clipboard updated)
        // This might involve notifying other parts of NovaDE or handling DND start.
        info!("New selection of type {:?} offered on seat {:?}", _type, _seat.name());
    }
    fn selection_cleared(&mut self, _type: smithay::wayland::selection::SelectionTarget, _seat: Seat<Self>) {
        // The current selection for this type was cleared (e.g., another client claimed it)
        info!("Selection of type {:?} cleared on seat {:?}", _type, _seat.name());
    }
}
// Primary selection is less common but part of data_device too
delegate_primary_selection!(DesktopState); // Requires PrimarySelectionHandler if used.
impl smithay::wayland::primary_selection::PrimarySelectionHandler for DesktopState {
    type PrimarySelectionUserData = ();
     fn new_primary_selection(&mut self, _type: smithay::wayland::primary_selection::PrimarySelectionTarget, _source: Option<smithay::wayland::primary_selection::PrimarySelectionSource>, _seat: Seat<Self>) {}
     fn primary_selection_cleared(&mut self, _type: smithay::wayland::primary_selection::PrimarySelectionTarget, _seat: Seat<Self>) {}
}


// --- DMABUF Handler (delegated) ---
delegate_dmabuf!(DesktopState);
// Note: DmabufGlobalData (formats, etc.) needs to be provided when creating the dmabuf global.
// DesktopState will also need to implement DmabufHandler for custom logic if any.
impl DmabufHandler for DesktopState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }
    // Allow all DMABUFs by default. Override if specific validation is needed.
    fn dmabuf_imported(&mut self, _global: &DmabufGlobalData, _dmabuf: DmabufData, _client_id: ClientId) -> bool {
        true
    }
}

// --- Shell Protocol Handlers (delegated) ---
delegate_xdg_shell!(DesktopState);
delegate_wlr_layer_shell!(DesktopState); // Smithay 0.30.0 provides this
delegate_xdg_decoration!(DesktopState);


// --- Other Protocol Handlers (delegated where possible) ---
delegate_xdg_activation!(DesktopState);
delegate_presentation_time!(DesktopState);
delegate_fractional_scale_manager!(DesktopState);
delegate_viewporter!(DesktopState);
delegate_single_pixel_buffer_manager!(DesktopState);
delegate_relative_pointer_manager!(DesktopState);
delegate_pointer_constraints!(DesktopState);
delegate_input_method_manager!(DesktopState);
delegate_text_input_manager!(DesktopState);
delegate_idle_notifier!(DesktopState);
// delegate_foreign_toplevel_manager!(DesktopState); // Needs smithay::wayland::foreign_toplevel module

// --- Manual Handler Implementations (where delegates are not enough or for more control) ---

// CompositorHandler implementation (extending the delegate)
impl CompositorHandler for DesktopState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        client.get_data::<CompositorClientState>().unwrap()
    }
    fn commit(&mut self, surface: &wl_surface::WlSurface) {
        smithay::wayland::compositor::handlers::commit_handler::<DesktopState>(surface);

        // Custom commit logic after Smithay's default handling
        // This is where you might check for roles and call role-specific commit handlers

        // Example: if it's an XDG Toplevel, update its state or damage
        if let Some(window) = self.space.lock().unwrap().window_for_surface(surface).cloned() {
            // Role-specific commit logic might have already been called by TraversalAction::call_handler
            // inside smithay's commit_handler.
            // If further action is needed specific to NovaDE after role processing:
            if let Some(_toplevel) = window.toplevel() {
                // Custom logic for XDG Toplevel commit
            }
        }
        // Similar for LayerSurfaces if custom commit logic beyond WlrLayerShellHandler is needed.
        if surface.data::<LayerSurfaceData>().is_some() {
            layer_shell_impl::handle_layer_surface_commit(self, surface);
        }


        // Damage tracking for rendering would also happen here or in a post-commit hook.
        // self.damage_surface(surface);
    }
    // Other CompositorHandler methods can be overridden if needed.
}


// SeatHandler implementation (extending the delegate)
impl SeatHandler for DesktopState {
    type KeyboardFocus = wl_surface::WlSurface;
    type PointerFocus = wl_surface::WlSurface;
    type TouchFocus = wl_surface::WlSurface;

    fn seat_state(&mut self) -> &mut SmithaySeatState<Self> {
        &mut self.seat_state.inner
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, target: Option<&Self::KeyboardFocus>, _window_id: Option<crate::window_management::WindowId>) {
        info!(seat = %seat.name(), focused = ?target.map(|s| s.id()), "Seat keyboard focus changed");
        // Propagate focus to text_input and input_method managers
        if let Some(text_input_state) = self.text_input_manager_state() {
             text_input_state.focus_changed(target, seat);
        }
        if let Some(input_method_state) = self.input_method_manager_state() {
             input_method_state.focus_changed(target, seat);
        }
        // TODO: Notify NovaDE window manager or other components about focus change.
        // TODO: Update window decorations (active/inactive state).
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: smithay::input::pointer::CursorImageStatus) {
        debug!(?image, "Cursor image status changed");
        *self.cursor_status.lock().unwrap() = image;
        // TODO: Schedule redraw for outputs where cursor is visible.
    }

    fn grab_initiated(&mut self, _seat: &Seat<Self>, surface: &wl_surface::WlSurface, grab: SeatGrab) {
        info!(surface = ?surface.id(), ?grab, "Seat grab initiated");
    }
    fn grab_ended(&mut self, _seat: &Seat<Self>) {
        info!("Seat grab ended");
    }
    fn send_relative_motion(&mut self, seat: &Seat<Self>, surface: &wl_surface::WlSurface, event: smithay::input::pointer::RelativeMotionEvent) {
        if let Some(relative_mgr_state) = self.relative_pointer_manager_state() {
            relative_mgr_state.send_relative_motion(seat, surface, event, self.clock.now());
        }
    }
}

// XdgShellHandler (extending delegate for custom logic)
impl XdgShellHandler for DesktopState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: xdg_toplevel::XdgToplevel) {
        info!("New XDG Toplevel: {:?}", surface.wl_surface().id());
        xdg_shell_impl::handle_new_xdg_toplevel(self, surface);
    }

    fn new_popup(&mut self, surface: xdg_popup::XdgPopup, _parent: Option<xdg_surface::XdgSurface>) {
        info!("New XDG Popup: {:?}", surface.wl_surface().id());
        xdg_shell_impl::handle_new_xdg_popup(self, surface);
    }

    fn grab(&mut self, surface: xdg_popup::XdgPopup, seat: wl_seat::WlSeat, serial: Serial) {
        info!("XDG Popup Grab: {:?}, seat: {:?}", surface.wl_surface().id(), seat.id());
        let seat_obj = Seat::from_resource(&seat).unwrap(); // Should always succeed if seat is valid
        xdg_shell_impl::handle_xdg_popup_grab(self, surface, &seat_obj, serial);
    }

    fn toplevel_request(&mut self, client: Client, surface: xdg_toplevel::XdgToplevel, request: xdg_toplevel::Request, serial: Serial) {
        let wl_surface_clone = surface.wl_surface().clone(); // Clone for potential use after move in request processing

        match request {
            xdg_toplevel::Request::Move { seat, .. } => {
                let seat_obj = Seat::from_resource(&seat).unwrap();
                xdg_shell_impl::handle_xdg_toplevel_move(self, &surface, &seat_obj, serial);
            }
            xdg_toplevel::Request::Resize { seat, edges, .. } => {
                let seat_obj = Seat::from_resource(&seat).unwrap();
                xdg_shell_impl::handle_xdg_toplevel_resize(self, &surface, &seat_obj, serial, edges);
            }
            xdg_toplevel::Request::SetTitle { title } => {
                let original_title = title.clone(); // Clone title for our handler
                xdg_shell_impl::handle_xdg_toplevel_set_title(self, &surface, original_title);
                // Fallthrough to default handler
                self.xdg_shell_state().process_toplevel_request(client, surface, xdg_toplevel::Request::SetTitle{title}, serial, self).ok();
            }
            xdg_toplevel::Request::SetAppId { app_id } => {
                let original_app_id = app_id.clone(); // Clone for our handler
                xdg_shell_impl::handle_xdg_toplevel_set_app_id(self, &surface, original_app_id);
                // Fallthrough to default handler
                self.xdg_shell_state().process_toplevel_request(client, surface, xdg_toplevel::Request::SetAppId{app_id}, serial, self).ok();
            }
            // Handle other specific requests similarly if custom logic + fallthrough is needed
            _ => {
                self.xdg_shell_state().process_toplevel_request(client, surface, request, serial, self).unwrap_or_else(|err| {
                    warn!("Error processing XDG toplevel request via default handler: {}", err);
                });
            }
        }
    }

    fn ack_configure(&mut self, surface: wl_surface::WlSurface, configure: Configure) {
        info!("XDG Surface {:?} acked configure with serial {}", surface.id(), configure.serial);
        xdg_shell_impl::handle_xdg_toplevel_ack_configure(self, &surface, &configure);
        // Let Smithay's XdgShellState handle the ack after custom logic
        self.xdg_shell_state.ack_configure(surface, configure, self).unwrap_or_else(|err| {
            warn!("Error processing XDG ack_configure via default handler: {}", err);
        });
    }
}


// WlrLayerShellHandler (extending delegate)
impl WlrLayerShellHandler for DesktopState {
    fn layer_shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(&mut self, surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, output: Option<wl_output::WlOutput>, layer: Layer, namespace: String) {
        info!("New Layer Surface: wl_surface: {:?}, output: {:?}, layer: {:?}, namespace: {}", surface.wl_surface().id(), output.as_ref().map(|o|o.id()), layer, namespace);
        layer_shell_impl::handle_new_layer_surface(self, surface, output, layer, namespace);
    }

    fn layer_surface_request(&mut self, client: Client, surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, request: zwlr_layer_surface_v1::Request, serial: Serial) {
        match request {
            zwlr_layer_surface_v1::Request::AckConfigure { serial: ack_serial } => {
                layer_shell_impl::handle_layer_ack_configure(self, &surface, ack_serial);
            }
            _ => {
                self.layer_shell_state().process_layer_surface_request(client, surface, request, serial, self).unwrap_or_else(|err| {
                    warn!("Error processing LayerSurface request via default handler: {}", err);
                });
            }
        }
    }
}


// OutputHandler (extending delegate)
impl OutputHandler for DesktopState {
    fn output_state(&mut self) -> &mut OutputManagerState {
        &mut self.output_manager_state
    }

    fn new_output(&mut self, output: wl_output::WlOutput) {
        info!("New WlOutput global created: {:?}", output.id());
        // Custom logic after Smithay's delegate handles OutputData and XdgOutputUserData.
    }

    fn output_destroyed(&mut self, output: wl_output::WlOutput) {
        info!("WlOutput global destroyed: {:?}", output.id());
        // Smithay's OutputManagerState handles cleanup related to XDG Output for this resource.
    }
}

// Client Disconnect Logic
impl Dispatch<Client, NovaClientState> for DesktopState {
    fn event(
        &mut self,
        event: smithay::reexports::wayland_server::backend::ClientEvent,
        _source: Option<ClientId>,
        _data: &mut NovaClientState,
    ) {
        if let smithay::reexports::wayland_server::backend::ClientEvent::Disconnected(client_id, reason) = event {
            info!("Client disconnected: {:?}, reason: {:?}", client_id, reason);
            // TODO: Implement comprehensive cleanup of resources associated with the disconnected client.
            warn!("Client resource cleanup upon disconnect is not yet fully implemented.");
        }
    }
}
