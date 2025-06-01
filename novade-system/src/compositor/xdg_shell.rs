use crate::compositor::surface_management::{
    SurfaceId, Rectangle, Size as SurfaceSize, SurfaceRegistry, SurfaceState as WlSurfaceState,
    Point as WlPoint,
};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use bitflags::bitflags;

// --- Error Enum ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XdgShellError {
    InvalidRole(&'static str),
    AlreadyHasRole(SurfaceId),
    SurfaceNotFound(SurfaceId),
    XdgSurfaceNotFound(SurfaceId),
    NotAnXdgSurface(SurfaceId),
    NotAToplevel(SurfaceId),
    NotAPopup(SurfaceId),
    MismatchedSerial { expected: Option<Serial>, got: Serial },
    InvalidPositionerSettings(&'static str),
    InvalidPositionerSizeOrRect { field: &'static str, value1: i32, value2: i32 },
    Destroyed,
    PositionerNotFound(XdgObjectId),
    ParentXdgSurfaceNotFound(SurfaceId),
    ParentIsSelf(SurfaceId),
    InvalidPopupParent(SurfaceId),
    ParentNotAToplevel(SurfaceId),
    RepositionTokenMismatch,
}

// --- Serial Type ---
pub type Serial = u32;
// --- XDG Object ID ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XdgObjectId(pub u32);
// --- XDG Shell Global State (Placeholder) ---
#[derive(Debug, Default)]
pub struct XdgShellGlobalState;

// --- XDG Toplevel ---
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ToplevelStateValue { Maximized, Minimized, Resizing, Activated, Fullscreen, TiledLeft, TiledRight, TiledTop, TiledBottom }
#[derive(Debug, Clone, Default)]
pub struct ToplevelState { pub states: Vec<ToplevelStateValue> }
impl ToplevelState {
    pub fn new() -> Self { Default::default() }
    pub fn add(&mut self, state_val: ToplevelStateValue) { if !self.states.contains(&state_val) { self.states.push(state_val); } }
    pub fn remove(&mut self, state_val: ToplevelStateValue) { self.states.retain(|&s| s != state_val); }
    pub fn has(&self, state_val: ToplevelStateValue) -> bool { self.states.contains(&state_val) }
}
#[derive(Debug, Clone, Default)]
pub struct XdgToplevel {
    pub title: Option<String>, pub app_id: Option<String>, pub parent_id: Option<SurfaceId>,
    pub min_size: Option<SurfaceSize>, pub max_size: Option<SurfaceSize>,
    pub current_state: ToplevelState, pub pending_state: ToplevelState,
}
impl XdgToplevel { pub fn new() -> Self { Default::default() } }

// --- XDG Positioner and Popup ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor { #[default] None, Top, Bottom, Left, Right, TopLeft, TopRight, BottomLeft, BottomRight }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Gravity { #[default] None, Top, Bottom, Left, Right, TopLeft, TopRight, BottomLeft, BottomRight }
bitflags! { #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub struct ConstraintAdjustment: u32 {
    const NONE = 0; const SLIDE_X = 1; const SLIDE_Y = 2; const FLIP_X = 4; const FLIP_Y = 8; const RESIZE_X = 16; const RESIZE_Y = 32;
}}
#[derive(Debug, Clone, Copy, Default)]
pub struct XdgPositioner {
    pub size: SurfaceSize, pub anchor_rect: Rectangle, pub anchor: Anchor, pub gravity: Gravity,
    pub constraint_adjustment: ConstraintAdjustment, pub offset: (i32, i32),
}
impl XdgPositioner { pub fn new() -> Self { Default::default() } }

#[derive(Debug, Clone)]
pub struct XdgPopup {
    pub parent_xdg_surface_id: SurfaceId, pub positioner_settings: XdgPositioner,
    pub committed_position: WlPoint, pub current_size: SurfaceSize,
    pub grab_requested: bool, pub reactive: bool, pub parent_configure_serial: Option<Serial>,
    pub reposition_token: Option<u32>,
}
impl XdgPopup {
    pub fn new(parent_xdg_surface_id: SurfaceId, positioner: XdgPositioner, reactive: bool, parent_serial: Option<Serial>) -> Self {
        XdgPopup { parent_xdg_surface_id, positioner_settings: positioner,
                   committed_position: WlPoint { x:0, y:0}, current_size: SurfaceSize {width:0, height:0},
                   grab_requested: false, reactive, parent_configure_serial: parent_serial, reposition_token: None }
    }
}

// --- XDG Surface ---
#[derive(Debug, Clone)]
pub enum XdgSurfaceRole { Toplevel(XdgToplevel), Popup(XdgPopup) }
#[derive(Debug, Clone)]
pub struct XdgSurface {
    pub surface_id: SurfaceId, pub role: Option<XdgSurfaceRole>,
    pub window_geometry: Option<Rectangle>, pub acked_configure_serial: Option<Serial>,
    pub pending_configure_serial: Option<Serial>, pub configure_sent_this_commit_cycle: bool,
    pub destroyed: bool,
}
impl XdgSurface {
    pub fn new(surface_id: SurfaceId) -> Self {
        XdgSurface { surface_id, role: None, window_geometry: None, acked_configure_serial: None,
                     pending_configure_serial: None, configure_sent_this_commit_cycle: false, destroyed: false }
    }
    pub fn has_role(&self) -> bool { self.role.is_some() }
}

// --- XDG Shell State ---
/// Manages the state of XDG shell objects (surfaces, toplevels, popups, positioners).
/// It's intended to be used by Wayland request handlers.
#[derive(Debug)]
pub struct XdgShellState {
    surface_registry: Rc<RefCell<SurfaceRegistry>>,
    pub xdg_surfaces: HashMap<SurfaceId, XdgSurface>,
    positioners: HashMap<XdgObjectId, XdgPositioner>,
    next_xdg_object_id: u32, next_configure_serial: Serial,
}

impl XdgShellState {
    pub fn new(surface_registry: Rc<RefCell<SurfaceRegistry>>) -> Self {
        XdgShellState { surface_registry, xdg_surfaces: HashMap::new(), positioners: HashMap::new(),
                        next_xdg_object_id: 0, next_configure_serial: 0 }
    }
    fn generate_xdg_object_id(&mut self) -> XdgObjectId { let id = self.next_xdg_object_id; self.next_xdg_object_id += 1; XdgObjectId(id) }
    fn generate_serial(&mut self) -> Serial { let serial = self.next_configure_serial; self.next_configure_serial = self.next_configure_serial.wrapping_add(1); serial }

    // --- xdg_wm_base ---
    pub fn destroy_wm_base(&mut self) {}
    pub fn create_positioner(&mut self) -> XdgObjectId { let id = self.generate_xdg_object_id(); self.positioners.insert(id, XdgPositioner::new()); id }
    pub fn get_xdg_surface_object(&mut self, surface_id: SurfaceId) -> Result<XdgObjectId, XdgShellError> {
        if self.xdg_surfaces.contains_key(&surface_id) { return Err(XdgShellError::AlreadyHasRole(surface_id)); }
        if self.surface_registry.borrow().get_surface(surface_id).is_none() { return Err(XdgShellError::SurfaceNotFound(surface_id)); }
        self.xdg_surfaces.insert(surface_id, XdgSurface::new(surface_id)); Ok(self.generate_xdg_object_id())
    }
    pub fn pong(&mut self, _serial: Serial) {}

    // --- xdg_surface ---
    pub fn destroy_xdg_surface(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> {
        let xdg_surface = self.xdg_surfaces.get_mut(&xdg_surface_id).ok_or(XdgShellError::XdgSurfaceNotFound(xdg_surface_id))?;
        if xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        if matches!(xdg_surface.role, Some(XdgSurfaceRole::Popup(_))) {
            // Queue unparenting for the underlying wl_surface.
            // Actual update to parent's subsurfaces list occurs in SurfaceRegistry::commit_surface_hierarchy.
            self.surface_registry.borrow_mut().set_parent(xdg_surface_id, None).map_err(|_| XdgShellError::InvalidRole("Failed to unparent wl_surface for popup"))?;
        }
        xdg_surface.role = None; xdg_surface.destroyed = true;
        self.xdg_surfaces.remove(&xdg_surface_id); Ok(())
    }
    pub fn get_toplevel_role_object(&mut self, xdg_surface_id: SurfaceId) -> Result<XdgObjectId, XdgShellError> {
        let xdg_surface = self.xdg_surfaces.get_mut(&xdg_surface_id).ok_or(XdgShellError::XdgSurfaceNotFound(xdg_surface_id))?;
        if xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        if xdg_surface.has_role() { return Err(XdgShellError::AlreadyHasRole(xdg_surface_id)); }
        xdg_surface.role = Some(XdgSurfaceRole::Toplevel(XdgToplevel::new())); Ok(self.generate_xdg_object_id())
    }
    pub fn get_popup_role_object(&mut self, xdg_surface_id: SurfaceId, parent_xdg_surface_id: SurfaceId, positioner_id: XdgObjectId) -> Result<XdgObjectId, XdgShellError> {
        let xdg_surface = self.xdg_surfaces.get_mut(&xdg_surface_id).ok_or(XdgShellError::XdgSurfaceNotFound(xdg_surface_id))?;
        if xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        if xdg_surface.has_role() { return Err(XdgShellError::AlreadyHasRole(xdg_surface_id)); }
        if xdg_surface_id == parent_xdg_surface_id { return Err(XdgShellError::ParentIsSelf(xdg_surface_id));}
        if self.surface_registry.borrow().get_surface(parent_xdg_surface_id).is_none() { return Err(XdgShellError::SurfaceNotFound(parent_xdg_surface_id)); }
        let parent_xdg_surface = self.xdg_surfaces.get(&parent_xdg_surface_id).ok_or(XdgShellError::ParentXdgSurfaceNotFound(parent_xdg_surface_id))?;
        if parent_xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        if !parent_xdg_surface.has_role() { return Err(XdgShellError::InvalidPopupParent(parent_xdg_surface_id)); }
        let positioner = *self.positioners.get(&positioner_id).ok_or(XdgShellError::PositionerNotFound(positioner_id))?;
        let popup_role = XdgPopup::new(parent_xdg_surface_id, positioner, true, parent_xdg_surface.pending_configure_serial);
        xdg_surface.role = Some(XdgSurfaceRole::Popup(popup_role));
        // Set up wl_surface hierarchy. This sets pending_parent on the child wl_surface.
        // The actual parenting (parent.subsurfaces update) happens after the child's wl_surface is committed
        // and SurfaceRegistry::commit_surface_hierarchy() is called.
        self.surface_registry.borrow_mut().set_parent(xdg_surface_id, Some(parent_xdg_surface_id)).map_err(|_| XdgShellError::InvalidRole("Failed to set wl_surface parent for popup"))?;
        Ok(self.generate_xdg_object_id())
    }
    pub fn set_window_geometry(&mut self, xdg_surface_id: SurfaceId, geometry: Rectangle) -> Result<(), XdgShellError> {
        let xdg_surface = self.xdg_surfaces.get_mut(&xdg_surface_id).ok_or(XdgShellError::XdgSurfaceNotFound(xdg_surface_id))?;
        if xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        xdg_surface.window_geometry = Some(geometry); Ok(())
    }
    pub fn ack_configure(&mut self, xdg_surface_id: SurfaceId, serial: Serial) -> Result<(), XdgShellError> {
        let xdg_surface = self.xdg_surfaces.get_mut(&xdg_surface_id).ok_or(XdgShellError::XdgSurfaceNotFound(xdg_surface_id))?;
        if xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        if xdg_surface.pending_configure_serial != Some(serial) { return Err(XdgShellError::MismatchedSerial { expected: xdg_surface.pending_configure_serial, got: serial }); }
        xdg_surface.acked_configure_serial = Some(serial); xdg_surface.pending_configure_serial = None;
        xdg_surface.configure_sent_this_commit_cycle = false; Ok(())
    }
    /// Prepares for an xdg_surface.configure event by generating and storing a serial.
    /// This should be called by the compositor before it intends to send configure events.
    pub fn prepare_configure(&mut self, xdg_surface_id: SurfaceId) -> Result<Serial, XdgShellError> {
        let xdg_surface = self.xdg_surfaces.get_mut(&xdg_surface_id).ok_or(XdgShellError::XdgSurfaceNotFound(xdg_surface_id))?;
        if xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        let serial = self.generate_serial();
        xdg_surface.pending_configure_serial = Some(serial); xdg_surface.configure_sent_this_commit_cycle = true; Ok(serial)
    }

    // --- XdgToplevel ---
    fn get_toplevel_mut(&mut self, xdg_surface_id: SurfaceId) -> Result<&mut XdgToplevel, XdgShellError> {
        let xdg_surface = self.xdg_surfaces.get_mut(&xdg_surface_id).ok_or(XdgShellError::XdgSurfaceNotFound(xdg_surface_id))?;
        if xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        match xdg_surface.role { Some(XdgSurfaceRole::Toplevel(ref mut t)) => Ok(t), _ => Err(XdgShellError::NotAToplevel(xdg_surface_id)), }
    }
    pub fn destroy_toplevel_role(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> { Ok(())} // Placeholder from previous step
    pub fn set_toplevel_parent(&mut self, xdg_surface_id: SurfaceId, opt_parent_sid: Option<SurfaceId>) -> Result<(), XdgShellError> { Ok(()) } // Placeholder
    pub fn set_toplevel_title(&mut self, xdg_surface_id: SurfaceId, title: String) -> Result<(), XdgShellError> { self.get_toplevel_mut(xdg_surface_id)?.title = Some(title); Ok(()) }
    pub fn set_toplevel_app_id(&mut self, xdg_surface_id: SurfaceId, app_id: String) -> Result<(), XdgShellError> { self.get_toplevel_mut(xdg_surface_id)?.app_id = Some(app_id); Ok(()) }
    pub fn show_toplevel_window_menu(&mut self, xdg_surface_id: SurfaceId ) -> Result<(), XdgShellError> { self.get_toplevel_mut(xdg_surface_id)?; Ok(()) }
    pub fn toplevel_move(&mut self, xdg_surface_id: SurfaceId ) -> Result<(), XdgShellError> { self.get_toplevel_mut(xdg_surface_id)?; Ok(()) }
    pub fn toplevel_resize(&mut self, xdg_surface_id: SurfaceId ) -> Result<(), XdgShellError> { self.get_toplevel_mut(xdg_surface_id)?; Ok(()) }
    pub fn set_toplevel_max_size(&mut self, xdg_surface_id: SurfaceId, size: SurfaceSize) -> Result<(), XdgShellError> { self.get_toplevel_mut(xdg_surface_id)?.max_size = Some(size); Ok(()) }
    pub fn set_toplevel_min_size(&mut self, xdg_surface_id: SurfaceId, size: SurfaceSize) -> Result<(), XdgShellError> { self.get_toplevel_mut(xdg_surface_id)?.min_size = Some(size); Ok(()) }

    /// Updates the pending state of a toplevel (e.g., maximized, fullscreen) and
    /// ensures a configure event is queued by calling `prepare_configure`.
    /// This is typically called by client request handlers.
    pub fn update_toplevel_pending_state_and_configure(&mut self, xdg_surface_id: SurfaceId, state_value: ToplevelStateValue, add: bool) -> Result<(), XdgShellError> {
        let toplevel = self.get_toplevel_mut(xdg_surface_id)?;
        if add { toplevel.pending_state.add(state_value); } else { toplevel.pending_state.remove(state_value); }
        self.prepare_configure(xdg_surface_id)?; Ok(())
    }
    // Convenience methods for specific state changes
    pub fn set_toplevel_maximized(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> { self.update_toplevel_pending_state_and_configure(xdg_surface_id, ToplevelStateValue::Maximized, true) }
    pub fn unset_toplevel_maximized(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> { self.update_toplevel_pending_state_and_configure(xdg_surface_id, ToplevelStateValue::Maximized, false) }
    pub fn set_toplevel_fullscreen(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> { self.update_toplevel_pending_state_and_configure(xdg_surface_id, ToplevelStateValue::Fullscreen, true) }
    pub fn unset_toplevel_fullscreen(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> { self.update_toplevel_pending_state_and_configure(xdg_surface_id, ToplevelStateValue::Fullscreen, false) }
    pub fn set_toplevel_minimized(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> { self.update_toplevel_pending_state_and_configure(xdg_surface_id, ToplevelStateValue::Minimized, true) }

    /// Simulates the compositor deciding to send configure events to an XDG Toplevel.
    /// This applies the `pending_state` to `current_state` and calls `prepare_configure`
    /// to queue the `xdg_surface.configure` event.
    /// The window manager would call this when it needs to apply changes.
    pub fn send_toplevel_configure(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> {
        let xdg_surface = self.xdg_surfaces.get_mut(&xdg_surface_id).ok_or(XdgShellError::XdgSurfaceNotFound(xdg_surface_id))?;
        if xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        let toplevel = match xdg_surface.role {
            Some(XdgSurfaceRole::Toplevel(ref mut t)) => t,
            _ => return Err(XdgShellError::NotAToplevel(xdg_surface_id)),
        };
        toplevel.current_state = toplevel.pending_state.clone();
        let _serial = self.prepare_configure(xdg_surface_id)?;
        // log::info!("WM: Sending toplevel configure for {:?} (serial {}), states: {:?}", xdg_surface_id, _serial, toplevel.current_state.states);
        Ok(())
    }
    /// Simulates the compositor sending a close event to an XDG Toplevel.
    pub fn send_toplevel_close(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> {
        let _ = self.get_toplevel_mut(xdg_surface_id)?;
        // log::info!("WM: Sending close event to toplevel {:?}", xdg_surface_id);
        Ok(())
    }

    // --- XdgPopup ---
    fn get_popup_mut(&mut self, xdg_surface_id: SurfaceId) -> Result<&mut XdgPopup, XdgShellError> {
        let xdg_surface = self.xdg_surfaces.get_mut(&xdg_surface_id).ok_or(XdgShellError::XdgSurfaceNotFound(xdg_surface_id))?;
        if xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        match xdg_surface.role { Some(XdgSurfaceRole::Popup(ref mut p)) => Ok(p), _ => Err(XdgShellError::NotAPopup(xdg_surface_id)), }
    }
    pub fn destroy_popup_role(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> {
        let xdg_surface = self.xdg_surfaces.get_mut(&xdg_surface_id).ok_or(XdgShellError::XdgSurfaceNotFound(xdg_surface_id))?;
        if xdg_surface.destroyed { return Err(XdgShellError::Destroyed); }
        if !matches!(xdg_surface.role, Some(XdgSurfaceRole::Popup(_))) { return Err(XdgShellError::NotAPopup(xdg_surface_id)); }
        // Unparent the underlying wl_surface. This sets pending_parent = Some(None).
        // The actual removal from the old parent's subsurfaces list happens after the
        // (potentially next) commit of this wl_surface (if any) and then
        // SurfaceRegistry::commit_surface_hierarchy(), or implicitly if the wl_surface is destroyed.
        self.surface_registry.borrow_mut().set_parent(xdg_surface_id, None)
            .map_err(|_| XdgShellError::InvalidRole("Failed to unparent wl_surface for popup on destroy"))?;
        xdg_surface.role = None; Ok(())
    }
    pub fn grab_popup(&mut self, xdg_surface_id: SurfaceId, _seat_id: u32, _serial: Serial) -> Result<(), XdgShellError> {
        self.get_popup_mut(xdg_surface_id)?.grab_requested = true; Ok(())
    }
    pub fn reposition_popup(&mut self, xdg_surface_id: SurfaceId, positioner_id: XdgObjectId, token: u32) -> Result<(), XdgShellError> {
        let popup = self.get_popup_mut(xdg_surface_id)?;
        let new_positioner_settings = *self.positioners.get(&positioner_id).ok_or(XdgShellError::PositionerNotFound(positioner_id))?;
        popup.reposition_token = Some(token); popup.positioner_settings = new_positioner_settings; Ok(())
    }

    /// Simulates the compositor deciding to send configure events to an XDG Popup.
    /// This calculates the popup's position and size based on its positioner settings
    /// and parent geometry, then calls `prepare_configure` to queue the `xdg_surface.configure`.
    /// The window manager would call this, for example, after initial creation or when the parent moves/resizes.
    pub fn send_popup_configure(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> {
        let (parent_xdg_surface_id, positioner_settings) = {
            let popup = self.get_popup_mut(xdg_surface_id)?;
            (popup.parent_xdg_surface_id, popup.positioner_settings)
        };
        let parent_xdg_surface = self.xdg_surfaces.get(&parent_xdg_surface_id).ok_or(XdgShellError::ParentXdgSurfaceNotFound(parent_xdg_surface_id))?;
        let parent_geometry = parent_xdg_surface.window_geometry.unwrap_or_default();
        let new_x = parent_geometry.x + positioner_settings.offset.0; // Simplified
        let new_y = parent_geometry.y + positioner_settings.offset.1; // Simplified
        let new_width = if positioner_settings.size.width > 0 { positioner_settings.size.width } else { 64 };
        let new_height = if positioner_settings.size.height > 0 { positioner_settings.size.height } else { 64 };
        let popup_mut = self.get_popup_mut(xdg_surface_id)?;
        popup_mut.committed_position = WlPoint { x: new_x, y: new_y };
        popup_mut.current_size = SurfaceSize { width: new_width, height: new_height };
        let _serial = self.prepare_configure(xdg_surface_id)?;
        // log::info!("WM: Sending popup configure for {:?} (x:{}, y:{}, w:{}, h:{}), serial {}", xdg_surface_id, new_x, new_y, new_width, new_height, _serial);
        Ok(())
    }
    /// Simulates the compositor sending a popup_done event.
    pub fn send_popup_done(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> {
        let _ = self.get_popup_mut(xdg_surface_id)?;
        // log::info!("WM: Sending popup_done to popup {:?}", xdg_surface_id);
        Ok(())
    }
    /// Simulates the compositor sending a repositioned event.
    pub fn send_popup_repositioned(&mut self, xdg_surface_id: SurfaceId) -> Result<(), XdgShellError> {
        let popup = self.get_popup_mut(xdg_surface_id)?;
        let _reposition_token = popup.reposition_token.take();
        // log::info!("WM: Sending repositioned event to popup {:?}, token {:?}", xdg_surface_id, _reposition_token);
        Ok(())
    }

    // --- xdg_positioner requests ---
    /* ... methods from previous step ... */
    pub fn destroy_positioner(&mut self, xdg_positioner_id: XdgObjectId) -> Result<(), XdgShellError> { Ok(()) }
    pub fn set_positioner_size(&mut self, xdg_positioner_id: XdgObjectId, width: i32, height: i32) -> Result<(), XdgShellError> { Ok(()) }
    pub fn set_positioner_anchor_rect(&mut self, xdg_positioner_id: XdgObjectId, x: i32, y: i32, width: i32, height: i32) -> Result<(), XdgShellError> { Ok(()) }
    pub fn set_positioner_anchor(&mut self, xdg_positioner_id: XdgObjectId, anchor: Anchor) -> Result<(), XdgShellError> { Ok(()) }
    pub fn set_positioner_gravity(&mut self, xdg_positioner_id: XdgObjectId, gravity: Gravity) -> Result<(), XdgShellError> { Ok(()) }
    pub fn set_positioner_constraint_adjustment(&mut self, xdg_positioner_id: XdgObjectId, constraint_adjustment_val: u32) -> Result<(), XdgShellError> { Ok(()) }
    pub fn set_positioner_offset(&mut self, xdg_positioner_id: XdgObjectId, x: i32, y: i32) -> Result<(), XdgShellError> { Ok(()) }
    pub fn set_positioner_reactive(&mut self, xdg_positioner_id: XdgObjectId) -> Result<(), XdgShellError> { Ok(()) }
    pub fn set_positioner_parent_size(&mut self, xdg_positioner_id: XdgObjectId, _parent_width: i32, _parent_height: i32) -> Result<(), XdgShellError> { Ok(()) }
    pub fn set_positioner_parent_configure(&mut self, xdg_positioner_id: XdgObjectId, _serial: Serial) -> Result<(), XdgShellError> { Ok(()) }

    // --- Helpers ---
    pub fn get_xdg_surface_mut(&mut self, surface_id: SurfaceId) -> Result<&mut XdgSurface, XdgShellError> { Ok(self.xdg_surfaces.get_mut(&surface_id).unwrap())}
    pub fn get_positioner(&self, id: XdgObjectId) -> Result<&XdgPositioner, XdgShellError> { Ok(self.positioners.get(&id).unwrap())}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::surface_management::SurfaceRegistry;

    fn create_shell_state_with_one_positioner() -> (XdgShellState, XdgObjectId) {
        let registry = Rc::new(RefCell::new(SurfaceRegistry::new()));
        let mut state = XdgShellState::new(registry);
        let pos_id = state.create_positioner();
        (state, pos_id)
    }
    // ... other tests from previous steps ...
}
