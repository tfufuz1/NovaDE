// This is novade-system/src/compositor/layer_shell.rs
// Specific logic for wlr-layer-shell clients.

//! Implements the handling logic for the `wlr-layer-shell-unstable-v1` protocol.
//!
//! This module contains functions called by the `WlrLayerShellHandler`
//! implementation on `DesktopState`. These functions manage the lifecycle
//! of layer surfaces (e.g., panels, docks, notifications), their arrangement
//! on outputs, and responses to client requests. It leverages Smithay's
//! `LayerSurfaceData` and `layer_map_for_output` utilities.

use smithay::{
    desktop::{LayerSurface, Space, Output, layer_map_for_output, LayerSurfaceData},
    reexports::{
        wayland_protocols::wlr::layer_shell::v1::server::{
            // zwlr_layer_shell_v1::{ZwlrLayerShellV1, Request as LayerShellRequest, Event as LayerShellEvent}, // Manager not directly used here
            zwlr_layer_surface_v1::{
                ZwlrLayerSurfaceV1, // Request as LayerSurfaceRequest, // Not directly used here
                Event as LayerSurfaceEvent, Layer, // Anchor, KeyboardInteractivity (not directly used here)
            },
        },
        wayland_server::{
            protocol::{wl_surface::WlSurface, wl_output::WlOutput},
            // DisplayHandle, Client, // Not directly used in these helpers
            Resource,
        },
    },
    utils::{Serial}, // Point, Size, Logical, Rectangle, Physical (not directly used here)
    // wayland::shell::wlr_layer::WlrLayerShellState, // State is in DesktopState
};
use tracing::{info, warn, debug, error};

use crate::compositor::state::DesktopState;

/// Handles the creation of a new layer surface.
///
/// This function is typically called from `WlrLayerShellHandler::new_layer_surface`
/// on `DesktopState`. It determines the target output for the layer surface and
/// then uses Smithay's `WlrLayerShellState` to formally create and track the
/// `LayerSurface` object. Smithay's state will then send the initial configure event.
pub fn handle_new_layer_surface(
    state: &mut DesktopState,
    layer_surface_resource: ZwlrLayerSurfaceV1,
    output_resource: Option<WlOutput>,
    layer_kind: Layer,
    namespace: String,
) {
    let wl_surface = layer_surface_resource.wl_surface().clone();
    info!(
        surface = ?wl_surface.id(),
        output = ?output_resource.as_ref().map(|o| o.id()),
        ?layer_kind, %namespace,
        "Handling new layer surface request from client"
    );

    let space_guard = state.space.lock().unwrap();

    // Determine the target Smithay Output based on client hint or fallback.
    let target_smithay_output = output_resource
        .as_ref()
        .and_then(|o_res| Output::from_resource(o_res)) // Convert WlOutput resource to Smithay's Output
        .or_else(|| {
            warn!(surface = ?wl_surface.id(), "Client did not specify an output for layer surface. Attempting to use first available output.");
            space_guard.outputs().next().cloned() // Fallback to the first output in the space
        });

    let smithay_output_handle = match target_smithay_output {
        Some(s_out) => s_out,
        None => {
            warn!(surface = ?wl_surface.id(), "No suitable output found for layer surface. Closing the layer surface.");
            // Inform the client that the surface is closed as per protocol recommendation if setup fails.
            layer_surface_resource.send_event(LayerSurfaceEvent::Closed);
            // Smithay might automatically destroy the resource if its creation via WlrLayerShellState fails,
            // or if the client destroys it after receiving Closed.
            return;
        }
    };
    drop(space_guard); // Release lock on space

    // Delegate to Smithay's WlrLayerShellState to create and track the LayerSurface.
    // This will also typically send the first `configure` event to the client.
    if let Err(err) = state.layer_shell_state.new_layer_surface(
        layer_surface_resource.clone(), // The Wayland resource for the layer surface
        Some(smithay_output_handle),    // The Smithay Output handle it's assigned to
        layer_kind,                     // The requested layer (top, bottom, etc.)
        Some(namespace),                // The client-provided namespace
    ) {
        error!(surface = ?wl_surface.id(), "Failed to create Smithay LayerSurface via WlrLayerShellState: {}", err);
        // If Smithay's state fails to create/track it, ensure the client knows.
        if layer_surface_resource.is_alive() {
            layer_surface_resource.send_event(LayerSurfaceEvent::Closed);
        }
        return;
    }

    info!(surface = ?wl_surface.id(), "Successfully created and tracked Smithay LayerSurface.");
    // The initial configure and subsequent layout arrangement will be handled by Smithay's
    // WlrLayerShellState and calls to `layer_map_for_output().arrange_layers()`.
}

/// Handles an `ack_configure` request from a layer surface client.
///
/// This is called from `WlrLayerShellHandler::layer_surface_request` when the client
/// acknowledges a configuration event. It processes the acknowledgment using
/// Smithay's `LayerSurfaceData`, re-arranges layers on the output, and damages
/// the output to make changes visible.
pub fn handle_layer_ack_configure(
    state: &mut DesktopState,
    layer_surface_resource: &ZwlrLayerSurfaceV1,
    ack_serial: Serial,
) {
    let wl_surface = layer_surface_resource.wl_surface();
    debug!(surface = ?wl_surface.id(), serial = ?ack_serial, "LayerSurface: Received AckConfigure");

    // Retrieve Smithay's LayerSurfaceData associated with the wl_surface.
    if let Some(layer_surface_data) = wl_surface.data::<LayerSurfaceData>() {
        // Process the ack_configure using Smithay's helper.
        // This updates the committed state of the layer surface.
        if layer_surface_data.ack_configure(ack_serial).is_ok() {
            info!(surface = ?wl_surface.id(), "LayerSurface AckConfigure successful. Re-arranging layers.");

            // Get the output the layer surface is on.
            let smithay_output = layer_surface_data.layer_surface().output();
            // Find the corresponding Output handle in our space to call arrange_layers and damage.
            if let Some(output_handle) = state.space.lock().unwrap().outputs().find(|o| o == &smithay_output).cloned() {
                 // Re-calculate layout for all layer surfaces on this output.
                 layer_map_for_output(&output_handle).arrange_layers();
                 info!(surface = ?wl_surface.id(), output = %output_handle.name(), "Arranged layers and damaging output after ack_configure.");
                 // Damage the entire output to ensure changes are rendered.
                 output_handle.damage_whole();
            } else {
                warn!("Could not find output {} in space for layer surface ack_configure.", smithay_output.name());
            }
        } else {
            // Serial mismatch or other error in ack_configure. This is a protocol violation.
            warn!(surface = ?wl_surface.id(), %ack_serial, "LayerSurface AckConfigure serial mismatch or other error. Posting error to client.");
            layer_surface_resource.post_error(LayerSurfaceEvent::Closed {}, "AckConfigure serial mismatch".into());
        }
    } else {
        warn!(surface = ?wl_surface.id(), "AckConfigure for a layer surface without LayerSurfaceData. This should not happen.");
        // This indicates an internal compositor error or misconfiguration.
        if layer_surface_resource.is_alive() {
            layer_surface_resource.post_error(LayerSurfaceEvent::Closed {}, "Internal compositor error".into());
        }
    }
}

/// Handles commit operations for layer surfaces.
///
/// This is called from `CompositorHandler::commit` when a `wl_surface` with a
/// layer role is committed. It ensures that layers are re-arranged on the output
/// if the commit resulted in state changes, and damages the output.
pub fn handle_layer_surface_commit(
    state: &mut DesktopState,
    surface: &WlSurface,
) {
    if !surface.is_alive() { return; }

    // Check if the surface has LayerSurfaceData, indicating it's a layer surface.
    if let Some(layer_surface_data) = surface.data::<LayerSurfaceData>() {
        let layer_surface = layer_surface_data.layer_surface(); // Get the Smithay LayerSurface object
        debug!(surface = ?surface.id(), "Commit received for LayerSurface");

        // Smithay's WlrLayerShellState and LayerSurfaceData (via commit_handler called by CompositorHandler)
        // handle applying pending state (size, anchor, margins, etc.) from client requests
        // and sending new `configure` events if the server-side state changes (e.g., output size changes).
        // This function is mainly for post-commit actions like re-layout and damage.

        let output_smithay = layer_surface.output(); // Get the output this layer surface is on.

        // Find the output in our space to arrange layers and damage.
        if let Some(output_handle) = state.space.lock().unwrap().outputs().find(|o| o == &output_smithay).cloned() {
             // Re-calculate layout for all layer surfaces on this output.
             layer_map_for_output(&output_handle).arrange_layers();
             info!(surface = ?surface.id(), output = %output_handle.name(), "Arranged layers and damaging output after layer surface commit.");
             // Damage the entire output. More precise damage could be calculated if needed.
             output_handle.damage_whole();
        } else {
            warn!("Could not find output {} in space for layer surface commit of surface {:?}.", output_smithay.name(), surface.id());
        }
    }
}
