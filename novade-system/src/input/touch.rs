// src/input/touch.rs
use crate::input::config::TouchConfig;
use input::event::touch::{
    TouchDownEvent, TouchFrameEvent, TouchMotionEvent, TouchUpEvent, TouchCancelEvent,
};
use std::collections::HashMap;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    pub id: i32, // Slot ID from libinput, used as the touch point ID
    pub x: f64,  // Current x coordinate
    pub y: f64,  // Current y coordinate
    // pub initial_x: f64, // Could store initial position for gesture recognition
    // pub initial_y: f64,
    // pub pressure: Option<f64>, // If pressure data is available and needed
}

#[derive(Debug)]
pub struct Touch {
    #[allow(dead_code)] // Config might be used later
    config: TouchConfig,
    active_points: HashMap<i32, TouchPoint>, // Keyed by slot ID

    // Conceptual stubs for advanced features
    // gesture_recognizer: Option<SomeGestureRecognizerType>, // Replace with actual type
    // calibration_matrix: Option<SomeCalibrationMatrixType>, // Replace with actual type
}

// Conceptual types for stubs (not actually defined here)
// struct SomeGestureRecognizerType;
// struct SomeCalibrationMatrixType { pub matrix: [f64; 6], }
// impl SomeCalibrationMatrixType {
//    fn transform(&self, x: f64, y: f64) -> (f64, f64) { (x,y) /* Placeholder */ }
// }


impl Touch {
    pub fn new(config: &TouchConfig) -> Self {
        debug!("Touch: Initializing with config: {:?}", config);
        Self {
            config: config.clone(),
            active_points: HashMap::new(),
            // gesture_recognizer: None, // Initialize gesture recognizer if/when available
            // calibration_matrix: None, // Load calibration matrix if path provided in config
        }
    }

    // fn is_palm(&self, event: &TouchDownEvent) -> bool {
    //   // Placeholder for palm detection logic
    //   // This might involve checking event.tool_type(), event.size_major/minor(),
    //   // or other heuristics if the 'input' crate exposes them for the event type.
    //   // Libinput itself performs some level of palm detection if configured.
    //   false
    // }

    /// Handles a touch down event.
    pub fn handle_down_event(&mut self, event: &TouchDownEvent) {
        // TODO: Implement palm rejection logic.
        // This might involve checking touch point size/shape (if available from libinput events
        // like `TouchPadTouchPoint` which has minor/major axis info, though base `TouchEvent` might not)
        // or using other heuristics.
        // If a touch is deemed a palm, it should be ignored (return early).
        // For example:
        // if self.is_palm(&event) {
        //   info!(slot = event.slot().unwrap_or(-1), "Touch event on slot {} rejected as palm.", event.slot().unwrap_or(-1));
        //   return;
        // }

        let id = event.slot().unwrap_or_else(|| {
            warn!("TouchDownEvent missing slot ID, using time as fallback (not robust).");
            event.time() as i32
        });

        let raw_x = event.x();
        let raw_y = event.y();

        // TODO: Apply touch calibration matrix here if available.
        // let (calibrated_x, calibrated_y) = if let Some(matrix) = &self.calibration_matrix {
        //   matrix.transform(raw_x, raw_y)
        // } else {
        //   (raw_x, raw_y)
        // };
        // Use calibrated_x, calibrated_y below.
        let x = raw_x; // Using raw for now
        let y = raw_y;


        let point = TouchPoint { id, x, y };
        self.active_points.insert(id, point);

        info!(
            "Touch: Down event: time={}, id={}, x={:.2}, y={:.2}. Active points: {}",
            event.time(), id, x, y, self.active_points.len()
        );
    }

    /// Handles a touch motion event.
    pub fn handle_motion_event(&mut self, event: &TouchMotionEvent) {
        let id = event.slot().unwrap_or_else(|| {
            warn!("TouchMotionEvent missing slot ID.");
            return;
        });
        let raw_x = event.x();
        let raw_y = event.y();

        // TODO: Apply touch calibration matrix here if available.
        // let (calibrated_x, calibrated_y) = if let Some(matrix) = &self.calibration_matrix {
        //   matrix.transform(raw_x, raw_y)
        // } else {
        //   (raw_x, raw_y)
        // };
        // Use calibrated_x, calibrated_y below.
        let x = raw_x; // Using raw for now
        let y = raw_y;

        if let Some(point) = self.active_points.get_mut(&id) {
            point.x = x;
            point.y = y;
            debug!(
                "Touch: Motion event: time={}, id={}, new_x={:.2}, new_y={:.2}",
                event.time(), id, x, y
            );
        } else {
            warn!(
                "Touch: Motion event for unknown touch ID {}: time={}, x={:.2}, y={:.2}",
                id, event.time(), x, y
            );
        }
    }

    /// Handles a touch up event.
    pub fn handle_up_event(&mut self, event: &TouchUpEvent) {
        let id = event.slot().unwrap_or_else(|| {
            warn!("TouchUpEvent missing slot ID.");
            return;
        });

        if self.active_points.remove(&id).is_some() {
            info!(
                "Touch: Up event: time={}, id={}. Active points: {}",
                event.time(), id, self.active_points.len()
            );
        } else {
            warn!(
                "Touch: Up event for unknown touch ID {}: time={}",
                id, event.time()
            );
        }
        // TODO: Send wl_touch.up to the client.
    }

    /// Handles a touch frame event.
    /// This signifies the end of a set of touch updates in an atomic batch.
    pub fn handle_frame_event(&mut self, _event: &TouchFrameEvent) {
        // TODO: Feed touch points from self.active_points to a gesture recognizer here.
        // Based on recognizer state, it might consume points or emit gesture events.
        // For example:
        // if let Some(recognizer) = &mut self.gesture_recognizer {
        //   match recognizer.process_frame(&self.active_points) {
        //     GestureEvent::Swipe { ... } => { /* log or handle swipe */ },
        //     GestureEvent::Pinch { ... } => { /* log or handle pinch */ },
        //     _ => {} // No gesture or gesture in progress
        //   }
        // }

        info!("Touch: Frame event received. (Pending events would be flushed now, potential gestures processed).");
    }

    /// Handles a touch cancel event.
    /// This signifies that the touch session was cancelled, e.g., by a gesture recognizer
    /// or palm detection taking over the touch points, or an external factor.
    pub fn handle_cancel_event(&mut self, _event: &TouchCancelEvent) {
        // TODO: If a gesture was in progress, ensure the gesture recognizer is reset.
        // if let Some(recognizer) = &mut self.gesture_recognizer {
        //    recognizer.cancel_sequence();
        // }

        info!("Touch: Cancel event received. Clearing all {} active touch points.", self.active_points.len());
        self.active_points.clear();
    }
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
