//! Defines the global compositor state (`CompositorState`) and related structures.
//!
//! `CompositorState` holds all Wayland globals, Smithay's helper states (like `SeatState`),
//! and any other compositor-wide resources. It is the main state object passed around
//! during event processing and request dispatch.

use smithay::reexports::calloop::LoopHandle;
use wayland_server::{
    Display, GlobalList, DelegateDispatch, Dispatch, Client, GlobalData, DisplayHandle, Global,
    protocol::{wl_registry, wl_compositor, wl_shm, wl_seat, wl_pointer, wl_keyboard, wl_touch, wl_output},
    delegate_dispatch
};
use smithay::wayland::seat::SeatState;

/// Global state for the Nova Wayland Compositor.
///
/// This struct holds all the necessary state for the compositor to run, including:
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
    /// Creates a new `CompositorState`.
    ///
    /// Initializes the Wayland display, global list, and any helper states.
    ///
    /// # Arguments
    ///
    /// * `_event_loop`: A handle to the Calloop event loop, currently unused but
    ///   often needed for initializing event sources or other loop-dependent components.
    pub fn new(_event_loop: &LoopHandle<Self>) -> Self { // Renamed event_loop to _event_loop as it's not used yet
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

// Main dispatch for wl_registry.
// This handles requests to the wl_registry global itself, primarily for binding to other globals.
// Note: Smithay 0.3 often uses a dedicated struct for wl_registry logic (like `RegistryState` in `wl_registry.rs`).
// If that pattern is fully adopted, this DelegateDispatch on CompositorState might be simplified or removed
// if CompositorState no longer directly handles wl_registry requests.
impl DelegateDispatch<wl_registry::WlRegistry, GlobalData, CompositorState> for CompositorState {
    /// Handles a request to the `wl_registry`.
    ///
    /// This is typically a `bind` request from a client to a specific global.
    /// The actual binding logic (creating the resource for the bound global) is usually handled by
    /// the `GlobalDispatch` implementation for that specific global interface on `CompositorState`.
    /// This `request` method on `wl_registry` itself is often left empty if `GlobalDispatch` covers all needs.
    fn request(
        &mut self,
        _client: &Client,
        _resource: &wl_registry::WlRegistry,
        _request: wl_registry::Request,
        _request: wl_registry::Request, // The specific request (e.g., bind)
        _data: &GlobalData, // UserData associated with WlRegistry if any (GlobalData here)
        _dhandle: &DisplayHandle, // Handle to the display
    ) {
        // Typically, wl_registry::bind requests are handled by the GlobalDispatch implementations
        // for the specific interfaces being bound. So, this function can often be empty.
        // If there were direct requests to wl_registry other than bind that need handling,
        // they would go here.
        // For now, we assume GlobalDispatch handles binds, and wl_registry has no other requests.
        // Smithay might also handle some parts of wl_registry internally.
    }
}

// This `Dispatch` implementation for `WlRegistry` on `CompositorState` is likely part of what
// `GlobalList` uses internally or if `CompositorState` were to manage `wl_registry` resources directly.
// Given that `wl_registry.rs` defines `RegistryState` which also implements `Dispatch` for `WlRegistry`,
// care must be taken to ensure clarity on which dispatcher is primary.
// Typically, `GlobalList::new()` sets up the initial `wl_registry` global and its dispatch.
// If `CompositorState` is the delegate for `wl_registry` resources created by `GlobalList`, this impl is relevant.
impl Dispatch<wl_registry::WlRegistry, GlobalData, CompositorState> for CompositorState {
    /// Handles a request to a `wl_registry` resource previously created for a client.
    /// This is similar to `DelegateDispatch::request` but part of the general `Dispatch` trait.
    /// As with `DelegateDispatch::request` for `wl_registry`, this is often empty.
    fn request(
        &mut self,
        _client: &Client,
        _resource: &wl_registry::WlRegistry,
        _request: wl_registry::Request,
        _data: &GlobalData,
        _dhandle: &DisplayHandle,
        _data_init: &mut wayland_server::DataInit<'_, Self>,
    ) {
        // Empty for now, as wl_registry::bind is the primary interaction,
        // handled by GlobalDispatch of other interfaces.
    }

    /// Called when a `wl_registry` resource associated with a client is destroyed.
    fn destroyed(
        &mut self,
        _client_id: wayland_server::backend::ClientId, // ID of the client whose resource was destroyed
        _resource_id: wayland_server::backend::ObjectId, // ID of the wl_registry resource destroyed
        _data: &GlobalData, // UserData associated with the resource
    ) {
        // Can be used for cleanup if CompositorState held per-registry-resource state.
        // Generally, wl_registry resources are per-client and managed by GlobalList.
    }
}


// DelegateDispatch declarations for various Wayland interfaces.
// These macros declare that `CompositorState` is capable of being a delegate dispatcher
// for these interfaces with the specified UserData types.
// The actual request handling logic is often in:
// 1. `GlobalDispatch::bind` on `CompositorState` for global object creation (defined in protocol files).
// 2. `DelegateDispatch::request` on the resource-specific data struct (e.g., `SurfaceData`, `SeatData`)
//    for requests to an existing resource instance (defined in protocol files).
// These `delegate_dispatch!` calls in `state.rs` ensure that `CompositorState` can be used
// as the `D` type (the global state context) in `implement_dispatch!` macros and `DataInit`.

/// Delegate for `wl_seat` with `GlobalData`.
/// This is primarily for the global `wl_seat` object itself. Client-specific `wl_seat`
/// resources created from this global will use `SeatData` for their dispatch.
delegate_dispatch!(@<wl_seat::WlSeat: GlobalData> CompositorState);

/// Delegate for `wl_pointer` with Smithay's `PointerUserData`.
/// Actual request handling is in `PointerData` in `wl_seat.rs`.
delegate_dispatch!(@<wl_pointer::WlPointer: smithay::wayland::seat::PointerUserData> CompositorState);

/// Delegate for `wl_keyboard` with Smithay's `KeyboardUserData`.
/// Actual request handling is in `KeyboardData` in `wl_seat.rs`.
delegate_dispatch!(@<wl_keyboard::WlKeyboard: smithay::wayland::seat::KeyboardUserData> CompositorState);

/// Delegate for `wl_touch` with Smithay's `TouchUserData`.
/// Actual request handling is in `TouchData` in `wl_seat.rs`.
delegate_dispatch!(@<wl_touch::WlTouch: smithay::wayland::seat::TouchUserData> CompositorState);

/// Delegate for `wl_output` with `GlobalData`.
/// This is for the global `wl_output` object. Client-specific `wl_output`
/// resources will use `OutputData` for their dispatch.
delegate_dispatch!(@<wl_output::WlOutput: GlobalData> CompositorState);

// Note: `wl_compositor` and `wl_shm` globals are handled by `implement_dispatch!(CompositorState => ...)`
// in their respective protocol files (`wl_compositor.rs`, `wl_shm.rs`). This directly makes
// `CompositorState` the dispatcher for requests to those global resources, so explicit
// `delegate_dispatch!` here for them might not be needed unless they are also delegated by other types.

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::reexports::calloop::EventLoop;
    use wayland_server::protocol::{wl_compositor, wl_shm, wl_seat, wl_output};
    use wayland_server::GlobalData; // Assuming this is the type used for global data in GlobalDispatch

    #[test]
    fn test_compositor_state_new() {
        let mut event_loop: EventLoop<CompositorState> = EventLoop::try_new().unwrap();
        let compositor_state = CompositorState::new(&event_loop.handle());

        // Assert that CompositorState is created successfully
        // (not null is implicit by type, successful creation is the main check)
        // For more detailed checks, one might inspect internal fields if they have predictable initial states.
        // For now, successful creation is the primary assertion.

        assert!(compositor_state.compositor_global.is_none(), "compositor_global should be None initially");
        assert!(compositor_state.shm_global.is_none(), "shm_global should be None initially");
        assert!(compositor_state.seat_global.is_none(), "seat_global should be None initially");
        assert!(compositor_state.output_global.is_none(), "output_global should be None initially");
        
        // Check SeatState initialization (e.g., no seats initially, default name)
        // Note: SeatState's internal fields might not be public.
        // This is a conceptual check. If SeatState provides getters, use them.
        // For now, we rely on it being initialized by SeatState::new().
        // assert_eq!(compositor_state.seat_state.seats().len(), 0); // Example if seats() was available
    }

    #[test]
    fn test_global_registration() {
        let mut event_loop: EventLoop<CompositorState> = EventLoop::try_new().unwrap();
        let mut compositor_state = CompositorState::new(&event_loop.handle());
        let cs = &mut compositor_state;

        // wl_compositor (version from main.rs is 4)
        // Note: The GlobalDispatch for wl_compositor is on CompositorState,
        // and it uses CompositorStateData.
        // make_global_with_data needs: <S: GlobalDispatch<I, U, D> + 'static, I: Interface + 'static, U: UserData + 'static, D: 'static, GlobalSpecificData: Send + Sync + 'static>
        // Here S = CompositorState, I = wl_compositor::WlCompositor, U = GlobalData, D = CompositorState, GlobalSpecificData = CompositorStateData
        // However, `make_global` is usually sufficient if the GlobalDispatch is on CompositorState and doesn't need extra global-specific data.
        // The `CompositorStateData` is used for the *resource* data, not the global itself.
        // If GlobalDispatch::bind uses GlobalData, then it's simpler.
        // The `make_global` or `make_global_with_data` should align with how GlobalDispatch is defined for these interfaces on CompositorState.

        // Let's check the `GlobalDispatch` implementations in the respective protocol files.
        // For wl_compositor in wl_compositor.rs: `impl GlobalDispatch<wl_compositor::WlCompositor, GlobalData, CompositorState> for CompositorState`
        // The resource data `CompositorStateData` is initialized inside `bind`.
        // So, we don't pass `CompositorStateData` when creating the global.
        // We pass `GlobalData` as the UserData for the `bind` method.
        let compositor_global = cs.global_list.make_global::<wl_compositor::WlCompositor, GlobalData, CompositorState>(
            &cs.display.handle(),
            4, // version from main.rs
            GlobalData, 
        );
        cs.compositor_global = Some(compositor_global);

        // wl_shm (version from main.rs is 1)
        // Similar to wl_compositor, GlobalDispatch is on CompositorState, resource data ShmState initialized in bind.
        let shm_global = cs.global_list.make_global::<wl_shm::WlShm, GlobalData, CompositorState>(
            &cs.display.handle(),
            1, // version
            GlobalData,
        );
        cs.shm_global = Some(shm_global);

        // wl_seat (version from main.rs is 7)
        // GlobalDispatch is on CompositorState, uses SeatStateGlobalData for the global data.
        // UserData for bind is GlobalData.
        let seat_global = cs.global_list.make_global_with_data::<wl_seat::WlSeat, GlobalData, CompositorState, crate::protocols::wl_seat::SeatStateGlobalData>(
            &cs.display.handle(),
            7, // version
            GlobalData,
            crate::protocols::wl_seat::SeatStateGlobalData::default()
        );
        cs.seat_global = Some(seat_global);
       
        // wl_output (version from main.rs is 3)
        // GlobalDispatch is on CompositorState, uses OutputGlobalData for the global data.
        // UserData for bind is GlobalData.
        let output_global = cs.global_list.make_global_with_data::<wl_output::WlOutput, GlobalData, CompositorState, crate::protocols::wl_output::OutputGlobalData>(
            &cs.display.handle(),
            3, // version
            GlobalData,
            crate::protocols::wl_output::OutputGlobalData::default()
        );
        cs.output_global = Some(output_global);

        assert!(cs.compositor_global.is_some(), "compositor_global should be Some after registration");
        assert!(cs.shm_global.is_some(), "shm_global should be Some after registration");
        assert!(cs.seat_global.is_some(), "seat_global should be Some after registration");
        assert!(cs.output_global.is_some(), "output_global should be Some after registration");

        // To further test, one might check cs.global_list.iter() or similar if available and useful.
        // For now, checking that the Option fields are populated is the primary goal.
    }
}
