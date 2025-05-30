use smithay::{
    backend::input::{InputEvent as BackendInputEvent, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent, PointerMotionAbsoluteEvent, LibinputInputBackend, Axis, ButtonState},
    input::{Seat, pointer::{PointerHandle, AxisFrame, MotionEvent, ButtonEvent, RelativeMotionEvent, GrabStartData as PointerGrabStartData, Focus}, SeatHandler}, // Added SeatHandler
    reexports::{
        wayland_server::protocol::wl_surface::WlSurface,
        input::event::pointer::{PointerAxisEventTrait, PointerButtonEventTrait, PointerMotionEventTrait, PointerMotionAbsoluteEventTrait},
        wayland_protocols::xdg::shell::server::xdg_toplevel, // For resize edge
    },
    utils::{Logical, Point, Serial, físico_a_lógico, Size},
    desktop::{Window, Space}, // Added Space for element_under
    output::Output, // For clamping
};
use crate::{
    compositor::core::state::DesktopState,
    input::errors::InputError,
    input::keyboard::set_keyboard_focus, // For click-to-focus
    // window_mechanics::interactive_ops::{start_interactive_move, start_interactive_resize}, // For later
};
use super::focus::update_pointer_focus_and_send_motion; // Corrected path
use std::sync::Arc; // For SurfaceData
use parking_lot::Mutex; // For SurfaceData
use crate::compositor::surface_management::SurfaceData; // For SurfaceData

// Marked as pub(super) as it's used by focus.rs too if focus logic needs it,
// but the prompt has it as pub(super) within event_translator.rs, used by handlers here.
// If find_surface_and_coords_at_global_point is only used within this module, it can be private.
// The mod.rs re-exports it, so it needs to be at least pub(crate).
// For now, pub(crate) to match re-export.
pub(crate) fn find_surface_and_coords_at_global_point(
    desktop_state: &DesktopState,
    global_pos: Point<f64, Logical>,
) -> (Option<WlSurface>, Point<f64, Logical>) {
    let mut found_surface_info: Option<(WlSurface, Point<f64, Logical>)> = None;

    // Use space.surface_under for a more direct way to get the surface and local coords
    let surface_under_output = desktop_state.space.surface_under(global_pos, true);

    if let Some((wl_surface, surface_local_coords)) = surface_under_output {
        // Verify if the wl_surface belongs to one of our managed windows and is mapped.
        // This check is important if space.surface_under might return surfaces not part of our window list.
        let is_managed_and_mapped = desktop_state.space.elements()
            .any(|win_element| win_element.wl_surface().as_ref() == Some(&wl_surface) && win_element.is_mapped());

        if is_managed_and_mapped {
            // Optional: Input region check (simplified for now)
            // smithay::wayland::compositor::with_states(&wl_surface, |states| {
            //     if let Some(surface_data_arc) = states.data_map.get::<Arc<Mutex<SurfaceData>>>() {
            //         let surface_data = surface_data_arc.lock();
            //         if let Some(input_region) = &surface_data.input_region_surface_local {
            //             if input_region.contains(surface_local_coords.to_i32_round()) {
            //                 found_surface_info = Some((wl_surface.clone(), surface_local_coords));
            //             }
            //         } else { // No input region, whole surface accepts input
            //             found_surface_info = Some((wl_surface.clone(), surface_local_coords));
            //         }
            //     } else { // No SurfaceData, assume whole surface accepts input for now
            //         found_surface_info = Some((wl_surface.clone(), surface_local_coords));
            //     }
            // });
            // Simplified: if surface_under found it and it's mapped, use it.
             found_surface_info = Some((wl_surface, surface_local_coords));
        }
    }

    if let Some((surface, local_coords)) = found_surface_info {
        (Some(surface), local_coords)
    } else {
        (None, global_pos) // No suitable surface found, return global_pos
    }
}

pub fn handle_pointer_motion_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: PointerMotionEvent<LibinputInputBackend>,
) -> Result<(), InputError> {
    let old_pointer_location = desktop_state.pointer_location;
    desktop_state.pointer_location += event.delta();

    // Clamp pointer_location to output boundaries
    let mut new_x = desktop_state.pointer_location.x;
    let mut new_y = desktop_state.pointer_location.y;

    if !desktop_state.outputs.is_empty() {
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for output in desktop_state.space.outputs() {
            if let Some(output_geometry) = desktop_state.space.output_geometry(output) {
                min_x = min_x.min(output_geometry.loc.x as f64);
                max_x = max_x.max((output_geometry.loc.x + output_geometry.size.w) as f64);
                min_y = min_y.min(output_geometry.loc.y as f64);
                max_y = max_y.max((output_geometry.loc.y + output_geometry.size.h) as f64);
            }
        }
        // Only clamp if values were updated (i.e. outputs exist)
        if min_x != f64::MAX { // implies max_x, min_y, max_y also set
             new_x = new_x.clamp(min_x, max_x);
             new_y = new_y.clamp(min_y, max_y);
        }
    }
    desktop_state.pointer_location = (new_x, new_y).into();

    if old_pointer_location.distance_squared(desktop_state.pointer_location) < f64::EPSILON && seat.get_pointer().unwrap().current_focus().is_some() {
        // If pointer hasn't moved significantly and focus is already set,
        // we might avoid re-calculating surface_under if it's expensive.
        // However, window stacking order might have changed, so it's safer to always check.
    }

    let (new_focus_surface, surface_local_coords) =
        find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);

    let pointer_handle = seat.get_pointer().ok_or_else(|| InputError::PointerHandleNotFound(seat.name().to_string()))?;

    update_pointer_focus_and_send_motion(
        desktop_state,
        seat,
        &pointer_handle,
        new_focus_surface,
        surface_local_coords,
        event.time(),
        event.serial(),
    )
}

pub fn handle_pointer_motion_absolute_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: PointerMotionAbsoluteEvent<LibinputInputBackend>,
) -> Result<(), InputError> {
    // Smithay's event.position_transformed(output_size) is good.
    // We need to determine which output this event belongs to.
    // Libinput devices can be associated with outputs.
    // For now, a simplification: assume it's for the "primary" output or first one.
    let mut transformed_pos = desktop_state.pointer_location; // Default to current if no output found

    if let Some(output) = desktop_state.space.outputs().next() { // Get the first output
        if let Some(output_geometry) = desktop_state.space.output_geometry(output) {
             // The event gives coordinates normalized to its own output.
             // We need to map this to global compositor space.
            let output_size_w = output_geometry.size.w;
            let output_size_h = output_geometry.size.h;

            let absolute_x_on_output = event.x_transformed(output_size_w as u32);
            let absolute_y_on_output = event.y_transformed(output_size_h as u32);

            transformed_pos = (
                output_geometry.loc.x as f64 + absolute_x_on_output,
                output_geometry.loc.y as f64 + absolute_y_on_output
            ).into();
        } else {
            tracing::warn!("PointerMotionAbsolute: Output found but no geometry. Using event's raw position as global.");
            transformed_pos = (event.x(), event.y()).into();
        }
    } else {
        tracing::warn!("PointerMotionAbsolute: No outputs found in space. Using event's raw position as global.");
        // Fallback: use raw coordinates, may not be ideal.
        transformed_pos = (event.x(), event.y()).into();
    }
    desktop_state.pointer_location = transformed_pos;


    let (new_focus_surface, surface_local_coords) =
        find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);

    let pointer_handle = seat.get_pointer().ok_or_else(|| InputError::PointerHandleNotFound(seat.name().to_string()))?;

    update_pointer_focus_and_send_motion(
        desktop_state,
        seat,
        &pointer_handle,
        new_focus_surface,
        surface_local_coords,
        event.time(),
        event.serial(),
    )
}

pub fn handle_pointer_button_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: PointerButtonEvent<LibinputInputBackend>,
) -> Result<(), InputError> {
    let pointer_handle = seat.get_pointer().ok_or_else(|| InputError::PointerHandleNotFound(seat.name().to_string()))?;
    let serial = event.serial();
    let time = event.time();

    pointer_handle.button(event.button(), event.button_state().into(), serial, time, Some(tracing::Span::current()));

    if event.button_state() == ButtonState::Pressed {
        // We need the current pointer focus to determine which surface to activate/focus
        let current_pointer_focus = pointer_handle.current_focus(); // This is Option<WlSurface>

        if let Some(surface_to_focus) = current_pointer_focus {
            // Click-to-focus: Set keyboard focus to the clicked surface
            if let Err(e) = set_keyboard_focus(desktop_state, seat.name(), Some(&surface_to_focus), serial) {
                tracing::warn!("Failed to set keyboard focus on click for seat {}: {:?}", seat.name(), e);
            }
            // TODO: Window manager actions like grab for move/resize.
            // This would involve checking if the surface_to_focus is part of a window decoration,
            // or if a specific key modifier is pressed (e.g., Alt + Click for move).
            // For example:
            // if let Some(window) = desktop_state.window_for_surface(&surface_to_focus) {
            //     if window.is_decoration_click(surface_local_coords, event.button()) {
            //          // Start move or resize grab
            //     }
            // }
        } else {
            // If no surface is focused by the pointer, but a button is pressed,
            // we might still want to set keyboard focus to None or handle it globally.
            // For now, if pointer has no focus, keyboard focus is not changed by click.
            tracing::debug!("Pointer button pressed on seat {} but no surface was under pointer focus.", seat.name());
        }
    }
    Ok(())
}

pub fn handle_pointer_axis_event(
    _desktop_state: &mut DesktopState, // Not mutable if only reading pointer_handle and not changing global state
    seat: &Seat<DesktopState>,
    event: PointerAxisEvent<LibinputInputBackend>,
) -> Result<(), InputError> {
    let pointer_handle = seat.get_pointer().ok_or_else(|| InputError::PointerHandleNotFound(seat.name().to_string()))?;

    let mut frame = AxisFrame::new(event.time(), event.serial(), Some(tracing::Span::current()));
    if event.has_axis(Axis::Horizontal) {
        frame = frame.value(Axis::Horizontal, event.axis_value(Axis::Horizontal).unwrap_or(0.0));
        if let Some(discrete) = event.axis_value_discrete(Axis::Horizontal) {
            frame = frame.discrete(Axis::Horizontal, discrete as i32);
        }
    }
    if event.has_axis(Axis::Vertical) {
        frame = frame.value(Axis::Vertical, event.axis_value(Axis::Vertical).unwrap_or(0.0));
        if let Some(discrete) = event.axis_value_discrete(Axis::Vertical) {
            frame = frame.discrete(Axis::Vertical, discrete as i32);
        }
    }

    if let Some(source) = event.axis_source() {
        frame = frame.source(source.into()); // Convert libinput::AxisSource to smithay::input::pointer::AxisSource
    } else {
        // Default to Wheel if not specified, as per some Wayland expectations
        frame = frame.source(smithay::input::pointer::AxisSource::Wheel);
    }

    pointer_handle.axis(frame);
    Ok(())
}
