// ANCHOR: XdgSurfaceSimplifiedDefinition
use smithay::reexports::wayland_server::protocol::wl_surface;
use std::sync::{Arc, Mutex}; // Mutex might not be needed here if state is in UserData

// Re-export XdgSurfaceRole from types.rs if needed locally, or rely on full path.
// pub use super::types::XdgSurfaceRole;

// The XdgSurfaceState struct that was here previously has had its fields integrated
// into XdgSurfaceUserData in `types.rs`.
// The XdgSurfaceRole enum is also defined in `types.rs`.

// ANCHOR: XdgSurfaceStructSimplified
/// Represents an XDG surface (xdg_surface protocol object).
///
/// This struct itself doesn't hold the complex state data directly anymore.
/// That state (like role, configure serials, geometries) is now part of
/// `XdgSurfaceUserData` which is attached to the Wayland resource
/// representing the xdg_surface.
///
/// The primary responsibility of this struct, when an instance is created,
/// is to be associated with the `xdg_surface` resource, usually via `Arc<Self>`
/// if we need to share its methods or its own `destroyed` flag.
///
/// However, given the current implementation where `XdgSurfaceUserData` holds most
/// state and `DesktopState` handles dispatch, this struct might primarily serve
/// as a type definition and a place for specific non-stateful helper methods
/// or constants related to an xdg_surface, if any emerge.
#[derive(Debug)]
pub struct XdgSurface {
    /// The underlying Wayland surface.
    wl_surface: wl_surface::WlSurface,

    // ANCHOR: XdgSurfaceDestroyedFlag
    /// Flag indicating if the xdg_surface object itself has been explicitly destroyed
    /// by a client request (`xdg_surface.destroy`). This is distinct from the
    /// wl_surface being destroyed or the role object (toplevel/popup) being destroyed.
    /// The `XdgSurfaceUserData.state` (with `XdgSurfaceState::Destroyed`) also tracks
    /// destruction from the perspective of the resource handler. This flag offers
    /// an object-level view if an `Arc<XdgSurface>` is stored as user_data.
    ///
    /// If `XdgSurfaceUserData` becomes the sole owner of all state including destruction,
    /// this field might be redundant. For now, keeping it to reflect the original intent.
    destroyed: Mutex<bool>,
    // ANCHOR_END: XdgSurfaceDestroyedFlag
}
// ANCHOR_END: XdgSurfaceStructSimplified

impl XdgSurface {
    // ANCHOR: XdgSurfaceNewSimplified
    /// Creates a new basic XdgSurface wrapper.
    ///
    /// Note: The comprehensive state (role, geometries, configure serials) is managed
    /// within `XdgSurfaceUserData` associated with the Wayland resource.
    pub fn new(wl_surface: wl_surface::WlSurface) -> Self {
        Self {
            wl_surface,
            destroyed: Mutex::new(false)),
        }
    }
    // ANCHOR_END: XdgSurfaceNewSimplified

    /// Returns the underlying `wl_surface::WlSurface`.
    pub fn wl_surface(&self) -> &wl_surface::WlSurface {
        &self.wl_surface
    }

    // ANCHOR: XdgSurfaceMarkDestroyedMethod
    /// Marks this XDG surface object as destroyed.
    /// This is typically called when handling the `xdg_surface.destroy` request.
    pub fn mark_destroyed(&self) {
        *self.destroyed.lock().unwrap() = true;
    }
    // ANCHOR_END: XdgSurfaceMarkDestroyedMethod

    // ANCHOR: XdgSurfaceIsAliveMethod
    /// Checks if the XDG surface is considered "alive".
    /// An XDG surface is alive if its underlying `wl_surface` is alive AND
    /// it has not been explicitly destroyed via `xdg_surface.destroy`.
    pub fn alive(&self) -> bool {
        let is_destroyed = *self.destroyed.lock().unwrap();
        self.wl_surface.alive() && !is_destroyed
    }
    // ANCHOR_END: XdgSurfaceIsAliveMethod
}
// ANCHOR_END: XdgSurfaceSimplifiedDefinition

// The Dispatch<xdg_surface::XdgSurface, ...> implementation has been removed.
// Dispatch for xdg_surface protocol objects is now handled by DesktopState using
// XdgSurfaceUserData, as defined in `novade-system/src/compositor/shell/xdg_shell/mod.rs`.

// ANCHOR: XdgSurfaceUnitTestsSimplified
#[cfg(test)]
mod tests {
    use super::*;
    use smithay::reexports::wayland_server::{
        Client, Display, Main,
        protocol::wl_surface,
        backend::ObjectData, // For mock
        Interface, // For mock
    };
    use std::sync::Arc; // For mock Arc<TestObjectData>

    // Minimal ObjectData mock for WlSurface
    #[derive(Default)]
    struct TestObjectData;
    impl ObjectData<wl_surface::WlSurface> for TestObjectData {
        fn request( self: Arc<Self>, _handle: &smithay::reexports::wayland_server::Handle, _client_data: &mut dyn smithay::reexports::wayland_server::ClientData, _client_id: smithay::reexports::wayland_server::backend::ClientId, _msg: smithay::reexports::wayland_server::Message<wl_surface::WlSurface>) -> Option<Arc<dyn ObjectData<wl_surface::WlSurface>>> {
            None
        }
        fn destroyed(self: Arc<Self>, _client_id: smithay::reexports::wayland_server::backend::ClientId, _object_id: smithay::reexports::wayland_server::backend::ObjectId) {}
    }


    // Helper to create a Display, Client, and a WlSurface for testing.
    // This is a simplified setup. For more integrated tests, see tests in `mod.rs`.
    fn test_setup_for_xdg_surface() -> (Display<()>, Client, Main<wl_surface::WlSurface>) {
        let mut display = Display::<()>::new().unwrap();
        let client = display.create_client(); // Needs ClientData, default for () is fine

        // Create a wl_surface resource.
        // For unit testing XdgSurface struct itself, we don't need full compositor state.
        // The closure for create_object should return Main<wl_surface::WlSurface>.
        let surface_main = client.create_object::<wl_surface::WlSurface>(
            &display.handle(), // Pass DisplayHandle
            wl_surface::WlSurface::interface().version,
            Arc::new(TestObjectData::default()) // Provide some ObjectData
        ).expect("Failed to create wl_surface for test");

        (display, client, surface_main)
    }

    #[test]
    fn xdg_surface_new_initializes_correctly_simplified() {
        let (_display, _client, wl_surface_resource_main) = test_setup_for_xdg_surface();
        // To get a WlSurface from Main<WlSurface>, you can use .as_ref() to borrow,
        // or .detach() if you need to own it (and it's not needed by Display anymore).
        let xdg_surface_instance = XdgSurface::new(wl_surface_resource_main.as_ref().clone());

        assert!(!*xdg_surface_instance.destroyed.lock().unwrap());
        assert!(xdg_surface_instance.alive()); // Depends on wl_surface.alive()
    }

    #[test]
    fn xdg_surface_mark_destroyed_updates_flag() {
        let (_display, _client, wl_surface_resource_main) = test_setup_for_xdg_surface();
        let xdg_surface_instance = XdgSurface::new(wl_surface_resource_main.as_ref().clone());

        assert!(!*xdg_surface_instance.destroyed.lock().unwrap());
        assert!(xdg_surface_instance.alive());

        xdg_surface_instance.mark_destroyed();

        assert!(*xdg_surface_instance.destroyed.lock().unwrap());
        // alive() also checks wl_surface.alive(). Assuming it's still alive for this test part.
        // If wl_surface itself was also dead, alive() would be false regardless of `destroyed` flag.
        assert!(!xdg_surface_instance.alive(), "alive() should be false after mark_destroyed()");
    }

    // Test wl_surface aliveness impact (conceptual - hard to make wl_surface not alive in unit test easily)
    // #[test]
    // fn xdg_surface_aliveness_depends_on_wl_surface() {
    //     // This would require more setup to simulate a WlSurface becoming !alive().
    //     // For instance, destroying the client or the WlSurface resource explicitly
    //     // and then running the display loop.
    // }
}
// ANCHOR_END: XdgSurfaceUnitTestsSimplified
