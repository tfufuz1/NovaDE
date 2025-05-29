use smithay::{
    desktop::{Window, WindowSurfaceType, WindowSurface as SmithayWindowSurfaceEnum, ResizeEdge as SmithayResizeEdge},
    reexports::wayland_server::{
        protocol::{wl_surface::WlSurface, wl_seat::WlSeat}, // WlSeat not directly used here but common in shell modules
        Serial, DisplayHandle,
    },
    utils::{Logical, Point, Rectangle, Size},
    wayland::{
        compositor::SurfaceData as SmithayCoreSurfaceData,
        seat::Seat,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, SurfaceCachedState,
            Configure as XdgConfigure,
            xdg_toplevel::{State as XdgToplevelState, ResizeEdge as XdgResizeEdge, WmCapabilities as XdgWmCapabilities},
            xdg_popup::PopupGrabMisbehavior,
            decoration::ssd::Mode as SsdMode,
            XdgToplevelSurfaceData, // This is the actual Smithay type we'd expect to use for user_data
        },
        presentation, // For send_frames_surface_destroying
        Serials,
    },
};
use std::{
    hash::{Hash, Hasher},
    sync::{Arc, Mutex}, // Added Mutex
    time::Duration,
};
use uuid::Uuid;

// Domain types
use novade_domain::common_types::DomainWindowIdentifier;

// Note: The prompt for `is_suspended` mentions `XdgToplevelSurfaceData`
// (e.g., `t.user_data().get::<XdgToplevelSurfaceData>()`). Smithay's `ToplevelSurface`
// has `user_data()` that can store any type. If we are to store custom flags like
// `minimized` or `suspended` (compositor-specific states not in XDG protocol),
// we would define our own struct (e.g., `OurToplevelUserData`) and store that.
// Smithay's `XdgToplevelSurfaceData` is for its internal XDG protocol state.
// For this implementation, I will assume `XdgToplevelSurfaceData` is used as a placeholder
// for where one *might* get such custom flags if they were part of Smithay's provided struct,
// or that the prompt implies we should define our own struct with these fields and store it.
// To keep it simple and use existing Smithay types as much as possible for now,
// I will rely only on `XdgToplevelState::Suspended` from the protocol for `is_suspended`.
// If custom `minimized` or `suspended` flags are truly needed from user_data,
// a dedicated struct for that user_data would be required.
// Let's use a temporary placeholder if the spec insists on custom flags from user_data.

use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

#[derive(Debug, Clone)] // Cannot derive Default easily with AtomicBool, implement manually or use default values in new_toplevel
pub struct CustomToplevelState { 
    pub minimized: Arc<AtomicBool>, // Changed to Arc<AtomicBool> for shared mutability
    pub custom_suspended: Arc<AtomicBool>, // Changed for consistency
}

impl CustomToplevelState {
    pub fn new() -> Self {
        Self {
            minimized: Arc::new(AtomicBool::new(false)),
            custom_suspended: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for CustomToplevelState {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone)]
pub struct ManagedWindow {
    pub id: Uuid,
    pub domain_id: DomainWindowIdentifier,
    pub xdg_surface: SmithayWindowSurfaceEnum,
    pub current_geometry: Mutex<Rectangle<i32, Logical>>, // Changed to Mutex
    pub is_mapped: bool,
    pub parent_id: Mutex<Option<Uuid>>, // Added parent_id field
}

impl ManagedWindow {
    pub fn new_toplevel(toplevel_surface: ToplevelSurface, domain_id: DomainWindowIdentifier) -> Self {
        let initial_geometry = toplevel_surface.geometry();
        // Initialize custom state for the toplevel surface's user_data if not already present
        toplevel_surface.user_data().insert_if_missing_threadsafe(CustomToplevelState::default);

        Self {
            id: Uuid::new_v4(),
            domain_id,
            xdg_surface: SmithayWindowSurfaceEnum::Wayland(toplevel_surface),
            current_geometry: Mutex::new(initial_geometry), // Initialize Mutex
            is_mapped: false,
            parent_id: Mutex::new(None), // Initialize parent_id
        }
    }

    pub fn new_popup(popup_surface: PopupSurface, parent_managed_window_id: Option<Uuid>) -> Self {
        let initial_geometry = popup_surface.geometry();
        Self {
            id: Uuid::new_v4(),
            domain_id: DomainWindowIdentifier::new_popup(Uuid::new_v4()), // Simplified domain ID for popups
            xdg_surface: SmithayWindowSurfaceEnum::WaylandPopup(popup_surface),
            current_geometry: Mutex::new(initial_geometry), // Initialize Mutex
            is_mapped: false,
            parent_id: Mutex::new(parent_managed_window_id), // Initialize parent_id
        }
    }

    // Helper to access the underlying ToplevelSurface if this window is a toplevel
    fn toplevel(&self) -> Option<&ToplevelSurface> {
        match &self.xdg_surface {
            SmithayWindowSurfaceEnum::Wayland(s) => Some(s),
            _ => None,
        }
    }

    // Helper to access the underlying PopupSurface if this window is a popup
    fn popup(&self) -> Option<&PopupSurface> {
        match &self.xdg_surface {
            SmithayWindowSurfaceEnum::WaylandPopup(s) => Some(s),
            _ => None,
        }
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

impl Window for ManagedWindow {
    fn id(&self) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.id.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn surface_type(&self) -> WindowSurfaceType {
        self.xdg_surface.surface_type()
    }

    fn wl_surface(&self) -> Option<WlSurface> {
        self.xdg_surface.wl_surface()
    }

    fn geometry(&self) -> Rectangle<i32, Logical> {
        *self.current_geometry.lock().unwrap() // Lock and clone/copy
    }

    fn is_mapped(&self) -> bool {
        self.is_mapped && self.xdg_surface.alive()
    }

    fn is_suspended(&self) -> bool {
        self.toplevel().map_or(false, |t| {
            let protocol_suspended = t.current_states().contains(XdgToplevelState::Suspended);
            
            let custom_flags_suspended = t.user_data().get::<CustomToplevelState>().map_or(false, |cs| {
                cs.minimized.load(AtomicOrdering::Relaxed) || cs.custom_suspended.load(AtomicOrdering::Relaxed)
            });

            protocol_suspended || custom_flags_suspended
        })
    }

    fn send_frame(&self, time: impl Into<Duration> + Copy) {
        // As per prompt: Delegate to smithay::wayland::presentation::send_frames_surface_destroying
        if let Some(surface) = self.wl_surface() {
            // This function is intended for when a surface is being destroyed to send final frame events.
            // For regular frame callbacks (wl_surface.frame), SmithayCoreSurfaceData::send_frame is more typical.
            // Following prompt literally:
            presentation::send_frames_surface_destroying(&surface, time.into());
            // TODO: Verify if this is the intended general frame callback mechanism for the Window trait.
            // It's plausible if `Window::send_frame` is only called by the compositor before destroying
            // a mapped surface that was part of the rendering, to ensure client gets final presentation feedback.
            // If it's for *every* frame rendered for an active window, SmithayCoreSurfaceData::send_frame is better.
        }
    }

    fn send_configure(&mut self) {
        match &self.xdg_surface {
            SmithayWindowSurfaceEnum::Wayland(toplevel) => {
                toplevel.send_configure();
            }
            SmithayWindowSurfaceEnum::WaylandPopup(popup) => {
                popup.send_configure();
            }
            _ => {}
        }
    }

    fn set_activated(&mut self, activated: bool) -> bool {
        if let Some(toplevel) = self.toplevel() {
            toplevel.with_pending_state(|state| {
                if activated {
                    state.states.set(XdgToplevelState::Activated);
                } else {
                    state.states.unset(XdgToplevelState::Activated);
                }
            });
            self.send_configure(); // Call send_configure as per prompt
            true
        } else {
            false
        }
    }

    fn set_size_request(&mut self, size: Size<i32, Logical>) {
        if let Some(toplevel) = self.toplevel() {
            toplevel.with_pending_state(|state| {
                state.size = Some(size);
            });
            self.send_configure();
        }
    }
    
    fn set_min_size(&mut self, min_size: Option<Size<i32, Logical>>) {
        if let Some(toplevel) = self.toplevel() {
            toplevel.with_pending_state(|state| {
                state.min_size = min_size;
            });
            self.send_configure();
        }
    }

    fn set_max_size(&mut self, max_size: Option<Size<i32, Logical>>) {
        if let Some(toplevel) = self.toplevel() {
            toplevel.with_pending_state(|state| {
                state.max_size = max_size;
            });
            self.send_configure();
        }
    }

    fn set_maximized(&mut self, maximized: bool) {
        if let Some(toplevel) = self.toplevel() {
            toplevel.with_pending_state(|state| {
                if maximized {
                    state.states.set(XdgToplevelState::Maximized);
                } else {
                    state.states.unset(XdgToplevelState::Maximized);
                }
            });
            self.send_configure();
        }
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        if let Some(toplevel) = self.toplevel() {
            toplevel.with_pending_state(|state| {
                if fullscreen {
                    state.states.set(XdgToplevelState::Fullscreen);
                } else {
                    state.states.unset(XdgToplevelState::Fullscreen);
                }
            });
            self.send_configure();
        }
    }
    
    fn set_minimized(&mut self, minimized: bool) {
        if let Some(toplevel) = self.toplevel() {
            // Ensure CustomToplevelState exists, then update its atomic flags.
            toplevel.user_data().insert_if_missing_threadsafe(CustomToplevelState::default);
            if let Some(custom_state) = toplevel.user_data().get::<CustomToplevelState>() {
                custom_state.minimized.store(minimized, AtomicOrdering::Relaxed);
                tracing::info!("ManagedWindow: set_minimized. Custom state 'minimized' stored as {} for toplevel ID {:?}", minimized, self.id);
                // If minimized also implies custom_suspended, set that too:
                // custom_state.custom_suspended.store(minimized, AtomicOrdering::Relaxed);
            } else {
                // This case should ideally not be reached if insert_if_missing_threadsafe works as expected.
                tracing::error!("ManagedWindow: CustomToplevelState not found after attempting insertion for toplevel ID {:?}.", self.id);
            }
            // No XDG state for minimized, so no send_configure() from ManagedWindow.
            // The XdgShellHandler::toplevel_request_set_minimized will handle compositor actions (like unmapping).
        }
    }

    fn set_tiled(&mut self, tiled_edges: &[SmithayResizeEdge]) {
         if let Some(toplevel) = self.toplevel() {
            toplevel.with_pending_state(|state| {
                // Clear existing tiled states
                state.states.remove(XdgToplevelState::TiledLeft);
                state.states.remove(XdgToplevelState::TiledRight);
                state.states.remove(XdgToplevelState::TiledTop);
                state.states.remove(XdgToplevelState::TiledBottom);

                for edge in tiled_edges {
                    match edge {
                        SmithayResizeEdge::Left => state.states.set(XdgToplevelState::TiledLeft),
                        SmithayResizeEdge::Right => state.states.set(XdgToplevelState::TiledRight),
                        SmithayResizeEdge::Top => state.states.set(XdgToplevelState::TiledTop),
                        SmithayResizeEdge::Bottom => state.states.set(XdgToplevelState::TiledBottom),
                        _ => {} // Other edges not directly mappable to XDG tiled states
                    }
                }
            });
            self.send_configure();
        }
    }

    fn parent(&self) -> Option<Arc<dyn Window>> {
        if self.popup().and_then(|p| p.parent()).is_some() {
             tracing::warn!("ManagedWindow::parent(): Parent lookup from WlSurface to Arc<ManagedWindow> is not implemented here. This requires access to global window list/map.");
        }
        None
    }
    
    fn user_data(&self) -> &smithay::reexports::wayland_server::UserDataMap {
        self.xdg_surface.user_data()
    }

    fn send_done(&self) {
        if let Some(popup) = self.popup() {
            popup.send_done();
        }
    }

    fn set_ssd_decorations(&mut self, decorations: SsdMode) {
        if let Some(toplevel) = self.toplevel() {
            toplevel.with_pending_state(|state| {
                state.decoration_mode = Some(decorations);
            });
            self.send_configure();
        }
    }
}
