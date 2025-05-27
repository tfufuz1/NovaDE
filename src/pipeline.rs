use crate::error::{Result, VulkanError};
use crate::device::LogicalDevice;
use crate::allocator::{Allocator, Allocation}; // For depth buffer
use crate::render_pass::RenderPass; // To link pipeline with render pass
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use vulkanalia::prelude::v1_0::*;
use memoffset;
use glam::{Mat4, Vec2, Vec3}; // Added for UBO and potentially Vertex math

// Vertex Struct Definition (updated for texture coordinates)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3], // Keep color for now, can be removed if texture dictates all color
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0) // pos
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, pos) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1) // color
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, color) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2) // tex_coord
                .format(vk::Format::R32G32_SFLOAT) // vec2
                .offset(memoffset::offset_of!(Vertex, tex_coord) as u32)
                .build(),
        ]
    }
}

// UniformBufferObject Struct Definition
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

// Struct for Pipeline Layout (updated for descriptor set layout)
pub struct PipelineLayout {
    device: Arc<Device>,
    pub layout: vk::PipelineLayout, // Made public for FrameRenderer
    pub descriptor_set_layout: vk::DescriptorSetLayout, // Store the layout
}

impl PipelineLayout {
    pub fn new(logical_device_wrapper: &LogicalDevice) -> Result<Self> {
        let device = logical_device_wrapper.raw().clone();

        let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0) // Binding 0 for UBO
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX); // UBO used in vertex shader

        let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1) // Binding 1 for Sampler
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT); // Sampler used in fragment shader

        let bindings = &[ubo_binding, sampler_binding];
        let dsl_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings); // Renamed to avoid conflict
        
        let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&dsl_info, None) }
            .map_err(VulkanError::VkResult)?;

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));
        
        let layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }
            .map_err(VulkanError::VkResult)?;
        
        log::debug!("Pipeline layout created with descriptor set layout.");
        Ok(Self { device, layout, descriptor_set_layout })
    }

    pub fn raw(&self) -> vk::PipelineLayout {
        self.layout
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
        log::debug!("Pipeline layout and descriptor set layout destroyed.");
    }
}

// Struct for Graphics Pipeline
pub struct GraphicsPipeline {
    device: Arc<Device>,
    pipeline: vk::Pipeline,
    // Depth buffer resources managed here as per VulkanRendererArchitektur.md
    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    depth_image_allocation: Option<Allocation>, // Store VMA allocation
}

impl GraphicsPipeline {
    pub fn new(
        logical_device_wrapper: &LogicalDevice,
        allocator: &Allocator,
        swapchain_extent: vk::Extent2D,
        render_pass: &RenderPass, // The RenderPass object itself
        pipeline_layout: &PipelineLayout, // The PipelineLayout object
        depth_format: vk::Format, // Obtained from RenderPass or PhysicalDevice
        vert_shader_path: &Path,
        frag_shader_path: &Path,
        pipeline_cache: vk::PipelineCache, // Added pipeline_cache parameter
    ) -> Result<Self> {
        let device = logical_device_wrapper.raw().clone();

        // 1. Load Shaders
        let vert_shader_code = read_shader_file(vert_shader_path)?;
        let frag_shader_code = read_shader_file(frag_shader_path)?;

        let vert_shader_module = create_shader_module(&device, &vert_shader_code)?;
        let frag_shader_module = create_shader_module(&device, &frag_shader_code)?;

        let vert_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0"); // Entry point (null-terminated)

        let frag_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(b"main\0");

        let shader_stages = &[vert_stage_info, frag_stage_info];

        // 2. Vertex Input State (updated for Vertex struct)
        let binding_description = Vertex::get_binding_description();
        let attribute_descriptions = Vertex::get_attribute_descriptions();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
            .vertex_attribute_descriptions(&attribute_descriptions);

        // 3. Input Assembly State
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        // 4. Viewport and Scissor (dynamic)
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        // 5. Rasterization State
        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE) // Changed from CLOCKWISE in tutorial for consistency
            .depth_bias_enable(false);

        // 6. Multisample State (no MSAA for now)
        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::_1);

        // 7. Depth/Stencil State
        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL) // Use LESS_OR_EQUAL for skyboxes / Z=1 cases
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        // 8. Color Blend State (no blending for now)
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all()) // R | G | B | A
            .blend_enable(false);
        // For alpha blending:
        // .blend_enable(true)
        // .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        // .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        // .color_blend_op(vk::BlendOp::ADD)
        // .src_alpha_blend_factor(vk::BlendFactor::ONE)
        // .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        // .alpha_blend_op(vk::BlendOp::ADD);


        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        // 9. Dynamic States
        let dynamic_states = &[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(dynamic_states);

        // 10. Graphics Pipeline Creation
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisample_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout.raw())
            .render_pass(render_pass.raw())
            .subpass(0); // Index of the subpass where this pipeline will be used

        let pipeline = unsafe {
            device.create_graphics_pipelines(pipeline_cache, &[pipeline_info], None) // Use provided cache
        }
        .map_err(|(e, _)| VulkanError::VkResult(e))?
        .remove(0);

        // Destroy shader modules as they are no longer needed after pipeline creation
        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }
        log::info!("Basic graphics pipeline created.");

        // Create Depth Buffer Image & View
        let (depth_image, depth_image_view, depth_image_allocation) = 
            Self::create_depth_resources_internal(logical_device_wrapper, allocator, swapchain_extent, depth_format)?;
        
        log::info!("Depth buffer resources created.");

        Ok(Self {
            device,
            pipeline,
            depth_image,
            depth_image_view,
            depth_image_allocation: Some(depth_image_allocation),
        })
    }
    
    // Renamed to make it clear it's for internal use but callable by FrameRenderer
    // Made pub(crate) for FrameRenderer access
    pub(crate) fn create_depth_resources_internal( 
        logical_device_wrapper: &LogicalDevice,
        allocator: &Allocator,
        swapchain_extent: vk::Extent2D,
        depth_format: vk::Format,
    ) -> Result<(vk::Image, vk::ImageView, Allocation)> {
        let extent = vk::Extent3D::builder()
            .width(swapchain_extent.width)
            .height(swapchain_extent.height)
            .depth(1);

        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::_2D)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .format(depth_format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .samples(vk::SampleCountFlags::_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (depth_image, allocation) = allocator.create_image(
            &image_info, 
            vk_mem_rs::MemoryUsage::GpuOnly, // Depth buffer is GPU only
            None,
        )?;

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
            
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(depth_image)
            .view_type(vk::ImageViewType::_2D)
            .format(depth_format)
            .subresource_range(subresource_range);
            
        let depth_image_view = unsafe { logical_device_wrapper.raw().create_image_view(&view_info, None) }
            .map_err(VulkanError::VkResult)?;
            
        // Note: Layout transition for depth image from UNDEFINED to DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        // is typically handled by the initialLayout and finalLayout in the render pass attachment description.
        // If not, an explicit barrier would be needed here or before first use.

        Ok((depth_image, depth_image_view, allocation))
    }


    pub fn raw(&self) -> vk::Pipeline {
        self.pipeline
    }
    
    // Method to get the depth image allocation, e.g., for FrameRenderer to destroy it
    pub fn take_depth_allocation(&mut self) -> Option<Allocation> {
        self.depth_image_allocation.take()
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image_view(self.depth_image_view, None);
            // The VMA allocation for depth_image needs to be freed using the allocator
            // Assuming allocator outlives or is available for this.
            // For simplicity, if this GraphicsPipeline struct also owned the allocator,
            // it would be straightforward. Otherwise, the owner of allocator needs to handle it,
            // or we pass the allocator on drop, or we don't use VMA for this specific image.
            // For now, we assume the allocation is taken and will be freed by the Allocator
            // when the `depth_image` is destroyed by `destroy_image`.
            // This requires `destroy_image` to be called on `self.depth_image` with `self.depth_image_allocation`.
            // The current structure means FrameRenderer will own Allocator and Pipeline, so it can call
            // allocator.destroy_image(self.depth_image, self.depth_image_allocation.take().unwrap())
            
            // To make GraphicsPipeline self-contained for cleanup of its OWNED resources:
            // This is tricky because Allocator is borrowed in new().
            // A better approach is for FrameRenderer to own Allocator, GraphicsPipeline,
            // and then FrameRenderer's Drop calls allocator.destroy_image.
            // So, GraphicsPipeline will not destroy the depth_image's VMA allocation directly.
            // It will only destroy the image view. The image and its memory are VMA's concern.
            // The `allocator.destroy_image` call should be made by the owner of the allocator.
            // For this subtask, we only destroy the view and pipeline.
            // The FrameRenderer will have to manage depth_image destruction via allocator.

            self.device.destroy_pipeline(self.pipeline, None);
        }
        log::debug!("Graphics pipeline and its depth image view destroyed.");
        // The depth_image and its VMA allocation must be destroyed by the Allocator instance.
    }
}

// Helper function to read shader files
pub(crate) fn read_shader_file(path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(path).map_err(|e| {
        VulkanError::ShaderLoadingError(format!("Failed to open shader file {:?}: {}", path, e))
    })?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| {
        VulkanError::ShaderLoadingError(format!("Failed to read shader file {:?}: {}", path, e))
    })?;
    Ok(buffer)
}

// Helper function to create shader modules
pub(crate) fn create_shader_module(device: &Device, code: &[u8]) -> Result<vk::ShaderModule> {
    // Ensure code is aligned to u32, as Vulkan expects.
    // The read_to_end into Vec<u8> might not guarantee this if file size is not multiple of 4.
    // However, SPIR-V files are typically well-formed. A robust solution might involve
    // checking alignment or copying to an aligned buffer if necessary.
    // For now, directly casting, assuming SPIR-V compiler produces aligned output.
    let shader_module_info = vk::ShaderModuleCreateInfo::builder()
        .code_size(code.len())
        .code(code.as_ptr() as *const u32); // Cast u8 slice to u32 slice pointer

    unsafe { device.create_shader_module(&shader_module_info, None) }
        .map_err(|e| VulkanError::ShaderModuleCreationError(e.to_string()))
}

// --- Compute Pipeline ---
pub struct ComputePipeline {
    device: Arc<Device>,
    pub layout: vk::PipelineLayout,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline: vk::Pipeline,
}

impl ComputePipeline {
    pub fn new(
        logical_device_wrapper: &Arc<LogicalDevice>,
        comp_shader_path: &Path,
        pipeline_cache: vk::PipelineCache, // Added pipeline_cache parameter
    ) -> Result<Self> {
        let device_arc = logical_device_wrapper.raw().clone(); 
        let device_ref = device_arc.as_ref(); 

        // Descriptor Set Layout for Compute
        let input_image_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0) // Input texture (sampled)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE);

        let output_image_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1) // Output texture (storage image)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE);

        let bindings = &[input_image_binding.build(), output_image_binding.build()]; // build() bindings
        let dsl_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
        let descriptor_set_layout = unsafe { device_ref.create_descriptor_set_layout(&dsl_info, None) }
            .map_err(VulkanError::VkResult)?;

        // Pipeline Layout for Compute
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));
        let layout = unsafe { device_ref.create_pipeline_layout(&pipeline_layout_info, None) }
            .map_err(VulkanError::VkResult)?;

        // Load Compute Shader
        let comp_shader_code = read_shader_file(comp_shader_path)?;
        let comp_shader_module = create_shader_module(device_ref, &comp_shader_code)?;

        let stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(comp_shader_module)
            .name(b"main\0"); // Null-terminated string

        let pipeline_info = vk::ComputePipelineCreateInfo::builder()
            .stage(stage_info.build())
            .layout(layout);

        let pipeline = unsafe { device_ref.create_compute_pipelines(pipeline_cache, &[pipeline_info.build()], None) } // Use provided cache
            .map_err(|(e, _)| VulkanError::VkResult(e))?
            .remove(0);

        unsafe { device_ref.destroy_shader_module(comp_shader_module, None) };
        log::info!("Compute pipeline created from {:?}", comp_shader_path);

        Ok(Self { device: device_arc, layout, descriptor_set_layout, pipeline })
    }
}

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
        log::debug!("Compute pipeline, layout, and DSL destroyed.");
    }
}
