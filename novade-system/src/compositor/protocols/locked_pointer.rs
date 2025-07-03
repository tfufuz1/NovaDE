// novade-system/src/compositor/protocols/locked_pointer.rs
// Implementation of the wp_pointer_constraints_unstable_v1 protocol,
// specifically focusing on wp_locked_pointer_v1.

use smithay::{
    delegate_pointer_constraints, // Smithay's delegate macro for this protocol
    reexports::{
        wayland_protocols::wp::pointer_constraints::zv1::server::{
            zwp_pointer_constraints_v1::{self, ZwpPointerConstraintsV1, Request as ManagerRequest, Lifetime},
            zwp_locked_pointer_v1::{self, ZwpLockedPointerV1, Request as LockedPointerRequest, Event as LockedPointerEvent},
            zwp_confined_pointer_v1::{self, ZwpConfinedPointerV1}, // Also part of pointer_constraints
        },
        wayland_server::{
            protocol::{wl_pointer, wl_surface, wl_region, wl_seat}, // Interaction with pointer, surface, region
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    input::{
        Seat, SeatHandler, SeatState, pointer::{PointerHandle, PointerGrab, GrabStatus, MotionEvent, ButtonEvent, AxisFrame, PointerInnerHandle}, // Pointer interaction
        keyboard::KeyboardHandle, // For focus
    },
    utils::{Serial, Logical, Point, Rectangle, Region}, // For geometry and regions
    wayland::pointer_constraints::{
        PointerConstraintsHandler, PointerConstraintsState, PointerConstraint, // Smithay's types
        LockedPointerData, ConfinedPointerData, // UserData for the respective objects
    },
    desktop::Window, // To check if a surface is a window that can be locked to
};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to hold `PointerConstraintsState` and interact with `SeatState`.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For Pointer Constraints, it would need to manage or access:
    // - PointerConstraintsState
    // - SeatState and the specific Seat/PointerHandle.
    // - Information about window focus and geometry.
    // - Logic to actually constrain the pointer movement or hide it.
}

#[derive(Debug, Error)]
pub enum PointerLockError {
    #[error("Pointer constraint already exists for this wl_pointer/surface or configuration is invalid")]
    ConstraintError,
    #[error("Surface is not eligible for pointer lock/confinement (e.g., not focused, not a window)")]
    SurfaceNotEligible,
    #[error("Region is invalid or not applicable")]
    InvalidRegion,
}

// The main compositor state (e.g., NovaCompositorState) would implement PointerConstraintsHandler
// and store PointerConstraintsState.
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub pointer_constraints_state: PointerConstraintsState,
//     pub seat_state: SeatState<Self>,
//     // Access to Space or window list to verify surfaces
//     ...
// }
//
// impl PointerConstraintsHandler for NovaCompositorState {
//     fn pointer_constraints_state(&mut self) -> &mut PointerConstraintsState {
//         &mut self.pointer_constraints_state
//     }
//
//     fn new_locked_pointer(&mut self, locked_pointer_obj: ZwpLockedPointerV1, pointer: wl_pointer::WlPointer, surface: wl_surface::WlSurface, lifetime: Lifetime) -> Result<(), ()> {
//         info!("New locked pointer {:?} requested for pointer {:?}, surface {:?}", locked_pointer_obj, pointer, surface);
//         // Client requests to lock the pointer to the surface.
//         // 1. Validate: Is surface eligible? Is pointer active? Is surface focused by this pointer?
//         // 2. If valid, activate the lock:
//         //    - Hide the cursor (if not already via hint).
//         //    - All further pointer events are relative (use wp_relative_pointer_v1).
//         //    - Pointer position might be "warped" to center of surface or a defined hint.
//         //    - Store `locked_pointer_obj` with the pointer/surface state.
//         //    - Send `locked` event on `locked_pointer_obj`.
//         // 3. If lifetime is Once, unlock when focus changes or button pressed. If Persistent, stays until explicitly unlocked or object destroyed.
//         // Smithay's PointerConstraintsState and PointerHandle::set_constraint can manage this.
//         let seat = find_seat_for_pointer(&self.seat_state, &pointer)?; // Find the seat
//         let pointer_handle = seat.get_pointer().ok_or(())?;
//         pointer_handle.set_constraint(Some(PointerConstraint::Locked { surface, lifetime, resource: locked_pointer_obj }), Serial::now()); // Simplified
//         Ok(())
//     }
//
//     fn new_confined_pointer(&mut self, confined_pointer_obj: ZwpConfinedPointerV1, ...) -> Result<(), ()> {
//         // Similar logic for confined pointer
//         Ok(())
//     }
//
//     fn constraint_destroyed(&mut self, pointer: wl_pointer::WlPointer, constraint_type: &str) {
//         // A lock or confinement was destroyed. Release it.
//         // pointer_handle.set_constraint(None, Serial::now());
//     }
// }
// delegate_pointer_constraints!(NovaCompositorState);

impl PointerConstraintsHandler for DesktopState { // Replace DesktopState with NovaCompositorState
    fn pointer_constraints_state(&mut self) -> &mut PointerConstraintsState {
        // TODO: Properly integrate PointerConstraintsState with DesktopState or NovaCompositorState.
        panic!("PointerConstraintsHandler::pointer_constraints_state() needs proper integration.");
        // Example: &mut self.nova_compositor_state.pointer_constraints_state
    }

    fn new_locked_pointer(
        &mut self,
        locked_pointer_resource: ZwpLockedPointerV1, // The ZwpLockedPointerV1 resource created by client
        wl_pointer_resource: &wl_pointer::WlPointer,   // The wl_pointer this lock is for
        surface: &wl_surface::WlSurface,             // The surface to lock the pointer to
        _region: Option<&wl_region::WlRegion>,       // Hint region (usually None for locked pointer)
        lifetime: Lifetime,                        // Oneshot or Persistent
    ) -> Result<(), ()> {
        info!(
            "Client requests new locked pointer {:?} for wl_pointer {:?}, surface {:?}, lifetime {:?}",
            locked_pointer_resource, wl_pointer_resource, surface, lifetime
        );

        // TODO: Access the actual Seat and PointerHandle for `wl_pointer_resource`.
        // This requires querying SeatState based on `wl_pointer_resource`.
        // let seat = self.find_seat_for_wl_pointer(wl_pointer_resource).ok_or(())?;
        // let pointer_handle = seat.get_pointer().ok_or(())?;

        // --- Placeholder for Seat/PointerHandle access ---
        let pointer_handle_opt: Option<PointerHandle<Self>> = None; // Replace with actual lookup
        warn!("PointerConstraintsHandler::new_locked_pointer: Seat/PointerHandle lookup is a placeholder.");
        // --- End Placeholder ---

        if let Some(pointer_handle) = pointer_handle_opt {
            // Validate if the surface is eligible for pointer lock.
            // Typically, the surface must be the current focus of the pointer.
            let current_focus = pointer_handle.current_focus();
            if current_focus.as_ref().map(|f| f.wl_surface().as_ref()) != Some(Some(surface)) {
                error!(
                    "Cannot lock pointer to surface {:?}: it does not have pointer focus. Current focus: {:?}",
                    surface, current_focus.map(|f| f.wl_surface())
                );
                // The protocol doesn't specify an error for this on manager.
                // ZwpLockedPointerV1 itself doesn't have errors for creation failure.
                // This implies the request might fail silently, or we destroy `locked_pointer_resource`.
                // Returning Err(()) tells Smithay we couldn't satisfy the request.
                return Err(());
            }

            // TODO: Check if surface is a valid window type (e.g., an XDG toplevel).
            // This prevents locking to arbitrary subsurfaces or other non-window surfaces.

            info!(
                "Applying pointer lock to surface {:?} for pointer {:?}",
                surface, wl_pointer_resource
            );

            // Smithay's `PointerHandle::set_constraint` is the way to activate the lock.
            // It takes a `PointerConstraint` enum.
            let constraint = PointerConstraint::Locked {
                surface: surface.clone(),
                lifetime,
                resource: locked_pointer_resource.clone(), // The ZwpLockedPointerV1 resource
            };

            // The serial for `set_constraint` should ideally come from the client request
            // that triggered this, if available, or `Serial::now()`.
            // The `lock_pointer` request on `ZwpPointerConstraintsV1` doesn't include a serial.
            // We use `Serial::now()` or a compositor-internal serial.
            let serial = Serial::now(); // Or a more contextually appropriate serial

            pointer_handle.set_constraint(Some(constraint), serial);

            // When `set_constraint` is called with `PointerConstraint::Locked`:
            // - Smithay's internal pointer grab logic changes.
            // - The cursor is typically hidden (PointerHandle::set_cursor with hidden icon).
            // - Pointer motion events are delivered as relative motion via `wp_relative_pointer_v1`
            //   if a `WpRelativePointerV1` object exists for this `wl_pointer`.
            //   The compositor's input loop needs to generate these relative events.
            // - The `locked` event is sent on `locked_pointer_resource` by Smithay.
            //   (This happens when the constraint becomes active, usually immediately if conditions met).

            // No, `PointerHandle::set_constraint` does not automatically send the `locked` event.
            // We need to send it after successfully applying the constraint.
            // Smithay's `PointerConstraintsState` and the delegate macro might handle this.
            //
            // Re-check Smithay: `PointerConstraintsState::new_locked_pointer` (which is called by delegate)
            // internally calls `PointerHandle::set_constraint` and then sends `locked` event.
            // So, this handler `new_locked_pointer` is called by Smithay *after* it has
            // already decided the lock can be attempted and has stored `LockedPointerData`.
            // Our role here is to perform any additional validation or compositor-specific setup.
            // If we return `Ok(())`, Smithay proceeds to send `locked`. If `Err(())`, it sends `unlocked`.

            // The primary validation (focus) should ideally happen *before* this handler,
            // in the `Dispatch` for `ZwpPointerConstraintsV1.lock_pointer`.
            // This handler is more of a notification that Smithay is proceeding with the lock.

            // Let's assume basic validation (surface exists, pointer exists) is done by Smithay.
            // Focus check is crucial.
            // If all good, we return Ok(()). Smithay will send `locked`.
            Ok(())
        } else {
            error!("Failed to get PointerHandle for wl_pointer {:?} to apply lock.", wl_pointer_resource);
            Err(()) // Signal failure to Smithay
        }
    }

    fn new_confined_pointer(
        &mut self,
        confined_pointer_resource: ZwpConfinedPointerV1,
        wl_pointer_resource: &wl_pointer::WlPointer,
        surface: &wl_surface::WlSurface,
        region: Option<&wl_region::WlRegion>, // Confinement region
        lifetime: Lifetime,
    ) -> Result<(), ()> {
        info!(
            "Client requests new confined pointer {:?} for wl_pointer {:?}, surface {:?}, region {:?}, lifetime {:?}",
            confined_pointer_resource, wl_pointer_resource, surface, region.map(|r| r.id()), lifetime
        );
        // Similar logic to locked_pointer, but for confinement.
        // - Validate focus, surface type.
        // - Get PointerHandle.
        // - Create `PointerConstraint::Confined`.
        // - Call `pointer_handle.set_constraint`.
        // - Smithay (via delegate and PointerConstraintsState) sends `confined` or `unconfined`.

        // TODO: Implement confinement logic, similar to locking.
        // This involves:
        //  - Getting PointerHandle.
        //  - Validating focus and surface eligibility.
        //  - Converting wl_region (if any) to smithay::utils::Region.
        //  - Calling `pointer_handle.set_constraint` with `PointerConstraint::Confined`.
        //  - The compositor's pointer grab logic must then respect this confinement region
        //    when processing motion events, clamping the pointer to the region.
        warn!("PointerConstraintsHandler::new_confined_pointer: Not yet fully implemented.");

        // Placeholder:
        let pointer_handle_opt: Option<PointerHandle<Self>> = None; // Replace
        if pointer_handle_opt.is_some() {
            // ... validation ...
            Ok(())
        } else {
            Err(())
        }
    }

    fn constraint_destroyed(&mut self, wl_pointer_resource: &wl_pointer::WlPointer, constraint_type: &str) {
        info!(
            "Pointer constraint ({}) destroyed for wl_pointer {:?}",
            constraint_type, wl_pointer_resource
        );
        // A `ZwpLockedPointerV1` or `ZwpConfinedPointerV1` object was destroyed by the client.
        // We need to remove the corresponding constraint from the pointer.

        // TODO: Access the actual Seat and PointerHandle for `wl_pointer_resource`.
        let pointer_handle_opt: Option<PointerHandle<Self>> = None; // Replace with actual lookup
        warn!("PointerConstraintsHandler::constraint_destroyed: Seat/PointerHandle lookup is a placeholder.");

        if let Some(pointer_handle) = pointer_handle_opt {
            // Check current constraint type and clear if it matches.
            // Or, more simply, just clear any constraint.
            // `PointerHandle::set_constraint(None, ...)` removes the current constraint.
            let current_constraint = pointer_handle.current_constraint();
            let should_clear = match current_constraint.as_ref() {
                Some(PointerConstraint::Locked { .. }) => constraint_type == "locked",
                Some(PointerConstraint::Confined { .. }) => constraint_type == "confined",
                None => false,
            };

            if should_clear {
                info!("Removing active {} constraint from pointer {:?}", constraint_type, wl_pointer_resource);
                pointer_handle.set_constraint(None, Serial::now()); // Or appropriate serial
                // If cursor was hidden due to lock, it should be restored by `set_constraint(None)`.
            } else {
                debug!(
                    "Constraint ({}) destroyed, but pointer {:?} had a different or no active constraint. Current: {:?}",
                    constraint_type, wl_pointer_resource, current_constraint.map(|c| c.constraint_type_str())
                );
            }
        } else {
            error!(
                "Failed to get PointerHandle for wl_pointer {:?} to clear constraint ({}).",
                wl_pointer_resource, constraint_type
            );
        }
    }
}

// delegate_pointer_constraints!(DesktopState); // Needs to be NovaCompositorState

/// Initializes and registers the ZwpPointerConstraintsV1 global.
/// `D` is your main compositor state type.
pub fn init_pointer_constraints<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<ZwpPointerConstraintsV1, ()> +
       Dispatch<ZwpPointerConstraintsV1, (), D> +
       Dispatch<ZwpLockedPointerV1, LockedPointerData, D> +   // UserData for ZwpLockedPointerV1
       Dispatch<ZwpConfinedPointerV1, ConfinedPointerData, D> + // UserData for ZwpConfinedPointerV1
       PointerConstraintsHandler + SeatHandler<D> + 'static,
       // D must also own PointerConstraintsState and SeatState.
{
    info!("Initializing ZwpPointerConstraintsV1 global (pointer-constraints-unstable-v1)");

    // Create PointerConstraintsState. This state needs to be managed by your compositor (in D).
    // Example: state.pointer_constraints_state = PointerConstraintsState::new();

    // Each Seat also needs to be aware of constraints. Smithay's Seat/PointerHandle
    // have methods like `set_constraint`.

    display.create_global::<D, ZwpPointerConstraintsV1, _>(
        1, // protocol version
        () // GlobalData for the manager (unit)
    )?;

    // Ensure `delegate_pointer_constraints!(D)` is called in your main compositor state setup.
    // This macro handles:
    // - Dispatching ZwpPointerConstraintsV1 requests (`lock_pointer`, `confine_pointer`).
    //   It calls the `PointerConstraintsHandler` methods (`new_locked_pointer`, `new_confined_pointer`).
    // - Dispatching ZwpLockedPointerV1/ZwpConfinedPointerV1 requests (destroy).
    //   It calls `PointerConstraintsHandler::constraint_destroyed`.
    // - Managing the lifetime of constraint objects and sending `locked`/`unlocked` or `confined`/`unconfined` events.

    info!("ZwpPointerConstraintsV1 global initialized.");
    Ok(())
}

// TODO:
// - Full State Integration:
//   - `PointerConstraintsState` and `SeatState` must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `PointerConstraintsHandler` and `SeatHandler`.
//   - `delegate_pointer_constraints!(NovaCompositorState);` macro must be used.
//   - Correctly look up `PointerHandle` from `wl_pointer` resources within handlers.
// - Pointer Grab Logic for Constraints:
//   - When a pointer is locked:
//     - The cursor image should be hidden (or use a specific "locked" cursor if desired).
//       Smithay's `PointerHandle::set_constraint` should handle cursor visibility changes.
//     - All mouse motion should be reported as relative motion via `wp_relative_pointer_v1`.
//       The compositor's input loop needs to generate these events. The absolute pointer
//       position effectively becomes meaningless to the client.
//     - Pointer events (buttons, axis) should still be delivered to the locked surface.
//   - When a pointer is confined:
//     - The cursor image remains visible.
//     - Absolute pointer motion is reported, but clamped to the confinement region (if any)
//       or the surface boundaries. The compositor's pointer grab needs to implement this clamping.
// - Lifetime Management (Oneshot vs. Persistent):
//   - Oneshot locks/confinements are automatically released by Smithay when:
//     - For locked: pointer focus changes, or a button is pressed (protocol spec varies, check Smithay).
//     - For confined: pointer focus changes.
//     - (The `ZwpLockedPointerV1.unlocked` / `ZwpConfinedPointerV1.unconfined` event is sent).
//   - Persistent locks/confinements remain until the client explicitly destroys the object
//     or requests a new constraint.
// - Region Handling for Confinement:
//   - If a `wl_region` is provided for confinement, convert it to `smithay::utils::Region`
//     and use it for clamping pointer motion. If no region, confine to surface extents.
// - Testing:
//   - Test with applications that use pointer locking (e.g., games like Quake, Xonotic in Wayland mode,
//     specific test clients like `weston-simple-pointer-constraints`).
//   - Verify cursor hiding, relative motion delivery during lock.
//   - Test confinement to surface and to specific regions.
//   - Test lifetime management (oneshot vs. persistent, release on focus change/button press).
//   - Ensure `locked`/`unlocked` and `confined`/`unconfined` events are sent correctly.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod locked_pointer; // Or pointer_constraints
