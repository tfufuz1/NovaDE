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
    },
    input::SeatHandler, // SeatHandler might not be directly needed in XdgShellHandler trait methods
};
use std::sync::Arc;

use crate::compositor::{
    core::state::NovadeCompositorState,
    xdg_shell::types::{DomainWindowIdentifier, ManagedWindow},
    // Removed unused XdgShellError import
};

// --- XDG Decoration Imports ---
use smithay::wayland::shell::xdg::decoration::{
    XdgDecorationHandler, XdgToplevelDecoration, Mode as XdgDecorationMode, ServerDecorationState
};
// ToplevelSurface is already imported above.

impl XdgShellHandler for NovadeCompositorState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "New XDG Toplevel created.");
        let domain_window_id = DomainWindowIdentifier::new_v4();
        let managed_window_arc = Arc::new(ManagedWindow::new_toplevel(surface.clone(), domain_window_id));

        {
            let mut mw_state = managed_window_arc.state.write().unwrap();
            mw_state.position = Point::from((100, 100)); // Example initial position
            mw_state.size = Size::from((300, 200));     // Example initial size
            // current_geometry on ManagedWindow struct is updated by space/layout logic
        }
        
        surface.with_pending_state(|xdg_state| {
            let mw_state = managed_window_arc.state.read().unwrap();
            xdg_state.size = Some(mw_state.size);
            // Consider setting activated state if this is the currently focused new window
        });
        let _configure_serial = surface.send_configure(); 
        // managed_window_arc.last_configure_serial = Some(_configure_serial); // Needs interior mutability for last_configure_serial

        self.windows.insert(domain_window_id, managed_window_arc.clone());
        tracing::info!("Created ManagedWindow with ID {:?}, initial size {:?}, configure serial {:?}", 
            managed_window_arc.id, managed_window_arc.state.read().unwrap().size, _configure_serial);
    }

    fn new_popup(&mut self, surface: PopupSurface, _client_data: &XdgWmBaseClientData) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "New XDG Popup created.");
        let parent_wl_surface = surface.parent_surface();
        let parent_arc = parent_wl_surface.and_then(|parent_surf| {
            self.windows.values().find(|win| win.wl_surface().map_or(false, |s| s == parent_surf)).cloned()
        });
        let parent_domain_id = parent_arc.as_ref().map_or_else(DomainWindowIdentifier::new_v4, |p| p.domain_id);
        let managed_popup_arc = Arc::new(ManagedWindow::new_popup(surface.clone(), parent_domain_id, parent_arc));
        
        // Store the Arc<ManagedWindow> in the PopupSurface's user data for later retrieval if needed,
        // and for lifetime management.
        surface.user_data().insert_if_missing_threadsafe(|| managed_popup_arc.clone());

        // Add a destruction hook to the underlying WlSurface of the popup.
        let popup_wl_surface = surface.wl_surface().clone();
        smithay::wayland::compositor::add_destruction_hook(&popup_wl_surface, |data_map_of_destroyed_surface| {
            // Attempt to retrieve our ManagedWindow Arc from user data.
            // Note: The key type for retrieval must match exactly what was inserted.
            // If we inserted Arc<ManagedWindow>, we retrieve Arc<ManagedWindow>.
            if let Some(retrieved_popup_arc) = data_map_of_destroyed_surface.get::<Arc<ManagedWindow>>() {
                tracing::info!(popup_managed_window_id = ?retrieved_popup_arc.id, "ManagedWindow for XDG Popup (surface {:?}) destroyed and its Arc dropped from user_data.", popup_wl_surface.id());
            } else {
                tracing::warn!("Could not retrieve ManagedWindow Arc from user_data for destroyed XDG Popup (surface {:?}). It might have been removed earlier or not set.", popup_wl_surface.id());
            }
            // The Arc itself is dropped when it goes out of scope here if this is the last reference,
            // or when the UserDataMap is cleared upon surface destruction.
        });

        let _configure_serial = surface.send_configure();
        tracing::debug!("Sent initial configure for new popup {:?}, stored ManagedWindow ({:?}) in user_data.", surface.wl_surface().id(), managed_popup_arc.id);
    }

    fn map_toplevel(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Mapping XDG Toplevel.");
        if let Some(window_arc) = self.windows.values().find(|win| win.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())).cloned() {
            // The Window::is_mapped() on ManagedWindow should reflect its status in the space.
            // is_mapped field on ManagedWindow struct is updated by the Window trait methods if needed.
            let initial_pos = window_arc.state.read().unwrap().position;
            self.space.map_window(window_arc.clone(), initial_pos, true); // activate on map
            tracing::info!("Mapped window {:?} to space at {:?}.", window_arc.id, initial_pos);
            self.space.damage_all_outputs();
        } else {
            tracing::error!("Tried to map a toplevel not found in internal tracking: {:?}", surface.wl_surface().id());
        }
    }

    fn unmap_toplevel(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Unmapping XDG Toplevel.");
        if let Some(window_arc) = self.windows.values().find(|win| win.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())).cloned() {
            self.space.unmap_window(&window_arc);
            tracing::info!("Unmapped window {:?} from space.", window_arc.id);
            self.space.damage_all_outputs();
        } else {
            tracing::error!("Tried to unmap a toplevel not found: {:?}", surface.wl_surface().id());
        }
    }
    
    fn ack_configure(&mut self, surface: WlSurface, configure_data: XdgSurfaceConfigure) {
        let serial = configure_data.serial;
        tracing::debug!(surface_id = ?surface.id(), ?serial, surface_type = ?configure_data.surface_type, "XDG Surface ack_configure received.");
        if let Err(e) = XdgShellState::handle_ack_configure(&surface, configure_data) {
            tracing::warn!("Error handling ack_configure from Smithay's XdgShellState: {}", e);
        }
    }

    fn move_request(&mut self, surface: &ToplevelSurface, _seat: &Seat<Self>, _serial: Serial) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG Toplevel move request.");
        if let Some(window) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())) {
            // TODO: Validate serial against window.last_configure_serial if it's reliably updated.
            let mut manager_data = window.manager_data.write().unwrap();
            manager_data.moving = true;
            // Actual grab logic (e.g., with PointerGrab) would start here.
            tracing::debug!("Window {:?} manager_data.moving set to true.", window.id);
            // Example: self.pointer_interaction.start_move_grab(window.clone(), seat, serial);
        }
    }

    fn resize_request(&mut self, surface: &ToplevelSurface, _seat: &Seat<Self>, _serial: Serial, edges: XdgResizeEdge) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), ?edges, "XDG Toplevel resize request.");
        if let Some(window) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())) {
            let mut manager_data = window.manager_data.write().unwrap();
            manager_data.resizing = true;
            manager_data.resize_edges = Some(edges);
            tracing::debug!("Window {:?} manager_data.resizing set to true, edges: {:?}.", window.id, edges);
            // Example: self.pointer_interaction.start_resize_grab(window.clone(), seat, serial, edges);
        }
    }

    fn set_title_request(&mut self, surface: &ToplevelSurface) {
        let title = surface.title();
        tracing::info!(surface_id = ?surface.wl_surface().id(), title = ?title, "XDG Toplevel set_title request.");
        // ManagedWindow::self_update() (from Window trait) should be called elsewhere to sync this.
    }

    fn set_app_id_request(&mut self, surface: &ToplevelSurface) {
        let app_id = surface.app_id();
        tracing::info!(surface_id = ?surface.wl_surface().id(), app_id = ?app_id, "XDG Toplevel set_app_id request.");
        // ManagedWindow::self_update() (from Window trait) should be called elsewhere to sync this.
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

    fn set_maximized_request(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG Toplevel set_maximized request.");
        if let Some(window_arc) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())).cloned() {
            let mut win_state_guard = window_arc.state.write().unwrap();
            if !win_state_guard.maximized {
                // Save current geometry before maximizing
                win_state_guard.saved_pre_action_geometry = Some(Rectangle::from_loc_and_size(
                    win_state_guard.position,
                    win_state_guard.size,
                ));
                tracing::debug!("Saved pre-maximize geometry: {:?}", win_state_guard.saved_pre_action_geometry);
            }
            win_state_guard.maximized = true;
            win_state_guard.minimized = false;

            let maximized_size = self.space.outputs()
                .find_map(|o| self.space.output_geometry(o).map(|g| g.size));

            surface.with_pending_state(|xdg_state| {
                xdg_state.states.set(XdgToplevelState::Maximized, true);
                xdg_state.size = maximized_size; 
            });
            let _serial = surface.send_configure();
            tracing::debug!("Maximized request for {:?}, sent configure with size {:?}, serial {:?}", surface.wl_surface().id(), maximized_size, _serial);
        }
    }

    fn unset_maximized_request(&mut self, surface: &ToplevelSurface) {
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
    
    fn set_minimized_request(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG Toplevel set_minimized request.");
        if let Some(window_arc) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())).cloned() {
            let mut win_state_guard = window_arc.state.write().unwrap();
            win_state_guard.minimized = true;
            // Client is expected to unmap itself. Compositor might unmap from space if client misbehaves.
            // No configure is sent for minimized state typically.
            tracing::debug!("Minimized request for {:?}. Client should unmap.", surface.wl_surface().id());
        }
    }

    fn set_fullscreen_request(&mut self, surface: &ToplevelSurface, output: Option<WaylandOutput>) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), ?output, "XDG Toplevel set_fullscreen request.");
        if let Some(window_arc) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())).cloned() {
            let mut win_state_guard = window_arc.state.write().unwrap();
            if !win_state_guard.fullscreen {
                // Save current geometry before fullscreen
                 win_state_guard.saved_pre_action_geometry = Some(Rectangle::from_loc_and_size(
                    win_state_guard.position,
                    win_state_guard.size,
                ));
                tracing::debug!("Saved pre-fullscreen geometry: {:?}", win_state_guard.saved_pre_action_geometry);
            }
            win_state_guard.fullscreen = true;
            win_state_guard.minimized = false;
            
            let target_output = output.as_ref().or_else(|| self.space.outputs().next());
            let fullscreen_size = target_output.and_then(|o| self.space.output_geometry(o)).map(|g| g.size);

            surface.with_pending_state(|xdg_state| {
                xdg_state.states.set(XdgToplevelState::Fullscreen, true);
                xdg_state.size = fullscreen_size;
            });
            let _serial = surface.send_configure();
            tracing::debug!("Fullscreen request for {:?}, output {:?}, sent configure with size {:?}, serial {:?}", surface.wl_surface().id(), target_output.map(|o|o.name()), fullscreen_size, _serial);
        }
    }

    fn unset_fullscreen_request(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG Toplevel unset_fullscreen request.");
         if let Some(window_arc) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())).cloned() {
            let mut win_state_guard = window_arc.state.write().unwrap();
            win_state_guard.fullscreen = false;

            let restored_geometry = win_state_guard.saved_pre_action_geometry.take();
            let (restored_pos, restored_size) = if let Some(geom) = restored_geometry {
                tracing::debug!("Restoring to saved geometry: {:?}", geom);
                win_state_guard.position = geom.loc;
                win_state_guard.size = geom.size;
                (Some(geom.loc), Some(geom.size))
            } else {
                tracing::warn!("No saved geometry found for unfullscreen, using current size.");
                (None, Some(win_state_guard.size))
            };

            surface.with_pending_state(|xdg_state| {
                xdg_state.states.set(XdgToplevelState::Fullscreen, false);
                xdg_state.size = restored_size;
            });
            let _serial = surface.send_configure();
            tracing::debug!("Unfullscreen request for {:?}, sent configure with size {:?}, serial {:?}", surface.wl_surface().id(), restored_size, _serial);
        }
    }

    fn toplevel_destroyed(&mut self, toplevel: ToplevelSurface) {
        let wl_surface_id = toplevel.wl_surface().id();
        tracing::info!(surface_id = ?wl_surface_id, "XDG Toplevel destroyed.");
        
        if let Some(window_arc) = self.windows.values().find(|win| win.xdg_surface.wl_surface().as_ref() == Some(toplevel.wl_surface())).cloned() {
            self.space.unmap_window(&window_arc); 
            if self.windows.remove(&window_arc.domain_id).is_some() {
                tracing::info!("Removed window with domain ID {:?} (surface {:?}) from tracking.", window_arc.domain_id, wl_surface_id);
            } else {
                tracing::warn!("Window for domain ID {:?} (surface {:?}) already removed before destruction signal.", window_arc.domain_id, wl_surface_id);
            }
        } else {
            tracing::warn!("Destroyed toplevel {:?} was not found in self.windows by its WlSurface.", wl_surface_id);
        }
        self.space.damage_all_outputs();
    }

    fn popup_destroyed(&mut self, popup: PopupSurface) {
        tracing::info!(surface_id = ?popup.wl_surface().id(), "XDG Popup destroyed signal received.");
        // The Arc<ManagedWindow> stored in the popup's user_data (if any) will be dropped automatically
        // when the PopupSurface and its WlSurface are destroyed by Smithay, as part of UserDataMap cleanup.
        // The destruction hook added in `new_popup` will log this.
        // If popups were tracked in a separate list in NovadeCompositorState, they would be removed here.
    }
}

impl XdgDecorationHandler for NovadeCompositorState {
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

    fn request_mode(&mut self, toplevel: ToplevelSurface, requested_mode: XdgDecorationMode) {
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
