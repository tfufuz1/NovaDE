//! Represents a `wl_pointer` object and its associated state and events.

use crate::input::seat::SeatId;
use crate::surface::SurfaceId;
use novade_buffer_manager::ClientId; // Assuming ClientId is globally unique

// TODO: Define error types for pointer operations if necessary.

/// Represents a server-side `wl_pointer` resource.
///
/// Each client that binds to `wl_seat` and requests a pointer gets one of these.
/// It tracks client-specific pointer state like the cursor image.
#[derive(Debug, Clone)]
pub struct WlPointer {
    /// The Wayland object ID for this specific `wl_pointer` instance, unique per client.
    pub object_id: u32,
    /// The ID of the client that owns this `wl_pointer` resource.
    pub client_id: ClientId,
    /// The ID of the `wl_seat` this pointer belongs to.
    pub seat_id: SeatId,

    /// The surface currently set as the cursor for this pointer.
    /// `None` means the default system cursor or a hidden cursor.
    pub cursor_surface: Option<SurfaceId>,
    /// The x-coordinate of the cursor hotspot, relative to the top-left of the `cursor_surface`.
    pub cursor_hotspot_x: i32,
    /// The y-coordinate of the cursor hotspot, relative to the top-left of the `cursor_surface`.
    pub cursor_hotspot_y: i32,

    /// The serial number of the last `wl_pointer.enter` event sent to the client for this pointer.
    /// This is used to validate `set_cursor` requests.
    pub last_enter_serial: u32,
}

impl WlPointer {
    /// Creates a new `WlPointer` instance.
    pub fn new(object_id: u32, client_id: ClientId, seat_id: SeatId) -> Self {
        Self {
            object_id,
            client_id,
            seat_id,
            cursor_surface: None,
            cursor_hotspot_x: 0,
            cursor_hotspot_y: 0,
            last_enter_serial: 0, // Initialize with a sensible default, will be updated on enter.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::seat::{SeatCapability, SeatId, SeatState};
    use crate::surface::{Surface, SurfaceId, SurfaceRegistry, SurfaceRole};
    use novade_buffer_manager::ClientId; // Already used above, ensure it's in scope
    use std::sync::Arc;

    // Helper to create a default ClientId
    fn test_client_id() -> ClientId { ClientId::new(1) }
    fn test_client_id_2() -> ClientId { ClientId::new(2) }


    #[test]
    fn test_wl_pointer_new() {
        let pointer = WlPointer::new(1, test_client_id(), SeatId(0));
        assert_eq!(pointer.object_id, 1);
        assert_eq!(pointer.client_id, test_client_id());
        assert_eq!(pointer.seat_id, SeatId(0));
        assert!(pointer.cursor_surface.is_none());
        assert_eq!(pointer.cursor_hotspot_x, 0);
        assert_eq!(pointer.cursor_hotspot_y, 0);
        assert_eq!(pointer.last_enter_serial, 0);
    }

    #[test]
    fn test_handle_set_cursor_success() {
        let mut surface_registry = SurfaceRegistry::new();
        let client_a = test_client_id();

        // Create a surface to be used as a cursor
        // Surface::new() needs a client_id, assuming it's added from previous focus subtask plan
        let (cursor_surface_id, cursor_surface_arc) = surface_registry.register_new_surface_with_client(client_a);

        let seat_id = SeatId(0);
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(seat_id, "seat0".to_string(), SeatCapability::Pointer)));

        let pointer_object_id = 101;
        let pointer_arc = Arc::new(Mutex::new(WlPointer::new(pointer_object_id, client_a, seat_id)));

        // Simulate pointer enter
        pointer_arc.lock().unwrap().last_enter_serial = 12345;

        let result = handle_set_cursor(
            pointer_arc.clone(),
            12345, // Valid serial
            Some(cursor_surface_id),
            5, 10,
            &surface_registry,
            seat_state_arc.clone(),
        );
        assert!(result.is_ok());

        let pointer_locked = pointer_arc.lock().unwrap();
        assert_eq!(pointer_locked.cursor_surface, Some(cursor_surface_id));
        assert_eq!(pointer_locked.cursor_hotspot_x, 5);
        assert_eq!(pointer_locked.cursor_hotspot_y, 10);

        let cursor_surface_locked = cursor_surface_arc.lock().unwrap();
        assert!(matches!(cursor_surface_locked.role, Some(SurfaceRole::Cursor)));
    }

    #[test]
    fn test_handle_set_cursor_to_none() {
        let mut surface_registry = SurfaceRegistry::new();
        let client_a = test_client_id();
        let (cursor_surface_id, _cursor_surface_arc) = surface_registry.register_new_surface_with_client(client_a);

        let seat_id = SeatId(0);
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(seat_id, "seat0".to_string(), SeatCapability::Pointer)));

        let pointer_object_id = 102;
        let pointer_arc = Arc::new(Mutex::new(WlPointer::new(pointer_object_id, client_a, seat_id)));
        pointer_arc.lock().unwrap().last_enter_serial = 123;
        pointer_arc.lock().unwrap().cursor_surface = Some(cursor_surface_id); // Pre-set a cursor

        let result = handle_set_cursor(
            pointer_arc.clone(),
            123, // Valid serial for hiding too, or can be ignored by some logic
            None, // Set to None
            0, 0,
            &surface_registry,
            seat_state_arc.clone(),
        );
        assert!(result.is_ok());
        assert!(pointer_arc.lock().unwrap().cursor_surface.is_none());
    }

    #[test]
    fn test_handle_set_cursor_invalid_serial() {
        let mut surface_registry = SurfaceRegistry::new();
        let client_a = test_client_id();
        let (cursor_surface_id, _cursor_surface_arc) = surface_registry.register_new_surface_with_client(client_a);

        let seat_id = SeatId(0);
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(seat_id, "seat0".to_string(), SeatCapability::Pointer)));

        let pointer_object_id = 103;
        let pointer_arc = Arc::new(Mutex::new(WlPointer::new(pointer_object_id, client_a, seat_id)));
        pointer_arc.lock().unwrap().last_enter_serial = 100; // Last enter was 100

        let result = handle_set_cursor(
            pointer_arc.clone(),
            101, // Mismatched serial
            Some(cursor_surface_id),
            0, 0,
            &surface_registry,
            seat_state_arc.clone(),
        );
        // Request should be ignored (Ok, but no change)
        assert!(result.is_ok());
        assert!(pointer_arc.lock().unwrap().cursor_surface.is_none()); // Should not have been set
    }

    #[test]
    fn test_handle_set_cursor_surface_not_found() {
        let surface_registry = SurfaceRegistry::new(); // Empty registry
        let client_a = test_client_id();
        let seat_id = SeatId(0);
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(seat_id, "seat0".to_string(), SeatCapability::Pointer)));
        let pointer_arc = Arc::new(Mutex::new(WlPointer::new(104, client_a, seat_id)));
        pointer_arc.lock().unwrap().last_enter_serial = 1;

        let result = handle_set_cursor(
            pointer_arc.clone(),
            1, Some(SurfaceId::new_unique()), // Non-existent SurfaceId
            0,0, &surface_registry, seat_state_arc
        );
        assert_eq!(result, Err(SetCursorError::SurfaceNotFound));
    }

    #[test]
    fn test_handle_set_cursor_role_conflict() {
        let mut surface_registry = SurfaceRegistry::new();
        let client_a = test_client_id();
        let (surface_id, surface_arc) = surface_registry.register_new_surface_with_client(client_a);

        // Give the surface a conflicting role
        surface_arc.lock().unwrap().role = Some(SurfaceRole::Toplevel);

        let seat_id = SeatId(0);
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(seat_id, "seat0".to_string(), SeatCapability::Pointer)));
        let pointer_arc = Arc::new(Mutex::new(WlPointer::new(105, client_a, seat_id)));
        pointer_arc.lock().unwrap().last_enter_serial = 1;

        let result = handle_set_cursor(
            pointer_arc.clone(),
            1, Some(surface_id), 0,0, &surface_registry, seat_state_arc
        );
        assert_eq!(result, Err(SetCursorError::RoleConflict));
    }


    #[test]
    fn test_handle_pointer_release() {
        let client_a = test_client_id();
        let seat_id = SeatId(0);
        let pointer_object_id = 201;
        let pointer_arc = Arc::new(Mutex::new(WlPointer::new(pointer_object_id, client_a, seat_id)));

        let mut global_pointers_map: HashMap<u32, Arc<Mutex<WlPointer>>> = HashMap::new();
        global_pointers_map.insert(pointer_object_id, pointer_arc.clone());

        assert!(global_pointers_map.contains_key(&pointer_object_id));
        handle_release(pointer_arc, &mut global_pointers_map);
        assert!(!global_pointers_map.contains_key(&pointer_object_id));
    }
}

// --- Event Data Structures (Conceptual for now) ---

/// Data for the `wl_pointer.enter` event.
#[derive(Debug, Clone, Copy)]
pub struct WlPointerEnterEvent {
    /// Serial number of the enter event.
    pub serial: u32,
    /// The `SurfaceId` of the surface that was entered.
    pub surface_id: SurfaceId,
    /// Surface-local x-coordinate of the pointer when entering.
    pub surface_x: f64, // Using f64 for wl_fixed_t representation
    /// Surface-local y-coordinate of the pointer when entering.
    pub surface_y: f64,
}

/// Data for the `wl_pointer.leave` event.
#[derive(Debug, Clone, Copy)]
pub struct WlPointerLeaveEvent {
    /// Serial number of the leave event.
    pub serial: u32,
    /// The `SurfaceId` of the surface that was left.
    pub surface_id: SurfaceId,
}

/// Data for the `wl_pointer.motion` event.
#[derive(Debug, Clone, Copy)]
pub struct WlPointerMotionEvent {
    /// Timestamp of the event, with millisecond granularity.
    pub time_ms: u32,
    /// Surface-local x-coordinate of the pointer.
    pub surface_x: f64,
    /// Surface-local y-coordinate of the pointer.
    pub surface_y: f64,
}

/// Data for the `wl_pointer.button` event.
#[derive(Debug, Clone, Copy)]
pub struct WlPointerButtonEvent {
    /// Serial number of the button event.
    pub serial: u32,
    /// Timestamp of the event, with millisecond granularity.
    pub time_ms: u32,
    /// The button that was pressed or released (e.g., from `input-event-codes.h`).
    pub button_code: u32,
    /// The state of the button (`WlPointerButtonState`).
    pub state: WlPointerButtonState,
}

/// State of a pointer button. Corresponds to `wl_pointer.button_state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WlPointerButtonState {
    Released = 0,
    Pressed = 1,
}

/// Data for the `wl_pointer.axis` event (scroll).
#[derive(Debug, Clone, Copy)]
pub struct WlPointerAxisEvent {
    /// Timestamp of the event, with millisecond granularity.
    pub time_ms: u32,
    /// The axis that was scrolled (`WlPointerAxis`).
    pub axis: WlPointerAxis,
    /// The amount scrolled along the axis, in unspecified units.
    pub value: f64,
}

/// Axis type for pointer scroll events. Corresponds to `wl_pointer.axis`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WlPointerAxis {
    VerticalScroll = 0,
    HorizontalScroll = 1,
}

// Placeholder for event sending functions.
// These would eventually use a Wayland connection object to send messages.

pub fn send_enter_event(_client_id: ClientId, _pointer_obj_id: u32, _event: &WlPointerEnterEvent) {
    // println!("Conceptual: Send wl_pointer.enter to client {}, pointer {}, surface {}, serial {}, at ({}, {})",
    //          client_id.0, pointer_obj_id, event.surface_id.0, event.serial, event.surface_x, event.surface_y);
}

pub fn send_leave_event(_client_id: ClientId, _pointer_obj_id: u32, _event: &WlPointerLeaveEvent) {
    // println!("Conceptual: Send wl_pointer.leave to client {}, pointer {}, surface {}, serial {}",
    //          client_id.0, pointer_obj_id, event.surface_id.0, event.serial);
}

// ... other conceptual send_xxx_event functions ...

// Request handlers for wl_pointer interface will be added here or in a separate file.

use std::collections::HashMap;
use crate::input::seat::SeatState; // For seat_state_arc in set_cursor
use crate::surface::{SurfaceRegistry, SurfaceRole}; // For SurfaceRegistry and SurfaceRole

// --- wl_pointer Request Handlers ---

/// Error type for `set_cursor` operation.
#[derive(Debug, PartialEq, Eq)]
pub enum SetCursorError {
    /// The provided surface already has a role that is incompatible with being a cursor.
    RoleConflict,
    /// The client attempting to set the cursor does not own the specified surface
    /// and the pointer focus is not on one of the client's surfaces.
    SurfaceAccessDenied, // Simplified check for now
    /// The provided surface ID was not found in the registry.
    SurfaceNotFound,
}


/// Handles the `wl_pointer.set_cursor` request.
///
/// Sets the cursor image for the pointer.
///
/// # Arguments
/// * `pointer_arc`: The `WlPointer` instance for which the cursor is being set.
/// * `serial`: The serial of the implicit grab on the pointer (e.g., from an enter event).
///             The cursor is only updated if this serial matches `pointer.last_enter_serial`.
/// * `surface_id_opt`: `Option<SurfaceId>` for the surface to use as the cursor image.
///                     If `None`, the cursor should be hidden or reset to default.
/// * `hotspot_x`, `hotspot_y`: Coordinates of the cursor hotspot relative to the surface.
/// * `surface_registry`: A reference to the `SurfaceRegistry` to access surface details.
/// * `seat_state_arc`: The seat state, used to check pointer focus. (Currently simplified).
///
/// # Returns
/// * `Ok(())` if the cursor was successfully set or hidden.
/// * `Err(SetCursorError)` if there's an issue (e.g., role conflict, surface not found).
///
/// # Wayland Spec Notes
/// - The cursor is updated only if `serial` matches the serial of the last enter event.
/// - If `surface_id_opt` is `Some`, the client must own the surface, OR the current pointer
///   focus must be on one of the client's surfaces. (Simplified: client owns surface).
/// - If the surface already has a role, a `role` protocol error (0) might be sent.
pub fn handle_set_cursor(
    pointer_arc: Arc<Mutex<WlPointer>>,
    serial: u32,
    surface_id_opt: Option<SurfaceId>,
    hotspot_x: i32,
    hotspot_y: i32,
    surface_registry: &SurfaceRegistry, // Assuming direct access for simplicity
    _seat_state_arc: Arc<Mutex<SeatState>>, // Used for focus check, simplified for now
) -> Result<(), SetCursorError> {
    let mut pointer = pointer_arc.lock().unwrap();

    // Wayland spec: "This request only takes effect if the pointer focus for this device is one
    // of the requesting client's surfaces or the surface parameter is the current pointer surface.
    // If there was a previous surface set with this request, set its role to اتفاقي , and
    //wl_surface.commit state to it." - This part is complex.
    // "If serial is not valid, this request is ignored."
    if pointer.last_enter_serial != serial && surface_id_opt.is_some() { // Serial check often only for new cursor
        // For hiding cursor (surface_id_opt is None), some compositors ignore serial.
        // Let's be strict for setting a new cursor.
        // Or, if serial is 0 and last_enter_serial is 0 (e.g. before first enter), allow?
        // For now, if serial is provided and doesn't match, ignore if setting a new surface.
        // If hiding (None), we might allow it regardless of serial.
        // The spec says: "If serial is not valid, this request is ignored." - applies to whole request.
        // However, weston seems to allow set_cursor(NULL) even with stale serial.
        // For now, let's allow hiding cursor even with stale serial.
        if surface_id_opt.is_some() {
             // Ignoring due to serial mismatch when setting a new cursor.
            return Ok(());
        }
    }

    // If there was a previous cursor surface, its role might need to be reset.
    // This depends on how roles are managed (e.g., if a surface can only have one role).
    // For simplicity, we don't explicitly reset the old surface's role here, assuming
    // it's either implicitly no longer a cursor or that role management is handled elsewhere.
    // A Surface::unset_role() or similar might be needed in a full system.

    if let Some(surface_id) = surface_id_opt {
        let surface_arc = surface_registry.get_surface(surface_id)
            .ok_or(SetCursorError::SurfaceNotFound)?;
        let mut surface = surface_arc.lock().unwrap();

        // TODO: Proper client ownership check based on Wayland spec.
        // For now, assume client owns the surface if it's trying to set it as cursor.
        // This check would involve comparing pointer.client_id with surface.client_id.
        // if pointer.client_id != surface.client_id {
        //     return Err(SetCursorError::SurfaceAccessDenied);
        // }

        // Check for role conflict (simplified)
        if surface.role.is_some() && !matches!(surface.role, Some(SurfaceRole::Cursor)) {
            // If it's already a cursor, it's fine. Otherwise, conflict.
             if !matches!(surface.role, Some(SurfaceRole::Cursor { .. })) { // Check if it's not already a cursor
                // For this simplified subtask, if it has *any* other role, we might deny.
                // A real compositor might allow a surface to be a cursor even if it's also, e.g., a toplevel.
                // However, wl_shell/xdg_shell surfaces cannot be used as cursors.
                // Let's assume for now that if it has a role, and it's not Cursor, it's a conflict.
                // The subtask mentions "send protocol error wl_pointer.error (role = 0)".
                // This SetCursorError::RoleConflict would map to that.
                return Err(SetCursorError::RoleConflict);
            }
        }

        surface.role = Some(SurfaceRole::Cursor); // Set role to Cursor
        // TODO: surface.commit() state if needed, as per spec.

        pointer.cursor_surface = Some(surface_id);
        pointer.cursor_hotspot_x = hotspot_x;
        pointer.cursor_hotspot_y = hotspot_y;
    } else {
        // Hide cursor
        pointer.cursor_surface = None;
        pointer.cursor_hotspot_x = 0; // Reset hotspot
        pointer.cursor_hotspot_y = 0;
    }

    Ok(())
}


/// Handles the `wl_pointer.release` request.
///
/// This request signifies that the client is destroying its `wl_pointer` proxy object.
/// The server should remove this `WlPointer` instance from its list of active pointers.
///
/// # Arguments
/// * `pointer_arc`: An `Arc<Mutex<WlPointer>>` of the pointer object to be released.
/// * `global_pointer_map`: A mutable reference to the global map that stores all active
///                         `WlPointer` objects, keyed by their Wayland object ID.
pub fn handle_release(
    pointer_arc: Arc<Mutex<WlPointer>>,
    global_pointer_map: &mut HashMap<u32, Arc<Mutex<WlPointer>>>,
) {
    let pointer_object_id = {
        let pointer = pointer_arc.lock().unwrap();
        pointer.object_id
    };

    if global_pointer_map.remove(&pointer_object_id).is_some() {
        println!(
            "Pointer object id {} released and removed from global map.",
            pointer_object_id
        );
    } else {
        eprintln!(
            "Warning: Tried to release pointer object id {}, but it was not found in the global map.",
            pointer_object_id
        );
    }
}
