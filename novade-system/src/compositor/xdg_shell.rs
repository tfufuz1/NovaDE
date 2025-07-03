// This is novade-system/src/compositor/xdg_shell.rs
// Specific logic for xdg-shell clients (xdg_wm_base, xdg_surface, xdg_toplevel).

//! Implements the handling logic for `xdg_shell` protocol requests.
//!
//! This module contains functions that are called by the `XdgShellHandler`
//! implementation on `DesktopState` (found in `handlers.rs`). These functions
//! manage the lifecycle and state changes of XDG surfaces, toplevels, and popups,
//! interacting with Smithay's window management primitives like `Space` and `PopupManager`.

use smithay::{
    desktop::{Window, Space, PopupManager, WindowSurfaceType, space::SpaceElement},
    input::Seat,
    reexports::wayland_server::{
        protocol::{wl_surface::WlSurface, wl_seat::WlSeat},
        DisplayHandle, Client, Resource,
    },
    utils::{Point, Logical, Serial, Rectangle, SERIAL_COUNTER},
    wayland::shell::xdg::{
        // XdgShellHandler, // Trait is implemented on DesktopState
        XdgShellState, XdgToplevelSurfaceData, XdgPopupSurfaceData,
        ToplevelState as SmithayToplevelState, Configure, XdgRequest, XdgWmBaseClientData,
        XdgPositionerUserData, XdgSurfaceUserData, XdgToplevelUserData, XdgPopupUserData,
        xdg_toplevel::{State as XdgToplevelStateRead, XdgToplevel}, // Renamed State to XdgToplevelStateRead
        xdg_popup::XdgPopup,
        xdg_surface::XdgSurface,
        XdgShellUserData,
    },
    delegate_xdg_shell,
};
use tracing::{info, warn, debug};
use crate::compositor::state::DesktopState;

/// Handles the creation of a new XDG surface.
/// Called when a client creates an `xdg_surface` from `xdg_wm_base`.
pub fn handle_new_xdg_surface(
    _state: &mut DesktopState, // State might be used for policies or logging in future
    surface: WlSurface,
    xdg_surface: XdgSurface,
) {
    info!(surface = ?surface.id(), xdg_surface = ?xdg_surface.id(), "New XDG surface created");
    // Smithay's XdgShellState typically handles associating XdgSurfaceUserData.
    // Initial configuration is usually sent when a role (toplevel, popup) is assigned.
}

/// Handles the creation of a new XDG toplevel window.
/// Called when an `xdg_surface` gets the `xdg_toplevel` role.
pub fn handle_new_xdg_toplevel(
    _state: &mut DesktopState, // DesktopState itself will manage the Window via XdgShellHandler trait
    surface: WlSurface,
    xdg_toplevel: XdgToplevel,
) {
    let xdg_surface_handle = xdg_toplevel.xdg_surface().clone();
    info!(surface = ?surface.id(), xdg_toplevel = ?xdg_toplevel.id(), "New XDG Toplevel role assigned");

    // Note: The actual creation of smithay::desktop::Window and mapping to Space
    // is handled by the XdgShellHandler::map_toplevel method on DesktopState,
    // which is called by Smithay when the toplevel is ready to be mapped.
    // This function's primary role now is to send the initial configure.

    let initial_configure_serial = SERIAL_COUNTER.next_serial();
    let mut configure_state = SmithayToplevelState::default();
    // TODO: Initial size and activation state should come from NovaDE window policy/config.
    configure_state.size = Some((800, 600).into());
    configure_state.activated = true; // New windows are typically activated by default.

    let configure = Configure {
        surface: xdg_surface_handle,
        state: configure_state,
        serial: initial_configure_serial,
    };
    xdg_toplevel.send_configure(configure);
    info!(surface = ?surface.id(), "Sent initial configure for new XDG Toplevel");
}

/// Handles the creation of a new XDG popup.
/// Called when an `xdg_surface` gets the `xdg_popup` role.
pub fn handle_new_xdg_popup(
    state: &mut DesktopState,
    surface: WlSurface,
    xdg_popup: XdgPopup,
) {
    info!(surface = ?surface.id(), xdg_popup = ?xdg_popup.id(), "New XDG Popup role assigned");

    // The XdgShellHandler::new_popup on DesktopState (delegated to XdgShellState)
    // will call PopupManager::track_popup.
    if let Err(err) = state.popups.lock().unwrap().track_popup(xdg_popup.clone()) {
        warn!(surface = ?surface.id(), "Failed to track popup via PopupManager: {}", err);
        // If tracking fails, the popup might not behave correctly.
        // Consider sending a protocol error or closing the popup surface.
        return;
    }
    // Smithay's PopupManager handles sending the initial configure for the popup
    // after it's committed and its position is determined.
}

/// Handles commit operations for generic XDG surfaces.
/// This is a general hook; role-specific commit logic is often more critical.
pub fn handle_xdg_surface_commit(
    _state: &mut DesktopState, // May be used for accessing space or popups if needed
    surface: &WlSurface,
    xdg_surface: &XdgSurface,
) {
    debug!(surface = ?surface.id(), "Commit received for XDG surface role: {:?}", xdg_surface.role());
    // Smithay's XdgShellState and the main wl_surface commit_handler (CompositorHandler::commit)
    // manage applying pending state and damage. Role-specific logic (like re-arranging
    // popups via PopupManager) is often triggered from there or within role-specific handlers.
}


/// Handles an `ack_configure` request from an XDG toplevel client.
pub fn handle_xdg_toplevel_ack_configure(
    state: &mut DesktopState,
    toplevel: &XdgToplevel,
    _configure_data: smithay::wayland::shell::xdg::AcknowledgeConfigure,
) {
    let surface = toplevel.xdg_surface().wl_surface();
    info!(surface = ?surface.id(), "XDG Toplevel acked configure");

    // Smithay's XdgShellHandler::ack_configure (on DesktopState) is called by the delegate.
    // That handler is responsible for processing AcknowledgeConfigure::new_state
    // and potentially reconfiguring the window or mapping it if it's the first ack.

    // We ensure foreign toplevels are notified of potential state changes or initial mapping.
    let space_guard = state.space.lock().unwrap();
    if let Some(window) = space_guard.elements().find(|w| w.wl_surface().as_ref() == Some(surface)).cloned() {
        // It's possible the window was just mapped by XdgShellState in response to this ack.
        if window.is_mapped() {
            // Ensure foreign_toplevel_manager knows it's mapped.
            // `window_mapped` is idempotent if called multiple times for the same window/manager.
            drop(space_guard); // Release lock before calling other state
            state.foreign_toplevel_manager_state.lock().unwrap().window_mapped(&window, &state.display_handle);
        } else {
            drop(space_guard);
        }
        state.foreign_toplevel_manager_state.lock().unwrap().window_state_changed(&window);
    }
}

/// Handles an `set_parent` request from an XDG toplevel client.
pub fn handle_xdg_toplevel_set_parent(_state: &mut DesktopState, toplevel: &XdgToplevel, parent: Option<&XdgToplevel>) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), parent = ?parent.map(|p| p.xdg_surface().wl_surface().id()), "XDG Toplevel set_parent request");
    // NovaDE currently does not implement specific logic for transient window relationships here.
    // Smithay's XdgShellState might store this parentage internally.
}

/// Handles a `set_title` request from an XDG toplevel client.
pub fn handle_xdg_toplevel_set_title(state: &mut DesktopState, toplevel: &XdgToplevel, title: String) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), title = %title, "XDG Toplevel set_title request");
    // Smithay's XdgShellState (via delegate) handles storing the title in XdgToplevelUserData.
    if let Some(window) = state.space.lock().unwrap().elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() {
        state.foreign_toplevel_manager_state.lock().unwrap().window_title_changed(&window, title);
    }
}

/// Handles a `set_app_id` request from an XDG toplevel client.
pub fn handle_xdg_toplevel_set_app_id(state: &mut DesktopState, toplevel: &XdgToplevel, app_id: String) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), app_id = %app_id, "XDG Toplevel set_app_id request");
    // Smithay's XdgShellState (via delegate) handles storing the app_id.
    if let Some(window) = state.space.lock().unwrap().elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() {
        state.foreign_toplevel_manager_state.lock().unwrap().window_appid_changed(&window, app_id);
    }
}

/// Handles a `set_fullscreen` request from an XDG toplevel client.
pub fn handle_xdg_toplevel_set_fullscreen(
    state: &mut DesktopState,
    toplevel: &XdgToplevel,
    fullscreen: bool,
    output_resource: Option<&smithay::reexports::wayland_server::protocol::wl_output::WlOutput>, // wl_output resource
) {
    let surface_id = toplevel.xdg_surface().wl_surface().id();
    info!(surface = ?surface_id, fullscreen, output = ?output_resource.as_ref().map(|o| o.id()), "XDG Toplevel set_fullscreen request");

    let Some(window) = state.space.lock().unwrap().elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() else {
        warn!(surface = ?surface_id, "Failed to find window in space for set_fullscreen request.");
        return;
    };

    let mut new_configure_state = window.toplevel().unwrap().current_state();

    if fullscreen {
        let space_guard = state.space.lock().unwrap(); // Lock space once for output iteration
        let target_output = output_resource
            .and_then(|o_res| space_guard.outputs().find(|s_out| s_out.owns(o_res)).cloned())
            .or_else(|| space_guard.outputs_for_element(&window).first().cloned())
            .or_else(|| space_guard.outputs().next().cloned());

        if let Some(output_handle) = target_output {
            let output_geometry = space_guard.output_geometry(&output_handle).unwrap_or_else(|| {
                warn!("Could not get geometry for output {:?}, using fallback fullscreen size.", output_handle.name());
                Rectangle::from_loc_and_size((0,0), (1920, 1080))
            });
            drop(space_guard); // Release lock before further space modifications

            // TODO: Store pre-fullscreen state (size, location) in Window UserData for proper restoration.
            new_configure_state.size = Some(output_geometry.size);
            new_configure_state.fullscreen = true;
            new_configure_state.maximized = false; // Fullscreen usually overrides maximized

            let mut space_modify_guard = state.space.lock().unwrap();
            space_modify_guard.map_element(window.clone(), output_geometry.loc, false); // map_element also configures size
            window.set_activated(true);
            space_modify_guard.raise_element(&window, true);
            drop(space_modify_guard);

            info!(surface = ?surface_id, "Configuring fullscreen on output {:?} with geometry {:?}", output_handle.name(), output_geometry);
        } else {
            drop(space_guard);
            warn!(surface = ?surface_id, "No output found to make window fullscreen. Ignoring request.");
            new_configure_state.fullscreen = false;
        }
    } else {
        // Un-fullscreen
        new_configure_state.fullscreen = false;
        // TODO: Restore pre-fullscreen size and position from Window UserData.
        // For now, if not maximized, it might keep its current size or client might suggest one.
        if !new_configure_state.maximized {
             new_configure_state.size = Some(window.geometry().size); // Example: keep current size
        }
        info!(surface = ?surface_id, "Disabling fullscreen.");
    }

    let configure = Configure {
        surface: toplevel.xdg_surface().clone(),
        state: new_configure_state.clone(),
        serial: SERIAL_COUNTER.next_serial(),
    };
    toplevel.send_configure(configure);
    state.foreign_toplevel_manager_state.lock().unwrap().window_state_changed(&window);
}

/// Handles a `set_maximized` request from an XDG toplevel client.
pub fn handle_xdg_toplevel_set_maximized(state: &mut DesktopState, toplevel: &XdgToplevel, maximized: bool) {
    let surface_id = toplevel.xdg_surface().wl_surface().id();
    info!(surface = ?surface_id, maximized, "XDG Toplevel set_maximized request");

    let Some(window) = state.space.lock().unwrap().elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() else {
        warn!(surface = ?surface_id, "Failed to find window in space for set_maximized request.");
        return;
    };

    let mut new_configure_state = window.toplevel().unwrap().current_state();

    if maximized {
        let space_guard = state.space.lock().unwrap();
        let target_output = space_guard.outputs_for_element(&window).first().cloned()
            .or_else(|| space_guard.outputs().next().cloned());

        if let Some(output_handle) = target_output {
            // TODO: Get available workspace area for the output, respecting panels and other layer shell surfaces.
            let output_geometry = space_guard.output_geometry(&output_handle).unwrap_or_else(|| {
                warn!("Could not get geometry for output {:?}, using fallback maximized size.", output_handle.name());
                Rectangle::from_loc_and_size((0,0), (1280, 720))
            });
            drop(space_guard);

            // TODO: Store pre-maximized state.
            new_configure_state.size = Some(output_geometry.size); // Maximize to full output/workspace area
            new_configure_state.maximized = true;
            new_configure_state.fullscreen = false;

            state.space.lock().unwrap().map_element(window.clone(), output_geometry.loc, false);
            info!(surface = ?surface_id, "Configuring maximized on output {:?} with geometry {:?}", output_handle.name(), output_geometry);
        } else {
            drop(space_guard);
            warn!(surface = ?surface_id, "No output found to maximize window on. Ignoring request.");
            new_configure_state.maximized = false;
        }
    } else {
        // Un-maximize
        new_configure_state.maximized = false;
        // TODO: Restore pre-maximized size and position.
        info!(surface = ?surface_id, "Disabling maximized.");
    }

    let configure = Configure {
        surface: toplevel.xdg_surface().clone(),
        state: new_configure_state.clone(),
        serial: SERIAL_COUNTER.next_serial(),
    };
    toplevel.send_configure(configure);
    state.foreign_toplevel_manager_state.lock().unwrap().window_state_changed(&window);
}

/// Handles a `set_minimized` request from an XDG toplevel client.
pub fn handle_xdg_toplevel_set_minimized(state: &mut DesktopState, toplevel: &XdgToplevel) {
    let surface_id = toplevel.xdg_surface().wl_surface().id();
    info!(surface = ?surface_id, "XDG Toplevel set_minimized request");

    let Some(window) = state.space.lock().unwrap().elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() else {
        warn!(surface = ?surface_id, "Failed to find window in space for set_minimized request.");
        return;
    };

    // Unmap the element from the space.
    state.space.lock().unwrap().unmap_elem(&window);
    info!(surface = ?surface_id, "Window minimized and unmapped.");
    // Notify foreign toplevels that the window is unmapped (effectively closed from their view or minimized).
    state.foreign_toplevel_manager_state.lock().unwrap().window_unmapped(&window);

    let mut current_xdg_state = window.toplevel().unwrap().current_state();
    current_xdg_state.activated = false; // Deactivate on minimize

    let configure = Configure {
        surface: toplevel.xdg_surface().clone(),
        state: current_xdg_state.clone(),
        serial: SERIAL_COUNTER.next_serial(),
    };
    toplevel.send_configure(configure);
    // Also notify state change (e.g. !activated)
    state.foreign_toplevel_manager_state.lock().unwrap().window_state_changed(&window);
}

/// Handles an interactive move request for an XDG toplevel.
pub fn handle_xdg_toplevel_move(state: &mut DesktopState, toplevel: &XdgToplevel, seat_resource: &WlSeat, _serial: Serial) {
    let surface_id = toplevel.xdg_surface().wl_surface().id();
    info!(surface = ?surface_id, "XDG Toplevel move request");

    let _smithay_seat = match Seat::from_resource(seat_resource) {
        Ok(s) => s,
        Err(_) => {
            warn!("Invalid seat resource in move request for toplevel {:?}", surface_id);
            return;
        }
    };
    if let Some(_window) = state.space.lock().unwrap().elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() {
        // TODO: Implement interactive move logic using Seat::start_pointer_grab
        // This involves creating a custom grab handler.
        warn!("Interactive move for toplevel {:?} not yet fully implemented.", surface_id);
    }
}

/// Handles an interactive resize request for an XDG toplevel.
pub fn handle_xdg_toplevel_resize(
    state: &mut DesktopState,
    toplevel: &XdgToplevel,
    seat_resource: &WlSeat,
    _serial: Serial,
    edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
) {
    let surface_id = toplevel.xdg_surface().wl_surface().id();
    info!(surface = ?surface_id, ?edges, "XDG Toplevel resize request");

    let _smithay_seat = match Seat::from_resource(seat_resource) {
        Ok(s) => s,
        Err(_) => {
            warn!("Invalid seat resource in resize request for toplevel {:?}", surface_id);
            return;
        }
    };
    if let Some(window) = state.space.lock().unwrap().elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() {
        // TODO: Implement interactive resize logic using Seat::start_pointer_grab
        warn!("Interactive resize for toplevel {:?} not yet fully implemented.", surface_id);

        let mut current_xdg_state = window.toplevel().unwrap().current_state();
        current_xdg_state.resizing = true;
        // The actual size will be updated during the grab.
        let configure = Configure {
            surface: toplevel.xdg_surface().clone(),
            state: current_xdg_state,
            serial: SERIAL_COUNTER.next_serial(),
        };
        toplevel.send_configure(configure);
        state.foreign_toplevel_manager_state.lock().unwrap().window_state_changed(&window);
    }
}

/// Handles a request to show the window menu for an XDG toplevel.
pub fn handle_xdg_toplevel_show_window_menu(
    _state: &mut DesktopState,
    toplevel: &XdgToplevel,
    _seat: &WlSeat,
    _serial: Serial,
    position: Point<i32, Logical>,
) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), ?position, "XDG Toplevel show_window_menu request");
    // NovaDE would need its own UI logic (e.g., a custom Layer Shell surface) to display a menu.
    warn!("Show window menu not implemented for toplevel {:?}", toplevel.xdg_surface().wl_surface().id());
}

/// Handles a grab request for an XDG popup.
pub fn handle_xdg_popup_grab(
    _state: &mut DesktopState, // Might need state to access PopupManager or Seat
    popup: &XdgPopup,
    _seat: &WlSeat, // The seat that initiated the grab
    _serial: Serial,
) {
    info!(surface = ?popup.xdg_surface().wl_surface().id(), "XDG Popup grab request");
    // This is complex. Smithay's PopupManager and XdgShellHandler::grab (on DesktopState)
    // typically handle the grab initiation and input routing.
    warn!("Popup grab for {:?} not yet fully implemented via this helper. Relies on XdgShellHandler/PopupManager.", popup.xdg_surface().wl_surface().id());
}

/// Handles a reposition request for an XDG popup.
pub fn handle_xdg_popup_reposition(
    _state: &mut DesktopState, // Might need state to access PopupManager
    popup: &XdgPopup,
    _positioner: smithay::wayland::shell::xdg::XdgPositionerUserData,
    _token: u32,
) {
    info!(surface = ?popup.xdg_surface().wl_surface().id(), ?_token, "XDG Popup reposition request");
    // Smithay's PopupManager should handle repositioning based on the new token.
    // The XdgShellHandler::reposition_request (on DesktopState) is the entry point.
    warn!("Popup reposition for {:?} not yet fully implemented via this helper. Relies on XdgShellHandler/PopupManager.", popup.xdg_surface().wl_surface().id());
}

// Note on window destruction for foreign_toplevel:
// When an XDG Toplevel is destroyed, XdgShellHandler::toplevel_destroyed on DesktopState
// (in handlers.rs) is called. This method *must* call
// state.foreign_toplevel_manager_state.lock().unwrap().window_unmapped(&window);
// to notify external clients. This is now included in the XdgShellHandler::toplevel_destroyed override.
