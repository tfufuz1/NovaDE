// ANCHOR: TilingModuleImplementation
//! Implements tiling algorithms and layout application logic.

use std::collections::HashMap;
use std::sync::Arc;
use smithay::utils::{Rectangle, Logical, Size, Point};
use smithay::desktop::Space; // Needed for apply_active_tiling_layout
use uuid::Uuid;

use crate::compositor::shell::xdg_shell::types::{DomainWindowIdentifier, ManagedWindow};
use crate::compositor::workspaces::{CompositorWorkspace, TilingLayout};
use crate::compositor::core::state::DesktopState; // Needed for apply_active_tiling_layout

// Import SSD constants
use crate::compositor::shell::xdg_shell::types::{DEFAULT_BORDER_SIZE, DEFAULT_TITLE_BAR_HEIGHT};


/// Calculates geometries for a master-stack tiling layout.
///
/// - If one window, it gets the full workspace_area.
/// - If multiple, one master window gets a large portion (e.g., left 50-70%),
///   and other windows (stack) are tiled vertically in the remaining portion.
pub fn calculate_master_stack_layout(
    windows_in_workspace: &[Arc<ManagedWindow>], // Windows to tile (already filtered for current workspace, mapped, non-minimized)
    workspace_area: Rectangle<i32, Logical>,
    master_factor: f32, // e.g., 0.6 for 60% master area
) -> HashMap<DomainWindowIdentifier, Rectangle<i32, Logical>> {
    let mut layouts = HashMap::new();
    let num_windows = windows_in_workspace.len();

    if num_windows == 0 || workspace_area.size.w == 0 || workspace_area.size.h == 0 {
        return layouts;
    }

    if num_windows == 1 {
        let window_arc = &windows_in_workspace[0];
        layouts.insert(window_arc.domain_id, workspace_area);
        return layouts;
    }

    // Identify master window: the one with tiling_master=true, or first one if none.
    let master_window_arc = windows_in_workspace.iter()
        .find(|w| *w.tiling_master.read().unwrap())
        .cloned()
        .unwrap_or_else(|| windows_in_workspace[0].clone());

    let stack_windows: Vec<Arc<ManagedWindow>> = windows_in_workspace.iter()
        .filter(|w| w.id != master_window_arc.id) // Compare by ManagedWindow's unique ID
        .cloned()
        .collect();

    let master_width = (workspace_area.size.w as f32 * master_factor).round() as i32;
    let stack_width = workspace_area.size.w - master_width;

    // Master window geometry
    let master_geom = Rectangle::from_loc_and_size(
        workspace_area.loc,
        (master_width, workspace_area.size.h).into(),
    );
    layouts.insert(master_window_arc.domain_id, master_geom);

    // Stack windows geometry
    if !stack_windows.is_empty() && stack_width > 0 {
        let num_stack_windows = stack_windows.len();
        let stack_window_height = (workspace_area.size.h / num_stack_windows as i32).max(1); // Ensure at least 1px height

        for (i, window_arc) in stack_windows.iter().enumerate() {
            let loc = Point::from((
                workspace_area.loc.x + master_width,
                workspace_area.loc.y + (i as i32 * stack_window_height),
            ));
            let size = Size::from((stack_width, stack_window_height));
            layouts.insert(window_arc.domain_id, Rectangle::from_loc_and_size(loc, size));
        }
    } else if !stack_windows.is_empty() && stack_width <= 0 {
        // Master took all space, hide stack windows (or give them 0x0 size at origin)
        for window_arc in stack_windows {
             layouts.insert(window_arc.domain_id, Rectangle::from_loc_and_size(workspace_area.loc, (0,0).into()));
        }
    }


    layouts
}

// ANCHOR: ApplyLayoutForOutputSignature
/// Applies the active tiling layout to windows on the specified output's active workspace.
pub fn apply_layout_for_output(
    desktop_state: &mut DesktopState,
    output_name: &str,
) {
// ANCHOR_END: ApplyLayoutForOutputSignature
    // ANCHOR: GetActiveWorkspaceForOutput
    let active_workspaces_guard = desktop_state.active_workspaces.read().unwrap();
    let active_workspace_id_on_output = match active_workspaces_guard.get(output_name) {
        Some(id) => *id,
        None => {
            tracing::warn!("apply_layout_for_output: No active workspace ID found for output {}. Cannot apply layout.", output_name);
            return;
        }
    };
    drop(active_workspaces_guard); // Release read lock

    let workspaces_on_output_vec = match desktop_state.output_workspaces.get(output_name) {
        Some(vec) => vec.clone(), // Clone Vec<Arc<RwLock<CompositorWorkspace>>>
        None => {
            tracing::warn!("apply_layout_for_output: No workspaces found for output {}. Cannot apply layout.", output_name);
            return;
        }
    };

            // ANCHOR_END: GetActiveWorkspaceForOutput
            return;
        }
    };

    let layout_mode = *active_workspace_arc.read().unwrap().tiling_layout.read().unwrap();

    // ANCHOR: ApplyTilingLayoutGetWindowsForOutputRefined
    let windows_to_layout: Vec<Arc<ManagedWindow>> = desktop_state.windows.values()
        .filter(|mw| *mw.workspace_id.read().unwrap() == Some(active_workspace_id_on_output))
        .filter(|mw| *mw.output_name.read().unwrap() == Some(output_name.to_string()))
        .filter(|mw| !mw.state.read().unwrap().minimized)
        .filter(|mw| matches!(mw.xdg_surface, CompositorWindowSurface::Toplevel(_))) // Use aliased WindowSurface
        .cloned()
        .collect();

    if windows_to_layout.is_empty() {
        tracing::debug!("No windows to layout on workspace {} for output {}.", active_workspace_id_on_output, output_name);
        if let Some(output_obj) = desktop_state.outputs.iter().find(|o| o.name() == output_name) {
            desktop_state.space.damage_output(output_obj, None, None);
        }
        return;
    }
    // ANCHOR_END: ApplyTilingLayoutGetWindowsForOutputRefined

    // ANCHOR: GetWorkspaceAreaForSpecificOutputRefined
    let output_geometry_in_global_space = desktop_state.outputs.iter()
        .find(|o| o.name() == output_name)
        .and_then(|o| desktop_state.space.output_geometry(o))
        .unwrap_or_else(|| {
            tracing::warn!("No output geometry found for tiling layout on output {}, workspace {}, defaulting to 800x600 at (0,0) global.", output_name, active_workspace_id_on_output);
            Rectangle::from_loc_and_size((0,0), (800,600))
        });
    // ANCHOR_END: GetWorkspaceAreaForSpecificOutputRefined

    // ANCHOR: ApplyTilingLayoutHandleNoneLayoutForOutputRefinedInPlace
    if layout_mode == TilingLayout::None {
        tracing::debug!("Workspace {} on output {} has TilingLayout::None. Ensuring all relevant windows are mapped with their current (floating) geometry.", active_workspace_id_on_output, output_name);
        for window_arc in &windows_to_layout {
            let geometry_guard = window_arc.current_geometry.read().unwrap();
            let window_global_geometry = *geometry_guard;
            drop(geometry_guard);

            let mut win_state_guard = window_arc.state.write().unwrap();
            if !win_state_guard.minimized {
                win_state_guard.is_mapped = true;
                // Ensure window position in space is correct; current_geometry is global.
                // Ensure window is on the correct output in the space.
                if let Some(output_obj) = desktop_state.outputs.iter().find(|o| o.name() == output_name) {
                    desktop_state.space.map_element_to_output(window_arc.clone(), output_obj, true);
                    desktop_state.space.map_window(window_arc.clone(), window_global_geometry.loc, false);
                } else {
                    tracing::warn!("Output {} not found when trying to map window {} in TilingLayout::None mode.", output_name, window_arc.id);
                }
            }
        }
        if let Some(output_obj) = desktop_state.outputs.iter().find(|o| o.name() == output_name) {
            desktop_state.space.damage_output(output_obj, None, None);
        }
        return;
    }
    // ANCHOR_END: ApplyTilingLayoutHandleNoneLayoutForOutputRefinedInPlace

    // For tiling, calculations are done relative to the output's origin (0,0)
    // then translated to global coordinates.
    let tiling_area_for_calc = Rectangle::from_loc_and_size(Point::default(), output_geometry_in_global_space.size);

    let new_geometries_relative_to_output = match layout_mode {
        TilingLayout::MasterStack => {
            calculate_master_stack_layout(&windows_to_layout, tiling_area_for_calc, 0.6)
        }
        TilingLayout::None => unreachable!(),
    };

    for window_arc in &windows_to_layout {
        if let Some(new_geom_relative) = new_geometries_relative_to_output.get(&window_arc.domain_id) {

            let new_global_geom = Rectangle::from_loc_and_size(
                output_geometry_in_global_space.loc + new_geom_relative.loc,
                new_geom_relative.size
            );

            let current_geom_guard = window_arc.current_geometry.read().unwrap();
            let needs_update = *current_geom_guard != new_global_geom || !window_arc.is_mapped() || layout_mode != TilingLayout::None;
            drop(current_geom_guard);

            if needs_update {
                tracing::info!("Tiling: Applying geometry {:?} to window {:?} (Domain ID: {:?}) on workspace {}",
                    new_geom, window_arc.id, window_arc.domain_id, active_workspace_id);

                *window_arc.current_geometry.write().unwrap() = *new_geom;
                let mut win_state = window_arc.state.write().unwrap();
                win_state.position = new_geom.loc;
                win_state.size = new_geom.size;
                // Ensure states like maximized/fullscreen are unset if we are applying a tiled geom
                win_state.maximized = false;
                win_state.fullscreen = false;
                win_state.is_mapped = true; // Tiled windows are by definition mapped on the active workspace
                drop(win_state);

                if let WindowSurface::Toplevel(toplevel_surface) = &window_arc.xdg_surface {
                    let manager_props = window_arc.manager_data.read().unwrap();
                    let is_ssd = manager_props.decorations;
                    drop(manager_props);

                    let content_size = if is_ssd {
                        Size::from((
                            (new_geom.size.w - 2 * DEFAULT_BORDER_SIZE).max(1),
                            (new_geom.size.h - DEFAULT_TITLE_BAR_HEIGHT - 2 * DEFAULT_BORDER_SIZE).max(1)
                        ))
                    } else {
                        new_geom.size
                    };

                    toplevel_surface.with_pending_state(|pending_state| {
                        pending_state.size = Some(content_size);
                        pending_state.states.unset(XdgToplevelStateSmithay::Maximized);
                        pending_state.states.unset(XdgToplevelStateSmithay::Fullscreen);
                    });
                    let _serial = toplevel_surface.send_configure();
                    // TODO: Update ManagedWindow.last_configure_serial if mutable
                }
                desktop_state.space.map_window(window_arc.clone(), new_geom.loc, false); // Map/update in space
            }
        }
    }
    desktop_state.space.damage_all_outputs();
}

// ANCHOR_END: TilingModuleImplementation
