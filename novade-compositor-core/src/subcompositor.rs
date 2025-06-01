//! # Subcompositor Logic
//!
//! This module implements the core logic for `wl_subcompositor` and `wl_subsurface`
//! Wayland interfaces. It handles the creation, destruction, and manipulation of
//! subsurfaces, including their relationship to parent surfaces, stacking order (Z-order),
//! and synchronization modes.
//!
//! The actual Wayland protocol binding would call into these functions.

use crate::surface::{SurfaceId, Surface, SurfaceRole};
use crate::surface::surface_registry::SurfaceRegistry;
use std::sync::{Arc, Mutex};

/// Defines the synchronization behavior of a subsurface relative to its parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubsurfaceSyncMode {
    /// The subsurface's state is cached upon its own commit and applied only when
    /// its parent surface's state is applied (committed). This is the default mode.
    Synchronized,
    /// The subsurface's state is applied independently of its parent's commit cycle.
    /// Changes to the subsurface become visible once its own `wl_surface.commit` is processed.
    Desynchronized,
}

impl Default for SubsurfaceSyncMode {
    fn default() -> Self {
        SubsurfaceSyncMode::Synchronized
    }
}

/// Holds the state specific to a surface when it acts as a subsurface.
#[derive(Debug, Clone)]
pub struct SubsurfaceState {
    /// The `SurfaceId` of the parent surface.
    pub parent_id: SurfaceId,
    /// The desired position of the subsurface relative to its parent's origin.
    /// This is set by `wl_subsurface.set_position` and applied on parent commit.
    pub desired_position: (i32, i32),
    /// The current applied position of the subsurface relative to its parent.
    pub current_position: (i32, i32),
    /// The synchronization mode of the subsurface.
    pub sync_mode: SubsurfaceSyncMode,
    /// Flag used in synchronized mode. If true, the subsurface has pending cached state
    /// that needs to be applied when its parent commits.
    pub needs_apply_on_parent_commit: bool,
}

impl SubsurfaceState {
    /// Creates a new `SubsurfaceState` associated with a parent surface.
    ///
    /// # Arguments
    /// * `parent_id`: The `SurfaceId` of the surface that will be the parent.
    pub fn new(parent_id: SurfaceId) -> Self {
        Self {
            parent_id,
            desired_position: (0, 0),
            current_position: (0, 0),
            sync_mode: SubsurfaceSyncMode::default(),
            needs_apply_on_parent_commit: false,
        }
    }
}

/// Errors that can occur during subsurface operations.
#[derive(Debug)]
pub enum SubsurfaceError {
    /// The provided surface ID is invalid, not found, or cannot be used in this context.
    BadSurface,
    /// The provided parent surface ID is invalid, not found, or cannot be used as a parent.
    BadParent,
    /// The surface already has an assigned role (e.g., toplevel, another subsurface object).
    SurfaceHasRole,
    /// The operation is intended for a surface with a subsurface role, but it doesn't have one.
    NotASubsurface,
    /// (Currently unused by functions returning it, but defined for completeness)
    /// Sibling operations require the target to be a child of the same parent,
    /// or other parent-related validation failed.
    NotAParent,
    /// An attempt was made to create a parent-child relationship that would form a cycle.
    CycleDetected,
    /// The specified sibling surface for a stacking operation was not found as a child of the same parent.
    SiblingNotFound,
}


/// Handles the `wl_subcompositor.get_subsurface` request logic.
///
/// This function establishes a parent-child relationship between two surfaces,
/// turning the child `surface_id` into a subsurface of `parent_id`.
///
/// # Arguments
/// * `_subcompositor_resource_id`: The Wayland resource ID of the `wl_subcompositor` global.
///   (Typically unused in the core logic, but present for API completeness).
/// * `_new_subsurface_resource_id`: The Wayland resource ID for the new `wl_subsurface` object.
///   (Typically unused in the core logic, as the association is `wl_subsurface` -> `child_surface_id`).
/// * `surface_id`: The `SurfaceId` of the `wl_surface` to become a subsurface.
/// * `parent_id`: The `SurfaceId` of the `wl_surface` to become the parent.
/// * `registry`: A reference to the `SurfaceRegistry` to access and modify surface states.
///
/// # Returns
/// `Ok(())` if the subsurface is successfully created and associated.
/// `Err(SubsurfaceError)` if any validation fails (e.g., cycle detected, surface already has a role).
pub fn get_subsurface(
    _subcompositor_resource_id: u32,
    _new_subsurface_resource_id: u32,
    surface_id: SurfaceId,
    parent_id: SurfaceId,
    registry: &SurfaceRegistry,
) -> Result<(), SubsurfaceError> {

    if surface_id == parent_id {
        return Err(SubsurfaceError::CycleDetected);
    }

    // Cycle detection: Check if surface_id is an ancestor of parent_id.
    let mut current_ancestor_id_opt = Some(parent_id);
    while let Some(current_ancestor_id) = current_ancestor_id_opt {
        if current_ancestor_id == surface_id {
            return Err(SubsurfaceError::CycleDetected);
        }
        let ancestor_surface_arc = registry.get_surface(current_ancestor_id);
        if let Some(arc) = ancestor_surface_arc {
            // It's important to handle potential panics if a lock is poisoned.
            // For simplicity in this example, using unwrap().
            let ancestor_surface = arc.lock().unwrap();
            current_ancestor_id_opt = ancestor_surface.parent;
        } else {
            // This implies parent_id (or an ancestor) does not exist in the registry,
            // which is a BadParent error condition.
            return Err(SubsurfaceError::BadParent);
        }
    }

    let surface_arc = registry.get_surface(surface_id).ok_or(SubsurfaceError::BadSurface)?;
    let parent_arc = registry.get_surface(parent_id).ok_or(SubsurfaceError::BadParent)?; // Already checked if parent_id itself is valid

    // Lock order: parent then child to avoid potential AB-BA deadlocks if simultaneous
    // operations occur on the same pair.
    let mut parent_surface = parent_arc.lock().unwrap();
    let mut surface = surface_arc.lock().unwrap();

    if surface.role.is_some() {
        return Err(SubsurfaceError::SurfaceHasRole);
    }

    surface.role = Some(SurfaceRole::Subsurface(SubsurfaceState::new(parent_id)));
    surface.parent = Some(parent_id);
    parent_surface.children.push(surface_id); // New subsurfaces are initially top-most among siblings.
    Ok(())
}

/// Handles the `wl_subsurface.destroy` request logic.
///
/// This function removes the subsurface role from the given surface. The `wl_surface`
/// itself is not destroyed by this call but is disassociated from its parent and
/// effectively unmapped. The client is responsible for destroying the `wl_surface` resource separately.
///
/// # Arguments
/// * `subsurface_surface_id`: The `SurfaceId` of the `wl_surface` whose subsurface role is to be destroyed.
/// * `registry`: A mutable reference to the `SurfaceRegistry` to modify surface states.
///   (Mutable because it might modify the parent's children list).
///
/// # Returns
/// `Ok(())` if the role was successfully removed or if the surface was not a subsurface (no-op).
/// `Err(SubsurfaceError::BadSurface)` if the `subsurface_surface_id` is not found in the registry.
pub fn destroy_subsurface_role(
    subsurface_surface_id: SurfaceId,
    registry: &mut SurfaceRegistry,
) -> Result<(), SubsurfaceError> {
    let surface_arc = registry.get_surface(subsurface_surface_id).ok_or(SubsurfaceError::BadSurface)?;

    let old_parent_id = { // Scope for surface lock
        let mut surface = surface_arc.lock().unwrap();
        if let Some(SurfaceRole::Subsurface(sub_state)) = &surface.role {
            let parent_id = sub_state.parent_id;
            surface.role = None;
            surface.parent = None;
            Some(parent_id)
        } else {
            // Not a subsurface, or role already cleared. This is a no-op as per Wayland spec.
            return Ok(());
        }
    };

    if let Some(parent_id) = old_parent_id {
        if let Some(parent_arc) = registry.get_surface(parent_id) {
            let mut parent_surface = parent_arc.lock().unwrap();
            parent_surface.children.retain(|&child_id| child_id != subsurface_surface_id);
        }
        // If parent_surface is not found, it might have been destroyed. The child is already unlinked.
    }
    Ok(())
}

/// Handles the `wl_subsurface.set_position` request logic.
///
/// Sets the desired position of the subsurface relative to its parent's origin.
/// This position is cached in `SubsurfaceState::desired_position` and is typically
/// applied to `SubsurfaceState::current_position` when the parent surface's state is committed.
///
/// # Arguments
/// * `subsurface_surface_id`: The `SurfaceId` of the subsurface.
/// * `x`: The x-coordinate of the desired position.
/// * `y`: The y-coordinate of the desired position.
/// * `registry`: A reference to the `SurfaceRegistry` to access the surface.
///
/// # Returns
/// `Ok(())` on success.
/// `Err(SubsurfaceError::BadSurface)` if the surface is not found.
/// `Err(SubsurfaceError::NotASubsurface)` if the surface does not have a subsurface role.
pub fn set_position(
    subsurface_surface_id: SurfaceId,
    x: i32,
    y: i32,
    registry: &SurfaceRegistry,
) -> Result<(), SubsurfaceError> {
    let surface_arc = registry.get_surface(subsurface_surface_id).ok_or(SubsurfaceError::BadSurface)?;
    let mut surface = surface_arc.lock().unwrap();

    if let Some(SurfaceRole::Subsurface(ref mut state)) = surface.role {
        state.desired_position = (x,y);
        Ok(())
    } else {
        Err(SubsurfaceError::NotASubsurface)
    }
}

/// Helper function to reorder a surface within its parent's children list.
/// This function assumes `surface_to_move_id` is already a child of `parent_surface`
/// and `reference_sibling_id` is also a child (unless it's a special case handled by caller).
fn reorder_children(
    parent_surface: &mut Surface,
    surface_to_move_id: SurfaceId,
    reference_sibling_id: SurfaceId,
    place_above: bool,
) -> Result<(), SubsurfaceError> {
    if !parent_surface.children.contains(&surface_to_move_id) {
        return Err(SubsurfaceError::BadSurface);
    }
    if surface_to_move_id == reference_sibling_id {
        return Ok(());
    }

    // Remove the surface to move from its current position.
    parent_surface.children.retain(|&id| id != surface_to_move_id);

    // Find the position of the reference sibling.
    if let Some(pos) = parent_surface.children.iter().position(|&id| id == reference_sibling_id) {
        if place_above { // Place surface_to_move_id immediately after reference_sibling_id
            parent_surface.children.insert(pos + 1, surface_to_move_id);
        } else { // Place surface_to_move_id immediately before reference_sibling_id
            parent_surface.children.insert(pos, surface_to_move_id);
        }
    } else {
        // This case should ideally be handled by the calling functions (place_above/place_below)
        // if the sibling is not found, as they implement the spec's fallback (top/bottom of stack).
        // If this helper is called with an invalid sibling_id that wasn't pre-validated, it's an error.
        return Err(SubsurfaceError::SiblingNotFound);
    }
    Ok(())
}


/// Handles the `wl_subsurface.place_above` request logic.
///
/// Places the subsurface `subsurface_surface_id` immediately above `sibling_surface_id`
/// in the stacking order of the parent. If `sibling_surface_id` is not a valid sibling
/// (not a child of the same parent, or is the subsurface itself), the subsurface is placed
/// at the top of the stack among its siblings.
///
/// # Arguments
/// * `subsurface_surface_id`: The subsurface to reorder.
/// * `sibling_surface_id`: The reference sibling surface.
/// * `registry`: Access to the surface registry.
///
/// # Returns
/// `Ok(())` on success, or an error if the surfaces are invalid or not related correctly.
pub fn place_above(
    subsurface_surface_id: SurfaceId,
    sibling_surface_id: SurfaceId,
    registry: &SurfaceRegistry,
) -> Result<(), SubsurfaceError> {
    let surface_arc = registry.get_surface(subsurface_surface_id).ok_or(SubsurfaceError::BadSurface)?;
    let parent_id = {
        let surface_guard = surface_arc.lock().unwrap();
        match &surface_guard.role {
            Some(SurfaceRole::Subsurface(state)) => state.parent_id,
            _ => return Err(SubsurfaceError::NotASubsurface),
        }
    };

    let parent_arc = registry.get_surface(parent_id).ok_or(SubsurfaceError::BadParent)?;
    let mut parent_surface = parent_arc.lock().unwrap();

    if subsurface_surface_id == sibling_surface_id { return Ok(()); } // No change relative to self

    // If sibling is not a child of the same parent, place at the top.
    if !parent_surface.children.contains(&sibling_surface_id) {
        parent_surface.children.retain(|&id| id != subsurface_surface_id);
        parent_surface.children.push(subsurface_surface_id);
        return Ok(());
    }

    reorder_children(&mut parent_surface, subsurface_surface_id, sibling_surface_id, true)
}

/// Handles the `wl_subsurface.place_below` request logic.
///
/// Places the subsurface `subsurface_surface_id` immediately below `sibling_surface_id`
/// in the stacking order of the parent. If `sibling_surface_id` is not a valid sibling
/// (not a child of the same parent, or is the subsurface itself), the subsurface is placed
/// at the bottom of the stack among its siblings.
///
/// # Arguments
/// * `subsurface_surface_id`: The subsurface to reorder.
/// * `sibling_surface_id`: The reference sibling surface.
/// * `registry`: Access to the surface registry.
///
/// # Returns
/// `Ok(())` on success, or an error if the surfaces are invalid or not related correctly.
pub fn place_below(
    subsurface_surface_id: SurfaceId,
    sibling_surface_id: SurfaceId,
    registry: &SurfaceRegistry,
) -> Result<(), SubsurfaceError> {
    let surface_arc = registry.get_surface(subsurface_surface_id).ok_or(SubsurfaceError::BadSurface)?;
    let parent_id = {
        let surface_guard = surface_arc.lock().unwrap();
        match &surface_guard.role {
            Some(SurfaceRole::Subsurface(state)) => state.parent_id,
            _ => return Err(SubsurfaceError::NotASubsurface),
        }
    };

    let parent_arc = registry.get_surface(parent_id).ok_or(SubsurfaceError::BadParent)?;
    let mut parent_surface = parent_arc.lock().unwrap();

    if subsurface_surface_id == sibling_surface_id { return Ok(()); } // No change relative to self

    // If sibling is not a child of the same parent, place at the bottom.
    if !parent_surface.children.contains(&sibling_surface_id) {
        parent_surface.children.retain(|&id| id != subsurface_surface_id);
        parent_surface.children.insert(0, subsurface_surface_id);
        return Ok(());
    }

    reorder_children(&mut parent_surface, subsurface_surface_id, sibling_surface_id, false)
}

/// Handles `wl_subsurface.set_sync` and `wl_subsurface.set_desync` requests logic.
///
/// Updates the synchronization mode of the subsurface.
///
/// # Arguments
/// * `subsurface_surface_id`: The `SurfaceId` of the subsurface.
/// * `sync_mode`: The new `SubsurfaceSyncMode` to apply.
/// * `registry`: A reference to the `SurfaceRegistry`.
///
/// # Returns
/// `Ok(())` on success.
/// `Err(SubsurfaceError::BadSurface)` if the surface is not found.
/// `Err(SubsurfaceError::NotASubsurface)` if the surface does not have a subsurface role.
pub fn set_sync_mode(
    subsurface_surface_id: SurfaceId,
    sync_mode: SubsurfaceSyncMode,
    registry: &SurfaceRegistry,
) -> Result<(), SubsurfaceError> {
    let surface_arc = registry.get_surface(subsurface_surface_id).ok_or(SubsurfaceError::BadSurface)?;
    let mut surface = surface_arc.lock().unwrap();

    if let Some(SurfaceRole::Subsurface(ref mut state)) = surface.role {
        state.sync_mode = sync_mode;
        // Note: If transitioning from Synchronized to Desynchronized, the Wayland spec implies
        // that any cached state should be applied immediately. This logic is currently handled
        // within `Surface::commit()` when a desynchronized subsurface (that was previously sync) commits.
        Ok(())
    } else {
        Err(SubsurfaceError::NotASubsurface)
    }
}

// Note: wl_subcompositor.destroy is typically handled by the client destroying the global object.
// No specific function is usually needed in the core logic for the subcompositor itself,
// unless the compositor needs to track multiple subcompositor instances or perform specific
// cleanup when a subcompositor global is unbound.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{Surface, SurfaceId, SurfaceState};
    use crate::surface::surface_registry::SurfaceRegistry;
    use crate::buffer_manager::BufferManager; // For unregister/destroy tests

    // Helper to create a registry and add a surface, returning its ID and Arc.
    fn create_surface_in_registry(registry: &mut SurfaceRegistry) -> (SurfaceId, Arc<Mutex<Surface>>) {
        registry.register_new_surface()
    }

    // Helper to get role for assertion convenience
    fn get_surface_role(surface_arc: &Arc<Mutex<Surface>>) -> Option<SurfaceRole> {
        surface_arc.lock().unwrap().role.clone()
    }

    #[test]
    fn test_get_subsurface_success() {
        let mut registry = SurfaceRegistry::new();
        let (parent_id, parent_arc) = create_surface_in_registry(&mut registry);
        let (child_id, child_arc) = create_surface_in_registry(&mut registry);

        let result = get_subsurface(0, 0, child_id, parent_id, &registry);
        assert!(result.is_ok());

        let child_surface = child_arc.lock().unwrap();
        assert_eq!(child_surface.parent, Some(parent_id));
        match &child_surface.role {
            Some(SurfaceRole::Subsurface(sub_state)) => {
                assert_eq!(sub_state.parent_id, parent_id);
                assert_eq!(sub_state.sync_mode, SubsurfaceSyncMode::Synchronized); // Default
                assert_eq!(sub_state.desired_position, (0,0)); // Default
            }
            _ => panic!("Child surface does not have Subsurface role."),
        }

        let parent_surface = parent_arc.lock().unwrap();
        assert!(parent_surface.children.contains(&child_id));
    }

    #[test]
    fn test_get_subsurface_fail_already_role() {
        let mut registry = SurfaceRegistry::new();
        let (parent_id, _) = create_surface_in_registry(&mut registry);
        let (child_id, child_arc) = create_surface_in_registry(&mut registry);

        // Give child a role first
        child_arc.lock().unwrap().role = Some(SurfaceRole::Toplevel);

        let result = get_subsurface(0, 0, child_id, parent_id, &registry);
        assert!(matches!(result, Err(SubsurfaceError::SurfaceHasRole)));
    }

    #[test]
    fn test_get_subsurface_fail_cycle_direct() {
        let mut registry = SurfaceRegistry::new();
        let (s1_id, _) = create_surface_in_registry(&mut registry);

        let result = get_subsurface(0, 0, s1_id, s1_id, &registry);
        assert!(matches!(result, Err(SubsurfaceError::CycleDetected)));
    }

    #[test]
    fn test_get_subsurface_fail_cycle_indirect() {
        let mut registry = SurfaceRegistry::new();
        let (s1_id, s1_arc) = create_surface_in_registry(&mut registry);
        let (s2_id, s2_arc) = create_surface_in_registry(&mut registry);
        let (s3_id, _) = create_surface_in_registry(&mut registry);

        // s1 -> s2 (s2 is child of s1)
        get_subsurface(0,0, s2_id, s1_id, &registry).unwrap();

        // s2 -> s3 (s3 is child of s2)
        get_subsurface(0,0, s3_id, s2_id, &registry).unwrap();

        // Attempt s3 -> s1 (should fail, creates s1 -> s2 -> s3 -> s1 cycle)
        let result = get_subsurface(0, 0, s1_id, s3_id, &registry);
        assert!(matches!(result, Err(SubsurfaceError::CycleDetected)));
    }

    #[test]
    fn test_destroy_subsurface_role() {
        let mut registry = SurfaceRegistry::new();
        let (parent_id, parent_arc) = create_surface_in_registry(&mut registry);
        let (child_id, child_arc) = create_surface_in_registry(&mut registry);

        get_subsurface(0, 0, child_id, parent_id, &registry).unwrap();
        assert!(child_arc.lock().unwrap().role.is_some());
        assert!(parent_arc.lock().unwrap().children.contains(&child_id));

        let result = destroy_subsurface_role(child_id, &mut registry);
        assert!(result.is_ok());

        assert!(child_arc.lock().unwrap().role.is_none(), "Child role should be None after destroy.");
        assert!(child_arc.lock().unwrap().parent.is_none(), "Child parent link should be None.");
        assert!(!parent_arc.lock().unwrap().children.contains(&child_id), "Child should be removed from parent's children list.");
    }

    #[test]
    fn test_parent_destruction_unmaps_children() {
        let mut registry = SurfaceRegistry::new();
        let mut buffer_manager = BufferManager::new(); // Needed for unregister_surface

        let (parent_id, _) = create_surface_in_registry(&mut registry);
        let (child1_id, child1_arc) = create_surface_in_registry(&mut registry);
        let (child2_id, child2_arc) = create_surface_in_registry(&mut registry);

        get_subsurface(0,0, child1_id, parent_id, &registry).unwrap();
        get_subsurface(0,0, child2_id, parent_id, &registry).unwrap();

        assert!(child1_arc.lock().unwrap().parent.is_some());
        assert!(child2_arc.lock().unwrap().parent.is_some());

        // Unregister (destroy) the parent surface
        registry.unregister_surface(parent_id, &mut buffer_manager).unwrap();

        // Children should still be in the registry but "orphaned" (parent=None)
        // Their role as Subsurface might still exist but parent_id in SubsurfaceState would be sentinel.
        let child1_surface = child1_arc.lock().unwrap();
        assert!(child1_surface.parent.is_none(), "Child1 should be orphaned (parent=None).");
        if let Some(SurfaceRole::Subsurface(ref state)) = child1_surface.role {
            // The parent_id in SubsurfaceState is set to a new unique ID (sentinel)
            // by prepare_for_destruction to signify it's no longer attached to the original parent.
            assert_ne!(state.parent_id, parent_id, "Child1's SubsurfaceState.parent_id should not be the old parent_id.");
        } else {
            // Depending on exact logic in prepare_for_destruction, role might be cleared or state updated.
            // Current prepare_for_destruction updates SubsurfaceState.parent_id.
        }

        let child2_surface = child2_arc.lock().unwrap();
        assert!(child2_surface.parent.is_none(), "Child2 should be orphaned.");
    }

    #[test]
    fn test_subsurface_set_position() {
        let mut registry = SurfaceRegistry::new();
        let (parent_id, _) = create_surface_in_registry(&mut registry);
        let (child_id, child_arc) = create_surface_in_registry(&mut registry);
        get_subsurface(0,0, child_id, parent_id, &registry).unwrap();

        let result = set_position(child_id, 100, 50, &registry);
        assert!(result.is_ok());

        match &child_arc.lock().unwrap().role {
            Some(SurfaceRole::Subsurface(state)) => {
                assert_eq!(state.desired_position, (100,50));
                // current_position would be updated on parent commit, not tested directly here yet.
            }
            _ => panic!("Not a subsurface"),
        }
    }

    #[test]
    fn test_subsurface_stacking() {
        let mut registry = SurfaceRegistry::new();
        let (p_id, p_arc) = create_surface_in_registry(&mut registry);
        let (s1_id, _) = create_surface_in_registry(&mut registry);
        let (s2_id, _) = create_surface_in_registry(&mut registry);
        let (s3_id, _) = create_surface_in_registry(&mut registry);

        get_subsurface(0,0,s1_id, p_id, &registry).unwrap(); // p -> [s1]
        get_subsurface(0,0,s2_id, p_id, &registry).unwrap(); // p -> [s1, s2]
        get_subsurface(0,0,s3_id, p_id, &registry).unwrap(); // p -> [s1, s2, s3]

        assert_eq!(p_arc.lock().unwrap().children, vec![s1_id, s2_id, s3_id]);

        // Place s1 above s2: s2, s1, s3
        place_above(s1_id, s2_id, &registry).unwrap();
        assert_eq!(p_arc.lock().unwrap().children, vec![s2_id, s1_id, s3_id], "s1 above s2");

        // Place s3 below s2: s3, s2, s1
        place_below(s3_id, s2_id, &registry).unwrap();
        assert_eq!(p_arc.lock().unwrap().children, vec![s3_id, s2_id, s1_id], "s3 below s2");

        // Place s1 above non-existent sibling (should go to top)
        let non_existent_sibling = SurfaceId::new_unique();
        place_above(s1_id, non_existent_sibling, &registry).unwrap();
        assert_eq!(p_arc.lock().unwrap().children.last(), Some(&s1_id), "s1 at top due to invalid sibling");

        // Place s2 below non-existent sibling (should go to bottom)
        // Current order might be [s3, sX, s1] (sX is s2 from previous state)
        // Let's reset for clarity for this part of test
        p_arc.lock().unwrap().children = vec![s3_id, s1_id]; // s2 was removed by previous place_above
        get_subsurface(0,0,s2_id, p_id, &registry).unwrap(); // re-add s2: [s3, s1, s2]
        assert_eq!(p_arc.lock().unwrap().children, vec![s3_id, s1_id, s2_id]);

        place_below(s2_id, non_existent_sibling, &registry).unwrap();
        assert_eq!(p_arc.lock().unwrap().children.first(), Some(&s2_id), "s2 at bottom due to invalid sibling");
    }

    #[test]
    fn test_subsurface_set_sync_desync() {
        let mut registry = SurfaceRegistry::new();
        let (parent_id, _) = create_surface_in_registry(&mut registry);
        let (child_id, child_arc) = create_surface_in_registry(&mut registry);
        get_subsurface(0,0, child_id, parent_id, &registry).unwrap();

        // Default is Synchronized
        match &child_arc.lock().unwrap().role {
            Some(SurfaceRole::Subsurface(state)) => assert_eq!(state.sync_mode, SubsurfaceSyncMode::Synchronized),
            _ => panic!(),
        }

        set_sync_mode(child_id, SubsurfaceSyncMode::Desynchronized, &registry).unwrap();
        match &child_arc.lock().unwrap().role {
            Some(SurfaceRole::Subsurface(state)) => assert_eq!(state.sync_mode, SubsurfaceSyncMode::Desynchronized),
            _ => panic!(),
        }

        set_sync_mode(child_id, SubsurfaceSyncMode::Synchronized, &registry).unwrap();
        match &child_arc.lock().unwrap().role {
            Some(SurfaceRole::Subsurface(state)) => assert_eq!(state.sync_mode, SubsurfaceSyncMode::Synchronized),
            _ => panic!(),
        }
    }

    #[test]
    fn test_commit_synchronized_subsurface_caches_state() {
        let mut registry = SurfaceRegistry::new();
        let mut buffer_manager = BufferManager::new();
        let client_id_val = ClientId::new(100);

        let (parent_id, parent_arc) = create_surface_in_registry(&mut registry);
        let (child_id, child_arc) = create_surface_in_registry(&mut registry);

        // Make child a synchronized subsurface of parent
        get_subsurface(0, 0, child_id, parent_id, &registry).unwrap();
        set_sync_mode(child_id, SubsurfaceSyncMode::Synchronized, &registry).unwrap();

        // Attach a buffer to the child surface
        let buffer1 = buffer_manager.register_buffer(BufferType::Shm, 10, 10, 40, BufferFormat::Argb8888, Some(client_id_val));
        let buffer1_id = buffer1.lock().unwrap().id;

        {
            let mut child_surface = child_arc.lock().unwrap();
            child_surface.attach_buffer(&mut buffer_manager, Some(buffer1.clone()), client_id_val, 0, 0).unwrap();

            // Commit the child (synchronized)
            let commit_res = child_surface.commit(&mut buffer_manager);
            assert!(commit_res.is_ok());

            // Verify state is cached, not applied to current_buffer
            assert!(child_surface.current_buffer.is_none(), "Child's current_buffer should be None before parent commit.");
            assert!(child_surface.cached_pending_buffer.is_some(), "Child's cached_pending_buffer should be Some.");
            assert_eq!(child_surface.cached_pending_buffer.as_ref().unwrap().lock().unwrap().id, buffer1_id);
            assert!(child_surface.cached_pending_attributes.is_some());
            assert_eq!(child_surface.cached_pending_attributes.unwrap().size, (10,10));

            match &child_surface.role {
                Some(SurfaceRole::Subsurface(state)) => {
                    assert!(state.needs_apply_on_parent_commit, "needs_apply_on_parent_commit should be true.");
                }
                _ => panic!("Child not a subsurface"),
            }
        }

        // Simulate parent committing (which would lead to this call for the child)
        {
            let mut child_surface = child_arc.lock().unwrap();
            let apply_res = child_surface.apply_cached_state_from_sync(&mut buffer_manager);
            assert!(apply_res.is_ok());

            assert!(child_surface.current_buffer.is_some(), "Child's current_buffer should be Some after apply_cached_state.");
            assert_eq!(child_surface.current_buffer.as_ref().unwrap().lock().unwrap().id, buffer1_id);
            assert!(child_surface.cached_pending_buffer.is_none(), "Cache should be cleared after apply.");
            assert!(child_surface.cached_pending_attributes.is_none(), "Cache should be cleared after apply.");
             match &child_surface.role {
                Some(SurfaceRole::Subsurface(state)) => {
                    assert!(!state.needs_apply_on_parent_commit, "needs_apply_on_parent_commit should be false after apply.");
                }
                _ => panic!("Child not a subsurface"),
            }
        }
    }

    #[test]
    fn test_commit_desynchronized_subsurface_applies_directly() {
        let mut registry = SurfaceRegistry::new();
        let mut buffer_manager = BufferManager::new();
        let client_id_val = ClientId::new(101);

        let (parent_id, _) = create_surface_in_registry(&mut registry);
        let (child_id, child_arc) = create_surface_in_registry(&mut registry);

        get_subsurface(0, 0, child_id, parent_id, &registry).unwrap();
        set_sync_mode(child_id, SubsurfaceSyncMode::Desynchronized, &registry).unwrap();

        let buffer1 = buffer_manager.register_buffer(BufferType::Shm, 20, 20, 80, BufferFormat::Argb8888, Some(client_id_val));
        let buffer1_id = buffer1.lock().unwrap().id;

        {
            let mut child_surface = child_arc.lock().unwrap();
            child_surface.attach_buffer(&mut buffer_manager, Some(buffer1.clone()), client_id_val, 0, 0).unwrap();

            let commit_res = child_surface.commit(&mut buffer_manager);
            assert!(commit_res.is_ok());

            // Verify state is applied directly
            assert!(child_surface.current_buffer.is_some(), "Child's current_buffer should be Some for desync.");
            assert_eq!(child_surface.current_buffer.as_ref().unwrap().lock().unwrap().id, buffer1_id);
            assert!(child_surface.cached_pending_buffer.is_none(), "Cache should be None for desync.");
            assert!(child_surface.cached_pending_attributes.is_none(), "Cache should be None for desync.");

            match &child_surface.role {
                Some(SurfaceRole::Subsurface(state)) => {
                    assert!(!state.needs_apply_on_parent_commit, "needs_apply_on_parent_commit should be false for desync.");
                }
                _ => panic!("Child not a subsurface"),
            }
        }
    }
}
