// novade-system/src/compositor/protocols/input_method_v2.rs
// Implementation of the input_method_unstable_v2 Wayland protocol

use smithay::{
    delegate_input_method_manager,
    reexports::{
        wayland_protocols_misc::input_method::v2::server::{
            zwp_input_method_context_v2::{self, ZwpInputMethodContextV2, Request as ContextRequest, Event as ContextEvent},
            zwp_input_method_keyboard_grab_v2::{self, ZwpInputMethodKeyboardGrabV2, Request as GrabRequest},
            zwp_input_method_manager_v2::{self, ZwpInputMethodManagerV2, Request as ManagerRequest},
            zwp_input_method_v2::{self, ZwpInputMethodV2, Request as InputMethodRequest, Event as InputMethodEvent},
        },
        wayland_server::{
            protocol::{wl_seat, wl_surface, wl_keyboard}, // wl_seat is crucial
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    input::{Seat, SeatHandler, SeatState, keyboard::{KeyboardHandle, KeyState, FilterResult, KeysymHandle, ModifiersState, LedState, XkbConfig}, GrabStatus, KeyboardGrab}, // Keyboard interaction
    utils::{Serial, Logical, Point, Rectangle},
    wayland::{
        input_method::{
            InputMethodHandler, InputMethodManagerState, InputMethodSeatData, // Seat data for IM
            InputMethodPopupSurfaceData, // For IM popups like candidate lists
            InputMethodKeyboardGrab, // Smithay's IM Keyboard Grab
        },
        seat::WaylandFocus, // To know which surface has text focus
        shell::xdg::XdgPopupSurfaceData, // IM popups are often XDG Popups
    },
    desktop::PopupManager, // To manage IM popup surfaces
};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to hold `InputMethodManagerState` and interact with `SeatState`.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For Input Method, it would need to manage or access:
    // - InputMethodManagerState
    // - SeatState and the currently focused Seat/Keyboard/Surface.
    // - PopupManager for candidate list popups.
    // - Potentially, a connection to an IME backend daemon (e.g., IBus, Fcitx) via D-Bus or similar.
}

#[derive(Debug, Error)]
pub enum InputMethodError {
    #[error("Input method unavailable for the given seat")]
    InputMethodUnavailable,
    #[error("Input method context already exists for this surface")]
    ContextExists,
    #[error("Keyboard grab failed for input method")]
    GrabFailed,
    #[error("IME Backend communication error: {0}")]
    BackendError(String),
    #[error("Invalid state or request for input method operation: {0}")]
    InvalidState(String),
}

// UserData for ZwpInputMethodV2 resource (the per-seat IME interface)
#[derive(Debug, Default)]
pub struct InputMethodData {
    // We might store which seat this IM is for, or if it's active.
    // Smithay's InputMethodSeatData on the Seat might be the primary store.
}

// UserData for ZwpInputMethodContextV2 resource (per text-input client surface)
#[derive(Debug, Default)]
pub struct InputMethodContextData {
    // Stores state related to a specific text input context, e.g., surrounding text, cursor rect.
    // This data is sent from the client (application) to the IME.
    pub surrounding_text: Option<String>,
    pub surrounding_cursor: Option<u32>, // byte position
    pub surrounding_anchor: Option<u32>, // byte position
    pub content_type_hint: Option<u32>, // zwp_input_method_context_v2::ContentType
    pub content_type_purpose: Option<u32>,
    // The wl_surface this context is for, might be useful for linking.
    // pub surface: Option<wl_surface::WlSurface>, // Or get from resource parent.
}


// The main compositor state (e.g., NovaCompositorState) would implement InputMethodHandler
// and store InputMethodManagerState.
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub input_method_manager_state: InputMethodManagerState,
//     pub seat_state: SeatState<Self>,
//     pub popup_manager: PopupManager, // For IM candidate popups
//     // Connection to IME backend (e.g. IBus proxy)
//     // pub ime_backend: Option<ImeBackendConnection>,
//     ...
// }
//
// impl InputMethodHandler for NovaCompositorState {
//     fn input_method_manager_state(&mut self) -> &mut InputMethodManagerState {
//         &mut self.input_method_manager_state
//     }
//
//     fn activate(&mut self, im: ZwpInputMethodV2, seat: Seat<Self>) {
//         info!("Input method {:?} activated for seat {:?}", im, seat.name());
//         // IME client (e.g., IBus UI) is ready to handle input for this seat.
//         // Store `im` with the seat's InputMethodSeatData.
//         // Potentially notify IME backend that Wayland IME is active.
//         seat.user_data().get::<InputMethodSeatData>().unwrap().input_method = Some(im);
//     }
//
//     fn deactivate(&mut self, im: ZwpInputMethodV2, seat: Seat<Self>) {
//         info!("Input method {:?} deactivated for seat {:?}", im, seat.name());
//         // IME client is no longer handling input for this seat.
//         // Clear `im` from seat's InputMethodSeatData.
//         // Potentially notify IME backend.
//         if let Some(active_im) = &seat.user_data().get::<InputMethodSeatData>().unwrap().input_method {
//             if active_im == &im {
//                 seat.user_data().get::<InputMethodSeatData>().unwrap().input_method = None;
//             }
//         }
//     }
//
//     // ... other InputMethodHandler methods ...
// }
// delegate_input_method_manager!(NovaCompositorState);


impl InputMethodHandler for DesktopState { // Replace DesktopState with NovaCompositorState
    fn input_method_manager_state(&mut self) -> &mut InputMethodManagerState {
        // TODO: Properly integrate InputMethodManagerState with DesktopState or NovaCompositorState.
        panic!("InputMethodHandler::input_method_manager_state() needs proper integration.");
        // Example: &mut self.nova_compositor_state.input_method_manager_state
    }

    fn activate(&mut self, im_obj: ZwpInputMethodV2, seat: Seat<Self>) {
        info!("Input method {:?} wants to activate for seat {:?}", im_obj, seat.name());
        // An IME client (e.g., IBus Wayland bridge) has created an `zwp_input_method_v2` object
        // and is ready to serve input methods for the given `seat`.

        // We should store this `im_obj` in the `InputMethodSeatData` associated with the `seat`.
        // This `im_obj` will be used to forward keyboard events (if grabbed) and text input state
        // (from `ZwpInputMethodContextV2`) to the IME client.
        let seat_im_data = seat.user_data().get::<InputMethodSeatData>().unwrap_or_else(|| {
            // This should not happen if seat was initialized correctly for input method.
            error!("InputMethodSeatData not found for seat {:?} during IM activate", seat.name());
            // If it can happen, we might need to create it here.
            // For now, assume it exists.
            panic!("InputMethodSeatData missing for seat {}", seat.name());
        });

        seat_im_data.input_method = Some(im_obj.clone()); // Store the active IME object for the seat
        debug!("Associated ZwpInputMethodV2 {:?} with seat {:?}", im_obj, seat.name());

        // TODO:
        // - Notify the actual IME backend (IBus, Fcitx daemon) that a Wayland IME is now active
        //   for this seat, if such a backend is used and needs explicit activation.
        // - The IME client (`im_obj`) will now typically wait for `ZwpInputMethodContextV2`
        //   objects to be created by applications, or it might try to grab the keyboard.
    }

    fn deactivate(&mut self, im_obj: ZwpInputMethodV2, seat: Seat<Self>) {
        info!("Input method {:?} wants to deactivate for seat {:?}", im_obj, seat.name());
        // The IME client is no longer serving input methods for this `seat`.

        let seat_im_data = seat.user_data().get::<InputMethodSeatData>().unwrap_or_else(|| {
            error!("InputMethodSeatData not found for seat {:?} during IM deactivate", seat.name());
            return;
        });

        if let Some(active_im) = &seat_im_data.input_method {
            if active_im == &im_obj {
                seat_im_data.input_method = None;
                debug!("Disassociated ZwpInputMethodV2 {:?} from seat {:?}", im_obj, seat.name());
                // TODO:
                // - Notify IME backend if needed.
                // - If this IME had a keyboard grab, it should be released.
                //   (Smithay's `InputMethodKeyboardGrab` handles this on `Drop`).
            } else {
                warn!(
                    "Deactivate requested for IM {:?} on seat {:?}, but current active IM is {:?}.",
                    im_obj, seat.name(), active_im
                );
            }
        } else {
            warn!("Deactivate requested for IM {:?} on seat {:?}, but no IM was active.", im_obj, seat.name());
        }
    }

    // This method is called when a client application creates an ZwpInputMethodContextV2 object.
    fn new_input_method_context(
        &mut self,
        im_context_obj: ZwpInputMethodContextV2, // The new context object from the app
        seat: Seat<Self>, // The seat this context is for
        // surface: wl_surface::WlSurface, // The application surface this context belongs to. This is not directly provided by Smithay here.
                                       // The context is created by ZwpInputMethodManagerV2.get_input_method_context(id, surface)
                                       // Smithay's delegate_input_method_manager should handle this association.
                                       // The `im_context_obj` can be queried for its surface.
    ) {
        // An application has created an input method context, indicating it's ready for text input.
        // This `im_context_obj` will be used by the application to send its state (surrounding text, etc.)
        // to the IME, and by the IME to send commit strings, preedit text, etc., to the application.

        // We need to:
        // 1. Find the active `ZwpInputMethodV2` object for the `seat` (if any).
        // 2. If an active IME exists, forward this new `im_context_obj` to it by sending
        //    the `zwp_input_method_v2.input_method_context(im_context_obj)` event.
        //    Smithay's `InputMethodSeatData::new_input_method_context` handles this.

        let surface = match im_context_obj.data::<InputMethodContextData>() {
            // This is tricky. The UserData for ZwpInputMethodContextV2 is InputMethodContextData.
            // We need the wl_surface it's for. This association is made when the client calls
            // ZwpInputMethodManagerV2.get_input_method_context(id, surface).
            // Smithay's machinery should ensure the ZwpInputMethodContextV2 resource is associated with the wl_surface.
            // We might need to query the resource map or have this info passed differently.
            //
            // Smithay's `delegate_input_method_manager` for `ZwpInputMethodManagerV2`'s `get_input_method_context`
            // request takes `(id: New<ZwpInputMethodContextV2>, surface: WlSurface)`.
            // It creates the `ZwpInputMethodContextV2` and should associate it with the surface.
            // This `new_input_method_context` handler is then called.
            // We need a way to get that `wl_surface` here.
            //
            // The `im_context_obj` itself doesn't have a direct `wl_surface()` method in its generated API.
            // This implies the association is managed by Smithay's internal state or UserData.
            // Let's assume `InputMethodSeatData::new_input_method_context` in Smithay handles this.
            None => { // This assumes InputMethodContextData would hold the surface, which it doesn't by default.
                error!("Cannot find wl_surface for new ZwpInputMethodContextV2 {:?}", im_context_obj);
                // This is a gap in how this handler is called vs. data needed.
                // For now, we proceed, but this is a critical point.
                // Smithay's `InputMethodSeatData::new_context` takes the `ZwpInputMethodContextV2`
                // and it internally sends the `input_method_context` event to the active IME.
                // It doesn't seem to require the surface at *this* specific handler call.
                // The surface was involved in *creating* the context.
                // The context object itself is the key.
            }
        };
        info!(
            "New input method context {:?} created for seat {:?}", // Surface info would be good here
            im_context_obj, seat.name()
        );


        let seat_im_data = seat.user_data().get::<InputMethodSeatData>().unwrap_or_else(|| {
            error!("InputMethodSeatData not found for seat {:?} during new IM context", seat.name());
            return;
        });

        // This is the key step: notify the active IME about the new context.
        // Smithay provides a helper for this on InputMethodSeatData.
        seat_im_data.new_context(&im_context_obj);
        debug!("Notified active IME on seat {:?} about new context {:?}", seat.name(), im_context_obj);
    }


    // Methods related to IM popups (e.g., candidate lists)
    fn new_popup_surface(
        &mut self,
        popup_surface_obj: ZwpInputMethodPopupSurfaceV2, // The new popup surface resource from IME
        // parent_im_context_surface: wl_surface::WlSurface, // Surface of the text input client (parent for popup)
                                                       // This is not directly given by Smithay's handler signature.
                                                       // The popup is created via ZwpInputMethodV2.get_popup_surface(id, surface_for_text_input)
        seat: Seat<Self>,
    ) {
        // The IME client wants to show a popup surface (e.g., candidate list).
        // This `popup_surface_obj` is a Wayland resource that needs to be associated with
        // an `xdg_popup` (or similar) to be displayed.
        // The `parent_im_context_surface` is the application's surface that the popup is relative to.

        // We need to:
        // 1. Get the `wl_surface` that backs `popup_surface_obj`.
        //    (This might be done by `popup_surface_obj.wl_surface()` if it's already configured, or later).
        // 2. Treat this `wl_surface` as an XDG Popup (or similar popup type).
        //    This means using `PopupManager::create_xdg_popup_from_wlr` (or equivalent for this protocol if different).
        //    The `zwp_input_method_popup_surface_v2` needs to be given the role of an `xdg_popup`.
        //    The protocol `zwp_input_method_popup_surface_v2` has a request `xdg_popup(xdg_popup_new_id, xdg_positioner)`.
        //    The IME client calls this to turn its popup surface into an XDG popup.

        // This handler `new_popup_surface` is called when the `ZwpInputMethodV2.get_popup_surface` request is made.
        // At this point, the `popup_surface_obj` is just a resource. The IME client then needs to
        // commit a role to its underlying `wl_surface` and then call `xdg_popup` on `popup_surface_obj`.

        // TODO: Access PopupManager from `self`.
        // let popup_manager = &mut self.popup_manager;

        info!(
            "IME requests new popup surface resource {:?} on seat {:?}",
            popup_surface_obj, seat.name()
        );
        // We store `InputMethodPopupSurfaceData` in the UserData of `popup_surface_obj`.
        // This data might hold a reference to the parent text input surface.
        // Smithay's `delegate_input_method_manager` and `InputMethodManagerState` handle setting up UserData.

        // The actual creation of an XDG popup from this will happen when the IME client calls
        // `zwp_input_method_popup_surface_v2.xdg_popup(...)`.
        // Our `Dispatch` handler for `ZwpInputMethodPopupSurfaceV2` will handle that request.
        debug!("Awaiting IME client to commit role and configure popup {:?} as XDG popup.", popup_surface_obj);
    }

    fn grab_keyboard(&mut self, keyboard_grab_obj: ZwpInputMethodKeyboardGrabV2, seat: Seat<Self>) -> Result<(), ()> {
        // The IME client wants to grab the keyboard for the `seat`.
        // This means all keyboard events for this seat should be forwarded to the IME
        // via `zwp_input_method_keyboard_grab_v2.key/modifiers/keymap` events,
        // instead of going to the application with current keyboard focus.
        info!("IME requests keyboard grab {:?} for seat {:?}", keyboard_grab_obj, seat.name());

        // We need to:
        // 1. Get the `KeyboardHandle` for the `seat`.
        // 2. Start a new keyboard grab using an `InputMethodKeyboardGrab` handler.
        //    This grab handler will forward events to `keyboard_grab_obj`.

        if let Some(keyboard) = seat.get_keyboard() {
            // `InputMethodKeyboardGrab` is a struct from Smithay that implements `KeyboardGrab`.
            // It takes the `ZwpInputMethodKeyboardGrabV2` resource to send events to.
            let grab_handler = InputMethodKeyboardGrab::new(&keyboard_grab_obj);
            keyboard.set_grab(seat.user_data().get().unwrap(), grab_handler, Serial::now(), GrabStatus::Grab); // TODO: get actual serial
            // The GrabStatus::Grab might need to be GrabStatus::Focus if we want to allow focus changes.
            // For IME, typically it's a full grab.
            // The `user_data().get().unwrap()` part is problematic, needs proper access to D.
            // `keyboard.set_grab(self, grab_handler, ...)` if DesktopState is D.
            info!("Keyboard grab started successfully for IME on seat {:?}", seat.name());
            Ok(())
        } else {
            error!("No keyboard found on seat {:?} to start IME grab.", seat.name());
            // Notify IME that grab failed. The protocol doesn't have an explicit error for this on grab request.
            // We might need to destroy `keyboard_grab_obj` with an error if possible, or it fails silently for IME.
            // Returning Err(()) signals to Smithay that we couldn't fulfill the grab.
            // Smithay might then destroy the keyboard_grab_obj resource.
            Err(())
        }
    }

    // Note: `reposition_popup` and other popup geometry methods are usually handled by
    // the XDG Shell handlers if the IM popup becomes an XDG Popup.
}

// delegate_input_method_manager!(DesktopState); // Needs to be NovaCompositorState

/// Initializes and registers the Input Method Manager global.
/// `D` is your main compositor state type.
pub fn init_input_method_manager<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<ZwpInputMethodManagerV2, ()> +
       Dispatch<ZwpInputMethodManagerV2, (), D> +
       Dispatch<ZwpInputMethodV2, InputMethodData, D> + // UserData for the per-seat IM object
       Dispatch<ZwpInputMethodContextV2, InputMethodContextData, D> + // UserData for per-app-surface context
       Dispatch<zwp_input_method_popup_surface_v2::ZwpInputMethodPopupSurfaceV2, InputMethodPopupSurfaceData, D> + // UserData for IM popups
       Dispatch<ZwpInputMethodKeyboardGrabV2, (), D> + // UserData for keyboard grab (often unit)
       InputMethodHandler + SeatHandler<D> + 'static, // SeatHandler for seat setup
       // D must also own InputMethodManagerState, SeatState, PopupManager.
{
    info!("Initializing ZwpInputMethodManagerV2 global (input-method-unstable-v2)");

    // Create InputMethodManagerState. This state needs to be managed by your compositor (in D).
    // Example: state.input_method_manager_state = InputMethodManagerState::new();

    // Each Seat also needs InputMethodSeatData in its UserData when created/initialized.
    // Example (when a seat is created):
    // seat.user_data().insert_if_missing(InputMethodSeatData::default);

    display.create_global::<D, ZwpInputMethodManagerV2, _>(
        1, // protocol version
        () // GlobalData for the manager (unit)
    )?;

    // Ensure `delegate_input_method_manager!(D)` is called in your main compositor state setup.
    // This macro sets up the necessary Dispatch implementations for all related input method objects.
    // It relies on `D` implementing `InputMethodHandler` and having `InputMethodManagerState`.

    info!("ZwpInputMethodManagerV2 global initialized.");
    Ok(())
}

// TODO:
// - Full State Integration:
//   - `InputMethodManagerState`, `SeatState`, `PopupManager` must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `InputMethodHandler` and `SeatHandler`.
//   - `delegate_input_method_manager!(NovaCompositorState);` macro must be used.
//   - Correctly access shared state within handlers instead of placeholders.
// - IME Backend Integration (IBus, Fcitx, etc.):
//   - This is the most complex part. The compositor needs to communicate with the system's
//     IME backend daemon (e.g., via D-Bus).
//   - When Wayland IME activates/deactivates, notify backend.
//   - Forward text input context state (surrounding text, cursor position, content type) from
//     `ZwpInputMethodContextV2` to the backend.
//   - Receive commit strings, pre-edit text, candidate lists, etc., from the backend and forward
//     them to the appropriate `ZwpInputMethodContextV2` and `ZwpInputMethodPopupSurfaceV2`.
//   - Handle keyboard events if grabbed: forward to backend, receive IME-processed key events or actions.
// - Popup Surface Management:
//   - Fully implement the logic for handling `ZwpInputMethodPopupSurfaceV2`, especially
//     its `xdg_popup` request, to display IME candidate windows using `PopupManager`.
// - Keyboard Grab Logic:
//   - Ensure `InputMethodKeyboardGrab` correctly forwards all necessary key events, modifiers,
//     and keymap changes to the IME via `ZwpInputMethodKeyboardGrabV2` events.
//   - Handle grab lifecycle correctly (release on IME deactivate, etc.).
// - Text Input v3 Coexistence:
//   - If `text_input_unstable_v3` is also implemented, ensure they interact correctly or
//     that one is preferred/used based on client support. Often, an IME uses IMv2, and
//     applications use TIv3, with the compositor bridging them if necessary, or the IME backend
//     handling compatibility.
// - Testing:
//   - Use an IME backend (e.g., IBus with an engine like ibus-libpinyin for Chinese, or ibus-anthy for Japanese)
//     configured for Wayland.
//   - Test text input in various Wayland applications (e.g., GTK4/Qt6 apps, text editors).
//   - Verify pre-edit text, candidate list display, commit strings.
//   - Test keyboard grab behavior.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod input_method_v2;
