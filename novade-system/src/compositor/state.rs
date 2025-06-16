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
        calloop::{EventLoop, LoopHandle},
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

        // ANCHOR: Defer initial layout application to after DesktopState is fully created.
        // The main setup function that calls DesktopState::new should call something like:
        // if let Some(initial_ws_id) = state_arc.lock().unwrap().workspace_manager.lock().unwrap().get_active_workspace_id() {
        //    state_arc.lock().unwrap().apply_layout_for_workspace(initial_ws_id);
        // }
        // after this `new` function returns and the state is fully wrapped in its Arc<StdMutex<DesktopState>>.

        let opengl_renderer = OpenGLRenderer::new(display_handle.clone()).expect("Failed to create OpenGLRenderer");
        info!("OpenGLRenderer initialized successfully for DesktopState.");
        let backend_data = Arc::new(StdMutex::new(BackendData { renderer: opengl_renderer }));

        info!("NovaDE DesktopState initialized successfully");
        Self {
            display, event_loop_handle: event_loop.handle(), compositor_state, xdg_shell_state,
            xdg_decoration_state, shm_state, data_device_state, output_state, input_state, seat,
            popups, workspace_manager, display_manager: display_manager_arc, backend_data, space,
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

        // Update Smithay Space representation and inform client
        // ANCHOR: This part needs careful handling of window's current output vs target output
        // and ensuring correct wl_surface.enter/leave events are emitted by Smithay.
        // This might involve `smithay_window.unmap_output()` and `smithay_window.map_output(new_smithay_output)`.
        // For now, we'll map it to the new space at a default position on the new monitor.

        let target_output_work_area = {
            let display_manager_guard = self.display_manager.lock().map_err(|e| format!("Failed to lock DisplayManager: {:?}", e))?;
            display_manager_guard.get_output_by_id(&target_monitor_id)
                .map(|mo| mo.work_area) // This is physical
                .ok_or_else(|| format!("Target monitor {} not found in DisplayManager", target_monitor_id))?
        };

        // Calculate a new position on the target monitor (e.g., center or a default spot)
        // This position is in the global compositor space.
        let new_pos_logical = (target_output_work_area.position.x, target_output_work_area.position.y); // Top-left for now

        // map_element will handle making it appear on the correct output based on global coordinates.
        // Smithay should then send wl_surface.enter for the new output and wl_surface.leave for the old.
        self.space.map_element(smithay_window.clone(), new_pos_logical, false); // Don't activate automatically
        info!("Mapped SmithayWindow {:?} to new position {:?} on target monitor {}", window_id, new_pos_logical, target_monitor_id);

        // Re-apply layout for the source workspace (if it was different and active)
        if let Some(sws_id) = source_workspace_id_opt {
            if sws_id != target_workspace_id {
                // Check if source workspace is still active (it shouldn't be if target is on different monitor and becomes active)
                // Or, more simply, apply layout if it's tiling.
                // This needs a way to apply layout for a *specific*, potentially non-active, workspace.
                // ANCHOR: Add apply_layout_for_workspace(ws_id) if needed. (This is now available)
                // Re-apply layout for the source workspace.
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

        Ok(())
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

// ... (rest of the file: NovadeCompositorState, etc. - Copied from previous read_files output)
use smithay::{
    backend::egl::Egl,
    // backend::renderer::gles2::Gles2Renderer, // Already imported
    // desktop::{Space, Window}, // Already imported
    // reexports::calloop::LoopHandle, // Already imported
    // reexports::wayland_server::{Display, protocol::wl_surface::WlSurface}, // Already imported
    wayland::{
        // compositor::CompositorState, // Already imported
        // output::OutputManagerState, // Already imported
        // shell::xdg::XdgShellState, // Already imported
        // shm::ShmState, // Already imported
        seat::{SeatState, /*Seat, CursorImageStatus*/}, // Seat, CursorImageStatus already imported
        dmabuf::DmabufState,
    },
    backend::{
        // drm::{DrmDevice, DrmDisplay, DrmNode, DrmSurface}, // TODO: Integrate with generic FrameRenderer for DRM
        drm::{DrmNode as DrmNodeSmithay}, // Renamed to avoid conflict if another DrmNode is in scope
        // renderer::gles::{GlesRenderer, GlesTexture}, // GLES specific, removed
        session::Session,
    },
    // reexports::drm::control::crtc, // Related to DrmDevice/Display
    // utils::{Rectangle, Physical, Point, Logical, Transform}, // Already imported
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
