//! Logic for determining input focus based on pointer position and surface properties.

use crate::surface::{Surface, SurfaceId, SurfaceRegistryAccessor, Rectangle}; // Assuming SurfaceRegistryAccessor is in surface.rs
use crate::region::Region; // Assuming this is the correct path
use std::sync::{Arc, Mutex};

// TODO: Consider Z-ordering properly. For now, the last surface in the iteration that contains the point wins.
// TODO: Handle subsurfaces correctly. For now, focuses on the main surface of a tree.

/// Determines which surface is at a given global compositor coordinate.
///
/// This function iterates through registered surfaces, checking if the point
/// falls within their bounding box and input region.
///
/// # Arguments
/// * `point`: A tuple `(global_x, global_y)` representing the global compositor coordinates.
/// * `surface_registry`: An accessor to the `SurfaceRegistry` to iterate through surfaces.
///
/// # Returns
/// An `Option<SurfaceId>` of the topmost visible and hittable surface at the given point.
/// Returns `None` if no suitable surface is found.
pub fn surface_at(
    point: (f64, f64),
    surface_registry: &impl SurfaceRegistryAccessor,
    // Consider passing a list of top-level surfaces if Z-ordering is managed elsewhere.
    // For now, iterates all accessible surfaces from the registry.
) -> Option<SurfaceId> {
    let (px, py) = point;
    let mut found_surface_id: Option<SurfaceId> = None;

    // This assumes get_all_surfaces() or similar exists or can be implemented.
    // For now, we'll assume the registry can give us a way to iterate.
    // If SurfaceRegistry itself impls IntoIterator or has a method like `all_surfaces() -> Vec<Arc<Mutex<Surface>>>`

    // Placeholder: How to get all surfaces? Assume SurfaceRegistryAccessor provides a way.
    // This is a conceptual part that needs to be filled by SurfaceRegistry's actual API.
    // Let's imagine `surface_registry.get_all_surface_ids()` exists for now.

    // A more realistic approach would be to iterate surfaces in Z-order (topmost first).
    // The current iteration order is likely HashMap iteration order, which is arbitrary.
    // For a basic implementation, the *last* surface found that contains the point might be chosen.

    // To implement this properly, SurfaceRegistry needs a method like `surfaces_in_z_order()`
    // or `surface_at` should be a method of `SurfaceRegistry` itself.
    // For this subtask, we'll simulate a simple iteration.
    // The actual implementation of iterating surfaces depends on SurfaceRegistry design.
    // We'll assume a conceptual `get_all_surfaces_for_picking()` that returns relevant surfaces.

    // Let's simulate with a placeholder. This part needs actual SurfaceRegistry integration.
    // This is a simplification: in reality, you'd iterate from top to bottom.
    let all_surface_ids = surface_registry.get_all_surface_ids_for_picking_placeholder();

    for surface_id in all_surface_ids {
        if let Some(surface_arc) = surface_registry.get_surface(surface_id) {
            let surface = surface_arc.lock().unwrap(); // Handle potential poisoning

            // TODO: Only consider visible/mapped surfaces. Add a check like `surface.is_mapped()`.
            // TODO: Consider surface transformations if point is in a different coordinate space.

            let surface_rect = Rectangle::new(
                surface.current_attributes.position.0,
                surface.current_attributes.position.1,
                surface.current_attributes.size.0 as i32,
                surface.current_attributes.size.1 as i32,
            );

            if surface_rect.is_empty() {
                continue;
            }

            // Check if point is within the surface's bounding box (global coordinates)
            if px >= surface_rect.x as f64 && px < (surface_rect.x + surface_rect.width) as f64 &&
               py >= surface_rect.y as f64 && py < (surface_rect.y + surface_rect.height) as f64
            {
                // Point is within bounding box. Now check input region.
                // Input region coordinates are surface-local.
                let local_x = px - surface_rect.x as f64;
                let local_y = py - surface_rect.y as f64;

                match &surface.current_input_region {
                    Some(input_region) => {
                        if input_region.contains_point(local_x as i32, local_y as i32) {
                            // Point is within the specified input region.
                            // Simplification: first one found wins, or last one if iterating bottom-up.
                            // For true Z-order, this loop needs to be ordered, and we pick the first.
                            found_surface_id = Some(surface.id);
                        }
                    }
                    None => {
                        // No input region means the entire surface accepts input.
                        found_surface_id = Some(surface.id);
                    }
                }
            }
        }
    }
    found_surface_id
}

// Placeholder for SurfaceRegistryAccessor extension, if needed by focus.rs
// This trait would need to be implemented by the actual SurfaceRegistry.
// pub trait SurfaceRegistryFocusExt {
//     fn get_all_surface_ids_for_picking_placeholder(&self) -> Vec<SurfaceId>;
// }
// Replaced by adding get_all_surface_ids directly to SurfaceRegistryAccessor and SurfaceRegistry

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{Surface, SurfaceId, SurfaceRegistry, Rectangle, SurfaceAttributes, SurfaceRole};
    use crate::region::{Region, RegionId};
    use novade_buffer_manager::ClientId; // For creating surfaces
    use std::sync::{Arc, Mutex};

    fn test_client_id(id: u64) -> ClientId { ClientId::new(id) }

    // Helper to simplify test setup
    fn create_surface_with_attrs(
        registry: &mut SurfaceRegistry,
        client_id: ClientId,
        pos: (i32, i32),
        size: (u32, u32),
        input_region_opt: Option<Region>
    ) -> Arc<Mutex<Surface>> {
        let (_id, surface_arc) = registry.register_new_surface(client_id);
        {
            let mut surface = surface_arc.lock().unwrap();
            surface.current_attributes.position = pos;
            surface.current_attributes.size = size;
            surface.current_input_region = input_region_opt;
            // Assume surface is mapped/visible for picking tests
        }
        surface_arc
    }

    #[test]
    fn test_surface_at_no_surfaces() {
        let registry = SurfaceRegistry::new();
        assert!(surface_at((10.0, 10.0), &registry).is_none());
    }

    #[test]
    fn test_surface_at_single_surface_hit() {
        let mut registry = SurfaceRegistry::new();
        let client = test_client_id(1);
        let surface_arc = create_surface_with_attrs(&mut registry, client, (0,0), (100,100), None);
        let surface_id = surface_arc.lock().unwrap().id;

        assert_eq!(surface_at((50.0, 50.0), &registry), Some(surface_id));
    }

    #[test]
    fn test_surface_at_single_surface_miss() {
        let mut registry = SurfaceRegistry::new();
        let client = test_client_id(1);
        create_surface_with_attrs(&mut registry, client, (0,0), (100,100), None);

        assert!(surface_at((150.0, 150.0), &registry).is_none());
    }

    #[test]
    fn test_surface_at_input_region_hit() {
        let mut registry = SurfaceRegistry::new();
        let client = test_client_id(1);
        let mut input_region = Region::new(RegionId::new_unique());
        input_region.add(Rectangle::new(10,10,30,30)); // Input region [10,10, width 30, height 30]

        let surface_arc = create_surface_with_attrs(&mut registry, client, (0,0), (100,100), Some(input_region));
        let surface_id = surface_arc.lock().unwrap().id;

        assert_eq!(surface_at((20.0, 20.0), &registry), Some(surface_id)); // Hits within input region
    }

    #[test]
    fn test_surface_at_input_region_miss() {
        let mut registry = SurfaceRegistry::new();
        let client = test_client_id(1);
        let mut input_region = Region::new(RegionId::new_unique());
        input_region.add(Rectangle::new(10,10,30,30));

        create_surface_with_attrs(&mut registry, client, (0,0), (100,100), Some(input_region));

        assert!(surface_at((5.0, 5.0), &registry).is_none()); // Hits surface bounds but outside input region
        assert!(surface_at((50.0, 50.0), &registry).is_none()); // Hits surface bounds but outside input region
    }

    #[test]
    fn test_surface_at_overlapping_surfaces_no_zorder() {
        // Current surface_at iterates HashMap, order is not guaranteed.
        // This test assumes the *last* iterated surface containing the point will be returned.
        // This will need adjustment when proper Z-ordering is implemented.
        let mut registry = SurfaceRegistry::new();
        let client = test_client_id(1);

        let _surface1_arc = create_surface_with_attrs(&mut registry, client, (0,0), (100,100), None);
        let surface2_arc = create_surface_with_attrs(&mut registry, client, (10,10), (50,50), None); // Smaller, on top if iterated last
        let surface2_id = surface2_arc.lock().unwrap().id;

        // To make this test somewhat stable without true Z-order, we rely on the fact that
        // surface_at iterates what get_all_surface_ids returns. If HashMap iteration is consistent
        // (which it's not strictly guaranteed to be across all Rust versions/platforms for small N),
        // then the last one inserted *might* be the last one iterated.
        // A better test would mock get_all_surface_ids to control iteration order.
        // For now, we accept that this might pick surface1 if iteration order changes.
        // The important part is that *a* correct surface is picked.

        // To make it more deterministic for now, let's clear and re-insert in a specific order
        // to somewhat control the iteration if HashMap iterates in insertion order for few elements.
        // This is still not robust for testing Z-order.
        let surface_ids_before_clear: Vec<_> = registry.get_all_surface_ids();
        for id_to_remove in surface_ids_before_clear {
            registry.unregister_surface_for_test(id_to_remove);
        }
        assert!(registry.get_all_surface_ids().is_empty());

        let _s1_arc_re = create_surface_with_attrs(&mut registry, client, (0,0), (100,100), None);
        let s2_arc_re = create_surface_with_attrs(&mut registry, client, (10,10), (50,50), None);
        let s2_id_re = s2_arc_re.lock().unwrap().id;

        // If surface_at picks the last iterated surface that contains the point,
        // and HashMap iterates in something like insertion order here, s2 should be picked.
        let picked_id = surface_at((20.0, 20.0), &registry);
        assert_eq!(picked_id, Some(s2_id_re),
            "Expected surface 2 to be picked due to current iteration behavior. This test needs Z-order.");
    }
     #[test]
    fn test_surface_at_point_on_edge() {
        let mut registry = SurfaceRegistry::new();
        let client = test_client_id(1);
        let s1 = create_surface_with_attrs(&mut registry, client, (10, 10), (20, 20), None); // 10,10 to 29,29
        let s1_id = s1.lock().unwrap().id;

        assert_eq!(surface_at((10.0, 10.0), &registry), Some(s1_id)); // Top-left corner
        assert_eq!(surface_at((29.999, 10.0), &registry), Some(s1_id)); // Near top-right corner
        assert!(surface_at((30.0, 10.0), &registry).is_none());      // Just outside top-right corner

        assert_eq!(surface_at((10.0, 29.999), &registry), Some(s1_id)); // Near bottom-left corner
        assert!(surface_at((10.0, 30.0), &registry).is_none());      // Just outside bottom-left corner
    }
}

// Placeholder for SurfaceRegistryFocusExt extension, if needed by focus.rs
// This trait would need to be implemented by the actual SurfaceRegistry.
// pub trait SurfaceRegistryFocusExt {
//     fn get_all_surface_ids_for_picking_placeholder(&self) -> Vec<SurfaceId>;
// }
// Replaced by adding get_all_surface_ids directly to SurfaceRegistryAccessor and SurfaceRegistry
