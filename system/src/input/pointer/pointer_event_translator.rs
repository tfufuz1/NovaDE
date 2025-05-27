use crate::compositor::core::state::DesktopState;
use crate::compositor::xdg_shell::types::ManagedWindow; // Assuming this is your Window type for Space
use smithay::desktop::{Window, WindowSurfaceType};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle};
use smithay::wayland::compositor::with_states; // For accessing SurfaceData/input_region
use crate::compositor::surface_management::SurfaceData; // Path to your SurfaceData
use std::sync::Arc; // For SurfaceData if stored in Arc<Mutex<...>>

use smithay::backend::input::{
    LibinputInputBackend, PointerAxisEvent, PointerButtonEvent,
    PointerMotionEvent, PointerMotionAbsoluteEvent, ButtonState as LibinputButtonState,
};
use smithay::input::pointer::{PointerHandle, ButtonState as SmithayButtonState, AxisSource};
use smithay::input::Seat;
use smithay::reexports::wayland_protocols::wayland::server::protocol::wl_pointer; // For wl_pointer::Axis enum

// Helper function to find the surface and its local coordinates at a given global point.
pub fn find_surface_and_coords_at_global_point(
    desktop_state: &DesktopState,
    global_pos: Point<f64, Logical>,
) -> (Option<WlSurface>, Point<f64, Logical>) {
    for window_arc in desktop_state.space.elements_under(global_pos).rev() {
        let window: &ManagedWindow = &(*window_arc);

        if !window.is_mapped() {
            continue;
        }

        let window_loc = desktop_state.space.element_location(window_arc).unwrap_or_default();
        let window_geom = window.geometry();
        
        let window_total_geometry = Rectangle::from_loc_and_size(window_loc, window_geom.size);

        if !window_total_geometry.to_f64().contains(global_pos) {
            continue;
        }

        // Calculate position relative to the window's origin
        let point_relative_to_window_origin = global_pos - window_loc.to_f64();

        // Use Window::surface_under() to find the most specific surface (main, subsurface, or popup).
        // WindowSurfaceType::ALL ensures we check all types.
        if let Some((target_surface, location_on_target_surface)) = 
            window.surface_under(point_relative_to_window_origin, WindowSurfaceType::ALL)
        {
            // Now, verify input region for this specific target_surface.
            // The location_on_target_surface is already relative to this target_surface.
            let mut is_within_input_region = false;
            with_states(&target_surface, |states| {
                // Check if the surface has specific SurfaceData with an input region.
                if let Some(surface_data_arc) = states.data_map.get::<Arc<std::sync::Mutex<SurfaceData>>>() {
                    let surface_data_guard = surface_data_arc.lock().unwrap();
                    if let Some(input_region_local_to_target) = &surface_data_guard.input_region_surface_local {
                        // location_on_target_surface is Point<f64, Logical>
                        // input_region_local_to_target is Region<Logical> which typically stores i32.
                        if input_region_local_to_target.contains(location_on_target_surface.to_i32_round()) {
                            is_within_input_region = true;
                        }
                        // If input_region is Some but doesn't contain the point, is_within_input_region remains false.
                    } else {
                        // No specific input region defined in SurfaceData, so the entire surface is considered.
                        is_within_input_region = true;
                    }
                } else {
                    // No custom SurfaceData found, assume the entire surface accepts input.
                    // This is a fallback; ideally, all relevant surfaces would have SurfaceData.
                    is_within_input_region = true;
                }
            });

            if is_within_input_region {
                return (Some(target_surface.clone()), location_on_target_surface);
            }
            // If not within the input region of the specific surface found by surface_under,
            // we don't return this surface and let the loop continue to check other windows below.
            // This implements click-through for areas outside a surface's defined input region.
        }
        // If surface_under returned None but we are within the window's bounding box,
        // it means the point is on the window's "decorations" or an area not covered by any wl_surface.
        // In this case, we also don't return a surface and let the loop continue.
    }
    (None, global_pos) // No suitable surface found among all windows.
}

pub fn handle_pointer_motion_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: PointerMotionEvent<LibinputInputBackend>,
) {
    let pointer_handle = match seat.get_pointer() {
        Some(h) => h,
        None => return,
    };

    // Update global pointer location based on event delta
    desktop_state.pointer_location += event.delta();

    // --- Apply Pointer Constraints ---
    // Get the surface currently focused by the pointer (for constraint context)
    let constraint_focus_surface_option: Option<WlSurface> = pointer_handle.current_focus().cloned();

    // Apply constraints to desktop_state.pointer_location
    // constrain_motion takes &mut Point<f64, Logical> for the location to be adjusted
    desktop_state.pointer_constraints_state.constrain_motion(
        &pointer_handle, // Needs a &PointerHandle<DesktopState>
        &mut desktop_state.pointer_location,
        constraint_focus_surface_option.as_ref(), // The surface the constraint might be relative to
    );
    // --- End Apply Pointer Constraints ---

    // Optional: Clamp pointer_location to output boundaries here
    // desktop_state.pointer_location = clamp_to_outputs(desktop_state.pointer_location, &desktop_state.space);

    // Recalculate target surface and local coords based on potentially constrained pointer_location
    let (final_target_surface_option, final_surface_local_coords) =
        find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);

    pointer_handle.motion(
        desktop_state,
        final_target_surface_option.as_ref(),
        event.serial(),
        desktop_state.pointer_location, // Use the (potentially constrained) global coordinates
        final_surface_local_coords,
        event.time(),
        Some(tracing::Span::current()),
    );
    
    desktop_state.active_input_surface = final_target_surface_option.map(|s| s.downgrade());
}

pub fn handle_pointer_motion_absolute_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: PointerMotionAbsoluteEvent<LibinputInputBackend>,
) {
    let pointer_handle = match seat.get_pointer() {
        Some(h) => h,
        None => return,
    };

    let output_geometry = desktop_state.space.outputs()
        .next()
        .map(|o| desktop_state.space.output_geometry(o).unwrap_or_else(|| Rectangle::from_loc_and_size((0,0), (1920, 1080))))
        .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0), (1920, 1080)));

    // Transform absolute coordinates (potentially normalized or per-output) to global logical coordinates
    // The event.absolute_x_transformed() and y_transformed() methods scale normalized (0-1) coordinates
    // to the provided width/height. If the event provides coordinates in a different system,
    // this part might need adjustment (e.g., if they are already in a global device space).
    // The subtask description had new_x, new_y but the original code had global_x, global_y.
    // I will use global_x, global_y to keep it consistent with the original code's variable naming for clarity.
    let global_x = event.absolute_x_transformed(output_geometry.size.w as u32) + output_geometry.loc.x as f64;
    let global_y = event.absolute_y_transformed(output_geometry.size.h as u32) + output_geometry.loc.y as f64;
    
    // Set initial pointer_location from absolute event
    desktop_state.pointer_location = Point::from((global_x, global_y));

    // --- Apply Pointer Constraints ---
    let constraint_focus_surface_option: Option<WlSurface> = pointer_handle.current_focus().cloned();
    desktop_state.pointer_constraints_state.constrain_motion(
        &pointer_handle,
        &mut desktop_state.pointer_location,
        constraint_focus_surface_option.as_ref(),
    );
    // --- End Apply Pointer Constraints ---

    // Optional: Clamp pointer_location to output boundaries here
    // desktop_state.pointer_location = clamp_to_outputs(desktop_state.pointer_location, &desktop_state.space);

    let (final_target_surface_option, final_surface_local_coords) =
        find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);

    pointer_handle.motion( // Using motion here is fine as pointer_location is now global and constrained
        desktop_state,
        final_target_surface_option.as_ref(),
        event.serial(),
        desktop_state.pointer_location,
        final_surface_local_coords,
        event.time(),
        Some(tracing::Span::current()),
    );
    desktop_state.active_input_surface = final_target_surface_option.map(|s| s.downgrade());
}

pub fn handle_pointer_button_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: PointerButtonEvent<LibinputInputBackend>,
) {
    let pointer_handle = match seat.get_pointer() {
        Some(h) => h,
        None => return,
    };

    let serial = event.serial();
    let time = event.time();
    
    let smithay_button_state = match event.button_state() {
        LibinputButtonState::Pressed => SmithayButtonState::Pressed,
        LibinputButtonState::Released => SmithayButtonState::Released,
    };

    pointer_handle.button(
        desktop_state,
        event.button(),
        smithay_button_state,
        serial,
        time,
        Some(tracing::Span::current()),
    );

    if smithay_button_state == SmithayButtonState::Pressed {
        let (focused_surface_option, _coords) = 
            find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);

        if let Some(surface_to_focus) = focused_surface_option {
            // Assuming ManagedWindow has a method like `is_focusable_toplevel`
            // or check can be done via space.window_for_surface and its properties.
            let window_can_focus = desktop_state.space.window_for_surface(&surface_to_focus, WindowSurfaceType::TOPLEVEL)
                .map_or(false, |w| w.is_mapped()); // Simplified: any mapped toplevel can get focus
                                                   // A more specific `w.can_receive_focus()` would be better.

            if window_can_focus {
                 match crate::input::keyboard::focus::set_keyboard_focus(desktop_state, seat.name(), Some(&surface_to_focus), serial) {
                    Ok(_) => tracing::debug!("Keyboard focus set to surface {:?} due to pointer click.", surface_to_focus.id()),
                    Err(e) => tracing::error!("Failed to set keyboard focus on click: {:?}", e),
                }
            } else if desktop_state.space.window_for_surface(&surface_to_focus, WindowSurfaceType::POPUP).is_some() {
                tracing::debug!("Click on a popup surface {:?}, keyboard focus not changed.", surface_to_focus.id());
            }
        } else {
            match crate::input::keyboard::focus::set_keyboard_focus(desktop_state, seat.name(), None, serial) {
                Ok(_) => tracing::debug!("Keyboard focus cleared due to click on empty space."),
                Err(e) => tracing::error!("Failed to clear keyboard focus on click: {:?}", e),
            }
        }
    }
}

pub fn handle_pointer_axis_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: PointerAxisEvent<LibinputInputBackend>,
) {
    let pointer_handle = match seat.get_pointer() {
        Some(h) => h,
        None => return,
    };

    // axis_values_discrete gives (Option<f64>, Option<f64>) for (Horizontal, Vertical)
    let (h_discrete_opt, v_discrete_opt) = event.axis_values_discrete();
    let horizontal_amount_discrete = h_discrete_opt.unwrap_or(0.0);
    let vertical_amount_discrete = v_discrete_opt.unwrap_or(0.0);

    // axis_value gives f64 for a single axis
    let horizontal_amount_continuous = if horizontal_amount_discrete == 0.0 { event.axis_value(smithay::reexports::input::event::pointer::Axis::Horizontal) } else { 0.0 };
    let vertical_amount_continuous = if vertical_amount_discrete == 0.0 { event.axis_value(smithay::reexports::input::event::pointer::Axis::Vertical) } else { 0.0 };
    
    let source = match event.axis_source() {
        Some(s) => match s {
            smithay::reexports::input::event::pointer::AxisSource::Wheel => AxisSource::Wheel,
            smithay::reexports::input::event::pointer::AxisSource::Finger => AxisSource::Finger,
            smithay::reexports::input::event::pointer::AxisSource::Continuous => AxisSource::Continuous,
            // Smithay 0.3.0 AxisSource enum also has WheelTilt.
            // Libinput's AxisSource enum might have more variants that need mapping.
            _ => AxisSource::Wheel, // Fallback for other libinput sources
        },
        None => AxisSource::Wheel, // Default if libinput provides no source
    };

    let time = event.time();
    let serial = event.serial();

    if vertical_amount_discrete.abs() > 1e-6 || vertical_amount_continuous.abs() > 1e-6 {
        pointer_handle.axis(
            desktop_state,
            wl_pointer::Axis::VerticalScroll,
            source,
            vertical_amount_discrete,
            vertical_amount_continuous,
            serial,
            time,
            Some(tracing::Span::current()),
        );
    }

    if horizontal_amount_discrete.abs() > 1e-6 || horizontal_amount_continuous.abs() > 1e-6 {
        pointer_handle.axis(
            desktop_state,
            wl_pointer::Axis::HorizontalScroll,
            source,
            horizontal_amount_discrete,
            horizontal_amount_continuous,
            serial,
            time,
            Some(tracing::Span::current()),
        );
    }
}
