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

use novade_core::types::geometry::Rect;
use smithay::output::Output as SmithayOutput;
use smithay::backend::drm::Mode as DrmModeSmithay; // Renamed to avoid conflict if novade_core also has Mode
use std::sync::{Arc, Mutex as StdMutex}; // Using StdMutex for data structures
use tracing::warn;

/// Represents information about a managed display/output.
#[derive(Debug, Clone)]
pub struct ManagedOutput {
    /// Unique identifier for the output (e.g., Smithay Output name).
    pub id: String,
    /// Name of the output, often same as ID or a more descriptive one if available.
    pub name: String,
    /// Description of the output (e.g., monitor model).
    /// ANCHOR: This may require backend-specific ways to get (e.g., EDID for DRM).
    pub description: String,
    /// Position and size of the output in the global compositor space.
    pub geometry: Rect,
    /// Usable work area of the output, excluding panels, docks, etc.
    /// ANCHOR: For now, this will be the same as `geometry`. Needs to be calculated later
    /// by considering panels, docks, or other reserved areas on this output.
    pub work_area: Rect,
    /// Display scale factor.
    pub scale: f64,
    /// Whether this is the primary display.
    pub is_primary: bool,
    /// Backend-specific mode information (e.g., from DRM).
    /// ANCHOR: This might need to be a more generic type if not always DRM,
    /// or store a `novade_core::types::display::DisplayMode`.
    /// For now, storing Smithay's DrmMode directly if available from a DRM backend.
    pub current_drm_mode: Option<DrmModeSmithay>,
    /// The underlying Smithay Output object.
    /// Storing this allows access to more detailed info and Smithay operations if needed.
    pub smithay_output: SmithayOutput,
}

/// Manages all detected displays/outputs.
#[derive(Debug, Default)]
pub struct DisplayManager {
    outputs: Vec<ManagedOutput>,
    primary_output_id: Option<String>, // Stores the ID of the primary output
}

impl DisplayManager {
    /// Creates a new, empty `DisplayManager`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new managed output.
    /// If output_data.is_primary is true, it attempts to set this as the primary output.
    /// If no primary output is set and this is the first output, it becomes primary.
    pub fn add_output(&mut self, mut output_data: ManagedOutput) {
        if self.outputs.iter().any(|o| o.id == output_data.id) {
            tracing::warn!("Output with ID {} already exists. Updating it.", output_data.id);
            self.outputs.retain(|o| o.id != output_data.id); // Remove old one
        }

        if output_data.is_primary {
            // If the new output is marked as primary, unset current primary's flag
            if let Some(current_primary_id) = &self.primary_output_id {
                if current_primary_id != &output_data.id { // Avoid self-unsetting if it was already primary
                    if let Some(old_primary) = self.outputs.iter_mut().find(|o| o.id == *current_primary_id) {
                        old_primary.is_primary = false;
                    }
                }
            }
            self.primary_output_id = Some(output_data.id.clone());
        } else if self.primary_output_id.is_none() && self.outputs.is_empty() {
            // If it's the first output and no primary is set, make this one primary.
            output_data.is_primary = true;
            self.primary_output_id = Some(output_data.id.clone());
        }
        
        self.outputs.push(output_data);
        // ANCHOR: Sort outputs if a specific order is desired (e.g., by position).
    }

    /// Removes an output by its ID.
    /// If the primary output is removed, it tries to set the next available output as primary.
    pub fn remove_output(&mut self, output_id: &str) {
        self.outputs.retain(|o| o.id != output_id);
        if self.primary_output_id.as_deref() == Some(output_id) {
            self.primary_output_id = None; // Clear old primary
            if let Some(first_remaining_output) = self.outputs.first_mut() {
                first_remaining_output.is_primary = true;
                self.primary_output_id = Some(first_remaining_output.id.clone());
                tracing::info!("Primary output {} removed. New primary set to {}.", output_id, first_remaining_output.id);
            } else {
                tracing::info!("Primary output {} removed. No other outputs to set as primary.", output_id);
            }
        }
    }

    /// Gets an immutable reference to an output by its ID.
    pub fn get_output_by_id(&self, output_id: &str) -> Option<&ManagedOutput> {
        self.outputs.iter().find(|o| o.id == output_id)
    }
    
    /// Gets a mutable reference to an output by its ID.
    pub fn get_mut_output_by_id(&mut self, output_id: &str) -> Option<&mut ManagedOutput> {
        self.outputs.iter_mut().find(|o| o.id == output_id)
    }

    /// Gets an immutable reference to the primary output, if set.
    pub fn get_primary_output(&self) -> Option<&ManagedOutput> {
        self.primary_output_id.as_ref().and_then(|id| self.get_output_by_id(id))
    }
    
    /// Gets a mutable reference to the primary output, if set.
    pub fn get_mut_primary_output(&mut self) -> Option<&mut ManagedOutput> {
        self.primary_output_id.clone().and_then(move |id| self.get_mut_output_by_id(&id))
    }

    /// Returns an immutable reference to the list of all managed outputs.
    pub fn all_outputs(&self) -> &Vec<ManagedOutput> {
        &self.outputs
    }

    /// Returns a mutable reference to the list of all managed outputs.
    pub fn all_outputs_mut(&mut self) -> &mut Vec<ManagedOutput> {
        &mut self.outputs
    }

    // ANCHOR: Hotplugging integration point for DesktopState's OutputHandler
    // When Smithay's OutputHandler::new_output is called:
    //   - Extract info from smithay::output::Output.
    //   - Create ManagedOutput.
    //   - Call self.add_output(managed_output).
    //   - Potentially trigger workspace reassignment or layout updates.
    // When Smithay's OutputHandler::output_destroyed is called:
    //   - Call self.remove_output(output_name).
    //   - Potentially trigger workspace reassignment or layout updates.
}
