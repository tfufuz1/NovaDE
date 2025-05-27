// system/src/input/tablet/event_translator.rs
use crate::compositor::core::state::DesktopState;
use smithay::backend::input::{
    LibinputInputBackend,
    tablet_tool::{
        TabletToolAxisEvent, TabletToolProximityEvent, TabletToolTipEvent, TabletToolButtonEvent,
        TabletToolProximityState as LibinputProximityState, TabletToolTipState as LibinputTipState,
        ButtonState as LibinputButtonState,
    }
};
use smithay::input::Seat;
use smithay::input::pointer::PointerHandle; // PointerHandle for cursor interaction
use smithay::input::tablet::{
    TabletHandle, TabletToolHandle, TabletPadHandle, // Smithay tablet handles
    ProximityState as SmithayProximityState, TipState as SmithayTipState, ButtonState as SmithayButtonState,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface; // For focus
use smithay::utils::{Logical, Point};
use crate::input::pointer::pointer_event_translator::find_surface_and_coords_at_global_point; // For focus

// Helper to get focused surface for tablet events, similar to pointer/touch
// Tablet events are often tied to where the pointer cursor is.
fn get_tablet_focus(desktop_state: &DesktopState) -> (Option<WlSurface>, Point<f64, Logical>) {
    find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location)
}

pub fn handle_tablet_tool_axis_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: TabletToolAxisEvent<LibinputInputBackend>,
    _seat_name: &str, // seat_name might be useful for logging or multi-seat setups
) {
    if let Some(tablet_handle) = seat.get_tablet() { // Get TabletHandle from Seat
        if let Some(tool_handle) = tablet_handle.get_tool(event.tool()) { // Get TabletToolHandle
            let (focus_surface, focus_local_coords) = get_tablet_focus(desktop_state);
            
            // Determine output size for coordinate transformation for position
            // This assumes the event.position_transformed needs output dimensions.
            // If it's already global, this might not be necessary or should be (0,0).
            let output_geometry = desktop_state.space.outputs()
                .next()
                .map(|o| desktop_state.space.output_geometry(o).unwrap_or_default())
                .unwrap_or_else(|| smithay::utils::Rectangle::from_loc_and_size((0,0), (1920, 1080)));

            let position = event.position_transformed(output_geometry.size.w as u32, output_geometry.size.h as u32);

            tool_handle.axis(
                desktop_state, // SeatData
                focus_surface.as_ref(),
                event.pressure_raw_value(), 
                event.tilt_raw_values(),   
                position, 
                event.wheel_delta_discrete(), 
                event.wheel_delta(),          
                event.serial(),
                event.time(),
                Some(tracing::Span::current())
            );
             // Update pointer location if the tablet tool also moves the cursor
            if event.is_pointer_motion_event() { 
                if let Some(pointer) = seat.get_pointer() {
                    // The position from TabletToolAxisEvent is typically already in the compositor's global coordinate space
                    // if the device is mapped to the whole screen.
                    // If it's relative to an output, further transformation might be needed.
                    // For now, assume `position` can be used as the new global pointer location.
                    desktop_state.pointer_location = position; 
                    
                    // We need new local coords for the pointer motion based on the potentially new global pointer_location
                    let (pointer_focus_surface, pointer_focus_local_coords) = 
                        find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);

                    pointer.motion(
                        desktop_state, 
                        pointer_focus_surface.as_ref(), 
                        event.serial(), 
                        desktop_state.pointer_location, 
                        pointer_focus_local_coords, 
                        event.time(), 
                        Some(tracing::Span::current())
                    );
                }
            }
        } else {
            tracing::warn!("Tablet tool not found for axis event: {:?}", event.tool());
        }
    } else {
        tracing::warn!("Tablet handle not found for seat during tablet tool axis event.");
    }
}

pub fn handle_tablet_tool_proximity_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: TabletToolProximityEvent<LibinputInputBackend>,
    _seat_name: &str,
) {
    if let Some(tablet_handle) = seat.get_tablet() {
        if let Some(tool_handle) = tablet_handle.get_tool(event.tool()) {
            let (focus_surface, _focus_local_coords) = get_tablet_focus(desktop_state);
            tool_handle.proximity(
                desktop_state,
                focus_surface.as_ref(),
                event.state().into(), 
                event.serial(),
                event.time(),
                Some(tracing::Span::current())
            );
        } else {
            tracing::warn!("Tablet tool not found for proximity event: {:?}", event.tool());
        }
    } else {
        tracing::warn!("Tablet handle not found for seat during tablet tool proximity event.");
    }
}

pub fn handle_tablet_tool_tip_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: TabletToolTipEvent<LibinputInputBackend>,
    _seat_name: &str,
) {
    if let Some(tablet_handle) = seat.get_tablet() {
        if let Some(tool_handle) = tablet_handle.get_tool(event.tool()) {
            let (focus_surface, _focus_local_coords) = get_tablet_focus(desktop_state);
            tool_handle.tip(
                desktop_state,
                focus_surface.as_ref(),
                event.state().into(), 
                event.serial(),
                event.time(),
                Some(tracing::Span::current())
            );
        } else {
            tracing::warn!("Tablet tool not found for tip event: {:?}", event.tool());
        }
    } else {
        tracing::warn!("Tablet handle not found for seat during tablet tool tip event.");
    }
}

pub fn handle_tablet_tool_button_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: TabletToolButtonEvent<LibinputInputBackend>,
    _seat_name: &str,
) {
    if let Some(tablet_handle) = seat.get_tablet() {
        if let Some(tool_handle) = tablet_handle.get_tool(event.tool()) {
            tool_handle.button(
                desktop_state,
                event.button(),
                event.button_state().into(), 
                event.serial(),
                event.time(),
                Some(tracing::Span::current())
            );
        } else {
            tracing::warn!("Tablet tool not found for button event: {:?}", event.tool());
        }
    } else {
        tracing::warn!("Tablet handle not found for seat during tablet tool button event.");
    }
}

// From impls for state conversion
impl From<LibinputProximityState> for SmithayProximityState {
    fn from(state: LibinputProximityState) -> Self {
        match state {
            LibinputProximityState::In => SmithayProximityState::In,
            LibinputProximityState::Out => SmithayProximityState::Out,
        }
    }
}

impl From<LibinputTipState> for SmithayTipState {
    fn from(state: LibinputTipState) -> Self {
        match state {
            LibinputTipState::Down => SmithayTipState::Down,
            LibinputTipState::Up => SmithayTipState::Up,
        }
    }
}

impl From<LibinputButtonState> for SmithayButtonState {
     fn from(state: LibinputButtonState) -> Self {
         match state {
             LibinputButtonState::Pressed => SmithayButtonState::Pressed,
             LibinputButtonState::Released => SmithayButtonState::Released,
         }
     }
}
