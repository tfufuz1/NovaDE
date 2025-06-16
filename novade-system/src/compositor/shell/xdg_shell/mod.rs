// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # XDG Shell Implementation
//!
//! This module implements the XDG shell protocol for the compositor.
//! It defines errors, handlers, and manages the XDG shell global.
//! Types are re-exported from the `types` submodule.

pub mod errors;
pub mod types;
pub mod handlers;
pub mod xdg_surface; // ANCHOR: AddXdgSurfaceModule

pub use errors::XdgShellError;
pub use types::*; // Re-export types like ManagedWindow, XdgSurfaceUserData

use crate::compositor::core::state::DesktopState;
use smithay::reexports::wayland_server::{
    Client, DisplayHandle, GlobalDispatch, DataInit, New, Resource, Dispatch, ClientData,
};
use smithay::reexports::wayland_protocols::xdg::shell::server::{
    xdg_wm_base::XdgWmBase,
    xdg_surface::XdgSurface as WaylandProtocolXdgSurface, // Protocol object for requests & errors
    xdg_surface::Request as XdgSurfaceRequest,
    xdg_surface::Error as XdgSurfaceErrorMethodLevel, // For use with resource.post_error
};
use smithay::wayland::shell::xdg::{
    XdgShellState, XdgWmBaseClientData, XdgActivationState, XdgSurfaceData,
    XdgWmBaseHandler, XdgSurface as SmithayXdgSurface, // Smithay's wrapper object
    Configure as XdgSurfaceConfigureData, // Data for AckConfigure
};
use smithay::delegate_xdg_wm_base;
use smithay::delegate_xdg_surface; // Import the delegate macro for XdgSurface
use std::sync::Arc;


// ANCHOR: GlobalDispatchXdgWmBase
// GlobalDispatch for XdgWmBase remains largely the same for global object creation.
impl GlobalDispatch<XdgWmBase, Arc<XdgWmBaseClientData>> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<XdgWmBase>,
        global_data: &Arc<XdgWmBaseClientData>, // Renamed for clarity
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!(client_id = ?_client.id(), "Client bound to XdgWmBase global");
        // Initialize the XdgWmBase resource with its client-specific data (XdgWmBaseClientData)
        // and our DesktopState as the dispatcher.
        data_init.init(resource, global_data.clone());
    }

    fn can_view(_client: Client, _global_data: &Arc<XdgWmBaseClientData>) -> bool {
        true // All clients can see the XDG WM Base global
    }
}
// ANCHOR_END: GlobalDispatchXdgWmBase

// ANCHOR: DelegateXdgWmBase
// Delegate XdgWmBase requests to DesktopState
delegate_xdg_wm_base!(DesktopState);

impl XdgWmBaseHandler for DesktopState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    // ANCHOR: NewXdgSurfaceHandler
    fn new_xdg_surface(
        &mut self,
        _dh: &DisplayHandle,
        _client: Client, // Renamed from client for consistency, though not used here
        xdg_surface_smithay: SmithayXdgSurface, // Renamed for clarity
    ) {
        let wl_surface = xdg_surface_smithay.wl_surface().clone();
        tracing::info!(
            // client_id = ?_client.id(), // Not strictly needed for this log message
            surface_id = ?wl_surface.id(),
            xdg_surface_protocol_id = ?xdg_surface_smithay.resource().id(),
            "XdgWmBase: new_xdg_surface request processed. Attaching XdgSurfaceUserData."
        );

        let xdg_surface_user_data = Arc::new(XdgSurfaceUserData::new(wl_surface.clone()));
        xdg_surface_smithay.user_data().insert_if_missing_threadsafe(|| xdg_surface_user_data.clone());

        if wl_surface.data_map().get::<XdgSurfaceData>().is_none() {
            tracing::warn!(surface_id = ?wl_surface.id(), "WlSurface underlying SmithayXdgSurface {:?} does not have Smithay's XdgSurfaceData. This is unexpected.", xdg_surface_smithay.resource().id());
        }

        tracing::info!(
            xdg_surface_protocol_id = ?xdg_surface_smithay.resource().id(),
            "XdgSurfaceUserData attached to new SmithayXdgSurface."
        );
    }
}
// ANCHOR_END: DelegateXdgWmBase

// ANCHOR: DelegateXdgSurface
// Delegate XdgSurface requests to DesktopState
// The UserData for XdgSurface (protocol object) will be Arc<XdgSurfaceUserData>,
// which we attached in XdgWmBaseHandler::new_xdg_surface.
delegate_xdg_surface!(DesktopState);

impl Dispatch<WaylandProtocolXdgSurface, Arc<XdgSurfaceUserData>> for DesktopState {
    fn request(
        state: &mut Self,
        _client: &Client,
        xdg_surface_resource: &WaylandProtocolXdgSurface,
        request: XdgSurfaceRequest,
        data: &Arc<XdgSurfaceUserData>,
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        let surface_id = data.wl_surface.id(); // WlSurface ID from our user data
        tracing::debug!(resource_id = ?xdg_surface_resource.id(), ?request, %surface_id, "XdgSurface request received");

        match request {
            XdgSurfaceRequest::Destroy => {
                // This is the client explicitly destroying the xdg_surface object.
                // Smithay handles cleanup of the XdgSurface wrapper object (SmithayXdgSurface)
                // and associated role objects (ToplevelSurface, PopupSurface) if they exist.
                // Our XdgShellHandler::{toplevel_destroyed, popup_destroyed} will be called by Smithay
                // as part of that cleanup if a role was assigned.
                // Here, we primarily ensure our XdgSurfaceUserData reflects this if no role was assigned,
                // or if destruction happens before role-specific destruction handlers.
                let mut xdg_user_data_state = data.state.lock().unwrap();
                if *xdg_user_data_state != XdgSurfaceState::Destroyed {
                    *xdg_user_data_state = XdgSurfaceState::Destroyed;
                    tracing::info!(%surface_id, xdg_surface_protocol_id = ?xdg_surface_resource.id(), "XdgSurface explicitly destroyed by client. Marked state as Destroyed.");
                } else {
                    tracing::info!(%surface_id, xdg_surface_protocol_id = ?xdg_surface_resource.id(), "XdgSurface already marked Destroyed, client sent another destroy.");
                }
                // Smithay will automatically mark the resource as inert. No need to post error for subsequent requests.
            }
            XdgSurfaceRequest::AckConfigure { serial } => {
                // ANCHOR_REF: AckConfigureLogic
                let mut xdg_user_data_state_guard = data.state.lock().unwrap(); // For lifecycle state
                let last_compositor_serial_guard = data.last_compositor_configure_serial.lock().unwrap();

                tracing::info!(
                    %surface_id,
                    xdg_surface_protocol_id = ?xdg_surface_resource.id(),
                    ack_serial = %serial, // Serial from the request (u32)
                    last_compositor_configure_serial = ?*last_compositor_serial_guard,
                    current_lifecycle_state = ?*xdg_user_data_state_guard,
                    "XdgSurface.AckConfigure received."
                );

                if *xdg_user_data_state_guard == XdgSurfaceState::Destroyed {
                    tracing::warn!(%surface_id, "Client acked configure for a destroyed xdg_surface. Ignoring.");
                    return;
                }

                // TODO: Add robust serial validation against last_compositor_configure_serial.
                // This involves ensuring `last_compositor_configure_serial` is correctly set when the compositor
                // sends a configure event (e.g., via xdg_toplevel.configure or xdg_popup.configure).
                // The serial provided in ack_configure should match one of these sent serials.
                // Smithay's `Serial` type is `struct Serial(u32);`.
                // If `serial` is a `u32`, it can be converted via `Serial(serial)` or `serial.into()`.

                let mut applied_pending_state = false;
                if *xdg_user_data_state_guard == XdgSurfaceState::PendingConfiguration {
                    // Typically, the first ack_configure after getting a role (toplevel/popup)
                    // moves the surface to a configured state.
                    // The serial validation is important here.

                    let pending_geom_guard = data.pending_window_geometry.lock().unwrap();
                    if pending_geom_guard.is_some() {
                        *data.current_window_geometry.lock().unwrap() = *pending_geom_guard;
                         tracing::debug!(%surface_id, "Applied pending window geometry: {:?}", *pending_geom_guard);
                    }
                    // We don't drop pending_geom_guard explicitly; it's released at scope end.

                    *data.last_acked_configure_serial.lock().unwrap() = Some(smithay::utils::Serial(serial));
                    *xdg_user_data_state_guard = XdgSurfaceState::Configured;
                    applied_pending_state = true;

                    tracing::info!(%surface_id, xdg_surface_protocol_id = ?xdg_surface_resource.id(), "XdgSurface state changed from PendingConfiguration to Configured. Applied pending geometry if any.");
                } else if *xdg_user_data_state_guard == XdgSurfaceState::Configured {
                    // If already configured, this ack might correspond to a later configure (e.g., resize).
                    // Role objects (ToplevelSurface, PopupSurface) primarily manage these cycles.
                    // We just update our record of the last acked serial.
                    *data.last_acked_configure_serial.lock().unwrap() = Some(smithay::utils::Serial(serial));
                    tracing::debug!(%surface_id, "XdgSurface already configured. Updated last_acked_serial to {}.", serial);
                }

                // Drop the main state guard before potentially calling other functions that might lock.
                drop(xdg_user_data_state_guard);
                drop(last_compositor_serial_guard);

                if applied_pending_state {
                    // `find_managed_window_by_wl_surface` is not defined in this file.
                    // This integration is deeper and would be part of DesktopState methods.
                    // if let Some(managed_window) = find_managed_window_by_wl_surface(state, &data.wl_surface) {
                    //      tracing::debug!(managed_window_id = ?managed_window.id, "Notified ManagedWindow of xdg_surface ack_configure (serial: {}).", serial);
                    // } else {
                    //      tracing::debug!(%surface_id, "AckConfigure for XDG surface not tied to a ManagedWindow (e.g., popup or pre-map toplevel).");
                    // }
                }
            }
            XdgSurfaceRequest::SetWindowGeometry { x, y, width, height } => {
                // ANCHOR: HandleSetWindowGeometry
                let new_geometry = smithay::utils::Rectangle::from_loc_and_size(
                    smithay::utils::Point::from((x,y)),
                    smithay::utils::Size::from((width, height))
                );
                *data.pending_window_geometry.lock().unwrap() = Some(new_geometry);
                tracing::info!(
                    %surface_id,
                    xdg_surface_protocol_id = ?xdg_surface_resource.id(),
                    geometry = ?new_geometry,
                    "XdgSurface.SetWindowGeometry request received. Pending geometry updated."
                );
                // This geometry is a hint from the client. It's stored in pending_window_geometry.
                // The compositor is not obligated to use this geometry.
                // It's typically applied when the client acks a configure sequence from a role object.
                // ANCHOR_END: HandleSetWindowGeometry
            }
            _ => {
                tracing::warn!(%surface_id, xdg_surface_protocol_id = ?xdg_surface_resource.id(), "Unhandled XdgSurface request: {:?}", request);
            }
        }
    }
}
// ANCHOR_END: DelegateXdgSurface


// Function to ensure XDG Shell global is active
// ANCHOR: CreateXdgShellGlobalFn
pub fn create_xdg_shell_global(
    _desktop_state: &DesktopState,
    _display_handle: &DisplayHandle,
) -> Result<(), String> {
    // Responsibility for global creation is typically within DesktopState::new() or where XdgShellState is initialized.
    // This function can serve as a verification or a placeholder if explicit call is needed later.
    tracing::info!("XDG WM Base global registration and XDG Surface dispatch are assumed to be handled by DesktopState initialization and delegate macros.");
    Ok(())
}
// ANCHOR_END: CreateXdgShellGlobalFn

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::core::state::DesktopState;
    use crate::compositor::shell::xdg_shell::types::{XdgSurfaceUserData, XdgSurfaceRole, XdgSurfaceState};
    use smithay::reexports::wayland_server::{
        Display, DisplayHandle, Client, protocol::wl_surface::WlSurface, UserData, Main,
        backend::{ClientId, GlobalId, Handle, ObjectData, ObjectId, DisconnectReason},
        globals::GlobalData,
        Interface, Message,
    };
    use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_surface::{
        XdgSurface as WaylandProtocolXdgSurface, // The Resource trait is on this
        Request as XdgSurfaceRequest,
        Error as XdgSurfaceError, // The error enum, not the method post_error
    };
    use smithay::wayland::shell::xdg::{XdgShellState, XdgActivationState, XDG_WM_BASE_VERSION};
    use std::sync::Arc;

    // Minimal test client data
    #[derive(Default, Clone)]
    struct TestClientData { user_data: UserData }
    impl ClientData for TestClientData {
        fn initialized(&self, _client_id: ClientId) {}
        fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
        fn data_map(&self) -> &UserData { &self.user_data }
    }

    // Mock ObjectData for WlSurface and XdgSurface resources
    #[derive(Default)]
    struct TestObjectData;
    impl<R: Interface + AsRef<Resource<R>> + Unpin + 'static> ObjectData<R> for TestObjectData {
        fn request(self: Arc<Self>, _handle: &Handle, _client_data: &mut dyn ClientData, _client_id: ClientId, _msg: Message<R>) -> Option<Arc<dyn ObjectData<R>>> { None }
        fn destroyed(self: Arc<Self>, _client_id: ClientId, _object_id: ObjectId) {}
    }

    // Helper to create a basic DesktopState, DisplayHandle, Client, and a WlSurface with XdgSurfaceUserData
    fn setup_test_environment_with_xdg_surface() -> (DesktopState, DisplayHandle, Client, Main<WaylandProtocolXdgSurface>, Arc<XdgSurfaceUserData>) {
        let mut display: Display<DesktopState> = Display::new().unwrap();
        let dh = display.handle();

        let xdg_activation_state = XdgActivationState::new();
        let (xdg_shell_state, _xdg_wm_base_global) = XdgShellState::new_with_activation(&dh, &xdg_activation_state);

        let mut desktop_state = DesktopState::new_for_test(xdg_shell_state.clone()); // Assuming a simplified constructor for tests

        // Register XDG WM Base global for XdgShellState to be functional with clients
        dh.create_global::<DesktopState, XdgWmBase, Arc<XdgWmBaseClientData>>(
            XDG_WM_BASE_VERSION,
            xdg_shell_state.xdg_wm_base_data().clone(),
        );

        let client = display.create_client(TestClientData::default().into());
        let wl_surface_main = client.create_object::<WlSurface, _>(&dh, WlSurface::interface().version, Arc::new(TestObjectData)).unwrap();

        // Create Smithay's XdgSurface (wrapper) and attach our XdgSurfaceUserData
        // This simulates what XdgWmBaseHandler::new_xdg_surface does.
        wl_surface_main.as_ref().data_map().insert_if_missing_threadsafe(|| Arc::new(smithay::wayland::compositor::SurfaceData::new(None, Rectangle::from_loc_and_size((0,0),(0,0))))); // Smithay needs this for XdgSurface::new_*
        let smithay_xdg_surface = SmithayXdgSurface::new_unassigned(wl_surface_main.as_ref().clone());
        let xdg_surface_user_data = Arc::new(XdgSurfaceUserData::new(wl_surface_main.as_ref().clone()));
        smithay_xdg_surface.user_data().insert_if_missing_threadsafe(|| xdg_surface_user_data.clone());

        // Create the WaylandProtocolXdgSurface resource (the one that Dispatch works on)
        // This is a bit simplified; normally Smithay creates this resource.
        // We need its UserData to be our XdgSurfaceUserData.
        let xdg_surface_protocol_resource = client.create_object_from_existing::<WaylandProtocolXdgSurface, _>(
            &dh,
            smithay_xdg_surface.resource().id(), // Use the ID from the SmithayXdgSurface's underlying resource
            xdg_surface_user_data.clone(), // Attach our user data directly
        ).unwrap();


        (desktop_state, dh, client, xdg_surface_protocol_resource, xdg_surface_user_data)
    }


    #[test]
    fn test_xdg_surface_ack_configure_state_transition() {
        let (mut state, dh, client, xdg_surface_resource, user_data) = setup_test_environment_with_xdg_surface();

        assert_eq!(*user_data.state.lock().unwrap(), XdgSurfaceState::PendingConfiguration);

        let request = XdgSurfaceRequest::AckConfigure { serial: 123 };
        let mut data_init = DataInit::new_dummy(); // Dummy DataInit

        DesktopState::request(&mut state, &client, &xdg_surface_resource, request, &user_data, &dh, &mut data_init);

        assert_eq!(*user_data.state.lock().unwrap(), XdgSurfaceState::Configured);
    }

    #[test]
    fn test_xdg_surface_destroy_request_marks_state_destroyed() {
        let (mut state, dh, client, xdg_surface_resource, user_data) = setup_test_environment_with_xdg_surface();

        assert_ne!(*user_data.state.lock().unwrap(), XdgSurfaceState::Destroyed);

        let request = XdgSurfaceRequest::Destroy;
        let mut data_init = DataInit::new_dummy(); // Dummy DataInit

        DesktopState::request(&mut state, &client, &xdg_surface_resource, request, &user_data, &dh, &mut data_init);

        assert_eq!(*user_data.state.lock().unwrap(), XdgSurfaceState::Destroyed);
    }

    #[test]
    fn test_xdg_surface_ack_configure_on_destroyed_surface() {
        let (mut state, dh, client, xdg_surface_resource, user_data) = setup_test_environment_with_xdg_surface();

        // Manually set state to Destroyed
        *user_data.state.lock().unwrap() = XdgSurfaceState::Destroyed;

        let request = XdgSurfaceRequest::AckConfigure { serial: 456 };
        let mut data_init = DataInit::new_dummy();

        DesktopState::request(&mut state, &client, &xdg_surface_resource, request, &user_data, &dh, &mut data_init);

        // State should remain Destroyed
        assert_eq!(*user_data.state.lock().unwrap(), XdgSurfaceState::Destroyed);
        // No error should be posted for ack_configure on a destroyed surface as per current logic.
    }

    // Note: Testing XdgWmBaseHandler::new_xdg_surface requires simulating a client calling xdg_wm_base.get_xdg_surface.
    // This is more involved and might need a proper client-server test setup or more advanced mocking.
    // The current tests focus on the Dispatch<WaylandProtocolXdgSurface, ...> logic.
}
