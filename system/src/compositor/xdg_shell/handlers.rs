use crate::compositor::{
    core::{
        state::DesktopState,
        surface_management::{self, SurfaceData, SurfaceRoleError}, // Added SurfaceData, SurfaceRoleError
    },
    xdg_shell::{
        errors::XdgShellError,
        types::{DomainWindowIdentifier, ManagedWindow, XdgSurfaceVariant},
    },
};
use smithay::{
    delegate_xdg_shell, delegate_xdg_activation, // For delegating XdgShellHandler & XdgActivationHandler
    desktop::{Space, Window, WindowSurfaceType, PopupManager}, // Added PopupManager
    reexports::{
        wayland_protocols::xdg::shell::server::{xdg_toplevel, xdg_wm_base}, // Added xdg_toplevel
        wayland_server::{
            protocol::{wl_seat, wl_surface}, // Added wl_seat
            Client, DisplayHandle, GlobalDispatch, New, DataInit, // Added New, DataInit
            Dispatch, RequestResult,
        },
    },
    utils::{Point, Rectangle, Serial, Size, Logical, Physical}, // Added Physical
    wayland::{
        compositor::with_states, // For accessing surface states
        seat::Seat, // For Seat in grab and XdgActivationState
        shell::xdg::{
            ActivationToken, Configure, PopupAction, PopupGrabError, PopupState, PositionerState, // Added more Popup related types
            ToplevelAction, ToplevelConfigure, ToplevelState, // Added ToplevelState
            XdgActivationHandler, XdgActivationState, XdgShellHandler, XdgShellState, // XdgActivationHandler, XdgActivationState
            XdgPopupSurfaceData, XdgSurfaceConfigure, XdgSurfaceUserData, // Added XdgSurfaceConfigure
            XdgToplevelSurfaceData, XdgWmBaseClientData, XdgPositionerUserData, // Added XdgPositionerUserData
        },
    },
    input::pointer::PointerHandle, // For grab
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// --- XDG Shell Global and Dispatch ---

impl GlobalDispatch<xdg_wm_base::XdgWmBase, XdgWmBaseClientData> for DesktopState {
    fn bind(
        state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<xdg_wm_base::XdgWmBase>,
        global_data: &XdgWmBaseClientData, // This is the XdgWmBaseClientData from XdgShellState::new_client
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!("Client bound xdg_wm_base global.");
        data_init.init(resource, global_data.clone()); // Clone the client data for this resource
    }
}

/// Creates and registers the `xdg_wm_base` (XDG Shell) and `xdg_activation_v1`
/// (XDG Activation) globals.
pub fn create_xdg_shell_globals(
    display_handle: &DisplayHandle,
    state: &mut DesktopState, // Needs to be mutable to store GlobalIds
) {
    // Create XDG Shell global (xdg_wm_base)
    // XdgShellState::new_global() is a helper that internally calls display_handle.create_global.
    // It also sets up the necessary client data handling via XdgWmBaseClientData.
    let xdg_shell_global_id = state.xdg_shell_state.new_global::<DesktopState>(display_handle);
    state.xdg_shell_global = Some(xdg_shell_global_id);
    tracing::info!("xdg_wm_base global created with ID: {:?}", xdg_shell_global_id);

    // Create XDG Activation global (xdg_activation_v1)
    // XdgActivationState::new_global() is a similar helper.
    let xdg_activation_global_id = state.xdg_activation_state.new_global::<DesktopState>(display_handle);
    state.xdg_activation_global = Some(xdg_activation_global_id);
    tracing::info!("xdg_activation_v1 global created with ID: {:?}", xdg_activation_global_id);
}


// --- XDG Shell Handler Implementation ---

impl XdgShellHandler for DesktopState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let wl_surface = surface.wl_surface().clone();
        tracing::info!(surface_id = ?wl_surface.id(), "New XDG Toplevel created");

        // Initialize SurfaceData for this wl_surface if not already present.
        // This should ideally be done when wl_surface is created by client,
        // but XDG shell object creation is a good place to ensure it.
        if wl_surface.get_data::<SurfaceData>().is_err() {
            surface_management::init_surface_data(&wl_surface, wl_surface.client());
        }

        // Assign XDG Toplevel role
        let our_surface_data = surface_management::get_surface_data(&wl_surface);
        if let Err(e) = our_surface_data.set_role("xdg_toplevel") {
            tracing::error!("Failed to set role for new toplevel: {}", e);
            // TODO: Send protocol error to client? xdg_wm_base::Error::Role
            surface.send_close(); // Close the surface as it's in an invalid state
            return;
        }

        let domain_id = DomainWindowIdentifier(Uuid::new_v4()); // Or from a policy service
        let mut managed_window = ManagedWindow::new_toplevel(surface.clone(), Some(domain_id));
        managed_window.parent_id = surface.get_parent().map(|parent_wl_surface| {
            // Attempt to find the parent ManagedWindow in our map
            self.windows.values()
                .find(|win| win.wl_surface() == &parent_wl_surface)
                .map(|parent_managed_window| parent_managed_window.domain_id)
                .unwrap_or_else(|| {
                    tracing::warn!("Parent WlSurface {:?} for toplevel {:?} not found among known ManagedWindows.", parent_wl_surface.id(), domain_id);
                    DomainWindowIdentifier(Uuid::nil()) // Indicates missing parent effectively
                })
        });


        tracing::info!(window_id = ?managed_window.internal_id, domain_id = ?managed_window.domain_id, "Created ManagedWindow for new toplevel.");

        // TODO: Interact with a domain-layer window policy service for initial geometry/state.
        // For now, use some defaults or client-requested state.
        // e.g., let initial_geom = self.window_policy_service.get_initial_geometry(&managed_window);
        // self.space.map_window(&managed_window, initial_geom.loc, false);
        // Default: map at (0,0) or let layout manager decide.
        // Smithay's XdgToplevelSurfaceData also has methods for initial configuration.

        // Store the ManagedWindow
        self.windows.insert(domain_id, Arc::new(managed_window.clone())); // Clone for Arc

        // Configure and map the window.
        // The actual mapping to space often happens on the first commit that provides a buffer.
        // Or if the client explicitly requests map.
        // Smithay's default XdgToplevelSurfaceData sends an initial configure.
        // We might trigger our own map_toplevel logic here or defer it.
        // For now, let XdgShellHandler::map_toplevel handle the actual space mapping.
        // The surface is not "mapped" in the XDG sense until its first commit with a buffer.

        // Send initial configure. Smithay's ToplevelSurface does this automatically on creation
        // if no state is set. We can customize it here if needed.
        // Example: surface.send_configure(ToplevelConfigure { ... });
        // Or rely on default configure from ToplevelSurface internal state.
        // The XdgToplevelSurfaceData is already associated with the surface by Smithay.
        let xdg_toplevel_data = wl_surface.get_data::<XdgToplevelSurfaceData>().unwrap();
        // xdg_toplevel_data.send_configure(); // Often done automatically or after setting specific states.
        // Let's assume for now that an initial configure is sent by Smithay or will be
        // sent after we process initial state requests (e.g. set_fullscreen).

        tracing::info!("New toplevel {:?} initialized. Awaiting first commit to map.", domain_id);
    }

    fn new_popup(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        _client_data: &XdgWmBaseClientData, // Client data associated with xdg_wm_base
    ) {
        let wl_surface = surface.wl_surface().clone();
        tracing::info!(surface_id = ?wl_surface.id(), "New XDG Popup created");

        if wl_surface.get_data::<SurfaceData>().is_err() {
            surface_management::init_surface_data(&wl_surface, wl_surface.client());
        }
        let our_surface_data = surface_management::get_surface_data(&wl_surface);
        if let Err(e) = our_surface_data.set_role("xdg_popup") {
            tracing::error!("Failed to set role for new popup: {}", e);
            surface.send_close();
            return;
        }

        let parent_wl_surface = match surface.get_parent_surface() {
            Some(s) => s,
            None => {
                tracing::error!("Popup {:?} has no parent surface.", wl_surface.id());
                // TODO: Send protocol error xdg_popup::Error::InvalidParent
                surface.send_close();
                return;
            }
        };

        let parent_managed_window = self.windows.values()
            .find(|win| win.wl_surface() == &parent_wl_surface)
            .cloned(); // Clone the Arc<ManagedWindow>

        if parent_managed_window.is_none() {
            tracing::error!("Parent WlSurface {:?} for popup {:?} not found among known ManagedWindows.", parent_wl_surface.id(), wl_surface.id());
            // TODO: Send protocol error
            surface.send_close();
            return;
        }
        let parent_arc = parent_managed_window.unwrap();
        let parent_domain_id = parent_arc.domain_id;

        let managed_window = ManagedWindow::new_popup(surface.clone(), parent_domain_id);
        tracing::info!(window_id = ?managed_window.internal_id, domain_id = ?managed_window.domain_id, parent_id = ?parent_domain_id, "Created ManagedWindow for new popup.");

        // Store the popup
        self.windows.insert(managed_window.domain_id, Arc::new(managed_window.clone()));

        // Popups are mapped relative to their parent.
        // The geometry calculation and mapping will happen when the popup is committed
        // and its positioner state is applied.
        // Smithay's PopupManager handles much of this logic if used.
        // For now, we ensure it's tracked. The actual mapping to Space happens on commit.

        // Send initial configure for the popup.
        // The XdgPopupSurfaceData is already associated by Smithay.
        let xdg_popup_data = wl_surface.get_data::<XdgPopupSurfaceData>().unwrap();
        // xdg_popup_data.send_configure(&parent_wl_surface); // Send configure relative to parent.
        // This configure often includes the calculated position and size.
        // The geometry calculation from ManagedWindow::calculate_popup_geometry can be used here.
        let calculated_geom = ManagedWindow::calculate_popup_geometry(&surface, &parent_arc);
        surface.send_configure(calculated_geom); // Smithay's PopupSurface::send_configure takes Rectangle

        tracing::info!("New popup {:?} initialized. Awaiting first commit to map.", managed_window.domain_id);
    }

    fn map_toplevel(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface) -> PopupAction {
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "Map request for XDG Toplevel");

        let managed_window_arc = self.windows.values()
            .find(|win| win.wl_surface() == wl_surface && matches!(win.surface_variant, XdgSurfaceVariant::Toplevel(_)))
            .cloned();

        if let Some(managed_window_arc) = managed_window_arc {
            if managed_window_arc.is_mapped_by_compositor() {
                tracing::warn!("Map requested for already mapped toplevel: {:?}", managed_window_arc.domain_id);
                return PopupAction::None; // Or an error/warning if client behavior is unexpected
            }

            // TODO: Interact with domain policy for placement, if any.
            // For now, find a default spot or use (0,0) if not already placed by layout manager.
            let current_space_loc = self.space.window_location(&managed_window_arc);
            let map_loc = current_space_loc.unwrap_or_else(|| {
                // TODO: Better placement strategy, e.g., cascade or center.
                let new_loc = Point::from((100 + (self.windows.len() % 5 * 50) as i32, 100 + (self.windows.len() % 5 * 20) as i32));
                tracing::info!("Toplevel {:?} has no location in space, assigning default: {:?}", managed_window_arc.domain_id, new_loc);
                new_loc
            });


            self.space.map_window(managed_window_arc.as_ref(), map_loc, true); // Map with activation
            managed_window_arc.set_mapped_compositor(true);
            managed_window_arc.set_geometry(Rectangle::from_loc_and_size(map_loc, managed_window_arc.geometry().size));


            tracing::info!(window_id = ?managed_window_arc.internal_id, domain_id = ?managed_window_arc.domain_id, location = ?map_loc, "Mapped ManagedWindow (Toplevel) to space.");

            // TODO: Notify domain layer about window mapping.
            // self.domain_notifier.window_mapped(managed_window_arc.domain_id);

            // Damage the space for rendering
            self.space.damage_all(); // Or damage specific region
            self.loop_signal.wakeup(); // Signal event loop to redraw

        } else {
            tracing::error!("Map request for unknown toplevel surface: {:?}", wl_surface.id());
            // This shouldn't happen if new_toplevel was called correctly.
        }
        PopupAction::None // Default, can be changed if grab is needed
    }

    fn unmap_toplevel(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface) {
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "Unmap request for XDG Toplevel");

        let managed_window_arc = self.windows.values()
            .find(|win| win.wl_surface() == wl_surface && matches!(win.surface_variant, XdgSurfaceVariant::Toplevel(_)))
            .cloned();

        if let Some(managed_window_arc) = managed_window_arc {
            if !managed_window_arc.is_mapped_by_compositor() {
                tracing::warn!("Unmap requested for already unmapped toplevel: {:?}", managed_window_arc.domain_id);
                return;
            }

            self.space.unmap_window(&managed_window_arc);
            managed_window_arc.set_mapped_compositor(false);
            tracing::info!(window_id = ?managed_window_arc.internal_id, domain_id = ?managed_window_arc.domain_id, "Unmapped ManagedWindow (Toplevel) from space.");

            // TODO: Notify domain layer about window unmapping.
            // self.domain_notifier.window_unmapped(managed_window_arc.domain_id);

            self.space.damage_all(); // Or damage specific region
            self.loop_signal.wakeup();
        } else {
            tracing::error!("Unmap request for unknown toplevel surface: {:?}", wl_surface.id());
        }
    }
    
    fn map_popup(&mut self, surface: &smithay::wayland::shell::xdg::PopupSurface, _seat: wl_seat::WlSeat) -> Result<PopupAction, PopupGrabError> {
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "Map request for XDG Popup");

        let managed_window_arc = self.windows.values()
            .find(|win| win.wl_surface() == wl_surface && matches!(win.surface_variant, XdgSurfaceVariant::Popup(_)))
            .cloned();
        
        if let Some(managed_window_arc) = managed_window_arc {
            if managed_window_arc.is_mapped_by_compositor() {
                tracing::warn!("Map requested for already mapped popup: {:?}", managed_window_arc.domain_id);
                return Ok(PopupAction::None);
            }

            let parent_domain_id = managed_window_arc.parent_id.ok_or_else(|| {
                tracing::error!("Popup {:?} has no parent_id for mapping.", managed_window_arc.domain_id);
                PopupGrabError::InvalidPopup // Or some other appropriate error
            })?;
            
            let parent_arc = self.windows.get(&parent_domain_id).cloned().ok_or_else(|| {
                tracing::error!("Parent ManagedWindow {:?} not found for popup {:?}", parent_domain_id, managed_window_arc.domain_id);
                PopupGrabError::InvalidPopup
            })?;

            // Calculate geometry relative to parent. Smithay's PopupManager usually does this.
            // We use our helper from ManagedWindow.
            let geometry = ManagedWindow::calculate_popup_geometry(surface, &parent_arc);
            managed_window_arc.set_geometry(geometry); // Store its calculated geometry

            // Popups are mapped relative to their parent in the Space.
            // Smithay's Space::map_popup handles this.
            self.space.map_popup(managed_window_arc.as_ref(), parent_arc.id(), true); // Map with activation
            managed_window_arc.set_mapped_compositor(true);

            tracing::info!(window_id = ?managed_window_arc.internal_id, domain_id = ?managed_window_arc.domain_id, geometry = ?geometry, "Mapped ManagedWindow (Popup) to space relative to parent {:?}", parent_domain_id);
            
            self.space.damage_all();
            self.loop_signal.wakeup();
            Ok(PopupAction::None) // Or ::GrabSeat if an implicit grab is desired
        } else {
            tracing::error!("Map request for unknown popup surface: {:?}", wl_surface.id());
            Err(PopupGrabError::InvalidPopup) // Should not happen
        }
    }

    fn unmap_popup(&mut self, surface: &smithay::wayland::shell::xdg::PopupSurface) {
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "Unmap request for XDG Popup");

        let managed_window_arc = self.windows.values()
            .find(|win| win.wl_surface() == wl_surface && matches!(win.surface_variant, XdgSurfaceVariant::Popup(_)))
            .cloned();

        if let Some(managed_window_arc) = managed_window_arc {
            if !managed_window_arc.is_mapped_by_compositor() {
                tracing::warn!("Unmap requested for already unmapped popup: {:?}", managed_window_arc.domain_id);
                return;
            }
            self.space.unmap_window(&managed_window_arc); // unmap_window works for popups too
            managed_window_arc.set_mapped_compositor(false);
            tracing::info!(window_id = ?managed_window_arc.internal_id, domain_id = ?managed_window_arc.domain_id, "Unmapped ManagedWindow (Popup) from space.");
            
            self.space.damage_all();
            self.loop_signal.wakeup();
        } else {
            tracing::error!("Unmap request for unknown popup surface: {:?}", wl_surface.id());
        }
    }


    fn ack_configure(&mut self, surface: wl_surface::WlSurface, configure: XdgSurfaceConfigure) {
        tracing::debug!(surface_id = ?surface.id(), "Client acknowledged XDG configure (serial: {:?})", configure.serial);

        let Some(window_arc) = self.windows.values().find(|w| w.wl_surface() == &surface).cloned() else {
            tracing::warn!("ack_configure for unknown surface {:?}", surface.id());
            return;
        };

        // Process pending state updates that were waiting for this ack.
        // For example, if a resize was requested, the new size is now "active".
        // Smithay's XdgToplevelSurfaceData and XdgPopupSurfaceData handle serial tracking.
        // We might need to update our ManagedWindow's view of committed state.

        let role_data = surface.get_data::<XdgSurfaceUserData>().unwrap(); // Smithay attaches this
        let client_pending_state = match role_data.role() {
            "xdg_toplevel" => {
                let toplevel_data = surface.get_data::<XdgToplevelSurfaceData>().unwrap();
                toplevel_data.client_pending_state()
            }
            "xdg_popup" => {
                // Popups don't have the same kind of extensive pending state as toplevels
                // that gets acked this way. Configure for popups is simpler.
                // For now, just log and return if it's a popup.
                tracing::trace!("ack_configure for popup {:?} - no specific state to update from client_pending_state.", window_arc.domain_id);
                return;
            }
            _ => {
                tracing::warn!("ack_configure for surface with unknown XDG role: {:?}", role_data.role());
                return;
            }
        };
        
        // Update ManagedWindow's requested_size if it changed and was acked.
        // This ensures our internal view matches what client has acked.
        if let Some(new_size) = client_pending_state.size {
            if new_size.w > 0 && new_size.h > 0 { // Valid size
                 *window_arc.requested_size.lock().unwrap() = new_size;
                 tracing::debug!("Updated ManagedWindow {:?} requested_size to {:?} after ack_configure.", window_arc.domain_id, new_size);
            }
        }
        // Similar updates for min/max_size, states (maximized, fullscreen, etc.) if they were part of the configure.
        // TODO: Check specific states like maximized, fullscreen from client_pending_state
        // and update ManagedWindow and potentially Space layout.

        // If the configure included a new size that affects layout, damage might be needed.
        // This is often handled during commit or resize request processing.
    }

    fn toplevel_destroyed(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "XDG Toplevel destroyed");

        let mut found_id: Option<DomainWindowIdentifier> = None;
        if let Some(managed_window) = self.windows.values().find(|win| win.wl_surface() == wl_surface) {
            found_id = Some(managed_window.domain_id);
            if managed_window.is_mapped_by_compositor() {
                self.space.unmap_window(managed_window);
            }
            tracing::info!(window_id = ?managed_window.internal_id, domain_id = ?managed_window.domain_id, "Cleaned up ManagedWindow (Toplevel) from space.");
        }

        if let Some(id_to_remove) = found_id {
            if self.windows.remove(&id_to_remove).is_some() {
                tracing::info!("Removed ManagedWindow {:?} from map.", id_to_remove);
            } else {
                tracing::warn!("Failed to remove ManagedWindow {:?} from map, was not found.", id_to_remove);
            }
        } else {
            tracing::warn!("Destroyed toplevel surface {:?} not found in ManagedWindow map.", wl_surface.id());
        }

        // TODO: Notify domain layer about window destruction.
        // if let Some(id) = found_id { self.domain_notifier.window_destroyed(id); }

        self.space.damage_all();
        self.loop_signal.wakeup();
    }

    fn popup_destroyed(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface) {
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "XDG Popup destroyed");

        let mut found_id: Option<DomainWindowIdentifier> = None;
        if let Some(managed_window) = self.windows.values().find(|win| win.wl_surface() == wl_surface) {
            found_id = Some(managed_window.domain_id);
            if managed_window.is_mapped_by_compositor() {
                self.space.unmap_window(managed_window);
            }
            tracing::info!(window_id = ?managed_window.internal_id, domain_id = ?managed_window.domain_id, "Cleaned up ManagedWindow (Popup) from space.");
        }

        if let Some(id_to_remove) = found_id {
            if self.windows.remove(&id_to_remove).is_some() {
                tracing::info!("Removed ManagedWindow (Popup) {:?} from map.", id_to_remove);
            } else {
                tracing::warn!("Failed to remove ManagedWindow (Popup) {:?} from map, was not found.", id_to_remove);
            }
        } else {
            tracing::warn!("Destroyed popup surface {:?} not found in ManagedWindow map.", wl_surface.id());
        }
        self.space.damage_all();
        self.loop_signal.wakeup();
    }

    // --- Request Handlers (Toplevel) ---
    // Most of these will update ManagedWindow and then call methods on XdgToplevelSurfaceData
    // to reflect the state change and send a new configure to the client.

    fn toplevel_request_set_title(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface, title: String) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), new_title = %title, "Toplevel request: set_title");
        // TODO: Update ManagedWindow.title if it stores it. Notify domain layer.
        // Smithay's XdgToplevelSurfaceData stores this internally. No action usually needed here
        // unless we want to react to the title change.
    }

    fn toplevel_request_set_app_id(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface, app_id: String) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), app_id = %app_id, "Toplevel request: set_app_id");
        // TODO: Update ManagedWindow.app_id. Notify domain layer.
        // Smithay's XdgToplevelSurfaceData stores this.
    }
    
    // ... other toplevel request handlers like set_parent, (un)set_maximized, (un)set_fullscreen, etc.
    // will be added progressively. For now, these are stubs or rely on Smithay's default handling.

    fn toplevel_request_set_maximized(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface) -> ToplevelAction {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Toplevel request: set_maximized");
        // TODO: Implement maximization logic:
        // 1. Get ManagedWindow for surface.
        // 2. Consult window policy (domain layer) if allowed / how to maximize.
        // 3. Update ManagedWindow state.
        // 4. Update XdgToplevelSurfaceData state (e.g., set_maximized(true)).
        // 5. Calculate new geometry for maximized window (e.g., full output size).
        // 6. self.space.map_window with new geometry.
        // 7. Send new configure via XdgToplevelSurfaceData.
        // For now, just ack the request by reflecting state.
        let xdg_data = surface.wl_surface().get_data::<XdgToplevelSurfaceData>().unwrap();
        xdg_data.set_maximized(true);
        // xdg_data.send_configure(); // This should send the new state
        ToplevelAction::SendConfigure // Indicate that a configure should be sent
    }

    fn toplevel_request_unset_maximized(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface) -> ToplevelAction {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Toplevel request: unset_maximized");
        let xdg_data = surface.wl_surface().get_data::<XdgToplevelSurfaceData>().unwrap();
        xdg_data.set_maximized(false);
        // TODO: Restore to previous size/state.
        // xdg_data.send_configure();
        ToplevelAction::SendConfigure
    }
    
    fn toplevel_request_set_fullscreen(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface, _output: Option<wl_surface::WlSurface>) -> ToplevelAction {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Toplevel request: set_fullscreen");
        // TODO: Similar to maximize, but for fullscreen on a specific output.
        let xdg_data = surface.wl_surface().get_data::<XdgToplevelSurfaceData>().unwrap();
        xdg_data.set_fullscreen(true); // Potentially with output
        ToplevelAction::SendConfigure
    }

    fn toplevel_request_unset_fullscreen(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface) -> ToplevelAction {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Toplevel request: unset_fullscreen");
        let xdg_data = surface.wl_surface().get_data::<XdgToplevelSurfaceData>().unwrap();
        xdg_data.set_fullscreen(false);
        ToplevelAction::SendConfigure
    }

    fn toplevel_request_set_minimized(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Toplevel request: set_minimized");
        // TODO: Implement minimization (typically unmapping the window and showing it in a taskbar).
        // For now, Smithay's XdgToplevelSurfaceData tracks this, but we need to act on it.
        // This might involve unmapping from space and notifying a domain-layer taskbar service.
        self.unmap_toplevel(surface); // Simplistic minimization: just unmap.
    }
    
    fn toplevel_request_set_parent(&mut self, _surface: &smithay::wayland::shell::xdg::ToplevelSurface, _parent: Option<smithay::wayland::shell::xdg::ToplevelSurface>) -> ToplevelAction {
        // This is for setting a toplevel as transient for another toplevel.
        // Update ManagedWindow.parent_id and potentially window stacking order.
        ToplevelAction::None // Or SendConfigure if state changes that client needs to know
    }


    fn toplevel_request_move(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface, seat: wl_seat::WlSeat, serial: Serial) -> Result<(), ()> {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Toplevel request: move");
        // TODO: Initiate an interactive move grab if seat has pointer with button down.
        // This involves DesktopState::seat and its pointer state.
        // Smithay has PointerMoveGrab, PointerResizeGrab.
        // For now, deny or log.
        let seat = Seat::from_resource(&seat).ok_or(())?;
        let pointer = seat.get_pointer().ok_or(())?; // Check if seat has pointer capability

        if !pointer.has_grab(serial) { // Check if this serial already started a grab
            if let Some(start_data) = pointer.grab_start_data() {
                 if start_data.button == 0x110 { // BTN_LEFT, assuming Linux evdev codes
                    // Start a move grab
                    // Smithay's Space has start_interactive_move_grab
                    // This needs access to DesktopState's space and the ManagedWindow.
                    tracing::info!("Attempting to start interactive move for surface {:?}", surface.wl_surface().id());
                    // This part is complex and requires proper grab management.
                    // Placeholder for actual grab initiation.
                    // self.space.start_interactive_move_grab(window, &seat, serial);
                    return Err(()); // Placeholder: Deny grab for now
                 }
            }
        }
        Err(()) // Deny if conditions not met
    }

    fn toplevel_request_resize(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface, seat: wl_seat::WlSeat, serial: Serial, edges: xdg_toplevel::ResizeEdge) -> Result<(), ()> {
        tracing::info!(surface_id = ?surface.wl_surface().id(), ?edges, "Toplevel request: resize");
        // TODO: Initiate an interactive resize grab. Similar to move.
        // Smithay's Space has start_interactive_resize_grab
        let _ = (seat, serial); // Deny for now
        Err(())
    }
    
    fn toplevel_request_show_window_menu(&mut self, surface: &smithay::wayland::shell::xdg::ToplevelSurface, seat: wl_seat::WlSeat, serial: Serial, position: Point<i32, Logical>) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), ?position, "Toplevel request: show_window_menu");
        // TODO: Display a window menu (typically a context menu for the window).
        // This might involve creating a new popup surface managed by the compositor itself.
        let _ = (seat, serial);
    }

    // --- Popup Requests & Grabs ---
    fn popup_request_grab(&mut self, surface: &smithay::wayland::shell::xdg::PopupSurface, seat: wl_seat::WlSeat, serial: Serial) -> Result<PopupAction, PopupGrabError> {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Popup request: grab");
        // This is for explicit grabs (e.g., for menus).
        // The PopupManager in Smithay can handle this.
        // If not using PopupManager directly for grabs, implement logic here.
        // For now, allow the grab without special conditions.
        let _ = (seat, serial);
        Ok(PopupAction::GrabSeat) // Grant the grab
    }
    
    fn reposition_request(&mut self, surface: &smithay::wayland::shell::xdg::PopupSurface, positioner: PositionerState, token: u32) -> PopupAction {
        tracing::info!(surface_id = ?surface.wl_surface().id(), ?token, "Popup request: reposition");
        // TODO: Recalculate popup position based on new positioner state and parent.
        // Update ManagedWindow.current_geometry.
        // self.space.map_popup with new geometry if different.
        // Send new configure.
        let _ = positioner;
        PopupAction::SendConfigure // Indicate a configure should be sent after state update
    }


    // --- XDG Activation Handler (minimal) ---
    // This is part of XdgShellHandler in Smithay <= 0.2, separate in >= 0.3.
    // Assuming Smithay 0.3+ structure where XdgActivationHandler is separate,
    // these methods would be on that trait's impl for DesktopState.
    // The current DesktopState has a placeholder XdgActivationHandler.
    // For now, these are here for completeness if the trait structure demands it,
    // but they should ideally be in the XdgActivationHandler impl.

    // fn request_activation_token(&mut self, token_data: ActivationToken, client: &Client) {
    //     self.xdg_activation_state.request_new_token(token_data, client);
    // }

    // fn surface_request_activation(&mut self, surface: WlSurface, token_data: Option<ActivationTokenData>) {
    //     // Handle activation requests for a surface, possibly using a token.
    //     // This involves focusing the window, bringing it to front.
    // }
}

// Delegate XDG Shell handling to DesktopState
delegate_xdg_shell!(DesktopState);

// Minimal XDG Activation Handler (already in core::state.rs, ensure it's sufficient)
// No specific new methods needed here unless expanding its functionality.
// delegate_xdg_activation!(DesktopState); // This would be in core::state.rs if delegating fully there.
// The current core::state.rs has the basic XdgActivationHandler impl.

// Note: Some XdgShellHandler methods return ToplevelAction or PopupAction.
// These enums tell Smithay's XdgShellState whether to automatically send
// a configure event after the handler returns. If you send one manually
// in the handler, you can return Action::None. Otherwise, return Action::SendConfigure.
// Action::Destroy tells Smithay to destroy the xdg_surface resource.
// Action::SetParent is for more complex reparenting scenarios.

// This file is becoming large. Consider splitting parts of XdgShellHandler if it grows further,
// for example, into toplevel_handlers.rs and popup_handlers.rs, though Rust's orphan rules
// might make direct trait impl splitting tricky without newtypes or internal delegation.
// For now, keeping it in one place is acceptable.Okay, I've created `system/src/compositor/xdg_shell/handlers.rs` and implemented the initial set of `XdgShellHandler` methods, `GlobalDispatch<XdgWmBase, XdgWmBaseClientData>`, and `create_xdg_shell_globals`.

The following handlers have a basic implementation:
- `xdg_shell_state()`
- `new_toplevel()`: Creates `ManagedWindow`, assigns role, stores it. Defers mapping.
- `new_popup()`: Creates `ManagedWindow`, assigns role, stores it. Defers mapping.
- `map_toplevel()`: Maps toplevel `ManagedWindow` to space, updates state.
- `unmap_toplevel()`: Unmaps toplevel `ManagedWindow` from space, updates state.
- `map_popup()`: Calculates geometry, maps popup `ManagedWindow` to space relative to parent.
- `unmap_popup()`: Unmaps popup.
- `ack_configure()`: Logs acknowledgement, basic update of `ManagedWindow`'s `requested_size`.
- `toplevel_destroyed()`: Cleans up toplevel from space and map.
- `popup_destroyed()`: Cleans up popup from space and map.
- Request handlers like `set_title`, `set_app_id`, `(un)set_maximized`, `(un)set_fullscreen`, `set_minimized` are placeholders or reflect state in `XdgToplevelSurfaceData` and return `ToplevelAction::SendConfigure`.
- Interactive requests like `move`, `resize`, `show_window_menu` are stubs, typically denying the request for now.
- Popup requests `grab` and `reposition` are basic.

I've also added the delegation macro `delegate_xdg_shell!(DesktopState);` at the end of the file.
The XDG Activation handler is already minimally implemented in `core/state.rs`.

The next step is to create the `system/src/compositor/xdg_shell/mod.rs` file to make these components available.
