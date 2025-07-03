// novade-system/src/compositor/protocols/text_input_v3.rs
// Implementation of the text_input_unstable_v3 Wayland protocol

use smithay::{
    delegate_text_input_manager,
    reexports::{
        wayland_protocols_misc::text_input::v3::server::{
            zwp_text_input_manager_v3::{self, ZwpTextInputManagerV3, Request as ManagerRequest},
            zwp_text_input_v3::{self, ZwpTextInputV3, Request as TextInputRequest, Event as TextInputEvent, ChangeCause, ContentHint, ContentPurpose},
        },
        wayland_server::{
            protocol::{wl_seat, wl_surface}, // wl_seat and wl_surface are crucial
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    input::{Seat, SeatHandler, SeatState, keyboard::KeyboardHandle}, // Keyboard focus is key
    utils::{Serial, Logical, Point, Rectangle, Size},
    wayland::{
        text_input::{
            TextInputHandler, TextInputManagerState, TextInputSeatData, // Seat data for TI
            TextInput, // Smithay's wrapper around ZwpTextInputV3
        },
        seat::WaylandFocus, // To know which surface has text focus
    },
    // If TextInput interacts with InputMethod (e.g. compositor bridges them)
    // wayland::input_method::InputMethodSeatData, // To potentially notify IM of text field changes
};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to hold `TextInputManagerState` and interact with `SeatState`.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For Text Input, it would need to manage or access:
    // - TextInputManagerState
    // - SeatState and the currently focused Seat/Keyboard/Surface.
    // - Potentially, interaction with an InputMethod module if the compositor bridges them.
}

#[derive(Debug, Error)]
pub enum TextInputError {
    #[error("Text input unavailable for the given seat or surface")]
    TextInputUnavailable,
    #[error("Text input already active for this surface")]
    TextInputActive,
    #[error("Invalid state or request for text input operation: {0}")]
    InvalidState(String),
}

// UserData for ZwpTextInputV3 resource. Smithay's `TextInput` struct wraps this.
// We might not need custom UserData if `TextInput` holds all necessary state.
// However, `TextInput` itself is often stored in `TextInputSeatData` on the Seat.

// The main compositor state (e.g., NovaCompositorState) would implement TextInputHandler
// and store TextInputManagerState.
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub text_input_manager_state: TextInputManagerState,
//     pub seat_state: SeatState<Self>,
//     // Potentially access to InputMethod state if bridging
//     // pub input_method_manager_state: InputMethodManagerState,
//     ...
// }
//
// impl TextInputHandler for NovaCompositorState {
//     fn text_input_manager_state(&mut self) -> &mut TextInputManagerState {
//         &mut self.text_input_manager_state
//     }
//
//     fn text_input(&mut self, text_input_obj: ZwpTextInputV3, seat: Seat<Self>, surface: wl_surface::WlSurface) {
//         info!("New text input {:?} created for seat {:?}, surface {:?}", text_input_obj, seat.name(), surface);
//         // A client application has created a ZwpTextInputV3 object for a surface.
//         // This means the surface is now a text input field.
//         // We need to store this `text_input_obj` (or Smithay's `TextInput` wrapper)
//         // in the `TextInputSeatData` associated with the `seat`.
//
//         let ti = TextInput::new(text_input_obj, surface); // Smithay's wrapper
//         seat.user_data().get::<TextInputSeatData>().unwrap().set_text_input(Some(ti));
//
//         // TODO: If bridging to an InputMethod, notify the IM that a text field is active.
//         // This might involve creating/activating an ZwpInputMethodContextV2 for the IM.
//     }
//
//     fn unset_text_input(&mut self, seat: Seat<Self>, surface: wl_surface::WlSurface) {
//         info!("Text input unset for seat {:?}, surface {:?}", seat.name(), surface);
//         // The client application is no longer using the surface as a text input field.
//         if let Some(current_ti_surface) = seat.user_data().get::<TextInputSeatData>().unwrap().text_input().map(|ti| ti.surface().clone()) {
//             if current_ti_surface == surface {
//                 seat.user_data().get::<TextInputSeatData>().unwrap().set_text_input(None);
//                 // TODO: If bridging, notify IM that text field is gone.
//             }
//         }
//     }
//
//     // ... other TextInputHandler methods ...
// }
// delegate_text_input_manager!(NovaCompositorState);

impl TextInputHandler for DesktopState { // Replace DesktopState with NovaCompositorState
    fn text_input_manager_state(&mut self) -> &mut TextInputManagerState {
        // TODO: Properly integrate TextInputManagerState with DesktopState or NovaCompositorState.
        panic!("TextInputHandler::text_input_manager_state() needs proper integration.");
        // Example: &mut self.nova_compositor_state.text_input_manager_state
    }

    fn text_input(
        &mut self,
        text_input_resource: ZwpTextInputV3, // The ZwpTextInputV3 resource created by the client
        seat: Seat<Self>,                    // The seat this text input is for
        surface: wl_surface::WlSurface,      // The surface that will receive text input
    ) {
        info!(
            "New text input {:?} requested for seat {:?}, surface {:?}",
            text_input_resource, seat.name(), surface
        );
        // A client application (e.g., text editor, browser) has created a `ZwpTextInputV3` object,
        // indicating that `surface` is now an active text input field.

        // We use Smithay's `TextInput` struct to wrap the `text_input_resource` and `surface`.
        // This `TextInput` object is then stored in the `TextInputSeatData` associated with the `seat`.
        // `TextInputSeatData` should have been added to the seat's UserData when the seat was initialized.

        let text_input_wrapper = TextInput::new(text_input_resource.clone(), surface.clone());
        debug!("Created TextInput wrapper: {:?}", text_input_wrapper);

        let seat_ti_data = seat.user_data().get::<TextInputSeatData>().unwrap_or_else(|| {
            error!("TextInputSeatData not found for seat {:?} during new text input", seat.name());
            // This implies the seat was not correctly initialized for text input.
            panic!("TextInputSeatData missing for seat {}", seat.name());
        });

        // Set this as the active text input for the seat.
        // `TextInputSeatData::set_text_input` also handles sending `enter` and `leave` events
        // to the ZwpTextInputV3 objects when the focused text input changes.
        seat_ti_data.set_text_input(Some(text_input_wrapper));
        info!(
            "Text input {:?} successfully associated with seat {:?} and surface {:?}",
            text_input_resource, seat.name(), surface
        );

        // TODO: Interaction with Input Method (IME)
        // If an Input Method is active on this seat (e.g., from input_method_v2),
        // the compositor might need to bridge information between TextInputV3 and the IM.
        // For example:
        // - When a text field (TIv3) becomes active, notify the IM (IMv2). This might involve
        //   creating an ZwpInputMethodContextV2 for the IM, representing this TIv3 field.
        // - Forward `set_surrounding_text`, `set_content_type`, `set_cursor_rectangle` from TIv3
        //   to the IM's context.
        // - When the IM sends `commit_string` or `preedit_string` (via IMv2 context),
        //   forward these to the active TIv3 object (`text_input_resource`).
        // This bridging logic can be complex and depends on the compositor's architecture.
        // Some compositors leave this bridging to an external IME daemon that speaks both protocols.
        // Smithay's design tends to encourage the compositor to handle this if direct integration is desired.

        // For now, we assume TIv3 operates independently or that an external entity bridges.
        // If the compositor *is* the bridge, this is where you'd start that process.
        // Example: if let Some(im_seat_data) = seat.user_data().get::<InputMethodSeatData>() {
        //     if let Some(active_im) = &im_seat_data.input_method {
        //         // active_im is ZwpInputMethodV2
        //         // Create a ZwpInputMethodContextV2 for it, associated with `surface` or `text_input_resource`.
        //         // Forward initial state from `text_input_resource` if available, or wait for client calls.
        //         warn!("TODO: Bridge new TextInput {:?} to active InputMethod {:?}", text_input_resource, active_im);
        //     }
        // }
    }

    fn unset_text_input(&mut self, seat: Seat<Self>, surface: wl_surface::WlSurface) {
        info!("Text input unset for seat {:?}, surface {:?}", seat.name(), surface);
        // The client application is no longer using `surface` as a text input field,
        // or the `ZwpTextInputV3` object was destroyed.

        let seat_ti_data = seat.user_data().get::<TextInputSeatData>().unwrap_or_else(|| {
            error!("TextInputSeatData not found for seat {:?} during unset text input", seat.name());
            return;
        });

        // Check if the surface being unset is indeed the currently active one.
        if let Some(active_ti) = seat_ti_data.text_input() {
            if active_ti.surface() == &surface {
                seat_ti_data.set_text_input(None); // This sends the `leave` event to the old TI.
                info!("Text input for surface {:?} on seat {:?} has been unset.", surface, seat.name());

                // TODO: Interaction with Input Method (IME)
                // If this TIv3 field was bridged to an IMv2 context, that context should now be
                // deactivated or destroyed.
                // Example: if let Some(im_seat_data) = seat.user_data().get::<InputMethodSeatData>() {
                //     if let Some(active_im) = &im_seat_data.input_method {
                //         // Find and deactivate/destroy the ZwpInputMethodContextV2 associated with `surface`.
                //         warn!("TODO: Deactivate/destroy IM context for surface {:?} bridged from TextInput", surface);
                //     }
                // }
            } else {
                warn!(
                    "Request to unset text input for surface {:?} on seat {:?}, but it was not the active one. Active: {:?}",
                    surface, seat.name(), active_ti.surface()
                );
            }
        } else {
            warn!(
                "Request to unset text input for surface {:?} on seat {:?}, but no text input was active.",
                surface, seat.name()
            );
        }
    }

    // This handler is called when the client (application) commits state changes
    // to its ZwpTextInputV3 object (e.g., after calling set_surrounding_text, set_content_type, etc.,
    // followed by a wl_surface.commit on the text input surface, or an explicit ZwpTextInputV3.commit).
    fn text_input_commit(
        &mut self,
        text_input_obj: &ZwpTextInputV3, // The ZwpTextInputV3 object that committed changes
        seat: Seat<Self>,               // The seat this text input is for
        surface: &wl_surface::WlSurface,  // The surface of the text input field
    ) {
        info!(
            "Commit received for text input {:?} on seat {:?}, surface {:?}",
            text_input_obj, seat.name(), surface
        );

        // The application has updated its text input state (e.g., surrounding text, cursor rectangle).
        // This information needs to be made available to the Input Method if one is active and bridged.

        // Retrieve the current state from the `text_input_obj`.
        // Smithay's `TextInput` wrapper provides convenient accessors for this.
        // We need to find the `TextInput` wrapper corresponding to `text_input_obj`.
        // It should be the active one on `seat_ti_data`.

        let seat_ti_data = seat.user_data().get::<TextInputSeatData>().unwrap_or_else(|| {
            error!("TextInputSeatData not found for seat {:?} during TI commit", seat.name());
            return;
        });

        if let Some(active_ti) = seat_ti_data.text_input() {
            if active_ti.inner() == text_input_obj { // Check if it's the same ZwpTextInputV3 resource
                // Access committed state from `active_ti` (the TextInput wrapper)
                let pending_state = active_ti.pending_state(); // This gets the ZwpTextInputV3_pending_state

                debug!(
                    "TextInput {:?} committed state: surrounding_text: {:?}, cursor_rect: {:?}, content_hint: {:?}, reason: {:?}",
                    text_input_obj,
                    pending_state.surrounding_text,
                    pending_state.cursor_rectangle,
                    pending_state.content_hint,
                    pending_state.reason, // ChangeCause
                );

                // TODO: Bridge this state to the Input Method context (ZwpInputMethodContextV2)
                // if an IM is active and bridged.
                // This would involve:
                // 1. Getting the active `ZwpInputMethodV2` for the seat.
                // 2. Getting (or creating) the `ZwpInputMethodContextV2` for this `surface`/`text_input_obj`.
                // 3. Sending events like `zwp_input_method_context_v2.surrounding_text(...)`,
                //    `zwp_input_method_context_v2.content_type(...)`, etc., to that context.
                //
                // Example:
                // if let Some(im_context_for_this_surface) = find_im_context_for_surface(&seat, &surface) {
                //     if let Some(text) = &pending_state.surrounding_text {
                //         im_context_for_this_surface.surrounding_text(text.clone(), pending_state.surrounding_cursor, pending_state.surrounding_anchor);
                //     }
                //     if let Some(rect) = pending_state.cursor_rectangle {
                //          im_context_for_this_surface.cursor_rectangle(rect.x, rect.y, rect.width, rect.height);
                //     }
                //     // ... and so on for other states ...
                //     // After sending all relevant state updates to the IM context, the IM might need a "commit" too.
                //     // The IM protocol (v2) doesn't have an explicit commit on the context, but the IM client
                //     // processes these events as they arrive.
                //     warn!("TODO: Forward committed TI state for {:?} to active IM context.", text_input_obj);
                // }

            } else {
                warn!("Commit for text input {:?} on seat {:?}, but it's not the active one.", text_input_obj, seat.name());
            }
        } else {
            warn!("Commit for text input {:?} on seat {:?}, but no text input is active.", text_input_obj, seat.name());
        }
    }
}


// delegate_text_input_manager!(DesktopState); // Needs to be NovaCompositorState

/// Initializes and registers the Text Input Manager global.
/// `D` is your main compositor state type.
pub fn init_text_input_manager<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<ZwpTextInputManagerV3, ()> +
       Dispatch<ZwpTextInputManagerV3, (), D> +
       Dispatch<ZwpTextInputV3, (), D> + // UserData for ZwpTextInputV3 is often unit, state is in TextInput wrapper
       TextInputHandler + SeatHandler<D> + 'static,
       // D must also own TextInputManagerState and SeatState.
{
    info!("Initializing ZwpTextInputManagerV3 global (text-input-unstable-v3)");

    // Create TextInputManagerState. This state needs to be managed by your compositor (in D).
    // Example: state.text_input_manager_state = TextInputManagerState::new();

    // Each Seat also needs TextInputSeatData in its UserData when created/initialized.
    // Example (when a seat is created):
    // seat.user_data().insert_if_missing(TextInputSeatData::default);

    display.create_global::<D, ZwpTextInputManagerV3, _>(
        1, // protocol version
        () // GlobalData for the manager (unit)
    )?;

    // Ensure `delegate_text_input_manager!(D)` is called in your main compositor state setup.
    // This macro sets up the necessary Dispatch implementations for ZwpTextInputManagerV3 and ZwpTextInputV3.
    // It relies on `D` implementing `TextInputHandler` and having `TextInputManagerState`.

    info!("ZwpTextInputManagerV3 global initialized.");
    Ok(())
}

// TODO:
// - Full State Integration:
//   - `TextInputManagerState` and `SeatState` must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `TextInputHandler` and `SeatHandler`.
//   - `delegate_text_input_manager!(NovaCompositorState);` macro must be used.
//   - Correctly access shared state within handlers.
// - Bridging with Input Method (IMv2):
//   - This is the most significant part if the compositor handles bridging.
//   - When TIv3 becomes active/inactive, or its state changes (via `text_input_commit`),
//     update the corresponding IMv2 context.
//   - When IMv2 sends commit_string, preedit_string, delete_surrounding_text, etc.,
//     forward these actions to the active TIv3 object using its requests
//     (e.g., `zwp_text_input_v3.commit_string`, `zwp_text_input_v3.preedit_string`).
//   - This requires careful state management to map TIv3 instances to IMv2 contexts.
// - Keyboard Event Forwarding:
//   - If no IME is active (or if IME passes through events), keyboard events from the seat
//     (after processing by XKB) need to be delivered to the focused application.
//     Typically, this means sending `wl_keyboard.key`, `wl_keyboard.modifiers` to the
//     `wl_keyboard` resource of the client owning the focused surface (which also has the active TIv3).
//     Smithay's `Seat::get_keyboard().send_key(...)` etc. are used for this.
//     The TIv3 protocol itself doesn't handle key event delivery; it's about text state.
// - Testing:
//   - Test with various Wayland applications that support text-input-unstable-v3 (e.g., modern GTK/Qt apps).
//   - Verify that `enter` and `leave` events are correctly sent to TIv3 objects as focus changes.
//   - Verify that state changes from the app (surrounding text, content type, cursor rect) are received.
//   - If bridging to IMv2:
//     - Test full input lifecycle with an IME (e.g., IBus): focus text field, IME activates,
//       type pre-edit, select candidates, commit text to application.
//     - Test that application state changes (e.g., moving cursor) are reflected in the IME.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod text_input_v3;
