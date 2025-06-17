use super::object::{ObjectId, WaylandObject, ProtocolError, RequestContext}; // Changed
use super::wire::WlArgument; // Changed
use super::buffer::{WlBuffer, BufferId}; // Changed
use std::sync::Arc;

// --- Type Aliases and Basic Enums/Structs ---
pub type SurfaceId = ObjectId;
// BufferId is now defined in buffer.rs, but Surface will store Arc<WlBuffer>

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRole {
    None,
    Toplevel,
    Subsurface,
    Cursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SurfaceInternalState {
    Alive,
    DestroyedPendingChanges,
    Destroyed,
}

// --- Surface State Management ---

#[derive(Clone, Default)] // Derive Clone for SurfacePendingState
pub struct SurfacePendingState {
    pub buffer: Option<Arc<WlBuffer>>, // Changed from BufferId to Arc<WlBuffer>
    pub damage_surface: Vec<Rect>,
    pub damage_buffer: Vec<Rect>,
    pub opaque_region: Option<Vec<Rect>>,
    pub input_region: Option<Vec<Rect>>,
    pub buffer_transform: i32,
    pub buffer_scale: i32,
}

// Manual Debug impl for SurfacePendingState because Arc<WlBuffer> makes it non-trivial
impl std::fmt::Debug for SurfacePendingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurfacePendingState")
         .field("buffer_id", &self.buffer.as_ref().map(|b| b.id()))
         .field("damage_surface", &self.damage_surface)
         .field("damage_buffer", &self.damage_buffer)
         // ... other fields
         .finish()
    }
}


#[derive(Clone, Default)] // Derive Clone for SurfaceCommittedState
pub struct SurfaceCommittedState {
    pub buffer: Option<Arc<WlBuffer>>, // Changed from BufferId to Arc<WlBuffer>
    pub damage_surface: Vec<Rect>,
    pub damage_buffer: Vec<Rect>,
    pub opaque_region: Option<Vec<Rect>>,
    pub input_region: Option<Vec<Rect>>,
    pub buffer_transform: i32,
    pub buffer_scale: i32,
}

// Manual Debug impl for SurfaceCommittedState
impl std::fmt::Debug for SurfaceCommittedState {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurfaceCommittedState")
         .field("buffer_id", &self.buffer.as_ref().map(|b| b.id()))
         .field("damage_surface", &self.damage_surface)
         .field("damage_buffer", &self.damage_buffer)
         // ... other fields
         .finish()
    }
}


// --- Surface Struct ---
// Surface needs internal mutability for pending_state and current_state if handle_request takes &self
// and these states are directly modified.
// For now, we'll assume handle_request gets &mut self by some mechanism if direct mutation is desired,
// or that it returns actions to be applied.
// The current WaylandObject trait takes &self. Surface will use Mutex for its states.
use std::sync::Mutex;

#[derive(Debug)] // Can't auto-derive due to Mutex fields if we go that route
pub struct Surface {
    id: SurfaceId,
    version: u32,
    // These states need to be modifiable by handle_request(&self, ...)
    // if WaylandObject trait is not changed to take &mut self.
    // For now, let's assume handle_request is changed to &mut self or Surface is not directly Arc<dyn WaylandObject>
    // but Arc<Mutex<Surface>>.
    // The subtask says "WaylandObject for Surface", and handle_request was changed to take &self and RequestContext.
    // This means Surface's mutable fields must be behind Mutex/RwLock.
    internal_state: Mutex<SurfaceInternalState>,
    role: Mutex<Option<SurfaceRole>>,
    pending: Mutex<SurfacePendingState>,
    current: Mutex<SurfaceCommittedState>,
    subsurface_ids: Mutex<Vec<ObjectId>>, // IDs of WlSubsurface objects
}


impl Surface {
    pub fn new(id: SurfaceId, version: u32) -> Self {
        Surface {
            id,
            version,
            internal_state: Mutex::new(SurfaceInternalState::Alive),
            role: Mutex::new(None),
            pending: Mutex::new(SurfacePendingState::default()),
            current: Mutex::new(SurfaceCommittedState::default()),
            subsurface_ids: Mutex::new(Vec::new()),
        }
    }

    // Called by WlSubcompositor when a WlSubsurface is created for this parent.
    pub fn add_subsurface(&self, subsurface_object_id: ObjectId) -> Result<(), ProtocolError> {
        // TODO: Check for cycles: if subsurface_object_id's surface is this surface or an ancestor.
        // This is complex and usually handled by preventing a surface from being its own parent/ancestor.
        let mut subsurfaces = self.subsurface_ids.lock().unwrap();
        if !subsurfaces.contains(&subsurface_object_id) {
            subsurfaces.push(subsurface_object_id);
        }
        Ok(())
    }

    // Called by WlSubsurface when it's destroyed or dropped.
    pub fn remove_subsurface(&self, subsurface_object_id: ObjectId) -> Result<(), ProtocolError> {
        let mut subsurfaces = self.subsurface_ids.lock().unwrap();
        subsurfaces.retain(|&id| id != subsurface_object_id);
        Ok(())
    }

    // Helper to get subsurface states during commit. Needs RequestContext to fetch WlSubsurface objects.
    fn commit_synchronized_subsurfaces(&self, context: &RequestContext) {
        let subsurface_ids_guard = self.subsurface_ids.lock().unwrap();
        for subsurface_obj_id in subsurface_ids_guard.iter() {
            match context.object_manager.get_typed_object::<super::subsurface::WlSubsurface>(*subsurface_obj_id) { // Changed
                Ok(subsurface_arc) => {
                    if subsurface_arc.is_synchronized() {
                        subsurface_arc.apply_pending_state();
                        // Recursively commit the actual surface content of the subsurface
                        // This is important: the wl_subsurface commit applies its own state (pos, sync)
                        // AND then the wl_surface it represents needs to commit its content.
                        // The Wayland spec says: "Changes to a wl_subsurface are applied when the parent surface is committed."
                        // "Committing a synchronized subsurface will cache the wl_surface state to be applied when the parent
                        //  commits [...] If the subsurface is desynchronized, the wl_surface.commit on the subsurface surface
                        //  is applied immediately."
                        // This means if a subsurface is synchronized, its *wl_surface's* pending state is also cached.
                        // When the parent commits, the subsurface's pending (pos, etc) is applied, AND its wl_surface's
                        // pending state (buffer, damage) is applied.
                        subsurface_arc.surface().apply_commit_from_parent(context); // New method needed on Surface
                    }
                }
                Err(e) => {
                    eprintln!("Error getting WlSubsurface object {} during parent commit: {:?}", subsurface_obj_id, e);
                }
            }
        }
    }

    // New method for Surface, called by parent or by its own commit if not a subsurface.
    // It needs RequestContext if it has subsurfaces itself.
    pub(crate) fn apply_commit_from_parent(&self, context: &RequestContext) {
        self.apply_commit_internal(context);
    }

    pub fn is_destroyed(&self) -> bool {
        let state = self.internal_state.lock().unwrap();
        matches!(*state, SurfaceInternalState::Destroyed | SurfaceInternalState::DestroyedPendingChanges)
    }

    // Example role setting, adapted for internal mutability
    pub fn set_role(&self, role_to_set: SurfaceRole) -> Result<(), String> {
        let mut role_guard = self.role.lock().unwrap();
        if role_guard.is_some() && role_to_set != SurfaceRole::None {
            return Err("Surface already has a role".to_string());
        }
        *role_guard = Some(role_to_set);
        Ok(())
    }


    fn apply_commit(&self) { // Takes &self due to internal mutability
        // Renamed to apply_commit_internal to distinguish from the public commit request handler
        // This internal version takes RequestContext for subsurfaces.
        // The public commit request (opcode 6) will call this.
    }

    // The main commit logic, now internal.
    fn apply_commit_internal(&self, context: &RequestContext) { // Takes RequestContext
        let mut pending_state = self.pending.lock().unwrap();
        let mut current_state = self.current.lock().unwrap();
        let mut internal_state_guard = self.internal_state.lock().unwrap();

        if *internal_state_guard == SurfaceInternalState::DestroyedPendingChanges || *internal_state_guard == SurfaceInternalState::Destroyed {
            if pending_state.buffer.is_some() {
                // Client attached a buffer to a dying/dead surface. Release it immediately.
                if let Some(buffer_to_release) = pending_state.buffer.take() {
                    buffer_to_release.mark_as_released();
                }
            }
        }

        // Release previous buffer if different from new one
        if current_state.buffer.as_ref().map(Arc::as_ptr) != pending_state.buffer.as_ref().map(Arc::as_ptr) {
            if let Some(old_buffer) = current_state.buffer.take() {
                old_buffer.mark_as_released();
            }
        }

        *current_state = pending_state.clone(); // Atomically apply pending state
        *pending_state = SurfacePendingState::default(); // Reset pending state

        if *internal_state_guard == SurfaceInternalState::DestroyedPendingChanges {
            if current_state.buffer.is_none() {
                *internal_state_guard = SurfaceInternalState::Destroyed;
            }
        }

        // After applying own state, commit synchronized subsurfaces
        // This needs to be done carefully to avoid issues if a subsurface's surface is this surface (cycles).
        // The check for surface_id == parent_surface_id in WlSubcompositor helps, but deeper cycles are possible.
        // For now, assume no direct cycles of a surface being its own subsurface's content provider.
        if *self.role.lock().unwrap() != Some(SurfaceRole::Subsurface) ||
           !context.object_manager.get_typed_object::<crate::wayland::subsurface::WlSubsurface>(self.id)
               // This check is tricky: if this surface IS a subsurface, its commit is handled differently.
               // If this surface is a subsurface, its content commit is either immediate (desync)
               // or triggered by its parent (sync). It should not typically trigger further sub-commits itself
               // unless it's also a parent to other subsurfaces (which is allowed).
               // The crucial part is that a wl_surface.commit from a client on a subsurface's wl_surface
               // might be cached if that subsurface is synchronized.
               // Let's simplify: All surfaces, when they commit, also trigger their sync subsurfaces.
               .map_or(false, |ss_obj| ss_obj.is_synchronized())
        {
             self.commit_synchronized_subsurfaces(context);
        }
    }
}

impl WaylandObject for Surface {
    fn id(&self) -> ObjectId { self.id }
    fn version(&self) -> u32 { self.version }
    fn interface_name(&self) -> &'static str { "wl_surface" }

    fn handle_request(
        &self, // Note: &self, requires internal mutability for state changes
        opcode: u16,
        args: Vec<WlArgument>,
        context: &mut RequestContext,
    ) -> Result<(), ProtocolError> {
        let mut internal_state_guard = self.internal_state.lock().unwrap();

        if *internal_state_guard == SurfaceInternalState::Destroyed && opcode != 0 {
            eprintln!("wl_surface {}: request (opcode {}) received while fully destroyed, ignoring.", self.id, opcode);
            return Ok(());
        }
        // If DestroyedPendingChanges, some operations might still be allowed or modified (e.g. commit)

        match opcode {
            0 => { // destroy
                if *internal_state_guard == SurfaceInternalState::Alive {
                    let current_buffer_is_some = self.current.lock().unwrap().buffer.is_some();
                    let pending_buffer_is_some = self.pending.lock().unwrap().buffer.is_some();

                    if current_buffer_is_some || pending_buffer_is_some {
                        *internal_state_guard = SurfaceInternalState::DestroyedPendingChanges;
                    } else {
                        *internal_state_guard = SurfaceInternalState::Destroyed;
                    }
                    // Actual removal from ObjectManager might be triggered by a signal or event
                    // after this state change is fully processed and effects (like buffer release) are done.
                }
                Ok(())
            }
            1 => { // attach(buffer: object | null, x: int, y: int)
                if *internal_state_guard == SurfaceInternalState::Destroyed { return Ok(()); }

                let buffer_arg_id = match args.get(0) {
                    Some(WlArgument::Object(id)) => *id, // Can be 0 for null buffer
                    _ => return Err(ProtocolError::InvalidArguments), // Must be an object
                };

                let mut pending_state = self.pending.lock().unwrap();
                if buffer_arg_id == 0 { // Detach (null buffer)
                    if let Some(old_pending_buffer) = pending_state.buffer.take() {
                        // If it was pending, it's not used by current state yet.
                        // Should it be marked released? If it was never committed, it was never "used" by surface.
                        // For simplicity, if it's detached before commit, it's just gone from pending.
                        // If a committed buffer is detached via pending state, apply_commit handles release.
                    }
                } else {
                    // Get the WlBuffer object from ObjectManager using the typed getter
                    let wl_buffer: Arc<WlBuffer> = context.object_manager.get_typed_object(buffer_arg_id)?;

                    wl_buffer.mark_as_used(); // Mark as used by this surface (pending commit)
                    pending_state.buffer = Some(wl_buffer);
                }
                // x and y are ignored for wl_surface.attach.
                Ok(())
            }
            2 => { // damage(x: int, y: int, width: int, height: int)
                if *internal_state_guard == SurfaceInternalState::Destroyed { return Ok(()); }
                if args.len() < 4 { return Err(ProtocolError::InvalidArguments); }
                let x = match args[0] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                let y = match args[1] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                let width = match args[2] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                let height = match args[3] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                if width < 0 || height < 0 { return Err(ProtocolError::InvalidArguments); }

                let mut pending_state = self.pending.lock().unwrap();
                pending_state.damage_surface.push(Rect { x, y, width, height });
                Ok(())
            }
            3 => { /* frame */ eprintln!("wl_surface {}: frame - Unimplemented", self.id); Ok(()) }
            4 => { /* set_opaque_region */ eprintln!("wl_surface {}: set_opaque_region - Unimplemented", self.id); Ok(()) }
            5 => { /* set_input_region */ eprintln!("wl_surface {}: set_input_region - Unimplemented", self.id); Ok(()) }
            6 => { // commit
                // If destroyed, commit might still process buffer release.
                // The public commit request now calls the internal version with context.
                self.apply_commit_internal(context);
                Ok(())
            }
            7 => { // set_buffer_transform (transform: int) - Since version 2
                if self.version < 2 { return Err(ProtocolError::InvalidVersion); }
                if *internal_state_guard == SurfaceInternalState::Destroyed { return Ok(()); }
                if args.is_empty() { return Err(ProtocolError::InvalidArguments); }
                let transform = match args[0] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                self.pending.lock().unwrap().buffer_transform = transform;
                Ok(())
            }
            8 => { // set_buffer_scale (scale: int) - Since version 3
                if self.version < 3 { return Err(ProtocolError::InvalidVersion); }
                if *internal_state_guard == SurfaceInternalState::Destroyed { return Ok(()); }
                if args.is_empty() { return Err(ProtocolError::InvalidArguments); }
                let scale = match args[0] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                self.pending.lock().unwrap().buffer_scale = scale;
                Ok(())
            }
            9 => { // damage_buffer (x: int, y: int, width: int, height: int) - Since version 4
                if self.version < 4 { return Err(ProtocolError::InvalidVersion); }
                if *internal_state_guard == SurfaceInternalState::Destroyed { return Ok(()); }
                if args.len() < 4 { return Err(ProtocolError::InvalidArguments); }
                let x = match args[0] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                let y = match args[1] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                let width = match args[2] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                let height = match args[3] { WlArgument::Int(val) => val, _ => return Err(ProtocolError::InvalidArguments) };
                if width < 0 || height < 0 { return Err(ProtocolError::InvalidArguments); }
                self.pending.lock().unwrap().damage_buffer.push(Rect { x, y, width, height });
                Ok(())
            }
            _ => Err(ProtocolError::InvalidOpcode(opcode)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::object::{ObjectManager, RequestContext}; // Changed
    use super::shm::{ShmFormat, ShmPoolSharedData}; // Changed
    use std::os::unix::io::RawFd;
    use memmap2::MmapMut;

    // Helper to create a mock WlBuffer for testing Surface attach
    fn create_mock_buffer(id: BufferId, version: u32, manager: &mut ObjectManager) -> Arc<WlBuffer> {
        // Create a dummy ShmPoolSharedData for the mock buffer
        // This requires a valid FD and size for mmap, which is heavy for a unit test.
        // Let's simplify: WlBuffer::new_shm needs Arc<ShmPoolSharedData>.
        // We can't easily mock MmapMut without an actual FD.
        // For testing attach, we primarily need an Arc<WlBuffer> that can be identified.
        //
        // Alternative: WlBuffer has a test-only constructor that doesn't need real SHM.
        // Or, we accept that these tests become integration tests if they touch FS for mmap.

        // For now, let's assume WlBuffer has a simplified test constructor or we mock it at a higher level.
        // Since WlBuffer is concrete, we can create it if its constructor is usable.
        // WlBuffer::new_shm is the current constructor.
        // This test will effectively become an integration test due to mmap.
        let dummy_fd: RawFd = unsafe { memfd_create::memfd_create("test_buffer_fd", memfd_create::MemFdCreateFlag::empty()).unwrap() };
        nix::unistd::ftruncate(dummy_fd, 4096).unwrap(); // Set size for mmap
        let mmap = unsafe { memmap2::MmapOptions::new().len(4096).map_mut(&unsafe{std::fs::File::from_raw_fd(dummy_fd)}).unwrap() };
        let pool_data = Arc::new(ShmPoolSharedData { mmap, fd: dummy_fd, size: 4096 });

        let buffer = WlBuffer::new_shm(id, version, 64, 64, 64*4, ShmFormat::Argb8888, pool_data, 0, 0);
        let arc_buffer = Arc::new(buffer);
        manager.register_new_object(id, arc_buffer.clone()).unwrap(); // Register it as if client created it
        arc_buffer
    }


    #[test]
    fn test_surface_new_internal_mut() {
        let surface = Surface::new(1, 1);
        assert_eq!(surface.id(), 1);
        assert_eq!(*surface.internal_state.lock().unwrap(), SurfaceInternalState::Alive);
        assert!(surface.current.lock().unwrap().buffer.is_none());
    }

    #[test]
    fn test_surface_attach_commit_release_buffer() {
        let mut manager = ObjectManager::new();
        let surface = Surface::new(1, 1); // Surface object itself

        let buffer1 = create_mock_buffer(101, 1, &mut manager);
        let buffer2 = create_mock_buffer(102, 1, &mut manager);

        let mut context = RequestContext { object_manager: &mut manager, client_id: 0 };

        // Attach buffer1
        surface.handle_request(1, vec![WlArgument::Object(buffer1.id())], &mut context).unwrap();
        assert_eq!(surface.pending.lock().unwrap().buffer.as_ref().map(|b| b.id()), Some(buffer1.id()));
        assert!(!buffer1.is_released(), "Buffer1 should be marked as used (not released) after attach");

        // Commit buffer1
        surface.handle_request(6, vec![], &mut context).unwrap(); // commit
        assert_eq!(surface.current.lock().unwrap().buffer.as_ref().map(|b| b.id()), Some(buffer1.id()));
        assert!(surface.pending.lock().unwrap().buffer.is_none());
        assert!(!buffer1.is_released(), "Buffer1 should still be used after commit");

        // Attach buffer2
        surface.handle_request(1, vec![WlArgument::Object(buffer2.id())], &mut context).unwrap();
        assert_eq!(surface.pending.lock().unwrap().buffer.as_ref().map(|b| b.id()), Some(buffer2.id()));
        assert!(!buffer2.is_released(), "Buffer2 should be marked as used after attach");
        assert!(!buffer1.is_released(), "Buffer1 should still be current, thus used");

        // Commit buffer2
        surface.handle_request(6, vec![], &mut context).unwrap(); // commit
        assert_eq!(surface.current.lock().unwrap().buffer.as_ref().map(|b| b.id()), Some(buffer2.id()));
        assert!(buffer1.is_released(), "Buffer1 should be released after buffer2 is committed");
        assert!(!buffer2.is_released(), "Buffer2 should be used after commit");

        // Detach buffer (attach null)
        surface.handle_request(1, vec![WlArgument::Object(0)], &mut context).unwrap(); // attach null
        assert!(surface.pending.lock().unwrap().buffer.is_none());

        // Commit detachment
        surface.handle_request(6, vec![], &mut context).unwrap(); // commit
        assert!(surface.current.lock().unwrap().buffer.is_none());
        assert!(buffer2.is_released(), "Buffer2 should be released after detach and commit");
    }

    // Test for downcasting in attach (conceptual, needs WaylandObject::as_any or similar)
    // This test is more about the mechanism than surface logic itself.
    // It's currently non-functional because downcast_arc is not a method of Arc<dyn WaylandObject>.
    // Add to WaylandObject: `fn as_any(&self) -> &dyn std::any::Any;`
    // And in impl: `fn as_any(&self) -> &dyn std::any::Any { self }`
    // Then use: `obj.as_any().downcast_ref::<SpecificType>()`
    // For Arc: `let arc_specific = arc_dyn_obj.clone().downcast_arc::<SpecificType>().unwrap();`
    // This requires `WaylandObject` to also be `std::any::Any`.

    #[test]
    fn test_surface_damage_accumulation_and_commit_clearing() {
        let mut manager = ObjectManager::new(); // Needed for RequestContext
        let surface_v4 = Surface::new(1, 4); // Use version 4 to allow damage_buffer
        let mut context = RequestContext { object_manager: &mut manager, client_id: 0 };

        let rect1 = Rect { x: 0, y: 0, width: 10, height: 10 };
        let rect2 = Rect { x: 20, y: 20, width: 5, height: 5 };
        let buf_rect1 = Rect { x: 1, y: 1, width: 8, height: 8 };
        let buf_rect2 = Rect { x: 15, y: 15, width: 3, height: 3 };

        // Damage surface
        surface_v4.handle_request(2, vec![WlArgument::Int(rect1.x), WlArgument::Int(rect1.y), WlArgument::Int(rect1.width), WlArgument::Int(rect1.height)], &mut context).unwrap();
        // Damage buffer
        surface_v4.handle_request(9, vec![WlArgument::Int(buf_rect1.x), WlArgument::Int(buf_rect1.y), WlArgument::Int(buf_rect1.width), WlArgument::Int(buf_rect1.height)], &mut context).unwrap();

        {
            let pending_state = surface_v4.pending.lock().unwrap();
            assert_eq!(pending_state.damage_surface.len(), 1);
            assert_eq!(pending_state.damage_surface[0], rect1);
            assert_eq!(pending_state.damage_buffer.len(), 1);
            assert_eq!(pending_state.damage_buffer[0], buf_rect1);
        }

        // Add more damage
        surface_v4.handle_request(2, vec![WlArgument::Int(rect2.x), WlArgument::Int(rect2.y), WlArgument::Int(rect2.width), WlArgument::Int(rect2.height)], &mut context).unwrap();
        surface_v4.handle_request(9, vec![WlArgument::Int(buf_rect2.x), WlArgument::Int(buf_rect2.y), WlArgument::Int(buf_rect2.width), WlArgument::Int(buf_rect2.height)], &mut context).unwrap();

        {
            let pending_state = surface_v4.pending.lock().unwrap();
            assert_eq!(pending_state.damage_surface.len(), 2);
            assert_eq!(pending_state.damage_surface[1], rect2);
            assert_eq!(pending_state.damage_buffer.len(), 2);
            assert_eq!(pending_state.damage_buffer[1], buf_rect2);
        }

        // Commit
        surface_v4.handle_request(6, vec![], &mut context).unwrap(); // commit

        {
            let current_state = surface_v4.current.lock().unwrap();
            assert_eq!(current_state.damage_surface.len(), 2);
            assert_eq!(current_state.damage_surface[0], rect1);
            assert_eq!(current_state.damage_surface[1], rect2);
            assert_eq!(current_state.damage_buffer.len(), 2);
            assert_eq!(current_state.damage_buffer[0], buf_rect1);
            assert_eq!(current_state.damage_buffer[1], buf_rect2);
        }

        {
            let pending_state_after_commit = surface_v4.pending.lock().unwrap();
            assert!(pending_state_after_commit.damage_surface.is_empty(), "Pending surface damage should be cleared after commit");
            assert!(pending_state_after_commit.damage_buffer.is_empty(), "Pending buffer damage should be cleared after commit");
        }
    }

    #[test]
    fn test_surface_damage_buffer_version_check() {
        let mut manager = ObjectManager::new();
        let surface_v3 = Surface::new(1, 3); // Version 3, does not support damage_buffer
        let mut context = RequestContext { object_manager: &mut manager, client_id: 0 };

        let buf_rect = Rect { x: 0, y: 0, width: 10, height: 10 };
        let args = vec![WlArgument::Int(buf_rect.x), WlArgument::Int(buf_rect.y), WlArgument::Int(buf_rect.width), WlArgument::Int(buf_rect.height)];

        match surface_v3.handle_request(9, args, &mut context) {
            Err(ProtocolError::InvalidVersion) => { /* Expected */ }
            _ => panic!("Expected InvalidVersion error for damage_buffer on surface v3"),
        }

        {
            let pending_state = surface_v3.pending.lock().unwrap();
            assert!(pending_state.damage_buffer.is_empty(), "Damage buffer should not be added for v3 surface");
        }
    }

    #[test]
    fn test_damage_with_negative_dimensions_is_error() {
        let mut manager = ObjectManager::new();
        let surface = Surface::new(1, 1);
        let mut context = RequestContext { object_manager: &mut manager, client_id: 0 };

        // Negative width for damage
        let args_neg_width = vec![WlArgument::Int(0), WlArgument::Int(0), WlArgument::Int(-10), WlArgument::Int(10)];
        assert_eq!(surface.handle_request(2, args_neg_width, &mut context), Err(ProtocolError::InvalidArguments));

        // Negative height for damage_buffer (v4 surface needed)
        let surface_v4 = Surface::new(2, 4);
        let args_neg_height_buf = vec![WlArgument::Int(0), WlArgument::Int(0), WlArgument::Int(10), WlArgument::Int(-10)];
        assert_eq!(surface_v4.handle_request(9, args_neg_height_buf, &mut context), Err(ProtocolError::InvalidArguments));
    }
}
