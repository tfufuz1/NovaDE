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
use smithay::wayland::foreign_toplevel::ForeignToplevelManagerState;

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
    pub foreign_toplevel_state: ForeignToplevelManagerState,
}

impl DesktopState {
    // #[allow(clippy::too_many_arguments)] // No longer needed
    pub fn new(
        loop_handle: LoopHandle<'static, Self>, // Changed from event_loop
        display_handle: DisplayHandle,
    ) -> Self {
        // let loop_handle = event_loop.handle(); // loop_handle is now passed directly
        let clock = Clock::new(None).expect("Failed to create clock");

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
        let foreign_toplevel_state = ForeignToplevelManagerState::new();

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
            foreign_toplevel_state,
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

                match self.active_renderer_type {
                    ActiveRendererType::Gles => {
                        // This path is problematic as gles_renderer was removed and texture_handle expects WgpuRenderableTexture.
                        // For now, this means GLES support is effectively disabled for texture import.
                        tracing::warn!("GLES renderer path in commit: gles_renderer field is removed. Cannot import texture for surface_id = {:?}.", surface.id());
                        surface_data.texture_handle = None;
                        surface_data.current_buffer_info = None;
                        // if let Some(gles_renderer) = self.gles_renderer.as_mut() { // gles_renderer removed
                        //     if let Some(dmabuf_attributes) = self.dmabuf_state.get_dmabuf_attributes(buffer_to_texture) {
                        //         tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Attempting DMABUF import for surface (GLES)");
                        //         match gles_renderer.create_texture_from_dmabuf(&dmabuf_attributes) {
                        //             Ok(new_texture_box) => {
                        //                 tracing::warn!("GLES DMABUF texture created, but SurfaceData expects Arc<WgpuRenderableTexture>. This texture will not be usable with WGPU renderer.");
                        //                 surface_data.texture_handle = None;
                        //                 tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully created GLES texture from DMABUF");
                        //                 surface_data.current_buffer_info = Some(crate::compositor::surface_management::AttachedBufferInfo {
                                            buffer: buffer_to_texture.clone(),
                                            scale: current_surface_attributes.buffer_scale,
                                            transform: current_surface_attributes.buffer_transform,
                                            dimensions: (dmabuf_attributes.width(), dmabuf_attributes.height()).into(),
                                        });
                                    }
                                    Err(e) => {
                                        tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Failed to create GLES texture from DMABUF: {:?}", e);
                                        surface_data.texture_handle = None;
                                        surface_data.current_buffer_info = None;
                                    }
                                }
                            } else {
                                tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Attempting SHM import (GLES)");
                                match gles_renderer.create_texture_from_shm(buffer_to_texture) {
                        //             Ok(new_texture_box) => {
                        //                 tracing::warn!("GLES SHM texture created, but SurfaceData expects Arc<WgpuRenderableTexture>. This texture will not be usable with WGPU renderer.");
                        //                 surface_data.texture_handle = None;
                        //                 tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully created GLES texture from SHM");
                        //                 let dimensions = buffer_dimensions(buffer_to_texture).map_or_else(Default::default, |d| d.size);
                                        surface_data.current_buffer_info = Some(crate::compositor::surface_management::AttachedBufferInfo {
                                            buffer: buffer_to_texture.clone(),
                                            scale: current_surface_attributes.buffer_scale,
                                            transform: current_surface_attributes.buffer_transform,
                                            dimensions,
                                        });
                                    }
                                    Err(e) => {
                                        tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Failed to create GLES texture from SHM: {:?}", e);
                                        surface_data.texture_handle = None;
                                        surface_data.current_buffer_info = None;
                                    }
                                }
                            }
                        } else {
                        //     tracing::warn!(surface_id = ?surface.id(), client_info = %client_id_str, "GLES renderer selected but not available for texture import.");
                        //     surface_data.texture_handle = None;
                        //     surface_data.current_buffer_info = None;
                        // }
                    }
                    ActiveRendererType::Vulkan => {
                        // This path is problematic as vulkan_frame_renderer was removed.
                        tracing::warn!("Vulkan renderer path in commit: vulkan_frame_renderer field is removed. Cannot import texture for surface_id = {:?}.", surface.id());
                        surface_data.texture_handle = None;
                        surface_data.current_buffer_info = None;
                        // if let ( // vulkan_frame_renderer removed
                        //     Some(vk_renderer_mutex),
                        //     Some(vk_allocator),
                        //     Some(vk_instance),
                            Some(vk_physical_device),
                            Some(vk_logical_device)
                        ) = (
                            self.vulkan_frame_renderer.as_ref(),
                            self.vulkan_allocator.as_ref(),
                            self.vulkan_instance.as_ref(),
                            self.vulkan_physical_device_info.as_ref(),
                            self.vulkan_logical_device.as_ref()
                        ) {
                            let mut vk_renderer = vk_renderer_mutex.lock().unwrap(); // Handle MutexGuard

                            if let Some(dmabuf_attributes) = self.dmabuf_state.get_dmabuf_attributes(buffer_to_texture) {
                                tracing::debug!(surface_id = ?surface.id(), client_info = %client_id_str, "Attempting DMABUF import for surface (Vulkan path)");
                                match vk_renderer.import_dmabuf_texture(
                                    &dmabuf_attributes,
                                    vk_instance,
                                    vk_physical_device,
                                    vk_logical_device,
                                    vk_allocator
                                ) {
                        //     Ok(new_texture_box) => {
                        //         tracing::warn!("Vulkan (GL interop) DMABUF texture created, but SurfaceData expects Arc<WgpuRenderableTexture>.");
                        //         surface_data.texture_handle = None;
                        //         tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully imported DMABUF as Vulkan texture.");
                        //         surface_data.current_buffer_info = Some(crate::compositor::surface_management::AttachedBufferInfo {
                                            buffer: buffer_to_texture.clone(),
                                            scale: current_surface_attributes.buffer_scale,
                                            transform: current_surface_attributes.buffer_transform,
                                            dimensions: (dmabuf_attributes.width(), dmabuf_attributes.height()).into(),
                                        });
                                    }
                                    Err(e) => {
                                        tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Failed to create Vulkan texture from DMABUF: {:?}", e);
                                        surface_data.texture_handle = None;
                                        surface_data.current_buffer_info = None;
                                    }
                                }
                            } else {
                                tracing::debug!(surface_id = ?surface.id(), client_info = %client_id_str, "Attempting SHM import for surface (Vulkan path)");
                                match vk_renderer.import_shm_texture(
                                    buffer_to_texture,
                                    vk_allocator,
                                    vk_logical_device
                                ) {
                        //     Ok(new_texture_box) => {
                        //         tracing::warn!("Vulkan (GL interop) SHM texture created, but SurfaceData expects Arc<WgpuRenderableTexture>.");
                        //         surface_data.texture_handle = None;
                        //         tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully imported SHM as Vulkan texture.");
                        //         let dimensions = buffer_dimensions(buffer_to_texture).map_or_else(Default::default, |d| d.size);
                                        surface_data.current_buffer_info = Some(crate::compositor::surface_management::AttachedBufferInfo {
                                            buffer: buffer_to_texture.clone(),
                                            scale: current_surface_attributes.buffer_scale,
                                            transform: current_surface_attributes.buffer_transform,
                                            dimensions,
                                        });
                                    }
                                    Err(e) => {
                                        tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Failed to create Vulkan texture from SHM: {:?}", e);
                                        surface_data.texture_handle = None;
                                        surface_data.current_buffer_info = None;
                                    }
                                }
                            }
                        } else {
                        //     tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Vulkan renderer selected, but some core components (renderer, allocator, instance, devices) are missing. Cannot import texture.");
                        //     surface_data.texture_handle = None;
                        //     surface_data.current_buffer_info = None;
                        // }
                    }
                    ActiveRendererType::Wgpu => {
                        if let Some(wgpu_renderer_concrete_mutexed) = self.wgpu_renderer_concrete.as_ref() {
                            let mut wgpu_renderer = wgpu_renderer_concrete_mutexed.lock().unwrap();
                            if self.dmabuf_state.get_dmabuf_attributes(buffer_to_texture).is_some() {
                                tracing::warn!(surface_id = ?surface.id(), client_info = %client_id_str, "DMABUF import for WGPU not yet implemented in commit path. Clearing texture.");
                                surface_data.texture_handle = None;
                                // If DMABUF import fails for this new buffer, it's unusable with current renderer caps.
                                // The surface_data.current_buffer_info was already updated to this new buffer's info.
                                // Setting it to None here means this new buffer attachment is effectively void.
                                surface_data.current_buffer_info = None;
                            } else { // SHM Buffer
                                tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Attempting SHM import for WGPU renderer.");
                                match wgpu_renderer.create_texture_from_shm(buffer_to_texture) {
                                    Ok(wgpu_texture_arc) => {
                                        surface_data.texture_handle = Some(wgpu_texture_arc);
                                        // surface_data.current_buffer_info was already updated at the start of the
                                        // `smithay::wayland::compositor::with_states` block if a new buffer was attached.
                                        // We don't need to set it again here if texture creation is successful.
                                        // It correctly reflects the new buffer's attributes.
                                        tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully created WGPU texture from SHM. New buffer info: {:?}", surface_data.current_buffer_info);
                                    }
                                    Err(e) => {
                                        tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Failed to create WGPU texture from SHM: {:?}", e);
                                        surface_data.texture_handle = None;
                                        // If texture creation failed for this *new* buffer, then this buffer cannot be displayed.
                                        // The surface_data.current_buffer_info (already pointing to this new buffer) should be cleared.
                                        surface_data.current_buffer_info = None;
                                    }
                                }
                            }
                        } else {
                            tracing::warn!("WGPU renderer selected but wgpu_renderer_concrete is None. Cannot import texture for surface_id = {:?}.", surface.id());
                            surface_data.texture_handle = None;
                            surface_data.current_buffer_info = None;
                        }
                    }
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
        tracing::trace!(
            seat = %seat.name(),
            new_status = ?image,
            "Seat cursor image updated"
        );
        *self.current_cursor_status.lock().unwrap() = image.clone();
        tracing::info!(
            "Renderer to be signaled: Update cursor image to {:?} at location {:?}.",
            image,
            self.pointer_location
        );
    }
}
