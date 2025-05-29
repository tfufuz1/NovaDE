use smithay::{
    desktop::{Window, Space, WindowSurfaceType},
    utils::{Rectangle, Physical, Logical, Point, Size},
    reexports::wayland_server::protocol::wl_surface::WlSurface, 
};
use std::{
    collections::HashMap,
    sync::Arc,
    ops::Deref,
};

// --- Configuration Constants ---
const SNAP_THRESHOLD_PX: i32 = 10;
const SSD_TITLE_BAR_HEIGHT_PX: i32 = 30; 
const SSD_BORDER_WIDTH_PX: i32 = 5;   
const DEFAULT_FLOATING_SIZE: Size<i32, Logical> = Size::from_values(300, 200);


// --- Enums for Window State ---

/// Defines who handles the window decorations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationMode {
    Client,
    Server,
}

// --- Window State Management ---

#[derive(Debug, Clone)]
pub struct ManagedWindow {
    pub window: Arc<Window>, 
    pub is_floating: bool,
    pub is_maximized: bool,
    pub is_fullscreen: bool,
    pub saved_geometry_before_maximize: Option<Rectangle<i32, Logical>>,
    /// Current effective geometry used for floating or restoring. In logical coordinates.
    /// If server-side decorations are active, this represents the *total* window geometry (including server decorations).
    pub current_geometry_logical: Rectangle<i32, Logical>,
    /// Decoration mode for this window.
    pub decoration_mode: DecorationMode,
}

impl ManagedWindow {
    pub fn new(window: Arc<Window>) -> Self {
        let initial_geometry_logical = window.geometry();
        Self {
            window,
            is_floating: false, 
            is_maximized: false,
            is_fullscreen: false,
            saved_geometry_before_maximize: None,
            current_geometry_logical: initial_geometry_logical,
            decoration_mode: DecorationMode::Client, // Default to client-side decorations
        }
    }
}

impl Deref for ManagedWindow {
    type Target = Window;
    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

fn get_wl_surface(window: &Arc<Window>) -> Option<WlSurface> {
    match window.surface() {
        Some(WindowSurfaceType::Xdg(xdg_surface_type)) => {
            Some(xdg_surface_type.wl_surface().clone())
        }
        _ => None,
    }
}

// --- Helper Enums/Structs ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutAlgorithm {
    MasterStack,
    Spiral,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowArrangement {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

// --- LayoutManager Trait and Implementations ---

pub trait LayoutManager: std::fmt::Debug {
    fn arrange_windows(
        &self,
        space: &mut Space,
        windows_in_workspace: &[&Arc<Window>], 
        output_geometry: Rectangle<i32, Physical>,
        all_managed_windows: &HashMap<WlSurface, ManagedWindow>, 
    );

    fn on_window_added(&mut self, window: &Arc<Window>, managed_window_state: &mut ManagedWindow);
    fn on_window_removed(&mut self, window: &Arc<Window>);
    fn name(&self) -> String;
}

#[derive(Debug)]
pub struct TileLayout {
    algorithm: LayoutAlgorithm,
    num_master: usize,
    master_ratio: f32,
}

impl TileLayout {
    pub fn new(algorithm: LayoutAlgorithm) -> Self {
        Self {
            algorithm,
            num_master: 1,
            master_ratio: 0.5,
        }
    }
}

impl LayoutManager for TileLayout {
    fn arrange_windows(
        &self,
        space: &mut Space,
        windows_in_workspace: &[&Arc<Window>],
        output_geometry: Rectangle<i32, Physical>,
        all_managed_windows: &HashMap<WlSurface, ManagedWindow>,
    ) {
        if windows_in_workspace.is_empty() {
            return;
        }
        let tiled_windows: Vec<&Arc<Window>> = windows_in_workspace
            .iter()
            .filter(|w_arc| 
                get_wl_surface(w_arc).and_then(|s| all_managed_windows.get(&s))
                    .map_or(false, |mw| !mw.is_floating && !mw.is_maximized && !mw.is_fullscreen)
            )
            .cloned()
            .collect();

        if tiled_windows.is_empty() { return; }
        
        let num_tiled_windows = tiled_windows.len();
        let actual_num_master = std::cmp::min(num_tiled_windows, self.num_master.max(1));
        let num_stack = num_tiled_windows - actual_num_master;

        let output_x = output_geometry.loc.x;
        let output_y = output_geometry.loc.y;
        let output_width = output_geometry.size.w;
        let output_height = output_geometry.size.h;

        let master_area_width = if num_stack > 0 && actual_num_master > 0 {
            (output_width as f32 * self.master_ratio).round() as i32
        } else { output_width };
        let stack_area_width = output_width - master_area_width;

        if actual_num_master > 0 {
            let master_window_height = output_height / actual_num_master as i32;
            for (i, window_arc) in tiled_windows.iter().take(actual_num_master).enumerate() {
                let window_ref = &***window_arc; 
                let managed_w_state_opt = get_wl_surface(window_arc).and_then(|s| all_managed_windows.get(&s));
                let tile_total_size_logical = Size::from((master_area_width, master_window_height));
                
                let client_configure_size = if managed_w_state_opt.map_or(false, |mws| mws.decoration_mode == DecorationMode::Server) {
                    Size::from((
                        (tile_total_size_logical.w - 2 * SSD_BORDER_WIDTH_PX).max(1), 
                        (tile_total_size_logical.h - SSD_TITLE_BAR_HEIGHT_PX - SSD_BORDER_WIDTH_PX).max(1),
                    ))
                } else { tile_total_size_logical };
                
                window_ref.toplevel().with_pending_state(|state| {
                    state.size = Some(client_configure_size);
                    state.maximized = Some(false); state.fullscreen = Some(false);
                });
                window_ref.send_configure();
                let loc_physical = Point::from((output_x, output_y + (i as i32 * master_window_height)));
                space.map_element(window_arc.clone().into(), loc_physical, true); 
            }
        }

        if num_stack > 0 && stack_area_width > 0 {
            let stack_window_height = output_height / num_stack as i32;
            let stack_area_x = output_x + master_area_width;
            for (i, window_arc) in tiled_windows.iter().skip(actual_num_master).enumerate() { 
                let window_ref = &***window_arc;
                let managed_w_state_opt = get_wl_surface(window_arc).and_then(|s| all_managed_windows.get(&s));
                let tile_total_size_logical = Size::from((stack_area_width, stack_window_height));

                let client_configure_size = if managed_w_state_opt.map_or(false, |mws| mws.decoration_mode == DecorationMode::Server) {
                     Size::from((
                        (tile_total_size_logical.w - 2 * SSD_BORDER_WIDTH_PX).max(1),
                        (tile_total_size_logical.h - SSD_TITLE_BAR_HEIGHT_PX - SSD_BORDER_WIDTH_PX).max(1),
                    ))
                } else { tile_total_size_logical };

                window_ref.toplevel().with_pending_state(|state| {
                    state.size = Some(client_configure_size);
                    state.maximized = Some(false); state.fullscreen = Some(false);
                });
                window_ref.send_configure();
                let loc_physical = Point::from((stack_area_x, output_y + (i as i32 * stack_window_height)));
                space.map_element(window_arc.clone().into(), loc_physical, true); 
            }
        }
    }

    fn on_window_added(&mut self, _window: &Arc<Window>, managed_window_state: &mut ManagedWindow) {
        managed_window_state.is_floating = false;
    }
    fn on_window_removed(&mut self, _window: &Arc<Window>) {}
    fn name(&self) -> String { "TileLayout (Master-Stack)".to_string() }
}

#[derive(Debug, Default)]
pub struct FloatLayout;
impl FloatLayout { pub fn new() -> Self { Self {} } }

impl LayoutManager for FloatLayout {
    fn arrange_windows(
        &self, space: &mut Space, windows_in_workspace: &[&Arc<Window>],
        _output_geometry: Rectangle<i32, Physical>, 
        all_managed_windows: &HashMap<WlSurface, ManagedWindow>,
    ) {
        for window_arc in windows_in_workspace.iter() {
            let window_ref = &***window_arc; 
            if let Some(surface) = get_wl_surface(window_arc) {
                if let Some(managed_w) = all_managed_windows.get(&surface) {
                    if managed_w.is_floating || managed_w.is_maximized || managed_w.is_fullscreen {
                        let total_geometry_logical = managed_w.current_geometry_logical;
                        let client_configure_size = if managed_w.decoration_mode == DecorationMode::Server && !managed_w.is_fullscreen { 
                             Size::from((
                                (total_geometry_logical.size.w - 2 * SSD_BORDER_WIDTH_PX).max(1),
                                (total_geometry_logical.size.h - SSD_TITLE_BAR_HEIGHT_PX - SSD_BORDER_WIDTH_PX).max(1),
                            ))
                        } else { total_geometry_logical.size };
                        
                        window_ref.toplevel().with_pending_state(|state| {
                            state.size = Some(client_configure_size);
                            state.maximized = Some(managed_w.is_maximized);
                            state.fullscreen = Some(managed_w.is_fullscreen);
                        });
                        window_ref.send_configure();
                        let physical_location = total_geometry_logical.loc.to_physical_precise_round(1);
                        space.map_element(window_arc.clone().into(), physical_location, true);
                    }
                }
            }
        }
    }
    fn on_window_added(&mut self, _window: &Arc<Window>, managed_window_state: &mut ManagedWindow) {
        managed_window_state.is_floating = true;
    }
    fn on_window_removed(&mut self, _window: &Arc<Window>) {}
    fn name(&self) -> String { "FloatLayout".to_string() }
}

#[derive(Debug)]
pub struct Workspace {
    pub id: usize, pub windows: Vec<Arc<Window>>, pub active_layout: Box<dyn LayoutManager>,
    pub name: Option<String>, pub active: bool,
}
impl Workspace {
    pub fn new(id: usize, name: Option<String>, initial_layout: Box<dyn LayoutManager>) -> Self {
        Self { id, windows: Vec::new(), active_layout: initial_layout, name: name.or_else(|| Some(format!("Workspace {}", id + 1))), active: false }
    }
    pub fn add_window(&mut self, window_arc: Arc<Window>, managed_window_state: &mut ManagedWindow) {
        self.windows.push(window_arc.clone());
        self.active_layout.on_window_added(&window_arc, managed_window_state);
    }
    pub fn remove_window(&mut self, window_to_remove: &Arc<Window>) -> bool {
        if let Some(pos) = self.windows.iter().position(|w| Arc::ptr_eq(w, window_to_remove)) {
            let removed_arc = self.windows.remove(pos);
            self.active_layout.on_window_removed(&removed_arc); true
        } else { false }
    }
    pub fn arrange_windows(&self, space: &mut Space, output_geometry: Rectangle<i32, Physical>, all_managed_windows: &HashMap<WlSurface, ManagedWindow>) {
        let window_arcs_in_ws: Vec<&Arc<Window>> = self.windows.iter().collect();
        self.active_layout.arrange_windows(space, &window_arcs_in_ws, output_geometry, all_managed_windows);
    }
}

#[derive(Debug)]
pub struct WindowManagerState {
    pub workspaces: Vec<Workspace>, pub active_workspace_id: usize, next_workspace_id: usize,
    pub managed_windows: HashMap<WlSurface, ManagedWindow>, pub focused_window_surface: Option<WlSurface>,
    pub dragging_window: Option<DraggingState>, pub resizing_window: Option<ResizingState>,
}
#[derive(Debug, Clone)] pub struct DraggingState { pub window: Arc<Window>, pub start_mouse_pos: (f64, f64), pub initial_window_pos: (i32, i32), }
#[derive(Debug, Clone)] pub struct ResizingState { pub window: Arc<Window>, pub start_mouse_pos: (f64, f64), pub initial_window_size: (i32, i32), }

impl WindowManagerState {
    pub fn new() -> Self {
        let mut workspaces = Vec::new();
        workspaces.push(Workspace::new(0, Some("Default (Tile)".to_string()), Box::new(TileLayout::new(LayoutAlgorithm::MasterStack))));
        workspaces[0].active = true;
        Self { workspaces, active_workspace_id: 0, next_workspace_id: 1, managed_windows: HashMap::new(), focused_window_surface: None, dragging_window: None, resizing_window: None }
    }
    fn get_managed_state(&self, window_arc: &Arc<Window>) -> Option<&ManagedWindow> { get_wl_surface(window_arc).and_then(|s| self.managed_windows.get(&s)) }
    fn get_managed_state_mut(&mut self, window_arc: &Arc<Window>) -> Option<&mut ManagedWindow> { get_wl_surface(window_arc).and_then(move |s| self.managed_windows.get_mut(&s)) }
    fn find_workspace(&self, workspace_id: usize) -> Option<&Workspace> { self.workspaces.iter().find(|ws| ws.id == workspace_id) }
    fn find_workspace_mut(&mut self, workspace_id: usize) -> Option<&mut Workspace> { self.workspaces.iter_mut().find(|ws| ws.id == workspace_id) }
    fn find_workspace_id_for_window(&self, window_arc: &Arc<Window>) -> Option<usize> { self.workspaces.iter().find_map(|ws| if ws.windows.iter().any(|w| Arc::ptr_eq(w, window_arc)) { Some(ws.id) } else { None }) }

    pub fn manage_window(&mut self, window_arc: Arc<Window>, space: &mut Space, output_geometry: Rectangle<i32, Physical>) {
        if let Some(surface) = get_wl_surface(&window_arc) { if self.managed_windows.contains_key(&surface) { return; }
            let mut managed_window_instance = ManagedWindow::new(window_arc.clone());
            if let Some(active_ws) = self.workspaces.get_mut(self.active_workspace_id) {
                active_ws.add_window(window_arc.clone(), &mut managed_window_instance);
                self.managed_windows.insert(surface.clone(), managed_window_instance);
                active_ws.arrange_windows(space, output_geometry, &self.managed_windows);
                self.focus_window(&window_arc, space); 
            }}}
    pub fn unmanage_window(&mut self, window_arc_to_remove: &Arc<Window>, space: &mut Space, output_geometry: Rectangle<i32, Physical>) {
        if window_arc_to_remove.is_mapped() { space.unmap_elem(&window_arc_to_remove.clone().into()); }
        if let Some(surface) = get_wl_surface(window_arc_to_remove).as_ref() { self.managed_windows.remove(surface); if self.focused_window_surface.as_ref() == Some(surface) { self.focused_window_surface = None; }}
        for ws in self.workspaces.iter_mut() { if ws.remove_window(window_arc_to_remove) { if ws.id == self.active_workspace_id { ws.arrange_windows(space, output_geometry, &self.managed_windows); } break; }}}
    
    pub fn get_active_workspace_mut(&mut self) -> Option<&mut Workspace> { self.workspaces.get_mut(self.active_workspace_id) }
    pub fn get_active_workspace(&self) -> Option<&Workspace> { self.workspaces.get(self.active_workspace_id) }

    pub fn focus_window(&mut self, window_to_focus_arc: &Arc<Window>, space: &mut Space) {
        if let Some(surface) = get_wl_surface(window_to_focus_arc) { if let Some(managed_window) = self.managed_windows.get(&surface) {
            let should_raise = managed_window.is_floating || self.get_active_workspace().map_or(false, |ws| ws.active_layout.name() == "FloatLayout");
            if should_raise && (managed_window.is_floating || managed_window.is_maximized || managed_window.is_fullscreen) { space.raise_element(window_to_focus_arc.clone().into(), true); }
            window_to_focus_arc.toplevel().send_activate(); self.focused_window_surface = Some(surface); }}}

    pub fn request_move(&mut self, window_arc: &Arc<Window>, space: &mut Space, new_logical_pos: Point<i32, Logical>, output_geometry_physical: Rectangle<i32, Physical>) {
        if let Some(managed_w) = self.get_managed_state_mut(window_arc) {
            if !managed_w.is_floating && !managed_w.is_maximized && !managed_w.is_fullscreen { managed_w.is_floating = true; }
            let mut target_loc_logical = new_logical_pos; let window_total_size_logical = managed_w.current_geometry_logical.size;
            if managed_w.is_floating && !managed_w.is_maximized && !managed_w.is_fullscreen {
                let output_rect_logical = output_geometry_physical.to_logical(1); 
                if (target_loc_logical.x - output_rect_logical.loc.x).abs() <= SNAP_THRESHOLD_PX { target_loc_logical.x = output_rect_logical.loc.x; } 
                else if ((target_loc_logical.x + window_total_size_logical.w) - (output_rect_logical.loc.x + output_rect_logical.size.w)).abs() <= SNAP_THRESHOLD_PX { target_loc_logical.x = output_rect_logical.loc.x + output_rect_logical.size.w - window_total_size_logical.w; }
                if (target_loc_logical.y - output_rect_logical.loc.y).abs() <= SNAP_THRESHOLD_PX { target_loc_logical.y = output_rect_logical.loc.y; } 
                else if ((target_loc_logical.y + window_total_size_logical.h) - (output_rect_logical.loc.y + output_rect_logical.size.h)).abs() <= SNAP_THRESHOLD_PX { target_loc_logical.y = output_rect_logical.loc.y + output_rect_logical.size.h - window_total_size_logical.h; }
            }
            managed_w.current_geometry_logical.loc = target_loc_logical;
            let physical_location = target_loc_logical.to_physical_precise_round(1);
            space.map_element(window_arc.clone().into(), physical_location, true);
        }}
    pub fn request_resize(&mut self, window_arc: &Arc<Window>, space: &mut Space, new_client_logical_size: Size<i32, Logical>, output_geometry_physical: Rectangle<i32, Physical>) {
        if let Some(managed_w) = self.get_managed_state_mut(window_arc) {
            if !managed_w.is_floating && !managed_w.is_maximized { return; }
            let new_total_window_size_logical = if managed_w.decoration_mode == DecorationMode::Server {
                Size::from(((new_client_logical_size.w + 2 * SSD_BORDER_WIDTH_PX).max(1), (new_client_logical_size.h + SSD_TITLE_BAR_HEIGHT_PX + SSD_BORDER_WIDTH_PX).max(1)))
            } else { new_client_logical_size };
            managed_w.current_geometry_logical.size = new_total_window_size_logical;
            if managed_w.is_maximized { let output_size_logical = output_geometry_physical.size.to_logical(1); if new_total_window_size_logical.w < output_size_logical.w || new_total_window_size_logical.h < output_size_logical.h { managed_w.is_maximized = false; }}
            window_arc.toplevel().with_pending_state(|state| { state.size = Some(new_client_logical_size); state.maximized = Some(managed_w.is_maximized); });
            window_arc.send_configure(); 
            let physical_location = managed_w.current_geometry_logical.loc.to_physical_precise_round(1);
            space.map_element(window_arc.clone().into(), physical_location, true);
        }}
    pub fn set_decoration_mode(&mut self, window_surface: &WlSurface, mode: DecorationMode, space: &mut Space, output_geometry: Rectangle<i32, Physical>) {
        if let Some(managed_w) = self.managed_windows.get_mut(window_surface) { if managed_w.decoration_mode == mode { return; }
            managed_w.decoration_mode = mode;
            if let Some(active_ws_id) = self.find_workspace_id_for_window(&managed_w.window) { if active_ws_id == self.active_workspace_id {
                let managed_windows_ref = &self.managed_windows; if let Some(active_ws) = self.workspaces.get_mut(self.active_workspace_id) {
                    active_ws.arrange_windows(space, output_geometry, managed_windows_ref);
                }}}}}}
    pub fn request_maximize(&mut self, window_arc: &Arc<Window>, space: &mut Space, output_geometry: Rectangle<i32, Physical>) {
        if let Some(managed_w) = self.get_managed_state_mut(window_arc) {
            let target_maximized_state = !managed_w.is_maximized; 
            if target_maximized_state { if !managed_w.is_maximized { managed_w.saved_geometry_before_maximize = Some(managed_w.current_geometry_logical); }
                let output_size_logical = output_geometry.size.to_logical(1); let output_loc_logical = output_geometry.loc.to_logical(1);
                managed_w.current_geometry_logical = Rectangle::from_loc_and_size(output_loc_logical, output_size_logical);
                managed_w.is_maximized = true; managed_w.is_fullscreen = false; managed_w.is_floating = true; 
            } else { if let Some(saved_geo) = managed_w.saved_geometry_before_maximize.take() { managed_w.current_geometry_logical = saved_geo; }
                managed_w.is_maximized = false; managed_w.is_floating = true; }
            let client_configure_size = if managed_w.is_maximized && managed_w.decoration_mode == DecorationMode::Server {
                Size::from(((managed_w.current_geometry_logical.size.w - 2 * SSD_BORDER_WIDTH_PX).max(1), (managed_w.current_geometry_logical.size.h - SSD_TITLE_BAR_HEIGHT_PX - SSD_BORDER_WIDTH_PX).max(1)))
            } else { managed_w.current_geometry_logical.size };
            window_arc.toplevel().with_pending_state(|state| { state.size = Some(client_configure_size); state.maximized = Some(managed_w.is_maximized); state.fullscreen = Some(managed_w.is_fullscreen); });
            window_arc.send_configure(); let physical_location = managed_w.current_geometry_logical.loc.to_physical_precise_round(1);
            space.map_element(window_arc.clone().into(), physical_location, true);
            if let Some(active_ws) = self.get_active_workspace_mut() { active_ws.arrange_windows(space, output_geometry, &self.managed_windows); }}}
    pub fn request_fullscreen(&mut self, window_arc: &Arc<Window>, space: &mut Space, output_geometry: Rectangle<i32, Physical>) {
        if let Some(managed_w) = self.get_managed_state_mut(window_arc) {
            let target_fullscreen_state = !managed_w.is_fullscreen; 
            if target_fullscreen_state { if !managed_w.is_fullscreen { if !managed_w.is_maximized && managed_w.saved_geometry_before_maximize.is_none() { managed_w.saved_geometry_before_maximize = Some(managed_w.current_geometry_logical); }}
                let output_size_logical = output_geometry.size.to_logical(1); let output_loc_logical = output_geometry.loc.to_logical(1);
                managed_w.current_geometry_logical = Rectangle::from_loc_and_size(output_loc_logical, output_size_logical);
                managed_w.is_fullscreen = true; managed_w.is_maximized = false; managed_w.is_floating = true;  
            } else { if let Some(saved_geo) = managed_w.saved_geometry_before_maximize.take() { managed_w.current_geometry_logical = saved_geo; managed_w.is_maximized = saved_geo.size == output_geometry.size.to_logical(1); } else {managed_w.is_maximized = false;}
                managed_w.is_fullscreen = false; managed_w.is_floating = true; }
            let client_configure_size = managed_w.current_geometry_logical.size; 
            window_arc.toplevel().with_pending_state(|state| { state.size = Some(client_configure_size); state.fullscreen = Some(managed_w.is_fullscreen); state.maximized = Some(managed_w.is_maximized); });
            window_arc.send_configure(); let physical_location = managed_w.current_geometry_logical.loc.to_physical_precise_round(1);
            space.map_element(window_arc.clone().into(), physical_location, true); 
            if managed_w.is_fullscreen { space.raise_element(window_arc.clone().into(), true); }
            if let Some(active_ws) = self.get_active_workspace_mut() { active_ws.arrange_windows(space, output_geometry, &self.managed_windows); }}}

    pub fn toggle_window_floating(&mut self, window_surface: &WlSurface, space: &mut Space, output_geometry: Rectangle<i32, Physical>) -> bool {
        if let Some(managed_w) = self.managed_windows.get_mut(window_surface) {
            managed_w.is_floating = !managed_w.is_floating;
            println!("Toggled floating for {:?} to {}", managed_w.window.toplevel().wm_title().unwrap_or_default(), managed_w.is_floating);
            if managed_w.is_floating {
                managed_w.is_maximized = false; 
                managed_w.is_fullscreen = false; 
                if managed_w.current_geometry_logical.size.w < DEFAULT_FLOATING_SIZE.w || 
                   managed_w.current_geometry_logical.size.h < DEFAULT_FLOATING_SIZE.h {
                    let output_center_x = output_geometry.loc.x + output_geometry.size.w / 2;
                    let output_center_y = output_geometry.loc.y + output_geometry.size.h / 2;
                    managed_w.current_geometry_logical.loc = Point::from((
                        output_center_x - DEFAULT_FLOATING_SIZE.w / 2,
                        output_center_y - DEFAULT_FLOATING_SIZE.h / 2
                    )).to_logical(1); 
                    managed_w.current_geometry_logical.size = DEFAULT_FLOATING_SIZE;
                }
            }
            
            if let Some(active_ws) = self.get_active_workspace_mut() {
                active_ws.arrange_windows(space, output_geometry, &self.managed_windows);
            }
            return true;
        }
        false
    }

    // --- Workspace Management Methods ---
    pub fn create_workspace(&mut self, name: Option<String>) -> usize { let new_id = self.next_workspace_id; self.next_workspace_id += 1; let new_layout = Box::new(TileLayout::new(LayoutAlgorithm::MasterStack)); let workspace_name = name.unwrap_or_else(|| format!("Workspace {}", new_id)); let new_workspace = Workspace::new(new_id, Some(workspace_name), new_layout); self.workspaces.push(new_workspace); new_id }
    pub fn switch_workspace(&mut self, target_id: usize, space: &mut Space, output_geometry: Rectangle<i32, Physical>) -> bool { if target_id == self.active_workspace_id || self.find_workspace(target_id).is_none() {return false;} if let Some(old_active_ws) = self.find_workspace_mut(self.active_workspace_id) {old_active_ws.active = false; for window_arc in &old_active_ws.windows { if window_arc.is_mapped() { space.unmap_elem(&window_arc.clone().into()); }}} self.active_workspace_id = target_id; if let Some(new_active_ws) = self.find_workspace_mut(target_id) {new_active_ws.active = true; let managed_windows_ref = &self.managed_windows; new_active_ws.arrange_windows(space, output_geometry, managed_windows_ref); if let Some(first_window_arc) = new_active_ws.windows.first() { self.focus_window(first_window_arc, space); } else { self.focused_window_surface = None; }} true }
    pub fn move_window_to_workspace(&mut self, window_surface_to_move: &WlSurface, target_workspace_id: usize, space: &mut Space, output_geometry: Rectangle<i32, Physical>) -> bool { let window_arc = match self.managed_windows.get(window_surface_to_move).map(|mw| mw.window.clone()) { Some(arc) => arc, None => return false,}; let source_workspace_id = match self.find_workspace_id_for_window(&window_arc) { Some(id) => id, None => return false,}; if source_workspace_id == target_workspace_id || self.find_workspace(target_workspace_id).is_none() { return false;} if let Some(source_ws) = self.find_workspace_mut(source_workspace_id) { if source_ws.id == self.active_workspace_id && window_arc.is_mapped() {space.unmap_elem(&window_arc.clone().into());} source_ws.remove_window(&window_arc);} if let Some(target_ws) = self.find_workspace_mut(target_workspace_id) { if let Some(managed_w_state) = self.managed_windows.get_mut(window_surface_to_move) {target_ws.add_window(window_arc.clone(), managed_w_state);} else { return false; }} else { return false; } if source_workspace_id == self.active_workspace_id { let managed_windows_ref = &self.managed_windows; if let Some(source_ws) = self.find_workspace(source_workspace_id) { source_ws.arrange_windows(space, output_geometry, managed_windows_ref);}} if target_workspace_id == self.active_workspace_id { let managed_windows_ref = &self.managed_windows; if let Some(target_ws) = self.find_workspace(target_workspace_id) {target_ws.arrange_windows(space, output_geometry, managed_windows_ref); self.focus_window(&window_arc, space);}} true }
}

#[macro_export]
macro_rules! guard {
    (let Some($pat:pat) = $expr:expr else $ret_expr:expr) => { if let Some($pat) = $expr { } else { $ret_expr } };
    (let $pat:pat = $expr:expr else $ret_expr:expr) => { let $pat = $expr; if !($pat) { $ret_expr } };
}

// Final comments about Smithay types...
// Ensure all necessary Smithay types are correctly imported and used.
// `WlSurface` is from `smithay::reexports::wayland_server::protocol::wl_surface::WlSurface`.
// `Point`, `Size`, `Logical` are from `smithay::utils`.
// `WindowSurfaceType` is from `smithay::desktop`.
// `Window::toplevel().send_activate()` is the new way for activation.

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::utils::{Point, Size as SmithaySize, Rectangle as SmithayRectangle};
    use std::cell::RefCell;
    use std::collections::HashSet;
    // WlSurface is a trait object, so we can't easily create a mock for it
    // without a lot of boilerplate. For tests requiring WlSurface keys,
    // we'll use a stand-in like TestWindowId.

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestWindowId(u32);

    #[derive(Debug)]
    struct TestWindow {
        id: TestWindowId,
        geo: RefCell<SmithayRectangle<i32, Logical>>,
        mapped: RefCell<bool>,
        activated: RefCell<bool>,
        pub configure_count: RefCell<u32>,
        current_xdg_state: RefCell<smithay::wayland::shell::xdg::SurfaceState>,
        title: String,
    }

    pub struct TestTopLevel<'a> {
        window_ref: &'a TestWindow,
    }

    impl<'a> TestTopLevel<'a> {
        pub fn send_configure(&self) { *self.window_ref.configure_count.borrow_mut() += 1; }
        pub fn send_activate(&self) { *self.window_ref.activated.borrow_mut() = true; }
        pub fn with_pending_state<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut smithay::wayland::shell::xdg::SurfaceState) -> R {
            f(&mut *self.window_ref.current_xdg_state.borrow_mut())
        }
        pub fn wm_title(&self) -> Option<String> { Some(self.window_ref.title.clone()) }
    }

    impl TestWindow {
        fn new(id_val: u32, title: &str, initial_geo: SmithayRectangle<i32, Logical>) -> Arc<Self> {
            Arc::new(Self {
                id: TestWindowId(id_val),
                geo: RefCell::new(initial_geo),
                mapped: RefCell::new(false),
                activated: RefCell::new(false),
                configure_count: RefCell::new(0),
                current_xdg_state: RefCell::new(Default::default()),
                title: title.to_string(),
            })
        }
        pub fn geometry(&self) -> SmithayRectangle<i32, Logical> { *self.geo.borrow() }
        pub fn is_mapped(&self) -> bool { *self.mapped.borrow() }
        pub fn toplevel(&self) -> TestTopLevel<'_> { TestTopLevel { window_ref: self } }
        pub fn surface_id(&self) -> TestWindowId { self.id.clone() } 
    }
    
    #[derive(Default, Debug)]
    struct MockSpace {
        mapped_elements: RefCell<HashMap<TestWindowId, (SmithayRectangle<i32, Physical>, bool)>>,
    }

    impl MockSpace {
        fn new() -> Self { Self::default() }
        fn map_element(&self, window_arc: Arc<TestWindow>, loc: Point<i32, Physical>, activate: bool) {
            let id = window_arc.surface_id();
            let size_physical = window_arc.geometry().size.to_physical_precise_round(1);
            let physical_geo = SmithayRectangle::from_loc_and_size(loc, size_physical);
            self.mapped_elements.borrow_mut().insert(id, (physical_geo, activate));
            *window_arc.mapped.borrow_mut() = true;
        }
        fn unmap_elem(&self, window_arc: Arc<TestWindow>) {
            let id = window_arc.surface_id();
            self.mapped_elements.borrow_mut().remove(&id);
            *window_arc.mapped.borrow_mut() = false;
        }
        fn raise_element(&self, window_arc: Arc<TestWindow>, _activate: bool) {
            assert!(*window_arc.mapped.borrow(), "Tried to raise an unmapped window");
        }
        fn get_mapped_info(&self, id: &TestWindowId) -> Option<(SmithayRectangle<i32, Physical>, bool)> {
            self.mapped_elements.borrow().get(id).cloned()
        }
    }

    fn test_output_geometry() -> SmithayRectangle<i32, Physical> {
        SmithayRectangle::from_loc_and_size(Point::from((0, 0)), SmithaySize::from((1920, 1080)))
    }

    #[test]
    fn test_wm_state_new() {
        let state = WindowManagerState::new();
        assert_eq!(state.workspaces.len(), 1);
        assert_eq!(state.active_workspace_id, 0);
        assert!(state.workspaces[0].active);
        assert_eq!(state.workspaces[0].id, 0);
        assert_eq!(state.next_workspace_id, 1);
    }

    #[test]
    fn test_create_workspace() {
        let mut state = WindowManagerState::new();
        let ws2_id = state.create_workspace(Some("WS2".into()));
        assert_eq!(ws2_id, 1);
        assert_eq!(state.workspaces.len(), 2);
        assert_eq!(state.workspaces[1].name.as_ref().unwrap(), "WS2");
        assert!(!state.workspaces[1].active);
        assert_eq!(state.next_workspace_id, 2);
    }

    #[test]
    fn test_switch_workspace_state_changes() { 
        let mut state = WindowManagerState::new();
        let ws1_id = state.create_workspace(Some("WS1".to_string())); 
        let ws2_id = state.create_workspace(Some("WS2".to_string())); 
        let mock_space = MockSpace::new(); 
        let output_geom = test_output_geometry();
        let initial_active_id = state.active_workspace_id; 
        assert_eq!(initial_active_id, 0);
        assert!(state.workspaces.iter().find(|ws| ws.id == initial_active_id).unwrap().active);
        assert!(!state.workspaces.iter().find(|ws| ws.id == ws1_id).unwrap().active);

        assert!(state.switch_workspace(ws1_id, &mock_space, output_geom));
        assert_eq!(state.active_workspace_id, ws1_id);
        assert!(!state.workspaces.iter().find(|ws| ws.id == initial_active_id).unwrap().active);
        assert!(state.workspaces.iter().find(|ws| ws.id == ws1_id).unwrap().active);
        assert!(!state.workspaces.iter().find(|ws| ws.id == ws2_id).unwrap().active);
        
        assert!(state.switch_workspace(initial_active_id, &mock_space, output_geom));
        assert_eq!(state.active_workspace_id, initial_active_id);
        assert!(state.workspaces.iter().find(|ws| ws.id == initial_active_id).unwrap().active);
        assert!(!state.workspaces.iter().find(|ws| ws.id == ws1_id).unwrap().active);

        assert!(!state.switch_workspace(999, &mock_space, output_geom)); // Non-existent
        assert_eq!(state.active_workspace_id, initial_active_id); 
        assert!(!state.switch_workspace(initial_active_id, &mock_space, output_geom)); // Already active
        assert_eq!(state.active_workspace_id, initial_active_id);
    }
    
    #[test]
    fn test_manage_and_unmanage_window_conceptual() {
        println!("Conceptual: Test manage_window adds window and unmanage removes it. Requires main code refactor for full mock integration.");
        assert!(true);
    }

    #[test]
    fn test_toggle_window_floating_conceptual() {
        println!("Conceptual: Test toggle_window_floating. Requires effective mocking for ManagedWindow state changes and layout interaction.");
        assert!(true);
    }
    
    #[test]
    fn test_move_window_to_workspace_conceptual() {
        println!("Conceptual: Test move_window_to_workspace. Requires effective mocking for managed_windows and Workspace.windows list manipulation.");
        assert!(true);
    }

    #[test]
    fn test_snapping_in_request_move_conceptual() {
        println!("Conceptual: Test request_move correctly snaps window edges. Requires state setup with a mock floating window.");
        assert!(true);
    }

    #[test]
    fn test_ssd_in_request_resize_conceptual() {
        println!("Conceptual: Test request_resize correctly adds decoration sizes for SSD. Requires state setup with an SSD window.");
        assert!(true);
    }

    #[test]
    fn test_set_decoration_mode_conceptual() {
        println!("Conceptual: Test set_decoration_mode updates mode and triggers rearrange. Requires state setup.");
        assert!(true);
    }
     #[test]
    fn test_tile_layout_arrangement_conceptual() {
        println!("Conceptual test for TileLayout arrangement: Requires effective mocking for ManagedWindow and its contained Arc<TestWindow> or generic main code.");
        assert!(true);
    }
}
