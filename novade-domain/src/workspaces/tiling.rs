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

//! Defines tiling algorithms and related data structures for workspaces.

use std::collections::HashMap;
use novade_core::types::geometry::Rect;
use crate::workspaces::core::WindowId; // Assuming this is novade_domain::workspaces::core::WindowId

/// Trait for all tiling algorithms.
pub trait TilingAlgorithm: std::fmt::Debug + Send + Sync {
    /// Arranges the given windows within the specified screen area.
    ///
    /// # Arguments
    /// * `windows`: A slice of `WindowId`s to be arranged.
    /// * `screen_area`: The available `Rect` (e.g., monitor work area) for tiling.
    ///
    /// # Returns
    /// A `HashMap` mapping each `WindowId` to its calculated `Rect` geometry.
    /// Returns an empty map if no windows are provided or if the algorithm cannot arrange.
    fn arrange(&self, windows: &[WindowId], screen_area: Rect) -> HashMap<WindowId, Rect>;

    /// Returns the name of the layout algorithm.
    fn name(&self) -> String;
}

/// Master-Stack Tiling Algorithm.
/// Divides the screen into a master area and a stack area.
#[derive(Debug, Clone, PartialEq)]
pub struct MasterStackLayout {
    /// Number of windows in the master area.
    pub num_master: usize,
    /// Width percentage of the master area (0.0 to 1.0).
    pub master_width_percentage: f32,
    // ANCHOR: Add gap, margin configurations.
}

impl Default for MasterStackLayout {
    fn default() -> Self {
        Self {
            num_master: 1,
            master_width_percentage: 0.5,
        }
    }
}

impl TilingAlgorithm for MasterStackLayout {
    fn name(&self) -> String {
        "MasterStack".to_string()
    }

    fn arrange(&self, windows: &[WindowId], screen_area: Rect) -> HashMap<WindowId, Rect> {
        let mut geometries = HashMap::new();
        if windows.is_empty() || screen_area.size.width == 0 || screen_area.size.height == 0 {
            return geometries;
        }

        let num_windows = windows.len();
        let master_count = self.num_master.min(num_windows); // Cannot have more master windows than available windows

        if master_count == 0 { // All windows are stacked
            let stack_width = screen_area.size.width;
            let window_height = screen_area.size.height / num_windows as i32;
            for (i, window_id) in windows.iter().enumerate() {
                geometries.insert(*window_id, Rect {
                    position: novade_core::types::geometry::Point {
                        x: screen_area.position.x,
                        y: screen_area.position.y + (i as i32 * window_height),
                    },
                    size: novade_core::types::geometry::Size {
                        width: stack_width,
                        height: window_height,
                    },
                });
            }
        } else if master_count == num_windows { // All windows are in master area (e.g. single window)
            let master_window_height = screen_area.size.height / master_count as i32;
            for (i, window_id) in windows.iter().take(master_count).enumerate() {
                geometries.insert(*window_id, Rect {
                     position: novade_core::types::geometry::Point {
                        x: screen_area.position.x,
                        y: screen_area.position.y + (i as i32 * master_window_height),
                    },
                    size: novade_core::types::geometry::Size {
                        width: screen_area.size.width,
                        height: master_window_height,
                    },
                });
            }
        } else { // Mixed master and stack
            let master_area_width = (screen_area.size.width as f32 * self.master_width_percentage).round() as i32;
            let stack_area_width = screen_area.size.width - master_area_width;

            // Arrange master windows
            let master_window_height = screen_area.size.height / master_count as i32;
            for (i, window_id) in windows.iter().take(master_count).enumerate() {
                geometries.insert(*window_id, Rect {
                    position: novade_core::types::geometry::Point {
                        x: screen_area.position.x,
                        y: screen_area.position.y + (i as i32 * master_window_height),
                    },
                    size: novade_core::types::geometry::Size {
                        width: master_area_width,
                        height: master_window_height,
                    },
                });
            }

            // Arrange stack windows
            let stack_windows = &windows[master_count..];
            let num_stack_windows = stack_windows.len();
            if num_stack_windows > 0 {
                let stack_window_height = screen_area.size.height / num_stack_windows as i32;
                for (i, window_id) in stack_windows.iter().enumerate() {
                    geometries.insert(*window_id, Rect {
                        position: novade_core::types::geometry::Point {
                            x: screen_area.position.x + master_area_width,
                            y: screen_area.position.y + (i as i32 * stack_window_height),
                        },
                        size: novade_core::types::geometry::Size {
                            width: stack_area_width,
                            height: stack_window_height,
                        },
                    });
                }
            }
        }
        geometries
    }
}

/// Spiral (Fibonacci) Tiling Algorithm.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpiralLayout;
// ANCHOR: Add configuration options for SpiralLayout if needed.

impl TilingAlgorithm for SpiralLayout {
    fn name(&self) -> String {
        "Spiral".to_string()
    }

    fn arrange(&self, windows: &[WindowId], screen_area: Rect) -> HashMap<WindowId, Rect> {
        let mut geometries = HashMap::new();
        if windows.is_empty() || screen_area.size.width == 0 || screen_area.size.height == 0 {
            return geometries;
        }

        // ANCHOR: Implement actual spiral tiling logic.
        // This is a placeholder that tiles windows in a simple horizontal row for now.
        // A true spiral layout is more complex.
        let window_width = screen_area.size.width / windows.len() as i32;
        let window_height = screen_area.size.height;

        for (i, window_id) in windows.iter().enumerate() {
            geometries.insert(*window_id, Rect {
                position: novade_core::types::geometry::Point {
                    x: screen_area.position.x + (i as i32 * window_width),
                    y: screen_area.position.y,
                },
                size: novade_core::types::geometry::Size {
                    width: window_width,
                    height: window_height,
                },
            });
        }
        geometries
    }
}

// Enum to represent different tiling algorithm configurations
// This will be used in WorkspaceLayout.
// This is moved from manager.rs to here to be alongside its implementations.
#[derive(Debug, Clone, PartialEq)]
pub enum TilingOptions {
    MasterStack(MasterStackLayout),
    Spiral(SpiralLayout),
    // Add other layouts here
}

impl TilingOptions {
    /// Gets the underlying TilingAlgorithm trait object.
    pub fn as_algorithm(&self) -> Box<dyn TilingAlgorithm> {
        match self {
            TilingOptions::MasterStack(ms) => Box::new(ms.clone()),
            TilingOptions::Spiral(s) => Box::new(s.clone()),
        }
    }
}
