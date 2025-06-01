// src/input/pointer.rs

use super::config::PointerConfig;
// Import Processed*Event types from focus.rs
use super::focus::{
    ProcessedPointerMotionEvent,
    ProcessedPointerButtonEvent,
    ProcessedPointerScrollEvent,
    ScrollSource,
    ButtonState, // Make sure ButtonState is defined here or commonly accessible
};

// ButtonState might be defined in focus.rs or a common types module.
// If it's defined here, ensure it's consistent.
// pub enum ButtonState { Pressed, Released } // Already in focus.rs, use that.

pub struct Pointer {
    x: f64,
    y: f64,
    config: PointerConfig,
    // focus_manager field removed
}

impl Pointer {
    // focus_manager parameter removed
    pub fn new(config: PointerConfig) -> Self {
        tracing::info!("Pointer: Initializing with config: {:?}", config);
        Self {
            x: 0.0,
            y: 0.0,
            config,
        }
    }

    // Returns Option<ProcessedPointerMotionEvent>
    pub fn handle_motion_event(&mut self, dx: f64, dy: f64, time: u32) -> Option<ProcessedPointerMotionEvent> {
        let sensitivity = self.config.sensitivity.unwrap_or(1.0);
        let mut sensitive_dx = dx * sensitivity;
        let mut sensitive_dy = dy * sensitivity;

        let accel_factor = self.config.acceleration_factor.unwrap_or(0.0);
        if accel_factor > 0.0 {
            let velocity = (sensitive_dx.powi(2) + sensitive_dy.powi(2)).sqrt();
            if velocity > 0.0 {
                let accel = 1.0 + (velocity * accel_factor);
                sensitive_dx *= accel;
                sensitive_dy *= accel;
            }
        }

        self.x += sensitive_dx;
        self.y += sensitive_dy;

        tracing::debug!(
            "Pointer: Motion dx={}, dy={} (sensitive_dx={}, sensitive_dy={}) -> new coords x={}, y={}",
            dx, dy, sensitive_dx, sensitive_dy, self.x, self.y
        );

        let event = ProcessedPointerMotionEvent {
            abs_x: self.x,
            abs_y: self.y,
            rel_dx: sensitive_dx,
            rel_dy: sensitive_dy,
            time,
            serial: 0, // Placeholder, FocusManager should fill this
        };
        // self.focus_manager.handle_pointer_motion(event); // Removed call
        Some(event)
    }

    // Returns Option<ProcessedPointerButtonEvent>
    pub fn handle_button_event(&mut self, raw_button_code: u32, state: ButtonState, time: u32) -> Option<ProcessedPointerButtonEvent> {
        let mapped_button_code = self.config.button_mapping
            .as_ref()
            .and_then(|mapping| mapping.get(&raw_button_code).copied())
            .unwrap_or(raw_button_code);

        tracing::debug!(
            "Pointer: Button event raw_code={}, mapped_code={}, state={:?}, time={}, at x={},y={}",
            raw_button_code, mapped_button_code, state, time, self.x, self.y
        );

        let event = ProcessedPointerButtonEvent {
            button_code: mapped_button_code,
            state,
            abs_x: self.x,
            abs_y: self.y,
            time,
            serial: 0, // Placeholder, FocusManager should fill this
        };
        // self.focus_manager.handle_pointer_button(event); // Removed call
        Some(event)
    }

    // Returns Option<ProcessedPointerScrollEvent>
    pub fn handle_scroll_event(
        &mut self,
        dx_discrete: f64,
        dy_discrete: f64,
        dx_continuous: f64,
        dy_continuous: f64,
        source: ScrollSource,
        time: u32
    ) -> Option<ProcessedPointerScrollEvent> {
        let scroll_dx = if dx_continuous.abs() > 0.0 { dx_continuous } else { dx_discrete };
        let scroll_dy = if dy_continuous.abs() > 0.0 { dy_continuous } else { dy_discrete };

        tracing::debug!(
            "Pointer: Scroll event dx={}, dy={}, source={:?}, time={}, at x={},y={}",
            scroll_dx, scroll_dy, source, time, self.x, self.y
        );

        let event = ProcessedPointerScrollEvent {
            delta_x: scroll_dx,
            delta_y: scroll_dy,
            source,
            abs_x: self.x,
            abs_y: self.y,
            time,
            serial: 0, // Placeholder, FocusManager should fill this
        };
        // self.focus_manager.handle_pointer_scroll(event); // Removed call
        Some(event)
    }

    // TODO: Implement Pointer Constraints
    // TODO: Implement more sophisticated Pointer Acceleration Curves
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::config::PointerConfig;
    use std::collections::HashMap;

    fn default_pointer_config() -> PointerConfig {
        PointerConfig {
            acceleration_factor: Some(0.0), // No accel by default for simple tests
            sensitivity: Some(1.0),
            acceleration_curve: None,
            button_mapping: None,
        }
    }

    #[test]
    fn test_pointer_new() {
        let pointer = Pointer::new(default_pointer_config());
        assert_eq!(pointer.x, 0.0);
        assert_eq!(pointer.y, 0.0);
        assert_eq!(pointer.config.sensitivity, Some(1.0));
    }

    #[test]
    fn test_handle_motion_event_no_accel_no_sens() {
        let mut cfg = default_pointer_config();
        cfg.sensitivity = Some(1.0); // Ensure it's exactly 1.0
        cfg.acceleration_factor = Some(0.0); // Ensure no accel
        let mut pointer = Pointer::new(cfg);

        let event = pointer.handle_motion_event(10.0, -5.0, 100).unwrap();
        assert_eq!(pointer.x, 10.0);
        assert_eq!(pointer.y, -5.0);
        assert_eq!(event.abs_x, 10.0);
        assert_eq!(event.abs_y, -5.0);
        assert_eq!(event.rel_dx, 10.0);
        assert_eq!(event.rel_dy, -5.0);
    }

    #[test]
    fn test_handle_motion_event_with_sensitivity() {
        let mut cfg = default_pointer_config();
        cfg.sensitivity = Some(2.0);
        let mut pointer = Pointer::new(cfg);

        let event = pointer.handle_motion_event(10.0, -5.0, 100).unwrap();
        assert_eq!(pointer.x, 20.0); // 10.0 * 2.0
        assert_eq!(pointer.y, -10.0); // -5.0 * 2.0
        assert_eq!(event.rel_dx, 20.0);
        assert_eq!(event.rel_dy, -10.0);
    }

    #[test]
    fn test_handle_motion_event_with_linear_acceleration() {
        let mut cfg = default_pointer_config();
        cfg.sensitivity = Some(1.0);
        cfg.acceleration_factor = Some(0.1); // Simple factor
        let mut pointer = Pointer::new(cfg);

        // dx=3, dy=4, velocity (hypotenuse) = 5
        // accel = 1.0 + (5.0 * 0.1) = 1.5
        // final_dx = 3.0 * 1.5 = 4.5
        // final_dy = 4.0 * 1.5 = 6.0
        let event = pointer.handle_motion_event(3.0, 4.0, 100).unwrap();
        assert_eq!(pointer.x, 4.5);
        assert_eq!(pointer.y, 6.0);
        assert_eq!(event.rel_dx, 4.5);
        assert_eq!(event.rel_dy, 6.0);
    }

    #[test]
    fn test_handle_motion_event_no_movement_no_accel_change() {
        let mut cfg = default_pointer_config();
        cfg.acceleration_factor = Some(0.5); // Non-zero accel factor
        let mut pointer = Pointer::new(cfg);
        let event = pointer.handle_motion_event(0.0, 0.0, 100).unwrap();
        assert_eq!(pointer.x, 0.0);
        assert_eq!(pointer.y, 0.0);
        assert_eq!(event.rel_dx, 0.0);
        assert_eq!(event.rel_dy, 0.0);
    }

    #[test]
    fn test_handle_button_event_no_mapping() {
        let pointer = Pointer::new(default_pointer_config());
        let event = pointer.handle_button_event(1, ButtonState::Pressed, 100).unwrap();
        assert_eq!(event.button_code, 1);
        assert_eq!(event.state, ButtonState::Pressed);
        assert_eq!(event.abs_x, 0.0); // Pointer hasn't moved
    }

    #[test]
    fn test_handle_button_event_with_mapping() {
        let mut cfg = default_pointer_config();
        let mut mapping = HashMap::new();
        mapping.insert(1, 272); // Map button 1 (e.g. left) to BTN_LEFT (Linux evdev code)
        mapping.insert(3, 273); // Map button 3 (e.g. right) to BTN_RIGHT
        cfg.button_mapping = Some(mapping);
        let pointer = Pointer::new(cfg);

        let event1 = pointer.handle_button_event(1, ButtonState::Pressed, 100).unwrap();
        assert_eq!(event1.button_code, 272);

        let event2 = pointer.handle_button_event(3, ButtonState::Released, 101).unwrap();
        assert_eq!(event2.button_code, 273);

        let event_unmapped = pointer.handle_button_event(2, ButtonState::Pressed, 102).unwrap();
        assert_eq!(event_unmapped.button_code, 2); // Unmapped, so raw code
    }

    #[test]
    fn test_handle_scroll_event_discrete() {
        let mut pointer = Pointer::new(default_pointer_config());
        let event = pointer.handle_scroll_event(1.0, -1.0, 0.0, 0.0, ScrollSource::Wheel, 100).unwrap();
        assert_eq!(event.delta_x, 1.0);
        assert_eq!(event.delta_y, -1.0);
        assert_eq!(event.source, ScrollSource::Wheel);
    }

    #[test]
    fn test_handle_scroll_event_continuous_priority() {
        let mut pointer = Pointer::new(default_pointer_config());
        // Provide both discrete and continuous, continuous should be used
        let event = pointer.handle_scroll_event(1.0, -1.0, 0.5, -0.7, ScrollSource::Finger, 100).unwrap();
        assert_eq!(event.delta_x, 0.5);
        assert_eq!(event.delta_y, -0.7);
        assert_eq!(event.source, ScrollSource::Finger);
    }
     #[test]
    fn test_handle_scroll_event_continuous_only() {
        let mut pointer = Pointer::new(default_pointer_config());
        let event = pointer.handle_scroll_event(0.0, 0.0, 0.25, -0.35, ScrollSource::Continuous, 100).unwrap();
        assert_eq!(event.delta_x, 0.25);
        assert_eq!(event.delta_y, -0.35);
        assert_eq!(event.source, ScrollSource::Continuous);
    }
}
