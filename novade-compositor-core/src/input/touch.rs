//! Represents a `wl_touch` object, its state (including active touch points),
//! and associated events.

use crate::input::seat::SeatId;
use crate::surface::SurfaceId;
use novade_buffer_manager::ClientId;
use std::collections::HashMap;

// TODO: Define error types for touch operations if necessary.

/// Represents the state of a single active touch point.
#[derive(Debug, Clone, Copy)]
pub struct TouchPointState {
    /// The surface that this touch point is currently interacting with.
    pub surface_id: SurfaceId,
    /// The x-coordinate of the touch point, local to the `surface_id`.
    pub position_x: f64,
    /// The y-coordinate of the touch point, local to the `surface_id`.
    pub position_y: f64,
    /// Length of the major axis of the contact ellipse (optional, v6+).
    pub shape_major: Option<f64>,
    /// Length of the minor axis of the contact ellipse (optional, v6+).
    pub shape_minor: Option<f64>,
    /// Angle between major axis and y-axis (optional, v6+).
    pub orientation: Option<f64>,
}

/// Represents a server-side `wl_touch` resource.
///
/// Each client that binds to `wl_seat` and requests a touch interface gets one of these.
/// It tracks client-specific touch state, primarily the set of active touch points.
#[derive(Debug, Clone)]
pub struct WlTouch {
    /// The Wayland object ID for this specific `wl_touch` instance, unique per client.
    pub object_id: u32,
    /// The ID of the client that owns this `wl_touch` resource.
    pub client_id: ClientId,
    /// The ID of the `wl_seat` this touch interface belongs to.
    pub seat_id: SeatId,

    /// Tracks the state of currently active touch points for this `wl_touch` interface.
    /// The key is the `touch_id` (e.g., from a multitouch slot).
    pub active_touch_points: HashMap<i32, TouchPointState>,
}

impl WlTouch {
    /// Creates a new `WlTouch` instance.
    pub fn new(object_id: u32, client_id: ClientId, seat_id: SeatId) -> Self {
        Self {
            object_id,
            client_id,
            seat_id,
            active_touch_points: HashMap::new(),
        }
    }
}

// --- Event Data Structures (Conceptual for now) ---

/// Data for the `wl_touch.down` event.
#[derive(Debug, Clone, Copy)]
pub struct WlTouchDownEvent {
    /// Serial number of the down event.
    pub serial: u32,
    /// Timestamp of the event with millisecond granularity.
    pub time_ms: u32,
    /// The `SurfaceId` of the surface that received the touch-down.
    pub surface_id: SurfaceId,
    /// The ID of the touch point.
    pub touch_id: i32,
    /// Surface-local x-coordinate of the touch point.
    pub x: f64,
    /// Surface-local y-coordinate of the touch point.
    pub y: f64,
}

/// Data for the `wl_touch.up` event.
#[derive(Debug, Clone, Copy)]
pub struct WlTouchUpEvent {
    /// Serial number of the up event.
    pub serial: u32,
    /// Timestamp of the event with millisecond granularity.
    pub time_ms: u32,
    /// The ID of the touch point that was released.
    pub touch_id: i32,
}

/// Data for the `wl_touch.motion` event.
#[derive(Debug, Clone, Copy)]
pub struct WlTouchMotionEvent {
    /// Timestamp of the event with millisecond granularity.
    pub time_ms: u32,
    /// The ID of the touch point that moved.
    pub touch_id: i32,
    /// Surface-local x-coordinate of the new touch point position.
    pub x: f64,
    /// Surface-local y-coordinate of the new touch point position.
    pub y: f64,
}

/// Data for the `wl_touch.frame` event.
/// This event groups a sequence of touch events. It is an empty struct as it carries no data itself.
#[derive(Debug, Clone, Copy)]
pub struct WlTouchFrameEvent;

/// Data for the `wl_touch.cancel` event.
/// This event indicates that the touch sequence has been cancelled.
#[derive(Debug, Clone, Copy)]
pub struct WlTouchCancelEvent;

/// Data for the `wl_touch.shape` event (Wayland protocol version 6+).
#[derive(Debug, Clone, Copy)]
pub struct WlTouchShapeEvent {
    /// The ID of the touch point whose shape changed.
    pub touch_id: i32,
    /// Length of the major axis of the contact ellipse.
    pub major: f64,
    /// Length of the minor axis of the contact ellipse.
    pub minor: f64,
}

/// Data for the `wl_touch.orientation` event (Wayland protocol version 6+).
#[derive(Debug, Clone, Copy)]
pub struct WlTouchOrientationEvent {
    /// The ID of the touch point whose orientation changed.
    pub touch_id: i32,
    /// Angle between the major axis of the touch ellipse and the y-axis of the surface.
    pub orientation: f64,
}

// --- Conceptual Event Sending Functions ---
// These would update WlTouch::active_touch_points and then send Wayland messages.

pub fn send_down_event(
    touch_arc: Arc<Mutex<WlTouch>>,
    event_data: &WlTouchDownEvent,
) {
    let mut touch = touch_arc.lock().unwrap();
    let point_state = TouchPointState {
        surface_id: event_data.surface_id,
        position_x: event_data.x,
        position_y: event_data.y,
        shape_major: None,
        shape_minor: None,
        orientation: None,
    };
    touch.active_touch_points.insert(event_data.touch_id, point_state);
    // println!("Conceptual: Send wl_touch.down to client {}, touch_obj {}, ...", touch.client_id, touch.object_id);
}

pub fn send_up_event(touch_arc: Arc<Mutex<WlTouch>>, event_data: &WlTouchUpEvent) {
    let mut touch = touch_arc.lock().unwrap();
    touch.active_touch_points.remove(&event_data.touch_id);
    // println!("Conceptual: Send wl_touch.up to client {}, touch_obj {}, ...", touch.client_id, touch.object_id);
}

pub fn send_motion_event(
    touch_arc: Arc<Mutex<WlTouch>>,
    event_data: &WlTouchMotionEvent,
) {
    let mut touch = touch_arc.lock().unwrap();
    if let Some(point) = touch.active_touch_points.get_mut(&event_data.touch_id) {
        point.position_x = event_data.x;
        point.position_y = event_data.y;
    }
    // println!("Conceptual: Send wl_touch.motion to client {}, touch_obj {}, ...", touch.client_id, touch.object_id);
}

pub fn send_frame_event(_touch_arc: Arc<Mutex<WlTouch>>) {
    // println!("Conceptual: Send wl_touch.frame");
}

pub fn send_cancel_event(touch_arc: Arc<Mutex<WlTouch>>) {
    let mut touch = touch_arc.lock().unwrap();
    touch.active_touch_points.clear(); // Cancel all active points for this interface
    // println!("Conceptual: Send wl_touch.cancel");
}

pub fn send_shape_event(touch_arc: Arc<Mutex<WlTouch>>, event_data: &WlTouchShapeEvent) {
     let mut touch = touch_arc.lock().unwrap();
    if let Some(point) = touch.active_touch_points.get_mut(&event_data.touch_id) {
        point.shape_major = Some(event_data.major);
        point.shape_minor = Some(event_data.minor);
    }
    // println!("Conceptual: Send wl_touch.shape");
}

pub fn send_orientation_event(touch_arc: Arc<Mutex<WlTouch>>, event_data: &WlTouchOrientationEvent) {
     let mut touch = touch_arc.lock().unwrap();
    if let Some(point) = touch.active_touch_points.get_mut(&event_data.touch_id) {
        point.orientation = Some(event_data.orientation);
    }
    // println!("Conceptual: Send wl_touch.orientation");
}

// Request handlers for wl_touch interface will be added here or in a separate file.
// e.g., handle_release (for wl_touch proxy)

// --- wl_touch Request Handlers ---

/// Handles the `wl_touch.release` request.
///
/// This request signifies that the client is destroying its `wl_touch` proxy object.
/// The server should remove this `WlTouch` instance from its list of active touch interfaces
/// to free up resources. Any active touch points associated with this interface are implicitly cancelled.
///
/// # Arguments
/// * `touch_arc`: An `Arc<Mutex<WlTouch>>` of the touch object to be released.
///                The lock will be taken to access its `object_id`.
/// * `global_touch_map`: A mutable reference to the global map (e.g., in `CompositorState`
///                         or `InputManager`) that stores all active `WlTouch` objects,
///                         keyed by their Wayland object ID.
pub fn handle_release(
    touch_arc: Arc<Mutex<WlTouch>>,
    global_touch_map: &mut HashMap<u32, Arc<Mutex<WlTouch>>>,
) {
    let touch_object_id = { // Scope for lock
        let mut touch = touch_arc.lock().unwrap();
        // Implicitly, all active touch points for this interface are now gone.
        // No explicit wl_touch.cancel event is sent on release, but the client
        // will no longer receive events for these points from this object.
        touch.active_touch_points.clear();
        touch.object_id
    };

    if global_touch_map.remove(&touch_object_id).is_some() {
        println!(
            "Touch object id {} released and removed from global map.",
            touch_object_id
        );
    } else {
        eprintln!(
            "Warning: Tried to release touch object id {}, but it was not found in the global map.",
            touch_object_id
        );
    }
}
