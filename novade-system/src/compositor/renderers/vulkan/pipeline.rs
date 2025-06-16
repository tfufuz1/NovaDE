//! Manages Vulkan pipeline creation and related utilities.
//!
//! This module is responsible for:
//! - Loading SPIR-V shader code from files.
//! - Creating Vulkan shader modules (`VkShaderModule`).
//! - Creating depth buffer resources (image, memory allocation, and image view).
//! - Defining and creating pipeline layouts (`VkPipelineLayout`), which describe the
//!   set of resources (descriptor sets, push constants) used by a pipeline.
//! - Defining and creating graphics pipelines (`VkPipeline`), which configure the
//!   fixed-function stages (e.g., rasterization, viewport) and programmable shader stages
//!   for rendering.
//! - Defining and creating compute pipelines (`VkPipeline`) for general-purpose GPU computations.
//! - Defining common data structures used in pipelines, such as `UniformBufferObject` and `GraphicsPushConstants`.

use crate::compositor::renderer::vulkan::instance::VulkanInstance;
use crate::compositor::renderer::vulkan::vertex_input::Vertex;
use crate::compositor::renderer::vulkan::error::{Result, VulkanError};
use crate::compositor::renderer::vulkan::device::LogicalDevice;
use crate::compositor::renderer::vulkan::allocator::Allocator;
use crate::compositor::renderer::vulkan::physical_device::PhysicalDeviceInfo;

use ash::vk;
use bytemuck; // For Pod, Zeroable traits, and bytes_of for push constants

// --- Uniform Buffer Object Definition ---

/// Defines the structure of the Uniform Buffer Object (UBO) passed to shaders.
///
/// This struct is typically used to pass global data that changes less frequently
/// than per-draw data (for which push constants might be used). For example,
/// it could contain view and projection matrices, or global scene parameters.
///
/// It must be `#[repr(C)]` to ensure a defined memory layout compatible with shaders.
/// The fields should adhere to Vulkan's UBO layout rules (e.g., alignment).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UniformBufferObject {
    pub mvp_matrix: [[f32; 4]; 4], // Model-View-Projection
    pub surface_props: [f32; 4], // x: alpha_multiplier, yzw: unused or for future color ops
}
// --- End Uniform Buffer Object Definition ---

// --- Push Constants Definition ---

/// Defines the structure for push constants used in the graphics pipeline.
///
/// Push constants provide a fast way to upload small amounts of frequently changing
/// data directly to shaders without needing descriptor sets. They are typically used
/// for per-draw data like object transformations or material parameters.
///
/// It must be `#[repr(C)]` and should derive `bytemuck::Pod` and `bytemuck::Zeroable`
/// for easy and safe casting to a byte slice when calling `vkCmdPushConstants`.
/// The total size and member layout must respect Vulkan's push constant limits and alignment.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GraphicsPushConstants {
    /// A tint color applied in the shader, typically RGBA.
    pub tint_color: [f32; 4], 
    // scale: f32, // Removed, handled by MVP matrix in UBO
    pub offset: [f32; 2],      // To store (x, y) of the element's top-left corner (texture sampling related)
    pub element_size: [f32; 2], // To store (width, height) of the element (texture sampling related)
    // Note: Total size is now (4 + 2 + 2) * sizeof(f32) = 8 * 4 = 32 bytes.
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GammaPushConstants {
    pub gamma_value: f32,
}
// --- End Push Constants Definition ---

use log::{debug, info, error, warn};
use std::{fs::File, io::Read, path::Path};
use vk_mem;

/// Loads SPIR-V bytecode from a specified file path.
///
/// Reads the entire content of the file into a `Vec<u8>`. It also performs a basic
/// validation to ensure the file size is a multiple of 4, as SPIR-V consists of
/// 32-bit words.
///
/// # Arguments
///
/// * `path_str`: A string slice representing the path to the SPIR-V shader file.
///
/// # Returns
///
/// A `Result` containing a `Vec<u8>` with the SPIR-V bytecode on success.
/// On failure, returns a `VulkanError::ShaderLoadingError` if the file cannot be
/// opened or read, or if its size is invalid.
pub fn load_spirv_file(path_str: &str) -> Result<Vec<u8>> {
    debug!("Loading SPIR-V shader from: {}", path_str);
    let path = Path::new(path_str);
    let mut file = File::open(path).map_err(|e| 
        VulkanError::ShaderLoadingError(format!("Failed to open shader file {}: {}", path_str, e))
    )?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| 
        VulkanError::ShaderLoadingError(format!("Failed to read shader file {}: {}", path_str, e))
    )?;
    
    if buffer.len() % 4 != 0 {
        let err_msg = format!("SPIR-V file '{}' size ({}) is not a multiple of 4.", path_str, buffer.len());
        error!("{}", err_msg);
        return Err(VulkanError::ShaderLoadingError(err_msg));
    }
    info!("Successfully loaded SPIR-V shader from: {}, size: {} bytes", path_str, buffer.len());
    Ok(buffer)
}

/// Creates a Vulkan shader module (`VkShaderModule`) from SPIR-V bytecode.
///
/// Shader modules are wrappers around shader code that can be used to create
/// pipeline shader stages.
///
/// # Arguments
///
/// * `device`: A reference to the `ash::Device` (logical device) used to create the shader module.
/// * `spirv_code`: A slice of bytes (`&[u8]`) representing the SPIR-V bytecode.
///   The length of this slice must be a multiple of 4.
///
/// # Returns
///
/// A `Result` containing the created `vk::ShaderModule` on success, or a `VulkanError` on failure.
/// Possible errors include:
/// - `VulkanError::ShaderLoadingError`: If `spirv_code.len()` is not a multiple of 4.
/// - `VulkanError::VkResult`: If `vkCreateShaderModule` fails.
///
/// # Safety
///
/// - The `device` handle must be a valid `ash::Device`.
/// - The `spirv_code` slice must contain valid SPIR-V bytecode.
/// - The `spirv_code.as_ptr()` must be suitable for casting to `*const u32`. This implies
///   that the data should be aligned to a 4-byte boundary, although `ash` might handle
///   some cases of unaligned data internally. A warning is logged if potential misalignment is detected.
pub fn create_shader_module(
    device: &ash::Device,
    spirv_code: &[u8],
) -> Result<vk::ShaderModule> {
    debug!("Creating shader module from SPIR-V code ({} bytes)", spirv_code.len());
    
    if spirv_code.len() % 4 != 0 {
        return Err(VulkanError::ShaderLoadingError(
            "SPIR-V code size is not a multiple of 4 bytes for shader module creation.".to_string()
        ));
    }
    if spirv_code.as_ptr().align_offset(std::mem::align_of::<u32>()) != 0 {
        warn!("SPIR-V code is not aligned to 4 bytes during shader module creation. This might be handled by ash, but it's not ideal.");
    }

    let create_info = vk::ShaderModuleCreateInfo::builder()
        .code_size(spirv_code.len())
        .code(unsafe {
            // This cast is safe if spirv_code.len() is a multiple of 4 and
            // the data originates from a source that respects u32 alignment (like a Vec<u32> or a properly formatted file).
            std::slice::from_raw_parts(
                spirv_code.as_ptr() as *const u32,
                spirv_code.len() / std::mem::size_of::<u32>(),
            )
        });

    unsafe { device.create_shader_module(&create_info, None) }.map_err(VulkanError::from)
}

/// Creates depth resources (image, memory allocation, and image view) for the depth buffer.
///
/// The depth buffer is used in rendering to ensure correct Z-ordering of objects.
/// This function selects a suitable depth format from a list of candidates,
/// creates a `VkImage` with optimal tiling and GPU-only memory, and then creates
/// a `VkImageView` for it.
///
/// # Arguments
///
/// * `logical_device`: A reference to the `LogicalDevice`.
/// * `physical_device_info`: A reference to `PhysicalDeviceInfo` for the selected GPU,
///   used for querying format support and device properties.
/// * `instance_handle`: A reference to the `ash::Instance`, needed by `find_supported_format`.
/// * `allocator`: A reference to the VMA `Allocator` for creating the depth image and its memory.
/// * `extent`: The `vk::Extent2D` (width and height) for the depth image, typically matching the swapchain extent.
///
/// # Returns
///
/// A `Result` containing a tuple `(vk::Image, vk_mem::Allocation, vk::ImageView, vk::Format)`
/// representing the depth image, its VMA allocation, its image view, and the chosen depth format,
/// respectively, on success.
/// On failure, returns a `VulkanError`. Possible errors include:
/// - `VulkanError::UnsupportedFormat`: If no suitable depth format is found.
/// - `VulkanError::ResourceCreationError`: If creating the depth image or its view fails.
/// - Errors propagated from `allocator.create_image`.
pub fn create_depth_resources(
    logical_device: &LogicalDevice,
    physical_device_info: &PhysicalDeviceInfo,
    instance_handle: &ash::Instance,
    allocator: &Allocator,
    extent: vk::Extent2D,
) -> Result<(vk::Image, vk_mem::Allocation, vk::ImageView, vk::Format)> {
    info!("Creating depth resources with extent: {:?}", extent);
    let device_name = unsafe { std::ffi::CStr::from_ptr(physical_device_info.properties.device_name.as_ptr()) }
        .to_str().unwrap_or("Unknown Device");

    let depth_format_candidates = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];
    let depth_format = crate::compositor::renderer::vulkan::physical_device::find_supported_format(
        instance_handle, physical_device_info.physical_device, &depth_format_candidates,
        vk::ImageTiling::OPTIMAL, vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    ).ok_or_else(|| VulkanError::UnsupportedFormat(
        format!("Device {}: No suitable depth format found from candidates: {:?}", device_name, depth_format_candidates)
    ))?;
    info!("Device {}: Selected depth format: {:?}", device_name, depth_format);

    let image_create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D).format(depth_format)
        .extent(vk::Extent3D { width: extent.width, height: extent.height, depth: 1 })
        .mip_levels(1).array_layers(1).samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL).usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let allocation_create_info = vk_mem::AllocationCreateInfo { usage: vk_mem::MemoryUsage::GpuOnly, ..Default::default() };

    let (depth_image, depth_allocation, _allocation_info) = allocator
        .create_image(&image_create_info, &allocation_create_info)
        .map_err(|e| VulkanError::ResourceCreationError {
            resource_type: "DepthImage".to_string(),
            message: format!("VMA failed to create depth image for {}: {}", device_name, e)
        })?;
    debug!("Device {}: Depth image created: {:?}", device_name, depth_image);

    let view_create_info = vk::ImageViewCreateInfo::builder()
        .image(depth_image).view_type(vk::ImageViewType::TYPE_2D).format(depth_format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0,
            level_count: 1, base_array_layer: 0, layer_count: 1,
        });

    let depth_image_view = unsafe { logical_device.raw.create_image_view(&view_create_info, None) }?;
    debug!("Device {}: Depth image view created: {:?}", device_name, depth_image_view);

    Ok((depth_image, depth_allocation, depth_image_view, depth_format))
}

/// Represents a Vulkan Pipeline Layout (`VkPipelineLayout`).
///
/// The pipeline layout defines the interface between shader stages and shader resources.
/// It specifies the descriptor set layouts and push constant ranges that a pipeline will use.
/// This struct holds the `vk::PipelineLayout` handle and a clone of the `ash::Device`
/// handle for automatic cleanup via the `Drop` trait.
#[derive(Debug)]
pub struct PipelineLayout {
    /// The raw Vulkan `vk::PipelineLayout` handle.
    pub raw: vk::PipelineLayout,
    /// A clone of the `ash::Device` handle used to create this layout, for `Drop`.
    logical_device_raw: ash::Device,
}

impl PipelineLayout {
    /// Creates a new Vulkan pipeline layout.
    ///
    /// # Arguments
    ///
    /// * `logical_device`: A reference to the `LogicalDevice` for creating the layout.
    /// * `descriptor_set_layouts`: A slice of `vk::DescriptorSetLayout` handles that the
    ///   pipeline will use. Can be empty if no descriptor sets are used.
    /// * `push_constant_ranges`: A slice of `vk::PushConstantRange` structures defining
    ///   any push constants used by the pipeline. Can be empty.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `PipelineLayout` on success, or a `VulkanError`
    /// (typically `VulkanError::VkResult`) if `vkCreatePipelineLayout` fails.
    ///
    /// # Safety
    ///
    /// - `logical_device` must be a valid Vulkan logical device.
    /// - All `vk::DescriptorSetLayout` handles in `descriptor_set_layouts` must be valid.
    /// - All `vk::PushConstantRange` structures must be valid.
    pub fn new(
        logical_device: &LogicalDevice,
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> Result<Self> {
        info!(
            "Creating pipeline layout with {} DSL(s) and {} PCR(s).",
            descriptor_set_layouts.len(), push_constant_ranges.len()
        );
        
        let mut layout_create_info_builder = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(descriptor_set_layouts);
        if !push_constant_ranges.is_empty() {
            layout_create_info_builder = layout_create_info_builder.push_constant_ranges(push_constant_ranges);
        }
        let layout_create_info = layout_create_info_builder.build();

        let raw_pipeline_layout = unsafe {
            logical_device.raw.create_pipeline_layout(&layout_create_info, None)
        }?;
        debug!("Pipeline layout created: {:?}", raw_pipeline_layout);

        Ok(Self {
            raw: raw_pipeline_layout,
            logical_device_raw: logical_device.raw.clone(),
        })
    }
}

impl Drop for PipelineLayout {
    /// Cleans up the Vulkan `VkPipelineLayout` resource.
    /// # Safety
    /// - The `logical_device_raw` handle must still be valid.
    /// - The pipeline layout must not be in use by any command buffer or pipeline when dropped.
    fn drop(&mut self) {
        debug!("Dropping pipeline layout: {:?}", self.raw);
        unsafe { self.logical_device_raw.destroy_pipeline_layout(self.raw, None); }
        debug!("Pipeline layout {:?} destroyed.", self.raw);
    }
}

/// Represents a Vulkan Graphics Pipeline (`VkPipeline`).
///
/// The graphics pipeline defines the sequence of operations that transform vertex data
/// into a rendered image. It includes programmable shader stages (vertex, fragment, etc.)
/// and fixed-function stages (input assembly, rasterization, viewport/scissor, color blending, etc.).
///
/// This struct holds the `vk::Pipeline` handle, owns its `PipelineLayout`, and keeps
/// a clone of the `ash::Device` for cleanup.
#[derive(Debug)]
pub struct GraphicsPipeline {
    /// The raw Vulkan `vk::Pipeline` handle for the graphics pipeline.
    pub raw: vk::Pipeline,
    /// The `PipelineLayout` associated with and owned by this graphics pipeline.
    pub layout: PipelineLayout,
    /// A clone of the `ash::Device` handle for `Drop`.
    logical_device_raw: ash::Device,
}

impl GraphicsPipeline {
    /// Creates a new Vulkan graphics pipeline.
    ///
    /// This function configures a fairly standard graphics pipeline with:
    /// - Vertex and fragment shader stages.
    /// - Vertex input state based on the `Vertex` struct.
    /// - Triangle list input assembly.
    /// - Dynamic viewport and scissor states.
    /// - Fill polygon mode rasterization with back-face culling.
    /// - No multisampling (MSAA count 1).
    /// - Depth testing and writing enabled with `LESS` comparison.
    /// - No color blending.
    ///
    /// # Arguments
    ///
    /// * `logical_device`: A reference to the `LogicalDevice`.
    /// * `render_pass`: The `vk::RenderPass` with which this pipeline will be compatible.
    /// * `_swapchain_extent`: The extent of the swapchain, typically used for initial viewport/scissor
    ///   setup, though these are set to dynamic in this configuration. Marked as unused (`_`) if
    ///   not directly used in fixed-function state.
    /// * `pipeline_layout`: The `PipelineLayout` (which this `GraphicsPipeline` will take ownership of)
    ///   defining descriptor sets and push constants.
    /// * `vertex_shader_module`: The compiled `vk::ShaderModule` for the vertex shader.
    /// * `fragment_shader_module`: The compiled `vk::ShaderModule` for the fragment shader.
    /// * `pipeline_cache`: A `vk::PipelineCache` handle to potentially speed up pipeline creation
    ///   and allow caching pipeline state to disk.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `GraphicsPipeline` on success, or a `VulkanError`
    /// (typically `VulkanError::PipelineCreationError`) if `vkCreateGraphicsPipelines` fails.
    ///
    /// # Safety
    ///
    /// - All Vulkan handles (`logical_device`, `render_pass`, `pipeline_layout.raw`, shader modules, `pipeline_cache`)
    ///   must be valid.
    /// - The pipeline configuration (shaders, render pass, layout) must be compatible.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        logical_device: &LogicalDevice,
        render_pass: vk::RenderPass,
        _swapchain_extent: vk::Extent2D, // TODO: This might be better named 'viewport_extent' or similar if not always swapchain
        pipeline_layout: PipelineLayout, 
        vertex_shader_module: vk::ShaderModule,
        fragment_shader_module: vk::ShaderModule,
        pipeline_cache: vk::PipelineCache,
        enable_depth_test: bool, // New parameter
        enable_depth_write: bool, // New parameter
    ) -> Result<Self> {
        info!("Creating graphics pipeline (depth_test: {}, depth_write: {})...", enable_depth_test, enable_depth_write);
        let main_function_name = std::ffi::CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder().stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader_module).name(&main_function_name).build(),
            vk::PipelineShaderStageCreateInfo::builder().stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_shader_module).name(&main_function_name).build(),
        ];

        let binding_description = [Vertex::get_binding_description()];
        let attribute_descriptions = Vertex::get_attribute_descriptions();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding_description)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST).primitive_restart_enable(false);
        
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder().viewport_count(1).scissor_count(1);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false).rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL).line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK).front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false).rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(enable_depth_test)
            .depth_write_enable(enable_depth_write)
            .depth_compare_op(if enable_depth_test { vk::CompareOp::LESS } else { vk::CompareOp::ALWAYS }) // Meaningful only if test is enabled
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        // ANCHOR[Blend_State_PMA]
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true) // Enable blending
            .src_color_blend_factor(vk::BlendFactor::ONE) // For PMA: SrcColor * 1
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA) // For PMA: DstColor * (1 - SrcAlpha)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE) // Alpha component usually handled like this for PMA
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build();
        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false).attachments(std::slice::from_ref(&color_blend_attachment));

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages).vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info).viewport_state(&viewport_info)
            .rasterization_state(&rasterization_info).multisample_state(&multisample_info)
            .depth_stencil_state(&depth_stencil_info).color_blend_state(&color_blend_info)
            .layout(pipeline_layout.raw).render_pass(render_pass)
            .subpass(0).dynamic_state(&dynamic_state_info).build();

        let graphics_pipelines = match unsafe {
            logical_device.raw.create_graphics_pipelines(pipeline_cache, &[pipeline_create_info], None)
        } {
            Ok(pipelines) => pipelines,
            Err((mut partially_created_pipelines, error_code)) => {
                let err_msg = format!("vkCreateGraphicsPipelines failed: {:?}", error_code);
                error!("{}. Cleaning up {} partially created pipelines.", err_msg, partially_created_pipelines.len());
                for p in partially_created_pipelines.drain(..) { unsafe { logical_device.raw.destroy_pipeline(p, None); } }
                return Err(VulkanError::PipelineCreationError(err_msg));
            }
        };
        
        info!("Graphics pipeline created successfully: {:?}", graphics_pipelines[0]);
        Ok(Self {
            raw: graphics_pipelines[0], 
            layout: pipeline_layout,
            logical_device_raw: logical_device.raw.clone(),
        })
    }
}

impl Drop for GraphicsPipeline {
    /// Cleans up the Vulkan `VkPipeline` resource. The owned `PipelineLayout` is also dropped.
    /// # Safety
    /// - `logical_device_raw` must be valid.
    /// - The pipeline must not be in use by any command buffer when dropped.
    fn drop(&mut self) {
        debug!("Dropping graphics pipeline: {:?}", self.raw);
        unsafe { self.logical_device_raw.destroy_pipeline(self.raw, None); }
        debug!("Graphics pipeline {:?} destroyed.", self.raw);
    }
}

// --- Compute Pipeline Functions ---

/// Creates a pipeline layout for compute operations.
///
/// This is a convenience function that reuses `PipelineLayout::new`.
/// By default, it creates a layout with no push constant ranges.
///
/// # Arguments
///
/// * `logical_device`: A reference to the `LogicalDevice`.
/// * `descriptor_set_layouts`: A slice of `vk::DescriptorSetLayout` handles.
///
/// # Returns
///
/// A `Result` containing the `PipelineLayout` on success, or a `VulkanError` on failure.
pub fn create_compute_pipeline_layout(
    logical_device: &LogicalDevice,
    descriptor_set_layouts: &[vk::DescriptorSetLayout],
) -> Result<PipelineLayout> {
    PipelineLayout::new(logical_device, descriptor_set_layouts, &[])
}

/// Creates a new Vulkan compute pipeline (`VkPipeline`).
///
/// Compute pipelines are used for general-purpose computation on the GPU,
/// separate from the graphics rendering pipeline.
///
/// # Arguments
///
/// * `logical_device`: A reference to the `LogicalDevice`.
/// * `pipeline_layout`: The `vk::PipelineLayout` for the compute pipeline. This should be
///   created using `create_compute_pipeline_layout` or `PipelineLayout::new`.
/// * `compute_shader_module`: The compiled `vk::ShaderModule` for the compute shader.
/// * `pipeline_cache`: A `vk::PipelineCache` handle to speed up pipeline creation.
///
/// # Returns
///
/// A `Result` containing the created `vk::Pipeline` on success, or a `VulkanError`
/// (typically `VulkanError::PipelineCreationError`) if `vkCreateComputePipelines` fails.
///
/// # Safety
///
/// - All Vulkan handles must be valid.
/// - The pipeline configuration must be valid.
pub fn create_compute_pipeline(
    logical_device: &LogicalDevice,
    pipeline_layout: vk::PipelineLayout,
    compute_shader_module: vk::ShaderModule,
    pipeline_cache: vk::PipelineCache,
) -> Result<vk::Pipeline> {
    info!("Creating compute pipeline...");
    let main_function_name = std::ffi::CString::new("main").unwrap();

    let compute_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::COMPUTE).module(compute_shader_module).name(&main_function_name);

    let compute_pipeline_create_info = vk::ComputePipelineCreateInfo::builder()
        .stage(compute_shader_stage_info.build()).layout(pipeline_layout);

    let compute_pipelines = match unsafe {
        logical_device.raw.create_compute_pipelines(pipeline_cache, &[compute_pipeline_create_info.build()], None)
    } {
        Ok(pipelines) => pipelines,
        Err((mut partially_created_pipelines, error_code)) => {
            let err_msg = format!("vkCreateComputePipelines failed: {:?}", error_code);
            error!("{}. Cleaning up {} partially created pipelines.", err_msg, partially_created_pipelines.len());
            for p in partially_created_pipelines.drain(..) { unsafe { logical_device.raw.destroy_pipeline(p, None); } }
            return Err(VulkanError::PipelineCreationError(err_msg));
        }
    };
        
    info!("Compute pipeline created successfully: {:?}", compute_pipelines[0]);
    Ok(compute_pipelines[0])
}
// --- End Compute Pipeline Functions ---
