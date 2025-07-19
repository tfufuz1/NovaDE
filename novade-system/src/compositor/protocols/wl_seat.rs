// Copyright 2024 NovaDE
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use smithay::reexports::wayland_server::{
    protocol::{wl_keyboard, wl_pointer, wl_seat, wl_touch},
    Client, Dispatch, DisplayHandle, GlobalDispatch, New,
};
use smithay::wayland::seat::{Seat, SeatDispatch, SeatHandler, SeatState};

use crate::nova_compositor_logic::state::NovaState;

impl GlobalDispatch<wl_seat::WlSeat, ()> for NovaState {
    fn bind(&mut self, handle: &DisplayHandle, _client: &Client, new: New<wl_seat::WlSeat>) {
        let seat = self.seat_state.get_mut().unwrap().new_seat(handle, new);
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for NovaState {
    fn request(
        &mut self,
        _client: &Client,
        seat: &wl_seat::WlSeat,
        request: wl_seat::Request,
        _data: &(),
        _dhandle: &DisplayHandle,
    ) {
        let seat_state = self.seat_state.get_mut().unwrap();

        match request {
            wl_seat::Request::GetPointer { id } => {
                seat_state.new_pointer(seat, id);
            }
            wl_seat::Request::GetKeyboard { id } => {
                seat_state.new_keyboard(seat, id);
            }
            wl_seat::Request::GetTouch { id } => {
                seat_state.new_touch(seat, id);
            }
            wl_seat::Request::Release => {
                // TODO: Handle release
            }
            _ => unreachable!(),
        }
    }
}

impl SeatHandler for NovaState {
    type KeyboardFocus = smithay::wayland::compositor::Surface;
    type PointerFocus = smithay::wayland::compositor::Surface;
    type TouchFocus = smithay::wayland::compositor::Surface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        self.seat_state.get_mut().unwrap()
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&Self::KeyboardFocus>) {}
    fn grab_changed(
        &mut self,
        _seat: &Seat<Self>,
        _grab: Option<
            smithay::wayland::seat::Grab<Self, Self::KeyboardFocus, Self::PointerFocus, Self::TouchFocus>,
        >,
    ) {
    }
}

pub fn init_seat(display: &mut DisplayHandle, state: &mut NovaState) {
    let mut seat_state = SeatState::new();
    let _seat = seat_state.new_seat(display, "seat-0".into(), |_, _| {});
    state.seat_state = Some(seat_state);
    display.create_global::<NovaState, wl_seat::WlSeat, _>(7, ());
}
