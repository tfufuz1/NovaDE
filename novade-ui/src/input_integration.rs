// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Input Integration Module
//!
//! This module provides integration between the UI layer and input devices.
//! It handles keyboard, mouse, touch, and other input events for the NovaDE desktop environment.

use gtk4 as gtk;
use gtk::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, HashSet};

use crate::error::UiError;
use crate::common::{UiResult, UiComponent};
use crate::compositor_integration::CompositorIntegration;

/// Input integration manager
pub struct InputIntegration {
    /// The GTK application
    app: gtk::Application,
    
    /// Compositor integration
    compositor: Arc<CompositorIntegration>,
    
    /// Keyboard state
    keyboard_state: Arc<RwLock<KeyboardState>>,
    
    /// Mouse state
    mouse_state: Arc<RwLock<MouseState>>,
    
    /// Touch state
    touch_state: Arc<RwLock<TouchState>>,
    
    /// Gesture recognizers
    gesture_recognizers: Arc<RwLock<Vec<Box<dyn GestureRecognizer + Send + Sync>>>>,
    
    /// Input event handlers
    event_handlers: Arc<RwLock<HashMap<InputEventType, Vec<Box<dyn Fn(&InputEvent) -> UiResult<()> + Send + Sync>>>>>,
    
    /// Input settings
    settings: Arc<RwLock<InputSettings>>,
}

/// Keyboard state
pub struct KeyboardState {
    /// Currently pressed keys
    pressed_keys: HashSet<u32>,
    
    /// Keyboard layout
    layout: String,
    
    /// Keyboard repeat rate
    repeat_rate: u32,
    
    /// Keyboard repeat delay
    repeat_delay: u32,
    
    /// Keyboard modifiers
    modifiers: KeyboardModifiers,
    
    /// Is caps lock on
    caps_lock: bool,
    
    /// Is num lock on
    num_lock: bool,
}

/// Keyboard modifiers
#[derive(Default, Clone, Copy)]
pub struct KeyboardModifiers {
    /// Shift key
    pub shift: bool,
    
    /// Control key
    pub ctrl: bool,
    
    /// Alt key
    pub alt: bool,
    
    /// Super key
    pub super_key: bool,
    
    /// Alt Gr key
    pub alt_gr: bool,
    
    /// Meta key
    pub meta: bool,
}

/// Mouse state
pub struct MouseState {
    /// Mouse position
    position: (i32, i32),
    
    /// Mouse buttons
    buttons: MouseButtons,
    
    /// Mouse scroll delta
    scroll_delta: (f64, f64),
    
    /// Mouse speed
    speed: f64,
    
    /// Mouse acceleration
    acceleration: f64,
}

/// Mouse buttons
#[derive(Default, Clone, Copy)]
pub struct MouseButtons {
    /// Left button
    pub left: bool,
    
    /// Right button
    pub right: bool,
    
    /// Middle button
    pub middle: bool,
    
    /// Back button
    pub back: bool,
    
    /// Forward button
    pub forward: bool,
}

/// Touch state
pub struct TouchState {
    /// Active touch points
    touch_points: HashMap<u32, TouchPoint>,
    
    /// Is touch enabled
    enabled: bool,
    
    /// Touch device capabilities
    capabilities: TouchCapabilities,
}

/// Touch point
pub struct TouchPoint {
    /// Touch ID
    id: u32,
    
    /// Touch position
    position: (i32, i32),
    
    /// Touch pressure
    pressure: f64,
    
    /// Touch size
    size: (f64, f64),
}

/// Touch capabilities
#[derive(Default, Clone, Copy)]
pub struct TouchCapabilities {
    /// Supports multi-touch
    pub multi_touch: bool,
    
    /// Supports pressure
    pub pressure: bool,
    
    /// Supports size
    pub size: bool,
    
    /// Supports rotation
    pub rotation: bool,
}

/// Input event
pub struct InputEvent {
    /// Event type
    event_type: InputEventType,
    
    /// Event time
    time: u32,
    
    /// Event source
    source: InputSource,
    
    /// Event data
    data: InputEventData,
}

/// Input event type
pub enum InputEventType {
    /// Key press
    KeyPress,
    
    /// Key release
    KeyRelease,
    
    /// Mouse button press
    ButtonPress,
    
    /// Mouse button release
    ButtonRelease,
    
    /// Mouse motion
    Motion,
    
    /// Mouse scroll
    Scroll,
    
    /// Touch begin
    TouchBegin,
    
    /// Touch update
    TouchUpdate,
    
    /// Touch end
    TouchEnd,
    
    /// Gesture begin
    GestureBegin,
    
    /// Gesture update
    GestureUpdate,
    
    /// Gesture end
    GestureEnd,
}

/// Input source
pub enum InputSource {
    /// Keyboard
    Keyboard,
    
    /// Mouse
    Mouse,
    
    /// Touchpad
    Touchpad,
    
    /// Touchscreen
    Touchscreen,
    
    /// Tablet
    Tablet,
    
    /// Gamepad
    Gamepad,
    
    /// Other input device
    Other,
}

/// Input event data
pub enum InputEventData {
    /// Key event data
    Key {
        /// Key code
        key_code: u32,
        
        /// Key symbol
        key_sym: u32,
        
        /// Key modifiers
        modifiers: KeyboardModifiers,
        
        /// Key state
        state: bool,
    },
    
    /// Button event data
    Button {
        /// Button code
        button: u32,
        
        /// Button position
        position: (i32, i32),
        
        /// Button state
        state: bool,
    },
    
    /// Motion event data
    Motion {
        /// Motion position
        position: (i32, i32),
        
        /// Motion delta
        delta: (i32, i32),
    },
    
    /// Scroll event data
    Scroll {
        /// Scroll position
        position: (i32, i32),
        
        /// Scroll delta
        delta: (f64, f64),
    },
    
    /// Touch event data
    Touch {
        /// Touch ID
        id: u32,
        
        /// Touch position
        position: (i32, i32),
        
        /// Touch pressure
        pressure: f64,
        
        /// Touch size
        size: (f64, f64),
    },
    
    /// Gesture event data
    Gesture {
        /// Gesture type
        gesture_type: GestureType,
        
        /// Gesture position
        position: (i32, i32),
        
        /// Gesture state
        state: GestureState,
        
        /// Gesture parameters
        parameters: HashMap<String, f64>,
    },
}

/// Gesture type
pub enum GestureType {
    /// Tap gesture
    Tap,
    
    /// Long press gesture
    LongPress,
    
    /// Swipe gesture
    Swipe,
    
    /// Pinch gesture
    Pinch,
    
    /// Rotate gesture
    Rotate,
    
    /// Edge swipe gesture
    EdgeSwipe,
}

/// Gesture state
pub enum GestureState {
    /// Gesture began
    Begin,
    
    /// Gesture updated
    Update,
    
    /// Gesture ended
    End,
    
    /// Gesture cancelled
    Cancel,
}

/// Input settings
pub struct InputSettings {
    /// Keyboard settings
    pub keyboard: KeyboardSettings,
    
    /// Mouse settings
    pub mouse: MouseSettings,
    
    /// Touch settings
    pub touch: TouchSettings,
    
    /// Gesture settings
    pub gesture: GestureSettings,
}

/// Keyboard settings
pub struct KeyboardSettings {
    /// Keyboard layout
    pub layout: String,
    
    /// Keyboard repeat rate
    pub repeat_rate: u32,
    
    /// Keyboard repeat delay
    pub repeat_delay: u32,
    
    /// Enable keyboard shortcuts
    pub enable_shortcuts: bool,
    
    /// Custom keyboard shortcuts
    pub custom_shortcuts: HashMap<String, u32>,
}

/// Mouse settings
pub struct MouseSettings {
    /// Mouse speed
    pub speed: f64,
    
    /// Mouse acceleration
    pub acceleration: f64,
    
    /// Natural scrolling
    pub natural_scrolling: bool,
    
    /// Scroll speed
    pub scroll_speed: f64,
    
    /// Double click time
    pub double_click_time: u32,
}

/// Touch settings
pub struct TouchSettings {
    /// Enable touch
    pub enable_touch: bool,
    
    /// Touch sensitivity
    pub sensitivity: f64,
    
    /// Enable touch gestures
    pub enable_gestures: bool,
    
    /// Edge swipe threshold
    pub edge_swipe_threshold: i32,
}

/// Gesture settings
pub struct GestureSettings {
    /// Enable gestures
    pub enable_gestures: bool,
    
    /// Gesture sensitivity
    pub sensitivity: f64,
    
    /// Tap timeout
    pub tap_timeout: u32,
    
    /// Long press timeout
    pub long_press_timeout: u32,
    
    /// Swipe threshold
    pub swipe_threshold: f64,
    
    /// Pinch threshold
    pub pinch_threshold: f64,
    
    /// Rotate threshold
    pub rotate_threshold: f64,
}

/// Gesture recognizer trait
pub trait GestureRecognizer {
    /// Processes an input event
    fn process_event(&mut self, event: &InputEvent) -> Option<InputEvent>;
    
    /// Gets the gesture type
    fn gesture_type(&self) -> GestureType;
    
    /// Resets the recognizer state
    fn reset(&mut self);
}

impl InputIntegration {
    /// Creates a new input integration manager
    pub fn new(
        app: gtk::Application,
        compositor: Arc<CompositorIntegration>,
    ) -> Self {
        Self {
            app,
            compositor,
            keyboard_state: Arc::new(RwLock::new(KeyboardState {
                pressed_keys: HashSet::new(),
                layout: "us".to_string(),
                repeat_rate: 25,
                repeat_delay: 400,
                modifiers: KeyboardModifiers::default(),
                caps_lock: false,
                num_lock: true,
            })),
            mouse_state: Arc::new(RwLock::new(MouseState {
                position: (0, 0),
                buttons: MouseButtons::default(),
                scroll_delta: (0.0, 0.0),
                speed: 1.0,
                acceleration: 1.0,
            })),
            touch_state: Arc::new(RwLock::new(TouchState {
                touch_points: HashMap::new(),
                enabled: true,
                capabilities: TouchCapabilities::default(),
            })),
            gesture_recognizers: Arc::new(RwLock::new(Vec::new())),
            event_handlers: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(InputSettings {
                keyboard: KeyboardSettings {
                    layout: "us".to_string(),
                    repeat_rate: 25,
                    repeat_delay: 400,
                    enable_shortcuts: true,
                    custom_shortcuts: HashMap::new(),
                },
                mouse: MouseSettings {
                    speed: 1.0,
                    acceleration: 1.0,
                    natural_scrolling: true,
                    scroll_speed: 1.0,
                    double_click_time: 400,
                },
                touch: TouchSettings {
                    enable_touch: true,
                    sensitivity: 1.0,
                    enable_gestures: true,
                    edge_swipe_threshold: 20,
                },
                gesture: GestureSettings {
                    enable_gestures: true,
                    sensitivity: 1.0,
                    tap_timeout: 300,
                    long_press_timeout: 500,
                    swipe_threshold: 10.0,
                    pinch_threshold: 0.1,
                    rotate_threshold: 0.1,
                },
            })),
        }
    }
    
    /// Processes an input event
    pub fn process_event(&self, event: InputEvent) -> UiResult<()> {
        // Update the input state
        match &event.event_type {
            InputEventType::KeyPress | InputEventType::KeyRelease => {
                self.update_keyboard_state(&event)?;
            }
            InputEventType::ButtonPress | InputEventType::ButtonRelease | InputEventType::Motion | InputEventType::Scroll => {
                self.update_mouse_state(&event)?;
            }
            InputEventType::TouchBegin | InputEventType::TouchUpdate | InputEventType::TouchEnd => {
                self.update_touch_state(&event)?;
            }
            _ => {}
        }
        
        // Process gestures
        let mut gesture_event = None;
        
        {
            let mut recognizers = self.gesture_recognizers.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on gesture recognizers".to_string())
            })?;
            
            for recognizer in recognizers.iter_mut() {
                if let Some(e) = recognizer.process_event(&event) {
                    gesture_event = Some(e);
                    break;
                }
            }
        }
        
        // Dispatch the event to handlers
        let event_handlers = self.event_handlers.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on event handlers".to_string())
        })?;
        
        if let Some(handlers) = event_handlers.get(&event.event_type) {
            for handler in handlers {
                handler(&event)?;
            }
        }
        
        // Dispatch gesture event if any
        if let Some(gesture_event) = gesture_event {
            let event_handlers = self.event_handlers.read().map_err(|_| {
                UiError::LockError("Failed to acquire read lock on event handlers".to_string())
            })?;
            
            if let Some(handlers) = event_handlers.get(&gesture_event.event_type) {
                for handler in handlers {
                    handler(&gesture_event)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Updates the keyboard state
    fn update_keyboard_state(&self, event: &InputEvent) -> UiResult<()> {
        if let InputEventData::Key { key_code, key_sym, modifiers, state } = &event.data {
            let mut keyboard_state = self.keyboard_state.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on keyboard state".to_string())
            })?;
            
            if *state {
                keyboard_state.pressed_keys.insert(*key_code);
            } else {
                keyboard_state.pressed_keys.remove(key_code);
            }
            
            keyboard_state.modifiers = *modifiers;
            
            // Update caps lock and num lock state
            if *key_code == 66 { // Caps Lock key code
                if *state {
                    keyboard_state.caps_lock = !keyboard_state.caps_lock;
                }
            } else if *key_code == 77 { // Num Lock key code
                if *state {
                    keyboard_state.num_lock = !keyboard_state.num_lock;
                }
            }
        }
        
        Ok(())
    }
    
    /// Updates the mouse state
    fn update_mouse_state(&self, event: &InputEvent) -> UiResult<()> {
        let mut mouse_state = self.mouse_state.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on mouse state".to_string())
        })?;
        
        match &event.data {
            InputEventData::Button { button, position, state } => {
                mouse_state.position = *position;
                
                // Update button state
                match button {
                    1 => mouse_state.buttons.left = *state,
                    2 => mouse_state.buttons.middle = *state,
                    3 => mouse_state.buttons.right = *state,
                    8 => mouse_state.buttons.back = *state,
                    9 => mouse_state.buttons.forward = *state,
                    _ => {}
                }
            }
            InputEventData::Motion { position, delta } => {
                mouse_state.position = *position;
            }
            InputEventData::Scroll { position, delta } => {
                mouse_state.position = *position;
                mouse_state.scroll_delta = *delta;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Updates the touch state
    fn update_touch_state(&self, event: &InputEvent) -> UiResult<()> {
        if let InputEventData::Touch { id, position, pressure, size } = &event.data {
            let mut touch_state = self.touch_state.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on touch state".to_string())
            })?;
            
            match event.event_type {
                InputEventType::TouchBegin | InputEventType::TouchUpdate => {
                    // Add or update touch point
                    touch_state.touch_points.insert(*id, TouchPoint {
                        id: *id,
                        position: *position,
                        pressure: *pressure,
                        size: *size,
                    });
                }
                InputEventType::TouchEnd => {
                    // Remove touch point
                    touch_state.touch_points.remove(id);
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Registers an event handler
    pub fn register_event_handler(
        &self,
        event_type: InputEventType,
        handler: Box<dyn Fn(&InputEvent) -> UiResult<()> + Send + Sync>,
    ) -> UiResult<()> {
        let mut event_handlers = self.event_handlers.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on event handlers".to_string())
        })?;
        
        event_handlers.entry(event_type).or_insert_with(Vec::new).push(handler);
        
        Ok(())
    }
    
    /// Registers a gesture recognizer
    pub fn register_gesture_recognizer(
        &self,
        recognizer: Box<dyn GestureRecognizer + Send + Sync>,
    ) -> UiResult<()> {
        let mut gesture_recognizers = self.gesture_recognizers.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on gesture recognizers".to_string())
        })?;
        
        gesture_recognizers.push(recognizer);
        
        Ok(())
    }
    
    /// Gets the keyboard state
    pub fn get_keyboard_state(&self) -> UiResult<KeyboardState> {
        let keyboard_state = self.keyboard_state.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on keyboard state".to_string())
        })?;
        
        Ok(keyboard_state.clone())
    }
    
    /// Gets the mouse state
    pub fn get_mouse_state(&self) -> UiResult<MouseState> {
        let mouse_state = self.mouse_state.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on mouse state".to_string())
        })?;
        
        Ok(mouse_state.clone())
    }
    
    /// Gets the touch state
    pub fn get_touch_state(&self) -> UiResult<TouchState> {
        let touch_state = self.touch_state.read().map_err(|_| {
            UiError::LockError("Failed to acquire read lock on touch state".to_string())
        })?;
        
        Ok(touch_state.clone())
    }
    
    /// Updates the input settings
    pub fn update_settings(&self, settings_update: impl FnOnce(&mut InputSettings)) -> UiResult<()> {
        let mut settings = self.settings.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on settings".to_string())
        })?;
        
        settings_update(&mut settings);
        
        // Apply settings
        self.apply_settings(&settings)?;
        
        Ok(())
    }
    
    /// Applies the input settings
    fn apply_settings(&self, settings: &InputSettings) -> UiResult<()> {
        // Apply keyboard settings
        {
            let mut keyboard_state = self.keyboard_state.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on keyboard state".to_string())
            })?;
            
            keyboard_state.layout = settings.keyboard.layout.clone();
            keyboard_state.repeat_rate = settings.keyboard.repeat_rate;
            keyboard_state.repeat_delay = settings.keyboard.repeat_delay;
        }
        
        // Apply mouse settings
        {
            let mut mouse_state = self.mouse_state.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on mouse state".to_string())
            })?;
            
            mouse_state.speed = settings.mouse.speed;
            mouse_state.acceleration = settings.mouse.acceleration;
        }
        
        // Apply touch settings
        {
            let mut touch_state = self.touch_state.write().map_err(|_| {
                UiError::LockError("Failed to acquire write lock on touch state".to_string())
            })?;
            
            touch_state.enabled = settings.touch.enable_touch;
        }
        
        Ok(())
    }
}

impl Clone for KeyboardState {
    fn clone(&self) -> Self {
        Self {
            pressed_keys: self.pressed_keys.clone(),
            layout: self.layout.clone(),
            repeat_rate: self.repeat_rate,
            repeat_delay: self.repeat_delay,
            modifiers: self.modifiers,
            caps_lock: self.caps_lock,
            num_lock: self.num_lock,
        }
    }
}

impl Clone for MouseState {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            buttons: self.buttons,
            scroll_delta: self.scroll_delta,
            speed: self.speed,
            acceleration: self.acceleration,
        }
    }
}

impl Clone for TouchState {
    fn clone(&self) -> Self {
        Self {
            touch_points: self.touch_points.clone(),
            enabled: self.enabled,
            capabilities: self.capabilities,
        }
    }
}

impl Clone for TouchPoint {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            position: self.position,
            pressure: self.pressure,
            size: self.size,
        }
    }
}

impl UiComponent for InputIntegration {
    fn init(&self) -> UiResult<()> {
        // Set up event handlers for GTK events
        let self_clone = self.clone();
        
        // Connect to key press events
        self.app.connect_key_press_event(move |_, event| {
            let key_event = InputEvent {
                event_type: InputEventType::KeyPress,
                time: event.time(),
                source: InputSource::Keyboard,
                data: InputEventData::Key {
                    key_code: event.hardware_keycode(),
                    key_sym: event.keyval(),
                    modifiers: KeyboardModifiers {
                        shift: event.state().contains(gdk::ModifierType::SHIFT_MASK),
                        ctrl: event.state().contains(gdk::ModifierType::CONTROL_MASK),
                        alt: event.state().contains(gdk::ModifierType::MOD1_MASK),
                        super_key: event.state().contains(gdk::ModifierType::MOD4_MASK),
                        alt_gr: event.state().contains(gdk::ModifierType::MOD5_MASK),
                        meta: event.state().contains(gdk::ModifierType::META_MASK),
                    },
                    state: true,
                },
            };
            
            if let Err(e) = self_clone.process_event(key_event) {
                eprintln!("Failed to process key press event: {}", e);
            }
            
            gtk::Inhibit(false)
        });
        
        // Connect to key release events
        let self_clone = self.clone();
        self.app.connect_key_release_event(move |_, event| {
            let key_event = InputEvent {
                event_type: InputEventType::KeyRelease,
                time: event.time(),
                source: InputSource::Keyboard,
                data: InputEventData::Key {
                    key_code: event.hardware_keycode(),
                    key_sym: event.keyval(),
                    modifiers: KeyboardModifiers {
                        shift: event.state().contains(gdk::ModifierType::SHIFT_MASK),
                        ctrl: event.state().contains(gdk::ModifierType::CONTROL_MASK),
                        alt: event.state().contains(gdk::ModifierType::MOD1_MASK),
                        super_key: event.state().contains(gdk::ModifierType::MOD4_MASK),
                        alt_gr: event.state().contains(gdk::ModifierType::MOD5_MASK),
                        meta: event.state().contains(gdk::ModifierType::META_MASK),
                    },
                    state: false,
                },
            };
            
            if let Err(e) = self_clone.process_event(key_event) {
                eprintln!("Failed to process key release event: {}", e);
            }
            
            gtk::Inhibit(false)
        });
        
        // Similar connections would be made for mouse and touch events
        
        Ok(())
    }
    
    fn shutdown(&self) -> UiResult<()> {
        // Clean up resources
        let mut gesture_recognizers = self.gesture_recognizers.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on gesture recognizers".to_string())
        })?;
        
        gesture_recognizers.clear();
        
        let mut event_handlers = self.event_handlers.write().map_err(|_| {
            UiError::LockError("Failed to acquire write lock on event handlers".to_string())
        })?;
        
        event_handlers.clear();
        
        Ok(())
    }
}

impl Clone for InputIntegration {
    fn clone(&self) -> Self {
        Self {
            app: self.app.clone(),
            compositor: self.compositor.clone(),
            keyboard_state: self.keyboard_state.clone(),
            mouse_state: self.mouse_state.clone(),
            touch_state: self.touch_state.clone(),
            gesture_recognizers: self.gesture_recognizers.clone(),
            event_handlers: self.event_handlers.clone(),
            settings: self.settings.clone(),
        }
    }
}
