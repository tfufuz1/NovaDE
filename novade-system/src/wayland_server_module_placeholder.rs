// src/wayland_server_module_placeholder.rs

use crate::input::focus::{
    ProcessedKeyEvent,
    ProcessedPointerMotionEvent,
    ProcessedPointerButtonEvent,
    ProcessedPointerScrollEvent,
    ProcessedTouchDownEvent,
    ProcessedTouchMotionEvent,
    ProcessedTouchUpEvent,
    ProcessedTouchFrameEvent,
    SurfaceId,
};
use crate::input::keyboard::{ModifiersState, KeyState};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WlSeatCapability {
    Pointer,
    Keyboard,
    Touch,
}

// Placeholder for keymap details
#[derive(Debug, Clone)]
pub struct StubbedKeymapDetails {
    pub format: String, // e.g., "xkb_v1"
    pub fd: i32,        // Dummy file descriptor
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct WaylandServerHandle;

impl WaylandServerHandle {
    pub fn new() -> Self {
        println!("WaylandServerHandle (Stub): Initialized.");
        Self
    }

    // --- Seat Capabilities ---
    pub fn update_seat_capabilities(&self, capabilities: &Vec<WlSeatCapability>) {
        println!("WaylandServerHandle (Stub): update_seat_capabilities called with: {:?}", capabilities);
        // In a real server: wl_seat.capabilities event sent to clients
    }

    // --- Keyboard Keymap and Repeat Info ---
    pub fn send_keyboard_keymap(&self, _surface_id: SurfaceId, keymap_details: &StubbedKeymapDetails) {
        println!("WaylandServerHandle (Stub): send_keyboard_keymap to surface. Details: {:?}", keymap_details);
        // In a real server: wl_keyboard.keymap(format, fd, size)
    }

    pub fn send_keyboard_repeat_info(&self, _surface_id: SurfaceId, rate: i32, delay: i32) {
        println!("WaylandServerHandle (Stub): send_keyboard_repeat_info to surface. Rate: {}, Delay: {}", rate, delay);
        // In a real server: wl_keyboard.repeat_info(rate, delay)
    }

    // --- Existing Keyboard Event Stubs ---
    pub fn send_keyboard_enter(&self, surface_id: SurfaceId, serial: u32, _current_keys: &HashSet<u32>, _current_mods: &ModifiersState) {
        println!(
            "WaylandServerHandle (Stub): send_keyboard_enter to Surface ID: {}, Serial: {}, Keys: (...), Mods: (...)",
            surface_id, serial
        );
    }

    pub fn send_keyboard_leave(&self, surface_id: SurfaceId, serial: u32) {
        println!("WaylandServerHandle (Stub): send_keyboard_leave from Surface ID: {}, Serial: {}", surface_id, serial);
    }

    pub fn send_keyboard_event_to_surface(&self, surface_id: SurfaceId, event: &ProcessedKeyEvent, serial: u32) {
         println!(
            "WaylandServerHandle (Stub): send_keyboard_event_to_surface (ProcessedKeyEvent) to Surface ID: {}, Keysym: {}, State: {:?}, Modifiers: {:?}, Time: {}, Serial: {}",
            surface_id, event.keysym, event.state, event.modifiers, event.time, serial
        );
    }

    pub fn send_keyboard_modifiers(&self, surface_id: SurfaceId, mods: &ModifiersState, serial: u32) {
        println!(
            "WaylandServerHandle (Stub): send_keyboard_modifiers to Surface ID: {}, Mods: {:?}, Serial: {}",
            surface_id, mods, serial
        );
    }

    // --- Existing Pointer Event Stubs ---
    pub fn send_pointer_enter(&self, surface_id: SurfaceId, x: f64, y: f64, serial: u32) {
        println!(
            "WaylandServerHandle (Stub): send_pointer_enter to Surface ID: {}, X: {}, Y: {}, Serial: {}",
            surface_id, x, y, serial
        );
    }

    pub fn send_pointer_leave(&self, surface_id: SurfaceId, serial: u32) {
        println!("WaylandServerHandle (Stub): send_pointer_leave from Surface ID: {}, Serial: {}", surface_id, serial);
    }

    pub fn send_pointer_motion(&self, surface_id: SurfaceId, event: &ProcessedPointerMotionEvent) {
        println!(
            "WaylandServerHandle (Stub): send_pointer_motion to Surface ID: {}, Time: {}, X: {}, Y: {}",
            surface_id, event.time, event.abs_x, event.abs_y
        );
    }

    pub fn send_pointer_button(&self, surface_id: SurfaceId, event: &ProcessedPointerButtonEvent, serial: u32) {
        println!(
            "WaylandServerHandle (Stub): send_pointer_button to Surface ID: {}, Time: {}, Button: {}, State: {:?}, Serial: {}",
            surface_id, event.time, event.button_code, event.state, serial
        );
    }

    pub fn send_pointer_axis(&self, surface_id: SurfaceId, event: &ProcessedPointerScrollEvent) {
        println!(
            "WaylandServerHandle (Stub): send_pointer_axis to Surface ID: {}, Time: {}, Dx: {}, Dy: {}, Source: {:?}",
            surface_id, event.time, event.delta_x, event.delta_y, event.source
        );
    }

    pub fn send_pointer_frame(&self, surface_id: SurfaceId) {
        println!("WaylandServerHandle (Stub): send_pointer_frame to Surface ID: {}", surface_id);
    }

    // --- Existing Touch Event Stubs ---
    pub fn send_touch_down(&self, surface_id: SurfaceId, event: &ProcessedTouchDownEvent, serial: u32) {
        println!(
            "WaylandServerHandle (Stub): send_touch_down to Surface ID: {}, Serial: {}, ID: {}, X: {}, Y: {}, Time: {}",
            surface_id, serial, event.id, event.x, event.y, event.time
        );
    }

    pub fn send_touch_up(&self, surface_id: SurfaceId, event: &ProcessedTouchUpEvent, serial: u32) {
        println!(
            "WaylandServerHandle (Stub): send_touch_up to Surface ID: {}, Serial: {}, ID: {}, Time: {}",
            surface_id, serial, event.id, event.time
        );
    }

    pub fn send_touch_motion(&self, surface_id: SurfaceId, event: &ProcessedTouchMotionEvent) {
        println!(
            "WaylandServerHandle (Stub): send_touch_motion to Surface ID: {}, ID: {}, X: {}, Y: {}, Time: {}",
            surface_id, event.id, event.x, event.y, event.time
        );
    }

    pub fn send_touch_frame(&self, surface_id: SurfaceId) {
        println!("WaylandServerHandle (Stub): send_touch_frame to Surface ID: {}", surface_id);
    }

    pub fn send_touch_cancel(&self, surface_id: SurfaceId) {
        println!("WaylandServerHandle (Stub): send_touch_cancel to Surface ID: {}", surface_id);
    }

    // --- Existing Seat Capability Stubs ---
    pub fn get_current_seat_capabilities(&self) -> Vec<WlSeatCapability> {
        println!("WaylandServerHandle (Stub): get_current_seat_capabilities called.");
        vec![WlSeatCapability::Pointer, WlSeatCapability::Keyboard, WlSeatCapability::Touch]
    }

    pub fn wl_seat_get_pointer_stub(&self, seat_name: &str) {
        println!("WaylandServerHandle (Stub): Client requested pointer for seat '{}'.", seat_name);
    }
    pub fn wl_seat_get_keyboard_stub(&self, seat_name: &str) {
        println!("WaylandServerHandle (Stub): Client requested keyboard for seat '{}'.", seat_name);
    }
    pub fn wl_seat_get_touch_stub(&self, seat_name: &str) {
        println!("WaylandServerHandle (Stub): Client requested touch for seat '{}'.", seat_name);
    }
}


#[derive(Debug, Clone)]
pub struct Rect { pub x: i32, pub y: i32, pub w: u32, pub h: u32 }

#[derive(Debug, Clone)]
pub struct StubbedSurfaceInfo {
    pub id: SurfaceId,
    pub x: i32, pub y: i32, pub width: u32, pub height: u32,
    pub z_order: i32,
    pub input_regions: Option<Vec<Rect>>,
}

#[derive(Debug, Clone)]
pub struct SurfaceManagerHandle;

impl SurfaceManagerHandle {
    pub fn new() -> Self {
        println!("SurfaceManagerHandle (Stub): Initialized.");
        Self
    }

    pub fn get_surface_info(&self, id: SurfaceId) -> Option<StubbedSurfaceInfo> {
        println!("SurfaceManagerHandle (Stub): get_surface_info(id: {}) called.", id);
        // Return a dummy surface for any requested ID for now
        Some(StubbedSurfaceInfo {
            id, x: 0, y: 0, width: 1024, height: 768, z_order: 0,
            input_regions: Some(vec![Rect { x: 0, y: 0, w: 1024, h: 768 }]),
        })
    }

    pub fn get_surfaces_at_coords(&self, x: f64, y: f64) -> Vec<StubbedSurfaceInfo> {
        println!("SurfaceManagerHandle (Stub): get_surfaces_at_coords(x: {}, y: {}) called.", x, y);
        let point_x = x as i32;
        let point_y = y as i32;
        let mut surfaces_found = Vec::new();

        let candidate_surfaces = vec![
            StubbedSurfaceInfo {
                id: 1, x: 0, y: 0, width: 800, height: 600, z_order: 0,
                input_regions: Some(vec![Rect { x:0, y:0, w:800, h:600}]),
            },
            StubbedSurfaceInfo {
                id: 2, x: 50, y: 50, width: 100, height: 100, z_order: 1,
                input_regions: Some(vec![Rect { x:0, y:0, w:100, h:100}]),
            },
            // Add a surface that might not have input regions defined
            StubbedSurfaceInfo {
                id: 3, x: 200, y: 200, width: 300, height: 300, z_order: -1, // lower z-order
                input_regions: None, // Whole surface is input region
            }
        ];

        for s in candidate_surfaces.iter() {
            let surface_global_x_end = s.x + s.width as i32;
            let surface_global_y_end = s.y + s.height as i32;

            if point_x >= s.x && point_x < surface_global_x_end &&
               point_y >= s.y && point_y < surface_global_y_end {
                if let Some(input_regions) = &s.input_regions {
                    if input_regions.is_empty() {
                         surfaces_found.push(s.clone());
                         continue;
                    }
                    for region in input_regions {
                        let region_global_x = s.x + region.x;
                        let region_global_y = s.y + region.y;
                        let region_global_x_end = region_global_x + region.w as i32;
                        let region_global_y_end = region_global_y + region.h as i32;

                        if point_x >= region_global_x && point_x < region_global_x_end &&
                           point_y >= region_global_y && point_y < region_global_y_end {
                            surfaces_found.push(s.clone());
                            break;
                        }
                    }
                } else {
                    surfaces_found.push(s.clone());
                }
            }
        }

        surfaces_found.sort_by(|a, b| b.z_order.cmp(&a.z_order));
        surfaces_found
    }
}
