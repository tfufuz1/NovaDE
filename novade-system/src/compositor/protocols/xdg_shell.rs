// novade-system/src/compositor/protocols/xdg_shell.rs
// Implementation of the xdg-shell Wayland protocol

use smithay::{
    delegate_xdg_shell,
    desktop::{Window, Kind, Space, PopupManager, WindowSurfaceType},
    input::{Seat, SeatHandler, SeatState, pointer::PointerHandle},
    reexports::{
        wayland_server::{
            protocol::{wl_output, wl_seat, wl_surface, xdg_shell},
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource,
        },
        calloop::LoopHandle,
    },
    utils::{Logical, Point, Rectangle, Serial},
    wayland::{
        compositor::{CompositorState, CompositorHandler, with_states},
        shell::xdg::{
            XdgShellHandler, XdgShellState, XdgToplevelSurfaceData, XdgPopupSurfaceData,
            ToplevelState as XdgToplevelState, Configure, PositionerState,
            PopupState as XdgPopupState, XdgRequest, XdgShellClientData,
        },
        seat::WaylandFocus,
    },
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tracing::{info, warn, error, debug, instrument};

/// Placeholder for the main desktop state.
///
/// This struct is intended to hold all relevant state for the compositor,
/// including window management, popups, and handlers for various Wayland protocols.
/// In a real compositor, this would be part of a larger state structure that
/// implements the necessary traits for Smithay's handlers.
// TODO: Replace with actual DesktopState from the main compositor logic.
//       This will likely be a larger struct `NovaCompositorState` that embeds
//       `XdgShellState`, `PopupManager`, `Vec<Window>`, etc., and implements
//       `XdgShellHandler` and other necessary traits.
#[derive(Debug, Default)]
pub struct DesktopState {
    /// List of all managed toplevel windows.
    pub windows: Vec<Window>,
    /// Manages XDG popups.
    pub popups: PopupManager,
    // Add other relevant desktop state fields here
    // e.g., active window, focus information, layout managers (Space).
}

impl DesktopState {
    /// Helper to find a window by its underlying `wl_surface`.
    fn window_by_surface(&self, surface: &wl_surface::WlSurface) -> Option<&Window> {
        self.windows.iter().find(|w| w.wl_surface().as_ref().map_or(false, |s| s == surface))
    }
}

/// Errors that can occur during XDG Shell operations within NovaDE.
#[derive(Debug, Error)]
pub enum XdgShellError {
    /// Indicates that a `wl_surface` expected to be an XDG toplevel or popup was not found
    /// or does not have the correct XDG role.
    #[error("XDG surface (toplevel or popup) not found for wl_surface: {surface_id:?}")]
    SurfaceNotFound {
        /// The Wayland object ID of the `wl_surface` that was not found or had an invalid role.
        surface_id: u32
    },

    /// Indicates that an operation was attempted on a `wl_surface` that already has an
    /// XDG shell role assigned, and the operation would conflict with this existing role.
    #[error("Surface {surface_id:?} already has an XDG shell role")]
    SurfaceHasRole {
        /// The Wayland object ID of the `wl_surface`.
        surface_id: u32
    },

    /// Errors related to the `xdg_positioner` object used for placing popups.
    #[error("Invalid xdg_positioner state: {reason}")]
    InvalidPositioner {
        /// A description of why the positioner state is invalid.
        reason: String
    },

    /// An error occurred while trying to create or manage an XDG popup.
    #[error("XDG popup operation failed for surface {surface_id:?}: {source}")]
    PopupError {
        /// The Wayland object ID of the popup's `wl_surface`.
        surface_id: u32,
        /// The underlying error from Smithay's `PopupManager` or related logic.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>, // Can wrap smithay::desktop::PopupError
    },

    /// A requested XDG toplevel action (e.g., maximize, minimize) could not be performed.
    #[error("Failed to perform action '{action}' on toplevel {surface_id:?}: {reason}")]
    ToplevelActionFailed {
        /// The action that failed (e.g., "maximize", "set_parent").
        action: String,
        /// The Wayland object ID of the toplevel's `wl_surface`.
        surface_id: u32,
        /// The reason for the failure.
        reason: String,
    },

    /// A required Wayland resource (e.g., `wl_seat`) was not available or invalid.
    #[error("Required Wayland resource '{resource_name}' (id: {resource_id:?}) is invalid or missing")]
    InvalidResource {
        /// Name of the Wayland resource type (e.g., "wl_seat").
        resource_name: String,
        /// Optional Wayland object ID of the resource, if available.
        resource_id: Option<u32>,
    },
}


/// Placeholder for NovaDE's specific XDG Shell state.
///
/// In a mature compositor, this struct would be part of the global `NovaCompositorState`.
/// It holds Smithay's `XdgShellState` and potentially other state specific to
/// NovaDE's handling of XDG shell surfaces.
// TODO: This struct will likely be merged into a global `NovaCompositorState`.
//       The `XdgShellState` field will be a member of that global state.
pub struct NovaXdgShellState {
    /// Smithay's core XDG Shell state.
    pub xdg_shell_state: XdgShellState,
    // Potentially other state specific to NovaDE's handling of xdg-shell,
    // for example, custom policies or window management details not covered by `DesktopState`.
}

impl NovaXdgShellState {
    /// Creates a new `NovaXdgShellState`.
    pub fn new() -> Self {
        NovaXdgShellState {
            xdg_shell_state: XdgShellState::new(),
            // Initialize other fields if any
        }
    }
}

impl XdgShellHandler for DesktopState {
    #[instrument(skip(self), name = "xdg_shell_handler_state_access")]
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        // This assumes DesktopState will eventually hold NovaXdgShellState or XdgShellState directly.
        // For now, we'll need a placeholder or a way to access it.
        // This is a common challenge when integrating library handlers.
        // TODO: Properly integrate XdgShellState. This method should return a mutable reference
        //       to the `XdgShellState` instance owned by the main compositor state (e.g., `NovaCompositorState`).
        //       Example: `&mut self.nova_compositor_state.xdg_shell_state_field`
        error!("XdgShellHandler::xdg_shell_state() called on placeholder DesktopState. Needs proper integration with global compositor state.");
        panic!("XdgShellHandler::xdg_shell_state() needs proper integration with DesktopState/NovaCompositorState");
    }

    #[instrument(skip(self, surface), name = "xdg_new_toplevel")]
    fn new_toplevel(&mut self, surface: WindowSurfaceType) {
        info!(surface_details = ?surface.wl_surface().id(), "New XDG toplevel surface requested");
        // This is where a new window (toplevel surface) is created.
        // We need to create a Smithay `Window` object and add it to our DesktopState.
        let window = Window::new(Kind::Xdg(surface));
        debug!(window_details = ?window, "Created new Window for XDG toplevel");
        self.windows.push(window);
        // TODO: Set initial geometry, focus, send initial configure, etc.
        // TODO: Signal layout change or new window to other parts of NovaDE (e.g., window manager, taskbar).
    }

    #[instrument(skip(self, surface, positioner), name = "xdg_new_popup")]
    fn new_popup(&mut self, surface: XdgPopupSurfaceData, positioner: PositionerState) {
        let surface_id = surface.wl_surface().id();
        info!(popup_surface_id = ?surface_id, parent_surface_id = ?surface.parent().map(|p| p.id()), "New XDG popup surface requested");
        // This is where a new popup surface is created.
        // We need to manage its position based on the parent surface and the positioner rules.
        match self.popups.create_popup(surface, positioner) {
            Ok(popup_handle) => {
                // TODO: Manage popup geometry, focus, stacking relative to parent, etc.
                // The PopupManager handles a lot of this.
                debug!(popup_handle = ?popup_handle, popup_surface_id = ?surface_id, "XDG popup created successfully");
            }
            Err(e) => {
                error!(popup_surface_id = ?surface_id, error = %e, "Failed to create XDG popup");
                // Consider how to propagate this error if necessary.
                // For now, PopupManager handles the error internally by destroying the popup resource.
            }
        }
    }

    #[instrument(skip(self, surface, seat), name = "xdg_popup_grab")]
    fn grab(&mut self, surface: XdgPopupSurfaceData, seat: wl_seat::WlSeat, serial: Serial) {
        let surface_id = surface.wl_surface().id();
        info!(popup_surface_id = ?surface_id, seat_id = ?seat.id(), grab_serial = ?serial, "XDG popup grab requested");
        // This handles popup grabs (e.g., for menus that need to capture input).
        // Smithay's PopupManager should typically handle the grab logic.
        // TODO: Ensure PopupManager is correctly invoked or that its grab logic is sufficient.
        //       This might involve initiating a seat grab if the popup requests explicit grab behavior.
        if let Some(_popup) = self.popups.find_popup(&surface.wl_surface()) {
             debug!(popup_surface_id = ?surface_id, "Popup grab for surface relies on PopupManager's internal grab logic.");
             // Smithay's PopupManager might initiate a seat grab internally if the popup is interactive.
        } else {
            warn!(popup_surface_id = ?surface_id, "Grab requested for a non-existent or unmanaged XDG popup surface.");
        }
    }

    #[instrument(skip(self, surface, positioner), name = "xdg_reposition_request")]
    fn reposition_request(&mut self, surface: XdgPopupSurfaceData, positioner: PositionerState, token: u32) {
        let surface_id = surface.wl_surface().id();
        info!(popup_surface_id = ?surface_id, reposition_token = token, "XDG popup reposition requested");
         match self.popups.reposition_popup(surface, positioner, token) {
            Ok(_) => debug!(popup_surface_id = ?surface_id, "XDG popup repositioned successfully."),
            Err(e) => error!(popup_surface_id = ?surface_id, error = %e, "Failed to reposition XDG popup."),
        }
    }

    // --- Toplevel Requests ---
    #[instrument(skip(self, surface, seat), name = "xdg_toplevel_request_move")]
    fn toplevel_request_move(&mut self, surface: WindowSurfaceType, seat: wl_seat::WlSeat, serial: Serial) {
        let surface_id = surface.wl_surface().id();
        info!(toplevel_surface_id = ?surface_id, seat_id = ?seat.id(), event_serial = ?serial, "XDG toplevel requested interactive move");
        // Handle client request to start an interactive move.
        // This usually involves initiating a pointer grab on the seat.
        // TODO: Implement interactive move logic:
        //       1. Find the `Window` associated with `surface`.
        //       2. Verify the `seat` and `serial` are valid for initiating a move.
        //       3. Start a pointer grab (e.g., `seat.get_pointer().unwrap().set_grab(...)`)
        //          with a custom grab handler that moves the window.
        //       4. Update window position based on grab motion.
        //       5. End grab on button release.
        warn!(toplevel_surface_id = ?surface_id, "TODO: Implement interactive move logic for XDG toplevel.");
    }

    #[instrument(skip(self, surface, seat), name = "xdg_toplevel_request_resize")]
    fn toplevel_request_resize(&mut self, surface: WindowSurfaceType, seat: wl_seat::WlSeat, serial: Serial, edges: xdg_toplevel::ResizeEdge) {
        let surface_id = surface.wl_surface().id();
        info!(toplevel_surface_id = ?surface_id, seat_id = ?seat.id(), event_serial = ?serial, ?edges, "XDG toplevel requested interactive resize");
        // Handle client request to start an interactive resize.
        // TODO: Implement interactive resize logic (similar to move, but for resizing based on `edges`).
        warn!(toplevel_surface_id = ?surface_id, "TODO: Implement interactive resize logic for XDG toplevel.");
    }

    #[instrument(skip(self, surface), name = "xdg_toplevel_request_maximize")]
    fn toplevel_request_maximize(&mut self, surface: WindowSurfaceType) {
        let surface_id = surface.wl_surface().id();
        info!(toplevel_surface_id = ?surface_id, "XDG toplevel requested maximize");
        if let Some(window) = self.windows.iter_mut().find(|w| w.wl_surface().as_ref().map_or(false, |s| s == surface.wl_surface())) {
            // TODO: Implement actual maximization logic:
            //       1. Update window state (e.g., `window.set_maximized(true)`).
            //       2. Calculate new geometry for maximized state (e.g., fill work area of an output).
            //       3. Send a configure event to the client with the new state and geometry.
            //          `window.toplevel().send_configure();` might be part of this,
            //          but specific state flags for maximized need to be set in `ToplevelState`.
            warn!(toplevel_surface_id = ?surface_id, "TODO: Implement actual maximization logic for XDG toplevel.");
            window.toplevel().send_configure(); // Placeholder: send configure to ack, but no state change yet
        } else {
            warn!(toplevel_surface_id = ?surface_id, "Maximize requested for non-existent/unmanaged toplevel.");
        }
    }

    #[instrument(skip(self, surface), name = "xdg_toplevel_request_unmaximize")]
    fn toplevel_request_unmaximize(&mut self, surface: WindowSurfaceType) {
        let surface_id = surface.wl_surface().id();
        info!(toplevel_surface_id = ?surface_id, "XDG toplevel requested unmaximize");
        if let Some(window) = self.windows.iter_mut().find(|w| w.wl_surface().as_ref().map_or(false, |s| s == surface.wl_surface())) {
            // TODO: Implement unmaximization:
            //       1. Update window state (`window.set_maximized(false)`).
            //       2. Restore previous geometry or calculate new one.
            //       3. Send configure event.
            warn!(toplevel_surface_id = ?surface_id, "TODO: Implement unmaximization logic for XDG toplevel.");
            window.toplevel().send_configure();
        } else {
            warn!(toplevel_surface_id = ?surface_id, "Unmaximize requested for non-existent/unmanaged toplevel.");
        }
    }

    #[instrument(skip(self, surface), name = "xdg_toplevel_request_fullscreen")]
    fn toplevel_request_fullscreen(&mut self, surface: WindowSurfaceType, output: Option<wl_output::WlOutput>) {
        let surface_id = surface.wl_surface().id();
        info!(toplevel_surface_id = ?surface_id, output_id = ?output.as_ref().map(|o|o.id()), "XDG toplevel requested fullscreen");
        if let Some(window) = self.windows.iter_mut().find(|w| w.wl_surface().as_ref().map_or(false, |s| s == surface.wl_surface())) {
            // TODO: Implement fullscreen logic:
            //       1. Determine target output (use provided `output` or select one).
            //       2. Update window state (`window.set_fullscreen(true, target_output_handle)`).
            //       3. Set geometry to fill the output.
            //       4. Send configure event with fullscreen state.
            warn!(toplevel_surface_id = ?surface_id, "TODO: Implement fullscreen logic for XDG toplevel.");
            window.toplevel().send_configure();
        } else {
            warn!(toplevel_surface_id = ?surface_id, "Fullscreen requested for non-existent/unmanaged toplevel.");
        }
    }

    #[instrument(skip(self, surface), name = "xdg_toplevel_request_unfullscreen")]
    fn toplevel_request_unfullscreen(&mut self, surface: WindowSurfaceType) {
        let surface_id = surface.wl_surface().id();
        info!(toplevel_surface_id = ?surface_id, "XDG toplevel requested unfullscreen");
         if let Some(window) = self.windows.iter_mut().find(|w| w.wl_surface().as_ref().map_or(false, |s| s == surface.wl_surface())) {
            // TODO: Implement unfullscreen logic.
            warn!(toplevel_surface_id = ?surface_id, "TODO: Implement unfullscreen logic for XDG toplevel.");
            window.toplevel().send_configure();
        } else {
            warn!(toplevel_surface_id = ?surface_id, "Unfullscreen requested for non-existent/unmanaged toplevel.");
        }
    }

    #[instrument(skip(self, surface), name = "xdg_toplevel_request_minimize")]
    fn toplevel_request_minimize(&mut self, surface: WindowSurfaceType) {
        let surface_id = surface.wl_surface().id();
        info!(toplevel_surface_id = ?surface_id, "XDG toplevel requested minimize");
        if let Some(window) = self.windows.iter_mut().find(|w| w.wl_surface().as_ref().map_or(false, |s| s == surface.wl_surface())) {
            // TODO: Implement minimization logic (e.g., hide window, set minimized state, update taskbar).
            warn!(toplevel_surface_id = ?surface_id, "TODO: Implement minimization logic for XDG toplevel.");
            // Minimization might not always involve a configure event for the surface itself,
            // but rather changes in how the compositor manages/displays it.
            // However, sending a configure with the new state is good practice.
            // window.toplevel().send_configure();
        } else {
            warn!(toplevel_surface_id = ?surface_id, "Minimize requested for non-existent/unmanaged toplevel.");
        }
    }

    #[instrument(skip(self, surface, parent), name = "xdg_toplevel_set_parent")]
    fn toplevel_set_parent(&mut self, surface: WindowSurfaceType, parent: Option<WindowSurfaceType>) {
        let surface_id = surface.wl_surface().id();
        let parent_id = parent.as_ref().map(|p| p.wl_surface().id());
        info!(toplevel_surface_id = ?surface_id, parent_surface_id = ?parent_id, "XDG toplevel set_parent requested");
        // Handle setting a parent for a toplevel (e.g., for dialogs).
        // This affects stacking order and potentially behavior (e.g., dialog stays on top of parent).
        // TODO: Implement logic to:
        //       1. Find `Window` for `surface` and `parent` (if Some).
        //       2. Update window hierarchy (e.g., `window.set_parent(parent_window)`).
        //       3. This might trigger layout/stacking changes.
        warn!(toplevel_surface_id = ?surface_id, "TODO: Implement set_parent logic for XDG toplevel.");
    }

    #[instrument(skip(self, surface), name = "xdg_toplevel_set_title")]
    fn toplevel_set_title(&mut self, surface: WindowSurfaceType, title: String) {
        let surface_id = surface.wl_surface().id();
        info!(toplevel_surface_id = ?surface_id, new_title = %title, "XDG toplevel set_title requested");
        if let Some(window) = self.windows.iter_mut().find(|w| w.wl_surface().as_ref().map_or(false, |s| s == surface.wl_surface())) {
            // TODO: Store the title (e.g., `window.set_title(title)`).
            //       This might trigger a UI update if the title is displayed in server-side decorations or a taskbar.
            debug!(toplevel_surface_id = ?surface_id, "Title set for window. (Actual storage TODO)");
        } else {
            warn!(toplevel_surface_id = ?surface_id, "Set_title requested for non-existent/unmanaged toplevel.");
        }
    }

    #[instrument(skip(self, surface), name = "xdg_toplevel_set_app_id")]
    fn toplevel_set_app_id(&mut self, surface: WindowSurfaceType, app_id: String) {
        let surface_id = surface.wl_surface().id();
        info!(toplevel_surface_id = ?surface_id, new_app_id = %app_id, "XDG toplevel set_app_id requested");
         if let Some(window) = self.windows.iter_mut().find(|w| w.wl_surface().as_ref().map_or(false, |s| s == surface.wl_surface())) {
            // TODO: Store the app_id (e.g., `window.set_app_id(app_id)`).
            //       Useful for window matching, theming, desktop file association, etc.
            debug!(toplevel_surface_id = ?surface_id, "App ID set for window. (Actual storage TODO)");
        } else {
            warn!(toplevel_surface_id = ?surface_id, "Set_app_id requested for non-existent/unmanaged toplevel.");
        }
    }

    #[instrument(skip(self, surface, configure), name = "xdg_toplevel_ack_configure")]
    fn toplevel_ack_configure(&mut self, surface: WindowSurfaceType, configure: Configure) {
        let surface_id = surface.wl_surface().id();
        info!(toplevel_surface_id = ?surface_id, configure_serial = ?configure.serial, "XDG toplevel ack_configure received");
        // Client acknowledges a configuration event sent by the server.
        // This is important for flow control and ensuring state synchronization.
        // The `configure` object contains the serial of the server's configure event.
        // TODO: Validate the configure serial against pending configure events for the window.
        //       Update window state if the acknowledged configuration implies changes
        //       (e.g., a resize is now complete from the client's perspective).
        //       Smithay's `Window::on_commit` or similar mechanisms often handle state updates
        //       based on `ack_configure` by checking `XdgToplevelSurfaceData::pending_configure_serial`.
        with_states(surface.wl_surface(), |states| {
            if let Some(toplevel_data) = states.data_map.get::<XdgToplevelSurfaceData>() {
                // Smithay's XdgToplevelSurfaceData handles the ack_configure logic internally.
                // We might log or trigger further actions if needed.
                debug!(toplevel_surface_id = ?surface_id, "ack_configure processed by XdgToplevelSurfaceData.");
            }
        });
    }

    // --- Common XDG Surface Requests ---
    #[instrument(skip(self, surface, configure), name = "xdg_surface_ack_configure")]
    fn xdg_surface_ack_configure(&mut self, surface: wl_surface::WlSurface, configure: Configure) {
        let surface_id = surface.id();
        info!(surface_id = ?surface_id, configure_serial = ?configure.serial, "XDG surface ack_configure received");
        // This is a general ack_configure for any XDG surface (toplevel or popup).
        // Smithay's XdgShellHandler might require this to be distinct from toplevel_ack_configure,
        // or it might be that one calls the other, or this is called after Smithay's internal processing.
        // Typically, Smithay's internal logic for XDG surfaces (toplevel or popup) handles the ack
        // by updating their respective pending states. We can use this callback for additional logic if needed.
        let is_toplevel = self.windows.iter().any(|w| w.wl_surface().as_ref().map_or(false, |s| s == &surface));
        let is_popup = self.popups.find_popup(&surface).is_some();

        if is_toplevel {
            debug!(surface_id = ?surface_id, "ack_configure is for an XDG toplevel surface.");
            // Further actions if toplevel_ack_configure wasn't sufficient or if this provides more context.
        } else if is_popup {
            debug!(surface_id = ?surface_id, "ack_configure is for an XDG popup surface.");
            // Popups also receive configure events (e.g., for size changes based on content).
        } else {
            warn!(surface_id = ?surface_id, "ack_configure received for an unknown XDG surface type.");
        }
    }

    #[instrument(skip(self, surface, geometry), name = "xdg_surface_set_window_geometry")]
    fn xdg_surface_set_window_geometry(&mut self, surface: wl_surface::WlSurface, geometry: Rectangle<i32, Logical>) {
        let surface_id = surface.id();
        info!(surface_id = ?surface_id, ?geometry, "XDG surface set_window_geometry requested");
        // Client suggests a window geometry (the part of the surface that is opaque or contains content).
        // This is useful for server-side decorations (to know where to draw them relative to content)
        // or for layout calculations by the compositor.
        with_states(&surface, |states| {
            // Smithay stores this geometry internally in XdgSurfaceUserData (e.g., XdgToplevelSurfaceData).
            // We might want to react to it, e.g., if drawing custom decorations or for layout decisions.
            if states.data_map.get::<XdgToplevelSurfaceData>().is_some() {
                debug!(surface_id = ?surface_id, "Window geometry hint set for XDG toplevel surface.");
            } else if states.data_map.get::<XdgPopupSurfaceData>().is_some() {
                debug!(surface_id = ?surface_id, "Window geometry hint set for XDG popup surface.");
            } else {
                 warn!(surface_id = ?surface_id, "set_window_geometry for an XDG surface of unknown type.");
            }
            // Accessing the actual data:
            // if let Some(toplevel_data) = states.data_map.get_mut::<XdgToplevelSurfaceData>() {
            //     toplevel_data.window_geometry = Some(geometry); // Smithay does this internally
            // }
        });
    }
}

// TODO: The `delegate_xdg_shell!(DesktopState);` macro call needs to be in the module
//       where the main compositor state (`NovaCompositorState`) is defined and
//       `NovaCompositorState` implements `XdgShellHandler` and other required traits.
//       It cannot live solely in this file if `DesktopState` is just a placeholder part
//       of a larger, yet-to-be-defined `NovaCompositorState`.
//
// delegate_xdg_shell!(NovaCompositorState); // Example, assuming NovaCompositorState is the main one.


// TODO: Add unit tests for handler methods.
// These will likely be integration tests requiring a mock Wayland client and server setup.
// #[cfg(test)]
// mod tests {
//     use super::*;
//     // ... test setup ...
// }

/// Initializes and registers the XDG Shell global (`xdg_wm_base`).
///
/// This function should be called once during compositor startup.
/// It makes the XDG Shell functionality available to Wayland clients.
///
/// # Arguments
///
/// * `display`: A handle to the Wayland display.
///
/// # Type Parameters
///
/// * `D`: The main compositor state type. This type must implement
///   `GlobalDispatch<xdg_wm_base, ()>` and `Dispatch` for all XDG Shell related objects.
///   It also needs to implement `XdgShellHandler` (usually via the main compositor state struct
///   which embeds `XdgShellState` and other necessary states like `DesktopState` components).
///
/// # Errors
///
/// Returns an error if the global could not be created.
#[instrument(skip(display))]
pub fn init_xdg_shell<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // May be needed if XdgShellState requires it for certain operations
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<xdg_shell::XdgWmBase, ()> +
       Dispatch<xdg_shell::XdgWmBase, XdgShellClientData, D> + // XdgShellClientData for per-client state
       Dispatch<xdg_shell::XdgPositioner, PositionerState, D> +
       Dispatch<xdg_shell::XdgSurface, XdgShellSurfaceData, D> + // XdgShellSurfaceData for xdg_surface
       Dispatch<xdg_shell::XdgToplevel, ToplevelSurfaceData, D> +
       Dispatch<xdg_shell::XdgPopup, XdgPopupSurfaceData, D> +
       XdgShellHandler + // The main state D must implement the handler logic
       'static,
{
    info!("Initializing XDG Shell (xdg_wm_base) global environment.");

    // The XdgShellState should be part of your main compositor state `D`.
    // `D` then implements `XdgShellHandler`, and `fn xdg_shell_state(&mut self) -> &mut XdgShellState`
    // provides access to it.

    // The `delegate_xdg_shell!(D)` macro uses this setup to correctly dispatch requests.

    // Create the xdg_wm_base global. Version 6 is common for xdg-shell.
    let _xdg_wm_base_global = display.create_global::<D, xdg_shell::XdgWmBase, _>(6, ());
    info!("xdg_wm_base global (version 6) created successfully.");

    // Note: The `XdgShellState::new()` call happens when you initialize your main compositor state `D`.
    // For example:
    // pub struct NovaCompositorState {
    //     pub xdg_shell_state: XdgShellState,
    //     // ... other states ...
    // }
    // impl NovaCompositorState {
    //     pub fn new() -> Self { Self { xdg_shell_state: XdgShellState::new(), ... } }
    // }
    // The `delegate_xdg_shell!(NovaCompositorState);` macro ties it all together.

    Ok(())
}

// General Note on `DesktopState` vs. `NovaCompositorState`:
// The `DesktopState` struct used in this file is a placeholder for the parts of the
// true main compositor state (let's call it `NovaCompositorState`) that are relevant
// to XDG Shell (like `Vec<Window>`, `PopupManager`).
// `NovaCompositorState` would also hold `XdgShellState` itself and all other necessary
// Smithay state objects (`SeatState`, `CompositorState`, `ShmState`, `OutputManagerState`, etc.).
// The `impl XdgShellHandler for DesktopState` would become `impl XdgShellHandler for NovaCompositorState`.
// The `delegate_xdg_shell!` macro would then be invoked on `NovaCompositorState`.
// This current file structure allows focusing on the XDG Shell logic, with the understanding
// that it will be integrated into a larger state system.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod xdg_shell;
// And that `protocols/mod.rs` is declared in `novade-system/src/compositor/mod.rs`
// pub mod protocols;
// And that `compositor` module is declared in `novade-system/src/lib.rs` or `main.rs`
// pub mod compositor;
