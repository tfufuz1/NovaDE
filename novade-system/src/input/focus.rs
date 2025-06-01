// src/input/focus.rs

use crate::wayland_server_module_placeholder::{
    ClientId, SurfaceId, WaylandServerHandle, SurfaceManagerHandle,
    PointerObjectId, KeyboardObjectId
};
use crate::input::keyboard::ModifiersState as InputModifierState;
use tracing::{debug, info, warn};
use std::collections::VecDeque; // Added for focus history

const MAX_FOCUS_HISTORY_SIZE: usize = 10;

#[derive(Debug, Clone, Copy, Default, PartialEq)] // Added PartialEq for history operations
struct FocusedElements {
    keyboard_surface: Option<SurfaceId>,
    keyboard_client: Option<ClientId>,
    pointer_surface: Option<SurfaceId>, // Pointer focus isn't typically part of Alt-Tab history
    pointer_client: Option<ClientId>,   // but kept for struct consistency.
}

#[derive(Debug, Clone, Copy)]
enum GrabType {
    KeyboardOnly,
    PointerOnly, // Typically implies keyboard focus might also follow or be managed by client
    Full,        // Both keyboard and pointer are grabbed
}

#[derive(Debug, Clone, Copy)]
struct GrabState {
    grabbed_surface: SurfaceId, // The surface that initiated the grab
    grab_client: ClientId,      // The client owning the grabbed surface
    grab_type: GrabType,
}

pub struct FocusManager {
    focused: FocusedElements,
    previous_focus_state: Option<FocusedElements>,
    grab_state: Option<GrabState>,
    focus_history: VecDeque<FocusedElements>, // For Alt-Tab like cycling

    pointer_x: f64,
    pointer_y: f64,
    wayland_handle: WaylandServerHandle,
    surface_manager: SurfaceManagerHandle,

    active_pointer_obj: Option<PointerObjectId>,
    active_keyboard_obj: Option<KeyboardObjectId>,
}

impl FocusManager {
    pub fn new(wayland_handle: WaylandServerHandle, surface_manager: SurfaceManagerHandle) -> Self {
        info!("FocusManager: Initializing...");
        Self {
            focused: FocusedElements::default(),
            previous_focus_state: None,
            grab_state: None,
            focus_history: VecDeque::with_capacity(MAX_FOCUS_HISTORY_SIZE),
            pointer_x: 0.0,
            pointer_y: 0.0,
            wayland_handle,
            surface_manager,
            active_pointer_obj: Some(PointerObjectId(1)),
            active_keyboard_obj: Some(KeyboardObjectId(1)),
        }
    }

    pub fn update_pointer_position(&mut self, new_x: f64, new_y: f64) {
        self.pointer_x = new_x;
        self.pointer_y = new_y;
        // debug!("FocusManager: Pointer position updated to ({:.2}, {:.2})", new_x, new_y);

        let mut target_surface_info = self.surface_manager.surface_at(new_x, new_y);

        // If grabbed, pointer events might be restricted or redirected
        if let Some(grab) = self.grab_state {
            match grab.grab_type {
                GrabType::PointerOnly | GrabType::Full => {
                    // For simplicity in this stub: if grabbed, all pointer events go to the grabbing surface's client.
                    // A real implementation would check if new_x, new_y is within the grab_surface or its sub-surfaces,
                    // or if the grab is "modal" in a way that all pointer input outside is ignored or sent to the grabber.
                    // We'll assume the pointer is now "on" the grabbed surface for event delivery purposes.
                    debug!("FocusManager: Input grab active ({:?}). Pointer events directed to grabbed surface {:?} (client {:?})",
                           grab.grab_type, grab.grabbed_surface, grab.grab_client);
                    target_surface_info = Some((grab.grabbed_surface, grab.grab_client));
                }
                _ => {} // KeyboardOnly grab doesn't affect pointer focus directly here
            }
        }

        let current_target_surface_id = target_surface_info.map(|(sid, _)| sid);
        if self.focused.pointer_surface != current_target_surface_id {
            let serial = self.wayland_handle.next_serial(); // Generate one serial for this focus change

            if let (Some(old_surface), Some(old_client), Some(pointer_obj)) =
                (self.focused.pointer_surface, self.focused.pointer_client, self.active_pointer_obj) {
                if old_surface != current_target_surface_id.unwrap_or(SurfaceId(0)) { // Avoid leave if re-entering same
                    info!("FocusManager: Pointer leaving surface {:?} (client {:?}), serial {}", old_surface, old_client, serial);
                    self.wayland_handle.send_wl_pointer_leave(old_client, pointer_obj, old_surface, serial);
                }
            }

            self.focused.pointer_surface = current_target_surface_id;
            self.focused.pointer_client = target_surface_info.map(|(_, cid)| cid);

            if let (Some(new_surface), Some(new_client), Some(pointer_obj)) =
                (self.focused.pointer_surface, self.focused.pointer_client, self.active_pointer_obj) {
                // TODO: sx, sy should be surface-local coordinates.
                let surface_local_x = new_x; // Placeholder
                let surface_local_y = new_y; // Placeholder
                info!("FocusManager: Pointer entering surface {:?} (client {:?}) at L({:.2}, {:.2}), serial {}",
                      new_surface, new_client, surface_local_x, surface_local_y, serial);
                self.wayland_handle.send_wl_pointer_enter(new_client, pointer_obj, new_surface, surface_local_x, surface_local_y, serial);
            }
            // debug!("FocusManager: Pointer focus changed to surface {:?} (client {:?})",
            //       self.focused.pointer_surface, self.focused.pointer_client);
        }
    }

    pub fn set_keyboard_focus(&mut self, surface_id: Option<SurfaceId>, client_id: Option<ClientId>) {
        // If a grab is active and includes keyboard, focus is forced to the grabber.
        if let Some(grab) = self.grab_state {
            match grab.grab_type {
                GrabType::KeyboardOnly | GrabType::Full => {
                    if surface_id != Some(grab.grabbed_surface) || client_id != Some(grab.grab_client) {
                        info!("FocusManager: Keyboard focus attempted change during grab, redirecting to grabbed surface {:?} (client {:?})",
                              grab.grabbed_surface, grab.grab_client);
                    }
                    // Force focus to the grabbing surface/client
                    self._do_set_keyboard_focus(Some(grab.grabbed_surface), Some(grab.grab_client));
                    return;
                }
                _ => {} // PointerOnly grab doesn't force keyboard focus here.
            }
        }
        self._do_set_keyboard_focus(surface_id, client_id, false); // false indicates not a grab-forced focus
    }

    fn _do_set_keyboard_focus(&mut self, surface_id: Option<SurfaceId>, client_id: Option<ClientId>, is_grab_forced: bool) {
        if self.focused.keyboard_surface == surface_id && self.focused.keyboard_client == client_id {
            // No change in focus, but ensure modifiers are up-to-date if requested (e.g. for a new client binding kbd)
             if let (Some(_new_surface), Some(new_client), Some(kbd_obj)) = (surface_id, client_id, self.active_keyboard_obj) {
                let serial = self.wayland_handle.next_serial();
                let (dummy_mods_depressed, dummy_mods_latched, dummy_mods_locked, dummy_group) = (0,0,0,0); // Placeholder
                self.wayland_handle.send_wl_keyboard_modifiers(new_client, kbd_obj, serial, dummy_mods_depressed, dummy_mods_latched, dummy_mods_locked, dummy_group);
             }
            return;
        }

        let old_focused_elements = self.focused; // Capture for history if it was valid

        let serial = self.wayland_handle.next_serial();

        if let (Some(old_surface), Some(old_client), Some(kbd_obj)) =
            (self.focused.keyboard_surface, self.focused.keyboard_client, self.active_keyboard_obj) {
            info!("FocusManager: Keyboard leaving surface {:?} (client {:?}), serial {}", old_surface, old_client, serial);
            self.wayland_handle.send_wl_keyboard_leave(old_client, kbd_obj, old_surface, serial);
        }

        self.focused.keyboard_surface = surface_id;
        self.focused.keyboard_client = client_id;
        // Pointer focus is not directly changed here, but could be if policy dictates (e.g. click-to-focus sets kbd focus)

        if let (Some(new_surface), Some(new_client), Some(kbd_obj)) =
            (self.focused.keyboard_surface, self.focused.keyboard_client, self.active_keyboard_obj) {
            info!("FocusManager: Keyboard entering surface {:?} (client {:?}), serial {}", new_surface, new_client, serial);
            let pressed_keys_on_enter: Vec<u32> = Vec::new();
            self.wayland_handle.send_wl_keyboard_enter(new_client, kbd_obj, new_surface, &pressed_keys_on_enter, serial);

            let (dummy_mods_depressed, dummy_mods_latched, dummy_mods_locked, dummy_group) = (0,0,0,0);
            self.wayland_handle.send_wl_keyboard_modifiers(new_client, kbd_obj, serial, dummy_mods_depressed, dummy_mods_latched, dummy_mods_locked, dummy_group);

            // Update focus history if this isn't a grab-forced focus and is a valid new focus target
            if !is_grab_forced {
                let new_focus_entry = FocusedElements {
                    keyboard_client: Some(new_client),
                    keyboard_surface: Some(new_surface),
                    pointer_surface: None, // History primarily tracks keyboard focus targets
                    pointer_client: None,
                };

                // Remove if already in history to bring to front
                self.focus_history.retain(|x| *x != new_focus_entry);
                self.focus_history.push_front(new_focus_entry);
                if self.focus_history.len() > MAX_FOCUS_HISTORY_SIZE {
                    self.focus_history.pop_back();
                }
                debug!("FocusManager: Updated focus history. New front: {:?}. History size: {}",
                       self.focus_history.front(), self.focus_history.len());
            }

        } else { // Focus set to None
             if !is_grab_forced && (old_focused_elements.keyboard_client.is_some() || old_focused_elements.keyboard_surface.is_some()) {
                // If focus was cleared (set to None) and it previously had a value,
                // we don't add a "None" entry to history, but the leave event is sent.
                // The history should only contain actual focus targets.
                 debug!("FocusManager: Keyboard focus cleared.");
             }
        }
        debug!("FocusManager: Keyboard focus is now: surface {:?} (client {:?})",
              self.focused.keyboard_surface, self.focused.keyboard_client);
    }

    // --- Input Grab Mechanism ---
    pub fn request_grab(&mut self, surface_id: SurfaceId, client_id: ClientId, grab_type: GrabType) {
        if self.grab_state.is_some() {
            warn!("FocusManager: Grab already active. Ignoring new grab request for surface {:?}.", surface_id);
            return;
        }

        self.previous_focus_state = Some(self.focused); // Store current focus state
        self.grab_state = Some(GrabState {
            grabbed_surface: surface_id,
            grab_client: client_id,
            grab_type,
        });
        info!("FocusManager: Input grab ({:?}) activated for surface {:?} (client {:?})", grab_type, surface_id, client_id);

        // If grab includes keyboard, force keyboard focus to the grabbing surface
        match grab_type {
            GrabType::KeyboardOnly | GrabType::Full => {
                info!("FocusManager: Grab includes keyboard, setting keyboard focus to grabbed surface.");
                self._do_set_keyboard_focus(Some(surface_id), Some(client_id), true); // true: is_grab_forced
            }
            _ => {}
        }
    }

    pub fn release_grab(&mut self) {
        if self.grab_state.is_none() {
            warn!("FocusManager: No active grab to release.");
            return;
        }
        let grab_was_forcing_kbd_focus = match self.grab_state.unwrap().grab_type { // Safe unwrap due to check above
            GrabType::KeyboardOnly | GrabType::Full => true,
            _ => false,
        };

        info!("FocusManager: Input grab released. Restoring previous focus if available.");
        self.grab_state = None;

        if let Some(prev_focus) = self.previous_focus_state.take() {
            self._do_set_keyboard_focus(prev_focus.keyboard_surface, prev_focus.keyboard_client, false);
            self.update_pointer_position(self.pointer_x, self.pointer_y);
        } else {
            // If no specific previous state, clear keyboard focus if it was forced by grab,
            // otherwise let it be (it might have been changed by other means during grab if not a full kbd grab).
            if grab_was_forcing_kbd_focus {
                 self._do_set_keyboard_focus(None, None, false);
            }
            self.update_pointer_position(self.pointer_x, self.pointer_y);
        }
    }

    // --- Focus History Cycling ---
    pub fn cycle_focus_forward(&mut self) {
        if self.grab_state.is_some() {
            warn!("FocusManager: Cannot cycle focus while a grab is active.");
            return;
        }
        if self.focus_history.len() > 1 {
            if let Some(current_focus_entry) = self.focus_history.pop_front() {
                self.focus_history.push_back(current_focus_entry); // Move current to back
                if let Some(next_focus_target) = self.focus_history.front() {
                    info!("FocusManager: Cycling focus forward to: client {:?}, surface {:?}",
                          next_focus_target.keyboard_client, next_focus_target.keyboard_surface);
                    self._do_set_keyboard_focus(next_focus_target.keyboard_surface, next_focus_target.keyboard_client, false);
                } else {
                     // Should not happen if len > 1 and we just pushed current_focus_entry back
                    error!("FocusManager: Focus history inconsistent during cycle_focus_forward.");
                }
            }
        } else {
            info!("FocusManager: Not enough elements in focus history to cycle forward (size: {}).", self.focus_history.len());
        }
    }

    pub fn cycle_focus_backward(&mut self) {
        if self.grab_state.is_some() {
            warn!("FocusManager: Cannot cycle focus while a grab is active.");
            return;
        }
        if self.focus_history.len() > 1 {
            if let Some(last_focus_entry) = self.focus_history.pop_back() {
                self.focus_history.push_front(last_focus_entry); // Move last to front
                 // The element that was just moved to the front is the new target.
                if let Some(new_focus_target) = self.focus_history.front() {
                    info!("FocusManager: Cycling focus backward to: client {:?}, surface {:?}",
                        new_focus_target.keyboard_client, new_focus_target.keyboard_surface);
                    self._do_set_keyboard_focus(new_focus_target.keyboard_surface, new_focus_target.keyboard_client, false);
                } else {
                    error!("FocusManager: Focus history inconsistent during cycle_focus_backward.");
                }
            }
        } else {
             info!("FocusManager: Not enough elements in focus history to cycle backward (size: {}).", self.focus_history.len());
        }
    }

    // --- Event Delivery Methods (modified for grab) ---
    pub fn deliver_pointer_motion(&self, time: u32, dx: f64, dy: f64) {
        let (target_client, target_pointer_obj) =
            if let Some(grab) = &self.grab_state {
                match grab.grab_type {
                    GrabType::PointerOnly | GrabType::Full => (Some(grab.grab_client), self.active_pointer_obj),
                    _ => (self.focused.pointer_client, self.active_pointer_obj), // Kbd only grab, pointer acts normally
                }
            } else {
                (self.focused.pointer_client, self.active_pointer_obj)
            };

        if let (Some(client), Some(pointer_obj)) = (target_client, target_pointer_obj) {
            // sx, sy for wl_pointer.motion are surface-local.
            // This requires transformation logic not yet in place. Using global for now.
            let surface_x = self.pointer_x;
            let surface_y = self.pointer_y;
            debug!("FocusManager: Delivering pointer motion (dx={:.2}, dy={:.2}, time={}) to client {:?} via obj {:?}. Target pos ({:.2}, {:.2})",
                  dx, dy, time, client, pointer_obj, surface_x, surface_y);
            self.wayland_handle.send_wl_pointer_motion(client, pointer_obj, time, surface_x, surface_y);
        } else {
            // debug!("FocusManager: No client/surface has pointer focus or grab, dropping motion event.");
        }
    }

    pub fn deliver_pointer_button(&self, time: u32, button: u32, state: u32) {
        let (target_client, target_pointer_obj) =
            if let Some(grab) = &self.grab_state {
                match grab.grab_type {
                    GrabType::PointerOnly | GrabType::Full => (Some(grab.grab_client), self.active_pointer_obj),
                    _ => (self.focused.pointer_client, self.active_pointer_obj),
                }
            } else {
                (self.focused.pointer_client, self.active_pointer_obj)
            };

        if let (Some(client), Some(pointer_obj)) = (target_client, target_pointer_obj) {
            info!("FocusManager: Delivering pointer button (button={}, state={}, time={}) to client {:?}",
                  button, state, time, client);
            let serial = self.wayland_handle.next_serial();
            self.wayland_handle.send_wl_pointer_button(client, pointer_obj, time, button, state, serial);
        } else {
            // debug!("FocusManager: No client/surface has pointer focus or grab, dropping button event.");
        }
    }

    pub fn deliver_pointer_axis(&self, time: u32, axis: u32, value: f64) {
        let (target_client, target_pointer_obj) =
            if let Some(grab) = &self.grab_state {
                match grab.grab_type {
                    GrabType::PointerOnly | GrabType::Full => (Some(grab.grab_client), self.active_pointer_obj),
                    _ => (self.focused.pointer_client, self.active_pointer_obj),
                }
            } else {
                (self.focused.pointer_client, self.active_pointer_obj)
            };

        if let (Some(client), Some(pointer_obj)) = (target_client, target_pointer_obj) {
            info!("FocusManager: Delivering pointer axis (axis={}, value={:.2}, time={}) to client {:?}",
                  axis, value, time, client);
            self.wayland_handle.send_wl_pointer_axis(client, pointer_obj, time, axis, value);
        } else {
            // debug!("FocusManager: No client/surface has pointer focus or grab, dropping axis event.");
        }
    }

    pub fn deliver_keyboard_key(&self, time: u32, key: u32, state: u32, current_mods: Option<InputModifierState>) {
        let (target_client, target_kbd_obj) =
            if let Some(grab) = &self.grab_state {
                match grab.grab_type {
                    GrabType::KeyboardOnly | GrabType::Full => (Some(grab.grab_client), self.active_keyboard_obj),
                    _ => (self.focused.keyboard_client, self.active_keyboard_obj), // Ptr only grab, kbd acts normally
                }
            } else {
                (self.focused.keyboard_client, self.active_keyboard_obj)
            };

        if let (Some(client), Some(kbd_obj)) = (target_client, target_kbd_obj) {
            let serial = self.wayland_handle.next_serial();
            if let Some(mods) = current_mods {
                 self.wayland_handle.send_wl_keyboard_modifiers(
                    client, kbd_obj, serial,
                    mods.depressed.bits(),
                    mods.latched.bits(),
                    mods.locked.bits(),
                    0 // group - TODO: get from xkb_state.group()
                );
            }
            info!("FocusManager: Delivering keyboard key (key={}, state={}, time={}) to client {:?}",
                  key, state, time, client);
            self.wayland_handle.send_wl_keyboard_key(client, kbd_obj, time, key, state, serial);
        } else {
            // debug!("FocusManager: No client has keyboard focus or grab, dropping key event.");
        }
    }

    pub fn set_active_wl_objects(&mut self, pointer: Option<PointerObjectId>, keyboard: Option<KeyboardObjectId>) {
        self.active_pointer_obj = pointer;
        self.active_keyboard_obj = keyboard;
        info!("FocusManager: Active Wayland objects updated: pointer={:?}, keyboard={:?}", pointer, keyboard);
    }
}
