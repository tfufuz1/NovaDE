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
    protocol::wl_output, Client, Dispatch, DisplayHandle, GlobalDispatch,
};
use smithay::wayland::output::{Output, OutputHandler, PhysicalProperties, Mode, Scale};
use smithay::utils::{Point, Transform};

use crate::nova_compositor_logic::state::NovaState;

impl GlobalDispatch<wl_output::WlOutput, ()> for NovaState {
    fn bind(&mut self, handle: &DisplayHandle, _client: &Client, new: smithay::reexports::wayland_server::New<wl_output::WlOutput>) {
        let output = Output::new(
            handle,
            new,
            "novade-output".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: smithay::reexports::wayland_server::protocol::wl_output::Subpixel::Unknown,
                make: "NovaDE".into(),
                model: "Virtual".into(),
            },
            |_, _| {},
        );
        output.change_current_state(
            Some(Mode {
                size: (1920, 1080).into(),
                refresh: 60_000,
            }),
            Some(Transform::Normal),
            Some(Scale::Integer(1)),
            Some(Point::from((0, 0))),
        );
    }
}

impl Dispatch<wl_output::WlOutput, ()> for NovaState {
    fn request(
        &mut self,
        _client: &Client,
        _resource: &wl_output::WlOutput,
        request: wl_output::Request,
        _data: &(),
        _dhandle: &DisplayHandle,
    ) {
        if let wl_output::Request::Release = request {
            // TODO: handle release
        }
    }
}

impl OutputHandler for NovaState {}

pub fn init_output(display: &mut DisplayHandle, state: &mut NovaState) {
    display.create_global::<NovaState, wl_output::WlOutput, _>(4, ());
}
