use crate::compositor::state::DesktopState; // Assuming DesktopState is in crate::compositor::state
use crate::compositor::layer_shell::state::LayerSurfaceData; // Our LayerSurfaceData

use smithay::{
    desktop::{LayerSurface, space::SpaceElement}, // LayerSurface and SpaceElement for interaction with Smithay's desktop concepts
    reexports::wayland_server::{
        protocol::{wl_output::WlOutput, wl_surface::WlSurface},
        Client, DisplayHandle,
    },
    wayland::{
        compositor::{get_role, CompositorHandler}, // get_role might be useful
        shell::wlr_layer::{LayerShellHandler, LayerShellRequest, LayerSurfaceData as SmithayLayerSurfaceData}, // Smithay's LayerSurfaceData for new_layer_surface
    },
    utils::SERIAL_COUNTER, // For generating serials for configure events
};
use tracing::{info, debug, warn};

// This will be part of DesktopState, defined elsewhere.
// For now, we assume DesktopState has:
// pub layer_surfaces: Vec<LayerSurfaceData>,
// pub space: Space<WindowElement>, // Or some way to manage elements and outputs
// And methods like:
// fn find_output_for_surface(&self, surface: &WlSurface) -> Option<&Output>;
// fn reconfigure_output_usable_area(&mut self, output: &Output);
// fn send_frames(&mut self);

impl LayerShellHandler for DesktopState {
    fn new_layer_surface(
        &mut self,
        dh: &DisplayHandle,
        client: Client, // Client is not directly used here but available
        surface: LayerSurface, // This is smithay::desktop::LayerSurface
        data: &SmithayLayerSurfaceData, // Smithay's internal data for the layer surface
    ) {
        info!(surface = ?surface.wl_surface(), "New layer surface created");

        // Set a role for the wl_surface, if not already set.
        // This helps in identifying the surface's purpose.
        if get_role(surface.wl_surface()).is_none() {
            // This role assignment might be handled by Smithay internally when creating LayerSurface
            // but it's good practice to be aware of it.
            // For layer surfaces, the role is implicitly "wl_layer_surface".
        }

        let mut layer_data = LayerSurfaceData::new(surface.clone()); // Clone the Arc<LayerSurfaceInner>

        // The client might request a specific output. LayerSurface::output() gives Option<WlOutput>
        // If no output is set by client, compositor can choose (e.g. primary or cursor output).
        // For now, we assume an output is found or client handles it.
        // Determine and store the assigned output name
        let mut assigned_output_name_str = None;
        if let Some(wl_out) = surface.output() {
            if let Some(s_out) = self.outputs().find(|o| o.wl_object().as_ref() == Some(&wl_out)) {
                assigned_output_name_str = Some(s_out.name());
            } else {
                warn!(surface = ?surface.wl_surface(), output = ?wl_out.id(), "Client requested layer surface on WlOutput not found by compositor. Assigning to primary/default.");
                // Fallback: Assign to primary or first available if requested output not found
                if let Some(primary_out) = self.display_manager.lock().unwrap().get_primary_output() {
                    assigned_output_name_str = Some(primary_out.name.clone());
                } else if let Some(first_out) = self.outputs().next() {
                    assigned_output_name_str = Some(first_out.name());
                }
            }
        } else {
            // No output specified by client, assign to primary or first available
            if let Some(primary_out) = self.display_manager.lock().unwrap().get_primary_output() {
                assigned_output_name_str = Some(primary_out.name.clone());
                debug!(surface = ?surface.wl_surface(), output_name = ?assigned_output_name_str, "No output specified by client, assigned to primary output.");
            } else if let Some(first_out) = self.outputs().next().cloned() { // Clone to get owned name
                assigned_output_name_str = Some(first_out.name());
                debug!(surface = ?surface.wl_surface(), output_name = ?assigned_output_name_str, "No output specified by client, no primary, assigned to first available output.");
            } else {
                warn!(surface = ?surface.wl_surface(), "No output available in compositor to assign layer surface.");
            }
        }

        if assigned_output_name_str.is_none() {
            warn!(surface = ?surface.wl_surface(), "Layer surface could not be assigned to any output. It might not be displayed.");
            // Surface will be tracked but not displayed until an output becomes available and it's reassigned.
        }
        layer_data.assigned_output_name = assigned_output_name_str.clone();

        // Initial positioning and sizing will happen on the first commit typically.
        // However, we might need to send an initial configure.
        // Smithay's LayerSurface handles sending initial configure based on its state.
        // The size (0,0) means the client can choose its initial size.
        surface.send_configure();
        debug!(surface = ?surface.wl_surface(), "Sent initial configure for layer surface");

        self.layer_surfaces.push(layer_data);
        // If it was assigned to an output, arrange layers on that output now.
        if let Some(name) = assigned_output_name_str {
            if let Some(output_s) = self.outputs().find(|o| o.name() == name).cloned() {
                 self.arrange_layers_on_output(output_s, dh);
            }
        }
        self.ask_for_render(); // Mark that a render is needed
    }

    fn layer_surface_request(
        &mut self,
        dh: &DisplayHandle,
        client: Client,
        surface: LayerSurface,
        request: LayerShellRequest, // e.g. SetLayer, SetAnchor, SetExclusiveZone etc.
        data: &SmithayLayerSurfaceData,
    ) {
        info!(surface = ?surface.wl_surface(), ?request, "Layer surface request");
        // Smithay's LayerSurface automatically updates its pending state upon receiving these requests.
        // The changes are then applied on commit.
        // We don't need to manually handle each request here if using LayerSurface's internal state management.
        // The `layer_shell_committed` hook is where we react to the applied changes.

        // Example of direct handling if needed (though LayerSurface does this):
        // match request {
        //     LayerShellRequest::SetSize { width, height } => { ... }
        //     LayerShellRequest::SetAnchor { anchor } => { ... }
        //     ...
        // }
        // After LayerSurface processes the request and updates its state, it will need a configure.
        // Typically, this leads to a commit from the client, then we process in layer_shell_committed.
    }

    fn layer_shell_committed(
        &mut self,
        dh: &DisplayHandle,
        surface: &LayerSurface, // Note: this is &LayerSurface, not &WlSurface
    ) {
        debug!(surface = ?surface.wl_surface(), "Layer shell committed for surface");

        let wl_surface = surface.wl_surface();
        if let Some(layer_data) = self.layer_surfaces.iter_mut().find(|s| s.wl_surface() == wl_surface) {
            let old_exclusive_zone = layer_data.exclusive_zone;
            let old_layer = layer_data.current_layer;
            let old_anchors = layer_data.anchors;
            let old_margins = layer_data.margins;
            let old_desired_size = layer_data.desired_size;

            // Update our LayerSurfaceData from the LayerSurface's current (committed) state
            layer_data.update_from_pending(); // This now reads from surface.current_state()

            let current_state = surface.current_state();
            let mut needs_reconfigure = false;

            if old_layer != current_state.layer ||
               old_anchors != current_state.anchor ||
               old_margins != current_state.margin ||
               old_desired_size != current_state.size ||
               layer_data.exclusive_zone_changed // This is set in update_from_pending
            {
                needs_reconfigure = true;
            }

            // If exclusive zone changed, we need to recalculate usable area for the affected output(s)
            if layer_data.exclusive_zone_changed {
                info!(surface = ?wl_surface, zone = layer_data.exclusive_zone, "Exclusive zone changed");
                if let Some(output_name) = layer_data.assigned_output_name.as_ref() {
                    if let Some(output_s) = self.outputs().find(|o| o.name() == *output_name).cloned() {
                        self.recalculate_output_usable_area(output_s.clone(), dh);
                        self.arrange_layers_on_output(output_s.clone(), dh);
                        self.arrange_windows_on_output(output_s.clone(), dh);
                    } else {
                        warn!(surface = ?wl_surface, output_name = ?output_name, "Assigned output for layer surface not found for exclusive zone update.");
                    }
                } else {
                    warn!(surface = ?wl_surface, "Layer surface with exclusive zone has no assigned output name.");
                }
                layer_data.exclusive_zone_changed = false; // Reset the flag
            } else if needs_reconfigure {
                if let Some(output_name) = layer_data.assigned_output_name.as_ref() {
                    if let Some(output_s) = self.outputs().find(|o| o.name() == *output_name).cloned() {
                        self.arrange_layers_on_output(output_s.clone(), dh);
                    } else {
                        warn!(surface = ?wl_surface, output_name = ?output_name, "Assigned output for layer surface not found for reconfigure.");
                    }
                } else {
                     warn!(surface = ?wl_surface, "Layer surface needing reconfigure has no assigned output name.");
                }
            }


            // The LayerSurface itself will send a .configure event to the client
            // with the new size and state ID after this commit.
            // We must call `surface.send_configure()` to finalize the configuration.
            // This also implies the client must ack_configure.
            surface.send_configure();
            debug!(surface = ?wl_surface, "Sent configure for layer surface post-commit");

        } else {
            warn!(surface = ?wl_surface, "Committed hook called for untracked layer surface.");
        }
        self.ask_for_render(); // Mark that a render is needed
    }

    fn layer_surface_destroyed(
        &mut self,
        dh: &DisplayHandle,
        surface: LayerSurface, // This is smithay::desktop::LayerSurface
    ) {
        info!(surface = ?surface.wl_surface(), "Layer surface destroyed");
        let wl_surface = surface.wl_surface(); // Get &WlSurface for comparison

        let mut reconfigure_output_name : Option<String> = None;

        if let Some(index) = self.layer_surfaces.iter().position(|s| s.wl_surface() == wl_surface) {
            let removed_ls_data = self.layer_surfaces.remove(index);
            if removed_ls_data.has_exclusive_zone() {
                reconfigure_output_name = removed_ls_data.assigned_output_name;
            }
        } else {
            warn!(surface = ?wl_surface, "Destroyed event for an untracked layer surface.");
            return;
        }

        if let Some(output_name_to_reconfigure) = reconfigure_output_name {
            if let Some(output_s) = self.outputs().find(|o| o.name() == output_name_to_reconfigure).cloned() {
                self.recalculate_output_usable_area(output_s.clone(), dh);
                self.arrange_layers_on_output(output_s.clone(), dh);
                self.arrange_windows_on_output(output_s.clone(), dh);
            } else {
                 warn!(surface = ?wl_surface, output_name = ?output_name_to_reconfigure, "Assigned output for destroyed layer surface not found for exclusive zone update.");
            }
        }
        self.ask_for_render(); // Mark that a render is needed
    }
}

// These are placeholder methods that would be part of DesktopState
// and need a proper implementation.
impl DesktopState {
    // Placeholder for finding an output. Smithay's Space might handle this.
    // fn find_output_for_surface(&self, surface: &WlSurface) -> Option<&Output> {
    //     // Complex logic to map surface to an output, potentially via cursor position or focus
    //     self.space.outputs().find(|o| {
    //         // Check if surface is on this output
    //         // This is non-trivial. LayerSurface has an `output()` method.
    //         false // Placeholder
    //     })
    // }

    // Placeholder for reconfiguring all outputs or specific ones
    // This would involve recalculating usable areas and potentially re-tiling windows.
    // fn reconfigure_all_outputs(&mut self, dh: &DisplayHandle) {
    //     for output in self.space.outputs().cloned().collect::<Vec<_>>() {
    //         self.recalculate_output_usable_area(&output, dh);
    //         // Then arrange windows, layers etc. on this output
    //     }
    //     info!("Reconfigured all outputs (placeholder)");
    // }

    // Placeholder for recalculating usable area of a single output
    // fn recalculate_output_usable_area(&mut self, output: &Output, dh: &DisplayHandle) {
    //     // Actual implementation in state.rs example was more detailed.
    //     // Here we'd call that logic.
    //     // output.state_mut().recalculate_usable_area(&self.layer_surfaces);
    //     info!(output = ?output.name(), "Recalculated usable area for output (placeholder)");
    //     // This should then likely trigger a re-layout of xdg-surfaces on this output.
    // }

    // Placeholder: Marks that the compositor should render the next frame.
    // fn ask_for_render(&self) {
    //     // In a real compositor, this would schedule a repaint.
    //     // For example, by signaling an event loop or calling a rendering function.
    // }
}

// Note on Smithay's LayerSurface:
// - It handles the zwlr_layer_surface_v1 protocol interactions internally.
// - Requests like SetAnchor, SetLayer, etc., update its *pending_state*.
// - When the client commits the wl_surface associated with the LayerSurface,
//   the pending state is applied to become the *current_state*.
// - The `layer_shell_committed` hook is called *after* this state transition.
// - The compositor's main job in `layer_shell_committed` is to:
//   1. Read the `current_state` from the `LayerSurface`.
//   2. Update its own layout data (like our `LayerSurfaceData`).
//   3. Perform layout calculations (positioning, sizing).
//   4. If the size/state changed, send a `configure` event back to the client using `LayerSurface::send_configure()`.
//      This tells the client its new size and state ID.
//   5. The client must then `ack_configure`. Smithay's `LayerSurface` tracks this.
//
// The `CompositorHandler::commit` is still relevant for the underlying `wl_surface`
// to handle buffer attachment, damage tracking, etc. `layer_shell_committed` is specific
// to the layer shell state changes.
//
// For `DesktopState` to fully support this, it will need:
// - A list of `LayerSurfaceData`.
// - A way to manage outputs and their geometries/usable areas. Smithay's `Space` is often used.
// - Logic to arrange/sort layer surfaces based on layer type and z_index.
// - Integration with the rendering pipeline.
// - Methods like `recalculate_output_usable_area`, `arrange_layers_on_output`, `arrange_windows_on_output`
//   which are currently placeholders in the example DesktopState impl.
// - `outputs()` method to iterate over known smithay::output::Output instances.
// - `ask_for_render()` to signal the main loop.
// These would typically be part of the main compositor logic in `DesktopState`.
// The current implementation assumes these methods/fields will be added to `DesktopState`.
//
// The `layer_surface_request` handler is mostly for observation or if we need to
// deny/modify requests *before* they hit LayerSurface's pending state, but typically
// LayerSurface handles this well.
//
// `z_index` handling within a layer: The `zwlr_layer_shell_v1` protocol itself
// doesn't have a `z_index` request. This would need a custom protocol extension
// or be managed by the compositor based on other criteria (e.g., creation order).
// The current `LayerSurfaceData` has a `z_index` field, but no mechanism to update it from client.
// Sorting will first be by Layer enum, then potentially by this z_index if implemented.
```
