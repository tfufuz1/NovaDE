use smithay::{
    backend::input::{InputEvent as BackendInputEvent, TouchDownEvent, TouchUpEvent, TouchMotionEvent, TouchFrameEvent, TouchCancelEvent, LibinputInputBackend},
    input::{Seat, touch::{TouchHandle, TouchSlotId, TouchDownEvent as SmithayTouchDownEvent, TouchMotionEvent as SmithayTouchMotionEvent, TouchUpEvent as SmithayTouchUpEvent, GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent}, SeatHandler}, // Added SeatHandler
    reexports::{
        wayland_server::protocol::wl_surface::{WlSurface, Weak},
        input::event::touch::{TouchDownEventTrait, TouchUpEventTrait, TouchMotionEventTrait, TouchFrameEventTrait, TouchCancelEventTrait},
    },
    utils::{Logical, Point, Serial, Size}, // Added Size
    output::Output, // For output geometry
    desktop::Window, // For finding window by surface
};
use crate::{
    compositor::core::state::DesktopState,
    input::{
        errors::InputError,
        pointer::find_surface_and_coords_at_global_point, // Reuse from pointer module
        keyboard::set_keyboard_focus, // For touch-to-focus
    },
};
use std::collections::HashMap;


pub fn handle_touch_down_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: TouchDownEvent<LibinputInputBackend>,
) -> Result<(), InputError> {
    let touch_handle = seat.get_touch().ok_or_else(|| InputError::TouchHandleNotFound(seat.name().to_string()))?;
    let slot = event.slot().ok_or_else(|| InputError::InternalError("Touch down event without Slot ID".into()))?;

    let output_geometry = desktop_state.space.outputs()
        .find_map(|o| desktop_state.space.output_geometry(o));

    let output_size = output_geometry.map_or(Size::from((1920, 1080)), |geo| geo.size);
    let output_loc = output_geometry.map_or(Point::from((0,0)), |geo| geo.loc);

    let global_x = event.x_transformed(output_size.w as u32) + output_loc.x as f64;
    let global_y = event.y_transformed(output_size.h as u32) + output_loc.y as f64;
    let global_pos: Point<f64, Logical> = (global_x, global_y).into();


    let (focused_surface_option, surface_local_coords) =
        find_surface_and_coords_at_global_point(desktop_state, global_pos);

    if let Some(surface) = focused_surface_option {
        desktop_state.touch_focus_per_slot.insert(slot, surface.downgrade());

        touch_handle.down(
            event.serial(),
            event.time(),
            slot,
            surface_local_coords,
            &surface,
            Some(tracing::Span::current()),
        );

        if let Err(e) = set_keyboard_focus(desktop_state, seat.name(), Some(&surface), event.serial()) {
            tracing::warn!("Failed to set keyboard focus on touch down for seat {}: {:?}", seat.name(), e);
        }
        tracing::debug!("Touch down on surface {:?} (slot {:?}), global_pos {:?}, local_coords {:?}", surface.id(), slot, global_pos, surface_local_coords);

    } else {
        desktop_state.touch_focus_per_slot.remove(&slot);
        tracing::debug!("Touch down (slot {:?}) at {:?} found no surface.", slot, global_pos);
    }
    Ok(())
}

pub fn handle_touch_up_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: TouchUpEvent<LibinputInputBackend>,
) -> Result<(), InputError> {
    let touch_handle = seat.get_touch().ok_or_else(|| InputError::TouchHandleNotFound(seat.name().to_string()))?;
    let slot = event.slot().ok_or_else(|| InputError::InternalError("Touch up event without Slot ID".into()))?;

    touch_handle.up(event.serial(), event.time(), slot, Some(tracing::Span::current()));

    if desktop_state.touch_focus_per_slot.remove(&slot).is_some() {
        tracing::debug!("Touch up for slot {:?}, focus removed from DesktopState.", slot);
    } else {
        tracing::debug!("Touch up for slot {:?}, no prior focus found in DesktopState for this slot.", slot);
    }
    Ok(())
}

pub fn handle_touch_motion_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    event: TouchMotionEvent<LibinputInputBackend>,
) -> Result<(), InputError> {
    let touch_handle = seat.get_touch().ok_or_else(|| InputError::TouchHandleNotFound(seat.name().to_string()))?;
    let slot = event.slot().ok_or_else(|| InputError::InternalError("Touch motion event without Slot ID".into()))?;

    if let Some(focused_surface_weak) = desktop_state.touch_focus_per_slot.get(&slot) {
        if let Some(focused_surface) = focused_surface_weak.upgrade() {
            let output_geometry = desktop_state.space.outputs()
                .find_map(|o| desktop_state.space.output_geometry(o));

            let output_size = output_geometry.map_or(Size::from((1920, 1080)), |geo| geo.size);
            let output_loc = output_geometry.map_or(Point::from((0,0)), |geo| geo.loc);

            let global_x = event.x_transformed(output_size.w as u32) + output_loc.x as f64;
            let global_y = event.y_transformed(output_size.h as u32) + output_loc.y as f64;
            let global_pos: Point<f64, Logical> = (global_x, global_y).into();

            let mut surface_local_coords = global_pos; // Fallback

            // Try to find the window corresponding to the focused_surface to get its geometry
            let window_geometry = desktop_state.space.elements()
                .find(|w| w.wl_surface().as_ref() == Some(&focused_surface))
                .map(|w| w.geometry());

            if let Some(geo) = window_geometry {
                 surface_local_coords = (global_pos.x - geo.loc.x as f64, global_pos.y - geo.loc.y as f64).into();
            } else {
                // This can happen if the surface is a subsurface or some other element not directly in space.elements()
                // Or if the window got unmapped between touch_down and touch_motion.
                // In such a case, sending motion events might not be meaningful or could be misdirected.
                // For now, we'll log and potentially not send the motion, or send with global_pos as local (which is likely wrong).
                tracing::warn!(
                    "Could not find ManagedWindow for WlSurface {:?} during touch motion for slot {:?}. Using global coordinates as local.",
                    focused_surface.id(), slot
                );
                // If we can't determine local coords, it might be better to not send the motion,
                // or clear focus for the slot. For now, we proceed with potentially incorrect coords.
            }

            touch_handle.motion(
                event.serial(),
                event.time(),
                slot,
                surface_local_coords, // This needs to be surface-local for the focused_surface
                Some(tracing::Span::current()),
            );
            tracing::trace!("Touch motion on surface {:?} (slot {:?}), global {:?}, local_coords {:?}", focused_surface.id(), slot, global_pos, surface_local_coords);
        } else {
            desktop_state.touch_focus_per_slot.remove(&slot);
            tracing::debug!("Touch motion for slot {:?}, but focused surface was dropped. Focus removed.", slot);
        }
    } else {
        tracing::trace!("Touch motion for slot {:?}, but no surface is focused in DesktopState for this slot.", slot);
    }
    Ok(())
}

pub fn handle_touch_frame_event(
    _desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    _event: TouchFrameEvent<LibinputInputBackend>,
) -> Result<(), InputError> {
    let touch_handle = seat.get_touch().ok_or_else(|| InputError::TouchHandleNotFound(seat.name().to_string()))?;
    touch_handle.frame(Some(tracing::Span::current()));
    tracing::trace!("Touch frame sent for seat {}.", seat.name());
    Ok(())
}

pub fn handle_touch_cancel_event(
    desktop_state: &mut DesktopState,
    seat: &Seat<DesktopState>,
    _event: TouchCancelEvent<LibinputInputBackend>,
) -> Result<(), InputError> {
    let touch_handle = seat.get_touch().ok_or_else(|| InputError::TouchHandleNotFound(seat.name().to_string()))?;
    touch_handle.cancel(Some(tracing::Span::current()));

    // Smithay's TouchHandle::cancel will notify clients to end all active touch points.
    // We should also clear our internal state for all slots.
    desktop_state.touch_focus_per_slot.clear();
    tracing::debug!("Touch cancel event for seat {}, all touch focus slots cleared from DesktopState.", seat.name());
    Ok(())
}
