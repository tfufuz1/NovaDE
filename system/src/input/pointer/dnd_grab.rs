// system/src/input/pointer/dnd_grab.rs
use crate::compositor::core::state::DesktopState;
use crate::input::pointer::pointer_event_translator::find_surface_and_coords_at_global_point;
use smithay::desktop::WindowSurfaceType;
use smithay::input::pointer::{
    AxisFrame, ButtonEvent, Focus, GrabStartData, MotionEvent, PointerGrab, PointerInnerHandle, RelativeMotionEvent
};
use smithay::input::Seat;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{DisplayHandle, Serial};
use smithay::utils::{Logical, Point};
use smithay::wayland::seat::SeatData; // For SeatData access in PointerInnerHandle
use smithay::wayland::SerialCounter; // For generating serials for set_cursor

// Data to start the DnD grab
#[derive(Debug, Clone)]
pub struct DndGrabStart {
    // The WlSurface that is being dragged (the "icon" or representation of the dragged data)
    // This might be None if the DnD operation doesn't provide a specific drag surface.
    pub drag_surface: Option<WlSurface>,
    // The serial of the event that started the drag (e.g., a touch down or button press)
    pub start_serial: Serial,
    // Potentially other data from the DataDevice source, like the current offer
}

pub struct DndPointerGrab {
    pub start_data: GrabStartData<DesktopState>, // Data provided by Smithay when grab starts
    pub dnd_start_info: DndGrabStart,          // Our custom data for this grab
    pub current_icon_name: Option<String>,       // To track the current dnd cursor icon name
}

impl DndPointerGrab {
    // Helper to set the cursor, potentially using a themed cursor
    fn set_dnd_cursor(&mut self, handle: &mut PointerInnerHandle<'_, DesktopState>, icon_name: &str) {
        if self.current_icon_name.as_deref() != Some(icon_name) {
            // It's better to use Seat::set_cursor via the PointerInnerHandle's seat data
            // This requires DesktopState to have a way to get the current serial for set_cursor
            // For now, we'll just log, as direct Seat::set_cursor is tricky from PointerGrab.
            // The cursor update should ideally be driven by the DataDeviceHandler reacting to
            // potential drop targets.
            //
            // A more complete way:
            // let seat_data = handle.seat_data(); // This would be &DesktopState
            // let seat = &seat_data.seat; // Assuming DesktopState.seat is the one
            // seat.set_cursor(seat_data, handle.current_serial(), Some(cursor_surface_for_icon), (hotspot_x, hotspot_y));
            // This is complex because we need a WlSurface for the icon.
            //
            // Simpler for now: update current_icon_name, and let DesktopState's main loop
            // or SeatHandler::cursor_image handle the actual theme loading if current_cursor_status is updated.
            // This grab handler would update a field in DesktopState like `dnd_active_icon_name: Option<String>`.
            // Then, the main rendering or cursor update path would use this.
            //
            // For this subtask, we'll focus on the logic within the grab.
            // The actual cursor change will be a TODO or a simplified log.
            tracing::info!("DnD Grab: Requesting cursor change to '{}'", icon_name);
            self.current_icon_name = Some(icon_name.to_string());

            // To actually change the cursor *now*, we'd need to update DesktopState.current_cursor_status
            // and tell the SeatHandler to re-evaluate, or directly call methods that lead to
            // Seat::set_cursor being called with a themed surface.
            // This is usually done by the DataDeviceHandler calling PointerHandle::set_cursor.
            // Let's assume the DataDeviceHandler will call pointer_handle.set_cursor with a Named cursor.
        }
    }
}

impl PointerGrab<DesktopState> for DndPointerGrab {
    fn motion(
        &mut self,
        data: &mut DesktopState, // This is &mut DesktopState (the SeatData)
        handle: &mut PointerInnerHandle<'_, DesktopState>,
        _focus: Option<(Focus<DesktopState>, Point<f64, Logical>)>, // Smithay provides current focus
        event: &MotionEvent,
    ) {
        // Update global pointer location (already done by the main pointer motion handler before grab starts)
        // Here, data.pointer_location is the most up-to-date.
        let pointer_location = data.pointer_location; // Use the already updated pointer_location

        let (target_surface_opt, _surface_local_coords) =
            find_surface_and_coords_at_global_point(data, pointer_location);

        // TODO: Interact with DataDeviceHandler in DesktopState:
        // 1. data.data_device_state.motion(target_surface_opt.as_ref(), event.serial, pointer_location, event.time);
        //    This would internally send wl_data_device.enter/leave/motion.
        // 2. Based on the DataDeviceHandler's assessment of the target (can it accept the drop type?):
        //    data.data_device_state.update_drag_cursor(); // This method would call pointer_handle.set_cursor()
        //                                                 // with an appropriate themed cursor name like "grabbing", "dnd-ok", "dnd-invalid"
        
        // Placeholder for cursor update logic:
        if target_surface_opt.is_some() {
             // self.set_dnd_cursor(handle, "dnd-ok"); // Example if target is valid
             tracing::debug!("DnD motion over a surface: {:?}. Serial: {}", target_surface_opt.as_ref().map(|s|s.id()), event.serial);
        } else {
            // self.set_dnd_cursor(handle, "dnd-grabbing"); // Example if no specific target or invalid
            tracing::debug!("DnD motion over empty space. Serial: {}", event.serial);
        }
        
        // The dragged surface/icon (this.dnd_start_info.drag_surface) should also be moved
        // to follow the cursor. This is typically handled by the compositor's rendering logic
        // for drag icons, not directly in the pointer grab motion for *other* surfaces.
        // Smithay's `DataDeviceHandler::start_drag_grab` often handles creating a special surface for the icon.
    }

    fn relative_motion(
        &mut self,
        data: &mut DesktopState,
        handle: &mut PointerInnerHandle<'_, DesktopState>,
        _focus: Option<(Focus<DesktopState>, Point<f64, Logical>)>,
        event: &RelativeMotionEvent,
    ) {
        // Similar to motion, but using relative delta if needed by DataDeviceHandler
        let pointer_location = data.pointer_location;
         let (target_surface_opt, _surface_local_coords) =
            find_surface_and_coords_at_global_point(data, pointer_location);

        tracing::debug!("DnD relative_motion over {:?}. Serial: {}", target_surface_opt.as_ref().map(|s|s.id()), event.serial);
        // TODO: data.data_device_state.motion(...)
    }


    fn button(
        &mut self,
        data: &mut DesktopState, // &mut DesktopState
        handle: &mut PointerInnerHandle<'_, DesktopState>,
        event: &ButtonEvent,
    ) {
        // Check if the button that initiated the DnD is released
        // Typically, left button release signifies a drop.
        // The specific button might be stored in DndGrabStart or assumed.
        if event.button_state() == smithay::input::pointer::ButtonState::Released {
            tracing::info!("DnD drop initiated by button release. Button: {}, Serial: {}", event.button_code(), event.serial());

            // TODO: Interact with DataDeviceHandler in DesktopState:
            // 1. data.data_device_state.drop(event.serial, event.time);
            //    This would internally send wl_data_device.drop to the current target.

            // End the grab
            handle.unset_grab(data, event.serial(), event.time(), true); // `true` to send button release to original client if needed
        }
    }

    fn axis(
        &mut self,
        _data: &mut DesktopState,
        _handle: &mut PointerInnerHandle<'_, DesktopState>,
        _details: AxisFrame,
        _focus: Option<(Focus<DesktopState>, Point<f64, Logical>)>,
    ) {
        // Usually, axis events (scrolling) are ignored during a DnD grab.
    }

    fn start_data(&self) -> &GrabStartData<DesktopState> {
        &self.start_data
    }
    
    // Add new methods from PointerGrab trait if smithay version requires them
    fn unset(&mut self, data: &mut DesktopState, serial: Serial, time: u32) {
        // Called when the grab is programmatically unset or cancelled.
        tracing::info!("DnD Grab unset. Serial: {}, Time: {}", serial, time);
        // TODO: data.data_device_state.drag_cancelled(serial, time);
        // Restore default cursor or let SeatHandler do it.
         data.current_cursor_status.lock().unwrap() = smithay::input::pointer::CursorImageStatus::Default;
    }
}
