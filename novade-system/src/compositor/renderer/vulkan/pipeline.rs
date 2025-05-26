use crate::compositor::renderer::vulkan::device::LogicalDevice;
use crate::compositor::renderer::vulkan::allocator::Allocator;
use crate::compositor::renderer::vulkan::physical_device::PhysicalDeviceInfo; // For find_supported_format via create_depth_resources
use crate::compositor::renderer::vulkan::instance::VulkanInstance; // For find_supported_format via create_depth_resources

use ash::vk;

// --- Uniform Buffer Object Definition ---
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UniformBufferObject {
    pub color_multiplier: [f32; 4],
    // pub model_matrix: [[f32; 4]; 4], // Example for a matrix
}
// --- End Uniform Buffer Object Definition ---

use log::{debug, info, error};
use std::{fs::File, io::Read, path::Path};
use vk_mem;

/// Loads SPIR-V code from a file.
///
/// # Arguments
/// * `path_str`: Path to the SPIR-V file.
///
/// # Returns
/// `Result<Vec<u8>, std::io::Error>` containing the SPIR-V bytecode or an IO error.
pub fn load_spirv_file(path_str: &str) -> Result<Vec<u8>, std::io::Error> {
    let path = Path::new(path_str);
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    // SPIR-V code is often stored as Vec<u32>, but ash expects &[u8] for shader module creation.
    // Ensure the byte buffer is suitable. If it was Vec<u32>, it might need alignment/casting.
    // However, reading directly to Vec<u8> is fine.
    // Vulkan expects the data to be aligned to 4 bytes (u32 alignment).
    // Reading into Vec<u8> should be fine as long as the file itself is correctly formatted.
    // The create_shader_module function will cast it to &[u32] internally if needed by ash,
    // but ash's create_shader_module takes &[u8] directly (or rather *const u32, with size in bytes).
    // Let's ensure our buffer is a multiple of 4 bytes, as SPIR-V is a stream of 32-bit words.
    if buffer.len() % 4 != 0 {
        // This would be an invalid SPIR-V file.
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "SPIR-V file size is not a multiple of 4.",
        ));
    }
    Ok(buffer)
}

/// Creates a Vulkan shader module from SPIR-V bytecode.
///
/// # Arguments
/// * `device`: Reference to the `ash::Device` (logical device).
/// * `spirv_code`: Slice of bytes representing the SPIR-V code.
///
/// # Returns
/// `Result<vk::ShaderModule, String>` containing the created shader module or an error message.
pub fn create_shader_module(
    device: &ash::Device,
    spirv_code: &[u8],
) -> Result<vk::ShaderModule, String> {
    debug!("Creating shader module from SPIR-V code ({} bytes)", spirv_code.len());
    // Ensure the code is suitable for casting to *const u32
    if spirv_code.as_ptr().align_offset(std::mem::align_of::<u32>()) != 0 {
        // This case should ideally be handled by loading into a Vec<u32> first if alignment is an issue.
        // However, Vec<u8> from file read should generally be sufficiently aligned or ash handles it.
        // The create_info takes a *const u32, so the slice of bytes needs to be reinterpreted.
        error!("SPIR-V code is not aligned to 4 bytes. This might lead to issues.");
    }
    if spirv_code.len() % 4 != 0 {
        return Err("SPIR-V code size is not a multiple of 4 bytes.".to_string());
    }

    let create_info = vk::ShaderModuleCreateInfo::builder()
        .code_size(spirv_code.len())
        // The `p_code` field expects a *const u32.
        // We must cast our &[u8] to &[u32]. This is safe if the input spirv_code
        // is properly aligned and its length is a multiple of 4.
        .code(unsafe {
            std::slice::from_raw_parts(
                spirv_code.as_ptr() as *const u32,
                spirv_code.len() / std::mem::size_of::<u32>(),
            )
        });

    unsafe { device.create_shader_module(&create_info, None) }
        .map_err(|e| format!("Failed to create shader module: {}", e))
}


/// Creates depth resources (image, memory, image view) for the depth buffer.
///
/// # Arguments
/// * `logical_device`: Reference to the `LogicalDevice`.
/// * `physical_device_info`: Information about the physical device, used for finding supported format.
/// * `allocator`: Reference to the VMA `Allocator`.
/// * `extent`: The dimensions for the depth image.
///
/// # Returns
/// A tuple containing the `vk::Image`, `vk_mem::Allocation`, `vk::ImageView`, and the chosen `vk::Format`.
pub fn create_depth_resources(
    logical_device: &LogicalDevice,
    physical_device_info: &PhysicalDeviceInfo, // Needed to call find_supported_format
    instance_handle: &ash::Instance, // Needed for find_supported_format
    allocator: &Allocator,
    extent: vk::Extent2D,
) -> Result<(vk::Image, vk_mem::Allocation, vk::ImageView, vk::Format), String> {
    info!("Creating depth resources with extent: {:?}", extent);

    let depth_format_candidates = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];
    let depth_format = crate::compositor::renderer::vulkan::physical_device::find_supported_format(
        instance_handle, // Pass the ash::Instance here
        physical_device_info.physical_device,
        &depth_format_candidates,
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
    .ok_or_else(|| "Failed to find a suitable depth format.".to_string())?;
    info!("Selected depth format: {:?}", depth_format);

    let image_create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(depth_format)
        .extent(vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let allocation_create_info = vk_mem::AllocationCreateInfo {
        usage: vk_mem::MemoryUsage::GpuOnly, // Depth buffer is typically GPU-only
        ..Default::default()
    };

    let (depth_image, depth_allocation, _allocation_info) = allocator
        .create_image(&image_create_info, &allocation_create_info)
        .map_err(|e| format!("Failed to create depth image with VMA: {}", e))?;
    debug!("Depth image created: {:?}", depth_image);

    let view_create_info = vk::ImageViewCreateInfo::builder()
        .image(depth_image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(depth_format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH, // Only DEPTH aspect
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });

    let depth_image_view = unsafe {
        logical_device
            .raw
            .create_image_view(&view_create_info, None)
    }
    .map_err(|e| format!("Failed to create depth image view: {}", e))?;
    debug!("Depth image view created: {:?}", depth_image_view);

    Ok((depth_image, depth_allocation, depth_image_view, depth_format))
}


/// Represents the Vulkan pipeline layout.
#[derive(Debug)]
pub struct PipelineLayout {
    pub raw: vk::PipelineLayout,
    logical_device_raw: ash::Device, // Keep a clone for Drop
}

impl PipelineLayout {
    /// Creates a new Vulkan pipeline layout.
    ///
    /// # Arguments
    /// * `logical_device`: Reference to the `LogicalDevice`.
    /// * `descriptor_set_layouts`: Slice of `vk::DescriptorSetLayout` handles. Can be empty.
    ///
    /// # Returns
    /// `Result<Self, String>` containing the new `PipelineLayout` or an error message.
    pub fn new(
        logical_device: &LogicalDevice,
        descriptor_set_layouts: &[vk::DescriptorSetLayout], // Now takes a slice of layouts
    ) -> Result<Self, String> {
        info!("Creating pipeline layout with {} descriptor set layout(s).", descriptor_set_layouts.len());
        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(descriptor_set_layouts) // Use the passed layouts
            .push_constant_ranges(&[]); // No push constants for now

        let raw_pipeline_layout = unsafe {
            logical_device
                .raw
                .create_pipeline_layout(&layout_create_info, None)
        }
        .map_err(|e| format!("Failed to create pipeline layout: {}", e))?;
        debug!("Pipeline layout created: {:?}", raw_pipeline_layout);

        Ok(Self {
            raw: raw_pipeline_layout,
            logical_device_raw: logical_device.raw.clone(),
        })
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        debug!("Dropping pipeline layout: {:?}", self.raw);
        unsafe {
            self.logical_device_raw
                .destroy_pipeline_layout(self.raw, None);
        }
        debug!("Pipeline layout {:?} destroyed.", self.raw);
    }
}

/// Represents the Vulkan graphics pipeline.
#[derive(Debug)]
pub struct GraphicsPipeline {
    pub raw: vk::Pipeline,
    pub layout: PipelineLayout, // Owns the layout
    logical_device_raw: ash::Device, // Keep a clone for Drop
}

impl GraphicsPipeline {
    /// Creates a new Vulkan graphics pipeline.
    ///
    /// # Arguments
    /// * `logical_device`: Reference to the `LogicalDevice`.
    /// * `render_pass`: The `vk::RenderPass` the pipeline will be used with.
    /// * `swapchain_extent`: The extent of the swapchain images, determining viewport/scissor.
    /// * `pipeline_layout`: Reference to the `PipelineLayout`.
    /// * `vertex_shader_module`: The compiled vertex shader module.
    /// * `fragment_shader_module`: The compiled fragment shader module.
    ///
    /// # Returns
    /// `Result<Self, String>` containing the new `GraphicsPipeline` or an error message.
    #[allow(clippy::too_many_arguments)] // Common for pipeline creation
    pub fn new(
        logical_device: &LogicalDevice,
        render_pass: vk::RenderPass,
        // swapchain_extent is not directly used here anymore if viewport/scissor are dynamic
        // and GraphicsPipeline doesn't need to know it for other reasons.
        // Let's keep it for now if other parts of pipeline setup might depend on it implicitly.
        _swapchain_extent: vk::Extent2D, // Parameter might be unused if viewport/scissor fully dynamic
        pipeline_layout: PipelineLayout, // Takes ownership of the layout
        vertex_shader_module: vk::ShaderModule,
        fragment_shader_module: vk::ShaderModule,
    ) -> Result<Self, String> {
        info!("Creating graphics pipeline...");

        let main_function_name = std::ffi::CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader_module)
                .name(&main_function_name)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_shader_module)
                .name(&main_function_name)
                .build(),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[]) // No vertex bindings for now
            .vertex_attribute_descriptions(&[]); // No vertex attributes for now

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        // Viewport and Scissor will be dynamic
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);
        
        // Dummy viewport and scissor for pipeline creation, will be set dynamically
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            //.p_viewports: not needed if dynamic
            .scissor_count(1);
            //.p_scissors: not needed if dynamic


        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE) // Standard for Vulkan
            .depth_bias_enable(false);

        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false) // No blending for now
            .build();
        
        let color_blend_attachments = [color_blend_attachment];
        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info) // Viewport/scissor are dynamic
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&color_blend_info)
            .layout(pipeline_layout.raw)
            .render_pass(render_pass)
            .subpass(0) // Index of the subpass where this pipeline will be used
            .dynamic_state(&dynamic_state_info) // Specify dynamic states
            .build();

        let graphics_pipelines = unsafe {
            logical_device.raw.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_create_info],
                None,
            )
        }
        .map_err(|(pipelines, result)| {
            // Cleanup potentially created pipelines on error
            for pipeline in pipelines {
                unsafe { logical_device.raw.destroy_pipeline(pipeline, None); }
            }
            format!("Failed to create graphics pipeline: {:?}", result)
        })?;

        info!("Graphics pipeline created successfully.");
        Ok(Self {
            raw: graphics_pipelines[0], // We created only one pipeline
            layout: pipeline_layout,    // Take ownership
            logical_device_raw: logical_device.raw.clone(),
        })
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        debug!("Dropping graphics pipeline: {:?}", self.raw);
        unsafe {
            self.logical_device_raw.destroy_pipeline(self.raw, None);
        }
        // The self.layout (PipelineLayout) will be dropped automatically,
        // and its Drop implementation will destroy the vk::PipelineLayout.
        debug!("Graphics pipeline {:?} destroyed.", self.raw);
    }
}
