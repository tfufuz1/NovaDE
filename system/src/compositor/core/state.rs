use crate::compositor::renderer_interface::abstraction::FrameRenderer;
use smithay::{
    backend::{
        drm::{DrmDevice, DrmDisplay, DrmNode, DrmSurface},
        egl::EGLDisplay,
        session::Session,
    },
    delegate_compositor, delegate_shm,
    desktop::{Space, Window, WindowSurfaceType, PopupManager}, // Added PopupManager
    reexports::{
        calloop::{generic::Generic, Interest, LoopHandle, LoopSignal, Mode},
        wayland_server::{
            protocol::{wl_compositor, wl_shm, wl_subcompositor, wl_surface::WlSurface}, // Added WlSurface
            Client, DisplayHandle, GlobalDispatch, GlobalId, // Added GlobalId
            ClientHandler as WaylandClientHandler, ClientId,
        },
    },
    wayland::{
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        shm::{ShmHandler, ShmState},
        buffer::BufferHandler,
        shell::xdg::{XdgShellState, XdgShellHandler, XdgToplevelSurfaceData, XdgPopupSurfaceData, ToplevelSurface, PopupSurface, XdgWmBaseClientData, XdgPositionerUserData, XdgSurfaceUserData, XdgActivationState, XdgActivationHandler},
        seat::Seat,
    },
    input::{SeatHandler, SeatState, pointer::CursorImageStatus, keyboard::KeyboardHandle, touch::TouchHandle, TouchSlot}, // Use SeatHandler directly, added TouchHandle and TouchSlot
    utils::{Point, Size, Rectangle, Logical, Physical, Serial, Transform, Clock}, // Added Clock
};
use smithay::wayland::tablet_manager::{TabletManagerState, TabletManagerHandler, TabletSeatTrait}; // ADDED for tablet support
use smithay::wayland::pointer_constraints::{ // ADDED for pointer constraints
    PointerConstraintsState, PointerConstraintsHandler, PointerConstraint,
    LockedPointerData, ConfinedPointerData, ConstraintState
};
use smithay::reexports::wayland_protocols::wp::pointer_constraints::zv1::server::{
    zwp_pointer_constraints_v1, zwp_locked_pointer_v1, zwp_confined_pointer_v1
};
use smithay::reexports::wayland_server::{Resource, New, DataInit, Dispatch}; // For GlobalDispatch & Dispatch
use std::{time::Duration, collections::HashMap, sync::{Arc, Mutex, Weak}, env}; // Mutex is already here, Added env
// Ensure wl_surface is specifically available if not covered by wildcard
use smithay::reexports::wayland_server::protocol::wl_surface;
use smithay::wayland::cursor::{CursorTheme, load_theme}; // ADDED for themed cursors
use smithay::wayland::compositor::CompositorToken; // For create_surface
use uuid::Uuid;
use crate::{
    compositor::{
        shm::ShmError,
        xdg_shell::types::ManagedWindow,
        display_loop::client_data::ClientData,
    },
    input::{keyboard::XkbKeyboardData}, // Removed TouchFocusData
    outputs::manager::OutputManager,
};
use smithay::{
    reexports::wayland_protocols::xdg::xdg_output::server::zxdg_output_manager_v1::ZxdgOutputManagerV1, // For GlobalDispatch
    wayland::output::{OutputManagerState, OutputHandler, OutputData}, // For OutputHandler
    reexports::wayland_server::protocol::wl_output, // For OutputHandler
};

/// Data associated with each client's compositor state.
#[derive(Debug, Clone)]
pub struct ClientCompositorData {
    // Smithay examples often use `ClientState` which is a more general store.
    // For now, let's keep it simple or use Smithay's CompositorClientState directly if sufficient.
    // If we need custom client-specific compositor data, we can add fields here.
    // For example, if a client could have specific capabilities or restrictions.
    _placeholder: (), // Replace with actual fields if needed
}

impl Default for ClientCompositorData {
    fn default() -> Self {
        Self { _placeholder: () }
    }
}

/// The global state for the compositor.
#[derive(Debug)]
pub struct DesktopState {
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, Self>, // 'static might need adjustment based on main loop
    pub loop_signal: LoopSignal,
    pub clock: Clock<u64>, // ADDED/UNCOMMENTED

    // Smithay states
    pub compositor_state: CompositorState,
    pub shm_state: ShmState,
    pub xdg_shell_state: XdgShellState, // Added XDG Shell State
    pub xdg_activation_state: XdgActivationState,

    pub space: Space<ManagedWindow>,
    pub windows: HashMap<crate::compositor::xdg_shell::types::DomainWindowIdentifier, Arc<ManagedWindow>>,

    pub shm_global: Option<GlobalId>,
    pub xdg_shell_global: Option<GlobalId>,
    pub xdg_activation_global: Option<GlobalId>,

    // --- Input Related State ---
    pub seat_state: SeatState<Self>, // Smithay's manager for seat Wayland objects
    pub seat: Seat<Self>,            // Smithay's core Seat object
    pub seat_name: String,           // Name of the primary seat, e.g., "seat0"
    pub keyboard_data_map: HashMap<String, Arc<Mutex<XkbKeyboardData>>>, // MODIFIED type
    pub current_cursor_status: Arc<Mutex<CursorImageStatus>>, // For renderer to observe
    pub pointer_location: Point<f64, Logical>, // Global pointer coordinates
    pub active_input_surface: Option<Weak<WlSurface>>, // General purpose, might be kb focus
    // pub touch_focus_data: TouchFocusData, // REMOVED
    pub active_touch_targets: HashMap<TouchSlot, Weak<wl_surface::WlSurface>>, // Per-slot touch targets

    // --- Cursor Theming State ---
    pub loaded_theme: Arc<CursorTheme>,
    pub cursor_surface: wl_surface::WlSurface, // Dedicated surface for compositor-drawn cursors
    pub pointer_hotspot: Point<i32, Logical>, // Hotspot for the compositor-drawn cursor

    // --- Output Related State ---
    pub output_manager_state: OutputManagerState, // Smithay's manager for output Wayland objects
    pub output_manager: Arc<Mutex<OutputManager>>, // Our manager for OutputDevice instances

    // --- Tablet Manager State ---
    pub tablet_manager_state: TabletManagerState, // ADDED

    // --- Pointer Constraints State ---
    pub pointer_constraints_state: PointerConstraintsState, // ADDED

    // --- Graphics Backend Handles ---
    // These are initialized in the display_loop and passed to DesktopState
    pub renderer: Box<dyn FrameRenderer>,
    pub session: Arc<Session>, // Session needs to be Arc for potential sharing or longer lifetime needs
    pub drm_device: Arc<DrmDevice>, // DrmDevice is often Arc for sharing with DrmDisplay and event processing
    pub drm_display: DrmDisplay, // DrmDisplay might not need to be Arc if solely owned by DesktopState
    pub drm_surfaces: Arc<Mutex<HashMap<crtc::Handle, DrmSurface>>>, // Map of CRTC to DrmSurface
    pub drm_node: DrmNode,
    pub egl_display: EGLDisplay, // EGLDisplay might be needed by the renderer or for creating new contexts

    // Other fields like output_manager will be added later (this was a note for other fields, output_manager is here)
}

// Struct to hold graphics backend handles passed to DesktopState
// This helps in organizing the parameters for DesktopState::new
#[derive(Debug)]
pub struct GraphicsBackendHandles {
    pub session: Arc<Session>,
    pub drm_device: Arc<DrmDevice>,
    pub drm_display: DrmDisplay,
    pub drm_surfaces: Arc<Mutex<HashMap<crtc::Handle, DrmSurface>>>,
    pub drm_node: DrmNode,
    pub egl_display: EGLDisplay,
use crate::compositor::surface_management::{self, SurfaceData, AttachedBufferInfo}; // For SurfaceData access
use crate::compositor::renderer_interface::abstraction::RenderableTexture; // For texture handle in SurfaceData
use smithay::backend::renderer::element::AsRenderElements; // For render_elements_from_space
use smithay::backend::renderer::utils::Rectangle as RendererRectangle; // For damage rects
use std::error::Error; // For Result types in new methods

use crate::compositor::renderer_interface::abstraction::RenderElement; // Import the trait itself

// Concrete implementation for RenderElement wrapping ManagedWindow
// This is a simplified version. A more robust implementation might be needed
// depending on how textures and damage are managed and accessed.
#[derive(Debug)]
struct WindowRenderElement<'a> {
    window_id: Uuid,
    // wl_surface: &'a WlSurface, // Not strictly needed if all info is extracted
    transform: Transform, // Copied from SurfaceData
    window_location: Point<i32, Physical>,
    window_geometry_physical: Rectangle<i32, Physical>,
    texture_ref: &'a dyn RenderableTexture,
    damage_regions_buffer_coords: Vec<Rectangle<i32, BufferCoords>>, // Copied
}

impl<'a> RenderElement<'a> for WindowRenderElement<'a> {
    fn id(&self) -> Uuid { self.window_id }
    fn location(&self, _scale: f64) -> Point<i32, Physical> { self.window_location }
    fn geometry(&self, _scale: f64) -> Rectangle<i32, Physical> {
        // This should be the buffer geometry. Assuming window_geometry_physical.size is buffer size for now.
        Rectangle::from_loc_and_size((0,0), self.window_geometry_physical.size)
    }
    fn texture(&self, _scale: f64) -> &'a dyn RenderableTexture { self.texture_ref }
    fn damage(&self, _output_scale: f64, _space_size: Size<i32, Physical>) -> Vec<Rectangle<i32, Physical>> {
        // Transform damage from buffer to physical coordinates if necessary.
        // For now, assume damage_regions_buffer_coords can be used or adapted.
        // This simplification assumes 1:1 mapping from buffer to physical for damage rects.
        self.damage_regions_buffer_coords.iter().map(|rect_buffer_coords| {
            // This transformation is likely incorrect if buffer scale/transform is not identity.
            // For now, direct cast, assuming physical damage is requested by renderer.
            RendererRectangle::from_loc_and_size(
                (rect_buffer_coords.loc.x, rect_buffer_coords.loc.y),
                (rect_buffer_coords.size.w, rect_buffer_coords.size.h)
            )
        }).collect()
    }
    fn transform(&self) -> Transform { self.transform }
    fn alpha(&self) -> f32 { 1.0 } // TODO
    }
    fn opaque_regions(&self, _scale: f64) -> Option<Vec<Rectangle<i32, Physical>>> {
        // TODO: Implement based on surface opaque region if available
        None
    }
}

#[derive(Debug)]
struct CursorRenderElement<'a> {
    texture: &'a dyn RenderableTexture,
    location: Point<i32, Physical>, // Physical location on screen
}

impl<'a> RenderElement<'a> for CursorRenderElement<'a> {
    fn id(&self) -> Uuid { Uuid::nil() } // Cursor doesn't need a persistent ID for damage tracking in the same way windows do
    fn location(&self, _scale: f64) -> Point<i32, Physical> { self.location }
    fn geometry(&self, _scale: f64) -> Rectangle<i32, Physical> {
         Rectangle::from_loc_and_size((0,0), Size::from((self.texture.width_px() as i32, self.texture.height_px() as i32)))
    }
    fn texture(&self, _scale: f64) -> &'a dyn RenderableTexture { self.texture }
    fn damage(&self, _output_scale: f64, _space_size: Size<i32, Physical>) -> Vec<Rectangle<i32, Physical>> {
        // Full damage for cursor
        vec![Rectangle::from_loc_and_size((0,0), Size::from((self.texture.width_px() as i32, self.texture.height_px() as i32)))]
    }
    fn transform(&self) -> Transform { Transform::Normal }
    fn alpha(&self) -> f32 { 1.0 }
    fn opaque_regions(&self, _scale: f64) -> Option<Vec<Rectangle<i32, Physical>>> { Some(vec![self.geometry(_scale)]) } // Cursor is usually opaque
}


impl DesktopState {
    // Methods moved from display_loop/mod.rs and implemented here
    pub fn render_frame(&mut self, _background_color: [f32; 4]) -> Result<(), Box<dyn Error>> {
        // TODO: Multi-output rendering requires iterating through active outputs (from OutputManager)
        // and calling render_frame_on_output for each.
        // For now, assuming a single primary output identified by the first DRM surface.
        
        let output_details = {
            let surfaces_guard = self.drm_surfaces.lock().unwrap();
            surfaces_guard.iter().next().map(|(crtc, surface)| {
                let size = surface.size();
                // This should come from the OutputDevice's current mode and scale
                (crtc.clone(), Rectangle::from_loc_and_size((0,0), size), 1.0f64)
            })
        };

        if let Some((crtc_handle, output_render_geometry, output_scale)) = output_details {
            // Gather render elements. This part is tricky due to lifetimes.
            // We need to collect all SurfaceData guards and texture references first.
            // This approach avoids holding MutexGuards across the renderer call.
            // 1. Collect data (including textures) from windows.
            struct WindowRenderData<'a> {
                window_id: Uuid,
                texture_ref: &'a dyn RenderableTexture,
                location: Point<i32, Physical>,
                geometry_physical: Rectangle<i32, Physical>,
                transform: Transform,
                damage_buffer_coords: Vec<Rectangle<i32, BufferCoords>>,
            }
            let mut window_render_data_list = Vec::new();
            // Need to ensure guards are dropped before renderer is called if textures are references.
            // This requires careful lifetime management. Let's assume textures are Arc or similar,
            // or the renderer can handle short-lived texture references if it copies them.
            // For Box<dyn RenderableTexture>, we are passing references.
            
            // Store guards here temporarily and ensure they are dropped.
            let mut temp_guards = Vec::new();
            for window_arc in self.space.elements() {
                let guard = surface_management::get_surface_data(&window_arc.wl_surface()).lock().unwrap();
                if let Some(texture_handle) = &guard.texture_handle {
                    window_render_data_list.push(WindowRenderData {
                        window_id: window_arc.id(),
                        texture_ref: texture_handle.as_ref(),
                        location: self.space.element_location(window_arc).unwrap_or_default(),
                        geometry_physical: window_arc.geometry(),
                        transform: guard.current_transform().unwrap_or(Transform::Normal),
                        damage_buffer_coords: guard.damage_regions_buffer_coords.clone(),
                    });
                }
                temp_guards.push(guard); // Keep guard alive until all data is extracted
            }
            drop(temp_guards); // Explicitly drop guards

            // 2. Prepare cursor data if visible.
            struct CursorRenderData<'a> {
                texture_ref: &'a dyn RenderableTexture,
                location: Point<i32, Physical>,
            }
            let mut cursor_render_data_option = None;
            let cursor_status_guard = self.current_cursor_status.lock().unwrap(); // Hold lock for status
            let mut temp_cursor_guard = None; // For cursor surface data guard

            if let CursorImageStatus::Surface(ref cursor_wl_surface) = *cursor_status_guard {
                let guard = surface_management::get_surface_data(cursor_wl_surface).lock().unwrap();
                if let Some(texture_handle) = &guard.texture_handle {
                    let hotspot = self.pointer_hotspot;
                    let cursor_location_logical = self.pointer_location;
                    cursor_render_data_option = Some(CursorRenderData {
                        texture_ref: texture_handle.as_ref(),
                        location: Point::from((
                            (cursor_location_logical.x - hotspot.x as f64) as i32,
                            (cursor_location_logical.y - hotspot.y as f64) as i32,
                        )),
                    });
                }
                temp_cursor_guard = Some(guard); // Keep guard alive
            }
            drop(cursor_status_guard); // Drop status lock
            drop(temp_cursor_guard);   // Drop cursor surface data lock

            // 3. Build RenderElement list for the renderer.
            let mut elements_to_render_dyn: Vec<&dyn RenderElement> = Vec::new();
            let mut concrete_window_elements: Vec<WindowRenderElement> = Vec::new();
            for data in &window_render_data_list {
                concrete_window_elements.push(WindowRenderElement {
                    window_id: data.window_id,
                    transform: data.transform,
                    window_location: data.location,
                    window_geometry_physical: data.geometry_physical,
                    texture_ref: data.texture_ref,
                    damage_regions_buffer_coords: data.damage_buffer_coords.clone(),
                });
            }
            for el in &concrete_window_elements { elements_to_render_dyn.push(el); }
            
            let mut concrete_cursor_element = None; // Needs to live as long as elements_to_render_dyn
            if let Some(data) = &cursor_render_data_option {
                let hotspot = self.pointer_hotspot;
                let cursor_location_logical = self.pointer_location;
                let cursor_pos_physical = Point::from((
                    (cursor_location_logical.x - hotspot.x as f64) as i32,
                    (cursor_location_logical.y - hotspot.y as f64) as i32,
                ));
                // Correctly use the texture from CursorRenderData
                concrete_cursor_element = Some(CursorRenderElement {
                    texture: data.texture_ref, 
                    location: cursor_pos_physical,
                });
                if let Some(el) = &concrete_cursor_element { elements_to_render_dyn.push(el); }
            }


            match self.renderer.render_frame(output_render_geometry, output_scale, elements_to_render_dyn.into_iter()) {
                Ok(_render_damage) => { /* Use damage if needed by present_frame */ }
                Err(e) => tracing::error!("Error rendering frame: {}", e),
            }

            if let Err(e) = self.renderer.present_frame(Some(crtc_handle.into())) {
                tracing::error!("Error presenting frame on CRTC {:?}: {}", crtc_handle, e);
            }
        } else {
            tracing::warn!("render_frame called but no DRM surfaces available.");
        }
        
        self.space.send_frames(self.clock.now().try_into().unwrap_or_default());
        Ok(())
    }

    pub fn process_drm_events(&mut self) -> Result<(), smithay::backend::drm::DrmError> {
        self.drm_device.process_events()
    }

    pub fn dispatch_clients(&mut self) -> Result<(), Box<dyn Error>> {
        self.display_handle.dispatch_clients(self).map_err(Into::into)
    }

    pub fn flush_clients(&mut self) -> Result<(), Box<dyn Error>> {
        self.display_handle.flush_clients(self).map_err(Into::into)
    }

    pub fn new(
        display_handle: DisplayHandle,
        loop_handle: LoopHandle<'static, Self>,
        loop_signal: LoopSignal,
        renderer: Box<dyn FrameRenderer>,
        graphics_handles: GraphicsBackendHandles,
    ) -> Self {
        let seat_name = "seat0".to_string(); // Default seat name

        // Initialize SeatState for managing seat-related Wayland globals
        let mut seat_state = SeatState::new();

        // Create the primary compositor Seat object
        // The seat needs to be created via SeatState to correctly associate with Wayland globals.
        // However, Seat::new is also a valid way if not using SeatState for global creation directly.
        // Let's ensure consistency: Smithay examples often create Seat via SeatState.
        // SeatState::new_seat is not a method. Seat::new is typical.
        // The wl_seat global is created later via seat_state.new_wl_seat().
        let mut seat = Seat::new(&display_handle, seat_name.clone(), None); // Logger can be added
        let clock = Clock::new(None); // ADDED/UNCOMMENTED initialization, using None for span for now
        
        let mut compositor_state = CompositorState::new();
        let shm_state = ShmState::new(vec![], None); // Assuming tracing::Span::current() would be passed if enabled for ShmState

        // Initialize Cursor Theme and Surface
        let theme_name = env::var("XCURSOR_THEME").unwrap_or_else(|_| "default".to_string());
        let theme_size = env::var("XCURSOR_SIZE").ok().and_then(|s| s.parse().ok()).unwrap_or(24);
        
        let loaded_theme = Arc::new(load_theme(
            Some(&theme_name),
            theme_size,
            shm_state.wl_shm()
        ));

        let cursor_surface = compositor_state.create_surface_with_data(
            &display_handle,
            smithay::wayland::compositor::SurfaceAttributes::default(),
            (), // No specific user data for the cursor surface itself initially
        );
        // It's good practice to set an input region for the cursor surface if it's to be interactive,
        // but for a simple display cursor, it might not be strictly necessary.
        // For now, we'll assume it's just for display.

        let tablet_manager_state = TabletManagerState::new::<Self>(&display_handle); // ADDED
        let pointer_constraints_state = PointerConstraintsState::new(); // ADDED: PointerConstraintsState::new() does not take display_handle

        // Store SeatState in the seat's user data if SeatState itself needs to be accessed via Seat later.
        // Or, more commonly, SeatState is owned by DesktopState directly.
        // The SeatHandler methods get `&mut SeatState<Self>` via `self.seat_state`.
        // seat.user_data().insert_if_missing(|| seat_state.clone()); // SeatState is not Clone

        // Field for current cursor texture (software cursor)
        let current_cursor_texture = Arc::new(Mutex::new(None::<Box<dyn RenderableTexture>>));


        Self {
            display_handle: display_handle.clone(),
            // current_cursor_texture, // This was for software cursor via renderer, SeatHandler::cursor_image will manage it
            loop_handle,
            loop_signal,
            clock, // ADDED/UNCOMMENTED
            compositor_state, // Initialized above
            shm_state,       // Initialized above
            xdg_shell_state: XdgShellState::new(&display_handle, PopupManager::new(None), None),
            xdg_activation_state: XdgActivationState::new(&display_handle, seat.clone(), None), // XdgActivationState needs the Seat
            space: Space::new(None),
            windows: HashMap::new(),
            shm_global: None,
            xdg_shell_global: None,
            xdg_activation_global: None,

            seat_state, // Initialize SeatState
            seat,       // Initialize Seat
            seat_name,  // Initialize seat_name
            keyboard_data_map: HashMap::new(), // Initialization is fine, type already changed
            current_cursor_status: Arc::new(Mutex::new(CursorImageStatus::Default)),
            pointer_location: Point::from((0.0, 0.0)),
            active_input_surface: None,
            // touch_focus_data: TouchFocusData::default(), // REMOVED
            active_touch_targets: HashMap::new(), // Initialize new field
            
            loaded_theme, // ADDED
            cursor_surface, // ADDED
            pointer_hotspot: (0,0).into(), // ADDED initialization

            output_manager_state: OutputManagerState::new_with_xdg_output::<Self>(&display_handle), // Initialize with XDG support
            output_manager: Arc::new(Mutex::new(OutputManager::new())),
            tablet_manager_state, // ADDED
            pointer_constraints_state, // ADDED

            // Store the graphics backend handles
            renderer,
            session: graphics_handles.session,
            drm_device: graphics_handles.drm_device,
            drm_display: graphics_handles.drm_display,
            drm_surfaces: graphics_handles.drm_surfaces,
            drm_node: graphics_handles.drm_node,
            egl_display: graphics_handles.egl_display,
        }
    }
}

// Required import for the GraphicsBackendHandles struct fields if not already present at the top
use smithay::reexports::drm::control::crtc;

smithay::delegate_client_handler!(DesktopState);

// --- SeatHandler Implementation ---
impl SeatHandler for DesktopState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface; // Or WlSurface if touch focus is on a surface

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&Self::KeyboardFocus>) {
        tracing::debug!(
            seat_name = %seat.name(),
            new_focus_surface_id = ?focused.map(|s| s.id()),
            "SeatHandler::focus_changed (keyboard) triggered by Smithay."
        );
        let new_focused_wl_surface_weak = focused.map(|s| s.downgrade());
        self.active_input_surface = new_focused_wl_surface_weak;
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        let mut current_status_guard = self.current_cursor_status.lock().unwrap();
        
        match image {
            CursorImageStatus::Named(name) => {
                if let Some(cursor_image_buffer) = self.loaded_theme.get_cursor(&name) {
                    self.cursor_surface.attach(Some(cursor_image_buffer.buffer()), 0, 0);
                    if self.cursor_surface.send_pending_events().is_ok() { // Check if surface is alive
                        self.cursor_surface.damage_buffer(0, 0, cursor_image_buffer.width() as i32, cursor_image_buffer.height() as i32);
                        self.cursor_surface.commit();
                        self.pointer_hotspot = (cursor_image_buffer.hotspot_x() as i32, cursor_image_buffer.hotspot_y() as i32).into();
                        *current_status_guard = CursorImageStatus::Surface(self.cursor_surface.clone());
                        tracing::debug!("Cursor set to themed: '{}'", name);
                    } else {
                        *current_status_guard = CursorImageStatus::Default; // Fallback if surface is dead
                        tracing::warn!("Failed to set themed cursor '{}': cursor_surface is not alive.", name);
                    }
                } else {
                    tracing::warn!("Cursor name '{}' not found in theme, using default.", name);
                    // Try to load "left_ptr" as a fallback default
                    if let Some(default_cursor_buffer) = self.loaded_theme.get_cursor("left_ptr")
                        .or_else(|| self.loaded_theme.cursors().get(0).cloned()) // Absolute fallback
                    {
                        self.cursor_surface.attach(Some(default_cursor_buffer.buffer()), 0, 0);
                        if self.cursor_surface.send_pending_events().is_ok() {
                             self.cursor_surface.damage_buffer(0, 0, default_cursor_buffer.width() as i32, default_cursor_buffer.height() as i32);
                             self.cursor_surface.commit();
                             self.pointer_hotspot = (default_cursor_buffer.hotspot_x() as i32, default_cursor_buffer.hotspot_y() as i32).into();
                            *current_status_guard = CursorImageStatus::Surface(self.cursor_surface.clone());
                             tracing::debug!("Cursor set to fallback default theme cursor ('left_ptr' or first available).");
                        } else {
                            *current_status_guard = CursorImageStatus::Hidden; // Fallback if surface is dead
                            tracing::warn!("Failed to set fallback themed cursor: cursor_surface is not alive.");
                        }
                    } else {
                        *current_status_guard = CursorImageStatus::Hidden; // Ultimate fallback
                        tracing::error!("Could not load any default cursor from the theme.");
                    }
                }
            }
            CursorImageStatus::Surface(surface) => {
                // Client provides the surface. Renderer handles hotspot based on client data.
                *current_status_guard = CursorImageStatus::Surface(surface);
                 tracing::debug!("Cursor set to client-provided surface.");
            }
            CursorImageStatus::Hidden => {
                *current_status_guard = CursorImageStatus::Hidden;
                tracing::debug!("Cursor set to hidden.");
            }
            CursorImageStatus::Default => {
                // This case might be hit if Smithay itself decides to revert to default.
                // Similar to Named("default") or a specific default cursor like "left_ptr"
                if let Some(cursor_image_buffer) = self.loaded_theme.get_cursor("left_ptr")
                    .or_else(|| self.loaded_theme.get_cursor("default"))
                    .or_else(|| self.loaded_theme.cursors().get(0).cloned())
                {
                    self.cursor_surface.attach(Some(cursor_image_buffer.buffer()), 0, 0);
                     if self.cursor_surface.send_pending_events().is_ok() {
                        self.cursor_surface.damage_buffer(0, 0, cursor_image_buffer.width() as i32, cursor_image_buffer.height() as i32);
                        self.cursor_surface.commit();
                        self.pointer_hotspot = (cursor_image_buffer.hotspot_x() as i32, cursor_image_buffer.hotspot_y() as i32).into();
                        *current_status_guard = CursorImageStatus::Surface(self.cursor_surface.clone());
                        tracing::debug!("Cursor set to default theme cursor (via Default status).");
                    } else {
                        *current_status_guard = CursorImageStatus::Hidden; // Fallback if surface is dead
                         tracing::warn!("Failed to set default themed cursor (via Default status): cursor_surface is not alive.");
                    }
                } else {
                    *current_status_guard = CursorImageStatus::Hidden; // Ultimate fallback
                    tracing::error!("Could not load any default cursor from the theme (via Default status).");
                }
            }
        }
    }
}
smithay::delegate_seat_handler!(DesktopState); // Ensures DesktopState delegates SeatHandler calls correctly

// --- TabletManagerHandler Implementation ---
impl TabletManagerHandler for DesktopState {
    fn new_tablet_seat(&mut self, seat: Seat<Self>) -> Box<dyn TabletSeatTrait<Self>> {
        tracing::info!("Neuer Tablet-Seat für Seat '{}' angefordert.", seat.name());
        Box::new(smithay::wayland::tablet_manager::TabletSeat::new(seat))
    }
}
smithay::delegate_tablet_manager!(DesktopState);

// --- PointerConstraintsHandler Implementation ---
impl PointerConstraintsHandler for DesktopState {
    fn new_constraint(
        &mut self,
        constraint: &PointerConstraint,
    ) {
        if let Some(surface) = constraint.surface() {
             tracing::info!(
                "Neuer Pointer-Constraint {:?} für Surface {:?} erstellt.",
                constraint.constraint_type(),
                surface.id()
            );
        } else {
            tracing::warn!("Neuer Pointer-Constraint ohne zugehörige Surface erstellt.");
        }
    }
    // constraint_broken can be implemented if custom logic is needed when constraints are no longer valid
}

// --- GlobalDispatch for PointerConstraints ---
impl GlobalDispatch<zwp_pointer_constraints_v1::ZwpPointerConstraintsV1, ()> for DesktopState {
    fn bind(
        &mut self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<zwp_pointer_constraints_v1::ZwpPointerConstraintsV1>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        // PointerConstraintsState handles the binding and further dispatching internally
        self.pointer_constraints_state.new_global(resource, data_init);
        tracing::info!("zwp_pointer_constraints_v1 global gebunden.");
    }
}

// --- Dispatch for LockedPointer ---
impl Dispatch<zwp_locked_pointer_v1::ZwpLockedPointerV1, LockedPointerData> for DesktopState {
    fn request(
        &mut self,
        _client: &Client,
        _resource: &zwp_locked_pointer_v1::ZwpLockedPointerV1,
        request: zwp_locked_pointer_v1::Request,
        data: &LockedPointerData,
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        self.pointer_constraints_state.handle_locked_pointer_request(request, data, data_init);
    }

    fn destroyed(
        &mut self,
        _client: ClientId,
        resource: &zwp_locked_pointer_v1::ZwpLockedPointerV1,
        data: &LockedPointerData,
    ) {
        self.pointer_constraints_state.handle_locked_pointer_destroyed(resource, data);
        tracing::info!("zwp_locked_pointer_v1 zerstört: {:?}", resource.id());
    }
}

// --- Dispatch for ConfinedPointer ---
impl Dispatch<zwp_confined_pointer_v1::ZwpConfinedPointerV1, ConfinedPointerData> for DesktopState {
    fn request(
        &mut self,
        _client: &Client,
        _resource: &zwp_confined_pointer_v1::ZwpConfinedPointerV1,
        request: zwp_confined_pointer_v1::Request,
        data: &ConfinedPointerData,
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        self.pointer_constraints_state.handle_confined_pointer_request(request, data, data_init);
    }

    fn destroyed(
        &mut self,
        _client: ClientId,
        resource: &zwp_confined_pointer_v1::ZwpConfinedPointerV1,
        data: &ConfinedPointerData,
    ) {
        self.pointer_constraints_state.handle_confined_pointer_destroyed(resource, data);
        tracing::info!("zwp_confined_pointer_v1 zerstört: {:?}", resource.id());
    }
}

// Delegate dispatch for pointer constraint protocols to PointerConstraintsState
smithay::delegate_dispatch!(DesktopState: [
    zwp_pointer_constraints_v1::ZwpPointerConstraintsV1: ()
] => PointerConstraintsState);
smithay::delegate_dispatch!(DesktopState: [
    zwp_locked_pointer_v1::ZwpLockedPointerV1: LockedPointerData
] => PointerConstraintsState);
smithay::delegate_dispatch!(DesktopState: [
    zwp_confined_pointer_v1::ZwpConfinedPointerV1: ConfinedPointerData
] => PointerConstraintsState);


// --- OutputHandler Implementation ---
impl OutputHandler for DesktopState {
    fn output_state(&mut self) -> &mut OutputManagerState {
        &mut self.output_manager_state
    }

    fn new_output(&mut self, output: &wl_output::WlOutput, _output_data: &OutputData) {
        tracing::info!(output_name = %output.name(), output_description = %output.description(), "New wl_output global created by Smithay: {:?}", output.id());
        // Smithay's OutputManagerState handles the creation. We might log or perform
        // additional setup if needed, but usually, Smithay takes care of it.
        // The OutputDevice and its globals are managed via OutputManager::handle_hotplug_event
        // and output_device_created_notifications.
    }

    fn output_destroyed(&mut self, output: &wl_output::WlOutput, _output_data: &OutputData) {
        tracing::info!(output_name = %output.name(), "wl_output global destroyed by Smithay: {:?}", output.id());
        // Smithay handles the destruction. If we need to clean up anything in DesktopState
        // directly related to this specific wl_output global (not the OutputDevice itself,
        // which is handled by OutputManager), it would go here.
    }
}
smithay::delegate_output!(DesktopState); // Delegate wl_output handling to DesktopState

// --- GlobalDispatch for ZxdgOutputManagerV1 ---
// This ensures that clients can bind to the XDG Output Manager.
impl GlobalDispatch<ZxdgOutputManagerV1, ()> for DesktopState {
    fn bind(
        &mut self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<ZxdgOutputManagerV1>,
        _global_data: &(), // No specific global data for ZxdgOutputManagerV1 itself
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!("Client binding ZxdgOutputManagerV1");
        // Use Smithay's OutputManagerState to handle the binding and further dispatch.
        // This correctly sets up the ZxdgOutputManagerV1 resource to use Smithay's internal
        // dispatching logic, which will handle get_xdg_output requests based on existing
        // wl_output globals and their associated XDG output data.
        self.output_manager_state.xdg_output_manager_bind_dispatch(resource, data_init);
    }
}

// Smithay's OutputManagerState, when initialized with new_with_xdg_output,
// internally handles the Dispatch implementations for ZxdgOutputManagerV1 and ZxdgOutputV1.
// We just need to ensure the delegation macro for xdg_output is present.
smithay::delegate_xdg_output!(DesktopState); // Delegate zxdg_output_v1 and zxdg_output_manager_v1 requests


// Implement CompositorHandler for DesktopState
impl CompositorHandler for DesktopState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        // Smithay's CompositorHandler trait expects a CompositorClientState.
        // We can store this in the client's UserDataMap or use a dedicated field if ClientCompositorData grows.
        // For now, relying on Smithay's default implementation or storing it via UserData.
        // This part might need adjustment based on how ClientCompositorData is actually used.
        client
            .get_data::<CompositorClientState>()
            .unwrap() // Or handle error appropriately
    }

    fn commit(&mut self, surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface) {
        // Use surface_management::commit_surface_state_buffer or similar, then our custom logic.
        // Smithay's on_commit_buffer_handler usually goes first if not using custom buffer management.
        smithay::wayland::compositor::on_commit_buffer_handler::<Self>(surface);

        let client_id = surface.client().map(|c| c.id()); // Get client ID for SurfaceData::new if needed

        surface_management::with_surface_data_mut(surface, |surface_data| {
            let surface_attributes = smithay::wayland::compositor::surface_attributes(surface);

            // 1. Handle SHM buffer import and texture creation
            if let Some(buffer) = surface_attributes.buffer.as_ref() {
                if smithay::wayland::shm::is_shm_buffer(buffer) {
                    match self.renderer.create_texture_from_shm(buffer) {
                        Ok(texture) => {
                            surface_data.texture_handle = Some(texture);
                            tracing::info!("SHM buffer imported as texture for surface {:?}", surface.id());
                        }
                        Err(e) => {
                            tracing::error!("Failed to create texture from SHM buffer for surface {:?}: {}", surface.id(), e);
                            surface_data.texture_handle = None; // Clear previous texture
                        }
                    }
                } else if let Ok(dmabuf) = smithay::wayland::dmabuf::get_dmabuf(buffer) {
                     match self.renderer.create_texture_from_dmabuf(&dmabuf) {
                        Ok(texture) => {
                            surface_data.texture_handle = Some(texture);
                            tracing::info!("DMABUF imported as texture for surface {:?}", surface.id());
                        }
                        Err(e) => {
                            tracing::error!("Failed to create texture from DMABUF for surface {:?}: {}", surface.id(), e);
                            surface_data.texture_handle = None;
                        }
                    }
                } else {
                    tracing::warn!("Surface {:?} committed with an unsupported buffer type. Clearing texture.", surface.id());
                    surface_data.texture_handle = None;
                }

                // 2. Update current_buffer_info (simplified)
                surface_data.current_buffer_info = Some(AttachedBufferInfo {
                    buffer: buffer.clone(),
                    damage: surface_attributes.damage_buffer.clone(), // Damage in buffer coordinates
                    transform: surface_attributes.buffer_transform,
                    scale: surface_attributes.buffer_scale,
                });
            } else {
                // No buffer attached (client attached null buffer)
                surface_data.texture_handle = None;
                surface_data.current_buffer_info = None;
                tracing::info!("Surface {:?} committed with null buffer. Texture and buffer info cleared.", surface.id());
            }

            // 3. Manage damage tracking
            // surface_attributes.damage_surface contains damage in surface coordinates.
            // This needs to be transformed to buffer coordinates if that's what your renderer expects
            // or accumulated in physical coordinates for the space.
            // For SurfaceData, we store buffer damage.
            surface_data.damage_regions_buffer_coords.clear();
            surface_data.damage_regions_buffer_coords.extend_from_slice(&surface_attributes.damage_buffer);

            // Store other attributes
            surface_data.current_scale_factor = surface_attributes.buffer_scale;
            // surface_data.current_transform = surface_attributes.buffer_transform; // Already in AttachedBufferInfo

        });

        // If the surface is part of the space (i.e., a window), mark its region for redraw.
        // This depends on how ManagedWindow relates to WlSurface and how Space tracks damage.
        // Smithay's Space::damage_element can be used if ManagedWindow is the element.
        if let Some(window) = self.space.elements().find(|w| w.wl_surface() == *surface) {
            // The damage here should be in global coordinates.
            // Smithay's render_elements_from_space helper usually calculates this.
            // For now, let's assume full damage for the window on commit.
            self.space.damage_element(window, None, None); // Damage the whole window area
        }


        // Handle XDG surface role specific commit logic (e.g. configure acks)
        if let Some(toplevel_surface) = smithay::wayland::shell::xdg::ToplevelSurface::try_from(surface) {
            // Handle toplevel specific commits if necessary, though ack_configure is separate
        } else if let Some(popup_surface) = smithay::wayland::shell::xdg::PopupSurface::try_from(surface) {
            // Handle popup specific commits
        }


        tracing::debug!("CompositorHandler: Commit processed for surface {:?}", surface.id());
    }

    fn new_surface(&mut self, surface: &WlSurface, client_data: &CompositorClientState) { // Added client_data
        let client_id = client_data.client_id(); // Get client ID from CompositorClientState

        // Initialize SurfaceData for the new surface
        // The SurfaceData::new method in surface_management.rs already uses surface.client()
        // if we need the client_id there.
        surface_management::get_surface_data(surface); // This ensures it's initialized

        // Add destruction hook for renderer resource cleanup
        let renderer_clone = self.renderer.clone_renderer(); // Assuming FrameRenderer has a way to be cloned (e.g. Arc internal) or we pass a handle
        let surface_clone = surface.clone();
        surface.add_destruction_hook(move |_surface_destroyed_data| { // _surface_destroyed_data is UserData from surface
            tracing::info!("Surface {:?} destroyed. Cleaning up associated renderer resources.", surface_clone.id());
            surface_management::with_surface_data_mut(&surface_clone, |surface_data| {
                if let Some(texture_handle) = surface_data.texture_handle.take() {
                    // The renderer needs a method to release textures by their handle/ID.
                    // The texture_handle is Box<dyn RenderableTexture>, which has an id() method.
                    if let Err(e) = renderer_clone.release_texture(texture_handle.id()) {
                        tracing::warn!("Error releasing texture for destroyed surface {:?}: {}", surface_clone.id(), e);
                    }
                }
            });
        });
        tracing::info!("New surface {:?} created and SurfaceData initialized. Destruction hook added.", surface.id());
    }
}

// Implement GlobalDispatch for WlCompositor and WlSubcompositor (existing)
impl GlobalDispatch<wl_compositor::WlCompositor, ()> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: smithay::reexports::wayland_server::New<wl_compositor::WlCompositor>,
        _global_data: &(),
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, Self>,
    ) {
        tracing::info!("Binding WlCompositor");
        data_init.init(resource, ());
    }
}

impl GlobalDispatch<wl_subcompositor::WlSubcompositor, ()> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: smithay::reexports::wayland_server::New<wl_subcompositor::WlSubcompositor>,
        _global_data: &(),
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, Self>,
    ) {
        tracing::info!("Binding WlSubcompositor");
        data_init.init(resource, ());
    }
}


// Delegate Smithay traits (existing)
delegate_compositor!(DesktopState);
delegate_shm!(DesktopState);

// BufferHandler (existing)
impl BufferHandler for DesktopState {
    fn buffer_destroyed(
        &mut self,
        buffer_to_destroy: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) {
        tracing::info!("Buffer {:?} destroyed.", buffer_to_destroy.id());

        // Iterate through all surfaces to see if this buffer was backing any texture.
        // This is inefficient. A better approach would be to have a map from buffer ID to surface ID
        // or for the renderer to manage textures by buffer ID if it caches them that way.
        // For now, we iterate.
        
        let mut affected_surfaces = Vec::new();
        for window_arc in self.space.elements() {
            let wl_surface = window_arc.wl_surface();
            let mut surface_data_guard = surface_management::get_surface_data(&wl_surface).lock().unwrap();
            
            if let Some(buffer_info) = &surface_data_guard.current_buffer_info {
                if buffer_info.buffer == *buffer_to_destroy {
                    affected_surfaces.push(wl_surface.clone());
                    // Clear texture handle and buffer info from this specific surface_data
                    if let Some(texture_handle) = surface_data_guard.texture_handle.take() {
                        if let Err(e) = self.renderer.release_texture(texture_handle.id()) {
                            tracing::warn!("Error releasing texture for buffer {:?} on surface {:?}: {}", buffer_to_destroy.id(), wl_surface.id(), e);
                        }
                    }
                    surface_data_guard.current_buffer_info = None;
                    // Mark for redraw/damage - handled by space.damage_element below
                }
            }
        }

        for surface in affected_surfaces {
            tracing::info!("Cleared texture from surface {:?} due to buffer destruction.", surface.id());
            if let Some(window) = self.space.elements().find(|w| w.wl_surface() == surface) {
                self.space.damage_element(window, None, None); // Damage the whole window
            }
        }

        // Also check the cursor surface if it's using this buffer (less likely for SHM buffers but possible)
        let cursor_wl_surface = self.cursor_surface.clone(); // Clone to avoid borrow issues if cursor_surface is also in space
        let mut cursor_surface_data = surface_management::get_surface_data(&cursor_wl_surface).lock().unwrap();
        if let Some(buffer_info) = &cursor_surface_data.current_buffer_info {
            if buffer_info.buffer == *buffer_to_destroy {
                if let Some(texture_handle) = cursor_surface_data.texture_handle.take() {
                     if let Err(e) = self.renderer.release_texture(texture_handle.id()) {
                        tracing::warn!("Error releasing texture for cursor buffer {:?}: {}", buffer_to_destroy.id(), e);
                    }
                }
                cursor_surface_data.current_buffer_info = None;
                // If the cursor texture was from this buffer, current_cursor_status might need update
                // but SeatHandler::cursor_image should handle setting a new one or default.
                tracing::info!("Cleared texture from cursor surface due to buffer destruction.");
            }
        }
    }
}

// ShmHandler
impl ShmHandler for DesktopState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
    // shm_formats needs to be implemented if not using default
    fn shm_formats(&self) -> &[wl_shm::Format] {
        // Provide the list of SHM formats supported by the renderer.
        // This typically comes from the renderer capabilities.
        // For Gles2NovaRenderer, it would depend on what Gles2 can handle.
        // Common formats: Argb8888, Xrgb8888.
        // For now, returning a common set. This should be queried from the renderer.
        &[wl_shm::Format::Argb8888, wl_shm::Format::Xrgb8888, wl_shm::Format::Rgb565]
    }
}

// XdgShellHandler (minimal for now, will be expanded in xdg_shell/handlers.rs)
impl XdgShellHandler for DesktopState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        tracing::info!("New toplevel surface created: {:?}", surface.wl_surface());
        // Full ManagedWindow creation and mapping will be in xdg_shell/handlers.rs
        // Ensure SurfaceData is initialized for the underlying WlSurface.
        // This should be handled by CompositorHandler::new_surface if it's consistently called first.
        // If not, initialize here:
        surface_management::get_surface_data(surface.wl_surface()); // Ensures initialization

        // Store XdgToplevelSurfaceData in SurfaceData.user_data_ext if needed for custom logic.
        // Smithay already stores it in WlSurface's data_map.
        // let xdg_toplevel_data = surface.wl_surface().get_data::<XdgToplevelSurfaceData>().unwrap();
        // surface_management::with_surface_data_mut(surface.wl_surface(), |data| {
        //    data.user_data_ext = Some(Arc::new(Mutex::new(xdg_toplevel_data.clone())));
        // });
        // This is often not necessary as XdgToplevelSurfaceData is accessible via surface.wl_surface().get_data().

        // For now, just acknowledge to prevent client errors, actual logic in xdg_shell/handlers.rs
        let initial_configure_serial = surface.send_configure();
        tracing::info!("New toplevel surface {:?} created. Initial configure serial: {:?}", surface.wl_surface().id(), initial_configure_serial);
    }

    fn new_popup(&mut self, surface: PopupSurface, _client_data: &XdgWmBaseClientData) {
        tracing::info!("New popup surface created: {:?}", surface.wl_surface().id());
        surface_management::get_surface_data(surface.wl_surface()); // Ensures initialization

        // Store XdgPopupSurfaceData if needed, similar to toplevel.
        // Smithay already stores it.

        // For now, just acknowledge.
        let initial_configure_serial = surface.send_configure();
        tracing::info!("New popup surface {:?} created. Initial configure serial: {:?}", surface.wl_surface().id(), initial_configure_serial);
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        tracing::info!("Toplevel surface destroyed: {:?}", surface.wl_surface());
        // TODO: Cleanup ManagedWindow from space and windows map.
    }

    fn popup_destroyed(&mut self, surface: PopupSurface) {
        tracing::info!("Popup surface destroyed: {:?}", surface.wl_surface());
        // TODO: Cleanup ManagedWindow from space and windows map.
    }

    fn ack_configure(&mut self, surface: smithay::reexports::wayland_server::protocol::wl_surface::WlSurface, configure: smithay::wayland::shell::xdg::XdgSurfaceConfigure) {
        tracing::info!("Client acknowledged configure for surface: {:?}", surface);
        let _ = configure; // Data from configure can be used if needed.
        // TODO: Update ManagedWindow state based on ack if necessary.
    }

    // ... other XdgShellHandler methods will be implemented in xdg_shell/handlers.rs
    // For now, these stubs prevent the program from crashing if these events occur.
    fn grab(&mut self, _surface: PopupSurface, _seat: Seat<Self>, _serial: Serial) {
        tracing::warn!("XdgShellHandler::grab not yet implemented.");
    }

}

// XdgActivationHandler (minimal for now)
impl smithay::wayland::shell::xdg::activation::XdgActivationHandler for DesktopState {
    fn activation_state(&mut self) -> &mut smithay::wayland::shell::xdg::activation::XdgActivationState {
        &mut self.xdg_activation_state
    }

    fn request_new_token(&mut self, _token_data: smithay::wayland::shell::xdg::activation::TokenData, client: &Client) { // Added client
        tracing::info!("Client {:?} requested new XDG activation token.", client.id());
        // Smithay's XdgActivationState handles token creation and storage.
        // We might add custom logic here if needed (e.g. logging, policy checks).
    }

    fn token_activated(&mut self, token: String, activated_surface_role: smithay::wayland::shell::xdg::activation::ActivatedSurfaceRole) { // Corrected params
        tracing::info!("XDG activation token {} activated for role {:?}.", token, activated_surface_role);
        // TODO: Handle window activation logic (e.g., bring window to front, focus).
        // This would involve finding the ManagedWindow associated with the surface
        // that corresponds to this token and role, then calling set_activated(true) on it,
        // and potentially focusing the seat on it.
    }
}

// ClientHandler implementation for DesktopState
impl WaylandClientHandler for DesktopState {
    fn client_created(&mut self, client_id: ClientId, client: Arc<Client>) {
        tracing::info!("New client connected: ID {:?}", client_id);

        // Create our custom ClientData
        let client_data_arc = Arc::new(ClientData::new());
        tracing::info!("Created ClientData with internal UUID: {}", client_data_arc.id);

        // Store our ClientData in the client's global UserDataMap
        client.user_data().insert_if_missing_threadsafe(|| client_data_arc.clone());

        // Initialize Smithay's per-client states and store them in our ClientData's UserDataMap.
        // Note: Smithay's states often expect to be stored directly in the wayland_server::Client's UserDataMap.
        // The plan implies storing them within *our* ClientData's UserDataMap. Let's adjust if Smithay's
        // `new_client` methods directly manipulate `client.user_data()` vs returning data for us to store.

        // CompositorState per-client data
        // Smithay's CompositorState::new_client takes `&Client` and stores CompositorClientState in its UserDataMap.
        // It does not return the state for us to store elsewhere.
        self.compositor_state.new_client(&client);
        // To verify, we could check if client.user_data().get::<CompositorClientState>().is_some() here.

        // XdgShellState per-client data
        // XdgShellState::new_client returns XdgWmBaseClientData, which we *are* supposed to store.
        let xdg_wm_base_client_data = self.xdg_shell_state.new_client(&self.display_handle, &client);
        client_data_arc.user_data_map.insert_if_missing_threadsafe(|| xdg_wm_base_client_data);
        tracing::debug!("Stored XdgWmBaseClientData in our ClientData for client {:?}", client_id);

        // ShmState per-client data (if any)
        // ShmState::new_client is similar to CompositorState, it manages its own client data.
        self.shm_state.new_client(&client);

        // TODO: Potentially other per-client states (e.g., for custom protocols).
        // Example: If we had ClientCompositorData as a separate struct we manage:
        // let our_compositor_data = ClientCompositorData::default();
        // client_data_arc.user_data_map.insert_if_missing_threadsafe(|| our_compositor_data);

        tracing::info!("Client {:?} fully initialized with its data maps.", client_id);
    }

    fn client_disconnected(&mut self, client_id: ClientId, client: Client) {
        tracing::info!("Client disconnected: ID {:?}", client_id);
        // Smithay's states (CompositorState, XdgShellState, ShmState) have `client_destroyed`
        // methods that are typically called by Display::dispatch_clients when a client disconnects.
        // These methods clean up their internal per-client data.
        // Our `ClientData` (and anything in its `user_data_map`) stored in `client.user_data()`
        // will be dropped automatically when the `Arc<Client>` is dropped, as Smithay
        // removes the client from its internal list.

        // If we needed to do explicit cleanup beyond what RAII provides for ClientData:
        if let Some(client_data_arc) = client.user_data().get::<Arc<ClientData>>() {
            tracing::info!("Cleaning up ClientData with internal UUID: {}", client_data_arc.id);
            // Any specific cleanup related to client_data_arc.user_data_map contents could go here,
            // but typically UserDataMap handles drops of its stored Arcs correctly.
        } else {
            tracing::warn!("Could not find our ClientData for disconnected client {:?}", client_id);
        }

        // Call Smithay's client_destroyed methods
        self.compositor_state.client_destroyed(&client_id);
        self.xdg_shell_state.client_destroyed(&client); // Note: XdgShellState takes &Client
        self.shm_state.client_destroyed(&client_id);
        self.xdg_activation_state.client_destroyed(&client_id);


        // Any other state cleanup related to this client.
        // E.g., explicitly unmapping/destroying windows owned by this client.
        // Smithay's shell integrations often handle this when the surface roles are destroyed.
        // We also have toplevel_destroyed / popup_destroyed in XdgShellHandler.
        // Let's ensure all windows owned by this client are cleaned up.
        let client_windows: Vec<_> = self.windows.iter()
            .filter(|(_, managed_window)| {
                managed_window.wl_surface().client().map_or(false, |c| c.id() == client_id)
            })
            .map(|(id, _)| *id)
            .collect();

        for window_domain_id in client_windows {
            tracing::info!("Performing cleanup for window {:?} due to client {:?} disconnect.", window_domain_id, client_id);
            if let Some(managed_window_arc) = self.windows.remove(&window_domain_id) {
                 if managed_window_arc.is_mapped_by_compositor() {
                    self.space.unmap_window(&managed_window_arc);
                 }
                 // Other cleanup related to ManagedWindow, like notifying domain layer.
                 tracing::info!("Removed and unmapped window {:?} for disconnected client.", window_domain_id);
            }
        }
        if !self.windows.is_empty() { // Damage only if necessary
            self.space.damage_all();
            self.loop_signal.wakeup();
        }
    }
}
