use crate::compositor::core::state::DesktopState;
use smithay::wayland::shell::xdg::{
    decoration::{
        XdgDecorationHandler, XdgDecorationState, // XdgDecorationState will be added to DesktopState later
        ToplevelDecorationUserData,
    },
    ToplevelSurface,
};
// Import Mode directly as XdgDecorationMode
use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::{
    Mode as XdgDecorationMode, ZxdgToplevelDecorationV1,
};
use tracing::{debug, info, warn};

impl XdgDecorationHandler for DesktopState {
    // UserData for the zxdg_toplevel_decoration_v1 object itself.
    // Smithay's ToplevelDecorationUserData stores the current mode.
    type DecorationUserData = ToplevelDecorationUserData;

    fn decoration_state(&mut self) -> &mut XdgDecorationState {
        // This field will be added to DesktopState in a subsequent step.
        // For now, this line would cause a compile error until DesktopState is updated.
        // However, the subtask asks to implement the handler assuming it exists.
        &mut self.xdg_decoration_state
    }

    fn new_decoration(&mut self, _toplevel: &ToplevelSurface, decoration: &ZxdgToplevelDecorationV1) {
        info!(
            "New XDG Toplevel Decoration created for surface {:?}",
            _toplevel.wl_surface().id()
        );

        // Compositor policy: default to client-side decorations.
        let preferred_mode = XdgDecorationMode::ClientSide;
        debug!(
            "TODO: Consult WindowManagementPolicyService for preferred decoration mode for surface {:?}",
            _toplevel.wl_surface().id()
        );

        // Store this initial mode in the decoration's UserData.
        // ToplevelDecorationUserData is initialized by Smithay when the decoration object is created.
        if let Some(data) = decoration.user_data().get::<ToplevelDecorationUserData>() {
            data.current_mode.store(Some(preferred_mode));
        } else {
            warn!(
                "Could not get ToplevelDecorationUserData for new decoration object of surface {:?}",
                _toplevel.wl_surface().id()
            );
            // Even if UserData is missing (which shouldn't happen if Smithay initializes it),
            // still try to configure the client with the preferred mode.
        }

        // Send the configure event to the client.
        decoration.configure(preferred_mode);
        info!(
            "Configured decoration for surface {:?} with initial mode: {:?}",
            _toplevel.wl_surface().id(), preferred_mode
        );
    }

    fn request_mode(
        &mut self,
        _toplevel: &ToplevelSurface,
        decoration: &ZxdgToplevelDecorationV1,
        mode: XdgDecorationMode,
    ) {
        info!(
            "Client for surface {:?} requested decoration mode: {:?}",
            _toplevel.wl_surface().id(),
            mode
        );
        debug!(
            "TODO: Consult WindowManagementPolicyService to determine if mode {:?} is acceptable for surface {:?}",
            mode,
            _toplevel.wl_surface().id()
        );

        // Compositor Policy (Simplified for now): Accept any valid mode requested.
        // A real compositor might restrict modes (e.g., force server-side if client requests None).
        let accepted_mode = mode; // Assuming we accept what client requests for now.

        if let Some(data) = decoration.user_data().get::<ToplevelDecorationUserData>() {
            data.current_mode.store(Some(accepted_mode));
        } else {
            warn!(
                "Could not get ToplevelDecorationUserData for decoration object of surface {:?}",
                _toplevel.wl_surface().id()
            );
        }
        decoration.configure(accepted_mode);
        info!(
            "Configured decoration for surface {:?} with requested mode: {:?}",
            _toplevel.wl_surface().id(), accepted_mode
        );

        // Example of rejecting a mode (if policy dictated):
        // let policy_allows_mode = false; // Example
        // if policy_allows_mode {
        //     if let Some(data) = decoration.user_data().get::<ToplevelDecorationUserData>() {
        //         data.current_mode.store(Some(mode));
        //     }
        //     decoration.configure(mode);
        // } else {
        //     warn!(
        //         "Compositor rejected mode {:?} for surface {:?}. Re-configuring with preferred/current mode.",
        //         mode,
        //         _toplevel.wl_surface().id()
        //     );
        //     let current_or_preferred_mode = decoration
        //         .user_data()
        //         .get::<ToplevelDecorationUserData>()
        //         .and_then(|d| d.current_mode.load())
        //         .unwrap_or(XdgDecorationMode::ClientSide); // Fallback to client-side
        //     decoration.configure(current_or_preferred_mode);
        // }
    }

    fn unset_mode(&mut self, _toplevel: &ToplevelSurface, decoration: &ZxdgToplevelDecorationV1) {
        info!(
            "Client for surface {:?} requested to unset decoration mode.",
            _toplevel.wl_surface().id()
        );

        // When client unsets mode, compositor should choose its preferred mode.
        let preferred_mode = XdgDecorationMode::ClientSide; // Default preference
        debug!(
            "TODO: Consult WindowManagementPolicyService for preferred decoration mode on unset for surface {:?}",
            _toplevel.wl_surface().id()
        );

        if let Some(data) = decoration.user_data().get::<ToplevelDecorationUserData>() {
            data.current_mode.store(Some(preferred_mode));
        } else {
            warn!(
                "Could not get ToplevelDecorationUserData for decoration object of surface {:?}",
                _toplevel.wl_surface().id()
            );
        }
        decoration.configure(preferred_mode);
        info!(
            "Configured decoration for surface {:?} with preferred mode after unset: {:?}",
            _toplevel.wl_surface().id(), preferred_mode
        );
    }
}

// --- GlobalDispatch Implementation ---
use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_decoration_manager_v1::XdgDecorationManagerV1;
use smithay::reexports::wayland_server::{GlobalDispatch, New, DataInit, Client, DisplayHandle as WaylandDisplayHandle};
// DesktopState is already imported at the top of the file.

impl GlobalDispatch<XdgDecorationManagerV1, ()> for DesktopState {
    type UserData = (); // No specific per-client global state for the manager itself.

    fn bind(
        _state: &mut Self, // DesktopState, XdgDecorationState is accessed via delegate
        _handle: &WaylandDisplayHandle,
        _client: &Client,
        resource: New<XdgDecorationManagerV1>,
        _global_data: &Self::UserData, // This is &()
        data_init: &mut DataInit<'_, Self>,
    ) {
        info!(client_id = ?_client.id(), resource_id = ?resource.id(), "Client binding to xdg_decoration_manager_v1");

        // Initialize the XdgDecorationManagerV1 resource.
        // Smithay's XdgDecorationState will handle requests on this manager via the delegate macros.
        data_init.init(resource, ());
    }
}
