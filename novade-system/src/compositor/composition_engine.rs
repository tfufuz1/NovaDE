// novade-system/src/compositor/composition_engine.rs
// Main module for the composition engine.

use crate::renderer_interface::RendererInterface;
use novade_compositor_core::surface::SurfaceId;
use std::collections::HashMap;
use super::scene_graph::SceneGraph; // Make sure this is uncommented
use crate::compositor::scene_graph::SurfaceAttributes; // Assuming this path
use novade_core::types::geometry::{Point2D, Size2D, Transform, Rectangle}; // Ensure these are available

pub struct CompositionEngine<R: RendererInterface> {
    renderer: R,
    scene_graph: SceneGraph,
    active_surfaces: HashMap<SurfaceId, SurfaceAttributes>, // Changed type
}

impl<R: RendererInterface> CompositionEngine<R> {
    pub fn new(renderer: R) -> Self {
        CompositionEngine {
            renderer,
            scene_graph: SceneGraph::new(),
            active_surfaces: HashMap::new(),
        }
    }

    pub fn composite_frame(&mut self /*, outputs: &OutputManager */) {
        // In a real system, `active_surfaces` would be populated/updated by Wayland event handlers
        // reacting to wl_surface.commit, xdg_surface.configure, etc.
        // For now, it's populated by `add_surface`.

        // Define example output geometry
        let output_geometry = Rectangle::from_coords(0.0, 0.0, 1920.0, 1080.0); // Example output

        // 1. Update scene graph using the stored attributes and output geometry
        self.scene_graph.update(&self.active_surfaces, &output_geometry);

        let renderable_nodes = self.scene_graph.get_renderable_nodes();

        if renderable_nodes.is_empty() {
            // Potentially clear the screen or do nothing
            // self.renderer.clear_screen(); // Example
            // self.renderer.present();
            return;
        }
        // ... rest of the compositing logic
        println!("Compositing frame with {} renderable nodes.", renderable_nodes.len());
    }

    // Updated to accept attributes
    pub fn add_surface(&mut self, surface_id: SurfaceId, attributes: SurfaceAttributes) {
        self.active_surfaces.insert(surface_id, attributes);
        println!("Surface {:?} added to composition engine with attributes.", surface_id);
    }

    pub fn remove_surface(&mut self, surface_id: SurfaceId) {
        self.active_surfaces.remove(&surface_id);
        println!("Surface {:?} removed from composition engine.", surface_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer_interface::MockRenderer;
    use novade_compositor_core::surface::SurfaceId; // Make sure this is accessible
    // Use the locally defined Transform for tests too, if scene_graph::Transform is used by SurfaceAttributes
    use crate::compositor::scene_graph::Transform;
    use novade_core::types::geometry::{Point, Size, Rect}; // Using base types for clarity
    // Re-alias for test convenience if needed, or use Point::<f32> etc.
    type Point2D = Point<f32>;
    type Size2D = Size<f32>;
    type Rectangle = Rect<f32>;


    #[test]
    fn test_engine_creation() {
        let mock_renderer = MockRenderer::new();
        let _engine = CompositionEngine::new(mock_renderer);
        // Add assertions here
    }

    #[test]
    fn test_add_surface_and_update_scenegraph() {
        let mock_renderer = MockRenderer::new();
        let mut engine = CompositionEngine::new(mock_renderer);

        let surface1_id = SurfaceId::new(1);
        let attrs1 = SurfaceAttributes {
            position: Point2D::new(10.0, 10.0), // Position in output space
            size: Size2D::new(100.0, 100.0),
            transform: Transform::identity(), // No local scale/rotation
            is_visible: true,
            z_order: 1,
            opaque_region: Some(Rectangle::from_coords(0.0,0.0,100.0,100.0)),
            parent: None, // Top-level surface
        };
        engine.add_surface(surface1_id, attrs1.clone());

        let surface2_id = SurfaceId::new(2);
        let attrs2 = SurfaceAttributes {
            position: Point2D::new(5.0, 5.0), // Relative to parent (surface1)
            size: Size2D::new(50.0, 50.0),
            transform: Transform::identity(), // No local rotation/scale for simplicity
            is_visible: true,
            z_order: 2, // On top of surface1
            opaque_region: Some(Rectangle::from_coords(0.0,0.0,50.0,50.0)),
            parent: Some(surface1_id), // Child of surface1
        };
        engine.add_surface(surface2_id, attrs2.clone());

        engine.composite_frame();

        let nodes = engine.scene_graph.get_renderable_nodes();
        assert_eq!(nodes.len(), 2); // Both should be visible

        // Find node1 and node2 for checks - sort by Z to make sure order is predictable if needed
        // but direct find is also fine.
        let node1 = nodes.iter().find(|n| n.surface_id == surface1_id).expect("Surface 1 not found in scene graph");
        let node2 = nodes.iter().find(|n| n.surface_id == surface2_id).expect("Surface 2 not found in scene graph");

        // Check node1 (parent)
        // Its final_transform is effectively: Identity * Translate(10,10) * Identity
        assert_eq!(node1.final_transform.matrix[0][2], 10.0, "Node1 X translation");
        assert_eq!(node1.final_transform.matrix[1][2], 10.0, "Node1 Y translation");
        assert_eq!(node1.clipped_rect.origin.x, 10.0, "Node1 clipped x");
        assert_eq!(node1.clipped_rect.origin.y, 10.0, "Node1 clipped y");
        assert_eq!(node1.clipped_rect.size.width, 100.0, "Node1 clipped width");

        // Check node2 (child)
        // Its final_transform is: Node1_Final_Transform * Translate(5,5) * Identity
        // Parent's world translation is (10,10). Child's position relative to parent is (5,5).
        // So child's world translation part should be (10+5, 10+5) = (15,15).
        assert_eq!(node2.final_transform.matrix[0][2], 15.0, "Node2 X translation");
        assert_eq!(node2.final_transform.matrix[1][2], 15.0, "Node2 Y translation");
        assert_eq!(node2.clipped_rect.origin.x, 15.0, "Node2 clipped x");
        assert_eq!(node2.clipped_rect.origin.y, 15.0, "Node2 clipped y");
        assert_eq!(node2.clipped_rect.size.width, 50.0, "Node2 clipped width");
    }
}
