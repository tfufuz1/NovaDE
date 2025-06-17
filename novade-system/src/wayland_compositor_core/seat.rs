use super::object::{WaylandObject, ObjectId, ProtocolError, RequestContext, ObjectManager}; // Changed
use super::surface::Surface; // Changed
use super::wire::{self, WlArgument, MessageHeader}; // Changed
use std::sync::{Arc, Mutex, Weak};
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd}; // For keymap FD
use std::fs::File;
use std::io::Write;
use tempfile::tempfile; // For creating a temporary keymap file

// --- WlKeyboard ---
#[derive(Debug)]
pub struct WlKeyboard {
    id: ObjectId,
    version: u32,
    seat: Weak<WlSeat>, // Reference back to the seat
    // Modifiers state could be here or in WlSeat
}

impl WlKeyboard {
    pub fn new(id: ObjectId, version: u32, seat: Weak<WlSeat>) -> Self {
        Self { id, version, seat }
    }

    // Event sending methods. These construct messages and push them to RequestContext.client_event_queue.

    pub fn send_keymap(&self, context: &mut RequestContext, format: u32, fd: RawFd, size: u32) -> Result<(), ProtocolError> {
        // wl_keyboard.keymap(format: uint, fd: fd, size: uint)
        let args = vec![
            WlArgument::Uint(format), // e.g., KEYMAP_FORMAT_XKB_V1 = 1
            WlArgument::Fd(fd),       // FD is dupped by client, server closes its end.
            WlArgument::Uint(size),
        ];
        let message = wire::serialize_message(self.id, 0, &args)
            .map_err(|_| ProtocolError::ImplementationError)?; // Serialization error
        context.client_event_queue.push(message);
        // The FD needs to be managed. Client dups it. Server should close its copy after sending.
        // For now, assume the FD passed in is owned and will be closed by caller if needed,
        // or that wire serialization for FDs implies they are sent and then can be closed.
        // A robust solution would involve FD lifecycle management.
        // For this subtask, the FD is passed raw.
        Ok(())
    }

    pub fn send_enter(&self, context: &mut RequestContext, serial: u32, surface_id: ObjectId, pressed_keys: Vec<u32>) -> Result<(), ProtocolError> {
        // wl_keyboard.enter(serial: uint, surface: object, keys: array)
        let keys_bytes: Vec<u8> = pressed_keys.iter().flat_map(|k| k.to_ne_bytes()).collect();
        let args = vec![
            WlArgument::Uint(serial),
            WlArgument::Object(surface_id),
            WlArgument::Array(keys_bytes),
        ];
        let message = wire::serialize_message(self.id, 1, &args).map_err(|_| ProtocolError::ImplementationError)?;
        context.client_event_queue.push(message);
        Ok(())
    }

    pub fn send_leave(&self, context: &mut RequestContext, serial: u32, surface_id: ObjectId) -> Result<(), ProtocolError> {
        // wl_keyboard.leave(serial: uint, surface: object)
        let args = vec![
            WlArgument::Uint(serial),
            WlArgument::Object(surface_id),
        ];
        let message = wire::serialize_message(self.id, 2, &args).map_err(|_| ProtocolError::ImplementationError)?;
        context.client_event_queue.push(message);
        Ok(())
    }

    pub fn send_key(&self, context: &mut RequestContext, serial: u32, time: u32, key_code: u32, state: u32) -> Result<(), ProtocolError> {
        // wl_keyboard.key(serial: uint, time: uint, key: uint, state: uint)
        // key_code is the raw scancode. state is 0 for release, 1 for press.
        let args = vec![
            WlArgument::Uint(serial),
            WlArgument::Uint(time),
            WlArgument::Uint(key_code),
            WlArgument::Uint(state), // WL_KEYBOARD_KEY_STATE_RELEASED or WL_KEYBOARD_KEY_STATE_PRESSED
        ];
        let message = wire::serialize_message(self.id, 3, &args).map_err(|_| ProtocolError::ImplementationError)?;
        context.client_event_queue.push(message);
        Ok(())
    }

    pub fn send_modifiers(&self, context: &mut RequestContext, serial: u32, mods_depressed: u32, mods_latched: u32, mods_locked: u32, group: u32) -> Result<(), ProtocolError> {
        // wl_keyboard.modifiers(serial: uint, mods_depressed: uint, mods_latched: uint, mods_locked: uint, group: uint)
        let args = vec![
            WlArgument::Uint(serial),
            WlArgument::Uint(mods_depressed),
            WlArgument::Uint(mods_latched),
            WlArgument::Uint(mods_locked),
            WlArgument::Uint(group),
        ];
        let message = wire::serialize_message(self.id, 4, &args).map_err(|_| ProtocolError::ImplementationError)?;
        context.client_event_queue.push(message);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    // ObjectManager is already imported via super::object above if it was public there
    // No, RequestContext takes ObjectManager, so it must be in scope.
    // Tests use ObjectManager directly.
    use super::object::ObjectManager; // Explicitly import for tests if not covered by main use
    use super::surface::Surface; // For focus testing, already in main use
    use std::sync::Arc;
    use std::os::unix::io::AsRawFd; // For tests involving FDs if any

    // Helper to create a WlSeat and an ObjectManager for tests
    fn setup_seat_test_environment() -> (Arc<WlSeat>, ObjectManager, Vec<Vec<u8>>) {
        let mut object_manager = ObjectManager::new();
        let seat_id = object_manager.generate_new_id(); // Use manager to get ID
        let seat_version = 1;
        let seat = Arc::new(WlSeat::new(seat_id, seat_version));
        object_manager.register_new_object(seat_id, seat.clone()).unwrap();
        let client_event_queue = Vec::new(); // Mock event queue
        (seat, object_manager, client_event_queue)
    }

    // Helper to create a simple surface for focus tests
    fn create_test_surface(id: ObjectId, version: u32, manager: &mut ObjectManager) -> Arc<Surface> {
        let surface = Arc::new(Surface::new(id, version));
        manager.register_new_object(id, surface.clone()).unwrap();
        surface
    }


    #[test]
    fn test_wl_seat_get_keyboard() {
        let (seat, mut object_manager, mut client_event_queue) = setup_seat_test_environment();
        let new_keyboard_id = 100; // Client requested ID

        let mut context = RequestContext {
            object_manager: &mut object_manager,
            client_id: 1, // Dummy client ID
            client_event_queue: &mut client_event_queue,
        };

        // Call get_keyboard request
        let get_kbd_args = vec![WlArgument::NewId(new_keyboard_id)];
        seat.handle_request(1, get_kbd_args, &mut context).unwrap();

        // Verify WlKeyboard object was created
        let kbd_obj = object_manager.get_object(new_keyboard_id).expect("Keyboard object not found");
        assert_eq!(kbd_obj.interface_name(), "wl_keyboard");

        // Verify seat stored the keyboard Arc
        assert!(seat.keyboard_obj.lock().unwrap().is_some());
        assert_eq!(seat.keyboard_obj.lock().unwrap().as_ref().unwrap().id(), new_keyboard_id);

        // Verify capabilities updated and event sent
        assert_eq!(*seat.capabilities.lock().unwrap(), SeatCapability::Keyboard as u32);
        assert_eq!(context.client_event_queue.len(), 2); // Should be 2: capabilities + keymap

        // Check capabilities event (opcode 0 for wl_seat)
        let caps_event_msg = &context.client_event_queue[0];
        let (header, args) = wire::deserialize_message(&mut caps_event_msg.as_slice(), &[wire::ArgType::Uint]).unwrap();
        assert_eq!(header.object_id, seat.id());
        assert_eq!(header.opcode(), 0); // capabilities opcode
        assert_eq!(args[0], WlArgument::Uint(SeatCapability::Keyboard as u32));

        // Check keymap event (opcode 0 for wl_keyboard)
        let keymap_event_msg = &context.client_event_queue[1];
        // Deserialize with fd - complex, let's just check header for now
        let (header_km, _args_km) = wire::deserialize_message(&mut keymap_event_msg.as_slice(), &[wire::ArgType::Uint, wire::ArgType::Fd, wire::ArgType::Uint]).unwrap();
        assert_eq!(header_km.object_id, new_keyboard_id);
        assert_eq!(header_km.opcode(), 0); // keymap opcode
    }

    #[test]
    fn test_wl_seat_capabilities_change() {
        let (seat, _object_manager, mut client_event_queue) = setup_seat_test_environment();
        let mut context = RequestContext {
            object_manager: &mut ObjectManager::new(), // Dummy for this test
            client_id: 1,
            client_event_queue: &mut client_event_queue,
        };

        seat.add_capability(SeatCapability::Pointer, &mut context).unwrap();
        assert_eq!(*seat.capabilities.lock().unwrap(), SeatCapability::Pointer as u32);
        assert_eq!(context.client_event_queue.len(), 1); // capabilities event

        context.client_event_queue.clear();
        seat.add_capability(SeatCapability::Keyboard, &mut context).unwrap();
        assert_eq!(*seat.capabilities.lock().unwrap(), (SeatCapability::Pointer | SeatCapability::Keyboard) as u32);
        assert_eq!(context.client_event_queue.len(), 1);

        context.client_event_queue.clear();
        seat.remove_capability(SeatCapability::Pointer, &mut context).unwrap();
        assert_eq!(*seat.capabilities.lock().unwrap(), SeatCapability::Keyboard as u32);
        assert_eq!(context.client_event_queue.len(), 1);
    }

    #[test]
    fn test_wl_seat_keyboard_focus_events() {
        let (seat, mut object_manager, mut client_event_queue) = setup_seat_test_environment();
        let surface1_id = 201;
        let surface2_id = 202;
        let surface1 = create_test_surface(surface1_id, 1, &mut object_manager);
        let _surface2 = create_test_surface(surface2_id, 1, &mut object_manager); // Not used directly, but for a second focus target

        // First, ensure a keyboard exists for the seat
        let kbd_id = 101;
        let kbd = Arc::new(WlKeyboard::new(kbd_id, 1, Arc::downgrade(&seat)));
        object_manager.register_new_object(kbd_id, kbd.clone()).unwrap();
        *seat.keyboard_obj.lock().unwrap() = Some(kbd);


        let mut context = RequestContext {
            object_manager: &mut object_manager,
            client_id: 1,
            client_event_queue: &mut client_event_queue,
        };

        // Set focus to surface1
        seat.set_keyboard_focus(surface1.clone(), &mut context).unwrap();
        assert_eq!(seat.keyboard_focus.lock().unwrap().as_ref().unwrap().id(), surface1_id);
        assert_eq!(context.client_event_queue.len(), 1); // Enter event for surface1

        // Check enter event
        let enter_event_msg = &context.client_event_queue[0];
        let (header, args) = wire::deserialize_message(&mut enter_event_msg.as_slice(), &[wire::ArgType::Uint, wire::ArgType::Object, wire::ArgType::Array]).unwrap();
        assert_eq!(header.object_id, kbd_id); // Event is from WlKeyboard
        assert_eq!(header.opcode(), 1);      // enter opcode
        match args[1] { WlArgument::Object(id) => assert_eq!(id, surface1_id), _ => panic!("Expected object arg for surface") };

        context.client_event_queue.clear();

        // Clear focus
        seat.clear_keyboard_focus(&mut context).unwrap();
        assert!(seat.keyboard_focus.lock().unwrap().is_none());
        assert_eq!(context.client_event_queue.len(), 1); // Leave event for surface1

        // Check leave event
        let leave_event_msg = &context.client_event_queue[0];
        let (header_l, args_l) = wire::deserialize_message(&mut leave_event_msg.as_slice(), &[wire::ArgType::Uint, wire::ArgType::Object]).unwrap();
        assert_eq!(header_l.object_id, kbd_id);
        assert_eq!(header_l.opcode(), 2); // leave opcode
        match args_l[1] { WlArgument::Object(id) => assert_eq!(id, surface1_id), _ => panic!("Expected object arg for surface") };
    }

    #[test]
    fn test_wl_keyboard_send_events() {
        let (_seat_arc, _om, mut client_event_queue) = setup_seat_test_environment();
        // WlKeyboard needs a weak ref to seat, but for sending events directly, it's not strictly needed if context is right.
        let keyboard = WlKeyboard::new(100, 1, Weak::new()); // Dummy ID, version, seat for this test

        let mut context = RequestContext {
            object_manager: &mut ObjectManager::new(), // Not used by these send methods directly
            client_id: 1,
            client_event_queue: &mut client_event_queue,
        };

        // Key event
        keyboard.send_key(&mut context, 123, 456, 30 /* 'a' keycode */, 1 /* pressed */).unwrap();
        assert_eq!(context.client_event_queue.len(), 1);
        let key_msg = &context.client_event_queue[0];
        let (header, args) = wire::deserialize_message(&mut key_msg.as_slice(), &[wire::ArgType::Uint, wire::ArgType::Uint, wire::ArgType::Uint, wire::ArgType::Uint]).unwrap();
        assert_eq!(header.object_id, keyboard.id());
        assert_eq!(header.opcode(), 3); // key opcode
        assert_eq!(args[2], WlArgument::Uint(30)); // key_code
        assert_eq!(args[3], WlArgument::Uint(1));  // state

        context.client_event_queue.clear();

        // Modifiers event
        keyboard.send_modifiers(&mut context, 124, 1, 0, 0, 0).unwrap(); // Shift pressed
        assert_eq!(context.client_event_queue.len(), 1);
        let mod_msg = &context.client_event_queue[0];
        let (header_m, args_m) = wire::deserialize_message(&mut mod_msg.as_slice(), &[wire::ArgType::Uint; 5]).unwrap();
        assert_eq!(header_m.object_id, keyboard.id());
        assert_eq!(header_m.opcode(), 4); // modifiers opcode
        assert_eq!(args_m[1], WlArgument::Uint(1)); // mods_depressed
    }

    #[test]
    fn test_wl_keyboard_release_clears_from_seat() {
        let (seat, mut object_manager, mut client_event_queue) = setup_seat_test_environment();
        let kbd_id = 105;

        let mut context = RequestContext {
            object_manager: &mut object_manager,
            client_id: 1,
            client_event_queue: &mut client_event_queue,
        };

        // Create keyboard via seat.get_keyboard
        seat.handle_request(1, vec![WlArgument::NewId(kbd_id)], &mut context).unwrap();
        let keyboard_obj_arc = object_manager.get_typed_object::<WlKeyboard>(kbd_id).unwrap();
        assert!(seat.keyboard_obj.lock().unwrap().is_some());

        // Release the keyboard
        keyboard_obj_arc.handle_request(0, vec![], &mut context).unwrap(); // release keyboard

        assert!(seat.keyboard_obj.lock().unwrap().is_none(), "Keyboard object should be cleared from seat after release");
        assert!(object_manager.get_object(kbd_id).is_err(), "Keyboard object should be destroyed from manager");
    }
}

impl WaylandObject for WlKeyboard {
    fn id(&self) -> ObjectId { self.id }
    fn version(&self) -> u32 { self.version }
    fn interface_name(&self) -> &'static str { "wl_keyboard" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> { self }

    fn handle_request(
        &self,
        opcode: u16,
        _args: Vec<WlArgument>, // release takes no args
        context: &mut RequestContext,
    ) -> Result<(), ProtocolError> {
        match opcode {
            0 => { // release
                // Client is destroying its wl_keyboard handle.
                // The actual keyboard capability of the seat might remain.
                // If this WlKeyboard object is referred to by WlSeat, that reference should be cleared.
                if let Some(seat_arc) = self.seat.upgrade() {
                    seat_arc.clear_keyboard_object_if_matches(self.id);
                }
                context.object_manager.destroy_object(self.id);
                Ok(())
            }
            _ => Err(ProtocolError::InvalidOpcode(opcode)),
        }
    }
}

// --- WlSeat ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SeatCapability {
    Pointer = 1,
    Keyboard = 2,
    Touch = 4,
}

#[derive(Debug)]
pub struct WlSeat {
    id: ObjectId,
    version: u32,
    // Using Mutex for internal mutable fields, as WaylandObject::handle_request takes &self.
    capabilities: Mutex<u32>, // Bitmask of SeatCapability
    keyboard_obj: Mutex<Option<Arc<WlKeyboard>>>, // The WlKeyboard object created for this seat
    // pointer_obj: Mutex<Option<Arc<WlPointer>>>,
    // touch_obj: Mutex<Option<Arc<WlTouch>>>,
    keyboard_focus: Mutex<Option<Arc<Surface>>>,
    // TODO: Store serials for events
    next_serial: Mutex<u32>,
}

impl WlSeat {
    pub fn new(id: ObjectId, version: u32) -> Self {
        Self {
            id,
            version,
            capabilities: Mutex::new(0), // Initially no capabilities
            keyboard_obj: Mutex::new(None),
            keyboard_focus: Mutex::new(None),
            next_serial: Mutex::new(0),
        }
    }

    fn get_serial(&self) -> u32 {
        let mut serial = self.next_serial.lock().unwrap();
        *serial += 1;
        *serial -1
    }

    pub fn add_capability(&self, cap: SeatCapability, context: &mut RequestContext) -> Result<(), ProtocolError> {
        let mut caps = self.capabilities.lock().unwrap();
        let old_caps = *caps;
        *caps |= cap as u32;
        if *caps != old_caps {
            self.send_capabilities(context, *caps)?;
        }
        Ok(())
    }

    pub fn remove_capability(&self, cap: SeatCapability, context: &mut RequestContext) -> Result<(), ProtocolError> {
        let mut caps = self.capabilities.lock().unwrap();
        let old_caps = *caps;
        *caps &= !(cap as u32);
        if *caps != old_caps {
            self.send_capabilities(context, *caps)?;
        }
        Ok(())
    }

    pub fn send_capabilities(&self, context: &mut RequestContext, capabilities_mask: u32) -> Result<(), ProtocolError> {
        // wl_seat.capabilities(capabilities: uint) - opcode 0
        let args = vec![WlArgument::Uint(capabilities_mask)];
        let message = wire::serialize_message(self.id, 0, &args).map_err(|_| ProtocolError::ImplementationError)?;
        context.client_event_queue.push(message);
        Ok(())
    }

    // Called by WlKeyboard when it's released, to break cycle / clear seat's reference
    pub(crate) fn clear_keyboard_object_if_matches(&self, keyboard_id: ObjectId) {
        let mut kb_obj_guard = self.keyboard_obj.lock().unwrap();
        if let Some(kb_arc) = &*kb_obj_guard {
            if kb_arc.id() == keyboard_id {
                *kb_obj_guard = None;
                // TODO: Update capabilities if keyboard was the only one
                // This would require a context to send capability event.
                // For now, assume capability change is handled separately.
            }
        }
    }

    pub fn set_keyboard_focus(&self, surface_arc: Arc<Surface>, context: &mut RequestContext) -> Result<(), ProtocolError> {
        let serial = self.get_serial();
        let mut kbd_focus_guard = self.keyboard_focus.lock().unwrap();

        // 1. Send leave to old focus (if any and different)
        if let Some(old_focus_surf) = kbd_focus_guard.as_ref() {
            if old_focus_surf.id() != surface_arc.id() {
                 if let Some(keyboard) = self.keyboard_obj.lock().unwrap().as_ref() {
                    keyboard.send_leave(context, serial, old_focus_surf.id())?;
                }
            }
        }

        // 2. Update focus
        *kbd_focus_guard = Some(surface_arc.clone());

        // 3. Send enter to new focus
        if let Some(keyboard) = self.keyboard_obj.lock().unwrap().as_ref() {
            // TODO: Get current pressed keys array for the enter event.
            // This requires tracking system-wide key state or querying input backend.
            // For now, send an empty array.
            let pressed_keys_dummy = Vec::<u32>::new();
            keyboard.send_enter(context, serial, surface_arc.id(), pressed_keys_dummy)?;

            // TODO: Send current modifiers state as well after enter.
            // keyboard.send_modifiers(context, serial, ...)?;
        }
        Ok(())
    }

    pub fn clear_keyboard_focus(&self, context: &mut RequestContext) -> Result<(), ProtocolError> {
        let serial = self.get_serial();
        let mut kbd_focus_guard = self.keyboard_focus.lock().unwrap();
        if let Some(old_focus_surf) = kbd_focus_guard.take() { // take() clears the Option
            if let Some(keyboard) = self.keyboard_obj.lock().unwrap().as_ref() {
                keyboard.send_leave(context, serial, old_focus_surf.id())?;
            }
        }
        Ok(())
    }

    // Method for input backend to call
    pub fn handle_key_event(&self, time: u32, key_code: u32, state: u32, context: &mut RequestContext) -> Result<(), ProtocolError> {
        let kbd_focus_guard = self.keyboard_focus.lock().unwrap();
        if kbd_focus_guard.is_some() { // Check if there is a focused surface
            if let Some(keyboard) = self.keyboard_obj.lock().unwrap().as_ref() {
                let serial = self.get_serial();
                keyboard.send_key(context, serial, time, key_code, state)?;
                // TODO: Update and send modifier state after key event.
                // This would involve getting modifier state from input backend or xkbcommon.
                // keyboard.send_modifiers(context, serial, ...)?;
            }
        }
        Ok(())
    }
}

impl WaylandObject for WlSeat {
    fn id(&self) -> ObjectId { self.id }
    fn version(&self) -> u32 { self.version }
    fn interface_name(&self) -> &'static str { "wl_seat" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> { self }

    fn handle_request(
        &self, // Takes &self, uses internal mutability for state
        opcode: u16,
        args: Vec<WlArgument>,
        context: &mut RequestContext,
    ) -> Result<(), ProtocolError> {
        match opcode {
            0 => { // get_pointer(id: new_id)
                eprintln!("WlSeat {}: get_pointer - Unimplemented", self.id);
                // let new_pointer_id = match args[0] { WlArgument::NewId(id) => id, _ => return Err(ProtocolError::InvalidArguments) };
                // Create WlPointer, register it, store Arc in self.pointer_obj
                // Update and send capabilities if pointer capability was not set.
                Err(ProtocolError::ImplementationError) // Mark as unimplemented
            }
            1 => { // get_keyboard(id: new_id)
                if args.is_empty() { return Err(ProtocolError::InvalidArguments); }
                let new_keyboard_id = match args[0] { WlArgument::NewId(id) => id, _ => return Err(ProtocolError::InvalidArguments) };

                // Check if a keyboard already exists for this seat for this client.
                // A client should typically get one keyboard object per seat.
                // If one exists, spec is a bit unclear, could return existing or new one.
                // For simplicity, if one is already set, we might not create another or could error.
                // Let's assume we create a new one and it replaces the old one for this client's handle.
                // However, WlSeat stores one Option<Arc<WlKeyboard>>, implying one keyboard device.

                let mut kb_obj_guard = self.keyboard_obj.lock().unwrap();
                if kb_obj_guard.is_some() {
                    // A keyboard object already exists for this seat.
                    // Depending on compositor policy, could return existing, error, or replace.
                    // For now, let's say it's an error to request it again if already exists.
                    // Or, more Wayland-like: client gets a new ID, but it refers to the same underlying keyboard.
                    // What's stored in WlSeat is the *canonical* keyboard object for the seat's capability.
                    // This is simpler: if not yet created, create it. If requested again, client gets new ID for same conceptual keyboard.
                }

                let keyboard = Arc::new(WlKeyboard::new(new_keyboard_id, self.version, Arc::downgrade(&context.object_manager.get_typed_object::<WlSeat>(self.id)?)));
                context.object_manager.register_new_object(new_keyboard_id, keyboard.clone())?;

                *kb_obj_guard = Some(keyboard.clone()); // Store the canonical keyboard for the seat

                self.add_capability(SeatCapability::Keyboard, context)?;

                // Send initial keymap
                // For now, a dummy keymap. Real one needs xkbcommon.
                // Create a temp file for the keymap
                let mut keymap_file = tempfile().map_err(|_| ProtocolError::ImplementationError)?;
                let keymap_data = b"dummy_xkb_keymap_v1"; // Replace with actual XKB data
                keymap_file.write_all(keymap_data).map_err(|_| ProtocolError::ImplementationError)?;
                keymap_file.flush().map_err(|_| ProtocolError::ImplementationError)?;

                let fd_for_client = match nix::unistd::dup(keymap_file.as_raw_fd()) {
                    Ok(duped) => duped,
                    Err(_) => return Err(ProtocolError::ImplementationError),
                };

                keyboard.send_keymap(context, 1 /* XKB_V1 */, fd_for_client, keymap_data.len() as u32)?;
                // keymap_file (and its FD) is dropped here, closing server's original FD.
                // fd_for_client should be closed by the wire protocol after sending if not auto-closed.
                // For now, assume it's sent and client owns its duped copy.
                // If send_keymap doesn't close fd, then caller (here) should.
                // Let's assume send_keymap implies FD is consumed by wire. If not, must close fd_for_client.
                // For safety, let's close it if send_keymap doesn't claim ownership.
                // RawFd isn't owned, so we must close it.
                nix::unistd::close(fd_for_client).map_err(|_| ProtocolError::ImplementationError)?;


                // If there's a focused surface, send enter event.
                let kbd_focus_guard = self.keyboard_focus.lock().unwrap();
                if let Some(focus_surf) = kbd_focus_guard.as_ref() {
                    let serial = self.get_serial();
                     // TODO: Get current pressed keys array
                    keyboard.send_enter(context, serial, focus_surf.id(), Vec::new())?;
                }


                Ok(())
            }
            2 => { // get_touch(id: new_id)
                eprintln!("WlSeat {}: get_touch - Unimplemented", self.id);
                Err(ProtocolError::ImplementationError)
            }
            3 => { // release (destroys the wl_seat object)
                // This is less common for wl_seat as it's usually global.
                // If a client releases it, it means this client no longer wants to use this seat.
                // The seat itself (and its capabilities) should remain for other clients.
                // We just destroy this client's handle to the seat object.
                context.object_manager.destroy_object(self.id);
                Ok(())
            }
            _ => Err(ProtocolError::InvalidOpcode(opcode)),
        }
    }
}
