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
    protocol::{wl_shm, wl_shm_pool},
    Client, Dispatch, DisplayHandle, GlobalDispatch, New,
};
use smithay::wayland::shm::{shm_init, ShmDispatch, ShmState};

use crate::nova_compositor_logic::state::NovaState;

impl GlobalDispatch<wl_shm::WlShm, ()> for NovaState {
    fn bind(&mut self, handle: &DisplayHandle, _client: &Client, new: New<wl_shm::WlShm>) {
        shm_init(handle, new, self, |_state, _pool| {});
    }
}

impl Dispatch<wl_shm::WlShm, ()> for NovaState {
    fn request(
        &mut self,
        _client: &Client,
        _resource: &wl_shm::WlShm,
        request: wl_shm::Request,
        _data: &(),
        dhandle: &DisplayHandle,
    ) {
        let state = self.shm_state();
        match request {
            wl_shm::Request::CreatePool { id, fd, size } => {
                state.new_pool(dhandle, id, fd, size, |_state, _pool| {});
            }
            _ => unreachable!(),
        }
    }
}

impl ShmDispatch for NovaState {
    type UserData = ();
    fn shm_state(&self) -> &ShmState<Self> {
        &self.shm_state
    }
}

pub fn init_shm(display: &mut DisplayHandle, state: &mut NovaState) {
    let shm_state = ShmState::new(display, vec![], |_, _| {});
    state.shm_state = Some(shm_state);
    display.create_global::<NovaState, wl_shm::WlShm, _>(1, ());
}
