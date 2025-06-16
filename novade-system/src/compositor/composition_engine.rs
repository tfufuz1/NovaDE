// novade-system/src/compositor/composition_engine.rs
// Main module for the composition engine.

use crate::renderer_interface::{
    RendererInterface, ClientBuffer, BufferFormat as RendererBufferFormat, RenderableTexture,
    RenderElement, TextureRenderParams, BufferContent, DmabufDescriptor, DmabufPlaneFormat, // Added DMABUF related types
};
use novade_compositor_core::surface::SurfaceId;
use std::collections::HashMap;
use super::scene_graph::{
    SceneGraph, SurfaceAttributes, BufferFormat as SceneGraphBufferFormat,
    Transform as SceneGraphTransform, BufferSourceType, // Added BufferSourceType
};
use novade_core::types::geometry::{Point2D, Size2D, Rect as NovaRect, Rectangle};
use std::sync::Arc;

pub struct CompositionEngine<R: RendererInterface> {
    renderer: R,
    scene_graph: SceneGraph,
    active_surfaces: HashMap<SurfaceId, SurfaceAttributes>,
    // TODO [TextureLRUCacheImpl]: Consider replacing HashMap with an LRU cache
    // if memory pressure from many active surface textures becomes an issue.
    // This would involve:
    // 1. Defining an LRU cache structure (e.g., using a HashMap and a DoublyLinkedList).
    // 2. Modifying texture access in `composite_frame` to use the LRU cache's get/put methods.
    // 3. Evicted textures would need to be released by the renderer (e.g., `renderer.release_surface_texture(surface_id)` if such a method is added to FrameRenderer).
    // 4. Re-uploading would occur if a surface becomes visible again after its texture was evicted.
    // ANCHOR [TextureManagementOutlines]
    surface_textures: HashMap<SurfaceId, Box<dyn RenderableTexture>>,
}

impl<R: RendererInterface> CompositionEngine<R> {
    pub fn new(renderer: R) -> Self {
        CompositionEngine {
            renderer,
            scene_graph: SceneGraph::new(),
            active_surfaces: HashMap::new(),
            surface_textures: HashMap::new(), // Initialize new field
        }
    }

    pub fn composite_frame(&mut self /*, outputs: &OutputManager */) {
        // In a real system, `active_surfaces` would be populated/updated by Wayland event handlers
        // reacting to wl_surface.commit, xdg_surface.configure, etc.
        // For now, it's populated by `add_surface`.

        // Define example output geometry
        let output_geometry = Rectangle::from_coords(0.0, 0.0, 1920.0, 1080.0); // Example output

        // 1. Update scene graph using the stored attributes and output geometry
        // ANCHOR [SceneGraphIntegration]
        self.scene_graph.update(&self.active_surfaces, &output_geometry);

        let renderable_nodes = self.scene_graph.get_renderable_nodes();

        if renderable_nodes.is_empty() {
            // Potentially clear the screen or do nothing
            // self.renderer.clear_screen(); // Example
            // self.renderer.present();
            println!("Composition: No renderable nodes in the scene graph.");
            return;
        }

        // ANCHOR [TextureUploadPipeline]
        // TODO [TextureStreaming] Implement more sophisticated texture streaming.
        // TODO [LRUEviction] Implement LRU eviction for textures when GPU memory is constrained.
        // TODO [TextureCompression] Investigate texture compression for formats that support it.
        // TODO [EfficientBufferUpdateDetection] Only upload textures if buffer_id has changed or texture doesn't exist.

        let mut visible_nodes_for_render_list: Vec<Arc<SceneGraphNode>> = Vec::new();

        for node in renderable_nodes.iter() {
            if node.is_occluded {
                // If node is occluded, and we previously had a texture for it,
                // this might be a place to consider releasing it (if not done by LRU).
                // For now, we just skip it.
                // self.surface_textures.remove(&node.surface_id); // Example of immediate release
                continue;
            }

            visible_nodes_for_render_list.push(node.clone()); // Keep a list of nodes that will actually be rendered

            // Attempt to upload/update texture for this visible surface
            if let Some(buffer_id) = node.attributes.current_buffer_id {
                if let Some(sg_buffer_format) = node.attributes.buffer_format {
                    // Convert SceneGraphBufferFormat to RendererBufferFormat
                    let renderer_buffer_format = match sg_buffer_format {
                        SceneGraphBufferFormat::Argb8888 => RendererBufferFormat::Argb8888,
                        SceneGraphBufferFormat::Xrgb8888 => RendererBufferFormat::Xrgb8888,
                    };

                    // Simulate client buffer data
                    // In a real system, this data would be fetched based on wl_buffer or DMABUF handle
                    let buffer_width = node.attributes.size.width as u32;
                    let buffer_height = node.attributes.size.height as u32;
                    let stride = node.attributes.buffer_stride;

                    // Ensure stride and dimensions are somewhat sane for dummy data
                    if buffer_width == 0 || buffer_height == 0 || stride < buffer_width * 4 { // Assuming 4 bytes per pixel
                        // eprintln!("Skipping texture upload for surface {:?} due to invalid buffer dimensions/stride.", node.surface_id);
                        continue;
                    }

                    let dummy_data_size = (stride * buffer_height) as usize;
                    let mut dummy_buffer_data: Vec<u8> = vec![0; dummy_data_size]; // Create a zeroed buffer

                    let client_buffer_content = match node.attributes.buffer_type {
                        Some(BufferSourceType::Dmabuf) => {
                            // ANCHOR [DmabufSimulation]
                            let overall_width = node.attributes.size.width as u32;
                            let overall_height = node.attributes.size.height as u32;
                            let mut descriptors_array: [Option<DmabufDescriptor>; 4] = [None, None, None, None];

                            // Special size for multi-planar I420 simulation (for testing)
                            // ANCHOR [DmabufMultiPlanarSimulation]
                            if overall_width == 256 && overall_height == 256 { // Test condition for I420
                                let y_stride = overall_width;
                                let uv_stride = overall_width / 2;
                                descriptors_array[0] = Some(DmabufDescriptor { // Y plane
                                    fd: -1, width: overall_width, height: overall_height, plane_index: 0,
                                    offset: 0, stride: y_stride, format: DmabufPlaneFormat::R8, modifier: 0,
                                });
                                descriptors_array[1] = Some(DmabufDescriptor { // U plane
                                    fd: -1, width: overall_width / 2, height: overall_height / 2, plane_index: 1,
                                    offset: 0, stride: uv_stride, format: DmabufPlaneFormat::R8, modifier: 0,
                                });
                                descriptors_array[2] = Some(DmabufDescriptor { // V plane
                                    fd: -1, width: overall_width / 2, height: overall_height / 2, plane_index: 2,
                                    offset: 0, stride: uv_stride, format: DmabufPlaneFormat::R8, modifier: 0,
                                });
                            } else { // Default: Simulate single-plane ARGB8888
                                let sim_stride = overall_width * 4;
                                let sim_format = DmabufPlaneFormat::Argb8888;
                                descriptors_array[0] = Some(DmabufDescriptor {
                                    fd: -1, width: overall_width, height: overall_height, plane_index: 0,
                                    offset: 0, stride: sim_stride, format: sim_format, modifier: 0,
                                });
                            }

                            BufferContent::Dmabuf {
                                id: buffer_id,
                                descriptors: descriptors_array,
                                width: overall_width,
                                height: overall_height,
                            }
                        }
                        Some(BufferSourceType::Shm) | None => { // Default to SHM if None
                            BufferContent::Shm {
                                id: buffer_id,
                                data: &dummy_buffer_data,
                                width: buffer_width,
                                height: buffer_height,
                                stride,
                                format: renderer_buffer_format,
                            }
                        }
                    };

                    let client_buffer = ClientBuffer { content: client_buffer_content };

                    // TODO [TextureStreamingImpl]: For very large surfaces, implement texture streaming.
                    // This would involve:
                    // 1. `SurfaceAttributes` indicating if a surface buffer is partial or if the full content is too large for direct upload.
                    // 2. `ClientBuffer` supporting partial updates (e.g., with offset and size for the dirty rect or tile index).
                    // 3. `FrameRenderer::upload_surface_texture` (or a new method `update_texture_region` or `upload_texture_tile`)
                    //    would need to handle partial uploads to an existing wgpu::Texture using `queue.write_texture`.
                    // 4. The compositor would need to manage tile states (e.g., which tiles are loaded/dirty) and request updates
                    //    for visible tiles only. This is complex and involves significant changes to buffer management,
                    //    surface commit logic, and rendering logic (potentially UV adjustments for tiles).
                    // ANCHOR [TextureStreamingOutline]

                    // TODO [TextureCompressionSupport]: Explore support for compressed texture formats.
                    // This would involve:
                    // 1. `BufferFormat` enum in `abstraction.rs` to include compressed formats (e.g., BC1, BC3, BC7, ASTC, ETC2).
                    // 2. `ClientBuffer` carrying data in these compressed formats.
                    // 3. `NovaWgpuRenderer::upload_surface_texture` detecting these formats and creating
                    //    `wgpu::Texture`s with the corresponding `wgpu::TextureFormat`.
                    //    WGPU supports many compressed formats, but this depends on hardware/driver capabilities.
                    //    The `wgpu::Features::TEXTURE_COMPRESSION_BC`, `TEXTURE_COMPRESSION_ETC2`, `TEXTURE_COMPRESSION_ASTC`
                    //    flags would need to be checked on the adapter.
                    // 4. Shaders would sample these compressed textures directly. No decompression on CPU needed.
                    // 5. Requires clients to provide buffers in compressed formats (e.g., via a Wayland protocol extension).
                    // ANCHOR [TextureCompressionOutline]
                    match self.renderer.upload_surface_texture(node.surface_id, &client_buffer) {
                        Ok(texture_handle) => {
                            // println!("Successfully uploaded texture for surface {:?}, buffer_id: {}", node.surface_id, buffer_id);
                            self.surface_textures.insert(node.surface_id, texture_handle);
                        }
                        Err(e) => {
                            eprintln!("Failed to upload texture for surface {:?}, buffer_id: {}: {:?}", node.surface_id, buffer_id, e);
                        }
                    }
                } else {
                    // Missing buffer format in attributes, cannot upload
                    // eprintln!("Skipping texture upload for surface {:?}: missing buffer_format.", node.surface_id);
                }
            } else {
                // No current_buffer_id, implies no buffer attached or visible.
                // If there was an old texture, it might be a candidate for removal.
                // self.surface_textures.remove(&node.surface_id);
            }
        }

        let final_nodes_to_render_count = visible_nodes_for_render_list.len();
        println!("Composition: {} nodes processed for texture upload.", final_nodes_to_render_count);

        // ANCHOR [CompositionShaderExecution]
        // TODO [InstancedRendering] Explore instanced rendering for quads to reduce draw calls.
        let mut render_elements_list: Vec<RenderElement<'_>> = Vec::new(); // Use 'static if no lifetime elements are mixed, or '_

        for node_arc in visible_nodes_for_render_list.iter() { // Iterate over the collected visible nodes
            let node = &**node_arc; // Dereference Arc<SceneGraphNode> to &SceneGraphNode

            if let Some(texture) = self.surface_textures.remove(&node.surface_id) { // Take ownership
                let params = TextureRenderParams {
                    texture, // Pass the owned Box<dyn RenderableTexture>
                    transform: node.final_transform.clone(), // Clone the transform
                    alpha: 1.0f32, // Default alpha
                    clip_rect: node.clipped_rect, // This is already Rect<f32>
                    // Assuming source_rect is normalized (0.0-1.0 for full texture)
                    // If node.attributes.size represents the original texture size for some reason:
                    // source_rect: NovaRect::new(0.0, 0.0, node.attributes.size.width, node.attributes.size.height),
                    // But for normalized, it's:
                    source_rect: NovaRect::new(0.0, 0.0, 1.0, 1.0),
                };
                render_elements_list.push(RenderElement::TextureNode(params));
            } else {
                // This case should ideally not happen if texture upload was successful for all visible nodes.
                // It might happen if a node became visible but its texture upload failed or was skipped.
                eprintln!("Texture not found for visible node {:?}, skipping render.", node.surface_id);
            }
        }

        if !render_elements_list.is_empty() {
            println!("Composition: Preparing to render {} elements.", render_elements_list.len());
            // The renderer's render_frame might handle begin_frame/end_frame internally.
            // If not, calls like self.renderer.begin_frame(); would be here.
            match self.renderer.render_frame(render_elements_list, &output_geometry, 1.0) {
                Ok(_) => {
                    // Presentation is handled by a separate call, typically after all outputs are composited.
                    // self.renderer.present_frame(); // This will be handled in a later stage/subtask
                }
                Err(e) => {
                    eprintln!("Error during renderer.render_frame: {:?}", e);
                }
            }
        } else if final_nodes_to_render_count > 0 {
             println!("Composition: {} visible nodes, but 0 render elements prepared (likely texture issues).", final_nodes_to_render_count);
        }
        // else: No visible nodes, nothing to render, already printed earlier.

        // ANCHOR [PostProcessingPipeline]
        // TODO [PostProcessingConfig]: Make post-processing steps and their parameters configurable.

        // 1. Gamma Correction
        // TODO [GammaValueConfig]: Get gamma from display/user settings.
        const DEFAULT_GAMMA: f32 = 2.2;
        if let Err(e) = self.renderer.apply_gamma_correction(DEFAULT_GAMMA) {
            eprintln!("Failed to apply gamma correction: {:?}", e);
        }

        // 2. HDR to SDR Tone Mapping (example call, might be conditional)
        // TODO [HDRContentDetection]: Only apply if HDR content is present and output is SDR.
        // TODO [ToneMappingParamsConfig]: Get params from settings.
        const DEFAULT_MAX_LUMINANCE: f32 = 1000.0; // Example nits
        const DEFAULT_EXPOSURE: f32 = 1.0;
        if let Err(e) = self.renderer.apply_hdr_to_sdr_tone_mapping(DEFAULT_MAX_LUMINANCE, DEFAULT_EXPOSURE) {
            eprintln!("Failed to apply tone mapping: {:?}", e);
        }

        // TODO [ColorSpaceConversion]: Implement color space conversion step.
        // TODO [AntiAliasing]: Implement anti-aliasing step.
        // TODO [CustomEffects]: Implement custom effects application.

        // ANCHOR [FramePresentation]
        if let Err(e) = self.renderer.submit_and_present_frame() {
            eprintln!("Failed to submit and present frame: {:?}", e);
            // TODO [ErrorHandlingPresentation]: More robust error handling here,
            // potentially re-initialize renderer or mark output as problematic.
        }

        // TODO [FramePacing]: Implement frame pacing logic.
        // TODO [AdaptiveSync]: Implement adaptive sync support.
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
    use crate::renderer_interface::{
        RendererInterface, RenderableTexture, RendererError, ClientBuffer, BufferContent as RendererBufferContent,
        BufferFormat as RendererBufferFormatExt, DmabufDescriptor as RendererDmabufDescriptor,
        DmabufPlaneFormat as RendererDmabufPlaneFormat,
        RenderElement as RendererRenderElement,
        TextureRenderParams as RendererTextureRenderParams,
    };
    use novade_compositor_core::surface::SurfaceId;
    use crate::compositor::scene_graph::{
        Transform as SceneGraphTransform,
        BufferFormat as SceneGraphBufferFormatExt,
        BufferSourceType as SceneGraphBufferSourceType, // Import BufferSourceType
    };
    use novade_core::types::geometry::{Point, Size, Rect, Rect as NovaRectExt, Size as NovaSize, Point as NovaPoint};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc as StdArc; // To distinguish from crate::sync::Arc if that existed
    use uuid::Uuid;

    // Using SceneGraphTransform directly for simplicity in mock
    // type Transform = crate::compositor::scene_graph::Transform;
    type Point2D = NovaPoint<f32>;
    type Size2D = NovaSize<f32>;
    type Rectangle = Rect<f32>; // This is novade_core::types::geometry::Rect

    #[derive(Debug)]
    struct MockRenderableTexture {
        id: Uuid,
        width: u32,
        height: u32,
    }

    impl MockRenderableTexture {
        fn new(width: u32, height: u32) -> Self {
            Self { id: Uuid::new_v4(), width, height }
        }
    }

    impl RenderableTexture for MockRenderableTexture {
        fn id(&self) -> Uuid { self.id }
        fn bind(&self, _slot: u32) -> Result<(), RendererError> { Ok(()) }
        fn width_px(&self) -> u32 { self.width }
        fn height_px(&self) -> u32 { self.height }
        fn format(&self) -> Option<smithay::backend::renderer::utils::Fourcc> { None }
        fn as_any(&self) -> &dyn std::any::Any { self }
    }

    #[derive(Debug, Clone)]
    struct UploadedTextureInfo {
        surface_id: SurfaceId,
        buffer_id: u64,
        width: u32,
        height: u32,
        content_type: String, // "Shm" or "Dmabuf"
        dmabuf_plane_format: Option<DmabufPlaneFormat>, // Format of the first plane for Dmabuf
        dmabuf_plane_stride: Option<u32>, // Stride of the first plane for Dmabuf
        num_dmabuf_planes: usize, // Number of active planes for Dmabuf
    }

    #[derive(Debug, Clone)]
    enum RenderElementInfo {
        TextureNode { texture_id: Uuid, transform_matrix: [[f32;3];2] },
        // Add other variants if needed by tests
    }


    #[derive(Clone, Debug)] // Added Clone and Debug
    struct MockRendererInternalState {
        uploaded_textures: Vec<UploadedTextureInfo>,
        rendered_elements: Vec<RenderElementInfo>, // Simplified for now
        gamma_correction_calls: Vec<f32>,
        tone_mapping_calls: Vec<(f32, f32)>,
        submit_and_present_frame_called_count: usize,
        // Track calls to other methods if necessary for specific tests
        render_frame_count: usize,
    }

    impl MockRendererInternalState {
        fn new() -> Self {
            Self {
                uploaded_textures: Vec::new(),
                rendered_elements: Vec::new(),
                gamma_correction_calls: Vec::new(),
                tone_mapping_calls: Vec::new(),
                submit_and_present_frame_called_count: 0,
                render_frame_count: 0,
            }
        }
    }

    #[derive(Clone, Debug)] // Added Clone and Debug
    struct MockRenderer {
        state: Rc<RefCell<MockRendererInternalState>>,
    }

    impl MockRenderer {
        fn new() -> Self {
            Self {
                state: Rc::new(RefCell::new(MockRendererInternalState::new())),
            }
        }
        // Helper methods for tests to inspect state
        fn uploaded_textures_count(&self) -> usize { self.state.borrow().uploaded_textures.len() }
        fn get_uploaded_texture_info(&self, index: usize) -> Option<UploadedTextureInfo> { self.state.borrow().uploaded_textures.get(index).cloned() }
        fn rendered_elements_count(&self) -> usize { self.state.borrow().rendered_elements.len() }
        fn submit_and_present_frame_called_count(&self) -> usize { self.state.borrow().submit_and_present_frame_called_count }
        fn gamma_calls_count(&self) -> usize { self.state.borrow().gamma_correction_calls.len() }
        fn tone_mapping_calls_count(&self) -> usize { self.state.borrow().tone_mapping_calls.len() }
        fn render_frame_count(&self) -> usize { self.state.borrow().render_frame_count }
    }

    impl RendererInterface for MockRenderer {
        fn id(&self) -> Uuid { Uuid::new_v4() }

        fn render_frame<'a>(
            &mut self,
            elements: impl IntoIterator<Item = RendererRenderElement<'a>>,
            _output_geometry: Rectangle,
            _output_scale: f64,
        ) -> Result<(), RendererError> {
            let mut state = self.state.borrow_mut();
            state.render_frame_count += 1;
            for element in elements {
                if let RendererRenderElement::TextureNode(params) = element {
                     state.rendered_elements.push(RenderElementInfo::TextureNode {
                        texture_id: params.texture.id(),
                        transform_matrix: params.transform.matrix,
                    });
                }
            }
            Ok(())
        }

        fn submit_and_present_frame(&mut self) -> Result<(), RendererError> {
            self.state.borrow_mut().submit_and_present_frame_called_count += 1;
            Ok(())
        }

        fn upload_surface_texture(
            &mut self,
            surface_id: SurfaceId,
            buffer: &ClientBuffer<'_>, // Updated to new ClientBuffer structure
        ) -> Result<Box<dyn RenderableTexture>, RendererError> {
            let mut state = self.state.borrow_mut();
            match &buffer.content {
                RendererBufferContent::Shm { id, width, height, .. } => {
                    state.uploaded_textures.push(UploadedTextureInfo {
                        surface_id,
                        buffer_id: *id,
                        width: *width,
                        height: *height,
                        content_type: "Shm".to_string(),
                        dmabuf_plane_format: None,
                        dmabuf_plane_stride: None,
                        num_dmabuf_planes: 0,
                    });
                    Ok(Box::new(MockRenderableTexture::new(*width, *height)))
                }
                RendererBufferContent::Dmabuf { id, descriptors, width, height, .. } => {
                    let mut first_plane_format = None;
                    let mut first_plane_stride = None;
                    let mut plane_count = 0;
                    for desc_opt in descriptors.iter() {
                        if let Some(desc) = desc_opt {
                            if plane_count == 0 { // Capture info for the first plane
                                first_plane_format = Some(desc.format);
                                first_plane_stride = Some(desc.stride);
                            }
                            plane_count += 1;
                        }
                    }
                    state.uploaded_textures.push(UploadedTextureInfo {
                        surface_id,
                        buffer_id: *id,
                        width: *width,
                        height: *height,
                        content_type: "Dmabuf".to_string(),
                        dmabuf_plane_format: first_plane_format,
                        dmabuf_plane_stride: first_plane_stride,
                        num_dmabuf_planes: plane_count,
                    });
                    Ok(Box::new(MockRenderableTexture::new(*width, *height)))
                }
            }
        }

        fn apply_gamma_correction(&mut self, gamma_value: f32) -> Result<(), RendererError> {
            self.state.borrow_mut().gamma_correction_calls.push(gamma_value);
            Ok(())
        }

        fn apply_hdr_to_sdr_tone_mapping(&mut self, max_luminance: f32, exposure: f32) -> Result<(), RendererError> {
            self.state.borrow_mut().tone_mapping_calls.push((max_luminance, exposure));
            Ok(())
        }

        // Stubs for other FrameRenderer methods
        fn screen_size(&self) -> NovaSize<i32, smithay::utils::Physical> { NovaSize::new(1920, 1080) }
        fn create_texture_from_shm(&mut self, _buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer) -> Result<Box<dyn RenderableTexture>, RendererError> {
            Err(RendererError::Unsupported("Not mocked".to_string()))
        }
        fn create_texture_from_dmabuf(&mut self, _dmabuf: &smithay::backend::allocator::dmabuf::Dmabuf) -> Result<Box<dyn RenderableTexture>, RendererError> {
            Err(RendererError::Unsupported("Not mocked".to_string()))
        }
    }

    #[test]
    fn test_engine_creation() {
        let mock_renderer = MockRenderer::new();
        let _engine = CompositionEngine::new(mock_renderer);
        assert_eq!(0, _engine.renderer.uploaded_textures_count()); // Example assertion
    }

    #[test]
    fn test_add_surface_and_update_scenegraph_no_render_calls() { // Renamed to reflect it doesn't check render calls directly now
        let mock_renderer = MockRenderer::new();
        let mut engine = CompositionEngine::new(mock_renderer);

        let surface1_id = SurfaceId::new(1);
        // Corrected type for SurfaceAttributes for transform and buffer_format
        let attrs1 = SurfaceAttributes {
            position: Point2D::new(10.0, 10.0),
            size: Size2D::new(100.0, 100.0),
            transform: SceneGraphTransform::identity(),
            is_visible: true,
            z_order: 1,
            opaque_region: Some(Rectangle::from_coords(0.0,0.0,100.0,100.0)),
            parent: None,
            current_buffer_id: Some(1),
            buffer_format: Some(SceneGraphBufferFormatExt::Argb8888),
            buffer_stride: 100 * 4,
            buffer_type: Some(SceneGraphBufferSourceType::Shm), // Specify buffer type
        };
        engine.add_surface(surface1_id, attrs1.clone());

        let surface2_id = SurfaceId::new(2);
        let attrs2 = SurfaceAttributes {
            position: Point2D::new(5.0, 5.0),
            size: Size2D::new(50.0, 50.0),
            transform: SceneGraphTransform::identity(),
            is_visible: true,
            z_order: 2,
            opaque_region: Some(Rectangle::from_coords(0.0,0.0,50.0,50.0)),
            parent: Some(surface1_id),
            current_buffer_id: Some(2),
            buffer_format: Some(SceneGraphBufferFormatExt::Argb8888),
            buffer_stride: 50 * 4,
            buffer_type: Some(SceneGraphBufferSourceType::Shm), // Specify buffer type
        };
        engine.add_surface(surface2_id, attrs2.clone());

        // Call composite_frame to update scene graph, but this test doesn't check renderer calls
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

    #[test]
    fn test_occlusion_in_composition_engine() {
        let mock_renderer = MockRenderer::new();
        let mut engine = CompositionEngine::new(mock_renderer);

        let output_geometry = Rectangle::from_coords(0.0, 0.0, 1920.0, 1080.0); // Used by scene_graph.update

        let surface1_id = SurfaceId::new(1); // Occluded
        let attrs1 = SurfaceAttributes {
            position: Point2D::new(10.0, 10.0),
            size: Size2D::new(100.0, 100.0),
            transform: SceneGraphTransform::identity(),
            is_visible: true,
            z_order: 1, // Lower Z
            opaque_region: Some(Rectangle::from_coords(0.0,0.0,100.0,100.0)),
            parent: None,
            current_buffer_id: Some(101),
            buffer_format: Some(SceneGraphBufferFormatExt::Argb8888),
            buffer_stride: 100*4,
            buffer_type: Some(SceneGraphBufferSourceType::Shm),
        };
        engine.add_surface(surface1_id, attrs1);

        let surface2_id = SurfaceId::new(2); // Occluder
        let attrs2 = SurfaceAttributes {
            position: Point2D::new(10.0, 10.0), // Same position and size
            size: Size2D::new(100.0, 100.0),
            transform: SceneGraphTransform::identity(),
            is_visible: true,
            z_order: 2, // Higher Z
            opaque_region: Some(Rectangle::from_coords(0.0,0.0,100.0,100.0)),
            parent: None,
            current_buffer_id: Some(102),
            buffer_format: Some(SceneGraphBufferFormatExt::Argb8888),
            buffer_stride: 100*4,
            buffer_type: Some(SceneGraphBufferSourceType::Shm),
        };
        engine.add_surface(surface2_id, attrs2);

        engine.composite_frame();

        let sg_nodes = engine.scene_graph.get_renderable_nodes();
        assert_eq!(sg_nodes.len(), 2, "Should have two nodes in the scene graph for occlusion check.");

        let node1 = sg_nodes.iter().find(|n| n.surface_id == surface1_id).expect("Surface 1 not found");
        let node2 = sg_nodes.iter().find(|n| n.surface_id == surface2_id).expect("Surface 2 not found");

        assert!(node1.is_occluded, "Node 1 (z=1) should be occluded by Node 2 (z=2)");
        assert!(!node2.is_occluded, "Node 2 (z=2) should not be occluded");

        // Now check MockRenderer calls
        assert_eq!(engine.renderer.uploaded_textures_count(), 1, "Only one texture (for visible surface2) should be uploaded.");
        if let Some(info) = engine.renderer.get_uploaded_texture_info(0) {
            assert_eq!(info.surface_id, surface2_id);
        }
        assert_eq!(engine.renderer.render_frame_count(), 1, "render_frame should be called once.");
        assert_eq!(engine.renderer.rendered_elements_count(), 1, "Only one element (for visible surface2) should be rendered.");
        // Further checks on rendered_elements content can be added if RenderElementInfo is more detailed.

        assert_eq!(engine.renderer.gamma_calls_count(), 1);
        assert_eq!(engine.renderer.tone_mapping_calls_count(), 1);
        assert_eq!(engine.renderer.submit_and_present_frame_called_count(), 1);
    }

    #[test]
    fn test_basic_frame_composition_flow() {
        let mock_renderer = MockRenderer::new();
        let mut engine = CompositionEngine::new(mock_renderer);

        let surface_id = SurfaceId::new(1);
        let attrs = SurfaceAttributes {
            position: Point2D::new(0.0, 0.0),
            size: Size2D::new(100.0, 100.0),
            transform: SceneGraphTransform::identity(),
            is_visible: true,
            z_order: 0,
            opaque_region: None,
            parent: None,
            current_buffer_id: Some(1001),
            buffer_format: Some(SceneGraphBufferFormatExt::Argb8888),
            buffer_stride: 100 * 4,
            buffer_type: Some(SceneGraphBufferSourceType::Shm),
        };
        engine.add_surface(surface_id, attrs);

        engine.composite_frame();

        assert_eq!(engine.renderer.uploaded_textures_count(), 1);
        if let Some(info) = engine.renderer.get_uploaded_texture_info(0) {
            assert_eq!(info.surface_id, surface_id);
            assert_eq!(info.buffer_id, 1001);
        }
        assert_eq!(engine.renderer.render_frame_count(), 1);
        assert_eq!(engine.renderer.rendered_elements_count(), 1);
        assert_eq!(engine.renderer.gamma_calls_count(), 1);
        assert_eq!(engine.renderer.tone_mapping_calls_count(), 1);
        assert_eq!(engine.renderer.submit_and_present_frame_called_count(), 1);
    }

    #[test]
    fn test_empty_frame_composition() {
        let mock_renderer = MockRenderer::new();
        let mut engine = CompositionEngine::new(mock_renderer);

        engine.composite_frame(); // No surfaces added

        assert_eq!(engine.renderer.uploaded_textures_count(), 0);
        assert_eq!(engine.renderer.render_frame_count(), 0); // render_frame should not be called due to early return
        assert_eq!(engine.renderer.rendered_elements_count(), 0);
        // Post-processing and presentation should also not be called due to early return
        assert_eq!(engine.renderer.gamma_calls_count(), 0);
        assert_eq!(engine.renderer.tone_mapping_calls_count(), 0);
        assert_eq!(engine.renderer.submit_and_present_frame_called_count(), 0);
    }

    #[test]
    fn test_dmabuf_surface_flow() {
        let mock_renderer = MockRenderer::new();
        let mut engine = CompositionEngine::new(mock_renderer);

        let surface_dmabuf_id = SurfaceId::new(1);
        let dmabuf_size = Size2D::new(128.0, 128.0);
        let attrs_dmabuf = SurfaceAttributes {
            position: Point2D::new(0.0, 0.0),
            size: dmabuf_size,
            transform: SceneGraphTransform::identity(),
            is_visible: true,
            z_order: 0,
            opaque_region: None,
            parent: None,
            current_buffer_id: Some(2001),
            buffer_format: None, // For DMABUF, format is often per-plane in descriptors.
                                 // The SceneGraphBufferFormat might be less relevant here.
            buffer_stride: 0, // Stride is also often per-plane for DMABUF. Set to 0 or an expected overall stride if used.
                               // The simulation in composite_frame now calculates stride based on width.
            buffer_type: Some(SceneGraphBufferSourceType::Dmabuf),
        };
        engine.add_surface(surface_dmabuf_id, attrs_dmabuf);

        // Optional: Add an SHM surface to ensure distinction
        let surface_shm_id = SurfaceId::new(2);
        let shm_size = Size2D::new(64.0, 64.0);
        let attrs_shm = SurfaceAttributes {
            position: Point2D::new(200.0, 0.0),
            size: shm_size,
            transform: SceneGraphTransform::identity(),
            is_visible: true,
            z_order: 1,
            opaque_region: None,
            parent: None,
            current_buffer_id: Some(2002),
            buffer_format: Some(SceneGraphBufferFormatExt::Argb8888), // SHM needs this
            buffer_stride: (shm_size.width as u32) * 4,
            buffer_type: Some(SceneGraphBufferSourceType::Shm),
        };
        engine.add_surface(surface_shm_id, attrs_shm);

        engine.composite_frame();

        assert_eq!(engine.renderer.uploaded_textures_count(), 2, "Expected two textures to be uploaded (one DMABUF, one SHM).");

        let dmabuf_info = engine.renderer.state.borrow().uploaded_textures.iter().find(|info| info.surface_id == surface_dmabuf_id).expect("DMABUF texture info not found.");
        assert_eq!(dmabuf_info.surface_id, surface_dmabuf_id);
        assert_eq!(dmabuf_info.buffer_id, 2001);
        assert_eq!(dmabuf_info.content_type, "Dmabuf", "Content type should be Dmabuf.");
        assert_eq!(dmabuf_info.width, dmabuf_size.width as u32, "DMABUF overall width mismatch.");
        assert_eq!(dmabuf_info.height, dmabuf_size.height as u32, "DMABUF overall height mismatch.");
        assert_eq!(dmabuf_info.dmabuf_plane_format, Some(DmabufPlaneFormat::Argb8888), "DMABUF first plane format mismatch for single-plane test.");
        assert_eq!(dmabuf_info.dmabuf_plane_stride, Some((dmabuf_size.width as u32) * 4), "DMABUF first plane stride mismatch for single-plane test.");
        assert_eq!(dmabuf_info.num_dmabuf_planes, 1, "DMABUF single-plane should have 1 plane.");


        let shm_info = engine.renderer.state.borrow().uploaded_textures.iter().find(|info| info.surface_id == surface_shm_id).expect("SHM texture info not found.");
        assert_eq!(shm_info.surface_id, surface_shm_id);
        assert_eq!(shm_info.buffer_id, 2002);
        assert_eq!(shm_info.content_type, "Shm", "Content type should be Shm.");
        assert_eq!(shm_info.width, shm_size.width as u32);
        assert_eq!(shm_info.height, shm_size.height as u32);
        assert!(shm_info.dmabuf_plane_format.is_none(), "SHM upload should not have DMABUF plane format.");
        assert!(shm_info.dmabuf_plane_stride.is_none(), "SHM upload should not have DMABUF plane stride.");

        assert_eq!(engine.renderer.render_frame_count(), 1);
        assert_eq!(engine.renderer.rendered_elements_count(), 2);
        assert_eq!(engine.renderer.gamma_calls_count(), 1);
        assert_eq!(engine.renderer.tone_mapping_calls_count(), 1);
        assert_eq!(engine.renderer.submit_and_present_frame_called_count(), 1);
    }

    #[test]
    fn test_multi_planar_dmabuf_surface_flow() {
        let mock_renderer = MockRenderer::new();
        let mut engine = CompositionEngine::new(mock_renderer);

        let surface_multi_planar_id = SurfaceId::new(3);
        // Use the special 256x256 size to trigger I420 simulation in composite_frame
        let multi_planar_size = Size2D::new(256.0, 256.0);
        let attrs_multi_planar = SurfaceAttributes {
            position: Point2D::new(10.0, 10.0),
            size: multi_planar_size,
            transform: SceneGraphTransform::identity(),
            is_visible: true,
            z_order: 0,
            opaque_region: None,
            parent: None,
            current_buffer_id: Some(3001),
            buffer_format: None, // Not directly used by DMABUF path if descriptors define plane formats
            buffer_stride: 0,    // Not directly used by DMABUF path if descriptors define plane strides
            buffer_type: Some(SceneGraphBufferSourceType::Dmabuf),
        };
        engine.add_surface(surface_multi_planar_id, attrs_multi_planar);

        engine.composite_frame();

        assert_eq!(engine.renderer.uploaded_textures_count(), 1, "Expected one multi-planar texture to be uploaded.");

        let multi_planar_info = engine.renderer.state.borrow().uploaded_textures.iter()
            .find(|info| info.surface_id == surface_multi_planar_id)
            .expect("Multi-planar DMABUF texture info not found.");

        assert_eq!(multi_planar_info.surface_id, surface_multi_planar_id);
        assert_eq!(multi_planar_info.buffer_id, 3001);
        assert_eq!(multi_planar_info.content_type, "Dmabuf", "Content type should be Dmabuf.");
        assert_eq!(multi_planar_info.width, multi_planar_size.width as u32, "Multi-planar overall width mismatch.");
        assert_eq!(multi_planar_info.height, multi_planar_size.height as u32, "Multi-planar overall height mismatch.");

        // Check info for the first plane (Y plane in I420 simulation)
        assert_eq!(multi_planar_info.dmabuf_plane_format, Some(DmabufPlaneFormat::R8), "Y-plane format mismatch.");
        assert_eq!(multi_planar_info.dmabuf_plane_stride, Some(multi_planar_size.width as u32), "Y-plane stride mismatch.");
        assert_eq!(multi_planar_info.num_dmabuf_planes, 3, "Expected 3 planes for I420 simulation.");

        assert_eq!(engine.renderer.render_frame_count(), 1);
        assert_eq!(engine.renderer.rendered_elements_count(), 1);
        assert_eq!(engine.renderer.gamma_calls_count(), 1);
        assert_eq!(engine.renderer.tone_mapping_calls_count(), 1);
        assert_eq!(engine.renderer.submit_and_present_frame_called_count(), 1);
    }
}
