use ash::{vk, Device as AshDevice};
use std::sync::Arc;
use super::shader::VulkanShaderModule; // Assuming VulkanShaderModule is in super::shader

// ANCHOR: VulkanPipeline Struct Definition
pub struct VulkanPipeline {
    device: Arc<AshDevice>,
    pipeline_layout: vk::PipelineLayout,
    // descriptor_set_layouts: Vec<vk::DescriptorSetLayout>, // For future use
    pipeline: vk::Pipeline,
}

// ANCHOR: VulkanPipeline Implementation
impl VulkanPipeline {
    pub fn new(
        device: Arc<AshDevice>,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
        vert_shader_module: &VulkanShaderModule,
        frag_shader_module: &VulkanShaderModule,
        texture_descriptor_set_layout: vk::DescriptorSetLayout, // Added
    ) -> Result<Self, String> {

        // ANCHOR_EXT: Shader Stages
        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module.handle())
            .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap()); // Entry point

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module.handle())
            .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap()); // Entry point

        let shader_stages = [vert_shader_stage_info.build(), frag_shader_stage_info.build()];

        // ANCHOR_EXT: Vertex Input State (for textured quad)
        // vec2 pos, vec2 texcoord
        const STRIDE: u32 = (2 + 2) * std::mem::size_of::<f32>() as u32;

        let binding_descriptions = [vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(STRIDE)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()];

        let attribute_descriptions = [
            // Position (vec2)
            vk::VertexInputAttributeDescription::builder()
                .location(0) // layout (location = 0) in shader
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0)
                .build(),
            // Texture Coordinate (vec2)
            vk::VertexInputAttributeDescription::builder()
                .location(1) // layout (location = 1) in shader
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset((2 * std::mem::size_of::<f32>()) as u32) // Offset after position
                .build(),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        // ANCHOR_EXT: Input Assembly State
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        // ANCHOR_EXT: Viewport and Scissor
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

        // ANCHOR_EXT: Viewport State
        // If not using dynamic viewport/scissor, this is how you set them:
        // let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        //     .viewports(std::slice::from_ref(&viewport))
        //     .scissors(std::slice::from_ref(&scissor));
        // However, we'll use dynamic states for viewport and scissor for more flexibility.
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1) // Dynamic state will provide actual viewport
            .scissor_count(1); // Dynamic state will provide actual scissor


        // ANCHOR_EXT: Rasterization State
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE) // Assuming standard winding order
            .depth_bias_enable(false);

        // ANCHOR_EXT: Multisample State
        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            // .min_sample_shading(1.0) // Optional
            // .sample_mask(&[]) // Optional
            // .alpha_to_coverage_enable(false) // Optional
            // .alpha_to_one_enable(false); // Optional

        // ANCHOR_EXT: Depth/Stencil State (Disabled for now)
        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL) // Common default
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);
            // .min_depth_bounds(0.0) // Optional
            // .max_depth_bounds(1.0); // Optional

        // ANCHOR_EXT: Color Blend Attachment State (Opaque for now)
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false) // No blending
            // Example for alpha blending (if blend_enable(true)):
            // .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            // .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            // .color_blend_op(vk::BlendOp::ADD)
            // .src_alpha_blend_factor(vk::BlendFactor::ONE)
            // .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            // .alpha_blend_op(vk::BlendOp::ADD)
            .build();

        // ANCHOR_EXT: Color Blend State
        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY) // Optional
            .attachments(std::slice::from_ref(&color_blend_attachment))
            .blend_constants([0.0, 0.0, 0.0, 0.0]); // Optional

        // ANCHOR_EXT: Pipeline Layout
        let set_layouts = [texture_descriptor_set_layout];
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&set_layouts) // Use the passed-in descriptor set layout
            .push_constant_ranges(&[]); // No push constants yet

        let pipeline_layout = unsafe {
            device.create_pipeline_layout(&pipeline_layout_create_info, None)
        }.map_err(|e| format!("Failed to create pipeline layout: {}", e))?;

        // ANCHOR_EXT: Dynamic States
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);

        // ANCHOR_EXT: Graphics Pipeline Create Info
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state) // Viewport and scissor set via dynamic states
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil_state) // Or None if completely disabled
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state_info) // Specify dynamic states
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0) // Index of the subpass where this pipeline will be used
            .base_pipeline_handle(vk::Pipeline::null()) // Optional: for pipeline derivatives
            .base_pipeline_index(-1); // Optional

        let pipelines = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&pipeline_info.build()), None)
        }.map_err(|e| {
            // Cleanup pipeline layout if pipeline creation fails
            unsafe { device.destroy_pipeline_layout(pipeline_layout, None); }
            format!("Failed to create graphics pipeline: {:?}", e) // Using {:?} for result type
        })?;

        let pipeline = pipelines[0]; // We only create one pipeline

        Ok(Self {
            device,
            pipeline_layout,
            pipeline,
        })
    }

    // ANCHOR: Accessors
    pub fn handle(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn layout_handle(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
}

// ANCHOR: VulkanPipeline Drop Implementation
impl Drop for VulkanPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            // Destroy descriptor set layouts here if they were created and stored
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
        // println!("VulkanPipeline (handle: {:?}, layout: {:?}) dropped.", self.pipeline, self.pipeline_layout);
    }
}
