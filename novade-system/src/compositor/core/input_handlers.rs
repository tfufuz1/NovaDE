// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

use smithay::{
    input::{Seat, SeatHandler, SeatState, pointer::CursorImageStatus},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::seat::WaylandFocus, // For Option<&WlSurface> as focus target type
    wayland::shell::xdg::ToplevelState as XdgToplevelState, // For activating windows
    desktop::Window, // To access ToplevelSurface via Window trait methods
};
// Corrected: NovadeCompositorState should be DesktopState
use crate::compositor::core::state::DesktopState;
use crate::compositor::shell::xdg_shell::types::ManagedWindow; // For find_managed_window_by_wl_surface
use std::sync::Arc;
use smithay::{
    backend::input::{ButtonState, MouseButton},
    input::{
        pointer::{AxisFrame, MotionEvent, PointerHandle, RelativeMotionEvent},
        keyboard::KeyboardHandle, // For setting focus
    },
    reexports::wayland_server::{Serial, Weak}, // Added Serial, Weak
    utils::SERIAL_COUNTER, // For generating serials for events
};

// Helper to find ManagedWindow by WlSurface from DesktopState.
// TODO: Consider moving this to a shared module if used in more places (e.g., xdg_shell/handlers.rs also has it).
fn find_managed_window_by_wl_surface_from_input_handler(desktop_state: &DesktopState, surface: &WlSurface) -> Option<Arc<ManagedWindow>> {
    desktop_state.windows.values()
        .find(|win_arc| win_arc.wl_surface().as_ref() == Some(surface))
        .cloned()
}

impl SeatHandler for DesktopState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    // This `focus_changed` method is called by Smithay when keyboard focus changes,
    // e.g., as a result of `KeyboardHandle::set_focus`.
    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&Self::KeyboardFocus>, old_focus: Option<&Self::KeyboardFocus>) {
        tracing::debug!("(Input Handler) Keyboard focus changed. New: {:?}, Old: {:?}", focused.map(|s| s.id()), old_focus.map(|s| s.id()));

        self.active_input_surface = focused.map(|s| s.downgrade());

        // Deactivate old XDG toplevel focus (e.g., update window decorations)
        if let Some(old_surface) = old_focus {
            if old_surface.alive() {
                if let Some(window_arc) = find_managed_window_by_wl_surface_from_input_handler(self, old_surface) {
                    if let Some(toplevel) = window_arc.xdg_surface.toplevel() {
                        toplevel.with_pending_state(|xdg_state| {
                            xdg_state.states.unset(XdgToplevelState::Activated);
                        });
                        toplevel.send_configure();
                        tracing::trace!("(Input Handler) Deactivated XDG Toplevel (surface {:?}) due to focus loss.", old_surface.id());
                    }
                    // Update our internal activated state for the ManagedWindow
                    let mut win_state = window_arc.state.write().unwrap();
                    win_state.activated = false;
                }
            }
        }

        // Activate new XDG toplevel focus
        if let Some(new_surface) = focused {
            if new_surface.alive() {
                if let Some(window_arc) = find_managed_window_by_wl_surface_from_input_handler(self, new_surface) {
                    if let Some(toplevel) = window_arc.xdg_surface.toplevel() {
                        toplevel.with_pending_state(|xdg_state| {
                            xdg_state.states.set(XdgToplevelState::Activated, true);
                        });
                        toplevel.send_configure();
                        tracing::trace!("(Input Handler) Activated XDG Toplevel (surface {:?}) due to focus gain.", new_surface.id());
                    }
                    // Update our internal activated state for the ManagedWindow
                    let mut win_state = window_arc.state.write().unwrap();
                    win_state.activated = true;

                    // Raise the window to the top.
                    self.space.raise_window(&window_arc, true);
                    tracing::trace!("(Input Handler) Raised window {:?} (surface {:?}) to top due to focus gain.", window_arc.id, new_surface.id());
                }
            }
        }
    }

    // `cursor_image` is primarily handled by the implementation in `compositor/core/state.rs`
    // because that implementation deals with texture loading for rendering.
    // If `delegate_seat!` uses that one, this one might not be called or can be minimal.
    // The `DesktopState` struct itself has `fn cursor_image` as part of `impl SeatHandler for DesktopState`.
    // That one is more detailed. This one would conflict if both are active for the same type.
    // Assuming the one in `core/state.rs` is the primary one.
    // fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
    //     tracing::trace!("(Input Handler Stub) Cursor image status changed: {:?}", image);
    // }


    // --- Pointer Event Callbacks from Smithay's Input Backend Dispatch ---

    fn pointer_motion_event(
        &mut self,
        seat: &Seat<Self>,
        event: &MotionEvent,
    ) {
        self.pointer_location = event.location; // Update global pointer location in DesktopState
        let time = event.time;
        let serial = SERIAL_COUNTER.next_serial(); // Generate a new serial for enter/leave/motion events

        let mut new_focus_details = None;
        if let Some((surface_under, surface_local_coords)) = self.space.surface_under(self.pointer_location, true) {
            if surface_under.alive() {
                new_focus_details = Some((surface_under.clone(), surface_local_coords));
            }
        }

        let pointer_handle = seat.get_pointer().expect("Pointer capability missing on seat for motion event.");

        // Handle enter/leave logic
        let old_focus_weak = self.pointer_focus.clone(); // self.pointer_focus is Option<Weak<WlSurface>> in DesktopState
        let new_focus_surface_opt = new_focus_details.as_ref().map(|(s, _)| s);

        if new_focus_surface_opt != old_focus_weak.as_ref().and_then(Weak::upgrade).as_ref() {
            if let Some(old_focus_strong) = old_focus_weak.and_then(Weak::upgrade) {
                if old_focus_strong.alive() {
                    pointer_handle.leave(&old_focus_strong, serial, time);
                    tracing::trace!("Pointer left surface {:?}", old_focus_strong.id());
                }
            }
            if let Some(new_focus_strong) = new_focus_surface_opt {
                 if new_focus_strong.alive() {
                    pointer_handle.enter(new_focus_strong, serial, time);
                    tracing::trace!("Pointer entered surface {:?}", new_focus_strong.id());
                }
            }
            self.pointer_focus = new_focus_surface_opt.map(|s| s.downgrade());
        }

        // Send motion event to the current surface
        if let Some((focused_surface, local_coords)) = new_focus_details {
            if focused_surface.alive() {
                 pointer_handle.motion(&focused_surface, serial, time, local_coords);
                 // Removed per-event tracing for motion to reduce log spam, but can be re-enabled for debugging.
                 // tracing::trace!("Pointer motion on surface {:?} at local {:?}, global {:?}", focused_surface.id(), local_coords, self.pointer_location);
            }
        }
        // If no surface is under the cursor, pointer_handle.motion is not called.
    }

    fn pointer_button_event(
        &mut self,
        seat: &Seat<Self>,
        event: &smithay::input::pointer::ButtonEvent,
    ) {
        let pointer_handle = seat.get_pointer().expect("Pointer capability missing on seat for button event.");
        let serial = event.serial;
        let time = event.time;
        let button_code = event.button;
        let button_state = event.state;

        let mouse_button = match button_code {
            0x110 | 272 => MouseButton::Left, // BTN_LEFT (libinput code | evdev code)
            0x111 | 273 => MouseButton::Right, // BTN_RIGHT
            0x112 | 274 => MouseButton::Middle, // BTN_MIDDLE
            _ => MouseButton::Other(button_code as u16),
        };

        if let Some(focused_surface_arc) = self.pointer_focus.as_ref().and_then(|w| w.upgrade()) {
            if focused_surface_arc.alive() {
                pointer_handle.button(&focused_surface_arc, serial, time, mouse_button, button_state);
                tracing::debug!("Pointer button {:?} ({:?}) state {:?} on surface {:?}. Serial: {:?}",
                              mouse_button, button_code, button_state, focused_surface_arc.id(), serial);

                // Basic focus switching on button press (typically left button)
                if mouse_button == MouseButton::Left && button_state == ButtonState::Pressed {
                    let keyboard = seat.get_keyboard().expect("Keyboard capability missing on seat for focus switch.");
                    // Check if the currently pointer-focused surface is also keyboard-focused.
                    // Smithay's KeyboardHandle::current_focus() returns Option<WlSurface>.
                    let current_kbd_focus_is_target = keyboard.current_focus().as_ref() == Some(&focused_surface_arc);

                    if !current_kbd_focus_is_target {
                        tracing::info!("Pointer button press on surface {:?}, setting keyboard focus.", focused_surface_arc.id());
                        // The `focus_changed` method of this SeatHandler will be called as a result of `set_focus`.
                        keyboard.set_focus(self, Some(focused_surface_arc.clone()), serial);
                    }
                }
            }
        } else {
            tracing::debug!("Pointer button {:?} ({:?}) state {:?} with no focused surface. Serial: {:?}",
                          mouse_button, button_code, button_state, serial);
        }
    }

    fn pointer_axis_event(
        &mut self,
        seat: &Seat<Self>,
        event: &smithay::input::pointer::AxisEvent,
    ) {
        let pointer_handle = seat.get_pointer().expect("Pointer capability missing on seat for axis event.");
        if let Some(focused_surface_arc) = self.pointer_focus.as_ref().and_then(|w| w.upgrade()) {
            if focused_surface_arc.alive() {
                // Construct AxisFrame from event.
                // Smithay 0.10 AxisEvent provides absolute, discrete, and stop fields.
                let mut axis_frame = AxisFrame::new(event.time).source(event.source);
                if let Some(h_abs) = event.absolute.get(smithay::input::pointer::Axis::Horizontal) {
                    axis_frame = axis_frame.value(smithay::input::pointer::Axis::Horizontal, h_abs);
                }
                if let Some(v_abs) = event.absolute.get(smithay::input::pointer::Axis::Vertical) {
                     axis_frame = axis_frame.value(smithay::input::pointer::Axis::Vertical, v_abs);
                }
                if let Some(h_disc) = event.discrete.get(smithay::input::pointer::Axis::Horizontal) {
                    axis_frame = axis_frame.discrete(smithay::input::pointer::Axis::Horizontal, h_disc);
                }
                if let Some(v_disc) = event.discrete.get(smithay::input::pointer::Axis::Vertical) {
                     axis_frame = axis_frame.discrete(smithay::input::pointer::Axis::Vertical, v_disc);
                }
                if event.stop.get(smithay::input::pointer::Axis::Horizontal) { // Check if stop is Some(true) or just exists
                    axis_frame = axis_frame.stop(smithay::input::pointer::Axis::Horizontal);
                }
                if event.stop.get(smithay::input::pointer::Axis::Vertical) {
                    axis_frame = axis_frame.stop(smithay::input::pointer::Axis::Vertical);
                }

                pointer_handle.axis(&focused_surface_arc, axis_frame);
                // tracing::trace!("Pointer axis event on surface {:?}: {:?}", focused_surface_arc.id(), event);
            }
        }
    }

    // Stub for touch focus changed, as touch handling is not part of this MVP task.
    fn touch_focus_changed(&mut self, _seat: &Seat<Self>, focused: Option<&Self::TouchFocus>, _old_focus: Option<&Self::TouchFocus>) {
        tracing::trace!("Touch focus changed to: {:?}. (Handler in input_handlers.rs - Touch not implemented in MVP)", focused.map(|s| s.id()));
    }

    // Remove pointer_focus_changed stub if pointer_motion_event handles focus changes (enter/leave).
    // Smithay 0.10 SeatHandler does not have `pointer_focus_changed`. Enter/leave is managed via motion.
    // fn pointer_focus_changed(&mut self, _seat: &Seat<Self>, focused: Option<&Self::PointerFocus>, _old_focus: Option<&Self::PointerFocus>) {
    //     tracing::trace!("Pointer focus changed to: {:?}", focused.map(|s| s.id()));
    // }
}
