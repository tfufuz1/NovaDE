use smithay::{
    // delegate_xdg_shell is in core/state.rs
    reexports::wayland_server::{
        protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
        DisplayHandle, 
    },
    utils::{Logical, Point, Rectangle, Serial, Size},
    wayland::{
        compositor::SurfaceData as SmithaySurfaceData, 
        seat::Seat, 
        shell::xdg::{
            Configure, PopupSurface, PositionerState, ToplevelConfigure, ToplevelSurface,
            XdgShellHandler, XdgShellState, XdgToplevelSurfaceData, XdgWmBaseClientData,
            ToplevelState as XdgToplevelState, 
            XdgPopupSurfaceData, XdgSurfaceConfigure, // Correct type for ack_configure data
        },
    },
    input::SeatHandler, 
};
use std::sync::{Arc, Mutex}; // Added Mutex for potential interior mutability if needed later

use crate::compositor::{
    core::state::DesktopState,
    xdg_shell::{
        types::{DomainWindowIdentifier, ManagedWindow},
        errors::XdgShellError, 
    },
    // NovadeSurfaceData is not directly used here, SurfaceData on WlSurface is from Smithay
};

impl XdgShellHandler for DesktopState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "New XDG Toplevel created.");
        
        let domain_window_id = DomainWindowIdentifier::new_v4();
        // is_mapped will be false by default in ManagedWindow::new_toplevel
        let mut managed_window = ManagedWindow::new_toplevel(surface.clone(), domain_window_id);
        
        // Placeholder initial geometry
        managed_window.current_geometry = Rectangle::from_loc_and_size((100, 100), (300, 200));
        
        // Send initial configure
        surface.with_pending_state(|state| {
            state.size = Some(managed_window.current_geometry.size);
            // Can also set initial states like Resizing, Activated here if needed
        });
        let configure_serial = surface.send_configure();
        managed_window.last_configure_serial = Some(configure_serial); // Store serial

        let window_arc = Arc::new(managed_window); // Wrap in Arc after initial setup
        self.windows.insert(domain_window_id, window_arc.clone());

        tracing::info!("Created ManagedWindow with ID {:?}, configured with size {:?} and serial {:?}", 
            window_arc.id, window_arc.geometry().size, configure_serial);
        // Window is not mapped to space yet; that happens in `map_toplevel`.
    }

    fn new_popup(&mut self, surface: PopupSurface, _client_data: &XdgWmBaseClientData) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "New XDG Popup created.");
        
        // Find parent ManagedWindow (simplified)
        let parent_wl_surface = surface.parent_surface();
        let parent_managed_window_arc = parent_wl_surface.and_then(|parent_surf| {
            self.windows.values().find(|win| win.wl_surface().map_or(false, |s| s == parent_surf)).cloned()
        });
        let parent_domain_id = parent_managed_window_arc.as_ref().map_or_else(DomainWindowIdentifier::new_v4, |p| p.domain_id);

        let _managed_popup = ManagedWindow::new_popup(surface.clone(), parent_domain_id, parent_managed_window_arc);
        // Popups are not typically added to the main window HashMap or Space.
        // Their lifetime and mapping are tied to their parent.
        
        let _configure_serial = surface.send_configure(); // Send initial configure for the popup
        // For popups, last_configure_serial might be less critical unless specific ack logic is needed.
        tracing::debug!("Sent initial configure for new popup {:?}", surface.wl_surface().id());
    }

    fn map_toplevel(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Mapping XDG Toplevel.");

        let found_window_arc = self.windows.values_mut().find(|win| { // Use values_mut to potentially modify
            win.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())
        });

        if let Some(window_arc_mut) = found_window_arc { // This is &mut Arc<ManagedWindow>
            // To modify `is_mapped` inside Arc, we need Arc::get_mut, which only works if strong_count == 1.
            // This is unlikely if the window is already in `self.windows` and potentially elsewhere.
            // Solution: Use interior mutability for `is_mapped` in ManagedWindow, or make it a Cell/RefCell.
            // For this plan, we will assume `ManagedWindow::is_mapped` is updated via `self_update` or similar
            // if it needs to reflect the Space's state change.
            // The primary action here is adding to Space.
            // The `Window::is_mapped()` trait method will be the source of truth for Space.

            // Get an Arc clone for Space.map_window
            let window_clone_for_space = self.windows.values().find(|win| {
                 win.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())
            }).unwrap().clone(); // Unwrap is safe as we just found it with values_mut.

            self.space.map_window(window_clone_for_space.clone(), (100, 100), true); // Placeholder location, activate
            
            // If ManagedWindow.is_mapped needs to be true for Window::is_mapped() to work correctly with Space:
            // This requires `is_mapped` to have interior mutability (e.g. `Mutex<bool>` or `Cell<bool>`)
            // For example, if `is_mapped: Cell<bool>`:
            // window_clone_for_space.is_mapped.set(true);
            // Or, if ManagedWindow had a method: window_clone_for_space.set_mapped(true);

            tracing::info!("Mapped window {:?} to space.", window_clone_for_space.id);
            self.space.damage_all_outputs();
        } else {
            tracing::error!("Tried to map a toplevel not found in internal tracking: {:?}", surface.wl_surface().id());
        }
    }

    fn unmap_toplevel(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "Unmapping XDG Toplevel.");
        let found_window_arc = self.windows.values().find(|win| {
            win.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())
        }).cloned(); // Clone Arc to use with space.unmap_window

        if let Some(window_arc) = found_window_arc {
            // Similar to map_toplevel, if `ManagedWindow.is_mapped` needs to be updated:
            // window_arc.is_mapped.set(false); // if using Cell<bool>
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
        // Custom logic: Find our ManagedWindow and validate serial or apply pending states.
        // This is mostly handled by Smithay's XdgShellState and Window::send_configure/self_update.
    }

    // --- Request Handlers ---
    fn move_request(&mut self, surface: &ToplevelSurface, _seat: &Seat<Self>, serial: Serial) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), ?serial, "XDG Toplevel move request received.");
        // Placeholder: Actual move grab logic is complex.
        // Typically, you'd validate the serial and initiate a move grab with the input system.
    }

    fn resize_request(&mut self, surface: &ToplevelSurface, _seat: &Seat<Self>, serial: Serial, edges: smithay::reexports::protocols::xdg::shell::server::xdg_toplevel::ResizeEdge) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), ?serial, ?edges, "XDG Toplevel resize request received.");
        // Placeholder: Actual resize grab logic is complex.
    }

    fn set_title_request(&mut self, surface: &ToplevelSurface) {
        let title = surface.title();
        tracing::info!(surface_id = ?surface.wl_surface().id(), title = ?title, "XDG Toplevel set_title request.");
        // Title is stored on ToplevelSurface. ManagedWindow::self_update() will sync it.
    }

    fn set_app_id_request(&mut self, surface: &ToplevelSurface) {
        let app_id = surface.app_id();
        tracing::info!(surface_id = ?surface.wl_surface().id(), app_id = ?app_id, "XDG Toplevel set_app_id request.");
        // AppId is stored on ToplevelSurface. ManagedWindow::self_update() will sync it.
    }

    fn set_maximized_request(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG Toplevel set_maximized request.");
        surface.with_pending_state(|state| state.states.set(XdgToplevelState::Maximized, true));
        let serial = surface.send_configure();
        // Update ManagedWindow's last_configure_serial if reliable tracking is needed beyond XdgShellState.
        if let Some(win_arc) = self.windows.values().find(|w| w.xdg_surface.wl_surface().as_ref() == Some(surface.wl_surface())) {
            // This needs interior mutability for last_configure_serial on ManagedWindow
            // win_arc.last_configure_serial.set(Some(serial)); // Example if Cell<Option<Serial>>
            tracing::debug!("Maximized request for {:?}, new configure serial {:?}", surface.wl_surface().id(), serial);
        }
    }

    fn unset_maximized_request(&mut self, surface: &ToplevelSurface) {
        tracing::info!(surface_id = ?surface.wl_surface().id(), "XDG Toplevel unset_maximized request.");
        surface.with_pending_state(|state| state.states.set(XdgToplevelState::Maximized, false));
        let serial = surface.send_configure();
        // Similar update for last_configure_serial if needed.
        tracing::debug!("Unmaximized request for {:?}, new configure serial {:?}", surface.wl_surface().id(), serial);
    }
    
    // Implement other request handlers (setMinimized, setFullscreen, etc.) similarly.
    // For brevity, only a few are detailed here.

    fn toplevel_destroyed(&mut self, toplevel: ToplevelSurface) {
        let wl_surface_id = toplevel.wl_surface().id();
        tracing::info!(surface_id = ?wl_surface_id, "XDG Toplevel destroyed.");
        
        let mut found_domain_id = None;
        // Find by wl_surface comparison as ToplevelSurface itself might be a new wrapper post-destruction signal.
        if let Some(window_arc) = self.windows.values()
            .find(|win| win.xdg_surface.wl_surface().as_ref() == Some(toplevel.wl_surface()))
            .cloned() {
            
            found_domain_id = Some(window_arc.domain_id);
            self.space.unmap_window(&window_arc); 
            tracing::info!("Unmapped window with internal ID {:?} from space due to destruction.", window_arc.id);
        }

        if let Some(domain_id) = found_domain_id {
            if self.windows.remove(&domain_id).is_some() {
                tracing::info!("Removed window with domain ID {:?} from internal tracking.", domain_id);
            } else {
                // This case might occur if the window was already removed due to some other event.
                tracing::warn!("Window for domain ID {:?} (from destroyed toplevel {:?}) not found in self.windows for removal, might have been already removed.", domain_id, wl_surface_id);
            }
        } else {
            tracing::warn!("Destroyed toplevel {:?} was not found in self.windows by its WlSurface.", wl_surface_id);
        }
        
        self.space.damage_all_outputs();
    }

    fn popup_destroyed(&mut self, popup: PopupSurface) {
        tracing::info!(surface_id = ?popup.wl_surface().id(), "XDG Popup destroyed.");
        // Cleanup if popups are stored in parent ManagedWindow or similar structure.
    }
}
