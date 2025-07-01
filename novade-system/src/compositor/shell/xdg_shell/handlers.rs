use smithay::{
    reexports::wayland_server::{
        protocol::{wl_seat::WlSeat, wl_surface::WlSurface, wl_output::WlOutput as WaylandOutput},
        DisplayHandle, Resource, // Added Resource for .post_error()
    },
    utils::{Logical, Point, Rectangle, Serial, Size},
    wayland::{
        seat::Seat,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface,
            XdgShellHandler, XdgShellState, XdgWmBaseClientData, // Keep XdgWmBaseClientData for new_popup signature
            ToplevelState as XdgToplevelStateSmithay, // Aliased to avoid conflict with our XdgSurfaceState
            XdgSurfaceConfigure,
            ResizeEdge as XdgResizeEdge, // Explicit import
            xdg_surface::Error as XdgSurfaceError, // For posting errors
        },
    },
    input::SeatHandler,
};
use std::sync::Arc;

use crate::compositor::{
    core::state::DesktopState,
    shell::xdg_shell::types::{
        DomainWindowIdentifier, ManagedWindow, XdgSurfaceUserData, XdgSurfaceRole, XdgSurfaceState,
    },
    errors::XdgShellError,
};
// uuid::Uuid is not directly used here anymore, DomainWindowIdentifier::new_v4() handles it.

// --- XDG Decoration Imports ---
// ANCHOR: XdgDecorationImportBlock
use smithay::wayland::shell::xdg::decoration::{
    XdgDecorationHandler, XdgToplevelDecoration, Mode as XdgDecorationMode, ServerDecorationState,
};
// ANCHOR_END: XdgDecorationImportBlock

// Note: The helper functions find_managed_window_by_wl_surface_mut and find_managed_window_by_wl_surface
// have been moved to DesktopState as methods. Call sites in this file will be updated to use self.find_managed_window_by_wl_surface().

impl XdgShellHandler for DesktopState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    // ANCHOR: NewToplevelHandler
    fn new_toplevel(&mut self, toplevel_surface: ToplevelSurface) {
        let xdg_surface_wrapper = toplevel_surface.xdg_surface().clone(); // Smithay's XdgSurface wrapper
        let wl_surface = xdg_surface_wrapper.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "XDG New Toplevel requested");

        let user_data_arc = match xdg_surface_wrapper.user_data().get::<Arc<XdgSurfaceUserData>>() {
            Some(data) => data.clone(),
            None => {
                tracing::error!(surface_id = ?wl_surface.id(), "XdgSurfaceUserData not found for new toplevel. This is a bug from new_xdg_surface handler.");
                toplevel_surface.xdg_surface().post_error(XdgSurfaceError::Defunct, "Internal compositor error: UserData missing");
                return;
            }
        };

        // Validate and set role
        {
            let mut role_guard = user_data_arc.role.lock().unwrap();
            if *role_guard != XdgSurfaceRole::None {
                tracing::warn!(surface_id = ?wl_surface.id(), current_role = ?*role_guard, "Client attempted to assign role 'toplevel' to an XDG surface that already has a role.");
                toplevel_surface.xdg_surface().post_error(XdgSurfaceError::Role, "Surface already has a role.");
                return;
            }
            *role_guard = XdgSurfaceRole::Toplevel;

            let mut state_guard = user_data_arc.state.lock().unwrap();
            if *state_guard == XdgSurfaceState::Destroyed {
                tracing::warn!(surface_id = ?wl_surface.id(), "Client attempted to create toplevel for an already destroyed XDG surface.");
                toplevel_surface.xdg_surface().post_error(XdgSurfaceError::Defunct, "Surface is defunct.");
                return;
            }
            // Initial state is PendingConfiguration, will transition on first ack_configure
        }
        tracing::info!(surface_id = ?wl_surface.id(), "XDG surface role set to Toplevel. State is PendingConfiguration.");

        let domain_window_id = DomainWindowIdentifier::new_v4();
        let mut managed_window = ManagedWindow::new_toplevel(toplevel_surface.clone(), domain_window_id);

        // Initial configure: Send a size and state.
        // Client is expected to ack_configure this serial.
        // Use a default size or calculate based on output. For now, let's use ManagedWindow's default.
        let initial_window_state = managed_window.state.read().unwrap();
        let initial_content_size = if managed_window.manager_data.read().unwrap().decorations {
            // If SSD, client gets content area size
            Size::from((
                (initial_window_state.size.w - 2 * types::DEFAULT_BORDER_SIZE).max(1),
                (initial_window_state.size.h - types::DEFAULT_TITLE_BAR_HEIGHT - 2 * types::DEFAULT_BORDER_SIZE).max(1)
            ))
        } else {
            initial_window_state.size
        };

        toplevel_surface.with_pending_state(|xdg_toplevel_pending_state| {
            xdg_toplevel_pending_state.size = Some(initial_content_size);
            // No states like maximized/fullscreen initially.
        });
        let configure_serial = toplevel_surface.send_configure();

        // Store this serial in both ManagedWindow (for its own tracking) and XdgSurfaceUserData (for ack_configure validation)
        managed_window.last_configure_serial = Some(configure_serial);
        *user_data_arc.last_compositor_configure_serial.lock().unwrap() = Some(configure_serial);
        // XdgSurfaceUserData.state remains PendingConfiguration until client acks.

        let window_arc = Arc::new(managed_window);
        self.windows.insert(domain_window_id, window_arc.clone());

        tracing::info!(
            "New XDG Toplevel (domain_id: {:?}, compositor_id: {:?}) created. Sent initial configure (serial: {:?}, content_size: {:?}). UserData role: {:?}, state: {:?}. Awaiting map and ack_configure.",
            domain_window_id,
            window_arc.id,
            configure_serial,
            initial_content_size,
            user_data_arc.role.lock().unwrap(),
            user_data_arc.state.lock().unwrap()
        );
    }
    }
    // ANCHOR_END: NewToplevelHandler

    // ANCHOR: NewPopupHandler
    fn new_popup(&mut self, popup_surface: PopupSurface, _client_data: &XdgWmBaseClientData) {
        let xdg_surface_wrapper = popup_surface.xdg_surface().clone(); // Smithay's XdgSurface wrapper
        let wl_surface = xdg_surface_wrapper.wl_surface();
        tracing::info!(popup_surface_id = ?wl_surface.id(), "XDG New Popup requested");

        let user_data_arc = match xdg_surface_wrapper.user_data().get::<Arc<XdgSurfaceUserData>>() {
            Some(data) => data.clone(),
            None => {
                tracing::error!(surface_id = ?wl_surface.id(), "XdgSurfaceUserData not found for new popup. This is a bug from new_xdg_surface handler.");
                popup_surface.xdg_surface().post_error(XdgSurfaceError::Defunct, "Internal compositor error: UserData missing");
                return;
            }
        };

        let parent_wl_surface = match popup_surface.get_parent_surface() {
            Some(s) => s,
            None => {
                tracing::warn!(surface_id = ?wl_surface.id(), "Client attempted to create a popup without a parent surface.");
                // Protocol: "if parent is not set, the xdg_wm_base.error.invalid_popup_parent protocol error is raised."
                // Smithay's XdgShellState might handle this before calling new_popup.
                // If we reach here, it implies Smithay allowed it, or the parent became invalid after XDG surface creation.
                // For safety, post error on xdg_surface.
                popup_surface.xdg_surface().post_error(XdgSurfaceError::InvalidPopupParent, "Popup parent is missing or invalid.");
                return;
            }
        };

        // Validate and set role
        {
            let mut role_guard = user_data_arc.role.lock().unwrap();
            if *role_guard != XdgSurfaceRole::None {
                tracing::warn!(surface_id = ?wl_surface.id(), current_role = ?*role_guard, "Client attempted to assign role 'popup' to an XDG surface that already has a role.");
                popup_surface.xdg_surface().post_error(XdgSurfaceError::Role, "Surface already has a role.");
                return;
            }
            *role_guard = XdgSurfaceRole::Popup;

            let mut state_guard = user_data_arc.state.lock().unwrap();
            if *state_guard == XdgSurfaceState::Destroyed {
                tracing::warn!(surface_id = ?wl_surface.id(), "Client attempted to create popup for an already destroyed XDG surface.");
                popup_surface.xdg_surface().post_error(XdgSurfaceError::Defunct, "Surface is defunct.");
                return;
            }
        }
        tracing::info!(surface_id = ?wl_surface.id(), "XDG surface role set to Popup. State is PendingConfiguration.");

        let parent_managed_window_arc = match self.find_managed_window_by_wl_surface(&parent_wl_surface) {
            Some(arc) => arc,
            None => {
                tracing::warn!(surface_id = ?wl_surface.id(), parent_surface_id = ?parent_wl_surface.id(), "Parent WlSurface for popup does not correspond to a known ManagedWindow.");
                popup_surface.xdg_surface().post_error(XdgSurfaceError::InvalidPopupParent, "Popup parent is not a recognized compositor window.");
                return;
            }
        };
        
        tracing::info!(popup_surface_id = ?wl_surface.id(), parent_window_id = ?parent_managed_window_arc.id, "XDG New Popup parent lookup successful.");

        let mut managed_popup = ManagedWindow::new_popup(
            popup_surface.clone(),
            parent_managed_window_arc.domain_id(), // Popups can share domain context, or have their own
            Some(parent_managed_window_arc.clone())
        );

        let positioner_state = popup_surface.get_positioner(); // Positioner as defined by client
        let parent_geometry = parent_managed_window_arc.geometry(); // Current geometry of parent ManagedWindow

        let popup_initial_geom = smithay::wayland::shell::xdg::calculate_popup_geometry(
            &positioner_state,
            parent_geometry, // Geometry of the parent ManagedWindow's content area or overall rect
            wl_surface,      // The WlSurface of the popup itself
        );

        // Update ManagedWindow's geometry. Note: current_geometry is Arc<RwLock<...>>
        *managed_popup.current_geometry.write().unwrap() = popup_initial_geom;
        // Also update the WindowState's position and size fields
        managed_popup.state.write().unwrap().position = popup_initial_geom.loc;
        managed_popup.state.write().unwrap().size = popup_initial_geom.size;


        // Send initial configure for the popup
        let configure_serial = popup_surface.send_configure();
        managed_popup.last_configure_serial = Some(configure_serial);
        *user_data_arc.last_compositor_configure_serial.lock().unwrap() = Some(configure_serial);

        let popup_arc = Arc::new(managed_popup);
        self.windows.insert(popup_arc.domain_id(), popup_arc.clone());

        tracing::info!(
            "New XDG Popup (domain_id: {:?}, compositor_id: {:?}) created. Sent initial configure (serial: {:?}). Calculated geom: {:?}. Parent: {:?}. UserData role: {:?}, state: {:?}.",
            popup_arc.domain_id(),
            popup_arc.id,
            configure_serial,
            popup_initial_geom,
            parent_managed_window_arc.id,
            user_data_arc.role.lock().unwrap(),
            user_data_arc.state.lock().unwrap()
        );
        // Note: Popups are typically mapped implicitly by the client after receiving configure and committing content.
        // There isn't a separate `map_popup` handler in XdgShellHandler.
        // Visibility and stacking are handled by the compositor during rendering.
        self.space.damage_all_outputs(); // Damage because a new surface (popup) might appear.
    }
    // ANCHOR_END: NewPopupHandler


    fn map_toplevel(&mut self, surface: &ToplevelSurface) {
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "XDG Toplevel map request");

        // ANCHOR_REF: XdgSurfaceUserDataMapAccess
        // Retrieve our XdgSurfaceUserData to check its state, if necessary.
        let xdg_surface_obj = surface.xdg_surface(); // Get the underlying XdgSurface
        if let Some(user_data_arc) = xdg_surface_obj.user_data().get::<Arc<XdgSurfaceUserData>>() {
            let mut state_guard = user_data_arc.state.lock().unwrap();
            if *state_guard == XdgSurfaceState::Destroyed {
                tracing::warn!(surface_id = ?wl_surface.id(), "Attempted to map an XDG surface that is already destroyed.");
                // Don't map, client should not be doing this.
                // Smithay might also prevent this.
                return;
            }
            // If state is PendingConfiguration, it will transition to Configured after first commit/ack.
            // For map, we assume configuration is being handled.
            // If we wanted to enforce map only after first configure:
            // if *state_guard == XdgSurfaceState::PendingConfiguration {
            //     tracing::warn!(surface_id = ?wl_surface.id(), "Attempted to map an XDG surface that is not yet configured.");
            //     xdg_surface_obj.post_error(XdgSurfaceError::UnconfiguredSurface, "Surface must be configured before map.");
            //     return;
            // }
        } else {
            // This is a critical error, as XdgSurfaceUserData should always be present from new_xdg_surface.
            tracing::error!(surface_id = ?wl_surface.id(), "XdgSurfaceUserData not found on XDG Toplevel map. This is a bug.");
            xdg_surface_obj.post_error(XdgSurfaceError::Defunct, "Internal compositor error: surface data missing on map");
            return;
        }


        // Ensure the surface has our SurfaceData (from surface_management.rs), indicating it's a surface we know and manage
        // This also implies it has gone through CompositorHandler::new_surface
        if wl_surface.data_map().get::<Arc<std::sync::Mutex<crate::compositor::surface_management::SurfaceData>>>().is_none() {
            tracing::error!("Attempted to map XDG Toplevel (surface {:?}) that does not have our compositor SurfaceData. This should not happen.", wl_surface.id());
            // TODO: Consider destroying the client or the surface if it's in such an inconsistent state.
            surface.xdg_surface().post_error(XdgSurfaceError::Defunct, "Internal compositor error: missing core surface data");
            return;
        }


        if let Some(window_arc) = self.find_managed_window_by_wl_surface(wl_surface) { // MODIFIED
            // Arc::strong_count(&window_arc) can be logged here to check reference counts if needed.
            {
                let mut managed_win_state_guard = window_arc.state.write().unwrap();
                managed_win_state_guard.is_mapped = true;
                // ANCHOR: SetActivatedOnMap
                // Set activated to true when mapped, assuming it will receive focus.
                managed_win_state_guard.activated = true;
                // ANCHOR_END: SetActivatedOnMap

                // ANCHOR: UnminimizeOnMap
                // If the window was minimized, mapping it should unminimize it.
                if managed_win_state_guard.minimized {
                    managed_win_state_guard.minimized = false;
                    tracing::info!("Window {:?} unminimized due to map request.", window_arc.id);
                    surface.with_pending_state(|xdg_toplevel_pending_state| {
                        xdg_toplevel_pending_state.states.unset(XdgToplevelStateSmithay::Minimized);
                    });
                }
                // ANCHOR_END: UnminimizeOnMap

                // ANCHOR: AssignToActiveWorkspaceOnMap
                // ANCHOR: PerOutputWindowAssignmentOnMap
                let primary_output_name_guard = self.primary_output_name.read().unwrap();
                let target_output_name = primary_output_name_guard.as_deref()
                    .unwrap_or_else(|| {
                        // Fallback if no primary output is set (should ideally not happen in multi-monitor)
                        self.outputs.first().map_or("HEADLESS-1", |o| o.name()).to_string()
                    });
                drop(primary_output_name_guard);

                let active_workspaces_guard = self.active_workspaces.read().unwrap();
                let active_ws_id_on_target_output = active_workspaces_guard.get(&target_output_name).copied();
                drop(active_workspaces_guard);

                if let Some(active_ws_id) = active_ws_id_on_target_output {
                    *window_arc.workspace_id.write().unwrap() = Some(active_ws_id);
                    *window_arc.output_name.write().unwrap() = Some(target_output_name.clone());

                    if let Some(workspaces_on_output) = self.output_workspaces.get(&target_output_name) {
                        if let Some(active_workspace_arc) = workspaces_on_output.iter().find(|ws_arc| ws_arc.read().unwrap().id == active_ws_id) {
                            active_workspace_arc.read().unwrap().add_window(window_arc.domain_id);
                            tracing::info!("Assigned window {:?} (domain_id: {:?}) to active workspace {} on output {}.",
                                window_arc.id, window_arc.domain_id, active_ws_id, target_output_name);
                        } else {
                             tracing::error!("Active workspace ID {} not found on output {} for window {:?}.",
                                active_ws_id, target_output_name, window_arc.id);
                        }
                    } else {
                        tracing::error!("No workspaces found for output {} when assigning window {:?}.",
                            target_output_name, window_arc.id);
                    }
                } else {
                     tracing::error!("No active workspace ID found for target output {} when assigning window {:?}.",
                        target_output_name, window_arc.id);
                }
                // ANCHOR_END: PerOutputWindowAssignmentOnMap
                // ANCHOR_END: AssignToActiveWorkspaceOnMap
            }

            // ANCHOR: CallApplyLayoutForOutputOnMap
            // Determine target output name again for calling apply_layout_for_output
            let target_output_name_for_layout = window_arc.output_name.read().unwrap().clone()
                .unwrap_or_else(|| {
                    tracing::warn!("Window {:?} has no output assigned, defaulting to primary for layout application.", window_arc.id);
                    self.primary_output_name.read().unwrap().as_deref().unwrap_or("HEADLESS-1").to_string()
                });

            crate::compositor::tiling::apply_layout_for_output(self, &target_output_name_for_layout);
            // ANCHOR_END: CallApplyLayoutForOutputOnMap

            tracing::info!("XDG Toplevel {:?} (surface {:?}) processed for mapping. Layout applied on output {}. Activation set.",
                         window_arc.id, wl_surface.id(), target_output_name_for_layout);

            let seat = &self.seat;
            if let Some(keyboard) = seat.get_keyboard() {
                if wl_surface.alive() {
                    keyboard.set_focus(self, Some(wl_surface.clone()), Serial::now());
                    tracing::info!("Set keyboard focus to newly mapped XDG Toplevel {:?} (surface {:?}).",
                                 window_arc.id, wl_surface.id());
                    // Note: `activated` was set above. If focus setting fails or is deferred,
                    // `activated` might be prematurely true. True activation depends on successful focus.
                    // For this subtask, we link it to the intent to focus on map.
                } else {
                    tracing::warn!("Attempted to focus newly mapped XDG Toplevel {:?} (surface {:?}), but its WlSurface is no longer alive.",
                                 window_arc.id, wl_surface.id());
                    // If focus failed, reflect this in activation state.
                    window_arc.state.write().unwrap().activated = false;
                    tracing::info!("XDG Toplevel {:?} marked as deactivated due to failed focus attempt on map.", window_arc.id);
                }
            } else {
                tracing::warn!("No keyboard found on seat {:?} to focus newly mapped XDG Toplevel {:?}.",
                             seat.name(), window_arc.id);
                // If no keyboard, focus cannot be given, so it's not truly activated.
                window_arc.state.write().unwrap().activated = false;
                tracing::info!("XDG Toplevel {:?} marked as deactivated due to no keyboard on map.", window_arc.id);
            }

            self.space.damage_all_outputs();
        } else {
            tracing::warn!("Map request for an XDG Toplevel whose WlSurface ({:?}) is not associated with any ManagedWindow.",
                         wl_surface.id());
            // This can happen if new_toplevel failed to create a ManagedWindow due to role error or other issues.
            // The XDG surface might still exist but not be tracked by our window manager.
            surface.xdg_surface().post_error(XdgSurfaceError::Defunct, "Compositor failed to track this toplevel for mapping.");
        }
    }

    fn unmap_toplevel(&mut self, surface: &ToplevelSurface) {
        let wl_surface = surface.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "XDG Toplevel unmap request");

        // It's good practice to check XdgSurfaceUserData state here too, though unmap is usually safe.
        // If surface was destroyed, this might be redundant but harmless.
        let xdg_surface_obj = surface.xdg_surface();
        if let Some(user_data_arc) = xdg_surface_obj.user_data().get::<Arc<XdgSurfaceUserData>>() {
            let state_guard = user_data_arc.state.lock().unwrap();
            if *state_guard == XdgSurfaceState::Destroyed {
                 tracing::info!(surface_id = ?wl_surface.id(), "Unmap requested for an already destroyed XDG Toplevel. Cleanup should have occurred.");
                // No error to client usually, unmap is part of cleanup.
            }
        } else {
            tracing::error!(surface_id = ?wl_surface.id(), "XdgSurfaceUserData not found on XDG Toplevel unmap. This is a bug.");
            // Don't post error here as surface might be getting destroyed anyway.
        }

        if let Some(window_arc) = self.find_managed_window_by_wl_surface(wl_surface) { // MODIFIED
            {
                let mut managed_win_state_guard = window_arc.state.write().unwrap();
                managed_win_state_guard.is_mapped = false;
                // ANCHOR: SetDeactivatedOnUnmap
                // Deactivate when unmapped
                managed_win_state_guard.activated = false;
                tracing::info!("XDG Toplevel {:?} marked as deactivated due to unmap.", window_arc.id);
                // ANCHOR_END: SetDeactivatedOnUnmap
            }
            self.space.unmap_window(&window_arc);
            tracing::info!("XDG Toplevel {:?} unmapped from space.", window_arc.id);
            self.space.damage_all_outputs();
        } else {
            tracing::warn!("Unmap request for unknown XDG Toplevel (surface_id: {:?})", wl_surface.id());
        }
    }


    // Using self.find_managed_window_by_wl_surface (read-only) for these setters
    fn toplevel_request_set_title(&mut self, surface: &ToplevelSurface, title: String) {
        if let Some(window_arc) = self.find_managed_window_by_wl_surface(surface.wl_surface()) { // MODIFIED
            let mut managed_win_state_guard = window_arc.state.write().unwrap();
            managed_win_state_guard.title = Some(title.clone());
            drop(managed_win_state_guard);
            tracing::info!("Window {:?} requested title change to: {}", window_arc.id, title);
        }
    }

    fn toplevel_request_set_app_id(&mut self, surface: &ToplevelSurface, app_id: String) {
         if let Some(window_arc) = self.find_managed_window_by_wl_surface(surface.wl_surface()) { // MODIFIED
            let mut managed_win_state_guard = window_arc.state.write().unwrap();
            managed_win_state_guard.app_id = Some(app_id.clone());
            drop(managed_win_state_guard);
            tracing::info!("Window {:?} requested app_id change to: {}", window_arc.id, app_id);
        }
    }

    fn set_parent_request(&mut self, surface: &ToplevelSurface, parent_surface_opt: Option<&ToplevelSurface>) {
        let child_wl_surface = surface.wl_surface();
        match parent_surface_opt {
            // ANCHOR: SetParentLogicStart
            Some(parent_toplevel_surface) => {
                let parent_wl_surface = parent_toplevel_surface.wl_surface();
                tracing::info!(
                    child_surface_id = ?child_wl_surface.id(),
                    parent_surface_id = ?parent_wl_surface.id(),
                    "XDG Toplevel set_parent request received."
                );
                if let Some(child_mw) = self.find_managed_window_by_wl_surface(child_wl_surface) { // MODIFIED
                    if let Some(parent_mw) = self.find_managed_window_by_wl_surface(parent_wl_surface) { // MODIFIED
                        // The ManagedWindow.parent field is Option<Weak<ManagedWindow>>.
                        // To update this without &mut ManagedWindow, it needs interior mutability (e.g. RwLock).
                        // Assuming ManagedWindow.parent is NOT behind RwLock for this example, direct update is not straightforward.
                        // We log the intent. Smithay's ToplevelSurface handles the protocol-level parent link.
                        tracing::info!("ManagedWindow {:?} requested to be child of ManagedWindow {:?}. (Parent field update depends on ManagedWindow internal mutability for `parent`)", child_mw.id, parent_mw.id);
                        // If ManagedWindow.parent were, e.g., parent: Arc<RwLock<Option<Weak<ManagedWindow>>>>,
                        // then: *child_mw.parent.write().unwrap() = Some(Arc::downgrade(&parent_mw));
                    } else {
                        tracing::warn!("Parent ToplevelSurface {:?} not found as ManagedWindow for set_parent request of child {:?}.", parent_wl_surface.id(), child_wl_surface.id());
                    }
                } else {
                     tracing::warn!("Child ToplevelSurface {:?} not found as ManagedWindow for set_parent request.", child_wl_surface.id());
                }
            }
            None => {
                tracing::info!(
                    child_surface_id = ?child_wl_surface.id(),
                    "XDG Toplevel set_parent request received with None (unset parent)."
                );
                if let Some(child_mw) = self.find_managed_window_by_wl_surface(child_wl_surface) { // MODIFIED
                    // Similar to setting parent, unsetting requires interior mutability or &mut.
                    tracing::info!("ManagedWindow {:?} requested to unset parent. (Parent field update depends on ManagedWindow internal mutability for `parent`)", child_mw.id);
                    // If ManagedWindow.parent were Arc<RwLock<Option<Weak<ManagedWindow>>>>,
                    // then: *child_mw.parent.write().unwrap() = None;
                }
            }
            // ANCHOR_END: SetParentLogicEnd
        }
    }

    // ANCHOR: HandleSetMaximized
    fn toplevel_request_set_maximized(&mut self, surface: &ToplevelSurface) {
        if let Some(window_arc) = self.find_managed_window_by_wl_surface(surface.wl_surface()) {
            tracing::info!("Window {:?} requested set_maximized", window_arc.id);
            let mut win_state_guard = window_arc.state.write().unwrap();

            if !win_state_guard.maximized { // Only save if not already maximized
                win_state_guard.saved_pre_action_geometry = Some(window_arc.geometry()); // Reads RwLock on current_geometry
                tracing::debug!("Saved pre-maximize geometry: {:?}", win_state_guard.saved_pre_action_geometry);
            }
            win_state_guard.maximized = true;
            win_state_guard.fullscreen = false; // Cannot be fullscreen and maximized
            win_state_guard.minimized = false; // Cannot be minimized and maximized

            // Determine maximized geometry (e.g., full output)
            // For MVP, using the first output's geometry.
            let maximized_geometry = self.space.outputs()
                .next()
                .and_then(|o| self.space.output_geometry(o))
                .unwrap_or_else(|| {
                    tracing::warn!("No output found for maximized geometry, defaulting to 800x600 at (0,0)");
                    Rectangle::from_loc_and_size((0,0), (800,600))
                });

            // Update ManagedWindow's current geometry (overall window size including SSD)
            *window_arc.current_geometry.write().unwrap() = maximized_geometry;
            // Update WindowState's position and size fields to match overall window size
            win_state_guard.position = maximized_geometry.loc;
            win_state_guard.size = maximized_geometry.size;

            let manager_props = window_arc.manager_data.read().unwrap(); // Read manager data for decoration status
            let is_ssd = manager_props.decorations;
            drop(manager_props); // Release manager_data lock
            drop(win_state_guard); // Release WindowState lock

            // ANCHOR_REF: SSDConfigureSizeMaximized
            // Calculate content size for client configuration if SSD is active
            let configure_size = if is_ssd {
                Size::from((
                    (maximized_geometry.size.w - 2 * types::DEFAULT_BORDER_SIZE).max(1),
                    (maximized_geometry.size.h - types::DEFAULT_TITLE_BAR_HEIGHT - 2 * types::DEFAULT_BORDER_SIZE).max(1)
                ))
            } else {
                maximized_geometry.size
            };

            surface.with_pending_state(|xdg_toplevel_pending_state| {
                xdg_toplevel_pending_state.states.set(XdgToplevelStateSmithay::Maximized);
                xdg_toplevel_pending_state.size = Some(configure_size); // Send content size to client
            });
            let serial = surface.send_configure();
            // TODO: Update ManagedWindow.last_configure_serial if it's made mutable (e.g. Arc<Mutex<Option<Serial>>>)
            // window_arc.last_configure_serial = Some(serial);

            tracing::info!("Window {:?} maximized. Sent configure (serial: {:?}) with content size {:?}. Overall geometry: {:?}",
                         window_arc.id, serial, configure_size, maximized_geometry);
            self.space.map_window(window_arc.clone(), maximized_geometry.loc, false);
            self.space.damage_all_outputs();
        }
    }
    // ANCHOR_END: HandleSetMaximized

    // ANCHOR: HandleUnsetMaximized
    fn toplevel_request_unset_maximized(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG Toplevel unset_maximized request.");
        if let Some(window_arc) = self.find_managed_window_by_wl_surface(surface.wl_surface()) {
            let mut win_state_guard = window_arc.state.write().unwrap();
            if !win_state_guard.maximized {
                tracing::debug!("Window {:?} was not maximized, unset_maximized is a no-op.", window_arc.id);
                return;
            }
            win_state_guard.maximized = false;
            
            let restored_geometry = win_state_guard.saved_pre_action_geometry.take()
                .unwrap_or_else(|| {
                    tracing::warn!("No saved geometry for unmaximize of window {:?}, using default 300x200 at current pos.", window_arc.id);
                    Rectangle::from_loc_and_size(win_state_guard.position, Size::from((300,200)))
                });

            // Update ManagedWindow's current geometry (overall window size)
            *window_arc.current_geometry.write().unwrap() = restored_geometry;
            // Update WindowState's position and size fields
            win_state_guard.position = restored_geometry.loc;
            win_state_guard.size = restored_geometry.size;

            let manager_props = window_arc.manager_data.read().unwrap();
            let is_ssd = manager_props.decorations;
            drop(manager_props);
            drop(win_state_guard);

            // ANCHOR_REF: SSDConfigureSizeUnmaximized
            let configure_size = if is_ssd {
                Size::from((
                    (restored_geometry.size.w - 2 * types::DEFAULT_BORDER_SIZE).max(1),
                    (restored_geometry.size.h - types::DEFAULT_TITLE_BAR_HEIGHT - 2 * types::DEFAULT_BORDER_SIZE).max(1)
                ))
            } else {
                restored_geometry.size
            };

            surface.with_pending_state(|xdg_toplevel_pending_state| {
                xdg_toplevel_pending_state.states.unset(XdgToplevelStateSmithay::Maximized);
                xdg_toplevel_pending_state.size = Some(configure_size); // Send content size
            });
            let serial = surface.send_configure();
            // TODO: Update ManagedWindow.last_configure_serial if mutable
            // window_arc.last_configure_serial = Some(serial);

            tracing::debug!("Window {:?} unmaximized. Sent configure (serial: {:?}) with content size {:?}. Overall geometry: {:?}",
                          window_arc.id, serial, configure_size, restored_geometry);
            self.space.map_window(window_arc.clone(), restored_geometry.loc, false);
            self.space.damage_all_outputs();
        }
    }
    // ANCHOR_END: HandleUnsetMaximized

    // ANCHOR: HandleSetMinimized
    fn toplevel_request_set_minimized(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG Toplevel set_minimized request.");
        if let Some(window_arc) = self.find_managed_window_by_wl_surface(surface.wl_surface()) {
            let mut win_state_guard = window_arc.state.write().unwrap();
            if win_state_guard.minimized {
                tracing::debug!("Window {:?} already minimized.", window_arc.id);
                return;
            }
            win_state_guard.minimized = true;
            // Optionally save pre-minimize state if needed for quick restore, though XDG typically relies on client managing this.
            // win_state_guard.saved_pre_action_geometry = Some(window_arc.geometry());

            // Unmap the window
            if win_state_guard.is_mapped { // Check if it was mapped
                win_state_guard.is_mapped = false;
                win_state_guard.activated = false; // Deactivate on minimize
                self.space.unmap_window(&window_arc);
                tracing::info!("Window {:?} unmapped due to minimization.", window_arc.id);
            }
            drop(win_state_guard); // Release lock

            surface.with_pending_state(|xdg_toplevel_pending_state| {
                xdg_toplevel_pending_state.states.set(XdgToplevelStateSmithay::Minimized);
                // Size is not typically sent for minimized state, client usually handles restoration.
            });
            let serial = surface.send_configure();
            // TODO: Update ManagedWindow.last_configure_serial if mutable
            // window_arc.last_configure_serial = Some(serial);

            tracing::debug!("Window {:?} minimized. Sent configure (serial: {:?}).", window_arc.id, serial);
            self.space.damage_all_outputs(); // Damage where it was, if applicable
        }
    }
    // ANCHOR_END: HandleSetMinimized

    // Note: unset_minimized is not an explicit XDG request. Client remaps or activates.
    // We handle this in map_toplevel by clearing the minimized state.

    // ANCHOR: HandleSetFullscreen
    fn toplevel_request_set_fullscreen(&mut self, surface: &ToplevelSurface, output_opt: Option<&WaylandOutput>) {
        if let Some(window_arc) = self.find_managed_window_by_wl_surface(surface.wl_surface()) {
            tracing::info!("Window {:?} requested set_fullscreen on output {:?}", window_arc.id, output_opt.map(|o| o.name()));
            let mut win_state_guard = window_arc.state.write().unwrap();

            if !win_state_guard.fullscreen { // Only save if not already fullscreen
                 win_state_guard.saved_pre_action_geometry = Some(window_arc.geometry());
                 tracing::debug!("Saved pre-fullscreen geometry for {:?}: {:?}", window_arc.id, win_state_guard.saved_pre_action_geometry);
            }
            win_state_guard.fullscreen = true;
            win_state_guard.maximized = false; // Cannot be maximized and fullscreen
            win_state_guard.minimized = false; // Cannot be minimized and fullscreen

            let target_output = output_opt.or_else(|| self.space.outputs().next()); // Default to first output
            let fullscreen_geometry = target_output
                .and_then(|o| self.space.output_geometry(o))
                .unwrap_or_else(|| {
                    tracing::warn!("No output found for fullscreen geometry, defaulting to 800x600 at (0,0)");
                    Rectangle::from_loc_and_size((0,0), (800,600))
                });

            *window_arc.current_geometry.write().unwrap() = fullscreen_geometry;
            win_state_guard.position = fullscreen_geometry.loc;
            win_state_guard.size = fullscreen_geometry.size;

            let manager_props = window_arc.manager_data.read().unwrap();
            let is_ssd = manager_props.decorations;
            drop(manager_props);
            drop(win_state_guard);

            // ANCHOR_REF: SSDConfigureSizeFullscreen
            let configure_size = if is_ssd {
                // For fullscreen with SSD, client might still get full size, and compositor draws over.
                // Or, client gets content size. Let's give client full size for true fullscreen.
                // Some clients might draw their own decorations if they get a smaller size in fullscreen.
                // For true fullscreen, client should ideally not have decorations.
                // If SSD is forced, then content area is indeed smaller.
                // This depends on compositor policy. For now, assume client gets full size.
                // If SSD were to remain and carve out space:
                // Size::from((
                //    (fullscreen_geometry.size.w - 2 * types::DEFAULT_BORDER_SIZE).max(1),
                //    (fullscreen_geometry.size.h - types::DEFAULT_TITLE_BAR_HEIGHT - 2 * types::DEFAULT_BORDER_SIZE).max(1)
                // ))
                fullscreen_geometry.size
            } else {
                fullscreen_geometry.size
            };

            surface.with_pending_state(|xdg_toplevel_pending_state| {
                xdg_toplevel_pending_state.states.set(XdgToplevelStateSmithay::Fullscreen);
                xdg_toplevel_pending_state.size = Some(configure_size);
            });
            let serial = surface.send_configure();
            // TODO: Update ManagedWindow.last_configure_serial if mutable
            // window_arc.last_configure_serial = Some(serial);

            tracing::info!("Window {:?} set_fullscreen. Sent configure (serial: {:?}) with content size {:?}. Overall geometry: {:?}",
                         window_arc.id, serial, configure_size, fullscreen_geometry);
            self.space.map_window(window_arc.clone(), fullscreen_geometry.loc, true);
            self.space.damage_all_outputs();
        }
    }
    // ANCHOR_END: HandleSetFullscreen

    // ANCHOR: HandleUnsetFullscreen
    fn toplevel_request_unset_fullscreen(&mut self, surface: &ToplevelSurface) {
         if let Some(window_arc) = self.find_managed_window_by_wl_surface(surface.wl_surface()) {
            tracing::info!("Window {:?} requested unset_fullscreen", window_arc.id);
            let mut win_state_guard = window_arc.state.write().unwrap();
            if !win_state_guard.fullscreen {
                tracing::debug!("Window {:?} was not fullscreen, unset_fullscreen is a no-op.", window_arc.id);
                return;
            }
            win_state_guard.fullscreen = false;

            let restored_geometry = win_state_guard.saved_pre_action_geometry.take()
                .unwrap_or_else(|| {
                    tracing::warn!("No saved geometry for unset_fullscreen of window {:?}, using default 300x200 at current pos.", window_arc.id);
                    Rectangle::from_loc_and_size(win_state_guard.position, Size::from((300,200)))
                });

            *window_arc.current_geometry.write().unwrap() = restored_geometry;
            win_state_guard.position = restored_geometry.loc;
            win_state_guard.size = restored_geometry.size;

            let manager_props = window_arc.manager_data.read().unwrap();
            let is_ssd = manager_props.decorations;
            drop(manager_props);
            drop(win_state_guard);

            // ANCHOR_REF: SSDConfigureSizeUnfullscreen
            let configure_size = if is_ssd {
                Size::from((
                    (restored_geometry.size.w - 2 * types::DEFAULT_BORDER_SIZE).max(1),
                    (restored_geometry.size.h - types::DEFAULT_TITLE_BAR_HEIGHT - 2 * types::DEFAULT_BORDER_SIZE).max(1)
                ))
            } else {
                restored_geometry.size
            };

            surface.with_pending_state(|xdg_toplevel_pending_state| {
                xdg_toplevel_pending_state.states.unset(XdgToplevelStateSmithay::Fullscreen);
                xdg_toplevel_pending_state.size = Some(configure_size); // Send content size
            });
            let serial = surface.send_configure();
            // TODO: Update ManagedWindow.last_configure_serial if mutable
            // window_arc.last_configure_serial = Some(serial);

            tracing::info!("Window {:?} unset_fullscreen. Sent configure (serial: {:?}) with content size {:?}. Overall geometry: {:?}",
                         window_arc.id, serial, configure_size, restored_geometry);
            self.space.map_window(window_arc.clone(), restored_geometry.loc, false);
            self.space.damage_all_outputs();
        }
    }
    // ANCHOR_END: HandleUnsetFullscreen

    // ANCHOR: ToplevelDestroyedHandler
    fn toplevel_destroyed(&mut self, toplevel_surface: ToplevelSurface) {
        let xdg_surface_obj = toplevel_surface.xdg_surface();
        let wl_surface = xdg_surface_obj.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "XDG Toplevel destroyed by client");

        if let Some(user_data_arc) = xdg_surface_obj.user_data().get::<Arc<XdgSurfaceUserData>>() {
            let mut state_guard = user_data_arc.state.lock().unwrap();
            *state_guard = XdgSurfaceState::Destroyed;
            tracing::info!(surface_id = ?wl_surface.id(), "XdgSurfaceUserData state set to Destroyed.");
        } else {
            // This is unexpected if the surface was properly initialized.
            tracing::warn!(surface_id = ?wl_surface.id(), "XdgSurfaceUserData not found on toplevel destruction. Cannot update its state.");
        }

        if let Some(window_arc) = self.find_managed_window_by_wl_surface(wl_surface) { // MODIFIED (using self.method)
            // ANCHOR: RemoveWindowFromWorkspaceOnDestroy
            let mut destroyed_window_output_name: Option<String> = None;
            if let Some(workspace_id) = *window_arc.workspace_id.read().unwrap() {
                // Find which output this workspace belongs to by checking output_name in CompositorWorkspace
                for (output_name_key, workspaces_on_output) in &self.output_workspaces {
                    if let Some(workspace_arc) = workspaces_on_output.iter().find(|ws_arc| ws_arc.read().unwrap().id == workspace_id) {
                        workspace_arc.read().unwrap().remove_window(&window_arc.domain_id);
                        destroyed_window_output_name = Some(output_name_key.clone());
                        tracing::info!("Removed window {:?} (domain_id: {:?}) from workspace {:?} ({}) on output {}.",
                            window_arc.id, window_arc.domain_id, workspace_id, workspace_arc.read().unwrap().name, output_name_key);
                        break;
                    }
                }
                if destroyed_window_output_name.is_none() {
                     tracing::warn!("Workspace ID {:?} for destroyed window {:?} not found in any output's workspace list.", workspace_id, window_arc.id);
                }
            } else {
                tracing::warn!("Destroyed window {:?} had no workspace ID assigned.", window_arc.id);
            }
            // ANCHOR_END: RemoveWindowFromWorkspaceOnDestroy

            self.space.unmap_window(&window_arc);
            self.windows.remove(&window_arc.domain_id());
            tracing::info!("ManagedWindow {:?} (domain: {:?}) removed due to toplevel destruction.", window_arc.id, window_arc.domain_id());

            // ANCHOR: ApplyTilingOnDestroyRefactored
            // After removing a window, if its output had an active workspace that was tiled, re-apply layout for that output.
            if let Some(output_name) = destroyed_window_output_name {
                 if let Some(active_ws_id_on_output) = self.active_workspaces.read().unwrap().get(&output_name) {
                     if Some(*active_ws_id_on_output) == *window_arc.workspace_id.read().unwrap() { // Check if the workspace of the destroyed window is still the active one on that output
                        if let Some(workspaces_on_output) = self.output_workspaces.get(&output_name){
                            if let Some(active_ws_arc) = workspaces_on_output.iter().find(|ws| ws.read().unwrap().id == *active_ws_id_on_output) {
                                if *active_ws_arc.read().unwrap().tiling_layout.read().unwrap() != crate::compositor::workspaces::TilingLayout::None {
                                     crate::compositor::tiling::apply_layout_for_output(self, &output_name);
                                }
                            }
                        }
                     }
                 }
            }
            // ANCHOR_END: ApplyTilingOnDestroy
            self.space.damage_all_outputs();
        } else {
             tracing::warn!("Destroyed toplevel {:?} was not found in self.windows by its WlSurface.", wl_surface.id());
        }
    }
    // ANCHOR_END: ToplevelDestroyedHandler

    // ANCHOR: PopupDestroyedHandler
    fn popup_destroyed(&mut self, popup_surface: PopupSurface) {
        let xdg_surface_obj = popup_surface.xdg_surface();
        let wl_surface = xdg_surface_obj.wl_surface();
        tracing::info!(surface_id = ?wl_surface.id(), "XDG Popup destroyed by client");

        if let Some(user_data_arc) = xdg_surface_obj.user_data().get::<Arc<XdgSurfaceUserData>>() {
            let mut state_guard = user_data_arc.state.lock().unwrap();
            *state_guard = XdgSurfaceState::Destroyed;
            tracing::info!(surface_id = ?wl_surface.id(), "XdgSurfaceUserData state set to Destroyed.");
        } else {
            tracing::warn!(surface_id = ?wl_surface.id(), "XdgSurfaceUserData not found on popup destruction. Cannot update its state.");
        }

        if let Some(popup_arc) = find_managed_window_by_wl_surface(self, wl_surface) {
            self.windows.remove(&popup_arc.domain_id());
            tracing::info!("ManagedWindow (popup) {:?} (domain: {:?}) removed from self.windows due to popup destruction.", popup_arc.id, popup_arc.domain_id());
        }
        self.space.damage_all_outputs();
    }
    // ANCHOR_END: PopupDestroyedHandler


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
    // ANCHOR: RepositionPopupHandlerFinalImpl
    fn reposition_popup(&mut self, popup_surface: &PopupSurface, positioner: &PositionerState, _token: u32) {
        let wl_surface = popup_surface.wl_surface();
        tracing::info!("Reposition popup request for {:?} (token: {})", wl_surface.id(), _token);

        let managed_popup_arc = match self.find_managed_window_by_wl_surface(wl_surface) {
            Some(arc) => arc,
            None => {
                tracing::warn!("reposition_popup: ManagedWindow not found for popup surface {:?}", wl_surface.id());
                return;
            }
        };

        // Ensure the popup's XdgSurfaceUserData is still valid
        let xdg_surface_obj = popup_surface.xdg_surface();
        if let Some(user_data_arc) = xdg_surface_obj.user_data().get::<Arc<XdgSurfaceUserData>>() {
            let state_guard = user_data_arc.state.lock().unwrap();
            if *state_guard == XdgSurfaceState::Destroyed {
                tracing::warn!(surface_id = ?wl_surface.id(), "reposition_popup called on a destroyed XDG surface. Ignoring.");
                return;
            }
        } else {
            tracing::error!(surface_id = ?wl_surface.id(), "XdgSurfaceUserData not found on popup reposition. This is a bug.");
            return;
        }

        let parent_geometry = match managed_popup_arc.parent.as_ref().and_then(|weak_parent| weak_parent.upgrade()) {
            Some(parent_mw) => parent_mw.geometry(),
            None => {
                tracing::warn!("reposition_popup: Parent ManagedWindow not found or Weak link expired for popup {:?}", wl_surface.id());
                popup_surface.send_popup_done(); // Dismiss popup if parent is gone
                return;
            }
        };

        let new_popup_geometry = smithay::wayland::shell::xdg::calculate_popup_geometry(
            positioner,
            parent_geometry,
            wl_surface,
        );

        *managed_popup_arc.current_geometry.write().unwrap() = new_popup_geometry;
        tracing::info!("Popup {:?} new geometry after reposition: {:?}", wl_surface.id(), new_popup_geometry);

        let configure_serial = popup_surface.send_configure();

        // The ManagedWindow's last_configure_serial is Option<Serial>, not locked.
        // To update it, we would need &mut ManagedWindow or interior mutability for this field.
        // This was noted as a structural point. For now, we log and rely on client behavior for serial usage.
        tracing::info!(
            "Sent configure for popup {:?} (serial: {:?}) due to reposition request. New calculated geometry: {:?}",
            wl_surface.id(), configure_serial, new_popup_geometry
        );

        self.space.damage_all_outputs();
    }
    // ANCHOR_END: RepositionPopupHandlerFinalImpl

    // ANCHOR: WorkspaceAssignmentAndRemovalTests
    #[test]
    fn test_map_toplevel_assigns_to_active_workspace_and_applies_tiling() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);

        let active_ws_id = *state.active_compositor_workspace_id.read().unwrap();
        let active_ws_arc = state.compositor_workspaces.iter()
            .find(|ws| ws.read().unwrap().id == active_ws_id).unwrap().clone();

        // Set active workspace to tiling
        *active_ws_arc.write().unwrap().tiling_layout.write().unwrap() = crate::compositor::workspaces::TilingLayout::MasterStack;

        state.new_toplevel(toplevel_surface.clone()); // Create ManagedWindow
        let managed_window_arc = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();

        // Ensure it's not yet assigned a workspace by new_toplevel
        assert!(managed_window_arc.workspace_id.read().unwrap().is_none());

        state.map_toplevel(&toplevel_surface); // This should assign and apply tiling

        assert_eq!(*managed_window_arc.workspace_id.read().unwrap(), Some(active_ws_id));
        assert!(active_ws_arc.read().unwrap().contains_window(&managed_window_arc.domain_id));

        // Check if geometry reflects tiling (for a single window, it's the full workspace area)
        let output_geom = state.space.outputs().next().and_then(|o| state.space.output_geometry(o))
                            .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0),(800,600)));
        assert_eq!(*managed_window_arc.current_geometry.read().unwrap(), output_geom, "Single window in tiled layout should take full workspace area.");
    }

    #[test]
    fn test_map_toplevel_assigns_to_active_workspace() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);

        // Pre-condition: No ManagedWindow for this surface yet.
        assert!(state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).is_none());

        // Get active workspace ID *before* map_toplevel, as new_toplevel (called by map_toplevel if window is new)
        // might be the one doing the assignment in some flows, though our current target is map_toplevel.
        // Based on current plan, new_toplevel creates ManagedWindow, map_toplevel assigns workspace.
        let active_ws_id = *state.active_compositor_workspace_id.read().unwrap();

        // Call new_toplevel first to create the ManagedWindow, as map_toplevel expects it.
        state.new_toplevel(toplevel_surface.clone());
        let managed_window_arc = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface())
            .expect("ManagedWindow should exist after new_toplevel");

        // Ensure it's not yet assigned a workspace by new_toplevel
        assert!(managed_window_arc.workspace_id.read().unwrap().is_none());

        // Now call map_toplevel
        state.map_toplevel(&toplevel_surface);

        assert_eq!(*managed_window_arc.workspace_id.read().unwrap(), Some(active_ws_id), "Window should be assigned to the active workspace ID.");

        let active_workspace = state.compositor_workspaces.iter()
            .find(|ws_arc| ws_arc.read().unwrap().id == active_ws_id)
            .expect("Active workspace not found in state.compositor_workspaces");

        assert!(active_workspace.read().unwrap().contains_window(&managed_window_arc.domain_id), "Active workspace should contain the new window's domain_id.");
    }

    #[test]
    fn test_toplevel_destroyed_removes_from_workspace_and_applies_tiling() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);

        let active_ws_id = *state.active_compositor_workspace_id.read().unwrap();
        let active_ws_arc = state.compositor_workspaces.iter()
            .find(|ws| ws.read().unwrap().id == active_ws_id).unwrap().clone();
        *active_ws_arc.write().unwrap().tiling_layout.write().unwrap() = crate::compositor::workspaces::TilingLayout::MasterStack;

        // Add two windows
        let ts1 = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(ts1.clone());
        let mw1 = state.find_managed_window_by_wl_surface(ts1.wl_surface()).unwrap();
        *mw1.workspace_id.write().unwrap() = Some(active_ws_id);
        active_ws_arc.read().unwrap().add_window(mw1.domain_id);
        state.map_toplevel(&ts1); // Maps and applies initial tiling (mw1 takes full space)

        let ts2 = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(ts2.clone());
        let mw2 = state.find_managed_window_by_wl_surface(ts2.wl_surface()).unwrap();
        *mw2.workspace_id.write().unwrap() = Some(active_ws_id);
        active_ws_arc.read().unwrap().add_window(mw2.domain_id);
        state.map_toplevel(&ts2); // Maps and applies tiling (mw1 master, mw2 stack)

        let mw1_domain_id = mw1.domain_id; // Save for later check
        let mw2_domain_id = mw2.domain_id;

        assert_eq!(active_ws_arc.read().unwrap().windows.read().unwrap().len(), 2);
        // At this point, mw1 is master, mw2 is stack. (Geometries would reflect this)

        // Action: Destroy mw1 (the master)
        state.toplevel_destroyed(ts1.clone());

        assert!(state.find_managed_window_by_wl_surface(ts1.wl_surface()).is_none());
        assert!(!active_ws_arc.read().unwrap().contains_window(&mw1_domain_id));
        assert_eq!(active_ws_arc.read().unwrap().windows.read().unwrap().len(), 1, "Only mw2 should remain in workspace");

        // Verify mw2 (the remaining window) is re-tiled to take full space
        let output_geom = state.space.outputs().next().and_then(|o| state.space.output_geometry(o))
                            .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0),(800,600)));
        assert_eq!(*mw2.current_geometry.read().unwrap(), output_geom, "Remaining window mw2 should now take full workspace area.");
    }
    // ANCHOR_END: WorkspaceAssignmentAndRemovalTests

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
    use crate::compositor::core::state::DesktopState;
    use crate::compositor::shell::xdg_shell::types::{ManagedWindow, WindowState, DomainWindowIdentifier, WindowLayer};
    use smithay::reexports::wayland_server::{
        Display, DisplayHandle, Client, protocol::wl_surface::WlSurface, globals::GlobalData,
        UserData, backend::{ClientId, GlobalId, Handle, ObjectData, ObjectId, DisconnectReason},
        Interface, Message, Main,
    };
    use smithay::reexports::calloop::EventLoop;
    use smithay::utils::{Point, Size, Rectangle};
    use smithay::wayland::shell::xdg::{
        ToplevelSurface, XdgShellHandler, XdgToplevelState, WindowSurface, XdgShellState, XdgActivationState, XDG_WM_BASE_VERSION, SmithayXdgSurface
    };
    use std::sync::Arc;

    // Minimal test client data
    #[derive(Default, Clone)]
    struct TestClientData { user_data: UserData }
    impl ClientData for TestClientData {
        fn initialized(&self, _client_id: ClientId) {}
        fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
        fn data_map(&self) -> &UserData { &self.user_data }
    }

    // Mock ObjectData for WlSurface
    #[derive(Default)]
    struct TestObjectData;
    impl<R: Interface + AsRef<Resource<R>> + Unpin + 'static> ObjectData<R> for TestObjectData {
        fn request(self: Arc<Self>, _handle: &Handle, _client_data: &mut dyn ClientData, _client_id: ClientId, _msg: Message<R>) -> Option<Arc<dyn ObjectData<R>>> { None }
        fn destroyed(self: Arc<Self>, _client_id: ClientId, _object_id: ObjectId) {}
    }

    // Helper to create a minimal DesktopState for testing
    fn create_minimal_test_state() -> DesktopState {
        let mut event_loop: EventLoop<'static, DesktopState> = EventLoop::try_new().unwrap();
        let display_handle = event_loop.handle().insert_source(
            Display::<DesktopState>::new().unwrap(),
            |_, _, _| {},
        ).unwrap();

        // DesktopState::new now returns Result, unwrap for test simplicity
        DesktopState::new(event_loop.handle(), display_handle.clone()).expect("Failed to create test DesktopState")
    }

    // Helper to create a Client associated with the test DesktopState's display
    fn create_test_client(state: &mut DesktopState) -> Client {
        state.display_handle.create_client(TestClientData::default().into())
    }

    // Helper to create a mock ToplevelSurface
    // Needs DesktopState to access its XdgShellState and DisplayHandle
    fn mock_toplevel_surface(state: &mut DesktopState, client: &Client) -> ToplevelSurface {
        let dh = state.display_handle.clone();
        let wl_surface_main = client.create_object::<WlSurface, _>(&dh, WlSurface::interface().version, Arc::new(TestObjectData)).unwrap();
        let wl_surface = wl_surface_main.as_ref();

        // Ensure WlSurface has Smithay's SurfaceData and XdgSurfaceData
        wl_surface.data_map().insert_if_missing_threadsafe(|| Arc::new(smithay::wayland::compositor::SurfaceData::new(None, Rectangle::from_loc_and_size((0,0),(0,0)))));
        wl_surface.data_map().insert_if_missing_threadsafe(|| smithay::wayland::shell::xdg::XdgSurfaceData::new());


        // Create Smithay's XdgSurface (wrapper) and attach our XdgSurfaceUserData
        let smithay_xdg_surface = SmithayXdgSurface::new_unassigned(wl_surface.clone());
        let xdg_user_data_for_surface = Arc::new(XdgSurfaceUserData::new(wl_surface.clone()));
        smithay_xdg_surface.user_data().insert_if_missing_threadsafe(|| xdg_user_data_for_surface.clone());

        // Create the ToplevelSurface role object
        let toplevel = ToplevelSurface::from_xdg_surface(smithay_xdg_surface, Default::default()).unwrap();
        toplevel
    }

    #[test]
    fn test_toplevel_set_title() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);

        // Manually call new_toplevel to create the ManagedWindow
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).expect("ManagedWindow not found after new_toplevel");

        let test_title = "Hello NovaDE".to_string();
        state.toplevel_request_set_title(&toplevel_surface, test_title.clone());

        let win_state = managed_window.state.read().unwrap();
        assert_eq!(win_state.title, Some(test_title));
    }

    #[test]
    fn test_toplevel_set_app_id() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).expect("ManagedWindow not found");

        let test_app_id = "org.novade.test_app".to_string();
        state.toplevel_request_set_app_id(&toplevel_surface, test_app_id.clone());

        let win_state = managed_window.state.read().unwrap();
        assert_eq!(win_state.app_id, Some(test_app_id));
    }

    #[test]
    fn test_map_unmap_toplevel_activation() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).expect("ManagedWindow not found");

        // Initial state
        assert_eq!(managed_window.state.read().unwrap().activated, false);

        // Map
        state.map_toplevel(&toplevel_surface);
        assert_eq!(managed_window.state.read().unwrap().activated, true, "Window should be activated on map");

        // Unmap
        state.unmap_toplevel(&toplevel_surface);
        assert_eq!(managed_window.state.read().unwrap().activated, false, "Window should be deactivated on unmap");
    }

    #[test]
    fn test_toplevel_set_parent() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);

        let child_ts = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(child_ts.clone());
        let child_mw = state.find_managed_window_by_wl_surface(child_ts.wl_surface()).unwrap();

        let parent_ts = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(parent_ts.clone());
        let _parent_mw = state.find_managed_window_by_wl_surface(parent_ts.wl_surface()).unwrap();

        // Test setting a parent
        // As direct update to ManagedWindow.parent is complex and not done in current handler,
        // this test primarily ensures the handler logic runs without panic and Smithay handles parent linking.
        // We can't easily assert child_mw.parent here without more ManagedWindow methods or interior mutability.
        state.set_parent_request(&child_ts, Some(&parent_ts));
        // TODO: If ManagedWindow.parent becomes RwLock<Option<Weak<...>>>, assert actual linkage.
        // For now, we trust Smithay and our logging in the handler.

        // Test unsetting parent
        state.set_parent_request(&child_ts, None);
        // TODO: Similar assertion if parent field is made updatable and verifiable.
    }

    // ANCHOR: XdgDecorationHandlerTests
    // --- Tests for XdgDecorationHandler ---
    #[test]
    fn test_decoration_request_mode_sets_ssd_flag() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        // new_toplevel calls new_decoration internally via Smithay dispatch if XdgDecorationManager global exists
        // For this test, we are calling the handler methods directly.
        // We need to ensure a ManagedWindow exists first.
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();

        // Simulate XdgToplevelDecoration object creation for this toplevel,
        // which usually happens when client binds to xdg_decoration_manager and gets a decoration object.
        // Our new_decoration handler would then be called.
        // Here, we'll manually call new_decoration to ensure manager_data.decorations is set to true initially.
        state.new_decoration(toplevel_surface.clone());
        assert_eq!(managed_window.manager_data.read().unwrap().decorations, true, "Initial decoration mode should be SSD after new_decoration");

        // Client requests ClientSideDecorations (CSD)
        state.request_mode(toplevel_surface.clone(), XdgDecorationMode::ClientSide);
        assert_eq!(managed_window.manager_data.read().unwrap().decorations, false, "Decorations flag should be false for CSD");

        // Client requests ServerSideDecorations (SSD)
        state.request_mode(toplevel_surface.clone(), XdgDecorationMode::ServerSide);
        assert_eq!(managed_window.manager_data.read().unwrap().decorations, true, "Decorations flag should be true for SSD");
    }

    #[test]
    fn test_decoration_unset_mode_reverts_to_ssd_flag() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();

        // Simulate new_decoration being called first.
        state.new_decoration(toplevel_surface.clone());

        // Set to CSD first
        state.request_mode(toplevel_surface.clone(), XdgDecorationMode::ClientSide);
        assert_eq!(managed_window.manager_data.read().unwrap().decorations, false, "Decorations flag should be false for CSD before unset");

        // Client requests unset_mode
        state.unset_mode(toplevel_surface.clone());
        assert_eq!(managed_window.manager_data.read().unwrap().decorations, true, "Decorations flag should revert to true (SSD) on unset_mode");
    }
    // ANCHOR_END: XdgDecorationHandlerTests

    // ANCHOR: ToplevelStateChangeTestsWithSSD
    // --- Tests for Toplevel State Changes (Maximized, Minimized, Fullscreen) considering SSD ---

    #[test]
    fn test_toplevel_set_maximized_with_ssd() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();

        // Enable SSD for this window
        managed_window.manager_data.write().unwrap().decorations = true;
        let initial_geom = Rectangle::from_loc_and_size((100,100), (200,150));
        *managed_window.current_geometry.write().unwrap() = initial_geom;
        managed_window.state.write().unwrap().size = initial_geom.size;
        managed_window.state.write().unwrap().position = initial_geom.loc;


        state.toplevel_request_set_maximized(&toplevel_surface);

        let win_state = managed_window.state.read().unwrap();
        assert!(win_state.maximized);
        assert!(!win_state.fullscreen);
        assert!(!win_state.minimized);

        let maximized_overall_geom = state.space.outputs().next().and_then(|o| state.space.output_geometry(o))
                                     .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0),(800,600)));
        assert_eq!(*managed_window.current_geometry.read().unwrap(), maximized_overall_geom);
        assert_eq!(win_state.size, maximized_overall_geom.size); // Check WindowState also updated
        assert_eq!(win_state.position, maximized_overall_geom.loc);

        // ANCHOR: TestNoteSSDContentSizeVerification
        // Note: The actual size sent to the client (configure_size) is logged by the handler.
        // It should be `maximized_overall_geom.size` minus SSD constants.
        // Manually verify logs for: "Sent configure (serial: ...) with content size {:?}"
        // Example expected content size: (800 - 2*5, 600 - 30 - 2*5) = (790, 560) if output is 800x600
        // ANCHOR_END: TestNoteSSDContentSizeVerification
    }

    #[test]
    fn test_toplevel_unset_maximized_with_ssd() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();

        managed_window.manager_data.write().unwrap().decorations = true;
        let original_geom = Rectangle::from_loc_and_size((50,50), (400,300));

        // Simulate being maximized: set current state and save original_geom
        let mut win_state_write = managed_window.state.write().unwrap();
        win_state_write.saved_pre_action_geometry = Some(original_geom);
        win_state_write.maximized = true;
        let maximized_output_geom = state.space.outputs().next().and_then(|o| state.space.output_geometry(o)).unwrap_or_default();
        *managed_window.current_geometry.write().unwrap() = maximized_output_geom;
        win_state_write.size = maximized_output_geom.size; // Update WindowState to reflect maximized size
        win_state_write.position = maximized_output_geom.loc;
        drop(win_state_write);


        state.toplevel_request_unset_maximized(&toplevel_surface);

        let win_state_read = managed_window.state.read().unwrap();
        assert!(!win_state_read.maximized);
        assert_eq!(*managed_window.current_geometry.read().unwrap(), original_geom);
        assert_eq!(win_state_read.size, original_geom.size); // Check WindowState restored
        assert_eq!(win_state_read.position, original_geom.loc);
        // ANCHOR_REF: TestNoteSSDContentSizeVerification (for unmaximized)
        // Note: The actual size sent to the client is logged by the handler.
        // It should be `original_geom.size` minus SSD constants.
    }

    #[test]
    fn test_toplevel_set_minimized() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();

        // Ensure it's mapped first
        managed_window.state.write().unwrap().is_mapped = true;

        state.toplevel_request_set_minimized(&toplevel_surface);

        let win_state = managed_window.state.read().unwrap();
        assert!(win_state.minimized);
        assert!(!win_state.is_mapped, "Window should be unmapped when minimized");
        assert!(!win_state.activated, "Window should be deactivated when minimized");
    }

    #[test]
    fn test_map_clears_minimized_state() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();

        managed_window.state.write().unwrap().minimized = true; // Simulate minimized state
        managed_window.state.write().unwrap().is_mapped = false; // Should be unmapped if minimized

        state.map_toplevel(&toplevel_surface); // This should unminimize

        let win_state = managed_window.state.read().unwrap();
        assert!(!win_state.minimized, "Minimized state should be cleared on map");
        assert!(win_state.is_mapped, "Window should be mapped");
        assert!(win_state.activated, "Window should be activated on map");
    }


    #[test]
    fn test_toplevel_set_fullscreen_with_ssd() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();

        managed_window.manager_data.write().unwrap().decorations = true;
        let initial_geom = Rectangle::from_loc_and_size((100,100), (200,150));
        *managed_window.current_geometry.write().unwrap() = initial_geom;
        managed_window.state.write().unwrap().size = initial_geom.size;
        managed_window.state.write().unwrap().position = initial_geom.loc;


        state.toplevel_request_set_fullscreen(&toplevel_surface, None); // Fullscreen on default output

        let win_state = managed_window.state.read().unwrap();
        assert!(win_state.fullscreen);
        assert!(!win_state.maximized);

        let fullscreen_overall_geom = state.space.outputs().next().and_then(|o| state.space.output_geometry(o))
                                     .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0),(800,600)));
        assert_eq!(*managed_window.current_geometry.read().unwrap(), fullscreen_overall_geom);
        assert_eq!(win_state.size, fullscreen_overall_geom.size); // Check WindowState also updated
        assert_eq!(win_state.position, fullscreen_overall_geom.loc);
        // ANCHOR_REF: TestNoteSSDContentSizeVerification (for fullscreen)
        // Note: The actual size sent to the client is logged by the handler.
        // Current logic for fullscreen sends the full output size even with SSD.
    }

    #[test]
    fn test_toplevel_unset_fullscreen_with_ssd() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(toplevel_surface.clone());
        let managed_window = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();

        managed_window.manager_data.write().unwrap().decorations = true;
        let original_geom = Rectangle::from_loc_and_size((50,50), (400,300));

        let mut win_state_write = managed_window.state.write().unwrap();
        win_state_write.saved_pre_action_geometry = Some(original_geom);
        win_state_write.fullscreen = true;
        let fullscreen_output_geom = state.space.outputs().next().and_then(|o| state.space.output_geometry(o)).unwrap_or_default();
        *managed_window.current_geometry.write().unwrap() = fullscreen_output_geom;
        win_state_write.size = fullscreen_output_geom.size; // Update WindowState to reflect fullscreen size
        win_state_write.position = fullscreen_output_geom.loc;
        drop(win_state_write);


        state.toplevel_request_unset_fullscreen(&toplevel_surface);

        let win_state_read = managed_window.state.read().unwrap();
        assert!(!win_state_read.fullscreen);
        assert_eq!(*managed_window.current_geometry.read().unwrap(), original_geom);
        assert_eq!(win_state_read.size, original_geom.size); // Check WindowState restored
        assert_eq!(win_state_read.position, original_geom.loc);
        // ANCHOR_REF: TestNoteSSDContentSizeVerification (for unfullscreen)
        // Note: The actual size sent to the client is logged by the handler.
        // It should be `original_geom.size` minus SSD constants.
    }
    // ANCHOR_END: ToplevelStateChangeTestsWithSSD

    // --- Tests for xdg_popup ---

    // Helper to create a mock PopupSurface
    // Needs a parent ToplevelSurface for context
    fn mock_popup_surface(state: &mut DesktopState, client: &Client, parent_toplevel: &ToplevelSurface) -> PopupSurface {
        let dh = state.display_handle.clone();
        let wl_surface_main = client.create_object::<WlSurface, _>(&dh, WlSurface::interface().version, Arc::new(TestObjectData)).unwrap();
        let wl_surface = wl_surface_main.as_ref();

        // Ensure WlSurface has Smithay's SurfaceData and XdgSurfaceData
        wl_surface.data_map().insert_if_missing_threadsafe(|| Arc::new(smithay::wayland::compositor::SurfaceData::new(None, Rectangle::from_loc_and_size((0,0),(0,0)))));
        wl_surface.data_map().insert_if_missing_threadsafe(|| smithay::wayland::shell::xdg::XdgSurfaceData::new());

        let smithay_xdg_surface = SmithayXdgSurface::new_popup(wl_surface.clone(), parent_toplevel.wl_surface().clone());
        let xdg_user_data_for_popup = Arc::new(XdgSurfaceUserData::new(wl_surface.clone()));
        smithay_xdg_surface.user_data().insert_if_missing_threadsafe(|| xdg_user_data_for_popup.clone());

        let popup = PopupSurface::from_xdg_surface(smithay_xdg_surface, Default::default()).unwrap();

        // Mock a basic positioner for the popup
        let positioner_data = smithay::wayland::shell::xdg::PositionerState {
            rect_size: Some(Size::from((100, 50))),
            anchor_rect: Some(Rectangle::from_loc_and_size((10, 10), (1, 1))), // Anchor to a point on parent
            anchor_edges: Default::default(), // xdg_positioner::Anchor::TopLeft,
            gravity: Default::default(), // xdg_positioner::Gravity::TopLeft,
            constraint_adjustment: Default::default(), // xdg_positioner::ConstraintAdjustment::empty(),
            offset: Point::from((5, 5)),
            reactive: false, // Not directly used by calculate_popup_geometry but part of state
            parent_size: Some(parent_toplevel.current_state().size.unwrap_or_default()), // Used by some constraints
            parent_configure_serial: parent_toplevel.current_configure_serial(), // Used by some constraints
        };
        // This is tricky: PopupSurface itself holds the PositionerState.
        // We can't easily inject a mock PositionerState into an existing PopupSurface post-creation
        // without Smithay internal access or a more complex setup involving client requests.
        // For testing calculate_popup_geometry, we'd call it directly with a mock PositionerState.
        // For testing new_popup, it uses the positioner already associated with the PopupSurface
        // by the (mocked) client's prior requests.
        // The `popup.get_positioner()` will reflect what Smithay's XdgSurface has from client.
        // For this test, we assume the client has set up a positioner that new_popup will use.
        // We can't easily *control* that positioner state from here in a simple unit test of new_popup.
        // So, we test that new_popup runs and creates a ManagedWindow.
        // The actual geometry calculation is implicitly tested via calculate_popup_geometry which is a Smithay utility.

        popup
    }

    #[test]
    fn test_new_popup_creates_managed_window() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let parent_toplevel = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(parent_toplevel.clone()); // Ensure parent ManagedWindow exists

        let popup_surface = mock_popup_surface(&mut state, &client, &parent_toplevel);

        // Call new_popup
        let client_data = XdgWmBaseClientData::new(XDG_WM_BASE_VERSION); // Dummy client data
        state.new_popup(popup_surface.clone(), &client_data);

        let managed_popup = state.find_managed_window_by_wl_surface(popup_surface.wl_surface());
        assert!(managed_popup.is_some(), "ManagedWindow for popup should be created and stored.");

        if let Some(mp) = managed_popup {
            assert!(matches!(mp.xdg_surface, WindowSurface::Popup(_)));
            assert!(mp.parent.is_some(), "Popup's ManagedWindow should have a parent link.");
            if let Some(parent_weak) = &mp.parent {
                assert!(parent_weak.upgrade().is_some(), "Popup's parent Weak link should be valid.");
                assert_eq!(parent_weak.upgrade().unwrap().xdg_surface.wl_surface().unwrap().id(), parent_toplevel.wl_surface().id());
            }
            // Geometry check is tricky as default positioner might result in (0,0) size if not constrained.
            // The main thing is that it ran and created the window.
            // Default mock_popup_surface doesn't have a fully configured positioner via client calls.
        }
    }

    #[test]
    fn test_reposition_popup_updates_geometry() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let parent_toplevel = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(parent_toplevel.clone());
        let parent_mw = state.find_managed_window_by_wl_surface(parent_toplevel.wl_surface()).unwrap();
        // Set a known geometry for the parent for predictable calculations
        let parent_initial_geom = Rectangle::from_loc_and_size((100, 100), (300, 200));
        *parent_mw.current_geometry.write().unwrap() = parent_initial_geom;


        let popup_surface = mock_popup_surface(&mut state, &client, &parent_toplevel);
        let client_data = XdgWmBaseClientData::new(XDG_WM_BASE_VERSION);
        state.new_popup(popup_surface.clone(), &client_data);
        let managed_popup = state.find_managed_window_by_wl_surface(popup_surface.wl_surface()).unwrap();
        let initial_popup_geom = *managed_popup.current_geometry.read().unwrap();

        // Create a new PositionerState for repositioning
        let new_positioner_state = PositionerState {
            rect_size: Some(Size::from((120, 60))), // Different size
            anchor_rect: Some(Rectangle::from_loc_and_size((20, 20), (1, 1))), // Different anchor
            offset: Point::from((10, 10)), // Different offset
            anchor_edges: Default::default(), // xdg_positioner::Anchor::BottomRight,
            gravity: Default::default(), // xdg_positioner::Gravity::BottomRight,
            constraint_adjustment: Default::default(),
            reactive: false,
            parent_size: Some(parent_initial_geom.size),
            parent_configure_serial: parent_toplevel.current_configure_serial(),
        };

        state.reposition_popup(&popup_surface, &new_positioner_state, 1); // Token is 1

        let repositioned_geom = *managed_popup.current_geometry.read().unwrap();

        // We expect the geometry to change from the initial one calculated by new_popup
        // The exact value depends on calculate_popup_geometry, so we check it's different.
        // A more precise test would replicate calculate_popup_geometry's logic for the new_positioner_state.
        assert_ne!(repositioned_geom, initial_popup_geom, "Popup geometry should change after reposition.");

        // Example of a more direct check if we knew the exact outcome of calculate_popup_geometry:
        // let expected_geom = smithay::wayland::shell::xdg::calculate_popup_geometry(
        //     &new_positioner_state, parent_initial_geom, popup_surface.wl_surface()
        // );
        // assert_eq!(repositioned_geom, expected_geom);
        // For now, just checking it's not the same as initial is a good first step.
    }

    #[test]
    fn test_popup_grab() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let parent_toplevel = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(parent_toplevel.clone());

        let popup_surface = mock_popup_surface(&mut state, &client, &parent_toplevel);
        let client_data = XdgWmBaseClientData::new(XDG_WM_BASE_VERSION);
        state.new_popup(popup_surface.clone(), &client_data);

        // This test mainly ensures the handler runs without panic.
        // Verifying the grab requires checking seat state, which is complex for a unit test here.
        // Smithay's grab_popup itself has its own tests.
        let seat_resource = state.seat.wl_seat().clone(); // Get the WlSeat resource from DesktopState.seat
        state.popup_grab(&popup_surface, &seat_resource, Serial::from(1));
        // No direct assert, but tracing logs in the method would indicate success/failure.
    }

    // ANCHOR: WorkspaceAssignmentAndTilingInteractionTests
    #[test]
    fn test_map_toplevel_assigns_to_active_workspace_and_applies_tiling() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);

        // Determine primary output and its active workspace for assertions
        let primary_output_name = state.primary_output_name.read().unwrap().clone().unwrap_or_else(|| state.outputs.first().unwrap().name());
        let active_ws_id = *state.active_workspaces.read().unwrap().get(&primary_output_name).unwrap();

        let active_ws_on_primary_output = state.output_workspaces.get(&primary_output_name).unwrap()
            .iter().find(|ws| ws.read().unwrap().id == active_ws_id).unwrap().clone();

        // Set active workspace on primary output to tiling
        *active_ws_on_primary_output.write().unwrap().tiling_layout.write().unwrap() = crate::compositor::workspaces::TilingLayout::MasterStack;

        // Create the ManagedWindow via new_toplevel first
        state.new_toplevel(toplevel_surface.clone());
        let managed_window_arc = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();

        // Before map_toplevel, workspace_id and output_name should be None
        assert!(managed_window_arc.workspace_id.read().unwrap().is_none());
        assert!(managed_window_arc.output_name.read().unwrap().is_none());

        state.map_toplevel(&toplevel_surface); // This should assign to primary output's active WS and apply tiling

        assert_eq!(*managed_window_arc.workspace_id.read().unwrap(), Some(active_ws_id));
        assert_eq!(*managed_window_arc.output_name.read().unwrap(), Some(primary_output_name.clone()));
        assert!(active_ws_on_primary_output.read().unwrap().contains_window(&managed_window_arc.domain_id));

        // Check if geometry reflects tiling (for a single window on an output, it's the full output area)
        let output_geom = state.outputs.iter().find(|o| o.name() == primary_output_name)
                            .and_then(|o| state.space.output_geometry(o))
                            .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0),(800,600)));
        assert_eq!(*managed_window_arc.current_geometry.read().unwrap(), output_geom, "Single window in tiled layout should take full output area.");
    }

    #[test]
    fn test_toplevel_destroyed_removes_from_workspace_and_applies_tiling() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);

        let primary_output_name = state.primary_output_name.read().unwrap().clone().unwrap();
        let active_ws_id = *state.active_workspaces.read().unwrap().get(&primary_output_name).unwrap();
        let active_ws_on_primary_output = state.output_workspaces.get(&primary_output_name).unwrap()
            .iter().find(|ws| ws.read().unwrap().id == active_ws_id).unwrap().clone();
        *active_ws_on_primary_output.write().unwrap().tiling_layout.write().unwrap() = crate::compositor::workspaces::TilingLayout::MasterStack;

        // Add two windows to the primary output's active workspace
        let ts1 = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(ts1.clone()); // Creates MW
        let mw1 = state.find_managed_window_by_wl_surface(ts1.wl_surface()).unwrap();
        state.map_toplevel(&ts1); // Assigns to ws, output, and applies tiling (mw1 takes full space)

        let ts2 = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(ts2.clone()); // Creates MW
        let mw2 = state.find_managed_window_by_wl_surface(ts2.wl_surface()).unwrap();
        // Manually set output and workspace for mw2 before map_toplevel for consistent testing of tiling effect
        *mw2.output_name.write().unwrap() = Some(primary_output_name.clone());
        *mw2.workspace_id.write().unwrap() = Some(active_ws_id);
        active_ws_on_primary_output.read().unwrap().add_window(mw2.domain_id);
        state.map_toplevel(&ts2); // Applies tiling again (mw1 master, mw2 stack)

        let mw1_domain_id = mw1.domain_id;
        assert_eq!(active_ws_on_primary_output.read().unwrap().windows.read().unwrap().len(), 2);

        state.toplevel_destroyed(ts1.clone());

        assert!(state.find_managed_window_by_wl_surface(ts1.wl_surface()).is_none());
        assert!(!active_ws_on_primary_output.read().unwrap().contains_window(&mw1_domain_id));
        assert_eq!(active_ws_on_primary_output.read().unwrap().windows.read().unwrap().len(), 1);

        let output_geom = state.outputs.iter().find(|o| o.name() == primary_output_name)
                            .and_then(|o| state.space.output_geometry(o))
                            .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0),(800,600)));
        assert_eq!(*mw2.current_geometry.read().unwrap(), output_geom, "Remaining window mw2 should now take full workspace area.");
    }
    // ANCHOR_END: WorkspaceAssignmentAndTilingInteractionTests


    // --- Additional Tests for XdgShellHandler logic ---

    #[test]
    fn test_new_toplevel_error_role_already_set() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        let xdg_surface_wrapper = toplevel_surface.xdg_surface();

        // First call to new_toplevel should succeed and set the role
        state.new_toplevel(toplevel_surface.clone());
        assert_eq!(*xdg_surface_wrapper.user_data().get::<Arc<XdgSurfaceUserData>>().unwrap().role.lock().unwrap(), XdgSurfaceRole::Toplevel);

        // Second call to new_toplevel for the same surface should fail due to role already set
        // We can't directly check for posted_error without a client mock, but we ensure it doesn't panic
        // and the number of managed windows doesn't increase.
        let initial_window_count = state.windows.len();
        state.new_toplevel(toplevel_surface.clone()); // Attempt to assign role again
        assert_eq!(state.windows.len(), initial_window_count, "Window count should not increase on role error.");
        // Protocol error XdgSurfaceError::Role should have been posted.
    }

    #[test]
    fn test_new_toplevel_error_surface_destroyed() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        let xdg_surface_wrapper = toplevel_surface.xdg_surface();
        let user_data_arc = xdg_surface_wrapper.user_data().get::<Arc<XdgSurfaceUserData>>().unwrap().clone();

        // Mark XdgSurfaceUserData as destroyed
        *user_data_arc.state.lock().unwrap() = XdgSurfaceState::Destroyed;

        let initial_window_count = state.windows.len();
        state.new_toplevel(toplevel_surface.clone());
        assert_eq!(state.windows.len(), initial_window_count, "Window count should not increase if XDG surface is already destroyed.");
        // Protocol error XdgSurfaceError::Defunct should have been posted.
    }

    #[test]
    fn test_new_popup_error_parent_not_managed() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);

        // Create a parent surface that won't be a ManagedWindow in our state
        let unmanaged_parent_toplevel = mock_toplevel_surface(&mut state, &client);
        // Do NOT call state.new_toplevel for unmanaged_parent_toplevel

        let popup_surface = mock_popup_surface(&mut state, &client, &unmanaged_parent_toplevel);

        let initial_window_count = state.windows.len();
        let client_data = XdgWmBaseClientData::new(XDG_WM_BASE_VERSION);
        state.new_popup(popup_surface.clone(), &client_data);
        assert_eq!(state.windows.len(), initial_window_count, "Popup count should not increase if parent is not a managed window.");
        // Protocol error XdgSurfaceError::InvalidPopupParent should have been posted.
    }

    #[test]
    fn test_map_toplevel_error_no_user_data() {
        // This tests a compositor bug scenario: if new_xdg_surface failed to attach XdgSurfaceUserData.
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);

        // Intentionally *don't* let new_xdg_surface run for its SmithayXdgSurface,
        // or somehow remove the XdgSurfaceUserData from the SmithayXdgSurface.
        // This is tricky to set up perfectly as new_toplevel itself relies on it.
        // Instead, we can test the check inside map_toplevel by creating a toplevel_surface
        // whose underlying xdg_surface_wrapper does not have our UserData.
        // The current mock_toplevel_surface *does* add it.
        // A more direct test would be to ensure the check `xdg_surface_obj.user_data().get::<Arc<XdgSurfaceUserData>>()` exists.
        // For now, we assume that if this path is hit, an error is posted.
        // The existing `map_toplevel` has this check.

        // Simulate a scenario where the XdgSurfaceUserData is missing (e.g., by removing it after it was added by mock_toplevel_surface)
        // This is somewhat artificial for a unit test as mock_toplevel_surface ensures it's there.
        let xdg_surface_wrapper = toplevel_surface.xdg_surface();
        xdg_surface_wrapper.user_data().remove::<Arc<XdgSurfaceUserData>>(); // Try to remove it

        // We also need a ManagedWindow for map_toplevel to proceed to the check
        let domain_id = DomainWindowIdentifier::new_v4();
        let managed_window = ManagedWindow::new_toplevel(toplevel_surface.clone(), domain_id);
        state.windows.insert(domain_id, Arc::new(managed_window));

        state.map_toplevel(&toplevel_surface); // Should log error and post XdgSurfaceError::Defunct
        // Assert that the window did not actually get mapped in our state
        let win_arc = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();
        assert!(!win_arc.state.read().unwrap().is_mapped, "Window should not be mapped if XdgSurfaceUserData was missing.");
    }


    #[test]
    fn test_toplevel_destroyed_updates_user_data_and_removes_from_state() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        let xdg_surface_wrapper = toplevel_surface.xdg_surface();
        let user_data_arc = xdg_surface_wrapper.user_data().get::<Arc<XdgSurfaceUserData>>().unwrap().clone();

        state.new_toplevel(toplevel_surface.clone()); // Adds to state.windows
        let window_arc = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();
        let domain_id = window_arc.domain_id;

        // Map it to an output and workspace to test full cleanup
        let primary_output_name = state.primary_output_name.read().unwrap().clone().unwrap();
        let active_ws_id = *state.active_workspaces.read().unwrap().get(&primary_output_name).unwrap();
        *window_arc.output_name.write().unwrap() = Some(primary_output_name.clone());
        *window_arc.workspace_id.write().unwrap() = Some(active_ws_id);
        state.output_workspaces.get(&primary_output_name).unwrap().iter()
            .find(|ws| ws.read().unwrap().id == active_ws_id).unwrap()
            .read().unwrap().add_window(domain_id);
        state.map_toplevel(&toplevel_surface);


        assert!(state.windows.contains_key(&domain_id));
        assert_eq!(*user_data_arc.state.lock().unwrap(), XdgSurfaceState::PendingConfiguration); // Or Configured if map implies ack

        state.toplevel_destroyed(toplevel_surface.clone());

        assert_eq!(*user_data_arc.state.lock().unwrap(), XdgSurfaceState::Destroyed);
        assert!(!state.windows.contains_key(&domain_id), "Window should be removed from DesktopState.windows");
        assert!(state.space.element_for_surface(toplevel_surface.wl_surface()).is_none(), "Window should be unmapped from space");

        let ws_arc = state.output_workspaces.get(&primary_output_name).unwrap().iter()
            .find(|ws| ws.read().unwrap().id == active_ws_id).unwrap();
        assert!(!ws_arc.read().unwrap().contains_window(&domain_id), "Window should be removed from its workspace");
    }

    #[test]
    fn test_popup_destroyed_updates_user_data_and_removes_from_state() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let parent_toplevel_surface = mock_toplevel_surface(&mut state, &client);
        state.new_toplevel(parent_toplevel_surface.clone()); // Parent needs to be a managed window

        let popup_surface = mock_popup_surface(&mut state, &client, parent_toplevel_surface.wl_surface());
        let xdg_surface_wrapper = popup_surface.xdg_surface();
        let user_data_arc = xdg_surface_wrapper.user_data().get::<Arc<XdgSurfaceUserData>>().unwrap().clone();

        let client_data = XdgWmBaseClientData::new(XDG_WM_BASE_VERSION);
        state.new_popup(popup_surface.clone(), &client_data); // Adds to state.windows

        let popup_arc = state.find_managed_window_by_wl_surface(popup_surface.wl_surface()).unwrap();
        let popup_domain_id = popup_arc.domain_id;

        assert!(state.windows.contains_key(&popup_domain_id));
        assert_ne!(*user_data_arc.state.lock().unwrap(), XdgSurfaceState::Destroyed);

        state.popup_destroyed(popup_surface.clone());

        assert_eq!(*user_data_arc.state.lock().unwrap(), XdgSurfaceState::Destroyed);
        assert!(!state.windows.contains_key(&popup_domain_id), "Popup window should be removed from DesktopState.windows");
        // Popups are not directly mapped to space in the same way as toplevels in these tests,
        // but if they were, we'd check space.element_for_surface(...).is_none()
    }

    #[test]
    fn test_state_changes_update_serials() {
        let mut state = create_minimal_test_state();
        let client = create_test_client(&mut state);
        let toplevel_surface = mock_toplevel_surface(&mut state, &client);
        let xdg_surface_wrapper = toplevel_surface.xdg_surface();

        state.new_toplevel(toplevel_surface.clone());
        let window_arc = state.find_managed_window_by_wl_surface(toplevel_surface.wl_surface()).unwrap();
        let user_data_arc = xdg_surface_wrapper.user_data().get::<Arc<XdgSurfaceUserData>>().unwrap().clone();

        let initial_serial = user_data_arc.last_compositor_configure_serial.lock().unwrap().unwrap();

        state.toplevel_request_set_maximized(&toplevel_surface);
        let maximized_serial = user_data_arc.last_compositor_configure_serial.lock().unwrap().unwrap();
        assert_ne!(initial_serial, maximized_serial, "Serial should update after maximize");

        state.toplevel_request_unset_maximized(&toplevel_surface);
        let unmaximized_serial = user_data_arc.last_compositor_configure_serial.lock().unwrap().unwrap();
        assert_ne!(maximized_serial, unmaximized_serial, "Serial should update after unmaximize");

        state.toplevel_request_set_fullscreen(&toplevel_surface, None);
        let fullscreen_serial = user_data_arc.last_compositor_configure_serial.lock().unwrap().unwrap();
        assert_ne!(unmaximized_serial, fullscreen_serial, "Serial should update after fullscreen");

        state.toplevel_request_unset_fullscreen(&toplevel_surface);
        let unfullscreen_serial = user_data_arc.last_compositor_configure_serial.lock().unwrap().unwrap();
        assert_ne!(fullscreen_serial, unfullscreen_serial, "Serial should update after unfullscreen");

        state.toplevel_request_set_minimized(&toplevel_surface);
        let minimized_serial = user_data_arc.last_compositor_configure_serial.lock().unwrap().unwrap();
        assert_ne!(unfullscreen_serial, minimized_serial, "Serial should update after minimize");
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
