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
            TilingOptions::MasterStack(ms) => Box::new(ms.clone()),
            TilingOptions::Spiral(s) => Box::new(s.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::types::geometry::{Point, Size}; // Rect is already imported

    // Helper to create dummy WindowIds for testing
    fn make_win_ids(count: usize) -> Vec<WindowId> {
        (0..count).map(|i| WindowId::from_string(&format!("win{}", i + 1))).collect()
    }

    #[test]
    fn test_master_stack_no_windows() {
        let layout = MasterStackLayout::default();
        let windows = make_win_ids(0);
        let screen = Rect::new(Point::new(0,0), Size::new(1920,1080));
        let arrangement = layout.arrange(&windows, screen);
        assert!(arrangement.is_empty());
    }

    #[test]
    fn test_master_stack_one_window() {
        let layout = MasterStackLayout::default();
        let windows = make_win_ids(1);
        let screen = Rect::new(Point::new(0,0), Size::new(1920,1080));
        let arrangement = layout.arrange(&windows, screen);
        assert_eq!(arrangement.len(), 1);
        assert_eq!(arrangement[&windows[0]], screen); // Should take full screen
    }

    #[test]
    fn test_master_stack_master_only_two_windows() {
        let layout = MasterStackLayout { num_master: 2, master_width_percentage: 0.6, ..Default::default() };
        let windows = make_win_ids(2);
        let screen = Rect::new(Point::new(0,0), Size::new(1000,600));
        let arrangement = layout.arrange(&windows, screen);

        assert_eq!(arrangement.len(), 2);
        // Both should be in master area, stacked vertically, taking full width
        let win1_geom = arrangement[&windows[0]];
        let win2_geom = arrangement[&windows[1]];

        assert_eq!(win1_geom, Rect::new(Point::new(0,0), Size::new(1000,300)));
        assert_eq!(win2_geom, Rect::new(Point::new(0,300), Size::new(1000,300)));
    }

    #[test]
    fn test_master_stack_stack_only_two_windows() {
        // num_master = 0 means all windows go to stack area, which occupies full width
        let layout = MasterStackLayout { num_master: 0, ..Default::default() };
        let windows = make_win_ids(2);
        let screen = Rect::new(Point::new(0,0), Size::new(800,600));
        let arrangement = layout.arrange(&windows, screen);

        assert_eq!(arrangement.len(), 2);
        let win1_geom = arrangement[&windows[0]];
        let win2_geom = arrangement[&windows[1]];

        assert_eq!(win1_geom, Rect::new(Point::new(0,0), Size::new(800,300)));
        assert_eq!(win2_geom, Rect::new(Point::new(0,300), Size::new(800,300)));
    }


    #[test]
    fn test_master_stack_mixed_three_windows_one_master() {
        let layout = MasterStackLayout { num_master: 1, master_width_percentage: 0.5, ..Default::default() };
        let windows = make_win_ids(3); // win1 (master), win2, win3 (stack)
        let screen = Rect::new(Point::new(0,0), Size::new(1000,600));
        let arrangement = layout.arrange(&windows, screen);

        assert_eq!(arrangement.len(), 3);
        let master_geom = arrangement[&windows[0]];
        let stack1_geom = arrangement[&windows[1]];
        let stack2_geom = arrangement[&windows[2]];

        // Master window (win1)
        assert_eq!(master_geom.position.x, 0);
        assert_eq!(master_geom.position.y, 0);
        assert_eq!(master_geom.size.width, 500); // 50% of 1000
        assert_eq!(master_geom.size.height, 600); // Full height

        // Stack windows (win2, win3)
        assert_eq!(stack1_geom.position.x, 500); // Starts after master
        assert_eq!(stack1_geom.position.y, 0);
        assert_eq!(stack1_geom.size.width, 500); // Remaining 50%
        assert_eq!(stack1_geom.size.height, 300); // 600 / 2 stack windows

        assert_eq!(stack2_geom.position.x, 500);
        assert_eq!(stack2_geom.position.y, 300);
        assert_eq!(stack2_geom.size.width, 500);
        assert_eq!(stack2_geom.size.height, 300);
    }

    #[test]
    fn test_master_stack_mixed_four_windows_two_master() {
        let layout = MasterStackLayout { num_master: 2, master_width_percentage: 0.6, ..Default::default() };
        let windows = make_win_ids(4); // win1, win2 (master); win3, win4 (stack)
        let screen = Rect::new(Point::new(0,0), Size::new(1000,600));
        let arrangement = layout.arrange(&windows, screen);

        assert_eq!(arrangement.len(), 4);
        let master1_geom = arrangement[&windows[0]];
        let master2_geom = arrangement[&windows[1]];
        let stack1_geom = arrangement[&windows[2]];
        let stack2_geom = arrangement[&windows[3]];

        // Master windows
        assert_eq!(master1_geom, Rect::new(Point::new(0,0), Size::new(600,300)));
        assert_eq!(master2_geom, Rect::new(Point::new(0,300), Size::new(600,300)));

        // Stack windows
        assert_eq!(stack1_geom, Rect::new(Point::new(600,0), Size::new(400,300)));
        assert_eq!(stack2_geom, Rect::new(Point::new(600,300), Size::new(400,300)));
    }

    #[test]
    fn test_spiral_no_windows() {
        let layout = SpiralLayout::default();
        let windows = make_win_ids(0);
        let screen = Rect::new(Point::new(0,0), Size::new(1920,1080));
        let arrangement = layout.arrange(&windows, screen);
        assert!(arrangement.is_empty());
    }

    #[test]
    fn test_spiral_one_window() {
        let layout = SpiralLayout::default();
        let windows = make_win_ids(1);
        let screen = Rect::new(Point::new(0,0), Size::new(1920,1080));
        let arrangement = layout.arrange(&windows, screen);
        assert_eq!(arrangement.len(), 1);
        assert_eq!(arrangement[&windows[0]], screen);
    }

    #[test]
    fn test_spiral_multiple_windows_placeholder() {
        // ANCHOR: Update this test when SpiralLayout has actual spiral logic.
        // This test reflects the current placeholder behavior (horizontal row).
        let layout = SpiralLayout::default();
        let windows = make_win_ids(3);
        let screen = Rect::new(Point::new(0,0), Size::new(900,600));
        let arrangement = layout.arrange(&windows, screen);

        assert_eq!(arrangement.len(), 3);
        let win1_geom = arrangement[&windows[0]];
        let win2_geom = arrangement[&windows[1]];
        let win3_geom = arrangement[&windows[2]];

        assert_eq!(win1_geom, Rect::new(Point::new(0,0), Size::new(300,600)));
        assert_eq!(win2_geom, Rect::new(Point::new(300,0), Size::new(300,600)));
        assert_eq!(win3_geom, Rect::new(Point::new(600,0), Size::new(300,600)));

        // Check distinctness (basic)
        assert_ne!(win1_geom, win2_geom);
        assert_ne!(win2_geom, win3_geom);
    }
}
