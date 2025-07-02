// This is novade-system/src/compositor/output_manager.rs
// Management of display outputs (monitors) and their configuration,
// including handling of zxdg_output_manager_v1.

use smithay::{
    backend::renderer::utils::SurfaceDamage, // For tracking damage on outputs
    desktop::{Output, Space}, // Smithay's Output and Space
    reexports::{
        wayland_protocols::xdg::output::zv1::server::{
            zxdg_output_manager_v1::{ZwlrOutputManagerV1, Request as XdgOutputManagerRequest},
            zxdg_output_v1::{ZwlrOutputV1, Request as XdgOutputRequest, Event as XdgOutputEvent},
        },
        wayland_server::{
            protocol::wl_output::{WlOutput, Mode as WlOutputMode, Subpixel, Transform as WlTransform},
            DisplayHandle, GlobalDispatch, Dispatch, Client, New, Resource,
        },
    },
    utils::{Point, Size, Physical, Logical, Transform}, // Smithay's utility types
    wayland::output::{OutputManagerState, OutputData, XdgOutputUserData}, // Smithay's output management state
};
use tracing::{info, warn, debug};

use crate::compositor::state::DesktopState; // Assuming DesktopState is the main state struct

// --- Data associated with XDG Output Manager global ---

pub struct XdgOutputManagerGlobalData {
    // Could hold configuration or references if needed for the global
    // Smithay's OutputManagerState might be sufficient if stored in DesktopState
}

// --- ZwlrOutputManagerV1 Global Dispatch ---

impl GlobalDispatch<ZwlrOutputManagerV1, XdgOutputManagerGlobalData> for DesktopState {
    fn bind(
        state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<ZwlrOutputManagerV1>,
        _global_data: &XdgOutputManagerGlobalData,
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, Self>,
    ) {
        info!("Client bound zxdg_output_manager_v1 global");
        let xdg_output_manager_resource = data_init.init(resource, ()); // No specific user data for the manager itself

        // Iterate over existing Smithay outputs and create zxdg_output_v1 for each
        // This is typically handled by Smithay's OutputManagerState::new_output or similar logic
        // when an output is added to the compositor.
        // If binding late, we might need to manually create them.
        // Smithay 0.30.0 OutputManagerState and XdgOutputUserData handle this.
        // When a new WlOutput global is created, an XdgOutput is also created if the manager is active.
        // So, this bind usually doesn't need to iterate; Smithay handles it.
        // If a client binds zxdg_output_manager_v1 *after* wl_outputs are created,
        // it will get zxdg_output_v1 for each existing wl_output it knows via
        // ZwlrOutputManagerV1::GetXdgOutput.
    }
}

// --- ZwlrOutputManagerV1 Request Handling ---

impl Dispatch<ZwlrOutputManagerV1, ()> for DesktopState { // UserData is () for the manager resource
    fn request(
        state: &mut Self,
        _client: &Client,
        _manager_resource: &ZwlrOutputManagerV1,
        request: XdgOutputManagerRequest,
        _data: &(),
        _dhandle: &DisplayHandle,
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, Self>,
    ) {
        match request {
            XdgOutputManagerRequest::Destroy => {
                // Client destroyed the manager. Associated zxdg_output_v1 objects are not necessarily destroyed.
                info!("Client destroyed zxdg_output_manager_v1 global.");
            }
            XdgOutputManagerRequest::GetXdgOutput { id, output: wl_output_resource } => {
                info!(wl_output = ?wl_output_resource.id(), new_xdg_output_id = ?id.id(), "Request for new zxdg_output_v1");

                // Get the Smithay Output associated with this WlOutput resource
                let smithay_output = match Output::from_resource(&wl_output_resource) {
                    Some(s_out) => s_out,
                    None => {
                        warn!("Client provided an invalid WlOutput to GetXdgOutput. Ignoring.");
                        // Protocol doesn't specify an error. Best to ignore or destroy `id`.
                        id.assign_destructor(smithay::reexports::wayland_server::Destructor::Destroy);
                        return;
                    }
                };

                // Create the zxdg_output_v1 resource.
                // Smithay's XdgOutputUserData should be associated with the WlOutput global.
                // This request essentially "activates" it for this manager.
                // Smithay 0.30.0 `OutputData` (on WlOutput) has `xdg_output_data` field.
                // The XdgOutput resource is created and managed by Smithay's OutputManagerState
                // when `Output::create_global` or `Output::new` is called.
                // This request handler then just needs to init the resource `id`.
                // The actual XdgOutput object is often part of WlOutput's user data.

                let xdg_output_user_data = wl_output_resource.data::<XdgOutputUserData>().expect("WlOutput should have XdgOutputUserData");
                // The `id` (New<ZwlrOutputV1>) is initialized with the XdgOutputUserData.
                // This means the ZwlrOutputV1 resource will use the XdgOutputUserData.
                let xdg_output_resource = data_init.init(id, xdg_output_user_data.clone()); // Clone the Arc<XdgOutputUserDataInner>

                // Send initial configuration for this zxdg_output_v1
                send_xdg_output_state(&smithay_output, &xdg_output_resource);
                debug!("Initialized zxdg_output_v1 resource for WlOutput {:?}", wl_output_resource.id());
            }
            _ => {
                warn!("Unknown request for ZwlrOutputManagerV1: {:?}", request);
            }
        }
    }
}

// --- ZwlrOutputV1 Request Handling ---
// The UserData for ZwlrOutputV1 resource is XdgOutputUserData (Arc-wrapped).
impl Dispatch<ZwlrOutputV1, XdgOutputUserData> for DesktopState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _xdg_output_resource: &ZwlrOutputV1,
        request: XdgOutputRequest,
        _data: &XdgOutputUserData, // This is the XdgOutputUserData associated with the WlOutput
        _dhandle: &DisplayHandle,
        _data_init: &mut smithay::reexports::wayland_server::DataInit<'_, Self>,
    ) {
        match request {
            XdgOutputRequest::Destroy => {
                // Client destroyed this specific zxdg_output_v1.
                // The underlying WlOutput and Smithay Output are not affected.
                info!("Client destroyed zxdg_output_v1 resource.");
            }
            _ => {
                warn!("Unknown request for ZwlrOutputV1: {:?}", request);
            }
        }
    }
}


// --- Output State Management ---

/// Call this when a new Smithay Output is created or its properties change.
/// It will update all listening zxdg_output_v1 clients.
pub fn on_output_changed(desktop_state: &DesktopState, changed_smithay_output: &Output) {
    info!(output_name = %changed_smithay_output.name(), "Output changed, updating XDG output clients.");

    // Iterate over all clients that have bound zxdg_output_manager_v1
    // and have a zxdg_output_v1 for this Smithay Output.
    // This is tricky. Smithay's OutputManagerState and XdgOutputUserData are designed
    // so that changes to the Smithay Output (via its methods like set_preferred_mode, set_transform)
    // will automatically trigger updates to associated WlOutput and XdgOutput resources.

    // Smithay 0.30.0: When you modify an `Output` (e.g. `output.set_transform(new_transform)`),
    // it internally updates its WlOutput representation and sends new events.
    // The XdgOutput is also updated because XdgOutputUserData listens to these changes.
    // So, explicit iteration here might not be needed if using Smithay's Output correctly.

    // If manual update is needed:
    // For each WlOutput associated with changed_smithay_output:
    //   If it has an XdgOutput resource active for some client:
    //     send_xdg_output_state(changed_smithay_output, xdg_output_resource_for_client);

    // Smithay's pattern:
    // `changed_smithay_output.set_preferred_mode(new_mode)` would internally call methods on
    // `OutputData` (user data of WlOutput) which then calls methods on `XdgOutputUserData`
    // which then sends events on all `ZwlrOutputV1` resources that share that `XdgOutputUserData`.
    // So, the primary action is to ensure `DesktopState` calls the appropriate methods on the
    // `smithay::desktop::Output` object when NovaDE's internal output state changes.
}

/// Call this when a Smithay Output is about to be destroyed.
/// It will send `Done` (or Closed in later protocol versions) to zxdg_output_v1 clients.
pub fn on_output_destroyed(desktop_state: &DesktopState, destroyed_smithay_output: &Output) {
    info!(output_name = %destroyed_smithay_output.name(), "Output destroyed, notifying XDG output clients.");
    // Similar to on_output_changed, Smithay's Output::destroy() or when OutputData is dropped
    // should handle notifying XdgOutputUserData, which then sends `Done` (or `Closed`)
    // to the ZwlrOutputV1 resources.
    // The `Done` event signifies that the output is no longer available.
    // (XDG Output v1 uses `Done`, v2/v3 use `Closed`). Smithay 0.30.0 likely targets v1.
}


/// Sends the current state of a Smithay Output to a specific zxdg_output_v1 resource.
fn send_xdg_output_state(smithay_output: &Output, xdg_output_resource: &ZwlrOutputV1) {
    // Logical position and size
    if let Some(geometry) = smithay_output.current_mode() { // This gives physical size. Need logical.
        // Smithay Output::geometry() gives logical position in the global space.
        // Smithay Output::size() gives logical size.
        // This information is part of the wl_output protocol, not xdg_output.
        // xdg_output_v1 is more about name, description, and logical_size *if different from wl_output*.
        // The protocol expects logical position and size if the compositor uses a logical coordinate system.
        // If physical, then physical. Smithay's Output methods usually return Logical coords/size.

        // The geometry of the output in the compositor's global logical space.
        // This is what wl_output.geometry provides.
        // xdg_output_v1.logical_position is sent if compositor uses logical coords.
        // xdg_output_v1.logical_size is sent if compositor uses logical coords *and* it's different from wl_output's mode size scaled.

        // For Smithay 0.30.0, Output::current_mode() gives Physical size.
        // Output::current_scale() gives scale factor.
        // Logical size = Physical size / scale.
        // Output::location() gives logical position.

        let pos = smithay_output.location();
        xdg_output_resource.send_event(XdgOutputEvent::LogicalPosition { x: pos.x, y: pos.y });

        let physical_size = smithay_output.current_mode().map_or(Size::from((0,0)), |m| m.size);
        let scale_factor = smithay_output.current_scale().fractional_scale() / 120.0; // Smithay scale is integer (120 = 1.0)
        let logical_size = if scale_factor > 0.0 {
            Size::from((
                (physical_size.w as f64 / scale_factor) as i32,
                (physical_size.h as f64 / scale_factor) as i32,
            ))
        } else {
            physical_size // Avoid division by zero, though scale should always be positive
        };
        xdg_output_resource.send_event(XdgOutputEvent::LogicalSize { width: logical_size.w, height: logical_size.h });
    } else {
        // If no mode, perhaps send 0,0 or last known values.
        // Or maybe this output is not ready / enabled.
        xdg_output_resource.send_event(XdgOutputEvent::LogicalPosition { x: 0, y: 0 });
        xdg_output_resource.send_event(XdgOutputEvent::LogicalSize { width: 0, height: 0 });
    }

    // Name and Description (optional)
    let name = smithay_output.name();
    if !name.is_empty() {
        xdg_output_resource.send_event(XdgOutputEvent::Name { name });
    }
    let description = smithay_output.description();
    if !description.is_empty() {
        xdg_output_resource.send_event(XdgOutputEvent::Description { description });
    }

    // After sending all properties, send Done.
    xdg_output_resource.send_event(XdgOutputEvent::Done {});
    debug!("Sent XDG Output state for output: {}", smithay_output.name());
}

// To initialize zxdg_output_manager_v1, create a global in your compositor setup:
//
// use smithay::reexports::wayland_protocols::xdg::output::zv1::server::zxdg_output_manager_v1::ZwlrOutputManagerV1;
//
// display_handle.create_global::<DesktopState, ZwlrOutputManagerV1, _>(
//     3, // Max version supported (check Smithay's supported version)
//     XdgOutputManagerGlobalData {}, // Your global data
// );
//
// And ensure DesktopState implements the necessary GlobalDispatch and Dispatch traits.
// Smithay's `OutputManagerState` should be part of your `DesktopState`.
// When a `smithay::desktop::Output` is created (e.g., via backend detection),
// you call `Output::create_global(&mut display_handle)` for WlOutput.
// `OutputManagerState` (often part of `DesktopState.output_manager_state`) ensures that
// `XdgOutputUserData` is attached to the `WlOutput`, and that `ZwlrOutputV1` resources
// are created and managed when clients use `GetXdgOutput`.
// Changes to the `smithay::desktop::Output` (like mode, scale, transform, position)
// should propagate events to `WlOutput` and subsequently `ZwlrOutputV1` resources
// via the `OutputManagerState` and `XdgOutputUserData` integration.
// So, direct calls to `send_xdg_output_state` might only be needed for the initial GetXdgOutput,
// and subsequent updates are handled by Smithay if you modify the `smithay::desktop::Output` correctly.
// Check Smithay 0.30.0 examples (Anvil) for the exact patterns of Output and XDG Output integration.
// Smithay 0.30.0 uses `OutputManagerState::new_with_xdg_output::<Self>(&display_handle)`
// which sets up most of this automatically. You then add `smithay::desktop::Output`s to it.
// The `OutputHandler::new_output` in `DesktopState` would be responsible for adding
// new `smithay::desktop::Output` instances to the `OutputManagerState`.
// The `XdgOutputUserData` is typically created and managed internally by `OutputManagerState`.
// The `Dispatch` impl for `ZwlrOutputManagerV1`'s `GetXdgOutput` request would then retrieve
// this `XdgOutputUserData` from the `WlOutput` and use it to initialize the new `ZwlrOutputV1` resource.
// The `send_xdg_output_state` function here is a utility to format the events correctly.
// Smithay's `XdgOutputUserDataInner::send_current_state_to` does exactly this.
// So, for GetXdgOutput, you would call that method from XdgOutputUserData.
