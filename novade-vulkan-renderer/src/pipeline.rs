use ash::{vk, Device};
use std::sync::Arc;
use std::ffi::CString; // For shader entry point name "main"
use crate::utils; // For load_shader_module

// Embed shaders directly into the binary
// These paths are relative to the root of the crate (where Cargo.toml is)
// if pipeline.rs is in src/, then shaders/ would be ../shaders/
// However, include_bytes! resolves paths relative to the current source file.
// So, if shaders/ is at novade-vulkan-renderer/shaders/,
// and pipeline.rs is at novade-vulkan-renderer/src/pipeline.rs,
// then the path should be "../shaders/..."
// For now, let's assume the runner creates these files in a way that these paths work,
// or adjust if compilation fails due to missing files.
// A common pattern is to put shaders in a 'shaders' dir at the crate root.
// If pipeline.rs is in src/, then include_bytes!("../../shaders/triangle.vert.spv")
// For now, using a path that would work if shaders dir is inside src.
// This will likely need adjustment by the subtask runner.
// For robustness, I'll use placeholder paths and note this clearly.
// **Action for Subtask Runner**: Ensure these paths are correct for your environment
// or place the .spv files at `novade-vulkan-renderer/src/shaders/`
const VERT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/triangle.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("../shaders/triangle.frag.spv");


pub struct GraphicsPipeline {
    device: Arc<Device>,
    pub layout: vk::PipelineLayout,
    pub handle: vk::Pipeline,
    // Shader modules are temporary and destroyed after pipeline creation
}

impl GraphicsPipeline {
    pub fn new(
        device: Arc<Device>,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
    ) -> Result<Self, anyhow::Error> {
        // 1. Load Shader Modules
        let vert_shader_module = utils::load_shader_module(&device, VERT_SHADER_BYTES)?;
        let frag_shader_module = utils::load_shader_module(&device, FRAG_SHADER_BYTES)?;

        let main_function_name = CString::new("main").unwrap(); // Shader entry point

        // 2. Define Shader Stages
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(&main_function_name)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(&main_function_name)
                .build(),
        ];

        // 3. Vertex Input State (no vertex inputs, hardcoded in shader)
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[])
            .vertex_attribute_descriptions(&[]);

        // 4. Input Assembly State
        let input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        // 5. Viewport and Scissor
        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain_extent)
            .build();

        let viewports = [viewport];
        let scissors = [scissor];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        // 6. Rasterization State
        let rasterization_state_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK) // Cull back faces
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE) // Standard for Vulkan
            .depth_bias_enable(false)
            .line_width(1.0);

        // 7. Multisample State (no multisampling)
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        // 8. Color Blend State (no blending, pass through)
        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false) // No blending for an opaque triangle
            .color_write_mask(vk::ColorComponentFlags::RGBA) // Write all components
            .build();

        let color_blend_attachments = [color_blend_attachment_state];
        let color_blend_state_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false) // No logic op
            .attachments(&color_blend_attachments);

        // 9. Depth/Stencil State (no depth/stencil testing)
        let depth_stencil_state_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL) // Irrelevant if test is disabled
            .stencil_test_enable(false);

        // 10. Dynamic States (Viewport and Scissor can be changed without rebuilding pipeline)
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);

        // 11. Pipeline Layout (no descriptor sets or push constants for this simple shader)
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder();
        let layout = unsafe {
            device.create_pipeline_layout(&pipeline_layout_create_info, None)?
        };

        // 12. Create Graphics Pipeline
        let graphics_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_state_info)
            .multisample_state(&multisample_state_info)
            .color_blend_state(&color_blend_state_info)
            .depth_stencil_state(&depth_stencil_state_info)
            .dynamic_state(&dynamic_state_info) // Enable dynamic states
            .layout(layout)
            .render_pass(render_pass)
            .subpass(0) // Index of the subpass where this pipeline will be used
            .build();

        // create_graphics_pipelines can create multiple pipelines and return results for each.
        // We are creating only one.
        let pipeline_results = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[graphics_pipeline_create_info], None)
        };

        // Destroy shader modules as they are no longer needed after pipeline creation
        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }

        match pipeline_results {
            Ok(pipelines) => {
                if pipelines.is_empty() {
                     // Should not happen if create_graphics_pipelines returns Ok without errors.
                    Err(anyhow::anyhow!("create_graphics_pipelines returned Ok but with an empty vector of pipelines."))
                } else {
                    Ok(Self {
                        device,
                        layout,
                        handle: pipelines[0], // Take the first (and only) pipeline
                    })
                }
            }
            Err((_pipelines, err_result)) => {
                // Even on error, Vulkan might return partially created pipelines that need to be destroyed.
                // However, ash's current error handling for create_graphics_pipelines returns a tuple
                // where the first element is a Vec of successfully created pipelines (if any),
                // and the second is the vk::Result error code.
                // If err_result is an error, we should destroy any pipelines that were created.
                // For simplicity here, we assume if an error vk::Result is returned, no pipelines are usable.
                // A more robust handler might iterate and destroy `_pipelines`.
                Err(anyhow::anyhow!("Failed to create graphics pipeline: {:?}", err_result))
            }
        }
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.handle, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
