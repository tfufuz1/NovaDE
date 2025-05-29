// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

use smithay::{
    input::{Seat, SeatHandler, SeatState, pointer::CursorImageStatus},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::seat::WaylandFocus, // For Option<&WlSurface> as focus target type
    wayland::shell::xdg::ToplevelState as XdgToplevelState, // For activating windows
    desktop::Window, // To access ToplevelSurface via Window trait methods
};
use crate::compositor::core::state::NovadeCompositorState;
use std::sync::Arc; // Mutex is not directly used here, current_cursor_status is Mutex in NovadeCompositorState

impl SeatHandler for NovadeCompositorState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface; // Assuming WlSurface for touch focus as well

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&Self::KeyboardFocus>, old_focus: Option<&Self::KeyboardFocus>) {
        tracing::debug!("Keyboard focus changed. New: {:?}, Old: {:?}", focused.map(|s| s.id()), old_focus.map(|s| s.id()));

        // Deactivate old focus
        if let Some(old_surface) = old_focus {
            if old_surface.alive() { // Check if the surface still exists
                // Find the ManagedWindow for the old_surface
                if let Some(window_arc) = self.windows.values().find(|w| w.wl_surface().as_ref() == Some(old_surface)).cloned() {
                    let mut win_state = window_arc.state.write().unwrap();
                    if win_state.activated {
                        win_state.activated = false;
                        drop(win_state); // Release lock before calling configure

                        if let Some(toplevel) = window_arc.xdg_surface.toplevel() {
                            toplevel.with_pending_state(|xdg_state| {
                                xdg_state.states.unset(XdgToplevelState::Activated);
                            });
                            toplevel.send_configure();
                            tracing::trace!("Deactivated window: {:?}", window_arc.id);
                        }
                    }
                }
            }
        }

        // Activate new focus
        if let Some(new_surface) = focused {
            if new_surface.alive() { // Check if the surface still exists
                // Find the ManagedWindow for the new_surface
                if let Some(window_arc) = self.windows.values().find(|w| w.wl_surface().as_ref() == Some(new_surface)).cloned() {
                    let mut win_state = window_arc.state.write().unwrap();
                    if !win_state.activated {
                        win_state.activated = true;
                        drop(win_state); // Release lock

                        if let Some(toplevel) = window_arc.xdg_surface.toplevel() {
                            toplevel.with_pending_state(|xdg_state| {
                                xdg_state.states.set(XdgToplevelState::Activated, true);
                            });
                            toplevel.send_configure();
                            tracing::trace!("Activated window: {:?}", window_arc.id);
                        }
                    }
                    // Raise the window to the top.
                    // The `Window` trait is implemented for `ManagedWindow`.
                    self.space.raise_window(&window_arc, true);
                    tracing::trace!("Raised window {:?} to top.", window_arc.id);
                }
            }
        }
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        tracing::trace!("Cursor image status changed: {:?}", image);
        let mut guard = self.current_cursor_status.lock().unwrap();
        *guard = image;
    }

    // Basic logging for other focus events
    fn pointer_focus_changed(&mut self, _seat: &Seat<Self>, focused: Option<&Self::PointerFocus>, _old_focus: Option<&Self::PointerFocus>) {
        tracing::trace!("Pointer focus changed to: {:?}", focused.map(|s| s.id()));
    }

    fn touch_focus_changed(&mut self, _seat: &Seat<Self>, focused: Option<&Self::TouchFocus>, _old_focus: Option<&Self::TouchFocus>) {
        tracing::trace!("Touch focus changed to: {:?}", focused.map(|s| s.id()));
    }
}
