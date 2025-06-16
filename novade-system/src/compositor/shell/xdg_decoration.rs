// Copyright 2024 NovaDE Compositor contributors
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

//! XDG Decoration Manager module
//!
//! Handles the `zxdg_decoration_manager_v1` protocol for server-side decorations.

use smithay::reexports::wayland_server::{
    protocol::{
        wl_surface::WlSurface,
        xdg_surface::{self, XdgSurface}, // To link decoration with XdgSurface/Toplevel
    },
    Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New,
};
use smithay::reexports::wayland_protocols::unstable::xdg_decoration::v1::server::{
    zxdg_decoration_manager_v1::{self, ZxdgDecorationManagerV1},
    zxdg_toplevel_decoration_v1::{self, Mode as DecorationMode, ZxdgToplevelDecorationV1},
};

use crate::compositor::state::DesktopState; // Assuming DesktopState is the main state
use crate::compositor::shell::xdg::{XdgToplevelData, XdgSurfaceData, XdgRoleSpecificData}; // To store decoration mode
use std::sync::Mutex;

/// User data for the XDG Decoration Manager global.
#[derive(Debug, Default)]
pub struct XdgDecorationManagerState;

/// User data for the `zxdg_toplevel_decoration_v1` resource.
/// It needs to be associated with the WlSurface or XdgSurface
/// to identify which toplevel it's decorating.
#[derive(Debug)]
pub struct ToplevelDecorationUserData {
    pub wl_surface: WlSurface, // Or xdg_surface: XdgSurface for stronger typing
}

// ANCHOR: Implement GlobalDispatch and Dispatch traits for XDG Decoration Manager

// Global data for XDG Decoration Manager
#[derive(Debug, Default)]
pub struct XdgDecorationManagerGlobalData;

impl GlobalDispatch<ZxdgDecorationManagerV1, XdgDecorationManagerGlobalData, DesktopState> for XdgDecorationManagerState {
    fn bind(
        _state: &mut DesktopState,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<ZxdgDecorationManagerV1>,
        _global_data: &XdgDecorationManagerGlobalData,
        data_init: &mut DataInit<'_, DesktopState>,
    ) {
        data_init.init(resource, XdgDecorationManagerState::default());
    }
}

impl Dispatch<ZxdgDecorationManagerV1, XdgDecorationManagerState, DesktopState> for XdgDecorationManagerState {
    fn request(
        state: &mut DesktopState,
        _client: &Client,
        _manager: &ZxdgDecorationManagerV1,
        request: zxdg_decoration_manager_v1::Request,
        _data: &XdgDecorationManagerState,
        _dh: &DisplayHandle,
        data_init: &mut DataInit<'_, DesktopState>,
    ) {
        match request {
            zxdg_decoration_manager_v1::Request::Destroy => {
                // ANCHOR: XdgDecorationManager::destroy
                // Cleanup manager global state if any was stored in XdgDecorationManagerState
            }
            zxdg_decoration_manager_v1::Request::GetToplevelDecoration { id, toplevel } => {
                // ANCHOR: XdgDecorationManager::get_toplevel_decoration
                // `toplevel` is an xdg_toplevel object. We need its underlying wl_surface.
                // Smithay's XdgToplevel resource should provide access to its XdgSurface,
                // which in turn provides access to the WlSurface.
                // However, the `toplevel` argument here is `xdg_toplevel from wayland_protocols, not Smithay's wrapper.
                // We need to get the WlSurface associated with this xdg_toplevel.
                // This usually means the xdg_toplevel resource should have user data pointing to its WlSurface or XdgSurface.
                // Smithay's XdgSurfaceUserData is associated with XdgSurface, not XdgToplevel directly by default.
                // For now, let's assume we can get the WlSurface from the toplevel object.
                // This is tricky because `toplevel` is `xdg_toplevel_protocol::XdgToplevel`.
                // We need to find the WlSurface associated with this specific xdg_toplevel protocol object.
                // Smithay's XdgShellState might have a way to look this up, or the xdg_toplevel
                // resource itself needs to store its WlSurface in its user_data.
                // Smithay's `xdg_toplevel.wl_surface()` method is available on Smithay's XdgToplevel wrapper.
                // The protocol object `toplevel` itself does not directly expose this.
                //
                // A common pattern: the xdg_toplevel object's user_data is often the same as its xdg_surface's user_data.
                // And XdgSurfaceUserData has `wl_surface()`.
                // Let's assume `toplevel.data::<XdgSurfaceUserData>()` would work if XdgToplevel resource shares user_data.
                // This is true if `data_init.init(id, xdg_toplevel_user_data)` was used for xdg_toplevel.

                let xdg_surface_resource = match state.xdg_shell_state.xdg_toplevel_surface(&toplevel) {
                    Some(surface) => surface,
                    None => {
                        // TODO: Post protocol error: toplevel is already destroyed or not known.
                        log::error!("get_toplevel_decoration: xdg_toplevel resource {:?} not found or not associated with an xdg_surface.", toplevel.id());
                        // manager.post_error(zxdg_decoration_manager_v1::Error::UnmanagedToplevel, "toplevel not managed");
                        return;
                    }
                };
                let wl_surface = xdg_surface_resource.wl_surface().clone();


                // Default to server-side decorations if client doesn't set a mode.
                // This is typically compositor policy.
                let initial_mode = DecorationMode::ServerSide; // Or use a compositor config option.

                if let Some(surface_data_mutex) = wl_surface.data::<Mutex<XdgSurfaceData>>() {
                    let mut surface_data = surface_data_mutex.lock().unwrap();
                    if let XdgRoleSpecificData::Toplevel(toplevel_data) = &mut surface_data.role_data {
                        toplevel_data.decoration_mode = Some(initial_mode);
                        log::info!("XDG Toplevel Decoration for {:?}: Initial mode set to {:?}", wl_surface.id(), initial_mode);
                    } else {
                        // TODO: Protocol error, not a toplevel or role data missing
                        log::error!("get_toplevel_decoration: Surface {:?} is not a toplevel or has no toplevel data.", wl_surface.id());
                        return;
                    }
                } else {
                    // TODO: Protocol error, surface data missing
                    log::error!("get_toplevel_decoration: XdgSurfaceData not found for WlSurface {:?}", wl_surface.id());
                    return;
                }

                let decoration_resource = data_init.init(
                    id,
                    ToplevelDecorationUserData {
                        wl_surface: wl_surface.clone(),
                    },
                );

                // Send initial configure event for the decoration object
                decoration_resource.configure(initial_mode);
                log::debug!("Sent initial configure for zxdg_toplevel_decoration {:?} with mode {:?}", decoration_resource.id(), initial_mode);

            }
            _ => unimplemented!(),
        }
    }
}


impl Dispatch<ZxdgToplevelDecorationV1, ToplevelDecorationUserData, DesktopState> for XdgDecorationManagerState {
    fn request(
        _state: &mut DesktopState,
        _client: &Client,
        resource: &ZxdgToplevelDecorationV1,
        request: zxdg_toplevel_decoration_v1::Request,
        data: &ToplevelDecorationUserData, // UserData specific to this toplevel_decoration resource
        _dh: &DisplayHandle,
        _data_init: &mut DataInit<'_, DesktopState>,
    ) {
        let wl_surface = &data.wl_surface;

        match request {
            zxdg_toplevel_decoration_v1::Request::Destroy => {
                // ANCHOR: ZxdgToplevelDecorationV1::destroy
                // Resources are automatically cleaned up by Wayland server on destruction.
                // If we stored any specific state for this resource ID elsewhere, clean it up.
            }
            zxdg_toplevel_decoration_v1::Request::SetMode { mode } => {
                // ANCHOR: ZxdgToplevelDecorationV1::set_mode
                // Client requests a specific decoration mode.
                // Store this preference in XdgToplevelData.
                if let Some(surface_data_mutex) = wl_surface.data::<Mutex<XdgSurfaceData>>() {
                    let mut surface_data = surface_data_mutex.lock().unwrap();
                    if let XdgRoleSpecificData::Toplevel(toplevel_data) = &mut surface_data.role_data {
                        toplevel_data.decoration_mode = Some(mode);
                        log::info!("XDG Toplevel Decoration for {:?}: Mode set to {:?}", wl_surface.id(), mode);

                        // ANCHOR: Server-side decoration rendering implication
                        // If mode is ServerSide, compositor is responsible for drawing decorations.
                        // This is a hint; actual drawing is a rendering concern.

                        // Send a configure event to acknowledge the mode change.
                        resource.configure(mode);
                    } else {
                         log::warn!("SetMode called on decoration object for surface {:?} which is not a toplevel or has no data.", wl_surface.id());
                    }
                } else {
                    log::warn!("SetMode called on decoration object for surface {:?}, but XdgSurfaceData not found.", wl_surface.id());
                }
            }
            zxdg_toplevel_decoration_v1::Request::UnsetMode => {
                // ANCHOR: ZxdgToplevelDecorationV1::unset_mode
                // Client requests to unset the mode, implying compositor preference should be used.
                // This usually defaults to server-side decorations.
                let new_mode = DecorationMode::ServerSide; // Compositor's default preference

                if let Some(surface_data_mutex) = wl_surface.data::<Mutex<XdgSurfaceData>>() {
                    let mut surface_data = surface_data_mutex.lock().unwrap();
                    if let XdgRoleSpecificData::Toplevel(toplevel_data) = &mut surface_data.role_data {
                        toplevel_data.decoration_mode = Some(new_mode);
                        log::info!("XDG Toplevel Decoration for {:?}: Mode unset, reverted to {:?}", wl_surface.id(), new_mode);
                        resource.configure(new_mode);
                    } else {
                         log::warn!("UnsetMode called on decoration object for surface {:?} which is not a toplevel or has no data.", wl_surface.id());
                    }
                } else {
                     log::warn!("UnsetMode called on decoration object for surface {:?}, but XdgSurfaceData not found.", wl_surface.id());
                }
            }
            _ => unimplemented!(),
        }
    }
}

// ANCHOR: Add `mod xdg_decoration;` to `novade-system/src/compositor/shell/mod.rs`
// ANCHOR: Add XdgDecorationManagerState to DesktopState in `state.rs`
// ANCHOR: Add `delegate_xdg_decoration!(DesktopState);` in `state.rs`
// ANCHOR: Add `decoration_mode: Option<DecorationMode>` to `XdgToplevelData` in `xdg.rs`

/// Helper to get XdgToplevelData mutable reference, assuming the surface is a toplevel.
/// Panics if not a toplevel or data is missing.
pub fn with_toplevel_data<F, R>(surface: &WlSurface, mut callback: F) -> R
    where
        F: FnMut(&mut XdgToplevelData) -> R,
{
    let surface_data_mutex = surface.data::<Mutex<XdgSurfaceData>>()
        .expect("XdgSurfaceData not found on WlSurface. Programmer error.");
    let mut surface_data_guard = surface_data_mutex.lock().unwrap();
    match &mut surface_data_guard.role_data {
        XdgRoleSpecificData::Toplevel(toplevel_data) => callback(toplevel_data),
        _ => panic!("Surface is not an XDG Toplevel or role_data is incorrect."),
    }
}

impl XdgDecorationManagerState {
    pub fn new() -> Self {
        Self::default()
    }
}
