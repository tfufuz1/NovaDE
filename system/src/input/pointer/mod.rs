use crate::{
    compositor::core::state::DesktopState,
    input::keyboard::set_keyboard_focus, // For click-to-focus
    compositor::core::surface_management::get_surface_data, // To access SurfaceData
};
use smithay::{
    backend::input::{
        ButtonState, InputEvent, LibinputInputBackend, PointerAxisEvent, PointerButtonEvent,
        PointerMotionEvent, PointerMotionAbsoluteEvent, Axis, AxisSource, // Added Axis, AxisSource
    },
    input::{pointer::PointerHandle, Seat},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point, SERIAL_COUNTER as WSERIAL_COUNTER}, // Added SERIAL_COUNTER
};

/// Finds the topmost surface and its local coordinates at a given global point.
///
/// Iterates through elements in `desktop_state.space` under the `global_pos`.
/// For each `ManagedWindow`, it checks its input region.
///
/// # Returns
///
/// A tuple `(Option<WlSurface>, Point<f64, Logical>)`.
/// The `WlSurface` is the found surface, and the `Point` is `global_pos` transformed
/// into the local coordinate system of that surface. If no surface is found or
/// the point is outside the input region, `(None, global_pos)` might be returned,
/// or `(None, (0.0, 0.0))` if local coordinates are meaningless without a surface.
/// Smithay's pointer events expect local coordinates relative to the entered surface.
pub fn find_surface_and_coords_at_global_point(
    desktop_state: &DesktopState,
    global_pos: Point<f64, Logical>,
) -> (Option<WlSurface>, Point<f64, Logical>) {
    for element_arc in desktop_state.space.elements_under(global_pos) {
        // element_arc is Arc<ManagedWindow>
        let managed_window = element_arc.as_ref(); // Get &ManagedWindow from Arc
        let wl_surface = managed_window.wl_surface().clone(); // Clone WlSurface for return

        // Calculate surface-local coordinates. Space::elements_under gives elements whose
        // bounding box contains global_pos. The geometry of ManagedWindow (from Window trait)
        // is usually its size, and Space places it at a location.
        // The `global_pos` needs to be made relative to the window's origin in the Space.
        let window_loc = match desktop_state.space.window_location(managed_window) {
            Some(loc) => loc.to_f64(), // Convert Point<i32, Logical> to Point<f64, Logical>
            None => continue, // Window not found in space, should not happen if elements_under returned it
        };
        let surface_local_pos = global_pos - window_loc;

        // Check input region
        let surface_data = get_surface_data(&wl_surface);
        let input_region_local_opt = surface_data.input_region.lock().unwrap().clone();

        if let Some(input_region_local) = input_region_local_opt {
            // Input region is in surface-local logical coordinates.
            if input_region_local.to_f64().contains(surface_local_pos) {
                tracing::trace!("Pointer over surface {:?} at local coords {:?}, within input region.", wl_surface.id(), surface_local_pos);
                return (Some(wl_surface), surface_local_pos);
            }
        } else {
            // No specific input region, use entire surface bounds.
            // The surface_local_pos is already relative to the surface's origin.
            // The geometry() of ManagedWindow is its size.
            let surface_size = managed_window.geometry().size.to_f64();
            let surface_bounds = Rectangle::from_loc_and_size(Point::from((0.0, 0.0)), surface_size);
            if surface_bounds.contains(surface_local_pos) {
                tracing::trace!("Pointer over surface {:?} at local coords {:? (no input region, using bounds)}.", wl_surface.id(), surface_local_pos);
                return (Some(wl_surface), surface_local_pos);
            }
        }
    }
    // No surface found at this point, or point is outside relevant input regions.
    tracing::trace!("No surface found under pointer at global coords {:?}.", global_pos);
    (None, global_pos) // Return None and original global_pos, or (0,0) if local is always needed
}

/// Handles relative pointer motion events.
pub fn handle_pointer_motion_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: PointerMotionEvent<LibinputInputBackend>,
    _seat_name: &str, // seat_name can be used for logging or multi-seat later
) {
    let pointer_handle = match seat.get_pointer() {
        Some(h) => h,
        None => return, // No pointer capability on this seat
    };

    // Update global pointer location
    desktop_state.pointer_location += event.delta();

    // Clamp to output boundaries (simplified: assume single output covering space)
    // Get primary output geometry (or some sensible default if no outputs)
    let output_geometry = desktop_state.space.outputs()
        .next() // Get the first output
        .map(|o| desktop_state.space.output_geometry(o).unwrap_or_default())
        .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0), (1920, 1080))); // Fallback

    desktop_state.pointer_location.x = desktop_state.pointer_location.x.clamp(output_geometry.loc.x as f64, (output_geometry.loc.x + output_geometry.size.w) as f64);
    desktop_state.pointer_location.y = desktop_state.pointer_location.y.clamp(output_geometry.loc.y as f64, (output_geometry.loc.y + output_geometry.size.h) as f64);
    
    let (focused_surface_opt, surface_local_coords) =
        find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);

    pointer_handle.motion(
        event.time(),
        focused_surface_opt.as_ref(), // Option<&WlSurface>
        event.serial(),
        &surface_local_coords, // Local coordinates relative to focused surface
        Some(tracing::Span::current()),
    );
    
    // Update active_input_surface (general focus, might be refined for pointer-specific focus later)
    // desktop_state.active_input_surface = focused_surface_opt.map(|s| s.downgrade());
    // Smithay's SeatHandler::focus_changed handles active_input_surface for keyboard.
    // For pointer, the `pointer_handle.motion` call updates its internal focus state
    // and sends wl_pointer.enter/leave. We don't need to manage active_input_surface here
    // unless our domain logic specifically needs a pointer-only focus tracker separate from keyboard.

    tracing::trace!(
        "Pointer motion: global_pos={:?}, local_pos={:?}, focused_surface={:?}",
        desktop_state.pointer_location,
        surface_local_coords,
        focused_surface_opt.as_ref().map(|s| s.id())
    );
}

/// Handles absolute pointer motion events.
pub fn handle_pointer_motion_absolute_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: PointerMotionAbsoluteEvent<LibinputInputBackend>,
    _seat_name: &str,
) {
    let pointer_handle = match seat.get_pointer() {
        Some(h) => h,
        None => return,
    };

    // Determine output dimensions (simplified: assume primary output or fixed size)
    // A more robust solution would get dimensions from the event's device or associated output.
    let output_geometry = desktop_state.space.outputs()
        .next()
        .map(|o| desktop_state.space.output_geometry(o).unwrap_or_default())
        .unwrap_or_else(|| Rectangle::from_loc_and_size((0,0), (1920, 1080))); // Fallback

    desktop_state.pointer_location = Point::from((
        event.x_transformed(output_geometry.size.w as u32),
        event.y_transformed(output_geometry.size.h as u32),
    )) + output_geometry.loc.to_f64(); // Add output origin if space is multi-output

    // Clamp to output boundaries (already transformed to be within, but good practice)
    desktop_state.pointer_location.x = desktop_state.pointer_location.x.clamp(output_geometry.loc.x as f64, (output_geometry.loc.x + output_geometry.size.w) as f64);
    desktop_state.pointer_location.y = desktop_state.pointer_location.y.clamp(output_geometry.loc.y as f64, (output_geometry.loc.y + output_geometry.size.h) as f64);

    let (focused_surface_opt, surface_local_coords) =
        find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);

    pointer_handle.motion(
        event.time(),
        focused_surface_opt.as_ref(),
        event.serial(),
        &surface_local_coords,
        Some(tracing::Span::current()),
    );
    tracing::trace!(
        "Pointer absolute motion: global_pos={:?}, local_pos={:?}, focused_surface={:?}",
        desktop_state.pointer_location,
        surface_local_coords,
        focused_surface_opt.as_ref().map(|s| s.id())
    );
}

/// Handles pointer button events.
pub fn handle_pointer_button_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: PointerButtonEvent<LibinputInputBackend>,
    seat_name: &str,
) {
    let pointer_handle = match seat.get_pointer() {
        Some(h) => h,
        None => return,
    };

    pointer_handle.button(
        event.serial(),
        event.time(),
        event.button(), // u32 button code (e.g., 0x110 for BTN_LEFT)
        event.state(),  // smithay::backend::input::ButtonState
        Some(tracing::Span::current()),
    );
    tracing::debug!(
        "Pointer button: button={:#x}, state={:?}, serial={:?}, time={}",
        event.button(), event.state(), event.serial(), event.time()
    );

    if event.state() == ButtonState::Pressed {
        // Click-to-focus: Set keyboard focus to the surface under the pointer.
        let (surface_to_focus_opt, _) =
            find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);

        if let Some(surface_to_focus) = surface_to_focus_opt {
            // Check if the currently focused keyboard surface is different
            let keyboard = seat.get_keyboard();
            let current_keyboard_focus = keyboard.as_ref().and_then(|k| k.current_focus());
            
            if current_keyboard_focus.as_ref() != Some(&surface_to_focus) {
                 if let Err(e) = set_keyboard_focus(desktop_state, seat_name, Some(&surface_to_focus), event.serial()) {
                    tracing::warn!("Failed to set keyboard focus on pointer click: {}", e);
                } else {
                    tracing::info!("Keyboard focus set to surface {:?} due to pointer click.", surface_to_focus.id());
                }
            }
            // TODO: Placeholder for future: initiate window move/resize grabs.
            // This would involve checking if the click is on a window decoration,
            // consulting window policy, and then potentially starting a grab via
            // `pointer_handle.grab(...)` with a custom grab handler.
            // Example: if is_on_title_bar(&surface_to_focus, local_coords) { start_move_grab(...) }
        } else {
            // Clicked on empty space, clear keyboard focus.
            if let Err(e) = set_keyboard_focus(desktop_state, seat_name, None, event.serial()) {
                tracing::warn!("Failed to clear keyboard focus on pointer click in empty space: {}", e);
            } else {
                tracing::info!("Keyboard focus cleared due to pointer click in empty space.");
            }
        }
    }
}

/// Handles pointer axis (scroll) events.
pub fn handle_pointer_axis_event(
    _desktop_state: &mut DesktopState, // Not strictly needed if not changing global state here
    seat: &Seat<DesktopState>,
    event: PointerAxisEvent<LibinputInputBackend>,
    _seat_name: &str,
) {
    let pointer_handle = match seat.get_pointer() {
        Some(h) => h,
        None => return,
    };

    let source = match event.axis_source() {
        Some(s) => match s {
            smithay::backend::input::AxisSource::Wheel => AxisSource::Wheel,
            smithay::backend::input::AxisSource::Finger => AxisSource::Finger,
            smithay::backend::input::AxisSource::Continuous => AxisSource::Continuous,
            smithay::backend::input::AxisSource::WheelTilt => AxisSource::WheelTilt, // Smithay 0.3+
        },
        None => AxisSource::Wheel, // Default or infer if possible
    };

    let horizontal_amount = event.amount(Axis::Horizontal).unwrap_or(0.0);
    let vertical_amount = event.amount(Axis::Vertical).unwrap_or(0.0);
    let horizontal_amount_discrete = event.amount_discrete(Axis::Horizontal).unwrap_or(0.0);
    let vertical_amount_discrete = event.amount_discrete(Axis::Vertical).unwrap_or(0.0);


    if vertical_amount != 0.0 {
        pointer_handle.axis(
            event.time(),
            Axis::Vertical,
            vertical_amount.abs(), // Smithay expects positive amount
            if vertical_amount < 0.0 { -1 } else { 1 }, // And direction as i32
            source,
            event.serial(),
            Some(tracing::Span::current()),
        );
    }
    if horizontal_amount != 0.0 {
         pointer_handle.axis(
            event.time(),
            Axis::Horizontal,
            horizontal_amount.abs(),
            if horizontal_amount < 0.0 { -1 } else { 1 },
            source,
            event.serial(),
            Some(tracing::Span::current()),
        );
    }
    
    // Handle discrete scroll (e.g. mouse wheel clicks) if different from continuous
    // Smithay's PointerHandle::axis takes a single amount and direction.
    // The distinction between continuous and discrete might need to be handled by how
    // the `amount` is interpreted or if a separate `axis_discrete` method exists.
    // Smithay 0.3 PointerHandle::axis takes `value: f64` and `direction: i32`.
    // It does not have a separate discrete value. The backend (libinput) provides both.
    // We should send the one that is non-zero. If both are non-zero, libinput docs suggest
    // preferring discrete if available for that event.
    // For now, sending the continuous amount. Discrete might be used if amount is zero.
    // Or, one could send two separate axis events if both have values, though that's uncommon.

    if vertical_amount_discrete != 0.0 && vertical_amount == 0.0 { // Only send discrete if continuous was zero
         pointer_handle.axis(
            event.time(),
            Axis::Vertical,
            vertical_amount_discrete.abs() * 15.0, // Multiply discrete by a factor if needed for sensitivity
            if vertical_amount_discrete < 0.0 { -1 } else { 1 },
            source, // Source remains the same
            event.serial(),
            Some(tracing::Span::current()),
        );
    }
    if horizontal_amount_discrete != 0.0 && horizontal_amount == 0.0 {
        pointer_handle.axis(
            event.time(),
            Axis::Horizontal,
            horizontal_amount_discrete.abs() * 15.0,
            if horizontal_amount_discrete < 0.0 { -1 } else { 1 },
            source,
            event.serial(),
            Some(tracing::Span::current()),
        );
    }

    tracing::trace!(
        "Pointer axis: h_cont={:.2}, v_cont={:.2}, h_disc={:.2}, v_disc={:.2}, source={:?}",
        horizontal_amount, vertical_amount,
        horizontal_amount_discrete, vertical_amount_discrete,
        source
    );
}
