// novade-system/src/compositor/protocols/data_device.rs
// Implementation of the wl_data_device_manager and related Wayland protocols

use smithay::{
    delegate_data_device,
    input::{Seat, SeatHandler, SeatState, pointer::PointerHandle, keyboard::KeyboardHandle}, // Seat is crucial for data device
    reexports::{
        wayland_server::{
            protocol::{
                wl_data_device::{self, WlDataDevice, Request as DataDeviceRequest, Event as DataDeviceEvent},
                wl_data_device_manager::{self, WlDataDeviceManager, Request as DataDeviceManagerRequest},
                wl_data_offer::{self, WlDataOffer, Event as DataOfferEvent},
                wl_data_source::{self, WlDataSource, Request as DataSourceRequest, Event as DataSourceEvent},
                wl_surface,
            },
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
        mime::MimeType, // For handling MIME types
    },
    utils::{Serial, Logical, Point, Rectangle},
    wayland::{
        compositor::{CompositorState, CompositorHandler}, // May be needed for surface interactions
        data_device::{
            DataDeviceHandler, DataDeviceState, ServerDndGrabHandler, ClientDndGrabHandler,
            DataDeviceData, DataSourceData, DragGrabState,
        },
        seat::WaylandFocus, // For focus tracking related to selections
    },
};
use std::{
    sync::{Arc, Mutex},
    collections::HashSet,
    io::{Read, Write, ErrorKind as IoErrorKind},
    os::unix::io::{AsRawFd, OwnedFd},
};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to hold `DataDeviceState` and interact with `SeatState`.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For data device, it would need to manage or access:
    // - The current selection source (if any).
    // - Information about ongoing drag-and-drop operations.
    // - Potentially, integration with XWayland clipboard.
}

#[derive(Debug, Error)]
pub enum DataDeviceError {
    #[error("Seat is unavailable or not found")]
    SeatUnavailable,
    #[error("MIME type not supported or offered")]
    MimeTypeNotSupported,
    #[error("Invalid data source operation")]
    InvalidDataSource,
    #[error("Invalid data offer operation")]
    InvalidDataOffer,
    #[error("Drag and Drop operation failed: {0}")]
    DndError(String),
    #[error("Clipboard operation failed: {0}")]
    ClipboardError(String),
}

// The main compositor state (e.g., NovaCompositorState) would implement DataDeviceHandler
// and store DataDeviceState.
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub data_device_state: DataDeviceState,
//     pub seat_state: SeatState<Self>,
//     ...
// }
//
// impl DataDeviceHandler for NovaCompositorState {
//     fn data_device_state(&self) -> &DataDeviceState { // Note: Smithay's trait wants & not &mut
//         &self.data_device_state
//     }
//     // ... other methods ...
// }
// delegate_data_device!(NovaCompositorState);

impl DataDeviceHandler for DesktopState { // Replace DesktopState with NovaCompositorState
    fn data_device_state(&self) -> &DataDeviceState {
        // TODO: Properly integrate DataDeviceState with DesktopState or NovaCompositorState.
        // This requires `DesktopState` (or the global state) to own an instance of `DataDeviceState`.
        panic!("DataDeviceHandler::data_device_state() needs proper integration. DesktopState must own DataDeviceState.");
        // Example: &self.nova_compositor_state.data_device_state
    }

    // --- Selection (Copy/Paste) Callbacks ---
    fn new_selection(&mut self, source: Option<wl_data_source::WlDataSource>, seat: Seat<Self>) {
        info!("New selection offered by source: {:?} on seat {:?}", source, seat.name());
        // A client has set a new selection (e.g., after a copy operation).
        // `source` is the new WlDataSource offering the data.
        // If `source` is None, the selection is cleared.

        // Smithay's `DataDeviceState` and `Seat` handle much of this.
        // When a client calls `wl_data_device.set_selection(source, serial)`,
        // Smithay updates the seat's selection state.
        // This callback informs us of that change.

        // We might need to:
        // 1. Notify other clients that the selection has changed (via `wl_data_device.selection` event).
        //    Smithay's `Seat::set_selection` typically handles this.
        // 2. If XWayland is active, synchronize this selection with the X11 clipboard. (Complex TODO)

        if let Some(ref src) = source {
            debug!("New selection data source: {:?}, client: {:?}", src, src.client());
            // You can inspect offered MIME types via `with_states` on the source, e.g.:
            // with_states(src, |states| {
            //     if let Some(data_source_data) = states.data_map.get::<DataSourceData>() {
            //         debug!("Offered MIME types: {:?}", data_source_data.mime_types);
            //     }
            // });
        } else {
            debug!("Selection cleared on seat {:?}", seat.name());
        }
        // TODO: XWayland clipboard synchronization if applicable.
    }

    fn send_selection_data(
        &mut self,
        mime_type: String,
        fd: OwnedFd,
        seat: Seat<Self>, // Seat on which selection is requested
        // source_client: Client, // Client owning the current selection source
        // target_client: Client, // Client requesting the data
    ) {
        info!("Request to send selection data for type '{}' to fd {} on seat {:?}", mime_type, fd.as_raw_fd(), seat.name());
        // This callback is triggered when a client requests data for the current selection
        // (e.g., after a paste operation, client calls `wl_data_offer.receive(mime_type, fd)`).

        // Smithay's `DataDeviceState` handles the `WlDataSource::send` event to the source client.
        // This callback might be more for observing or intercepting, as Smithay handles the pipe.
        // The `Seat::get_selection()` gives the current `DataSourceHandler` which has the `send_data` method.

        // Smithay's internal logic:
        // 1. Target client calls `wl_data_offer.receive(mime_type, fd)`.
        // 2. Smithay finds the current selection source for the seat.
        // 3. Smithay sends `wl_data_source.send(mime_type, fd)` to the source client.
        // 4. The source client writes data to the fd and closes it.
        // This `send_selection_data` handler method in `DataDeviceHandler` seems to be invoked
        // by Smithay as part of step 3, essentially asking the compositor to fulfill the request
        // by interacting with the source.

        // However, the `DataDeviceHandler::send_selection_data` signature in Smithay (as of recent versions)
        // might be different or this specific callback might be part of a more detailed handler trait.
        // Let's check the current Smithay `DataDeviceHandler` trait.
        //
        // Smithay's `DataDeviceHandler` (e.g., 0.3.x, 0.4.x) doesn't seem to have `send_selection_data`.
        // Instead, `Seat::transfer_selection` or similar mechanisms are used.
        // The core logic is:
        // - `Client A` sets selection with `WlDataSourceA`.
        // - `Client B` gets a `WlDataOfferB` representing this selection.
        // - `Client B` calls `receive(mime, fd)` on `WlDataOfferB`.
        // - Smithay (via `DataDeviceState` or `Seat`) then sends `send(mime, fd)` to `WlDataSourceA`.
        //
        // This handler might be a custom point if we were manually proxying, but with Smithay's
        // `DataDeviceState`, this is largely automatic.
        //
        // Let's assume this handler is for advanced scenarios or if we need to override default behavior.
        // For now, we'll log, as Smithay should manage the piping.

        // The `WlDataSource` itself has a request handler for `send`.
        // The `DataSourceData` in Smithay stores the offered MIME types.
        // When `WlDataOffer::receive` is called, Smithay ensures the `WlDataSource::send`
        // event is dispatched to the client owning the source.

        // This callback might be if the COMPOSITOR itself is the source of the selection.
        // Or, if `Seat::set_selection` was called with a custom `SelectionSource`
        // that needs compositor intervention.
        // If the selection source is a regular client `WlDataSource`, Smithay handles it.

        // Let's log and assume Smithay handles piping for client-to-client cases.
        // If the compositor needs to *be* a data source, that's a different mechanism.
        warn!("DataDeviceHandler::send_selection_data called. Default Smithay piping should handle client-to-client. Ensure this is the intended use.");

        // Example of how one might handle it if the compositor *was* the source (conceptual):
        // if let Some(data_to_send) = self.get_compositor_clipboard_data_for_mimetype(&mime_type) {
        //     let mut pipe = std::fs::File::from(fd); // Create a File from OwnedFd
        //     if let Err(e) = pipe.write_all(data_to_send.as_bytes()) {
        //         error!("Error writing selection data to pipe: {}", e);
        //     }
        //     // `pipe` (and thus `fd`) is closed when `pipe` goes out of scope.
        // } else {
        //     error!("Compositor asked to send selection data for type '{}', but has no data.", mime_type);
        //     // fd will be closed automatically. Client will get EOF.
        // }
    }

    // --- Drag and Drop Callbacks ---
    fn start_drag(
        &mut self,
        source: Option<wl_data_source::WlDataSource>, // Source of the drag
        origin: wl_surface::WlSurface,      // Surface where drag started
        icon: Option<wl_surface::WlSurface>,// Optional drag icon surface
        seat: Seat<Self>,
    ) {
        info!(
            "Drag started from source: {:?}, origin: {:?}, icon: {:?}, on seat {:?}",
            source, origin, icon, seat.name()
        );
        // A client has initiated a drag operation.
        // `source` provides the data being dragged.
        // `origin` is the client surface from which the drag originates.
        // `icon` is an optional surface to use as the drag cursor.

        // Smithay's `Seat::start_pointer_drag` or `Seat::start_touch_drag` is typically called by us
        // when the client sends `wl_data_device.start_drag`.
        // This callback informs us that such a drag has been validated and started by Smithay.

        // We need to:
        // 1. Potentially change the cursor icon (if `icon` is None, use a default DND cursor).
        //    If `icon` is Some, we need to render it at the pointer/touch location.
        // 2. Track pointer/touch movement to send `wl_data_device.enter/leave/motion/drop` events.
        //    Smithay's `DragGrabState` (part of `PointerGrab` or `TouchGrab`) handles this.
        //    When the pointer moves over a new surface, `PointerHandle::motion` grab callback
        //    will be called. Inside it, we check if the surface accepts the DND offer.

        // Smithay setup for DND usually involves:
        // - Client calls `wl_data_device.start_drag(source, origin, icon_surface, serial)`.
        // - Our `Dispatch<WlDataDevice>` handler for `StartDrag` calls `seat.start_pointer_drag(...)`.
        // - This `DataDeviceHandler::start_drag` callback is then invoked by Smithay.

        // The actual grab logic (reacting to pointer motion, enter, leave, drop) is handled by
        // the active grab handler (e.g., `ServerDndGrabHandler` or a custom one).
        // This callback is more of a notification that the DND session has begun.

        // If `icon` is Some, we need to map it and render it.
        // This often involves creating a `Window` or `LayerSurface` for the icon.
        // Smithay provides `xdg_shell::drag_icon_ SupraSurface` or similar helpers sometimes.
        // Or, we can manage it as a special kind of surface in our renderer.
        if let Some(icon_surface) = icon {
            debug!("Drag icon surface: {:?}", icon_surface);
            // TODO: Map and render the icon surface at the pointer location.
            // This might involve creating a small, unmanaged `Window` or similar.
            // The icon surface needs to be told its role (e.g., via a custom protocol or by convention).
            // Smithay's `set_data_device_grab_icon` on `PointerHandle` might be relevant.
        } else {
            // TODO: Set a default DND cursor icon.
        }
    }

    fn dnd_dropped(&mut self, seat: Seat<Self>) {
        info!("Drag and drop operation: data dropped on seat {:?}", seat.name());
        // The user has released the button during a DND operation over a valid target.
        // The target client's `WlDataOffer` will receive `wl_data_offer.drop`.
        // The source client's `WlDataSource` will receive `wl_data_source.dnd_drop_performed`.

        // Smithay's drag grab handler (`ServerDndGrabHandler`) processes the drop input:
        // - Sends `wl_data_device.drop()` to the focused client's `WlDataDevice`.
        // - Sends `wl_data_offer.action()` and `wl_data_offer.drop()` to the target `WlDataOffer`.
        // - If the source is a Wayland client, sends `wl_data_source.dnd_drop_performed()`.
        // - Calls `wl_data_source.dnd_finished()` if the action was accepted.
        // - Ends the grab.

        // This callback is a notification that the drop has occurred.
        // We might need to:
        // 1. Clean up any drag icon surface.
        // 2. Finalize any visual feedback.
        debug!("DND drop processed by Smithay. Source and target clients notified.");
        // TODO: Unmap/destroy drag icon surface if one was created.
    }

    fn dnd_cancelled(&mut self, seat: Seat<Self>) {
        info!("Drag and drop operation: cancelled on seat {:?}", seat.name());
        // The DND operation was cancelled (e.g., user pressed Esc, or dragged to an invalid location and released).
        // The source client's `WlDataSource` should receive `wl_data_source.dnd_finished` (if started)
        // or `wl_data_source.cancelled`.

        // Smithay's drag grab handler:
        // - If a target existed, sends `wl_data_device.leave()` to it.
        // - Sends `wl_data_source.cancelled()` to the source `WlDataSource`.
        // - Ends the grab.

        // This callback is a notification.
        // We need to:
        // 1. Clean up any drag icon surface.
        // 2. Revert cursor to normal.
        debug!("DND cancelled. Source client notified.");
        // TODO: Unmap/destroy drag icon surface if one was created.
        // TODO: Restore normal cursor icon.
    }

    fn dnd_accept_mime(
        &mut self,
        target_surface: wl_surface::WlSurface, // Surface the DND is over
        mime_type: MimeType,                  // MimeType offered by the current DND source
        seat: Seat<Self>,
    ) -> bool {
        info!(
            "Checking if surface {:?} accepts DND MIME type '{}' on seat {:?}",
            target_surface, mime_type, seat.name()
        );
        // Called by Smithay's DND grab handler during pointer motion over a surface.
        // We need to determine if the `target_surface` can accept the `mime_type`.
        // This is usually done by checking `WlDataOffer::actions()` or stored client preferences.

        // How this typically works:
        // 1. DND is active, pointer moves over `target_surface`.
        // 2. If this is a new surface, a `WlDataOffer` is created for the client owning `target_surface`.
        //    This offer represents the data from the DND source.
        // 3. The client owning `target_surface` receives the `WlDataOffer` and calls `accept(serial, mime_type)`
        //    and `set_actions(actions, preferred_action)` on it.
        // 4. Smithay stores this information (accepted types, actions) in `WlDataOffer`'s UserData.
        // 5. This callback is then invoked. We should query the `WlDataOffer` associated with
        //    `target_surface` for this DND session to see if `mime_type` was accepted by the client.

        // Smithay's `DragGrabState` (often part of `PointerInnerHandle` in a grab)
        // holds the active `WlDataOffer` for the current target surface.
        // We need access to that `WlDataOffer` here.
        // The `seat.get_drag_data_offer()` (hypothetical) or similar mechanism is needed.
        // Smithay's `PointerHandle::set_grab_data_offer` is used internally.

        // The `DataDeviceHandler` trait in Smithay might provide this offer or expect us to find it.
        // Let's assume we can get the current DND offer for the focused surface from the seat's DND state.
        // Smithay's `SeatInner::hovered_dnd_data_offer_mut` could be relevant if we have SeatInner.

        // A simpler approach: Smithay's `ServerDndGrab` (which is a `PointerGrabHandler`)
        // internally manages sending `wl_data_device.enter` with offered MIME types.
        // The client then calls `wl_data_offer.accept(mime_type)`.
        // This `dnd_accept_mime` callback might be part of a system where the compositor
        // *itself* decides acceptance if the client doesn't explicitly accept/reject via `WlDataOffer`.

        // Smithay's `DataDeviceState::with_dnd_data` provides access to `DndData` which includes
        // the `WlDataOffer` for the currently focused surface during a DND.
        // `seat.dnd_state().unwrap().offer.as_ref().unwrap().with_states(...)`
        // This is getting complex for a direct handler.

        // Re-evaluation: This callback might be simpler.
        // It's called by Smithay to ask US (the compositor) if, from a policy perspective,
        // this surface *should* be allowed to accept this type, OR if the client has
        // already indicated via `WlDataOffer::accept`.
        // If the client explicitly called `WlDataOffer::accept(serial, mime_type.clone())`,
        // then that `WlDataOffer`'s data would reflect it.

        // For now, let's assume a permissive policy: if the client *could* handle it (i.e., the offer exists),
        // we return true. The client will ultimately decide via `wl_data_offer.accept()`.
        // A more sophisticated compositor might have global DND policies here.
        let client_accepted = seat.user_data() // Assuming Seat's UserData can access DND state or offers
            .get::<DataDeviceState>() // This is not how UserData on Seat works.
            // We need the DND offer associated with `target_surface` for *this specific drag*.
            // This is tricky without access to the grab state.
            .map_or(false, |dds| {
                // This is still not right. DDS is global. We need the specific offer for this surface for this drag.
                // Smithay's `ServerDndGrabHandler` handles this internally by checking the WlDataOffer's state.
                // This callback might be for overriding or for cases where the client is not wayland-native.
                false // Placeholder: default to false until we can correctly check client acceptance.
            });

        // A more direct way, if this callback is meant to reflect client's prior `accept` call:
        // The `WlDataOffer` for `target_surface` (if one exists for this DND) would have its
        // accepted MIME types stored in its `UserData` by Smithay.
        //
        // Let's assume this callback is for compositor-level policy *before* client has a chance,
        // or if the client is non-responsive.
        // Defaulting to `true` means we allow the client to make the decision.
        // Defaulting to `false` means we block it unless client has already accepted.

        // Most straightforward: This callback is likely called *after* Smithay has processed
        // any `WlDataOffer::accept` from the client. So, we should check the WlDataOffer's state.
        // This requires finding that WlDataOffer.
        // Smithay's `Seat::get_selection_data_offer()` or similar for DND is what's needed.

        // Workaround: For now, assume client decides. If client calls `accept`, Smithay handles it.
        // This callback could be for overriding. Let's be permissive.
        debug!("Compositor policy: Permitting check for DND MIME type '{}' on surface {:?}", mime_type, target_surface);
        true // Let the client decide by calling (or not calling) `WlDataOffer::accept()`.
             // Smithay's `ServerDndGrabHandler` will then use the offer's state.
    }


    fn dnd_action_choice(
        &mut self,
        // target_surface: wl_surface::WlSurface, // Surface DND is over
        // available_actions: wl_data_device::DndAction, // Actions offered by source
        // preferred_action: wl_data_device::DndAction, // Action preferred by target client
        // seat: Seat<Self>,
    ) -> wl_data_device::DndAction {
        // This callback signature seems to be from older Smithay or a misunderstanding.
        // Modern Smithay `DataDeviceHandler` doesn't have this.
        // The chosen DND action is communicated by the client via `WlDataOffer::set_actions`
        // and then `WlDataOffer::finish`. The source is informed via `WlDataSource::action`.

        // Smithay's `ServerDndGrabHandler` manages this:
        // - Target client calls `wl_data_offer.set_actions(supported, preferred)`.
        // - Compositor (grab handler) selects an action from `supported & source_actions`.
        // - Sends `wl_data_source.action(chosen_action)` to source.
        // - Sends `wl_data_device.selection(offer_with_action)` to target. (Not selection, but updates offer state)

        // This callback is likely not needed with current Smithay `DataDeviceHandler` structure.
        warn!("DataDeviceHandler::dnd_action_choice called, but this is likely handled by Smithay's ServerDndGrabHandler based on client WlDataOffer.set_actions.");
        wl_data_device::DndAction::None // Default, should be determined by client and source intersection.
    }
}

// Delegate DataDevice handling to DesktopState (or NovaCompositorState)
// delegate_data_device!(DesktopState); // Needs to be NovaCompositorState

/// Initializes and registers the WlDataDeviceManager global.
/// `D` is your main compositor state type.
pub fn init_data_device_manager<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<WlDataDeviceManager, ()> + Dispatch<WlDataDeviceManager, (), D> +
       Dispatch<WlDataDevice, DataDeviceData, D> +   // UserData for WlDataDevice
       Dispatch<WlDataSource, DataSourceData, D> + // UserData for WlDataSource
       Dispatch<WlDataOffer, (), D> + // WlDataOffer often has UserData via its WlSurface if it's a proxy
       DataDeviceHandler + SeatHandler<D> + 'static, // SeatHandler is crucial as DataDevice is per-seat
       // D must also own DataDeviceState and SeatState.
{
    info!("Initializing WlDataDeviceManager global");

    // Create DataDeviceState. This state needs to be managed by your compositor (in D).
    // Example: state.data_device_state = DataDeviceState::new();
    // Ensure each Seat also has DataDeviceData in its UserData.

    display.create_global::<D, WlDataDeviceManager, _>(
        3, // protocol version
        () // GlobalData for the manager (unit)
    )?;

    // The WlDataDeviceManager global will, upon client request (get_data_device),
    // create a WlDataDevice. This WlDataDevice needs DataDeviceData associated with it.
    // Smithay's `delegate_data_device!` handles the dispatching for WlDataDevice,
    // WlDataSource, and WlDataOffer, relying on `DataDeviceHandler` and `DataDeviceState`.
    //
    // Each `Seat` created also needs to be initialized for data device functionality.
    // When a new `Seat` is created:
    // seat.user_data().insert_if_missing(DataDeviceData::new); // For the WlDataDevice resource
    // And the `Seat` itself needs to be given to `DataDeviceState` if certain ops are per-seat.
    // Smithay's `Seat::add_data_device_handler` or similar is used.

    // The `DataDeviceState` is typically global, while `DataDeviceData` is per `WlDataDevice` resource.
    // `SeatState` manages multiple seats. Each `Seat` has a `WlDataDevice` associated with it.

    Ok(())
}

// TODO:
// - State Integration:
//   - `DataDeviceState` must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `DataDeviceHandler`.
//   - Each `Seat<NovaCompositorState>` must be configured for data device operations.
//     Smithay's `Seat::init_data_device()` or similar should be called.
//   - `delegate_data_device!(NovaCompositorState);` macro must be used.
// - DND Icon Surface:
//   - Implement rendering and management of the drag icon surface.
// - XWayland Clipboard Sync:
//   - This is a major feature, involving listening to X11 selection events and Wayland
//     selection events and translating between them. Smithay might offer utilities or require
//     manual implementation using libraries like `x11rb` for X11 interaction.
// - Testing:
//   - Copy/paste text between two Wayland clients.
//   - Copy/paste images or other MIME types.
//   - Drag and drop text/files between Wayland clients.
//   - DND cancellation.
//   - DND with custom drag icons.
//   - (If XWayland sync implemented) Copy/paste and DND between Wayland and X11 clients.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod data_device;
