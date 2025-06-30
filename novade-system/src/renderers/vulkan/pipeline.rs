use ash::{vk, Device as AshDevice};
use std::sync::Arc;
use super::shader::VulkanShaderModule; // Assuming VulkanShaderModule is in super::shader

// ANCHOR: VulkanPipeline Struct Definition
pub struct VulkanPipeline {
    device: Arc<AshDevice>,
    pub pipeline_layout: vk::PipelineLayout, // Made public for binding descriptor sets
    // pub descriptor_set_layouts: Vec<vk::DescriptorSetLayout>, // Store if managed by VulkanPipeline
    pub pipeline: vk::Pipeline, // Made public for binding in command buffer
}

/// Creates a Vulkan Pipeline Layout.
///
/// The pipeline layout object defines the layout of resources (descriptor sets) and push constants
/// that can be accessed by a pipeline. It is created based on descriptor set layouts and
/// push constant ranges.
///
/// This function aligns with `Rendering Vulkan.md` (Spec 6.2 - Pipeline-Layout-Definition).
///
/// # Arguments
/// * `device`: A reference to the logical `ash::Device`.
/// * `set_layouts`: A slice of `vk::DescriptorSetLayout` handles. For a minimal pipeline
///   (like in WP-303 for a clear screen), this slice would be empty.
/// * `push_constant_ranges`: A slice of `vk::PushConstantRange`. For a minimal pipeline,
///   this slice would also be empty.
///
/// # Returns
/// A `Result` containing the created `vk::PipelineLayout` handle, or an error string
/// if creation fails.
///
/// # `Rendering Vulkan.md` Specification Mapping (Spec 6.2):
/// - `VkDescriptorSetLayoutBinding` / `VkDescriptorSetLayoutCreateInfo`: Assumed to be handled by
///   the caller when creating the `VkDescriptorSetLayout` objects passed in `set_layouts`.
/// - `VkPushConstantRange`: Passed directly via `push_constant_ranges`.
/// - `VkPipelineLayoutCreateInfo`: Configured with the provided `set_layouts` and `push_constant_ranges`.
pub fn create_vulkan_pipeline_layout(
    device: &AshDevice,
    set_layouts: &[vk::DescriptorSetLayout],
/// The pipeline layout defines the interface between shader stages and shader resources
/// (like descriptor sets and push constants).
/// Aligns with `Rendering Vulkan.md` (Spec 6.2).
///
/// # Arguments
/// * `device`: A reference to the logical `ash::Device`.
/// * `set_layouts`: A slice of `vk::DescriptorSetLayout` handles to be included in the pipeline layout.
///   For a minimal pipeline (WP-303), this would be empty.
/// * `push_constant_ranges`: A slice of `vk::PushConstantRange` defining push constant blocks.
///   For a minimal pipeline, this would be empty.
///
/// # Returns
/// A `Result` containing the created `vk::PipelineLayout` or an error string on failure.
pub fn create_vulkan_pipeline_layout(
    device: &AshDevice,
    set_layouts: &[vk::DescriptorSetLayout],
    push_constant_ranges: &[vk::PushConstantRange],
) -> Result<vk::PipelineLayout, String> {
    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts)
        .push_constant_ranges(push_constant_ranges);

    unsafe {
        device.create_pipeline_layout(&pipeline_layout_create_info, None)
    }
    .map_err(|e| format!("Failed to create pipeline layout: {}", e))
}


// ANCHOR: VulkanPipeline Implementation
impl VulkanPipeline {
    /// Creates a new `VulkanPipeline` for graphics operations, configured for minimal rendering.
    ///
    /// This constructor sets up a graphics pipeline tailored for the "first visible output"
    /// stage (WP-303). Key characteristics of this minimal pipeline include:
    /// - No vertex input attributes or bindings (vertices are generated in the shader or not used).
    /// - Expects simple vertex and fragment shaders (e.g., for a full-screen triangle or fixed color output).
    /// - Utilizes dynamic states for viewport and scissor, allowing them to be set at command buffer recording time.
    /// - Basic rasterization settings (e.g., fill mode, no culling).
    /// - No multisampling.
    /// - Depth and stencil tests are disabled.
    /// - Basic color blending (typically opaque).
    ///
    /// This method aligns with `Rendering Vulkan.md` (Spec 6.4 - Graphics-Pipeline-Erstellung).
    ///
    /// # Arguments
    /// * `device`: An `Arc<AshDevice>` reference to the logical device.
    /// * `render_pass`: The `vk::RenderPass` with which this pipeline must be compatible.
    /// * `_swapchain_extent`: The `vk::Extent2D` of the swapchain. While passed, it's not directly
    ///   used in pipeline creation if viewport and scissor are dynamic states (which they are here).
    ///   It's useful context for understanding the target dimensions.
    /// * `vert_shader_module`: A reference to the `VulkanShaderModule` for the vertex shader.
    /// * `frag_shader_module`: A reference to the `VulkanShaderModule` for the fragment shader.
    /// * `pipeline_layout`: The `vk::PipelineLayout` for this pipeline. For the minimal pipeline
    ///   of WP-303, this layout would typically be empty (created with no descriptor set layouts
    ///   and no push constant ranges via `create_vulkan_pipeline_layout`).
    ///
    /// # Returns
    /// A `Result` containing the new `VulkanPipeline` instance, or an error string if pipeline creation fails.
    ///
    /// # `Rendering Vulkan.md` Specification Mapping (Spec 6.4):
    /// - **6.4.1 (Shader Stages):** Configured with provided vertex and fragment shader modules.
    /// - **6.4.2 (Vertex Input):** Configured with provided binding and attribute descriptions.
    /// - **6.4.3 (Input Assembly):** `TRIANGLE_LIST` topology.
    /// - **6.4.4 (Viewport/Scissor):** Configured for dynamic state.
    /// - **6.4.5 (Rasterization):** Basic fill mode, culling typically BACK (can be NONE for simple cases).
    /// - **6.4.6 (Multisampling):** Disabled (`SAMPLE_COUNT_1_BIT`).
    /// - **6.4.7 (Depth/Stencil):** Configurable, typically enabled for 3D. For simple 2D, can be disabled.
    /// - **6.4.8, 6.4.9 (Color Blending):** Basic opaque blending.
    /// - **6.4.10 (Dynamic State):** Viewport and Scissor are dynamic.
    /// - **6.4.11 (Pipeline Create Info):** Assembles all above states with the provided `pipeline_layout` and `render_pass`.
    pub fn new_with_vertex_input( // Renamed from new_minimal
        device: Arc<AshDevice>,
        render_pass: vk::RenderPass,
        _swapchain_extent: vk::Extent2D,
        vert_shader_module: &VulkanShaderModule,
        frag_shader_module: &VulkanShaderModule,
        pipeline_layout: vk::PipelineLayout,
        binding_descriptions: &[vk::VertexInputBindingDescription],    // New parameter
        attribute_descriptions: &[vk::VertexInputAttributeDescription], // New parameter
    ) -> Result<Self, String> {

        // Shader Stages (Spec 6.4.1)
        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module.handle())
            .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").expect("Failed to create CStr for vertex shader entry point."));

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module.handle())
            .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").expect("Failed to create CStr for fragment shader entry point."));

        let shader_stages = [vert_shader_stage_info.build(), frag_shader_stage_info.build()];

        // Vertex Input State (Spec 6.4.2) - Now uses parameters
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(attribute_descriptions);

        // Input Assembly State (Spec 6.4.3)
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST) // Standard for drawing triangles
            .primitive_restart_enable(false);

        // Viewport State (Spec 6.4.4) - Using dynamic viewport and scissor
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1) // Will be set dynamically
            .scissor_count(1);  // Will be set dynamically

        // Rasterization State (Spec 6.4.5)
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false) // We want to rasterize
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE) // No culling for a simple clear/triangle
            .front_face(vk::FrontFace::CLOCKWISE); // Or COUNTER_CLOCKWISE, depends on triangle winding

        // Multisample State (Spec 6.4.6)
        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1); // No MSAA

        // Depth/Stencil State (Spec 6.4.7) - Disabled for simple 2D clear/triangle
        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        // Color Blend Attachment State (Spec 6.4.8)
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA) // Write to all color channels
            .blend_enable(false) // No blending for simple clear/triangle
            .build();

        // Color Blend State (Spec 6.4.9)
        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false) // No logical operations
            .attachments(std::slice::from_ref(&color_blend_attachment));

        // Dynamic States (Spec 6.4.10)
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);

        // Graphics Pipeline Create Info (Spec 6.4.11)
        let pipeline_info_builder = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout) // Use the pre-created layout
            .render_pass(render_pass)
            .subpass(0); // Index of the subpass

        let pipeline_create_info = pipeline_info_builder.build();

        let pipelines = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
        }.map_err(|(_pipelines, result)| { // create_graphics_pipelines returns Result<Vec<Pipeline>, (Vec<Pipeline>, Result)>
            // Note: The pipeline_layout is owned by the caller of new_minimal, so we don't destroy it here.
            format!("Failed to create graphics pipeline: {:?}", result)
        })?;

        if pipelines.is_empty() {
             // This case should ideally be caught by the map_err above, but as a safeguard:
            return Err("Graphics pipeline creation returned an empty vector, but no error code.".to_string());
        }
        let pipeline = pipelines[0]; // We expect one pipeline

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
