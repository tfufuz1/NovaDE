//! Manages state and logic for the `wlr-foreign-toplevel-management-unstable-v1` protocol.
//!
//! This protocol allows external clients (like taskbars or docks) to get information
//! about toplevel windows managed by the compositor and request actions on them.

use std::collections::HashMap;
use smithay::{
    reexports::wayland_server::{
        protocol::wl_output::WlOutput, // Though not directly used in this struct, it's relevant for handles
        backend::GlobalId,
        DisplayHandle, Resource, Main, Client,
    },
    reexports::wayland_protocols::unstable::foreign_toplevel_management::v1::server::{
        zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
        zwlr_foreign_toplevel_handle_v1::{
            ZwlrForeignToplevelHandleV1, State as ToplevelHandleState, // Renamed to avoid conflict
            Event as HandleEvent, DoneData, // Other request/event types used in Dispatch
        },
    },
    desktop::{Window, space::SpaceElement},
    utils::{Serial, Rectangle, Size, Logical, SERIAL_COUNTER},
};
use tracing::{info, debug, warn};
use std::sync::{Arc, Mutex as StdMutex};

use crate::compositor::state::DesktopState;

/// Manages the list of clients bound to the foreign toplevel manager global
/// and the handles created for each toplevel window.
#[derive(Default, Debug)]
pub struct ForeignToplevelManagerState {
    /// List of clients that have bound to the `zwlr_foreign_toplevel_manager_v1` global.
    managers: Vec<Main<ZwlrForeignToplevelManagerV1>>,
    /// Tracks active toplevel handles.
    /// Key: GlobalId of the `ZwlrForeignToplevelHandleV1` resource.
    /// Value: The Smithay `Window` this handle represents, and the `Main` resource for the handle.
    known_handles: HashMap<GlobalId, (Window, Main<ZwlrForeignToplevelHandleV1>)>,
}

impl ForeignToplevelManagerState {
    /// Creates a new, empty state for foreign toplevel management.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new manager client to the list and informs it of existing toplevels.
    pub fn add_manager(&mut self, manager: Main<ZwlrForeignToplevelManagerV1>, desktop_state: &DesktopState) {
        info!("Foreign Toplevel Manager bound by client: {:?}", manager.client().map(|c| c.id()));
        let space = desktop_state.space.lock().unwrap();
        for window_element in space.elements() {
            if let Some(window) = window_element.as_window() {
                 if window.toplevel().is_some() {
                    self.create_handle_for_window(&manager, window, &desktop_state.display_handle);
                }
            }
        }
        self.managers.push(manager);
    }

    /// Removes a manager client from the list, typically when it's destroyed or stops.
    pub fn remove_manager(&mut self, manager_resource: &ZwlrForeignToplevelManagerV1) {
        info!("Foreign Toplevel Manager unbound by client: {:?}", manager_resource.client().map(|c| c.id()));
        let manager_id = manager_resource.id();
        self.managers.retain(|m| m.id() != manager_id);

        // Clean up handles associated with the removed manager
        let client_of_removed_manager = manager_resource.client();
        self.known_handles.retain(|_gid, (_w, h)| {
            if h.client().map(|c_handle| c_handle.id()) == client_of_removed_manager.map(|c_mgr| c_mgr.id()) {
                if h.is_alive() {
                    // The protocol doesn't strictly require sending closed if the manager is gone,
                    // as the handles will also be destroyed by the client library or compositor.
                    // However, sending closed can be a good cleanup signal if handles might persist temporarily.
                    // For now, we assume client-side destruction of handles when manager is gone.
                    debug!("Removing handle {:?} associated with stopped manager {:?}", h.id(), manager_id);
                }
                false // Remove from known_handles
            } else {
                true
            }
        });
    }

    /// Creates a `zwlr_foreign_toplevel_handle_v1` for a given window and manager.
    fn create_handle_for_window(
        &mut self,
        manager: &Main<ZwlrForeignToplevelManagerV1>,
        window: &Window,
        dh: &DisplayHandle,
    ) {
        // Avoid creating duplicate handles for the same window under the same manager
        if self.known_handles.values().any(|(w,h)| w.wl_surface().unwrap().id() == window.wl_surface().unwrap().id() && h.client().map(|c|c.id()) == manager.client().map(|c|c.id())) {
            debug!("Handle for window {:?} and manager {:?} already exists.", window.wl_surface().unwrap().id(), manager.id());
            return;
        }

        let xdg_toplevel = window.toplevel().expect("Foreign toplevel should only be for XDG toplevels");
        let client = manager.client().expect("Manager resource should have a client");

        // The UserData for the handle resource will be the Smithay Window itself.
        // This allows the Dispatch<ZwlrForeignToplevelHandleV1, Window> impl to directly access it.
        let handle_resource_main: Main<ZwlrForeignToplevelHandleV1> = match client.create_resource_with_id(dh, 1, window.clone(), |new_resource| {
            new_resource.implement_closure(
                |request, _window_clone: Window| { // _window_clone is the UserData
                    warn!("Request on handle (via closure) - should be via Dispatch on DesktopState: {:?}", request);
                },
                None,
                window.clone()
            )
        }) {
            Ok(res) => res,
            Err(e) => {
                warn!("Failed to create foreign_toplevel_handle resource for client {:?}: {}", client.id(), e);
                return;
            }
        };

        self.known_handles.insert(handle_resource_main.id(), (window.clone(), handle_resource_main.clone()));

        // Send initial state for the new handle
        handle_resource_main.title(xdg_toplevel.title().unwrap_or_default());
        handle_resource_main.app_id(xdg_toplevel.app_id().unwrap_or_default());

        let mut states_vec = Vec::new();
        let current_xdg_state = xdg_toplevel.current_state();
        if current_xdg_state.maximized { states_vec.push(ToplevelHandleState::Maximized); }
        if current_xdg_state.fullscreen { states_vec.push(ToplevelHandleState::Fullscreen); }
        if current_xdg_state.activated { states_vec.push(ToplevelHandleState::Activated); }
        if current_xdg_state.resizing { states_vec.push(ToplevelHandleState::Resizing); }
        // TODO: Minimized state needs to be inferred if not directly in XDG state.
        handle_resource_main.state(&states_vec);

        // TODO: Send output_enter/leave events based on window.outputs()
        // For now, just send done.
        handle_resource_main.done(DoneData { serial: SERIAL_COUNTER.next_serial() });

        info!("Created foreign_toplevel_handle {:?} for window {:?}", handle_resource_main.id(), window.wl_surface().unwrap().id());
    }

    /// Called when a new XDG toplevel window is mapped and should be advertised.
    pub fn window_mapped(&mut self, window: &Window, dh: &DisplayHandle) {
        if window.toplevel().is_none() { return; }
        info!("Window mapped, notifying foreign toplevel managers: {:?}", window.wl_surface().unwrap().id());
        for manager in self.managers.iter().filter(|m| m.is_alive()) {
            self.create_handle_for_window(manager, window, dh);
        }
    }

    /// Called when an XDG toplevel window is unmapped (closed/destroyed).
    pub fn window_unmapped(&mut self, window: &Window) {
        if window.toplevel().is_none() { return; }
        let surface_id_to_unmap = window.wl_surface().unwrap().id();
        info!("Window unmapped/closed, notifying foreign toplevel managers: {:?}", surface_id_to_unmap);

        let handles_to_close: Vec<Main<ZwlrForeignToplevelHandleV1>> = self.known_handles.values()
            .filter(|(w, _h)| w.wl_surface().map_or(false, |s| s.id() == surface_id_to_unmap))
            .map(|(_w, h)| h.clone())
            .collect();

        for handle_resource in handles_to_close {
            if handle_resource.is_alive() {
                handle_resource.closed();
                // Protocol requires done after closed if it's the last event in a group.
                handle_resource.done(DoneData{ serial: SERIAL_COUNTER.next_serial()});
                info!("Sent 'closed' for foreign_toplevel_handle {:?}", handle_resource.id());
            }
            self.known_handles.remove(&handle_resource.id());
        }
    }

    /// Called when an XDG toplevel's title changes.
    pub fn window_title_changed(&mut self, window: &Window, new_title: String) {
        if window.toplevel().is_none() { return; }
        info!("Window title changed for {:?}, notifying foreign toplevels.", window.wl_surface().unwrap().id());
        self.for_each_handle_for_window(window, |handle| {
            handle.title(new_title.clone());
            handle.done(DoneData { serial: SERIAL_COUNTER.next_serial() });
        });
    }

    /// Called when an XDG toplevel's app_id changes.
    pub fn window_appid_changed(&mut self, window: &Window, new_app_id: String) {
        if window.toplevel().is_none() { return; }
        info!("Window app_id changed for {:?}, notifying foreign toplevels.", window.wl_surface().unwrap().id());
        self.for_each_handle_for_window(window, |handle| {
            handle.app_id(new_app_id.clone());
            handle.done(DoneData { serial: SERIAL_COUNTER.next_serial() });
        });
    }

    /// Called when an XDG toplevel's state (maximized, fullscreen, activated) changes.
    pub fn window_state_changed(&mut self, window: &Window) {
        if window.toplevel().is_none() { return; }
        let xdg_toplevel = window.toplevel().unwrap();
        let current_xdg_state = xdg_toplevel.current_state();
        let mut states_vec = Vec::new();
        if current_xdg_state.maximized { states_vec.push(ToplevelHandleState::Maximized); }
        if current_xdg_state.fullscreen { states_vec.push(ToplevelHandleState::Fullscreen); }
        if current_xdg_state.activated { states_vec.push(ToplevelHandleState::Activated); }
        if current_xdg_state.resizing { states_vec.push(ToplevelHandleState::Resizing); }
        // TODO: Minimized state needs better tracking. If unmapped but alive, it's likely minimized.
        // This requires DesktopState to pass that info or for this method to query the Space.
        // For now, relying on XDG states.

        info!("Window state changed for {:?}, notifying foreign toplevels. States: {:?}", window.wl_surface().unwrap().id(), states_vec);
        self.for_each_handle_for_window(window, |handle| {
            handle.state(&states_vec);
            handle.done(DoneData { serial: SERIAL_COUNTER.next_serial() });
        });
    }

    /// Iterates over all known, live handles for a given window and applies a function.
    fn for_each_handle_for_window<F>(&self, window: &Window, mut func: F)
    where
        F: FnMut(&Main<ZwlrForeignToplevelHandleV1>),
    {
        let window_surface_id = match window.wl_surface() {
            Some(s) => s.id(),
            None => return, // Should not happen for a valid window
        };

        for (_gid, (w, handle_resource)) in &self.known_handles {
            if w.wl_surface().map_or(false, |s| s.id() == window_surface_id) {
                if handle_resource.is_alive() {
                    func(handle_resource);
                }
            }
        }
    }

    /// Retrieves the Smithay `Window` associated with a given foreign toplevel handle resource.
    pub fn get_window_for_handle(&self, handle_resource: &ZwlrForeignToplevelHandleV1) -> Option<Window> {
        self.known_handles.get(&handle_resource.id()).map(|(w, _h)| w.clone())
    }
}

/// User data to be associated with the `ZwlrForeignToplevelManagerV1` resource (per client).
/// Empty for now, as global state in `ForeignToplevelManagerState` tracks managers.
#[derive(Default, Debug)]
pub struct ForeignToplevelManagerClientData;

// ForeignToplevelHandleUserData is not used because the Window object itself is used as UserData for the handle resource.
// This simplifies lookup in the Dispatch<ZwlrForeignToplevelHandleV1, Window> implementation.
