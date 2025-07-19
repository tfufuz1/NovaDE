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
    protocol::{wl_compositor, wl_region, wl_surface},
    Client, Dispatch, DisplayHandle, GlobalDispatch, New,
};
use smithay::wayland::compositor::{
    compositor_init, CompositorClientState, CompositorDispatch, CompositorState, Region, Surface,
};

use crate::nova_compositor_logic::state::NovaState;

impl GlobalDispatch<wl_compositor::WlCompositor, ()> for NovaState {
    fn bind(
        &mut self,
        handle: &DisplayHandle,
        _client: &Client,
        new: New<wl_compositor::WlCompositor>,
    ) {
        compositor_init(handle, new, self, |state, surface| {
            state.new_surface(surface);
        });
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for NovaState {
    fn request(
        &mut self,
        _client: &Client,
        _resource: &wl_compositor::WlCompositor,
        request: wl_compositor::Request,
        _data: &(),
        dhandle: &DisplayHandle,
    ) {
        match request {
            wl_compositor::Request::CreateSurface { id } => {
                let surface = Surface::new(dhandle, id, |state: &mut Self, surface| {
                    state.new_surface(surface.clone());
                });
            }
            wl_compositor::Request::CreateRegion { id } => {
                Region::new(dhandle, id);
            }
            _ => unreachable!(),
        }
    }
}

impl CompositorDispatch for NovaState {
    type UserData = ();

    fn new_surface(&mut self, surface: wl_surface::WlSurface, _udata: &Self::UserData) {
        // Handle new surface creation here
    }
}

pub fn init_compositor(display: &mut DisplayHandle, state: &mut NovaState) {
    let _compositor_global = display.create_global::<NovaState, wl_compositor::WlCompositor, _>(5, ());
}
