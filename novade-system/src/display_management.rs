// Copyright 2024 NovaDE Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Manages information about connected displays/outputs for the NovaDE compositor.

use novade_core::types::geometry::{Rect, Point, Size};
use smithay::output::Output as SmithayOutput;
use smithay::backend::drm::Mode as DrmModeSmithay;
use std::sync::{Arc, Mutex as StdMutex};
use tracing::warn;


/// Represents information about a managed display/output.
#[derive(Debug, Clone)]
pub struct ManagedOutput {
    pub id: String,
    pub name: String,
    pub description: String,
    pub geometry: Rect,
    pub work_area: Rect,
    pub scale: f64,
    pub is_primary: bool,
    pub current_drm_mode: Option<DrmModeSmithay>,
    pub smithay_output: SmithayOutput,
}

/// Manages all detected displays/outputs.
#[derive(Debug, Default)]
pub struct DisplayManager {
    outputs: Vec<ManagedOutput>,
    primary_output_id: Option<String>,
}

impl DisplayManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_output(&mut self, mut output_data: ManagedOutput) {
        if self.outputs.iter().any(|o| o.id == output_data.id) {
            tracing::warn!("Output with ID {} already exists. Updating it.", output_data.id);
            self.outputs.retain(|o| o.id != output_data.id);
        }

        if output_data.is_primary {
            if let Some(current_primary_id) = &self.primary_output_id {
                if current_primary_id != &output_data.id {
                    if let Some(old_primary) = self.outputs.iter_mut().find(|o| o.id == *current_primary_id) {
                        old_primary.is_primary = false;
                    }
                }
            }
            self.primary_output_id = Some(output_data.id.clone());
        } else if self.primary_output_id.is_none() && self.outputs.is_empty() {
            output_data.is_primary = true;
            self.primary_output_id = Some(output_data.id.clone());
        }
        
        self.outputs.push(output_data);
    }

    pub fn remove_output(&mut self, output_id: &str) {
        self.outputs.retain(|o| o.id != output_id);
        if self.primary_output_id.as_deref() == Some(output_id) {
            self.primary_output_id = None;
            if let Some(first_remaining_output) = self.outputs.first_mut() {
                first_remaining_output.is_primary = true;
                self.primary_output_id = Some(first_remaining_output.id.clone());
                tracing::info!("Primary output {} removed. New primary set to {}.", output_id, first_remaining_output.id);
            } else {
                tracing::info!("Primary output {} removed. No other outputs to set as primary.", output_id);
            }
        }
    }

    pub fn get_output_by_id(&self, output_id: &str) -> Option<&ManagedOutput> {
        self.outputs.iter().find(|o| o.id == output_id)
    }
    
    pub fn get_mut_output_by_id(&mut self, output_id: &str) -> Option<&mut ManagedOutput> {
        self.outputs.iter_mut().find(|o| o.id == output_id)
    }

    pub fn get_primary_output(&self) -> Option<&ManagedOutput> {
        self.primary_output_id.as_ref().and_then(|id| self.get_output_by_id(id))
    }
    
    pub fn get_mut_primary_output(&mut self) -> Option<&mut ManagedOutput> {
        self.primary_output_id.clone().and_then(move |id| self.get_mut_output_by_id(&id))
    }

    pub fn all_outputs(&self) -> &Vec<ManagedOutput> {
        &self.outputs
    }

    pub fn all_outputs_mut(&mut self) -> &mut Vec<ManagedOutput> {
        &mut self.outputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::output::{PhysicalProperties, Scale as SmithayScale, Mode as SmithayMode};
    use smithay::reexports::wayland_server::Display; // Minimal display for Output::new
    use smithay::reexports::wayland_server::DisplayHandle;


    // Helper to create a SmithayOutput for testing.
    // This is challenging as Output::new is not public and requires a DisplayHandle's internal state.
    // We'll use a stub approach where we can.
    // For tests where the SmithayOutput's full behavior isn't critical, this might suffice.
    // However, some SmithayOutput methods might panic if not properly initialized.
    fn create_test_smithay_output(dh: &DisplayHandle, name: &str, description: &str) -> SmithayOutput {
        // SmithayOutput::new is private. We have to use what's available.
        // A global Output is usually created via `display.create_global::<MyState, wl_output::WlOutput, _>(...)`
        // and then `Output::create(...)`.
        // For unit tests, this is very hard.
        // Let's assume we are testing the logic of DisplayManager, not the deep interaction with SmithayOutput.
        // The SmithayOutput in ManagedOutput is `Clone`, which means it's an Arc internally.
        // We can't easily make a "bare" one.
        //
        // Workaround: If we had a test display, we could make one.
        // let mut display = Display::<()>::new().unwrap();
        // let dh = display.handle();
        // let global = dh.create_global::<(), wl_output::WlOutput, _>(3,Default::default());
        // SmithayOutput::create(&global, name.to_string(), Some(dh.clone()), None) -> This also needs OutputState.
        //
        // Given the constraints, we will create ManagedOutput instances with a placeholder SmithayOutput
        // that is constructed to satisfy the type system but might not be fully functional.
        // This means tests on DisplayManager's interaction *through* the SmithayOutput might be limited.

        let physical_props = PhysicalProperties {
            size: (0,0).into(), // mm
            subpixel: smithay::reexports::wayland_server::protocol::wl_output::Subpixel::Unknown,
            make: "NovaDE Test Inc.".to_string(),
            model: "Mock Display".to_string(),
        };
        // This is the problematic part: SmithayOutput::new is not public.
        // We use the fact that SmithayOutput has a `From<Global<WlOutput>>`
        // but creating a Global<WlOutput> also requires a DisplayHandle with a known OutputState.
        // This is a common issue when unit testing parts of a larger system like a Wayland compositor.
        //
        // For now, to make tests compile and focus on DisplayManager's Vec logic:
        // We will use a "default" SmithayOutput, which might panic if certain methods are called.
        // This is highly dependent on Smithay's internal structure and testability.
        //
        // A Display object is needed to create globals, and Output objects are tied to globals.
        // We'll create a dummy Display for test scope.
        let mut test_display = Display::<()>::new().unwrap(); // State type doesn't matter much here.
        let test_dh = test_display.handle();

        let output_global = test_dh.create_global::<(), smithay::reexports::wayland_server::protocol::wl_output::WlOutput, _>(
            3, // version
            () // user_data for global, not OutputStateSmithay
        );
        let output = SmithayOutput::new(name.to_string(), physical_props, Some(test_dh.clone()));
        output.set_description(description);
        output
    }


    fn create_managed_output_for_test(dh: &DisplayHandle, id_str: &str, name_str: &str, is_primary: bool, scale_val: f64) -> ManagedOutput {
        let smithay_out = create_test_smithay_output(dh, name_str, &format!("Test Output {}", name_str));

        // Manually set scale on the SmithayOutput object if possible (Smithay's API might vary)
        // SmithayOutput::current_scale() returns a RefMut, direct setting might be tricky or via methods.
        // For testing, we assume the scale is set correctly when it's passed to ManagedOutput.
        // The scale in ManagedOutput is what DisplayManager uses.

        ManagedOutput {
            id: id_str.to_string(),
            name: name_str.to_string(),
            description: format!("Test Output {}", name_str),
            geometry: Rect { position: Point{x:0, y:0}, size: Size{width:1920, height:1080}},
            work_area: Rect { position: Point{x:0, y:0}, size: Size{width:1920, height:1080}},
            scale: scale_val,
            is_primary,
            current_drm_mode: None,
            smithay_output: smithay_out
        }
    }

    #[test]
    fn test_dm_new_empty() {
        let dm = DisplayManager::new();
        assert!(dm.all_outputs().is_empty());
        assert!(dm.get_primary_output().is_none());
    }

    #[test]
    fn test_dm_add_output() {
        let mut test_display = Display::<()>::new().unwrap();
        let dh = test_display.handle();
        let mut dm = DisplayManager::new();

        let output1 = create_managed_output_for_test(&dh, "DP-1", "DisplayPort 1", false, 1.0);
        dm.add_output(output1.clone());
        assert_eq!(dm.all_outputs().len(), 1);
        assert_eq!(dm.all_outputs()[0].id, "DP-1");
        assert!(dm.all_outputs()[0].is_primary, "First output should become primary");
        assert_eq!(dm.get_primary_output().unwrap().id, "DP-1");

        let output2 = create_managed_output_for_test(&dh, "HDMI-1", "HDMI 1", true, 2.0);
        dm.add_output(output2.clone());
        assert_eq!(dm.all_outputs().len(), 2);
        assert!(dm.get_output_by_id("HDMI-1").unwrap().is_primary, "HDMI-1 should be primary");
        assert!(!dm.get_output_by_id("DP-1").unwrap().is_primary, "DP-1 should no longer be primary");
        assert_eq!(dm.get_primary_output().unwrap().id, "HDMI-1");

        // Test adding an output that already exists (should update)
        let output1_updated = create_managed_output_for_test(&dh, "DP-1", "DisplayPort 1 Updated", false, 1.5);
        dm.add_output(output1_updated.clone());
        assert_eq!(dm.all_outputs().len(), 2, "Should still be 2 outputs after update");
        assert_eq!(dm.get_output_by_id("DP-1").unwrap().scale, 1.5);
        assert_eq!(dm.get_primary_output().unwrap().id, "HDMI-1"); // Primary should not change on update of other
    }

    #[test]
    fn test_dm_remove_output() {
        let mut test_display = Display::<()>::new().unwrap();
        let dh = test_display.handle();
        let mut dm = DisplayManager::new();
        let output1 = create_managed_output_for_test(&dh, "DP-1", "DP-1", false, 1.0);
        let output2 = create_managed_output_for_test(&dh, "HDMI-1", "HDMI-1", false, 1.0);

        dm.add_output(output1); // Becomes primary
        dm.add_output(output2); // DP-1 is still primary

        assert_eq!(dm.all_outputs().len(), 2);
        dm.remove_output("DP-1"); // Remove primary
        assert_eq!(dm.all_outputs().len(), 1);
        assert!(dm.get_output_by_id("DP-1").is_none());
        assert!(dm.get_output_by_id("HDMI-1").is_some());
        assert!(dm.get_primary_output().is_some(), "A new primary should be set");
        assert_eq!(dm.get_primary_output().unwrap().id, "HDMI-1", "HDMI-1 should become primary");

        dm.remove_output("HDMI-1");
        assert!(dm.all_outputs().is_empty());
        assert!(dm.get_primary_output().is_none());
    }

    #[test]
    fn test_dm_get_output_by_id() {
        let mut test_display = Display::<()>::new().unwrap();
        let dh = test_display.handle();
        let mut dm = DisplayManager::new();
        let output1 = create_managed_output_for_test(&dh, "VGA-1", "VGA-1", false, 1.0);
        dm.add_output(output1);

        assert!(dm.get_output_by_id("VGA-1").is_some());
        assert!(dm.get_output_by_id("DVI-1").is_none());

        let mut_output = dm.get_mut_output_by_id("VGA-1").unwrap();
        mut_output.description = "New Description".to_string();
        assert_eq!(dm.get_output_by_id("VGA-1").unwrap().description, "New Description");
    }

    #[test]
    fn test_dm_primary_output_logic() {
        let mut test_display = Display::<()>::new().unwrap();
        let dh = test_display.handle();
        let mut dm = DisplayManager::new();

        let o1 = create_managed_output_for_test(&dh, "O1", "O1", false, 1.0);
        dm.add_output(o1); // o1 becomes primary
        assert_eq!(dm.get_primary_output().unwrap().id, "O1");

        let o2 = create_managed_output_for_test(&dh, "O2", "O2", true, 1.0); // o2 explicitly primary
        dm.add_output(o2);
        assert_eq!(dm.get_primary_output().unwrap().id, "O2");
        assert!(!dm.get_output_by_id("O1").unwrap().is_primary);

        let o3 = create_managed_output_for_test(&dh, "O3", "O3", false, 1.0);
        dm.add_output(o3);
        assert_eq!(dm.get_primary_output().unwrap().id, "O2"); // o2 remains primary

        dm.remove_output("O2"); // Remove primary o2
        assert_eq!(dm.get_primary_output().unwrap().id, "O1");
        assert!(dm.get_output_by_id("O1").unwrap().is_primary);
        assert!(!dm.get_output_by_id("O3").unwrap().is_primary);
    }
}
