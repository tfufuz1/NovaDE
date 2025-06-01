// src/input/touch.rs

use std::collections::HashMap;
// Import Processed*Event types from focus.rs
use super::focus::{
    ProcessedTouchDownEvent,
    ProcessedTouchMotionEvent,
    ProcessedTouchUpEvent,
    ProcessedTouchFrameEvent
};

#[derive(Debug, Clone)]
pub struct TouchPoint {
    pub id: i32,
    pub x: f64,
    pub y: f64,
    pub initial_x: f64,
    pub initial_y: f64,
    pub initial_time: u32,
}

pub struct Touch {
    active_points: HashMap<i32, TouchPoint>,
    // focus_manager field removed
}

impl Touch {
    // focus_manager parameter removed
    pub fn new() -> Self {
        tracing::info!("Touch: Initializing (refactored)...");
        Self {
            active_points: HashMap::new(),
        }
    }

    // Returns Option<ProcessedTouchDownEvent>
    pub fn handle_touch_down_event(&mut self, id: i32, x: f64, y: f64, time: u32) -> Option<ProcessedTouchDownEvent> {
        if self.active_points.contains_key(&id) {
            tracing::warn!("Touch: Touch down event for already active ID: {}. Ignoring.", id);
            return None;
        }

        let new_point = TouchPoint {
            id, x, y,
            initial_x: x, initial_y: y,
            initial_time: time,
        };
        self.active_points.insert(id, new_point);

        tracing::debug!("Touch: Down event - ID: {}, X: {}, Y: {}, Time: {}", id, x, y, time);

        let event = ProcessedTouchDownEvent {
            id, x, y, time,
            serial: 0, // Placeholder, FocusManager should fill this
        };
        Some(event)
    }

    // Returns Option<ProcessedTouchMotionEvent>
    pub fn handle_touch_motion_event(&mut self, id: i32, x: f64, y: f64, time: u32) -> Option<ProcessedTouchMotionEvent> {
        match self.active_points.get_mut(&id) {
            Some(point) => {
                point.x = x;
                point.y = y;

                tracing::debug!("Touch: Motion event - ID: {}, X: {}, Y: {}, Time: {}", id, x, y, time);

                let event = ProcessedTouchMotionEvent {
                    id, x, y, time,
                    // No serial for motion in Wayland touch typically
                };
                Some(event)
            }
            None => {
                tracing::warn!("Touch: Motion event for unknown ID: {}. Ignoring.", id);
                None
            }
        }
    }

    // Returns Option<ProcessedTouchUpEvent>
    pub fn handle_touch_up_event(&mut self, id: i32, time: u32) -> Option<ProcessedTouchUpEvent> {
        if let Some(removed_point) = self.active_points.remove(&id) {
            tracing::debug!("Touch: Up event - ID: {}, Time: {}", id, time);

            let event = ProcessedTouchUpEvent {
                id,
                x: removed_point.x,
                y: removed_point.y,
                time,
                serial: 0, // Placeholder, FocusManager should fill this
            };
            Some(event)
        } else {
            tracing::warn!("Touch: Up event for unknown ID: {}. Ignoring.", id);
            None
        }
    }

    // Returns Option<ProcessedTouchFrameEvent>
    pub fn handle_touch_frame_event(&mut self, time: u32) -> Option<ProcessedTouchFrameEvent> {
        tracing::debug!("Touch: Frame event - Time: {}", time);
        let event = ProcessedTouchFrameEvent { time };
        Some(event)
    }

    // TODO: Implement more advanced Touch Gesture Recognition (e.g., pinch, rotate).
    // TODO: Implement Touch Calibration (mapping raw touch data to screen coordinates).
    // TODO: Implement Palm Rejection.
}
