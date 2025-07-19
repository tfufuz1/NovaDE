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
    protocol::{wl_subcompositor, wl_subsurface},
    Client, Dispatch, DisplayHandle, GlobalDispatch, New,
};
use smithay::wayland::compositor::get_role;
use smithay::wayland::subcompositor::{
    subcompositor_init, SubcompositorDispatch, SubcompositorState,
};

use crate::nova_compositor_logic::state::NovaState;

impl GlobalDispatch<wl_subcompositor::WlSubcompositor, ()> for NovaState {
    fn bind(
        &mut self,
        handle: &DisplayHandle,
        _client: &Client,
        new: New<wl_subcompositor::WlSubcompositor>,
    ) {
        subcompositor_init(handle, new, self, |state, subsurface| {
            // Handle new subsurface creation here
        });
    }
}

impl Dispatch<wl_subcompositor::WlSubcompositor, ()> for NovaState {
    fn request(
        &mut self,
        _client: &Client,
        _resource: &wl_subcompositor::WlSubcompositor,
        request: wl_subcompositor::Request,
        _data: &(),
        dhandle: &DisplayHandle,
    ) {
        match request {
            wl_subcompositor::Request::GetSubsurface { id, surface, parent } => {
                if let Some(role) = get_role(&surface) {
                    // Surface already has a role
                } else {
                    // todo: add subsurface
                }
            }
            _ => unreachable!(),
        }
    }
}

impl SubcompositorDispatch for NovaState {
    type UserData = ();

    fn on_commit(&mut self, surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface) {
        // Handle commit here
    }
}

pub fn init_subcompositor(display: &mut DisplayHandle, state: &mut NovaState) {
    let _subcompositor_global =
        display.create_global::<NovaState, wl_subcompositor::WlSubcompositor, _>(1, ());
}
