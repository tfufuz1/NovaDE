// This is novade-system/src/compositor/handlers.rs
// Implementations of Smithay's delegate traits and Dispatch/GlobalDispatch for Wayland protocols.

//! This module centralizes the implementation of various Smithay handler traits
//! (e.g., `CompositorHandler`, `SeatHandler`, `XdgShellHandler`, etc.) for the
//! main `DesktopState`. It uses Smithay's delegate macros where possible and
//! provides custom logic for NovaDE-specific behaviors or when more control
//! is needed than the delegates offer.

use smithay::{
    delegate_compositor, delegate_data_device, delegate_dmabuf, delegate_output,
    delegate_seat, delegate_shm, delegate_subcompositor, delegate_xdg_activation,
    delegate_xdg_decoration, delegate_xdg_shell, delegate_input_method_manager,
    delegate_text_input_manager, delegate_fractional_scale_manager, delegate_presentation_time,
    delegate_relative_pointer_manager, delegate_pointer_constraints, delegate_viewporter,
    delegate_primary_selection,
    delegate_wlr_layer_shell,
    delegate_idle_notifier,
    delegate_single_pixel_buffer_manager,

    reexports::{
        wayland_server::{
            DisplayHandle, Client, protocol::{wl_surface::{self, WlSurface}, wl_output::{self, WlOutput}, wl_seat::{self, WlSeat}, wl_buffer::WlBuffer},
            Dispatch, GlobalDispatch, New, Resource, ClientData, DataInit,
            backend::{GlobalId, ClientId, DisconnectReason},
        },
        wayland_protocols::{
            xdg::{
                shell::server::{xdg_toplevel::{self, XdgToplevel, State as XdgToplevelStateRead}, xdg_popup::{self, XdgPopup}, xdg_surface::{self, XdgSurface}, xdg_wm_base::{self, XdgWmBase}},
                decoration::zv1::server::zxdg_toplevel_decoration_v1::{self, Mode as XdgToplevelDecorationMode, ZxdgToplevelDecorationV1},
                activation::v1::server::xdg_activation_token_v1::{self, XdgActivationTokenV1},
            },
            wlr::layer_shell::v1::server::{zwlr_layer_shell_v1, zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1, Layer}},
            wp::presentation_time::server::wp_presentation_feedback,
            unstable::{
                foreign_toplevel_management::v1::server::{
                    zwlr_foreign_toplevel_manager_v1::{ZwlrForeignToplevelManagerV1, Request as ManagerRequest }, // Event as ManagerEvent (not used directly here)
                    zwlr_foreign_toplevel_handle_v1::{ZwlrForeignToplevelHandleV1, Request as HandleRequest, DoneData as HandleDoneData, State as HandleState}, // Event as HandleEvent (not used directly here)
                },
                 input_method::v2::server::{
                    zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
                    zwp_input_method_v2::{ZwpInputMethodV2},
                    zwp_input_method_keyboard_grab_v2::{ZwpInputMethodKeyboardGrabV2},
                    zwp_input_method_popup_surface_v2::{ZwpInputMethodPopupSurfaceV2},
                },
                text_input::v3::server::{
                    zwp_text_input_manager_v3::ZwpTextInputManagerV3,
                    zwp_text_input_v3::{ZwpTextInputV3, ContentHint, ContentPurpose},
                },
                 idle_notify::v1::server::zwp_idle_inhibitor_v1::{ZwpIdleInhibitorV1},
            },
        }
    },
    input::{Seat, SeatHandler, SeatGrab, SeatFocus, pointer::{PointerGrabStartData, CursorImageStatus, PointerMotionEventData, PointerButtonEventData, PointerAxisEventData, RelativeMotionEvent, PointerHandle}, keyboard::{KeyboardHandle, KeyboardKeyEventData, KeysymHandle as SmithayKeysymHandle, FilterResult as XkbFilterResult}, touch::{TouchTarget, TouchEventData, TouchHandle, TouchDownEventData, TouchUpEventData, TouchMotionEventData, TouchShapeEventData, TouchOrientationEventData, TouchCancelEventData, TouchFrameEventData, GrabStartData as TouchGrabStartData}},
    output::OutputHandler,
    utils::{Buffer, Physical, Point, Logical, Serial, Rectangle, Size, Transform, SERIAL_COUNTER, ClockId},
    wayland::{
        compositor::{CompositorClientState, CompositorHandler, CompositorState, TraversalAction, SurfaceAttributes as WlSurfaceAttributes, SubsurfaceCachedState, SurfaceData},
        data_device::{ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler, DataDeviceUserData, SelectionSource, SelectionTarget, OfferData, ServerDndGrabData},
        dmabuf::{DmabufData, DmabufHandler, DmabufState, DmabufGlobalData},
        output::{OutputManagerState, OutputData as SmithayOutputData, XdgOutputUserData},
        seat::{SeatState as SmithaySeatState, SeatGlobalData, SeatUserApi}, // Removed FilterResult as SeatFilterResult (already in smithay::input::keyboard)
        shell::{
            xdg::{XdgShellState, XdgShellHandler, XdgPopupSurfaceData, XdgToplevelSurfaceData, XdgSurfaceUserData, Configure, XdgWmBaseClientData, ToplevelSurface},
            wlr_layer::{WlrLayerShellState, WlrLayerShellHandler, LayerSurfaceData},
            xdg::decoration::{XdgDecorationHandler, XdgDecorationState},
        },
        shm::{ShmHandler, ShmState, ShmClientData},
        subcompositor::{SubcompositorHandler, SubcompositorState},
        xdg_activation::{XdgActivationHandler, XdgActivationState, XdgActivationTokenData, XdgActivationTokenSurfaceData},
        presentation::{PresentationHandler, PresentationState, PresentationFeedbackData, PresentationTimes, PresentationFeedbackFlags},
        fractional_scale::{FractionalScaleManagerState, FractionalScaleHandler, FractionalScaleManagerUserData, PreferredScale, Scale},
        viewporter::{ViewporterState, ViewporterHandler},
        single_pixel_buffer::{SinglePixelBufferState, SinglePixelBufferHandler},
        relative_pointer::{RelativePointerManagerState, RelativePointerManagerHandler},
        pointer_constraints::{PointerConstraintsState, PointerConstraintsHandler, PointerConstraintData, PointerConstraint, LockedPointerData, ConfinedPointerData},
        input_method::{InputMethodManagerState, InputMethodHandler, InputMethodKeyboardGrabCreator, InputMethodPopupSurfaceCreator, InputMethodSeatUserData},
        text_input::{TextInputManagerState, TextInputHandler, TextInputSeatUserData},
        idle_notify::{IdleNotifierState, IdleNotifierHandler, IdleNotifySeatUserData},
        primary_selection::{PrimarySelectionHandler, PrimarySelectionTarget, PrimarySelectionSource},
    },
    desktop::{Space, Window, PopupManager, WindowSurfaceType, layer_map_for_output},
};
use tracing::{info, warn, debug, error};

use crate::compositor::state::{DesktopState, ClientState as NovaClientState, NovaSeatState};
use crate::compositor::xdg_shell as xdg_shell_impl;
use crate::compositor::layer_shell as layer_shell_impl;
use crate::compositor::foreign_toplevel::{ForeignToplevelManagerClientData};


// --- Core Protocol Handlers (delegated) ---
delegate_compositor!(DesktopState);
delegate_subcompositor!(DesktopState);
delegate_shm!(DesktopState);
delegate_output!(DesktopState);
delegate_seat!(DesktopState);

// --- Data Device & Selection ---
delegate_data_device!(DesktopState);
impl SelectionHandler for DesktopState {
    type SelectionUserData = ();
    fn new_selection(&mut self, _type: smithay::wayland::selection::SelectionTarget, _source: Option<smithay::wayland::selection::SelectionSource>, seat: Seat<Self>) {
        info!(seat = %seat.name(), type = ?_type, "New selection offered");
    }
    fn selection_cleared(&mut self, _type: smithay::wayland::selection::SelectionTarget, seat: Seat<Self>) {
        info!(seat = %seat.name(), type = ?_type, "Selection cleared");
    }
}
delegate_primary_selection!(DesktopState);
impl smithay::wayland::primary_selection::PrimarySelectionHandler for DesktopState {
    type PrimarySelectionUserData = ();
     fn new_primary_selection(&mut self, _type: smithay::wayland::primary_selection::PrimarySelectionTarget, _source: Option<smithay::wayland::primary_selection::PrimarySelectionSource>, _seat: Seat<Self>) {}
     fn primary_selection_cleared(&mut self, _type: smithay::wayland::primary_selection::PrimarySelectionTarget, _seat: Seat<Self>) {}
}


// --- DMABUF Handler ---
delegate_dmabuf!(DesktopState);
impl DmabufHandler for DesktopState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }
    fn dmabuf_imported(&mut self, _global: &DmabufGlobalData, _dmabuf: DmabufData, client_id: ClientId) -> bool {
        debug!(?client_id, "DMABUF imported (default acceptance)");
        true // Allow all valid DMABUFs by default
    }
}

// --- Shell Protocol Handlers ---
delegate_xdg_shell!(DesktopState);
delegate_wlr_layer_shell!(DesktopState);


// --- XDG Decoration Handler ---
impl XdgDecorationHandler for DesktopState {
    fn xdg_decoration_state(&mut self) -> &mut XdgDecorationState {
        &mut self.xdg_decoration_state
    }
    fn new_decoration(&mut self, toplevel: ToplevelSurface, _decoration: ZxdgToplevelDecorationV1) {
        info!(surface = ?toplevel.wl_surface().id(), "XDG Toplevel Decoration object created.");
    }
    fn request_mode(&mut self, toplevel: ToplevelSurface, _decoration: ZxdgToplevelDecorationV1, client_preferred_mode: Option<XdgToplevelDecorationMode>) -> Option<XdgToplevelDecorationMode> {
        info!(surface = ?toplevel.wl_surface().id(), ?client_preferred_mode, "Client requested decoration mode.");
        // NovaDE Policy: Prefer Client-Side Decorations (CSD).
        let chosen_mode = XdgToplevelDecorationMode::ClientSide;
        info!(surface = ?toplevel.wl_surface().id(), "Compositor chose decoration mode: {:?}", chosen_mode);
        Some(chosen_mode)
    }
    fn destroy_decoration(&mut self, toplevel: ToplevelSurface, _decoration: ZxdgToplevelDecorationV1) {
        info!(surface = ?toplevel.wl_surface().id(), "XDG Toplevel Decoration object destroyed.");
    }
}
delegate_xdg_decoration!(DesktopState);


// --- XDG Activation Handler ---
impl XdgActivationHandler for DesktopState {
    fn activation_state(&mut self) -> &mut XdgActivationState {
        &mut self.xdg_activation_state
    }
    fn request_activate( &mut self, toplevel_surface_to_activate: WlSurface, token_data: XdgActivationTokenData, requesting_surface: Option<WlSurface>) {
        info!( activate_surface = ?toplevel_surface_to_activate.id(), token = %token_data.token_string, requesting_surface = ?requesting_surface.as_ref().map(|s| s.id()), "Client requested activation");
        if !toplevel_surface_to_activate.is_alive() {
            warn!("Activation requested for a dead surface: {:?}", toplevel_surface_to_activate.id());
            return;
        }
        let mut space = self.space.lock().unwrap();
        if let Some(window_to_activate) = space.elements().find(|w| w.wl_surface().as_ref() == Some(&toplevel_surface_to_activate)).cloned() {
            space.raise_element(&window_to_activate, true);
            let focus_target = window_to_activate.wl_surface().unwrap();
            let kbd = self.primary_seat.keyboard_handle().unwrap();
            kbd.set_focus(self, Some(focus_target.clone()), SERIAL_COUNTER.next_serial());

            if let Some(toplevel) = window_to_activate.toplevel() {
                let mut current_xdg_state = toplevel.current_state(); // Smithay's ToplevelState
                let mut needs_configure = false;
                if !current_xdg_state.activated {
                    current_xdg_state.activated = true;
                    needs_configure = true;
                }
                if !space.is_mapped(&window_to_activate) {
                    let (_output, location) = space.outputs_for_element(&window_to_activate).first().and_then(|o| space.output_geometry(o).map(|g| (o.clone(), g.loc))).unwrap_or_else(|| (space.outputs().next().cloned().unwrap(), (0,0).into()));
                    space.map_element(window_to_activate.clone(), location, true);
                    needs_configure = true;
                }
                if needs_configure {
                    let configure = Configure { surface: toplevel.xdg_surface().clone(), state: current_xdg_state, serial: SERIAL_COUNTER.next_serial(), };
                    toplevel.send_configure(configure);
                    info!("Activated window {:?} and sent configure.", focus_target.id());
                } else { info!("Window {:?} was already active and mapped appropriately.", focus_target.id()); }
            }
            // Notify foreign toplevel manager about state change (activation)
            drop(space); // Release lock before calling potentially re-entrant code
            self.foreign_toplevel_manager_state.lock().unwrap().window_state_changed(&window_to_activate);

        } else { warn!("Could not find window for surface {:?} to activate.", toplevel_surface_to_activate.id()); }
    }
    fn token_created(&mut self, _token: XdgActivationTokenV1, token_data: XdgActivationTokenData) {
        info!(token = %token_data.token_string, surface = ?token_data.surface.as_ref().map(|s|s.id()), "XDG Activation token created");
    }
    fn token_destroyed(&mut self, token_data: XdgActivationTokenData) {
        info!(token = %token_data.token_string, "XDG Activation token destroyed");
    }
}
delegate_xdg_activation!(DesktopState);

// --- Other Protocol Handlers (delegated or with custom logic) ---
delegate_presentation_time!(DesktopState);
delegate_single_pixel_buffer_manager!(DesktopState);
delegate_relative_pointer_manager!(DesktopState);
delegate_pointer_constraints!(DesktopState); // Will use PointerConstraintsHandler on DesktopState
delegate_input_method_manager!(DesktopState); // Will use InputMethod* traits on DesktopState
delegate_text_input_manager!(DesktopState); // Will use TextInputHandler on DesktopState


// --- Viewport Handler ---
impl ViewporterHandler for DesktopState {
    fn viewporter_state(&mut self) -> &mut ViewporterState {
        &mut self.viewporter_state
    }
    fn set_viewport_source(&mut self, surface: WlSurface, source_rect: Option<Rectangle<f64, Buffer>>) {
        info!(surface = ?surface.id(), ?source_rect, "Client set viewport source");
        let mut surface_attributes = surface.data::<SurfaceData>().unwrap().attributes.lock().unwrap();
        if surface_attributes.buffer_src_rect != source_rect {
            surface_attributes.buffer_src_rect = source_rect;
            drop(surface_attributes);
            if let Some(window) = self.space.lock().unwrap().window_for_surface(&surface).cloned() {
                for output in self.space.lock().unwrap().outputs_for_element(&window) { output.damage_whole(); }
            } else if let Some(layer_surface_data) = surface.data::<LayerSurfaceData>() {
                layer_surface_data.layer_surface().output().damage_whole();
            }
        }
    }
    fn set_viewport_destination(&mut self, surface: WlSurface, dest_size: Option<Size<i32, Buffer>>) {
        info!(surface = ?surface.id(), ?dest_size, "Client set viewport destination");
        let mut surface_attributes = surface.data::<SurfaceData>().unwrap().attributes.lock().unwrap();
        if surface_attributes.buffer_dest_size != dest_size {
            surface_attributes.buffer_dest_size = dest_size;
            drop(surface_attributes);
            if let Some(window) = self.space.lock().unwrap().window_for_surface(&surface).cloned() {
                for output in self.space.lock().unwrap().outputs_for_element(&window) { output.damage_whole(); }
            } else if let Some(layer_surface_data) = surface.data::<LayerSurfaceData>() {
                layer_surface_data.layer_surface().output().damage_whole();
            }
        }
    }
}
delegate_viewporter!(DesktopState);


// --- Fractional Scale Handler ---
impl FractionalScaleHandler for DesktopState {
    fn fractional_scale_state(&mut self) -> &mut FractionalScaleManagerState {
        &mut self.fractional_scale_manager_state
    }
    fn new_scale(&mut self, surface: WlSurface, new_scale_value: Option<u32>) {
        info!(surface = ?surface.id(), ?new_scale_value, "Client requested new fractional scale");
        let new_multiplier = new_scale_value.map(|s| (s, 120u32) );
        let mut surface_attributes = surface.data::<SurfaceData>().unwrap().attributes.lock().unwrap();
        if surface_attributes.fractional_scale_multiplier != new_multiplier {
            surface_attributes.fractional_scale_multiplier = new_multiplier;
            drop(surface_attributes);
            if let Some(window) = self.space.lock().unwrap().window_for_surface(&surface).cloned() {
                for output in self.space.lock().unwrap().outputs_for_element(&window) {
                    info!(output = %output.name(), surface = ?surface.id(), "Damaging output due to fractional scale change.");
                    output.damage_whole();
                }
            } else if let Some(layer_surface_data) = surface.data::<LayerSurfaceData>() {
                let layer_surface = layer_surface_data.layer_surface();
                let output = layer_surface.output();
                info!(output = %output.name(), surface = ?surface.id(), "Damaging output for layer surface due to fractional scale change.");
                output.damage_whole();
            }
        } else { info!(surface = ?surface.id(), "Requested fractional scale is the same as current. No change."); }
    }
}
delegate_fractional_scale_manager!(DesktopState);


// --- Idle Notifier Handler ---
impl IdleNotifierHandler for DesktopState {
    fn notifier_state(&mut self) -> &mut IdleNotifierState {
        &mut self.idle_notifier_state
    }
    fn new_inhibitor(&mut self, inhibitor: ZwpIdleInhibitorV1, surface: WlSurface) {
        info!(inhibitor = ?inhibitor.id(), surface = ?surface.id(), "New idle inhibitor created");
        self.record_user_activity(true); // Inhibitor creation is significant activity
    }
    fn inhibitor_destroyed(&mut self, inhibitor: ZwpIdleInhibitorV1, surface: WlSurface) {
        info!(inhibitor = ?inhibitor.id(), surface = ?surface.id(), "Idle inhibitor destroyed");
        self.record_user_activity(true); // Inhibitor destruction might change idle state, re-evaluate
    }
}
delegate_idle_notifier!(DesktopState);


// --- Foreign Toplevel Manager and Handle Dispatch ---
impl Dispatch<ZwlrForeignToplevelManagerV1, ForeignToplevelManagerClientData> for DesktopState {
    fn request( state: &mut Self, _client: &Client, manager_resource: &ZwlrForeignToplevelManagerV1, request: ManagerRequest, _data: &ForeignToplevelManagerClientData, _dhandle: &DisplayHandle, _data_init: &mut DataInit<'_, Self>, ) {
        match request {
            ManagerRequest::Stop => {
                info!("Foreign Toplevel Manager {:?} requested stop.", manager_resource.id());
                state.foreign_toplevel_manager_state.lock().unwrap().remove_manager(manager_resource);
            }
            ManagerRequest::Destroy => {
                info!("Foreign Toplevel Manager {:?} destroyed (by client).", manager_resource.id());
                 state.foreign_toplevel_manager_state.lock().unwrap().remove_manager(manager_resource);
            }
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }
    fn destroyed( state: &mut Self, _client_id: ClientId, manager_resource_id: GlobalId, _data: &ForeignToplevelManagerClientData, ) {
        info!("Foreign Toplevel Manager resource {:?} destroyed (by Smithay).", manager_resource_id);
        // Find the manager by its resource ID and remove it from tracking.
        // This requires manager_resource itself, not just its ID, to pass to remove_manager if it expects the resource.
        // Let's adjust ForeignToplevelManagerState::remove_manager to accept GlobalId or ensure it's robustly handled.
        // For now, assume remove_manager can take an ID or the list is filtered directly.
        state.foreign_toplevel_manager_state.lock().unwrap().managers.retain(|m| m.id() != manager_resource_id);
    }
}

impl Dispatch<ZwlrForeignToplevelHandleV1, Window> for DesktopState { // UserData is Window
    fn request( state: &mut Self, _client: &Client, handle_resource: &ZwlrForeignToplevelHandleV1, request: HandleRequest, window: &Window, _dhandle: &DisplayHandle, _data_init: &mut DataInit<'_, Self>,    ) {
        if !window.wl_surface().map_or(false, |s| s.is_alive()) || window.toplevel().is_none() {
            warn!("Request {:?} on foreign toplevel handle {:?} for a window that is dead or not a toplevel.", request, handle_resource.id());
            if handle_resource.is_alive() {
                handle_resource.closed();
                handle_resource.done(HandleDoneData{ serial: SERIAL_COUNTER.next_serial()});
            }
            return;
        };
        let xdg_toplevel = window.toplevel().unwrap();
        match request {
            HandleRequest::Activate(data) => {
                info!("Foreign Toplevel Handle {:?}: Activate request for seat {:?}", handle_resource.id(), data.seat.id());
                let mut space_guard = state.space.lock().unwrap();
                space_guard.raise_element(&window, true);
                if let Some(focus_target) = window.wl_surface() {
                    let kbd = state.primary_seat.keyboard_handle().unwrap();
                    kbd.set_focus(state, Some(focus_target.clone()), SERIAL_COUNTER.next_serial()); // Pass &mut DesktopState
                    let mut current_xdg_state = xdg_toplevel.current_state();
                    if !current_xdg_state.activated {
                        current_xdg_state.activated = true;
                         let configure = Configure { surface: xdg_toplevel.xdg_surface().clone(), state: current_xdg_state, serial: SERIAL_COUNTER.next_serial(), };
                        xdg_toplevel.send_configure(configure);
                    }
                }
            }
            HandleRequest::Close => {
                info!("Foreign Toplevel Handle {:?}: Close request.", handle_resource.id());
                xdg_toplevel.send_close();
            }
            HandleRequest::SetRectangle(data) => {
                info!("Foreign Toplevel Handle {:?}: SetRectangle (x:{}, y:{}, w:{}, h:{}) request - ignored by NovaDE policy.", handle_resource.id(), data.x, data.y, data.width, data.height);
            }
            HandleRequest::SetMaximized => {
                info!("Foreign Toplevel Handle {:?}: SetMaximized request.", handle_resource.id());
                xdg_shell_impl::handle_xdg_toplevel_set_maximized(state, &xdg_toplevel, true);
            }
            HandleRequest::UnsetMaximized => {
                info!("Foreign Toplevel Handle {:?}: UnsetMaximized request.", handle_resource.id());
                xdg_shell_impl::handle_xdg_toplevel_set_maximized(state, &xdg_toplevel, false);
            }
            HandleRequest::SetFullscreen(data) => {
                info!("Foreign Toplevel Handle {:?}: SetFullscreen request.", handle_resource.id());
                let output_resource = data.output.as_ref();
                xdg_shell_impl::handle_xdg_toplevel_set_fullscreen(state, &xdg_toplevel, true, output_resource);
            }
            HandleRequest::UnsetFullscreen => {
                info!("Foreign Toplevel Handle {:?}: UnsetFullscreen request.", handle_resource.id());
                xdg_shell_impl::handle_xdg_toplevel_set_fullscreen(state, &xdg_toplevel, false, None);
            }
            HandleRequest::SetMinimized => {
                info!("Foreign Toplevel Handle {:?}: SetMinimized request.", handle_resource.id());
                xdg_shell_impl::handle_xdg_toplevel_set_minimized(state, &xdg_toplevel);
            }
            HandleRequest::UnsetMinimized => {
                info!("Foreign Toplevel Handle {:?}: UnsetMinimized request (restore).", handle_resource.id());
                let mut space_guard = state.space.lock().unwrap();
                if !space_guard.is_mapped(&window) {
                    let last_pos = window.geometry().loc;
                    space_guard.map_element(window.clone(), last_pos, true);
                    info!("Restored (unminimized) window {:?}", window.wl_surface().unwrap().id());
                    let mut current_xdg_state = xdg_toplevel.current_state();
                    if !current_xdg_state.activated { // Also ensure it's marked as activated
                        current_xdg_state.activated = true;
                        let configure = Configure { surface: xdg_toplevel.xdg_surface().clone(), state: current_xdg_state, serial: SERIAL_COUNTER.next_serial(), };
                        xdg_toplevel.send_configure(configure);
                    }
                }
            }
            HandleRequest::Destroy => {
                info!("Foreign Toplevel Handle {:?} destroyed by client.", handle_resource.id());
                state.foreign_toplevel_manager_state.lock().unwrap().known_handles.remove(&handle_resource.id());
            }
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }
    fn destroyed( state: &mut Self, _client_id: ClientId, handle_resource_id: GlobalId, window_user_data: &Window, ) {
        info!("Foreign Toplevel Handle resource {:?} (for window {:?}) destroyed (by Smithay).", handle_resource_id, window_user_data.wl_surface().map(|s|s.id()));
        state.foreign_toplevel_manager_state.lock().unwrap().known_handles.remove(&handle_resource_id);
    }
}


// --- Text Input and Input Method Handlers ---
impl TextInputHandler for DesktopState {
    fn text_input_state(&mut self) -> &mut TextInputManagerState {
        &mut self.text_input_manager_state
    }
    fn new_text_input(&mut self, text_input: ZwpTextInputV3) {
        info!(text_input = ?text_input.id(), "New text input created");
    }
    fn text_input_focused(&mut self, text_input: ZwpTextInputV3, surface: WlSurface) {
        info!(text_input = ?text_input.id(), surface = ?surface.id(), "Text input focused");
    }
    fn text_input_unfocused(&mut self, text_input: ZwpTextInputV3, surface: WlSurface) {
        info!(text_input = ?text_input.id(), surface = ?surface.id(), "Text input unfocused");
    }
    fn set_surrounding_text(&mut self, text_input: ZwpTextInputV3, text: String, cursor: u32, anchor: u32) {
        debug!(text_input = ?text_input.id(), %text, cursor, anchor, "TI: Set surrounding text");
    }
    fn set_content_type(&mut self, text_input: ZwpTextInputV3, hint: ContentHint, purpose: ContentPurpose) {
        debug!(text_input = ?text_input.id(), ?hint, ?purpose, "TI: Set content type");
    }
    fn set_cursor_rectangle(&mut self, text_input: ZwpTextInputV3, x: i32, y: i32, width: i32, height: i32) {
        debug!(text_input = ?text_input.id(), x, y, width, height, "TI: Set cursor rectangle");
    }
    fn commit_state(&mut self, text_input: ZwpTextInputV3, serial: u32) {
        debug!(text_input = ?text_input.id(), serial, "TI: Commit state (ack from client)");
    }
    fn invoke_action(&mut self, text_input: ZwpTextInputV3, button: u32, index: u32) {
        debug!(text_input = ?text_input.id(), button, index, "TI: Invoke action");
    }
}

impl InputMethodHandler for DesktopState {
    fn input_method_state(&mut self) -> &mut InputMethodManagerState {
        &mut self.input_method_manager_state
    }
    fn new_input_method(&mut self, input_method: ZwpInputMethodV2, seat: Seat<Self>) {
        info!(input_method = ?input_method.id(), seat = %seat.name(), "New input method registered");
    }
    fn input_method_unavailable(&mut self, input_method: ZwpInputMethodV2) {
        info!(input_method = ?input_method.id(), "Input method unavailable");
    }
    fn commit(&mut self, input_method: ZwpInputMethodV2, commit_string: String) {
        debug!(input_method = ?input_method.id(), %commit_string, "IM: Commit string");
    }
    fn set_preedit(&mut self, input_method: ZwpInputMethodV2, preedit_string: String, cursor_begin: i32, cursor_end: i32) {
        debug!(input_method = ?input_method.id(), %preedit_string, cursor_begin, cursor_end, "IM: Set preedit");
    }
    fn delete_surrounding_text(&mut self, input_method: ZwpInputMethodV2, before_length: u32, after_length: u32) {
        debug!(input_method = ?input_method.id(), before_length, after_length, "IM: Delete surrounding text");
    }
    fn commit_content_type(&mut self, input_method: ZwpInputMethodV2) {
        debug!(input_method = ?input_method.id(), "IM: Commit content type (ack from IME)");
    }
}

impl InputMethodKeyboardGrabCreator for DesktopState {
    fn new_keyboard_grab(&mut self, seat: &Seat<Self>, grab_resource: New<ZwpInputMethodKeyboardGrabV2>) -> ZwpInputMethodKeyboardGrabV2 {
        info!(seat = %seat.name(), "IM: Creating new keyboard grab");
        let keyboard_handle = seat.keyboard_handle().expect("Seat should have a keyboard for IME grab");
        self.input_method_manager_state.new_keyboard_grab(grab_resource, keyboard_handle, &self.display_handle)
    }
}

impl InputMethodPopupSurfaceCreator for DesktopState {
     fn new_popup_surface(&mut self, surface_resource: New<ZwpInputMethodPopupSurfaceV2>, parent_input_method: &ZwpInputMethodV2) -> ZwpInputMethodPopupSurfaceV2 {
        info!(parent_im = ?parent_input_method.id(), "IM: Creating new popup surface");
        self.input_method_manager_state.new_popup_surface(surface_resource, parent_input_method, self.compositor_state(), &self.display_handle)
    }
}


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
        smithay::wayland::compositor::handlers::commit_handler::<DesktopState>(surface); // Smithay's main commit processing

        // Custom post-commit logic for different roles
        if let Some(window) = self.space.lock().unwrap().window_for_surface(surface).cloned() {
            if let Some(toplevel) = window.toplevel() {
                // Custom logic for XDG Toplevel commit, if any, beyond what XdgShellHandler does.
                // For example, if damage needs to be manually added based on its role.
            }
        } else if surface.data::<LayerSurfaceData>().is_some() {
            layer_shell_impl::handle_layer_surface_commit(self, surface);
        }

        // TODO RENDERER INTEGRATION for single_pixel_buffer_v1:
        // Check if attached WlBuffer is a single-pixel buffer and instruct renderer.
    }
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
        info!(seat = %seat.name(), focused_surface = ?target.map(|s| s.id()), "Seat keyboard focus changed");

        let text_input_state = self.text_input_manager_state();
        text_input_state.focus_changed(target, seat);

        let input_method_state = self.input_method_manager_state();
        input_method_state.focus_changed(target, seat);

        // Update foreign_toplevel state for *all* windows when focus changes.
        let space_guard = self.space.lock().unwrap();
        let windows_to_update: Vec<Window> = space_guard.elements()
            .filter_map(|e| e.as_window().cloned())
            .filter(|w| w.toplevel().is_some())
            .collect();
        drop(space_guard);

        for window in windows_to_update {
            self.foreign_toplevel_manager_state.lock().unwrap().window_state_changed(&window);
        }

        self.record_user_activity(true); // Focus change is significant activity
    }

    fn cursor_image(&mut self, seat: &Seat<Self>, image: smithay::input::pointer::CursorImageStatus) {
        debug!(?image, "Cursor image status changed for seat {:?}", seat.name());
        // TODO: Handle DND icons.
        *self.cursor_status.lock().unwrap() = image;
    }

    fn grab_initiated(&mut self, _seat: &Seat<Self>, _surface: &wl_surface::WlSurface, _grab: SeatGrab) {
        info!(surface = ?_surface.id(), grab = ?_grab, "Seat grab initiated");
        self.record_user_activity(true);
    }
    fn grab_ended(&mut self, _seat: &Seat<Self>) {
        info!("Seat grab ended");
        self.record_user_activity(true);
    }

    // Default event processing methods from Smithay's SeatHandler that call into SeatState
    // We override them to record user activity.
    fn pointer_motion(&mut self, seat: &Seat<Self>, surface: Option<&Self::PointerFocus>, event_data: &PointerMotionEventData) {
        self.record_user_activity(false);
        let seat_state = self.seat_state(); // Get &mut SmithaySeatState
        seat_state.pointer_motion(seat, surface, event_data, self); // Call Smithay's logic
    }

    fn pointer_button(&mut self, seat: &Seat<Self>, event_data: &PointerButtonEventData) {
        self.record_user_activity(true);
        let seat_state = self.seat_state();
        seat_state.pointer_button(seat, event_data, self);
    }

    fn pointer_axis(&mut self, seat: &Seat<Self>, event_data: &PointerAxisEventData) {
        self.record_user_activity(true);
        let seat_state = self.seat_state();
        seat_state.pointer_axis(seat, event_data, self);
    }

    fn touch_event(&mut self, seat: &Seat<Self>, event_data: &TouchEventData) {
        self.record_user_activity(true);
        let seat_state = self.seat_state();
        seat_state.touch_event(seat, event_data, self);
    }

    fn keyboard_key(&mut self, seat: &Seat<Self>, event_data: &smithay::input::keyboard::KeyboardKeyEventData) -> XkbFilterResult<smithay::input::keyboard::KeysymHandle<'_>> {
        self.record_user_activity(true);
        let seat_state = self.seat_state();
        seat_state.keyboard_key(seat, event_data, self)
    }

    fn send_relative_motion(&mut self, seat: &Seat<Self>, surface: &wl_surface::WlSurface, event: smithay::input::pointer::RelativeMotionEvent) {
        self.record_user_activity(false);
        self.relative_pointer_manager_state.send_relative_motion(seat, surface, event, self.clock.now());
        debug!(seat = %seat.name(), surface = ?surface.id(), "Sent relative motion event");
    }
}


// --- Pointer Constraints Handler ---
impl PointerConstraintsHandler for DesktopState {
    fn constraints_state(&mut self) -> &mut PointerConstraintsState {
        &mut self.pointer_constraints_state
    }
    fn new_constraint(&mut self, surface: WlSurface, data: PointerConstraintData) {
        info!(surface = ?surface.id(), constraint_type = ?data.constraint_type(), "New pointer constraint requested");
        match data.constraint_type() {
            smithay::wayland::pointer_constraints::ConstraintType::Locked => {
                info!(surface = ?surface.id(), "Pointer locked.");
                *self.cursor_status.lock().unwrap() = CursorImageStatus::Hidden;
            }
            smithay::wayland::pointer_constraints::ConstraintType::Confined => {
                info!(surface = ?surface.id(), "Pointer confined.");
            }
        }
        warn!("Pointer constraint active, but enforcement (clamping/hiding) needs full input path integration.");
    }
    fn constraint_destroyed(&mut self, surface: WlSurface, data: PointerConstraintData) {
        info!(surface = ?surface.id(), constraint_type = ?data.constraint_type(), "Pointer constraint destroyed");
        match data.constraint_type() {
            smithay::wayland::pointer_constraints::ConstraintType::Locked => {
                info!(surface = ?surface.id(), "Pointer unlocked. Restoring cursor to default (if no other constraints).");
                *self.cursor_status.lock().unwrap() = CursorImageStatus::Default;
            }
            smithay::wayland::pointer_constraints::ConstraintType::Confined => {
                info!(surface = ?surface.id(), "Pointer unconfined.");
            }
        }
        warn!("Pointer constraint lifted. Ensure cursor state and input processing revert correctly.");
    }
}

// XdgShellHandler (extending delegate for custom logic)
impl XdgShellHandler for DesktopState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }
    fn new_toplevel(&mut self, surface: xdg_toplevel::XdgToplevel) {
        info!("XdgShellHandler: New XDG Toplevel object created for surface: {:?}", surface.wl_surface().id());
        xdg_shell_impl::handle_new_xdg_toplevel(self, surface);
    }
    fn map_toplevel(&mut self, toplevel: XdgToplevel) -> Window {
        let wl_surface = toplevel.wl_surface().clone();
        info!("XdgShellHandler: Mapping XDG Toplevel: {:?}", wl_surface.id());
        let window = self.xdg_shell_state().map_toplevel(toplevel); // Let Smithay create the Window

        // Then, map it into our space
        let mut space_guard = self.space.lock().unwrap();
        space_guard.map_element(window.clone(), (0,0), true); // Example initial placement
        drop(space_guard); // Release lock before calling foreign_toplevel

        self.foreign_toplevel_manager_state.lock().unwrap().window_mapped(&window, &self.display_handle);
        self.foreign_toplevel_manager_state.lock().unwrap().window_state_changed(&window);
        window
    }
    fn toplevel_destroyed(&mut self, toplevel: XdgToplevel) {
        let wl_surface = toplevel.wl_surface().clone();
        info!("XdgShellHandler: XDG Toplevel destroyed: {:?}", wl_surface.id());
        // Find the window before it's removed by Smithay's default handler
        let window_to_notify = self.space.lock().unwrap().elements()
            .find(|w| w.wl_surface().as_ref() == Some(&wl_surface))
            .cloned();

        // Call Smithay's default destruction logic first
        self.xdg_shell_state().toplevel_destroyed(toplevel);

        // Then notify foreign toplevel if window was found
        if let Some(window) = window_to_notify {
            self.foreign_toplevel_manager_state.lock().unwrap().window_unmapped(&window);
        }
    }
    fn new_popup(&mut self, surface: xdg_popup::XdgPopup, _parent: Option<xdg_surface::XdgSurface>) {
        info!("New XDG Popup: {:?}", surface.wl_surface().id());
        xdg_shell_impl::handle_new_xdg_popup(self, surface);
    }
    fn grab(&mut self, surface: xdg_popup::XdgPopup, seat: wl_seat::WlSeat, serial: Serial) {
        info!("XDG Popup Grab: {:?}, seat: {:?}", surface.wl_surface().id(), seat.id());
        let seat_obj = Seat::from_resource(&seat).unwrap();
        xdg_shell_impl::handle_xdg_popup_grab(self, surface, &seat_obj, serial);
    }
    fn toplevel_request(&mut self, client: Client, surface: xdg_toplevel::XdgToplevel, request: xdg_toplevel::Request, serial: Serial) {
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
                let original_title = title.clone();
                xdg_shell_impl::handle_xdg_toplevel_set_title(self, &surface, original_title);
                self.xdg_shell_state().process_toplevel_request(client, surface, xdg_toplevel::Request::SetTitle{title}, serial, self).ok();
            }
            xdg_toplevel::Request::SetAppId { app_id } => {
                let original_app_id = app_id.clone();
                xdg_shell_impl::handle_xdg_toplevel_set_app_id(self, &surface, original_app_id);
                self.xdg_shell_state().process_toplevel_request(client, surface, xdg_toplevel::Request::SetAppId{app_id}, serial, self).ok();
            }
            _ => { // For other requests like SetFullscreen, SetMaximized, etc.
                // Let the xdg_shell_impl functions handle them first, then fall through to Smithay's default processing
                // This requires xdg_shell_impl to not consume the request or to re-create it if needed.
                // Alternatively, XdgShellHandler methods for these are called directly by the delegate.
                // The current structure where XdgShellHandler on DesktopState calls xdg_shell_impl is fine.
                self.xdg_shell_state().process_toplevel_request(client, surface, request, serial, self).unwrap_or_else(|err| {
                    warn!("Error processing XDG toplevel request via default handler: {}", err);
                });
            }
        }
    }
    fn ack_configure(&mut self, surface: wl_surface::WlSurface, configure: Configure) {
        info!("XDG Surface {:?} acked configure with serial {}", surface.id(), configure.serial);
        xdg_shell_impl::handle_xdg_toplevel_ack_configure(self, &surface, &configure); // Calls foreign_toplevel window_mapped and window_state_changed
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
    }
    fn output_destroyed(&mut self, output: wl_output::WlOutput) {
        info!("WlOutput global destroyed: {:?}", output.id());
    }
}

// Client Disconnect Logic
impl Dispatch<Client, NovaClientState> for DesktopState {
    fn event( &mut self, event: smithay::reexports::wayland_server::backend::ClientEvent, _source: Option<ClientId>, _data: &mut NovaClientState, ) {
        if let smithay::reexports::wayland_server::backend::ClientEvent::Disconnected(client_id, reason) = event {
            info!("Client disconnected: {:?}, reason: {:?}", client_id, reason);
            warn!("Client resource cleanup upon disconnect is not yet fully implemented.");
        }
    }
}
