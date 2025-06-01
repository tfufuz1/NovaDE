//! Manages regions, which are collections of rectangles used for defining
//! areas like damage, input, or opaque regions for surfaces.

use crate::surface::Rectangle; // Using Rectangle from surface.rs
use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
use std::collections::HashMap;

/// Represents a unique identifier for a `Region`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionId(u64);

impl RegionId {
    /// Creates a new, unique `RegionId`.
    pub fn new_unique() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        RegionId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Represents a region as a collection of non-overlapping rectangles.
///
/// The methods `add` and `subtract` work to maintain this non-overlapping property,
/// though `add` might use a simplification pass that could be improved for performance.
#[derive(Debug, Clone)]
pub struct Region {
    /// The unique identifier for this region.
    pub id: RegionId,
    /// The set of non-overlapping rectangles that define this region.
    /// The goal is to keep these rectangles disjoint.
    rectangles: Vec<Rectangle>,
}

impl Region {
    /// Creates a new, empty `Region`.
    ///
    /// # Arguments
    /// * `id`: The unique `RegionId` for this region.
    pub fn new(id: RegionId) -> Self {
        Self {
            id,
            rectangles: Vec::new(),
        }
    }

    /// Clears all rectangles from the region, making it empty.
    pub fn clear(&mut self) {
        self.rectangles.clear();
    }

    /// Returns a clone of the rectangles currently defining the region.
    /// These rectangles are intended to be non-overlapping.
    pub fn get_rectangles(&self) -> Vec<Rectangle> {
        self.rectangles.clone()
    }

    // Basic add: adds rect and tries to merge. More sophisticated logic needed for true non-overlapping regions.
    // For now, this is a simplified version focusing on growing the region.
    // A full implementation would involve complex clipping and merging.
    // Iterative pairwise merge:
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
                    // Only merge if the union is a simple rectangle (area matches or they were adjacent and now form a simple rect)
                    // This check is tricky. For now, always merge if intersecting or perfectly adjacent.
                    // A more robust check is needed for complex cases to ensure simplification.
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


    /// Adds a rectangle to the region.
    ///
    /// The region is then simplified by merging overlapping or adjacent rectangles
    /// to maintain a set of disjoint rectangles representing the total area.
    /// If `new_rect` is empty, the region is unchanged.
    ///
    /// # Arguments
    /// * `new_rect`: The `Rectangle` to add to this region.
    pub fn add(&mut self, new_rect: Rectangle) {
        if new_rect.is_empty() {
            return;
        }
        self.rectangles.push(new_rect);
        Region::simplify_rectangles(&mut self.rectangles);
    }

    /// Subtracts a rectangle from the region.
    ///
    /// This operation may fragment existing rectangles in the region.
    /// If `sub_rect` is empty, the region is unchanged.
    ///
    /// # Arguments
    /// * `sub_rect`: The `Rectangle` to subtract from this region.
    ///
    /// # Notes
    /// The current implementation of subtraction is complex and involves fragmenting
    /// rectangles.
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

            // Top part
            if existing_rect.y < sub_rect.y {
                new_rects.push(Rectangle::new(existing_rect.x, existing_rect.y, existing_rect.width, sub_rect.y - existing_rect.y));
            }
            // Bottom part
            if existing_rect.y + existing_rect.height > sub_rect.y + sub_rect.height {
                new_rects.push(Rectangle::new(existing_rect.x, sub_rect.y + sub_rect.height, existing_rect.width, (existing_rect.y + existing_rect.height) - (sub_rect.y + sub_rect.height)));
            }
            // Left part (within the vertical overlap of sub_rect and existing_rect)
            let y_overlap_start = existing_rect.y.max(sub_rect.y);
            let y_overlap_end = (existing_rect.y + existing_rect.height).min(sub_rect.y + sub_rect.height);
            if y_overlap_start < y_overlap_end { // If there is a vertical overlap
                if existing_rect.x < sub_rect.x {
                    new_rects.push(Rectangle::new(existing_rect.x, y_overlap_start, sub_rect.x - existing_rect.x, y_overlap_end - y_overlap_start));
                }
                // Right part
                if existing_rect.x + existing_rect.width > sub_rect.x + sub_rect.width {
                    new_rects.push(Rectangle::new(sub_rect.x + sub_rect.width, y_overlap_start, (existing_rect.x + existing_rect.width) - (sub_rect.x + sub_rect.width), y_overlap_end - y_overlap_start));
                }
            }
        }
        self.rectangles = new_rects;
        // Subtraction can create many small or overlapping/adjacent rectangles. Simplify.
        Region::simplify_rectangles(&mut self.rectangles);
    }
}


/// Manages a collection of `Region` objects.
#[derive(Default)]
pub struct RegionRegistry {
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
    /// # Returns
    /// A tuple containing the new `RegionId` and an `Arc<Mutex<Region>>` to the created region.
    pub fn create_region(&mut self) -> (RegionId, Arc<Mutex<Region>>) {
        let id = RegionId::new_unique();
        let region = Arc::new(Mutex::new(Region::new(id)));
        self.regions.insert(id, region.clone());
        (id, region)
    }

    /// Retrieves a shared pointer to a `Region` by its ID.
    ///
    /// # Arguments
    /// * `id`: The `RegionId` of the region to retrieve.
    ///
    /// # Returns
    /// An `Option<Arc<Mutex<Region>>>`. Returns `Some` if the region is found, `None` otherwise.
    pub fn get_region(&self, id: RegionId) -> Option<Arc<Mutex<Region>>> {
        self.regions.get(&id).cloned()
    }

    /// Destroys a `Region` by removing it from the registry.
    ///
    /// # Arguments
    /// * `id`: The `RegionId` of the region to destroy.
    ///
    /// # Returns
    /// An `Option<Arc<Mutex<Region>>>` containing the removed region if it existed, `None` otherwise.
    /// The returned `Arc` can be used if the caller needs to perform any final operations on the
    /// region's data before it's dropped (if this is the last `Arc`).
    pub fn destroy_region(&mut self, id: RegionId) -> Option<Arc<Mutex<Region>>> {
        self.regions.remove(&id)
    }
}
