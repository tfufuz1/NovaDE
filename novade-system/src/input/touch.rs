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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_new() {
        let touch = Touch::new();
        assert!(touch.active_points.is_empty());
    }

    #[test]
    fn test_handle_touch_down() {
        let mut touch = Touch::new();
        let event_opt = touch.handle_touch_down_event(0, 10.0, 20.0, 1000);
        assert!(event_opt.is_some());
        let event = event_opt.unwrap();

        assert_eq!(touch.active_points.len(), 1);
        let point = touch.active_points.get(&0).unwrap();
        assert_eq!(point.id, 0);
        assert_eq!(point.x, 10.0);
        assert_eq!(point.y, 20.0);
        assert_eq!(point.initial_x, 10.0);
        assert_eq!(point.initial_y, 20.0);
        assert_eq!(point.initial_time, 1000);

        assert_eq!(event.id, 0);
        assert_eq!(event.x, 10.0);
        assert_eq!(event.y, 20.0);
        assert_eq!(event.time, 1000);

        // Test duplicate down event for same ID
        let event_opt_dup = touch.handle_touch_down_event(0, 15.0, 25.0, 1001);
        assert!(event_opt_dup.is_none()); // Should be ignored
        assert_eq!(touch.active_points.len(), 1); // Count should not change
        assert_eq!(touch.active_points.get(&0).unwrap().x, 10.0); // Original point data should remain
    }

    #[test]
    fn test_handle_touch_motion() {
        let mut touch = Touch::new();
        touch.handle_touch_down_event(0, 10.0, 20.0, 1000);

        let event_opt = touch.handle_touch_motion_event(0, 15.0, 25.0, 1001);
        assert!(event_opt.is_some());
        let event = event_opt.unwrap();

        let point = touch.active_points.get(&0).unwrap();
        assert_eq!(point.x, 15.0);
        assert_eq!(point.y, 25.0);
        assert_eq!(point.initial_x, 10.0); // Initial should not change

        assert_eq!(event.id, 0);
        assert_eq!(event.x, 15.0);
        assert_eq!(event.y, 25.0);
        assert_eq!(event.time, 1001);

        // Motion for non-existent ID
        let event_opt_unknown = touch.handle_touch_motion_event(1, 30.0, 30.0, 1002);
        assert!(event_opt_unknown.is_none());
    }

    #[test]
    fn test_handle_touch_up() {
        let mut touch = Touch::new();
        touch.handle_touch_down_event(0, 10.0, 20.0, 1000);
        assert_eq!(touch.active_points.len(), 1);

        let event_opt = touch.handle_touch_up_event(0, 1002);
        assert!(event_opt.is_some());
        let event = event_opt.unwrap();

        assert!(touch.active_points.is_empty());
        assert_eq!(event.id, 0);
        assert_eq!(event.x, 10.0); // Last known position
        assert_eq!(event.y, 20.0); // Last known position
        assert_eq!(event.time, 1002);

        // Up for non-existent ID
        let event_opt_unknown = touch.handle_touch_up_event(1, 1003);
        assert!(event_opt_unknown.is_none());
    }

    #[test]
    fn test_handle_touch_frame() {
        let mut touch = Touch::new();
        let event_opt = touch.handle_touch_frame_event(1004);
        assert!(event_opt.is_some());
        assert_eq!(event_opt.unwrap().time, 1004);
    }

    #[test]
    fn test_multi_touch_tracking() {
        let mut touch = Touch::new();

        // Point 0 down
        touch.handle_touch_down_event(0, 10.0, 20.0, 1000);
        assert_eq!(touch.active_points.len(), 1);

        // Point 1 down
        touch.handle_touch_down_event(1, 30.0, 40.0, 1001);
        assert_eq!(touch.active_points.len(), 2);

        // Move point 0
        touch.handle_touch_motion_event(0, 12.0, 22.0, 1002);
        assert_eq!(touch.active_points.get(&0).unwrap().x, 12.0);
        assert_eq!(touch.active_points.get(&1).unwrap().x, 30.0); // Point 1 unchanged

        // Move point 1
        touch.handle_touch_motion_event(1, 33.0, 44.0, 1003);
        assert_eq!(touch.active_points.get(&0).unwrap().x, 12.0); // Point 0 unchanged
        assert_eq!(touch.active_points.get(&1).unwrap().x, 33.0);

        // Point 0 up
        touch.handle_touch_up_event(0, 1004);
        assert_eq!(touch.active_points.len(), 1);
        assert!(touch.active_points.get(&0).is_none());
        assert!(touch.active_points.get(&1).is_some());

        // Point 1 up
        touch.handle_touch_up_event(1, 1005);
        assert!(touch.active_points.is_empty());
    }
}
