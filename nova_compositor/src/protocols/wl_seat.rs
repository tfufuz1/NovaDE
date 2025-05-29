//! Implements the `wl_seat` Wayland global and related input device interfaces.
//!
//! `wl_seat` represents a group of input devices (pointer, keyboard, touch) available to a client.
//! This module leverages Smithay's seat management utilities.

use wayland_server::{
    protocol::{wl_seat, wl_pointer, wl_keyboard, wl_touch},
    Dispatch, DelegateDispatch, GlobalDispatch, Main, Resource, Client, DisplayHandle, GlobalData,
    NewProxy, UserData, implement_dispatch, DataInit // NewProxy for id types, UserData for type markers
};
use crate::state::CompositorState;
use smithay::wayland::seat::{
    Seat, SeatState, SeatData, // Core seat management types from Smithay
    PointerHandle, KeyboardHandle, TouchHandle, // Handles for input events (not used yet)
    SeatHandler, // Trait to integrate seat logic with CompositorState
    PointerUserData, KeyboardUserData, TouchUserData // UserData markers for input device resources
};
// use smithay::utils::SERIAL_COUNTER; // For generating serials, if needed directly. Smithay handles many serials.

/// Data associated with the `wl_seat` global itself in the `GlobalList`.
///
/// This can be used to store properties of the seat global that are not client-specific.
/// Currently, it's empty as `wl_seat` global properties (like name, capabilities) are
/// sent during the client's bind process.
#[derive(Debug, Default)]
pub struct SeatStateGlobalData;

/// Resource data for `wl_pointer` instances.
///
/// This struct holds any state specific to a client's `wl_pointer` resource.
/// For now, it's a placeholder, but could store cursor surface, focus, etc.
#[derive(Debug, Default)]
pub struct PointerData;

// DelegateDispatch for wl_pointer requests, handled by PointerData.
impl DelegateDispatch<wl_pointer::WlPointer, PointerUserData, CompositorState> for PointerData {
    /// Handles requests from a client to a `wl_pointer` resource.
    fn request(
        &mut self, // State for this specific wl_pointer resource (&mut PointerData).
        _client: &Client,
        resource: &wl_pointer::WlPointer, // The wl_pointer resource this request is for.
        request: wl_pointer::Request,
        _data: &PointerUserData, // UserData marker for wl_pointer dispatch.
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, CompositorState>, // Not typically used for wl_pointer requests.
    ) {
        match request {
            wl_pointer::Request::SetCursor { serial, surface, hotspot_x, hotspot_y } => {
                // Client requests to set the cursor appearance.
                // TODO: Implement actual cursor setting logic (e.g., store surface, update rendering).
                // `serial` is for associating this request with a specific input event sequence.
                println!(
                    "wl_pointer {:?}: SetCursor called. Serial: {}, hotspot_x: {}, hotspot_y: {}",
                    resource.id(), serial, hotspot_x, hotspot_y
                );
                if let Some(s) = surface {
                    println!("wl_pointer {:?}: Cursor surface is {:?}.", resource.id(), s.id());
                } else {
                    println!("wl_pointer {:?}: Cursor hidden (surface is None).", resource.id());
                }
            }
            wl_pointer::Request::Release => {
                // Client requests to release/destroy the wl_pointer resource.
                // Smithay handles resource destruction. Our `destroyed` method below will be called.
                println!("wl_pointer {:?}: Release called. (Handled by Smithay resource destruction)", resource.id());
            }
            _ => {
                 println!("wl_pointer {:?}: Unknown request: {:?}", resource.id(), request);
            }
        }
    }

    /// Called when the `wl_pointer` resource is destroyed.
    fn destroyed(
        &mut self, // State for this specific wl_pointer resource (&mut PointerData).
        _client_id: wayland_server::backend::ClientId,
        object_id: wayland_server::backend::ObjectId, // ID of the wl_pointer resource.
        _data: &PointerUserData, // UserData marker.
    ) {
        println!("wl_pointer {:?}: Resource destroyed.", object_id);
        // Cleanup for PointerData if any was needed.
    }
}
// Connects WlPointer requests to PointerData's Dispatch/DelegateDispatch implementations.
implement_dispatch!(PointerData => [wl_pointer::WlPointer: PointerUserData], CompositorState);

/// Resource data for `wl_keyboard` instances.
///
/// Holds state specific to a client's `wl_keyboard` resource. Placeholder for now.
#[derive(Debug, Default)]
pub struct KeyboardData;

// DelegateDispatch for wl_keyboard requests, handled by KeyboardData.
impl DelegateDispatch<wl_keyboard::WlKeyboard, KeyboardUserData, CompositorState> for KeyboardData {
    /// Handles requests from a client to a `wl_keyboard` resource.
    fn request(
        &mut self, // State for this specific wl_keyboard resource (&mut KeyboardData).
        _client: &Client,
        resource: &wl_keyboard::WlKeyboard, // The wl_keyboard resource.
        request: wl_keyboard::Request,
        _data: &KeyboardUserData, // UserData marker for wl_keyboard dispatch.
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, CompositorState>, // Not typically used for wl_keyboard requests.
    ) {
        match request {
            wl_keyboard::Request::Release => {
                // Client requests to release/destroy the wl_keyboard resource.
                // Smithay handles destruction. Our `destroyed` method below is called.
                println!("wl_keyboard {:?}: Release called. (Handled by Smithay resource destruction)", resource.id());
            }
            _ => {
                println!("wl_keyboard {:?}: Unknown request: {:?}", resource.id(), request);
            }
        }
    }

    /// Called when the `wl_keyboard` resource is destroyed.
    fn destroyed(
        &mut self, // State for this specific wl_keyboard resource (&mut KeyboardData).
        _client_id: wayland_server::backend::ClientId,
        object_id: wayland_server::backend::ObjectId, // ID of the wl_keyboard resource.
        _data: &KeyboardUserData, // UserData marker.
    ) {
        println!("wl_keyboard {:?}: Resource destroyed.", object_id);
        // Cleanup for KeyboardData if any was needed.
    }
}
// Connects WlKeyboard requests to KeyboardData's Dispatch/DelegateDispatch implementations.
implement_dispatch!(KeyboardData => [wl_keyboard::WlKeyboard: KeyboardUserData], CompositorState);

/// Resource data for `wl_touch` instances.
///
/// Holds state specific to a client's `wl_touch` resource. Placeholder for now.
#[derive(Debug, Default)]
pub struct TouchData;
// Connects WlTouch requests to TouchData's Dispatch/DelegateDispatch implementations.
// TODO: Implement DelegateDispatch for TouchData if wl_touch requests (like release) need handling.
implement_dispatch!(TouchData => [wl_touch::WlTouch: TouchUserData], CompositorState);


// GlobalDispatch for WlSeat on CompositorState.
// Handles new client bindings to the wl_seat global.
impl GlobalDispatch<wl_seat::WlSeat, GlobalData, CompositorState> for CompositorState {
    /// Called when a client binds to the `wl_seat` global.
    ///
    /// Initializes the `wl_seat` resource for the client using Smithay's `SeatData`.
    /// Advertises seat capabilities (pointer, keyboard) and name.
    fn bind(
        _state: &mut CompositorState, // Global compositor state, not directly used here.
        _handle: &DisplayHandle,
        _client: &Client,
        resource: Main<wl_seat::WlSeat>, // The wl_seat resource to initialize for the client.
        _global_data: &GlobalData, // UserData for this global (GlobalData from make_global_with_data).
        data_init: &mut DataInit<'_, CompositorState>, // Utility to initialize resource data.
    ) {
        println!("Client bound to wl_seat global. Initializing wl_seat resource {:?}.", resource.id());

        // Use Smithay's SeatData as the resource data for this client's wl_seat instance.
        // SeatData manages the associated pointer, keyboard, and touch objects for this seat.
        let client_seat_data = SeatData::new();
        let seat_resource = data_init.init_resource(resource, client_seat_data);

        // Advertise capabilities.
        // TODO: These should be dynamically determined by the actual input devices available.
        let capabilities = wl_seat::Capability::Pointer | wl_seat::Capability::Keyboard;
        // TODO: Add wl_seat::Capability::Touch if touch input is supported.
        seat_resource.capabilities(capabilities);
        
        // Advertise seat name.
        // TODO: Make this configurable or derive from a more robust source.
        seat_resource.name("nova_seat_0".to_string()); // Example name

        println!(
            "wl_seat resource {:?} initialized for client. Capabilities: {:?}, Name: '{}'",
            seat_resource.id(), capabilities, "nova_seat_0"
        );
    }

    /// Checks if the requested version of `wl_seat` is supported.
    fn check_versions(&self, _main: Main<wl_seat::WlSeat>, _versions: &[u32]) -> bool {
        true // Allow all versions for now (up to the one advertised by GlobalList).
    }
}

// DelegateDispatch for requests made on a specific client's wl_seat resource.
// This is dispatched to SeatData<CompositorState> as per implement_dispatch below.
impl DelegateDispatch<wl_seat::WlSeat, (), CompositorState> for SeatData<CompositorState> {
    /// Handles requests from a client to their `wl_seat` resource.
    fn request(
        &mut self, // State for this specific wl_seat resource (&mut SeatData<CompositorState>).
        client: &Client,
        resource: &wl_seat::WlSeat, // The wl_seat resource this request is for.
        request: wl_seat::Request,
        _data: &(), // UserData for wl_seat dispatch (here, unit type `()`).
        dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, CompositorState>, // For creating new resources (pointer, keyboard, touch).
    ) {
        match request {
            wl_seat::Request::GetPointer { id } => {
                // Client requests a wl_pointer object.
                // `self` is &mut SeatData<CompositorState>.
                // `get_pointer` is a Smithay utility on SeatData that creates and manages
                // the wl_pointer resource, associating it with PointerData.
                println!("wl_seat {:?}: GetPointer called (new id: {:?})", resource.id(), id.id());
                let _pointer_resource = self.get_pointer(id, data_init, client, dhandle, PointerData::default());
                println!("wl_pointer {:?} created for wl_seat {:?}.", _pointer_resource.id(), resource.id());
            }
            wl_seat::Request::GetKeyboard { id } => {
                // Client requests a wl_keyboard object.
                println!("wl_seat {:?}: GetKeyboard called (new id: {:?})", resource.id(), id.id());
                let _keyboard_resource = self.get_keyboard(id, data_init, client, dhandle, KeyboardData::default());
                println!("wl_keyboard {:?} created for wl_seat {:?}.", _keyboard_resource.id(), resource.id());
            }
            wl_seat::Request::GetTouch { id } => {
                // Client requests a wl_touch object.
                println!("wl_seat {:?}: GetTouch called (new id: {:?})", resource.id(), id.id());
                // TODO: Ensure TouchHandle is available/configured in SeatState if needed by get_touch.
                let _touch_resource = self.get_touch(id, data_init, client, dhandle, TouchData::default());
                println!("wl_touch {:?} created for wl_seat {:?}.", _touch_resource.id(), resource.id());
            }
            wl_seat::Request::Release => {
                // Client requests to release/destroy the wl_seat resource.
                // Smithay handles resource destruction. Our `destroyed` method below is called.
                println!("wl_seat {:?}: Release called. (Handled by Smithay resource destruction)", resource.id());
            }
            _ => {
                println!("wl_seat {:?}: Unknown request: {:?}", resource.id(), request);
            }
        }
    }

    /// Called when the client's `wl_seat` resource is destroyed.
    fn destroyed(
        &mut self, // State for this specific wl_seat resource (&mut SeatData<CompositorState>).
        _client_id: wayland_server::backend::ClientId,
        object_id: wayland_server::backend::ObjectId, // ID of the wl_seat resource.
        _data: &(), // UserData for wl_seat dispatch.
    ) {
        println!("wl_seat {:?}: Resource destroyed.", object_id);
        // Smithay's SeatData should handle cleanup of associated pointer/keyboard/touch
        // resources when its wl_seat resource is destroyed.
    }
}

// Connects WlSeat requests to SeatData<CompositorState>'s Dispatch/DelegateDispatch implementations.
// - `SeatData<CompositorState>` is the struct handling the dispatch.
// - `wl_seat::WlSeat` is the interface.
// - `()` is the UserData associated with the resource for dispatch purposes.
// - `CompositorState` is the global application data.
implement_dispatch!(SeatData<CompositorState> => [wl_seat::WlSeat: ()], CompositorState);

// SeatHandler implementation for CompositorState.
// This trait from Smithay integrates our CompositorState with Smithay's seat logic,
// allowing SeatState to manage seat-related functionalities like focus tracking.
impl SeatHandler for CompositorState {
    /// Defines the type used for pointer focus. This would typically be a surface type.
    /// Placeholder `()` indicates no detailed focus tracking data yet.
    type PointerFocus = (); // TODO: Replace with actual surface type (e.g., Arc<WlSurfaceState>) for focus.
    /// Defines the type used for keyboard focus.
    type KeyboardFocus = (); // TODO: Replace with actual surface type for focus.
    // TODO: Define KeyboardGrab and PointerGrab types if input grabs are implemented.
    // type KeyboardGrab = smithay::wayland::seat::keyboard::GrabStartData<Self>; 
    // type PointerGrab = smithay::wayland::seat::pointer::GrabStartData<Self>;

    /// Provides access to the `SeatState` stored within `CompositorState`.
    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state // `seat_state` must be a public field in CompositorState.
    }

    // Callbacks for focus changes. These are invoked by Smithay's seat logic.
    // TODO: Implement actual focus handling logic (e.g., sending enter/leave events).
    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&Self::PointerFocus>) {
        // This callback is triggered when pointer focus changes.
        // `_seat` is the Smithay Seat object.
        // `_focused` is the new focus target (e.g., a surface).
        // Here, one would typically send wl_pointer.leave to the old focus
        // and wl_pointer.enter to the new focus.
    }
    fn keyboard_focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&Self::KeyboardFocus>) {
        // This callback is triggered when keyboard focus changes.
        // Similar to pointer focus, send wl_keyboard.leave/enter events.
    }
    
    // Callbacks for input grab events. These are invoked if grabs are implemented.
    // TODO: Implement if input grabs (e.g., for popups, window moving/resizing) are needed.
    // fn pointer_grab_start(&mut self, _seat: &Seat<Self>, _start_data: &Self::PointerGrab) { }
    // fn pointer_grab_end(&mut self, _seat: &Seat<Self>) { }
    // fn keyboard_grab_start(&mut self, _seat: &Seat<Self>, _start_data: &Self::KeyboardGrab) { }
    // fn keyboard_grab_end(&mut self, _seat: &Seat<Self>) { }
}

// Note on UserData types:
// - `PointerUserData`, `KeyboardUserData`, `TouchUserData` are marker types from Smithay
//   used in `implement_dispatch!` to associate the correct Smithay internal UserData
//   with `wl_pointer`, `wl_keyboard`, and `wl_touch` resources managed by `SeatData`.
// - `PointerData`, `KeyboardData`, `TouchData` are our custom structs that hold the
//   actual resource-specific state for these input device interfaces.
```
