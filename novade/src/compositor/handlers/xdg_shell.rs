//! Handler for the `xdg_shell` Wayland protocol.
//!
//! This module implements `smithay::wayland::shell::xdg::XdgShellHandler` for
//! `NovaCompositorState`. XDG Shell is a crucial Wayland protocol that defines
//! standard interfaces for desktop-like window management, including creating
//! toplevel windows (`xdg_toplevel`) and popups (`xdg_popup`).
//!
//! This handler manages the lifecycle of these XDG surfaces, their roles,
//! and interactions like move, resize, and grab requests.

#![allow(unused_variables)]
#![allow(dead_code)]

use crate::compositor::state::NovaCompositorState;
use smithay::{
    delegate_xdg_shell,
    desktop::{Window, WindowSurfaceType, Space},
    reexports::wayland_server::{
        protocol::{
            wl_seat::WlSeat,
            wl_surface::WlSurface,
        },
        Client,
    },
    wayland::shell::xdg::{
        Configure, PositionerState, PopupRequest, RequestId, SurfaceCachedState, ToplevelRequest,
        XdgShellHandler, XdgShellState, XdgToplevelSurfaceData, ToplevelSurface, PopupSurface, XdgRoleError, // Added XdgRoleError
    },
    utils::{Serial, Point},
};

// The `smithay::wayland::shell::xdg::XdgShellState` is stored in `NovaCompositorState`.
// This handler provides access to it and defines callbacks for XDG Shell related events.
// Toplevel windows created via this handler are wrapped in `smithay::desktop::Window`
// and managed within `NovaCompositorState.space`. Popups are managed by `NovaCompositorState.popup_manager`.

impl XdgShellHandler for NovaCompositorState {
    /// Provides access to Smithay's `XdgShellState`, which tracks XDG surfaces
    /// and their roles.
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    /// Called when a client creates a new `xdg_toplevel` surface.
    ///
    /// A `smithay::desktop::Window` is created to wrap the `ToplevelSurface`,
    /// and this window is then mapped into the compositor's `Space`.
    ///
    /// TODO: Implement a proper window placement strategy instead of (0,0).
    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        match Window::new_wayland_window(self.logger.clone(), surface) {
            Ok(window) => {
                slog::info!(
                    self.logger,
                    "New XDG Toplevel created: {:?}",
                    window.wl_surface().map(|s| s.id())
                );
                // Map the new window into the space at a default location.
                // The `false` for `activate` means don't automatically focus it here.
                // Focus decisions are part of a broader window management policy (e.g., click-to-focus).
                self.space.map_element(window, (0, 0), false);
                // TODO: Send initial configure event to the toplevel.
                // TODO: Potentially set initial focus based on compositor policy.
            }
            Err(err) => {
                slog::warn!(
                    self.logger,
                    "Failed to create XDG Toplevel from surface: {}",
                    err // err might be XdgRoleError or similar
                );
            }
        }
    }

    /// Called when a client creates a new `xdg_popup` surface.
    ///
    /// The popup is created and managed by `NovaCompositorState.popup_manager`.
    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        slog::info!(
            self.logger,
            "New XDG Popup created: {:?}, for parent {:?}",
            surface.wl_surface().id(),
            surface.get_parent_surface().map(|s|s.id())
        );
        // Delegate popup creation and management to PopupManager.
        match self.popup_manager.create_popup(surface, positioner) {
            Ok(_) => { /* Popup created successfully and is now tracked by PopupManager */ }
            Err(err) => {
                slog::warn!(self.logger, "Failed to create XDG Popup: {}", err);
            }
        }
    }

    /// Called when a client requests an interactive grab for an `xdg_popup` (e.g., for a context menu).
    ///
    /// TODO: Implement popup grab logic, likely involving `PopupManager` and seat focus.
    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {
        slog::info!(
            self.logger,
            "XDG Popup grab initiated: popup {:?}, seat {:?}",
            surface.wl_surface().id(),
            seat.id()
        );
        // This should typically be handled by `PopupManager::grab_popup`.
        // Example:
        // match self.popup_manager.grab_popup(self, surface, &seat, serial) {
        //     Ok(_) => {},
        //     Err(err) => slog::warn!(self.logger, "Failed to start popup grab: {}", err),
        // }
        // TODO: Ensure grab implementation is correct.
    }

    /// Called when a client requests an interactive reposition for an `xdg_toplevel`.
    ///
    /// TODO: Implement interactive move logic, usually involving a pointer grab.
    /// The compositor should acknowledge this request by sending a `configure` event.
    fn reposition_request(
        &mut self,
        surface: ToplevelSurface,
        seat: WlSeat,
        request_id: RequestId,
        token: String,
    ) {
        slog::info!(
            self.logger,
            "XDG Toplevel reposition request: surface {:?}, token {}",
            surface.wl_surface().id(),
            token
        );
        // Acknowledge the request. Actual repositioning happens during a pointer grab.
        surface.send_configure();
        // TODO: Initiate an interactive move operation if conditions are met
        // (e.g., user is holding down a titlebar). This involves:
        // - Starting a pointer grab.
        // - Updating window position in `Space` based on pointer motion.
        // - Sending further `configure` events to the client.
    }

    /// Called when a client requests an interactive resize for an `xdg_toplevel`.
    ///
    /// TODO: Implement interactive resize logic, usually involving a pointer grab on window edges.
    /// The compositor should acknowledge this request by sending a `configure` event.
    fn resize_request(
        &mut self,
        surface: ToplevelSurface,
        seat: WlSeat,
        request_id: RequestId,
        edges: smithay::reexports::protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    ) {
        slog::info!(
            self.logger,
            "XDG Toplevel resize request: surface {:?}, edges {:?}",
            surface.wl_surface().id(),
            edges
        );
        surface.send_configure();
        // TODO: Initiate an interactive resize operation similar to reposition.
    }

    /// Called when a client acknowledges a configure event sent by the compositor.
    ///
    /// This is crucial for synchronized state changes (e.g., resizing). The compositor
    /// sends a configure event with new size/state, and the client applies it and acks.
    ///
    /// TODO: Handle `ack_configure` to finalize state changes (e.g., mark a resize as complete).
    fn ack_configure(&mut self, surface: WlSurface, configure: Configure) {
        slog::debug!(
            self.logger,
            "XDG Surface acked configure: surface {:?}, configure details: {:?}",
            surface.id(),
            configure // Contains serial and potentially other data of the acked configure.
        );
        // This is important for flow control. For instance, after sending a resize configure,
        // the compositor might wait for the ack before considering the new size "stable"
        // or before sending further configures.
    }

    /// Called when an `xdg_toplevel` surface is destroyed by the client.
    ///
    /// The corresponding `Window` is unmapped from the `Space`. If the destroyed
    /// window had keyboard focus, focus is cleared.
    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let wl_surface = surface.wl_surface();
        slog::info!(self.logger, "XDG Toplevel destroyed: {:?}", wl_surface.id());

        let window_to_remove = self.space.elements().find(|w| {
            w.wl_surface().as_ref() == Some(&wl_surface)
        }).cloned();

        if let Some(window) = window_to_remove {
            self.space.unmap_elem(&window);
            slog::debug!(self.logger, "Window for surface {:?} unmapped from space.", wl_surface.id());

            if self.keyboard_focus.as_ref() == Some(&wl_surface) {
                slog::debug!(self.logger, "Destroyed toplevel had keyboard focus. Clearing focus.");
                self.set_keyboard_focus(None);
                if let Some(seat) = self.seat.as_mut() { // Check if seat is initialized
                    if let Some(keyboard) = seat.get_keyboard() {
                        keyboard.set_focus(None, SERIAL_COUNTER.next_serial());
                    }
                }
            }
        } else {
            slog::warn!(self.logger, "Attempted to destroy a toplevel not found in space: {:?}", wl_surface.id());
        }
    }

    /// Called when an `xdg_popup` surface is destroyed by the client.
    ///
    /// Cleanup is largely handled by `PopupManager` when the underlying `WlSurface` is destroyed.
    fn popup_destroyed(&mut self, surface: PopupSurface) {
        slog::info!(self.logger, "XDG Popup destroyed: {:?}", surface.wl_surface().id());
        // `PopupManager` typically handles the unmapping and cleanup of popups
        // when their `WlSurface` is destroyed or their role is removed.
        // Additional cleanup specific to NovaDE can be done here if needed.
    }

    /// Called when a role assignment error occurs for an XDG surface.
    ///
    /// This can happen if a client tries to assign an XDG role to a surface that
    /// already has a role or if the assignment is otherwise invalid.
    fn assign_role_error(&mut self, client: Client, surface: WlSurface, error: XdgRoleError) {
        slog::warn!(
            self.logger,
            "XDG role assignment error for client {:?}, surface {:?}: {:?}",
            client.id(),
            surface.id(),
            error
        );
        // TODO: Decide how to handle such errors, e.g., send a protocol error to the client,
        // or destroy the surface.
    }
    // TODO: Implement other XdgShellHandler methods for full feature support, such as:
    // show_window_menu_request, set_fullscreen_request, set_maximized_request, etc.
}

// Delegate xdg_shell requests to NovaCompositorState.
// This macro implements `GlobalDispatch<XdgWmBase, XdgShellData<D>>` and the
// necessary `Dispatch` trait for `NovaCompositorState`.
// `XdgShellData` is a marker struct provided by Smithay for `xdg_wm_base` globals.
delegate_xdg_shell!(NovaCompositorState);


#[cfg(test)]
mod tests {
    // Unit tests for XDG shell handlers are complex due to the need for mock
    // ToplevelSurface, PopupSurface, and a realistic NovaCompositorState.
    // These tests are conceptual and primarily verify that the state's `Space`
    // would be modified as expected if the handler logic were called with valid objects.
    // True verification of XDG shell interactions relies heavily on integration testing.

    use super::*;
    use crate::compositor::state::NovaCompositorState;
    use smithay::reexports::wayland_server::{Display, protocol::wl_surface::WlSurface};
    // use smithay::wayland::shell::xdg::{ToplevelSurface, XdgShellHandler}; // Not directly used in simplified tests

    fn test_logger() -> slog::Logger {
        slog::Logger::root(slog::Discard, slog::o!())
    }

    fn create_test_state() -> (NovaCompositorState, DisplayHandle) {
        let logger = test_logger();
        let mut display: Display<NovaCompositorState> = Display::new().unwrap();
        let display_handle = display.handle();
        let state = NovaCompositorState::new(display_handle.clone(), logger);
        (state, display_handle)
    }

    #[test]
    fn test_xdg_new_toplevel_conceptual_adds_to_space() {
        let (mut state, _dh) = create_test_state();
        // In a real scenario, `new_toplevel` is called with a `ToplevelSurface`.
        // It then creates a `Window` and maps it to `state.space`.
        // Here, we can't easily mock `ToplevelSurface`.
        // We check that the space is initially empty. A real test would need an integration setup.
        assert_eq!(state.space.elements().count(), 0, "Space should be initially empty.");
        slog::info!(state.logger, "new_toplevel conceptual test: initial count checked.");
        // If we could call new_toplevel with a mock, we'd assert count becomes 1.
    }

    #[test]
    fn test_xdg_toplevel_destroyed_conceptual_removes_from_space() {
        let (mut state, dh) = create_test_state();
        // Similar to the above, this is conceptual. If a window *was* in the space,
        // and its corresponding `ToplevelSurface` was passed to `toplevel_destroyed`,
        // the window should be removed from the space.
        slog::info!(state.logger, "toplevel_destroyed conceptual test: verifies logic if window existed.");
        // To test properly:
        // 1. Create a mock client and ToplevelSurface.
        // 2. Call state.new_toplevel(mock_surface) to add it.
        // 3. Assert count is 1.
        // 4. Call state.toplevel_destroyed(mock_surface).
        // 5. Assert count is 0.
    }
}
```
