//! Represents a `wl_keyboard` object, its state, and associated events.

use crate::input::seat::SeatId;
use crate::surface::SurfaceId;
use novade_buffer_manager::ClientId;
use std::os::unix::io::RawFd; // For keymap_fd

// TODO: Define error types for keyboard operations if necessary.

/// Represents the state of keyboard modifiers.
///
/// Corresponds to the state sent in `wl_keyboard.modifiers` events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModifiersState {
    /// Bitmask of currently depressed modifiers.
    pub depressed: u32,
    /// Bitmask of latched modifiers (modifiers that activate on next key press).
    pub latched: u32,
    /// Bitmask of locked modifiers (modifiers that remain active until unset).
    pub locked: u32,
    /// Logical group of the layout (effective group after processing depressed, latched, and locked).
    pub group: u32,
}

/// Represents a server-side `wl_keyboard` resource.
///
/// Each client that binds to `wl_seat` and requests a keyboard gets one of these.
/// It tracks client-specific keyboard state, keymaps, and repeat information.
#[derive(Debug, Clone)]
pub struct WlKeyboard {
    /// The Wayland object ID for this specific `wl_keyboard` instance, unique per client.
    pub object_id: u32,
    /// The ID of the client that owns this `wl_keyboard` resource.
    pub client_id: ClientId,
    /// The ID of the `wl_seat` this keyboard belongs to.
    pub seat_id: SeatId,

    /// Format of the keymap. Typically `wl_keyboard.keymap_format.xkb_v1`.
    pub keymap_format: Option<u32>, // u32 to match wl_keyboard.keymap_format enum
    /// File descriptor for the keymap data (typically an XKB keymap).
    /// `None` if no keymap has been sent or if it's not FD-based.
    pub keymap_fd: Option<RawFd>, // Placeholder, actual FD management is complex
    /// Size of the keymap data in bytes.
    pub keymap_size: Option<u32>,

    /// Key repeat rate in characters per second. 0 disables repeat.
    pub repeat_rate: i32,
    /// Delay in milliseconds before key repeat starts.
    pub repeat_delay: i32,

    /// Current state of keyboard modifiers for this client.
    pub modifiers_state: ModifiersState,

    /// The serial number of the last `wl_keyboard.enter` event sent to the client.
    pub last_enter_serial: u32,
}

impl WlKeyboard {
    /// Creates a new `WlKeyboard` instance with default repeat info.
    ///
    /// Keymap information is typically sent immediately after creation by the compositor.
    pub fn new(object_id: u32, client_id: ClientId, seat_id: SeatId) -> Self {
        Self {
            object_id,
            client_id,
            seat_id,
            keymap_format: None, // Will be set by send_keymap
            keymap_fd: None,     // Will be set by send_keymap
            keymap_size: None,   // Will be set by send_keymap
            repeat_rate: 0,      // Default: repeat disabled or to be set by send_repeat_info
            repeat_delay: 0,     // Default: repeat disabled or to be set by send_repeat_info
            modifiers_state: ModifiersState::default(),
            last_enter_serial: 0, // Initialize, will be updated on enter.
        }
    }
}

// --- Event Data Structures (Conceptual for now) ---

/// Data for the `wl_keyboard.keymap` event.
#[derive(Debug, Clone)]
pub struct WlKeyboardKeymapEvent {
    /// Format of the keymap data (e.g., `wl_keyboard.keymap_format.xkb_v1`).
    pub format: u32,
    /// File descriptor containing the keymap.
    pub fd: RawFd,
    /// Size of the keymap data in bytes.
    pub size: u32,
}

/// Data for the `wl_keyboard.enter` event.
#[derive(Debug, Clone)]
pub struct WlKeyboardEnterEvent {
    /// Serial number of the enter event.
    pub serial: u32,
    /// The `SurfaceId` of the surface that gained keyboard focus.
    pub surface_id: SurfaceId,
    /// Array of keycodes for keys currently pressed when focus entered.
    pub keys_pressed: Vec<u32>, // Represents wl_array in C
}

/// Data for the `wl_keyboard.leave` event.
#[derive(Debug, Clone, Copy)]
pub struct WlKeyboardLeaveEvent {
    /// Serial number of the leave event.
    pub serial: u32,
    /// The `SurfaceId` of the surface that lost keyboard focus.
    pub surface_id: SurfaceId,
}

/// Data for the `wl_keyboard.key` event.
#[derive(Debug, Clone, Copy)]
pub struct WlKeyboardKeyEvent {
    /// Serial number of the key event.
    pub serial: u32,
    /// Timestamp of the event with millisecond granularity.
    pub time_ms: u32,
    /// Key code that was pressed or released (e.g., from `input-event-codes.h`).
    pub key_code: u32,
    /// State of the key (`WlKeyboardKeyState`).
    pub state: WlKeyboardKeyState,
}

/// State of a key. Corresponds to `wl_keyboard.key_state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WlKeyboardKeyState {
    Released = 0,
    Pressed = 1,
}

/// Data for the `wl_keyboard.modifiers` event.
#[derive(Debug, Clone, Copy)]
pub struct WlKeyboardModifiersEvent {
    /// Serial number of the modifiers event.
    pub serial: u32,
    /// Bitmask of currently depressed modifiers.
    pub mods_depressed: u32,
    /// Bitmask of latched modifiers.
    pub mods_latched: u32,
    /// Bitmask of locked modifiers.
    pub mods_locked: u32,
    /// Logical keyboard group.
    pub group: u32,
}

/// Data for the `wl_keyboard.repeat_info` event.
#[derive(Debug, Clone, Copy)]
pub struct WlKeyboardRepeatInfoEvent {
    /// Key repeat rate in characters per second.
    pub rate: i32,
    /// Delay in milliseconds before key repeat starts.
    pub delay: i32,
}

// --- Conceptual Event Sending Functions ---

/// Conceptually sends the `wl_keyboard.keymap` event to the client.
///
/// In a real implementation, this would involve creating a temporary file with XKB data,
/// sending the file descriptor, format, and size to the client via the Wayland protocol.
pub fn send_keymap(keyboard_arc: Arc<Mutex<WlKeyboard>>) {
    let mut keyboard = keyboard_arc.lock().unwrap();
    // For this subtask, we simulate setting up the keymap details.
    keyboard.keymap_format = Some(1); // wl_keyboard.keymap_format.xkb_v1
    // In a real scenario:
    // 1. Generate XKB keymap (e.g., using xkbcommon).
    // 2. Write it to a temporary file or memfd.
    // 3. Get the RawFd and size.
    // keyboard.keymap_fd = Some(actual_fd);
    // keyboard.keymap_size = Some(actual_size);
    // For now, these remain None or placeholders.
    println!(
        "Conceptual: Seat [{}], Keyboard [{}], Client [{}]: Sending wl_keyboard.keymap (format: {:?}, fd: {:?}, size: {:?})",
        keyboard.seat_id.0, keyboard.object_id, keyboard.client_id, keyboard.keymap_format, keyboard.keymap_fd, keyboard.keymap_size
    );
    // Here, the actual Wayland message would be sent.
}

/// Conceptually sends the `wl_keyboard.repeat_info` event to the client.
pub fn send_repeat_info(keyboard_arc: Arc<Mutex<WlKeyboard>>) {
    let mut keyboard = keyboard_arc.lock().unwrap();
    // Set default repeat info if not already customized.
    // These are common defaults but can be configurable.
    if keyboard.repeat_rate == 0 && keyboard.repeat_delay == 0 { // Check if not already set
        keyboard.repeat_rate = 25; // characters per second
        keyboard.repeat_delay = 400; // milliseconds
    }

    let event_data = WlKeyboardRepeatInfoEvent {
        rate: keyboard.repeat_rate,
        delay: keyboard.repeat_delay,
    };
    println!(
        "Conceptual: Seat [{}], Keyboard [{}], Client [{}]: Sending wl_keyboard.repeat_info (rate: {}, delay: {})",
        keyboard.seat_id.0, keyboard.object_id, keyboard.client_id, event_data.rate, event_data.delay
    );
    // Here, the actual Wayland message `wl_keyboard.repeat_info(rate, delay)` would be sent.
}

// Other conceptual event sending functions (enter, leave, key, modifiers) would follow a similar pattern.
pub fn send_enter_event_conceptual(
    _client_id: ClientId,
    _keyboard_obj_id: u32,
    _serial: u32,
    _surface_id: SurfaceId,
    _keys_pressed: Vec<u32>
) {
    // println!("Conceptual: Send wl_keyboard.enter to client {}, kbd {}, surface {}, serial {}, keys {:?}",
    //          _client_id.0, _keyboard_obj_id, _surface_id.0, _serial, _keys_pressed);
}

pub fn send_leave_event_conceptual(
    _client_id: ClientId,
    _keyboard_obj_id: u32,
    _serial: u32,
    _surface_id: SurfaceId
) {
    // println!("Conceptual: Send wl_keyboard.leave to client {}, kbd {}, surface {}, serial {}",
    //          _client_id.0, _keyboard_obj_id, _surface_id.0, _serial);
}

pub fn send_modifiers_event_conceptual(
    _client_id: ClientId,
    _keyboard_obj_id: u32,
    _serial: u32,
    _modifiers: &ModifiersState,
) {
    // println!("Conceptual: Send wl_keyboard.modifiers to client {}, kbd {}, serial {}, mods {:?}",
    //          _client_id.0, _keyboard_obj_id, _serial, _modifiers);
}
// etc.

// --- wl_keyboard Request Handlers ---

/// Handles the `wl_keyboard.release` request.
///
/// This request signifies that the client is destroying its `wl_keyboard` proxy object.
/// The server should remove this `WlKeyboard` instance from its list of active keyboards
/// to free up resources.
///
/// # Arguments
/// * `keyboard_arc`: An `Arc<Mutex<WlKeyboard>>` of the keyboard object to be released.
///                 The lock will be taken to access its `object_id`.
/// * `global_keyboard_map`: A mutable reference to the global map (e.g., in `CompositorState`
///                          or `InputManager`) that stores all active `WlKeyboard` objects,
///                          keyed by their Wayland object ID.
pub fn handle_release(
    keyboard_arc: Arc<Mutex<WlKeyboard>>,
    global_keyboard_map: &mut HashMap<u32, Arc<Mutex<WlKeyboard>>>,
) {
    let keyboard_object_id = { // Scope for lock
        let keyboard = keyboard_arc.lock().unwrap();
        keyboard.object_id
    };

    if global_keyboard_map.remove(&keyboard_object_id).is_some() {
        println!(
            "Keyboard object id {} released and removed from global map.",
            keyboard_object_id
        );
    } else {
        eprintln!(
            "Warning: Tried to release keyboard object id {}, but it was not found in the global map.",
            keyboard_object_id
        );
    }
    // Any associated resources within WlKeyboard (like keymap_fd if it were real and owned by this object)
    // would be cleaned up when the Arc is dropped, assuming proper Drop impls or manual cleanup here.
    // For RawFd, if it's managed (e.g., dup'd), it should be closed.
    // Currently, keymap_fd is Option<RawFd> and not actively managed beyond setting to None.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::seat::SeatId;
    use novade_buffer_manager::ClientId;
    use std::sync::Arc; // Required for Arc tests
    use std::collections::HashMap; // Required for HashMap in release test

    fn test_client_id() -> ClientId { ClientId::new(1) }

    #[test]
    fn test_wl_keyboard_new() {
        let kbd = WlKeyboard::new(1, test_client_id(), SeatId(0));
        assert_eq!(kbd.object_id, 1);
        assert_eq!(kbd.client_id, test_client_id());
        assert_eq!(kbd.seat_id, SeatId(0));
        assert!(kbd.keymap_format.is_none()); // Initially None
        assert!(kbd.keymap_fd.is_none());
        assert!(kbd.keymap_size.is_none());
        assert_eq!(kbd.repeat_rate, 0); // Default as per new()
        assert_eq!(kbd.repeat_delay, 0); // Default as per new()
        assert_eq!(kbd.modifiers_state, ModifiersState::default());
        assert_eq!(kbd.last_enter_serial, 0);
    }

    #[test]
    fn test_keyboard_initial_state_after_conceptual_sends() {
        // This test verifies the state set by the conceptual send_keymap and send_repeat_info
        let kbd_arc = Arc::new(Mutex::new(WlKeyboard::new(1, test_client_id(), SeatId(0))));

        // Simulate what handle_get_keyboard would do conceptually
        send_keymap(kbd_arc.clone());
        send_repeat_info(kbd_arc.clone());

        let kbd = kbd_arc.lock().unwrap();
        assert_eq!(kbd.keymap_format, Some(1), "Keymap format should be set by send_keymap");
        // keymap_fd and keymap_size are still None in conceptual send_keymap
        assert!(kbd.keymap_fd.is_none());
        assert!(kbd.keymap_size.is_none());
        assert_eq!(kbd.repeat_rate, 25, "Repeat rate should be set by send_repeat_info");
        assert_eq!(kbd.repeat_delay, 400, "Repeat delay should be set by send_repeat_info");
    }

    #[test]
    fn test_handle_keyboard_release() {
        let client_a = test_client_id();
        let seat_id = SeatId(0);
        let keyboard_object_id = 301;
        let keyboard_arc = Arc::new(Mutex::new(WlKeyboard::new(keyboard_object_id, client_a, seat_id)));

        let mut global_keyboards_map: HashMap<u32, Arc<Mutex<WlKeyboard>>> = HashMap::new();
        global_keyboards_map.insert(keyboard_object_id, keyboard_arc.clone());

        assert!(global_keyboards_map.contains_key(&keyboard_object_id));
        handle_release(keyboard_arc, &mut global_keyboards_map);
        assert!(!global_keyboards_map.contains_key(&keyboard_object_id));
    }
}
