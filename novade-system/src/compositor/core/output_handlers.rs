// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

use smithay::{
    output::{Output, OutputHandler, OutputManagerState, Mode, PhysicalProperties},
    reexports::wayland_server::protocol::wl_output::WlOutput,
    utils::Point,
    desktop::Space,
};
use crate::compositor::core::state::DesktopState; // Changed from NovadeCompositorState

impl OutputHandler for DesktopState { // Changed from NovadeCompositorState
    fn output_state(&mut self) -> &mut OutputManagerState {
        &mut self.output_manager_state
    }

    fn new_output(&mut self, output: Output) {
        tracing::info!("New output detected: {}", output.name());
        
        // Simplified positioning: Winit backend creates only one output, map it at (0,0)
        let position = Point::from((0, 0));
        tracing::info!("Mapping new output {} at default position {:?}", output.name(), position);

        self.space.map_output(&output, position);
        
        // Smithay's Output objects (version 0.10+) are typically created with their global
        // already managed by the backend that creates them (e.g., DrmBackend, WinitBackend).
        // If an Output is passed to new_output, its global should have been handled.
        // We just need to store it and map it to our space.
        
        // Check if the output is already in the list to avoid duplicates if new_output is called multiple times
        // for the same output (e.g. by create_global and then some other path).
        // Output::name() should be unique.
        if !self.outputs.iter().any(|o| o.name() == output.name()) {
            self.outputs.push(output.clone()); // Clone the output for storing
            tracing::info!("Output {} added to DesktopState.outputs.", output.name());
        } else {
            tracing::warn!("Output {} was already present in DesktopState.outputs. Not adding again.", output.name());
        }

        // Trigger a refresh of all outputs in the space due to layout change.
        // self.space.damage_all_outputs(); // This method might not exist. Damage specific output or all relevant windows.
        // For simplicity, let's damage the newly mapped output.
        self.space.damage_output(&output, None, None);
        tracing::info!("Output {} mapped to space at {:?} and relevant areas damaged.", output.name(), position);
    }

    fn output_mode_updated(&mut self, output: &Output, new_mode: Mode) {
        tracing::info!("Output {} mode updated to: {:?}@{}mHz", output.name(), new_mode.size, new_mode.refresh);
        // The Output's internal state is updated by Smithay when its current_mode() is changed.
        // The Output object itself will send the necessary wl_output.mode event to clients.
        // We need to ensure our Space is aware of this change if it affects layout
        // and damage the output so it's redrawn.
        
        // Re-mapping the output in the space implicitly updates its geometry if the mode changed size.
        // If the position logic were more complex, we might need to recalculate it here.
        // For now, assuming position remains (0,0) or is handled by space.map_output if it re-evaluates.
        if let Some(current_pos) = self.space.output_geometry(output).map(|geo| geo.loc) {
             self.space.map_output(output, current_pos); // Re-map with current position to update size if needed
        } else {
            // This case should ideally not happen if the output was previously mapped.
            // If it does, map it at a default position.
            tracing::warn!("Output {} mode updated, but it was not found in the space. Re-mapping at (0,0).", output.name());
            self.space.map_output(output, (0,0).into());
        }
        
        self.space.damage_output(output, None, None); // Damage the entire output
        tracing::debug!("Output {} mode change processed, space updated and output damaged.", output.name());
    }

    fn output_destroyed(&mut self, destroyed_output: &Output) {
        tracing::info!("Output destroyed: {}", destroyed_output.name());
        self.space.unmap_output(destroyed_output);
        self.outputs.retain(|o| o.name() != destroyed_output.name());
        // The global for the output is automatically cleaned up by Smithay when the Output is dropped,
        // as the Global resource is typically owned by the Output object.
        
        // Trigger a refresh of all outputs due to layout change.
        // self.space.damage_all_outputs(); // This method might not exist. Damage relevant areas.
        // This might be too broad; consider damaging only areas affected by the output's removal.
        // For now, let's assume the space handles this implicitly or damage needs to be more targeted.
        // If other outputs' positions change, they need to be damaged.
        // For a single Winit output, this is less critical.
        tracing::info!("Output {} unmapped from space and removed from state. Consider damaging affected areas if layout changes.", destroyed_output.name());
    }
}
