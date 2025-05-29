use smithay::{
    delegate_compositor, delegate_damage_tracker, delegate_dmabuf, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell, // Added delegate_dmabuf
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
        dmabuf::DmabufState, // Added DmabufState
    },
    backend::renderer::utils::buffer_dimensions,
    desktop::{Space, DamageTrackerState},
    output::Output,
    input::{Seat, SeatState, pointer::CursorImageStatus},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex, Mutex}, // Added Mutex for VulkanFrameRenderer
    time::Instant,
};
use crate::compositor::surface_management::{AttachedBufferInfo, SurfaceData}; 
use crate::compositor::core::ClientCompositorData;
use crate::compositor::xdg_shell::types::{DomainWindowIdentifier, ManagedWindow};

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
    /// Vulkan renderer is active.
    Vulkan,
}

// Main compositor state
pub struct NovadeCompositorState {
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, Self>,
    pub clock: Clock<u64>,
    pub compositor_state: CompositorState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub gles_renderer: Option<crate::compositor::renderers::gles2::renderer::Gles2Renderer>, // Renamed from renderer
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

    // Vulkan Renderer Components
    pub vulkan_instance: Option<Arc<VulkanInstance>>,
    pub vulkan_physical_device_info: Option<Arc<PhysicalDeviceInfo>>,
    pub vulkan_logical_device: Option<Arc<LogicalDevice>>,
    pub vulkan_allocator: Option<Arc<Allocator>>, // Assuming crate::compositor::renderer::vulkan::allocator::Allocator
    pub vulkan_frame_renderer: Option<Arc<Mutex<VulkanFrameRenderer>>>,
    
    /// Specifies which renderer is currently active.
    pub active_renderer_type: ActiveRendererType,

    // --- Added for MCP and CPU Usage Service ---
    pub mcp_connection_service: Option<Arc<TokioMutex<MCPConnectionService>>>,
    pub cpu_usage_service: Option<Arc<dyn ICpuUsageService>>,
    // pub mcp_client_spawner: Option<Arc<dyn IMCPClientService>>,
}

impl NovadeCompositorState {
    #[allow(clippy::too_many_arguments)] // Constructor naturally has many arguments for state initialization
    pub fn new(
        event_loop: &mut EventLoop<'static, Self>,
        display_handle: DisplayHandle,
        gles_renderer: Option<crate::compositor::renderers::gles2::renderer::Gles2Renderer>,
        vulkan_instance: Option<Arc<VulkanInstance>>,
        vulkan_physical_device_info: Option<Arc<PhysicalDeviceInfo>>,
        vulkan_logical_device: Option<Arc<LogicalDevice>>,
        vulkan_allocator: Option<Arc<Allocator>>,
        vulkan_frame_renderer: Option<Arc<Mutex<VulkanFrameRenderer>>>,
        active_renderer_type: ActiveRendererType,
    ) -> Self {
        let loop_handle = event_loop.handle();
        let clock = Clock::new(None).expect("Failed to create clock");

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let space = Space::new(tracing::info_span!("novade_space"));
        let damage_tracker_state = DamageTrackerState::new();
        let mut seat_state = SeatState::new();
        let seat_name = "seat0".to_string();
        let seat = seat_state.new_wl_seat(&display_handle, seat_name.clone(), Some(tracing::Span::current()));
        let dmabuf_state = DmabufState::new();

        Self {
            display_handle,
            loop_handle,
            clock,
            compositor_state,
            shm_state,
            output_manager_state,
            gles_renderer, // Now an Option passed in
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
            vulkan_instance,
            vulkan_physical_device_info,
            vulkan_logical_device,
            vulkan_allocator,
            vulkan_frame_renderer,
            active_renderer_type,
            // --- Initialize new service fields ---
            mcp_connection_service: None,
            cpu_usage_service: None,
            // mcp_client_spawner: None,
        }
    }
}

impl CompositorHandler for NovadeCompositorState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        client
            .get_data::<ClientCompositorData>()
            .expect("ClientCompositorData not initialized for this client.")
            .compositor_state()
    }

    fn new_surface(&mut self, surface: &WlSurface) {
        let client_id = surface.client().expect("Surface must have a client.").id();
        tracing::info!(surface_id = ?surface.id(), ?client_id, "New WlSurface created");

        let surface_data = Arc::new(StdMutex::new(
            crate::compositor::surface_management::SurfaceData::new(client_id),
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
                        if let Some(gles_renderer) = self.gles_renderer.as_mut() {
                            if let Some(dmabuf_attributes) = self.dmabuf_state.get_dmabuf_attributes(buffer_to_texture) {
                                tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Attempting DMABUF import for surface (GLES)");
                                match gles_renderer.create_texture_from_dmabuf(&dmabuf_attributes) {
                                    Ok(new_texture) => {
                                        surface_data.texture_handle = Some(new_texture);
                                        tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully created GLES texture from DMABUF");
                                        surface_data.current_buffer_info = Some(crate::compositor::surface_management::AttachedBufferInfo {
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
                                    Ok(new_texture) => {
                                        surface_data.texture_handle = Some(new_texture);
                                        tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully created GLES texture from SHM");
                                        let dimensions = buffer_dimensions(buffer_to_texture).map_or_else(Default::default, |d| d.size);
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
                            tracing::warn!(surface_id = ?surface.id(), client_info = %client_id_str, "GLES renderer selected but not available for texture import.");
                            surface_data.texture_handle = None;
                            surface_data.current_buffer_info = None;
                        }
                    }
                    ActiveRendererType::Vulkan => {
                        if let (
                            Some(vk_renderer_mutex),
                            Some(vk_allocator),
                            Some(vk_instance),
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
                                    Ok(new_texture) => {
                                        surface_data.texture_handle = Some(new_texture);
                                        tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully imported DMABUF as Vulkan texture.");
                                        surface_data.current_buffer_info = Some(crate::compositor::surface_management::AttachedBufferInfo {
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
                                    Ok(new_texture) => {
                                        surface_data.texture_handle = Some(new_texture);
                                        tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully imported SHM as Vulkan texture.");
                                        let dimensions = buffer_dimensions(buffer_to_texture).map_or_else(Default::default, |d| d.size);
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
                            tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Vulkan renderer selected, but some core components (renderer, allocator, instance, devices) are missing. Cannot import texture.");
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

#[derive(Debug)]
pub struct ClientCompositorData {
    _placeholder: (), 
    pub client_specific_state: CompositorClientState,
}

impl ClientCompositorData {
    pub fn new() -> Self {
        Self {
            _placeholder: (),
            client_specific_state: CompositorClientState::default(),
        }
    }
    pub fn compositor_state(&self) -> &CompositorClientState {
        &self.client_specific_state
    }
}

impl Default for ClientCompositorData {
    fn default() -> Self {
        Self::new()
    }
}

delegate_compositor!(NovadeCompositorState); // Renamed DesktopState
delegate_shm!(NovadeCompositorState);       // Renamed DesktopState

impl ShmHandler for NovadeCompositorState { // Renamed DesktopState
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl BufferHandler for NovadeCompositorState { // Renamed DesktopState
    fn buffer_destroyed(&mut self, buffer: &WlBuffer) {
        tracing::debug!(buffer_id = ?buffer.id(), "WlBuffer destroyed notification received in BufferHandler.");
    }
}

// Delegate DmabufHandler if NovadeCompositorState implements it
delegate_dmabuf!(NovadeCompositorState);
// Delegate OutputHandler if NovadeCompositorState implements it
delegate_output!(NovadeCompositorState);
// Delegate SeatHandler if NovadeCompositorState implements it
delegate_seat!(NovadeCompositorState);
// Delegate XdgShellHandler if NovadeCompositorState implements it
delegate_xdg_shell!(NovadeCompositorState);
// Delegate DamageTrackerHandler if NovadeCompositorState implements it
delegate_damage_tracker!(NovadeCompositorState);
