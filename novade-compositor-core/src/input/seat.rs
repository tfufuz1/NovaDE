//! Represents a `wl_seat` global and its associated state.
//!
//! A seat is a collection of input devices (pointer, keyboard, touch) that
//! a user interacts with. Each client binds to the `wl_seat` global to receive
//! input events and create device-specific objects like `wl_pointer`.

use bitflags::bitflags;
use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
use std::collections::HashMap;
use crate::surface::SurfaceId; // For focus tracking
use novade_buffer_manager::ClientId; // Assuming ClientId is globally unique

/// Unique identifier for a `wl_seat` instance.
///
/// Although most compositors will only have one seat (e.g., "seat0"),
/// the Wayland protocol allows for multiple seats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeatId(u64);

impl SeatId {
    /// Creates a new, unique `SeatId`.
    pub fn new_unique() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0); // Start IDs from 0 for seats.
        SeatId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

bitflags! {
    /// Represents the capabilities of a `wl_seat`.
    ///
    /// These flags indicate whether the seat has pointer, keyboard, or touch input devices.
    /// Corresponds to the `wl_seat.capabilities` event.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SeatCapability: u32 {
        /// The seat has pointer devices (e.g., a mouse).
        const Pointer = 1;
        /// The seat has keyboard devices.
        const Keyboard = 2;
        /// The seat has touch devices.
        const Touch = 4;
    }
}

// --- Placeholder structs for Wayland input device objects ---

/// Placeholder for a client's `wl_pointer` object.
///
/// In a full implementation, this would store more state related to the pointer,
/// such as cursor image, surface enter/leave state, etc.
#[derive(Debug)]
pub struct WlPointer {
    /// The Wayland object ID for this `wl_pointer` instance.
    pub object_id: u32,
    // Potentially: Arc<Mutex<Surface>> for cursor surface
}

/// Placeholder for a client's `wl_keyboard` object.
#[derive(Debug)]
pub struct WlKeyboard {
    /// The Wayland object ID for this `wl_keyboard` instance.
    pub object_id: u32,
    // Potentially: keymap, repeat info, focus tracking state
}

/// Placeholder for a client's `wl_touch` object.
#[derive(Debug)]
pub struct WlTouch {
    /// The Wayland object ID for this `wl_touch` instance.
    pub object_id: u32,
    // Potentially: touch point tracking, focus
}

/// Represents the state of a single `wl_seat`.
///
/// This struct holds all information about a seat, including its capabilities,
/// the input devices currently associated with it, focus information, and
/// active client proxies for those devices.
#[derive(Debug)]
pub struct SeatState {
    /// Unique ID of this seat.
    pub id: SeatId,
    /// Name of the seat (e.g., "seat0"). Sent via `wl_seat.name` event.
    pub name: String,
    /// Current capabilities of the seat (pointer, keyboard, touch).
    pub capabilities: SeatCapability,

    // Conceptual representation of physical devices or their logical state.
    // For now, we might not need full structs if we're just tracking IDs.
    // These could be populated when actual input hardware is configured.
    // pub physical_pointer: Option<InputDevicePointerState>,
    // pub physical_keyboard: Option<InputDeviceKeyboardState>,
    // pub physical_touch: Option<InputDeviceTouchState>,

    /// The surface that currently has pointer focus for this seat.
    pub pointer_focus: Option<SurfaceId>,
    /// The surface that currently has keyboard focus for this seat.
    pub keyboard_focus: Option<SurfaceId>,
    // Touch focus is more complex (per touch point), might be handled within WlTouch or similar.

    // pub active_pointer_proxies: HashMap<ClientId, u32>, // Replaced by global WlPointer map
    // pub active_keyboard_proxies: HashMap<ClientId, u32>, // Replaced by global WlKeyboard map
    // pub active_touch_proxies: HashMap<ClientId, u32>, // Replaced by global WlTouch map

    // TODO: Store actual WlPointer, WlKeyboard, WlTouch objects (Arc<Mutex<...>>)
    // if they need to hold more state that is shared across clients or tied to the seat.
    // For now, the proxies map might be enough if these structs are just ID holders.
}

impl SeatState {
    /// Creates a new `SeatState`.
    ///
    /// # Arguments
    /// * `id`: The unique `SeatId` for this seat.
    /// * `name`: The name of the seat (e.g., "seat0").
    /// * `capabilities`: The initial `SeatCapability` for this seat.
    pub fn new(id: SeatId, name: String, capabilities: SeatCapability) -> Self {
        Self {
            id,
            name,
            capabilities,
            pointer_focus: None,
            keyboard_focus: None,
            // active_pointer_proxies: HashMap::new(), // Replaced
            // active_keyboard_proxies: HashMap::new(), // Replaced
            // active_touch_proxies: HashMap::new(), // Replaced
        }
    }

    /// Updates the capabilities of this seat.
    ///
    /// If the capabilities change, this should conceptually trigger sending the
    /// `wl_seat.capabilities` event to all clients bound to this seat.
    ///
    /// # Arguments
    /// * `new_capabilities`: The new set of `SeatCapability` for this seat.
    pub fn update_capabilities(&mut self, new_capabilities: SeatCapability) {
        if self.capabilities != new_capabilities {
            self.capabilities = new_capabilities;
            // TODO: Send wl_seat.capabilities event to relevant clients.
            // This typically involves iterating over all client proxies associated with this seat.
            println!("Seat [{}]: Capabilities updated to {:?}", self.name, self.capabilities);
        }
    }

    /// Adds a capability to the seat.
    pub fn add_capability(&mut self, cap: SeatCapability) {
        let new_caps = self.capabilities | cap;
        self.update_capabilities(new_caps);
    }

    /// Removes a capability from the seat.
    pub fn remove_capability(&mut self, cap: SeatCapability) {
        let new_caps = self.capabilities - cap;
        self.update_capabilities(new_caps);
    }

    // TODO: Methods for get_pointer_id_for_client, etc. if WlPointer objects are stored directly.
    // For now, the handlers will manage the proxy HashMaps directly.
}

// --- Focus Handling Methods on SeatState ---
// These methods would typically be called by an InputManager or the main compositor loop
// in response to input events from the backend.

impl SeatState {
    /// Updates the pointer focus based on the current global pointer coordinates.
    ///
    /// This involves:
    /// 1. Determining which surface is currently under the pointer (`surface_at`).
    /// 2. Comparing with the previous focus.
    /// 3. Sending conceptual `wl_pointer.leave` events for the old focus.
    /// 4. Sending conceptual `wl_pointer.enter` events for the new focus.
    /// 5. Updating the `last_enter_serial` on the relevant `WlPointer` objects.
    /// 6. If focus remains on the same surface, conceptually sending `wl_pointer.motion`.
    ///
    /// # Arguments
    /// * `global_x`, `global_y`: Global compositor coordinates of the pointer.
    /// * `serial`: The serial for this input event sequence.
    /// * `surface_registry`: Access to all registered surfaces.
    /// * `pointer_interfaces`: A map of active `WlPointer` objects (Wayland object ID -> Arc<Mutex<WlPointer>>).
    ///                         This is needed to find the correct client's pointer object to update its serial
    ///                         and to conceptually send events to.
    pub fn update_pointer_focus(
        &mut self,
        global_x: f64,
        global_y: f64,
        serial: u32,
        surface_registry: &impl crate::surface::surface_registry::SurfaceRegistryAccessor,
        pointer_interfaces: &HashMap<u32, Arc<Mutex<crate::input::pointer::WlPointer>>>,
    ) {
        let new_focus_surface_id_opt = crate::input::focus::surface_at((global_x, global_y), surface_registry);
        let old_focus_surface_id_opt = self.pointer_focus.take(); // Take old focus to avoid borrow issues

        if new_focus_surface_id_opt != old_focus_surface_id_opt {
            // --- Handle Leave ---
            if let Some(old_focus_id) = old_focus_surface_id_opt {
                if let Some(old_surface_arc) = surface_registry.get_surface(old_focus_id) {
                    let old_surface = old_surface_arc.lock().unwrap();
                    // Conceptually send leave to all WlPointer proxies for the client owning old_surface
                    for pointer_arc in pointer_interfaces.values() {
                        let mut pointer = pointer_arc.lock().unwrap();
                        if pointer.client_id == old_surface.client_id && pointer.seat_id == self.id {
                            crate::input::pointer::send_leave_event(
                                pointer.client_id,
                                pointer.object_id,
                                &crate::input::pointer::WlPointerLeaveEvent { serial, surface_id: old_focus_id },
                            );
                            // No need to reset pointer.last_enter_serial on leave.
                        }
                    }
                }
            }

            // --- Handle Enter ---
            self.pointer_focus = new_focus_surface_id_opt; // Set new focus
            if let Some(new_focus_id) = new_focus_surface_id_opt {
                if let Some(new_surface_arc) = surface_registry.get_surface(new_focus_id) {
                    let new_surface = new_surface_arc.lock().unwrap();
                    let local_x = global_x - new_surface.current_attributes.position.0 as f64;
                    let local_y = global_y - new_surface.current_attributes.position.1 as f64;

                    // Conceptually send enter to all WlPointer proxies for the client owning new_surface
                    for pointer_arc in pointer_interfaces.values() {
                        let mut pointer = pointer_arc.lock().unwrap();
                        if pointer.client_id == new_surface.client_id && pointer.seat_id == self.id {
                            pointer.last_enter_serial = serial; // Update serial *before* sending enter
                            crate::input::pointer::send_enter_event(
                                pointer.client_id,
                                pointer.object_id,
                                &crate::input::pointer::WlPointerEnterEvent {
                                    serial,
                                    surface_id: new_focus_id,
                                    surface_x: local_x,
                                    surface_y: local_y,
                                },
                            );
                        }
                    }
                } else {
                    self.pointer_focus = None; // Surface disappeared before we could use it
                }
            }
        } else if let Some(current_focus_id) = self.pointer_focus { // Focus remains on the same surface
             if let Some(current_surface_arc) = surface_registry.get_surface(current_focus_id) {
                let current_surface = current_surface_arc.lock().unwrap();
                let local_x = global_x - current_surface.current_attributes.position.0 as f64;
                let local_y = global_y - current_surface.current_attributes.position.1 as f64;

                // Conceptually send motion to all WlPointer proxies for the client owning current_surface
                for pointer_arc in pointer_interfaces.values() {
                    let pointer = pointer_arc.lock().unwrap();
                    if pointer.client_id == current_surface.client_id && pointer.seat_id == self.id {
                        // Conceptual send_motion_event - not fully defined in pointer.rs yet
                        // crate::input::pointer::send_motion_event(pointer.client_id, pointer.object_id, ...);
                        println!("Conceptual: Send wl_pointer.motion to client {:?}, pointer {}, surface {}, local ({}, {})",
                                 pointer.client_id, pointer.object_id, current_focus_id.0, local_x, local_y);
                    }
                }
            }
        }
    }

    /// Sets the keyboard focus for this seat to the given surface.
    ///
    /// This involves:
    /// 1. Comparing with the previous focus.
    /// 2. Sending conceptual `wl_keyboard.leave` events for the old focus.
    /// 3. Sending conceptual `wl_keyboard.enter`, `wl_keyboard.modifiers` events for the new focus.
    /// 4. Updating `last_enter_serial` on relevant `WlKeyboard` objects.
    ///
    /// # Arguments
    /// * `new_focus_surface_id_opt`: `Option<SurfaceId>` of the surface to grant keyboard focus.
    ///                                `None` to clear keyboard focus.
    /// * `serial`: The serial for this input event sequence.
    /// * `surface_registry`: Access to all registered surfaces.
    /// * `keyboard_interfaces`: A map of active `WlKeyboard` objects.
    pub fn set_keyboard_focus(
        &mut self,
        new_focus_surface_id_opt: Option<SurfaceId>,
        serial: u32,
        surface_registry: &impl crate::surface::surface_registry::SurfaceRegistryAccessor,
        keyboard_interfaces: &HashMap<u32, Arc<Mutex<crate::input::keyboard::WlKeyboard>>>,
    ) {
        let old_focus_surface_id_opt = self.keyboard_focus.take();

        if new_focus_surface_id_opt != old_focus_surface_id_opt {
            // --- Handle Leave ---
            if let Some(old_focus_id) = old_focus_surface_id_opt {
                if let Some(old_surface_arc) = surface_registry.get_surface(old_focus_id) {
                    let old_surface = old_surface_arc.lock().unwrap();
                    for keyboard_arc in keyboard_interfaces.values() {
                        let keyboard = keyboard_arc.lock().unwrap();
                        if keyboard.client_id == old_surface.client_id && keyboard.seat_id == self.id {
                             crate::input::keyboard::send_leave_event_conceptual( // Assuming a conceptual sender
                                keyboard.client_id,
                                keyboard.object_id,
                                serial, // Leave serial
                                old_focus_id
                            );
                        }
                    }
                }
            }

            // --- Handle Enter ---
            self.keyboard_focus = new_focus_surface_id_opt;
            if let Some(new_focus_id) = new_focus_surface_id_opt {
                 if let Some(new_surface_arc) = surface_registry.get_surface(new_focus_id) {
                    let new_surface = new_surface_arc.lock().unwrap();
                    // Placeholder for actual pressed keys state
                    let keys_pressed_on_enter: Vec<u32> = Vec::new();
                    // Placeholder for actual modifiers state
                    let current_modifiers = ModifiersState::default();

                    for keyboard_arc in keyboard_interfaces.values() {
                        let mut keyboard = keyboard_arc.lock().unwrap();
                        if keyboard.client_id == new_surface.client_id && keyboard.seat_id == self.id {
                            keyboard.last_enter_serial = serial;
                            keyboard.modifiers_state = current_modifiers; // Update before sending

                            crate::input::keyboard::send_enter_event_conceptual(
                                keyboard.client_id, keyboard.object_id, serial, new_focus_id, keys_pressed_on_enter.clone()
                            );
                            crate::input::keyboard::send_modifiers_event_conceptual(
                                keyboard.client_id, keyboard.object_id, serial, &current_modifiers
                            );
                        }
                    }
                } else {
                    self.keyboard_focus = None; // Surface disappeared
                }
            }
        }
    }
}


/// Manages all `wl_seat` instances within the compositor.
///
/// Typically, a compositor will have at least one primary seat (e.g., "seat0").
/// This manager allows for creation and retrieval of seat states.
#[derive(Debug, Default)]
pub struct SeatManager {
    seats: HashMap<SeatId, Arc<Mutex<SeatState>>>,
    main_seat_id: Option<SeatId>, // Tracks the ID of the primary seat
}

impl SeatManager {
    /// Creates a new, empty `SeatManager`.
    pub fn new() -> Self {
        Self {
            seats: HashMap::new(),
            main_seat_id: None,
        }
    }

    /// Creates a new seat with the given name and capabilities, adds it to the manager,
    /// and returns a shared pointer to its state.
    ///
    /// If no main seat is currently set, this new seat becomes the main seat.
    ///
    /// # Arguments
    /// * `name`: The name for the new seat (e.g., "seat0").
    /// * `capabilities`: The initial `SeatCapability` for the new seat.
    ///
    /// # Returns
    /// An `Arc<Mutex<SeatState>>` for the newly created seat.
    pub fn create_seat(&mut self, name: String, capabilities: SeatCapability) -> Arc<Mutex<SeatState>> {
        let id = SeatId::new_unique();
        let seat_state = Arc::new(Mutex::new(SeatState::new(id, name, capabilities)));

        self.seats.insert(id, seat_state.clone());
        if self.main_seat_id.is_none() {
            self.main_seat_id = Some(id);
        }

        seat_state
    }

    /// Retrieves a seat by its `SeatId`.
    ///
    /// # Arguments
    /// * `id`: The `SeatId` of the seat to retrieve.
    ///
    /// # Returns
    /// An `Option<Arc<Mutex<SeatState>>>` containing the seat state if found, otherwise `None`.
    pub fn get_seat(&self, id: SeatId) -> Option<Arc<Mutex<SeatState>>> {
        self.seats.get(&id).cloned()
    }

    /// Retrieves the main seat for the compositor.
    ///
    /// This is a convenience method, often used in single-seat setups.
    ///
    /// # Returns
    /// An `Option<Arc<Mutex<SeatState>>>` containing the main seat state if one is set,
    /// otherwise `None`.
    pub fn get_main_seat(&self) -> Option<Arc<Mutex<SeatState>>> {
        self.main_seat_id.and_then(|id| self.get_seat(id))
    }
}

/// Errors that can occur during `wl_seat` request handling.
#[derive(Debug, thiserror::Error)] // Assuming thiserror could be used eventually
pub enum SeatError {
    /// Requested capability (e.g., pointer, keyboard) is not available on this seat.
    /// Corresponds to `wl_seat` error `0` (missing_capability).
    #[error("Seat '{seat_name}' does not have capability: {capability:?}")]
    MissingCapability {
        seat_name: String,
        capability: SeatCapability,
    },
    // TODO: Add other seat-specific errors if needed.
}

// These handlers would typically be called by a Wayland dispatcher.
// They operate on the shared SeatState and client information.

/// Handles the `wl_seat.get_pointer` request.
///
/// If the seat has the `Pointer` capability, it registers the new `wl_pointer` object ID
/// for the client. Otherwise, a protocol error should be sent.
///
/// # Arguments
/// * `seat_state_arc`: A shared pointer to the `SeatState`.
/// * `client_id`: The ID of the client making the request.
/// * `new_pointer_id`: The Wayland object ID for the new `wl_pointer` to be created.
///
/// # Returns
/// * `Ok(WlPointer)`: If successful, returning a placeholder `WlPointer`.
/// * `Err(SeatError::MissingCapability)`: If the seat does not have the pointer capability.
pub fn handle_get_pointer(
    seat_state_arc: Arc<Mutex<SeatState>>,
    client_id: ClientId,
    new_pointer_id: u32,
) -> Result<WlPointer, SeatError> {
    let mut seat_state = seat_state_arc.lock().unwrap(); // Handle potential poisoning

    if !seat_state.capabilities.contains(SeatCapability::Pointer) {
        return Err(SeatError::MissingCapability {
            seat_name: seat_state.name.clone(),
            capability: SeatCapability::Pointer,
        });
    }

    // TODO: Create and manage the actual wl_pointer server-side object.
    // For now, client-specific WlPointer instances are managed globally.
    // seat_state.active_pointer_proxies.insert(client_id, new_pointer_id);
    println!(
        "Seat [{}]: Client {:?} requested wl_pointer id {}. (Actual object managed globally)",
        seat_state.name, client_id, new_pointer_id
    );

    // The actual WlPointer object would be registered with the Wayland display server here.
    Ok(WlPointer { object_id: new_pointer_id })
}

/// Handles the `wl_seat.get_keyboard` request.
///
/// Similar to `handle_get_pointer`, but for keyboards.
pub fn handle_get_keyboard(
    seat_state_arc: Arc<Mutex<SeatState>>,
    client_id: ClientId,
    new_keyboard_id: u32,
) -> Result<WlKeyboard, SeatError> {
    let mut seat_state = seat_state_arc.lock().unwrap();

    if !seat_state.capabilities.contains(SeatCapability::Keyboard) {
        return Err(SeatError::MissingCapability {
            seat_name: seat_state.name.clone(),
            capability: SeatCapability::Keyboard,
        });
    }

    // seat_state.active_keyboard_proxies.insert(client_id, new_keyboard_id); // Managed globally now

    let new_wl_keyboard = Arc::new(Mutex::new(WlKeyboard::new(new_keyboard_id, client_id, seat_state.id)));

    // TODO: Store new_wl_keyboard in a global map (e.g., CompositorState.active_keyboards).
    // Example: global_data.active_keyboards.insert(new_keyboard_id, new_wl_keyboard.clone());

    println!(
        "Seat [{}]: Client {:?} requested wl_keyboard id {}. (Actual object managed globally)",
        seat_state.name, client_id, new_keyboard_id
    );

    // Conceptually, send initial keymap and repeat_info after client binds.
    // These calls would typically be made by the dispatcher after the object is created
    // and registered with the Wayland display.
    crate::input::keyboard::send_keymap(new_wl_keyboard.clone());
    crate::input::keyboard::send_repeat_info(new_wl_keyboard.clone());

    // The dispatcher expects the raw struct for registration, not the Arc usually.
    // Or, if the dispatcher handles Arcs, then clone it.
    // For now, we return the inner object as per previous pattern, assuming dispatcher handles it.
    // This might need adjustment based on how the global map and dispatcher work.
    let keyboard_to_return = new_wl_keyboard.lock().unwrap().clone();
    Ok(keyboard_to_return)
}

/// Handles the `wl_seat.get_touch` request.
///
/// Similar to `handle_get_pointer`, but for touch devices.
pub fn handle_get_touch(
    seat_state_arc: Arc<Mutex<SeatState>>,
    client_id: ClientId,
    new_touch_id: u32,
) -> Result<WlTouch, SeatError> {
    let mut seat_state = seat_state_arc.lock().unwrap();

    if !seat_state.capabilities.contains(SeatCapability::Touch) {
        return Err(SeatError::MissingCapability {
            seat_name: seat_state.name.clone(),
            capability: SeatCapability::Touch,
        });
    }

    // seat_state.active_touch_proxies.insert(client_id, new_touch_id); // Managed globally now

    let new_wl_touch = Arc::new(Mutex::new(WlTouch::new(new_touch_id, client_id, seat_state.id)));

    // TODO: Store new_wl_touch in a global map (e.g., CompositorState.active_touch_interfaces).
    // Example: global_data.active_touch_interfaces.insert(new_touch_id, new_wl_touch.clone());

    println!(
        "Seat [{}]: Client {:?} requested wl_touch id {}. (Actual object managed globally)",
        seat_state.name, client_id, new_touch_id
    );

    // The dispatcher expects the raw struct for registration, not the Arc usually.
    // This might need adjustment based on how the global map and dispatcher work.
    let touch_to_return = new_wl_touch.lock().unwrap().clone();
    Ok(touch_to_return)
}

/// Handles the `wl_seat.release` request.
///
/// This request indicates that the client is destroying its `wl_seat` proxy object.
/// The server should clean up any resources associated with this client's seat interface,
/// such as any `wl_pointer`, `wl_keyboard`, or `wl_touch` objects created by this client
/// for this seat. The global `wl_seat` itself is not destroyed.
///
/// # Arguments
/// * `seat_state_arc`: A shared pointer to the `SeatState`.
/// * `client_id`: The ID of the client releasing the seat.
/// * `seat_object_id`: The Wayland object ID of the `wl_seat` proxy being released.
///                     (Currently unused in this simplified handler but part of protocol).
pub fn handle_release(
    seat_state_arc: Arc<Mutex<SeatState>>,
    client_id: ClientId,
    _seat_object_id: u32, // Often useful for logging or validation
) {
    let mut seat_state = seat_state_arc.lock().unwrap();

    // Remove any active device proxies for this client from this seat.
    // WlPointer objects are managed globally now, so no direct removal from seat_state.active_pointer_proxies.
    // The global map of WlPointers should be cleaned by the caller of handle_release_pointer_object or similar.
    // if let Some(pointer_id) = seat_state.active_pointer_proxies.remove(&client_id) {
    //     // TODO: Destroy the actual wl_pointer server-side object.
    //     println!("Seat [{}]: Client {:?} released wl_pointer id {}.", seat_state.name, client_id, pointer_id);
    // }
    // WlKeyboard objects are managed globally. Their removal from the global map
    // would be triggered by wl_keyboard.release, not wl_seat.release directly for specific keyboard instances.
    // However, if a client releases the *seat*, all its associated device proxies are implicitly released.
    // So, we still need to clean up any *references* or *associations* this seat might hold
    // for this client's keyboard, even if the main WlKeyboard object is in a global map.
    // For now, active_keyboard_proxies was removed, so this specific line is not needed here.
    // if let Some(keyboard_id) = seat_state.active_keyboard_proxies.remove(&client_id) {
    //     // TODO: Destroy the actual wl_keyboard server-side object.
    //     println!("Seat [{}]: Client {:?} released wl_keyboard id {}.", seat_state.name, client_id, keyboard_id);
    // }
    // WlTouch objects are managed globally.
    // if let Some(touch_id) = seat_state.active_touch_proxies.remove(&client_id) {
    //     // TODO: Destroy the actual wl_touch server-side object.
    //     println!("Seat [{}]: Client {:?} released wl_touch id {}.", seat_state.name, client_id, touch_id);
    // }

    println!("Seat [{}]: Client {:?} released its wl_seat proxy.", seat_state.name, client_id);
    // Note: The wl_seat global itself is not destroyed here. Only the client's specific
    // resources related to its binding of this seat are cleaned up.
}

// Conceptual event sending:
// When capabilities change on SeatState.update_capabilities:
//   Iterate all clients that have bound this seat.
//   For each client, send wl_seat.capabilities(seat_object_id_for_client, new_caps).
//
// When a client binds to wl_seat global:
//   1. Create client's wl_seat proxy (object_id).
//   2. Send wl_seat.capabilities(object_id, seat_state.capabilities).
//   3. Send wl_seat.name(object_id, seat_state.name).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::keyboard::WlKeyboard; // For handle_get_keyboard
    use crate::input::pointer::WlPointer;   // For handle_get_pointer
    use crate::input::touch::WlTouch;       // For handle_get_touch
    // ClientId is already imported via novade_buffer_manager used by WlPointer etc.

    fn test_client_id(id: u64) -> ClientId { ClientId::new(id) }

    #[test]
    fn test_seat_creation_and_manager() {
        let mut seat_manager = SeatManager::new();
        assert!(seat_manager.get_main_seat().is_none());

        let caps = SeatCapability::Pointer | SeatCapability::Keyboard;
        let seat_arc = seat_manager.create_seat("seat0".to_string(), caps);
        let seat_id = seat_arc.lock().unwrap().id;
        let seat_name = seat_arc.lock().unwrap().name.clone();

        assert_eq!(seat_name, "seat0");
        assert_eq!(seat_arc.lock().unwrap().capabilities, caps);

        assert!(seat_manager.get_seat(seat_id).is_some());
        assert_eq!(seat_manager.get_seat(seat_id).unwrap().lock().unwrap().id, seat_id);

        assert!(seat_manager.get_main_seat().is_some());
        assert_eq!(seat_manager.get_main_seat().unwrap().lock().unwrap().id, seat_id);

        // Create another seat, main seat should not change
        let seat2_arc = seat_manager.create_seat("seat1".to_string(), SeatCapability::Touch);
        assert_ne!(seat_manager.get_main_seat().unwrap().lock().unwrap().id, seat2_arc.lock().unwrap().id);
        assert_eq!(seat_manager.get_main_seat().unwrap().lock().unwrap().id, seat_id);
    }

    #[test]
    fn test_seat_capability_changes() {
        let seat_id = SeatId::new_unique();
        let mut seat_state = SeatState::new(seat_id, "seat0".to_string(), SeatCapability::empty());

        assert_eq!(seat_state.capabilities, SeatCapability::empty());

        seat_state.add_capability(SeatCapability::Pointer);
        assert_eq!(seat_state.capabilities, SeatCapability::Pointer);
        // Conceptual: wl_seat.capabilities event would be sent here.

        seat_state.add_capability(SeatCapability::Keyboard);
        assert_eq!(seat_state.capabilities, SeatCapability::Pointer | SeatCapability::Keyboard);

        seat_state.update_capabilities(SeatCapability::Touch | SeatCapability::Keyboard);
        assert_eq!(seat_state.capabilities, SeatCapability::Touch | SeatCapability::Keyboard);

        seat_state.remove_capability(SeatCapability::Keyboard);
        assert_eq!(seat_state.capabilities, SeatCapability::Touch);

        seat_state.remove_capability(SeatCapability::Pointer); // Removing non-existent cap
        assert_eq!(seat_state.capabilities, SeatCapability::Touch);

        seat_state.update_capabilities(SeatCapability::empty());
        assert_eq!(seat_state.capabilities, SeatCapability::empty());
    }

    #[test]
    fn test_handle_get_pointer_ok() {
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(
            SeatId::new_unique(),
            "seat0".to_string(),
            SeatCapability::Pointer,
        )));
        let client = test_client_id(1);
        let pointer_id = 101;

        match handle_get_pointer(seat_state_arc.clone(), client, pointer_id) {
            Ok(pointer) => assert_eq!(pointer.object_id, pointer_id),
            Err(e) => panic!("handle_get_pointer failed: {:?}", e),
        }
        // Conceptual: Check if pointer_id is tracked in a global map for (client, seat_id) pair.
    }

    #[test]
    fn test_handle_get_pointer_no_capability() {
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(
            SeatId::new_unique(),
            "seat0".to_string(),
            SeatCapability::Keyboard, // No Pointer capability
        )));
        let client = test_client_id(1);
        let pointer_id = 102;

        match handle_get_pointer(seat_state_arc.clone(), client, pointer_id) {
            Err(SeatError::MissingCapability { capability, .. }) => {
                assert_eq!(capability, SeatCapability::Pointer);
            }
            _ => panic!("Expected MissingCapability error"),
        }
    }

    #[test]
    fn test_handle_get_keyboard_ok_and_events_prepared() {
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(
            SeatId::new_unique(),
            "seat0".to_string(),
            SeatCapability::Keyboard,
        )));
        let client = test_client_id(1);
        let keyboard_id = 201;

        match handle_get_keyboard(seat_state_arc.clone(), client, keyboard_id) {
            Ok(keyboard) => {
                assert_eq!(keyboard.object_id, keyboard_id);
                // Check if conceptual send_keymap and send_repeat_info modified the WlKeyboard state
                // (as per their current placeholder implementation)
                assert_eq!(keyboard.keymap_format, Some(1)); // XKB_V1
                assert_eq!(keyboard.repeat_rate, 25);
                assert_eq!(keyboard.repeat_delay, 400);
            }
            Err(e) => panic!("handle_get_keyboard failed: {:?}", e),
        }
    }

    #[test]
    fn test_handle_get_keyboard_no_capability() {
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(
            SeatId::new_unique(),
            "seat0".to_string(),
            SeatCapability::Pointer, // No Keyboard capability
        )));
        let client = test_client_id(1);
        let keyboard_id = 202;

        match handle_get_keyboard(seat_state_arc.clone(), client, keyboard_id) {
            Err(SeatError::MissingCapability { capability, .. }) => {
                assert_eq!(capability, SeatCapability::Keyboard);
            }
            _ => panic!("Expected MissingCapability error"),
        }
    }

    #[test]
    fn test_handle_get_touch_ok() {
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(
            SeatId::new_unique(),
            "seat0".to_string(),
            SeatCapability::Touch,
        )));
        let client = test_client_id(1);
        let touch_id = 301;

        match handle_get_touch(seat_state_arc.clone(), client, touch_id) {
            Ok(touch) => assert_eq!(touch.object_id, touch_id),
            Err(e) => panic!("handle_get_touch failed: {:?}", e),
        }
    }

    #[test]
    fn test_handle_get_touch_no_capability() {
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(
            SeatId::new_unique(),
            "seat0".to_string(),
            SeatCapability::Pointer, // No Touch capability
        )));
        let client = test_client_id(1);
        let touch_id = 302;

        match handle_get_touch(seat_state_arc.clone(), client, touch_id) {
            Err(SeatError::MissingCapability { capability, .. }) => {
                assert_eq!(capability, SeatCapability::Touch);
            }
            _ => panic!("Expected MissingCapability error"),
        }
    }

    #[test]
    fn test_handle_seat_release() {
        let client_id1 = test_client_id(1);
        let seat_name = "seat0_release_test".to_string();
        let seat_state_arc = Arc::new(Mutex::new(SeatState::new(
            SeatId::new_unique(),
            seat_name.clone(),
            SeatCapability::Keyboard | SeatCapability::Touch,
        )));

        // At this point, active_keyboard_proxies and active_touch_proxies are no longer
        // part of SeatState directly. The handle_release function in seat.rs was updated
        // to reflect this (it comments out their removal).
        // So, this test primarily ensures that calling handle_release completes
        // without error and that the conceptual logging indicates the release.
        // No direct state change on SeatState itself is expected regarding these proxy maps from this handler.
        // Actual cleanup of WlKeyboard/WlTouch objects happens via their own release handlers
        // and removal from global maps.

        // Client 1 releases its seat proxy
        handle_release(seat_state_arc.clone(), client_id1, 1001); // 1001 is a dummy seat object ID for client's wl_seat

        // We can't assert much here about the internal state of SeatState regarding
        // specific device proxies for *this specific client* because that tracking was removed.
        // We're just ensuring it runs. The println! in handle_release is the main observable effect here.
        // If SeatState had other client-specific resources tied directly to the seat proxy itself
        // (not the device proxies), we would check those.
        let locked_seat_state = seat_state_arc.lock().unwrap();
        assert_eq!(locked_seat_state.name, seat_name); // Verify it's still the same seat
    }

    // --- Focus Logic Tests (within SeatState) ---

    // Mock SurfaceRegistry for focus tests
    struct MockSurfaceRegistry {
        surfaces: HashMap<SurfaceId, Arc<Mutex<Surface>>>,
        surface_order_for_picking: Vec<SurfaceId>, // Control picking order
    }
    impl MockSurfaceRegistry {
        fn new() -> Self { Self { surfaces: HashMap::new(), surface_order_for_picking: Vec::new() } }
        fn add_surface(&mut self, surface_arc: Arc<Mutex<Surface>>) {
            let id = surface_arc.lock().unwrap().id;
            self.surfaces.insert(id, surface_arc.clone());
            self.surface_order_for_picking.push(id); // Add to picking order
        }
        fn set_picking_order(&mut self, order: Vec<SurfaceId>) {
            self.surface_order_for_picking = order;
        }
    }
    impl crate::surface::surface_registry::SurfaceRegistryAccessor for MockSurfaceRegistry {
        fn get_surface(&self, id: SurfaceId) -> Option<Arc<Mutex<Surface>>> {
            self.surfaces.get(&id).cloned()
        }
        fn get_all_surface_ids(&self) -> Vec<SurfaceId> { // Used by real surface_at if not mocked
            self.surface_order_for_picking.clone()
        }
    }
    // Implement the conceptual trait for focus.rs surface_at if it was defined there.
    // For now, surface_at directly uses get_all_surface_ids from SurfaceRegistryAccessor.

    fn create_test_surface_for_focus(client_id: ClientId, pos: (i32,i32), size: (u32,u32)) -> Arc<Mutex<Surface>> {
        let mut s = Surface::new(client_id);
        s.current_attributes.position = pos;
        s.current_attributes.size = size;
        // Input region is None (infinite) by default
        Arc::new(Mutex::new(s))
    }

    #[test]
    fn test_update_pointer_focus_enter_leave() {
        let client1 = test_client_id(1);
        let client2 = test_client_id(2);
        let seat_id = SeatId(0);

        let mut seat_state = SeatState::new(seat_id, "seat0".to_string(), SeatCapability::Pointer);

        let mut surface_registry = MockSurfaceRegistry::new();
        let s1_arc = create_test_surface_for_focus(client1, (0,0), (100,100));
        let s1_id = s1_arc.lock().unwrap().id;
        surface_registry.add_surface(s1_arc.clone());

        let s2_arc = create_test_surface_for_focus(client2, (100,0), (100,100));
        let s2_id = s2_arc.lock().unwrap().id;
        surface_registry.add_surface(s2_arc.clone());

        // Mock active WlPointers (object_id -> Arc<Mutex<WlPointer>>)
        let mut pointer_interfaces: HashMap<u32, Arc<Mutex<WlPointer>>> = HashMap::new();
        let pointer1_obj_id = 1001;
        let pointer1_arc = Arc::new(Mutex::new(WlPointer::new(pointer1_obj_id, client1, seat_id)));
        pointer_interfaces.insert(pointer1_obj_id, pointer1_arc.clone());

        let pointer2_obj_id = 1002;
        let pointer2_arc = Arc::new(Mutex::new(WlPointer::new(pointer2_obj_id, client2, seat_id)));
        pointer_interfaces.insert(pointer2_obj_id, pointer2_arc.clone());

        // 1. Initial focus update (e.g. pointer appears over S1)
        seat_state.update_pointer_focus(50.0, 50.0, 1, &surface_registry, &pointer_interfaces);
        assert_eq!(seat_state.pointer_focus, Some(s1_id));
        assert_eq!(pointer1_arc.lock().unwrap().last_enter_serial, 1); // Serial updated for client1's pointer
        assert_eq!(pointer2_arc.lock().unwrap().last_enter_serial, 0); // Client2 not focused

        // 2. Move pointer from S1 to S2
        seat_state.update_pointer_focus(150.0, 50.0, 2, &surface_registry, &pointer_interfaces);
        assert_eq!(seat_state.pointer_focus, Some(s2_id));
        // Conceptual: check send_leave for s1_id for client1's pointer
        // Conceptual: check send_enter for s2_id for client2's pointer
        assert_eq!(pointer1_arc.lock().unwrap().last_enter_serial, 1); // Unchanged for client1
        assert_eq!(pointer2_arc.lock().unwrap().last_enter_serial, 2); // Updated for client2

        // 3. Move pointer to empty space
        seat_state.update_pointer_focus(300.0, 50.0, 3, &surface_registry, &pointer_interfaces);
        assert!(seat_state.pointer_focus.is_none());
        // Conceptual: check send_leave for s2_id for client2's pointer
        assert_eq!(pointer2_arc.lock().unwrap().last_enter_serial, 2); // Unchanged
    }

    #[test]
    fn test_update_pointer_focus_motion_on_same_surface() {
        let client1 = test_client_id(1);
        let seat_id = SeatId(0);
        let mut seat_state = SeatState::new(seat_id, "seat0".to_string(), SeatCapability::Pointer);
        let mut surface_registry = MockSurfaceRegistry::new();
        let s1_arc = create_test_surface_for_focus(client1, (0,0), (100,100));
        let s1_id = s1_arc.lock().unwrap().id;
        surface_registry.add_surface(s1_arc.clone());

        let mut pointer_interfaces: HashMap<u32, Arc<Mutex<WlPointer>>> = HashMap::new();
        let pointer1_obj_id = 1001;
        let pointer1_arc = Arc::new(Mutex::new(WlPointer::new(pointer1_obj_id, client1, seat_id)));
        pointer_interfaces.insert(pointer1_obj_id, pointer1_arc.clone());

        seat_state.update_pointer_focus(50.0, 50.0, 1, &surface_registry, &pointer_interfaces);
        assert_eq!(seat_state.pointer_focus, Some(s1_id));
        assert_eq!(pointer1_arc.lock().unwrap().last_enter_serial, 1);

        seat_state.update_pointer_focus(60.0, 60.0, 2, &surface_registry, &pointer_interfaces);
        assert_eq!(seat_state.pointer_focus, Some(s1_id)); // Focus remains
        assert_eq!(pointer1_arc.lock().unwrap().last_enter_serial, 1); // Serial for enter does not change on motion
        // Conceptual: check send_motion for s1_id for client1's pointer
    }

    #[test]
    fn test_set_keyboard_focus_enter_leave() {
        let client1 = test_client_id(1);
        let client2 = test_client_id(2);
        let seat_id = SeatId(0);
        let mut seat_state = SeatState::new(seat_id, "seat0".to_string(), SeatCapability::Keyboard);

        let mut surface_registry = MockSurfaceRegistry::new();
        let s1_arc = create_test_surface_for_focus(client1, (0,0), (100,100));
        let s1_id = s1_arc.lock().unwrap().id;
        surface_registry.add_surface(s1_arc.clone());

        let s2_arc = create_test_surface_for_focus(client2, (100,0), (100,100));
        let s2_id = s2_arc.lock().unwrap().id;
        surface_registry.add_surface(s2_arc.clone());

        let mut keyboard_interfaces: HashMap<u32, Arc<Mutex<WlKeyboard>>> = HashMap::new();
        let kbd1_obj_id = 2001;
        let kbd1_arc = Arc::new(Mutex::new(WlKeyboard::new(kbd1_obj_id, client1, seat_id)));
        keyboard_interfaces.insert(kbd1_obj_id, kbd1_arc.clone());
        let kbd2_obj_id = 2002;
        let kbd2_arc = Arc::new(Mutex::new(WlKeyboard::new(kbd2_obj_id, client2, seat_id)));
        keyboard_interfaces.insert(kbd2_obj_id, kbd2_arc.clone());

        // 1. Set focus to S1
        seat_state.set_keyboard_focus(Some(s1_id), 1, &surface_registry, &keyboard_interfaces);
        assert_eq!(seat_state.keyboard_focus, Some(s1_id));
        assert_eq!(kbd1_arc.lock().unwrap().last_enter_serial, 1);
        assert_eq!(kbd2_arc.lock().unwrap().last_enter_serial, 0);

        // 2. Set focus to S2
        seat_state.set_keyboard_focus(Some(s2_id), 2, &surface_registry, &keyboard_interfaces);
        assert_eq!(seat_state.keyboard_focus, Some(s2_id));
        // Conceptual: check leave for S1 (client1), enter for S2 (client2)
        assert_eq!(kbd1_arc.lock().unwrap().last_enter_serial, 1); // Unchanged
        assert_eq!(kbd2_arc.lock().unwrap().last_enter_serial, 2); // Updated

        // 3. Set focus to None
        seat_state.set_keyboard_focus(None, 3, &surface_registry, &keyboard_interfaces);
        assert!(seat_state.keyboard_focus.is_none());
        // Conceptual: check leave for S2 (client2)
        assert_eq!(kbd2_arc.lock().unwrap().last_enter_serial, 2); // Unchanged
    }
}
