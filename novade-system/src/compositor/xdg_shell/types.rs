use smithay::{
    desktop::{Window, WindowSurfaceType, Space}, // Added Space for potential future use with Window
    reexports::wayland_server::{
        protocol::wl_surface::WlSurface,
        Resource, // For WlSurface.id()
    },
    utils::{Logical, Point, Rectangle, Size, Serial, Transform}, // Added Transform
    wayland::{
        compositor::SurfaceData as SmithaySurfaceData, // To check surface aliveness
        shell::xdg::{PopupSurface, ToplevelSurface, WindowSurface, XdgToplevelSurfaceData, ToplevelState as XdgToplevelState},
        seat::WaylandFocus, // For wl_surface() return type Option<&WlSurface>
    },
    wayland::presentation::presentation_time, // Correct path for Smithay 0.10
};
use std::{
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, Weak}, // Added Weak for parent reference
};
use uuid::Uuid;

// Placeholder for Domain WindowIdentifier
// In a real system, this would come from `crate::core::types` or similar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct DomainWindowIdentifier(Uuid);

impl DomainWindowIdentifier {
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug)]
pub struct ManagedWindow {
    pub id: Uuid, // Internal compositor ID
    pub domain_id: DomainWindowIdentifier,
    pub xdg_surface: WindowSurface, // Toplevel or Popup
    pub current_geometry: Rectangle<i32, Logical>,
    pub is_mapped: bool,
    // For popups, a link to the parent ManagedWindow might be useful
    pub parent: Option<Weak<ManagedWindow>>, // Weak reference to avoid cycles if popups store parent
    // Fields for title and app_id
    pub title: Option<String>,
    pub app_id: Option<String>,
    // Storing pending configure serial for ack_configure validation
    pub last_configure_serial: Option<Serial>,
}

impl ManagedWindow {
    pub fn new_toplevel(toplevel_surface: ToplevelSurface, domain_id: DomainWindowIdentifier) -> Self {
        let initial_geometry = Rectangle::from_loc_and_size((0, 0), (0, 0)); // Initialized later

        // Update title and app_id from the toplevel surface's initial state if available
        let title = toplevel_surface.title();
        let app_id = toplevel_surface.app_id();

        Self {
            id: Uuid::new_v4(),
            domain_id,
            xdg_surface: WindowSurface::Toplevel(toplevel_surface),
            current_geometry: initial_geometry,
            is_mapped: false,
            parent: None,
            title,
            app_id,
            last_configure_serial: None,
        }
    }

    pub fn new_popup(popup_surface: PopupSurface, parent_domain_id: DomainWindowIdentifier, parent_window: Option<Arc<ManagedWindow>>) -> Self {
        // Popups get geometry relative to their parent.
        // For this basic implementation, we'll set a zero geometry.
        // Full popup logic (positioning, grab, etc.) is complex.
        let title = popup_surface.wl_surface().data_map().get::<XdgToplevelSurfaceData>().and_then(|d| d.title.clone());
        let app_id = popup_surface.wl_surface().data_map().get::<XdgToplevelSurfaceData>().and_then(|d| d.app_id.clone());

        Self {
            id: Uuid::new_v4(),
            domain_id: DomainWindowIdentifier::new_v4(), // Popups might have their own domain ID or be linked differently.
            xdg_surface: WindowSurface::Popup(popup_surface),
            current_geometry: Rectangle::from_loc_and_size((0, 0), (0, 0)),
            is_mapped: false, // Popups are mapped based on parent state or explicit grab
            parent: parent_window.map(|p| Arc::downgrade(&p)),
            title,
            app_id,
            last_configure_serial: None,
        }
    }

    pub fn wl_surface_ref(&self) -> Option<&WlSurface> {
        self.xdg_surface.wl_surface().as_ref()
    }
}

impl PartialEq for ManagedWindow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for ManagedWindow {}

impl Hash for ManagedWindow {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// --- smithay::desktop::Window Trait Implementation ---
impl Window for ManagedWindow {
    fn id(&self) -> usize {
        // Using a stable hash of the Uuid for the usize ID required by Space.
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.id.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn wl_surface(&self) -> Option<WlSurface> {
        Some(self.xdg_surface.wl_surface().clone())
    }
    
    fn focus_surface(&self) -> Option<&WlSurface> {
        self.wl_surface_ref()
    }

    fn surface_type(&self) -> WindowSurfaceType {
        self.xdg_surface.surface_type()
    }

    fn geometry(&self) -> Rectangle<i32, Logical> {
        self.current_geometry
    }

    fn is_mapped(&self) -> bool {
        self.is_mapped && self.xdg_surface.alive()
    }
    
    fn is_suspended(&self) -> bool {
        self.xdg_surface.toplevel().map_or(false, |t| {
            let current_states = t.current_states();
            current_states.contains(XdgToplevelState::Suspended) || current_states.contains(XdgToplevelState::Minimized)
        })
    }

    fn send_frame(&self, time: u32, throttle_harming_region: Option<Rectangle<i32, Logical>>) {
        if let Some(surface) = self.xdg_surface.wl_surface().as_ref() {
            if surface.alive() { // Check if surface is still alive before sending frame
                presentation_time::send_frames_surface_dest_harming_region_transform(
                    surface,
                    time,
                    throttle_harming_region,
                    Transform::Normal, // Assuming normal transform for now
                );
            }
        }
    }
    
    fn send_configure(&mut self) {
        match &self.xdg_surface {
            WindowSurface::Toplevel(toplevel) => {
                // Smithay 0.10.0: configure() returns the serial
                let serial = toplevel.send_configure();
                self.last_configure_serial = Some(serial);
            }
            WindowSurface::Popup(popup) => {
                let serial = popup.send_configure();
                self.last_configure_serial = Some(serial);
            }
        }
    }

    fn requested_extents(&self) -> Option<Rectangle<i32, Logical>> {
        None // Can be implemented if client-side decorations or specific extent requests are handled.
    }
    
    fn user_data(&self) -> &smithay::utils::UserDataMap {
        self.xdg_surface.user_data()
    }

    fn self_update(&mut self) {
        // This method allows the window to synchronize its state with the underlying XDG surface.
        // For example, update title or app_id if they changed on the XDG surface.
        if let WindowSurface::Toplevel(toplevel) = &self.xdg_surface {
            self.title = toplevel.title();
            self.app_id = toplevel.app_id();
            // Could also check current_states() here if needed for other properties.
        }
        // Popups don't typically have titles/app_ids in the same way.
    }

    // Optional methods from Window trait, can be default or implemented:
    // fn on_commit(&mut self) { /* ... */ }
    // fn damage_applied(&mut self) { /* ... */ }
    // fn is_solid(&self) -> bool { false } 
    // fn z_index(&self) -> u8 { 0 } 
}
