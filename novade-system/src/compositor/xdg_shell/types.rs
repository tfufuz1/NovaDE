use smithay::{
    desktop::{Window, WindowSurfaceType, Space}, // Added Space for potential future use with Window
    reexports::wayland_server::{
        protocol::wl_surface::WlSurface,
        Resource, // For WlSurface.id()
    },
    utils::{Logical, Point, Rectangle, Size, Serial, Transform}, // Added Transform
    wayland::{
        compositor::SurfaceData as SmithaySurfaceData, // To check surface aliveness
        shell::xdg::{PopupSurface, ToplevelSurface, WindowSurface, XdgToplevelSurfaceData, ToplevelState as XdgToplevelState, ResizeEdge},
        seat::WaylandFocus, // For wl_surface() return type Option<&WlSurface>
    },
    wayland::presentation::presentation_time, // Correct path for Smithay 0.10
};
use std::{
    hash::{Hash, Hasher},
    sync::{Arc, RwLock, Weak}, // Added RwLock, Mutex removed as RwLock is generally preferred
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

/// Window state
#[derive(Debug, Clone)]
pub struct WindowState {
    /// Is the window maximized
    pub maximized: bool,
    
    /// Is the window fullscreen
    pub fullscreen: bool,
    
    /// Is the window minimized
    pub minimized: bool,
    
    /// Is the window activated (has focus)
    pub activated: bool,
    
    /// Window geometry (requested by client or set by compositor)
    pub geometry: Option<Rectangle<i32, Logical>>,
    
    /// Window position (current position on screen)
    pub position: Point<i32, Logical>,
    
    /// Window size (current size on screen)
    pub size: Size<i32, Logical>,
    
    /// Window minimum size
    pub min_size: Size<i32, Logical>,
    
    /// Window maximum size
    pub max_size: Size<i32, Logical>,

    /// Saved geometry before a maximize or fullscreen action
    pub saved_pre_action_geometry: Option<Rectangle<i32, Logical>>,
}

/// Window manager data, specific to compositor's internal state management
#[derive(Debug, Clone)]
pub struct WindowManagerData {
    /// Is the window being moved
    pub moving: bool,
    
    /// Is the window being resized
    pub resizing: bool,
    
    /// Resize edges
    pub resize_edges: Option<ResizeEdge>,
    
    /// Window workspace
    pub workspace: u32, // Or Option<u32> if a window might not be on any workspace
    
    /// Window layer
    pub layer: WindowLayer,
    
    /// Window opacity
    pub opacity: f64,
    
    /// Window z-index (relative to other windows in the same layer/space)
    pub z_index: i32,
    
    /// Window decorations state (e.g. server-side or client-side)
    pub decorations: bool, // true for server-side, false for client-side
}

/// Window layer for stacking order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)] // Added PartialOrd, Ord for easy sorting/comparison
pub enum WindowLayer {
    /// Background layer (e.g., wallpapers)
    Background,
    
    /// Bottom layer (e.g., docks, panels that should be below normal windows)
    Bottom,
    
    /// Normal window layer
    Normal,
    
    /// Top layer (e.g., panels, notifications that should be above normal windows)
    Top,
    
    /// Overlay layer (e.g., screen lockers, critical alerts, input method popups)
    Overlay,
}

#[derive(Debug)]
pub struct ManagedWindow {
    pub id: Uuid, // Internal compositor ID
    pub domain_id: DomainWindowIdentifier,
    pub xdg_surface: WindowSurface, // Toplevel or Popup
    pub current_geometry: Rectangle<i32, Logical>,
    pub is_mapped: bool,
    pub parent: Option<Weak<ManagedWindow>>,
    // Fields for title and app_id (direct, as per existing ManagedWindow in types.rs)
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub last_configure_serial: Option<Serial>,
    // Added fields from xdg_shell/mod.rs's ManagedWindow
    pub state: Arc<RwLock<WindowState>>,
    pub manager_data: Arc<RwLock<WindowManagerData>>,
}

impl ManagedWindow {
    pub fn new_toplevel(toplevel_surface: ToplevelSurface, domain_id: DomainWindowIdentifier) -> Self {
        let initial_geometry = Rectangle::from_loc_and_size((0, 0), (0, 0));
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
            state: Arc::new(RwLock::new(WindowState {
                maximized: false,
                fullscreen: false,
                minimized: false,
                activated: false, // Should be set true when focused
                geometry: None, // Client's requested geometry
                position: Point::from((100, 100)), // Default initial position
                size: Size::from((300, 200)),    // Default initial size
                min_size: Size::from((1, 1)),
                max_size: Size::from((0, 0)), // 0 means unlimited
                saved_pre_action_geometry: None,
            })),
            manager_data: Arc::new(RwLock::new(WindowManagerData {
                moving: false,
                resizing: false,
                resize_edges: None,
                workspace: 0, // Default workspace
                layer: WindowLayer::Normal,
                opacity: 1.0,
                z_index: 0,
                decorations: true, // Default to SSD
            })),
        }
    }

    pub fn new_popup(popup_surface: PopupSurface, _parent_domain_id: DomainWindowIdentifier, parent_window: Option<Arc<ManagedWindow>>) -> Self {
        let title = popup_surface.wl_surface().data_map().get::<XdgToplevelSurfaceData>().and_then(|d| d.title.clone());
        let app_id = popup_surface.wl_surface().data_map().get::<XdgToplevelSurfaceData>().and_then(|d| d.app_id.clone());

        Self {
            id: Uuid::new_v4(),
            // Popups might share parent's domain_id or have a new one. For now, new.
            domain_id: DomainWindowIdentifier::new_v4(), 
            xdg_surface: WindowSurface::Popup(popup_surface),
            current_geometry: Rectangle::from_loc_and_size((0, 0), (0, 0)),
            is_mapped: false,
            parent: parent_window.map(|p| Arc::downgrade(&p)),
            title,
            app_id,
            last_configure_serial: None,
            // Popups usually don't have complex state/manager_data like toplevels,
            // but initializing them for consistency.
            state: Arc::new(RwLock::new(WindowState {
                maximized: false, fullscreen: false, minimized: false, activated: false,
                geometry: None, position: Point::from((0,0)), size: Size::from((0,0)),
                min_size: Size::from((0,0)), max_size: Size::from((0,0)),
                saved_pre_action_geometry: None,
            })),
            manager_data: Arc::new(RwLock::new(WindowManagerData {
                moving: false, resizing: false, resize_edges: None, workspace: 0,
                layer: WindowLayer::Overlay, // Popups are often overlays
                opacity: 1.0, z_index: 0, decorations: false, // Popups don't have decorations
            })),
        }
    }

    pub fn wl_surface_ref(&self) -> Option<&WlSurface> {
        self.xdg_surface.wl_surface().as_ref()
    }

    // Helper methods to access interior mutability, if needed for handlers
    // Example:
    // pub fn with_state<F, R>(&self, func: F) -> R where F: FnOnce(&mut WindowState) -> R {
    //     let mut guard = self.state.write().unwrap();
    //     func(&mut *guard)
    // }
    // pub fn with_manager_data<F, R>(&self, func: F) -> R where F: FnOnce(&mut WindowManagerData) -> R {
    //     let mut guard = self.manager_data.write().unwrap();
    //     func(&mut *guard)
    // }
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

    fn on_commit(&mut self) {
        self.self_update();
    }

    // Optional methods from Window trait, can be default or implemented:
    // fn damage_applied(&mut self) { /* ... */ }
    // fn is_solid(&self) -> bool { false } 
    // fn z_index(&self) -> u8 { 0 } 
}
