// system/src/input/touch/touch_event_translator.rs
use crate::compositor::core::state::DesktopState;
// use crate::input::errors::InputError; // Not directly used in these handlers
use crate::input::pointer::pointer_event_translator::find_surface_and_coords_at_global_point; // Re-use from pointer module

use smithay::backend::input::{
    TouchDownEvent, TouchUpEvent, TouchMotionEvent, TouchFrameEvent, TouchCancelEvent, LibinputInputBackend,
    TouchSlot
};
use smithay::input::{Seat, touch::TouchHandle};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle}; // Added Rectangle
use std::collections::HashMap; // For managing per-slot focus
use smithay::desktop::Window; // For w.is_mapped() and w.can_receive_focus()

pub fn handle_touch_down_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: TouchDownEvent<LibinputInputBackend>,
) {
    let touch_handle = match seat.get_touch() {
        Some(h) => h,
        None => return,
    };

    let slot = match event.slot() {
        Some(s) => s,
        None => {
            tracing::error!("Touch down event ohne Slot ID empfangen.");
            return;
        }
    };
    
    // Determine output size for coordinate transformation
    let output_geometry = desktop_state.space.outputs()
        .next()
        .map(|o| desktop_state.space.output_geometry(o).unwrap_or_else(|| Rectangle::from_loc_and_size(Point::from((0,0)), (1920, 1080).into())))
        .unwrap_or_else(|| Rectangle::from_loc_and_size(Point::from((0,0)), (1920, 1080).into())); 

    let global_pos: Point<f64, Logical> = event.position_transformed(output_geometry.size.w as u32, output_geometry.size.h as u32);

    let (surface_under_touch, surface_local_coords) =
        find_surface_and_coords_at_global_point(desktop_state, global_pos);

    if let Some(ref surface) = surface_under_touch {
        tracing::debug!("Touch down auf Surface {:?} an Position {:?}, Slot {:?}", surface.id(), surface_local_coords, slot);
        
        desktop_state.active_touch_targets.insert(slot, surface.downgrade());

        touch_handle.down(
            desktop_state, // Pass &mut DesktopState as SeatData
            surface,
            event.serial(),
            event.time(),
            slot,
            surface_local_coords,
            Some(tracing::Span::current()),
        );

        // Touch-to-focus
        let window_can_focus = desktop_state.space.window_for_surface(surface, smithay::desktop::WindowSurfaceType::TOPLEVEL)
            .map_or(false, |w| w.is_mapped() && w.can_receive_focus()); // Assuming ManagedWindow impls Window and has can_receive_focus

        if window_can_focus {
            match crate::input::keyboard::focus::set_keyboard_focus(desktop_state, seat.name(), Some(surface), event.serial()) {
                Ok(_) => tracing::debug!("Tastaturfokus auf Surface {:?} aufgrund von Touch gesetzt.", surface.id()),
                Err(e) => tracing::error!("Fehler beim Setzen des Tastaturfokus durch Touch: {:?}", e),
            }
        }
    } else {
        tracing::debug!("Touch down auf leerem Raum, kein Surface-Ziel für Slot {:?}.", slot);
        // If an existing touch point for this slot had a surface, it should be cleared.
        // However, a new "down" implies a new interaction. If a surface was previously targeted by this slot
        // and an "up" or "cancel" was missed, this new "down" effectively starts fresh for the slot.
        desktop_state.active_touch_targets.remove(&slot);
        // Smithay's TouchHandle::down requires a surface to send the event to a client.
        // If the interaction is not on a surface, we might not call `touch_handle.down()`.
        // Alternatively, if there's a global touch grab, it might receive events even without a surface.
        // For now, without a surface, we don't forward to `touch_handle.down()`.
    }
}

pub fn handle_touch_up_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: TouchUpEvent<LibinputInputBackend>,
) {
    let touch_handle = match seat.get_touch() {
        Some(h) => h,
        None => return,
    };
    
    let slot = match event.slot() {
        Some(s) => s,
        None => {
            tracing::error!("Touch up event ohne Slot ID empfangen.");
            return;
        }
    };

    tracing::debug!("Touch up für Slot {:?}", slot);

    desktop_state.active_touch_targets.remove(&slot);

    touch_handle.up(
        desktop_state,
        event.serial(),
        event.time(),
        slot,
        Some(tracing::Span::current()),
    );
}

pub fn handle_touch_motion_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: TouchMotionEvent<LibinputInputBackend>,
) {
    let touch_handle = match seat.get_touch() {
        Some(h) => h,
        None => return,
    };
    
    let slot = match event.slot() {
        Some(s) => s,
        None => {
            tracing::error!("Touch motion event ohne Slot ID empfangen.");
            return;
        }
    };

    if let Some(target_surface_weak) = desktop_state.active_touch_targets.get(&slot) {
        if let Some(target_surface) = target_surface_weak.upgrade() {
            let output_geometry = desktop_state.space.outputs()
                .next()
                .map(|o| desktop_state.space.output_geometry(o).unwrap_or_else(|| Rectangle::from_loc_and_size(Point::from((0,0)), (1920, 1080).into())))
                .unwrap_or_else(|| Rectangle::from_loc_and_size(Point::from((0,0)), (1920, 1080).into()));

            let global_pos: Point<f64, Logical> = event.position_transformed(output_geometry.size.w as u32, output_geometry.size.h as u32);

            let surface_local_coords = if let Some(window_arc) = desktop_state.space.elements().find(|elem| elem.wl_surface() == Some(&target_surface)) {
                if let Some(window_loc) = desktop_state.space.element_location(&*window_arc) {
                    global_pos - window_loc.to_f64()
                } else {
                    global_pos 
                }
            } else {
                global_pos
            };

            tracing::trace!("Touch motion auf Surface {:?} an lokalen Koord. {:?}, Slot {:?}", target_surface.id(), surface_local_coords, slot);

            touch_handle.motion(
                desktop_state,
                &target_surface,
                event.serial(),
                event.time(),
                slot,
                surface_local_coords,
                Some(tracing::Span::current()),
            );
        } else {
            desktop_state.active_touch_targets.remove(&slot);
            tracing::debug!("Touch motion für Slot {:?}, aber Ziel-Surface ist nicht mehr gültig.", slot);
        }
    } else {
        tracing::trace!("Touch motion für Slot {:?}, aber kein aktives Ziel-Surface gefunden.", slot);
    }
}

pub fn handle_touch_frame_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    _event: TouchFrameEvent<LibinputInputBackend>,
) {
    let touch_handle = match seat.get_touch() {
        Some(h) => h,
        None => return,
    };
    touch_handle.frame(desktop_state, Some(tracing::Span::current()));
}

pub fn handle_touch_cancel_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    _event: TouchCancelEvent<LibinputInputBackend>,
) {
    let touch_handle = match seat.get_touch() {
        Some(h) => h,
        None => return,
    };
    tracing::debug!("Touch cancel event.");
    desktop_state.active_touch_targets.clear();
    touch_handle.cancel(desktop_state, Some(tracing::Span::current()));
}
