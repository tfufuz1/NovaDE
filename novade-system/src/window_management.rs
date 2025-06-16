//! Window management module for the NovaDE system layer.
//!
//! This module provides window management functionality for the NovaDE desktop environment,
//! focusing on Wayland-native operations via the compositor's internal state.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex}; // Renamed to avoid conflict with Smithay's Mutex if any
use novade_core::types::geometry::{Point, Size, Rect};
use novade_domain::workspaces::core::{Window as DomainWindow, WindowId, WindowState as DomainWindowState, WindowType as DomainWindowType};
use crate::error::{SystemError, SystemResult, to_system_error, SystemErrorKind};

// Compositor specific imports
use smithay::desktop::{Space, Window as SmithayWindow};
use smithay::input::{Seat, pointer::PointerHandle, keyboard::KeyboardHandle, SeatHandler}; // SeatHandler for focus
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::XdgToplevel;
use smithay::utils::SERIAL_COUNTER;


use crate::compositor::state::DesktopState; // Main compositor state
use crate::compositor::shell::xdg::{XdgSurfaceData, XdgToplevelData, XdgRoleSpecificData, ToplevelState as NovaToplevelState}; // Our XDG data

// ANCHOR: WindowId Management - How to map SmithayWindow to novade_domain::workspaces::core::WindowId
// Option 1: Generate WindowId on creation and store it in XdgSurfaceData. (This is implemented)
// Option 2: Use WlSurface::id() or a hash of it. (Less ideal)
// Option 3: Use SmithayWindow internal ID if accessible and suitable. (Smithay's Window::id() is smithay::utils::Id, not UUID based)

// Import the WindowManager trait from novade-domain
use novade_domain::workspaces::traits::{WindowManager as DomainWindowManager, DomainResult};


/// Wayland-native window manager implementation for NovaDE.
/// Interacts directly with the compositor's DesktopState.
pub struct NovaWindowManager {
    /// Handle to the main compositor state.
    desktop_state: Arc<StdMutex<DesktopState>>,
    // ANCHOR: Window Cache Evaluation
    // The window_cache might be useful for short-term caching of converted DomainWindow objects
    // if conversion from SmithayWindow is expensive and called frequently.
    // However, direct querying from DesktopState.space is often preferred for real-time accuracy.
    // For now, we will remove the explicit cache and perform conversions on-demand.
    // If performance issues arise, a cache can be reintroduced.
}

impl NovaWindowManager {
    /// Creates a new NovaDE window manager.
    ///
    /// # Arguments
    ///
    /// * `desktop_state` - A handle to the main compositor state.
    ///
    /// # Returns
    ///
    /// A new NovaDE window manager.
    pub fn new(desktop_state: Arc<StdMutex<DesktopState>>) -> SystemResult<Self> {
        Ok(NovaWindowManager {
            desktop_state,
        })
    }

    // Helper to find a SmithayWindow by our WindowId.
    // This requires a mechanism to map WindowId to SmithayWindow.
    // ANCHOR: This is a critical part for ID mapping.
    // For now, we iterate all windows. This is inefficient.
    // A HashMap<WindowId, WlSurfaceId (or SmithayWindow::id())> in DesktopState or NovaWindowManager
    // would be better, or storing WindowId in XdgSurfaceData.
    fn find_smithay_window_by_id(&self, id: WindowId) -> Option<SmithayWindow> {
        let state = self.desktop_state.lock().unwrap();
        state.space.elements().find(|w| {
            // Placeholder for ID comparison.
            // Assume convert_smithay_window_to_domain_window can extract/generate a comparable ID
            // or that we have a way to get our WindowId from SmithayWindow's data.
            // For this iteration, let's imagine `get_domain_id_from_smithay_window` exists.
            get_domain_id_from_smithay_window(w) == Some(id)
        }).cloned()
    }

    fn get_xdg_toplevel_from_smithay_window(&self, smithay_window: &SmithayWindow) -> Option<XdgToplevel> {
        match smithay_window.toplevel() {
            Some(smithay::desktop::WindowSurfaceType::Xdg(toplevel)) => Some(toplevel.xdg_toplevel().clone()),
            _ => None,
        }
    }
}

// Retrieves the stored NovaDE domain WindowId from a SmithayWindow.
fn get_domain_id_from_smithay_window(window: &SmithayWindow) -> Option<WindowId> {
    window.wl_surface().and_then(|surface| {
        surface.data::<StdMutex<XdgSurfaceData>>().and_then(|data_mutex| {
            data_mutex.lock().ok().and_then(|data| data.domain_id)
        })
    })
}


#[async_trait]
impl DomainWindowManager for NovaWindowManager {
    async fn get_windows(&self) -> DomainResult<Vec<DomainWindow>> {
        let state = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        
        let windows = state.space.elements()
            .filter_map(|smithay_window| convert_smithay_window_to_domain_window(smithay_window, &state))
            .collect();
        Ok(windows)
    }
    
    async fn get_window(&self, id: WindowId) -> DomainResult<DomainWindow> {
        let desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        let smithay_window_ref = self.find_smithay_window_by_id_with_state(id, &desktop_state_guard)
            .ok_or_else(|| format!("Window not found: {}", id))?;
        
        convert_smithay_window_to_domain_window(smithay_window_ref, &desktop_state_guard)
            .ok_or_else(|| format!("Window found but failed to convert: {}", id))
    }
    
    async fn focus_window(&self, id: WindowId) -> DomainResult<()> {
        let mut desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;

        let smithay_window_clone = self.find_smithay_window_by_id_with_state(id, &desktop_state_guard)
            .ok_or_else(|| format!("Window not found for focus: {}", id))?
            .clone();

        desktop_state_guard.space.raise_element(&smithay_window_clone, true);

        if let Some(keyboard) = desktop_state_guard.seat.get_keyboard() {
            keyboard.set_focus(&mut *desktop_state_guard, Some(smithay_window_clone.clone().into()), SERIAL_COUNTER.next_serial());
            smithay_window_clone.set_activated(true);
            for window_ref in desktop_state_guard.space.elements().filter(|w| w.id() != smithay_window_clone.id()) {
                window_ref.set_activated(false);
            }
            Ok(())
        } else {
            Err("No keyboard found on seat".to_string())
        }
    }
    
    async fn move_window(&self, id: WindowId, position: Point) -> DomainResult<()> {
        let mut desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        let smithay_window_clone = self.find_smithay_window_by_id_with_state(id, &desktop_state_guard)
            .ok_or_else(|| format!("Window not found for move: {}", id))?
            .clone();

        let new_location = smithay::utils::Point::from((position.x, position.y));
        desktop_state_guard.space.map_element(smithay_window_clone, new_location, false);
        Ok(())
    }
    
    async fn resize_window(&self, id: WindowId, size: Size) -> DomainResult<()> {
        let desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        let smithay_window_ref = self.find_smithay_window_by_id_with_state(id, &desktop_state_guard)
            .ok_or_else(|| format!("Window not found for resize: {}", id))?;

        if let Some(xdg_toplevel) = self.get_xdg_toplevel_from_smithay_window(smithay_window_ref) {
            xdg_toplevel.send_configure_bounds(smithay::utils::Size::from((size.width, size.height)));
            xdg_toplevel.send_pending_configure();
            Ok(())
        } else {
            Err("Window is not an XDG toplevel".to_string())
        }
    }
    
    async fn set_window_state(&self, id: WindowId, domain_state: DomainWindowState) -> DomainResult<()> {
        let desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        let smithay_window_ref = self.find_smithay_window_by_id_with_state(id, &desktop_state_guard)
            .ok_or_else(|| format!("Window not found for set_state: {}", id))?;

        if let Some(xdg_toplevel) = self.get_xdg_toplevel_from_smithay_window(smithay_window_ref) {
            match domain_state {
                DomainWindowState::Maximized => xdg_toplevel.set_maximized(),
                DomainWindowState::Minimized => xdg_toplevel.set_minimized(),
                DomainWindowState::Fullscreen => xdg_toplevel.set_fullscreen(None),
                DomainWindowState::Normal | DomainWindowState::Floating => {
                    xdg_toplevel.unset_maximized();
                    xdg_toplevel.unset_fullscreen();
                }
                DomainWindowState::Tiled(_)=> return Err("Tiled state not yet supported".to_string()),
                _ => return Err(format!("Unsupported window state transition: {:?}", domain_state)),
            }
            xdg_toplevel.send_pending_configure();
            Ok(())
        } else {
            Err("Window is not an XDG toplevel".to_string())
        }
    }
    
    async fn close_window(&self, id: WindowId) -> DomainResult<()> {
        let desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        let smithay_window_ref = self.find_smithay_window_by_id_with_state(id, &desktop_state_guard)
            .ok_or_else(|| format!("Window not found for close: {}", id))?;

        if let Some(xdg_toplevel) = self.get_xdg_toplevel_from_smithay_window(smithay_window_ref) {
            xdg_toplevel.send_close();
            Ok(())
        } else {
            Err("Window is not an XDG toplevel".to_string())
        }
    }

    async fn hide_window_for_workspace(&self, id: WindowId) -> DomainResult<()> {
        let desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        let smithay_window = self.find_smithay_window_by_id_with_state(id, &desktop_state_guard)
            .ok_or_else(|| format!("Window not found for hide: {}", id))?;
        
        // ANCHOR: Actual "hiding" vs "unmapping".
        // Unmapping removes it from the space, effectively hiding it.
        // True visibility might be a rendering flag later.
        tracing::info!("Hiding (unmapping) window {:?} for workspace switch.", id);
        desktop_state_guard.space.unmap_elem(smithay_window);
        // smithay_window.set_visible(false); // If SmithayWindow had a direct visibility flag.
        Ok(())
    }

    async fn show_window_for_workspace(&self, id: WindowId) -> DomainResult<()> {
        let desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        let smithay_window = self.find_smithay_window_by_id_with_state(id, &desktop_state_guard)
            .ok_or_else(|| format!("Window not found for show: {}", id))?;

        // ANCHOR: Actual "showing" vs "mapping".
        // Mapping ensures it's in the space. If it was unmapped, this brings it back.
        // The window should ideally be mapped to its last known position or a default one for the workspace.
        // `space.map_element` requires a location. We might need to store/retrieve this.
        // For now, if it's already in the space (e.g. never unmapped, just on an inactive workspace),
        // calling map() might just ensure it's drawn. If it was unmapped, we need its location.
        if !smithay_window.is_mapped() {
             // We need a location to map it. Defaulting to (0,0) if unmapped.
             // A better approach would be to store its last known relative pos in workspace.
            tracing::info!("Showing (mapping) window {:?} at (0,0) for workspace switch.", id);
            desktop_state_guard.space.map_element(smithay_window.clone(), (0,0), false);
        } else {
            // Ensure it's raised if already mapped but on an inactive workspace.
            desktop_state_guard.space.raise_element(smithay_window, false);
            tracing::info!("Ensuring window {:?} is visible (raised) for workspace switch.", id);
        }
        // smithay_window.set_visible(true);
        Ok(())
    }

    async fn get_primary_output_id(&self) -> DomainResult<Option<String>> {
        let desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        let display_manager_guard = desktop_state_guard.display_manager.lock().map_err(|e| format!("Failed to lock DisplayManager: {}", e))?;
        Ok(display_manager_guard.get_primary_output().map(|o| o.id.clone()))
    }

    async fn get_output_work_area(&self, output_id: &str) -> DomainResult<Rect> {
        let desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        let display_manager_guard = desktop_state_guard.display_manager.lock().map_err(|e| format!("Failed to lock DisplayManager: {}", e))?;

        display_manager_guard.get_output_by_id(output_id)
            .map(|o| o.work_area) // work_area is already a novade_core::types::geometry::Rect
            .ok_or_else(|| format!("Output with ID {} not found", output_id))
    }

    async fn get_focused_output_id(&self) -> DomainResult<Option<String>> {
        // ANCHOR: Implement proper focused output tracking.
        // This requires knowing which output currently has input focus (e.g., contains the focused window or cursor).
        // For now, default to the primary output if it exists.
        let desktop_state_guard = self.desktop_state.lock().map_err(|e| format!("Failed to lock DesktopState: {}", e))?;
        let display_manager_guard = desktop_state_guard.display_manager.lock().map_err(|e| format!("Failed to lock DisplayManager: {}", e))?;
        Ok(display_manager_guard.get_primary_output().map(|o| o.id.clone()))
    }
}

// Converts a SmithayWindow to the NovaDE domain's Window representation.
fn convert_smithay_window_to_domain_window(smithay_window: &SmithayWindow, desktop_state: &DesktopState) -> Option<DomainWindow> {
    let domain_id = get_domain_id_from_smithay_window(smithay_window)?;

    let title = smithay_window.title().unwrap_or_else(|| "Untitled".to_string());
    
    let geometry = desktop_state.space.element_geometry(smithay_window)
        .map(|geo| Rect::new(Point::new(geo.loc.x, geo.loc.y), Size::new(geo.size.w, geo.size.h)))
        .unwrap_or_else(|| Rect::new(Point::new(0,0), Size::new(0,0))); // Should always have geometry if in space

    // Determine WindowType
    let window_type = match smithay_window.toplevel() {
        Some(smithay::desktop::WindowSurfaceType::Xdg(toplevel)) => {
            // ANCHOR: Distinguish Dialog, Utility from Normal toplevels.
            // This might require checking xdg_toplevel parent or other hints if the protocol supports them.
            // For now, all XDG toplevels are considered "Normal".
            // Popups are generally not managed as "windows" in the same way by this WindowManager.
            DomainWindowType::Normal
        }
        _ => DomainWindowType::Unknown, // Should not happen for windows in space from XDG shell
    };

    // Determine WindowState
    let mut domain_state = DomainWindowState::Normal; // Default
    let is_activated = smithay_window.is_activated(); // Smithay's Window tracks activation state

    if let Some(surface) = smithay_window.wl_surface() {
        if let Some(data_mutex) = surface.data::<StdMutex<XdgSurfaceData>>() {
            if let Ok(surface_data) = data_mutex.lock() { // Check lock success
                if let XdgRoleSpecificData::Toplevel(toplevel_data) = &surface_data.role_data {
                    if toplevel_data.current_state.fullscreen {
                        domain_state = DomainWindowState::Fullscreen;
                    } else if toplevel_data.current_state.maximized {
                        domain_state = DomainWindowState::Maximized;
                    } else if toplevel_data.current_state.minimized {
                        // ANCHOR: Minimized state might mean unmapped. How is this represented?
                        // If minimized windows are unmapped from the main space, they might not appear
                        // via `state.space.elements()`. This conversion assumes they are still in space
                        // but have a "minimized" flag.
                        domain_state = DomainWindowState::Minimized;
                    }
                    // ANCHOR: Tiled/Floating state determination will depend on tiling manager integration.
                    // For now, if not maximized/fullscreen/minimized, it's Normal (implies Floating in a tiling context).
                }
            }
        }
    }
    // TODO: Incorporate `is_activated` into DomainWindowState if relevant,
    // or handle it separately if focus is not a "state" in DomainWindow.
    // DomainWindowState doesn't have an "Active" variant. Focus is usually separate.

    Some(DomainWindow::new(domain_id, title, window_type, geometry, domain_state))
}

// No tests for now as they would require a running compositor instance or extensive mocking.
// #[cfg(test)]
// mod tests {
//     use super::*;
//     // Placeholder tests
// }
