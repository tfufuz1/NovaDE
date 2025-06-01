use std::collections::HashMap;
use std::cmp::{max, min, Ordering}; // For intersection, union, and sorting

// --- Error Enum Definition ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceManagementError {
    SurfaceNotFound(SurfaceId),
    ParentNotFound(SurfaceId),
    SiblingNotFound(SurfaceId),
    InvalidBuffer(String),
    InvalidSurfaceState { current: SurfaceState, expected: Option<Vec<SurfaceState>>, operation: &'static str },
    HierarchyCycle(SurfaceId, SurfaceId), // child_id, attempted_parent_id
    SelfParentingAttempt(SurfaceId),
    NotSameParent { surface_id: SurfaceId, sibling_id: SurfaceId, surface_parent: Option<SurfaceId>, sibling_parent: Option<SurfaceId> },
    // BufferAlreadyAttached(SurfaceId), // Example for future use
    NoBufferAttached(SurfaceId), // When an operation requires a buffer that's missing
    AlreadyDestroyed(SurfaceId),
}

// Type alias for Result using the common error type
type Result<T, E = SurfaceManagementError> = std::result::Result<T, E>;


// Define a newtype for SurfaceId for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceId(pub u32);

// Define the possible states of a surface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceState {
    Created, PendingBuffer, Committed, Destroyed,
}

// Define a point in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point { pub x: i32, pub y: i32 }

// Define a size in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size { pub width: i32, pub height: i32 }

// Define a rectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle { pub x: i32, pub y: i32, pub width: i32, pub height: i32 }

impl Rectangle {
    pub fn area(&self) -> i32 { if self.width <= 0 || self.height <= 0 { 0 } else { self.width * self.height } }
    pub fn intersect(&self, other: &Rectangle) -> Option<Rectangle> {
        let x1 = max(self.x, other.x); let y1 = max(self.y, other.y);
        let x2 = min(self.x + self.width, other.x + other.width);
        let y2 = min(self.y + self.height, other.y + other.height);
        if x1 < x2 && y1 < y2 { Some(Rectangle { x: x1, y: y1, width: x2 - x1, height: y2 - y1 }) } else { None }
    }
    pub fn union(&self, other: &Rectangle) -> Rectangle {
        if self.area() == 0 { return *other; } if other.area() == 0 { return *self; }
        let x1 = min(self.x, other.x); let y1 = min(self.y, other.y);
        let x2 = max(self.x + self.width, other.x + other.width);
        let y2 = max(self.y + self.height, other.y + other.height);
        Rectangle { x: x1, y: y1, width: x2 - x1, height: y2 - y1 }
    }
    pub fn clip_to_bounds(&self, bounds_size: &Size) -> Option<Rectangle> {
        if bounds_size.width <= 0 || bounds_size.height <= 0 { return None; }
        let bounds_rect = Rectangle { x: 0, y: 0, width: bounds_size.width, height: bounds_size.height };
        self.intersect(&bounds_rect)
    }
}

// Define surface transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform { Normal, Rotated90, Rotated180, Rotated270, Flipped, FlippedRotated90, FlippedRotated180, FlippedRotated270 }

// Buffer Management Types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferType { Shm, Dma, Gpu }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferDetails { pub id: u64, pub buffer_type: BufferType, pub width: i32, pub height: i32, _marker: () }
pub type BufferRef = BufferDetails;
pub fn validate_buffer(buffer: &BufferRef) -> bool { buffer.width > 0 && buffer.height > 0 }

// Damage regions
#[derive(Debug, Clone, Default)]
pub struct Damage { pub rects: Vec<Rectangle> }
impl Damage {
    pub fn new() -> Self { Default::default() }
    pub fn clear(&mut self) { self.rects.clear(); }
    pub fn is_empty(&self) -> bool { self.rects.is_empty() }
}

// Main Surface struct
#[derive(Debug, Clone)]
pub struct Surface {
    pub id: SurfaceId,
    pub state: SurfaceState,
    pub position: Point, pub size: Size, pub transform: Transform, pub alpha: f32,
    pub buffer: Option<BufferRef>, pub pending_buffer: Option<BufferRef>,
    pub committed_buffer_offset: Option<Point>, pub pending_buffer_offset: Option<Point>,
    pub pending_damage: Damage, pub committed_damage: Damage,
    pub input_region: Option<Vec<Rectangle>>, pub opaque_region: Option<Vec<Rectangle>>,
    pub subsurfaces: Vec<SurfaceId>, pub parent: Option<SurfaceId>, pub z_order: i32,
    pub pending_parent: Option<Option<SurfaceId>>, pub pending_z_order: Option<i32>,
    pub is_subsurface: bool,
}

impl Surface {
    fn new(id: SurfaceId) -> Self {
        Surface {
            id, state: SurfaceState::Created,
            position: Point { x: 0, y: 0 }, size: Size { width: 0, height: 0 },
            transform: Transform::Normal, alpha: 1.0,
            buffer: None, pending_buffer: None,
            committed_buffer_offset: None, pending_buffer_offset: None,
            pending_damage: Damage::default(), committed_damage: Damage::default(),
            input_region: None, opaque_region: None,
            subsurfaces: Vec::new(), parent: None, z_order: 0,
            pending_parent: None, pending_z_order: None, is_subsurface: false,
        }
    }

    pub fn attach_buffer(&mut self, buffer: Option<BufferRef>, x: i32, y: i32) -> Result<()> {
        if self.state == SurfaceState::Destroyed {
            return Err(SurfaceManagementError::AlreadyDestroyed(self.id));
        }
        if let Some(ref b) = buffer {
            if !validate_buffer(b) { return Err(SurfaceManagementError::InvalidBuffer("Dimensions must be positive.".to_string())); }
            self.pending_buffer = Some(b.clone());
            self.pending_buffer_offset = Some(Point { x, y });
            if self.state == SurfaceState::Created || self.state == SurfaceState::Committed {
                self.state = SurfaceState::PendingBuffer;
            }
        } else {
            self.pending_buffer = None;
            self.pending_buffer_offset = None;
        }
        Ok(())
    }

    pub fn add_damage(&mut self, rect: Rectangle) -> Result<()> {
         if self.state == SurfaceState::Destroyed {
            return Err(SurfaceManagementError::AlreadyDestroyed(self.id));
        }
        if self.size.width <= 0 || self.size.height <= 0 || rect.area() <= 0 { return Ok(()); } // No error, just no-op
        if let Some(clipped_rect) = rect.clip_to_bounds(&self.size) {
            if clipped_rect.area() > 0 { self.pending_damage.rects.push(clipped_rect); }
        }
        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        if self.state == SurfaceState::Destroyed { return Err(SurfaceManagementError::AlreadyDestroyed(self.id)); }

        match self.state {
            SurfaceState::Created => {
                if self.pending_buffer.is_none() && self.pending_parent.is_none() && self.pending_damage.is_empty() && self.pending_z_order.is_none() {
                    // Committing from Created with nothing pending at all.
                    // This could be a no-op Ok(()), or an error if a buffer is strictly expected to move out of Created.
                    // Let's treat it as needing a buffer if no other hierarchy ops are pending.
                    return Err(SurfaceManagementError::NoBufferAttached(self.id));
                } else if self.pending_buffer.is_none() && self.pending_parent.is_none() {
                     // Has pending damage/z-order but no buffer and not becoming a subsurface
                    return Err(SurfaceManagementError::NoBufferAttached(self.id));
                }
            }
            SurfaceState::PendingBuffer | SurfaceState::Committed => {} // Valid states
            SurfaceState::Destroyed => unreachable!(), // Handled at the start
        }

        if let Some(pending_buffer) = self.pending_buffer.take() {
            self.buffer = Some(pending_buffer.clone());
            self.size = Size { width: pending_buffer.width, height: pending_buffer.height };
            self.committed_buffer_offset = self.pending_buffer_offset.take();
        } else if self.state == SurfaceState::PendingBuffer && self.buffer.is_none() && self.pending_parent.is_none() {
            return Err(SurfaceManagementError::NoBufferAttached(self.id));
        }

        if self.buffer.is_none() && self.pending_parent.is_none() &&
           (self.state == SurfaceState::Created || self.state == SurfaceState::PendingBuffer) {
             return Err(SurfaceManagementError::NoBufferAttached(self.id));
        }

        if !self.pending_damage.is_empty() {
            self.committed_damage = std::mem::take(&mut self.pending_damage);
        } else { self.committed_damage.clear(); }

        if let Some(pending_parent_action) = self.pending_parent.take() {
            // This updates the surface's own view of its parent.
            // The SurfaceRegistry::commit_surface_hierarchy() call will later update
            // the parent's subsurfaces list based on this committed state.
            self.parent = pending_parent_action;
        }
        if let Some(pending_z) = self.pending_z_order.take() { self.z_order = pending_z; }
        self.is_subsurface = self.parent.is_some();

        self.state = SurfaceState::Committed;
        Ok(())
    }

    pub fn destroy(&mut self) -> Result<()> {
        if self.state == SurfaceState::Destroyed {
            return Err(SurfaceManagementError::AlreadyDestroyed(self.id));
        }
        self.state = SurfaceState::Destroyed;
        self.pending_buffer = None;
        self.pending_buffer_offset = None;
        self.pending_damage.clear();
        self.pending_parent = None;
        self.pending_z_order = None;
        // Current buffer, committed damage etc are kept as they reflect the last visible state.
        // Subsurface/parent links are cleared by registry.
        Ok(())
    }
}

// SurfaceRegistry
#[derive(Debug, Default)]
pub struct SurfaceRegistry {
    surfaces: HashMap<SurfaceId, Surface>,
    next_surface_id: u32,
}

impl SurfaceRegistry {
    pub fn new() -> Self { Default::default() }
    pub fn create_surface(&mut self) -> SurfaceId {
        let id = SurfaceId(self.next_surface_id); self.next_surface_id += 1;
        self.surfaces.insert(id, Surface::new(id)); id
    }
    pub fn get_surface(&self, id: SurfaceId) -> Option<&Surface> { self.surfaces.get(&id) }
    pub fn get_surface_mut(&mut self, id: SurfaceId) -> Option<&mut Surface> { self.surfaces.get_mut(&id) }

    pub fn set_parent(&mut self, surface_id: SurfaceId, new_parent_id_opt: Option<SurfaceId>) -> Result<()> {
        let surface_is_destroyed = self.surfaces.get(&surface_id).map_or(true, |s| s.state == SurfaceState::Destroyed);
        if surface_is_destroyed { return Err(SurfaceManagementError::AlreadyDestroyed(surface_id));}


        if surface_id == new_parent_id_opt.unwrap_or(SurfaceId(u32::MAX)) {
            return Err(SurfaceManagementError::SelfParentingAttempt(surface_id));
        }

        if let Some(p_id) = new_parent_id_opt {
            let parent_surface = self.surfaces.get(&p_id).ok_or(SurfaceManagementError::ParentNotFound(p_id))?;
            if parent_surface.state == SurfaceState::Destroyed {
                 return Err(SurfaceManagementError::AlreadyDestroyed(p_id));
            }

            let mut current_check_id = p_id;
            loop {
                let S = self.surfaces.get(&current_check_id).ok_or(SurfaceManagementError::ParentNotFound(current_check_id))?; // Should exist
                if S.parent == Some(surface_id) { return Err(SurfaceManagementError::HierarchyCycle(surface_id, p_id)); }
                if S.parent.is_none() { break; }
                current_check_id = S.parent.unwrap();
                if current_check_id == p_id { break; } // Cycle in parent chain itself, or reached root
            }
        }

        let surface = self.surfaces.get_mut(&surface_id).ok_or(SurfaceManagementError::SurfaceNotFound(surface_id))?;
        surface.pending_parent = Some(new_parent_id_opt);
        Ok(())
    }

    fn get_z_order_info(&self, s_id: SurfaceId, target_s_id: SurfaceId)
        -> Result<(Option<SurfaceId>, Option<SurfaceId>, i32)> {
        let s = self.surfaces.get(&s_id).ok_or(SurfaceManagementError::SurfaceNotFound(s_id))?;
        let target_s = self.surfaces.get(&target_s_id).ok_or(SurfaceManagementError::SiblingNotFound(target_s_id))?;

        if s.state == SurfaceState::Destroyed { return Err(SurfaceManagementError::AlreadyDestroyed(s_id)); }
        if target_s.state == SurfaceState::Destroyed { return Err(SurfaceManagementError::AlreadyDestroyed(target_s_id)); }

        let s_parent = s.pending_parent.as_ref().map(|&opt_p| opt_p).unwrap_or(s.parent);
        let target_s_parent = target_s.pending_parent.as_ref().map(|&opt_p| opt_p).unwrap_or(target_s.parent);

        if s_parent != target_s_parent || s_parent.is_none() {
            return Err(SurfaceManagementError::NotSameParent { surface_id: s_id, sibling_id: target_s_id, surface_parent: s_parent, sibling_parent: target_s_parent });
        }
        Ok((s_parent, target_s_parent, target_s.pending_z_order.unwrap_or(target_s.z_order)))
    }

    pub fn place_above(&mut self, surface_id: SurfaceId, sibling_id: SurfaceId) -> Result<()> {
        let (_, _, sibling_z) = self.get_z_order_info(surface_id, sibling_id)?;
        let surface = self.surfaces.get_mut(&surface_id).unwrap(); // Existence checked by get_z_order_info
        surface.pending_z_order = Some(sibling_z + 1);
        Ok(())
    }

    pub fn place_below(&mut self, surface_id: SurfaceId, sibling_id: SurfaceId) -> Result<()> {
        let (_, _, sibling_z) = self.get_z_order_info(surface_id, sibling_id)?;
        let surface = self.surfaces.get_mut(&surface_id).unwrap();
        surface.pending_z_order = Some(sibling_z - 1);
        Ok(())
    }
    
    pub fn destroy_surface(&mut self, surface_id: SurfaceId) -> Result<()> {
        // Collect children IDs first to avoid borrowing issues with recursive calls
        let children_ids: Vec<SurfaceId> = {
            let surface = self.surfaces.get_mut(&surface_id)
                .ok_or(SurfaceManagementError::SurfaceNotFound(surface_id))?;

            surface.destroy()?; // Mark as destroyed and clear pending state
            surface.subsurfaces.clone() // Clone the list of children
        };

        // Recursively destroy children
        for child_id in children_ids {
            self.destroy_surface(child_id)?; // Errors during child destruction will propagate
        }

        // Remove the surface itself from the registry
        self.surfaces.remove(&surface_id);
        
        // Parent's subsurfaces list will be updated by the next commit_surface_hierarchy call
        // as it won't include destroyed surfaces.
        Ok(())
    }


    pub fn commit_surface_hierarchy(&mut self) {
        let mut parent_to_children_map: HashMap<SurfaceId, Vec<(SurfaceId, i32)>> = HashMap::new();

        for surface in self.surfaces.values() {
            // Only consider committed, non-destroyed surfaces for hierarchy
            if surface.state == SurfaceState::Destroyed { continue; }

            if let Some(parent_id) = surface.parent {
                 // Ensure parent itself is not destroyed
                if self.surfaces.get(&parent_id).map_or(true, |p| p.state == SurfaceState::Destroyed) {
                    continue;
                }
                parent_to_children_map
                    .entry(parent_id)
                    .or_default()
                    .push((surface.id, surface.z_order));
            }
        }

        for surface in self.surfaces.values_mut() {
            if surface.state == SurfaceState::Destroyed {
                surface.subsurfaces.clear(); // Destroyed surfaces have no children in active hierarchy
                continue;
            }

            surface.subsurfaces.clear();
            if let Some(children_with_z) = parent_to_children_map.get(&surface.id) {
                let mut sorted_children = children_with_z.clone();
                sorted_children.sort_by_key(|&(_, z)| z);
                surface.subsurfaces = sorted_children.iter().map(|&(id, _)| id).collect();
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn sample_buffer(id: u64, w: i32, h: i32) -> BufferRef {
        BufferDetails { id, buffer_type: BufferType::Shm, width: w, height: h, _marker: () }
    }
    fn committed_surface(registry: &mut SurfaceRegistry, id_val: u32) -> SurfaceId {
        let id = SurfaceId(id_val);
        registry.surfaces.insert(id, Surface::new(id));
        let surface = registry.get_surface_mut(id).unwrap();
        surface.attach_buffer(Some(sample_buffer(id_val as u64,10,10)),0,0).unwrap();
        surface.commit().unwrap();
        id
    }

    #[test]
    fn test_surface_destroy() {
        let mut surface = Surface::new(SurfaceId(0));
        surface.pending_buffer = Some(sample_buffer(1,10,10));
        surface.pending_damage.rects.push(Rectangle{x:0,y:0,width:1,height:1});

        assert!(surface.destroy().is_ok());
        assert_eq!(surface.state, SurfaceState::Destroyed);
        assert!(surface.pending_buffer.is_none());
        assert!(surface.pending_damage.is_empty());

        // Try destroying again
        assert_eq!(surface.destroy(), Err(SurfaceManagementError::AlreadyDestroyed(surface.id)));
    }

    #[test]
    fn test_attach_buffer_on_destroyed_surface() {
        let mut surface = Surface::new(SurfaceId(0));
        surface.destroy().unwrap();
        assert_eq!(
            surface.attach_buffer(Some(sample_buffer(1,10,10)),0,0),
            Err(SurfaceManagementError::AlreadyDestroyed(surface.id))
        );
    }


    #[test]
    fn test_registry_destroy_surface_simple() {
        let mut registry = SurfaceRegistry::new();
        let sid = committed_surface(&mut registry, 0);
        
        assert!(registry.destroy_surface(sid).is_ok());
        assert!(registry.get_surface(sid).is_none());
    }

    #[test]
    fn test_registry_destroy_surface_not_found() {
        let mut registry = SurfaceRegistry::new();
        assert_eq!(registry.destroy_surface(SurfaceId(0)), Err(SurfaceManagementError::SurfaceNotFound(SurfaceId(0))));
    }

    #[test]
    fn test_registry_destroy_surface_recursive() {
        let mut registry = SurfaceRegistry::new();
        let p_id = committed_surface(&mut registry, 0);
        let c1_id = committed_surface(&mut registry, 1);
        let c2_id = committed_surface(&mut registry, 2);
        let gc_id = committed_surface(&mut registry, 3); // Grandchild

        registry.set_parent(c1_id, Some(p_id)).unwrap();
        registry.get_surface_mut(c1_id).unwrap().commit().unwrap();
        registry.set_parent(c2_id, Some(p_id)).unwrap();
        registry.get_surface_mut(c2_id).unwrap().commit().unwrap();
        registry.set_parent(gc_id, Some(c1_id)).unwrap();
        registry.get_surface_mut(gc_id).unwrap().commit().unwrap();
        
        registry.commit_surface_hierarchy(); // Build initial hierarchy

        let parent = registry.get_surface(p_id).unwrap();
        assert_eq!(parent.subsurfaces.len(), 2);

        // Destroy parent
        assert!(registry.destroy_surface(p_id).is_ok());

        assert!(registry.get_surface(p_id).is_none());
        assert!(registry.get_surface(c1_id).is_none(), "Child 1 not destroyed");
        assert!(registry.get_surface(c2_id).is_none(), "Child 2 not destroyed");
        assert!(registry.get_surface(gc_id).is_none(), "Grandchild not destroyed");
    }

    #[test]
    fn test_commit_surface_hierarchy_skips_destroyed() {
        let mut registry = SurfaceRegistry::new();
        let p_id = committed_surface(&mut registry, 0);
        let c1_id = committed_surface(&mut registry, 1);
        let c2_id = committed_surface(&mut registry, 2);

        registry.set_parent(c1_id, Some(p_id)).unwrap();
        registry.get_surface_mut(c1_id).unwrap().commit().unwrap();
        registry.set_parent(c2_id, Some(p_id)).unwrap();
        registry.get_surface_mut(c2_id).unwrap().commit().unwrap();
        registry.commit_surface_hierarchy();

        assert_eq!(registry.get_surface(p_id).unwrap().subsurfaces.len(), 2);

        // Destroy c1
        registry.destroy_surface(c1_id).unwrap();
        assert!(registry.get_surface(c1_id).is_none());
        // c1 is marked Destroyed in its own struct if we didn't remove immediately
        // but destroy_surface removes it from the map.

        registry.commit_surface_hierarchy(); // Rebuild hierarchy

        let parent_after_destroy = registry.get_surface(p_id).unwrap();
        assert_eq!(parent_after_destroy.subsurfaces.len(), 1);
        assert_eq!(parent_after_destroy.subsurfaces[0], c2_id);
    }

    #[test]
    fn test_set_parent_on_destroyed_surface_errors() {
        let mut registry = SurfaceRegistry::new();
        let sid = committed_surface(&mut registry, 0);
        let pid = committed_surface(&mut registry, 1);
        registry.destroy_surface(sid).unwrap();
        assert_eq!(registry.set_parent(sid, Some(pid)), Err(SurfaceManagementError::AlreadyDestroyed(sid)));
    }

    #[test]
    fn test_set_parent_to_a_destroyed_parent_errors() {
        let mut registry = SurfaceRegistry::new();
        let sid = committed_surface(&mut registry, 0);
        let pid = committed_surface(&mut registry, 1);
        registry.destroy_surface(pid).unwrap();
        assert_eq!(registry.set_parent(sid, Some(pid)), Err(SurfaceManagementError::AlreadyDestroyed(pid)));
    }

    // Test specific error types
    #[test]
    fn test_error_commit_no_buffer_attached() {
        let mut surface = Surface::new(SurfaceId(0));
        // No pending buffer, no pending parent, from Created state
        assert_eq!(surface.commit(), Err(SurfaceManagementError::NoBufferAttached(surface.id)));
    }

    #[test]
    fn test_error_hierarchy_self_parenting() {
        let mut registry = SurfaceRegistry::new();
        let sid = registry.create_surface();
        assert_eq!(registry.set_parent(sid, Some(sid)), Err(SurfaceManagementError::SelfParentingAttempt(sid)));
    }

    #[test]
    fn test_error_hierarchy_cycle() {
        let mut registry = SurfaceRegistry::new();
        let s1 = committed_surface(&mut registry, 1);
        let s2 = committed_surface(&mut registry, 2);
        registry.set_parent(s2, Some(s1)).unwrap(); // s1 -> s2
        registry.get_surface_mut(s2).unwrap().commit().unwrap();
        registry.commit_surface_hierarchy();

        assert_eq!(registry.set_parent(s1, Some(s2)), Err(SurfaceManagementError::HierarchyCycle(s1,s2)));
    }
}
