use smithay::{
    reexports::wayland_server::{
        protocol::{wl_surface::WlSurface, wl_seat::WlSeat, wl_output}, // Added wl_output for one of the request handlers
        DisplayHandle, Client, Resource, Serial,
    },
    utils::{Logical, Point, Rectangle, Size},
    wayland::{
        compositor::with_states as with_surface_states,
        seat::Seat,
        shell::xdg::{
            Configure as XdgSurfaceConfigure, PositionerState, PopupSurface, ToplevelSurface, XdgShellHandler,
            XdgShellState, XdgWmBaseClientData,
            xdg_toplevel::{State as XdgToplevelState, ResizeEdge as XdgResizeEdge, WmCapabilities as XdgWmCapabilities},
            xdg_wm_base, // For error posting
            XdgPopupSurfaceData, XdgToplevelSurfaceData, // Smithay's user data for surfaces
        },
        SerialCounter, // For generating configure serials
    },
    desktop::{
        utils as desktop_utils,
        WindowSurfaceType, Window, 
    },
    input::pointer::{GrabStartData as PointerGrabStartData, Focus as PointerFocus, PointerTarget, PointerHandle},
};
use std::sync::Arc;
use crate::window_mechanics::interactive_ops::{MoveResizeState, InteractiveOpType, NovaMoveGrab, NovaResizeGrab};
use tracing::{error, info, warn, debug};
use uuid::Uuid;

use crate::compositor::{
    core::state::DesktopState,
    xdg_shell::{types::ManagedWindow, errors::XdgShellError},
};
use novade_domain::common_types::DomainWindowIdentifier;

// Placeholder for actual geometry calculation logic if domain services are not used.
fn get_placeholder_toplevel_geometry() -> Rectangle<i32, Logical> {
    Rectangle::from_loc_and_size((100, 100), (640, 480)) // Slightly offset for visibility
}

impl XdgShellHandler for DesktopState {
    type XdgShellClientData = XdgWmBaseClientData; // Using Smithay's default client data

    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        // DesktopState.xdg_shell_state is no longer an Option
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let surface_id = surface.wl_surface().id();
        info!("New XDG Toplevel created: {:?}", surface_id);

        let domain_window_id = DomainWindowIdentifier::new(
            format!("xdg-toplevel-{}", Uuid::new_v4())
        );

        let mut managed_window = ManagedWindow::new_toplevel(surface.clone(), domain_window_id.clone());

        // Initial Geometry & State
        let placeholder_geometry = get_placeholder_toplevel_geometry();
        managed_window.current_geometry = placeholder_geometry;
        // ManagedWindow::new_toplevel now inserts CustomToplevelState.
        // Smithay's XdgToplevelSurfaceData is automatically added by Smithay.

        surface.with_pending_state(|state| {
            state.size = Some(placeholder_geometry.size);
            // Set initial capabilities if needed
            state.wm_capabilities.set(XdgWmCapabilities::WindowMenu);
            state.wm_capabilities.set(XdgWmCapabilities::Maximize);
            state.wm_capabilities.set(XdgWmCapabilities::Fullscreen);
            // XDG protocol does not have a "minimize" capability/state.
        });
        surface.send_configure();
        info!("Sent initial configure for toplevel {:?}", surface_id);

        let window_arc = Arc::new(managed_window);
        
        // Store using DomainWindowIdentifier as key
        if self.windows.insert(domain_window_id.clone(), window_arc.clone()).is_some() {
            warn!("Replaced an existing window with the same DomainWindowIdentifier: {:?}", domain_window_id);
        }
        info!("ManagedWindow (toplevel) {:?} added to DesktopState.windows with Domain ID: {:?}", window_arc.id, domain_window_id);
    }

    fn new_popup(&mut self, surface: PopupSurface, _client_data: &Self::XdgShellClientData) {
        let surface_id = surface.wl_surface().id();
        info!("New XDG Popup created: {:?}", surface_id);

        let parent_wl_surface = match surface.get_parent_surface() {
            Some(parent) => parent,
            None => {
                error!("XDG Popup {:?} created without a parent. This should be a protocol error.", surface_id);
                // Posting an error to the client is complex here as we don't have direct access to the client resource.
                // Smithay's XdgShellState should ideally prevent this state.
                // For now, we destroy the surface resource if it's not already destroyed.
                surface.wl_surface().destroy();
                return;
            }
        };

        let parent_managed_window_arc = self.windows.values()
            .find(|mw| mw.wl_surface().as_ref() == Some(&parent_wl_surface))
            .cloned();

        let (parent_mw_id, parent_geometry) = match parent_managed_window_arc {
            Some(parent_arc) => (parent_arc.id, parent_arc.geometry()),
            None => {
                error!("Parent ManagedWindow not found for popup {:?}, parent WlSurface: {:?}", surface_id, parent_wl_surface.id());
                surface.wl_surface().destroy();
                return;
            }
        };
        
        // Smithay's XdgPopupSurfaceData should be automatically added by Smithay.
        let xdg_popup_data = surface.user_data().get::<XdgPopupSurfaceData>()
            .expect("XdgPopupSurfaceData missing from popup surface user_data. This is unexpected.");

        // The prompt mentions: surface.wl_surface().get_property::<Rectangle<i32, Logical>>("POPUP_SURFACE_SIZE_HINT").unwrap_or_default()
        // This is not a standard Smithay property. Popup size is usually determined by the client's first commit.
        // For initial configuration, we might not know the size.
        // calculate_popup_geometry_from_state takes an explicit popup_size.
        // Let's assume a zero size initially, client will commit a buffer and then we'll know.
        // Or, use the geometry from the positioner if it implies a size (unlikely).
        // The positioner primarily defines location constraints.
        // A common approach is to send an initial configure and let the client commit its content.
        // The geometry calculated here is more about placement.
        // Let's use a (0,0) size hint, meaning the actual size will come from the client's buffer.
        let calculated_popup_geometry = desktop_utils::calculate_popup_geometry_from_state(
            &xdg_popup_data.positioner,
            parent_geometry,
            Size::from((0,0)), // Placeholder for popup_size_hint, actual size from client commit
        );
        
        let domain_window_id = DomainWindowIdentifier::new(
            format!("xdg-popup-{}-for-{}", Uuid::new_v4(), parent_mw_id)
        );
        // Pass Some(parent_mw_id) to match ManagedWindow::new_popup signature
        let mut managed_popup = ManagedWindow::new_popup(surface.clone(), Some(parent_mw_id));
        managed_popup.current_geometry = calculated_popup_geometry;


        surface.send_configure(); // Send initial configure for the popup
        info!("Sent initial configure for popup {:?}", surface_id);

        let popup_arc = Arc::new(managed_popup);
        if self.windows.insert(domain_window_id.clone(), popup_arc.clone()).is_some() {
             warn!("Replaced an existing window (popup) with the same DomainWindowIdentifier: {:?}", domain_window_id);
        }
        info!("ManagedWindow (popup) {:?} added to DesktopState.windows with Domain ID: {:?}", popup_arc.id, domain_window_id);
    }

    fn map_toplevel(&mut self, surface: &ToplevelSurface) {
        let wl_surface = surface.wl_surface();
        info!("Map request for XDG Toplevel: {:?}", wl_surface.id());

        let window_arc = find_managed_window_by_surface_in_state(self, wl_surface).cloned();

        if let Some(window_arc_found) = window_arc {
            let mut new_window_state = (*window_arc_found).clone(); // Clone the ManagedWindow data
            new_window_state.is_mapped = true;
            let updated_arc = Arc::new(new_window_state);

            // Replace the old Arc with the new one in the map
            self.windows.insert(updated_arc.domain_id.clone(), updated_arc.clone());
            
            if let Some(space) = self.space.as_mut() {
                space.map_element(updated_arc.clone(), updated_arc.current_geometry.loc, true);
                info!("Mapped window {:?} (Domain ID: {:?}) to space at {:?}, activated.", updated_arc.id, updated_arc.domain_id, updated_arc.current_geometry.loc);
                // TODO: Placeholder for domain layer notification
                // self.workspace_manager_service.assign_window_to_active_workspace(...)
                space.damage_all_outputs();
            } else {
                error!("Space not available for mapping window {:?}", updated_arc.id);
            }
        } else {
            error!("ManagedWindow not found for mapping toplevel: {:?}", wl_surface.id());
        }
    }

    fn unmap_toplevel(&mut self, surface: &ToplevelSurface) {
        let wl_surface = surface.wl_surface();
        info!("Unmap request for XDG Toplevel: {:?}", wl_surface.id());

        let window_arc = find_managed_window_by_surface_in_state(self, wl_surface).cloned();

        if let Some(window_arc_found) = window_arc {
            if !window_arc_found.is_mapped {
                info!("Window {:?} (Domain ID: {:?}) was already unmapped. Ignoring unmap request.", window_arc_found.id, window_arc_found.domain_id);
                return;
            }

            let mut new_window_state = (*window_arc_found).clone();
            new_window_state.is_mapped = false;
            let updated_arc = Arc::new(new_window_state);
            
            self.windows.insert(updated_arc.domain_id.clone(), updated_arc.clone());

            if let Some(space) = self.space.as_mut() {
                space.unmap_elem(&updated_arc); // unmap_elem takes &W
                info!("Unmapped window {:?} (Domain ID: {:?}) from space.", updated_arc.id, updated_arc.domain_id);
                // TODO: Placeholder for domain layer notification
                space.damage_all_outputs();
            } else {
                error!("Space not available for unmapping window {:?}", updated_arc.id);
            }
        } else {
            error!("ManagedWindow not found for unmapping toplevel: {:?}", wl_surface.id());
        }
    }
    
    fn ack_configure(&mut self, surface: WlSurface, configure: XdgSurfaceConfigure) {
        info!("Ack configure for surface {:?}", surface.id());
        match desktop_utils::handle_xdg_surface_ack_configure(&surface, configure) {
            Ok(_) => {
                if ToplevelSurface::try_from_wlsurface(&surface).is_some() {
                    debug!("Toplevel {:?} acked configure.", surface.id());
                    // TODO: Potentially call domain policy if layout needs update.
                }
            }
            Err(err) => {
                warn!("Invalid ack_configure for surface {:?}: {:?}", surface.id(), err);
            }
        }
    }

    // --- Request Handlers (Stubs/Basic Logging) ---
    fn toplevel_request_set_title(&mut self, surface: &ToplevelSurface, title: String) {
        tracing::info!("Client for surface {:?} requested to set title to: {}", surface.wl_surface().id(), title);
        surface.set_title(title);
        tracing::debug!("TODO: Notify domain layer (e.g., TaskListService) about title change for surface {:?}", surface.wl_surface().id());
    }

    fn toplevel_request_set_app_id(&mut self, surface: &ToplevelSurface, app_id: String) {
        tracing::info!("Client for surface {:?} requested to set app_id to: {}", surface.wl_surface().id(), app_id);
        surface.set_app_id(app_id);
        tracing::debug!("TODO: Notify domain layer (e.g., WindowGroupingService) about app_id change for surface {:?}", surface.wl_surface().id());
    }

    fn toplevel_request_set_maximized(&mut self, surface: &ToplevelSurface) {
        tracing::info!("Client for surface {:?} requested to set maximized state.", surface.wl_surface().id());
        tracing::debug!("TODO: Consult WindowManagementPolicyService for maximized geometry and allowance for surface {:?}", surface.wl_surface().id());
        
        // For now, assume maximization is allowed and use a placeholder for size.
        // The actual size would depend on output geometry.
        surface.with_pending_state(|state| {
            state.states.set(XdgToplevelState::Maximized);
            // TODO: state.size = Some(output_size_for_maximize); 
            // If size is not set, client might choose one, or it's based on current.
            // For this step, we let the client adjust or keep current if not specified.
        });
        surface.send_configure();
        tracing::debug!("TODO: Update ManagedWindow geometry and notify WorkspaceManagerService/WindowMechanics for surface {:?}", surface.wl_surface().id());
    }

    fn toplevel_request_unset_maximized(&mut self, surface: &ToplevelSurface) {
        tracing::info!("Client for surface {:?} requested to unset maximized state.", surface.wl_surface().id());
        tracing::debug!("TODO: Consult WindowManagementPolicyService for restored geometry for surface {:?}", surface.wl_surface().id());

        // For now, assume unmaximization is allowed.
        // The actual restored size should ideally come from pre-maximize state or policy.
        surface.with_pending_state(|state| {
            state.states.unset(XdgToplevelState::Maximized);
            // TODO: state.size = Some(restored_size_from_policy_or_pre_maximize_state);
            // If size is not set, client will decide based on its pre-maximized state or defaults.
        });
        surface.send_configure();
        tracing::debug!("TODO: Update ManagedWindow geometry and notify WorkspaceManagerService/WindowMechanics for surface {:?}", surface.wl_surface().id());
    }

    fn toplevel_request_set_min_size(&mut self, surface: &ToplevelSurface, min_size: Option<Size<i32, Logical>>) {
        tracing::info!(
            "Client for surface {:?} requested to set min_size to: {:?}",
            surface.wl_surface().id(),
            min_size
        );

        // Update Smithay's internal state for the min_size constraint.
        surface.set_min_size(min_size);

        tracing::debug!(
            "TODO: Min_size hint received for surface {:?}. WindowManagementPolicyService or WindowMechanics might need to re-evaluate current window size. If current_size < new_min_size, a reconfigure might be needed.",
            surface.wl_surface().id()
        );
        // No configure is sent for this request by default; it's a constraint for future configurations.
        // However, if the current size violates the new min_size, the compositor might choose to reconfigure.
        // For this subtask, we only store the constraint.
    }

    fn toplevel_request_set_max_size(&mut self, surface: &ToplevelSurface, max_size: Option<Size<i32, Logical>>) {
        tracing::info!(
            "Client for surface {:?} requested to set max_size to: {:?}",
            surface.wl_surface().id(),
            max_size
        );

        // Update Smithay's internal state for the max_size constraint.
        surface.set_max_size(max_size);

        tracing::debug!(
            "TODO: Max_size hint received for surface {:?}. WindowManagementPolicyService or WindowMechanics might need to re-evaluate current window size. If current_size > new_max_size, a reconfigure might be needed.",
            surface.wl_surface().id()
        );
        // No configure is sent for this request by default; it's a constraint for future configurations.
        // However, if the current size violates the new max_size, the compositor might choose to reconfigure.
        // For this subtask, we only store the constraint.
    }
    
    fn toplevel_request_set_fullscreen(&mut self, surface: &ToplevelSurface, output: Option<wl_output::WlOutput>) {
        tracing::info!("Client for surface {:?} requested to set fullscreen state on output {:?}.", surface.wl_surface().id(), output.as_ref().map(|o| o.id()));
        tracing::debug!("TODO: Consult WindowManagementPolicyService for fullscreen geometry and allowance on output {:?} for surface {:?}", output.as_ref().map(|o| o.id()), surface.wl_surface().id());

        // For now, assume fullscreen is allowed.
        // The actual size would depend on the target output's geometry.
        surface.with_pending_state(|state| {
            state.states.set(XdgToplevelState::Fullscreen);
            state.fullscreen_output = output.as_ref().cloned(); // Store the target output
            // TODO: state.size = Some(output_size_for_fullscreen);
            // If size is not set, client might choose one, or it's based on output.
        });
        surface.send_configure();
        tracing::debug!("TODO: Update ManagedWindow geometry and notify WorkspaceManagerService/WindowMechanics for surface {:?}", surface.wl_surface().id());
    }

    fn toplevel_request_unset_fullscreen(&mut self, surface: &ToplevelSurface) {
        tracing::info!("Client for surface {:?} requested to unset fullscreen state.", surface.wl_surface().id());
        tracing::debug!("TODO: Consult WindowManagementPolicyService for restored geometry for surface {:?}", surface.wl_surface().id());

        // For now, assume unfullscreen is allowed.
        // The actual restored size should ideally come from pre-fullscreen state or policy.
        surface.with_pending_state(|state| {
            state.states.unset(XdgToplevelState::Fullscreen);
            state.fullscreen_output = None;
            // TODO: state.size = Some(restored_size_from_policy_or_pre_fullscreen_state);
        });
        surface.send_configure();
        tracing::debug!("TODO: Update ManagedWindow geometry and notify WorkspaceManagerService/WindowMechanics for surface {:?}", surface.wl_surface().id());
    }

    fn toplevel_request_set_minimized(&mut self, surface: &ToplevelSurface) {
        tracing::info!("Client for surface {:?} requested to set minimized state.", surface.wl_surface().id());
        tracing::debug!("TODO: Consult WindowManagementPolicyService for minimized state allowance for surface {:?}", surface.wl_surface().id());

        // XDG Shell itself doesn't have a "minimized" state to send to the client.
        // This is a compositor-internal state. We update our ManagedWindow.
        // The ManagedWindow::set_minimized method handles updating CustomToplevelState.
        update_managed_window_state(self, surface.wl_surface(), |mw| {
            mw.set_minimized(true); // This internally updates CustomToplevelState in user_data
        });
        
        tracing::debug!("TODO: Unmap ManagedWindow via WindowMechanics, notify domain layer (e.g., TaskListService) for surface {:?}", surface.wl_surface().id());
        // No surface.send_configure() here as there's no XDG minimized state to configure.
    }
    
    fn toplevel_request_move(&mut self, surface: &ToplevelSurface, seat: &Seat<Self>, serial: Serial) {
        tracing::info!("Client for surface {:?} requested interactive move (serial: {:?}, seat: {:?})", surface.wl_surface().id(), serial, seat.name());

        if !seat.validate_grab_serial(serial) {
            tracing::warn!("Invalid serial for move request from client for surface {:?}", surface.wl_surface().id());
            return;
        }
        
        let pointer_handle = match seat.get_pointer() {
            Some(p) => p,
            None => {
                tracing::warn!("No pointer found on seat {:?} for move request.", seat.name());
                return;
            }
        };

        let window_arc = match find_managed_window_by_surface_in_state(self, surface.wl_surface()) {
            Some(w_arc) => w_arc.clone(),
            None => {
                warn!("ManagedWindow not found for move request of surface {:?}", surface.wl_surface().id());
                return;
            }
        };

        let initial_window_geometry = window_arc.geometry(); // From Window trait
        let op_state = MoveResizeState {
            window_arc: window_arc.clone(),
            op_type: InteractiveOpType::Move,
            start_pointer_pos_global: self.pointer_location, // From DesktopState
            initial_window_geometry,
        };

        let focus_target: PointerTarget<DesktopState> = match op_state.window_arc.wl_surface() {
            Some(s) => s.into(), // WlSurface can be converted into PointerTarget
            None => {
                warn!("Cannot start move grab: ManagedWindow has no WlSurface. Window ID: {:?}", op_state.window_arc.id);
                return;
            }
        };
        let start_data = PointerGrabStartData {
            focus: Some((focus_target, (0,0).into())), // Focus on the surface origin relative to itself
            location: self.pointer_location, // Current global pointer location from DesktopState
        };
        
        let grab = NovaMoveGrab::new(start_data, op_state);
        pointer_handle.set_grab(self, grab, serial, PointerFocus::Clear); // `self` is &mut DesktopState
        tracing::info!("Move grab started for window {:?}", window_arc.id);
    }

    fn toplevel_request_resize(&mut self, surface: &ToplevelSurface, seat: &Seat<Self>, serial: Serial, edges: XdgResizeEdge) {
        tracing::info!("Client for surface {:?} requested interactive resize (serial: {:?}, seat: {:?}, edges: {:?})", surface.wl_surface().id(), serial, seat.name(), edges);
        
        if !seat.validate_grab_serial(serial) {
            tracing::warn!("Invalid serial for resize request from client for surface {:?}", surface.wl_surface().id());
            return;
        }

        let pointer_handle = match seat.get_pointer() {
            Some(p) => p,
            None => {
                tracing::warn!("No pointer found on seat {:?} for resize request.", seat.name());
                return;
            }
        };

        let window_arc = match find_managed_window_by_surface_in_state(self, surface.wl_surface()) {
            Some(w_arc) => w_arc.clone(),
            None => {
                warn!("ManagedWindow not found for resize request of surface {:?}", surface.wl_surface().id());
                return;
            }
        };
        
        let initial_window_geometry = window_arc.geometry();
        let op_state = MoveResizeState {
            window_arc: window_arc.clone(),
            op_type: InteractiveOpType::Resize(edges),
            start_pointer_pos_global: self.pointer_location,
            initial_window_geometry,
        };

        let focus_target: PointerTarget<DesktopState> = match op_state.window_arc.wl_surface() {
            Some(s) => s.into(),
            None => {
                warn!("Cannot start resize grab: ManagedWindow has no WlSurface. Window ID: {:?}", op_state.window_arc.id);
                return;
            }
        };
        let start_data = PointerGrabStartData {
            focus: Some((focus_target, (0,0).into())),
            location: self.pointer_location,
        };

        let grab = NovaResizeGrab::new(start_data, op_state);
        pointer_handle.set_grab(self, grab, serial, PointerFocus::Clear);
        tracing::info!("Resize grab started for window {:?} with edges {:?}", window_arc.id, edges);
    }

    fn toplevel_request_show_window_menu(&mut self, surface: &ToplevelSurface, seat: &Seat<Self>, serial: Serial, position: Point<i32, Logical>) {
        tracing::info!("Client for surface {:?} requested to show window menu (serial: {:?}, seat: {:?}, position: {:?})", surface.wl_surface().id(), serial, seat.name(), position);

        if !seat.validate_grab_serial(serial) {
            tracing::warn!("Invalid serial for show_window_menu request from client for surface {:?}", surface.wl_surface().id());
            // Window menu is often tied to an input event (e.g. right click on title bar).
            // If serial is invalid, it means the triggering event is not the one the client thinks it is.
            return;
        }

        tracing::debug!("TODO: Signal UI layer to show a window context menu for surface {:?} at global coordinates derived from 'position'.", surface.wl_surface().id());
        // This would involve:
        // 1. Translating the surface-local `position` to global screen coordinates.
        // 2. Triggering a UI element (e.g., a special layer-shell surface or an overlay plane)
        //    that displays the context menu.
        // 3. Handling interactions with that menu, which might then call back into
        //    compositor logic (e.g., to request maximize, minimize, close).
    }

    fn toplevel_request_set_window_geometry(&mut self, surface: &ToplevelSurface, geometry: Rectangle<i32, Logical>) {
        tracing::info!(
            "Client for surface {:?} requested to set window geometry to: {:?}",
            surface.wl_surface().id(),
            geometry
        );

        // Update Smithay's internal state for the window geometry hint.
        surface.set_window_geometry(geometry);

        tracing::debug!(
            "TODO: Window geometry hint received for surface {:?}. WindowManagementPolicyService or decoration logic might use this. Current compositor geometry for this window remains authoritative.",
            surface.wl_surface().id()
        );
        // No configure is sent for this request; it's a hint from the client.
    }
    
    fn grab(&mut self, surface: PopupSurface, seat: &Seat<Self>, serial: Serial) {
        info!("Popup grab requested for {:?} by seat {:?}, serial {:?}", surface.wl_surface().id(), seat.name(), serial);
        // This is for an "implicit grab" when a popup is created.
        // Smithay's XdgShellState usually handles the grab logic.
        // We might need to focus the popup or ensure input goes to it.
        // For now, just logging.
    }

    fn toplevel_request_set_parent(&mut self, surface: &ToplevelSurface, parent: Option<&ToplevelSurface>) {
        let child_wl_surface = surface.wl_surface();
        tracing::info!(
            "Client for surface {:?} requested to set parent to {:?}",
            child_wl_surface.id(),
            parent.map(|p| p.wl_surface().id())
        );

        // Let Smithay handle its internal parent/child relationship for the XDG protocol.
        surface.set_parent(parent);

        // Now, update our ManagedWindow's parent_id.
        if let Some(child_mw_arc) = find_managed_window_by_surface_in_state(self, child_wl_surface).cloned() {
            let parent_managed_window_uuid: Option<Uuid> = parent.and_then(|parent_toplevel_surface| {
                find_managed_window_by_surface_in_state(self, parent_toplevel_surface.wl_surface())
                    .map(|parent_mw| parent_mw.id) // Get the Uuid of the parent ManagedWindow
            });

            if parent.is_some() && parent_managed_window_uuid.is_none() {
                tracing::warn!(
                    "Parent ToplevelSurface {:?} provided for child {:?}, but corresponding parent ManagedWindow not found in our records.",
                    parent.unwrap().wl_surface().id(),
                    child_wl_surface.id()
                );
            }

            match child_mw_arc.parent_id.lock() {
                Ok(mut guard) => {
                    *guard = parent_managed_window_uuid;
                    tracing::info!(
                        "Set ManagedWindow {:?} parent_id to {:?}",
                        child_mw_arc.id, parent_managed_window_uuid
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to lock parent_id Mutex for ManagedWindow {:?}: {}", child_mw_arc.id, e);
                }
            }
            
            tracing::debug!(
                "TODO: Notify domain layer (e.g., WindowManagementService) about parent change for window {:?} (ManagedWindow ID: {:?}), new parent ManagedWindow ID: {:?}",
                child_mw_arc.domain_id,
                child_mw_arc.id,
                parent_managed_window_uuid
            );

        } else {
            tracing::warn!(
                "ManagedWindow not found for surface {:?} when trying to set parent.",
                child_wl_surface.id()
            );
        }
    }


    fn toplevel_destroyed(&mut self, toplevel: ToplevelSurface) {
        let surface_id = toplevel.wl_surface().id();
        info!("XDG Toplevel destroyed: {:?}", surface_id);

        let window_arc = find_managed_window_by_surface_in_state(self, toplevel.wl_surface()).cloned();
        if let Some(window_arc_found) = window_arc {
            if window_arc_found.is_mapped { // Check our own flag
                if let Some(space) = self.space.as_mut() {
                    space.unmap_elem(&window_arc_found);
                    info!("Unmapped window {:?} (Domain ID: {:?}) from space.", window_arc_found.id, window_arc_found.domain_id);
                }
            }
            self.windows.remove(&window_arc_found.domain_id);
            info!("Removed ManagedWindow {:?} (Domain ID: {:?}) from DesktopState.windows.", window_arc_found.id, window_arc_found.domain_id);
            // TODO: Placeholder for domain layer notification
        } else {
            warn!("Could not find ManagedWindow in DesktopState.windows for destroyed toplevel: {:?}", surface_id);
        }
        
        if let Some(space) = self.space.as_mut() {
            space.damage_all_outputs();
        }
    }

    fn popup_destroyed(&mut self, popup: PopupSurface) {
        let surface_id = popup.wl_surface().id();
        info!("XDG Popup destroyed: {:?}", surface_id);

        let window_arc = find_managed_window_by_surface_in_state(self, popup.wl_surface()).cloned();
        if let Some(window_arc_found) = window_arc {
            self.windows.remove(&window_arc_found.domain_id);
            info!("Removed ManagedWindow (popup) {:?} (Domain ID: {:?}) from DesktopState.windows.", window_arc_found.id, window_arc_found.domain_id);
             // Popups are generally not directly in self.space for unmapping, handled via parent.
            // TODO: Placeholder for domain layer notification if popups are tracked.
        } else {
            warn!("Could not find ManagedWindow in DesktopState.windows for destroyed popup: {:?}", surface_id);
        }
    }
}

// Helper to find ManagedWindow by WlSurface.
// Inefficient, placeholder. Better: Uuid in WlSurface UserDataMap.
fn find_managed_window_by_surface_in_state<'a>(state: &'a DesktopState, surface: &WlSurface) -> Option<&'a Arc<ManagedWindow>> {
    state.windows.values().find(|mw| mw.wl_surface().as_ref() == Some(surface))
}

// Helper for mutating ManagedWindow state when it's stored in Arc within DesktopState.windows
// This is also a bit convoluted due to clone-and-replace. Interior mutability on ManagedWindow fields
// or changing DesktopState.windows to store &mut ManagedWindow (if lifetimes allow) would be cleaner.
fn update_managed_window_state<F>(desktop_state: &mut DesktopState, surface: &WlSurface, mutator: F)
where
    F: FnOnce(&mut ManagedWindow),
{
    if let Some(window_arc) = find_managed_window_by_surface_in_state(desktop_state, surface).cloned() {
        let mut current_mw_data = (*window_arc).clone(); // Clone inner ManagedWindow
        mutator(&mut current_mw_data); // Mutate the clone
        let new_arc = Arc::new(current_mw_data); // Create new Arc
        desktop_state.windows.insert(new_arc.domain_id.clone(), new_arc); // Replace in map
    } else {
        warn!("Attempted to update state for a window not found for WlSurface: {:?}", surface.id());
    }
}


// --- GlobalDispatch Implementations ---
use smithay::reexports::wayland_server::{GlobalDispatch, New, DataInit, DisplayHandle as WaylandDisplayHandle, Client as WaylandClient};
use smithay::wayland::shell::xdg::xdg_wm_base::XdgWmBase; // For GlobalDispatch target
use smithay::reexports::wayland_protocols::xdg::activation::v1::server::xdg_activation_v1::XdgActivationV1; // For XDG Activation global

impl GlobalDispatch<XdgWmBase, XdgWmBaseClientData> for DesktopState {
    // UserData is XdgWmBaseClientData by type alias in XdgShellHandler
    fn bind(
        state: &mut Self, // DesktopState
        _handle: &WaylandDisplayHandle,
        client: &WaylandClient,
        resource: New<XdgWmBase>,
        _global_data: &XdgWmBaseClientData, // This is the global's data, not used here.
        data_init: &mut DataInit<'_, Self>,
    ) {
        info!(client_id = ?client.id(), resource_id = ?resource.id(), "Client binding to xdg_wm_base");
        
        // XdgShellState::new_client internally creates and stores XdgWmBaseClientData for the client.
        // It returns a reference to this client-specific data.
        let client_xdg_shell_data = state.xdg_shell_state.new_client(client);
        
        // Initialize the XdgWmBase resource with the client-specific data.
        data_init.init(resource, client_xdg_shell_data.clone()); // Clone Arc<Mutex<XdgClientData>>
    }
}

impl GlobalDispatch<XdgActivationV1, ()> for DesktopState {
    fn bind(
        _state: &mut Self, // DesktopState
        _handle: &WaylandDisplayHandle,
        client: &WaylandClient,
        resource: New<XdgActivationV1>,
        _global_data: &(), // No global data for XdgActivationV1 itself
        data_init: &mut DataInit<'_, Self>,
    ) {
        info!(client_id = ?client.id(), resource_id = ?resource.id(), "Client binding to xdg_activation_v1");
        
        // XdgActivationState handles the binding and client interactions.
        // We just need to initialize the resource. It doesn't typically have its own user data.
        // The XdgActivationState itself will be accessed via delegate_xdg_activation when requests come in.
        data_init.init(resource, ());
    }
}
