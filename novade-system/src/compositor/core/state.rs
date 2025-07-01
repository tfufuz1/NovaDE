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
    wayland::dmabuf::{DmabufHandler, DmabufGlobal, ImportNotifier}, // Added for DmabufHandler
    backend::allocator::dmabuf::Dmabuf, // Added for DmabufHandler
    reexports::wayland_protocols::wp::linux_dmabuf::zv1::server::zwp_linux_buffer_params_v1, // Added for DmabufHandler
    backend::renderer::gles2::Gles2Texture, // Added for placeholder renderer Texture type
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex, Mutex}, // Added Mutex for VulkanFrameRenderer
    time::Instant,
};
use crate::input::keyboard::xkb_config::XkbKeyboardData; // Added XkbKeyboardData
use crate::compositor::surface_management::{AttachedBufferInfo, SurfaceData};
use crate::compositor::core::ClientCompositorData;
use crate::compositor::render::renderer::CompositorRenderer; // Added for DesktopState.renderer
use crate::compositor::render::dmabuf_importer::DmabufImporter; // Added for DesktopState.dmabuf_importer
use crate::compositor::shell::xdg_shell::types::{DomainWindowIdentifier, ManagedWindow};
// ANCHOR: ImportCompositorWorkspaceAndUuid
use crate::compositor::workspaces::CompositorWorkspace;
use uuid::Uuid;
// ANCHOR_END: ImportCompositorWorkspaceAndUuid
// ANCHOR: AddOutputConfigImportForMultiMonitor
use crate::compositor::outputs::OutputConfig;
// ANCHOR_END: AddOutputConfigImportForMultiMonitor
use novade_domain::DomainServices;
use crate::input::input_dispatcher::InputDispatcher;
use crate::input::keyboard_layout::KeyboardLayoutManager;
// use crate::renderer::wgpu_renderer::NovaWgpuRenderer; // Removed
// use crate::compositor::renderer_interface::abstraction::FrameRenderer; // Removed
use crate::display_management::WaylandDisplayManager;

// mod input_handlers; // Removed as SeatHandler is now fully in state.rs
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

/// # DesktopState - The Central Compositor State
///
/// `DesktopState` is the primary struct holding all state information for the Novade compositor.
/// It consolidates various Smithay helper states (e.g., `CompositorState`, `ShmState`, `SeatState`,
/// `OutputManagerState`, `XdgShellState`, etc.) and manages system-wide resources like
/// the display handle, event loop handle, rendering components, and registered Wayland globals.
///
/// This struct implements numerous `*Handler` traits from Smithay (e.g., `CompositorHandler`,
/// `ShmHandler`, `SeatHandler`) and uses Smithay's delegation macros (`delegate_*`) to
/// correctly dispatch Wayland protocol requests to the appropriate handlers and state components.
///
/// Its responsibilities include:
/// - Initializing and storing core Wayland protocol states.
/// - Managing client data and surface-specific data (`ClientCompositorData`, `SurfaceData`).
/// - Handling input events via `InputDispatcher` and `SeatState`.
/// - Orchestrating rendering through the `active_renderer` (a `FrameRenderer` trait object).
/// - Managing Wayland globals and their lifetimes.
/// - Storing application-specific state like window lists (`ManagedWindow`), output configurations,
///   and connections to domain services.
// TODO MVP: Review if all necessary MVP core protocol states (wl_display, wl_registry, wl_compositor,
// TODO MVP: wl_surface, wl_shm, wl_buffer, wl_callback) are fully initialized and handled.
// TODO MVP: Current assessment is that they are, via Smithay's state types and the
// TODO MVP: GlobalDispatch implementations in `globals.rs`.
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
    pub dmabuf_state: DmabufState, // Already present
    pub dmabuf_importer: Option<DmabufImporter>, // Added
    pub xdg_decoration_state: XdgDecorationState,
    pub screencopy_state: ScreencopyState, // Added screencopy_state

    // Vulkan Renderer Components - REMOVED
    // pub vulkan_instance: Option<Arc<VulkanInstance>>, // TODO Post-MVP: Re-evaluate Vulkan direct integration if needed.
    // pub vulkan_physical_device_info: Option<Arc<PhysicalDeviceInfo>>,
    // pub vulkan_logical_device: Option<Arc<LogicalDevice>>,
    // pub vulkan_allocator: Option<Arc<Allocator>>,
    // pub vulkan_frame_renderer: Option<Arc<Mutex<VulkanFrameRenderer>>>,
    
    /// Specifies which renderer is currently active.
    /// TODO: This should ideally be dynamically determined or configured rather than defaulting to GLES.
    pub active_renderer_type: ActiveRendererType,
    // pub active_renderer: Option<Arc<Mutex<dyn FrameRenderer>>>, // This line is effectively removed / replaced by the line below
    pub renderer: Option<Arc<Mutex<dyn CompositorRenderer<Texture = Arc<Gles2Texture>>>>>,

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
    // pub wgpu_renderer: Option<Arc<Mutex<NovaWgpuRenderer>>>,
    // pub wgpu_renderer_concrete: Option<Arc<Mutex<NovaWgpuRenderer>>>, // Removed
    pub display_manager: Arc<WaylandDisplayManager>,

    // --- Cursor Rendering State ---
    // Use the RenderableTexture trait defined alongside FrameRenderer trait
    pub active_cursor_texture: Option<Arc<dyn crate::compositor::state::RenderableTexture>>, // Changed Box to Arc
    pub cursor_hotspot: Point<i32, Logical>,

    // --- Layer Shell State (Placeholder for Smithay 0.3.0) ---
    // Smithay 0.3.0 does not have built-in LayerShellState. This is a placeholder.
    // A full implementation would require manual protocol handling or a newer Smithay.
    // TODO Post-MVP: Implement layer shell support if required.
    pub layer_shell_data: MyCustomLayerShellData,

    // ANCHOR: AddWorkspaceFieldsToDesktopState
    // pub compositor_workspaces: Vec<Arc<RwLock<CompositorWorkspace>>>, // Replaced by output_workspaces
    // pub active_compositor_workspace_id: Arc<RwLock<Uuid>>, // Replaced by active_workspaces
    // ANCHOR: PerOutputWorkspaceFieldsForMultiMonitor
    pub output_workspaces: HashMap<String, Vec<Arc<RwLock<CompositorWorkspace>>>>, // Key: Output.name()
    pub active_workspaces: Arc<RwLock<HashMap<String, Uuid>>>, // Key: Output.name(), Value: Active Workspace ID
    pub primary_output_name: Arc<RwLock<Option<String>>>, // Name of the designated primary output
    // ANCHOR_END: PerOutputWorkspaceFieldsForMultiMonitor
    // ANCHOR_END: AddWorkspaceFieldsToDesktopState
}
// TODO MVP: Ensure all core protocol handlers (wl_display, wl_registry, wl_compositor,
// TODO MVP: wl_surface, wl_shm, wl_buffer, wl_callback) are adequately stubbed or implemented.
// TODO MVP: Review `globals.rs` for `GlobalDispatch` implementations and ensure they cover client binding
// TODO MVP: for all MVP-required globals.
// TODO MVP: Current assessment:
// TODO MVP: - wl_display: Implicitly handled by Smithay's Display.
// TODO MVP: - wl_registry: Client binding handled by GlobalDispatch in `globals.rs` for registered globals.
// TODO MVP: - wl_compositor, wl_subcompositor: Handled by `CompositorHandler` and `GlobalDispatch` in `globals.rs`.
// TODO MVP: - wl_surface: Handled by `CompositorHandler`.
// TODO MVP: - wl_shm: Handled by `ShmHandler`, `BufferHandler` (for destruction), and `GlobalDispatch` in `globals.rs`.
// TODO MVP: - wl_buffer: Attachment handled in `CompositorHandler::commit`, destruction in `BufferHandler`.
// TODO MVP: - wl_callback: Handled in `CompositorHandler::commit` via `surface_data.frame_callbacks`.

/// Placeholder for custom layer shell data management.
/// In a real scenario with manual protocol implementation, this would store
/// information about layer surfaces, their states, etc.
#[derive(Debug, Default)]
pub struct MyCustomLayerShellData {
    // Example: layers: Vec<Weak<WlSurface>>, // or a more complex struct per layer
    // For this placeholder, it's empty.
}

// ANCHOR: DesktopStateHelperMethodsImpl
use crate::error::SystemResult; // For WaylandDisplayManager::new()

impl DesktopState {
    /// Finds a `ManagedWindow` by its underlying `WlSurface`.
    /// This is a common operation needed by various handlers.
    pub(crate) fn find_managed_window_by_wl_surface(&self, surface: &WlSurface) -> Option<Arc<ManagedWindow>> {
        self.windows.values()
            .find(|win_arc| win_arc.wl_surface().as_ref() == Some(surface))
            .cloned()
    }

    // ANCHOR: SwitchToWorkspaceOnOutputRefactored
    /// Switches to the workspace with the given ID on a specific output.
    pub fn switch_to_workspace_on_output(&mut self, output_name: &str, new_workspace_id: Uuid) {
        let mut active_workspaces_guard = self.active_workspaces.write().unwrap();
        let old_active_workspace_id_on_output = active_workspaces_guard.get(output_name).copied();

        if old_active_workspace_id_on_output == Some(new_workspace_id) {
            tracing::debug!("Attempted to switch to already active workspace (ID: {}) on output {}. No action taken.", new_workspace_id, output_name);
            return;
        }

        // Check if the target output exists
        if !self.outputs.iter().any(|o| o.name() == output_name) {
            tracing::warn!("switch_to_workspace_on_output: Output {} not found.", output_name);
            return;
        }
        // Check if the new_workspace_id is valid for the given output_name
        let target_ws_exists_on_output = self.output_workspaces.get(output_name)
            .map_or(false, |workspaces_on_output| {
                workspaces_on_output.iter().any(|ws_arc| ws_arc.read().unwrap().id == new_workspace_id)
            });

        if !target_ws_exists_on_output {
            tracing::warn!("switch_to_workspace_on_output: Workspace ID {} does not belong to output {}.", new_workspace_id, output_name);
            return;
        }

        tracing::info!("Switching workspace on output {} from {:?} to {}", output_name, old_active_workspace_id_on_output, new_workspace_id);

        // Unmap windows from the old active workspace on this output
        if let Some(old_ws_id) = old_active_workspace_id_on_output {
            for window_arc in self.windows.values() {
                let mut should_unmap = false;
                if *window_arc.output_name.read().unwrap() == Some(output_name.to_string()) {
                    if *window_arc.workspace_id.read().unwrap() == Some(old_ws_id) {
                        should_unmap = true;
                    }
                }
                if should_unmap {
                    if window_arc.is_mapped() {
                        self.space.unmap_window(window_arc);
                        tracing::info!("Unmapped window {:?} (domain_id: {:?}) from old workspace {} on output {}",
                            window_arc.id, window_arc.domain_id, old_ws_id, output_name);
                    }
                    let mut win_state = window_arc.state.write().unwrap();
                    win_state.is_mapped = false;
                    win_state.activated = false;
                }
            }
        }

        // Update the active workspace ID for the specified output
        active_workspaces_guard.insert(output_name.to_string(), new_workspace_id);
        drop(active_workspaces_guard); // Release lock early

        // Apply layout for the new active workspace on this specific output.
        crate::compositor::tiling::apply_layout_for_output(self, output_name);

        self.space.damage_all_outputs();
        tracing::info!("Switched active workspace on output {} to {}", output_name, new_workspace_id);
    }
    // ANCHOR_END: SwitchToWorkspaceOnOutputRefactored

    // ANCHOR: SetWorkspaceTilingLayoutRefactored
    /// Sets the tiling layout for a given workspace and applies it if that workspace is active on its output.
    pub fn set_workspace_tiling_layout(&mut self, workspace_id: Uuid, new_layout_mode: TilingLayout) {
        let mut found_workspace_output_name: Option<String> = None;

        'outer: for workspaces_on_output in self.output_workspaces.values() {
            for ws_arc in workspaces_on_output {
                let mut ws_write_guard = ws_arc.write().unwrap(); // Lock for potential modification
                if ws_write_guard.id == workspace_id {
                    *ws_write_guard.tiling_layout.write().unwrap() = new_layout_mode;
                    found_workspace_output_name = Some(ws_write_guard.output_name.clone());
                    tracing::info!("Set tiling layout for workspace {} ({}) on output {} to {:?}",
                        ws_write_guard.name, ws_write_guard.id, ws_write_guard.output_name, new_layout_mode);
                    break 'outer;
                }
            }
        }

        if let Some(output_name_of_changed_ws) = found_workspace_output_name {
            let active_workspaces_guard = self.active_workspaces.read().unwrap();
            if active_workspaces_guard.get(&output_name_of_changed_ws) == Some(&workspace_id) {
                // The workspace whose layout changed is active on its output. Re-apply layout for that output.
                crate::compositor::tiling::apply_layout_for_output(self, &output_name_of_changed_ws);
            }
        } else {
            tracing::warn!("set_workspace_tiling_layout: Workspace {} not found.", workspace_id);
        }
    }
    // ANCHOR_END: SetWorkspaceTilingLayout

    // ANCHOR: MoveWindowToOutputImpl
    /// Moves a window to a specified output and the active workspace on that output.
    pub fn move_window_to_output(&mut self, window_domain_id: &DomainWindowIdentifier, target_output_name: &str) {
        let window_arc = match self.windows.get(window_domain_id) {
            Some(arc) => arc.clone(),
            None => {
                tracing::warn!("move_window_to_output: Window with Domain ID {:?} not found.", window_domain_id);
                return;
            }
        };

        // Ensure target output exists
        if !self.outputs.iter().any(|o| o.name() == target_output_name) {
            tracing::warn!("move_window_to_output: Target output {} not found.", target_output_name);
            return;
        }

        let old_output_name_opt = window_arc.output_name.read().unwrap().clone();
        let old_workspace_id_opt = *window_arc.workspace_id.read().unwrap();

        // 1. Remove from old workspace (if any)
        if let (Some(old_oname), Some(old_ws_id)) = (&old_output_name_opt, old_workspace_id_opt) {
            if let Some(workspaces_on_old_output) = self.output_workspaces.get(old_oname) {
                if let Some(old_ws_arc) = workspaces_on_old_output.iter().find(|ws| ws.read().unwrap().id == old_ws_id) {
                    old_ws_arc.read().unwrap().remove_window(window_domain_id);
                    tracing::info!("Window {:?} removed from old workspace {} on output {}", window_arc.id, old_ws_id, old_oname);
                }
            }
        }

        // 2. Update window's output and workspace assignment
        let active_workspaces_guard = self.active_workspaces.read().unwrap();
        let new_active_ws_id_on_target_output = match active_workspaces_guard.get(target_output_name) {
            Some(id) => *id,
            None => {
                tracing::error!("move_window_to_output: No active workspace found for target output {}. Cannot move window.", target_output_name);
                // Attempt to add back to old workspace if removal was successful? Or leave it orphaned?
                // For now, log and leave it without a workspace on the new output.
                *window_arc.workspace_id.write().unwrap() = None;
                *window_arc.output_name.write().unwrap() = Some(target_output_name.to_string());
                // A window without a workspace might not be ideal. Consider assigning to a default/first ws on target output.
                return;
            }
        };
        drop(active_workspaces_guard);

        *window_arc.output_name.write().unwrap() = Some(target_output_name.to_string());
        *window_arc.workspace_id.write().unwrap() = Some(new_active_ws_id_on_target_output);

        if let Some(workspaces_on_new_output) = self.output_workspaces.get(target_output_name) {
            if let Some(new_ws_arc) = workspaces_on_new_output.iter().find(|ws| ws.read().unwrap().id == new_active_ws_id_on_target_output) {
                new_ws_arc.read().unwrap().add_window(window_domain_id);
                tracing::info!("Window {:?} added to new workspace {} on output {}", window_arc.id, new_active_ws_id_on_target_output, target_output_name);
            }
        }

        // 3. Update window geometry (simple approach: center on new output, keep size)
        // A more sophisticated approach would try to maintain relative position or handle tiling.
        let current_size = window_arc.geometry().size; // Keep current size
        if let Some(target_output_obj) = self.outputs.iter().find(|o| o.name() == target_output_name) {
            if let Some(output_geometry) = self.space.output_geometry(target_output_obj) {
                let new_loc_x = output_geometry.loc.x + (output_geometry.size.w / 2 - current_size.w / 2);
                let new_loc_y = output_geometry.loc.y + (output_geometry.size.h / 2 - current_size.h / 2);
                let new_window_global_geom = Rectangle::from_loc_and_size(Point::from((new_loc_x, new_loc_y)), current_size);

                *window_arc.current_geometry.write().unwrap() = new_window_global_geom;
                let mut win_state = window_arc.state.write().unwrap();
                win_state.position = new_window_global_geom.loc; // Update WindowState as well
                win_state.size = new_window_global_geom.size;
                // Ensure window is mapped on the new output.
                // If it was mapped on old output, unmap first (done by switch_to_workspace or similar)
                // For direct move, we might need to unmap from old output's space representation if space is per-output.
                // Smithay's Space is global, so changing output association is more about where tiling places it.
                win_state.is_mapped = true; // Ensure it's considered mapped for the new layouting.
            }
        }

        // 4. Re-tile old output (if window moved from a different output)
        if let Some(old_oname_val) = old_output_name_opt {
            if old_oname_val != target_output_name {
                crate::compositor::tiling::apply_layout_for_output(self, &old_oname_val);
            }
        }

        // 5. Re-tile new output
        crate::compositor::tiling::apply_layout_for_output(self, target_output_name);

        // 6. Update focus if necessary (e.g., if the moved window was focused)
        // For MVP, focus will follow pointer or next click.

        tracing::info!("Window {:?} moved to output {}. Output's active workspace: {}", window_arc.id, target_output_name, new_active_ws_id_on_target_output);
        self.space.damage_all_outputs(); // Damage all as window moved between outputs
    }
    // ANCHOR_END: MoveWindowToOutputImpl
}
// ANCHOR_END: DesktopStateHelperMethodsImpl

impl DesktopState {
    // #[allow(clippy::too_many_arguments)] // No longer needed
    pub fn new(
        loop_handle: LoopHandle<'static, Self>, // Changed from event_loop
        display_handle: DisplayHandle,
    ) -> SystemResult<Self> { // Return SystemResult
        // let loop_handle = event_loop.handle(); // loop_handle is now passed directly
        let clock = Clock::new(None).expect("Failed to create clock");

        // ANCHOR: MultiOutputAndWorkspaceInitializationInNew
        // Define default output configurations (e.g., for headless or fallback)
        // In a real scenario, this would come from backend discovery (DRM, winit) or config file.
        let output_configs = vec![
            OutputConfig {
                name: "HEADLESS-1".to_string(),
                resolution: (1920, 1080).into(),
                position: (0, 0).into(),
                scale: 1,
                is_primary: true,
            },
            // Example of a second output:
            // OutputConfig {
            //     name: "HEADLESS-2".to_string(),
            //     resolution: (1280, 720).into(),
            //     position: (1920, 0).into(), // Positioned to the right of the first one
            //     scale: 1,
            //     is_primary: false,
            // },
        ];

        let mut outputs_vec_init = Vec::new();
        let mut output_workspaces_map_init = HashMap::new();
        let mut active_workspaces_map_init = HashMap::new();
        let mut primary_output_name_init: Option<String> = None;

        let mut space = Space::new(tracing::info_span!("novade_space")); // Create space here to map outputs

        for config in output_configs {
            let output = Output::new(
                config.name.clone(),
                smithay::output::PhysicalProperties {
                    size: config.resolution.to_physical_precise_round(config.scale),
                    subpixel: smithay::output::Subpixel::Unknown,
                    make: "NovaDE".into(),
                    model: "VirtualOutput".into(),
                },
                None // Logger
            );
            output.change_current_state(
                Some(smithay::output::Mode { size: config.resolution, refresh: 60000 }), // 60Hz
                Some(smithay::utils::Transform::Normal),
                Some(smithay::output::Scale::Integer(config.scale)),
                Some(config.position)
            );
            output.set_preferred_mode(smithay::output::Mode { size: config.resolution, refresh: 60000 });

            space.map_output(&output, config.position);
            outputs_vec_init.push(output.clone());

            if config.is_primary || primary_output_name_init.is_none() {
                primary_output_name_init = Some(config.name.clone());
            }

            let mut workspaces_for_this_output = Vec::new();
            let num_ws_per_output = 4;
            let mut first_ws_id_for_this_output = Uuid::nil();
            for i in 0..num_ws_per_output {
                let ws_name = format!("{} WS{}", config.name.chars().take(4).collect::<String>(), i + 1);
                let workspace = Arc::new(RwLock::new(CompositorWorkspace::new(ws_name, config.name.clone())));
                if i == 0 {
                    first_ws_id_for_this_output = workspace.read().unwrap().id;
                }
                workspaces_for_this_output.push(workspace);
            }
            output_workspaces_map_init.insert(config.name.clone(), workspaces_for_this_output);
            active_workspaces_map_init.insert(config.name.clone(), first_ws_id_for_this_output);
        }

        if primary_output_name_init.is_none() && !outputs_vec_init.is_empty() {
            primary_output_name_init = Some(outputs_vec_init[0].name().to_string());
            tracing::warn!("No primary output designated in configs, defaulting to first output: {}", primary_output_name_init.as_ref().unwrap());
        } else if primary_output_name_init.is_none() && outputs_vec_init.is_empty() {
             tracing::error!("No outputs configured. This is likely an error.");
             // Potentially return an error here or panic, as a compositor without outputs is unusual.
        }
        // ANCHOR_END: MultiOutputAndWorkspaceInitializationInNew

        let display_manager = Arc::new(WaylandDisplayManager::new()?);

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        // let space = Space::new(tracing::info_span!("novade_space")); // Moved up for output mapping
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

        let dmabuf_state = DmabufState::new(); // Already present
        let dmabuf_importer = DmabufImporter::new().expect("Failed to create DmabufImporter"); // Added
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle);
        let screencopy_state = ScreencopyState::new::<Self>(&display_handle, None); // Initialize ScreencopyState

        Ok(Self { // Added Ok() for SystemResult
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
            dmabuf_state, // Already present
            dmabuf_importer: Some(dmabuf_importer), // Added
            xdg_decoration_state,
            screencopy_state, // Add to struct instantiation
            // vulkan_instance: None, // Removed
            // vulkan_physical_device_info: None, // Removed
            // vulkan_logical_device: None, // Removed
            // vulkan_allocator: None, // Removed
            // vulkan_frame_renderer: None, // Removed
            active_renderer_type: ActiveRendererType::Gles, // Default, backend should update
            // active_renderer: None, // This field is being removed
            renderer: None, // Added for the new renderer field
            // --- Initialize new service fields ---
            mcp_connection_service: None,
            cpu_usage_service: None,
            // mcp_client_spawner: None,
            domain_services: None,
            input_dispatcher,
            keyboard_layout_manager,
            // wgpu_renderer: None, // Removed specific field
            // wgpu_renderer_concrete: None, // This field is being removed
            active_input_surface: None,
            keyboard_data_map,
            touch_focus_per_slot: HashMap::new(),
            display_manager,
            active_cursor_texture: None,
            cursor_hotspot: (0, 0).into(), // Initialize hotspot
            layer_shell_data: MyCustomLayerShellData::default(), // Initialize placeholder
            outputs: outputs_vec_init, // Store the initialized outputs
            // ANCHOR: UpdateDesktopStateFieldsInitialization
            output_workspaces: output_workspaces_map_init,
            active_workspaces: Arc::new(RwLock::new(active_workspaces_map_init)),
            primary_output_name: Arc::new(RwLock::new(primary_output_name_init)),
            // ANCHOR_END: UpdateDesktopStateFieldsInitialization
        })
    }

    // TODO: This render_frame method is being added/adapted to DesktopState
    // to align with the subtask's request to use `self.active_cursor_texture` directly.
    // It mirrors the logic previously in `NovadeCompositorState::render_frame`.
    // The interaction between DesktopState and NovadeCompositorState regarding rendering
    // might need further review.
    pub fn render_frame(
        &mut self,
        output_obj: &Output, // The smithay Output object for context
    ) -> Result<(), String> {
        let renderer_arc = match &self.renderer {
            Some(renderer_instance) => renderer_instance.clone(),
            None => {
                tracing::warn!("render_frame called on DesktopState but no renderer is initialized for output {}.", output_obj.name());
                return Err(format!("No active renderer for output {}", output_obj.name()));
            }
        };

        let output_geometry_logical = output_obj.current_geometry().unwrap_or_else(|| {
            tracing::warn!("Output object {} has no current geometry, using default.", output_obj.name());
            Default::default()
        });
        let output_scale = output_obj.current_scale().fractional_scale();

        // Determine physical_output_size using current_mode or fallback to scaled logical size
        let physical_output_size = output_obj.current_mode()
            .map(|mode| mode.size)
            .unwrap_or_else(|| {
                tracing::debug!("Output object {} has no current mode, calculating physical size from logical geometry and scale.", output_obj.name());
                output_geometry_logical.size.to_physical_precise_round(output_scale)
            });

        if physical_output_size.w == 0 || physical_output_size.h == 0 {
            tracing::warn!("Physical output size is zero for output {}, skipping rendering.", output_obj.name());
            return Ok(()); // Skip rendering if output size is invalid
        }

        let mut renderer_guard = renderer_arc.lock().unwrap();

        let begin_frame_rect = Rectangle::from_loc_and_size(Point::from((0,0)), physical_output_size);
        renderer_guard.begin_frame(begin_frame_rect)
            .map_err(|e| format!("Renderer begin_frame failed for output {}: {:?}", output_obj.name(), e))?;

        // Use RenderElement from render::renderer
        let mut render_elements: Vec<crate::compositor::render::renderer::RenderElement<'_, Gles2Texture>> = Vec::new();

        // Iterate over windows and popups, ensuring popups are rendered on top.
        // Smithay's `Space::elements_for_output` should provide elements in a generally correct rendering order
        // (bottom to top). Popups associated with toplevels should be handled correctly if they are
        // part of the space or if we render them in a second pass.
        // For now, let's iterate once and rely on Space's ordering.
        // We might need a separate list for popups if complex stacking is required beyond Space's capabilities.

        let space_elements_on_output = self.space.elements_for_output(output_obj).cloned().collect::<Vec<_>>();

        for window_arc in &space_elements_on_output { // window_arc is Arc<ManagedWindow>
            if !window_arc.is_mapped() { // Skip unmapped windows
                continue;
            }

            let wl_surface = match window_arc.wl_surface() {
                Some(s) => s,
                None => continue, // Should not happen for a mapped ManagedWindow
            };

            if !wl_surface.alive() {
                tracing::trace!("Surface not alive: {:?} on output {}", wl_surface.id(), output_obj.name());
                continue;
            }

            // Get overall window geometry (logical, screen-relative from Space)
            let logical_overall_geo = match self.space.element_geometry(window_arc) {
                Some(geo) => geo,
                None => {
                    tracing::trace!("No geometry for window (domain_id: {:?}) on output {}", window_arc.domain_id, output_obj.name());
                    continue;
                }
            };

            // Convert overall geometry to physical, output-relative
            // The geometry from space is already output-local if space.elements_for_output is used correctly.
            // However, space.element_geometry returns global coordinates.
            // We need to make it relative to the current output's logical position if not already.
            // And then scale.
            // Let's assume logical_overall_geo from space.element_geometry is global.
            // We need to subtract output_geometry_logical.loc to make it output-local before scaling.
            let output_local_logical_loc = logical_overall_geo.loc - output_geometry_logical.loc;
            let physical_overall_loc_on_output = output_local_logical_loc.to_physical_precise_round(output_scale);
            let physical_overall_size = logical_overall_geo.size.to_physical_precise_round(output_scale);
            let physical_overall_geo_on_output = Rectangle::from_loc_and_size(physical_overall_loc_on_output, physical_overall_size);


            let manager_data = window_arc.manager_data.read().unwrap();
            let is_ssd = manager_data.decorations;
            let window_state_data = window_arc.state.read().unwrap(); // For title, activation state

            let client_surface_physical_geo: Rectangle<i32, Physical>;

            if is_ssd {
                // TODO SSD: Draw actual decorations (borders, title bar)
                // For now, we'll just calculate the content area for the client surface.
                // These would be RenderElement::SolidColor or similar.
                // Example: Border thickness and title bar height in logical pixels
                let border_px = (types::DEFAULT_BORDER_SIZE as f64 * output_scale).round() as i32;
                let title_bar_px = (types::DEFAULT_TITLE_BAR_HEIGHT as f64 * output_scale).round() as i32;

                // Define decoration rectangles based on physical_overall_geo_on_output
                // Example: Top border
                // let top_border_rect = Rectangle::from_loc_and_size(
                //     physical_overall_geo_on_output.loc,
                //     Size::from((physical_overall_geo_on_output.size.w, border_px))
                // );
                // render_elements.push(crate::compositor::render::renderer::RenderElement::SolidColor {
                //     color: [0.2, 0.2, 0.2, 1.0], // Example color
                //     geometry: top_border_rect,
                //     damage: &[top_border_rect], // For simplicity, damage whole element
                // });
                // Similar for other borders and title bar background.
                // Text rendering for title is more complex and would be a TODO for the renderer.

                client_surface_physical_geo = Rectangle::from_loc_and_size(
                    Point::from((
                        physical_overall_geo_on_output.loc.x + border_px,
                        physical_overall_geo_on_output.loc.y + title_bar_px + border_px,
                    )),
                    Size::from((
                        (physical_overall_geo_on_output.size.w - 2 * border_px).max(0),
                        (physical_overall_geo_on_output.size.h - title_bar_px - 2 * border_px).max(0),
                    ))
                );
            } else {
                client_surface_physical_geo = physical_overall_geo_on_output;
            }

            // Add client surface content to render elements
            if let Some(surface_data_mutex) = wl_surface.data_map().get::<StdMutex<SurfaceData>>() {
                let surface_data = surface_data_mutex.lock().unwrap();
                if let Some(texture_arc) = surface_data.texture_handle.clone() {
                     // The texture_arc is Arc<dyn RenderableTexture>. We need to downcast or ensure it's the correct type.
                     // Assuming self.renderer uses Gles2Texture as its Self::Texture.
                    if let Some(gles_texture_arc) = texture_arc.as_any().downcast_ref::<Arc<Gles2Texture>>() {
                        render_elements.push(crate::compositor::render::renderer::RenderElement::Surface {
                            texture: gles_texture_arc.clone(),
                            geometry: client_surface_physical_geo,
                            damage: &[], // TODO: Pass actual damage from SurfaceData or Space
                            alpha: manager_data.opacity as f32, // Use window opacity
                            transform: surface_data.current_buffer_info.as_ref().map_or(Transform::Normal, |info| info.transform),
                        });
                        tracing::trace!("Added client surface {:?} content (texture: {:?}) with physical_geo {:?} to render list for output {}", wl_surface.id(), gles_texture_arc.unique_id(), client_surface_physical_geo, output_obj.name());
                    } else {
                        tracing::warn!("Could not downcast RenderableTexture to Arc<Gles2Texture> for surface {:?}", wl_surface.id());
                    }
                } else {
                     tracing::trace!("No texture handle for surface {:?} on output {}", wl_surface.id(), output_obj.name());
                }
            } else {
                tracing::warn!("SurfaceData not found for surface {:?} during rendering.", wl_surface.id());
            }
        }


        let dmabuf_importer = self.dmabuf_importer.as_ref().ok_or_else(|| {
            let err_msg = "DmabufImporter not initialized during render_frame.";
            tracing::error!("{}", err_msg);
            err_msg.to_string()
        })?;

        // The render_frame in CompositorRenderer trait takes `elements: Vec<RenderElement<'a, Self::Texture>>`
        // We are constructing this `render_elements` vector.
        // The damage parameter for `render_frame` refers to output_damage.
        // For individual elements, damage is passed within RenderElement::Surface.
        // TODO: Get actual output damage from DamageTrackerState.
        let output_damage: &[Rectangle<i32, Physical>] = &[]; // Placeholder for actual output damage

        renderer_guard.render_elements(render_elements, output_damage)
            .map_err(|e| format!("Renderer render_elements failed for output {}: {:?}", output_obj.name(), e))?;

        // TODO: Cursor Rendering
        // if let Some(cursor_texture_arc) = &self.active_cursor_texture {
        //     if let Some(gles_cursor_texture_arc) = cursor_texture_arc.as_any().downcast_ref::<Arc<Gles2Texture>>() {
        //          let cursor_render_element = crate::compositor::render::renderer::RenderElement::Cursor {
        //              texture: gles_cursor_texture_arc.clone(),
        //              position: self.pointer_location.to_physical_precise_round(output_scale) + output_geometry_logical.loc.to_physical_precise_round(output_scale), // Ensure cursor pos is output-local
        //              hotspot: self.cursor_hotspot.to_physical_precise_round(output_scale), // Scale hotspot too
        //              damage: &[], // TODO: Cursor damage
        //          };
        //          renderer_guard.render_elements(vec![cursor_render_element], &[]).map_err(|e| format!("Cursor rendering failed: {:?}", e))?;
        //      }
        // }


        renderer_guard.finish_frame()
            .map_err(|e| format!("Renderer finish_frame failed for output {}: {:?}", output_obj.name(), e))?;

        let time_ms = self.clock.now().try_into().unwrap_or_default();
        // Send frame callbacks for all rendered surfaces (elements in space_elements_on_output)
        for window_arc in &space_elements_on_output {
            if let Some(surface_wl) = window_arc.wl_surface() {
                if surface_wl.is_alive() {
                    smithay::wayland::compositor::with_states(&surface_wl, |states| {
                    if let Some(surface_user_data) = states.data_map.get::<std::cell::RefCell<smithay::wayland::compositor::SurfaceData>>() {
                        let mut surface_data_inner = surface_user_data.borrow_mut();
                        if !surface_data_inner.frame_callbacks.is_empty() {
                            tracing::trace!("Sending frame callbacks for surface {:?} on output {}", surface_wl.id(), output_obj.name());
                            for callback in surface_data_inner.frame_callbacks.drain(..) {
                                callback.done(time_ms);
                            }
                        }
                    } else {
                        tracing::warn!("Could not get smithay::wayland::compositor::SurfaceData for surface {:?} during frame callback processing on output {}.", surface_wl.id(), output_obj.name());
                    }
                });
            }
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(xdg_shell_state: XdgShellState) -> Self {
        let mut event_loop: EventLoop<'static, DesktopState> = EventLoop::try_new().unwrap();
        let display_handle = event_loop.handle().insert_source(
            Display::<DesktopState>::new().unwrap(),
            |_, _, _| {},
        ).unwrap();
        let clock = Clock::new(None).expect("Failed to create clock");
        let display_manager = Arc::new(WaylandDisplayManager::new().unwrap());
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let space = Space::new(tracing::info_span!("novade_test_space"));
        let damage_tracker_state = DamageTrackerState::new();
        let mut seat_state = SeatState::new();
        let seat_name = "seat_test".to_string();
        let seat = seat_state.new_wl_seat(&display_handle, seat_name.clone(), None);
        let dmabuf_state = DmabufState::new();
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle);
        let screencopy_state = ScreencopyState::new::<Self>(&display_handle, None);

        let input_dispatcher = InputDispatcher::new();
        let keyboard_layout_manager = KeyboardLayoutManager::new().unwrap();

        // Workspace Initialization (copied from DesktopState::new but simplified for tests)
        let mut workspaces_vec = Vec::new();
        let num_workspaces = 1; // Default to 1 for most tests, can be expanded by test itself
        let mut first_workspace_id = Uuid::nil();
        for i in 0..num_workspaces {
            let ws_name = format!("Test Workspace {}", i + 1);
            let workspace = Arc::new(RwLock::new(CompositorWorkspace::new(ws_name)));
            if i == 0 {
                first_workspace_id = workspace.read().unwrap().id;
            }
            workspaces_vec.push(workspace);
        }
         if workspaces_vec.is_empty() { // Should not happen with num_workspaces > 0
            let default_ws = Arc::new(RwLock::new(CompositorWorkspace::new("Test Workspace 1".to_string())));
            first_workspace_id = default_ws.read().unwrap().id;
            workspaces_vec.push(default_ws);
        }

        Self {
            display_handle,
            loop_handle: event_loop.handle(),
            clock,
            compositor_state,
            shm_state,
            output_manager_state,
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
            dmabuf_importer: None,
            xdg_decoration_state,
            screencopy_state,
            active_renderer_type: ActiveRendererType::Gles,
            renderer: None,
            mcp_connection_service: None,
            cpu_usage_service: None,
            domain_services: None,
            input_dispatcher,
            keyboard_layout_manager,
            active_input_surface: None,
            keyboard_data_map: HashMap::new(),
            touch_focus_per_slot: HashMap::new(),
            display_manager,
            active_cursor_texture: None,
            cursor_hotspot: (0,0).into(),
            layer_shell_data: MyCustomLayerShellData::default(),
            output_workspaces: output_workspaces_map_test,
            active_workspaces: Arc::new(RwLock::new(active_workspaces_map_test)),
            primary_output_name: Arc::new(RwLock::new(Some(test_output_name))), // Ensure primary_output_name is Arc<RwLock<>>
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

                // Check if self.renderer (the new field) is Some
                if let Some(renderer_arc_mutex) = self.renderer.as_ref() {
                    // Lock the renderer. Note: The type here is Arc<Mutex<dyn CompositorRenderer<...>>>
                    let mut renderer_guard = renderer_arc_mutex.lock().unwrap(); // TODO: Handle Mutex poison

                    // Attempt DMABUF import first
                    // The DmabufState already handles the import notification in dmabuf_imported.
                    // Here, we need to use the renderer to create a texture from the Dmabuf object
                    // that Smithay has validated via DmabufHandler.
                    if let Ok(dmabuf) = Dmabuf::try_from(buffer_to_texture.clone()) { // Clone WlBuffer to Dmabuf
                        tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Buffer is DMABUF. Attempting import with CompositorRenderer.");
                        // Use the new import_dmabuf method from CompositorRenderer trait
                        match renderer_guard.import_dmabuf(&dmabuf, Some(surface)) {
                            Ok(texture_arc) => { // Returns Arc<Gles2Texture> or similar
                                surface_data.texture_handle = Some(texture_arc as Arc<dyn crate::compositor::render::renderer::RenderableTexture>);
                                tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully created texture from DMABUF via CompositorRenderer. New buffer info: {:?}", surface_data.current_buffer_info);
                            }
                            Err(err) => {
                                tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Failed to create texture from DMABUF via CompositorRenderer: {:?}", err);
                                surface_data.texture_handle = None;
                                surface_data.current_buffer_info = None;
                            }
                        }
                    } else { // Not a DMABUF, assume SHM.
                        tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Buffer is not DMABUF (or Dmabuf::try_from failed), attempting SHM import with CompositorRenderer.");
                        // Use the new import_shm_buffer method from CompositorRenderer trait
                        match renderer_guard.import_shm_buffer(buffer_to_texture, Some(surface), self) {
                            Ok(texture_arc) => { // Returns Arc<Gles2Texture> or similar
                                surface_data.texture_handle = Some(texture_arc as Arc<dyn crate::compositor::render::renderer::RenderableTexture>);
                                tracing::info!(surface_id = ?surface.id(), client_info = %client_id_str, "Successfully created texture from SHM via CompositorRenderer. New buffer info: {:?}", surface_data.current_buffer_info);
                            }
                            Err(err) => {
                                tracing::error!(surface_id = ?surface.id(), client_info = %client_id_str, "Failed to create texture from SHM via CompositorRenderer: {:?}", err);
                                surface_data.texture_handle = None;
                                surface_data.current_buffer_info = None;
                            }
                        }
                    }
                } else { // self.renderer is None
                    tracing::warn!("self.renderer is None. Cannot import texture for surface_id = {:?}.", surface.id());
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
// delegate_dmabuf!(DesktopState); // Already present, will be verified.
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
    type KeyboardFocus = WlSurface; // Keep as WlSurface for broad compatibility
    type PointerFocus = WlSurface;  // Keep as WlSurface
    type TouchFocus = WlSurface;    // Keep as WlSurface

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let old_focus_wl_surface = self.active_input_surface.as_ref().and_then(Weak::upgrade);

        if old_focus_wl_surface.as_ref().map(|s| s.id()) == focused.map(|s| s.id()) {
            // Focus hasn't meaningfully changed.
            // Ensure the currently focused window (if any) is marked active.
            if let Some(focused_surf) = focused {
                if let Some(window_arc) = self.find_managed_window_by_wl_surface(focused_surf) {
                    let mut win_state = window_arc.state.write().unwrap();
                    if !win_state.activated {
                        win_state.activated = true;
                        // Also update XDG toplevel state if it's a toplevel
                        if let Some(toplevel) = window_arc.xdg_surface.toplevel() {
                            toplevel.with_pending_state(|xdg_state| {
                                xdg_state.states.set(smithay::wayland::shell::xdg::ToplevelState::Activated, true);
                            });
                            toplevel.send_configure();
                             tracing::info!("(Re)activated XDG Toplevel (domain_id: {:?}) on focus_changed (no actual focus change).", window_arc.domain_id);
                        }
                    }
                }
            }
            return;
        }

        tracing::debug!(seat = %seat.name(), new_focus = ?focused.map(|s| s.id()), old_focus = ?old_focus_wl_surface.as_ref().map(|s|s.id()), "Keyboard focus changed (DesktopState::SeatHandler)");

        self.active_input_surface = focused.map(|s| s.downgrade());

        // Deactivate old XDG Toplevel window
        if let Some(old_surface) = old_focus_wl_surface {
            if old_surface.alive() {
                if let Some(window_arc) = self.find_managed_window_by_wl_surface(&old_surface) {
                    window_arc.state.write().unwrap().activated = false;
                    if let Some(toplevel) = window_arc.xdg_surface.toplevel() {
                        toplevel.with_pending_state(|xdg_state| {
                            xdg_state.states.unset(smithay::wayland::shell::xdg::ToplevelState::Activated);
                        });
                        toplevel.send_configure();
                        tracing::info!("Deactivated XDG Toplevel (domain_id: {:?}, surface {:?}) due to focus loss.", window_arc.domain_id, old_surface.id());
                    }
                }
            }
        }

        // Activate new XDG Toplevel window
        if let Some(new_surface) = focused {
            if new_surface.alive() {
                if let Some(window_arc) = self.find_managed_window_by_wl_surface(new_surface) {
                    window_arc.state.write().unwrap().activated = true;
                    if let Some(toplevel) = window_arc.xdg_surface.toplevel() {
                        toplevel.with_pending_state(|xdg_state| {
                            xdg_state.states.set(smithay::wayland::shell::xdg::ToplevelState::Activated, true);
                        });
                        toplevel.send_configure();
                        tracing::info!("Activated XDG Toplevel (domain_id: {:?}, surface {:?}) due to focus gain.", window_arc.domain_id, new_surface.id());
                    }
                    if window_arc.xdg_surface.toplevel().is_some() { // Only raise toplevels
                        self.space.raise_window(&window_arc, true);
                        tracing::debug!("Raised window (domain_id: {:?}) to top due to focus gain.", window_arc.domain_id);
                    }
                }
            }
        }
    }

    fn cursor_image(&mut self, seat: &Seat<Self>, image: CursorImageStatus) {
        tracing::debug!(seat_name = %seat.name(), cursor_status = ?image, "DesktopState::SeatHandler: cursor_image updated");

        let mut current_status_gaurd = self.current_cursor_status.lock().unwrap();
        *current_status_gaurd = image.clone(); // Update the generic cursor status for other parts of the system

        // Ensure this logic uses the new `renderer` field if cursor rendering is to be updated.
        // For now, this logic uses `active_renderer`, which is being phased out.
        // This will be addressed in a subsequent refactoring task.
        // The current subtask is focused on adding DmabufHandler and new fields.
        match image {
            CursorImageStatus::Surface(surface) => {
                let buffer = smithay::wayland::compositor::with_states(&surface, |states| {
                    states.data_map.get::<WlSurfaceAttributes>().unwrap().buffer.clone()
                });

                if let Some(buffer) = buffer {
                    // Use self.renderer for cursor texture creation
                    if let Some(renderer_arc_mutex) = &self.renderer {
                        let mut renderer_guard = renderer_arc_mutex.lock().unwrap(); // TODO: Handle Mutex poison
                        match renderer_guard.import_shm_buffer(&buffer, Some(&surface), self) { // Pass self for DesktopState context
                            Ok(texture_arc) => { // Returns Arc<Gles2Texture> or similar
                                self.active_cursor_texture = Some(texture_arc as Arc<dyn crate::compositor::render::renderer::RenderableTexture>); // Cast and store
                                tracing::info!("Successfully created texture for cursor surface {:?}", surface.id());
                            }
                            Err(err) => {
                                tracing::error!("Failed to create texture for cursor surface {:?}: {:?}", surface.id(), err);
                                self.active_cursor_texture = None;
                            }
                        }
                    } else {
                        tracing::warn!("No active renderer to create cursor texture for surface.");
                        self.active_cursor_texture = None;
                    }
                } else {
                    tracing::warn!("Cursor surface {:?} has no attached buffer.", surface.id());
                    self.active_cursor_texture = None;
                }
            }
            CursorImageStatus::Uploaded { buffer, hotspot } => {
                 // Use self.renderer for cursor texture creation
                if let Some(renderer_arc_mutex) = &self.renderer {
                    let mut renderer_guard = renderer_arc_mutex.lock().unwrap(); // TODO: Handle Mutex poison
                    match renderer_guard.import_shm_buffer(&buffer, None, self) { // Pass self for DesktopState context
                        Ok(texture_arc) => { // Returns Arc<Gles2Texture> or similar
                            self.active_cursor_texture = Some(texture_arc as Arc<dyn crate::compositor::render::renderer::RenderableTexture>); // Cast and store
                            self.cursor_hotspot = hotspot.into();
                            tracing::info!("Successfully created texture for uploaded cursor buffer.");
                        }
                        Err(err) => {
                            tracing::error!("Failed to create texture for uploaded cursor buffer: {:?}", err);
                            self.active_cursor_texture = None;
                        }
                    }
                } else {
                    tracing::warn!("No active renderer to create uploaded cursor texture.");
                    self.active_cursor_texture = None;
                }
            }
            CursorImageStatus::Hidden => {
                self.active_cursor_texture = None;
                tracing::info!("Cursor set to hidden.");
            }
            _ => {
                self.active_cursor_texture = None;
            }
        }
    }

    fn pointer_motion_event(
        &mut self,
        seat: &Seat<Self>,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        self.pointer_location = event.location;
        let serial = SERIAL_COUNTER.next_serial();
        let time = event.time;

        // Determine the surface currently under the pointer
        let new_pointer_focus_details = self.space.surface_under(self.pointer_location, true);

        let pointer_handle = seat.get_pointer().expect("Pointer capability missing on seat for motion event.");

        // Manage enter/leave events
        let old_pointer_focus = pointer_handle.current_focus();
        let new_pointer_focus_surface = new_pointer_focus_details.as_ref().map(|(s, _)| s);

        if old_pointer_focus.as_ref().map(|s| s.id()) != new_pointer_focus_surface.map(|s|s.id()) {
            if let Some(old_focus_strong) = old_pointer_focus {
                if old_focus_strong.alive() {
                    pointer_handle.leave(&old_focus_strong, serial, time);
                    tracing::trace!("Pointer left surface {:?}", old_focus_strong.id());
                }
            }
            if let Some(new_focus_strong) = new_pointer_focus_surface {
                if new_focus_strong.alive() {
                    pointer_handle.enter(new_focus_strong, serial, time, new_pointer_focus_details.as_ref().unwrap().1);
                    tracing::trace!("Pointer entered surface {:?} at {:?}", new_focus_strong.id(), new_pointer_focus_details.as_ref().unwrap().1);
                }
            }
        }

        // Send motion event to the surface currently under the pointer
        if let Some((focused_surface, mut local_coords)) = new_pointer_focus_details {
            if focused_surface.alive() {
                let mut event_consumed_by_ssd = false;
                if let Some(window) = self.find_managed_window_by_wl_surface(&focused_surface) {
                    if window.manager_data.read().unwrap().decorations {
                        let overall_geo = window.geometry();
                        let window_local_pointer_coords_f64 = self.pointer_location - overall_geo.loc.to_f64();

                        let border_f = types::DEFAULT_BORDER_SIZE as f64;
                        let titlebar_f = types::DEFAULT_TITLE_BAR_HEIGHT as f64;

                        let title_bar_rect_local = Rectangle::from_loc_and_size(
                            Point::from((border_f, border_f)),
                            Size::from((overall_geo.size.w as f64 - 2.0 * border_f, titlebar_f))
                        );

                        if title_bar_rect_local.contains(window_local_pointer_coords_f64) {
                            tracing::trace!("Pointer on title bar of SSD window (domain_id: {:?})", window.domain_id);
                            // TODO: Change cursor to move cursor, prepare for move grab on button press.
                            event_consumed_by_ssd = true;
                        }
                        // TODO: Add similar checks for border regions for resize cursors/actions.
                    }
                }
                if !event_consumed_by_ssd {
                    pointer_handle.motion(&focused_surface, serial, time, local_coords);
                }
            }
        }
    }

    fn pointer_button_event(
        &mut self,
        seat: &Seat<Self>,
        event: &smithay::input::pointer::ButtonEvent,
    ) {
        let pointer_handle = seat.get_pointer().expect("Pointer capability missing on seat for button event.");
        let serial = event.serial;
        let time = event.time;
        let button_code = event.button;
        let button_state = event.state;

        let mouse_button = smithay::backend::input::MouseButton::from_code(button_code).unwrap_or(MouseButton::Other(button_code as u16));

        if let Some((surface_under, surface_local_coords)) = self.space.surface_under(self.pointer_location, true) {
            if surface_under.alive() {
                let mut event_consumed_by_ssd = false;
                if let Some(window) = self.find_managed_window_by_wl_surface(&surface_under) {
                    if window.manager_data.read().unwrap().decorations {
                        let overall_geo = window.geometry();
                        let window_local_pointer_coords_f64 = self.pointer_location - overall_geo.loc.to_f64();
                        let border_f = types::DEFAULT_BORDER_SIZE as f64;
                        let titlebar_f = types::DEFAULT_TITLE_BAR_HEIGHT as f64;

                        let title_bar_rect_local = Rectangle::from_loc_and_size(
                            Point::from((border_f, border_f)),
                            Size::from((overall_geo.size.w as f64 - 2.0 * border_f, titlebar_f))
                        );

                        if title_bar_rect_local.contains(window_local_pointer_coords_f64) {
                            tracing::debug!("Button {:?} on title bar of SSD window (domain_id: {:?}). Compositor should handle.", mouse_button, window.domain_id);
                            if mouse_button == MouseButton::Left && button_state == ButtonState::Pressed {
                                // TODO: Initiate interactive move operation.
                                tracing::info!("TODO: Initiate interactive move for window {:?}", window.id);
                            }
                            event_consumed_by_ssd = true;
                        }
                        // TODO: Add similar checks for border clicks to initiate resize.
                    }

                    // Click-to-focus and raise, happens regardless of SSD if event not consumed
                    if !event_consumed_by_ssd && mouse_button == MouseButton::Left && button_state == ButtonState::Pressed {
                        let keyboard = seat.get_keyboard().expect("Keyboard capability missing for click-to-focus.");
                        if keyboard.current_focus().as_ref() != Some(&surface_under) {
                            keyboard.set_focus(self, Some(surface_under.clone()), serial);
                        }
                        if window.xdg_surface.toplevel().is_some() {
                             self.space.raise_window(&window, true);
                        }
                    }
                }

                if !event_consumed_by_ssd {
                    pointer_handle.button(&surface_under, serial, time, mouse_button, button_state);
                    tracing::debug!("Pointer button {:?} ({:?}) state {:?} on surface {:?} (local_coords: {:?}). Serial: {:?}",
                                  mouse_button, button_code, button_state, surface_under.id(), surface_local_coords, serial);
                }
            }
        } else {
             tracing::debug!("Pointer button {:?} ({:?}) state {:?} with no focused surface. Serial: {:?}",
                          mouse_button, button_code, button_state, serial);
        }
    }

    fn pointer_axis_event(
        &mut self,
        seat: &Seat<Self>,
        event: &smithay::input::pointer::AxisEvent,
    ) {
        let pointer_handle = seat.get_pointer().expect("Pointer capability missing on seat for axis event.");
        if let Some((surface_under, _surface_local_coords)) = self.space.surface_under(self.pointer_location, true) {
            if surface_under.alive() {
                let mut axis_frame = smithay::input::pointer::AxisFrame::new(event.time).source(event.source);
                if let Some(h_abs) = event.absolute.get(smithay::input::pointer::Axis::Horizontal) {
                    axis_frame = axis_frame.value(smithay::input::pointer::Axis::Horizontal, h_abs);
                }
                if let Some(v_abs) = event.absolute.get(smithay::input::pointer::Axis::Vertical) {
                     axis_frame = axis_frame.value(smithay::input::pointer::Axis::Vertical, v_abs);
                }
                if let Some(h_disc) = event.discrete.get(smithay::input::pointer::Axis::Horizontal) {
                    axis_frame = axis_frame.discrete(smithay::input::pointer::Axis::Horizontal, h_disc);
                }
                if let Some(v_disc) = event.discrete.get(smithay::input::pointer::Axis::Vertical) {
                     axis_frame = axis_frame.discrete(smithay::input::pointer::Axis::Vertical, v_disc);
                }
                if event.stop.get(&smithay::input::pointer::Axis::Horizontal) == Some(&true) {
                    axis_frame = axis_frame.stop(smithay::input::pointer::Axis::Horizontal);
                }
                if event.stop.get(&smithay::input::pointer::Axis::Vertical) == Some(&true) {
                    axis_frame = axis_frame.stop(smithay::input::pointer::Axis::Vertical);
                }
                pointer_handle.axis(&surface_under, axis_frame);
            }
        }
    }

    fn touch_focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>, _old_focus: Option<&WlSurface>) {
        // TODO: Implement if touch interaction is part of the requirements beyond MVP.
        // Similar logic to keyboard focus_changed regarding activation.
        tracing::trace!("Touch focus changed (DesktopState::SeatHandler) - STUB");
    }
}

// DmabufHandler implementation for DesktopState
impl DmabufHandler for DesktopState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }

    // Called when a client requests to create a DMABUF from parameters.
    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal, // The DmabufGlobal that received the import request
        dmabuf: Dmabuf,
        notifier: ImportNotifier, // Used to signal success or failure of the import
    ) {
        tracing::info!(
            "DMABUF import requested: format={:?}, planes={}, width={}, height={}, flags={:?}",
            dmabuf.format(),
            dmabuf.num_planes(),
            dmabuf.width(),
            dmabuf.height(),
            dmabuf.flags()
        );

        // The actual import into a texture will be handled by the active renderer
        // when the buffer is attached to a surface via `CompositorHandler::commit`.
        // Here, we just need to acknowledge the DMABUF parameters are understood by Smithay.
        // If the renderer needed to pre-validate or pre-allocate here, this is where it would go.
        // For now, we assume the parameters are valid if Smithay created the Dmabuf object.

        // TODO: If we had a way to check if the *current* renderer supports this dmabuf
        // (format, modifiers), we could do it here.
        // For now, we optimistically succeed the import at the protocol level.
        // The real test comes at commit time.

        if self.renderer.is_none() { // Check the new renderer field
            tracing::error!("No active renderer to handle DMABUF import notification.");
            // Even if no renderer, Smithay has "accepted" the buffer based on params.
            // The failure will occur at commit time if the (future) renderer can't use it.
            // It might be better to fail here if no renderer is present.
            // However, the DmabufHandler trait expects us to call success or failure on the notifier.
            // Let's assume for now that if smithay gives us a Dmabuf, the params are okay.
        }

        // We don't create a texture here. That happens on WlSurface.commit.
        // We just notify that the dmabuf object itself is valid from protocol perspective.
        if let Err(e) = notifier.successful::<DesktopState>() {
             tracing::warn!("Failed to notify DMABUF import success: {:?}", e);
        }

        tracing::info!("DMABUF object {:?} successfully acknowledged at protocol level.", dmabuf.handles().nth(0));
    }
}

// ANCHOR: WorkspaceCoreStateTests
#[cfg(test)]
mod state_tests {
    use super::*;
    use crate::compositor::shell::xdg_shell::types::{ManagedWindow, WindowState, XdgSurfaceRole, XdgSurfaceUserData, XdgSurfaceState};
    use smithay::reexports::wayland_server::{
        protocol::wl_surface::WlSurface,
        globals::GlobalData, Main,
        backend::{ClientData, ClientId, Handle, ObjectData, ObjectId, DisconnectReason},
        Interface, Message,
    };
    use smithay::wayland::shell::xdg::{XdgShellState, XdgActivationState, ToplevelSurface, PopupSurface, XdgSurface as SmithayXdgSurface, WindowSurface};
    use smithay::utils::Rectangle;


    // Minimal test client data
    #[derive(Default, Clone)]
    struct TestClientData { user_data: UserData }
    impl ClientData for TestClientData {
        fn initialized(&self, _client_id: ClientId) {}
        fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
        fn data_map(&self) -> &UserData { &self.user_data }
    }

    // Mock ObjectData for WlSurface
    #[derive(Default)]
    struct TestObjectData;
    impl<R: Interface + AsRef<Resource<R>> + Unpin + 'static> ObjectData<R> for TestObjectData {
        fn request(self: Arc<Self>, _handle: &Handle, _client_data: &mut dyn ClientData, _client_id: ClientId, _msg: Message<R>) -> Option<Arc<dyn ObjectData<R>>> { None }
        fn destroyed(self: Arc<Self>, _client_id: ClientId, _object_id: ObjectId) {}
    }

    fn mock_managed_window(state: &mut DesktopState, client: &Client, name_suffix: &str) -> Arc<ManagedWindow> {
        let dh = state.display_handle.clone();
        let wl_surface_main = client.create_object::<WlSurface, _>(&dh, WlSurface::interface().version, Arc::new(TestObjectData)).unwrap();
        let wl_surface = wl_surface_main.as_ref();

        wl_surface.data_map().insert_if_missing_threadsafe(|| Arc::new(smithay::wayland::compositor::SurfaceData::new(None, Rectangle::from_loc_and_size((0,0),(0,0)))));
        wl_surface.data_map().insert_if_missing_threadsafe(|| smithay::wayland::shell::xdg::XdgSurfaceData::new());

        let smithay_xdg_surface = SmithayXdgSurface::new_unassigned(wl_surface.clone());
        let xdg_user_data = Arc::new(XdgSurfaceUserData::new(wl_surface.clone()));
        *xdg_user_data.role.lock().unwrap() = XdgSurfaceRole::Toplevel; // Pretend it's a toplevel
        smithay_xdg_surface.user_data().insert_if_missing_threadsafe(|| xdg_user_data);

        let toplevel_surface = ToplevelSurface::from_xdg_surface(smithay_xdg_surface, Default::default()).unwrap();
        toplevel_surface.set_title(format!("Test Window {}", name_suffix));

        let domain_id = DomainWindowIdentifier::new_v4();
        let mw = ManagedWindow::new_toplevel(toplevel_surface, domain_id);
        Arc::new(mw)
    }


    #[test]
    fn test_desktop_state_new_initializes_workspaces() {
        let xdg_shell_state = XdgShellState::new_with_activation(
            &Display::<DesktopState>::new().unwrap().handle(), // Dummy display handle for init
            &XdgActivationState::new()
        ).0;
        let state = DesktopState::new_for_test(xdg_shell_state);

        assert_eq!(state.compositor_workspaces.len(), 1, "Should initialize with 1 workspace in test mode"); // new_for_test creates 1

        let active_ws_id = *state.active_compositor_workspace_id.read().unwrap();
        let first_ws_id = state.compositor_workspaces[0].read().unwrap().id;
        assert_eq!(active_ws_id, first_ws_id, "Active workspace should be the first one by default");
        assert_eq!(state.compositor_workspaces[0].read().unwrap().name, "Test Workspace 1");
    }

    #[test]
    fn test_switch_to_workspace_by_id() {
        let xdg_shell_state = XdgShellState::new_with_activation(
            &Display::<DesktopState>::new().unwrap().handle(),
            &XdgActivationState::new()
        ).0;
        let mut state = DesktopState::new_for_test(xdg_shell_state);
        let client = state.display_handle.create_client(TestClientData::default().into());

        // Create more workspaces for testing switch
        let ws2_name = "Test Workspace 2".to_string();
        let workspace2 = Arc::new(RwLock::new(CompositorWorkspace::new(ws2_name.clone())));
        let ws2_id = workspace2.read().unwrap().id;
        state.compositor_workspaces.push(workspace2.clone());

        let initial_active_ws_id = *state.active_compositor_workspace_id.read().unwrap();
        assert_ne!(initial_active_ws_id, ws2_id);

        // Create windows and assign them to workspaces
        let window1 = mock_managed_window(&mut state, &client, "1");
        *window1.workspace_id.write().unwrap() = Some(initial_active_ws_id);
        state.compositor_workspaces[0].read().unwrap().add_window(window1.domain_id);
        state.windows.insert(window1.domain_id, window1.clone());
        state.space.map_window(window1.clone(), (0,0).into(), false); // Map to initial active workspace
        window1.state.write().unwrap().is_mapped = true;


        let window2 = mock_managed_window(&mut state, &client, "2");
        *window2.workspace_id.write().unwrap() = Some(ws2_id);
        workspace2.read().unwrap().add_window(window2.domain_id);
        state.windows.insert(window2.domain_id, window2.clone());
        // window2 is initially unmapped as it's on an inactive workspace

        // Switch to workspace 2
        state.switch_to_workspace_by_id(ws2_id);

        assert_eq!(*state.active_compositor_workspace_id.read().unwrap(), ws2_id);
        assert!(!window1.state.read().unwrap().is_mapped, "Window1 on old workspace should be unmapped");
        assert!(window2.state.read().unwrap().is_mapped, "Window2 on new workspace should be mapped");
        assert!(state.space.element_for_surface(&window2.wl_surface().unwrap()).is_some(), "Window2 should be in space");
        assert!(state.space.element_for_surface(&window1.wl_surface().unwrap()).is_none(), "Window1 should not be in space");

        // Switch back
        state.switch_to_workspace_by_id(initial_active_ws_id);
        assert_eq!(*state.active_compositor_workspace_id.read().unwrap(), initial_active_ws_id);
        assert!(window1.state.read().unwrap().is_mapped, "Window1 should be re-mapped");
        assert!(!window2.state.read().unwrap().is_mapped, "Window2 should be unmapped again");
    }
    use crate::compositor::workspaces::TilingLayout; // For setting tiling layout in tests

    #[test]
    fn test_set_workspace_tiling_layout_updates_mode_and_applies() {
        let xdg_shell_state = XdgShellState::new_with_activation(
            &Display::<DesktopState>::new().unwrap().handle(),
            &XdgActivationState::new()
        ).0;
        let mut state = DesktopState::new_for_test(xdg_shell_state);
        let client = state.display_handle.create_client(TestClientData::default().into());

        let active_ws_id = *state.active_compositor_workspace_id.read().unwrap();
        let active_ws_arc = state.compositor_workspaces.iter()
            .find(|ws| ws.read().unwrap().id == active_ws_id).unwrap().clone();

        // Add a window to the active workspace
        let window1 = mock_managed_window(&mut state, &client, "W1");
        *window1.workspace_id.write().unwrap() = Some(active_ws_id);
        active_ws_arc.read().unwrap().add_window(window1.domain_id);
        state.windows.insert(window1.domain_id, window1.clone());

        let initial_floating_geom = Rectangle::from_loc_and_size((10,10), (100,100));
        *window1.current_geometry.write().unwrap() = initial_floating_geom;
        window1.state.write().unwrap().is_mapped = true;
        state.space.map_window(window1.clone(), initial_floating_geom.loc, false);

        assert_eq!(*active_ws_arc.read().unwrap().tiling_layout.read().unwrap(), TilingLayout::None);

        // Set tiling layout to MasterStack
        state.set_workspace_tiling_layout(active_ws_id, TilingLayout::MasterStack);

        assert_eq!(*active_ws_arc.read().unwrap().tiling_layout.read().unwrap(), TilingLayout::MasterStack);

        let output_geom = state.space.outputs().next().and_then(|o| state.space.output_geometry(o))
                            .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0),(800,600)));

        let tiled_geom = *window1.current_geometry.read().unwrap();
        assert_eq!(tiled_geom, output_geom, "Window geometry should be updated to full workspace area after tiling for a single window.");
        assert!(window1.state.read().unwrap().is_mapped, "Window should remain mapped.");
    }

    #[test]
    fn test_set_workspace_tiling_layout_for_inactive_workspace() {
        let xdg_shell_state = XdgShellState::new_with_activation(
            &Display::<DesktopState>::new().unwrap().handle(),
            &XdgActivationState::new()
        ).0;
        let mut state = DesktopState::new_for_test(xdg_shell_state);

        let ws2 = Arc::new(RwLock::new(CompositorWorkspace::new("Inactive WS".to_string())));
        let ws2_id = ws2.read().unwrap().id;
        state.compositor_workspaces.push(ws2.clone());

        let active_ws_id = *state.active_compositor_workspace_id.read().unwrap();
        assert_ne!(active_ws_id, ws2_id, "Workspace 2 should be inactive.");

        state.set_workspace_tiling_layout(ws2_id, TilingLayout::MasterStack);
        assert_eq!(*ws2.read().unwrap().tiling_layout.read().unwrap(), TilingLayout::MasterStack);
    }
}
// ANCHOR_END: WorkspaceCoreStateTests
