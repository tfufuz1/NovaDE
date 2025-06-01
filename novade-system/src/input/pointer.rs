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
