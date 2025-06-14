use smithay::{
    delegate_compositor, delegate_damage_tracker, delegate_dmabuf, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell, delegate_xdg_decoration, delegate_screencopy, // Added screencopy
    reexports::{
        calloop::{EventLoop, LoopHandle},
        wayland_server::{
            backend::ClientData,
            protocol::{
                wl_buffer::WlBuffer, wl_compositor::WlCompositor, wl_shm::WlShm,
                wl_subcompositor::WlSubcompositor, wl_surface::WlSurface,
            },
            Client, DataInit, Display, DisplayHandle, GlobalDispatch, New, Resource,
        },
    },
    utils::{Clock, Logical, Point, Rectangle, Buffer as SmithayBuffer},
    wayland::{
        compositor::{
            self, add_destruction_hook, CompositorClientState, CompositorHandler, CompositorState,
            SurfaceAttributes as WlSurfaceAttributes, SubsurfaceRole,
        },
        output::OutputManagerState,
        shm::{BufferHandler, ShmHandler, ShmState},
        shell::xdg::XdgShellState,
        shell::xdg::decoration::XdgDecorationState, 
        screencopy::ScreencopyState, // Added ScreencopyState
        dmabuf::DmabufState, 
    },
    backend::renderer::utils::buffer_dimensions,
    desktop::{Space, DamageTrackerState},
    output::Output,
    input::{Seat, SeatHandler, SeatState, pointer::CursorImageStatus, keyboard::KeyboardHandle, touch::TouchSlotId}, // Added KeyboardHandle and TouchSlotId
    reexports::wayland_server::protocol::wl_surface::{WlSurface, Weak}, // Added WlSurface, Weak
    wayland::seat::WaylandSeatData, // Added WaylandSeatData
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex, Mutex}, // Added Mutex for VulkanFrameRenderer
    time::Instant,
};
use crate::input::keyboard::xkb_config::XkbKeyboardData; // Added XkbKeyboardData
use crate::compositor::surface_management::{AttachedBufferInfo, SurfaceData}; 
use crate::compositor::core::ClientCompositorData;
use crate::compositor::shell::xdg_shell::types::{DomainWindowIdentifier, ManagedWindow};
use novade_domain::DomainServices;
use crate::input::input_dispatcher::InputDispatcher;
use crate::input::keyboard_layout::KeyboardLayoutManager;
use crate::renderer::wgpu_renderer::NovaWgpuRenderer;
use crate::compositor::renderer_interface::abstraction::FrameRenderer; // Added FrameRenderer import
use crate::display_management::WaylandDisplayManager;

mod input_handlers; // Added module declaration
mod output_handlers; // Added module declaration for output handlers
mod screencopy_handlers; // Added module declaration for screencopy handlers

// Vulkan specific imports
use crate::compositor::renderer::vulkan::{
    instance::VulkanInstance,
    physical_device::PhysicalDeviceInfo,
    logical_device::LogicalDevice,
    allocator::Allocator, // Assuming this is the correct path for the Vulkan allocator
    frame_renderer::FrameRenderer as VulkanFrameRenderer,
};


// --- Imports for MCP and CPU Usage Service ---
use tokio::sync::Mutex as TokioMutex; // Using tokio's Mutex for async services
use novade_domain::ai_interaction_service::MCPConnectionService;
use novade_domain::cpu_usage_service::ICpuUsageService;
// Consider if mcp_client_spawner also needs to be stored
// use crate::mcp_client_service::IMCPClientService;

/// Represents the active renderer type being used by the compositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveRendererType {
    /// GLES2 renderer is active.
    Gles,
    /// Vulkan renderer is active (currently GLES-on-Vulkan interop or placeholder).
    Vulkan,
    /// WGPU renderer is active.
    Wgpu,
}

// Main compositor state
pub struct DesktopState {
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, Self>,
    pub clock: Clock<u64>,
    pub compositor_state: CompositorState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    // pub gles_renderer: Option<crate::compositor::renderers::gles2::renderer::Gles2Renderer>, // Removed
    pub xdg_shell_state: XdgShellState,
    pub space: Space<ManagedWindow>,
    pub windows: HashMap<DomainWindowIdentifier, Arc<ManagedWindow>>,
    pub outputs: Vec<Output>,
    pub last_render_time: Instant,
    pub damage_tracker_state: DamageTrackerState,
    pub seat_state: SeatState<Self>,
    pub seat_name: String,
    pub seat: Seat<Self>,
    pub pointer_location: Point<f64, Logical>,
    pub current_cursor_status: Arc<StdMutex<CursorImageStatus>>,
    pub dmabuf_state: DmabufState,
    pub xdg_decoration_state: XdgDecorationState,
    pub screencopy_state: ScreencopyState, // Added screencopy_state

    // Vulkan Renderer Components - REMOVED
    // pub vulkan_instance: Option<Arc<VulkanInstance>>,
    // pub vulkan_physical_device_info: Option<Arc<PhysicalDeviceInfo>>,
    // pub vulkan_logical_device: Option<Arc<LogicalDevice>>,
    // pub vulkan_allocator: Option<Arc<Allocator>>,
    // pub vulkan_frame_renderer: Option<Arc<Mutex<VulkanFrameRenderer>>>,
    
    /// Specifies which renderer is currently active.
    pub active_renderer_type: ActiveRendererType,
    pub active_renderer: Option<Arc<Mutex<dyn FrameRenderer>>>, // Unified active renderer

    // --- Added for MCP and CPU Usage Service ---
    pub mcp_connection_service: Option<Arc<TokioMutex<MCPConnectionService>>>,
    pub cpu_usage_service: Option<Arc<dyn ICpuUsageService>>,
    // pub mcp_client_spawner: Option<Arc<dyn IMCPClientService>>,
    pub domain_services: Option<std::sync::Arc<novade_domain::DomainServices>>,

    // --- Input Management ---
    pub input_dispatcher: InputDispatcher,
    pub keyboard_layout_manager: KeyboardLayoutManager,
    pub active_input_surface: Option<smithay::reexports::wayland_server::Weak<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>>,
    pub keyboard_data_map: std::collections::HashMap<String, crate::input::keyboard::xkb_config::XkbKeyboardData>,
    pub touch_focus_per_slot: HashMap<TouchSlotId, Weak<WlSurface>>,

    // --- WGPU Renderer ---
    // pub wgpu_renderer: Option<Arc<Mutex<NovaWgpuRenderer>>>, // Removed specific WGPU field
    // Adding concrete WGPU renderer for commit path as a temporary solution
    pub wgpu_renderer_concrete: Option<Arc<Mutex<NovaWgpuRenderer>>>,
    pub display_manager: Arc<WaylandDisplayManager>,

    // --- Cursor Rendering State ---
    // Use the RenderableTexture trait defined alongside FrameRenderer trait
    pub active_cursor_texture: Option<Arc<dyn crate::compositor::state::RenderableTexture>>, // Changed Box to Arc
    pub cursor_hotspot: Point<i32, Logical>,

    // --- Layer Shell State (Placeholder for Smithay 0.3.0) ---
    // Smithay 0.3.0 does not have built-in LayerShellState. This is a placeholder.
    // A full implementation would require manual protocol handling or a newer Smithay.
    pub layer_shell_data: MyCustomLayerShellData,
}

/// Placeholder for custom layer shell data management.
/// In a real scenario with manual protocol implementation, this would store
/// information about layer surfaces, their states, etc.
#[derive(Debug, Default)]
pub struct MyCustomLayerShellData {
    // Example: layers: Vec<Weak<WlSurface>>, // or a more complex struct per layer
    // For this placeholder, it's empty.
}


use crate::error::SystemResult; // For WaylandDisplayManager::new()

impl DesktopState {
    // #[allow(clippy::too_many_arguments)] // No longer needed
    pub fn new(
        loop_handle: LoopHandle<'static, Self>, // Changed from event_loop
        display_handle: DisplayHandle,
    ) -> SystemResult<Self> { // Return SystemResult
        // let loop_handle = event_loop.handle(); // loop_handle is now passed directly
        let clock = Clock::new(None).expect("Failed to create clock");

        let display_manager = Arc::new(WaylandDisplayManager::new()?);

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let space = Space::new(tracing::info_span!("novade_space"));
        let damage_tracker_state = DamageTrackerState::new();
        let mut seat_state = SeatState::new();
        let seat_name = "seat0".to_string();

        let input_dispatcher = InputDispatcher::new();
        let keyboard_layout_manager = KeyboardLayoutManager::new()
            .expect("Failed to initialize KeyboardLayoutManager");

        let seat = seat_state.new_wl_seat(&display_handle, seat_name.clone(), Some(tracing::Span::current()));
        seat.user_data().insert_if_missing(smithay::wayland::seat::WaylandSeatData::default);

        let mut keyboard_data_map = std::collections::HashMap::new();
        match crate::input::keyboard::xkb_config::XkbKeyboardData::new(&keyboard_layout_manager.xkb_config_cloned()) {
            Ok(xkb_data) => {
                keyboard_data_map.insert(seat.name().to_string(), xkb_data);
                tracing::info!("XkbKeyboardData initialized and added for seat: {}", seat.name());
            }
            Err(e) => {
                tracing::error!("Failed to initialize XkbKeyboardData for seat {}: {:?}", seat.name(), e);
                // Potentially, we might not want to panic here, but log and continue without XKB on this seat.
                // Or, if XKB is critical, then perhaps return an error from DesktopState::new.
                // For now, logging the error.
            }
        }

        // Configure the keyboard for the seat using KeyboardLayoutManager
        if let Err(e) = seat.add_keyboard(keyboard_layout_manager.xkb_config_cloned(), 200, 25) {
            tracing::warn!("Failed to add keyboard to seat with XKB config: {}", e);
        } else {
            tracing::info!("Added keyboard to seat '{}' with XKB config from KeyboardLayoutManager.", seat.name());
        }

        if let Err(e) = seat.add_pointer() {
            tracing::warn!("Failed to add pointer capability to seat in DesktopState::new: {}", e);
        }
        if let Err(e) = seat.add_touch() {
            tracing::warn!("Failed to add touch capability to seat in DesktopState::new: {}", e);
        }

        let dmabuf_state = DmabufState::new();
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle);
        let screencopy_state = ScreencopyState::new::<Self>(&display_handle, None); // Initialize ScreencopyState

        Self {
            display_handle,
            loop_handle,
            clock,
            compositor_state,
            shm_state,
            output_manager_state,
            // gles_renderer: None, // Removed
            xdg_shell_state,
            space,
            windows: HashMap::new(),
            outputs: Vec::new(),
            last_render_time: Instant::now(),
            damage_tracker_state,
            seat_state,
            seat_name,
            seat,
            pointer_location: (0.0, 0.0).into(),
            current_cursor_status: Arc::new(StdMutex::new(CursorImageStatus::Default)),
            dmabuf_state,
            xdg_decoration_state,
            screencopy_state, // Add to struct instantiation
            // vulkan_instance: None, // Removed
            // vulkan_physical_device_info: None, // Removed
            // vulkan_logical_device: None, // Removed
            // vulkan_allocator: None, // Removed
            // vulkan_frame_renderer: None, // Removed
            active_renderer_type: ActiveRendererType::Gles, // Default, backend should update
            active_renderer: None, // Initialize unified renderer as None
            // --- Initialize new service fields ---
            mcp_connection_service: None,
            cpu_usage_service: None,
            // mcp_client_spawner: None,
            domain_services: None,
            input_dispatcher,
            keyboard_layout_manager,
            // wgpu_renderer: None, // Removed specific field
            wgpu_renderer_concrete: None, // Initialize concrete WGPU renderer as None
            active_input_surface: None,
            keyboard_data_map,
            touch_focus_per_slot: HashMap::new(),
            display_manager,
            active_cursor_texture: None,
            cursor_hotspot: (0, 0).into(), // Initialize hotspot
            layer_shell_data: MyCustomLayerShellData::default(), // Initialize placeholder
        })
    }

    // TODO: This render_frame method is being added/adapted to DesktopState
    // to align with the subtask's request to use `self.active_cursor_texture` directly.
    // It mirrors the logic previously in `NovadeCompositorState::render_frame`.
    // The interaction between DesktopState and NovadeCompositorState regarding rendering
    // might need further review.
    pub fn render_frame(
        &mut self,
        background_color: [f32; 4],
        output_size: smithay::utils::Size<i32, Physical>,
        output_scale: f64,
    ) -> Result<(), String> {
        let frame_renderer_arc = match &self.active_renderer {
            Some(renderer) => renderer.clone(),
            None => {
                tracing::warn!("render_frame called on DesktopState but no active_renderer is initialized.");
                return Ok(());
            }
        };
        let mut frame_renderer_guard = frame_renderer_arc.lock().unwrap();

        let mut render_elements = Vec::new();

        // Collect surface elements from self.space
        // This requires ManagedWindow to provide wl_surface() and for SurfaceData to be accessible.
        // Assuming SurfaceData is in wl_surface's data_map and texture_handle is Arc<dyn RenderableTexture>.
        // The SurfaceData in surface_management.rs has texture_handle: Option<Box<dyn RenderableTexture>>
        // RenderElement::Surface expects Box<dyn RenderableTexture>. This is consistent.
        let space_elements = self.space.elements().cloned().collect::<Vec<_>>();
        for element_window in &space_elements { // element_window is Arc<ManagedWindow>
            if let Some(wl_surface) = element_window.wl_surface() { // Assuming ManagedWindow has wl_surface()
                if !wl_surface.is_alive() {
                    tracing::trace!("Surface not alive: {:?}", wl_surface.id());
                    continue;
                }
                // Accessing SurfaceData from compositor::surface_management
                if let Some(s_data_arc) = wl_surface.data_map().get::<Arc<StdMutex<crate::compositor::surface_management::SurfaceData>>>() {
                    let s_data = s_data_arc.lock().unwrap();
                    if let Some(texture_handle_box) = &s_data.texture_handle {
                        // RenderElement::Surface expects Box<dyn RenderableTexture>.
                        // We have a &Box here. To pass ownership, we'd need to clone the Box's content.
                        // This requires RenderableTexture to have a method for cloning into a Box.
                        // E.g., trait RenderableTexture: ... { fn box_clone(&self) -> Box<dyn RenderableTexture>; }
                        // And impl Clone for Box<dyn RenderableTexture> where T: RenderableTexture + Clone { ... }
                        // For now, this is a conceptual problem. If FrameRenderer takes &[RenderElement],
                        // we might not need to clone the Box itself, but the RenderElement construction needs a Box.
                        // Let's assume for the diff that we can get a suitable Box.
                        // The simplest way if the Box cannot be cloned is to not render surfaces in this pass
                        // or to change how RenderElement takes the texture (e.g. by reference if possible, or Arc).
                        // SurfaceData.texture_handle is Arc<dyn RenderableTexture>
                        // RenderElement::Surface expects Arc<dyn RenderableTexture>
                        // We can clone the Arc.
                        if let Some(geo) = self.space.element_geometry(element_window) {
                            // Need to get surface transformation properly.
                            // smithay::wayland::compositor::get_surface_transformation is not available directly here.
                            // CompositorState has this. self.compositor_state.get_surface_transformation(wl_surface)
                            let transform = self.compositor_state.get_surface_transformation(&wl_surface);
                            // TODO: Proper damage tracking for surfaces.
                            let damage = vec![Rectangle::from_loc_and_size((0,0), geo.size)]; // Placeholder: full damage

                            render_elements.push(crate::compositor::state::RenderElement::Surface {
                                surface: unsafe { &*(wl_surface.inner_ptr() as *const WlSurface) }, // Extending lifetime for 'a. Unsafe but common for render elements.
                                texture: texture_handle_box.clone(), // Clone Arc
                                location: geo.loc,
                                size: geo.size,
                                transform,
                                damage,
                            });
                            tracing::trace!("Added RenderElement::Surface for {:?}, geom: {:?}", wl_surface.id(), geo);
                        } else { tracing::trace!("No geometry for surface {:?}", wl_surface.id());}
                    } else { tracing::trace!("No texture_handle for surface {:?}", wl_surface.id()); }
                } else { tracing::warn!("No SurfaceData (compositor::surface_management) for surface {:?}", wl_surface.id()); }
            }
        }

        // Collect cursor element using DesktopState's fields
        let cursor_status_is_visible = self.current_cursor_status.lock().unwrap().is_visible();
        if cursor_status_is_visible {
            if let Some(cursor_texture_arc) = &self.active_cursor_texture { // This is Arc<dyn RenderableTexture>
                // RenderElement::Cursor expects Arc<dyn RenderableTexture>
                let cursor_element = crate::compositor::state::RenderElement::Cursor {
                    texture: cursor_texture_arc.clone(),
                    location: self.pointer_location.to_i32_round(),
                    hotspot: self.cursor_hotspot,
                };
                render_elements.push(cursor_element);
                tracing::trace!("Added RenderElement::Cursor to render list. Pos: {:?}, Hotspot: {:?}", self.pointer_location, self.cursor_hotspot);
            } else {
                tracing::trace!("Cursor is set to be visible, but no active_cursor_texture is available in DesktopState.");
            }
        }

        let render_span = tracing::info_span!("desktop_state_render_frame", output_size = ?output_size, num_elements = render_elements.len());
        let _render_guard = render_span.enter();

        match frame_renderer_guard.render_frame(output_size, output_scale, &render_elements, background_color) {
            Ok(_rendered_damage) => {
                // Handle frame callbacks - this logic might need to be adapted depending on where smithay_compositor::SurfaceData is.
                // Assuming space_elements contains items that have WlSurface.
                let time_ms = self.clock.now().try_into().unwrap_or_default(); // Use self.clock

                for element_window in &space_elements {
                    if let Some(wl_surface) = element_window.wl_surface() {
                        if wl_surface.is_alive() {
                            if let Some(data_refcell) = wl_surface.data_map().get::<std::cell::RefCell<smithay::wayland::compositor::SurfaceData>>() {
                                let mut surface_data_inner = data_refcell.borrow_mut();
                                if !surface_data_inner.frame_callbacks.is_empty() {
                                    tracing::trace!(parent: &render_span, "Sending frame callbacks for surface {:?}", wl_surface.id());
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
                tracing::error!(parent: &render_span, target: "Renderer", "DesktopState's FrameRenderer failed to render frame: {}", e);
                Err(format!("FrameRenderer failed: {}", e))
            }
        }
    }
}

impl CompositorHandler for DesktopState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client
            .get_data::<ClientCompositorData>()
            .expect("ClientCompositorData not initialized for this client.")
            .compositor_state // Directly access the public field
    }

    fn new_surface(&mut self, surface: &WlSurface) {
        let client_id = surface.client().expect("Surface must have a client.").id();
        tracing::info!(surface_id = ?surface.id(), ?client_id, "New WlSurface created");

        let client_id_str = format!("{:?}", client_id);
        let surface_data = Arc::new(StdMutex::new(
            crate::compositor::surface_management::SurfaceData::new(client_id_str),
        ));
        
        surface.data_map().insert_if_missing_threadsafe(move || surface_data);

        add_destruction_hook(surface, |data_map_of_destroyed_surface| {
            let surface_data_arc = data_map_of_destroyed_surface
                .get::<Arc<StdMutex<crate::compositor::surface_management::SurfaceData>>>()
                .expect("SurfaceData missing in destruction hook")
                .clone();
            
            let surface_id_for_log = { 
                let sd = surface_data_arc.lock().unwrap(); 
                sd.id
            };
            tracing::info!(
                "WlSurface with internal ID {:?} destroyed, SurfaceData cleaned up from UserDataMap.",
                surface_id_for_log
            );
        });
    }

    /// Handles buffer commits for a `WlSurface`.
    ///
    /// This method is called by Smithay when a client commits changes to a surface,
    /// potentially attaching a new buffer (`WlBuffer`). The core logic involves:
    ///
    /// 1.  **Accessing Surface State**: It uses `smithay::wayland::compositor::with_states`
    ///     to get thread-safe access to the surface's attributes and its associated
    ///     `SurfaceData` (from `crate::compositor::surface_management`).
    /// 2.  **Buffer Handling**:
    ///     - If a new `WlBuffer` is attached:
    ///         - It first attempts to import the buffer as a DMABUF by calling
    ///           `self.dmabuf_state.get_dmabuf_attributes(&wl_buffer)`.
    ///         - If successful (i.e., it's a DMABUF), it calls
    ///           `self.renderer.create_texture_from_dmabuf()` to import it into the
    ///           GLES2 renderer, creating a texture typically bound to `GL_TEXTURE_EXTERNAL_OES`.
    ///         - If it's not a DMABUF (or DMABUF import fails), it falls back to assuming
    ///           it's an SHM buffer and calls `self.renderer.create_texture_from_shm()`.
    ///         - The resulting texture (`Box<dyn RenderableTexture>`) is stored in
    ///           `surface_data.texture_handle`.
    ///         - Information about the attached buffer (dimensions, scale, transform) is stored in
    ///           `surface_data.current_buffer_info`.
    ///         - Appropriate logging is performed for success or failure of these operations.
    ///     - If a buffer is detached, `surface_data.texture_handle` and
    ///       `surface_data.current_buffer_info` are cleared.
    ///     - If the same buffer is re-committed with different attributes (e.g., scale, transform),
    ///       only `surface_data.current_buffer_info` is updated.
    /// 3.  **State Updates**: It updates damage tracking information (`surface_data.damage_buffer_coords`),
    ///     opaque region, and input region based on the committed surface attributes.
    ///
    /// This method is crucial for integrating client buffers into the compositor's rendering pipeline,
    /// supporting both DMABUF and SHM buffer types.
    fn commit(&mut self, surface: &WlSurface) {
        let client_info_for_commit = surface.client().map(|c| format!("{:?}", c.id())).unwrap_or_else(|| "<unknown_client>".to_string());
        tracing::debug!(surface_id = ?surface.id(), client_info = %client_info_for_commit, "Commit received for WlSurface");

        smithay::wayland::compositor::with_states(surface, |states| {
            let surface_data_arc = states
                .data_map
                .get::<Arc<StdMutex<crate::compositor::surface_management::SurfaceData>>>()
                .expect("SurfaceData missing on commit")
                .clone();
            
            let mut surface_data = surface_data_arc.lock().unwrap();
            let current_surface_attributes = states.cached_state.current::<WlSurfaceAttributes>();

            if current_surface_attributes.buffer.is_some() {
                let buffer_object = current_surface_attributes.buffer.as_ref().unwrap();
                let dimensions = buffer_dimensions(buffer_object); 

                let new_buffer_info = crate::compositor::surface_management::AttachedBufferInfo {
                    buffer: buffer_object.clone(),
                    scale: current_surface_attributes.buffer_scale,
                    transform: current_surface_attributes.buffer_transform,
                    dimensions: dimensions.map_or_else(Default::default, |d| d.size),
                };
                surface_data.current_buffer_info = Some(new_buffer_info);
                let client_id_str_dbg = surface.client().map(|c| format!("{:?}", c.id())).unwrap_or_else(|| "<unknown_client>".to_string());
                tracing::debug!(
                    surface_id = ?surface.id(), client_info = %client_id_str_dbg,
                    "Attached new buffer. Dimensions: {:?}, Scale: {}, Transform: {:?}",
                    dimensions, current_surface_attributes.buffer_scale, current_surface_attributes.buffer_transform
                );
            } else if current_surface_attributes.buffer.is_none() {
                surface_data.current_buffer_info = None;
                let client_id_str_dbg = surface.client().map(|c| format!("{:?}", c.id())).unwrap_or_else(|| "<unknown_client>".to_string());
                tracing::debug!(surface_id = ?surface.id(), client_info = %client_id_str_dbg, "Buffer detached.");
            }
            
            let previous_buffer_id = surface_data.current_buffer_info.as_ref().map(|info| info.buffer.id());
            let new_buffer_wl = current_surface_attributes.buffer.as_ref();
            let new_buffer_id = new_buffer_wl.map(|b| b.id());

            let new_buffer_attached = new_buffer_wl.is_some() && new_buffer_id != previous_buffer_id;
            let buffer_detached = new_buffer_wl.is_none() && previous_buffer_id.is_some();

            if new_buffer_attached {
                let buffer_to_texture = new_buffer_wl.unwrap();
                let client_id_str = surface.client().map(|c| format!("{:?}", c.id())).unwrap_or_else(|| "<unknown_client>".to_string());

                if let Some(active_renderer_arc_mutex) = self.active_renderer.as_ref() {
                    let mut renderer = active_renderer_arc_mutex.lock().unwrap();

                    // Attempt DMABUF import first if the buffer is a DMABUF
                    match smithay::backend::allocator::dmabuf::attributes_for_buffer(buffer_to_texture) {
                        Ok(dmabuf_attrs) => { // It's a DMABUF
                            tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Buffer is DMABUF. Attempting import with generic FrameRenderer.");
                            match renderer.create_texture_from_dmabuf(&dmabuf_attrs) {
                                Ok(texture_box) => {
                                    surface_data.texture_handle = Some(Arc::from(texture_box)); // Store as Arc
                                    // current_buffer_info was already updated with the new buffer's general info.
                                    // No need to clear it on success.
                                    tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully created texture from DMABUF via generic FrameRenderer. New buffer info: {:?}", surface_data.current_buffer_info);
                                }
                                Err(err) => {
                                    tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Failed to create texture from DMABUF via generic FrameRenderer: {}", err);
                                    surface_data.texture_handle = None;
                                    surface_data.current_buffer_info = None; // New buffer is unusable if import fails.
                                }
                            }
                        }
                        Err(_) => { // Not a DMABUF, or failed to get attributes. Assume SHM.
                            tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Buffer is not DMABUF (or failed to get attrs), attempting SHM import with generic FrameRenderer.");
                            // FrameRenderer::create_texture_from_shm returns Result<Box<dyn RenderableTexture>, RendererError>
                            match renderer.create_texture_from_shm(buffer_to_texture) {
                                Ok(texture_box) => {
                                    surface_data.texture_handle = Some(Arc::from(texture_box)); // Store as Arc
                                    // current_buffer_info was already updated.
                                    tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully created texture from SHM via generic FrameRenderer. New buffer info: {:?}", surface_data.current_buffer_info);
                                }
                                Err(err) => {
                                    tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Failed to create texture from SHM via generic FrameRenderer: {}", err);
                                    surface_data.texture_handle = None;
                                    surface_data.current_buffer_info = None; // New buffer is unusable if import fails.
                                }
                            }
                        }
                    }
                } else { // self.active_renderer is None
                    tracing::warn!("self.active_renderer is None. Cannot import texture for surface_id = {:?}.", surface.id());
                    surface_data.texture_handle = None;
                    surface_data.current_buffer_info = None;
                }
            } else if buffer_detached {
                let client_id_str = surface.client().map(|c| format!("{:?}", c.id())).unwrap_or_else(|| "<unknown_client>".to_string());
                tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Buffer detached, clearing texture and buffer info.");
                surface_data.texture_handle = None;
                surface_data.current_buffer_info = None;
            } else if new_buffer_wl.is_some() && new_buffer_id == previous_buffer_id {
                if let Some(info) = surface_data.current_buffer_info.as_mut() {
                    info.scale = current_surface_attributes.buffer_scale;
                    info.transform = current_surface_attributes.buffer_transform;
                }
            }

            surface_data.damage_buffer_coords.clear(); 
            surface_data.damage_buffer_coords.extend_from_slice(&current_surface_attributes.damage_buffer);
            let client_id_str_trace = surface.client().map(|c| format!("{:?}", c.id())).unwrap_or_else(|| "<unknown_client>".to_string());
            tracing::trace!(
                surface_id = ?surface.id(), client_info = %client_id_str_trace,
                "Damage received (buffer_coords): {:?}",
                current_surface_attributes.damage_buffer
            );

            surface_data.opaque_region_surface_local = current_surface_attributes.opaque_region.clone();
            surface_data.input_region_surface_local = current_surface_attributes.input_region.clone();
        });
    }

    fn new_subsurface(&mut self, surface: &WlSurface, parent: &WlSurface) {
        tracing::info!(surface_id = ?surface.id(), parent_id = ?parent.id(), "New WlSubsurface created");

        let surface_data_arc = surface.data_map()
            .get::<Arc<StdMutex<crate::compositor::surface_management::SurfaceData>>>()
            .expect("SurfaceData not found for subsurface. new_surface should have run.")
            .clone();
        let parent_surface_data_arc = parent.data_map()
            .get::<Arc<StdMutex<crate::compositor::surface_management::SurfaceData>>>()
            .expect("SurfaceData not found for parent surface.")
            .clone();

        surface_data_arc.lock().unwrap().parent = Some(parent.downgrade());
        parent_surface_data_arc.lock().unwrap().children.push(surface.downgrade());
    }

    fn destroyed(&mut self, _surface: &WlSurface) {
        tracing::trace!("CompositorHandler::destroyed called for surface {:?}", _surface.id());
    }
}

// #[derive(Debug)] // Default derive should be fine if all fields are Debug
#[derive(Debug, Default)] // Add Default for easier initialization
pub struct ClientCompositorData {
    pub compositor_state: CompositorClientState,
    // Potentially other client-specific states from other protocols later,
    // e.g. xdg_activation_client_data: XdgActivationClientData (hypothetical)
}

impl ClientCompositorData {
    pub fn new() -> Self {
        Self {
            compositor_state: CompositorClientState::default(),
        }
    }
    // compositor_state() getter removed as the field is public now
}

delegate_compositor!(DesktopState);
delegate_shm!(DesktopState);

impl ShmHandler for DesktopState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl BufferHandler for DesktopState {
    fn buffer_destroyed(&mut self, buffer: &WlBuffer) {
        tracing::debug!(buffer_id = ?buffer.id(), "WlBuffer destroyed (BufferHandler impl in DesktopState).");
        let mut windows_to_damage = Vec::new();

        // Iterate over a collection of Arcs, so clone window_arc_clone for use in this iteration.
        // self.windows stores Arc<ManagedWindow>.
        // self.space.elements() also returns Arcs. Using self.windows to be specific about what's being checked.
        for window_arc_clone in self.windows.values().cloned() {
            // window_arc_clone is Arc<ManagedWindow>
            if let Some(surface) = window_arc_clone.wl_surface() { // ManagedWindow should provide wl_surface()
                if let Some(surface_data_arc) = surface.data_map().get::<Arc<StdMutex<crate::compositor::surface_management::SurfaceData>>>() {
                    let mut surface_data = surface_data_arc.lock().unwrap();
                    if let Some(buffer_info) = &surface_data.current_buffer_info {
                        if buffer_info.buffer.id() == buffer.id() {
                            tracing::info!(
                                "Buffer {:?} (used by surface {:?}, window with domain_id {:?}) was destroyed. Clearing texture and buffer info.",
                                buffer.id(), surface.id(), window_arc_clone.domain_id() // Assuming domain_id() method on ManagedWindow
                            );
                            surface_data.texture_handle = None;
                            surface_data.current_buffer_info = None;
                            windows_to_damage.push(window_arc_clone.clone()); // Clone Arc for vector
                        }
                    }
                }
            }
        }

        for window_to_damage in windows_to_damage {
            // Ensure window_to_damage is the correct type expected by space.damage_window.
            // If space stores elements as Arc<ManagedWindow>, this is fine.
            self.space.damage_window(&window_to_damage, None, None);
            tracing::debug!("Damaged window (domain_id {:?}) due to buffer destruction.", window_to_damage.domain_id());
        }
    }
}

// Delegate DmabufHandler if DesktopState implements it
delegate_dmabuf!(DesktopState);
// Delegate OutputHandler if DesktopState implements it
delegate_output!(DesktopState);
// Delegate SeatHandler if DesktopState implements it
delegate_seat!(DesktopState);
// Delegate XdgShellHandler if DesktopState implements it
delegate_xdg_shell!(DesktopState);
// Delegate XdgDecorationHandler
delegate_xdg_decoration!(DesktopState);
// Delegate ScreencopyHandler
delegate_screencopy!(DesktopState);
// Delegate DamageTrackerHandler if DesktopState implements it
delegate_damage_tracker!(DesktopState);

impl SeatHandler for DesktopState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&Self::KeyboardFocus>) {
        let client_name = focused
            .and_then(|s| s.client())
            .map(|c| format!("{:?}", c.id()))
            .unwrap_or_else(|| "<none>".to_string());

        tracing::debug!(
            seat = %seat.name(),
            focused_surface_id = ?focused.map(|s| s.id()),
            client = %client_name,
            "Seat focus changed"
        );

        self.active_input_surface = focused.map(|s| s.downgrade());

        if let Some(surface) = focused {
            let surface_id_for_log = surface.id();
            tracing::info!(
                "Domain layer would be notified: Keyboard focus changed to surface_id: {:?}",
                surface_id_for_log
            );
        } else {
            tracing::info!("Domain layer would be notified: Keyboard focus lost (cleared)");
        }
        // IMPORTANT: Do NOT call keyboard_handle.set_focus() here.
        // This method is the *result* of set_focus having been called.
    }

    fn cursor_image(&mut self, seat: &Seat<Self>, image: CursorImageStatus) {
        tracing::debug!(seat_name = %seat.name(), cursor_status = ?image, "SeatHandler: cursor_image updated");

        let mut current_status_gaurd = self.current_cursor_status.lock().unwrap();
        *current_status_gaurd = image.clone(); // Update the generic cursor status for other parts of the system

        match image {
            CursorImageStatus::Surface(surface) => {
                // This is the case where wl_pointer.set_cursor is called with a surface.
                // We need to get the WlBuffer from this surface.
                let buffer = smithay::wayland::compositor::with_states(&surface, |states| {
                    states.data_map.get::<self::WlSurfaceAttributes>().unwrap().buffer.clone()
                });

                if let Some(buffer) = buffer {
                    if let Some(renderer_arc) = &self.active_renderer {
                        let mut renderer = renderer_arc.lock().unwrap();
                        match renderer.create_texture_from_shm(&buffer) {
                            Ok(texture_handle_box) => { // create_texture_from_shm returns a Box
                                self.active_cursor_texture = Some(Arc::from(texture_handle_box)); // Convert Box to Arc
                                // Hotspot usually comes from an accompanying wl_surface.attach or a protocol extension.
                                // Smithay's CursorImageStatus::Surface doesn't directly provide it.
                                // For now, we'll use a default hotspot or what's already in self.cursor_hotspot.
                                // A more complete solution would involve retrieving hotspot from client (e.g. via surface commit data).
                                // Let's assume hotspot is (0,0) for now or managed by client via other means that update self.cursor_hotspot
                                tracing::info!("Successfully created texture for cursor surface {:?}", surface.id());
                            }
                            Err(err) => {
                                tracing::error!("Failed to create texture for cursor surface {:?}: {}", surface.id(), err);
                                self.active_cursor_texture = None;
                            }
                        }
                    } else {
                        tracing::warn!("No active_renderer to create cursor texture.");
                        self.active_cursor_texture = None;
                    }
                } else {
                    tracing::warn!("Cursor surface {:?} has no attached buffer.", surface.id());
                    self.active_cursor_texture = None;
                }
                 // self.cursor_hotspot should be updated based on client request, often part of wl_pointer.set_cursor
                 // For this example, we assume it's either (0,0) or set elsewhere.
                 // If CursorImageStatus::Uploaded provided it, we'd use that.
            }
            CursorImageStatus::Uploaded { buffer, hotspot } => {
                // Client pre-uploaded a buffer (less common for standard cursors but possible).
                if let Some(renderer_arc) = &self.active_renderer {
                    let mut renderer = renderer_arc.lock().unwrap();
                    match renderer.create_texture_from_shm(&buffer) {
                        Ok(texture_handle_box) => { // create_texture_from_shm returns a Box
                            self.active_cursor_texture = Some(Arc::from(texture_handle_box)); // Convert Box to Arc
                            self.cursor_hotspot = hotspot.into(); // Use provided hotspot
                            tracing::info!("Successfully created texture for uploaded cursor buffer.");
                        }
                        Err(err) => {
                            tracing::error!("Failed to create texture for uploaded cursor buffer: {}", err);
                            self.active_cursor_texture = None;
                        }
                    }
                } else {
                    tracing::warn!("No active_renderer to create uploaded cursor texture.");
                    self.active_cursor_texture = None;
                }
            }
            CursorImageStatus::Hidden => {
                self.active_cursor_texture = None;
                tracing::info!("Cursor set to hidden.");
            }
            _ => { // Default, None, etc.
                // This case might not be strictly necessary if Hidden covers it.
                self.active_cursor_texture = None;
            }
        }
    }
}
