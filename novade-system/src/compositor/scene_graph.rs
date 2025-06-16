// novade-system/src/compositor/scene_graph.rs
use novade_compositor_core::surface::{SurfaceId, SurfaceState}; // Assuming SurfaceState exists
// Assuming Point2D = Point<f32>, Size2D = Size<f32>, Rectangle = Rect<f32> from novade_core
use novade_core::types::geometry::{Point, Size, Rect};
use std::collections::{HashMap, HashSet}; // Added HashSet
use std::sync::Arc;

const GRID_CELL_SIZE: f32 = 256.0; // Example cell size
// MAX_OUTPUT constants are for initial placeholder, actual geometry is used in rebuild.
const MAX_OUTPUT_WIDTH_FOR_GRID: f32 = 1920.0 * 2.0;
const MAX_OUTPUT_HEIGHT_FOR_GRID: f32 = 1080.0 * 2.0;

#[derive(Debug)]
pub struct SpatialIndex {
    grid: HashMap<(i32, i32), Vec<SurfaceId>>,
    grid_cols: i32,
    grid_rows: i32,
    cell_size: f32,
    indexed_output_geometry: Rectangle,
}

impl SpatialIndex {
    pub fn new(output_geometry: &Rectangle, cell_size: f32) -> Self {
        // Use output_geometry.size.width and .size.height for cols/rows calculation
        let grid_cols = (output_geometry.size.width / cell_size).ceil() as i32;
        let grid_rows = (output_geometry.size.height / cell_size).ceil() as i32;
        SpatialIndex {
            grid: HashMap::new(),
            grid_cols,
            grid_rows,
            cell_size,
            indexed_output_geometry: *output_geometry,
        }
    }

    pub fn rebuild_corrected(&mut self, nodes: &[Arc<SceneGraphNode>], output_geometry: &Rectangle) {
        self.grid.clear();
        self.indexed_output_geometry = *output_geometry;
        self.grid_cols = (output_geometry.size.width / self.cell_size).ceil() as i32;
        self.grid_rows = (output_geometry.size.height / self.cell_size).ceil() as i32;

        for node in nodes {
            // Use node.clipped_rect for indexing, as this is its visible part on the output
            if node.clipped_rect.size.width == 0.0 || node.clipped_rect.size.height == 0.0 {
                continue;
            }

            // Calculate node's clipped_rect relative to the output_geometry's origin for cell indexing
            let relative_x = node.clipped_rect.origin.x - output_geometry.origin.x;
            let relative_y = node.clipped_rect.origin.y - output_geometry.origin.y;

            let min_c = (relative_x / self.cell_size).floor() as i32;
            let max_c = ((relative_x + node.clipped_rect.size.width) / self.cell_size).ceil() as i32;
            let min_r = (relative_y / self.cell_size).floor() as i32;
            let max_r = ((relative_y + node.clipped_rect.size.height) / self.cell_size).ceil() as i32;

            for r_idx in min_r..max_r { // Iterate up to max_r (exclusive)
                for c_idx in min_c..max_c { // Iterate up to max_c (exclusive)
                    // Ensure cells are within the valid grid range [0, grid_cols-1] and [0, grid_rows-1]
                    if r_idx >= 0 && r_idx < self.grid_rows && c_idx >= 0 && c_idx < self.grid_cols {
                        self.grid.entry((c_idx, r_idx)).or_default().push(node.surface_id);
                    }
                }
            }
        }
    }

    pub fn query(&self, query_rect: &Rectangle) -> HashSet<SurfaceId> {
        let mut potential_surfaces = HashSet::new();
        if query_rect.size.width == 0.0 || query_rect.size.height == 0.0 {
            return potential_surfaces;
        }

        // Calculate query_rect's position relative to the indexed_output_geometry's origin
        let relative_x = query_rect.origin.x - self.indexed_output_geometry.origin.x;
        let relative_y = query_rect.origin.y - self.indexed_output_geometry.origin.y;

        let min_c = (relative_x / self.cell_size).floor() as i32;
        let max_c = ((relative_x + query_rect.size.width) / self.cell_size).ceil() as i32;
        let min_r = (relative_y / self.cell_size).floor() as i32;
        let max_r = ((relative_y + query_rect.size.height) / self.cell_size).ceil() as i32;

        for r_idx in min_r..max_r {
            for c_idx in min_c..max_c {
                 if r_idx >= 0 && r_idx < self.grid_rows && c_idx >= 0 && c_idx < self.grid_cols {
                    if let Some(surfaces_in_cell) = self.grid.get(&(c_idx, r_idx)) {
                        for surface_id in surfaces_in_cell {
                            potential_surfaces.insert(*surface_id);
                        }
                    }
                }
            }
        }
        potential_surfaces
    }
}

// Define f32 based aliases
pub type Point2D = Point<f32>;
pub type Size2D = Size<f32>;
pub type Rectangle = Rect<f32>;

// Simplified Transform struct as novade-core does not provide it yet.
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    // Row-major 2x3 matrix for 2D affine transformations:
    // [[m00, m01, m02], (scale_x, skew_y, translate_x)
    //  [m10, m11, m12]] (skew_x, scale_y, translate_y)
    // Implicit third row is [0, 0, 1]
    pub matrix: [[f32; 3]; 2],
}

impl Transform {
    pub fn identity() -> Self {
        Transform {
            matrix: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
        }
    }

    // Concatenates self with other (result = self * other)
    pub fn then(&self, other: &Transform) -> Transform {
        let s = &self.matrix;
        let o = &other.matrix;
        Transform {
            matrix: [
                [
                    s[0][0] * o[0][0] + s[0][1] * o[1][0],
                    s[0][0] * o[0][1] + s[0][1] * o[1][1],
                    s[0][0] * o[0][2] + s[0][1] * o[1][2] + s[0][2],
                ],
                [
                    s[1][0] * o[0][0] + s[1][1] * o[1][0],
                    s[1][0] * o[0][1] + s[1][1] * o[1][1],
                    s[1][0] * o[0][2] + s[1][1] * o[1][2] + s[1][2],
                ],
            ],
        }
    }

    pub fn transform_point(&self, point: Point2D) -> Point2D {
        Point2D::new(
            self.matrix[0][0] * point.x + self.matrix[0][1] * point.y + self.matrix[0][2],
            self.matrix[1][0] * point.x + self.matrix[1][1] * point.y + self.matrix[1][2],
        )
    }

    pub fn transform_rect_bounding_box(&self, rect: Rectangle) -> Rectangle {
        let p0 = self.transform_point(Point2D::new(rect.origin.x, rect.origin.y));
        let p1 = self.transform_point(Point2D::new(rect.origin.x + rect.size.width, rect.origin.y));
        let p2 = self.transform_point(Point2D::new(rect.origin.x, rect.origin.y + rect.size.height));
        let p3 = self.transform_point(Point2D::new(rect.origin.x + rect.size.width, rect.origin.y + rect.size.height));

        let min_x = p0.x.min(p1.x).min(p2.x).min(p3.x);
        let max_x = p0.x.max(p1.x).max(p2.x).max(p3.x);
        let min_y = p0.y.min(p1.y).min(p2.y).min(p3.y);
        let max_y = p0.y.max(p1.y).max(p2.y).max(p3.y);

        Rectangle::from_coords(min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

// Represents a surface node in the scene graph
#[derive(Debug, Clone)]
pub struct SceneGraphNode {
    pub surface_id: SurfaceId,
    pub attributes: SurfaceAttributes, // Position, size, z-order, visibility, etc.
    pub final_transform: Transform, // Calculated final transform
    pub clipped_rect: Rectangle,    // Final rectangle after clipping
    pub children: Vec<Arc<SceneGraphNode>>, // For subsurfaces, if managed directly here
    pub z_order: i32,
    pub is_occluded: bool, // New field
}

// Helper trait for extended Rectangle methods
pub trait RectExt {
    fn is_empty_rect(&self) -> bool;
    fn is_fully_contained_by(&self, other: &Rectangle) -> bool;
}

impl RectExt for Rectangle {
    // Check if width or height is non-positive.
    // novade_core::Rect::is_empty checks if area is zero via Size::is_empty,
    // which checks T::is_zero(). For f32, T::zero() is 0.0.
    // This is fine, but being explicit with <= 0.0 is common for float dimensions.
    fn is_empty_rect(&self) -> bool {
        self.size.width <= 0.0 || self.size.height <= 0.0
    }

    fn is_fully_contained_by(&self, other: &Rectangle) -> bool {
        if self.is_empty_rect() { // An empty rect can be considered contained
            return true;
        }
        // If the container is empty and self is not, self cannot be contained.
        if other.is_empty_rect() {
            return false;
        }
        self.origin.x >= other.origin.x &&
        self.origin.y >= other.origin.y &&
        (self.origin.x + self.size.width) <= (other.origin.x + other.size.width) &&
        (self.origin.y + self.size.height) <= (other.origin.y + other.size.height)
    }
}


// Attributes of a surface relevant for scene graph construction
#[derive(Debug, Clone)]
pub struct SurfaceAttributes {
    pub position: Point2D,
    pub size: Size2D,
    pub transform: Transform, // Local transform of the surface
    pub is_visible: bool,
    pub z_order: i32, // Relative Z-order
    pub opaque_region: Option<Rectangle>, // For damage tracking and culling
    pub parent: Option<SurfaceId>, // New field
}

pub struct SceneGraph {
    nodes: Vec<Arc<SceneGraphNode>>, // Flattened list of top-level nodes for rendering, sorted by Z
    spatial_index: SpatialIndex, // New field
}

impl SceneGraph {
    pub fn new() -> Self {
        // Use from_coords for Rectangle consistent with novade-core
        let initial_output_placeholder = Rectangle::from_coords(0.0, 0.0, MAX_OUTPUT_WIDTH_FOR_GRID, MAX_OUTPUT_HEIGHT_FOR_GRID);
        SceneGraph {
            nodes: Vec::new(),
            spatial_index: SpatialIndex::new(&initial_output_placeholder, GRID_CELL_SIZE),
        }
    }

    pub fn update(
        &mut self,
        surface_data_map: &HashMap<SurfaceId, SurfaceAttributes>,
        output_geometry: &Rectangle // New parameter for output geometry
    ) {
        self.nodes.clear();
        let mut processing_nodes_map: HashMap<SurfaceId, SceneGraphNode> = HashMap::new();
        let mut global_transforms: HashMap<SurfaceId, Transform> = HashMap::new();

        let mut surface_ids_to_process: Vec<SurfaceId> = surface_data_map.keys().cloned().collect();
        // Simple Z-sorting for processing order (helps with parent-first but not guaranteed)
        surface_ids_to_process.sort_by_key(|id| surface_data_map.get(id).map_or(0, |attr| attr.z_order));


        for surface_id in &surface_ids_to_process {
            if let Some(attributes) = surface_data_map.get(surface_id) {
                if !attributes.is_visible {
                    continue;
                }

                let parent_transform = attributes.parent
                    .and_then(|pid| global_transforms.get(&pid))
                    .cloned()
                    .unwrap_or_else(Transform::identity);

                // Create a translation transform from the surface's position
                // This represents the surface's origin translation relative to its parent.
                let translation_transform = Transform {
                    matrix: [
                        [1.0, 0.0, attributes.position.x],
                        [0.0, 1.0, attributes.position.y],
                    ]
                };

                // The surface's local transform (e.g., scale, rotation specified by client, or buffer transform)
                let local_transform = &attributes.transform;

                // Calculate the world transform:
                // Parent's_World_Transform * Translation_Transform * Local_Surface_Transform
                let world_transform = parent_transform
                                      .then(&translation_transform)
                                      .then(local_transform);

                global_transforms.insert(*surface_id, world_transform.clone());

                // The surface's geometry is defined by its size, starting at (0,0) in its local coordinates
                let surface_rect_local = Rectangle::from_coords(0.0, 0.0, attributes.size.width, attributes.size.height);

                // Transform this local rectangle to world coordinates
                let transformed_bounding_box = world_transform.transform_rect_bounding_box(surface_rect_local);

                // Clip this world-coordinate rectangle against the output geometry
                let clipped_rect = transformed_bounding_box.intersection(output_geometry)
                    .unwrap_or_else(|| Rectangle::from_coords(0.0, 0.0, 0.0, 0.0)); // Zero sized if no intersection

                if clipped_rect.size.width > 0.0 && clipped_rect.size.height > 0.0 { // Only add if visible after clipping
                    processing_nodes_map.insert(*surface_id, SceneGraphNode {
                        surface_id: *surface_id,
                        attributes: attributes.clone(),
                        final_transform: world_transform, // Use the fully calculated world_transform
                        clipped_rect,
                        children: Vec::new(), // Still placeholder for subsurfaces
                        z_order: attributes.z_order,
                        is_occluded: false, // Initialize new field
                    });
                }
            }
        }

        let mut sorted_nodes: Vec<Arc<SceneGraphNode>> = processing_nodes_map
            .into_values()
            .map(Arc::new)
            .collect();

        // Z-Order Sorting: Higher z_order means it's rendered on top.
        sorted_nodes.sort_by(|a, b| a.z_order.cmp(&b.z_order));
        self.nodes = sorted_nodes;

        // Spatial indexing and occlusion culling would happen after this,
        // operating on the `self.nodes` or a copy.
        self.perform_spatial_indexing(output_geometry);
        self.perform_occlusion_culling(); // Already correctly placed, no change needed to the call itself

        println!("SceneGraph updated with {} nodes.", self.nodes.len());
    }

    // Placeholder for spatial indexing
    fn perform_spatial_indexing(&mut self, output_geometry: &Rectangle) {
        // Rebuild the spatial index if the output geometry has changed or if there are nodes.
        // If there are no nodes and the geometry hasn't changed, we can skip.
        if self.spatial_index.indexed_output_geometry != *output_geometry || !self.nodes.is_empty() {
            self.spatial_index.rebuild_corrected(&self.nodes, output_geometry);
            println!("Spatial index rebuilt for output {:?} with {} nodes.", output_geometry, self.nodes.len());
        } else if self.nodes.is_empty() && self.spatial_index.indexed_output_geometry == *output_geometry {
             // No nodes and output geometry is the same, spatial index is already empty and correct.
             // No action needed, but can log if desired.
             // println!("Spatial index unchanged: no nodes and same output geometry.");
        } else { // This case implies nodes.is_empty() but output_geometry *has* changed.
             // So, we need to update the index to reflect the new empty geometry.
            self.spatial_index.grid.clear(); // Clear out old data
            self.spatial_index.indexed_output_geometry = *output_geometry;
            self.spatial_index.grid_cols = (output_geometry.size.width / self.spatial_index.cell_size).ceil() as i32;
            self.spatial_index.grid_rows = (output_geometry.size.height / self.spatial_index.cell_size).ceil() as i32;
            println!("Spatial index reset for new empty output geometry {:?}", output_geometry);
        }
    }

    // Optimized occlusion culling using SpatialIndex
    fn perform_occlusion_culling(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        // Build a map for quick lookup of nodes by SurfaceId.
        // This map stores references to the nodes in their current state (before occlusion status update).
        let node_map: HashMap<SurfaceId, Arc<SceneGraphNode>> = self.nodes.iter().map(|node_arc| (node_arc.surface_id, node_arc.clone())).collect();

        let mut new_nodes_with_occlusion_info = Vec::with_capacity(self.nodes.len());

        // Iterate through nodes (sorted by Z from bottom to top)
        for current_node_arc in &self.nodes {
            let mut current_node_is_occluded = false;

            if current_node_arc.clipped_rect.is_empty_rect() {
                current_node_is_occluded = true;
            } else {
                // Query spatial_index for potential occluders
                let potential_occluder_ids = self.spatial_index.query(&current_node_arc.clipped_rect);

                for occluder_id in potential_occluder_ids {
                    if current_node_arc.surface_id == occluder_id { // Node cannot occlude itself
                        continue;
                    }

                    if let Some(potential_occluder_node_arc) = node_map.get(&occluder_id) {
                        // Occluder must have a higher Z-order
                        if potential_occluder_node_arc.z_order <= current_node_arc.z_order {
                            continue;
                        }

                        // Occluder's clipped_rect must not be empty
                        if potential_occluder_node_arc.clipped_rect.is_empty_rect() {
                            continue;
                        }

                        // Determine occluder's effective opaque area
                        let effective_opaque_area = potential_occluder_node_arc.attributes.opaque_region
                            .as_ref()
                            .map(|local_opaque_region| {
                                let world_opaque_region = potential_occluder_node_arc.final_transform.transform_rect_bounding_box(*local_opaque_region);
                                world_opaque_region.intersection(&potential_occluder_node_arc.clipped_rect)
                                                   .unwrap_or_else(|| Rectangle::from_coords(0.0, 0.0, 0.0, 0.0))
                            })
                            .filter(|r| !r.is_empty_rect()) // Ensure not empty after intersection
                            .unwrap_or(potential_occluder_node_arc.clipped_rect); // Fallback

                        if effective_opaque_area.is_empty_rect() {
                            continue;
                        }

                        // Check for full containment
                        if current_node_arc.clipped_rect.is_fully_contained_by(&effective_opaque_area) {
                            current_node_is_occluded = true;
                            break; // Found an occluder, no need to check others for this current_node
                        }
                    }
                }
            }

            let mut new_node_data = (**current_node_arc).clone();
            new_node_data.is_occluded = current_node_is_occluded;
            new_nodes_with_occlusion_info.push(Arc::new(new_node_data));
        }

        self.nodes = new_nodes_with_occlusion_info;
        println!("Occlusion culling (optimized) performed. {} nodes processed.", self.nodes.len());
    }

    pub fn get_renderable_nodes(&self) -> Vec<Arc<SceneGraphNode>> {
        self.nodes
            .iter()
            .filter(|node| !node.is_occluded)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::types::geometry::{Point, Size}; // For Point2D, Size2D aliases

    // Helper to create SurfaceAttributes for tests
    fn create_surface_attributes(x: f32, y: f32, w: f32, h: f32, z: i32, opaque: bool) -> SurfaceAttributes {
        SurfaceAttributes {
            position: Point2D::new(x, y),
            size: Size2D::new(w, h),
            transform: Transform::identity(),
            is_visible: true,
            z_order: z,
            opaque_region: if opaque { Some(Rectangle::from_coords(0.0, 0.0, w, h)) } else { None },
            parent: None,
        }
    }

    // Helper to create SurfaceAttributes with a specific local opaque region for tests
    fn create_surface_attributes_with_opaque_region(
        x: f32, y: f32, w: f32, h: f32, z: i32,
        opaque_rect: Option<Rectangle>
    ) -> SurfaceAttributes {
        SurfaceAttributes {
            position: Point2D::new(x, y),
            size: Size2D::new(w, h),
            transform: Transform::identity(),
            is_visible: true,
            z_order: z,
            opaque_region: opaque_rect,
            parent: None,
        }
    }


    // Helper to create a dummy SceneGraphNode for testing SpatialIndex (can be deprecated if SurfaceAttributes helpers are enough)
    fn create_test_node(id: u64, x: f32, y: f32, w: f32, h: f32, z: i32) -> Arc<SceneGraphNode> {
        Arc::new(SceneGraphNode {
            surface_id: SurfaceId::new(id),
            attributes: SurfaceAttributes {
                position: Point2D::new(x,y),
                size: Size2D::new(w,h),
                transform: Transform::identity(),
                is_visible: true,
                z_order: z,
                opaque_region: Some(Rectangle::from_coords(0.0,0.0,w,h)),
                parent: None,
            },
            clipped_rect: Rectangle::from_coords(x, y, w, h),
            final_transform: Transform {
                matrix: [[1.0, 0.0, x], [0.0, 1.0, y]]
            },
            children: Vec::new(),
            is_occluded: false, // Default to not occluded for test setup
        })
    }

    #[test]
    fn test_spatial_index_rebuild_and_query_basic() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 1024.0, 768.0);
        let cell_size = GRID_CELL_SIZE; // Use the constant from parent module
        let mut index = SpatialIndex::new(&output_geom, cell_size);

        let node1 = create_test_node(1, 10.0, 10.0, 100.0, 100.0, 1);  // Cell (0,0)
        let node2 = create_test_node(2, 300.0, 300.0, 50.0, 50.0, 2); // Cell (1,1) if cell_size is 256

        let nodes = vec![node1.clone(), node2.clone()];
        index.rebuild_corrected(&nodes, &output_geom);

        // Query for node1
        let query1 = Rectangle::from_coords(5.0, 5.0, 50.0, 50.0);
        let result1 = index.query(&query1);
        assert_eq!(result1.len(), 1);
        assert!(result1.contains(&node1.surface_id));

        // Query for node2
        let query2 = Rectangle::from_coords(290.0, 290.0, 70.0, 70.0);
        let result2 = index.query(&query2);
        assert_eq!(result2.len(), 1);
        assert!(result2.contains(&node2.surface_id));

        // Query overlapping both (if cell_size allows for distinct cells)
        // This query spans from node1's area to node2's area
        let query_both = Rectangle::from_coords(5.0, 5.0, 350.0, 350.0);
        let result_both = index.query(&query_both);
        assert!(result_both.contains(&node1.surface_id));
        assert!(result_both.contains(&node2.surface_id));
        assert_eq!(result_both.len(), 2);


        // Query in an empty area
        let query_empty = Rectangle::from_coords(600.0, 600.0, 100.0, 100.0);
        let result_empty = index.query(&query_empty);
        assert!(result_empty.is_empty());
    }

    #[test]
    fn test_spatial_index_node_spanning_multiple_cells() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 1024.0, 768.0);
        let cell_size = 100.0; // Smaller cell size for easier multi-cell span
        let mut index = SpatialIndex::new(&output_geom, cell_size);

        // This node spans (0,0), (1,0), (0,1), (1,1) if its origin is (50,50) and size (100,100)
        let node_span = create_test_node(1, 50.0, 50.0, 100.0, 100.0, 1);
        let nodes = vec![node_span.clone()];
        index.rebuild_corrected(&nodes, &output_geom);

        // Check that the node is found by querying any of the cells it overlaps
        // Cell (0,0) area for this node: query (50,50) to (99,99)
        let query_cell_00 = Rectangle::from_coords(50.0, 50.0, 10.0, 10.0);
        let result_00 = index.query(&query_cell_00);
        assert!(result_00.contains(&node_span.surface_id));

        // Cell (1,0) area for this node: query (101,50) to (150,99)
        let query_cell_10 = Rectangle::from_coords(110.0, 60.0, 10.0, 10.0);
        let result_10 = index.query(&query_cell_10);
        assert!(result_10.contains(&node_span.surface_id));

        // Cell (0,1) area
        let query_cell_01 = Rectangle::from_coords(60.0, 110.0, 10.0, 10.0);
        let result_01 = index.query(&query_cell_01);
        assert!(result_01.contains(&node_span.surface_id));

        // Cell (1,1) area
        let query_cell_11 = Rectangle::from_coords(110.0, 110.0, 10.0, 10.0);
        let result_11 = index.query(&query_cell_11);
        assert!(result_11.contains(&node_span.surface_id));

        // Query that covers the whole node
        let query_all_node = Rectangle::from_coords(50.0, 50.0, 100.0, 100.0);
        let result_all = index.query(&query_all_node);
        assert_eq!(result_all.len(), 1);
        assert!(result_all.contains(&node_span.surface_id));
    }

    #[test]
    fn test_spatial_index_rebuild_with_new_output_geometry() {
        let output_geom1 = Rectangle::from_coords(0.0, 0.0, 512.0, 512.0);
        let cell_size = 256.0;
        let mut index = SpatialIndex::new(&output_geom1, cell_size); // grid 2x2

        let node1 = create_test_node(1, 10.0, 10.0, 100.0, 100.0, 1); // Cell (0,0) in output_geom1
        index.rebuild_corrected(&[node1.clone()], &output_geom1);
        assert_eq!(index.grid_cols, 2);
        assert_eq!(index.grid_rows, 2);
        assert!(index.query(&Rectangle::from_coords(0.0,0.0,50.0,50.0)).contains(&node1.surface_id));

        // New, larger output geometry
        let output_geom2 = Rectangle::from_coords(0.0, 0.0, 1024.0, 1024.0); // grid 4x4
        // Node1 is still at (10,10) world coords, so still in cell (0,0) of the new larger grid
        index.rebuild_corrected(&[node1.clone()], &output_geom2);
        assert_eq!(index.grid_cols, 4);
        assert_eq!(index.grid_rows, 4);

        let results = index.query(&Rectangle::from_coords(0.0,0.0,50.0,50.0));
        assert!(results.contains(&node1.surface_id));
        assert_eq!(results.len(), 1);

        // Test with a node that would be in a different cell index due to changed output origin
        let output_geom3 = Rectangle::from_coords(-512.0, -512.0, 512.0, 512.0); // grid 2x2, origin shifted
        // Node1 is at (10,10). Relative to output_geom3, it's at (10 - (-512), 10 - (-512)) = (522, 522)
        // This would put it in cell (522/256, 522/256) = (2,2), which is outside this 2x2 grid defined from 0..width.
        // The current indexing logic maps to cells [0..cols-1, 0..rows-1].
        // So a node at (10,10) with an output origin of (-512, -512) should be correctly placed.
        // relative_x = 10 - (-512) = 522. min_c = floor(522/256) = 2.
        // However, the grid cells are indexed 0..grid_cols-1.
        // The rebuild_corrected logic ensures that only nodes within the output_geometry are added.
        // If node1's clipped_rect is (10,10,100,100), it's outside output_geom3 which is (-512,-512) to (0,0).
        // So it should not be in the index.

        // For this test, let's use a node that IS within output_geom3.
        // E.g. node at world (-10,-10) size (50,50).
        let node_in_g3 = create_test_node(2, -10.0, -10.0, 50.0, 50.0, 1);
        // relative_x = -10 - (-512) = 502. min_c = floor(502/256) = 1.
        // relative_y = -10 - (-512) = 502. min_r = floor(502/256) = 1.
        // So it should be in cell (1,1) of output_geom3's 2x2 grid.
        index.rebuild_corrected(&[node_in_g3.clone()], &output_geom3);
        assert_eq!(index.grid_cols, 2); // (512/256)
        assert_eq!(index.grid_rows, 2); // (512/256)

        let results_g3 = index.query(&Rectangle::from_coords(-50.0, -50.0, 100.0, 100.0)); // Query containing (-10,-10)
        assert!(results_g3.contains(&node_in_g3.surface_id), "Node in output_geom3 not found");
        assert_eq!(results_g3.len(), 1);

        // Ensure node1 (which is outside output_geom3) is not found
        let results_g3_for_node1 = index.query(&Rectangle::from_coords(0.0,0.0,50.0,50.0));
        assert!(!results_g3_for_node1.contains(&node1.surface_id), "Node1 should not be in output_geom3 index");

    }

    #[test]
    fn test_spatial_index_empty_nodes() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 1024.0, 768.0);
        let cell_size = 256.0;
        let mut index = SpatialIndex::new(&output_geom, cell_size);

        let nodes: Vec<Arc<SceneGraphNode>> = Vec::new();
        index.rebuild_corrected(&nodes, &output_geom);

        let query = Rectangle::from_coords(0.0, 0.0, 100.0, 100.0);
        let result = index.query(&query);
        assert!(result.is_empty());
        assert_eq!(index.grid.len(), 0); // Grid itself should be empty
    }

    #[test]
    fn test_spatial_index_query_empty_rect() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 1024.0, 768.0);
        let cell_size = 256.0;
        let mut index = SpatialIndex::new(&output_geom, cell_size);

        let node1 = create_test_node(1, 10.0, 10.0, 100.0, 100.0, 1);
        index.rebuild_corrected(&[node1], &output_geom);

        let query = Rectangle::from_coords(0.0, 0.0, 0.0, 0.0); // Empty query rect
        let result = index.query(&query);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_renderable_nodes_filters_occluded() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 800.0, 600.0);
        let mut sg = SceneGraph::new();
        let mut surface_data_map = HashMap::new();

        let surf1_id = SurfaceId::new(1); // Will be occluded
        let surf2_id = SurfaceId::new(2); // Occluder
        let surf3_id = SurfaceId::new(3); // Visible

        surface_data_map.insert(surf1_id, create_surface_attributes(10.0, 10.0, 50.0, 50.0, 1, true));
        surface_data_map.insert(surf2_id, create_surface_attributes(10.0, 10.0, 50.0, 50.0, 2, true)); // Occludes 1
        surface_data_map.insert(surf3_id, create_surface_attributes(100.0, 100.0, 50.0, 50.0, 3, true));

        sg.update(&surface_data_map, &output_geom);

        let renderable_nodes = sg.get_renderable_nodes();

        assert_eq!(renderable_nodes.len(), 2, "Should be 2 renderable nodes");
        assert!(renderable_nodes.iter().any(|n| n.surface_id == surf2_id), "Surf2 should be renderable");
        assert!(renderable_nodes.iter().any(|n| n.surface_id == surf3_id), "Surf3 should be renderable");
        assert!(!renderable_nodes.iter().any(|n| n.surface_id == surf1_id), "Surf1 should be occluded and not renderable");

        // Check Z-order preservation
        assert_eq!(renderable_nodes[0].surface_id, surf2_id);
        assert_eq!(renderable_nodes[1].surface_id, surf3_id);
    }

    #[test]
    fn test_occlusion_culling_simple_full_occlusion() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 800.0, 600.0);
        let mut sg = SceneGraph::new();
        let mut surface_data_map = HashMap::new();

        let surf1_id = SurfaceId::new(1); // Occluded node
        let surf2_id = SurfaceId::new(2); // Occluder node

        surface_data_map.insert(surf1_id, create_surface_attributes(10.0, 10.0, 50.0, 50.0, 1, true));
        surface_data_map.insert(surf2_id, create_surface_attributes(10.0, 10.0, 50.0, 50.0, 2, true));

        sg.update(&surface_data_map, &output_geom);
        let renderable = sg.get_renderable_nodes();
        assert_eq!(renderable.len(), 1);
        assert_eq!(renderable[0].surface_id, surf2_id);

        let node1_internal = sg.nodes.iter().find(|n| n.surface_id == surf1_id).unwrap();
        assert!(node1_internal.is_occluded);
    }

    #[test]
    fn test_occlusion_culling_no_occlusion_separate_positions() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 800.0, 600.0);
        let mut sg = SceneGraph::new();
        let mut surface_data_map = HashMap::new();

        let surf1_id = SurfaceId::new(1);
        let surf2_id = SurfaceId::new(2);
        surface_data_map.insert(surf1_id, create_surface_attributes(10.0, 10.0, 50.0, 50.0, 1, true));
        surface_data_map.insert(surf2_id, create_surface_attributes(70.0, 70.0, 50.0, 50.0, 2, true));

        sg.update(&surface_data_map, &output_geom);
        let renderable = sg.get_renderable_nodes();
        assert_eq!(renderable.len(), 2);
        assert!(renderable.iter().any(|n| n.surface_id == surf1_id));
        assert!(renderable.iter().any(|n| n.surface_id == surf2_id));
    }

    #[test]
    fn test_occlusion_culling_partial_overlap_no_full_occlusion() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 800.0, 600.0);
        let mut sg = SceneGraph::new();
        let mut surface_data_map = HashMap::new();

        let surf1_id = SurfaceId::new(1);
        let surf2_id = SurfaceId::new(2);
        surface_data_map.insert(surf1_id, create_surface_attributes(10.0, 10.0, 50.0, 50.0, 1, true));
        surface_data_map.insert(surf2_id, create_surface_attributes(30.0, 30.0, 50.0, 50.0, 2, true));

        sg.update(&surface_data_map, &output_geom);
        let renderable = sg.get_renderable_nodes();
        assert_eq!(renderable.len(), 2, "Both nodes should be renderable due to partial overlap");
    }

    #[test]
    fn test_occlusion_culling_specific_opaque_region_causes_occlusion() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 100.0, 100.0);
        let mut sg = SceneGraph::new();
        let mut surface_data_map = HashMap::new();

        let surf_occluded_id = SurfaceId::new(1);
        let surf_occluder_id = SurfaceId::new(2);

        // Occluded: full 50x50 at (10,10)
        surface_data_map.insert(surf_occluded_id, create_surface_attributes(10.0, 10.0, 50.0, 50.0, 1, true));
        // Occluder: full 50x50 at (10,10), but its opaque region is also 50x50 at its local (0,0)
        // which matches its full size and thus fully covers the occluded surface.
        surface_data_map.insert(surf_occluder_id, create_surface_attributes_with_opaque_region(
            10.0, 10.0, 50.0, 50.0, 2,
            Some(Rectangle::from_coords(0.0,0.0,50.0,50.0))
        ));

        sg.update(&surface_data_map, &output_geom);
        let renderable = sg.get_renderable_nodes();
        assert_eq!(renderable.len(), 1);
        assert_eq!(renderable[0].surface_id, surf_occluder_id);
    }

    #[test]
    fn test_occlusion_culling_specific_opaque_region_prevents_occlusion() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 100.0, 100.0);
        let mut sg = SceneGraph::new();
        let mut surface_data_map = HashMap::new();

        let surf_target_id = SurfaceId::new(1); // Target that might be occluded
        let surf_occluder_id = SurfaceId::new(2); // Occluder with small opaque region

        // Target surface: 50x50 at (10,10)
        surface_data_map.insert(surf_target_id, create_surface_attributes(10.0, 10.0, 50.0, 50.0, 1, true));

        // Occluder surface: Also 50x50 at (10,10) (same position and size as target)
        // BUT, its opaque region is only the top-left 10x10 of its area.
        // Local opaque region: Rectangle::from_coords(0.0, 0.0, 10.0, 10.0)
        // This means only a small part of the occluder is opaque.
        surface_data_map.insert(surf_occluder_id, create_surface_attributes_with_opaque_region(
            10.0, 10.0, 50.0, 50.0, 2,
            Some(Rectangle::from_coords(0.0, 0.0, 10.0, 10.0))
        ));

        sg.update(&surface_data_map, &output_geom);
        let renderable = sg.get_renderable_nodes();

        // The target surface (50x50) should NOT be occluded because the occluder's
        // small opaque region (10x10 at world 10,10) does not fully cover it.
        assert_eq!(renderable.len(), 2, "Both nodes should be renderable");
        assert!(renderable.iter().any(|n| n.surface_id == surf_target_id), "Target node should be renderable");
        assert!(renderable.iter().any(|n| n.surface_id == surf_occluder_id), "Occluder node should be renderable");
    }

    #[test]
    fn test_occlusion_by_opaque_region_none_fallback() { // Renamed from previous similar test for clarity
        let output_geom = Rectangle::from_coords(0.0, 0.0, 800.0, 600.0);
        let mut sg = SceneGraph::new();
        let mut surface_data_map = HashMap::new();

        let surf1_id = SurfaceId::new(1);
        let surf2_id = SurfaceId::new(2);

        surface_data_map.insert(surf1_id, create_surface_attributes(10.0, 10.0, 50.0, 50.0, 1, true));
        surface_data_map.insert(surf2_id, create_surface_attributes_with_opaque_region( // Using helper for consistency
            10.0, 10.0, 50.0, 50.0, 2,
            None // Opaque region is None, should fallback to clipped_rect
        ));

        sg.update(&surface_data_map, &output_geom);
        let renderable = sg.get_renderable_nodes();
        assert_eq!(renderable.len(), 1);
        assert_eq!(renderable[0].surface_id, surf2_id);
    }

    #[test]
    fn test_occlusion_culling_many_non_overlapping_surfaces() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 1000.0, 1000.0);
        let mut sg = SceneGraph::new();
        let mut surface_data_map = HashMap::new();
        let count = 100; // 100 non-overlapping surfaces

        for i in 0..count {
            let id = SurfaceId::new(i as u64);
            // Position them in a grid; 10x10 grid for 100 surfaces
            let x = (i % 10) as f32 * 60.0;
            let y = (i / 10) as f32 * 60.0;
            surface_data_map.insert(id, create_surface_attributes(x, y, 50.0, 50.0, i as i32, true));
        }

        sg.update(&surface_data_map, &output_geom);
        let renderable = sg.get_renderable_nodes();
        assert_eq!(renderable.len(), count, "All non-overlapping surfaces should be renderable");
    }

    #[test]
    fn test_occlusion_culling_scattered_occluding_pair() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 2000.0, 2000.0); // Large area
        let mut sg = SceneGraph::new();
        let mut surface_data_map = HashMap::new();

        // A few scattered non-occluding surfaces
        surface_data_map.insert(SurfaceId::new(1), create_surface_attributes(10.0, 10.0, 50.0, 50.0, 1, true));
        surface_data_map.insert(SurfaceId::new(2), create_surface_attributes(1800.0, 10.0, 50.0, 50.0, 2, true));
        surface_data_map.insert(SurfaceId::new(3), create_surface_attributes(10.0, 1800.0, 50.0, 50.0, 3, true));

        // The occluding pair in the middle
        let occluded_id = SurfaceId::new(100);
        let occluder_id = SurfaceId::new(101);
        surface_data_map.insert(occluded_id, create_surface_attributes(1000.0, 1000.0, 50.0, 50.0, 4, true));
        surface_data_map.insert(occluder_id, create_surface_attributes(1000.0, 1000.0, 50.0, 50.0, 5, true)); // Higher Z

        surface_data_map.insert(SurfaceId::new(4), create_surface_attributes(1800.0, 1800.0, 50.0, 50.0, 6, true));

        sg.update(&surface_data_map, &output_geom);
        let renderable = sg.get_renderable_nodes();

        assert_eq!(renderable.len(), 5, "Expected 5 renderable nodes (3 scattered, occluder, one other scattered)");
        assert!(!renderable.iter().any(|n| n.surface_id == occluded_id), "Occluded node should not be renderable");
        assert!(renderable.iter().any(|n| n.surface_id == occluder_id), "Occluder node should be renderable");
    }

    #[test]
    fn test_occlusion_culling_dense_cluster() {
        let output_geom = Rectangle::from_coords(0.0, 0.0, 200.0, 200.0);
        let mut sg = SceneGraph::new();
        let mut surface_data_map = HashMap::new();

        // Bottom layer, fully covered
        surface_data_map.insert(SurfaceId::new(1), create_surface_attributes(10.0, 10.0, 100.0, 100.0, 1, true));

        // Middle layer, two halves covering the bottom one
        surface_data_map.insert(SurfaceId::new(2), create_surface_attributes(10.0, 10.0, 50.0, 100.0, 2, true));
        surface_data_map.insert(SurfaceId::new(3), create_surface_attributes(60.0, 10.0, 50.0, 100.0, 2, true));

        // Top layer, small one on one of the halves
        surface_data_map.insert(SurfaceId::new(4), create_surface_attributes(10.0, 10.0, 20.0, 20.0, 3, true));
        // Top layer, another small one on the other half, but higher Z than its direct occluder
        surface_data_map.insert(SurfaceId::new(5), create_surface_attributes(60.0, 10.0, 20.0, 20.0, 3, true));


        sg.update(&surface_data_map, &output_geom);
        let renderable = sg.get_renderable_nodes();

        // Expected renderable: 4 and 5.
        // Node 1 is occluded by 2 and 3.
        // Node 2 is occluded by 4 (partially, but for this test let's assume full for simplicity of setup if 4 is on top and same spot)
        // Actually, the current logic is "fully_contained_by". Let's adjust.
        // If node 4 (20x20) is on node 2 (50x100), node 2 is NOT fully occluded.
        // If node 2 and 3 are at z=2, and node 1 is at z=1. Node 1 is occluded.
        // If node 4 (z=3) is on node 2 (z=2), node 2 is not occluded by 4 unless 4 covers it.
        // If node 5 (z=3) is on node 3 (z=2), node 3 is not occluded by 5 unless 5 covers it.

        // Let's redefine for clarity:
        // Surf 1 (ID 1, z=1): 10,10,100x100 <- Will be occluded by 2 and 3 together.
        // Surf 2 (ID 2, z=2): 10,10,50x100  <- Will be visible
        // Surf 3 (ID 3, z=2): 60,10,50x100  <- Will be visible
        // Surf 4 (ID 4, z=3): 10,10,20x20 (on top of Surf 2) <- Will be visible
        // Surf 5 (ID 5, z=3): 60,10,20x20 (on top of Surf 3) <- Will be visible
        // The current algorithm checks one-by-one occlusion. It does not combine occluders.
        // So Surf 1 will NOT be occluded by Surf 2 alone, nor by Surf 3 alone.
        // This means Surf 1, 2, 3, 4, 5 will all be renderable.

        // To test dense occlusion where one IS occluded by ONE other:
        surface_data_map.clear();
        // Bottom: Big one (10,10, 100x100), z=1 (surf_A)
        let surf_a_id = SurfaceId::new(10);
        surface_data_map.insert(surf_a_id, create_surface_attributes(10.0, 10.0, 100.0, 100.0, 1, true));
        // Middle: Smaller one on top of A (20,20, 50x50), z=2 (surf_B)
        let surf_b_id = SurfaceId::new(20);
        surface_data_map.insert(surf_b_id, create_surface_attributes(20.0, 20.0, 50.0, 50.0, 2, true));
        // Top: Exact same size and position as A, but higher Z, z=3 (surf_C)
        let surf_c_id = SurfaceId::new(30);
        surface_data_map.insert(surf_c_id, create_surface_attributes(10.0, 10.0, 100.0, 100.0, 3, true));

        sg.update(&surface_data_map, &output_geom);
        let renderable = sg.get_renderable_nodes();

        // Expected: B and C are renderable. A is occluded by C.
        assert_eq!(renderable.len(), 2, "Dense cluster: Expected 2 renderable nodes");
        assert!(renderable.iter().any(|n| n.surface_id == surf_b_id), "Surf B should be renderable");
        assert!(renderable.iter().any(|n| n.surface_id == surf_c_id), "Surf C should be renderable");
        assert!(!renderable.iter().any(|n| n.surface_id == surf_a_id), "Surf A should be occluded by C");

        let node_a_internal = sg.nodes.iter().find(|n| n.surface_id == surf_a_id).unwrap();
        assert!(node_a_internal.is_occluded);
    }

}
