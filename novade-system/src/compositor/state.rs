use std::{
    ffi::OsString,
    sync::{Arc, Mutex},
};

use smithay::{
    backend::renderer::{
        gles2::{Gles2Renderer, Gles2Error},
    },
    desktop::{Space, Window, PopupManager},
    input::{Seat, SeatState, pointer::{CursorImageStatus, GrabStartData}, keyboard::{ModifiersState, XkbConfig}, InputHandler, InputState as InputStateSmithay}, // Added GrabStartData, InputHandler
    output::{Output, OutputHandler, OutputState as OutputStateSmithay}, // Added OutputHandler
    reexports::{
        calloop::{EventLoop, LoopHandle},
        wayland_server::{
            backend::{ClientId, DisconnectReason}, // Added ClientId, DisconnectReason
            protocol::{wl_output, wl_surface::WlSurface},
            Display, DisplayHandle,
        },
    },
    utils::{Clock, Logical, Point, Rectangle, SERIAL_COUNTER},
    wayland::{
        compositor::{CompositorHandler, CompositorState, CompositorClientState}, // Added CompositorHandler
        data_device::{DataDeviceHandler, DataDeviceState}, // Added DataDeviceHandler
        output::OutputManagerState, // This is here, but DesktopState uses OutputStateSmithay
        shell::xdg::{self as smithay_xdg_shell, XdgShellHandler, XdgShellState, XdgSurface}, // Added smithay_xdg_shell alias, XdgShellHandler, XdgSurface
        shm::{ShmHandler, ShmState}, // Added ShmHandler
        socket::ListeningSocketSource,
    },
    delegate_xdg_shell::{self, PopupManagerHandler}, // Added delegate_xdg_shell, PopupManagerHandler
    xdg::popup::Popup, // Added Popup
};
use tracing::{debug, error, info, warn}; // Added debug
use wayland_server::backend::ClientData;

// Imports from existing code that might be needed by new structs or were missing contextually
use smithay::reexports::wayland_server::protocol::wl_shm::Format;
use smithay::backend::renderer::{Bind, DebugFlags, Frame, Offscreen, Renderer, Texture, Unbind};
use smithay::backend::renderer::element::{texture::TextureRenderBuffer, Element, Id, RenderElementState};
use smithay::backend::renderer::glow::GlowFrameRenderError;


// Dummy structs for NovaDEConfiguration and NovaDEDomainServices

/// Placeholder for NovaDE specific system configurations.
/// TODO: Define actual configuration fields and loading mechanisms.
pub struct NovaDEConfiguration;

/// Placeholder for NovaDE domain-specific services.
/// TODO: Define actual services like application management, etc.
pub struct NovaDEDomainServices;

/// Holds backend-specific data, primarily the renderer.
pub struct BackendData {
    /// The graphics renderer instance (e.g., GLES2).
    pub renderer: Gles2Renderer,
}

/// Central state for the NovaDE Wayland compositor.
///
/// This struct encapsulates all necessary Smithay states for Wayland protocol handling,
/// input/output management, window management (via `Space`), and rendering backend data.
/// It also includes placeholders for NovaDE-specific configurations and domain services.
pub struct DesktopState {
    /// Handle to the Wayland display, used to interact with the display server.
    pub display: Display<Self>,
    /// Handle to the Calloop event loop, used for managing event sources.
    pub event_loop_handle: calloop::Handle<'static, Self>,
    /// Manages Wayland compositor globals and surface creation.
    pub compositor_state: CompositorState,
    /// Manages XDG shell specific logic, including popups and window management events.
    pub xdg_shell_state: XdgShellState,
    /// Manages shared memory (SHM) buffers for client surfaces.
    pub shm_state: ShmState,
    /// Manages data transfer mechanisms like copy-paste and drag-and-drop.
    pub data_device_state: DataDeviceState,
    /// Manages Wayland outputs (monitors/displays).
    pub output_state: OutputStateSmithay, // Corrected type
    /// Manages input devices and their state.
    pub input_state: InputStateSmithay, // Corrected type
    /// Represents a user's seat, grouping input devices (keyboard, pointer).
    pub seat: Seat<Self>,
    /// Manages XDG popups and their positioning relative to parent surfaces.
    pub popups: PopupManager,
    /// Backend-specific data, like the renderer, wrapped in Arc<Mutex> for shared access.
    pub backend_data: Arc<Mutex<BackendData>>,
    /// Manages the layout and stacking of windows (surfaces) in the compositor.
    pub space: Space<Window>,
    // TODO: Integrate NovaDE specific configuration.
    // /// NovaDE specific configurations.
    // pub config: NovaDEConfiguration,
    // TODO: Integrate NovaDE domain services.
    // /// NovaDE specific domain services.
    // pub domain_services: NovaDEDomainServices,
}

impl DesktopState {
    /// Creates and initializes a new `DesktopState`.
    ///
    /// This method sets up all the necessary Smithay protocol states,
    /// initializes input devices (seat with pointer and keyboard),
    /// prepares the rendering backend data (GLES2 renderer), and
    /// configures other desktop components like `Space` and `PopupManager`.
    ///
    /// # Parameters
    /// - `event_loop`: A mutable reference to the Calloop event loop.
    ///                 The event loop handle will be stored in `DesktopState`.
    /// - `display`: The Wayland `Display` object, which will be stored and managed by `DesktopState`.
    ///
    /// # Panics
    /// This method might panic if `Gles2Renderer::new()` fails, for example,
    /// if no suitable GLES2 context can be created. This is indicated by `.expect()`.
    /// It also panics if `seat.add_keyboard()` fails.
    pub fn new(event_loop: &mut EventLoop<'static, DesktopState>, mut display: Display<Self>) -> Self {
        info!("Initializing NovaDE DesktopState");

        let display_handle = display.handle();

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        // Initialize SHM with default formats by passing an empty vec
        let shm_state = ShmState::new::<Self>(&display_handle, Vec::new());
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        let output_state = OutputStateSmithay::new();
        let input_state = InputStateSmithay::new();

        let seat_name = "seat0";
        let mut seat = Seat::new(&display_handle, seat_name.to_string());

        seat.add_pointer();
        seat.add_keyboard(XkbConfig::default(), 200, 25)
            .expect("Failed to create keyboard");

        let popups = PopupManager::new();
        let space = Space::new(info_span!("space"));

        // Initialize BackendData with Gles2Renderer
        // This might panic if GLES2 context is not available.
        // The issue implies to proceed with expect for now.
        let renderer = Gles2Renderer::new().expect("Failed to create GLES2 renderer");
        let backend_data = Arc::new(Mutex::new(BackendData { renderer }));

        info!("NovaDE DesktopState initialized successfully");

        Self {
            display,
            event_loop_handle: event_loop.handle(),
            compositor_state,
            xdg_shell_state,
            shm_state,
            data_device_state,
            output_state,
            input_state,
            seat,
            popups,
            backend_data,
            space,
            // config: NovaDEConfiguration, // Placeholder
            // domain_services: NovaDEDomainServices, // Placeholder
        }
    }
}

/// Handles Wayland compositor protocol events and manages `CompositorState`.
impl CompositorHandler for DesktopState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    /// Called when a Wayland client disconnects.
    fn client_disconnected(&mut self, client: ClientId, reason: DisconnectReason) {
        warn!(?client, ?reason, "Wayland client disconnected.");
        // TODO: Add cleanup logic for client resources.
    }
}

/// Handles Wayland data device (clipboard, drag-and-drop) events and manages `DataDeviceState`.
impl DataDeviceHandler for DesktopState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

/// Handles Wayland shared memory (SHM) events and manages `ShmState`.
impl ShmHandler for DesktopState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

/// Handles XDG Shell (window management) protocol events and manages `XdgShellState`.
impl smithay_xdg_shell::XdgShellHandler for DesktopState {
    fn xdg_shell_state(&self) -> &smithay_xdg_shell::XdgShellState {
        &self.xdg_shell_state
    }

    /// Called when a new XDG surface (e.g., a window) is created by a client.
    fn new_xdg_surface(&mut self, surface: smithay_xdg_shell::XdgSurface) {
        info!("Neues XDG-Surface erstellt: {:?}", surface.wl_surface());
        // TODO: Add logic to manage the new XDG surface (e.g., add to Space, focus management).
    }

    /// Called when a client requests an icon for an XDG surface.
    fn xdg_shell_icon(&self) -> Option<WlSurface> {
        None // Placeholder: Implement if icon support is desired.
    }
}

/// Handles output (display/monitor) related events and manages `OutputStateSmithay`.
impl OutputHandler for DesktopState {
    fn output_state(&mut self) -> &mut OutputStateSmithay {
        &mut self.output_state
    }

    /// Called when a new output is added to the compositor.
    fn new_output(&mut self, output: Output) {
        info!("Neuer Output hinzugefügt: {:?}", output.name());
        // TODO: Configure and manage the new output (e.g., add to Space, set mode).
    }

    /// Called when an output is removed from the compositor.
    fn output_destroyed(&mut self, output: Output) {
        info!("Output entfernt: {:?}", output.name());
        // TODO: Cleanup resources associated with the removed output.
    }
}

/// Handles input device related events and manages `InputStateSmithay` and `Seat`s.
impl InputHandler for DesktopState {
    fn input_state(&mut self) -> &mut InputStateSmithay {
        &mut self.input_state
    }

    /// Returns the name of the primary seat.
    fn seat_name(&self) -> String {
        self.seat.name().to_string()
    }

    /// Called when a new seat is added (though typically there's one primary seat).
    fn new_seat(&mut self, seat: Seat<Self>) {
        info!("Neuer Seat hinzugefügt: {}", seat.name());
    }

    /// Called when a seat is removed.
    fn seat_removed(&mut self, seat: Seat<Self>) {
        info!("Seat entfernt: {}", seat.name());
    }
}

/// Handles events related to XDG popups, part of the XDG Shell delegation.
impl PopupManagerHandler for DesktopState {
    /// Called when a grab is requested for an XDG popup.
    fn grab_xdg_popup(&mut self, popup: &Popup) -> Option<GrabStartData> {
        debug!("XDG Popup Grab angefordert: {:?}", popup.wl_surface());
        None // Default behavior: No special grab handling for now.
    }
}

// Smithay delegate macros
smithay::delegate_compositor!(DesktopState);
smithay::delegate_data_device!(DesktopState);
smithay::delegate_shm!(DesktopState);
smithay::delegate_xdg_shell!(DesktopState); // Handles XdgShellHandler and PopupManagerHandler
smithay::delegate_output!(DesktopState);
smithay::delegate_input!(DesktopState);

// Existing code starts here
use smithay::{
    backend::egl::Egl,
    backend::renderer::gles2::Gles2Renderer,
    desktop::{Space, Window},
    reexports::calloop::LoopHandle,
    reexports::wayland_server::{Display, protocol::wl_surface::WlSurface},
    wayland::{
        compositor::CompositorState,
        output::OutputManagerState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        seat::{SeatState, Seat, CursorImageStatus},
        dmabuf::DmabufState,
    },
    backend::{
        // drm::{DrmDevice, DrmDisplay, DrmNode, DrmSurface}, // TODO: Integrate with generic FrameRenderer for DRM
        drm::{DrmNode}, // DrmSurface and DrmDevice commented out for now
        // renderer::gles::{GlesRenderer, GlesTexture}, // GLES specific, removed
        session::Session,
    },
    // reexports::drm::control::crtc, // Related to DrmDevice/Display
    utils::{Rectangle, Physical, Point, Logical, Transform}, // Added Transform
};
use std::{cell::RefCell, collections::HashMap, time::SystemTime, sync::{Arc, Mutex}}; // Added Arc, Mutex
use smithay::wayland::compositor as smithay_compositor;
use tracing::{debug_span, error, info_span, trace, warn};

use crate::compositor::render::gl::{init_gl_renderer, GlInitError};

// TODO: Define these traits properly in a new renderer module (e.g., src/renderer/mod.rs)
// For now, these are conceptual placeholders.

// TODO NovaDE-Refactor: The following code, including NovadeCompositorState, RenderableTexture,
// FrameRenderer, and related components, was part of an earlier iteration.
// With the introduction of DesktopState as the primary compositor state manager,
// this older code will need to be reviewed, refactored to integrate with DesktopState,
// or deprecated if its functionality is fully superseded.
// For now, it is preserved to avoid breaking other parts of the system that might still depend on it.
/// Represents a texture that can be rendered.
/// This trait would be implemented by backend-specific texture types (e.g., WgpuTexture, GlesTextureWrapper).
pub trait RenderableTexture: std::fmt::Debug + Send + Sync {
    // fn id(&self) -> TextureId; // Example method
    // fn size(&self) -> Size<i32, Physical>; // Example method
}

/// Represents a frame renderer that can draw a set of elements to a target.
/// This trait would be implemented by different rendering backends (e.g., WgpuRenderer, GlesFrameRenderer).
pub trait FrameRenderer: Send + Sync {
    /// Creates a new texture from SHM buffer data.
    /// This will be called when a client attaches and commits a new SHM buffer to a surface.
    fn create_texture_from_shm(&mut self, buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer) -> Result<Box<dyn RenderableTexture>, RendererError>; // Error type changed to RendererError

    /// Creates a new texture from DMABUF data.
    /// This will be called when a client attaches and commits a new DMABUF buffer to a surface.
    fn create_texture_from_dmabuf(&mut self, dmabuf: &smithay::backend::allocator::dmabuf::Dmabuf) -> Result<Box<dyn RenderableTexture>, RendererError>;

    /// Renders a frame composed of multiple elements.
    ///
    /// # Arguments
    /// * `output_size`: The physical size of the output target.
    /// * `output_scale`: The scale factor of the output.
    /// * `elements`: A slice of `RenderElement`s to draw.
    /// * `clear_color`: The background color to clear the frame with.
    ///
    /// Returns `Ok(Vec<Rectangle<i32, Physical>>)` representing the damage regions that were actually rendered,
    /// or `Err(String)` if rendering failed.
    fn render_frame(
        &mut self,
        output_size: smithay::utils::Size<i32, Physical>, // Example: from DRM mode or Winit window size
        output_scale: f64, // Example: from Output scale or Winit scale factor
        elements: &[RenderElement],
        clear_color: [f32; 4],
    ) -> Result<Vec<Rectangle<i32, Physical>>, String>; // Returns damage rendered. Error type String for now.

    // fn present_frame(&mut self) -> Result<(), String>; // Example method for backends that need explicit presentation step
}

// TODO: Define RendererError properly. For now, it's a placeholder. It should be defined in the same module as FrameRenderer or be accessible.
// For the purpose of this change, we assume it's defined elsewhere and is compatible.
// use crate::renderer::RendererError; // Assuming RendererError is defined in a top-level renderer module

use uuid::Uuid; // Ensure Uuid is imported if used in the trait

/// Represents a texture that can be rendered by the generic FrameRenderer.
/// It allows for downcasting to concrete renderer-specific texture types.
pub trait RenderableTexture: std::fmt::Debug + Send + Sync + std::any::Any {
    /// Returns a unique identifier for the texture.
    fn id(&self) -> Uuid; // Or String, but Uuid was used in WgpuRenderableTexture

    /// Returns the width of the texture in pixels.
    fn width(&self) -> u32;

    /// Returns the height of the texture in pixels.
    fn height(&self) -> u32;

    /// Helper method to get the texture as `Any` for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;

    // Note: OpenGL-specific `bind()` method is removed.
    // `format()` returning Fourcc is also removed for now, can be added if needed by generic logic.
}

/// Placeholder for RendererError if not defined elsewhere.
/// In a real scenario, this would be a proper error enum for the renderer module.
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Unsupported pixel format: {0}")]
    UnsupportedPixelFormat(String),
    #[error("Invalid buffer type: {0}")]
    InvalidBufferType(String),
    #[error("Buffer swap failed: {0}")]
    BufferSwapFailed(String),
    #[error("Renderer is unsupported: {0}")]
    Unsupported(String),
    #[error("Generic renderer error: {0}")]
    Generic(String),
}


/// Enum representing different types of elements that can be rendered.
/// This will be passed to the `FrameRenderer`.
#[derive(Debug)]
pub enum RenderElement<'a> {
    Surface {
        surface: &'a WlSurface, // Reference to the Wayland surface
        texture: Arc<dyn RenderableTexture>, // Changed Box to Arc
        location: Point<i32, Logical>,
        size: smithay::utils::Size<i32, Logical>,
        transform: Transform, // wl_surface.transform
        damage: Vec<Rectangle<i32, Logical>>, // Damage in logical surface coordinates
    },
    Cursor {
        texture: Arc<dyn RenderableTexture>, // Changed Box to Arc
        location: Point<i32, Logical>, // Top-left position of the cursor (hotspot already applied)
        hotspot: (i32, i32), // Original hotspot, for reference or if renderer needs it
    },
    SolidColor { // Added SolidColor variant
        color: [f32; 4], // RGBA
        geometry: Rectangle<i32, Logical>, // Position and size in logical coordinates
    }
    // Could add other element types like XWayland windows, custom UI elements, etc.
}


/// Custom data associated with each `WlSurface`.
///
/// This struct stores rendering-specific data for a Wayland surface.
#[derive(Default, Debug)]
pub struct SurfaceDataExt {
    /// The backend-specific texture handle for the surface's buffer.
    pub texture_handle: Option<Arc<dyn RenderableTexture>>,
    /// Buffer damage information, typically in physical pixel coordinates.
    /// This might need to be re-evaluated for how it interacts with generic rendering.
    pub damage_buffer: Vec<Rectangle<i32, Physical>>,
    // TODO: wl_surface.attach will need to call frame_renderer.create_texture_from_shm()
    // and store the result in texture_handle. This should be done in the CompositorHandler logic.
}

// OutputRenderState is removed as it was GLES-specific.
// The FrameRenderer will manage its own per-output resources if needed (e.g., WGPU render targets).

/// Central state for the Novade Wayland compositor.
pub struct NovadeCompositorState {
    /// Handle to the Wayland display, used for interacting with clients and globals.
    pub display_handle: Display<Self>,
    /// Handle to the Calloop event loop, used for managing event sources.
    pub loop_handle: LoopHandle<'static, Self>,

    // Core Wayland protocol states
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<Self>,

    // Desktop management
    pub space: Space<Window>,

    // Input related
    pub seat: Seat<Self>,
    pub seat_name: String,

    // Graphics backend related
    /// The generic frame renderer. Initialized by the backend (Winit or DRM).
    pub frame_renderer: Option<Arc<Mutex<dyn FrameRenderer>>>,
    /// The session backend (e.g., `DirectSession` for DRM direct launch).
    pub session: smithay::backend::session::direct::DirectSession, // TODO: This might be backend specific (DRM)
    /// The primary DRM node used by the compositor.
    pub primary_drm_node: DrmNode, // TODO: This is DRM specific. How to handle for Winit?
    /// The GLES2 Renderer for hardware-accelerated rendering.
    pub gl_renderer: Gles2Renderer,

    // TODO: DRM Display and Surface Management - These are GLES and DRM specific.
    // Re-evaluate how DRM integration will work with a generic FrameRenderer.
    // pub drm_device: DrmDevice<GlesRenderer>,
    // pub drm_display: DrmDisplay<GlesRenderer>,
    // pub output_render_states: HashMap<crtc::Handle, OutputRenderState>, // Removed

    // DMABuf State
    pub dmabuf_state: DmabufState, // TODO: DMABuf import will need to interact with FrameRenderer

    // Cursor related
    pub cursor_image_status: Option<CursorImageStatus>,
    // pub cursor_texture: Option<GlesTexture>, // Removed, FrameRenderer will manage cursor texture
    /// Hotspot of the current cursor image.
    pub cursor_hotspot: (i32, i32),
    /// Current logical position of the pointer.
    pub pointer_location: Point<f64, Logical>,
}

impl NovadeCompositorState {
    /// Creates a new `NovadeCompositorState`.
    pub fn new(
        display_handle: Display<Self>,
        loop_handle: LoopHandle<'static, Self>,
        // gles_renderer: GlesRenderer, // Removed
        session: smithay::backend::session::direct::DirectSession, // TODO: Make session handling generic or backend-specific
        primary_drm_node: DrmNode, // TODO: DRM-specific, move to DRM backend initialization
        // drm_device: DrmDevice<GlesRenderer>, // Removed
        // drm_display: DrmDisplay<GlesRenderer>, // Removed
        // output_render_states: HashMap<crtc::Handle, OutputRenderState>, // Removed
        dmabuf_state: DmabufState,
    ) -> Result<Self, GlInitError> {
        info!("Beginne EGL- und Gles2Renderer-Initialisierung für NovadeCompositorState...");

        let egl = Egl::new().map_err(|egl_err| {
            error!("EGL-Initialisierung fehlgeschlagen: {}", egl_err);
            GlInitError::from(egl_err) // Relies on From<smithay::backend::egl::Error> for GlInitError
        })?;
        info!("EGL erfolgreich initialisiert.");

        let gl_renderer = init_gl_renderer(egl).map_err(|render_err| {
            error!("Gles2Renderer-Initialisierung fehlgeschlagen: {}", render_err);
            // init_gl_renderer already returns GlInitError, so this assignment is direct.
            render_err
        })?;
        info!("Gles2Renderer erfolgreich initialisiert und in NovadeCompositorState integriert.");

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        
        let shm_formats = vec![ // These are generally supported, FrameRenderer impls will handle them
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Argb8888,
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Xrgb8888,
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Abgr8888,
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Xbgr8888,
        ];
        let shm_state = ShmState::new::<Self>(&display_handle, shm_formats);
        // TODO: In wl_surface.commit for SHM, call self.frame_renderer.create_texture_from_shm()
        // and store the result in SurfaceDataExt.texture_handle.
        
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        
        let mut seat_state = SeatState::new();
        let seat_name = "novade_seat_0".to_string();
        let seat = seat_state.new_wl_seat(&display_handle, seat_name.clone());
        
        let space = Space::new(tracing::info_span!("space"));

        Self {
            display_handle,
            loop_handle,
            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            space,
            seat,
            seat_name,
            frame_renderer: None, // Initialized later by the backend
            session, // TODO: Re-evaluate session field for non-DRM backends
            primary_drm_node, // TODO: Re-evaluate primary_drm_node for non-DRM backends
            gl_renderer,
            // drm_device, // Removed
            // drm_display, // Removed
            // output_render_states, // Removed
            dmabuf_state, // TODO: DMABuf import will need FrameRenderer interaction
            cursor_image_status: None,
            // cursor_texture: None, // Removed
            cursor_hotspot: (0, 0),
            pointer_location: Point::from((0.0, 0.0)),
        })
    }
}

// Render function
impl NovadeCompositorState {
    /// Prepares and renders a frame using the generic `FrameRenderer`.
    ///
    /// This method collects all visible surface elements and cursor information,
    /// then delegates the actual rendering to the `frame_renderer`.
    ///
    /// # Arguments
    ///
    /// * `background_color`: A `[f32; 4]` array representing the RGBA color for the background.
    /// * `output_size`: The physical size of the output target (e.g., window size or mode size).
    /// * `output_scale`: The scale factor of the output.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>`: `Ok(())` if rendering was successful or if no renderer is present.
    ///   Returns an `Err(String)` for critical rendering failures.
    pub fn render_frame(
        &mut self,
        background_color: [f32; 4],
        output_size: smithay::utils::Size<i32, Physical>, // Passed in by backend (Winit or DRM)
        output_scale: f64, // Passed in by backend
    ) -> Result<(), String> {
        let frame_renderer_arc = match &self.frame_renderer {
            Some(renderer) => renderer.clone(),
            None => {
                // warn!("render_frame called but no frame_renderer is initialized.");
                return Ok(()); // Not an error, but nothing to do.
            }
        };
        let mut frame_renderer = frame_renderer_arc.lock().unwrap();

        let mut render_elements = Vec::new();

        // Collect surface elements
        let elements_to_render = self.space.elements().cloned().collect::<Vec<_>>();
        for element_window in &elements_to_render {
            if let Some(wl_surface) = element_window.wl_surface() {
                if !wl_surface.is_alive() {
                    trace!("Surface not alive");
                    continue;
                }
                if let Some(sde_ref) = wl_surface.get_data::<RefCell<SurfaceDataExt>>() {
                    let sde = sde_ref.borrow();
                    if let Some(texture_handle) = &sde.texture_handle {
                        if let Some(geo) = self.space.element_geometry(element_window) {
                            let location = geo.loc;
                            let size = geo.size;
                            let transform = self.compositor_state.get_surface_transformation(wl_surface);
                            // TODO: Damage tracking needs to be adapted.
                            // For now, assume full damage for simplicity or use sde.damage_buffer if compatible.
                            // The damage passed to RenderElement should be in logical surface coordinates.
                            let damage = vec![Rectangle::from_loc_and_size((0,0), size)]; // Placeholder: full damage

                            render_elements.push(RenderElement::Surface {
                                surface: wl_surface,
                                texture: texture_handle.clone(),
                                location,
                                size,
                                transform,
                                damage,
                            });
                        } else { trace!("No geometry for element"); }
                    } else { trace!("No texture_handle for surface"); }
                } else { warn!("No SurfaceDataExt for surface"); }
            }
        }

        // Collect cursor element
        // TODO: The cursor texture itself needs to be created and managed by the FrameRenderer
        // when the cursor is set via `SeatHandler::set_cursor`.
        // For now, if we have a cursor image status, we'd ideally pass necessary info.
        // This part needs careful integration with how FrameRenderer handles cursor textures.
        // For this refactor, we'll assume the FrameRenderer is notified of cursor changes elsewhere
        // and can retrieve the cursor texture if needed, or it's passed explicitly.
        // Let's simulate creating a conceptual cursor element if a cursor is set.
        // The actual `Arc<dyn RenderableTexture>` for the cursor would be managed by the FrameRenderer
        // and potentially stored in NovadeCompositorState if easily accessible.
        // For now, we won't add a cursor_texture_handle field to NovadeCompositorState,
        // assuming the FrameRenderer can manage this.
        // The `render_frame` will just signal that a cursor *should* be rendered.
        // A more robust solution might involve the FrameRenderer exposing a method to get the current cursor texture.

        if self.cursor_image_status.is_some() {
            // Placeholder: In a real scenario, the FrameRenderer would have created/cached this texture.
            // We'd need a way to get it here, or the FrameRenderer handles cursor rendering internally.
            // For now, let's assume the FrameRenderer is aware of the cursor state.
            // The `render_frame` call below will pass the elements, and the renderer can decide
            // if/how to draw a cursor based on its own state or a specific Cursor element.
            // To make it explicit, let's conceptualize that a cursor texture is available.
            // This implies that when `SeatHandler::set_cursor` is called, it updates not only
            // `cursor_image_status` but also tells the `FrameRenderer` to prepare/cache the texture.

            // To fit the RenderElement::Cursor structure, we need an Arc<dyn RenderableTexture>.
            // This is a bit of a chicken-and-egg problem for this specific refactor step
            // as `set_cursor` (which would create the texture via FrameRenderer) is not being modified here.
            // For now, we will OMIT adding the cursor to `render_elements` directly in this pass,
            // and assume the `FrameRenderer::render_frame` itself will query `NovadeCompositorState`
            // or have its own state for the cursor. The `RenderElement::Cursor` is defined for future use.
            // The alternative would be to add `cursor_texture_handle: Option<Arc<dyn RenderableTexture>>` to NovadeCompositorState,
            // updated by the `set_cursor` handler via the `FrameRenderer`.
            // Let's add a TODO here to revisit cursor rendering with FrameRenderer.
            // TODO: Revisit cursor rendering. The FrameRenderer should manage the cursor texture.
            // The `render_frame` function should ideally pass a `RenderElement::Cursor` if a cursor is active.
            // This requires `set_cursor` to interact with `FrameRenderer` to create the texture.
        // For now, we assume that if a cursor is visible and an active_cursor_texture exists (managed by DesktopState and SeatHandler),
        // we should try to render it.
        // TODO: Resolve access to DesktopState::active_cursor_texture from NovadeCompositorState::render_frame.
        // For this subtask, we'll proceed by checking self.cursor_image_status (from NovadeCompositorState)
        // and assume that if it indicates a visible cursor, a corresponding texture *would* be available
        // conceptually, even if we can't directly access DesktopState.active_cursor_texture here without API changes.
        // This means the actual texture for RenderElement::Cursor might be missing for now in this specific method.
        // The correct place to get active_cursor_texture is DesktopState.
        // Let's assume for the purpose of constructing the RenderElement, if status is visible, we'd get it.
        // This part of the code will be more illustrative of intent rather than fully functional without state access changes.

        // TODO: NovadeCompositorState::render_frame signature should be updated to accept
        // active_cursor_texture, pointer_location, and cursor_hotspot from DesktopState.
        // For now, we use self.pointer_location, self.cursor_hotspot, and self.cursor_image_status
        // which are fields of NovadeCompositorState.
        // We will assume active_cursor_texture_from_desktop_state: Option<Arc<dyn RenderableTexture>> is passed in.
        
        // Conceptual: These would be passed as arguments or accessed via a shared state reference.
        // let active_cursor_texture_from_desktop_state: Option<Arc<dyn RenderableTexture>> = ...;
        // let current_pointer_location_from_desktop_state: Point<f64, Logical> = self.pointer_location; // Use self for now
        // let current_cursor_hotspot_from_desktop_state: Point<i32, Logical> = self.cursor_hotspot.into(); // Use self for now

        // This is a placeholder for how the actual texture would be passed in.
        // In a real implementation, this Option<Arc<...>> would come from DesktopState.
        let active_cursor_texture_param: Option<Arc<dyn RenderableTexture>> = None; // Replace with actual parameter later

        if self.cursor_image_status.as_ref().map_or(false, |s| !matches!(s, CursorImageStatus::Hidden)) {
            if let Some(texture_arc) = active_cursor_texture_param { // Use the conceptual parameter
                 render_elements.push(RenderElement::Cursor {
                    texture: texture_arc.clone(), // Clone the Arc for the RenderElement
                    location: self.pointer_location.to_i32_round(),
                    hotspot: self.cursor_hotspot,
                });
                tracing::debug!("Added RenderElement::Cursor to render list. Hotspot: {:?}, Location: {:?}", self.cursor_hotspot, self.pointer_location.to_i32_round());
            } else {
                // If there's no texture but cursor is visible by status, it implies SeatHandler failed to create one,
                // or the parameter passing isn't implemented yet.
                tracing::warn!("Cursor is visible by status, but no active_cursor_texture was provided to render_frame.");
            }
        }
        }

        // Perform rendering
        let render_span = info_span!("renderer_render_frame", output_size = ?output_size, num_elements = render_elements.len());
        let _render_guard = render_span.enter();

        match frame_renderer.render_frame(output_size, output_scale, &render_elements, background_color) {
            Ok(_rendered_damage) => {
                // Frame callbacks are tricky with a generic renderer if it's fully async or handles presentation internally.
                // For now, assume presentation is somewhat synchronized with this call or handled by the backend.
                // The DRM backend used to handle page flips and then send callbacks.
                // The Winit backend might use `request_redraw` and presentation is handled by the event loop.
                // This part needs to be adapted based on the specific backend's FrameRenderer implementation.
                // TODO: Clarify how frame callbacks are handled with generic FrameRenderer.
                // For now, we will mimic the previous behavior of sending callbacks immediately after a successful render call.
                // This might not be accurate for all backends.
                
                let time_ms = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u32;

                for element_window in &elements_to_render {
                    if let Some(wl_surface) = element_window.wl_surface() {
                        if wl_surface.is_alive() {
                            // Use smithay_compositor::SurfaceData for frame callbacks
                            if let Some(data_refcell) = wl_surface.data_map().get::<RefCell<smithay_compositor::SurfaceData>>() {
                                let mut surface_data_inner = data_refcell.borrow_mut();
                                if !surface_data_inner.frame_callbacks.is_empty() {
                                    trace!(parent: &render_span, "Sending frame callbacks for surface {:?}", wl_surface.id());
                                    for callback in surface_data_inner.frame_callbacks.drain(..) {
                                        callback.done(time_ms);
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                error!(parent: &render_span, target: "Renderer", "FrameRenderer failed to render frame: {}", e);
                Err(format!("FrameRenderer failed: {}", e))
            }
        }
    }
}

// Delegate macros will be implemented in main.rs or a specific handlers file.
// NovadeCompositorState will need to implement various Smithay handler traits.
// Example TODO for CompositorHandler:
// impl CompositorHandler for NovadeCompositorState {
//     fn compositor_state(&mut self) -> &mut CompositorState { &mut self.compositor_state }
//     fn commit(&mut self, surface: &WlSurface) {
//         // Existing logic from smithay examples...
//         // Buffer attachment handling:
//         if let Some(buffer) = smithay::wayland::compositor::get_buffer(surface) {
//             // TODO: This is where SHM buffer import should happen using FrameRenderer
//             // if self.frame_renderer.is_some() && shm_attributes.is_some() {
//             //    let mut renderer = self.frame_renderer.as_mut().unwrap().lock().unwrap();
//             //    match renderer.create_texture_from_shm(&buffer) {
//             //        Ok(texture_handle) => {
//             //            let sde = surface.get_data::<RefCell<SurfaceDataExt>>().unwrap();
//             //            sde.borrow_mut().texture_handle = Some(texture_handle);
//             //        }
//             //        Err(err) => { error!("Failed to create texture from SHM buffer: {}", err); }
//             //    }
//             // }
//
//             // TODO: DMABUF import logic would also go here, using FrameRenderer to create textures.
//         }
//         // Damage tracking logic...
//     }
//     // Other required methods...
// }
// This setup is usually done where the event loop and state are managed together.
