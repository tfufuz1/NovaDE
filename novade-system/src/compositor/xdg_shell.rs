// This is novade-system/src/compositor/xdg_shell.rs
// Specific logic for xdg-shell clients (xdg_wm_base, xdg_surface, xdg_toplevel).

use smithay::{
    desktop::{Window, Space, PopupManager, WindowSurfaceType},
    input::Seat,
    reexports::wayland_server::{
        protocol::{wl_surface::WlSurface, wl_seat::WlSeat},
        DisplayHandle, Client, Resource,
    },
    utils::{Point, Logical, Serial},
    wayland::shell::xdg::{
        XdgShellHandler, XdgShellState, XdgToplevelSurfaceData, XdgPopupSurfaceData,
        ToplevelState as SmithayToplevelState, Configure, XdgRequest, XdgWmBaseClientData,
        XdgPositionerUserData, XdgSurfaceUserData, XdgToplevelUserData, XdgPopupUserData,
        xdg_toplevel::{State as XdgToplevelState, XdgToplevel},
        xdg_popup::XdgPopup,
        xdg_surface::XdgSurface,
        XdgShellUserData, // Smithay 0.30 might have a general user data struct for the global
    },
    // Smithay 0.30 might use delegate_xdg_shell! macro for much of this.
    // If implementing manually, you'll need RequestHandler for each interface.
    delegate_xdg_shell, // For the macro
};

use tracing::{info, warn, debug};

use crate::compositor::state::DesktopState; // Assuming DesktopState is the main state struct

// User data structs that might be attached to Wayland objects
// Smithay 0.30.0 often uses specific UserData structs for its handlers.
// For example, XdgSurface might have XdgSurfaceUserData.

// This file will primarily contain the logic called by the XdgShellHandler methods
// implemented in handlers.rs (or via the delegate_xdg_shell! macro if DesktopState implements it).

pub fn handle_new_xdg_surface(
    state: &mut DesktopState,
    surface: WlSurface,
    xdg_surface: XdgSurface,
) {
    info!(surface = ?surface.id(), xdg_surface = ?xdg_surface.id(), "New XDG surface created");
    // Associate XdgSurfaceUserData if not already done by Smithay's internals
    // xdg_surface.data_map().insert_if_missing(|| XdgSurfaceUserData::default());

    // Initial configuration might be sent here or upon role assignment
}

pub fn handle_new_xdg_toplevel(
    state: &mut DesktopState,
    surface: WlSurface, // The underlying WlSurface
    xdg_toplevel: XdgToplevel, // The XdgToplevel object
) {
    let xdg_surface = xdg_toplevel.xdg_surface().clone(); // Get the parent XdgSurface
    info!(surface = ?surface.id(), xdg_toplevel = ?xdg_toplevel.id(), "New XDG Toplevel created");

    // Create a Smithay Window for this toplevel
    // Smithay 0.30.0 might create the Window inside its XdgShellHandler logic.
    // If you need to create it manually or augment it:
    let window = Window::new_xdg_toplevel(xdg_toplevel.clone()); // Smithay 0.30 `Window::new` might take XdgToplevel directly.

    // Store the window in DesktopState's Space or manage it as needed.
    // The DesktopState::new_toplevel handler (called via delegate) would typically do this.
    // state.space.map_element(window.clone(), (0, 0), true); // Example: map at (0,0) and activate

    // Attach user data for the toplevel if needed (e.g., for tracking NovaDE-specific state)
    // xdg_toplevel.data_map().insert_if_missing(|| MyXdgToplevelSpecificData::new(window));

    // Initial configure sequence
    let initial_configure_serial = SERIAL_COUNTER.next_serial();
    let mut configure_state = SmithayToplevelState::default();
    // Set initial size, maximized, activated, etc. as per policy
    configure_state.size = Some((800, 600).into()); // Example initial size
    configure_state.activated = true; // Typically new windows are activated

    let configure = Configure {
        surface: xdg_surface.clone(), // The XdgSurface associated with the toplevel
        state: configure_state,
        serial: initial_configure_serial,
    };
    xdg_toplevel.send_configure(configure); // Smithay 0.30.0 `XdgToplevel` has `send_configure`
                                          // or this might be part of a `Configure` struct passed to `send_configure`.
                                          // Check Smithay 0.30.0 `xdg_toplevel.rs` examples.
    info!(surface = ?surface.id(), "Sent initial configure for new XDG Toplevel");
}

pub fn handle_new_xdg_popup(
    state: &mut DesktopState,
    surface: WlSurface, // The underlying WlSurface
    xdg_popup: XdgPopup, // The XdgPopup object
) {
    let xdg_surface = xdg_popup.xdg_surface().clone(); // Get the parent XdgSurface
    info!(surface = ?surface.id(), xdg_popup = ?xdg_popup.id(), "New XDG Popup created");

    // Track the popup using PopupManager
    // The DesktopState::new_popup handler (called via delegate) would typically do this.
    if let Err(err) = state.popups.track_popup(xdg_popup.clone()) { // Smithay 0.30 PopupManager takes XdgPopup
        warn!(surface = ?surface.id(), "Failed to track popup: {}", err);
        // Consider destroying the popup or sending a configure_bounds_done if appropriate
        return;
    }

    // Initial configure for the popup
    // Popups are typically configured based on their positioner logic.
    // Smithay's PopupManager often handles sending the initial configure after the popup is committed.
    // xdg_popup.send_configure(...); // This is usually handled by PopupManager
    // xdg_popup.send_popup_done(); // After successful configuration and mapping
}

pub fn handle_xdg_surface_commit(
    state: &mut DesktopState,
    surface: &WlSurface,
    xdg_surface: &XdgSurface, // The XdgSurface that was committed
) {
    debug!(surface = ?surface.id(), "Commit received for XDG surface");

    // Handle surface commit, which might involve:
    // - Damage tracking
    // - Texture uploading
    // - If it's the first commit with a role, mapping the window/popup

    // For Toplevels:
    if let Some(toplevel) = xdg_surface.toplevel() {
        // Check if it's the first commit that makes the window ready to be mapped
        let is_mapped = state.space.elements().any(|w| w.wl_surface().as_ref() == Some(surface)); // Check if window is in space
        if !is_mapped && xdg_surface.has_buffer() {
            // This logic is often in XdgShellHandler::ack_configure or a similar place
            // where the window is mapped after the client acknowledges the first configure.
            // For now, we assume mapping happens after ack_configure.
            info!(surface = ?surface.id(), "XDG Toplevel ready to be mapped (has buffer). Waiting for ack_configure.");
        }
    }

    // For Popups:
    if let Some(popup) = xdg_surface.popup() {
        // PopupManager usually handles the logic for mapping popups after their first commit
        // and configure.
        // It might involve calling `popup.send_configure_bounds_done()` or similar.
        // The `PopupManager::commit_popup_surface` or similar internal Smithay logic handles this.
        // state.popups.commit_popup_surface(surface); // This is an internal Smithay call pattern
    }

    // Generic surface damage handling related to XDG roles would be managed by Smithay's
    // XdgShellState and the main wl_surface commit handler.
    // No direct call to a generic handle_surface_commit here, as that's too broad.
    // Role-specific commit logic is usually triggered from the main commit_handler in handlers.rs
    // or by Smithay's internal mechanisms when roles are processed.
}


// --- Handlers for XDG Toplevel/Popup Requests ---
// These functions would be called by the actual RequestHandler implementations
// (often managed by the delegate_xdg_shell! macro).

pub fn handle_xdg_toplevel_ack_configure(
    state: &mut DesktopState,
    toplevel: &XdgToplevel,
    configure_data: smithay::wayland::shell::xdg::AcknowledgeConfigure, // Smithay 0.30.0 specific type
) {
    let surface = toplevel.xdg_surface().wl_surface();
    info!(surface = ?surface.id(), "XDG Toplevel acked configure");

    // If this is the ack for the first configure, and the surface has a buffer, map it.
    // This logic is crucial for making windows appear.
    if let Some(window) = state.space.elements().find(|w| w.wl_surface().as_ref() == Some(surface)).cloned() {
        if !window.is_mapped() && toplevel.xdg_surface().has_buffer() {
            info!(surface = ?surface.id(), "Mapping XDG Toplevel after ack_configure.");
            // state.space.map_element(window, window.geometry().loc, false); // Or use window.map() if available
            // Smithay 0.30.0 `Window` has a `Window::map()` method.
            // The window should already be in the space from `new_toplevel`.
            // The `map()` call makes it visible.
            // The XdgShellHandler in Smithay usually calls window.map() internally after this.
            // If you are using the delegate, this is likely handled.
            // If implementing manually, you call `window.map()`.
        }
    }
    // Process a_c.new_state for geometry changes, etc.
    // If a_c.new_state is Some, it means the client suggests a new size/state.
    // The compositor can choose to accept, ignore, or modify this.
    // This is part of the interactive resize/move feedback loop.
}


pub fn handle_xdg_toplevel_set_parent(state: &mut DesktopState, toplevel: &XdgToplevel, parent: Option<&XdgToplevel>) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), parent = ?parent.map(|p| p.xdg_surface().wl_surface().id()), "XDG Toplevel set_parent request");
    // Update window hierarchy if your compositor supports it.
}

pub fn handle_xdg_toplevel_set_title(state: &mut DesktopState, toplevel: &XdgToplevel, title: String) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), title = %title, "XDG Toplevel set_title request");
    // Store the title, perhaps in user data associated with the window/toplevel.
    // This could be used by a taskbar or window switcher.
    if let Some(window) = state.space.elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())) {
        // window.set_title(title); // If Window struct has such a method
        // Or store in XdgToplevelUserData
    }
}

pub fn handle_xdg_toplevel_set_app_id(state: &mut DesktopState, toplevel: &XdgToplevel, app_id: String) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), app_id = %app_id, "XDG Toplevel set_app_id request");
    // Store the app_id. Used for identifying the application (theming, grouping, desktop files).
    if let Some(window) = state.space.elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())) {
        // window.set_app_id(app_id); // If Window struct has such a method
        // Or store in XdgToplevelUserData
    }
}

pub fn handle_xdg_toplevel_set_fullscreen(
    state: &mut DesktopState,
    toplevel: &XdgToplevel,
    fullscreen: bool,
    output: Option<&smithay::reexports::wayland_server::protocol::wl_output::WlOutput>,
) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), fullscreen, output = ?output.as_ref().map(|o| o.id()), "XDG Toplevel set_fullscreen request");

    if let Some(window) = state.space.elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() {
        let mut current_state = window.toplevel().unwrap().current_state(); // Assuming XdgToplevel is accessible from Window
        current_state.fullscreen = fullscreen;
        // If fullscreen is true, and output is Some, try to make it fullscreen on that output.
        // If output is None, compositor chooses an output.
        // If fullscreen is false, revert to previous state.

        // This involves:
        // 1. Updating the window's state (e.g., in its UserData or a field in your Window wrapper).
        // 2. Resizing the window to occupy the output's dimensions (if fullscreening).
        // 3. Sending a new configure to the client.
        // 4. Re-arranging other windows if necessary (e.g., unmaximizing others).

        // Example:
        // window.set_fullscreen(fullscreen); // Update your internal state
        // let target_geo = if fullscreen { /* calculate fullscreen geometry */ } else { /* calculate normal geometry */ };
        // state.space.map_element(window, target_geo.loc, false); // This also sets size via configure
        // toplevel.send_configure(...) // Send the new state to the client

        // Smithay's XdgToplevel might have methods to directly set these states,
        // which then handle sending the configure.
        // e.g., toplevel.set_fullscreen(fullscreen);
        // toplevel.send_pending_configure(); // Or similar if changes are batched.
        // Check Smithay 0.30.0 XdgToplevel API.
        let configure = Configure {
            surface: toplevel.xdg_surface().clone(),
            state: current_state,
            serial: SERIAL_COUNTER.next_serial(),
        };
        toplevel.send_configure(configure);
    }
}

pub fn handle_xdg_toplevel_set_maximized(state: &mut DesktopState, toplevel: &XdgToplevel, maximized: bool) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), maximized, "XDG Toplevel set_maximized request");
    // Similar logic to set_fullscreen:
    // Update internal state, calculate new geometry, send configure.
    if let Some(window) = state.space.elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() {
        let mut current_state = window.toplevel().unwrap().current_state();
        current_state.maximized = maximized;

        // window.set_maximized(maximized);
        // toplevel.set_maximized(maximized); // If Smithay API supports this directly
        // toplevel.send_pending_configure();
        let configure = Configure {
            surface: toplevel.xdg_surface().clone(),
            state: current_state,
            serial: SERIAL_COUNTER.next_serial(),
        };
        toplevel.send_configure(configure);
    }
}

pub fn handle_xdg_toplevel_set_minimized(state: &mut DesktopState, toplevel: &XdgToplevel) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), "XDG Toplevel set_minimized request");
    // Minimizing typically involves:
    // 1. Unmapping the window from the screen (state.space.unmap_elem(&window)).
    // 2. Storing its state so it can be restored.
    // 3. Notifying the client (though xdg_toplevel doesn't have a "minimized" state to send in configure).
    //    Minimization is mostly a compositor-side concept reflected by the window not being visible.
    if let Some(window) = state.space.elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() {
        // window.set_minimized(true); // Update your internal state
        state.space.unmap_elem(&window);
        info!(surface = ?toplevel.xdg_surface().wl_surface().id(), "Window minimized and unmapped.");
        // Optionally, send a configure with activated = false if it was active
        let mut current_state = window.toplevel().unwrap().current_state();
        current_state.activated = false; // Deactivate on minimize
        let configure = Configure {
            surface: toplevel.xdg_surface().clone(),
            state: current_state,
            serial: SERIAL_COUNTER.next_serial(),
        };
        toplevel.send_configure(configure);
    }
}

pub fn handle_xdg_toplevel_move(state: &mut DesktopState, toplevel: &XdgToplevel, seat: &WlSeat, serial: Serial) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), "XDG Toplevel move request");
    // Initiate an interactive move operation.
    // This usually involves:
    // 1. Getting the Smithay Seat from WlSeat.
    // 2. Starting a pointer grab if not already active (e.g., implicit grab from button press).
    // 3. Storing the window being moved and the initial pointer position.
    // 4. On subsequent pointer motion events, updating the window's position.
    // 5. On pointer button release, finalizing the move.
    // Smithay's Seat::start_grab might be used, and the grab handler updates window position.
    // DesktopState would need to store which window is currently being interactively moved/resized.
    let smithay_seat = Seat::from_resource(seat).unwrap(); // Get Smithay Seat
    if let Some(window) = state.space.elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() {
        // state.start_interactive_move(window, smithay_seat, serial); // Your custom logic
        warn!("Interactive move not yet fully implemented.");
    }
}

pub fn handle_xdg_toplevel_resize(
    state: &mut DesktopState,
    toplevel: &XdgToplevel,
    seat: &WlSeat,
    serial: Serial,
    edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), ?edges, "XDG Toplevel resize request");
    // Initiate an interactive resize operation. Similar to move.
    // The `edges` indicate which edges/corners are being dragged.
    let smithay_seat = Seat::from_resource(seat).unwrap();
    if let Some(window) = state.space.elements().find(|w| w.wl_surface().as_ref() == Some(toplevel.xdg_surface().wl_surface())).cloned() {
        // state.start_interactive_resize(window, smithay_seat, serial, edges); // Your custom logic
        warn!("Interactive resize not yet fully implemented.");

        // During interactive resize, the compositor sends configure events with the new size.
        // The client should redraw and ack_configure.
        // The toplevel state's `resizing` flag should be true.
        let mut current_state = window.toplevel().unwrap().current_state();
        current_state.resizing = true; // Indicate resizing is in progress
        // current_state.size = Some(new_size); // This would be set during the grab
        let configure = Configure {
            surface: toplevel.xdg_surface().clone(),
            state: current_state,
            serial: SERIAL_COUNTER.next_serial(), // A new serial for this configure
        };
        toplevel.send_configure(configure);
    }
}

pub fn handle_xdg_toplevel_show_window_menu(
    state: &mut DesktopState,
    toplevel: &XdgToplevel,
    seat: &WlSeat,
    serial: Serial,
    position: Point<i32, Logical>,
) {
    info!(surface = ?toplevel.xdg_surface().wl_surface().id(), ?position, "XDG Toplevel show_window_menu request");
    // Display a window menu (e.g., right-click context menu) at the given position.
    // This is highly compositor-specific.
    warn!("Show window menu not implemented.");
}


// --- Handlers for XDG Popup Requests ---

pub fn handle_xdg_popup_grab(
    state: &mut DesktopState,
    popup: &XdgPopup,
    seat: &WlSeat,
    serial: Serial,
) {
    info!(surface = ?popup.xdg_surface().wl_surface().id(), "XDG Popup grab request");
    // Handle popup grabs. This is complex and involves managing input focus related to the popup.
    // Smithay's PopupManager and Seat::start_grab might be involved.
    // Often, a grab on a popup means subsequent pointer events are confined to it or its parent hierarchy.
    // The `XdgShellHandler::grab` or `PopupManagerHandler::grab_xdg_popup` in Smithay 0.30 handles this.
    warn!("Popup grab not yet fully implemented via this handler. Check PopupManagerHandler.");
}

pub fn handle_xdg_popup_reposition(
    state: &mut DesktopState,
    popup: &XdgPopup,
    positioner: smithay::wayland::shell::xdg::XdgPositionerUserData, // Smithay 0.30 type
    token: u32,
) {
    info!(surface = ?popup.xdg_surface().wl_surface().id(), ?token, "XDG Popup reposition request");
    // Reposition the popup based on the new positioner rules.
    // Smithay's PopupManager likely has a method for this.
    // state.popups.reposition_popup(popup.xdg_surface(), positioner_resource, token);
    // The XdgShellHandler::reposition_request in Smithay 0.30 handles this.
    warn!("Popup reposition not yet fully implemented via this handler. Check XdgShellHandler.");
}


// This is a helper, actual commit handling for wl_surface is usually in handlers.rs
mod compositor {
    pub mod handlers {
        use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
        use crate::compositor::state::DesktopState;
        pub fn handle_surface_commit(_state: &mut DesktopState, _surface: &WlSurface) {
            // actual implementation would live in handlers.rs
        }
    }
}

// If not using the delegate_xdg_shell! macro, you would need to implement
// wayland_server::GlobalHandler for XdgWmBase and wayland_server::RequestHandler
// for XdgWmBase, XdgSurface, XdgToplevel, XdgPopup, XdgPositioner.
// These implementations would then call the `handle_...` functions defined above.
// Smithay 0.30.0 provides XdgShellState which handles much of this when you
// call its ::new_global method and then delegate to it.
// The functions above are conceptual helpers for what those delegates would do.
// Ensure that DesktopState correctly implements smithay::wayland::shell::xdg::XdgShellHandler
// (usually via the delegate_xdg_shell!(DesktopState); macro in state.rs or core.rs).
// The methods of that trait will be the entry points called by Smithay.
// Example:
// impl XdgShellHandler for DesktopState {
//     fn xdg_shell_state(&mut self) -> &mut XdgShellState { &mut self.xdg_shell_state }
//
//     fn new_toplevel(&mut self, surface: &WlSurface, xdg_toplevel: XdgToplevel) {
//         handle_new_xdg_toplevel(self, surface.clone(), xdg_toplevel);
//     }
//     // ... other XdgShellHandler methods ...
// }
// This pattern applies to all the requests. The XdgShellHandler trait methods in DesktopState
// would call these functions.
