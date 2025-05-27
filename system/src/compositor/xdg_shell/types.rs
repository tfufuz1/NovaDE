use smithay::{
    desktop::{Window, WindowSurfaceType, Space},
    reexports::wayland_server::protocol::{wl_surface, wl_seat},
    utils::{Point, Size, Rectangle, Logical, Serial, BufferCoords, Physical, Transform}, // Added BufferCoords, Physical, Transform
    wayland::{
        compositor::SurfaceDataWrapper, // For accessing SurfaceData
        shell::xdg::{ToplevelSurface, PopupSurface, XdgPopupSurfaceData, XdgToplevelSurfaceData, PositionerState, XdgShellHandler}, // Added PositionerState, XdgShellHandler
    },
    input::Seat, // For Seat in grab
};
use uuid::Uuid;
use std::sync::{Arc, Mutex}; // Added Mutex
use crate::compositor::core::state::DesktopState; // For XdgShellHandler context
use crate::compositor::core::surface_management; // For SurfaceData access

/// A domain-specific identifier for a window.
/// For now, this is a simple wrapper around Uuid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DomainWindowIdentifier(pub Uuid);

impl Default for DomainWindowIdentifier {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Represents the type of XDG surface.
#[derive(Debug, Clone, PartialEq)]
pub enum XdgSurfaceVariant {
    Toplevel(ToplevelSurface),
    Popup(PopupSurface),
}

impl XdgSurfaceVariant {
    pub fn wl_surface(&self) -> &wl_surface::WlSurface {
        match self {
            XdgSurfaceVariant::Toplevel(t) => t.wl_surface(),
            XdgSurfaceVariant::Popup(p) => p.wl_surface(),
        }
    }
}

/// Represents a managed window in the compositor, either a toplevel or a popup.
#[derive(Debug, Clone, PartialEq)]
pub struct ManagedWindow {
    pub internal_id: Uuid, // Compositor-internal UUID for this managed instance
    pub domain_id: DomainWindowIdentifier, // ID used for domain-level communication
    pub surface_variant: XdgSurfaceVariant,
    // Geometry: Smithay's Space handles current geometry. We store requested/configured.
    pub current_geometry: Mutex<Rectangle<i32, Logical>>, // Actual current geometry on screen
    pub requested_size: Mutex<Size<i32, Logical>>, // Size requested by client or set by compositor
    pub min_size: Mutex<Option<Size<i32, Logical>>>,
    pub max_size: Mutex<Option<Size<i32, Logical>>>,
    pub parent_id: Option<DomainWindowIdentifier>, // For popups or transient toplevels
    pub is_mapped_compositor: Mutex<bool>, // Is the window currently mapped by the compositor?
                                  // `is_mapped()` from Window trait checks client's view.
}

impl ManagedWindow {
    pub fn new_toplevel(toplevel: ToplevelSurface, domain_id: Option<DomainWindowIdentifier>) -> Self {
        let internal_id = Uuid::new_v4();
        let did = domain_id.unwrap_or_else(|| DomainWindowIdentifier(internal_id)); // Use internal if no domain_id provided

        // Initial geometry can be placeholder, will be updated by layout/mapping logic
        let initial_geometry = Rectangle::from_loc_and_size((0, 0), (0, 0));
        let initial_requested_size = toplevel.current_state().size.unwrap_or_default();

        Self {
            internal_id,
            domain_id: did,
            surface_variant: XdgSurfaceVariant::Toplevel(toplevel.clone()),
            current_geometry: Mutex::new(initial_geometry),
            requested_size: Mutex::new(initial_requested_size),
            min_size: Mutex::new(toplevel.current_state().min_size),
            max_size: Mutex::new(toplevel.current_state().max_size),
            parent_id: toplevel.get_parent().map(|parent_surface| {
                // This is tricky: parent_surface is a WlSurface. We need its DomainWindowIdentifier.
                // This requires looking up the parent ManagedWindow.
                // For now, this logic will be incomplete or placeholder.
                // The actual parent_id should be resolved in new_toplevel handler.
                tracing::warn!("Toplevel parent resolution in ManagedWindow::new_toplevel is placeholder.");
                DomainWindowIdentifier(Uuid::nil()) // Placeholder
            }),
            is_mapped_compositor: Mutex::new(false),
        }
    }

    pub fn new_popup(popup: PopupSurface, parent_domain_id: DomainWindowIdentifier) -> Self {
        let internal_id = Uuid::new_v4();
        let initial_geometry = Rectangle::from_loc_and_size((0, 0), (0, 0)); // Popups are positioned relative to parent

        Self {
            internal_id,
            domain_id: DomainWindowIdentifier(internal_id), // Popups might not need distinct domain IDs from internal
            surface_variant: XdgSurfaceVariant::Popup(popup.clone()),
            current_geometry: Mutex::new(initial_geometry),
            requested_size: Mutex::new(Size::default()), // Popups don't have a "requested_size" in the same way
            min_size: Mutex::new(None),
            max_size: Mutex::new(None),
            parent_id: Some(parent_domain_id),
            is_mapped_compositor: Mutex::new(false),
        }
    }

    /// Helper to get the underlying WlSurface.
    pub fn wl_surface(&self) -> &wl_surface::WlSurface {
        self.surface_variant.wl_surface()
    }

    /// Sets the compositor-side mapped state.
    pub fn set_mapped_compositor(&self, mapped: bool) {
        *self.is_mapped_compositor.lock().unwrap() = mapped;
        // Potentially trigger damage calculations or notifications here or in the caller.
    }

    /// Checks the compositor-side mapped state.
    pub fn is_mapped_by_compositor(&self) -> bool {
        *self.is_mapped_compositor.lock().unwrap()
    }

    /// Updates the current geometry of the window.
    pub fn set_geometry(&self, geometry: Rectangle<i32, Logical>) {
        *self.current_geometry.lock().unwrap() = geometry;
    }

    /// Calculates the popup geometry based on its positioner and parent.
    /// This is a simplified version. Smithay's `PopupManager` handles more complex cases.
    pub fn calculate_popup_geometry(
        popup_surface: &PopupSurface,
        parent_window: &ManagedWindow,
    ) -> Rectangle<i32, Logical> {
        let popup_wl_surface = popup_surface.wl_surface();
        let parent_geometry = *parent_window.current_geometry.lock().unwrap();

        let positioner = popup_surface.user_data().get::<PositionerState>().unwrap(); // Smithay attaches this

        // Get surface content size (if available, otherwise use (0,0))
        let surface_content_size = surface_management::with_surface_data_mut_direct(popup_wl_surface, |data| {
            if let Some(buffer_info) = data.current_buffer_info.lock().unwrap().as_ref() {
                // This needs buffer dimensions, which are not directly in AttachedBufferInfo.
                // We'd need to access wl_buffer's data or renderer texture info.
                // For now, let's use a placeholder or assume size comes from positioner.
                // Smithay's PopupManager uses the buffer dimensions.
                // Placeholder:
                // if let Some(texture) = data.texture_handle.lock().unwrap().as_ref() {
                //     return Size::from((texture.width() as i32, texture.height() as i32));
                // }
            }
            Size::from((0,0)) // Fallback if no buffer/texture info
        });


        let rect = positioner.get_effective_rectangle(surface_content_size);


        // Position relative to parent surface geometry
        let popup_location = parent_geometry.loc + rect.loc;
        Rectangle::from_loc_and_size(popup_location, rect.size)
    }
}

impl Window for ManagedWindow {
    fn id(&self) -> usize {
        // Using a hash of the internal_id for the usize ID required by Space.
        // This is generally okay for Space's internal hashing but ensure it's
        // what you expect if you use this ID elsewhere.
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.internal_id.hash(&mut hasher);
        std::hash::Hasher::finish(&hasher) as usize
    }

    fn surface_type(&self) -> WindowSurfaceType {
        match self.surface_variant {
            XdgSurfaceVariant::Toplevel(_) => WindowSurfaceType::Toplevel,
            XdgSurfaceVariant::Popup(_) => WindowSurfaceType::Popup,
        }
    }

    fn wl_surface(&self) -> &wl_surface::WlSurface {
        self.surface_variant.wl_surface()
    }

    fn geometry(&self) -> Rectangle<i32, Logical> {
        // This should be the geometry *as seen by the client* (i.e., its "bounds").
        // For toplevels, it's their main geometry. For popups, it's relative to their parent.
        // Smithay's Space will offset this by the window's location in the space.
        // The `current_geometry` in `ManagedWindow` is intended to be the final screen geometry.
        // This might need refinement based on how Space uses `geometry()`.
        // Typically, for a toplevel, this would be `(0,0)` with `size = configured_size`.
        // The `Space` then places this rectangle at `window_location`.
        // Let's return a zero-origin rectangle with the configured size.
        let size = match &self.surface_variant {
            XdgSurfaceVariant::Toplevel(t) => {
                t.current_state().size.unwrap_or_else(|| *self.requested_size.lock().unwrap())
            }
            XdgSurfaceVariant::Popup(p) => {
                // For popups, geometry is more complex and often derived from the buffer.
                // Smithay's PopupManager calculates this. Here, we might need to refer
                // to the geometry calculated when the popup was mapped.
                // For now, use requested_size (which is likely (0,0) for popups initially)
                // or current_geometry's size.
                self.current_geometry.lock().unwrap().size
            }
        };
        Rectangle::from_loc_and_size(Point::from((0,0)), size)
    }

    fn is_mapped(&self) -> bool {
        // This method should reflect if the client considers the surface mapped.
        // Smithay's XDG shell types (`ToplevelSurface`, `PopupSurface`) have `is_mapped()`.
        match &self.surface_variant {
            XdgSurfaceVariant::Toplevel(t) => t.is_mapped(),
            XdgSurfaceVariant::Popup(p) => p.is_mapped(),
        }
    }

    fn is_suspended(&self) -> bool {
        // Typically relevant for dmabuf surfaces, can be false for SHM.
        // Or if the compositor explicitly suspends rendering for a window.
        surface_management::with_surface_data_mut_direct(self.wl_surface(), |data| {
            // Example: check if texture_handle is None while buffer is attached
            data.texture_handle.lock().unwrap().is_none() && data.current_buffer_info.lock().unwrap().is_some()
        }).unwrap_or(false) // Default to false if SurfaceData is somehow inaccessible
    }

    fn send_frame(&self, time: u32) {
        // Delegate to the underlying wl_surface to send the frame event.
        if let Err(err) = self.wl_surface().send_frame(time) {
            tracing::warn!(
                "Failed to send frame for surface {:?}: {}",
                self.wl_surface().id(),
                err
            );
        }
    }

    fn self_update(&mut self) {
        // This method is called by Space for tasks like updating internal state
        // or animations. For now, it's a no-op.
    }

    fn send_configure(&mut self) {
        // This is called by Space when it thinks the window should reconfigure.
        // We should send the appropriate XDG configure event.
        match &self.surface_variant {
            XdgSurfaceVariant::Toplevel(toplevel_surface) => {
                let xdg_toplevel_data = toplevel_surface.wl_surface().get_data::<XdgToplevelSurfaceData>().unwrap();
                xdg_toplevel_data.send_configure(); // Sends based on its current pending state
                tracing::debug!("ManagedWindow::send_configure sent toplevel configure for {:?}", self.internal_id);
            }
            XdgSurfaceVariant::Popup(popup_surface) => {
                // Popups are configured with their parent. This might be tricky to call from here
                // without access to the parent WlSurface directly.
                // The XdgPopupSurfaceData::send_configure requires the parent.
                // This might be better handled directly in the XdgShellHandler when a reconfigure is needed.
                tracing::warn!("ManagedWindow::send_configure for Popup is a no-op. Should be handled by XdgShellHandler.");
                // If we had parent WlSurface:
                // if let Some(parent_wl_surface) = find_parent_wl_surface_somehow() {
                //     let xdg_popup_data = popup_surface.wl_surface().get_data::<XdgPopupSurfaceData>().unwrap();
                //     xdg_popup_data.send_configure(&parent_wl_surface);
                // }
            }
        }
    }

    fn z_index(&self) -> u8 {
        // Determine z-index based on surface type. Popups are above toplevels.
        // Further refinement can be done for different types of toplevels (e.g., modals).
        match self.surface_variant {
            XdgSurfaceVariant::Toplevel(_) => 0, // Base layer for toplevels
            XdgSurfaceVariant::Popup(_) => 10,   // Popups above toplevels
        }
    }

    // Smithay 0.3+ requires these methods for more advanced window management features:
    fn set_activated(&self, _activated: bool) -> bool {
        // TODO: Implement activation logic (e.g., visual cues, raise window)
        // This involves potentially focusing the seat on this window's surface.
        // Return true if activation state changed.
        tracing::debug!("ManagedWindow::set_activated called for {:?}, new state: {}", self.internal_id, _activated);
        // This should interact with seat focus. For now, just log.
        // Example: self.wl_surface().set_client_focused(activated);
        // And update visual state.
        false // Placeholder
    }

    fn damage_output(&self, _output_name: &str, _scale: f64, _space: &Space<Self>, _location: Point<i32, Physical>, _damage: Rectangle<i32, Physical>) {
        // This is called by Space when it determines an output needs to be damaged due to this window.
        // This is more relevant for hardware rendering where you might manage per-output damage.
        // For now, we can log it. If using software rendering with full repaint, this might be a no-op.
        tracing::trace!(
            "ManagedWindow {:?}: damage_output called for output '{}', location {:?}, damage {:?}",
            self.internal_id, _output_name, _location, _damage
        );
        // In a real scenario, you'd pass this damage to your renderer for the specific output.
    }

    // Optional methods from Window trait (Smithay 0.2 style, might be different in 0.3+)
    // These might not be strictly necessary if using newer Space APIs or if not using
    // certain features like explicit window stacking or specific input grabbing.

    // fn on_commit(&self) {
    //     // Called when the underlying wl_surface is committed.
    //     // This is where you might update textures or cached state from SurfaceData.
    //     // The main commit handling is in CompositorHandler::commit, which then updates SurfaceData.
    //     // This callback on Window could be for Space-specific updates after a commit.
    // }

    // fn can_receive_pointer_input(&self) -> bool { true }
    // fn can_receive_keyboard_input(&self) -> bool { true }
}

// Helper to get XdgToplevelSurfaceData from ManagedWindow if it's a toplevel
impl ManagedWindow {
    pub fn toplevel_data(&self) -> Option<&XdgToplevelSurfaceData> {
        match &self.surface_variant {
            XdgSurfaceVariant::Toplevel(t) => t.wl_surface().get_data::<XdgToplevelSurfaceData>().ok(),
            _ => None,
        }
    }
}
