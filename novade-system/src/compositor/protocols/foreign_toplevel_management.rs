// novade-system/src/compositor/protocols/foreign_toplevel_management.rs
// Implementation of the wlr_foreign_toplevel_management_unstable_v1 Wayland protocol

use smithay::{
    delegate_foreign_toplevel_manager,
    reexports::{
        wayland_protocols_wlr::foreign_toplevel::v1::server::{
            zwlr_foreign_toplevel_handle_v1::{self, ZwlrForeignToplevelHandleV1, Request as HandleRequest, Event as HandleEvent, State as ToplevelState, Rectangle as ToplevelRectangle},
            zwlr_foreign_toplevel_manager_v1::{self, ZwlrForeignToplevelManagerV1, Request as ManagerRequest, Event as ManagerEvent},
        },
        wayland_server::{
            protocol::{wl_seat, wl_output, wl_surface}, // For activation, fullscreen output
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    utils::{Serial, Logical, Point, Size, Rectangle as SmithayRectangle}, // For geometry
    wayland::foreign_toplevel::{
        ForeignToplevelHandler, ForeignToplevelManagerState, ForeignToplevelHandle, // Smithay's types
        // No specific UserData structs defined by Smithay for these resources,
        // state is usually managed via ForeignToplevelHandle and events.
    },
    desktop::{Window, Kind as WindowKind, Space}, // To get list of toplevel windows and their states
    // Needed to interact with XDG shell windows for actions like maximize, minimize etc.
    wayland::shell::xdg::{XdgToplevelSurfaceData, ToplevelSurfaceData},
};
use std::{
    sync::{Arc, Mutex},
    collections::HashMap,
};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to hold `ForeignToplevelManagerState` and provide access to
// the list of toplevel windows and their properties.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For Foreign Toplevel Management, it would need to manage or access:
    // - ForeignToplevelManagerState
    // - A list of all current toplevel windows (e.g., from a `Space` or `Vec<Window>`).
    // - Mechanisms to manipulate these windows (activate, minimize, maximize, close, etc.),
    //   which usually involves interacting with their shell handlers (e.g., XdgShellHandler).
    // - Output information for fullscreen requests.
}

#[derive(Debug, Error)]
pub enum ForeignToplevelError {
    #[error("Toplevel window not found or handle is invalid")]
    ToplevelNotFound,
    #[error("Action on toplevel failed (e.g., maximize, close)")]
    ActionFailed,
    #[error("Output not found for fullscreen request")]
    OutputNotFound,
    // No specific errors defined in the protocol for manager/handle creation itself.
}


// The main compositor state (e.g., NovaCompositorState) would implement ForeignToplevelHandler
// and store ForeignToplevelManagerState.
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub foreign_toplevel_manager_state: ForeignToplevelManagerState,
//     pub space: Space<Window>, // To access toplevel windows
//     // Access to XdgShellState or other shell states to perform actions on windows.
//     ...
// }
//
// impl ForeignToplevelHandler for NovaCompositorState {
//     fn foreign_toplevel_manager_state(&mut self) -> &mut ForeignToplevelManagerState {
//         &mut self.foreign_toplevel_manager_state
//     }
//
//     // Request handlers for ZwlrForeignToplevelHandleV1 are implemented here.
//     // For example:
//     fn request_maximize(&mut self, toplevel_handle: ForeignToplevelHandle) {
//         info!("Foreign toplevel client requests maximize for handle {:?}", toplevel_handle.id());
//         if let Some(window) = self.space.elements().find(|w| w.toplevel_handle() == Some(toplevel_handle.clone())) {
//             // Assuming window is XDG Toplevel
//             if let WindowKind::Xdg(xdg_toplevel) = window.toplevel() {
//                 xdg_toplevel.send_configure_maximize(); // Or similar method
//             }
//         } else { warn!("No window found for toplevel handle {:?}", toplevel_handle.id()); }
//     }
//     // ... other request handlers like request_minimize, request_close, etc. ...
//
//     fn new_foreign_toplevel(&mut self, toplevel_handle: ForeignToplevelHandle) {
//        // Called when a new toplevel window appears that should be managed.
//        // `toplevel_handle` is the Smithay wrapper.
//        // The manager will send `toplevel` event to clients.
//     }
//
//     fn foreign_toplevel_closed(&mut self, toplevel_handle: ForeignToplevelHandle) {
//        // Called when a managed toplevel window is closed.
//        // The manager will send `closed` event on the handle and then destroy it.
//     }
// }
// delegate_foreign_toplevel_manager!(NovaCompositorState);


impl ForeignToplevelHandler for DesktopState { // Replace DesktopState with NovaCompositorState
    fn foreign_toplevel_manager_state(&mut self) -> &mut ForeignToplevelManagerState {
        // TODO: Properly integrate ForeignToplevelManagerState with DesktopState or NovaCompositorState.
        panic!("ForeignToplevelHandler::foreign_toplevel_manager_state() needs proper integration.");
        // Example: &mut self.nova_compositor_state.foreign_toplevel_manager_state
    }

    // --- Callbacks for ZwlrForeignToplevelHandleV1 requests ---
    // These are called by Smithay when a client sends a request on a specific handle.

    fn request_maximize(&mut self, handle: ForeignToplevelHandle) {
        info!("Foreign client requests MAXIMIZE for toplevel handle ID: {:?}", handle.id());
        // TODO: Find the Window associated with `handle`.
        //       Perform maximization (e.g., by calling appropriate methods on its XDG toplevel state).
        //       Update the handle's state via `handle.send_state(...)`.
        warn!("TODO: Implement request_maximize for foreign toplevel handle {:?}", handle.id());
        // Example:
        // if let Some(window) = self.find_window_for_foreign_handle(&handle) {
        //     if let WindowKind::Xdg(t) = window.toplevel() { t.set_maximized(true); }
        //     // Update handle state after action
        //     handle.send_state(&[ToplevelState::Maximized]); handle.send_done();
        // }
    }

    fn request_unmaximize(&mut self, handle: ForeignToplevelHandle) {
        info!("Foreign client requests UNMAXIMIZE for toplevel handle ID: {:?}", handle.id());
        // TODO: Implement unmaximization.
        warn!("TODO: Implement request_unmaximize for foreign toplevel handle {:?}", handle.id());
    }

    fn request_minimize(&mut self, handle: ForeignToplevelHandle) {
        info!("Foreign client requests MINIMIZE for toplevel handle ID: {:?}", handle.id());
        // TODO: Implement minimization.
        warn!("TODO: Implement request_minimize for foreign toplevel handle {:?}", handle.id());
    }

    fn request_unminimize(&mut self, handle: ForeignToplevelHandle) {
        info!("Foreign client requests UNMINIMIZE for toplevel handle ID: {:?}", handle.id());
        // TODO: Implement unminimization.
        warn!("TODO: Implement request_unminimize for foreign toplevel handle {:?}", handle.id());
    }

    fn request_activate(&mut self, handle: ForeignToplevelHandle, seat: wl_seat::WlSeat) {
        info!("Foreign client requests ACTIVATE for toplevel handle ID: {:?}, on seat {:?}", handle.id(), seat);
        // TODO: Find Window for handle. Find Smithay Seat for wl_seat.
        //       Activate the window (bring to front, give keyboard focus).
        //       Update handle state.
        warn!("TODO: Implement request_activate for foreign toplevel handle {:?}", handle.id());
    }

    fn request_close(&mut self, handle: ForeignToplevelHandle) {
        info!("Foreign client requests CLOSE for toplevel handle ID: {:?}", handle.id());
        // TODO: Find Window for handle. Send close request to the window (e.g., xdg_toplevel.close).
        //       The window itself will then be destroyed, which should trigger `foreign_toplevel_closed`.
        warn!("TODO: Implement request_close for foreign toplevel handle {:?}", handle.id());
    }

    fn set_rectangle(
        &mut self,
        handle: ForeignToplevelHandle,
        surface: wl_surface::WlSurface, // The wl_surface of the toplevel this rectangle is for
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        info!(
            "Foreign client sets RECTANGLE ({},{},{}x{}) for toplevel handle ID: {:?}, surface {:?}",
            x, y, width, height, handle.id(), surface
        );
        // This is called when the client (e.g., a panel) wants to define a specific rectangle
        // for the toplevel, often for thumbnailing or alternative representations.
        // The compositor can choose to use this information or ignore it.
        // It's not a request to resize the window, but rather a hint for the manager client.
        // Smithay's ForeignToplevelHandle stores this rectangle. We don't need to do much here
        // unless our compositor wants to react to this information specifically.
        debug!("Rectangle hint stored by Smithay for foreign toplevel handle {:?}", handle.id());
    }

    fn request_fullscreen(
        &mut self,
        handle: ForeignToplevelHandle,
        output: Option<wl_output::WlOutput>, // Optional target output
    ) {
        info!(
            "Foreign client requests FULLSCREEN for toplevel handle ID: {:?}, output: {:?}",
            handle.id(), output.as_ref().map(|o| o.id())
        );
        // TODO: Implement fullscreening the window.
        //       Find Smithay Output for wl_output if provided.
        //       Update handle state.
        warn!("TODO: Implement request_fullscreen for foreign toplevel handle {:?}", handle.id());
    }

    fn request_unfullscreen(&mut self, handle: ForeignToplevelHandle) {
        info!("Foreign client requests UNFULLSCREEN for toplevel handle ID: {:?}", handle.id());
        // TODO: Implement unfullscreening.
        warn!("TODO: Implement request_unfullscreen for foreign toplevel handle {:?}", handle.id());
    }


    // --- Callbacks related to compositor-side changes ---

    fn new_foreign_toplevel(&mut self, handle: ForeignToplevelHandle, window: &Window) {
        // This method is NOT part of Smithay's public ForeignToplevelHandler trait.
        // Instead, the compositor is responsible for creating ForeignToplevelHandle
        // instances when new toplevel windows appear and informing the ForeignToplevelManagerState.
        //
        // Example: When a new XDG toplevel window is mapped:
        // let handle = ForeignToplevelHandle::new(); // Create a new handle
        // self.foreign_toplevel_manager_state.new_toplevel(handle.clone(), &window_identifier_or_props);
        // window.set_foreign_toplevel_handle(Some(handle)); // Associate with your Window struct
        //
        // This function here is a conceptual placeholder for that logic.
        info!("Compositor has a NEW foreign toplevel (Window: {:?}, Handle ID: {:?}) that should be advertised.", window.wl_surface(), handle.id());

        // Populate initial state of the handle based on the `window` properties.
        // Smithay's `ForeignToplevelManagerState::new_toplevel` will send the `toplevel` event
        // to manager clients, which then bind to the handle. The handle then sends its state.
        // We need to ensure the handle has the correct initial state *before* clients bind.
        // `ForeignToplevelHandle::send_title`, `send_app_id`, `send_state`, `send_output_enter/leave`, etc.

        // Initial state population:
        // This is often done when the handle is first created and associated with a window.
        // `ForeignToplevelHandle` has methods to send these events.
        if let Some(title) = window.title() { handle.send_title(title); }
        if let WindowKind::Xdg(t) = window.toplevel() {
            if let Some(app_id) = t.app_id() { handle.send_app_id(app_id); }
        }
        // Send current states (maximized, minimized, active, fullscreen)
        let mut states = Vec::new();
        // if window.is_maximized() { states.push(ToplevelState::Maximized); } // Requires methods on Window
        // if window.is_minimized() { states.push(ToplevelState::Minimized); }
        // if window.is_activated() { states.push(ToplevelState::Activated); }
        // if window.is_fullscreen() { states.push(ToplevelState::Fullscreen); }
        handle.send_state(&states);

        // TODO: Send parent if applicable.
        // TODO: Send output enter/leave events based on which outputs the window is on.

        handle.send_done(); // Signal end of initial state batch
        debug!("Initial state sent for new foreign toplevel handle {:?}", handle.id());
    }

    fn foreign_toplevel_closed(&mut self, handle: ForeignToplevelHandle) {
        // This method is also NOT part of Smithay's public ForeignToplevelHandler trait.
        // When a managed toplevel window is closed (e.g., its XDG surface is destroyed),
        // the compositor must:
        // 1. Get the `ForeignToplevelHandle` associated with that window.
        // 2. Call `ForeignToplevelManagerState::closed(handle_id)` or similar on `foreign_toplevel_manager_state`.
        //    This will send the `closed` event on the handle resource to clients.
        //    The handle resource is then typically destroyed by clients or by Smithay.
        // 3. Clean up any internal association of the handle with the (now gone) window.

        info!("Compositor detected foreign toplevel (Handle ID: {:?}) was CLOSED.", handle.id());
        // `self.foreign_toplevel_manager_state().close_handle(&handle);` // Smithay provides this
        // This will send the `closed` event on the handle to all clients.
    }
}

// delegate_foreign_toplevel_manager!(DesktopState); // Needs to be NovaCompositorState

/// Initializes and registers the ZwlrForeignToplevelManagerV1 global.
/// `D` is your main compositor state type.
pub fn init_foreign_toplevel_manager<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<ZwlrForeignToplevelManagerV1, ()> +
       Dispatch<ZwlrForeignToplevelManagerV1, (), D> +
       Dispatch<ZwlrForeignToplevelHandleV1, UserData, D> + // UserData for handle (Smithay manages this via ForeignToplevelHandle)
       ForeignToplevelHandler + 'static,
       // D must also own ForeignToplevelManagerState and be able to access window list (e.g. Space).
{
    info!("Initializing ZwlrForeignToplevelManagerV1 global (wlr-foreign-toplevel-management-unstable-v1)");

    // Create ForeignToplevelManagerState. This state needs to be managed by your compositor (in D).
    // Example: state.foreign_toplevel_manager_state = ForeignToplevelManagerState::new();
    // It will keep track of all active ForeignToplevelHandle objects.

    display.create_global::<D, ZwlrForeignToplevelManagerV1, _>(
        3, // protocol version is 3
        () // GlobalData for the manager (unit)
    )?;

    // Ensure `delegate_foreign_toplevel_manager!(D)` is called in your main compositor state setup.
    // This macro handles:
    // - Dispatching ZwlrForeignToplevelManagerV1 requests (`stop`).
    // - Creating ZwlrForeignToplevelHandleV1 resources when the compositor calls
    //   `ForeignToplevelManagerState::new_toplevel(...)`.
    // - Forwarding requests on ZwlrForeignToplevelHandleV1 to the `ForeignToplevelHandler` methods.

    // The compositor is responsible for:
    // - Creating `ForeignToplevelHandle` instances for each new toplevel window.
    // - Calling `manager_state.new_toplevel(handle, ...)` to advertise it.
    // - Updating the state of these handles (title, app_id, states, outputs) as the window changes.
    //   (e.g., `handle.send_title(...)`, `handle.send_state(...)`, `handle.send_output_enter(...)`).
    // - Calling `manager_state.close_handle(handle)` when a window is closed.

    info!("ZwlrForeignToplevelManagerV1 global initialized.");
    Ok(())
}

// TODO:
// - Window Tracking and Handle Lifecycle:
//   - Implement robust logic to create a `ForeignToplevelHandle` when a new toplevel window appears
//     (e.g., XDG toplevel is mapped).
//   - Store this handle (e.g., in your `Window` struct or a map) and associate it with the window.
//   - Use `ForeignToplevelManagerState::new_toplevel()` to make Smithay aware of it.
//   - When the window's properties change (title, app_id, state like maximized/minimized/active,
//     outputs it's on), call the corresponding `send_*` methods on its `ForeignToplevelHandle`.
//   - When the window is closed, call `ForeignToplevelManagerState::close_handle()` and clean up.
// - Implementing Request Handlers:
//   - Fully implement all `request_*` methods in `ForeignToplevelHandler`. This involves:
//     - Finding the `Window` associated with the given `ForeignToplevelHandle`.
//     - Performing the requested action on that `Window` (e.g., interacting with its XDG shell state
//       to maximize, minimize, activate, close it, or set fullscreen).
//     - After the action is performed (or attempted), update the `ForeignToplevelHandle`'s state
//       by sending the new state array and `done()`.
// - State Synchronization:
//   - Ensure that the state reported by `ZwlrForeignToplevelHandleV1` (title, app_id, states, outputs)
//     is always consistent with the actual state of the window in the compositor.
// - Output Handling:
//   - Correctly send `output_enter` and `output_leave` events on the handle when the window
//     moves between outputs or its visibility on outputs changes.
// - Parent/Child Relationships:
//   - If windows have parent/child relationships (e.g., dialogs), set the parent on the
//     `ForeignToplevelHandle` using `handle.send_parent(parent_handle)`.
// - Full State Integration:
//   - `ForeignToplevelManagerState`, `Space` (or window list), and `SeatState` must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `ForeignToplevelHandler`.
//   - `delegate_foreign_toplevel_manager!(NovaCompositorState);` macro must be used.
// - Testing:
//   - Use clients that implement this protocol (e.g., Waybar, some docks, custom panel tools)
//     to verify that they can list and interact with toplevel windows correctly.
//   - Test all actions: maximize, minimize, activate, close, fullscreen, set_rectangle.
//   - Test state updates when windows change (e.g., title change, activation by other means).
//   - Test with multiple outputs and window movement between them.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod foreign_toplevel_management;
