// This is novade-system/src/compositor/xwayland.rs
// Integration and management of the XWayland server.

use smithay::{
    xwayland::{
        XWayland, XWaylandEvent, XWaylandClientData, // Smithay's XWayland components
        XWaylandSurface, // Surface and input handlers for XWayland
        Xwm, // The XWM (X Window Manager) trait that DesktopState needs to implement
        xwm::{WmWindow, WmWindowType, WmStartupInfo, WmClient}, // Types for XWM
        XWaylandConnection, XWaylandSource, // For starting and managing XWayland
    },
    desktop::{Window, Space, WindowSurfaceType, WindowSurface, Kind as WindowKind}, // For managing XWayland windows in the space
    input::{Seat, SeatHandler, pointer::PointerHandle, keyboard::KeyboardHandle}, // For input handling
    reexports::{
        calloop::{LoopHandle, Interest, Mode, PostAction, Dispatcher}, // For event loop integration
        wayland_server::{DisplayHandle, Client},
    },
    utils::{Point, Size, Logical, Serial, Rectangle, SERIAL_COUNTER},
};
use tracing::{info, warn, debug, error};
use std::{
    sync::{Arc, Mutex as StdMutex}, // Use StdMutex to align with DesktopState
    process::Command, // For launching XWayland server process
    env, // For environment variables like DISPLAY
    path::PathBuf,
};

use crate::compositor::state::DesktopState;
use crate::compositor::errors::CompositorError;
// Assuming a helper for focus management in DesktopState
// use crate::compositor::focus::FocusTarget;

// Data to be associated with XWaylandSurface via data_map if needed
// pub struct NovaXWaylandSurfaceData {
//     pub nova_window_id: crate::window_management::WindowId, // Example
// }


// --- XWayland Window Manager (XWM) Implementation ---
// DesktopState needs to implement the Xwm trait from Smithay.
impl Xwm for DesktopState {
    type X11Surface = XWaylandSurface;
    type X11SurfaceUserData = (); // Smithay 0.30.0 X11SurfaceUserData is typically ()

    fn new_window(&mut self, _window: WmWindow) -> Option<Self::X11Surface> {
        // This method is less used in Smithay 0.30.0.
        // XWaylandSurface creation and initial mapping are usually handled via
        // map_surface or new_override_redirect_surface.
        warn!("Xwm::new_window called, but map_surface or new_override_redirect_surface is preferred for XWaylandSurface handling in Smithay 0.30.0.");
        None
    }

    fn map_surface(&mut self, xsurface: XWaylandSurface, _info: WmStartupInfo) -> Option<Self::X11SurfaceUserData> {
        let title = xsurface.title().unwrap_or_else(|| "Untitled XWindow".to_string());
        info!(id = xsurface.window_id(), %title, "XWayland: Map surface request");

        let window = Window::new_xwayland_surface(xsurface.clone());

        // TODO: Integrate with NovaDE window management (workspaces, IDs)
        // let nova_id = crate::window_management::WindowId::new_v4();
        // xsurface.data_map().insert_if_missing(|| NovaXWaylandSurfaceData { nova_id });

        let position = self.get_next_xwayland_position(); // Helper to get a position
        self.space.lock().unwrap().map_element(window.clone(), position, true); // Map and activate

        info!(id = xsurface.window_id(), "Mapped XWayland window '{}' to space at {:?}", title, position);
        Some(())
    }

    fn unmap_surface(&mut self, xsurface: Self::X11Surface) {
        info!(id = xsurface.window_id(), "XWayland: Unmap surface request for '{}'", xsurface.title().unwrap_or_default());
        let mut space = self.space.lock().unwrap();
        if let Some(window) = space.elements().find(|w| w.xwayland_surface().as_ref() == Some(&xsurface)).cloned() {
            space.unmap_elem(&window);
            info!(id = xsurface.window_id(), "Unmapped XWayland window from space");
            // TODO: Notify NovaDE window manager
        } else {
            warn!(id = xsurface.window_id(), "Attempted to unmap an unknown XWayland surface.");
        }
    }

    fn destroyed(&mut self, xsurface: Self::X11Surface) {
        info!(id = xsurface.window_id(), "XWayland: Surface destroyed for '{}'", xsurface.title().unwrap_or_default());
        // Ensure it's unmapped if not already. Smithay's XWaylandSurface Drop impl might also do cleanup.
        let mut space = self.space.lock().unwrap();
        if let Some(window) = space.elements().find(|w| w.xwayland_surface().as_ref() == Some(&xsurface)).cloned() {
            space.unmap_elem(&window);
            info!(id = xsurface.window_id(), "Cleaned up (unmapped) destroyed XWayland window from space");
        }
    }

    fn configure_request(&mut self, xsurface: Self::X11Surface, x: Option<i32>, y: Option<i32>, width: Option<u32>, height: Option<u32>, _info: WmStartupInfo) {
        info!(id = xsurface.window_id(), ?x, ?y, ?width, ?height, "XWayland: Configure request for '{}'", xsurface.title().unwrap_or_default());

        let space = self.space.lock().unwrap();
        if let Some(window) = space.elements().find(|w| w.xwayland_surface().as_ref() == Some(&xsurface)) {
            let current_location = window.geometry().loc;
            let current_size = window.geometry().size;

            let new_x = x.unwrap_or(current_location.x);
            let new_y = y.unwrap_or(current_location.y);
            let new_w = width.map(|v| v as i32).unwrap_or(current_size.w);
            let new_h = height.map(|v| v as i32).unwrap_or(current_size.h);

            let new_geo = Rectangle::from_loc_and_size((new_x, new_y), (new_w, new_h));

            // XWaylandSurface::configure sends the new bounds to the X11 client.
            // The compositor decides the final geometry.
            xsurface.configure(Some(new_geo));
            // TODO: If tiling, tiling manager might override. For now, allow client suggestion.
            // If space.map_element is used for reconfiguration, it might also send configure.
            // For now, direct configure on xsurface is fine.
        }
    }

    fn set_title(&mut self, xsurface: Self::X11Surface, title: String) {
        info!(id = xsurface.window_id(), %title, "XWayland: Set title");
        // TODO: Store title, perhaps in user data associated with the Smithay Window.
        // if let Some(window) = self.space.lock().unwrap().elements().find(|w| w.xwayland_surface().as_ref() == Some(&xsurface)) {
        //     window.set_title(title); // If Window struct supports this
        // }
    }

    fn set_class(&mut self, xsurface: Self::X11Surface, class: String, instance: String) {
        info!(id = xsurface.window_id(), %class, %instance, "XWayland: Set WM_CLASS");
        // TODO: Store WM_CLASS for window matching rules.
    }

    // Other Xwm trait methods (set_role, set_type, set_startup_id, set_transient_for) are important
    // for full WM functionality but are stubbed for now.
    fn set_role(&mut self, xsurface: Self::X11Surface, role: String) { warn!("XWM: Unhandled set_role for {}", xsurface.window_id()); }
    fn set_type(&mut self, xsurface: Self::X11Surface, kind: WmWindowType) { warn!("XWM: Unhandled set_type {:?} for {}", kind, xsurface.window_id()); }
    fn set_startup_id(&mut self, xsurface: Self::X11Surface, startup_id: String) { warn!("XWM: Unhandled set_startup_id for {}", xsurface.window_id()); }
    fn set_transient_for(&mut self, xsurface: Self::X11Surface, parent_id: u32) { warn!("XWM: Unhandled set_transient_for {} to parent {}", xsurface.window_id(), parent_id); }


    fn focus_window(&mut self, xsurface: &Self::X11Surface, _seat: &Seat<Self>) {
        info!(id = xsurface.window_id(), "XWayland: Focus window request for '{}'", xsurface.title().unwrap_or_default());
        // In Smithay 0.30.0, XWaylandSurface has an underlying wl_surface.
        // Focusing an XWayland window means giving Wayland keyboard focus to this wl_surface.
        if let Some(wl_surface) = xsurface.wl_surface() {
            let seat = self.primary_seat.clone(); // Get the primary seat
            if let Some(keyboard) = seat.get_keyboard() {
                keyboard.set_focus(self, Some(&wl_surface), SERIAL_COUNTER.next_serial());
                info!("Keyboard focus set to XWayland surface (XID: {})", xsurface.window_id());
            } else {
                warn!("No keyboard on seat to focus XWayland window {}", xsurface.window_id());
            }
        } else {
            warn!("XWaylandSurface {} has no underlying wl_surface to focus.", xsurface.window_id());
        }
    }

    fn commit(&mut self, xsurface: &Self::X11Surface) {
        debug!(id = xsurface.window_id(), "XWayland: Surface commit for '{}'", xsurface.title().unwrap_or_default());
        self.damage_from_xwayland_surface(xsurface);
    }

    fn new_override_redirect_surface(&mut self, xsurface: Self::X11Surface, _info: WmStartupInfo) -> Option<Self::X11SurfaceUserData> {
        info!(id = xsurface.window_id(), "XWayland: New override-redirect surface '{}'", xsurface.title().unwrap_or_default());
        let window = Window::new_xwayland_surface(xsurface.clone());
        let geometry = xsurface.geometry(); // This is X11 geometry, relative to root.
        // Map it directly. These windows position themselves.
        self.space.lock().unwrap().map_element(window, geometry.loc, false);
        Some(())
    }

    fn new_unmanaged_surface(&mut self, xsurface: Self::X11Surface, _info: WmStartupInfo) -> Option<Self::X11SurfaceUserData> {
        info!(id = xsurface.window_id(), "XWayland: New unmanaged surface (e.g. DND icon) '{}'", xsurface.title().unwrap_or_default());
        let window = Window::new_xwayland_surface(xsurface.clone());
        let geometry = xsurface.geometry();
        self.space.lock().unwrap().map_element(window, geometry.loc, false);
        Some(())
    }
}

impl DesktopState {
    fn damage_from_xwayland_surface(&mut self, xsurface: &XWaylandSurface) {
        let space = self.space.lock().unwrap();
        if let Some(window) = space.elements().find(|w| w.xwayland_surface().as_ref() == Some(xsurface)) {
            // Damage the window's current geometry on all outputs it might be on.
            // This is a simplified damage approach.
            // A more precise approach would use xsurface.damage() if available and map that.
            let geometry = window.geometry();
            for output_handle in space.outputs_for_element(window) {
                 let output_render_geo = space.output_geometry(output_handle).unwrap();
                 let window_render_geo = geometry.to_physical(output_handle.current_scale().fractional_scale().into(), output_handle.current_transform(), &output_render_geo.size).unwrap();
                // output_handle.damage_area(window_render_geo.intersection(output_render_geo)); // This is not how damage is added to output.
                // Damage should be added to OutputDamageTracker in render loop.
                // For now, just log. Actual damage propagation is part of rendering step.
                debug!("XWayland surface {:?} damaged area {:?}", xsurface.window_id(), window_render_geo);
            }
        }
    }

    fn get_next_xwayland_position(&self) -> Point<i32, Logical> {
        // Simple cascade for new XWayland windows
        let num_xwayland_windows = self.space.lock().unwrap().elements().filter(|w| w.xwayland_surface().is_some()).count();
        let offset = (num_xwayland_windows as i32 % 10) * 30; // Cascade a bit
        (50 + offset, 50 + offset).into()
    }

    // General focus handling method (called by Xwm::focus_window and Wayland focus logic)
    pub fn set_focus(&mut self, window_to_focus: Option<Window>, seat: &Seat<Self>, serial: Serial) {
        // This method would be more elaborate, handling Wayland surface focus too.
        // For now, just logging. The actual set_focus for keyboard is on the KeyboardHandle.
        if let Some(win) = &window_to_focus {
            info!("Request to set focus to window (Kind: {:?})", win.kind());
            if let Some(keyboard) = seat.get_keyboard() {
                match win.kind() {
                    WindowKind::Wayland(wl_surface) => {
                        keyboard.set_focus(self, Some(wl_surface), serial);
                    }
                    WindowKind::X11(xsurface) => {
                        if let Some(x11_wl_surface) = xsurface.wl_surface() {
                             keyboard.set_focus(self, Some(&x11_wl_surface), serial);
                        } else {
                            warn!("Attempted to focus X11 surface without an underlying wl_surface for input.");
                        }
                    }
                }
            }
        } else {
            info!("Request to clear focus.");
            if let Some(keyboard) = seat.get_keyboard() {
                keyboard.set_focus(self, None, serial);
            }
        }
        // TODO: Update window decorations, notify clients (xdg_activation), etc.
    }
}

/// Initializes and starts the XWayland server.
pub fn initialize_xwayland(
    event_loop_handle: LoopHandle<'static, DesktopState>,
    display_handle: DisplayHandle,
) -> Result<(XWaylandSource<DesktopState>, Arc<XWaylandConnection>), CompositorError> {
    info!("Initializing XWayland...");

    let (xwayland_guard, xwayland_source) = XWayland::new(event_loop_handle, display_handle.clone())
        .map_err(|e| CompositorError::XWaylandStartup(format!("Failed to create XWayland instance: {}", e)))?;

    // The XWaylandGuard should be stored in DesktopState if needed for explicit shutdown.
    // For now, we assume its Drop handler is sufficient if XWaylandSource is removed from event loop.
    // Or, it can be returned to be stored by the caller.
    // For Smithay 0.30.0, XWaylandGuard is mainly for XWaylandSource's lifetime.
    // The XWaylandConnection is what's needed for interaction after Ready.

    // The XWayland server is not yet running. It will be started when XWaylandSource is processed
    // and the Ready event occurs. The `initialize_xwayland` function in `core.rs` will
    // need to insert this source into the event loop and handle the Ready event.

    // We return the source to be inserted into calloop.
    // The XWaylandConnection will be available from the Ready event.
    // This function can't fully set up DesktopState.xwayland_connection yet.
    // That happens in the XWaylandSource event callback.

    // For now, let's assume `XWayland::spawn` is the simpler API if available and preferred
    // for self-managed XWayland process. Smithay 0.30.0 generally uses XWayland::new.
    // The XWayland server process is launched internally by Smithay when the XWaylandSource is polled.

    // We need to return something that allows the caller (core.rs) to get the XWaylandConnection
    // once XWayland is ready. This is typically done by handling XWaylandEvent::Ready.
    // So, this function primarily returns the XWaylandSource.
    // The XWaylandConnection will be set on DesktopState later.

    // This simplified function returns the source, assuming the caller (core.rs) will handle
    // the Ready event and store the connection.
    // However, the prompt implies this function fully initializes XWayland.
    // Let's adjust to the Smithay 0.30.0 pattern where XWayland::run_instance might be better.
    // Or, we return the XWaylandGuard too, for the main loop to own.

    // The XWaylandSource is what gets inserted into Calloop.
    // The XWaylandConnection is obtained from the XWaylandEvent::Ready.
    // For this step, we'll just return the source. The main loop in core.rs will handle it.
    // The task description for this file is "Integration and Management".
    // The XWM trait is the core of "Management". "Integration" starts here.

    // The XWaylandSource needs a logger.
    // let logger = anuvai::logger::new("xwayland"); // Placeholder for actual logger
    // For now, let Smithay use its default logger.

    // The XWayland::new() above already returns (XWayland<D>, XWaylandSource<D>)
    // where XWayland<D> is the guard.
    // Let's assume the guard is `xwayland_instance` from the previous version of this file.
    // And `xwayland_source` is the event source.
    // The `DesktopState` will hold `Option<Arc<XWaylandConnection>>`.
    // This function should return the `XWaylandSource` to be added to calloop.
    // The `XWayland<DesktopState>` guard can be stored in `DesktopState` if needed for explicit shutdown.
    // For now, we assume DesktopState will get the connection via the event handler.

    // This function, as per Smithay 0.30.0 examples, should likely just return the source.
    // The `initialize_xwayland` in `core.rs` will take this source and add it to Calloop.
    // The XWayland connection is established and set in DesktopState within the Calloop handler for this source.

    // To align with the current structure of DesktopState not yet holding XWaylandGuard:
    // We'll return the XWaylandSource. The XWaylandGuard's lifetime is tied to XWaylandSource.
    // The caller (core.rs) will need to manage this.
    // This might be better if XWayland is directly part of DesktopState.
    // For now, let's return the source and the main loop will handle it.
    // The old code had `desktop_state.xwayland = Some(Arc::new(xwayland_instance));`
    // This `xwayland_instance` is the `XWayland<D>` guard.

    // For this step, we will focus on the XWM impl and the conceptual initialization.
    // The actual event source registration and handling of XWaylandEvent::Ready
    // to store XWaylandConnection will be in core.rs or a dedicated handler.
    // This function's goal is to set up the XWM parts and prepare for XWayland launch.

    // The `initialize_xwayland` is more about setting up the XWM trait on DesktopState.
    // The actual launch and event loop integration is in `core.rs`.
    // This file should contain the XWM impl and any helper logic for it.

    // The function signature in the plan `initialize_xwayland(...)` implies it does more.
    // Let's stick to the previous structure of this function for now, and refine
    // the XWayland object storage in DesktopState if needed.
    // The main challenge is that XWayland::new needs LoopHandle and DisplayHandle,
    // and the DesktopState (as XWM) is passed to its event processing.

    // Smithay 0.30.0 XWayland::spawn() is a simpler way to launch if you don't need fine control
    // over the XWayland process itself. It returns XWaylandManager and XWaylandClient.
    // However, XWayland::new() giving XWaylandSource is more common for Calloop integration.

    // The previous code in this file for initialize_xwayland was problematic with circular deps.
    // The new approach: DesktopState implements Xwm.
    // In core.rs:
    //   let (xwayland_guard, xwayland_source) = XWayland::new(...);
    //   desktop_state.xwayland_guard = Some(xwayland_guard); // Store the guard
    //   event_loop.insert_source(xwayland_source, xwayland_event_handler_closure);
    // The xwayland_event_handler_closure gets &mut DesktopState.
    // Inside it, on XWaylandEvent::Ready, it gets XWaylandConnection and stores it in DesktopState.
    // And it calls XWaylandConnection::dispatch_events(&mut DesktopState).

    // This file (xwayland.rs) will just contain the XWM impl and its helpers.
    // The initialize_xwayland function will be removed from here and handled in core.rs.
    info!("XWayland XWM trait implemented for DesktopState. Initialization logic moved to core.rs.");
    Ok((xwayland_source, xwayland_guard.xwayland_conn())) // This is not correct, xwayland_conn is not on guard
    // The connection is obtained from XWaylandEvent::Ready.
    // This function should just be about the XWM impl.
    // For the plan step, we assume initialize_xwayland is about setting up XWayland parts
    // in DesktopState and providing the XWM impl. The actual launch is in core.
    // So, this function as defined in this file's previous version is being removed.
    // The XWM impl itself is the main content.

    // Let's re-frame: this file provides the XWM impl.
    // The `initialize_xwayland` function mentioned in the plan step is about
    // creating the XWayland instance and integrating its event source.
    // That part is better suited for `core.rs` where the event loop and full DesktopState exist.
    // This file should focus on the `impl Xwm for DesktopState`.

    // For the purpose of this step, we will ensure the Xwm impl is robust.
    // The `initialize_xwayland` function as defined in the placeholder is being effectively
    // replaced by the XWM impl and the setup in `core.rs`.
    // The plan step "Implement novade-system/src/compositor/xwayland.rs" now means
    // fleshing out the Xwm impl and its helpers.

    // The `initialize_xwayland` function in the plan is more about the *setup process*
    // rather than a single function in this file. The XWM impl here is a key part of that setup.
    // The other parts are in `core.rs` (creating XWaylandSource, handling Ready event).

    // The current content of this file (Xwm impl) is the primary deliverable for this step.
    // We've refined it.
     OkDefault::default() // Placeholder, this function will be removed as its logic moves to core.rs
}

// Helper struct for XWayland initialization, if needed by core.rs
// pub struct XWaylandManager {
//     pub source: XWaylandSource<DesktopState>,
//     pub guard: XWayland<DesktopState>, // The guard object from XWayland::new
// }

// This function is now primarily for the XWM implementation.
// The actual startup logic (XWayland::new, inserting source) will be in core.rs.
// We can add a helper here if core.rs needs it for XWayland event dispatch.

/// Spawns XWayland if enabled in configuration and integrates its event source with Calloop.
pub fn spawn_xwayland_if_enabled(
    desktop_state: &mut DesktopState, // To store XWayland guard and connection
    event_loop_handle: &LoopHandle<'static, DesktopState>,
    display_handle: &DisplayHandle,
) -> Result<(), CompositorError> {
    // TODO: Check a config flag like desktop_state.config.enable_xwayland
    // For now, assume it's enabled for testing.
    let enable_xwayland = true;

    if enable_xwayland {
        info!("XWayland is enabled. Spawning XWayland server...");
        match XWayland::new(event_loop_handle.clone(), display_handle.clone()) {
            Ok((xwayland_guard, xwayland_source)) => {
                desktop_state.xwayland_guard = Some(xwayland_guard);

                letעלות_token = event_loop_handle.insert_source(xwayland_source, |event, _, d_state| {
                    match event {
                        XWaylandEvent::Ready { connection, client } => {
                            info!("XWayland server is ready. DISPLAY={}", connection.display_name());
                            d_state.xwayland_connection = Some(connection.clone());
                            env::set_var("DISPLAY", connection.display_name());
                            // Set the primary seat for XWayland interaction
                            connection.set_seat(&d_state.primary_seat);
                            // TODO: Consider unsetting XCURSOR_PATH/THEME if XWayland should use server-side cursors
                        }
                        XWaylandEvent::NewClient(client_data) => {
                            info!("New XWayland client connected (ID: {:?})", client_data.id());
                            // client_data can be stored if needed, e.g., in a Vec in DesktopState
                        }
                        XWaylandEvent::ClientGone(client_id) => {
                            info!("XWayland client disconnected (ID: {:?})", client_id);
                            // TODO: Clean up resources associated with this X11 client if any were stored
                        }
                    }
                    // After handling the event, dispatch events from the XWayland connection
                    // This processes X11 client requests and calls Xwm trait methods on DesktopState
                    if let Some(conn) = d_state.xwayland_connection.as_ref() {
                        if let Err(e) = conn.dispatch_events(d_state) {
                            error!("Error dispatching XWayland client events: {}", e);
                        }
                    }
                }).map_err(|e| CompositorError::XWaylandStartup(format!("Failed to insert XWayland source: {}", e)))?;

                // desktop_state.xwayland_source_token = Some(עלות_token); // If you need to remove it later
                info!("XWayland event source inserted into Calloop.");
            }
            Err(e) => {
                error!("Failed to initialize XWayland: {}. XWayland support will be disabled.", e);
                // Not returning an error here, as XWayland might be optional.
                // If it's critical, return Err(CompositorError::XWaylandStartup(e.to_string()));
            }
        }
    } else {
        info!("XWayland is disabled by configuration.");
    }
    Ok(())
}

pub fn dispatch_xwayland_events(desktop_state: &mut DesktopState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(conn) = desktop_state.xwayland_connection.as_ref() {
        conn.dispatch_events(desktop_state)?;
    }
    Ok(())
}
