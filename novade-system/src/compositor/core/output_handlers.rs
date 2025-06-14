// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

use smithay::{
    output::{Output, OutputHandler, OutputManagerState, Mode, PhysicalProperties},
    reexports::wayland_server::protocol::wl_output::WlOutput,
    utils::Point,
    desktop::Space,
};
// Corrected: NovadeCompositorState should be DesktopState
use crate::compositor::core::state::DesktopState;

impl OutputHandler for DesktopState {
    fn output_state(&mut self) -> &mut OutputManagerState {
        &mut self.output_manager_state
    }

    fn new_output(&mut self, output: Output) {
        tracing::info!("New output detected: {}", output.name());
        
        let mut current_max_x = 0;
        for existing_output_element in self.space.outputs() { // Iterates over Output and its location in Space
            // existing_output_element is &Output, not OutputWithLocation.
            // We need its geometry in the space.
            if let Some(geom) = self.space.output_geometry(existing_output_element) {
                if geom.loc.x + geom.size.w > current_max_x {
                    current_max_x = geom.loc.x + geom.size.w;
                }
            }
        }
        let position = Point::from((current_max_x, 0));
        tracing::info!("Calculated position for new output {}: {:?}", output.name(), position);

        self.space.map_output(&output, position);
        
        // Smithay's Output objects (version 0.10+) are typically created with their global
        // already managed by the backend that creates them (e.g., DrmBackend, WinitBackend).
        // If an Output is passed to new_output, its global should have been handled.
        // We just need to store it and map it to our space.
        
        self.outputs.push(output);
        // Trigger a refresh of all outputs in the space due to layout change.
        self.space.damage_all_outputs(); 
        tracing::info!("Output {} mapped to space at {:?} and added to state.", output.name(), position);
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
        self.space.damage_all_outputs();
        tracing::info!("Output {} unmapped from space and removed from state.", destroyed_output.name());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::core::state::DesktopState; // Assuming NovadeCompositorState is DesktopState
    use smithay::{
        reexports::calloop::EventLoop,
        reexports::wayland_server::Display,
        output::{Output, Mode, PhysicalProperties, Scale},
        utils::{Point, Transform},
    };
    use std::sync::Arc;

    // Helper to create a DesktopState for testing.
    // This is a simplified setup and might need adjustments based on DesktopState's evolution.
    fn create_test_desktop_state() -> DesktopState {
        let event_loop: EventLoop<DesktopState> = EventLoop::try_new().expect("Failed to create event loop for test");
        let display: Display<DesktopState> = Display::new().expect("Failed to create display for test");
        DesktopState::new(event_loop.handle(), display.handle()).expect("Failed to create DesktopState for test")
    }

    // Helper to create a dummy Smithay Output for testing.
    fn create_dummy_output(name: &str, mode: Mode) -> Output {
        let physical_props = PhysicalProperties {
            size: (345, 194).into(), // Example physical size in mm
            subpixel: smithay::output::Subpixel::Unknown,
            make: "TestMake".to_string(),
            model: "TestModel".to_string(),
        };
        let output = Output::new(name.to_string(), physical_props, None);
        output.add_mode(mode);
        output.set_preferred_mode(mode); // Set preferred before current
        assert!(output.set_current_mode(mode), "Failed to set current mode on dummy output");
        output
    }

    fn get_default_mode() -> Mode {
        Mode { size: (1920, 1080).into(), refresh: 60000 / 1000 } // 60Hz
    }

    #[test]
    fn test_new_output_adds_to_state_and_space() {
        let mut state = create_test_desktop_state();
        let output_mode = get_default_mode();
        let dummy_output = create_dummy_output("test-output-1", output_mode);

        assert_eq!(state.outputs.len(), 0, "Outputs list should be empty initially");
        assert!(state.space.outputs().next().is_none(), "Space should have no outputs initially");

        state.new_output(dummy_output.clone()); // Clone because new_output takes ownership

        assert_eq!(state.outputs.len(), 1, "Output should be added to the state's outputs list");
        assert_eq!(state.outputs[0].name(), "test-output-1");

        let space_outputs: Vec<_> = state.space.outputs().collect();
        assert_eq!(space_outputs.len(), 1, "Output should be mapped to the space");
        assert_eq!(space_outputs[0].name(), "test-output-1");

        // Check if it's positioned (default logic positions first output at (0,0))
        if let Some(geom) = state.space.output_geometry(&dummy_output) {
            assert_eq!(geom.loc, Point::from((0,0)), "First output should be at (0,0)");
        } else {
            panic!("Output not found in space after new_output");
        }
    }

    #[test]
    fn test_new_output_positions_multiple_outputs_side_by_side() {
        let mut state = create_test_desktop_state();
        let output_mode1 = Mode { size: (800, 600).into(), refresh: 60000/1000 };
        let output_mode2 = Mode { size: (1024, 768).into(), refresh: 60000/1000 };

        let output1 = create_dummy_output("output1", output_mode1);
        let output2 = create_dummy_output("output2", output_mode2);

        state.new_output(output1.clone());
        state.new_output(output2.clone());

        assert_eq!(state.outputs.len(), 2);
        if let Some(geom1) = state.space.output_geometry(&output1) {
            assert_eq!(geom1.loc, Point::from((0,0)));
            assert_eq!(geom1.size, output_mode1.size);
        } else {
            panic!("Output1 not found in space");
        }
        if let Some(geom2) = state.space.output_geometry(&output2) {
            // output_handlers.rs logic: current_max_x = geom.loc.x + geom.size.w
            assert_eq!(geom2.loc, Point::from((output_mode1.size.w, 0)), "Output2 should be positioned after Output1");
        } else {
            panic!("Output2 not found in space");
        }
    }

    #[test]
    fn test_output_mode_updated_reflects_in_space() {
        let mut state = create_test_desktop_state();
        let initial_mode = Mode { size: (800, 600).into(), refresh: 60000/1000 };
        let updated_mode = Mode { size: (1024, 768).into(), refresh: 60000/1000 };
        let dummy_output = create_dummy_output("test-output-mode", initial_mode);

        state.new_output(dummy_output.clone());

        // Manually update the output's current mode, as if Smithay did it via a backend
        assert!(dummy_output.set_current_mode(updated_mode), "Failed to set current mode on output for test");

        // Call the handler
        state.output_mode_updated(&dummy_output, updated_mode);

        if let Some(geom) = state.space.output_geometry(&dummy_output) {
            assert_eq!(geom.size, updated_mode.size, "Space geometry should reflect the updated mode size");
        } else {
            panic!("Output not found in space after mode update");
        }
    }

    #[test]
    fn test_output_destroyed_removes_from_state_and_space() {
        let mut state = create_test_desktop_state();
        let output_mode = get_default_mode();
        let dummy_output_to_destroy = create_dummy_output("test-output-destroy", output_mode);
        let dummy_output_to_keep = create_dummy_output("test-output-keep", output_mode);

        state.new_output(dummy_output_to_destroy.clone());
        state.new_output(dummy_output_to_keep.clone());

        assert_eq!(state.outputs.len(), 2, "Two outputs should be present initially");
        assert_eq!(state.space.outputs().count(), 2, "Space should have two outputs initially");

        state.output_destroyed(&dummy_output_to_destroy);

        assert_eq!(state.outputs.len(), 1, "Destroyed output should be removed from state list");
        assert_eq!(state.outputs[0].name(), "test-output-keep", "Kept output should remain in state list");

        assert_eq!(state.space.outputs().count(), 1, "Destroyed output should be unmapped from space");
        assert_eq!(state.space.outputs().next().unwrap().name(), "test-output-keep", "Kept output should remain in space");
    }
}
