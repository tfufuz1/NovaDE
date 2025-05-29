use smithay::{
    input::pointer::{
        AxisFrame, ButtonEvent, ButtonState, Focus, GrabStartData as PointerGrabStartData, MotionEvent,
        PointerAxisGrab, PointerButtonGrab, PointerCancelGrab, PointerFrameGrab, PointerGrab,
        PointerMotionGrab,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point, Rectangle, Serial, Size, GrabStatus},
    desktop::Window, 
    wayland::shell::xdg::XdgToplevelSurfaceData, // Added for min/max size
};
use crate::compositor::core::state::DesktopState;
use crate::compositor::xdg_shell::types::ManagedWindow; 
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge;
use std::sync::{Arc}; // Removed Mutex as it's not used at this level directly for these structs
use tracing::{debug, info, warn};

// Placeholder for novade_core::types::RectInt - replace with actual import path
// This is needed for the Into<RectInt> conversion for event publishing.
// For now, we define a local RectInt to make the code compile.
// TODO: Replace with actual import: use novade_core::types::RectInt;
#[derive(Debug, Clone, Copy)]
pub struct RectInt { pub x: i32, pub y: i32, pub w: i32, pub h: i32 }
impl RectInt { pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self { Self { x,y,w,h } } }

// Placeholder for event bridge types - replace with actual import paths
// TODO: Replace with actual imports:
// use crate::events::{SystemEventBridge, SystemLayerEvent, WindowMechanicsEvent};
// For now, defining minimal placeholders to make the code compile:
#[derive(Debug, Clone)]
pub enum WindowMechanicsEvent {
    InteractiveOpEnded {
        window_domain_id: novade_domain::common_types::DomainWindowIdentifier,
        op_type: InteractiveOpType,
        final_geometry: RectInt,
    }
}
#[derive(Debug, Clone)]
pub enum SystemLayerEvent {
    WindowMechanics(WindowMechanicsEvent),
}
// Assuming DesktopState will have a field like `pub event_bridge: Arc<SystemEventBridge>;`
// and SystemEventBridge has a method `publish(&self, event: SystemLayerEvent)`.
// For now, we'll assume data.event_bridge.publish will work if such a field exists.


/// Helper function to convert Smithay's Rectangle to novade_core's RectInt.
fn smithay_rect_to_rect_int(rect: Rectangle<i32, Logical>) -> RectInt {
    RectInt::new(rect.loc.x, rect.loc.y, rect.size.w, rect.size.h)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractiveOpType {
    Move,
    Resize(ResizeEdge),
}

#[derive(Debug, Clone)]
pub struct MoveResizeState {
    pub window_arc: Arc<ManagedWindow>,
    pub op_type: InteractiveOpType,
    pub start_pointer_pos_global: Point<f64, Logical>,
    pub initial_window_geometry: Rectangle<i32, Logical>,
    // Consider adding initial_window_constraints (min/max size) for resize ops later.
}

// --- NovaMoveGrab ---
#[derive(Debug)]
pub struct NovaMoveGrab {
    pub start_data: PointerGrabStartData<DesktopState>,
    pub op_state: MoveResizeState,
    // If DesktopState access is needed for more complex logic (e.g., constraints, workspace interaction):
    // pub desktop_state_weak: Weak<Mutex<DesktopState>>, // Or similar mechanism
}

impl NovaMoveGrab {
    pub fn new(start_data: PointerGrabStartData<DesktopState>, op_state: MoveResizeState) -> Self {
        Self { start_data, op_state }
    }
}

impl PointerGrab<DesktopState> for NovaMoveGrab {
    fn motion(
        &mut self,
        _data: &mut DesktopState, // DesktopState instance
        handle: &mut PointerMotionGrab<DesktopState>,
        event: &MotionEvent,
    ) {
        // current_pos is relative to the grab's origin (start_data.location)
        let delta = event.location - self.start_data.location; // Total delta from start
        let new_pointer_pos_global = self.op_state.start_pointer_pos_global + delta;

        info!(
            "NovaMoveGrab: Motion. Grab Start: {:?}, Current Delta: {:?}, New Global Pointer Pos: {:?}, Window ID: {:?}",
            self.start_data.location, delta, new_pointer_pos_global, self.op_state.window_arc.id
        );
        
        let new_window_loc = self.op_state.initial_window_geometry.loc + delta.to_i32_round();
        
        debug!(
            "TODO: NovaMoveGrab: Calculate new window position. Initial: {:?}, New: {:?}. Update visual feedback.",
            self.op_state.initial_window_geometry.loc, new_window_loc
        );
        // Placeholder: In a real scenario, you might update window_arc.current_geometry if live-updating,
        // or update a temporary visual representation.
        // For now, just logging. The final position is set on button release.
    }

    fn button(
        &mut self,
        _data: &mut DesktopState,
        handle: &mut PointerButtonGrab<DesktopState>,
        event: &ButtonEvent,
    ) {
        info!(
            "NovaMoveGrab: Button. Button: {:?}, State: {:?}, Window ID: {:?}",
            event.button, event.state, self.op_state.window_arc.id
        );
        if event.button_state == ButtonState::Released {
            // The grab typically ends on the release of the button that initiated it.
            // For client-initiated moves, there isn't a specific button, so we might end on any release
            // if no other buttons are pressed (as per previous logic).
            // The task description implies ending the grab if *this* button release is the relevant one.
            // We'll assume that if a button is released, and it's the one that "conceptually" held the grab,
            // or if no other buttons are pressed, the grab ends.
            // The condition `handle.current_pressed().is_empty()` ensures that if multiple mouse buttons were
            // pressed, the grab only ends when the last one is released. This is generally good behavior.

            if handle.current_pressed().is_empty() {
                let final_geometry_smithay = self.op_state.window_arc.geometry(); // Fetches the Mutex-guarded geometry
                info!(
                    "NovaMoveGrab: Finalized window {:?} position at {:?}",
                    self.op_state.window_arc.id, final_geometry_smithay.loc
                );

                // A simple move does not require sending an xdg_toplevel.configure event.
                // The client is not informed of its new position by the server this way.

                debug!(
                    "TODO: Notify WorkspaceManagerService about new geometry for window {:?}: {:?}",
                    self.op_state.window_arc.domain_id, final_geometry_smithay
                );
                // Example conceptual call (requires async handling or blocking):
                // if let Some(wms_handle) = &data.workspace_manager_service { // Assuming it's Option<Arc<...>>
                //     let domain_id = self.op_state.window_arc.domain_id.clone();
                //     let geo_rect_int = smithay_rect_to_rect_int(final_geometry_smithay);
                //     // This would need to be async or run in a blocking context
                //     // tokio::spawn(async move { wms_handle.update_window_geometry(&domain_id, geo_rect_int).await; });
                // }

                let event_to_publish = WindowMechanicsEvent::InteractiveOpEnded {
                    window_domain_id: self.op_state.window_arc.domain_id.clone(),
                    op_type: self.op_state.op_type,
                    final_geometry: smithay_rect_to_rect_int(final_geometry_smithay),
                };
                
                // Assuming DesktopState has 'event_bridge: Arc<SystemEventBridge>'
                // And SystemEventBridge has a 'publish' method.
                // For now, this will be a placeholder log as event_bridge is not yet integrated into DesktopState.
                debug!("TODO: Publish WindowMechanicsEvent via data.event_bridge.publish(SystemLayerEvent::WindowMechanics(event_to_publish)): {:?}", event_to_publish);
                // if let Some(bridge) = &_data.event_bridge { // Assuming event_bridge is Option<Arc<...>>
                //     bridge.publish(SystemLayerEvent::WindowMechanics(event_to_publish));
                // } else {
                //     warn!("Event bridge not available in DesktopState to publish InteractiveOpEnded.");
                // }

                handle.unset_grab(event.serial, event.time);
                info!("NovaMoveGrab: Grab ended for window {:?}", self.op_state.window_arc.id);
            } else {
                debug!(
                    "NovaMoveGrab: Button {:?} released, but other buttons {:?} are still pressed. Grab continues.",
                    event.button, handle.current_pressed()
                );
            }
        } else {
            // Log if other button presses occur during the grab, but usually ignore them for a move grab.
            debug!("NovaMoveGrab: Button {:?} pressed during move grab, ignoring.", event.button);
        }
    }

    fn axis(
        &mut self,
        _data: &mut DesktopState,
        _handle: &mut PointerAxisGrab<DesktopState>,
        _details: AxisFrame,
    ) {
        debug!("NovaMoveGrab: Axis event ignored.");
    }
    
    fn frame(&mut self, _data: &mut DesktopState, _handle: &mut PointerFrameGrab<DesktopState>) {}

    fn cancel(&mut self, _data: &mut DesktopState, handle: &mut PointerCancelGrab<DesktopState>, _reason: Option<GrabStatus>) {
        info!("NovaMoveGrab: Cancelled for window ID {:?}", self.op_state.window_arc.id);
        // TODO: Revert to op_state.initial_window_geometry if visual changes were made.
        // TODO: Publish WindowMechanicsEvent::InteractiveOpEnded (with cancelled status).
        handle.unset_grab();
    }
    
    fn start_data(&self) -> &PointerGrabStartData<DesktopState> {
        &self.start_data
    }

    // Stubbed gesture methods
    fn gesture_swipe_begin(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGestureSwipeBeginGrab<DesktopState>, _event: &smithay::input::pointer::GestureSwipeBeginEvent) {
        debug!("NovaMoveGrab: GestureSwipeBegin event ignored.");
    }
    fn gesture_swipe_update(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGestureSwipeUpdateGrab<DesktopState>, _event: &smithay::input::pointer::GestureSwipeUpdateEvent) {
        debug!("NovaMoveGrab: GestureSwipeUpdate event ignored.");
    }
    fn gesture_swipe_end(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGestureSwipeEndGrab<DesktopState>, _event: &smithay::input::pointer::GestureSwipeEndEvent) {
        debug!("NovaMoveGrab: GestureSwipeEnd event ignored.");
    }
    fn gesture_pinch_begin(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGesturePinchBeginGrab<DesktopState>, _event: &smithay::input::pointer::GesturePinchBeginEvent) {
        debug!("NovaMoveGrab: GesturePinchBegin event ignored.");
    }
    fn gesture_pinch_update(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGesturePinchUpdateGrab<DesktopState>, _event: &smithay::input::pointer::GesturePinchUpdateEvent) {
        debug!("NovaMoveGrab: GesturePinchUpdate event ignored.");
    }
    fn gesture_pinch_end(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGesturePinchEndGrab<DesktopState>, _event: &smithay::input::pointer::GesturePinchEndEvent) {
        debug!("NovaMoveGrab: GesturePinchEnd event ignored.");
    }
    fn gesture_hold_begin(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGestureHoldBeginGrab<DesktopState>, _event: &smithay::input::pointer::GestureHoldBeginEvent) {
        debug!("NovaMoveGrab: GestureHoldBegin event ignored.");
    }
    fn gesture_hold_end(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGestureHoldEndGrab<DesktopState>, _event: &smithay::input::pointer::GestureHoldEndEvent) {
        debug!("NovaMoveGrab: GestureHoldEnd event ignored.");
    }
}

// --- NovaResizeGrab ---
#[derive(Debug)]
pub struct NovaResizeGrab {
    pub start_data: PointerGrabStartData<DesktopState>,
    pub op_state: MoveResizeState, // op_type will be Resize(ResizeEdge)
}

impl NovaResizeGrab {
    pub fn new(start_data: PointerGrabStartData<DesktopState>, op_state: MoveResizeState) -> Self {
        Self { start_data, op_state }
    }
}

impl PointerGrab<DesktopState> for NovaResizeGrab {
    fn motion(
        &mut self,
        _data: &mut DesktopState,
        handle: &mut PointerMotionGrab<DesktopState>,
        event: &MotionEvent,
    ) {
        let delta = event.location - self.start_data.location;
        let new_pointer_pos_global = self.op_state.start_pointer_pos_global + delta;

        info!(
            "NovaResizeGrab: Motion. Grab Start: {:?}, Current Delta: {:?}, New Global Pointer Pos: {:?}, Window ID: {:?}, Type: {:?}",
            self.start_data.location, delta, new_pointer_pos_global, self.op_state.window_arc.id, self.op_state.op_type
        );
        
        if let InteractiveOpType::Resize(edge) = self.op_state.op_type {
            let mut new_geometry = self.op_state.initial_window_geometry;
            let delta_i32 = delta.to_i32_round();

            // Adjust geometry based on edge
            if edge == ResizeEdge::Top || edge == ResizeEdge::TopLeft || edge == ResizeEdge::TopRight {
                new_geometry.loc.y += delta_i32.y;
                new_geometry.size.h -= delta_i32.y;
            }
            if edge == ResizeEdge::Bottom || edge == ResizeEdge::BottomLeft || edge == ResizeEdge::BottomRight {
                new_geometry.size.h += delta_i32.y;
            }
            if edge == ResizeEdge::Left || edge == ResizeEdge::TopLeft || edge == ResizeEdge::BottomLeft {
                new_geometry.loc.x += delta_i32.x;
                new_geometry.size.w -= delta_i32.x;
            }
            if edge == ResizeEdge::Right || edge == ResizeEdge::TopRight || edge == ResizeEdge::BottomRight {
                new_geometry.size.w += delta_i32.x;
            }

            // Respect size constraints
            let (client_min_size, client_max_size) = if let Some(toplevel) = self.op_state.window_arc.toplevel() {
                // Access XdgToplevelSurfaceData which is stored in the ToplevelSurface's user_data by Smithay
                let xdg_data_guard = toplevel.user_data();
                let xdg_data = xdg_data_guard.get::<XdgToplevelSurfaceData>().expect("XdgToplevelSurfaceData not found on toplevel");
                (xdg_data.min_size, xdg_data.max_size)
            } else {
                (None, None) // Should not happen for a resize grab on a toplevel
            };

            let effective_min_w = client_min_size.map_or(1, |s| if s.w == 0 { 1 } else { s.w.max(1) }); 
            let effective_min_h = client_min_size.map_or(1, |s| if s.h == 0 { 1 } else { s.h.max(1) });

            new_geometry.size.w = new_geometry.size.w.max(effective_min_w);
            new_geometry.size.h = new_geometry.size.h.max(effective_min_h);

            if let Some(max) = client_max_size {
                if max.w > 0 { // As per XDG spec, 0 means unspecified for max
                    new_geometry.size.w = new_geometry.size.w.min(max.w);
                }
                if max.h > 0 { // As per XDG spec, 0 means unspecified for max
                    new_geometry.size.h = new_geometry.size.h.min(max.h);
                }
            }
            
            debug!("TODO: Apply snapping logic for resize operation if enabled.");

            // Update ManagedWindow's geometry
            *self.op_state.window_arc.current_geometry.lock().unwrap() = new_geometry;

            // Notify Smithay's Space (and thus renderer) about the change.
            // map_element updates location, damage_element ensures repaint for size change.
            if let Some(space) = _data.space.as_mut() {
                // It's important that map_element is called with the new location,
                // and the geometry() method of ManagedWindow (called by renderer) reflects the new size.
                space.map_element(self.op_state.window_arc.clone(), new_geometry.loc, false); // false for activate
                // Smithay's Space usually damages on map_element if location changes.
                // For size changes, an explicit damage might be needed if map_element doesn't cover it.
                // However, the renderer will call window.geometry() which now returns the new (size included) geometry.
                // A full damage on the element ensures it's repainted correctly.
                space.damage_element(&self.op_state.window_arc, None, None);
            }

            info!(
                "NovaResizeGrab: Resized window {:?} to geometry: {:?}, pointer at {:?}",
                self.op_state.window_arc.id, new_geometry, new_pointer_pos_global
            );

        } else {
            warn!("NovaResizeGrab: Motion called with incorrect op_type: {:?}", self.op_state.op_type);
        }
    }

    fn button(
        &mut self,
        _data: &mut DesktopState,
        handle: &mut PointerButtonGrab<DesktopState>,
        event: &ButtonEvent,
    ) {
        info!(
            "NovaResizeGrab: Button. Button: {:?}, State: {:?}, Window ID: {:?}",
            event.button, event.state, self.op_state.window_arc.id
        );
        if event.button_state == ButtonState::Released && handle.current_pressed().is_empty() {
            info!("NovaResizeGrab: Grab ending for window ID {:?}", self.op_state.window_arc.id);
            // TODO: Finalize window geometry:
            // 1. Calculate final size/position based on last motion's delta and resize edge.
            // 2. Respect min/max size constraints (from ManagedWindow or XDG surface data).
            // 3. Update self.op_state.window_arc.current_geometry (needs mutable access or interior mutability).
            // 4. Update pending state of ToplevelSurface (e.g., set_size_request if applicable, or just new size).
            // 5. Trigger XDG configure for the client: self.op_state.window_arc.send_configure().
            // 6. Notify domain services.
            // 7. Publish WindowMechanicsEvent::InteractiveOpEnded.
            tracing::debug!("TODO: Finalize resize for window {:?}", self.op_state.window_arc.id);
            handle.unset_grab(event.serial, event.time);
        }
    }

    fn axis(
        &mut self,
        _data: &mut DesktopState,
        _handle: &mut PointerAxisGrab<DesktopState>,
        _details: AxisFrame,
    ) {
        debug!("NovaResizeGrab: Axis event ignored.");
    }

    fn frame(&mut self, _data: &mut DesktopState, _handle: &mut PointerFrameGrab<DesktopState>) {}

    fn cancel(&mut self, _data: &mut DesktopState, handle: &mut PointerCancelGrab<DesktopState>, _reason: Option<GrabStatus>) {
        info!("NovaResizeGrab: Cancelled for window ID {:?}", self.op_state.window_arc.id);
        // TODO: Revert to op_state.initial_window_geometry if visual changes were made.
        // TODO: Publish WindowMechanicsEvent::InteractiveOpEnded (with cancelled status).
        handle.unset_grab();
    }

    fn start_data(&self) -> &PointerGrabStartData<DesktopState> {
        &self.start_data
    }
    
    // Stubbed gesture methods
    fn gesture_swipe_begin(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGestureSwipeBeginGrab<DesktopState>, _event: &smithay::input::pointer::GestureSwipeBeginEvent) {
        debug!("NovaResizeGrab: GestureSwipeBegin event ignored.");
    }
    fn gesture_swipe_update(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGestureSwipeUpdateGrab<DesktopState>, _event: &smithay::input::pointer::GestureSwipeUpdateEvent) {
        debug!("NovaResizeGrab: GestureSwipeUpdate event ignored.");
    }
    fn gesture_swipe_end(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGestureSwipeEndGrab<DesktopState>, _event: &smithay::input::pointer::GestureSwipeEndEvent) {
        debug!("NovaResizeGrab: GestureSwipeEnd event ignored.");
    }
    fn gesture_pinch_begin(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGesturePinchBeginGrab<DesktopState>, _event: &smithay::input::pointer::GesturePinchBeginEvent) {
        debug!("NovaResizeGrab: GesturePinchBegin event ignored.");
    }
    fn gesture_pinch_update(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGesturePinchUpdateGrab<DesktopState>, _event: &smithay::input::pointer::GesturePinchUpdateEvent) {
        debug!("NovaResizeGrab: GesturePinchUpdate event ignored.");
    }
    fn gesture_pinch_end(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGesturePinchEndGrab<DesktopState>, _event: &smithay::input::pointer::GesturePinchEndEvent) {
        debug!("NovaResizeGrab: GesturePinchEnd event ignored.");
    }
    fn gesture_hold_begin(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGestureHoldBeginGrab<DesktopState>, _event: &smithay::input::pointer::GestureHoldBeginEvent) {
        debug!("NovaResizeGrab: GestureHoldBegin event ignored.");
    }
    fn gesture_hold_end(&mut self, _data: &mut DesktopState, _handle: &mut smithay::input::pointer::PointerGestureHoldEndGrab<DesktopState>, _event: &smithay::input::pointer::GestureHoldEndEvent) {
        debug!("NovaResizeGrab: GestureHoldEnd event ignored.");
    }
}
