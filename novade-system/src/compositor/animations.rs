// Copyright 2024 NovaDE Compositor contributors
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

//! Basic animation system for the compositor.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use novade_domain::workspaces::core::WindowId; // Assuming this is the correct WindowId
use tracing::trace;

/// State of an animation.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AnimationState {
    Running,
    Completed,
}

/// Trait for a generic animation.
pub trait Animation: Send + Sync + std::fmt::Debug {
    fn start_time(&self) -> Instant;
    fn duration(&self) -> Duration;

    /// Updates the animation's internal state based on the current time.
    /// Returns `AnimationState::Completed` if the animation has finished.
    fn update(&mut self, now: Instant) -> AnimationState;

    /// Gets the current value of the animated property (e.g., opacity, offset).
    /// The meaning of this value depends on the specific animation type.
    fn current_value(&self) -> f32;

    /// A way to identify the type of animation, e.g., for opacity, position.
    /// This helps in managing multiple animations of different types on the same window.
    fn animation_type(&self) -> AnimationType;
}

/// Enum to identify different types of animations.
/// Useful if a window can have multiple, independent animations (e.g., fade and move).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationType {
    Opacity,
    // ANCHOR: Add other types like PositionX, PositionY, Scale, etc.
}


/// A simple linear fade animation.
#[derive(Debug)]
pub struct FadeAnimation {
    start_time: Instant,
    duration: Duration,
    initial_opacity: f32,
    final_opacity: f32,
    current_opacity: f32,
}

impl FadeAnimation {
    pub fn new(duration_ms: u64, initial_opacity: f32, final_opacity: f32) -> Self {
        Self {
            start_time: Instant::now(),
            duration: Duration::from_millis(duration_ms),
            initial_opacity,
            final_opacity,
            current_opacity: initial_opacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn 거의_같음(a: f32, b: f32, epsilon: f32) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn test_fade_animation_lifecycle() {
        let mut anim = FadeAnimation::new(100, 0.0, 1.0); // 100ms duration
        let start_time = anim.start_time();

        assert_eq!(anim.animation_type(), AnimationType::Opacity);
        assert_eq!(anim.current_value(), 0.0);
        assert_eq!(anim.update(start_time), AnimationState::Running);

        // Halfway
        let halfway_time = start_time + Duration::from_millis(50);
        assert_eq!(anim.update(halfway_time), AnimationState::Running);
        assert!(거의_같음(anim.current_value(), 0.5, 0.001));

        // Almost complete
        let almost_time = start_time + Duration::from_millis(99);
        assert_eq!(anim.update(almost_time), AnimationState::Running);
        assert!(anim.current_value() > 0.9 && anim.current_value() < 1.0);


        // Complete
        let end_time = start_time + Duration::from_millis(100);
        assert_eq!(anim.update(end_time), AnimationState::Completed);
        assert_eq!(anim.current_value(), 1.0);

        // After completion
        let after_time = start_time + Duration::from_millis(150);
        assert_eq!(anim.update(after_time), AnimationState::Completed);
        assert_eq!(anim.current_value(), 1.0);
    }

    #[test]
    fn test_fade_animation_reverse() {
        let mut anim = FadeAnimation::new(100, 1.0, 0.0); // Fade out
        let start_time = anim.start_time();

        assert_eq!(anim.current_value(), 1.0);
        let halfway_time = start_time + Duration::from_millis(50);
        anim.update(halfway_time);
        assert!(거의_같음(anim.current_value(), 0.5, 0.001));

        let end_time = start_time + Duration::from_millis(100);
        anim.update(end_time);
        assert_eq!(anim.current_value(), 0.0);
    }


    #[test]
    fn test_animation_manager_add_and_update() {
        let mut manager = AnimationManager::new();
        let window_id1 = WindowId::new();
        let now = Instant::now();

        manager.add_animation(window_id1, Box::new(FadeAnimation::new(100, 0.0, 1.0)));
        assert!(manager.has_active_animations(Some(window_id1)));
        assert!(manager.has_active_animations(None));
        assert!(manager.get_window_opacity(window_id1).is_some());
        assert!(거의_같음(manager.get_window_opacity(window_id1).unwrap(), 0.0, 0.001));

        // Update part way
        let time1 = now + Duration::from_millis(50);
        assert!(manager.update_animations(time1), "Animations should still be running");
        assert!(manager.has_active_animations(Some(window_id1)));
        let opacity1 = manager.get_window_opacity(window_id1).unwrap();
        assert!(opacity1 > 0.0 && opacity1 < 1.0, "Opacity is {}", opacity1); // Should be around 0.5

        // Update to completion
        let time2 = now + Duration::from_millis(100);
        assert!(!manager.update_animations(time2), "Animations should be completed"); // update_animations returns false when all complete
        assert!(!manager.has_active_animations(Some(window_id1)), "Window1 should have no active animations");
        assert!(!manager.has_active_animations(None), "Manager should have no active animations");
        // Opacity might be None after completion and removal, or could return final value.
        // Current get_window_opacity returns None if no *active* animation of that type.
        assert!(manager.get_window_opacity(window_id1).is_none(), "Opacity should be None after animation completes and is removed");
    }

    #[test]
    fn test_animation_manager_multiple_windows() {
        let mut manager = AnimationManager::new();
        let window_id1 = WindowId::new();
        let window_id2 = WindowId::new();
        let now = Instant::now();

        manager.add_animation(window_id1, Box::new(FadeAnimation::new(100, 0.0, 1.0)));
        manager.add_animation(window_id2, Box::new(FadeAnimation::new(200, 0.0, 1.0)));

        assert!(manager.has_active_animations(Some(window_id1)));
        assert!(manager.has_active_animations(Some(window_id2)));

        let time1 = now + Duration::from_millis(100);
        assert!(manager.update_animations(time1)); // win2 still running
        assert!(!manager.has_active_animations(Some(window_id1))); // win1 completed
        assert!(manager.has_active_animations(Some(window_id2)));

        let opacity2_at_100ms = manager.get_window_opacity(window_id2).unwrap();
        assert!(opacity2_at_100ms > 0.0 && opacity2_at_100ms < 1.0); // Should be around 0.5 for win2

        let time2 = now + Duration::from_millis(200);
        assert!(!manager.update_animations(time2)); // All complete
        assert!(!manager.has_active_animations(Some(window_id2)));
        assert!(!manager.has_active_animations(None));
    }

    #[test]
    fn test_animation_manager_replace_animation() {
        let mut manager = AnimationManager::new();
        let window_id1 = WindowId::new();
        let now = Instant::now();

        manager.add_animation(window_id1, Box::new(FadeAnimation::new(100, 0.0, 1.0)));
        assert!(거의_같음(manager.get_window_opacity(window_id1).unwrap(), 0.0, 0.001));

        // Replace with a new animation for the same window and type
        manager.add_animation(window_id1, Box::new(FadeAnimation::new(100, 0.5, 1.0)));
        // New animation starts from its initial value immediately
        assert!(거의_같음(manager.get_window_opacity(window_id1).unwrap(), 0.5, 0.001));

        // Ensure only one opacity animation exists
        let anims = manager.active_animations.get(&window_id1).unwrap();
        assert_eq!(anims.len(), 1);
        assert_eq!(anims[0].animation_type(), AnimationType::Opacity);


        let time1 = now + Duration::from_millis(50);
        manager.update_animations(time1);
        // Progress on the *new* animation (0.5 to 1.0 over 100ms, so at 50ms it's 0.75)
        assert!(거의_같음(manager.get_window_opacity(window_id1).unwrap(), 0.75, 0.001));
    }
}

impl Animation for FadeAnimation {
    fn start_time(&self) -> Instant {
        self.start_time
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn update(&mut self, now: Instant) -> AnimationState {
        let elapsed = now.saturating_duration_since(self.start_time);
        if elapsed >= self.duration {
            self.current_opacity = self.final_opacity;
            AnimationState::Completed
        } else {
            let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
            self.current_opacity = self.initial_opacity + (self.final_opacity - self.initial_opacity) * progress;
            AnimationState::Running
        }
    }

    fn current_value(&self) -> f32 {
        self.current_opacity.clamp(0.0, 1.0) // Ensure opacity stays within valid range
    }

    fn animation_type(&self) -> AnimationType {
        AnimationType::Opacity
    }
}

/// Manages all active animations in the compositor.
#[derive(Debug, Default)]
pub struct AnimationManager {
    // Using AnimationType as key within the Vec allows replacing an animation of the same type.
    // Or, if only one animation of each type is allowed, HashMap<AnimationType, Box<dyn Animation>>
    active_animations: HashMap<WindowId, Vec<Box<dyn Animation>>>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an animation for a specific window.
    /// If an animation of the same type already exists for this window, it's replaced.
    pub fn add_animation(&mut self, window_id: WindowId, animation: Box<dyn Animation>) {
        let anim_type = animation.animation_type();
        let window_anims = self.active_animations.entry(window_id).or_default();
        // Remove existing animation of the same type
        window_anims.retain(|anim| anim.animation_type() != anim_type);
        trace!("Adding {:?} animation for window {:?}", anim_type, window_id);
        window_anims.push(animation);
    }

    /// Updates all active animations. Removes completed animations.
    /// Returns true if any animation is still running, false otherwise.
    /// This can be used to schedule redraws.
    pub fn update_animations(&mut self, now: Instant) -> bool {
        let mut any_running = false;
        self.active_animations.retain(|window_id, animations| {
            animations.retain_mut(|anim| {
                let state = anim.update(now);
                if state == AnimationState::Running {
                    any_running = true;
                    true // Keep running animation
                } else {
                    trace!("Animation {:?} completed for window {:?}", anim.animation_type(), window_id);
                    false // Remove completed animation
                }
            });
            !animations.is_empty() // Keep window_id entry if it still has animations
        });
        any_running
    }

    /// Gets the current opacity for a window if a fade animation is active.
    /// Returns `None` if no opacity animation is running (implying full opacity or externally managed).
    pub fn get_window_opacity(&self, window_id: WindowId) -> Option<f32> {
        self.active_animations.get(&window_id)
            .and_then(|animations| {
                animations.iter().find_map(|anim| {
                    if anim.animation_type() == AnimationType::Opacity {
                        Some(anim.current_value())
                    } else {
                        None
                    }
                })
            })
    }

    // ANCHOR: Add methods to get other animated values, e.g., get_window_position_offset.
    // ANCHOR: Add method to check if any animation is running for a specific window or globally,
    // to help decide if a repaint is needed.
    pub fn has_active_animations(&self, window_id: Option<WindowId>) -> bool {
        if let Some(id) = window_id {
            self.active_animations.get(&id).map_or(false, |anims| !anims.is_empty())
        } else {
            !self.active_animations.is_empty()
        }
    }
}
