use super::object::{WaylandObject, ObjectId, ProtocolError, RequestContext}; // Changed
use super::surface::{Surface, SurfaceRole, Rect}; // Changed
use super::wire::WlArgument; // Changed
use std::sync::{Arc, Mutex, Weak};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubsurfaceSyncMode {
    Synchronized,
    Desynchronized,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SubsurfacePendingState {
    pub position: Option<(i32, i32)>, // Relative to parent surface
    // pub new_parent: Option<Weak<Surface>> // For re-parenting, not in core wl_subsurface
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SubsurfaceCommittedState {
    pub position: (i32, i32), // Always has a position once configured
    // pub parent: Weak<Surface>
}

// WlSubsurface itself.
// It is an interface to a wl_surface object, turning it into a subsurface.
// The wl_surface object (referred to as `surface` here) is the one providing content.
#[derive(Debug)]
pub struct WlSubsurface {
    id: ObjectId,
    version: u32,
    // The wl_surface object that this wl_subsurface is for.
    // This surface provides the content, size, damage, etc.
    surface: Arc<Surface>,
    // The parent wl_surface object.
    // Using Weak<Surface> for parent to avoid strong reference cycles if parent stores subsurfaces.
    // However, the parent *must* exist for the subsurface to be valid.
    // If parent is Weak, we need to upgrade it for every access.
    // Let's use Arc<Surface> for parent for now, and manage cycles carefully.
    // If Surface stores Vec<ObjectId> of subsurfaces, then Arc<Surface> here is fine.
    parent: Arc<Surface>,

    // Subsurface-specific state
    pending_state: Mutex<SubsurfacePendingState>,
    current_state: Mutex<SubsurfaceCommittedState>, // Position is relative to parent.
    sync_mode: Mutex<SubsurfaceSyncMode>,
}

impl WlSubsurface {
    pub fn new(
        id: ObjectId,
        version: u32,
        surface: Arc<Surface>,    // The surface becoming a subsurface
        parent: Arc<Surface>,     // The parent surface
    ) -> Result<Self, ProtocolError> {
        // A surface can only have one role.
        // wl_subsurface.get_subsurface docs:
        // "wl_surface@destroy will unmap and destroy the subsurface interface ..."
        // "... assigning a role to a wl_surface that already has a role is a wl_compositor error ..."
        // Here, 'surface' (the child) gets the Subsurface role.
        surface.set_role(SurfaceRole::Subsurface).map_err(|_e| {
            // TODO: This should be a specific wl_compositor error: ROLE
            ProtocolError::ImplementationError
        })?;

        // Check if parent already has a role that conflicts with being a parent (e.g. also subsurface)
        // This check might be more complex or handled by general role validation.
        // For now, assume parent can be a parent.

        Ok(Self {
            id,
            version,
            surface,
            parent,
            pending_state: Mutex::new(SubsurfacePendingState::default()),
            current_state: Mutex::new(SubsurfaceCommittedState::default()), // Default position (0,0)
            sync_mode: Mutex::new(SubsurfaceSyncMode::Synchronized), // Default is synchronized
        })
    }

    pub fn surface(&self) -> &Arc<Surface> {
        &self.surface
    }

    pub fn parent_surface(&self) -> &Arc<Surface> {
        &self.parent
    }

    pub fn is_synchronized(&self) -> bool {
        *self.sync_mode.lock().unwrap() == SubsurfaceSyncMode::Synchronized
    }

    // Called by parent Surface during its commit phase if synchronized.
    pub fn apply_pending_state(&self) {
        let mut current_state_guard = self.current_state.lock().unwrap();
        let mut pending_state_guard = self.pending_state.lock().unwrap();

        if let Some(pos) = pending_state_guard.position.take() {
            current_state_guard.position = pos;
        }
        // Other pending states like stacking order would be applied here.
    }

    // To get current committed position
    pub fn get_position(&self) -> (i32, i32) {
        self.current_state.lock().unwrap().position
    }
}

impl WaylandObject for WlSubsurface {
    fn id(&self) -> ObjectId { self.id }
    fn version(&self) -> u32 { self.version }
    fn interface_name(&self) -> &'static str { "wl_subsurface" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> { self }

    fn handle_request(
        &self,
        opcode: u16,
        args: Vec<WlArgument>,
        context: &mut RequestContext,
    ) -> Result<(), ProtocolError> {
        // TODO: Check if this WlSubsurface's associated wl_surface has been destroyed.
        // If so, many operations might be invalid or lead to errors.
        // The spec says: "If the wl_surface is destroyed, the wl_subsurface is/******/
        // implicitly destroyed." This means this object (WlSubsurface) should be cleaned up.

        match opcode {
            0 => { // destroy
                // When wl_subsurface is destroyed:
                // 1. Its associated wl_surface loses its subsurface role.
                // 2. It's detached from the parent.
                // 3. The wl_subsurface object itself is destroyed.
                self.surface.set_role(SurfaceRole::None).ok(); // Revert role, ignore error if already None or issue.
                self.parent.remove_subsurface(self.id)?; // Parent needs remove_subsurface(subsurface_obj_id)
                context.object_manager.destroy_object(self.id);
                Ok(())
            }
            1 => { // set_position(x: int, y: int)
                if args.len() < 2 { return Err(ProtocolError::InvalidArguments); }
                let x = match args[0] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                let y = match args[1] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };

                let mut pending_state_guard = self.pending_state.lock().unwrap();
                pending_state_guard.position = Some((x, y));
                Ok(())
            }
            2 => { // place_above(sibling: object)
                eprintln!("WlSubsurface {}: place_above - Unimplemented", self.id);
                // Would involve finding sibling WlSubsurface, then reordering in parent's list.
                Ok(())
            }
            3 => { // place_below(sibling: object)
                eprintln!("WlSubsurface {}: place_below - Unimplemented", self.id);
                Ok(())
            }
            4 => { // set_sync()
                *self.sync_mode.lock().unwrap() = SubsurfaceSyncMode::Synchronized;
                Ok(())
            }
            5 => { // set_desync()
                *self.sync_mode.lock().unwrap() = SubsurfaceSyncMode::Desynchronized;
                // If becoming desynchronized, any pending state on the wl_surface (content)
                // should be committed independently. This might mean triggering a commit on self.surface.
                // "If desynchronized, the wl_surface.commit on the subsurface surface is applied immediately."
                // This implies we might need to call self.surface.apply_commit() or similar.
                // However, wl_surface.commit is a client request. This is about when server applies it.
                // For now, just set the mode. The Surface's commit logic will consult this.
                Ok(())
            }
            _ => Err(ProtocolError::InvalidOpcode(opcode)),
        }
    }
}

impl Drop for WlSubsurface {
    fn drop(&mut self) {
        // Ensure role is cleared and it's unparented if not explicitly destroyed by client.
        // This can happen if client disconnects or only destroys parent surface.
        self.surface.set_role(SurfaceRole::None).ok();
        self.parent.remove_subsurface(self.id).ok(); // Best effort
        // println!("WlSubsurface {} dropped.", self.id);
    }
}
