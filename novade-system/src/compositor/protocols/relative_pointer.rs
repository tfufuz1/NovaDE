// novade-system/src/compositor/protocols/relative_pointer.rs
// Implementation of the wp_relative_pointer_manager_v1 Wayland protocol

use smithay::{
    delegate_relative_pointer_manager,
    reexports::{
        wayland_protocols::wp::relative_pointer::v1::server::{
            wp_relative_pointer_manager_v1::{self, WpRelativePointerManagerV1, Request as ManagerRequest},
            wp_relative_pointer_v1::{self, WpRelativePointerV1, Request as RelativePointerRequest, Event as RelativePointerEvent},
        },
        wayland_server::{
            protocol::{wl_pointer, wl_seat}, // Relative pointer is associated with a wl_pointer
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    input::{Seat, SeatHandler, SeatState, pointer::{PointerHandle, PointerGrab, GrabStatus, MotionEvent, RelativeMotionEvent, PointerInnerHandle, ButtonEvent, AxisFrame}}, // Pointer interaction
    utils::{Serial, Logical, Point, Time}, // For timestamps and coordinates
    wayland::pointer_constraints::PointerConstraint // If used in conjunction with pointer locking
};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to interact with `SeatState` and `PointerHandle`.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For Relative Pointer, it would need to manage or access:
    // - SeatState and the specific Seat/PointerHandle.
    // - Logic to generate and send relative motion events.
    // - Integration with pointer locking if both are used.
}

#[derive(Debug, Error)]
pub enum RelativePointerError {
    #[error("Relative pointer already exists for this wl_pointer or wl_pointer is invalid")]
    PointerError,
    // No specific errors defined in the protocol itself for requests.
}

// UserData for WpRelativePointerV1 resource.
// This might store a reference to the wl_pointer it's associated with,
// or simply be unit if the association is clear from context or other state.
// Smithay's delegate handles this association.
#[derive(Debug, Default, Clone)]
pub struct RelativePointerData {
    // wl_pointer: wl_pointer::WlPointer // The wl_pointer this relative pointer is for.
    // Smithay's delegate_relative_pointer_manager handles associating the WpRelativePointerV1
    // with the WlPointer resource. The UserData for WpRelativePointerV1 is often unit.
}


// The main compositor state (e.g., NovaCompositorState) would need to manage seats
// and provide pointer input. Smithay's `delegate_relative_pointer_manager` handles
// the creation and association of WpRelativePointerV1 objects.
//
// No specific "RelativePointerHandler" trait is usually needed from Smithay.
// The core logic involves:
// 1. Advertising the WpRelativePointerManagerV1 global.
// 2. When a client binds and calls get_relative_pointer(id, wl_pointer_resource):
//    - The delegate creates the WpRelativePointerV1 resource (`id`).
//    - Associates it with `wl_pointer_resource`.
// 3. When actual relative pointer motion occurs (e.g., from mouse input, potentially while locked):
//    - The compositor's input processing logic detects this.
//    - It finds the `WpRelativePointerV1` object associated with the active `wl_pointer`.
//    - It sends `wp_relative_pointer_v1.relative_motion` events.
//
// Example conceptual structure:
// pub struct NovaCompositorState {
//     ...
//     pub seat_state: SeatState<Self>,
//     // Other input-related states
//     ...
// }
//
// // NovaCompositorState needs to implement Dispatch for the relevant objects.
// delegate_relative_pointer_manager!(NovaCompositorState);

/// Call this function when relative pointer motion occurs for a given `wl_pointer`.
///
/// - `compositor_state`: Your main compositor state.
/// - `pointer_handle`: The Smithay `PointerHandle` associated with the `wl_pointer` that
///                     received relative motion.
/// - `dx_unaccel`, `dy_unaccel`: Unaccelerated relative motion delta (in surface-local logical coordinates, but unitless for relative).
/// - `dx_accel`, `dy_accel`: Accelerated relative motion delta (if pointer acceleration is applied).
///                           The protocol sends unaccelerated values.
/// - `time`: Timestamp of the event (CLOCK_MONOTONIC, in milliseconds).
///
/// `D` is your main compositor state.
pub fn on_relative_pointer_motion<D>(
    compositor_state: &mut D, // Your main compositor state
    pointer_resource: &wl_pointer::WlPointer, // The wl_pointer that this motion is for
    dx_unaccel: f64,
    dy_unaccel: f64,
    time_msec: u32, // Protocol expects u32 milliseconds for timestamp hi/lo
) where
    D: 'static, // Minimal constraints for this helper. Real constraints depend on state access.
                // We need a way to find the WpRelativePointerV1 associated with pointer_resource.
                // Smithay's Seat / PointerHandle user_data or internal maps usually handle this.
{
    // Smithay's `PointerHandle` has `relative_motion` method which can be used if you have it.
    // However, that method is for *processing* OS-level relative events and then calling grabs.
    // Here, we assume we *have* the relative delta and need to send it via the protocol.

    // Find the WpRelativePointerV1 object associated with `pointer_resource`.
    // Smithay's `delegate_relative_pointer_manager` ensures that if a client created a
    // `WpRelativePointerV1` for a `wl_pointer`, it's stored and accessible.
    // One way is to iterate through known `WpRelativePointerV1` resources and check their parent `wl_pointer`.
    // A more efficient way: `wl_pointer`'s `UserDataMap` could store its associated `WpRelativePointerV1`.
    // Or, `SeatUserData` could map `wl_pointer` IDs to `WpRelativePointerV1`s.

    // Smithay's `PointerHandle::send_relative_motion` (if such a method exists directly for protocol)
    // or accessing the WpRelativePointerV1 resource via UserData on the wl_pointer is common.
    // The `delegate_relative_pointer_manager` should make the WpRelativePointerV1 resource available
    // when the wl_pointer is active and has such an object.

    // Let's assume we can get the WpRelativePointerV1 object from the wl_pointer's UserDataMap
    // where it might have been stored by the delegate or our Dispatch logic.
    let relative_pointer_obj_opt = pointer_resource.data::<WpRelativePointerV1>();

    if let Some(relative_pointer_obj) = relative_pointer_obj_opt {
        // The protocol expects time as u32 high and u32 low for a 64-bit microsecond timestamp.
        // We are given `time_msec` (u32 milliseconds).
        // Let's convert milliseconds to microseconds (u64).
        let time_usec: u64 = (time_msec as u64) * 1000;
        let tv_sec_hi = (time_usec >> 32) as u32;
        let tv_sec_lo = (time_usec & 0xFFFFFFFF) as u32;
        // This is incorrect. tv_sec should be seconds, tv_nsec nanoseconds.
        // The protocol relative_motion event sends:
        // uint tv_sec_hi, uint tv_sec_lo, uint tv_nsec (for timestamp)
        // This implies a timespec-like structure.
        // tv_sec_hi/lo form a u64 for seconds. tv_nsec is nanoseconds part.
        // If `time_msec` is CLOCK_MONOTONIC milliseconds:
        let total_nsecs = (time_msec as u64) * 1_000_000; // Convert ms to ns
        let tv_sec = (total_nsecs / 1_000_000_000) as u64;
        let tv_nsec = (total_nsecs % 1_000_000_000) as u32;

        let tv_sec_high = (tv_sec >> 32) as u32;
        let tv_sec_low = (tv_sec & 0xFFFFFFFF) as u32;

        debug!(
            "Sending relative_motion to {:?}: dx_unaccel={}, dy_unaccel={}, time_sec_hi={}, time_sec_lo={}, time_nsec={}",
            relative_pointer_obj, dx_unaccel, dy_unaccel, tv_sec_high, tv_sec_low, tv_nsec
        );

        relative_pointer_obj.relative_motion(
            tv_sec_high,
            tv_sec_low,
            tv_nsec,
            dx_unaccel,
            dy_unaccel,
            dx_unaccel, // dx_unaccel (same as dx for this protocol version)
            dy_unaccel  // dy_unaccel (same as dy for this protocol version)
        );
    } else {
        // No relative pointer object for this wl_pointer, or not found via this method.
        // This is normal if the client hasn't requested one.
        // trace!("No WpRelativePointerV1 found for wl_pointer {:?} to send relative motion.", pointer_resource);
    }
}


/// Initializes and registers the WpRelativePointerManagerV1 global.
/// `D` is your main compositor state type.
pub fn init_relative_pointer_manager<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<WpRelativePointerManagerV1, ()> +
       Dispatch<WpRelativePointerManagerV1, (), D> +
       Dispatch<WpRelativePointerV1, RelativePointerData, D> + // UserData for WpRelativePointerV1
       SeatHandler<D> + // SeatHandler for wl_pointer management
       'static,
       // D must also own SeatState.
{
    info!("Initializing WpRelativePointerManagerV1 global");

    // The WpRelativePointerManagerV1 global itself is simple.
    // Its main role is to allow clients to get a WpRelativePointerV1 for a given wl_pointer.
    display.create_global::<D, WpRelativePointerManagerV1, _>(
        1, // protocol version
        () // GlobalData for the manager (unit)
    )?;

    // Ensure `delegate_relative_pointer_manager!(D)` is called in your main compositor state setup.
    // This macro handles:
    // - Dispatching WpRelativePointerManagerV1 requests (specifically `get_relative_pointer`).
    //   When `get_relative_pointer(id, wl_pointer_resource)` is called:
    //     - It creates the `WpRelativePointerV1` resource (`id`).
    //     - It associates this `WpRelativePointerV1` resource with the provided `wl_pointer_resource`.
    //       (Often by storing the WpRelativePointerV1 in the UserData of the wl_pointer_resource,
    //        or having RelativePointerData store the wl_pointer).
    // - Dispatching WpRelativePointerV1 requests (destroy).

    // The actual sending of `relative_motion` events is done by the compositor's input loop
    // when it processes raw mouse motion that should be treated as relative.
    // This usually happens when a pointer is locked (see wp_locked_pointer_v1) or
    // when an application is in a specific mode (e.g., a game that captures mouse input).

    info!("WpRelativePointerManagerV1 global initialized.");
    Ok(())
}

// TODO:
// - Input Loop Integration:
//   - The compositor's main input processing logic (handling events from libinput or similar)
//     needs to identify when pointer motion should be treated as relative.
//   - This often occurs when a pointer lock is active (see `wp_locked_pointer_v1`).
//   - When relative motion is detected, `on_relative_pointer_motion` (or similar logic)
//     must be called with the correct deltas and timestamp for the currently focused/active `wl_pointer`.
// - Timestamp Accuracy:
//   - Ensure the timestamp provided to `relative_motion` events is accurate and uses CLOCK_MONOTONIC.
// - State Integration:
//   - `SeatState` (managing `PointerHandle`s) must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `SeatHandler`.
//   - `delegate_relative_pointer_manager!(NovaCompositorState);` macro must be used.
//   - A mechanism to associate `WpRelativePointerV1` objects with their `wl_pointer` resources
//     is needed for `on_relative_pointer_motion` to find the correct object. Smithay's delegate
//     and UserData on resources typically achieve this.
// - Pointer Acceleration:
//   - The protocol sends unaccelerated deltas. If the compositor applies pointer acceleration
//     for normal cursor movement, it should ensure that unaccelerated deltas are available
//     for this protocol. Libinput usually provides both.
// - Testing:
//   - Test with applications that use relative pointer motion (e.g., games, 3D modeling software).
//   - Verify that deltas are reported correctly and that timestamps are sensible.
//   - Test in conjunction with pointer locking.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod relative_pointer;
