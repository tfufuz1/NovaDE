// Copyright 2024 NovaDE Compsositor contributors
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

//! XDG Shell Handlers

use smithay::reexports::wayland_server::{
    protocol::{
        wl_surface::WlSurface,
        xdg_wm_base::{Request, XdgWmBase},
    },
    Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch,
};
use smithay::wayland::shell::xdg::{
    XdgPositionerUserData, XdgShellHandler, XdgSurfaceUserData, XDG_SHELL_VERSION,
};

use crate::compositor::shell::xdg::{XdgShellState, XdgSurfaceData, XdgSurfaceRole};
use smithay::desktop::Window;

/// Global for XDG Shell interface
#[derive(Debug)]
pub struct XdgShellGlobalData;

impl<D> GlobalDispatch<XdgWmBase, XdgShellGlobalData, D> for XdgShellState
where
    D: GlobalDispatch<XdgWmBase, XdgShellGlobalData>,
    D: Dispatch<XdgWmBase, XdgShellState>,
    D: Dispatch<smithay::reexports::wayland_server::protocol::xdg_positioner::XdgPositioner, XdgPositionerUserData>,
    D: Dispatch<smithay::reexports::wayland_server::protocol::xdg_surface::XdgSurface, XdgSurfaceUserData>,
    D: XdgShellHandler,
    D: 'static,
{
    fn bind(
        _state: &mut D,
        _dh: &DisplayHandle,
        _client: &Client,
        resource: smithay::reexports::wayland_server::New<XdgWmBase>,
        _global_data: &XdgShellGlobalData,
        data_init: &mut DataInit<'_, D>,
    ) {
        data_init.init(resource, XdgShellState {});
    }
}

impl<D> Dispatch<XdgWmBase, XdgShellState, D> for XdgShellState
where
    D: Dispatch<XdgWmBase, XdgShellState>,
    D: Dispatch<smithay::reexports::wayland_server::protocol::xdg_positioner::XdgPositioner, XdgPositionerUserData>,
    D: Dispatch<smithay::reexports::wayland_server::protocol::xdg_surface::XdgSurface, XdgSurfaceUserData>,
    D: XdgShellHandler,
    D: 'static,
{
    fn request(
        _state: &mut D,
        _client: &Client,
        _resource: &XdgWmBase,
        request: Request,
        _data: &XdgShellState,
        _dh: &DisplayHandle,
        data_init: &mut DataInit<'_, D>,
    ) {
        match request {
            Request::Destroy => {
                // ANCHOR: XdgWmBase::destroy
                // All objects created by the interface (xdg_positioner, xdg_surface, xdg_toplevel, xdg_popup) are destroyed separately.
                // This request only destroys the xdg_wm_base object itself.
                // TODO: Any additional cleanup needed for xdg_wm_base global state?
            }
            Request::CreatePositioner { id } => {
                // ANCHOR: XdgWmBase::create_positioner
                data_init.init(
                    id,
                    XdgPositionerUserData {
                        // This is a placeholder; actual positioner logic will be more complex.
                        // For example, it might involve storing constraints for popup positioning.
                    },
                );
            }
            Request::GetXdgSurface { id, surface } => {
                // ANCHOR: XdgWmBase::get_xdg_surface
                // This assigns the xdg_surface role to a wl_surface.
                // A wl_surface can only be assigned a role once.
                // If it already has a role, a protocol error should be sent.
                // Smithay's XdgSurfaceUserData handles this check.

                let xdg_surface_data = XdgSurfaceData {
                    // Initially, the role is unknown until a toplevel or popup is created.
                    // However, for now, let's default to Toplevel and adjust later if needed
                    // or handle the unassigned role state explicitly.
                    // This part might need refinement based on how roles are determined.
                    role: XdgSurfaceRole::Toplevel, // Placeholder: Role will be defined by subsequent requests.
                    parent: None,
                    window: Window::new(WindowSurfaceType::Xdg(None)), // Placeholder: Window needs proper initialization
                };

                let user_data = XdgSurfaceUserData {
                    // TODO: Ensure this correctly links the surface data with the xdg_surface.
                    // The XdgSurfaceUserData might need to store XdgSurfaceData directly or via an Arc<Mutex<>>.
                    // For now, relying on Smithay's default behavior.
                    // We'll likely need to customize this to store our XdgSurfaceData.
                };

                // Initialize the xdg_surface with user data.
                // The user_data should ideally hold our XdgSurfaceData or a reference to it.
                // This is where we associate our custom XdgSurfaceData with the wl_surface.
                // Smithay's default XdgSurfaceUserData might not be sufficient.
                // We need to ensure that `surface.data::<XdgSurfaceData>()` can retrieve our data.
                // This might involve using `surface.assign_data()` or similar.

                // For now, let's assume smithay handles associating some form of user data.
                // We will need to explicitly store our XdgSurfaceData with the WlSurface.
                // This is typically done using `surface.data::Initial`.
                // If the surface already has data of this type, it would be an error.
                // Smithay's XdgShellHandler might handle this internally.

                // Create the xdg_surface resource
                let xdg_surface_resource = data_init.init(id, user_data);

                // Associate our custom XdgSurfaceData with the WlSurface
                // This makes XdgSurfaceData accessible via wl_surface.data::<XdgSurfaceData>()
                surface.set_data(xdg_surface_data);


                // TODO: Implement logic to check if the surface already has a role.
                // If so, send a protocol error (e.g., role_already_assigned).
                // Smithay's XdgShellHandler might do this automatically.

                // TODO: Store the xdg_surface and its data in XdgShellState if needed for tracking.
                // For example, state.xdg_surfaces.push(xdg_surface_resource);
            }
            Request::Pong { serial } => {
                // ANCHOR: XdgWmBase::pong
                // TODO: Implement pong handling. This involves checking the serial
                // against a previously sent ping event.
                // For now, just log or ignore.
                log::debug!("Client responded to ping with serial: {}", serial);
            }
            _ => unimplemented!(),
        }
    }
}

// ANCHOR: Implement XdgPositioner and XdgSurface dispatchers if not already handled by Smithay's XdgShellHandler.
// These will handle requests for xdg_positioner and xdg_surface objects.
// For example:
// impl<D> Dispatch<xdg_positioner::XdgPositioner, XdgPositionerState, D> for XdgShellState
// where D: ... { ... }
//
// impl<D> Dispatch<xdg_surface::XdgSurface, XdgSurfaceDataWrapper, D> for XdgShellState
// where D: ... { ... }
// (XdgSurfaceDataWrapper would be a struct that holds Arc<Mutex<XdgSurfaceData>>)

// ANCHOR: Implement XdgToplevel and XdgPopup handlers.
// These will be responsible for managing toplevel and popup specific logic.
// For example, handling requests like set_title, set_app_id for toplevels,
// and grab for popups.

// NOTE: The XdgShellState in this file is currently a placeholder.
// It might need to store lists of surfaces, popups, etc., if not handled by Smithay's DMs.
// The current implementation assumes Smithay's DataMaps (`UserDataMap`) are used
// to store data associated with Wayland objects (like WlSurface, XdgSurface).
// If XdgShellState needs to track global shell properties or lists of objects,
// it should be defined accordingly in xdg.rs and used here.
// For now, XdgShellState from xdg.rs is an empty struct, implying state is
// managed per-object via UserData.
//
// The `GlobalDispatch` and `Dispatch` implementations for `XdgShellState`
// suggest that `XdgShellState` itself is the user_data for `XdgWmBase` resource.
// This means any state specific to the `xdg_wm_base` global instance can be stored in `XdgShellState`.
// For instance, if we need to track all `xdg_surface`s created via this global,
// `XdgShellState` could have a `Vec<XdgSurface>`.
// However, individual surface data (like title, role) is better stored with the surface itself
// (e.g., in `XdgSurfaceData` associated with `WlSurface` or `XdgSurface` resource).

use smithay::reexports::wayland_server::protocol::{
    xdg_surface::{self, XdgSurface},
    xdg_toplevel, xdg_popup,
};
use smithay::wayland::shell::xdg::{PopupState, Role, SurfaceCachedState, ToplevelState as SmithayToplevelState}; // Renamed to avoid conflict
use crate::compositor::shell::xdg::{XdgToplevelData, XdgPopupData, ToplevelState}; // Our ToplevelState
use std::sync::{Arc, Mutex};
use smithay::utils::Serial;


// Helper to get XdgSurfaceData from a WlSurface
// This assumes XdgSurfaceData was stored using `surface.set_data()`
fn get_surface_data(surface: &WlSurface) -> &Mutex<XdgSurfaceData> {
    surface.data::<Mutex<XdgSurfaceData>>()
        .expect("XdgSurfaceData not found on WlSurface. This indicates a programmer error: XdgSurfaceData must be set when get_xdg_surface is called.")
}


impl<D> Dispatch<xdg_surface::XdgSurface, XdgSurfaceUserData, D> for XdgShellState
where
    D: Dispatch<xdg_surface::XdgSurface, XdgSurfaceUserData>,
    D: Dispatch<xdg_toplevel::XdgToplevel, XdgSurfaceUserData>, // Assuming XdgSurfaceUserData for toplevels for now
    D: Dispatch<xdg_popup::XdgPopup, XdgSurfaceUserData>, // Assuming XdgSurfaceUserData for popups for now
    D: XdgShellHandler, // Ensure D implements XdgShellHandler for configure logic
    D: 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        xdg_surface: &xdg_surface::XdgSurface,
        request: xdg_surface::Request,
        _data: &XdgSurfaceUserData, // This is Smithay's XdgSurfaceUserData
        dh: &DisplayHandle,
        data_init: &mut DataInit<'_, D>,
    ) {
        let wl_surface = xdg_surface.wl_surface();
        let surface_data_mutex = get_surface_data(wl_surface);
        let mut surface_data = surface_data_mutex.lock().unwrap();

        match request {
            xdg_surface::Request::Destroy => {
                // ANCHOR: XdgSurface::destroy
                // Smithay handles the actual destruction of the xdg_surface resource.
                // We need to ensure our associated XdgSurfaceData is cleaned up if necessary,
                // though if it's stored on WlSurface's UserDataMap, it should be dropped when WlSurface is destroyed.
                // If the Window associated with this surface needs explicit cleanup, do it here.
                log::debug!("XdgSurface destroyed for wl_surface: {:?}", wl_surface);
            }
            xdg_surface::Request::GetToplevel { id } => {
                // ANCHOR: XdgSurface::get_toplevel
                // Assign the toplevel role to this surface.
                // A surface can only be assigned one role.
                // Smithay's XdgShellHandler::surface_added should handle role conflict.
                // We need to store XdgToplevelData.

                // Check if a role has already been assigned. Smithay might do this.
                // For now, we assume our XdgSurfaceData.role tracks this.
                // TODO: Verify how Smithay's Role<XdgShellState> on XdgSurfaceUserData interacts with this.
                // It seems Smithay's `xdg_surface.data::<XdgSurfaceUserData>()?.role` is the source of truth for roles.

                if surface_data.role != XdgSurfaceRole::Toplevel && surface_data.parent.is_some() { // A crude check, Smithay handles this better.
                    // This check is not robust. Smithay's XdgSurfaceUserData.role should be checked.
                    // xdg_surface.post_error(xdg_wm_base::Error::Role, "Surface already has a role or parent");
                    // return;
                    log::warn!("Attempting to assign toplevel role, but surface might already have a role or specific setup.");
                }

                surface_data.role = XdgSurfaceRole::Toplevel;
                let toplevel_data = XdgToplevelData {
                    title: None,
                    app_id: None,
                    parent: None, // TODO: Set parent if applicable for transient toplevels
                    current_state: ToplevelState { // Our ToplevelState
                        maximized: false,
                        minimized: false,
                        fullscreen: false,
                    },
                };
                // Associate ToplevelData with the XdgSurfaceData or the Window.
                // For now, let's assume XdgSurfaceData will hold an Option<XdgToplevelData>.
                // This requires modifying XdgSurfaceData struct.
                // For now, we are not storing it directly in this handler.
                // It should be part of the XdgSurfaceData on the WlSurface.
                // Let's log that we would store it.
                log::debug!("XdgToplevel role assigned. Data: {:?}", toplevel_data);


                // Initialize the xdg_toplevel resource.
                // Smithay's XdgSurfaceUserData is used here.
                // We need to ensure our XdgToplevelData is accessible when handling toplevel requests.
                // This might mean XdgSurfaceUserData needs to be generic or store our data.
                let xdg_toplevel_user_data = xdg_surface.data::<XdgSurfaceUserData>().unwrap().clone();
                data_init.init(id, xdg_toplevel_user_data);


                // Update the Window associated with this surface.
                // surface_data.window.set_role_data(toplevel_data); // Conceptual
                // The Window's surface type might also need updating if it wasn't set correctly at get_xdg_surface
                if let WindowSurfaceType::Xdg(ref mut xdg_window_surface) = surface_data.window.surface_type_mut() {
                     // Smithay's Window expects an XdgToplevel or XdgPopup to be set.
                     // This is usually done via Window::new_xdg_toplevel or Window::new_xdg_popup.
                     // Here, we are creating it after the Window object.
                     // This part needs careful integration with Smithay's Window state.
                     // For now, we assume the Window was created with a generic XDG type
                     // and we are now specializing it.
                     // A more robust approach would be to create the Window *here* or update it.
                     // Smithay's `xdg_shell_handlers.rs` example shows `Window::new_xdg_toplevel`.
                }
                 // TODO: Trigger a configure event for the new toplevel.
                 // XdgShellHandler::configure_toplevel is relevant.
                 // state.configure_surface(dh, xdg_surface.wl_surface(), ...)
                 // For now, let's request a configure from smithay's handler
                 smithay::wayland::shell::xdg::send_configure(dh, xdg_surface);

            }
            xdg_surface::Request::GetPopup { id, parent_xdg_surface, positioner } => {
                // ANCHOR: XdgSurface::get_popup
                // Assign the popup role.
                // Similar to toplevel, check for existing roles.
                // Smithay's XdgShellHandler::surface_added should handle role conflict.
                // We need to store XdgPopupData.

                let parent_wl_surface = match parent_xdg_surface {
                    Some(parent_surface) => parent_surface.wl_surface().clone(),
                    None => {
                        // As per spec, parent can be None for transient popups not bound to a specific surface.
                        // However, most compositors might require a parent surface for positioning.
                        // Smithay's XdgPopupSurfaceRoleAttributes expects a parent WlSurface.
                        // TODO: Handle parent_xdg_surface being None if your compositor design allows.
                        // For now, assume parent is Some, or handle error.
                        // xdg_surface.post_error(xdg_wm_base::Error::InvalidPopupParent, "Popup parent cannot be None in this compositor");
                        // return;
                        log::warn!("Popup created without an explicit XDG parent surface. This might be handled by client providing a grab.");
                        // This scenario needs careful handling based on compositor policy.
                        // For now, we'll proceed but this popup might not be correctly parented in the scene graph.
                        // A common approach is to require a parent for popups.
                        // If we cannot get a WlSurface, we cannot proceed with typical parenting.
                        // Let's log and skip creating the popup for now if parent is None.
                        // This part needs a design decision.
                        // For the purpose of this example, let's assume parent_xdg_surface is always Some.
                        // If not, we should post an error or handle it gracefully.
                        // However, the protocol allows parent to be NULL for xdg_popup.
                        // Smithay's Window::new_xdg_popup requires a parent WlSurface.
                        // Let's assume for now that if parent_xdg_surface is None,
                        // the client has other means (like a grab) to position/manage the popup.
                        // This is complex. For a basic implementation, we might require a parent.
                        // Let's proceed assuming parent_xdg_surface.wl_surface() is valid if Some.
                        // A robust compositor would need to handle the None case carefully.
                        // For now, we'll try to get the WlSurface and expect it to be there.
                        // This is a simplification.
                        // A better way: if parent_xdg_surface is None, then the popup is parented to nothing *initially*.
                        // Its positioning then relies solely on the positioner and possibly a grab.
                        // Smithay's `PopupState` can be created without a parent `WlSurface` initially,
                        // but `Window::new_xdg_popup` needs one. This implies a disconnect or
                        // that the `Window` abstraction in Smithay might need adjustment for such cases,
                        // or such popups are not represented by `smithay::desktop::Window`.

                        // Let's assume we need a parent wl_surface for our XdgPopupData for now.
                        // If parent_xdg_surface is None, we cannot create XdgPopupData as defined.
                        // This should ideally result in a protocol error or specific handling.
                        // For now, we'll log an error and not proceed with creating the popup.
                        log::error!("xdg_popup creation without a parent XDG surface is not fully handled yet.");
                        // xdg_surface.post_error(xdg_wm_base::Error::InvalidPopupParent, "Popup parent required by this compositor.");
                        return;
                    }
                };


                surface_data.role = XdgSurfaceRole::Popup;
                surface_data.parent = Some(parent_wl_surface.clone());

                let popup_data = XdgPopupData {
                    parent: parent_wl_surface.clone(),
                    // TODO: store and use positioner data
                };
                log::debug!("XdgPopup role assigned. Data: {:?}", popup_data);
                // Associate PopupData, similar to ToplevelData.
                // This also requires modifying XdgSurfaceData or Window.

                // Initialize the xdg_popup resource.
                let xdg_popup_user_data = xdg_surface.data::<XdgSurfaceUserData>().unwrap().clone();
                data_init.init(id, xdg_popup_user_data);


                // Update/Create the Window object for this popup.
                // Smithay's Window::new_xdg_popup(parent_wl_surface, ...)
                // This is tricky if the Window was already created in get_xdg_surface.
                // It might be better to create the Window here, or update its role data.
                // If WindowSurfaceType was generic Xdg, update it.
                // surface_data.window.set_role_data(popup_data); // Conceptual

                // TODO: Trigger a configure event for the new popup.
                // XdgShellHandler::configure_popup is relevant.
                // state.configure_surface(dh, xdg_surface.wl_surface(), ...)
                smithay::wayland::shell::xdg::send_configure(dh, xdg_surface);
            }
            xdg_surface::Request::SetWindowGeometry { x, y, width, height } => {
                // ANCHOR: XdgSurface::set_window_geometry
                // Client informs us about the size and position of the window content,
                // relative to the parent surface for popups.
                // This is the client's view of its geometry.
                // We need to store this, possibly in XdgSurfaceData or Window's SurfaceAttributes.
                // This is distinct from the compositor-assigned geometry.
                log::debug!(
                    "Client set window geometry: x={}, y={}, width={}, height={}",
                    x, y, width, height
                );
                let mut attrs = surface_data.window.surface_attributes();
                attrs.window_geometry = Some(smithay::utils::Rectangle::from_loc_and_size(
                    (x, y),
                    (width, height),
                ));
                // The Window itself might need to be updated with this geometry.
                // surface_data.window.set_surface_attributes(attrs); // If such a method exists or is needed.
                // Smithay's SurfaceAttributes are often managed internally by the Window based on role.
                // For XDG surfaces, this geometry is part of the SurfaceCachedState.
                // We should update our XdgSurfaceData or rely on Smithay's caching if sufficient.
                // Let's assume for now this is mainly for information or specific layout calculations.
                // Smithay's `SurfaceData` (from `wl_surface.data()`) has `SurfaceAttributes`
                // which includes `window_geometry`. This is likely where it should be stored.
                // The `Window` object in Smithay wraps a `WlSurface` and manages its attributes.
                // So, updating `surface_data.window.surface_attributes()` might be the way,
                // but this API needs to be checked.
                // Smithay's `xdg_shell_handlers` example updates `xdg_surface.data::<XdgSurfaceUserData>().unwrap().cached_state`.
                // Let's follow that pattern.
                let user_data = xdg_surface.data::<XdgSurfaceUserData>().unwrap();
                let mut cached_state = user_data.cached_state.lock().unwrap();
                cached_state.window_geometry = Some(smithay::utils::Rectangle::from_loc_and_size(
                    (x, y),
                    (width, height),
                ));

            }
            xdg_surface::Request::AckConfigure { serial } => {
                // ANCHOR: XdgSurface::ack_configure
                // Client acknowledges a configure event.
                // This is crucial for synchronized state changes.
                // We need to match this serial with one we sent.
                // Smithay's XdgShellHandler::ack_configure handles this.
                // state.ack_configure(dh, xdg_surface.wl_surface(), serial);
                // This will typically involve committing the pending state associated with that serial.
                log::debug!("Client acknowledged configure with serial: {}", serial);

                // Call Smithay's handler or implement similar logic.
                // This usually involves finding the pending state for this serial and applying it.
                // For example, if a resize was requested, the new size becomes current here.
                // Smithay's `xdg_shell_handlers` uses `xdg_surface.ack_configure(serial)`.
                // Let's ensure our XdgShellHandler `state` can process this.
                // The `state` (which is `D: XdgShellHandler`) should have the method.
                state.ack_configure(serial, xdg_surface.wl_surface().clone(), xdg_surface.clone()).unwrap_or_else(|err| {
                    log::warn!("Error processing ack_configure: {:?}", err);
                });

                // If the configure was for a toplevel, we might need to update our XdgToplevelData's current_state
                // based on what was acknowledged.
                // This depends on how pending states are managed. Smithay's Window often handles this.
                // For example, if a configure requested maximization, and this ack matches that configure,
                // then current_state.maximized becomes true.
                // This logic is often tied to the `Window::on_commit` or similar callbacks.
            }
            _ => unimplemented!(),
        }
    }
}

// ANCHOR: Implement configure event sending logic.
// This is typically done by the XdgShellHandler trait implementation in the main compositor state.
// For example, when a surface needs to be resized, maximized, etc., the compositor
// would decide the new state, send a configure event with a new serial, and store this pending state.
// The client then applies the changes and ack_configures.
//
// Example of sending a configure (conceptual, actual call might be via XdgShellHandler trait):
// fn send_example_configure(dh: &DisplayHandle, surface: &xdg_surface::XdgSurface) {
//     let serial = SERIAL_COUNTER.next_serial(); // Manage serials globally or per surface
//     surface.configure(serial, 100, 100); // Example: configure size
//     // Store pending state associated with `serial`
// }

// Note on XdgSurfaceData modification:
// The current get_xdg_surface in handlers.rs does:
// surface.set_data(xdg_surface_data);
// This stores XdgSurfaceData directly. If we need interior mutability (e.g. for
// get_toplevel/get_popup to store XdgToplevelData/XdgPopupData inside XdgSurfaceData),
// then XdgSurfaceData should contain Arc<Mutex<InnerXdgSurfaceDataContent>> or similar,
// or the WlSurface itself should store Arc<Mutex<XdgSurfaceData>>.
// The current code uses `surface.set_data(Mutex<XdgSurfaceData>)` (implicitly from get_surface_data helper).
// This means `surface_data` in the Dispatch::request is `MutexGuard<XdgSurfaceData>`,
// so we can modify it directly.
// We need to ensure XdgSurfaceData struct in xdg.rs is updated to hold role-specific data:
//
// pub struct XdgSurfaceData {
//     pub role: XdgSurfaceRole,
//     pub parent: Option<WlSurface>,
//     pub window: Window,
//     pub role_data: Option<XdgSurfaceRoleSpecificData>, // New field
//     // pub geometry, etc. if not handled by Window's SurfaceAttributes
// }
//
// pub enum XdgSurfaceRoleSpecificData {
//     Toplevel(XdgToplevelData),
//     Popup(XdgPopupData),
// }
// This change in xdg.rs is implied by the logic in get_toplevel/get_popup.
// For now, the handler logs instead of storing this role_specific_data.
// It's important to make this change in xdg.rs for the data to be actually stored.
// The `Window` object from Smithay also plays a big role in managing XDG surface state.
// The interaction between our XdgSurfaceData and Smithay's Window needs to be clear.
// Smithay's Window often stores its own XDG-specific state (e.g., `WindowSurface::Xdg`).
// We need to decide if our XdgToplevelData/XdgPopupData duplicates or complements Smithay's.
// Ideally, they should complement, with our structs holding NovaDE-specific logic/state.
//
// The current XdgSurfaceUserData from smithay::wayland::shell::xdg is used for the xdg_surface resource.
// It contains `role: Role<D>`, `parent: Option<WlSurface>`, `cached_state: Mutex<SurfaceCachedState>`, etc.
// `SurfaceCachedState` includes `min_size`, `max_size`, `window_geometry`.
// `Role<D>` is an enum `None, Toplevel(ToplevelState<D>), Popup(PopupState<D>)`.
// `ToplevelState<D>` contains `app_id`, `title`, `min_size`, `max_size`, etc.
// `PopupState<D>` contains `parent`, `positioner_state`.
//
// This means Smithay already provides data structures for most XDG properties.
// Our `XdgToplevelData` and `XdgPopupData` might be redundant if they only store standard XDG states.
// They become useful if we add custom compositor-specific state beyond what XDG protocol defines
// or if we want a different organization.
//
// For `get_toplevel` and `get_popup`, Smithay's `XdgShellHandler::surface_added` is called,
// which in turn calls `self.new_toplevel()` or `self.new_popup()`. These typically
// associate Smithay's `ToplevelState` or `PopupState` with the `XdgSurfaceUserData.role`.
// Our `Dispatch` implementation for `xdg_surface` needs to coordinate with this.
// The `data_init.init(id, xdg_toplevel_user_data)` uses Smithay's `XdgSurfaceUserData`.
// This is correct for initializing the `xdg_toplevel` or `xdg_popup` resource itself.
//
// The main question is where *our* `XdgToplevelData` / `XdgPopupData` should live.
// 1. Inside our `XdgSurfaceData` (associated with `WlSurface`). This seems logical.
// 2. As part of a custom `UserData` for `xdg_toplevel`/`xdg_popup` resources (instead of just `XdgSurfaceUserData`).
//
// The current code logs the creation of `XdgToplevelData` but doesn't store it in `XdgSurfaceData`.
// This needs to be addressed by modifying `XdgSurfaceData` in `xdg.rs` and then storing it.
//
// Re-evaluation of `get_xdg_surface` in the previous step:
// It created `XdgSurfaceData` and `Window`.
// `surface.set_data(xdg_surface_data);`
// This should be `surface.set_data(Mutex::new(xdg_surface_data));` for the `get_surface_data` helper to work.
// Let's assume this was intended.
//
// The `Window` object created in `get_xdg_surface` was generic: `Window::new(WindowSurfaceType::Xdg(None))`.
// When `get_toplevel` or `get_popup` is called, this `Window` needs to be "specialized" or replaced.
// Smithay's `Window::new_xdg_toplevel(dh, surface.clone())` or `Window::new_xdg_popup(...)`
// are the typical ways to create these.
// This implies that the `Window` object in `XdgSurfaceData` might be better created/finalized
// when the role is known (i.e., in `get_toplevel`/`get_popup`).
// Or, `XdgSurfaceData.window` could be an `Option<Window>` initialized later.
// Or, the `Window` is created generically and then its internal XDG role state is configured.
// Smithay's `WindowSurfaceType::Xdg(Option<XdgShellSurfaceRoleAttributes<D>>)` suggests the latter.
// `XdgShellSurfaceRoleAttributes` can be `Toplevel(ToplevelState<D>)` or `Popup(PopupState<D>)`.
// So, in `get_toplevel`, we'd likely need to update `surface_data.window.surface_type_mut()`
// to `WindowSurfaceType::Xdg(Some(XdgShellSurfaceRoleAttributes::Toplevel(smithay_toplevel_state)))`.
// This `smithay_toplevel_state` would be the one Smithay manages. Our `XdgToplevelData` would be separate.

// Final plan for data storage:
// - `WlSurface` stores `Mutex<XdgSurfaceData>` (our custom data).
// - `XdgSurfaceData` stores our `XdgSurfaceRole`, `parent` (if popup), our `XdgToplevelData` or `XdgPopupData` (e.g. in an enum),
//   and potentially a `Window` object (or it's managed separately by Smithay's `Space`).
// - `xdg_surface` resource uses Smithay's `XdgSurfaceUserData`. This contains Smithay's role states.
// - `xdg_toplevel` / `xdg_popup` resources also use Smithay's `XdgSurfaceUserData` (or a specialized version if needed).
// This keeps our data separate but associated with `WlSurface`, while Smithay manages its own states for protocol handling.
// The `Window` object from Smithay is key for desktop integration (rendering, focus, etc.) and typically
// wraps/uses the Smithay role states. We need to ensure our `XdgSurfaceData::window` is this Smithay `Window`.

use smithay::reexports::wayland_protocols::xdg::shell::server::{
    xdg_toplevel::{self as xdg_toplevel_protocol, State as XdgToplevelStateSet}, // For sending configure states
    xdg_popup::{self as xdg_popup_protocol},
};
use smithay::wayland::seat::WaylandFocus; // For Window::is_activated
use crate::compositor::shell::xdg::XdgRoleSpecificData;


// Helper to get a mutable reference to XdgToplevelData
// This encapsulates the logic of accessing it through XdgSurfaceData.
// Note: This now expects XdgSurfaceData to hold XdgRoleSpecificData::Toplevel.
// It will panic if the role is incorrect, which is a programmer error.
fn with_toplevel_data<F, R>(surface: &WlSurface, mut callback: F) -> R
where
    F: FnMut(&mut XdgToplevelData) -> R,
{
    let surface_data_mutex = get_surface_data(surface);
    let mut surface_data = surface_data_mutex.lock().unwrap();
    match &mut surface_data.role_data {
        XdgRoleSpecificData::Toplevel(toplevel_data) => callback(toplevel_data),
        _ => panic!("Attempted to access toplevel data for a surface that is not a toplevel."),
    }
}

// Helper to get a mutable reference to XdgPopupData
fn with_popup_data<F, R>(surface: &WlSurface, mut callback: F) -> R
where
    F: FnMut(&mut XdgPopupData) -> R,
{
    let surface_data_mutex = get_surface_data(surface);
    let mut surface_data = surface_data_mutex.lock().unwrap();
    match &mut surface_data.role_data {
        XdgRoleSpecificData::Popup(popup_data) => callback(popup_data),
        _ => panic!("Attempted to access popup data for a surface that is not a popup."),
    }
}

// Helper to get XdgSurfaceData from a WlSurface, fallibly.
// Used when we don't want to panic if the data is missing or of the wrong type.
pub(crate) fn with_surface_data_fallible<F, R>(surface: &WlSurface, callback: F) -> Option<R>
where
    F: FnOnce(&mut XdgSurfaceData) -> R,
{
    surface.data::<Mutex<XdgSurfaceData>>().map(|mutex| {
        let mut guard = mutex.lock().unwrap();
        callback(&mut guard)
    })
}


impl<D> Dispatch<xdg_toplevel_protocol::XdgToplevel, XdgSurfaceUserData, D> for XdgShellState
where
    }
}


impl<D> Dispatch<xdg_toplevel_protocol::XdgToplevel, XdgSurfaceUserData, D> for XdgShellState
where
    D: Dispatch<xdg_toplevel_protocol::XdgToplevel, XdgSurfaceUserData> + XdgShellHandler + 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        resource: &xdg_toplevel_protocol::XdgToplevel,
        request: xdg_toplevel_protocol::Request,
        _data: &XdgSurfaceUserData, // Smithay's user data for the resource
        dh: &DisplayHandle,
        _data_init: &mut DataInit<'_, D>,
    ) {
        let wl_surface = resource.wl_surface();

        match request {
            xdg_toplevel_protocol::Request::Destroy => {
                // ANCHOR: XdgToplevel::destroy
                // Smithay handles resource destruction.
                // Our XdgToplevelData is part of XdgSurfaceData, which should be cleaned up
                // when WlSurface is destroyed if it's stored in its UserDataMap.
                // If Window needs explicit cleanup related to toplevel role, do it here.
                log::debug!("XdgToplevel destroyed for wl_surface: {:?}", wl_surface);
            }
            xdg_toplevel_protocol::Request::SetParent { parent } => {
                // ANCHOR: XdgToplevel::set_parent
                let parent_wl = parent.map(|t| t.wl_surface().clone());
                with_toplevel_data(wl_surface, |toplevel_data| {
                    toplevel_data.parent = parent_wl;
                });
                // TODO: This might affect window stacking order or behavior.
                // The compositor's window management logic would need to be informed.
            }
            xdg_toplevel_protocol::Request::SetTitle { title } => {
                // ANCHOR: XdgToplevel::set_title
                with_toplevel_data(wl_surface, |toplevel_data| {
                    toplevel_data.title = Some(title);
                });
                // TODO: Inform the Window representation or UI to update title display.
                // get_surface_data(wl_surface).lock().unwrap().window.set_title(title);
            }
            xdg_toplevel_protocol::Request::SetAppId { app_id } => {
                // ANCHOR: XdgToplevel::set_app_id
                with_toplevel_data(wl_surface, |toplevel_data| {
                    toplevel_data.app_id = Some(app_id);
                });
                // TODO: Inform the Window representation.
                // get_surface_data(wl_surface).lock().unwrap().window.set_app_id(app_id);
            }
            xdg_toplevel_protocol::Request::ShowWindowMenu { seat, serial, x, y } => {
                // ANCHOR: XdgToplevel::show_window_menu
                log::info!(
                    "Client requested window menu for surface {:?} at ({}, {}) with serial {} by seat {:?}",
                    wl_surface, x, y, serial, seat
                );
                // TODO: Implement window menu logic. This usually involves compositor-specific UI.
                // This might involve interaction with the seat and focus.
            }
            xdg_toplevel_protocol::Request::Move { seat, serial } => {
                // ANCHOR: XdgToplevel::move
                log::info!("Client requested move for surface {:?} with serial {} by seat {:?}", wl_surface, serial, seat);
                // TODO: Initiate an interactive move. This requires input system integration.
                // state.interactive_move(get_surface_data(wl_surface).lock().unwrap().window, seat, serial);
            }
            xdg_toplevel_protocol::Request::Resize { seat, serial, edges } => {
                // ANCHOR: XdgToplevel::resize
                log::info!(
                    "Client requested resize for surface {:?} with serial {} by seat {:?}, edges: {:?}",
                    wl_surface, serial, seat, edges
                );
                // TODO: Initiate an interactive resize. Requires input system integration.
                // state.interactive_resize(get_surface_data(wl_surface).lock().unwrap().window, seat, serial, edges);
            }
            xdg_toplevel_protocol::Request::SetMaxSize { width, height } => {
                // ANCHOR: XdgToplevel::set_max_size
                with_toplevel_data(wl_surface, |toplevel_data| {
                    toplevel_data.max_size = Some((width, height));
                });
                // TODO: This affects how the compositor handles window sizing.
                // A configure event might be needed if this constraint changes current size.
            }
            xdg_toplevel_protocol::Request::SetMinSize { width, height } => {
                // ANCHOR: XdgToplevel::set_min_size
                with_toplevel_data(wl_surface, |toplevel_data| {
                    toplevel_data.min_size = Some((width, height));
                });
                // TODO: Similar to max_size, may require re-evaluation of window size/configure.
            }
            xdg_toplevel_protocol::Request::SetMaximized => {
                // ANCHOR: XdgToplevel::set_maximized
                log::info!("Client requested maximize for surface {:?}", wl_surface);
                // This is a request that the compositor should try to honor.
                // The actual state change happens after a configure + ack_configure cycle.
                // We update our *desired* state here or in a pending state.
                with_toplevel_data(wl_surface, |toplevel_data| {
                    toplevel_data.current_state.maximized = true; // Or a pending_state.maximized
                });
                // TODO: The main window manager should decide if it can maximize, then
                // send a configure event. For now, assume we will try.
                // state.request_state_change(wl_surface, ToplevelStateChange::Maximize);
                // For now, directly call send_configure, assuming XdgShellHandler will pick up the state.
                if let Some(xdg_surface) = XdgSurface::from_resource(resource.wl_surface()) {
                     smithay::wayland::shell::xdg::send_configure(dh, &xdg_surface);
                }
            }
            xdg_toplevel_protocol::Request::UnsetMaximized => {
                // ANCHOR: XdgToplevel::unset_maximized
                log::info!("Client requested unmaximize for surface {:?}", wl_surface);
                with_toplevel_data(wl_surface, |toplevel_data| {
                    toplevel_data.current_state.maximized = false;
                });
                // TODO: Similar to set_maximized, involves window manager and configure cycle.
                // state.request_state_change(wl_surface, ToplevelStateChange::Unmaximize);
                if let Some(xdg_surface) = XdgSurface::from_resource(resource.wl_surface()) {
                     smithay::wayland::shell::xdg::send_configure(dh, &xdg_surface);
                }
            }
            xdg_toplevel_protocol::Request::SetFullscreen { output } => {
                // ANCHOR: XdgToplevel::set_fullscreen
                log::info!("Client requested fullscreen for surface {:?} on output {:?}", wl_surface, output);
                with_toplevel_data(wl_surface, |toplevel_data| {
                    toplevel_data.current_state.fullscreen = true;
                    // TODO: Store the target output if your WM supports multi-monitor fullscreen.
                });
                // TODO: Window manager decides, then configure.
                // state.request_state_change(wl_surface, ToplevelStateChange::Fullscreen(output));
                if let Some(xdg_surface) = XdgSurface::from_resource(resource.wl_surface()) {
                     smithay::wayland::shell::xdg::send_configure(dh, &xdg_surface);
                }
            }
            xdg_toplevel_protocol::Request::UnsetFullscreen => {
                // ANCHOR: XdgToplevel::unset_fullscreen
                log::info!("Client requested unfullscreen for surface {:?}", wl_surface);
                with_toplevel_data(wl_surface, |toplevel_data| {
                    toplevel_data.current_state.fullscreen = false;
                });
                // TODO: Window manager decides, then configure.
                // state.request_state_change(wl_surface, ToplevelStateChange::Unfullscreen);
                if let Some(xdg_surface) = XdgSurface::from_resource(resource.wl_surface()) {
                     smithay::wayland::shell::xdg::send_configure(dh, &xdg_surface);
                }
            }
            xdg_toplevel_protocol::Request::SetMinimized => {
                // ANCHOR: XdgToplevel::set_minimized
                log::info!("Client requested minimize for surface {:?}", wl_surface);
                with_toplevel_data(wl_surface, |toplevel_data| {
                    toplevel_data.current_state.minimized = true;
                });
                // TODO: Window manager action. Minimizing usually involves hiding the window
                // and not sending typical configure events for it until unminimized.
                // The client might not expect a configure for this, but its state is changed.
                // state.request_state_change(wl_surface, ToplevelStateChange::Minimize);
                // A configure might still be sent to signal other state changes or if minimization
                // is purely a hint to the client (rare).
                // Smithay's XdgShellHandler might send a configure if other states changed.
            }
            _ => unimplemented!(),
        }
    }
}

// In XdgShellHandler trait implementation (likely in your main compositor state):
// fn configure_toplevel(&mut self, dh: &DisplayHandle, surface: &WlSurface, xdg_surface: &XdgSurface, xdg_toplevel: &xdg_toplevel_protocol::XdgToplevel, serial: Serial) {
//    let (width, height, states) = with_toplevel_data(surface, |toplevel_data| {
//        // Determine width/height based on compositor layout and toplevel_data constraints
//        let current_size = self.get_window_size(&toplevel_data); // Example
//
//        let mut states_vec = Vec::new();
//        if toplevel_data.current_state.maximized { states_vec.push(XdgToplevelStateSet::Maximized); }
//        if toplevel_data.current_state.fullscreen { states_vec.push(XdgToplevelStateSet::Fullscreen); }
//        if toplevel_data.current_state.resizing { states_vec.push(XdgToplevelStateSet::Resizing); }
//        // Check window focus for activated state
//        let is_activated = get_surface_data(surface).unwrap().window.is_activated();
//        if is_activated { states_vec.push(XdgToplevelStateSet::Activated); }
//
//        (current_size.0, current_size.1, states_vec)
//    });
//
//    xdg_toplevel.configure(serial, width, height, states);
// }
//
// fn close_toplevel(&mut self, xdg_toplevel: &xdg_toplevel_protocol::XdgToplevel) {
//    xdg_toplevel.close();
// }


impl<D> Dispatch<xdg_popup_protocol::XdgPopup, XdgSurfaceUserData, D> for XdgShellState
where
    D: Dispatch<xdg_popup_protocol::XdgPopup, XdgSurfaceUserData> + XdgShellHandler + 'static,
{
    fn request(
        _state: &mut D,
        _client: &Client,
        resource: &xdg_popup_protocol::XdgPopup,
        request: xdg_popup_protocol::Request,
        _data: &XdgSurfaceUserData,
        _dh: &DisplayHandle,
        _data_init: &mut DataInit<'_, D>,
    ) {
        let wl_surface = resource.wl_surface();

        match request {
            xdg_popup_protocol::Request::Destroy => {
                // ANCHOR: XdgPopup::destroy
                // Smithay handles resource destruction.
                // Our XdgPopupData is part of XdgSurfaceData.
                log::debug!("XdgPopup destroyed for wl_surface: {:?}", wl_surface);
            }
            xdg_popup_protocol::Request::Grab { seat, serial } => {
                // ANCHOR: XdgPopup::grab
                log::info!(
                    "Client requested grab for popup {:?} with serial {} by seat {:?}",
                    wl_surface, serial, seat
                );
                // TODO: Implement popup grab logic. This is complex and involves:
                // - Ensuring this client is allowed to make the grab (e.g., focus).
                // - Interacting with the input system to redirect input to this popup.
                // - Handling implicit grabs (e.g., when a button is pressed over a client surface
                //   and a popup appears under the cursor).
                // - Sending xdg_popup.popup_done when the grab is released.
                // state.popup_grab(get_surface_data(wl_surface).lock().unwrap().window, seat, serial);
            }
            xdg_popup_protocol::Request::Reposition { positioner, token } => {
                // ANCHOR: XdgPopup::reposition
                log::info!(
                    "Client requested reposition for popup {:?} with positioner {:?} and token {}",
                    wl_surface, positioner, token
                );
                // TODO: Implement repositioning logic.
                // This involves using the xdg_positioner associated with `positioner_resource`
                // to calculate a new position for the popup, then sending a configure.
                // The `token` is used in the `repositioned` event.
                // state.reposition_popup(get_surface_data(wl_surface).lock().unwrap().window, positioner, token);
                // For now, just acknowledge by sending a configure if needed, or the repositioned event.
                // This is a simplification. A real implementation calculates new geometry.
                resource.repositioned(token); // Simplistic: assume repositioned to same spot.
            }
            _ => unimplemented!(),
        }
    }
}

// In XdgShellHandler trait implementation:
// fn configure_popup(&mut self, dh: &DisplayHandle, surface: &WlSurface, xdg_surface: &XdgSurface, xdg_popup: &xdg_popup_protocol::XdgPopup, serial: Serial) {
//    let (x, y, width, height) = with_popup_data(surface, |popup_data| {
//        // Calculate popup geometry based on parent, positioner, etc.
//        // This is highly compositor-specific.
//        let geometry = self.calculate_popup_geometry(&popup_data); // Example
//        (geometry.x, geometry.y, geometry.width, geometry.height)
//    });
//    xdg_popup.configure(serial, x, y, width, height);
// }
//
// fn dismiss_popup(&mut self, surface: &WlSurface, xdg_surface: &XdgSurface, xdg_popup: &xdg_popup_protocol::XdgPopup) {
//    // Mark the popup as dismissed. This might involve unmapping it, etc.
//    // Then send popup_done.
//    xdg_popup.popup_done();
//    // Clean up our state for this popup, potentially remove from window list.
//    log::info!("Popup {:?} dismissed", xdg_surface.wl_surface());
// }


// Previous Dispatch<xdg_surface::XdgSurface, ...> needs update for XdgRoleSpecificData
// Specifically in get_toplevel and get_popup to correctly initialize XdgRoleSpecificData.

impl<D> Dispatch<xdg_surface::XdgSurface, XdgSurfaceUserData, D> for XdgShellState
where
    D: Dispatch<xdg_surface::XdgSurface, XdgSurfaceUserData>,
    D: Dispatch<xdg_toplevel_protocol::XdgToplevel, XdgSurfaceUserData>,
    D: Dispatch<xdg_popup_protocol::XdgPopup, XdgSurfaceUserData>,
    D: Dispatch<smithay::reexports::wayland_server::protocol::xdg_positioner::XdgPositioner, XdgPositionerUserData>, // Added for completeness
    D: XdgShellHandler,
    D: 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        xdg_surface: &xdg_surface::XdgSurface,
        request: xdg_surface::Request,
        _data: &XdgSurfaceUserData,
        dh: &DisplayHandle,
        data_init: &mut DataInit<'_, D>,
    ) {
        let wl_surface = xdg_surface.wl_surface();

        // This is where we correctly initialize XdgRoleSpecificData
        // We need to ensure that when get_xdg_surface was called, XdgSurfaceData was
        // initialized with XdgRoleSpecificData::None.
        // Let's grab it first. If it's not Mutex<XdgSurfaceData>, the helper will panic.
        // This was set in the xdg_wm_base handler.

        match request {
            xdg_surface::Request::Destroy => {
                log::debug!("XdgSurface destroyed for wl_surface: {:?}", wl_surface);
                // Cleanup handled by UserDataMap dropping XdgSurfaceData
            }
            xdg_surface::Request::GetToplevel { id } => {
                // Lock early to modify role_data
                let surface_data_mutex = get_surface_data(wl_surface);
                let mut surface_data = surface_data_mutex.lock().unwrap();

                if !matches!(surface_data.role_data, XdgRoleSpecificData::None) {
                     // TODO: Post protocol error (role already assigned)
                    log::error!("Surface already has a role, cannot assign toplevel.");
                    // xdg_surface.post_error(xdg_wm_base::Error::Role, "Surface already has a role");
                    return;
                }

                surface_data.role = XdgSurfaceRole::Toplevel;
                surface_data.role_data = XdgRoleSpecificData::Toplevel(XdgToplevelData::default());

                // Drop the lock before calling data_init or other functions that might re-enter
                drop(surface_data);

                log::debug!("XdgToplevel role assigned and data initialized for wl_surface: {:?}", wl_surface);

                let xdg_toplevel_user_data = xdg_surface.data::<XdgSurfaceUserData>().unwrap().clone();
                data_init.init(id, xdg_toplevel_user_data);

                // Smithay's XdgShellHandler::surface_added will be called, which often calls new_toplevel.
                // Then a configure should be sent.
                smithay::wayland::shell::xdg::send_configure(dh, xdg_surface);
            }
            xdg_surface::Request::GetPopup { id, parent_xdg_surface, positioner } => {
                let surface_data_mutex = get_surface_data(wl_surface);
                let mut surface_data = surface_data_mutex.lock().unwrap();

                if !matches!(surface_data.role_data, XdgRoleSpecificData::None) {
                    // TODO: Post protocol error
                    log::error!("Surface already has a role, cannot assign popup.");
                    // xdg_surface.post_error(xdg_wm_base::Error::Role, "Surface already has a role");
                    return;
                }

                let parent_wl = match parent_xdg_surface {
                    Some(parent) => parent.wl_surface().clone(),
                    None => {
                        log::error!("Popup creation without a parent XDG surface is not fully handled yet.");
                        // xdg_surface.post_error(xdg_wm_base::Error::InvalidPopupParent, "Popup parent required");
                        return;
                    }
                };

                surface_data.role = XdgSurfaceRole::Popup;
                surface_data.parent = Some(parent_wl.clone());
                surface_data.role_data = XdgRoleSpecificData::Popup(XdgPopupData {
                    parent: parent_wl,
                    committed: false,
                    // TODO: positioner data
                });

                drop(surface_data);

                log::debug!("XdgPopup role assigned and data initialized for wl_surface: {:?}", wl_surface);

                let xdg_popup_user_data = xdg_surface.data::<XdgSurfaceUserData>().unwrap().clone();
                data_init.init(id, xdg_popup_user_data);

                // Smithay's XdgShellHandler::surface_added -> new_popup.
                // Then a configure should be sent.
                smithay::wayland::shell::xdg::send_configure(dh, xdg_surface);
            }
            xdg_surface::Request::SetWindowGeometry { x, y, width, height } => {
                let surface_data_mutex = get_surface_data(wl_surface);
                let surface_data = surface_data_mutex.lock().unwrap(); // read-only access to our data here

                log::debug!(
                    "Client set window geometry: x={}, y={}, width={}, height={}",
                    x, y, width, height
                );
                let user_data = xdg_surface.data::<XdgSurfaceUserData>().unwrap();
                let mut cached_state = user_data.cached_state.lock().unwrap();
                cached_state.window_geometry = Some(smithay::utils::Rectangle::from_loc_and_size(
                    (x, y),
                    (width, height),
                ));
            }
            xdg_surface::Request::AckConfigure { serial } => {
                log::debug!("Client acknowledged configure with serial: {}", serial);

                // The XdgShellHandler ack_configure should handle committing pending states
                // to our XdgToplevelData/XdgPopupData as well if needed.
                state.ack_configure(serial, wl_surface.clone(), xdg_surface.clone()).unwrap_or_else(|err| {
                    log::warn!("Error processing ack_configure: {:?}", err);
                });

                // If it was a popup, mark it as committed after first ack
                let surface_data_mutex = get_surface_data(wl_surface);
                let mut surface_data = surface_data_mutex.lock().unwrap();
                if let XdgRoleSpecificData::Popup(popup_data) = &mut surface_data.role_data {
                    if !popup_data.committed {
                        popup_data.committed = true;
                        log::debug!("Popup {:?} committed initial configure.", wl_surface);
                        // ANCHOR: Popup initial commit may trigger focus changes or other compositor actions.
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
}

// Ensure xdg_wm_base.get_xdg_surface initializes XdgSurfaceData correctly with XdgRoleSpecificData::None
// and uses Mutex for XdgSurfaceData.

impl<D> Dispatch<XdgWmBase, XdgShellState, D> for XdgShellState
where
    D: GlobalDispatch<XdgWmBase, XdgShellGlobalData>, // Required for GlobalDispatch constraint on XdgShellState itself
    D: Dispatch<XdgWmBase, XdgShellState>,
    D: Dispatch<smithay::reexports::wayland_server::protocol::xdg_positioner::XdgPositioner, XdgPositionerUserData>,
    D: Dispatch<xdg_surface::XdgSurface, XdgSurfaceUserData>,
    D: Dispatch<xdg_toplevel_protocol::XdgToplevel, XdgSurfaceUserData>, // Added
    D: Dispatch<xdg_popup_protocol::XdgPopup, XdgSurfaceUserData>,       // Added
    D: XdgShellHandler,
    D: 'static,
{
    fn request(
        state: &mut D, // Changed from _state to state to use it for XdgShellHandler calls
        _client: &Client,
        _resource: &XdgWmBase,
        request: Request,
        _data: &XdgShellState, // This is our user_data for XdgWmBase, which is XdgShellState itself.
        dh: &DisplayHandle,
        data_init: &mut DataInit<'_, D>,
    ) {
        match request {
            Request::Destroy => {
                // (Cleanup as before)
            }
            Request::CreatePositioner { id } => {
                data_init.init(id, XdgPositionerUserData {});
            }
            Request::GetXdgSurface { id, surface } => {
                // ANCHOR: XdgWmBase::get_xdg_surface (needs update for Mutex and XdgRoleSpecificData::None)

                // Check if surface already has XdgSurfaceData (from another shell, or already assigned)
                if surface.data::<Mutex<XdgSurfaceData>>().is_some() {
                    // TODO: Post protocol error (wl_surface has another role)
                    // _resource.post_error(xdg_wm_base::Error::Role, "Surface already has a role.");
                    log::error!("Attempted to get xdg_surface for a surface that already has XdgSurfaceData.");
                    return;
                }

                let xdg_surface_data = XdgSurfaceData {
                    // Role is initially unknown until get_toplevel/get_popup.
                    // Let's use a placeholder role or an "Unassigned" variant if XdgSurfaceRole had one.
                    // For now, Toplevel is a placeholder until a role request comes.
                    // This will be overwritten by get_toplevel/get_popup.
                    role: XdgSurfaceRole::Toplevel, // Placeholder, will be set correctly in GetToplevel/GetPopup
                    role_data: XdgRoleSpecificData::None, // Initialize with no specific role data
                    parent: None,
                    // ANCHOR: Window creation needs careful thought.
                    // If Window is created here, it's generic. It needs to be specialized
                    // or replaced when get_toplevel/get_popup is called.
                    // Alternatively, Window creation could be deferred to get_toplevel/get_popup.
                    // Smithay's examples often create the Window (e.g. Window::new_xdg_toplevel)
                    // inside the XdgShellHandler::new_toplevel callback.
                    // For now, let's assume a generic Window that might be updated.
                    window: Window::new(WindowSurfaceType::Xdg(None)),
                };

                // Store our XdgSurfaceData within a Mutex on the WlSurface.
                surface.set_data(Mutex::new(xdg_surface_data));

                // Initialize the xdg_surface resource with Smithay's XdgSurfaceUserData.
                let user_data = XdgSurfaceUserData {
                    // Smithay's XdgSurfaceUserData will internally handle role checks
                    // when get_toplevel/get_popup are called on the xdg_surface.
                };
                data_init.init(id, user_data);

                // Call XdgShellHandler::new_surface callback
                state.new_surface(&surface, xdg_surface);


            }
            Request::Pong { serial } => {
                log::debug!("Client responded to ping with serial: {}", serial);
                state.pong(serial);
            }
            _ => unimplemented!(),
        }
    }
}
