use std::{
    ffi::OsString,
    sync::{Arc, Mutex}, // Using std::sync::Mutex for DesktopState fields
};

use smithay::{
    backend::renderer::{
        gles2::{Gles2Renderer, Gles2Error}, // Keep Gles2 types if still used by OpenGLRenderer internally or for examples
    },
    desktop::{Space, Window, PopupManager},
    input::{Seat, SeatState, pointer::{CursorImageStatus, GrabStartData}, keyboard::{ModifiersState, XkbConfig}, InputHandler, InputState as InputStateSmithay},
    output::{Output as SmithayOutput, OutputHandler, OutputState as OutputStateSmithay}, // Renamed Smithay's Output
    reexports::{
        calloop::{EventLoop, LoopHandle, timer::Timer}, // Added Timer for ask_for_render
        wayland_server::{
            backend::{ClientId, DisconnectReason},
            protocol::{wl_output, wl_surface::WlSurface, wl_seat},
            Display, DisplayHandle,
        },
    },
    utils::{Clock, Logical, Point, Rectangle, SERIAL_COUNTER},
    wayland::{
        compositor::{CompositorHandler, CompositorState, CompositorClientState},
        data_device::{DataDeviceHandler, DataDeviceState},
        output::OutputManagerState,
        shell::xdg::{self as smithay_xdg_shell, XdgShellHandler, XdgShellState, XdgSurface, XdgPositioner, xdg_toplevel},
        shm::{ShmHandler, ShmState},
        socket::ListeningSocketSource,
    },
    delegate_xdg_shell::{self, PopupManagerHandler},
    xdg::popup::Popup,
    reexports::wayland_protocols::unstable::xdg_decoration::v1::server::{
        zxdg_decoration_manager_v1,
        zxdg_toplevel_decoration_v1,
    },
};
use crate::compositor::shell::xdg_decoration::XdgDecorationManagerState;
use crate::window_management::NovaWindowManager;
use novade_domain::workspaces::manager::WorkspaceManager;
use novade_domain::workspaces::traits::WindowManager as DomainWindowManagerTrait;
use crate::display_management::{DisplayManager, ManagedOutput};
use novade_core::types::geometry::Rect as NovaRect;
use novade_core::types::geometry::Point as NovaPoint;
use novade_core::types::geometry::Size as NovaSize;
use tracing::{debug, error, info, warn};
use std::sync::Mutex as StdMutex; // Standard library Mutex for our data
use wayland_server::backend::ClientData;
use crate::compositor::render::gl::OpenGLRenderer;
use crate::compositor::layer_shell::state::LayerSurfaceData; // Added for layer_shell
use smithay::wayland::shell::wlr_layer::{Layer, Anchor}; // Added for layer_shell method signatures
use smithay::desktop::WindowSurfaceType; // For apply_layout_for_workspace


// Dummy structs
pub struct NovaDEConfiguration;
pub struct NovaDEDomainServices;
pub struct BackendData {
    pub renderer: OpenGLRenderer,
}

pub struct DesktopState {
    pub display: Display<Self>,
    pub event_loop_handle: calloop::Handle<'static, Self>,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub xdg_decoration_state: XdgDecorationManagerState,
    pub shm_state: ShmState,
    pub data_device_state: DataDeviceState,
    pub output_state: OutputStateSmithay,
    pub input_state: InputStateSmithay,
    pub seat: Seat<Self>,
    pub popups: PopupManager,
    pub workspace_manager: Arc<StdMutex<WorkspaceManager>>,
    pub display_manager: Arc<StdMutex<DisplayManager>>,
    pub backend_data: Arc<StdMutex<BackendData>>,
    pub space: Space<Window>,
    pub layer_surfaces: Vec<LayerSurfaceData>, // Added for layer_shell
    pub render_timer: calloop::timer::Timer, // Added for ask_for_render
}

impl DesktopState {
    pub fn new(event_loop: &mut EventLoop<'static, Self>, mut display: Display<Self>, self_arc_for_nwm: Arc<StdMutex<DesktopState>>) -> Self {
        info!("Initializing NovaDE DesktopState");
        let display_handle = display.handle();

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let xdg_decoration_state = XdgDecorationManagerState::new();
        let shm_state = ShmState::new::<Self>(&display_handle, Vec::new());
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        let output_state = OutputStateSmithay::new();
        let input_state = InputStateSmithay::new();

        display_handle.create_global::<DesktopState, zxdg_decoration_manager_v1::ZxdgDecorationManagerV1, crate::compositor::shell::xdg_decoration::XdgDecorationManagerGlobalData>(
            1,
            crate::compositor::shell::xdg_decoration::XdgDecorationManagerGlobalData::default(),
        );

        // Initialize Layer Shell global
        smithay::wayland::shell::wlr_layer::WlrLayerShellState::new_global::<DesktopState>(
            &display_handle,
        );


        let seat_name = "seat0";
        let mut seat = Seat::new(&display_handle, seat_name.to_string());
        seat.add_pointer();
        seat.add_keyboard(XkbConfig::default(), 200, 25).expect("Failed to create keyboard");

        let popups = PopupManager::new();
        let mut space = Space::new(info_span!("space"));

        let mut display_manager = DisplayManager::new();
        // Initial population from space.outputs() is tricky as outputs are added by backend later.
        // This loop might be empty at this stage. OutputHandler::new_output is the main population point.
        for output in space.outputs() {
            let (managed_output, _is_first) =
                Self::convert_smithay_output_to_managed(&output, &space, display_manager.all_outputs().is_empty());
            display_manager.add_output(managed_output);
        }
        let display_manager_arc = Arc::new(StdMutex::new(display_manager));

        let nova_window_manager = Arc::new(NovaWindowManager::new(self_arc_for_nwm.clone()).expect("Failed to create NovaWindowManager"));
        let workspace_manager = Arc::new(StdMutex::new(WorkspaceManager::new(nova_window_manager as Arc<dyn DomainWindowManagerTrait>)));

        let opengl_renderer = OpenGLRenderer::new(display_handle.clone()).expect("Failed to create OpenGLRenderer");
        info!("OpenGLRenderer initialized successfully for DesktopState.");
        let backend_data = Arc::new(StdMutex::new(BackendData { renderer: opengl_renderer }));

        let render_timer = Timer::new().unwrap();

        info!("NovaDE DesktopState initialized successfully");
        Self {
            display, event_loop_handle: event_loop.handle(), compositor_state, xdg_shell_state,
            xdg_decoration_state, shm_state, data_device_state, output_state, input_state, seat,
            popups, workspace_manager, display_manager: display_manager_arc, backend_data, space,
            layer_surfaces: Vec::new(), // Initialize layer_surfaces
            render_timer, // Initialize render_timer
        }
    }

    fn convert_smithay_output_to_managed(output: &SmithayOutput, space: &Space<Window>, is_first_output: bool) -> (ManagedOutput, bool) {
        let geometry_opt = space.output_geometry(output);
        let name = output.name();
        let description = output.description();
        let (pos, size) = geometry_opt.map_or(((0,0).into(), (1920,1080).into()), |g| (g.loc, g.size));
        let nova_geometry = NovaRect::new(NovaPoint::new(pos.x, pos.y), NovaSize::new(size.w, size.h));
        let work_area = nova_geometry;
        let scale = output.current_scale().fractional_scale();
        let is_primary = is_first_output;
        let current_drm_mode = None;

        (
            ManagedOutput {
                id: name.clone(), name, description,
                geometry: nova_geometry, work_area, scale, is_primary,
                current_drm_mode, smithay_output: output.clone(),
            },
            is_first_output
        )
    }

    pub async fn switch_to_workspace(&self, new_workspace_id: novade_domain::workspaces::manager::WorkspaceId) -> Result<(), String> {
        let old_active_ws_id_opt: Option<novade_domain::workspaces::manager::WorkspaceId>;
        let windows_to_hide: Vec<novade_domain::workspaces::core::WindowId>;
        let windows_to_show: Vec<novade_domain::workspaces::core::WindowId>;
        let window_manager_ref: Arc<dyn DomainWindowManagerTrait>;
        let new_ws_monitor_id: Option<String>;

        {
            let mut wm_guard = self.workspace_manager.lock().map_err(|e| format!("Failed to lock WM: {:?}", e))?;
            old_active_ws_id_opt = wm_guard.get_active_workspace_id();

            if old_active_ws_id_opt == Some(new_workspace_id) { return Ok(()); }

            windows_to_hide = if let Some(old_id) = old_active_ws_id_opt {
                wm_guard.get_workspace_by_id(old_id).map_or(Vec::new(), |ws| ws.windows.clone())
            } else { Vec::new() };

            wm_guard.switch_workspace(new_workspace_id).await.map_err(|e| format!("WM failed to switch: {:?}", e))?;

            let new_ws = wm_guard.get_workspace_by_id(new_workspace_id).ok_or_else(||"Newly active ws not found".to_string())?;
            windows_to_show = new_ws.windows.clone();
            new_ws_monitor_id = new_ws.monitor_id.clone();
            window_manager_ref = wm_guard.window_manager.clone();
        }

        info!("Switched active workspace from {:?} to {:?}, on monitor {:?}", old_active_ws_id_opt, new_workspace_id, new_ws_monitor_id);

        for window_id in windows_to_hide {
            if !windows_to_show.contains(&window_id) {
                if let Err(e) = window_manager_ref.hide_window_for_workspace(window_id).await {
                    warn!("Failed to hide window {:?}: {}", window_id, e);
                }
            }
        }

        let mut first_window_to_focus: Option<novade_domain::workspaces::core::WindowId> = None;
        for window_id in windows_to_show {
            if let Err(e) = window_manager_ref.show_window_for_workspace(window_id).await {
                warn!("Failed to show window {:?}: {}", window_id, e);
            }
            if first_window_to_focus.is_none() { first_window_to_focus = Some(window_id); }
        }

        if let Some(window_id) = first_window_to_focus {
            if let Err(e) = window_manager_ref.focus_window(window_id).await {
                warn!("Failed to focus window {:?} in new ws: {}", window_id, e);
            }
        }

        // Apply layout for the new active workspace
        if let Some(new_active_id) = self.workspace_manager.lock().unwrap().get_active_workspace_id() {
            self.apply_layout_for_workspace(new_active_id);
        }
        Ok(())
    }

    pub fn create_new_workspace_sync(&self, name: String) -> Result<novade_domain::workspaces::manager::WorkspaceId, String> {
        let target_monitor_id = futures::executor::block_on(
            self.workspace_manager.lock().unwrap().window_manager.get_focused_output_id()
        ).ok().flatten(); // ANCHOR: block_on in sync method
        self.workspace_manager.lock().map_err(|e| format!("Failed to lock WM: {:?}",e))?
            .create_workspace(name, target_monitor_id)
    }

    pub fn get_available_workspaces_sync(&self) -> Result<Vec<(novade_domain::workspaces::manager::WorkspaceId, String, Option<String>)>, String> {
        Ok(self.workspace_manager.lock().map_err(|e| format!("Failed to lock WM: {:?}",e))?
            .get_all_workspaces().iter()
            .map(|ws| (ws.id, ws.name.clone(), ws.monitor_id.clone()))
            .collect())
    }

    pub fn get_current_workspace_id_sync(&self) -> Option<novade_domain::workspaces::manager::WorkspaceId> {
        self.workspace_manager.lock().ok()?.get_active_workspace_id()
    }

    fn find_smithay_window_by_domain_id(&self, domain_id: novade_domain::workspaces::core::WindowId) -> Option<Window> {
        self.space.elements().find(|w| {
            w.wl_surface().and_then(|surface| {
                surface.data::<StdMutex<crate::compositor::shell::xdg::XdgSurfaceData>>().and_then(|data_mutex| {
                    data_mutex.lock().ok().and_then(|data| data.domain_id.filter(|id| *id == domain_id))
                })
            }).is_some()
        }).cloned()
    }

    fn apply_active_workspace_layout(&self) {
        let workspace_manager_guard = self.workspace_manager.lock().unwrap();
        let active_workspace_opt = workspace_manager_guard.get_active_workspace();

        let (target_monitor_id_opt, windows_in_ws, layout_config_opt) =
            if let Some(ws) = active_workspace_opt {
                if matches!(ws.layout, novade_domain::workspaces::manager::WorkspaceLayout::Tiling(_)) {
                    (ws.monitor_id.clone(), ws.windows.clone(), Some(ws.layout.clone()))
                } else { return; }
            } else {
                warn!("apply_active_workspace_layout: No active workspace found.");
                return;
            };

        let window_manager_interface = workspace_manager_guard.window_manager.clone();
        drop(workspace_manager_guard);

        let screen_area_nova_rect: NovaRect = if let Some(monitor_id) = target_monitor_id_opt.as_ref() {
            match futures::executor::block_on(window_manager_interface.get_output_work_area(monitor_id)) { // ANCHOR: block_on
                Ok(rect) => rect,
                Err(e) => {
                    warn!("Failed to get work area for monitor {}: {}. Falling back to primary or default.", monitor_id, e);
                    futures::executor::block_on(window_manager_interface.get_primary_output_id()) // ANCHOR: block_on
                        .ok().flatten().and_then(|pid| futures::executor::block_on(window_manager_interface.get_output_work_area(&pid)).ok()) // ANCHOR: block_on
                        .unwrap_or_else(|| {
                            warn!("No primary output found for screen_area, using default 1920x1080.");
                            NovaRect::new(NovaPoint::new(0,0), NovaSize::new(1920,1080))
                        })
                }
            }
        } else {
            futures::executor::block_on(window_manager_interface.get_primary_output_id()) // ANCHOR: block_on
                .ok().flatten().and_then(|pid| futures::executor::block_on(window_manager_interface.get_output_work_area(&pid)).ok()) // ANCHOR: block_on
                .unwrap_or_else(|| {
                    warn!("No primary output found for screen_area (workspace has no monitor_id), using default 1920x1080.");
                    NovaRect::new(NovaPoint::new(0,0), NovaSize::new(1920,1080))
                })
        };

        if let Some(novade_domain::workspaces::manager::WorkspaceLayout::Tiling(tiling_options)) = layout_config_opt {
            info!("Applying tiling layout ({:?}) for active workspace on monitor {:?} with area {:?}", tiling_options, target_monitor_id_opt, screen_area_nova_rect);
            let algorithm = tiling_options.as_algorithm();
            let new_geometries = algorithm.arrange(&windows_in_ws, screen_area_nova_rect);

            for (domain_window_id, new_geom) in new_geometries {
                if let Some(smithay_window) = self.find_smithay_window_by_domain_id(domain_window_id) {
                    info!("Applying geometry {:?} to window {:?}", new_geom, domain_window_id);
                    self.space.map_element(smithay_window.clone(), (new_geom.position.x, new_geom.position.y), false);
                    if let Some(toplevel) = smithay_window.toplevel() {
                        match toplevel {
                            smithay::desktop::WindowSurfaceType::Xdg(xdg_toplevel_surface) => {
                                let xdg_toplevel = xdg_toplevel_surface.xdg_toplevel();
                                xdg_toplevel.send_configure_bounds(smithay::utils::Size::from((new_geom.size.width, new_geom.size.height)));
                                xdg_toplevel.set_maximized(false);
                                xdg_toplevel.set_fullscreen(false);
                                xdg_toplevel.set_resizing(false);
                                xdg_toplevel.send_pending_configure();

                                if let Some(surface) = smithay_window.wl_surface(){
                                    if let Some(data_mutex) = surface.data::<StdMutex<crate::compositor::shell::xdg::XdgSurfaceData>>() {
                                        if let Ok(mut surface_data) = data_mutex.lock() {
                                            if let crate::compositor::shell::xdg::XdgRoleSpecificData::Toplevel(toplevel_data) = &mut surface_data.role_data {
                                                toplevel_data.current_state.maximized = false;
                                                toplevel_data.current_state.fullscreen = false;
                                                toplevel_data.current_state.resizing = false;
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    warn!("Could not find SmithayWindow for domain_id {:?} during layout application.", domain_window_id);
                }
            }
        }
    }

    pub fn set_workspace_layout(&self,
        workspace_id: novade_domain::workspaces::manager::WorkspaceId,
        new_layout: novade_domain::workspaces::manager::WorkspaceLayout
    ) -> Result<(), String> {
        let mut workspace_manager = self.workspace_manager.lock().map_err(|e| format!("Failed to lock WorkspaceManager: {:?}", e))?;
        let mut was_active = false;
        if let Some(workspace) = workspace_manager.get_mut_workspace_by_id(workspace_id) {
            workspace.layout = new_layout;
            was_active = workspace.active;
            info!("Set layout for workspace {:?} to {:?}. Was active: {}", workspace_id, workspace.layout, was_active);
        } else {
            return Err(format!("Workspace {:?} not found.", workspace_id));
        }
        drop(workspace_manager);
        if was_active {
            info!("Applying new layout to active workspace {:?}.", workspace_id);
            self.apply_layout_for_workspace(workspace_id);
        }
        Ok(())
    }

    pub async fn move_window_to_monitor_by_id(&self, window_id: novade_domain::workspaces::core::WindowId, target_monitor_id: String) -> Result<(), String> {
        info!("Request to move window {:?} to monitor {}", window_id, target_monitor_id);

        let source_workspace_id_opt: Option<novade_domain::workspaces::manager::WorkspaceId>;
        let target_workspace_id: novade_domain::workspaces::manager::WorkspaceId;

        // Scope for workspace_manager lock to update domain state
        {
            let mut wm_guard = self.workspace_manager.lock().map_err(|e| format!("Failed to lock WorkspaceManager: {:?}", e))?;
            source_workspace_id_opt = wm_guard.find_workspace_for_window(window_id);
            target_workspace_id = wm_guard.move_window_to_monitor(window_id, target_monitor_id.clone())
                .map_err(|e| format!("WorkspaceManager failed to move window: {:?}", e))?;
        } // WorkspaceManager lock released

        let smithay_window = self.find_smithay_window_by_domain_id(window_id)
            .ok_or_else(|| format!("Failed to find SmithayWindow for domain ID {:?}", window_id))?;

        let target_smithay_output = self.outputs().find(|o| o.name() == target_monitor_id).cloned();

        if let Some(target_output) = target_smithay_output {
            // Unmap from old output (optional, map_element might handle this implicitly by remapping)
            // smithay_window.unmap_output(); // This API might not exist directly on Window, but on Space or Output.
            // Smithay's Space::map_element handles output assignment based on global coordinates.
            // We need to ensure the window's position is within the target output's area.

            let target_output_geometry = self.space.output_geometry(&target_output)
                .ok_or_else(|| format!("Could not get geometry for target output {}", target_monitor_id))?;

            // Calculate a new position on the target monitor (e.g., center or a default spot)
            let new_pos_global = target_output_geometry.loc; // Top-left for now

            self.space.map_element(smithay_window.clone(), new_pos_global, false); // Don't activate automatically
            info!("Mapped SmithayWindow {:?} to new position {:?} on target monitor {}", window_id, new_pos_global, target_monitor_id);

            // Re-apply layout for the source workspace (if it was different and active)
            if let Some(sws_id) = source_workspace_id_opt {
                if sws_id != target_workspace_id {
                    self.apply_layout_for_workspace(sws_id);
                }
            }

            // Apply layout for the target workspace.
            self.apply_layout_for_workspace(target_workspace_id);

            // Focus the moved window
            let window_manager_interface = self.workspace_manager.lock().unwrap().window_manager.clone();
            if let Err(e) = window_manager_interface.focus_window(window_id).await {
                warn!("Failed to focus moved window {:?}: {}", window_id, e);
            }
             self.ask_for_render(); // Content changed
            Ok(())
        } else {
            Err(format!("Target monitor {} (SmithayOutput) not found.", target_monitor_id))
        }
    }

    // Placeholder methods for layer shell integration
    // These would need actual logic based on how outputs and usable areas are managed.

    /// Returns an iterator over all managed outputs.
    /// This needs to be adapted to how DisplayManager or Space provides outputs.
    pub fn outputs<'a>(&'a self) -> impl Iterator<Item = &'a SmithayOutput> {
        // Assuming DisplayManager holds SmithayOutput instances or can provide them.
        // This is a common pattern but needs DisplayManager to expose this.
        // For now, using space.outputs() as a direct source from Smithay.
        self.space.outputs()
    }


    /// Recalculates the usable area of a given output, considering exclusive zones from layer shells.
    pub fn recalculate_output_usable_area(&mut self, output: SmithayOutput, dh: &DisplayHandle) {
        debug!(output_name = %output.name(), "Recalculating usable area for output (placeholder)");
        // 1. Get the full geometry of the output.
        //    ManagedOutput in DisplayManager should store this.
        //    Or, self.space.output_geometry(&output)
        let output_geometry = match self.space.output_geometry(&output) {
            Some(geom) => geom,
            None => {
                warn!("Cannot recalculate usable area for output {}: not mapped in space.", output.name());
                return;
            }
        };

        let mut exclusive_top = 0;
        let mut exclusive_bottom = 0;
        let mut exclusive_left = 0;
        let mut exclusive_right = 0;

        // 2. Iterate over all layer_surfaces that are on this output.
        for ls_data in self.layer_surfaces.iter().filter(|ls| ls.assigned_output_name.as_ref() == Some(&output.name())) {
            if ls_data.has_exclusive_zone() {
                // This is a simplified model. A more accurate one would check the actual
                // geometry of the layer surface and how it cuts into the output.
                // For example, a panel only anchored to top with exclusive zone X means top X pixels are reserved.
                let state = ls_data.layer_surface.current_state(); // get current state
                if state.anchor.contains(Anchor::TOP) && state.exclusive_zone > 0 {
                    exclusive_top = exclusive_top.max(state.exclusive_zone);
                }
                if state.anchor.contains(Anchor::BOTTOM) && state.exclusive_zone > 0 {
                    exclusive_bottom = exclusive_bottom.max(state.exclusive_zone);
                }
                if state.anchor.contains(Anchor::LEFT) && state.exclusive_zone > 0 {
                    exclusive_left = exclusive_left.max(state.exclusive_zone);
                }
                if state.anchor.contains(Anchor::RIGHT) && state.exclusive_zone > 0 {
                    exclusive_right = exclusive_right.max(state.exclusive_zone);
                }
            }
        }

        // 3. Calculate the new usable_area.
        let usable_x = output_geometry.loc.x + exclusive_left;
        let usable_y = output_geometry.loc.y + exclusive_top;
        let usable_w = (output_geometry.size.w - (exclusive_left + exclusive_right)).max(0);
        let usable_h = (output_geometry.size.h - (exclusive_top + exclusive_bottom)).max(0);
        let new_usable_area = Rectangle::from_loc_and_size((usable_x, usable_y), (usable_w, usable_h));

        // 4. Store this new_usable_area in your DisplayManager's ManagedOutput state.
        let mut dm = self.display_manager.lock().unwrap();
        if let Some(managed_output) = dm.get_output_mut(&output.name()) {
            let old_usable_area = managed_output.work_area;
            managed_output.work_area = NovaRect::new(
                NovaPoint::new(new_usable_area.loc.x, new_usable_area.loc.y),
                NovaSize::new(new_usable_area.size.w, new_usable_area.size.h)
            );
            info!(output_name = %output.name(), ?old_usable_area, new_work_area = ?managed_output.work_area, "Updated output usable area.");

            // 5. If the usable area changed, XDG Shell clients (windows) on this output
            //    might need to be reconfigured/re-arranged.
            if old_usable_area != managed_output.work_area {
                // This could trigger re-tiling or re-placement of windows.
                self.arrange_windows_on_output(output.clone(), dh);
            }
        } else {
            warn!("Output {} not found in DisplayManager during usable area update.", output.name());
        }
        self.ask_for_render(); // Usable area might affect rendering
    }

    /// Arranges layer shell surfaces on a given output according to their layer type, z_index, anchors, etc.
    pub fn arrange_layers_on_output(&mut self, output: SmithayOutput, dh: &DisplayHandle) {
        debug!(output_name = %output.name(), "Arranging layer shells on output (placeholder)");

        let output_geometry = match self.space.output_geometry(&output) {
            Some(geom) => geom,
            None => {
                warn!("Cannot arrange layers for output {}: not mapped in space.", output.name());
                return;
            }
        };
        // The usable_area for layers themselves is typically the full output_geometry,
        // unless a layer explicitly requests to be constrained by other layers' exclusive zones,
        // which is not standard for zwlr-layer-shell-v1 (they define zones, not react to them).
        let output_usable_area_for_layers = output_geometry;


        // 1. Filter layer_surfaces for the current output.
        // 2. Sort them by Layer enum, then by z_index (if we were to implement custom z_index).
        //    Smithay's LayerSurface doesn't have z_index, so sorting is by Layer enum only.
        //    Layer enum has a natural order.
        self.layer_surfaces.sort_by_key(|ls_data| ls_data.layer_surface.current_state().layer);

        // 3. For each layer surface, calculate its position and size.
        for ls_data in self.layer_surfaces.iter_mut() {
            if ls_data.assigned_output_name.as_ref() == Some(&output.name()) {
                // update_from_pending should have been called in layer_shell_committed
                // ls_data.update_from_pending(); // Ensure latest state from client is considered

                ls_data.position_and_size_for_output(&output_geometry, &output_usable_area_for_layers);

                // 4. Send configure to the client with the new geometry if it changed.
                //    LayerSurface::send_configure() handles this.
                //    It needs the new size. The position is handled by compositor.
                let new_size_for_client = ls_data.geometry.size;
                // LayerSurface configure takes the size it should be.
                // If the client set size to (0,0) (fill), our calculated size is what it gets.
                // If client set a specific size, that's what we used (unless anchored on both axes).
                // Smithay's LayerSurface.send_configure() will use its current_state.size typically.
                // Our LayerSurfaceData.geometry is the *actual* geometry in compositor space.
                // The client is configured with a size, and it positions itself at (0,0) in its buffer.
                // The compositor then positions this buffer.
                // If our calculated size differs from what client expects, we must send configure.
                // Smithay's `LayerSurface::send_configure` should be called after any state change
                // that requires client acknowledgement or buffer resize. This is usually done
                // in `layer_shell_committed` after our `LayerSurfaceData` is updated and geometry calculated.
                // Here, we are just ensuring the geometry field in LayerSurfaceData is correct.
                // The actual send_configure is expected to be handled by the main LayerShellHandler logic.
                debug!(surface = ?ls_data.wl_surface(), new_geom = ?ls_data.geometry, "Calculated new geometry for layer surface");
            }
        }
        self.ask_for_render(); // Layer positions might have changed
    }

    /// Arranges (e.g., tiles or cascades) XDG shell windows on the given output, respecting usable area.
    pub fn arrange_windows_on_output(&mut self, output: SmithayOutput, dh: &DisplayHandle) {
        debug!(output_name = %output.name(), "Arranging XDG windows on output (placeholder)");
        // This would typically involve:
        // 1. Getting the current usable_area for this output from DisplayManager.
        // 2. Getting all XDG windows (from self.space.elements()) that are mapped to this output.
        // 3. Applying the active workspace's layout algorithm (e.g., tiling) to these windows within the usable_area.
        //    - This is where `apply_layout_for_workspace` would be relevant if the active workspace is on this output.
        // 4. For each window, if its size or position changes, send configure events.

        // Example: if the active workspace is on this output, re-apply its layout.
        let active_ws_id_opt;
        let active_ws_monitor_id_opt;
        {
            let wm_guard = self.workspace_manager.lock().unwrap();
            active_ws_id_opt = wm_guard.get_active_workspace_id();
            active_ws_monitor_id_opt = active_ws_id_opt.and_then(|id| {
                wm_guard.get_workspace_by_id(id).and_then(|ws| ws.monitor_id.clone())
            });
        }

        if let (Some(active_ws_id), Some(active_ws_monitor_id)) = (active_ws_id_opt, active_ws_monitor_id_opt) {
            if active_ws_monitor_id == output.name() {
                info!("Output {} hosts the active workspace {:?}, re-applying layout.", output.name(), active_ws_id);
                self.apply_layout_for_workspace(active_ws_id);
            }
        }
        self.ask_for_render(); // Window positions might have changed
    }


    /// Signals that a render pass is needed.
    /// In a real compositor, this would schedule a repaint/redraw, often by signaling the event loop.
    pub fn ask_for_render(&self) {
        // This is a simplified way. A real compositor might use a more complex mechanism,
        // e.g., waking up a rendering thread or scheduling a calloop event source.
        // Using a calloop timer to trigger a redraw after a short delay (e.g., next event loop tick or few ms)
        // can help coalesce multiple render requests.
        if let Err(e) = self.event_loop_handle.insert_source(self.render_timer.clone(), |_, _, state: &mut DesktopState| {
            // This callback will be executed when the timer expires.
            // Here, you would call your actual rendering function.
            // For example: state.render_manager.render_frame();
            // Or if rendering is part of the main loop: state.needs_render = true; state.event_loop_handle.wakeup();
            debug!("Render timer expired, triggering render (placeholder)");
            // In a full setup, this would call the main rendering logic of the compositor.
            // For example, if novade-system/src/compositor/display_loop.rs has a render function:
            state.trigger_render_cycle();
        }) {
            warn!("Failed to arm render timer: {}", e);
        } else {
            self.render_timer.set_duration(std::time::Duration::from_millis(1)); //ほぼ即時
            debug!("Render requested, timer armed.");
        }
    }

    /// Triggers the rendering process for all outputs.
    /// This is typically called from the event loop when a render is scheduled.
    pub fn trigger_render_cycle(&mut self) {
        // This function will iterate over all outputs and call render_output_frame for each.
        // It needs access to the renderer, which is in self.backend_data.
        // We need to handle potential errors from rendering.
        debug!("Triggering render cycle for all outputs.");

        let mut backend_data = self.backend_data.lock().unwrap();
        let renderer = &mut backend_data.renderer;

        // Collect output names first to avoid borrowing issues if new outputs are added/removed during iteration.
        // However, self.outputs() borrows self, and renderer also needs &mut self indirectly.
        // This implies render_output_frame needs to be structured carefully or renderer access passed down.
        // For now, let's clone SmithayOutput to pass to render_output_frame.
        let outputs_to_render: Vec<SmithayOutput> = self.outputs().cloned().collect();
        let dh = self.display.handle(); // DisplayHandle needed for some Smithay functions

        for output in outputs_to_render {
            info!("Rendering frame for output: {}", output.name());
            if let Err(err) = self.render_output_frame(&dh, renderer, &output) {
                error!("Error rendering output {}: {:?}", output.name(), err);
                // Decide on error handling: stop rendering for this output, mark as damaged, etc.
            }
        }
    }

    /// Renders a single frame for a specific output.
    fn render_output_frame(
        &mut self,
        dh: &DisplayHandle,
        renderer: &mut OpenGLRenderer,
        output: &SmithayOutput,
    ) -> Result<(), crate::compositor::render::renderer::RenderError> {
        let output_geometry = self.space.output_geometry(output).ok_or_else(|| {
            warn!("Output {} has no geometry in space, skipping render.", output.name());
            crate::compositor::render::renderer::RenderError::InvalidRenderState(format!(
                "Output {} not mapped or no geometry",
                output.name()
            ))
        })?;
        let output_transform = output.current_transform(); // SmithayOutput provides this
        let output_scale = output.current_scale().fractional_scale(); // SmithayOutput provides this

        renderer.set_output_scale(output_scale)?; // Inform renderer of scale
        renderer.begin_frame(output_transform, output_geometry.size)?;

        let mut all_render_elements: Vec<crate::compositor::render::renderer::RenderElement<'_, OpenGLRendererTexture>> = Vec::new();
        // output_damage is used by renderer.submit() for partial presentation, not directly by render_elements.
        // We need to collect individual surface damage for RenderElement.
        // For overall output damage, Smithay's space.damage_output can be used.
        let mut collected_output_damage : Vec<Rectangle<i32, Physical>> = Vec::new();


        // Clear with a background color for the first pass
        let clear_color = Some(self.workspace_manager.lock().unwrap().get_active_workspace_background_color_or_default());


        // --- Helper to create RenderElement from a WlSurface ---
        // This is a simplified helper. Robust texture caching and damage handling are complex.
        // We'll use a closure to capture necessary variables like `renderer` and `self` (for shm_state).
        // This closure needs to be defined carefully due to borrowing rules if it's a method of DesktopState.
        // For now, inline the logic.

        // --- 1. Background Layers ---
        let mut background_elements = Vec::new();
        for ls_data in self.layer_surfaces.iter().filter(|ls|
            ls.assigned_output_name.as_ref() == Some(&output.name()) &&
            ls.layer_surface.current_state().layer == Layer::Background &&
            ls.layer_surface.is_mapped()
        ) {
            if let Ok(texture) = self.import_surface_texture(renderer, ls_data.wl_surface()) {
                let surface_damage = smithay::wayland::compositor::surface_damage_bounding_box(ls_data.wl_surface())
                    .unwrap_or_else(|| Rectangle::from_loc_and_size(Point::default(), ls_data.geometry.size)); // Full damage if not specified
                background_elements.push(crate::compositor::render::renderer::RenderElement::Surface {
                    texture,
                    geometry: ls_data.geometry,
                    damage: vec![surface_damage],
                    alpha: 1.0, // TODO: Support layer surface alpha
                    transform: output_transform, // Layer surfaces are usually not transformed beyond output
                });
                collected_output_damage.push(ls_data.geometry); // Damage the whole area for now
            }
        }
        // Render background elements first, with clear
        renderer.render_elements(background_elements, clear_color, &collected_output_damage)?;
        let mut subsequent_elements = Vec::new(); // For elements after the initial clear

        // --- 2. Bottom Layers ---
        for ls_data in self.layer_surfaces.iter().filter(|ls|
            ls.assigned_output_name.as_ref() == Some(&output.name()) &&
            ls.layer_surface.current_state().layer == Layer::Bottom &&
            ls.layer_surface.is_mapped()
        ) {
            if let Ok(texture) = self.import_surface_texture(renderer, ls_data.wl_surface()) {
                 let surface_damage = smithay::wayland::compositor::surface_damage_bounding_box(ls_data.wl_surface())
                    .unwrap_or_else(|| Rectangle::from_loc_and_size(Point::default(), ls_data.geometry.size));
                subsequent_elements.push(crate::compositor::render::renderer::RenderElement::Surface {
                    texture,
                    geometry: ls_data.geometry,
                    damage: vec![surface_damage],
                    alpha: 1.0,
                    transform: output_transform,
                });
                collected_output_damage.push(ls_data.geometry);
            }
        }

        // --- 3. XDG Windows (Space elements) + their Popups ---
        // This is the complex part. Smithay's `render_elements_output_with_renderer`
        // or `draw_window_surface_tree_raw` can be used.
        // For now, a simplified loop.
        let space_elements = self.space.elements_for_output(output).cloned().collect::<Vec<_>>();
        for window in space_elements.iter().filter(|w| w.is_mapped()) {
            if let Some(surface) = window.wl_surface() {
                 // Get window geometry from space
                let window_geometry = self.space.element_geometry(window).unwrap_or_default();
                if let Ok(texture) = self.import_surface_texture(renderer, surface) {
                    let surface_damage = smithay::wayland::compositor::surface_damage_bounding_box(surface)
                        .unwrap_or_else(|| Rectangle::from_loc_and_size(Point::default(), window_geometry.size));
                    subsequent_elements.push(crate::compositor::render::renderer::RenderElement::Surface {
                        texture,
                        geometry: window_geometry,
                        damage: vec![surface_damage],
                        alpha: 1.0, // TODO: Support window alpha
                        transform: output_transform, // Window transform might differ
                    });
                    collected_output_damage.push(window_geometry);
                }
                // TODO: Render XDG Popups for this window using self.popups.
                // smithay::desktop::space::draw_popups_surface_tree_raw(...)
            }
        }

        // --- 4. Top Layers ---
        for ls_data in self.layer_surfaces.iter().filter(|ls|
            ls.assigned_output_name.as_ref() == Some(&output.name()) &&
            ls.layer_surface.current_state().layer == Layer::Top &&
            ls.layer_surface.is_mapped()
        ) {
             if let Ok(texture) = self.import_surface_texture(renderer, ls_data.wl_surface()) {
                let surface_damage = smithay::wayland::compositor::surface_damage_bounding_box(ls_data.wl_surface())
                    .unwrap_or_else(|| Rectangle::from_loc_and_size(Point::default(), ls_data.geometry.size));
                subsequent_elements.push(crate::compositor::render::renderer::RenderElement::Surface {
                    texture,
                    geometry: ls_data.geometry,
                    damage: vec![surface_damage],
                    alpha: 1.0,
                    transform: output_transform,
                });
                collected_output_damage.push(ls_data.geometry);
            }
        }

        // --- 5. Overlay Layers ---
        for ls_data in self.layer_surfaces.iter().filter(|ls|
            ls.assigned_output_name.as_ref() == Some(&output.name()) &&
            ls.layer_surface.current_state().layer == Layer::Overlay &&
            ls.layer_surface.is_mapped()
        ) {
            if let Ok(texture) = self.import_surface_texture(renderer, ls_data.wl_surface()) {
                let surface_damage = smithay::wayland::compositor::surface_damage_bounding_box(ls_data.wl_surface())
                    .unwrap_or_else(|| Rectangle::from_loc_and_size(Point::default(), ls_data.geometry.size));
                subsequent_elements.push(crate::compositor::render::renderer::RenderElement::Surface {
                    texture,
                    geometry: ls_data.geometry,
                    damage: vec![surface_damage],
                    alpha: 1.0,
                    transform: output_transform,
                });
                collected_output_damage.push(ls_data.geometry);
            }
        }

        // Render all subsequent elements without clearing
        if !subsequent_elements.is_empty() {
            renderer.render_elements(subsequent_elements, None, &collected_output_damage)?;
        }

        // --- 6. Cursor ---
        // TODO: Render cursor using renderer.render_elements with RenderElement::Cursor
        // let cursor_pos = self.input_state.pointer_location_on_output(output); // Needs method
        // if self.seat.get_pointer().unwrap().has_grab() == false { ... }
        // renderer.render_elements(vec![RenderElement::Cursor { ... }], None, ...)?;


        renderer.finish_frame()?;

        // Send frame events to wl_surface clients, needs surface list from render pass
        // smithay::wayland::compositor::send_frames_surface_tree(&[&surface1, &surface2], self.start_time.elapsed(), Some(dh));
        // This needs to be done carefully after rendering and buffer swaps.
        // For now, this is a placeholder. The actual list of surfaces rendered on this output needs to be collected.
        // smithay::wayland::compositor::send_frames_for_surface_tree(&rendered_wl_surfaces_on_this_output, self.start_time.elapsed(), Some(dh), |_,_| true);
        // This requires collecting all WlSurface that had their buffer drawn.

        Ok(())
    }

    /// Helper function to import a WlSurface's buffer into an OpenGLTexture.
    /// This is a simplified version; robust caching and error handling would be more complex.
    fn import_surface_texture(
        &self, // Needs &self for shm_state access if renderer.import_shm_buffer takes it
        renderer: &mut OpenGLRenderer,
        surface: &WlSurface,
    ) -> Result<Arc<OpenGLRendererTexture>, crate::compositor::render::renderer::RenderError> {
        if let Some(buffer) = smithay::wayland::compositor::get_buffer(surface) {
            // Check buffer type. Smithay's `buffer_type` or direct wl_buffer methods can be used.
            // `wl_buffer_is_shm` is a C function, need to use Smithay's SHM state.
            let is_shm = buffer.data_map().get::<smithay::shm::ShmBuffer>().is_some();

            if is_shm {
                renderer.import_shm_buffer(&buffer, Some(surface), self) // Pass `self` for DesktopState context
            } else if let Some(dmabuf) = smithay::backend::renderer::get_dmabuf(&buffer).ok() {
                renderer.import_dmabuf(&dmabuf, Some(surface))
            } else {
                warn!("Surface {:?} has buffer of unknown type", surface.id());
                Err(crate::compositor::render::renderer::RenderError::InvalidBuffer(
                    "Unknown buffer type for surface".to_string(),
                ))
            }
        } else {
            // debug!("Surface {:?} has no attached buffer, skipping texture import.", surface.id());
            Err(crate::compositor::render::renderer::RenderError::InvalidBuffer(
                format!("Surface {:?} has no attached buffer", surface.id()),
            ))
        }
    }


    // This function was already present, just ensuring it fits with the new ask_for_render
    pub fn apply_layout_for_workspace(&self, workspace_id: novade_domain::workspaces::manager::WorkspaceId) {
        info!("Applying layout for workspace ID: {:?}", workspace_id);
        let workspace_manager_guard = self.workspace_manager.lock().unwrap();
        let target_workspace_opt = workspace_manager_guard.get_workspace_by_id(workspace_id);

        let (target_monitor_id_opt, windows_in_ws, layout_config_opt) =
            if let Some(ws) = target_workspace_opt {
                if matches!(ws.layout, novade_domain::workspaces::manager::WorkspaceLayout::Tiling(_)) {
                    (ws.monitor_id.clone(), ws.windows.clone(), Some(ws.layout.clone()))
                } else {
                    debug!("Workspace {:?} is not tiling, skipping layout application.", ws.id);
                    return;
                }
            } else {
                warn!("apply_layout_for_workspace: Workspace {:?} not found.", workspace_id);
                return;
            };

        let window_manager_interface = workspace_manager_guard.window_manager.clone();
        drop(workspace_manager_guard); // Release lock

        let screen_area_nova_rect: NovaRect = if let Some(monitor_id) = target_monitor_id_opt.as_ref() {
            // Use DisplayManager to get work_area, which should respect exclusive zones
            let dm = self.display_manager.lock().unwrap();
            if let Some(managed_output) = dm.get_output_by_id(monitor_id) {
                managed_output.work_area // This is already a NovaRect
            } else {
                warn!("Failed to get work area for monitor {} from DisplayManager. Falling back.", monitor_id);
                // Fallback logic (e.g. primary or default)
                dm.get_primary_output().map(|mo| mo.work_area).unwrap_or_else(|| {
                    warn!("No primary output found for screen_area, using default 1920x1080.");
                    NovaRect::new(NovaPoint::new(0,0), NovaSize::new(1920,1080))
                })
            }
        } else {
            // Workspace has no monitor_id, try primary output
            let dm = self.display_manager.lock().unwrap();
            dm.get_primary_output().map(|mo| mo.work_area).unwrap_or_else(|| {
                 warn!("No primary output (workspace has no monitor_id), using default 1920x1080 for layout.");
                 NovaRect::new(NovaPoint::new(0,0), NovaSize::new(1920,1080))
            })
        };


        if let Some(novade_domain::workspaces::manager::WorkspaceLayout::Tiling(tiling_options)) = layout_config_opt {
            info!("Applying tiling layout ({:?}) for workspace {:?} on monitor {:?} with usable area {:?}", tiling_options, workspace_id, target_monitor_id_opt, screen_area_nova_rect);
            let algorithm = tiling_options.as_algorithm();
            let new_geometries = algorithm.arrange(&windows_in_ws, screen_area_nova_rect);

            for (domain_window_id, new_geom) in new_geometries {
                if let Some(smithay_window) = self.find_smithay_window_by_domain_id(domain_window_id) {
                    info!("Applying geometry {:?} to window {:?}", new_geom, domain_window_id);
                    // map_element takes global coordinates. new_geom should be in global coordinates.
                    // The tiling algorithm should produce global coordinates based on screen_area_nova_rect.
                    self.space.map_element(smithay_window.clone(), (new_geom.position.x, new_geom.position.y), false);

                    if let Some(toplevel) = smithay_window.toplevel() {
                        match toplevel {
                            WindowSurfaceType::Xdg(xdg_toplevel_surface) => {
                                let xdg_toplevel = xdg_toplevel_surface.xdg_toplevel();
                                xdg_toplevel.send_configure_bounds(smithay::utils::Size::from((new_geom.size.width, new_geom.size.height)));
                                xdg_toplevel.set_maximized(false);
                                xdg_toplevel.set_fullscreen(false);
                                xdg_toplevel.set_resizing(false);
                                xdg_toplevel.send_pending_configure(); // This sends a serial

                                // Update our internal XdgSurfaceData as well
                                if let Some(surface) = smithay_window.wl_surface(){
                                    if let Some(data_mutex) = surface.data::<StdMutex<crate::compositor::shell::xdg::XdgSurfaceData>>() {
                                        if let Ok(mut surface_data) = data_mutex.lock() {
                                            if let crate::compositor::shell::xdg::XdgRoleSpecificData::Toplevel(toplevel_data) = &mut surface_data.role_data {
                                                toplevel_data.current_state.maximized = false;
                                                toplevel_data.current_state.fullscreen = false;
                                                toplevel_data.current_state.resizing = false;
                                                // size is tricky, xdg_toplevel.configure() sends the actual size
                                                // toplevel_data.current_state.size = Some((new_geom.size.width, new_geom.size.height));
                                            }
                                        }
                                    }
                                }
                            }
                           // _ => {} // Could be XWayland toplevel
                        }
                    }
                } else {
                    warn!("Could not find SmithayWindow for domain_id {:?} during layout application.", domain_window_id);
                }
            }
        }
        self.ask_for_render(); // Layout changed
    }
}

impl CompositorHandler for DesktopState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
    fn client_disconnected(&mut self, client: ClientId, reason: DisconnectReason) {
        warn!(?client, ?reason, "Wayland client disconnected.");
        // ANCHOR: When a client disconnects, its windows should be removed from workspaces and layouts reapplied.
    }
}

impl DataDeviceHandler for DesktopState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ShmHandler for DesktopState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl smithay_xdg_shell::XdgShellHandler for DesktopState {
    fn xdg_shell_state(&self) -> &smithay_xdg_shell::XdgShellState {
        &self.xdg_shell_state
    }

    fn new_surface(&mut self, surface: &WlSurface, xdg_surface: &XdgSurface) {
        info!("New XDG surface resource created: wl_surface {:?}, xdg_surface {:?}", surface.id(), xdg_surface.wl_surface().id());
    }

    fn new_toplevel(&mut self, surface: &WlSurface, xdg_surface: &XdgSurface, _toplevel: &smithay_xdg_shell::xdg_toplevel::XdgToplevel) {
        info!("New XDG Toplevel created: wl_surface {:?}, xdg_surface {:?}", surface.id(), xdg_surface.wl_surface().id());
        let window = Window::new_xdg_toplevel(xdg_surface.clone());

        let target_monitor_id: Option<String>;
        let initial_pos: (i32, i32);
        let work_area_rect: Option<NovaRect>;
        let wm_trait_obj = self.workspace_manager.lock().unwrap().window_manager.clone();

        // ANCHOR: block_on in sync handler `new_toplevel`. This is problematic.
        // `new_toplevel` is called from within Wayland dispatch, which is sync.
        // Ideally, these `WindowManager` calls should be refactored to be callable synchronously,
        // or `new_toplevel` and its callers need an async path.
        target_monitor_id = futures::executor::block_on(wm_trait_obj.get_focused_output_id())
            .ok().flatten()
            .or_else(|| {
                futures::executor::block_on(wm_trait_obj.get_primary_output_id()).ok().flatten()
            });

        if let Some(ref mon_id) = target_monitor_id {
            work_area_rect = futures::executor::block_on(wm_trait_obj.get_output_work_area(mon_id)).ok();
        } else {
            work_area_rect = None;
        }

        if let Some(work_area) = work_area_rect {
            let window_default_width = 640;
            let window_default_height = 480;
            initial_pos = (
                work_area.position.x + (work_area.size.width / 2) - (window_default_width / 2),
                work_area.position.y + (work_area.size.height / 2) - (window_default_height / 2)
            );
        } else {
            initial_pos = (0,0);
            warn!("No target monitor work_area found for new window, placing at (0,0).");
        }

        self.space.map_element(window.clone(), initial_pos, true);

        if let Some(data_mutex) = surface.data::<StdMutex<crate::compositor::shell::xdg::XdgSurfaceData>>() { // Changed Mutex to StdMutex
            let mut data = data_mutex.lock().unwrap();
            data.window = window.clone();
            let domain_id = novade_domain::workspaces::core::WindowId::new();
            data.domain_id = Some(domain_id);
            info!("Updated XdgSurfaceData with new toplevel Window and Domain ID {:?} for wl_surface {:?}", domain_id, surface.id());

            let current_domain_id = domain_id;
            drop(data);

            let mut workspace_manager_guard = self.workspace_manager.lock().unwrap();
            let target_workspace_id = if let Some(mon_id) = target_monitor_id.as_ref() {
                workspace_manager_guard.get_all_workspaces().iter().find(|ws| ws.monitor_id.as_ref() == Some(mon_id)).map(|ws| ws.id)
                    .unwrap_or_else(|| {
                        let new_ws_name = format!("Workspace on {}", mon_id);
                        let new_id = workspace_manager_guard.create_workspace(new_ws_name, Some(mon_id.clone()));
                        info!("Created new workspace {:?} for monitor {}", new_id, mon_id);
                        new_id
                    })
            } else {
                workspace_manager_guard.get_active_workspace_id()
                    .unwrap_or_else(|| workspace_manager_guard.get_all_workspaces().first().map(|ws| ws.id).expect("No workspaces available; should have a default"))
            };

            if let Err(e) = workspace_manager_guard.add_window_to_workspace(current_domain_id, target_workspace_id) {
                warn!("Failed to add window {:?} to workspace {:?}: {:?}", current_domain_id, target_workspace_id, e);
            } else {
                info!("Window {:?} added to workspace {:?} on monitor {:?}", current_domain_id, target_workspace_id, target_monitor_id);
            }

            let active_ws_id_check = workspace_manager_guard.get_active_workspace_id();
            let mut apply_layout_now = false;
            if active_ws_id_check == Some(target_workspace_id) {
                 if let Some(active_ws) = workspace_manager_guard.get_workspace_by_id(target_workspace_id) {
                    if matches!(active_ws.layout, novade_domain::workspaces::manager::WorkspaceLayout::Tiling(_)) {
                        apply_layout_now = true;
                    }
                 }
            }
            drop(workspace_manager_guard);

            if apply_layout_now {
                self.apply_layout_for_workspace(target_workspace_id);
            }
        } else {
            warn!("Could not find XdgSurfaceData to store new toplevel Window or Domain ID for wl_surface {:?}", surface.id());
        }
        debug!("New toplevel window {:?} added to space.", window.wl_surface().id());
    }

    fn new_popup(&mut self, surface: &WlSurface, xdg_surface: &XdgSurface, _popup: &smithay_xdg_shell::xdg_popup::XdgPopup) {
        info!("New XDG Popup created: wl_surface {:?}, xdg_surface {:?}", surface.id(), xdg_surface.wl_surface().id());
        let parent_wl_surface = crate::compositor::shell::xdg::handlers::with_surface_data_fallible(surface, |s_data| {
            s_data.parent.clone()
        }).flatten();

        if let Some(parent_surface) = parent_wl_surface {
            if self.space.window_for_surface(&parent_surface).is_some() {
                self.popups.track_popup(xdg_surface.clone()).expect("Failed to track popup");
                info!("Popup {:?} associated with parent and tracked by PopupManager.", xdg_surface.wl_surface().id());
            } else {
                warn!("Parent surface for popup {:?} has no associated Window in the space.", xdg_surface.wl_surface().id());
            }
        } else {
            warn!("Popup {:?} has no parent WlSurface in its XdgSurfaceData.", xdg_surface.wl_surface().id());
        }
    }

    fn ack_configure(&mut self, surface: WlSurface, _xdg_surface: XdgSurface, serial: Serial) {
        info!("XDG Surface {:?} acked configure with serial {}", surface.id(), serial);
        if let Some(window) = self.space.window_for_surface(&surface) {
            if !window.is_mapped() && window.surface_has_buffer() {
                info!("Mapping window for surface {:?} after initial ack_configure.", surface.id());
                window.map();
                 // ANCHOR: Potential re-application of layout after a window maps and might have new size.
                 // self.apply_active_workspace_layout();
            }
        }
        if self.popups.is_popup(&surface) {
            // ANCHOR: Need to get XdgSurface from WlSurface to call handle_ack_configure for popups.
        }
    }

    fn grab(&mut self, surface: XdgSurface, seat: wl_seat::WlSeat, _serial: Serial) {
        info!("Popup {:?} requested grab for seat {:?}", surface.wl_surface().id(), seat.id());
        if self.popups.is_popup(surface.wl_surface()) {
            debug!("Popup grab request for {:?} noted. PopupManager might handle it.", surface.wl_surface().id());
        }
    }

    fn reposition_request(&mut self, surface: XdgSurface, positioner: XdgPositioner, token: u32) {
        info!("Popup {:?} requested reposition with token {}", surface.wl_surface().id(), token);
        if self.popups.is_popup(surface.wl_surface()) {
            self.popups.reposition_popup(surface, positioner, token)
                .expect("Failed to initiate popup repositioning");
        }
    }

    fn xdg_shell_icon(&self) -> Option<WlSurface> { None }
    fn move_request(&mut self, surface: &XdgSurface, _seat: &wl_seat::WlSeat, _serial: Serial) {
        info!("Toplevel {:?} requested move", surface.wl_surface().id());
        // ANCHOR: Trigger interactive move via NovaWindowManager or similar.
        // Example: self.window_manager.lock().unwrap().start_interactive_move(domain_id_of_surface);
    }
    fn resize_request(&mut self, surface: &XdgSurface, _seat: &wl_seat::WlSeat, _serial: Serial, edges: xdg_toplevel::ResizeEdge) {
        info!("Toplevel {:?} requested resize with edges {:?}", surface.wl_surface().id(), edges);
        // ANCHOR: Trigger interactive resize.
    }
    fn show_window_menu_request(&mut self, surface: &XdgSurface, _seat: &wl_seat::WlSeat, _serial: Serial, location: Point<i32, Logical>) {
        info!("Toplevel {:?} requested show_window_menu at {:?}", surface.wl_surface().id(), location);
    }
    fn set_parent_request(&mut self, surface: &XdgSurface, parent: Option<&XdgSurface>) {
        info!("Toplevel {:?} requested set_parent to {:?}", surface.wl_surface().id(), parent.map(|p| p.wl_surface().id()));
    }
    fn set_title_request(&mut self, surface: &XdgSurface, title: String) {
        info!("Toplevel {:?} requested set_title to '{}'", surface.wl_surface().id(), title);
    }
    fn set_app_id_request(&mut self, surface: &XdgSurface, app_id: String) {
        info!("Toplevel {:?} requested set_app_id to '{}'", surface.wl_surface().id(), app_id);
    }
    fn set_fullscreen_request(&mut self, surface: &XdgSurface, fullscreen: bool, output: Option<&wl_output::WlOutput>) {
        info!("Toplevel {:?} requested fullscreen: {} on output {:?}", surface.wl_surface().id(), fullscreen, output.map(|o| o.id()));
        if let Some(window) = self.space.window_for_surface(surface.wl_surface()) {
            window.set_fullscreen(fullscreen);
            // ANCHOR: This should also update our XdgToplevelData and trigger apply_active_workspace_layout
            // if the workspace is tiling, as fullscreen usually means taking over the screen.
            // Or, if not tiling, adjust other windows.
            // For now, just setting Smithay state. The configure will be sent by xdg_toplevel handler.
        }
    }
    fn set_maximized_request(&mut self, surface: &XdgSurface, maximized: bool) {
        info!("Toplevel {:?} requested maximized: {}", surface.wl_surface().id(), maximized);
        if let Some(window) = self.space.window_for_surface(surface.wl_surface()) {
            window.set_maximized(maximized);
            // ANCHOR: Similar to fullscreen, update domain state and re-apply layout.
        }
    }
    fn set_minimized_request(&mut self, surface: &XdgSurface) {
        info!("Toplevel {:?} requested minimize", surface.wl_surface().id());
        // ANCHOR: Minimization logic: unmap window, update domain state, re-apply layout.
    }
    fn map_foreign_toplevel(&mut self, surface: &XdgSurface, _toplevel_handle: Box<dyn std::any::Any + Send + Sync>) {
        warn!("map_foreign_toplevel called for {:?}, but not implemented.", surface.wl_surface().id());
    }
}

impl OutputHandler for DesktopState {
    fn output_state(&mut self) -> &mut OutputStateSmithay {
        &mut self.output_state
    }

    fn new_output(&mut self, output: SmithayOutput) {
        info!("Smithay OutputHandler: New output added: {}", output.name());
        let mut total_width = 0;
        for existing_output_geometry in self.space.outputs().filter_map(|o| self.space.output_geometry(o)) {
            total_width = total_width.max(existing_output_geometry.loc.x + existing_output_geometry.size.w);
        }
        let new_output_pos = (total_width, 0).into();
        let mode = output.preferred_mode().or_else(|| output.modes().first()).unwrap_or_else(|| {
            warn!("Output {} has no modes, using default 800x600@60Hz for mapping.", output.name());
            smithay::output::Mode { size: (800,600).into(), refresh: 60000 }
        });
        self.space.map_output(&output, new_output_pos, mode);
        info!("Mapped new output {} to space at {:?} with mode {:?}", output.name(), new_output_pos, mode);

        let (managed_output, _is_first) = Self::convert_smithay_output_to_managed(
            &output,
            &self.space,
            self.display_manager.lock().unwrap().all_outputs().is_empty()
        );

        info!("Adding new output {:?} to DisplayManager.", managed_output.id);
        self.display_manager.lock().unwrap().add_output(managed_output);
        self.apply_active_workspace_layout();
    }

    fn output_destroyed(&mut self, output: SmithayOutput) {
        info!("Smithay OutputHandler: Output removed: {}", output.name());
        let output_name = output.name(); // Get name before unmap might invalidate it
        self.space.unmap_output(&output);
        self.display_manager.lock().unwrap().remove_output(&output_name);

        // ANCHOR: When an output is destroyed, workspaces on it need to be reassigned
        // to another output, and windows moved.
        // For now, just re-applying layout on potentially new primary might do something.
        self.apply_active_workspace_layout();
    }
}

impl InputHandler for DesktopState {
    fn input_state(&mut self) -> &mut InputStateSmithay {
        &mut self.input_state
    }
    fn seat_name(&self) -> String {
        self.seat.name().to_string()
    }
    fn new_seat(&mut self, seat: Seat<Self>) {
        info!("Neuer Seat hinzugefügt: {}", seat.name());
    }
    fn seat_removed(&mut self, seat: Seat<Self>) {
        info!("Seat entfernt: {}", seat.name());
    }
}

impl PopupManagerHandler for DesktopState {
    fn grab_xdg_popup(&mut self, popup: &Popup) -> Option<GrabStartData> {
        debug!("XDG Popup Grab angefordert: {:?}", popup.wl_surface());
        None
    }
}

// Smithay delegate macros
smithay::delegate_compositor!(DesktopState);
smithay::delegate_data_device!(DesktopState);
smithay::delegate_shm!(DesktopState);
smithay::delegate_xdg_shell!(DesktopState);
smithay::delegate_output!(DesktopState);
smithay::delegate_input!(DesktopState);
smithay::delegate_layer_shell!(DesktopState); // Added delegate for layer_shell

// Type alias for convenience
type OpenGLRendererTexture = crate::compositor::render::gl::OpenGLTexture;


// ... (rest of the file: NovadeCompositorState, etc. - Copied from previous read_files output)
// Ensure these smithay imports are appropriate for DesktopState, not NovadeCompositorState context
use smithay::{
    backend::egl::Egl, // EGL might be backend-specific, less likely in DesktopState directly
    wayland::{
        seat::{/*SeatState, Seat, CursorImageStatus*/}, // Already imported or part of DesktopState fields
        dmabuf::DmabufState, // If DesktopState manages DMABUF directly
    },
    backend::{
        drm::{DrmNode as DrmNodeSmithay}, // DRM node likely backend-specific
        session::Session, // Session likely backend-specific
    },
};
use std::{cell::RefCell, collections::HashMap, time::SystemTime, /*sync::{Arc, Mutex}*/}; // Arc, Mutex already imported
use smithay::wayland::compositor as smithay_compositor;
// use tracing::{debug_span, error, info_span, trace, warn}; // Already imported

use crate::compositor::render::gl::{init_gl_renderer, GlInitError};

pub trait RenderableTextureOld: std::fmt::Debug + Send + Sync {
}

pub trait FrameRendererOld: Send + Sync {
    fn create_texture_from_shm(&mut self, buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer) -> Result<Box<dyn RenderableTextureOld>, RendererErrorOld>;
    fn create_texture_from_dmabuf(&mut self, dmabuf: &smithay::backend::allocator::dmabuf::Dmabuf) -> Result<Box<dyn RenderableTextureOld>, RendererErrorOld>;
    fn render_frame(
        &mut self,
        output_size: smithay::utils::Size<i32, smithay::utils::Physical>,
        output_scale: f64,
        elements: &[RenderElementOld],
        clear_color: [f32; 4],
    ) -> Result<Vec<Rectangle<i32, smithay::utils::Physical>>, String>;
}

#[derive(Debug, thiserror::Error)]
pub enum RendererErrorOld {
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

use uuid::Uuid;

pub trait RenderableTextureUuid: std::fmt::Debug + Send + Sync + std::any::Any {
    fn id(&self) -> Uuid;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn as_any(&self) -> &dyn std::any::Any;
}


#[derive(Debug)]
pub enum RenderElementOld<'a> {
    Surface {
        surface: &'a WlSurface,
        texture: Arc<dyn RenderableTextureUuid>,
        location: Point<i32, Logical>,
        size: smithay::utils::Size<i32, Logical>,
        transform: smithay::utils::Transform,
        damage: Vec<Rectangle<i32, Logical>>,
    },
    Cursor {
        texture: Arc<dyn RenderableTextureUuid>,
        location: Point<i32, Logical>,
        hotspot: (i32, i32),
    },
    SolidColor {
        color: [f32; 4],
        geometry: Rectangle<i32, Logical>,
    }
}

#[derive(Default, Debug)]
pub struct SurfaceDataExtOld {
    pub texture_handle: Option<Arc<dyn RenderableTextureUuid>>,
    pub damage_buffer: Vec<Rectangle<i32, smithay::utils::Physical>>,
}

pub struct NovadeCompositorState {
    pub display_handle: Display<Self>,
    pub loop_handle: LoopHandle<'static, Self>,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<Self>,
    pub space: Space<Window>,
    pub seat: Seat<Self>,
    pub seat_name: String,
    pub frame_renderer: Option<Arc<Mutex<dyn FrameRendererOld>>>,
    pub session: smithay::backend::session::direct::DirectSession,
    pub primary_drm_node: DrmNodeSmithay,
    pub gl_renderer: Gles2Renderer,
    pub dmabuf_state: DmabufState,
    pub cursor_image_status: Option<CursorImageStatus>,
    pub cursor_hotspot: (i32, i32),
    pub pointer_location: Point<f64, Logical>,
}

impl NovadeCompositorState {
    pub fn new(
        display_handle: Display<Self>,
        loop_handle: LoopHandle<'static, Self>,
        session: smithay::backend::session::direct::DirectSession,
        primary_drm_node: DrmNodeSmithay,
        dmabuf_state: DmabufState,
    ) -> Result<Self, GlInitError> {
        info!("Beginne EGL- und Gles2Renderer-Initialisierung für NovadeCompositorState...");
        let egl = Egl::new().map_err(|egl_err| {
            error!("EGL-Initialisierung fehlgeschlagen: {}", egl_err);
            GlInitError::from(egl_err)
        })?;
        info!("EGL erfolgreich initialisiert.");
        let gl_renderer = init_gl_renderer(egl).map_err(|render_err| {
            error!("Gles2Renderer-Initialisierung fehlgeschlagen: {}", render_err);
            render_err
        })?;
        info!("Gles2Renderer erfolgreich initialisiert und in NovadeCompositorState integriert.");

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_formats = vec![
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Argb8888,
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Xrgb8888,
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Abgr8888,
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Xbgr8888,
        ];
        let shm_state = ShmState::new::<Self>(&display_handle, shm_formats);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let seat_name = "novade_seat_0".to_string();
        let seat = seat_state.new_wl_seat(&display_handle, seat_name.clone());
        let space = Space::new(tracing::info_span!("space"));

        Ok(Self {
            display_handle, loop_handle, compositor_state, xdg_shell_state, shm_state,
            output_manager_state, seat_state, space, seat, seat_name,
            frame_renderer: None, session, primary_drm_node, gl_renderer,
            dmabuf_state, cursor_image_status: None, cursor_hotspot: (0, 0),
            pointer_location: Point::from((0.0, 0.0)),
        })
    }
}

impl NovadeCompositorState {
    pub fn render_frame(
        &mut self,
        background_color: [f32; 4],
        output_size: smithay::utils::Size<i32, smithay::utils::Physical>,
        output_scale: f64,
    ) -> Result<(), String> {
        let frame_renderer_arc = match &self.frame_renderer {
            Some(renderer) => renderer.clone(),
            None => { return Ok(()); }
        };
        let mut frame_renderer = frame_renderer_arc.lock().unwrap();
        let mut render_elements = Vec::new();
        let elements_to_render = self.space.elements().cloned().collect::<Vec<_>>();

        for element_window in &elements_to_render {
            if let Some(wl_surface) = element_window.wl_surface() {
                if !wl_surface.is_alive() { trace!("Surface not alive"); continue; }
                if let Some(sde_ref) = wl_surface.get_data::<RefCell<SurfaceDataExtOld>>() {
                    let sde = sde_ref.borrow();
                    if let Some(texture_handle) = &sde.texture_handle {
                        if let Some(geo) = self.space.element_geometry(element_window) {
                            let damage = vec![Rectangle::from_loc_and_size((0,0), geo.size)];
                            render_elements.push(RenderElementOld::Surface {
                                surface: wl_surface,
                                texture: texture_handle.clone(),
                                location: geo.loc,
                                size: geo.size,
                                transform: self.compositor_state.get_surface_transformation(wl_surface),
                                damage,
                            });
                        } else { trace!("No geometry for element"); }
                    } else { trace!("No texture_handle for surface"); }
                } else { warn!("No SurfaceDataExtOld for surface"); }
            }
        }

        let active_cursor_texture_param: Option<Arc<dyn RenderableTextureUuid>> = None;

        if self.cursor_image_status.as_ref().map_or(false, |s| !matches!(s, CursorImageStatus::Hidden)) {
            if let Some(texture_arc) = active_cursor_texture_param {
                 render_elements.push(RenderElementOld::Cursor {
                    texture: texture_arc.clone(),
                    location: self.pointer_location.to_i32_round(),
                    hotspot: self.cursor_hotspot,
                });
            } else {
                tracing::warn!("Cursor is visible by status, but no active_cursor_texture was provided to render_frame.");
            }
        }

        let render_span = info_span!("renderer_render_frame", output_size = ?output_size, num_elements = render_elements.len());
        let _render_guard = render_span.enter();

        match frame_renderer.render_frame(output_size, output_scale, &render_elements, background_color) {
            Ok(_rendered_damage) => {
                let time_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as u32;
                for element_window in &elements_to_render {
                    if let Some(wl_surface) = element_window.wl_surface() {
                        if wl_surface.is_alive() {
                            if let Some(data_refcell) = wl_surface.data_map().get::<RefCell<smithay_compositor::SurfaceData>>() {
                                let mut surface_data_inner = data_refcell.borrow_mut();
                                if !surface_data_inner.frame_callbacks.is_empty() {
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
