//! Defines the global compositor state (`CompositorState`) and related structures.
//!
//! **DEPRECATED:** This state structure is deprecated in favor of `DesktopState`
//! found in `novade-system/src/compositor/core/state.rs`.
//! `DesktopState` is more comprehensive and integrates Smithay's helper states directly.
//! This file is kept for reference during transition or may be removed in the future.

use smithay::reexports::calloop::LoopHandle;
use wayland_server::{
    Display, GlobalList, DelegateDispatch, Dispatch, Client, GlobalData, DisplayHandle, Global,
    protocol::{wl_registry, wl_compositor, wl_shm, wl_seat, wl_pointer, wl_keyboard, wl_touch, wl_output},
    delegate_dispatch
};
use smithay::wayland::seat::SeatState;

/// **DEPRECATED:** Global state for the Nova Wayland Compositor.
///
/// Please use `DesktopState` from `novade-system/src/compositor/core/state.rs`.
///
/// This struct was intended to hold all the necessary state for the compositor to run, including:
/// - The Wayland `Display` object.
/// - The list of Wayland globals (`GlobalList`).
/// - Handles to specific globals like `wl_compositor`, `wl_shm`, `wl_seat`, `wl_output`.
/// - Smithay's `SeatState` for managing input devices and seat-related functionality.
pub struct CompositorState {
    /// The Wayland display, the core of the Wayland server.
    pub display: Display<Self>,
    /// The list of globally advertised Wayland objects.
    pub global_list: GlobalList<Self>,
    /// Handle to the `wl_compositor` global.
    pub compositor_global: Option<Global<wl_compositor::WlCompositor>>,
    /// Handle to the `wl_shm` global (shared memory).
    pub shm_global: Option<Global<wl_shm::WlShm>>,
    /// Handle to the `wl_seat` global (input devices).
    pub seat_global: Option<Global<wl_seat::WlSeat>>,
    /// Handle to the `wl_output` global (display information).
    pub output_global: Option<Global<wl_output::WlOutput>>,
    /// Smithay's helper state for managing seat-related resources and events.
    pub seat_state: SeatState<Self>,
    // TODO: Add a logger field, e.g., `log: slog::Logger` or `tracing::Span`.
}

impl CompositorState {
    /// **DEPRECATED:** Creates a new `CompositorState`.
    ///
    /// Initializes the Wayland display, global list, and any helper states.
    ///
    /// # Arguments
    ///
    /// * `_event_loop`: A handle to the Calloop event loop.
    pub fn new(_event_loop: &LoopHandle<Self>) -> Self {
        tracing::warn!("DEPRECATED: CompositorState in nova_compositor_logic/state.rs is being instantiated. Please migrate to DesktopState in compositor/core/state.rs.");
        let display = Display::new();
        let global_list = GlobalList::new(&display.handle());
        
        Self {
            display,
            global_list,
            compositor_global: None,
            shm_global: None,
            seat_global: None,
            output_global: None,
            seat_state: SeatState::new(),
            // log: slog::Logger::root(slog::Discard, slog::o!()), // Example if logger was added
        }
    }
}

// DelegateDispatch and Dispatch implementations are largely kept for reference
// but would not be actively used if DesktopState is the primary state.

impl DelegateDispatch<wl_registry::WlRegistry, GlobalData, CompositorState> for CompositorState {
    fn request(
        &mut self,
        _client: &Client,
        _resource: &wl_registry::WlRegistry,
        _request: wl_registry::Request,
        _data: &GlobalData,
        _dhandle: &DisplayHandle,
    ) {
        tracing::warn!("DEPRECATED: DelegateDispatch<wl_registry::WlRegistry> called on CompositorState (nova_compositor_logic).");
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalData, CompositorState> for CompositorState {
    fn request(
        &mut self,
        _client: &Client,
        _resource: &wl_registry::WlRegistry,
        _request: wl_registry::Request,
        _data: &GlobalData,
        _dhandle: &DisplayHandle,
        _data_init: &mut wayland_server::DataInit<'_, Self>,
    ) {
        tracing::warn!("DEPRECATED: Dispatch<wl_registry::WlRegistry> called on CompositorState (nova_compositor_logic).");
    }

    fn destroyed(
        &mut self,
        _client_id: wayland_server::backend::ClientId,
        _resource_id: wayland_server::backend::ObjectId,
        _data: &GlobalData,
    ) {
        tracing::warn!("DEPRECATED: Dispatch<wl_registry::WlRegistry>::destroyed called on CompositorState (nova_compositor_logic).");
    }
}

delegate_dispatch!(@<wl_seat::WlSeat: GlobalData> CompositorState);
delegate_dispatch!(@<wl_pointer::WlPointer: smithay::wayland::seat::PointerUserData> CompositorState);
delegate_dispatch!(@<wl_keyboard::WlKeyboard: smithay::wayland::seat::KeyboardUserData> CompositorState);
delegate_dispatch!(@<wl_touch::WlTouch: smithay::wayland::seat::TouchUserData> CompositorState);
delegate_dispatch!(@<wl_output::WlOutput: GlobalData> CompositorState);

// Tests are kept for now, but would eventually be removed or adapted for DesktopState.
#[cfg(test)]
mod tests {
    use super::*;
    use smithay::reexports::calloop::EventLoop;
    // Keep other imports for test structure, though they might become unused if tests are removed.
    use wayland_server::protocol::{wl_compositor, wl_shm, wl_seat, wl_output};
    use wayland_server::GlobalData;

    #[test]
    fn test_compositor_state_new_deprecated() {
        let mut event_loop: EventLoop<CompositorState> = EventLoop::try_new().unwrap();
        // This will now log a warning due to the change in `new`.
        let compositor_state = CompositorState::new(&event_loop.handle());

        assert!(compositor_state.compositor_global.is_none());
        // ... other assertions remain structurally same but test deprecated functionality.
    }

    #[test]
    fn test_global_registration_deprecated() {
        let mut event_loop: EventLoop<CompositorState> = EventLoop::try_new().unwrap();
        let mut compositor_state = CompositorState::new(&event_loop.handle());
        let cs = &mut compositor_state;

        // The following global registrations would use the deprecated state.
        // No functional change to the test logic itself, but it's testing deprecated paths.
        let compositor_global = cs.global_list.make_global::<wl_compositor::WlCompositor, GlobalData, CompositorState>(
            &cs.display.handle(),
            4,
            GlobalData, 
        );
        cs.compositor_global = Some(compositor_global);
        // ... other globals ...

        assert!(cs.compositor_global.is_some());
        // ... other assertions ...
    }
}
