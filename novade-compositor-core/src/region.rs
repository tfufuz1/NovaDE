//! Manages regions, which are collections of rectangles used for defining
//! areas like damage, input, or opaque regions for surfaces in the Novade compositor.
//!
//! This module provides the `Region` struct, representing a list of disjoint rectangles,
//! and the `RegionRegistry` for managing multiple `Region` instances.
//! It corresponds to the Wayland concept of `wl_region`.

use crate::surface::Rectangle; // Using Rectangle from surface.rs
use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
use std::collections::HashMap;

/// Represents a unique identifier for a `Region`.
///
/// This newtype wrapper around `u64` ensures type safety when referencing regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionId(u64);

impl RegionId {
    /// Creates a new, unique `RegionId`.
    ///
    /// This uses a global atomic counter to ensure that every ID generated is unique
    /// across the compositor session.
    pub fn new_unique() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1); // Start IDs from 1 for clarity.
        RegionId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Represents a region as a collection of non-overlapping (disjoint) rectangles.
///
/// This struct is analogous to a `wl_region` object in the Wayland protocol.
/// It is used to define areas of a surface, such as the opaque region, input region,
/// or damage tracked by the compositor.
///
/// Operations like `add` and `subtract` modify the set of rectangles and attempt
/// to maintain a simplified, disjoint representation of the region.
#[derive(Debug, Clone)]
pub struct Region {
    /// The unique identifier for this region.
    pub id: RegionId,
    /// Internal list of disjoint rectangles that constitute this region.
    /// The operations on `Region` aim to keep this list simplified, meaning
    /// rectangles should not overlap, and adjacent ones might be merged.
    rectangles: Vec<Rectangle>,
}

impl Region {
    /// Creates a new, empty `Region` with a given `RegionId`.
    ///
    /// An empty region contains no rectangles.
    ///
    /// # Arguments
    /// * `id`: The unique `RegionId` to assign to this region.
    pub fn new(id: RegionId) -> Self {
        Self {
            id,
            rectangles: Vec::new(),
        }
    }

    /// Clears all rectangles from the region, making it effectively empty.
    pub fn clear(&mut self) {
        self.rectangles.clear();
    }

    /// Checks if the region is empty (contains no rectangles).
    ///
    /// # Returns
    /// `true` if the region has no rectangles, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.rectangles.is_empty()
    }

    /// Returns a clone of the list of disjoint rectangles that currently define the region.
    ///
    /// This provides read-only access to the region's geometry.
    ///
    /// # Returns
    /// A `Vec<Rectangle>` containing clones of the region's constituent rectangles.
    pub fn get_rectangles(&self) -> Vec<Rectangle> {
        self.rectangles.clone()
    }

    /// Internal helper function to simplify the list of rectangles in a region.
    ///
    /// This function attempts to merge overlapping or exactly adjacent rectangles
    /// to reduce their number and maintain a more canonical representation.
    /// It also removes any empty rectangles.
    ///
    /// Note: The current merging logic is a basic iterative pairwise check.
    /// More sophisticated algorithms (e.g., sweep-line) could provide better
    /// simplification for complex scenarios but are more complex to implement.
    fn simplify_rectangles(rects: &mut Vec<Rectangle>) {
        let mut i = 0;
        while i < rects.len() {
            let mut j = i + 1;
            let mut merged_current = false;
            while j < rects.len() {
                if rects[i].intersects(&rects[j]) ||
                   // Check for exact adjacency to merge into a single larger rectangle
                   (rects[i].x == rects[j].x && rects[i].width == rects[j].width && (rects[i].y == rects[j].y + rects[j].height || rects[i].y + rects[i].height == rects[j].y)) || // Vertical adjacency
                   (rects[i].y == rects[j].y && rects[i].height == rects[j].height && (rects[i].x == rects[j].x + rects[j].width || rects[i].x + rects[i].width == rects[j].x))    // Horizontal adjacency
                {
                    let unioned = rects[i].union(&rects[j]);
                    rects[i] = unioned;
                    rects.remove(j);
                    merged_current = true;
                } else {
                    j += 1;
                }
            }
            if merged_current {
                i = 0; // Restart scan as rects[i] changed and might merge with earlier ones
            } else {
                i += 1;
            }
        }
        // Remove empty rectangles that might result from subtractions or initial adds
        rects.retain(|r| !r.is_empty());
    }


    /// Adds a rectangle to the region, corresponding to `wl_region.add`.
    ///
    /// The region is then simplified by merging overlapping or adjacent rectangles
    /// to maintain a set of disjoint rectangles representing the total area.
    /// If `new_rect` is empty (has non-positive width or height), the region is unchanged.
    ///
    /// # Arguments
    /// * `new_rect`: The `Rectangle` to add to this region. Its coordinates are relative
    ///   to the same origin as the region itself.
    pub fn add(&mut self, new_rect: Rectangle) {
        if new_rect.is_empty() {
            return;
        }
        self.rectangles.push(new_rect);
        Region::simplify_rectangles(&mut self.rectangles);
    }

    /// Subtracts a rectangle from the region, corresponding to `wl_region.subtract`.
    ///
    /// This operation modifies the region by removing the area covered by `sub_rect`.
    /// This may involve fragmenting existing rectangles within the region into smaller pieces.
    /// The resulting set of rectangles is then simplified.
    /// If `sub_rect` is empty, or the region itself is empty, no change occurs.
    ///
    /// # Arguments
    /// * `sub_rect`: The `Rectangle` to subtract from this region. Its coordinates are
    ///   relative to the same origin as the region itself.
    ///
    /// # Notes
    /// The current implementation of subtraction involves iterating through existing
    /// rectangles and calculating the remaining fragments. This can be complex and
    /// potentially produce a large number of small rectangles before simplification.
    pub fn subtract(&mut self, sub_rect: Rectangle) {
        if sub_rect.is_empty() || self.rectangles.is_empty() {
            return;
        }

        let mut new_rects: Vec<Rectangle> = Vec::new();
        for existing_rect in &self.rectangles {
            if !existing_rect.intersects(&sub_rect) {
                new_rects.push(*existing_rect);
                continue;
            }

            // If existing_rect is entirely contained within sub_rect, it's removed.
            if sub_rect.x <= existing_rect.x &&
               sub_rect.y <= existing_rect.y &&
               sub_rect.x + sub_rect.width >= existing_rect.x + existing_rect.width &&
               sub_rect.y + sub_rect.height >= existing_rect.y + existing_rect.height {
                continue; // This rectangle is fully subtracted
            }

            // Fragmentation logic:
            // Calculate up to 4 rectangles representing existing_rect - sub_rect

            // Top part (of existing_rect, above sub_rect's top edge)
            if existing_rect.y < sub_rect.y {
                new_rects.push(Rectangle::new(existing_rect.x, existing_rect.y, existing_rect.width, sub_rect.y - existing_rect.y));
            }
            // Bottom part (of existing_rect, below sub_rect's bottom edge)
            if existing_rect.y + existing_rect.height > sub_rect.y + sub_rect.height {
                new_rects.push(Rectangle::new(existing_rect.x, sub_rect.y + sub_rect.height, existing_rect.width, (existing_rect.y + existing_rect.height) - (sub_rect.y + sub_rect.height)));
            }
            // Left part (of existing_rect, to the left of sub_rect's left edge, within vertical overlap)
            let y_overlap_start = existing_rect.y.max(sub_rect.y);
            let y_overlap_end = (existing_rect.y + existing_rect.height).min(sub_rect.y + sub_rect.height);
            if y_overlap_start < y_overlap_end { // If there is a vertical overlap
                if existing_rect.x < sub_rect.x {
                    new_rects.push(Rectangle::new(existing_rect.x, y_overlap_start, sub_rect.x - existing_rect.x, y_overlap_end - y_overlap_start));
                }
                // Right part (of existing_rect, to the right of sub_rect's right edge, within vertical overlap)
                if existing_rect.x + existing_rect.width > sub_rect.x + sub_rect.width {
                    new_rects.push(Rectangle::new(sub_rect.x + sub_rect.width, y_overlap_start, (existing_rect.x + existing_rect.width) - (sub_rect.x + sub_rect.width), y_overlap_end - y_overlap_start));
                }
            }
        }
        self.rectangles = new_rects;
        // Subtraction can create many small or overlapping/adjacent rectangles. Simplify.
        Region::simplify_rectangles(&mut self.rectangles);
    }

    /// Checks if any part of this region intersects with the given `rect`.
    ///
    /// # Arguments
    /// * `rect`: The `Rectangle` to check for intersection.
    ///
    /// # Returns
    /// `true` if there is any overlap between this region and `rect`, `false` otherwise.
    /// Returns `false` if `rect` is empty or this region is empty.
    pub fn intersects_rect(&self, rect: &Rectangle) -> bool {
        if rect.is_empty() || self.is_empty() {
            return false;
        }
        self.rectangles.iter().any(|r| r.intersects(rect))
    }

    /// Checks if the region contains the given point `(x, y)`.
    ///
    /// A point is considered contained if it falls within any of the disjoint
    /// rectangles that make up this region. Edges are typically handled such that
    /// the start coordinate (x, y) is inclusive, and the end coordinate (x+width, y+height)
    /// is exclusive.
    ///
    /// # Arguments
    /// * `x`: The x-coordinate of the point.
    /// * `y`: The y-coordinate of the point.
    ///
    /// # Returns
    /// `true` if the point is within this region, `false` otherwise.
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.rectangles.iter().any(|r| {
            !r.is_empty() &&
            x >= r.x && x < (r.x + r.width) &&
            y >= r.y && y < (r.y + r.height)
        })
    }
}


/// Manages a collection of `Region` objects within the compositor.
///
/// This registry allows for the creation, retrieval, and destruction of `Region`
/// instances, each identified by a unique `RegionId`. It is analogous to how a
/// Wayland compositor would manage `wl_region` objects.
#[derive(Default)]
pub struct RegionRegistry {
    /// Internal storage for regions, mapping `RegionId` to a shared, mutable `Region`.
    regions: HashMap<RegionId, Arc<Mutex<Region>>>,
}

impl RegionRegistry {
    /// Creates a new, empty `RegionRegistry`.
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
        }
    }

    /// Creates a new, empty `Region`, stores it in the registry, and returns its ID and a shared pointer.
    ///
    /// This is the primary way to instantiate new regions that are managed by the compositor.
    /// The returned `Arc<Mutex<Region>>` allows for shared, mutable access to the region data.
    ///
    /// # Returns
    /// A tuple containing the new unique `RegionId` and an `Arc<Mutex<Region>>` to the created region.
    pub fn create_region(&mut self) -> (RegionId, Arc<Mutex<Region>>) {
        let id = RegionId::new_unique();
        let region = Arc::new(Mutex::new(Region::new(id)));
        self.regions.insert(id, region.clone());
        (id, region)
    }

    /// Retrieves a shared, mutable pointer to a `Region` by its ID.
    ///
    /// This allows other parts of the compositor to access and modify a region's
    /// geometry after it has been created.
    ///
    /// # Arguments
    /// * `id`: The `RegionId` of the region to retrieve.
    ///
    /// # Returns
    /// An `Option<Arc<Mutex<Region>>>`. Returns `Some` containing the shared pointer
    /// if the region is found, `None` otherwise.
    pub fn get_region(&self, id: RegionId) -> Option<Arc<Mutex<Region>>> {
        self.regions.get(&id).cloned()
    }

    /// Destroys a `Region` by removing it from the registry.
    ///
    /// This effectively makes the region inaccessible for future operations via its ID.
    /// The Wayland protocol equivalent is `wl_region.destroy`.
    ///
    /// # Arguments
    /// * `id`: The `RegionId` of the region to destroy.
    ///
    /// # Returns
    /// An `Option<Arc<Mutex<Region>>>` containing the removed region if it existed, `None` otherwise.
    /// The returned `Arc` can be used if the caller needs to perform any final operations on the
    /// region's data before its last reference is dropped and it is deallocated.
    pub fn destroy_region(&mut self, id: RegionId) -> Option<Arc<Mutex<Region>>> {
        self.regions.remove(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::Rectangle; // Ensure Rectangle is accessible

    // --- RegionId Tests ---
    #[test]
    fn test_region_id_uniqueness() {
        let id1 = RegionId::new_unique();
        let id2 = RegionId::new_unique();
        assert_ne!(id1, id2, "Region IDs should be unique");
    }

    // --- RegionRegistry Tests ---
    #[test]
    fn test_region_registry_create_destroy() {
        let mut registry = RegionRegistry::new();
        let (id, region_arc) = registry.create_region();

        // Verify creation and retrieval
        assert_eq!(id, region_arc.lock().unwrap().id, "Created region ID should match");
        assert!(registry.get_region(id).is_some(), "Region should exist in registry after creation");

        // Verify destruction
        let destroyed_region_arc = registry.destroy_region(id);
        assert!(destroyed_region_arc.is_some(), "Destroy_region should return the removed region");
        assert_eq!(id, destroyed_region_arc.unwrap().lock().unwrap().id, "Destroyed region ID should match");
        assert!(registry.get_region(id).is_none(), "Region should not exist in registry after destruction");
    }

    // --- Region Method Tests (Basic) ---
    #[test]
    fn test_region_new_is_empty() {
        let id = RegionId::new_unique();
        let region = Region::new(id);
        assert_eq!(region.id, id);
        assert!(region.rectangles.is_empty(), "Newly created region should have no rectangles");
        assert!(region.get_rectangles().is_empty(), "get_rectangles on new region should be empty");
        assert!(region.is_empty(), "is_empty on new region should be true");
    }

    #[test]
    fn test_region_clear() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(0, 0, 10, 10));
        assert!(!region.rectangles.is_empty(), "Region should not be empty after add");
        assert!(!region.is_empty(), "is_empty should be false after add");

        region.clear();
        assert!(region.rectangles.is_empty(), "Region should be empty after clear");
        assert!(region.is_empty(), "is_empty should be true after clear");
    }

    // --- Region::add() Tests ---
    #[test]
    fn test_add_single_rect() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        let rect = Rectangle::new(10, 10, 100, 100);
        region.add(rect);

        let rects = region.get_rectangles();
        assert_eq!(rects.len(), 1, "Region should contain one rectangle");
        assert_eq!(rects[0], rect, "The rectangle in the region should match the added one");
    }

    #[test]
    fn test_add_empty_rect() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        let rect = Rectangle::new(0,0,0,0);
        region.add(rect);
        assert!(region.get_rectangles().is_empty(), "Adding an empty rect should leave the region empty");

        region.add(Rectangle::new(10,10,10,10));
        region.add(Rectangle::new(0,0,-5,5)); // Add another invalid rect
        assert_eq!(region.get_rectangles().len(), 1, "Adding an invalid rect after a valid one should not corrupt");
    }

    #[test]
    fn test_add_disjoint_rects() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        let rect1 = Rectangle::new(0, 0, 10, 10);
        let rect2 = Rectangle::new(20, 20, 10, 10);

        region.add(rect1);
        region.add(rect2);

        let rects = region.get_rectangles();
        assert_eq!(rects.len(), 2, "Region should contain two disjoint rectangles");
        // Order might not be guaranteed, so check for presence
        assert!(rects.contains(&rect1));
        assert!(rects.contains(&rect2));
    }

    #[test]
    fn test_add_overlapping_rects_merge_to_one() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        // Overlap: [0,0,10,10] + [5,0,10,10] -> [0,0,15,10]
        region.add(Rectangle::new(0, 0, 10, 10));
        region.add(Rectangle::new(5, 0, 10, 10));

        let rects = region.get_rectangles();
        assert_eq!(rects.len(), 1, "Overlapping rectangles should merge into one");
        assert_eq!(rects[0], Rectangle::new(0, 0, 15, 10));
    }

    #[test]
    fn test_add_rect_contained_in_existing() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(0, 0, 20, 20));
        region.add(Rectangle::new(5, 5, 10, 10)); // Fully contained

        let rects = region.get_rectangles();
        assert_eq!(rects.len(), 1, "Adding a contained rectangle should not change the region");
        assert_eq!(rects[0], Rectangle::new(0, 0, 20, 20));
    }

    #[test]
    fn test_add_rect_containing_existing() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(5, 5, 10, 10));
        region.add(Rectangle::new(0, 0, 20, 20)); // Fully contains the previous one

        let rects = region.get_rectangles();
        assert_eq!(rects.len(), 1, "Adding a containing rectangle should result in the larger rect");
        assert_eq!(rects[0], Rectangle::new(0, 0, 20, 20));
    }

    #[test]
    fn test_add_adjacent_rects_merge_horizontal() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(0, 0, 10, 10));
        region.add(Rectangle::new(10, 0, 10, 10)); // Adjacent horizontally

        let rects = region.get_rectangles();
        assert_eq!(rects.len(), 1, "Horizontally adjacent rectangles should merge");
        assert_eq!(rects[0], Rectangle::new(0, 0, 20, 10));
    }

    #[test]
    fn test_add_adjacent_rects_merge_vertical() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(0, 0, 10, 10));
        region.add(Rectangle::new(0, 10, 10, 10)); // Adjacent vertically

        let rects = region.get_rectangles();
         assert_eq!(rects.len(), 1, "Vertically adjacent rectangles should merge. Got: {:?}", rects);
        assert_eq!(rects[0], Rectangle::new(0, 0, 10, 20));
    }

    // More complex merge tests for `add` would go here.
    // The current `simplify_rectangles` in `Region::add` is very basic and might not
    // correctly handle all complex merge scenarios (e.g., L-shapes becoming a single rect,
    // or multiple overlapping rects that don't form a single larger bounding box but
    // could be simplified to fewer disjoint rects).
    // For now, these tests reflect the current simple merge logic.

    // --- Region::subtract() Tests ---
    #[test]
    fn test_subtract_empty_from_region() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        let r1 = Rectangle::new(0,0,10,10);
        region.add(r1);
        region.subtract(Rectangle::new(0,0,0,0));
        assert_eq!(region.get_rectangles(), vec![r1]);
    }

    #[test]
    fn test_subtract_region_from_empty() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.subtract(Rectangle::new(0,0,10,10));
        assert!(region.get_rectangles().is_empty());
    }

    #[test]
    fn test_subtract_disjoint_rect() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        let r1 = Rectangle::new(0,0,10,10);
        region.add(r1);
        region.subtract(Rectangle::new(20,0,10,10));
        assert_eq!(region.get_rectangles(), vec![r1]);
    }

    #[test]
    fn test_subtract_rect_fully_containing_region_rect() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(5,5,10,10));
        region.subtract(Rectangle::new(0,0,20,20));
        assert!(region.get_rectangles().is_empty(), "Region should be empty after subtracting a containing rect. Got: {:?}", region.get_rectangles());
    }

    #[test]
    fn test_subtract_rect_fully_contained_in_region_rect() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(0,0,20,20));
        region.subtract(Rectangle::new(5,5,10,10));

        let rects = region.get_rectangles();
        // Expected: 4 rectangles around the subtracted hole.
        // [0,0,20,5], [0,5,5,10], [5,15,10,5], [15,5,5,10] - after simplification
        // Or: [0,0,20,5], [0,5,5,15], [5,15,15,5], [15,5,5,10] - another way to dice
        // The current logic produces:
        // Top: [0,0,20,5]
        // Bottom: [0,15,20,5]
        // Left (of hole): [0,5,5,10]
        // Right (of hole): [15,5,5,10]
        assert_eq!(rects.len(), 4, "Subtracting a contained rect should result in 4 fragments. Got: {:?}", rects);
        assert!(rects.contains(&Rectangle::new(0,0,20,5))); // Top
        assert!(rects.contains(&Rectangle::new(0,15,20,5))); // Bottom
        assert!(rects.contains(&Rectangle::new(0,5,5,10))); // Left
        assert!(rects.contains(&Rectangle::new(15,5,5,10))); // Right
    }

    #[test]
    fn test_subtract_partial_overlap_side() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(0,0,10,10));
        region.subtract(Rectangle::new(5,0,10,10)); // Subtracts right half

        let rects = region.get_rectangles();
        assert_eq!(rects.len(), 1, "Region should have one rectangle after partial side subtraction. Got: {:?}", rects);
        assert_eq!(rects[0], Rectangle::new(0,0,5,10));
    }

    #[test]
    fn test_subtract_partial_overlap_corner() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(0,0,10,10));
        region.subtract(Rectangle::new(5,5,10,10)); // Subtracts bottom-right corner area

        let rects = region.get_rectangles();
        // Expected: [0,0,10,5] (top part), [0,5,5,5] (bottom-left part)
        assert_eq!(rects.len(), 2, "Region should have two rectangles after corner subtraction. Got: {:?}", rects);
        assert!(rects.contains(&Rectangle::new(0,0,10,5)), "Missing top part"); // Top part
        assert!(rects.contains(&Rectangle::new(0,5,5,5)), "Missing bottom-left part"); // Bottom-left part
    }

    // --- Helper Method Tests ---
    #[test]
    fn test_region_intersects_rect() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(10, 10, 20, 20)); // Region is [10,10,20,20]
        region.add(Rectangle::new(50, 50, 10, 10)); // Region is [10,10,20,20] U [50,50,10,10]

        assert!(region.intersects_rect(&Rectangle::new(15, 15, 5, 5))); // Fully inside first rect
        assert!(region.intersects_rect(&Rectangle::new(5, 5, 10, 10))); // Overlaps top-left of first
        assert!(region.intersects_rect(&Rectangle::new(25, 25, 10, 10))); // Overlaps bottom-right of first
        assert!(region.intersects_rect(&Rectangle::new(55, 55, 5, 5))); // Fully inside second rect
        assert!(!region.intersects_rect(&Rectangle::new(0, 0, 5, 5))); // Disjoint
        assert!(!region.intersects_rect(&Rectangle::new(30, 10, 5, 20))); // Between rects
        assert!(!region.intersects_rect(&Rectangle::new(0,0,0,0))); // Empty rect
        assert!(!region.is_empty()); // Ensure region itself is not empty for these checks
    }

    #[test]
    fn test_region_contains_point() {
        let id = RegionId::new_unique();
        let mut region = Region::new(id);
        region.add(Rectangle::new(10, 10, 20, 20)); // [10,10] to [29,29]
        region.add(Rectangle::new(50, 50, 10, 10)); // [50,50] to [59,59]

        assert!(region.contains_point(15, 15)); // Inside first rect
        assert!(region.contains_point(10, 10)); // Edge of first rect (inclusive start)
        assert!(!region.contains_point(30, 30)); // Edge of first rect (exclusive end for x and y)
        assert!(!region.contains_point(30, 15)); // Edge of first rect (exclusive end for x)
        assert!(!region.contains_point(15, 30)); // Edge of first rect (exclusive end for y)
        assert!(region.contains_point(29, 29)); // Inside, near edge
        assert!(region.contains_point(55, 55)); // Inside second rect
        assert!(!region.contains_point(0, 0));   // Outside
        assert!(!region.contains_point(40, 40)); // Between rects
    }
}
