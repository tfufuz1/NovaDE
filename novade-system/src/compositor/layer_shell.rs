// This is novade-system/src/compositor/layer_shell.rs
// Specific logic for wlr-layer-shell clients.

use smithay::{
    desktop::{LayerSurface, Space, Output, layer_map_for_output, LayerSurfaceData},
    reexports::{
        wayland_protocols::wlr::layer_shell::v1::server::{
            zwlr_layer_shell_v1::{ZwlrLayerShellV1, Request as LayerShellRequest, Event as LayerShellEvent},
            zwlr_layer_surface_v1::{
                ZwlrLayerSurfaceV1, Request as LayerSurfaceRequest, Event as LayerSurfaceEvent,
                Layer, Anchor, KeyboardInteractivity
            },
        },
        wayland_server::{
            protocol::{wl_surface::WlSurface, wl_output::WlOutput},
            DisplayHandle, Client, Resource,
        },
    },
    utils::{Point, Size, Logical, Rectangle, Serial, SERIAL_COUNTER, Physical},
    wayland::shell::wlr_layer::WlrLayerShellState, // Smithay's state for layer shell
};
use tracing::{info, warn, debug};

use crate::compositor::state::DesktopState;

// This file contains functions called by the WlrLayerShellHandler trait methods
// (which are delegated in handlers.rs or implemented directly on DesktopState).

/// Handles the creation of a new layer surface.
/// This is typically called from `WlrLayerShellHandler::new_layer_surface`.
pub fn handle_new_layer_surface(
    state: &mut DesktopState,
    layer_surface_resource: ZwlrLayerSurfaceV1, // The Wayland resource for the layer surface
    output_resource: Option<WlOutput>,     // Optional client-provided output
    layer_kind: Layer,
    namespace: String,
) {
    let wl_surface = layer_surface_resource.wl_surface().clone();
    info!(
        surface = ?wl_surface.id(),
        output = ?output_resource.as_ref().map(|o| o.id()),
        ?layer_kind, %namespace,
        "Handling new layer surface request"
    );

    let space_guard = state.space.lock().unwrap();

    // Determine the target Smithay Output
    let target_smithay_output = output_resource
        .as_ref() // Keep the Option<&WlOutput>
        .and_then(|o_res| Output::from_resource(o_res))
        .or_else(|| {
            warn!("Client did not specify an output for layer surface. Attempting to use first available output.");
            space_guard.outputs().next().cloned()
        });

    let smithay_output = match target_smithay_output {
        Some(s_out) => s_out,
        None => {
            warn!(surface = ?wl_surface.id(), "No suitable output found for layer surface. Closing.");
            layer_surface_resource.send_event(LayerSurfaceEvent::Closed);
            // Make sure to destroy the object if it's not going to be used.
            // layer_surface_resource.destroy(); // Or post_error if appropriate.
            return;
        }
    };
    drop(space_guard); // Release lock

    // Let Smithay's WlrLayerShellState create and track the LayerSurface
    if let Err(err) = state.layer_shell_state.new_layer_surface(
        layer_surface_resource.clone(),
        Some(smithay_output),
        layer_kind,
        Some(namespace),
    ) {
        error!(surface = ?wl_surface.id(), "Failed to create Smithay LayerSurface: {}", err);
        // The protocol doesn't define an error for get_layer_surface failure.
        // Usually, the zwlr_layer_surface_v1 is created, and if something goes wrong,
        // a 'closed' event is sent on it. Smithay's state might handle this.
        // If not, we might need to manually send closed or destroy the resource.
        layer_surface_resource.send_event(LayerSurfaceEvent::Closed);
        return;
    }

    info!(surface = ?wl_surface.id(), "Successfully created and tracked Smithay LayerSurface.");
    // Smithay's WlrLayerShellState should send the initial configure.
}

/// Handles an AckConfigure request from a layer surface client.
pub fn handle_layer_ack_configure(
    state: &mut DesktopState,
    layer_surface_resource: &ZwlrLayerSurfaceV1,
    ack_serial: Serial,
) {
    let wl_surface = layer_surface_resource.wl_surface();
    debug!(surface = ?wl_surface.id(), serial = ?ack_serial, "LayerSurface: AckConfigure");

    if let Some(layer_surface_data) = wl_surface.data::<LayerSurfaceData>() {
        if layer_surface_data.ack_configure(ack_serial).is_ok() {
            info!(surface = ?wl_surface.id(), "LayerSurface AckConfigure successful.");

            let smithay_output = layer_surface_data.layer_surface().output();
            if let Some(output_handle) = state.space.lock().unwrap().outputs().find(|o| o == &smithay_output) {
                 layer_map_for_output(output_handle).arrange_layers();
                 // state.damage_output_by_name(&output_handle.name(), None); // Damage the output
            } else {
                warn!("Could not find output {} in space for layer surface ack_configure.", smithay_output.name());
            }
        } else {
            warn!(surface = ?wl_surface.id(), %ack_serial, "LayerSurface AckConfigure serial mismatch or other error.");
            layer_surface_resource.post_error(LayerSurfaceEvent::Closed {}, "AckConfigure serial mismatch".into());
        }
    } else {
        warn!(surface = ?wl_surface.id(), "AckConfigure for a layer surface without LayerSurfaceData.");
        layer_surface_resource.post_error(LayerSurfaceEvent::Closed {}, "Internal compositor error".into());
    }
}

/// Handles commit operations for layer surfaces.
pub fn handle_layer_surface_commit(
    state: &mut DesktopState,
    surface: &WlSurface,
) {
    if !surface.is_alive() { return; }

    if let Some(layer_surface_data) = surface.data::<LayerSurfaceData>() {
        let layer_surface = layer_surface_data.layer_surface();
        debug!(surface = ?surface.id(), "Commit for LayerSurface");

        // Smithay's WlrLayerShellState and LayerSurfaceData handle applying pending state
        // (size, anchor, margins, etc.) and sending new configures if needed.
        // The commit might have already been processed by Smithay's main commit_handler
        // via TraversalAction::call_handler for the LayerSurfaceData.

        let output_smithay = layer_surface.output();

        if let Some(output_handle) = state.space.lock().unwrap().outputs().find(|o| o == &output_smithay) {
             layer_map_for_output(output_handle).arrange_layers();
             // Mark the output for redraw. A more precise damage region could be calculated.
             // state.damage_output_by_name(&output_handle.name(), None);
        } else {
            warn!("Could not find output {} in space for layer surface commit.", output_smithay.name());
        }

        // If client changed size/state that needs a new configure, WlrLayerShellState should send it.
        // LayerSurfaceData::send_configure_if_needed might be relevant if called manually.
        // For now, assume Smithay handles this as part of its commit processing for the role.
    }
}
// Example of how DesktopState might provide damage_output_by_name if it's not directly on Smithay's Output
// This is already in the placeholder `layer_shell.rs` from a previous step, but shown here for context.
// impl DesktopState {
//     pub fn damage_output_by_name(&self, output_name: &str, damage: Option<Rectangle<i32, Logical>>) {
//         if let Some(output) = self.space.lock().unwrap().outputs().find(|o| o.name() == output_name) {
//             // Actual damage submission to renderer would happen here or via a render scheduler
//             info!("Damaging output: {} (Placeholder)", output_name);
//         }
//     }
// }
