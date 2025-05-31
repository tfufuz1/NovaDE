use smithay::{
    reexports::wayland_server::{
        protocol::{wl_seat::WlSeat, wl_surface::WlSurface, wl_output::WlOutput as WaylandOutput},
        DisplayHandle,
    },
    utils::{Logical, Point, Rectangle, Serial, Size},
    wayland::{
        seat::Seat,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface,
            XdgShellHandler, XdgShellState, XdgWmBaseClientData,
            ToplevelState as XdgToplevelState,
            XdgSurfaceConfigure,
            ResizeEdge as XdgResizeEdge, // Explicit import
        },
        // Ensure ToplevelState is imported for the helper function.
        // It's already aliased as XdgToplevelStateSmithay in the file.
        // If not, we'd add:
        // shell::xdg::ToplevelState as SmithayToplevelState,
    },
    input::SeatHandler, // SeatHandler might not be directly needed in XdgShellHandler trait methods
};
use std::sync::Arc;
use smithay::wayland::shell::xdg::ToplevelState as SmithayXdgToplevelState;

use crate::compositor::{
    core::state::DesktopState, // Changed from NovadeCompositorState
    shell::xdg_shell::types::{DomainWindowIdentifier, ManagedWindow}, // Changed path
    // Removed unused XdgShellError import
    errors::XdgShellError, // Import the error type
};
use uuid::Uuid; // For DomainWindowIdentifier if not already via types

// --- XDG Decoration Imports ---
use smithay::wayland::shell::xdg::decoration::{
    XdgDecorationHandler, XdgToplevelDecoration, Mode as XdgDecorationMode, ServerDecorationState
};
// ToplevelSurface is already imported above.

// Helper function to find ManagedWindow by WlSurface - takes &mut DesktopState to allow modification if needed, though current use is read-only
fn find_managed_window_by_wl_surface_mut(desktop_state: &mut DesktopState, surface: &WlSurface) -> Option<Arc<ManagedWindow>> {
    desktop_state.windows.values()
        .find(|win_arc| win_arc.wl_surface().as_ref() == Some(surface))
        .cloned()
}
// Read-only version
fn find_managed_window_by_wl_surface(desktop_state: &DesktopState, surface: &WlSurface) -> Option<Arc<ManagedWindow>> {
    desktop_state.windows.values()
        .find(|win_arc| win_arc.wl_surface().as_ref() == Some(surface))
        .cloned()
}

// Helper function to convert ManagedWindow's state to Smithay's XDG ToplevelState
fn managed_to_xdg_toplevel_state(win_state: &crate::compositor::shell::xdg_shell::types::WindowState) -> SmithayXdgToplevelState {
    let mut xdg_state = SmithayXdgToplevelState::empty();
    if win_state.maximized { xdg_state.set(SmithayXdgToplevelState::MAXIMIZED, true); }
    if win_state.fullscreen { xdg_state.set(SmithayXdgToplevelState::FULLSCREEN, true); }
    // Note: Smithay's XdgToplevelState does not have a 'MINIMIZED' state.
    // Minimized usually means unmapped. 'ACTIVATED' and 'SUSPENDED' are present.
    // 'SUSPENDED' could be a potential mapping for minimized if the window is not unmapped,
    // or we rely on the unmapped state to signal this implicitly to foreign toplevel consumers.
    // For now, only map activated. If a window is minimized, it will likely be unmapped,
    // and `remove_toplevel` would be called.
    if win_state.activated { xdg_state.set(SmithayXdgToplevelState::ACTIVATED, true); }
    // To represent 'minimized' or 'hidden', one might consider if the window is mapped or not
    // in conjunction with its other states, or use a custom protocol extension if needed by the taskbar.
    // For zwlr_foreign_toplevel_manager_v1, unmapping is the primary way to indicate a window is gone/hidden.
    xdg_state
}

impl XdgShellHandler for DesktopState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG New Toplevel requested");
        let domain_window_id = DomainWindowIdentifier::new_v4();

        let mut managed_window = ManagedWindow::new_toplevel(surface.clone(), domain_window_id);
        
        let initial_geometry = managed_window.current_geometry;

        surface.with_pending_state(|xdg_state| {
            xdg_state.size = Some(initial_geometry.size);
        });
        let configure_serial = surface.send_configure();
        managed_window.last_configure_serial = Some(configure_serial); // Set serial before Arc::new

        let window_arc = Arc::new(managed_window);
        self.windows.insert(domain_window_id, window_arc.clone());

        tracing::info!("New XDG Toplevel (domain_id: {:?}, compositor_id: {:?}) created and configured with serial {:?}. Awaiting map.",
                     domain_window_id, window_arc.id, configure_serial);
    }

    fn new_popup(&mut self, surface: PopupSurface, _client_data: &XdgWmBaseClientData) {
        let parent_wl_surface = match surface.get_parent_surface() {
            Some(s) => s,
            None => {
                tracing::error!("Popup {:?} created without a parent surface. Destroying popup.", surface.wl_surface().id());
                surface.send_popup_done();
                return;
            }
        };

        let parent_managed_window_arc = match find_managed_window_by_wl_surface(self, &parent_wl_surface) {
             Some(arc) => arc,
             None => {
                tracing::error!("Parent WlSurface {:?} for popup {:?} does not correspond to a known ManagedWindow. Destroying popup.", parent_wl_surface.id(), surface.wl_surface().id());
                surface.send_popup_done();
                return;
             }
        };
        
        tracing::info!(popup_surface_id = ?surface.wl_surface().id(), parent_surface_id = ?parent_wl_surface.id(), "XDG New Popup requested");

        let mut managed_popup = ManagedWindow::new_popup(surface.clone(), parent_managed_window_arc.domain_id(), Some(parent_managed_window_arc.clone()));

        let positioner_state = surface.get_positioner();
        let parent_geometry = parent_managed_window_arc.geometry();
        let popup_size = positioner_state.get_size().unwrap_or_else(|| Size::from((10,10)));
        let anchor_rect = positioner_state.get_anchor_rect().unwrap_or_else(|| Rectangle::from_loc_and_size((0,0), (0,0)));
        let offset = positioner_state.get_offset().unwrap_or_else(|| (0,0).into());

        let popup_loc_relative_to_parent_anchor = Point::from((anchor_rect.loc.x + offset.x, anchor_rect.loc.y + offset.y));
        let popup_global_loc = parent_geometry.loc + popup_loc_relative_to_parent_anchor;

        managed_popup.current_geometry = Rectangle::from_loc_and_size(popup_global_loc, popup_size);

        let configure_serial = surface.send_configure();
        managed_popup.last_configure_serial = Some(configure_serial);

        let popup_arc = Arc::new(managed_popup);
        // Storing popups in the main self.windows map.
        // This might need differentiation later if popups have very different management needs.
        self.windows.insert(popup_arc.domain_id(), popup_arc.clone());

        // Optional: Store Arc<ManagedWindow> in PopupSurface's user data
        surface.user_data().insert_if_missing_threadsafe(|| popup_arc.clone());
        // Add destruction hook as in existing code if desired (good for cleanup logging)
        let popup_wl_surface_clone = surface.wl_surface().clone(); // Clone for the closure
        smithay::wayland::compositor::add_destruction_hook(&popup_wl_surface_clone, move |data_map_of_destroyed_surface| {
            if data_map_of_destroyed_surface.get::<Arc<ManagedWindow>>().is_some() {
                tracing::info!("ManagedWindow Arc for XDG Popup (surface {:?}) was present in user_data upon destruction.", popup_wl_surface_clone.id());
            }
        });

        tracing::info!("New XDG Popup (domain_id: {:?}, compositor_id: {:?}) created, configured with serial {:?}. Parent: {:?}",
                     popup_arc.domain_id(), popup_arc.id, configure_serial, parent_managed_window_arc.id);
    }

    fn map_toplevel(&mut self, surface: &ToplevelSurface) {
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "XDG Toplevel map request");

        if let Some(window_arc) = find_managed_window_by_wl_surface(self, wl_surface) {
            {
                let mut state_guard = window_arc.state.write().unwrap();
                state_guard.is_mapped = true;
            }
            self.space.map_window(window_arc.clone(), window_arc.geometry().loc, true);
            self.foreign_toplevel_state.new_toplevel(window_arc.clone());
            tracing::info!("Notified foreign_toplevel_manager about new toplevel: {:?}", window_arc.id);
            tracing::info!("XDG Toplevel {:?} mapped to space at {:?}", window_arc.id, window_arc.geometry().loc);
            self.space.damage_all_outputs();
        } else {
            tracing::warn!("Map request for unknown XDG Toplevel (surface_id: {:?})", wl_surface.id());
        }
    }

    fn unmap_toplevel(&mut self, surface: &ToplevelSurface) {
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "XDG Toplevel unmap request");
        if let Some(window_arc) = find_managed_window_by_wl_surface(self, wl_surface) {
            {
                let mut state_guard = window_arc.state.write().unwrap();
                state_guard.is_mapped = false;
            }
            self.space.unmap_window(&window_arc);
            self.foreign_toplevel_state.remove_toplevel(&window_arc);
            tracing::info!("Notified foreign_toplevel_manager about removed toplevel: {:?}", window_arc.id);
            tracing::info!("XDG Toplevel {:?} unmapped from space.", window_arc.id);
            self.space.damage_all_outputs();
        } else {
            tracing::warn!("Unmap request for unknown XDG Toplevel (surface_id: {:?})", wl_surface.id());
        }
    }
    
    fn ack_configure(&mut self, surface: WlSurface, configure: xdg_surface::Configure) {
        let xdg_surface_data = surface.data_map().get::<XdgSurfaceUserData>().unwrap();
        tracing::debug!(surface_id = ?surface.id(), serial = ?configure.serial, "XDG Surface ack_configure received. Initial: {}", xdg_surface_data.initial_configure_sent);

        // Smithay's XdgShellState::handle_ack_configure can be used for basic validation if needed,
        // but often custom logic is applied here.
        // if let Err(e) = XdgShellState::handle_ack_configure(&surface, configure.clone()) {
        //     tracing::warn!("Error handling ack_configure via Smithay's XdgShellState: {}", e);
        // }

        if let Some(window_arc) = find_managed_window_by_wl_surface(self, &surface) {
            // The configure.data is SurfaceCachedState. If it contains new_window_geometry,
            // it means the client is acknowledging a resize.
            // We might need to update our ManagedWindow.current_geometry and re-map in space.
            // However, Smithay's ToplevelSurface itself updates its current_state upon ack.
            // Our ManagedWindow should ideally sync from ToplevelSurface::current_state() if needed.
            // For now, just log.
            if xdg_surface_data.initial_configure_sent && !window_arc.state.read().unwrap().is_mapped {
                 tracing::debug!("ack_configure for initial configure of window {:?}. Window mapped state: {}", window_arc.id, window_arc.state.read().unwrap().is_mapped);
            }
        } else {
            tracing::warn!("ack_configure for unknown surface: {:?}", surface.id());
        }
    }

    // Using find_managed_window_by_wl_surface (read-only) for these setters
    fn toplevel_request_set_title(&mut self, surface: &ToplevelSurface, title: String) {
        if let Some(window_arc) = find_managed_window_by_wl_surface(self, surface.wl_surface()) {
            let mut state_guard = window_arc.state.write().unwrap();
            state_guard.title = Some(title.clone());
            drop(state_guard);
            self.foreign_toplevel_state.update_title(&window_arc, title.clone());
            tracing::debug!("Notified foreign_toplevel_manager about title change for {:?}", window_arc.id);
            tracing::info!("Window {:?} requested title change to: {}", window_arc.id, title);
        }
    }

    fn toplevel_request_set_app_id(&mut self, surface: &ToplevelSurface, app_id: String) {
         if let Some(window_arc) = find_managed_window_by_wl_surface(self, surface.wl_surface()) {
            let mut state_guard = window_arc.state.write().unwrap();
            state_guard.app_id = Some(app_id.clone());
            drop(state_guard);
            self.foreign_toplevel_state.update_app_id(&window_arc, app_id.clone());
            tracing::debug!("Notified foreign_toplevel_manager about app_id change for {:?}", window_arc.id);
            tracing::info!("Window {:?} requested app_id change to: {}", window_arc.id, app_id);
        }
    }

    fn set_parent_request(&mut self, surface: &ToplevelSurface, parent_surface_opt: Option<&ToplevelSurface>) {
        let child_wl_surface = surface.wl_surface();
        match parent_surface_opt {
            Some(parent_surface) => {
                let parent_wl_surface = parent_surface.wl_surface();
                tracing::info!(
                    child_surface_id = ?child_wl_surface.id(),
                    parent_surface_id = ?parent_wl_surface.id(),
                    "XDG Toplevel set_parent request received."
                );
                // Smithay handles the actual parent linking for the XDG ToplevelSurface.
                // If our ManagedWindow.parent field needs to be updated for internal compositor logic
                // (beyond what ToplevelSurface::parent() provides), that logic would go here
                // or preferably in ManagedWindow::self_update() if it can access the necessary state.
                // For now, logging is sufficient as per the refined subtask plan.
            }
            None => {
                tracing::info!(
                    child_surface_id = ?child_wl_surface.id(),
                    "XDG Toplevel set_parent request received with None (unset parent)."
                );
                // Similar to setting a parent, if ManagedWindow.parent needs to be cleared,
                // that logic would be here or in self_update().
            }
        }
        // The client is expected to commit the child surface for the parent change to be fully applied by Smithay.
    }

    fn toplevel_request_set_maximized(&mut self, surface: &ToplevelSurface) { // Renamed from set_maximized_request
        if let Some(window_arc) = find_managed_window_by_wl_surface(self, surface.wl_surface()) {
            tracing::info!("Window {:?} requested set_maximized", window_arc.id);
            let mut win_state_guard = window_arc.state.write().unwrap();
            if !win_state_guard.maximized {
                // Save current geometry before maximizing. Use window_arc.geometry() for current actual geometry.
                win_state_guard.saved_pre_action_geometry = Some(window_arc.geometry());
                tracing::debug!("Saved pre-maximize geometry: {:?}", win_state_guard.saved_pre_action_geometry);
            }
            win_state_guard.maximized = true;
            win_state_guard.fullscreen = false; // Ensure not fullscreen
            let current_win_state_clone = win_state_guard.clone(); // Clone for the helper
            drop(win_state_guard);

            let new_xdg_state = managed_to_xdg_toplevel_state(&current_win_state_clone);
            self.foreign_toplevel_state.update_state(&window_arc, new_xdg_state);
            tracing::debug!("Notified foreign_toplevel_manager about state change for {:?}: {:?}", window_arc.id, new_xdg_state);

            let maximized_geometry = self.space.outputs()
                .next()
                .and_then(|o| self.space.output_geometry(o))
                .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0), (800,600))); // Fallback

            surface.with_pending_state(|xdg_state| {
                xdg_state.states.set(XdgToplevelStateSmithay::Maximized); // Use aliased XdgToplevelStateSmithay
                xdg_state.size = Some(maximized_geometry.size);
            });
            let serial = surface.send_configure();
            // TODO: Store serial in ManagedWindow if necessary for ack_configure logic
            // window_arc.last_configure_serial = Some(serial); // Needs interior mutability or map update
            tracing::info!("Window {:?} set_maximized request. Sent configure with serial {:?}, size {:?}.", window_arc.id, serial, maximized_geometry.size);
            self.space.damage_all_outputs();
        }
    }

    fn toplevel_request_unset_maximized(&mut self, surface: &ToplevelSurface) { // Renamed from unset_maximized_request
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG Toplevel unset_maximized request.");
        if let Some(window_arc) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())).cloned() {
            let mut win_state_guard = window_arc.state.write().unwrap();
            win_state_guard.maximized = false;
            
            let restored_geometry = win_state_guard.saved_pre_action_geometry.take();
            let (restored_pos, restored_size) = if let Some(geom) = restored_geometry {
                tracing::debug!("Restoring to saved geometry: {:?}", geom);
                // Update internal state to reflect restored position and size
                win_state_guard.position = geom.loc;
                win_state_guard.size = geom.size;
                (Some(geom.loc), Some(geom.size))
            } else {
                tracing::warn!("No saved geometry found for unmaximize, using current size.");
                (None, Some(win_state_guard.size)) // Keep current position, restore to current size (or a default)
            };
            let current_win_state_clone = win_state_guard.clone();
            drop(win_state_guard);

            let new_xdg_state = managed_to_xdg_toplevel_state(&current_win_state_clone);
            self.foreign_toplevel_state.update_state(&window_arc, new_xdg_state);
            tracing::debug!("Notified foreign_toplevel_manager about state change for {:?}: {:?}", window_arc.id, new_xdg_state);

            surface.with_pending_state(|xdg_state| {
                xdg_state.states.set(XdgToplevelState::Maximized, false);
                xdg_state.size = restored_size;
                // XDG protocol doesn't directly set position on configure, client usually handles it.
                // If we want to force a position, it would be managed by compositor layout logic after this.
            });
            let _serial = surface.send_configure();
            tracing::debug!("Unmaximized request for {:?}, sent configure with size {:?}, serial {:?}", surface.wl_surface().id(), restored_size, _serial);
        }
    }

    fn toplevel_request_set_minimized(&mut self, surface: &ToplevelSurface) { // Renamed from set_minimized_request
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG Toplevel set_minimized request.");
        if let Some(window_arc) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())).cloned() {
            let mut win_state_guard = window_arc.state.write().unwrap();
            win_state_guard.minimized = true;
            // Client is expected to unmap itself. Compositor might unmap from space if client misbehaves.
            // No configure is sent for minimized state typically.
            let current_win_state_clone = win_state_guard.clone();
            drop(win_state_guard);

            // Even if minimized means unmapped, we might want to update the state for foreign toplevel
            // if it has a concept of "hidden" or "suspended" that maps to minimized.
            // For now, our helper doesn't map minimized to anything in XdgToplevelState.
            // If the window unmaps, remove_toplevel will be called. If it stays mapped but "minimized",
            // this state update might be relevant if XdgToplevelState had a minimized/hidden flag.
            let new_xdg_state = managed_to_xdg_toplevel_state(&current_win_state_clone);
            self.foreign_toplevel_state.update_state(&window_arc, new_xdg_state);
            tracing::debug!("Notified foreign_toplevel_manager about state change (minimized) for {:?}: {:?}", window_arc.id, new_xdg_state);
            tracing::debug!("Minimized request for {:?}. Client should unmap.", surface.wl_surface().id());
        }
    }

    fn toplevel_request_set_fullscreen(&mut self, surface: &ToplevelSurface, output_opt: Option<&WaylandOutput>) { // Renamed from set_fullscreen_request
        if let Some(window_arc) = find_managed_window_by_wl_surface(self, surface.wl_surface()) {
            tracing::info!("Window {:?} requested set_fullscreen on output {:?}", window_arc.id, output_opt.map(|o| o.id()));
            let mut win_state_guard = window_arc.state.write().unwrap();
            if !win_state_guard.fullscreen {
                 win_state_guard.saved_pre_action_geometry = Some(window_arc.geometry());
                 tracing::debug!("Saved pre-fullscreen geometry for {:?}: {:?}", window_arc.id, win_state_guard.saved_pre_action_geometry);
            }
            win_state_guard.fullscreen = true;
            win_state_guard.maximized = false;
            let current_win_state_clone = win_state_guard.clone();
            drop(win_state_guard);

            let new_xdg_state = managed_to_xdg_toplevel_state(&current_win_state_clone);
            self.foreign_toplevel_state.update_state(&window_arc, new_xdg_state);
            tracing::debug!("Notified foreign_toplevel_manager about state change for {:?}: {:?}", window_arc.id, new_xdg_state);

            let target_output = output_opt.or_else(|| self.space.outputs().next());
            let fullscreen_geometry = target_output.and_then(|o| self.space.output_geometry(o));

            surface.with_pending_state(|xdg_state| {
                xdg_state.states.set(XdgToplevelStateSmithay::Fullscreen);
                xdg_state.size = fullscreen_geometry.map(|g| g.size);
            });
            let serial = surface.send_configure();
            // window_arc.last_configure_serial = Some(serial); // Update if needed

            if let Some(geo) = fullscreen_geometry {
                // TODO: This should ideally interact with a layout manager.
                // For now, we assume client will position itself at (0,0) on the output.
                // Or, we could adjust window_arc's position in space here.
                // self.space.map_window(&window_arc, geo.loc, false); // Example: if map_window handles position update
                tracing::info!("Window {:?} set_fullscreen request. Target output geometry {:?}. Sent configure with serial {:?}.", window_arc.id, geo, serial);
            } else {
                tracing::warn!("Could not determine fullscreen geometry for window {:?}. Sent configure with serial {:?}.", window_arc.id, serial);
            }
            self.space.damage_all_outputs();
        }
    }

    fn toplevel_request_unset_fullscreen(&mut self, surface: &ToplevelSurface) { // Renamed from unset_fullscreen_request
         if let Some(window_arc) = find_managed_window_by_wl_surface(self, surface.wl_surface()) {
            tracing::info!("Window {:?} requested unset_fullscreen", window_arc.id);
            let mut win_state_guard = window_arc.state.write().unwrap();
            win_state_guard.fullscreen = false;
            let restored_size = win_state_guard.saved_pre_action_geometry.map(|g| g.size);
            // Position restoration would be handled by layout logic or client, or from saved_pre_action_geometry.loc
            if let Some(saved_geom) = win_state_guard.saved_pre_action_geometry {
                 tracing::debug!("Restored size for {:?} to {:?}. Position would be {:?}", window_arc.id, saved_geom.size, saved_geom.loc);
            }
            let current_win_state_clone = win_state_guard.clone();
            drop(win_state_guard);

            let new_xdg_state = managed_to_xdg_toplevel_state(&current_win_state_clone);
            self.foreign_toplevel_state.update_state(&window_arc, new_xdg_state);
            tracing::debug!("Notified foreign_toplevel_manager about state change for {:?}: {:?}", window_arc.id, new_xdg_state);

            surface.with_pending_state(|xdg_state| {
                xdg_state.states.unset(XdgToplevelStateSmithay::Fullscreen);
                xdg_state.size = restored_size;
            });
            let serial = surface.send_configure();
            // window_arc.last_configure_serial = Some(serial);
            tracing::info!("Window {:?} unset_fullscreen request. Sent configure with serial {:?}, proposed size {:?}.", window_arc.id, serial, restored_size);
            self.space.damage_all_outputs();
        }
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) { // Name changed to surface for consistency with prompt
        let wl_surface = surface.wl_surface(); // Use wl_surface for clarity
        tracing::info!(surface_id = ?wl_surface.id(), "XDG Toplevel destroyed by client");
        if let Some(window_arc) = find_managed_window_by_wl_surface(self, wl_surface) {
            self.foreign_toplevel_state.remove_toplevel(&window_arc); // Notify before removing from space/map
            tracing::info!("Notified foreign_toplevel_manager about destroyed toplevel: {:?}", window_arc.id);
            self.space.unmap_window(&window_arc);
            self.windows.remove(&window_arc.domain_id());
            tracing::info!("ManagedWindow {:?} (domain: {:?}) removed due to toplevel destruction.", window_arc.id, window_arc.domain_id());
            self.space.damage_all_outputs();
        } else { // This case was missing in the prompt's version, good to keep.
             tracing::warn!("Destroyed toplevel {:?} was not found in self.windows by its WlSurface.", wl_surface.id());
        }
    }

    fn popup_destroyed(&mut self, surface: PopupSurface) { // Name changed to surface for consistency
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "XDG Popup destroyed by client");
        if let Some(popup_arc) = find_managed_window_by_wl_surface(self, wl_surface) {
            // Popups might not be directly in space in the same way, or might be handled by space's window hierarchy
            self.windows.remove(&popup_arc.domain_id());
            tracing::info!("ManagedWindow (popup) {:?} (domain: {:?}) removed from self.windows due to popup destruction.", popup_arc.id, popup_arc.domain_id());
        }
        // The destruction hook for UserDataMap (if used in new_popup) will also run.
        self.space.damage_all_outputs(); // Damage parent area or whole output
    }

    // Other required methods from the prompt (some might be stubs in existing file)
    fn toplevel_request_set_minimized(&mut self, surface: &ToplevelSurface) {
        if let Some(window_arc) = find_managed_window_by_wl_surface(self, surface.wl_surface()) {
            tracing::info!("Window {:?} requested set_minimized", window_arc.id);
            let mut win_state_guard = window_arc.state.write().unwrap();
            win_state_guard.minimized = true;
            let current_win_state_clone = win_state_guard.clone();
            drop(win_state_guard);

            let new_xdg_state = managed_to_xdg_toplevel_state(&current_win_state_clone);
            self.foreign_toplevel_state.update_state(&window_arc, new_xdg_state);
            tracing::debug!("Notified foreign_toplevel_manager about state change (set_minimized) for {:?}: {:?}", window_arc.id, new_xdg_state);

            self.space.unmap_window(&window_arc);
            tracing::info!("Window {:?} unmapped from space due to minimization request.", window_arc.id);
            self.space.damage_all_outputs();
        } else {
            tracing::warn!("Set_minimized request for unknown toplevel: {:?}", surface.wl_surface().id());
        }
    }

    fn toplevel_request_show_window_menu(&mut self, surface: &ToplevelSurface, seat: &wl_seat::WlSeat, serial: Serial, position: Point<i32, Logical>) {
        tracing::info!("Request to show window menu for {:?} at {:?} by seat {:?}", surface.wl_surface().id(), position, seat.id());
        // TODO: Implement window menu if applicable. This might involve sending an event to the domain layer
        // or opening a special compositor-drawn menu.
    }

    fn popup_grab(&mut self, surface: &PopupSurface, seat: &wl_seat::WlSeat, serial: Serial) {
        tracing::info!("Popup grab requested for {:?} by seat {:?}", surface.wl_surface().id(), seat.id());
        // Smithay's default XdgShellHandler might take care of the grab logic.
        // If we override the default grab (which we are by implementing XdgShellHandler),
        // we need to initiate the grab explicitly.
        let xdg_shell_state = self.xdg_shell_state(); // Get &mut XdgShellState
        match smithay::wayland::shell::xdg::grab_popup(xdg_shell_state, surface, seat, serial) {
            Ok(_) => tracing::debug!("Popup grab initiated successfully for {:?}.", surface.wl_surface().id()),
            Err(err) => tracing::warn!("Failed to start popup grab for {:?}: {}", surface.wl_surface().id(), err),
        }
    }

    fn reposition_popup(&mut self, surface: &PopupSurface, positioner: &PositionerState, token: u32) {
        tracing::info!("Reposition popup request for {:?} with token {}", surface.wl_surface().id(), token);
        // This is a simplified handler. A full implementation would:
        // 1. Find the ManagedWindow for this popup.
        // 2. Calculate new geometry based on parent and new positioner state.
        // 3. Update ManagedWindow.current_geometry.
        // 4. Send a new configure to the popup reflecting the new position/size.
        // Smithay's default handler does this. By overriding, we need to do it.
        // For now, just sending a basic configure. This might not be enough for true repositioning.
        // surface.send_configure(); // This is too basic.
        // The default handler does something like:
        // surface.with_pending_state(|popup_state: &mut smithay::wayland::shell::xdg::PopupState| {
        //     popup_state.geometry = crate::some_module::calculate_popup_geometry(surface, positioner); // You'd need this helper
        // });
        // surface.send_repositioned(token); // This is the correct method in Smithay 0.10+

        // For now, as a placeholder matching closer to what Smithay might do if we calculated geometry:
        surface.send_configure(); // This tells the client its configure is done.
                                  // Actual repositioning would need geometry calculation and applying it.
        tracing::warn!("Popup repositioning logic for {:?} is placeholder. Sent basic configure.", surface.wl_surface().id());
    }

    // --- Requests for move and resize ---
    // These are initiated by the client, typically after a pointer click/drag on a decoration or border.
    // Smithay's default handlers might do basic validation.
    // Our role is to integrate this with our window management (interactive_ops).

    fn toplevel_request_move(&mut self, surface: &ToplevelSurface, seat_handle: &wl_seat::WlSeat, serial: Serial) {
        if let Some(window_arc) = find_managed_window_by_wl_surface(self, surface.wl_surface()) {
            tracing::info!("Window {:?} requested interactive move via client request (serial: {:?})", window_arc.id, serial);
            // TODO: Validate seat, serial, and current input state.
            // Then, potentially start an interactive move operation using a pointer grab.
            // let smithay_seat = Seat::<Self>::from_resource(seat_handle).ok_or_else(|| XdgShellError::OperationNotPermitted("Seat not found".to_string()))?;
            // interactive_ops::start_interactive_move(self, &smithay_seat, window_arc.clone(), serial);
            tracing::warn!("Interactive move op (from client request) not fully implemented for window {:?}.", window_arc.id);
        }
    }
    fn toplevel_request_resize(&mut self, surface: &ToplevelSurface, seat_handle: &wl_seat::WlSeat, serial: Serial, edges: XdgResizeEdge) {
        if let Some(window_arc) = find_managed_window_by_wl_surface(self, surface.wl_surface()) {
            tracing::info!("Window {:?} requested interactive resize via client request (edges: {:?}, serial: {:?})", window_arc.id, edges, serial);
            // TODO: Validate seat, serial, edges, and current input state.
            // Then, potentially start an interactive resize operation using a pointer grab.
            // let smithay_seat = Seat::<Self>::from_resource(seat_handle).ok_or_else(|| XdgShellError::OperationNotPermitted("Seat not found".to_string()))?;
            // interactive_ops::start_interactive_resize(self, &smithay_seat, window_arc.clone(), serial, edges);
            tracing::warn!("Interactive resize op (from client request) not fully implemented for window {:?}.", window_arc.id);
        }
    }
}

impl XdgDecorationHandler for DesktopState {
    fn xdg_decoration_state(&mut self) -> &mut ServerDecorationState {
        &mut self.xdg_decoration_state
    }

    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        tracing::debug!(surface_id = ?toplevel.wl_surface().id(), "New XDG Toplevel Decoration created");
        // The ToplevelSurface itself is what gets decorated.
        // Smithay's XdgDecorationState handles associating the XdgToplevelDecoration object.
        // The client will typically request a mode. If not, default to ServerSide.
        // We can set an initial preferred mode on the toplevel if not set by client.
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(XdgDecorationMode::ServerSide);
        });
         // No need to call send_configure here, mode negotiation handles it.

        if let Some(window_arc) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(toplevel.wl_surface())).cloned() {
            let mut manager_data = window_arc.manager_data.write().unwrap();
            manager_data.decorations = true; // Default to SSD
            tracing::info!("Defaulted window {:?} to Server-Side Decorations.", window_arc.id);
        }
    }

    fn toplevel_request_show_window_menu(&mut self, surface: &ToplevelSurface, seat: &wl_seat::WlSeat, serial: Serial, position: Point<i32, Logical>) {
        tracing::info!("Request to show window menu for {:?} at {:?} by seat {:?}", surface.wl_surface().id(), position, seat.id());
        // TODO: Implement window menu if applicable. This might involve sending an event to the domain layer
        // or opening a special compositor-drawn menu.
    }

    fn popup_grab(&mut self, surface: &PopupSurface, seat: &wl_seat::WlSeat, serial: Serial) {
        tracing::info!("Popup grab requested for {:?} by seat {:?}", surface.wl_surface().id(), seat.id());
        let xdg_shell_state = self.xdg_shell_state();
        match smithay::wayland::shell::xdg::grab_popup(xdg_shell_state, surface, seat, serial) {
            Ok(_) => tracing::debug!("Popup grab initiated successfully for {:?}.", surface.wl_surface().id()),
            Err(err) => tracing::warn!("Failed to start popup grab for {:?}: {}", surface.wl_surface().id(), err),
        }
    }

    fn reposition_popup(&mut self, surface: &PopupSurface, positioner: &PositionerState, token: u32) {
        tracing::info!("Reposition popup request for {:?} with token {}", surface.wl_surface().id(), token);
        // Placeholder: Actual repositioning would involve recalculating geometry based on positioner
        // and then sending a configure event with the new geometry.
        surface.send_configure(); // Basic ack
        tracing::warn!("Popup repositioning logic for {:?} is placeholder. Sent basic configure.", surface.wl_surface().id());
    }

    fn toplevel_request_move(&mut self, surface: &ToplevelSurface, seat_handle: &wl_seat::WlSeat, serial: Serial) {
        if let Some(window_arc) = find_managed_window_by_wl_surface(self, surface.wl_surface()) {
            tracing::info!("Window {:?} requested interactive move via client request (serial: {:?})", window_arc.id, serial);
            // TODO: Validate seat, serial, and current input state.
            // Then, potentially start an interactive move operation using a pointer grab.
            // let smithay_seat = Seat::<Self>::from_resource(seat_handle).ok_or_else(|| XdgShellError::OperationNotPermitted("Seat not found".to_string()))?;
            // interactive_ops::start_interactive_move(self, &smithay_seat, window_arc.clone(), serial);
            tracing::warn!("Interactive move op (from client request) not fully implemented for window {:?}.", window_arc.id);
        }
    }
    fn toplevel_request_resize(&mut self, surface: &ToplevelSurface, seat_handle: &wl_seat::WlSeat, serial: Serial, edges: XdgResizeEdge) {
        if let Some(window_arc) = find_managed_window_by_wl_surface(self, surface.wl_surface()) {
            tracing::info!("Window {:?} requested interactive resize via client request (edges: {:?}, serial: {:?})", window_arc.id, edges, serial);
            // TODO: Validate seat, serial, edges, and current input state.
            // Then, potentially start an interactive resize operation using a pointer grab.
            // let smithay_seat = Seat::<Self>::from_resource(seat_handle).ok_or_else(|| XdgShellError::OperationNotPermitted("Seat not found".to_string()))?;
            // interactive_ops::start_interactive_resize(self, &smithay_seat, window_arc.clone(), serial, edges);
            tracing::warn!("Interactive resize op (from client request) not fully implemented for window {:?}.", window_arc.id);
        }
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        tracing::debug!(surface_id = ?toplevel.wl_surface().id(), ?requested_mode, "Client requested decoration mode");
        
        let chosen_mode = requested_mode; // For now, honor client's request directly.
                                        // Later, a policy could override this.
                                        // e.g., if requested_mode == ClientSide, allow it. Else, force ServerSide.
        
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(chosen_mode);
        });
        // The configure event for this state change will be sent by the normal commit cycle
        // or explicitly if XdgToplevelDecoration.set_mode is called by Smithay internally.
        // Smithay's XdgToplevelDecoration object will call set_mode if the requested_mode is acceptable.
        // For this handler, we primarily update our internal state.
        // Smithay's default XdgDecorationHandler might already call set_mode on the decoration object if we don't.
        // Let's ensure our ManagedWindow reflects the choice.

        if let Some(window_arc) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(toplevel.wl_surface())).cloned() {
            let mut manager_data = window_arc.manager_data.write().unwrap();
            manager_data.decorations = chosen_mode == XdgDecorationMode::ServerSide;
            tracing::info!("Window {:?} decoration mode set to {:?} (decorations active: {}).", window_arc.id, chosen_mode, manager_data.decorations);
        }

        // It's crucial that the client receives a configure event reflecting this mode.
        // Smithay's XdgToplevelDecoration should handle sending this if its mode is set.
        // We mark the toplevel as needing a configure.
        toplevel.send_configure(); // Ensure a configure is sent.
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        tracing::debug!(surface_id = ?toplevel.wl_surface().id(), "Client requested unset decoration mode (revert to default)");
        let chosen_mode = XdgDecorationMode::ServerSide; // Default to server-side

        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(chosen_mode);
        });

        if let Some(window_arc) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(toplevel.wl_surface())).cloned() {
            let mut manager_data = window_arc.manager_data.write().unwrap();
            manager_data.decorations = true; // ServerSide
             tracing::info!("Window {:?} decoration mode unset, reverted to Server-Side Decorations.", window_arc.id);
        }
        toplevel.send_configure(); // Ensure a configure is sent.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::core::state::DesktopState; // Changed from NovadeCompositorState
    use crate::compositor::shell::xdg_shell::types::{ManagedWindow, WindowState, DomainWindowIdentifier, WindowManagerData, WindowLayer}; // Changed path
    use smithay::reexports::wayland_server::{Display, DisplayHandle, Client, protocol::wl_surface::WlSurface, globals::GlobalData, UserData, backend::{ClientId, GlobalId}};
    use smithay::reexports::calloop::EventLoop;
    use smithay::utils::{Point, Size, Rectangle};
    use smithay::wayland::shell::xdg::{ToplevelSurface, XdgShellHandler, XdgToplevelState, WindowSurface}; // Added WindowSurface
    use std::sync::Arc;

    // Minimal test client data
    #[derive(Default, Clone)]
    struct TestClientData { user_data: UserData }
    impl smithay::reexports::wayland_server::backend::ClientData for TestClientData {
        fn initialized(&self, _client_id: ClientId) {}
        fn disconnected(&self, _client_id: ClientId, _reason: smithay::reexports::wayland_server::DisconnectReason) {}
        fn data_map(&self) -> &UserData { &self.user_data }
    }

    // Helper to create a DisplayHandle and a Client for tests.
    fn create_test_display_and_client() -> (DisplayHandle, Client) {
        let mut display: Display<DesktopState> = Display::new().unwrap(); // Changed from NovadeCompositorState
        let dh = display.handle();
        // Cast needed for Client::create_object as UserData trait bound is on DesktopState via ClientData on TestClientData.
        // This is a bit of a workaround for testing.
        let client = display.create_client(TestClientData::default());
        (dh, client)
    }
    
    // Helper to create a mock ToplevelSurface
    fn mock_toplevel_surface_for_test(dh: &DisplayHandle, client: &Client) -> ToplevelSurface {
        let surface = client.create_object::<WlSurface, _>(dh, 1, GlobalData).unwrap();
        // Manually attach some data XdgShellState expects, like XdgVersion
        surface.data_map().insert_if_missing_threadsafe(|| smithay::wayland::shell::xdg::XdgVersion::V6); // V6 or Stable
        ToplevelSurface::from_wl_surface(surface, Default::default()).unwrap()
    }

    // Helper to create a minimal DesktopState for testing
    fn create_minimal_test_state() -> (DesktopState, DisplayHandle) { // Changed from NovadeCompositorState
        let mut event_loop: EventLoop<'static, DesktopState> = EventLoop::try_new().unwrap(); // Changed
        let display_handle = event_loop.handle().insert_source(
            Display::<DesktopState>::new().unwrap(), // Changed
            |_, _, _| {},
        ).unwrap();
        
        // Initialize DesktopState with Nones or defaults for fields not directly used in these tests.
        // The ::new method in state.rs is quite complex. We simplify for unit test focus.
        // DesktopState::new now takes loop_handle and display_handle
        let state = DesktopState::new(event_loop.handle(), display_handle.clone());
        // Most fields are already defaulted to None or empty by the simplified DesktopState::new
        // We might need to manually set some if the tests rely on them, e.g. seat.
        // For now, let's assume the default new() is sufficient for what these tests cover.
        // Adding seat back as it's used in some handlers, though not directly by maximize tests.
        // let mut seat_state_for_test = smithay::input::SeatState::new();
        // let seat_for_test = seat_state_for_test.new_wl_seat(&display_handle, "seat0_test".to_string(), None);
        // state.seat_state = seat_state_for_test;
        // state.seat = seat_for_test;
        // state.seat_name = "seat0_test".to_string();

        (state, display_handle)
    }

    #[test]
    fn test_set_maximized_request_updates_state() {
        let (mut state, dh) = create_minimal_test_state();
        let client = state.display_handle.create_client(TestClientData::default());
        let toplevel_surface = mock_toplevel_surface_for_test(&dh, &client);
        
        let domain_id = DomainWindowIdentifier::new_v4();
        let initial_pos = Point::from((10, 20));
        let initial_size = Size::from((300, 400));

        // Create ManagedWindow and set its initial state
        let managed_window_arc = Arc::new(ManagedWindow::new_toplevel(toplevel_surface.clone(), domain_id));
        {
            let mut win_state = managed_window_arc.state.write().unwrap();
            win_state.position = initial_pos;
            win_state.size = initial_size;
            win_state.maximized = false;
        }
        state.windows.insert(domain_id, managed_window_arc.clone());

        // Call the handler
        state.set_maximized_request(&toplevel_surface);

        // Assertions
        let win_state_guard = managed_window_arc.state.read().unwrap();
        assert_eq!(win_state_guard.maximized, true);
        assert_eq!(win_state_guard.saved_pre_action_geometry, Some(Rectangle::from_loc_and_size(initial_pos, initial_size)));
        
        // Check pending state on ToplevelSurface (simplified check)
        // This requires ToplevelSurface to allow reading its pending state, which might not be straightforward.
        // For this test, we assume the call to `with_pending_state` inside the handler correctly sets it.
        // A more robust test would involve a client acking the configure and checking the current state.
        // Here, we can at least verify that `send_configure` was implicitly called if the API allows checking that,
        // or trust that setting pending state + send_configure works as expected in Smithay.
        // Smithay's ToplevelSurface testing often involves a full client/server interaction.
    }

    #[test]
    fn test_unset_maximized_request_updates_state() {
        let (mut state, dh) = create_minimal_test_state();
        let client = state.display_handle.create_client(TestClientData::default());
        let toplevel_surface = mock_toplevel_surface_for_test(&dh, &client);

        let domain_id = DomainWindowIdentifier::new_v4();
        let saved_pos = Point::from((50, 60));
        let saved_size = Size::from((640, 480));
        let saved_geometry = Rectangle::from_loc_and_size(saved_pos, saved_size);

        let managed_window_arc = Arc::new(ManagedWindow::new_toplevel(toplevel_surface.clone(), domain_id));
        {
            let mut win_state = managed_window_arc.state.write().unwrap();
            win_state.maximized = true;
            win_state.saved_pre_action_geometry = Some(saved_geometry);
            // Set current size to something different to ensure restoration happens
            win_state.size = Size::from((1920, 1080)); 
            win_state.position = Point::from((0,0));
        }
        state.windows.insert(domain_id, managed_window_arc.clone());

        state.unset_maximized_request(&toplevel_surface);

        let win_state_guard = managed_window_arc.state.read().unwrap();
        assert_eq!(win_state_guard.maximized, false);
        assert!(win_state_guard.saved_pre_action_geometry.is_none());
        assert_eq!(win_state_guard.size, saved_size); // Check if size was restored
        assert_eq!(win_state_guard.position, saved_pos); // Check if position was restored

        // Similar to set_maximized, checking ToplevelSurface pending state is complex here.
        // We trust that `with_pending_state` was called correctly.
    }
}
