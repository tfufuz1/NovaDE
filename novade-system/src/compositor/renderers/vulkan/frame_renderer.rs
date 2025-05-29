use crate::compositor::renderer::vulkan::{
    allocator::Allocator,
    device::LogicalDevice,
    physical_device::PhysicalDeviceInfo,
    instance::VulkanInstance,
    pipeline::{self, UniformBufferObject, PipelineLayout, GraphicsPipeline, create_compute_pipeline, create_compute_pipeline_layout, GraphicsPushConstants},
    render_pass::RenderPass,
    surface_swapchain::SurfaceSwapchain,
    framebuffer::create_framebuffers,
    texture::{self, Texture},
    vertex_input::Vertex,
    buffer_utils::create_and_fill_gpu_buffer,
    sync_primitives::FrameSyncPrimitives,
    error::{Result, VulkanError}, 
    dynamic_uniform_buffer::DynamicUboManager, // Import DynamicUboManager
};
use ash::vk;
use bytemuck; 
use log::{debug, info, warn, error};
use vk_mem;
use std::ffi::c_void;
use std::path::Path;
use std::fs;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const MAX_DYNAMIC_OBJECTS: usize = 64; 
const PIPELINE_CACHE_FILENAME: &str = "novade_pipeline.cache";

#[derive(Debug)]
pub struct FrameRenderer {
    logical_device_raw: ash::Device,
    allocator: Allocator,
    pub surface_swapchain: SurfaceSwapchain,
    render_pass: RenderPass,
    graphics_pipeline: GraphicsPipeline,
    texture: Option<Texture>,
    default_sampler: vk::Sampler,
    graphics_descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    graphics_descriptor_sets: Vec<vk::DescriptorSet>,
    compute_output_images: Vec<vk::Image>,
    compute_output_image_allocations: Vec<vk_mem::Allocation>,
    compute_output_image_views: Vec<vk::ImageView>,
    compute_descriptor_set_layout: vk::DescriptorSetLayout,
    compute_pipeline_layout: PipelineLayout,
    compute_pipeline: vk::Pipeline,
    compute_descriptor_sets: Vec<vk::DescriptorSet>,
    
    // Replaced individual UBO fields with DynamicUboManager
    dynamic_ubo_manager: DynamicUboManager<UniformBufferObject>,

    vertex_buffer: vk::Buffer,
    vertex_buffer_allocation: vk_mem::Allocation,
    index_buffer: vk::Buffer,
    index_buffer_allocation: vk_mem::Allocation,
    index_count: u32,
    framebuffers: Vec<vk::Framebuffer>,
    depth_image: vk::Image,
    depth_image_allocation: vk_mem::Allocation,
    depth_image_view: vk::ImageView,
    depth_format: vk::Format,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    sync_primitives: Vec<FrameSyncPrimitives>,
    current_frame_index: usize,
    swapchain_suboptimal: bool,
    pipeline_cache: vk::PipelineCache,
}

impl FrameRenderer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vulkan_instance: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice,
        allocator: Allocator, // Takes ownership
        surface_swapchain: SurfaceSwapchain,
        render_pass: RenderPass,
        vertex_shader_module: vk::ShaderModule,
        fragment_shader_module: vk::ShaderModule,
    ) -> Result<Self> {
        info!("Creating FrameRenderer with DynamicUboManager...");
        let logical_device_raw = logical_device.raw.clone();

        let initial_cache_data = match fs::read(PIPELINE_CACHE_FILENAME) {
            Ok(data) => { info!("Pipeline cache: Read {} bytes from '{}'.", data.len(), PIPELINE_CACHE_FILENAME); data }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => { info!("Pipeline cache file '{}' not found. Creating new cache.", PIPELINE_CACHE_FILENAME); Vec::new() }
            Err(e) => { warn!("Failed to read pipeline cache file '{}': {}. Proceeding with empty cache.", PIPELINE_CACHE_FILENAME, e); Vec::new() }
        };
        let pipeline_cache_create_info = vk::PipelineCacheCreateInfo::builder()
            .initial_data_size(initial_cache_data.len())
            .initial_data(if initial_cache_data.is_empty() { std::ptr::null() } else { initial_cache_data.as_ptr() as *const _ });
        let pipeline_cache = unsafe { logical_device_raw.create_pipeline_cache(&pipeline_cache_create_info, None) }?;
        
        let sampler_create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE).address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE).mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .min_lod(0.0).max_lod(1.0).anisotropy_enable(false).border_color(vk::BorderColor::FLOAT_OPAQUE_BLACK);
        let default_sampler = unsafe { logical_device_raw.create_sampler(&sampler_create_info, None) }?;

        let graphics_ubo_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC) 
            .descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let graphics_sampler_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let graphics_dsl_bindings = [graphics_ubo_binding.build(), graphics_sampler_binding.build()];
        let graphics_dsl_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&graphics_dsl_bindings);
        let graphics_descriptor_set_layout = unsafe { logical_device_raw.create_descriptor_set_layout(&graphics_dsl_create_info, None) }?;
        
        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0).size(std::mem::size_of::<GraphicsPushConstants>() as u32);
        let graphics_pipeline_layout_obj = PipelineLayout::new(
            logical_device, &[graphics_descriptor_set_layout], &[push_constant_range.build()]
        )?;
        
        let graphics_pipeline = GraphicsPipeline::new(
            logical_device, render_pass.raw, surface_swapchain.extent(),
            graphics_pipeline_layout_obj, vertex_shader_module, fragment_shader_module, pipeline_cache,
        )?;

        let mut compute_output_images = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut compute_output_image_allocations = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut compute_output_image_views = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let compute_image_format = vk::Format::R8G8B8A8_SRGB; 
        let compute_image_usage = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED;
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let (img, alloc, view) = texture::Texture::new_storage_image(
                &logical_device_raw, &allocator, surface_swapchain.extent().width,
                surface_swapchain.extent().height, compute_image_format, compute_image_usage,
            )?;
            compute_output_images.push(img); compute_output_image_allocations.push(alloc); compute_output_image_views.push(view);
        }

        let compute_input_sampler_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE);
        let compute_output_storage_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE);
        let compute_dsl_bindings = [compute_input_sampler_binding.build(), compute_output_storage_binding.build()];
        let compute_dsl_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&compute_dsl_bindings);
        let compute_descriptor_set_layout = unsafe { logical_device_raw.create_descriptor_set_layout(&compute_dsl_create_info, None) }?;
        
        let compute_pipeline_layout = create_compute_pipeline_layout(logical_device, &[compute_descriptor_set_layout])?;
        let compute_shader_spirv = pipeline::load_spirv_file("assets/shaders/invert.comp.spv")?;
        let compute_shader_module = pipeline::create_shader_module(&logical_device_raw, &compute_shader_spirv)?;
        let compute_pipeline = create_compute_pipeline(logical_device, compute_pipeline_layout.raw, compute_shader_module, pipeline_cache)?;
        unsafe { logical_device_raw.destroy_shader_module(compute_shader_module, None); }

        // --- Initialize DynamicUboManager ---
        let dynamic_ubo_manager = DynamicUboManager::<UniformBufferObject>::new(
            &allocator, 
            logical_device, 
            &physical_device_info.properties, 
            MAX_DYNAMIC_OBJECTS
        )?;
        info!("DynamicUboManager initialized.");

        let pool_sizes = [
            vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32).build(),
            vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32 * 2).build(),
            vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32).build(),
        ];
        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder().max_sets(MAX_FRAMES_IN_FLIGHT as u32 * 2).pool_sizes(&pool_sizes);
        let descriptor_pool = unsafe { logical_device_raw.create_descriptor_pool(&descriptor_pool_create_info, None) }?;

        let graphics_dsl_vec = vec![graphics_descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];
        let graphics_d_set_alloc_info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&graphics_dsl_vec);
        let graphics_descriptor_sets = unsafe { logical_device_raw.allocate_descriptor_sets(&graphics_d_set_alloc_info) }?;
        
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let ubo_info = vk::DescriptorBufferInfo::builder()
                .buffer(dynamic_ubo_manager.get_buffer(i)) // Use buffer from DynamicUboManager
                .offset(0)
                .range(dynamic_ubo_manager.get_item_size_for_descriptor()); // Use item size for range
            let compute_out_sampler_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(compute_output_image_views[i]).sampler(default_sampler); 
            let writes = [
                vk::WriteDescriptorSet::builder().dst_set(graphics_descriptor_sets[i]).dst_binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC).buffer_info(&[ubo_info.build()]).build(),
                vk::WriteDescriptorSet::builder().dst_set(graphics_descriptor_sets[i]).dst_binding(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(&[compute_out_sampler_info.build()]).build(),
            ];
            unsafe { logical_device_raw.update_descriptor_sets(&writes, &[]); }
        }
        
        let compute_dsl_vec = vec![compute_descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];
        let compute_d_set_alloc_info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&compute_dsl_vec);
        let compute_descriptor_sets = unsafe { logical_device_raw.allocate_descriptor_sets(&compute_d_set_alloc_info) }?;
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let storage_image_info = vk::DescriptorImageInfo::builder().image_layout(vk::ImageLayout::GENERAL).image_view(compute_output_image_views[i]);
            let output_write = vk::WriteDescriptorSet::builder()
                .dst_set(compute_descriptor_sets[i]).dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(&[storage_image_info.build()]).build();
            unsafe { logical_device_raw.update_descriptor_sets(&[output_write], &[]); }
        }

        let (depth_image, depth_allocation, depth_image_view, depth_format) =
            pipeline::create_depth_resources(logical_device, physical_device_info, vulkan_instance.raw(), &allocator, surface_swapchain.extent())?;
        let vertices = [
            Vertex { pos: [-0.5, -0.5], tex_coord: [0.0, 1.0] }, Vertex { pos: [0.5, -0.5], tex_coord: [1.0, 1.0] },
            Vertex { pos: [0.5, 0.5], tex_coord: [1.0, 0.0] },   Vertex { pos: [-0.5, 0.5], tex_coord: [0.0, 0.0] },
        ];
        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let index_count = indices.len() as u32;
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(physical_device_info.queue_family_indices.graphics_family.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe { logical_device_raw.create_command_pool(&pool_create_info, None) }?;
        let (vertex_buffer, vertex_buffer_allocation) = create_and_fill_gpu_buffer(&allocator, logical_device, command_pool, logical_device.queues.graphics_queue, &vertices, vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let (index_buffer, index_buffer_allocation) = create_and_fill_gpu_buffer(&allocator, logical_device, command_pool, logical_device.queues.graphics_queue, &indices, vk::BufferUsageFlags::INDEX_BUFFER)?;
        let framebuffers = create_framebuffers(logical_device, render_pass.raw, surface_swapchain.image_views(), depth_image_view, surface_swapchain.extent())?;
        let cmd_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder().command_pool(command_pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
        let command_buffers = unsafe { logical_device_raw.allocate_command_buffers(&cmd_buffer_allocate_info) }?;
        let mut sync_primitives = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for i in 0..MAX_FRAMES_IN_FLIGHT { sync_primitives.push(FrameSyncPrimitives::new(logical_device, i == 0)?); }
        
        Ok(Self {
            logical_device_raw, allocator, surface_swapchain, render_pass, graphics_pipeline,
            texture: None, default_sampler, graphics_descriptor_set_layout, descriptor_pool, graphics_descriptor_sets,
            compute_output_images, compute_output_image_allocations, compute_output_image_views,
            compute_descriptor_set_layout, compute_pipeline_layout, compute_pipeline, compute_descriptor_sets,
            dynamic_ubo_manager, // Store the manager
            vertex_buffer, vertex_buffer_allocation, index_buffer, index_buffer_allocation, index_count,
            framebuffers, depth_image, depth_image_allocation, depth_image_view, depth_format,
            command_pool, command_buffers, sync_primitives,
            current_frame_index: 0, swapchain_suboptimal: false, pipeline_cache,
        })
    }

    /// Updates the Uniform Buffer Object (UBO) for a specific object within a specific frame.
    pub fn update_uniform_buffer(&mut self, frame_index: usize, object_index: usize, data: UniformBufferObject) -> Result<()> {
        self.dynamic_ubo_manager.update_data(frame_index, object_index, data)
    }

    pub fn load_texture(
        &mut self, image_path: &str, physical_device_info: &PhysicalDeviceInfo,
        transfer_queue: vk::Queue, vulkan_instance: &VulkanInstance,
    ) -> Result<()> {
        info!("Loading texture from: {}", image_path);
        if self.texture.is_some() {
            info!("Replacing existing texture. Waiting for device idle...");
            unsafe { self.logical_device_raw.device_wait_idle() }?;
            self.texture = None; 
        }
        
        let new_texture_obj = texture::Texture::new_from_file(
            &self.logical_device_raw, 
            physical_device_info, &self.allocator, self.command_pool,
            transfer_queue, image_path, vulkan_instance.raw(),
        )?;
        info!("New texture {} loaded. Updating compute and graphics descriptor sets.", image_path);

        // Sampler for graphics descriptor set (binding 1) should now use the new texture's sampler
        let new_sampler_for_compute_out = new_texture_obj.sampler;

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            // Update Compute Descriptor Sets (Binding 0 for input texture)
            let compute_input_image_info = vk::DescriptorImageInfo::builder()
                .sampler(new_texture_obj.sampler).image_view(new_texture_obj.view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            let compute_input_write = vk::WriteDescriptorSet::builder()
                .dst_set(self.compute_descriptor_sets[i]).dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&[compute_input_image_info.build()]).build();
            
            // Update Graphics Descriptor Sets (Binding 1, for compute output, now uses new texture's sampler)
            let ubo_info = vk::DescriptorBufferInfo::builder()
                .buffer(self.dynamic_ubo_manager.get_buffer(i))
                .offset(0)
                .range(self.dynamic_ubo_manager.get_item_size_for_descriptor());
            let compute_out_sampler_info = vk::DescriptorImageInfo::builder()
               .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
               .image_view(self.compute_output_image_views[i])
               .sampler(new_sampler_for_compute_out); // Use new texture's sampler
            
            let graphics_ubo_write = vk::WriteDescriptorSet::builder().dst_set(self.graphics_descriptor_sets[i]).dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC).buffer_info(&[ubo_info.build()]).build();
            let graphics_image_write = vk::WriteDescriptorSet::builder().dst_set(self.graphics_descriptor_sets[i]).dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(&[compute_out_sampler_info.build()]).build();

            unsafe { self.logical_device_raw.update_descriptor_sets(&[compute_input_write, graphics_ubo_write, graphics_image_write], &[]); }
            debug!("Compute DS {} B0 and Graphics DS {} B1 updated for new texture sampler.", i, i);
        }
        self.texture = Some(new_texture_obj);
        Ok(())
    }

    fn record_graphics_pass(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer_index: u32, // Index into self.framebuffers
        current_frame_idx_for_descriptors: usize,
        elements: &[VulkanRenderElement],
        output_extent: vk::Extent2D,
    ) -> Result<()> {
        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0] } }, // Adjusted clear color
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } },
        ];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass.raw)
            .framebuffer(self.framebuffers[framebuffer_index as usize])
            .render_area(vk::Rect2D { offset: vk::Offset2D::default(), extent: output_extent })
            .clear_values(&clear_values);

        unsafe {
            self.logical_device_raw.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            self.logical_device_raw.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline.raw);

            self.logical_device_raw.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer], &[0]);
            self.logical_device_raw.cmd_bind_index_buffer(command_buffer, self.index_buffer, 0, vk::IndexType::UINT16);
            
            let viewport = vk::Viewport {
                x: 0.0, y: 0.0,
                width: output_extent.width as f32,
                height: output_extent.height as f32,
                min_depth: 0.0, max_depth: 1.0,
            };
            self.logical_device_raw.cmd_set_viewport(command_buffer, 0, &[viewport]);
            let scissor = vk::Rect2D { offset: vk::Offset2D::default(), extent: output_extent };
            self.logical_device_raw.cmd_set_scissor(command_buffer, 0, &[scissor]);

            for (object_idx, element) in elements.iter().enumerate() {
                if object_idx >= MAX_DYNAMIC_OBJECTS {
                    warn!("Exceeded MAX_DYNAMIC_OBJECTS ({}), skipping render element at index {}", MAX_DYNAMIC_OBJECTS, object_idx);
                    continue;
                }

                // For now, UBO is simple and doesn't use per-element data beyond offset.
                // A more complex UBO would take data from `element`.
                let ubo_data_for_element = UniformBufferObject { color_multiplier: [1.0, 1.0, 1.0, 1.0] }; 
                self.dynamic_ubo_manager.update_data(current_frame_idx_for_descriptors, object_idx, ubo_data_for_element)?;
                let dynamic_offset = object_idx as u32 * self.dynamic_ubo_manager.get_aligned_item_size() as u32;

                let image_info = vk::DescriptorImageInfo::builder()
                    .image_layout(element.texture.current_layout) // Should be SHADER_READ_ONLY_OPTIMAL
                    .image_view(element.texture.image_view)
                    .sampler(element.texture.sampler); // Or self.default_sampler

                let ubo_desc_info = vk::DescriptorBufferInfo::builder()
                    .buffer(self.dynamic_ubo_manager.get_buffer(current_frame_idx_for_descriptors))
                    .offset(0) 
                    .range(self.dynamic_ubo_manager.get_item_size_for_descriptor());

                let writes = [
                    vk::WriteDescriptorSet::builder()
                        .dst_set(self.graphics_descriptor_sets[current_frame_idx_for_descriptors])
                        .dst_binding(0) // UBO
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                        .buffer_info(std::slice::from_ref(&ubo_desc_info))
                        .build(),
                    vk::WriteDescriptorSet::builder()
                        .dst_set(self.graphics_descriptor_sets[current_frame_idx_for_descriptors])
                        .dst_binding(1) // Texture Sampler
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&image_info))
                        .build(),
                ];
                self.logical_device_raw.update_descriptor_sets(&writes, &[]);
                
                self.logical_device_raw.cmd_bind_descriptor_sets(
                    command_buffer, vk::PipelineBindPoint::GRAPHICS,
                    self.graphics_pipeline.layout.raw, 0,
                    &[self.graphics_descriptor_sets[current_frame_idx_for_descriptors]],
                    &[dynamic_offset],
                );

                // Convert glam::Mat4 to the format expected by GraphicsPushConstants if necessary
                // For now, assuming GraphicsPushConstants takes a simplified form or is adapted.
                // This is a placeholder for applying element.transform_matrix.
                let push_constant_data = GraphicsPushConstants {
                    tint_color: [1.0, 1.0, 1.0, 1.0], // Example: White, no tint
                    scale: 1.0, // Example: No additional scale
                    // model_view_projection: element.transform_matrix.to_cols_array_2d(), // If shader takes MVP
                };
                self.logical_device_raw.cmd_push_constants(
                    command_buffer, self.graphics_pipeline.layout.raw,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT, 0,
                    bytemuck::bytes_of(&push_constant_data),
                );
                
                self.logical_device_raw.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0);
            }
            
            self.logical_device_raw.cmd_end_render_pass(command_buffer);
        }
        Ok(())
    }

    pub fn draw_frame(
        &mut self,
        vulkan_instance: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice,
        elements: &[VulkanRenderElement], // Changed signature
    ) -> Result<()> {
        let current_sync_primitives = &self.sync_primitives[self.current_frame_index];
        unsafe { self.logical_device_raw.wait_for_fences(&[current_sync_primitives.in_flight_fence], true, u64::MAX) }?;

        let image_index = match unsafe {
            self.surface_swapchain.swapchain_loader.acquire_next_image(
                self.surface_swapchain.swapchain_khr(), u64::MAX,
                current_sync_primitives.image_available_semaphore, vk::Fence::null(),
            )
        } {
            Ok((idx, suboptimal)) => { if suboptimal { self.swapchain_suboptimal = true; } idx }
            Err(vk_err) => { 
                let vulkan_err: VulkanError = vk_err.into();
                match vulkan_err {
                    VulkanError::SwapchainOutOfDate | VulkanError::SurfaceLost => {
                        warn!("Swapchain event during acquire. Triggering recreation. Error: {:?}", vk_err);
                        self.swapchain_suboptimal = true; 
                        self.recreate_swapchain(vulkan_instance, physical_device_info, logical_device)?;
                        return Ok(()); 
                    }
                    _ => { error!("Failed to acquire swapchain image: {:?}", vulkan_err); return Err(vulkan_err); }
                }
            }
        };

        unsafe { self.logical_device_raw.reset_fences(&[current_sync_primitives.in_flight_fence]) }?;
        let current_command_buffer = self.command_buffers[self.current_frame_index];
        unsafe { self.logical_device_raw.reset_command_buffer(current_command_buffer, vk::CommandBufferResetFlags::empty()) }?;
        
        let cmd_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { self.logical_device_raw.begin_command_buffer(current_command_buffer, &cmd_begin_info) }?;

        if self.texture.is_some() {
            let texture_ref = self.texture.as_ref().unwrap();
            let input_texture_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::SHADER_READ).dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(texture_ref.image).subresource_range(vk::ImageSubresourceRange::builder().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(texture_ref.mip_levels).base_array_layer(0).layer_count(1).build())
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED);
            unsafe { self.logical_device_raw.cmd_pipeline_barrier(current_command_buffer, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[input_texture_barrier.build()])};
            
            let compute_output_image = self.compute_output_images[self.current_frame_index];
            let out_to_general_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::GENERAL)
                .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::SHADER_WRITE)
                .image(compute_output_image).subresource_range(vk::ImageSubresourceRange::builder().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1).build());
            unsafe { self.logical_device_raw.cmd_pipeline_barrier(current_command_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[out_to_general_barrier.build()])};

            unsafe {
                self.logical_device_raw.cmd_bind_pipeline(current_command_buffer, vk::PipelineBindPoint::COMPUTE, self.compute_pipeline);
                self.logical_device_raw.cmd_bind_descriptor_sets(current_command_buffer, vk::PipelineBindPoint::COMPUTE, self.compute_pipeline_layout.raw, 0, &[self.compute_descriptor_sets[self.current_frame_index]], &[]);
                let extent = self.surface_swapchain.extent();
                self.logical_device_raw.cmd_dispatch(current_command_buffer, (extent.width + 15) / 16, (extent.height + 15) / 16, 1);
            }

            let out_to_shader_read_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(compute_output_image).subresource_range(vk::ImageSubresourceRange::builder().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1).build());
            unsafe { self.logical_device_raw.cmd_pipeline_barrier(current_command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[out_to_shader_read_barrier.build()])};
        } else { 
            let compute_output_image = self.compute_output_images[self.current_frame_index];
            let out_to_shader_read_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(compute_output_image).subresource_range(vk::ImageSubresourceRange::builder().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1).build());
             unsafe { self.logical_device_raw.cmd_pipeline_barrier(current_command_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[out_to_shader_read_barrier.build()])};
        }
        
        // Compute pass is currently disabled when rendering client elements.
        // The self.texture field is no longer the primary input for the graphics pass if elements are present.
        // If elements is empty, could potentially run compute pass on self.texture if it exists.
        // For now, compute pass is skipped if drawing elements.

        if elements.is_empty() && self.texture.is_some() {
            // Original compute pass logic if no elements and self.texture exists (optional)
            // This part needs careful review if it's intended to run alongside client surfaces
            // or as a fallback. For now, let's assume if elements are provided, compute pass is skipped.
            // If elements are empty, it could render the old self.texture via compute pass.
            // However, the graphics pass descriptor for binding 1 is now dynamically updated.
            // This path is becoming complex to maintain alongside element rendering.
            // Simplified: If there are elements, compute pass is skipped.
            // If no elements, and self.texture exists, original compute + graphics pass on self.texture's output.
            // This part is effectively what was there before, but now conditional.

            // BEGIN: Original compute + graphics pass logic (conditional)
            let texture_ref = self.texture.as_ref().unwrap(); // Safe due to self.texture.is_some()
            let input_texture_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::SHADER_READ).dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(texture_ref.image).subresource_range(vk::ImageSubresourceRange::builder().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(texture_ref.mip_levels).base_array_layer(0).layer_count(1).build())
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED);
            unsafe { self.logical_device_raw.cmd_pipeline_barrier(current_command_buffer, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[input_texture_barrier.build()])};
            
            let compute_output_image = self.compute_output_images[self.current_frame_index];
            let out_to_general_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::GENERAL)
                .src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::SHADER_WRITE)
                .image(compute_output_image).subresource_range(vk::ImageSubresourceRange::builder().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1).build());
            unsafe { self.logical_device_raw.cmd_pipeline_barrier(current_command_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[out_to_general_barrier.build()])};

            unsafe {
                self.logical_device_raw.cmd_bind_pipeline(current_command_buffer, vk::PipelineBindPoint::COMPUTE, self.compute_pipeline);
                self.logical_device_raw.cmd_bind_descriptor_sets(current_command_buffer, vk::PipelineBindPoint::COMPUTE, self.compute_pipeline_layout.raw, 0, &[self.compute_descriptor_sets[self.current_frame_index]], &[]);
                let extent = self.surface_swapchain.extent();
                self.logical_device_raw.cmd_dispatch(current_command_buffer, (extent.width + 15) / 16, (extent.height + 15) / 16, 1);
            }

            let out_to_shader_read_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::GENERAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::SHADER_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(compute_output_image).subresource_range(vk::ImageSubresourceRange::builder().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1).build());
            unsafe { self.logical_device_raw.cmd_pipeline_barrier(current_command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[out_to_shader_read_barrier.build()])};
            
            // Graphics pass for the output of the compute shader
            let ubo_data_for_object0 = UniformBufferObject { color_multiplier: [1.0, 1.0, 1.0, 1.0] }; 
            self.update_uniform_buffer(self.current_frame_index, 0, ubo_data_for_object0)?;
            
            // Manually update descriptor set for binding 1 to use compute_output_image_views[self.current_frame_index]
            // This is what the old code effectively did via initial setup.
            let default_compute_output_image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(self.compute_output_image_views[self.current_frame_index])
                .sampler(self.default_sampler); // Or specific sampler if compute output used a different one

            let ubo_desc_info = vk::DescriptorBufferInfo::builder()
                .buffer(self.dynamic_ubo_manager.get_buffer(self.current_frame_index))
                .offset(0)
                .range(self.dynamic_ubo_manager.get_item_size_for_descriptor());
            
            let writes = [
                 vk::WriteDescriptorSet::builder()
                    .dst_set(self.graphics_descriptor_sets[self.current_frame_index])
                    .dst_binding(0) // UBO
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                    .buffer_info(std::slice::from_ref(&ubo_desc_info))
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(self.graphics_descriptor_sets[self.current_frame_index])
                    .dst_binding(1) // Texture Sampler
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&default_compute_output_image_info))
                    .build(),
            ];
            unsafe { self.logical_device_raw.update_descriptor_sets(&writes, &[]); }
            
            // Create a dummy VulkanRenderElement to pass to the modified record_graphics_pass
            // This is a bit of a hack. Ideally, record_graphics_pass would have different modes.
            // Or, a separate path for "no elements, render compute output".
            // For now, this path is less prioritized.
            // This part is problematic as VulkanRenderElement expects a VulkanRenderableTexture.
            // For simplicity, if elements is empty, the graphics pass will just clear.
            // self.record_graphics_pass(current_command_buffer, image_index, self.current_frame_index, &[], self.surface_swapchain.extent())?;

        }
        // END: Original compute + graphics pass logic (conditional)

        // Always record graphics pass, it will clear the screen if elements is empty.
        self.record_graphics_pass(
            current_command_buffer,
            image_index, 
            self.current_frame_index,
            elements,
            self.surface_swapchain.extent(),
        )?;
        
        unsafe { self.logical_device_raw.end_command_buffer(current_command_buffer) }?;

        let wait_semaphores = [current_sync_primitives.image_available_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [current_sync_primitives.render_finished_semaphore];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores).wait_dst_stage_mask(&wait_stages)
            .command_buffers(&[current_command_buffer]).signal_semaphores(&signal_semaphores);
        unsafe { self.logical_device_raw.queue_submit(logical_device.queues.graphics_queue, &[submit_info.build()], current_sync_primitives.in_flight_fence) }?;

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&[self.surface_swapchain.swapchain_khr()]).image_indices(&[image_index]);
        match unsafe { self.surface_swapchain.swapchain_loader.queue_present(logical_device.queues.present_queue, &present_info) } {
            Ok(suboptimal) => if suboptimal { self.swapchain_suboptimal = true; },
            Err(vk_err) => { 
                let vulkan_err: VulkanError = vk_err.into();
                match vulkan_err {
                    VulkanError::SwapchainOutOfDate | VulkanError::SurfaceLost => { self.swapchain_suboptimal = true; }
                    _ => { error!("Failed to present swapchain image: {:?}", vulkan_err); return Err(vulkan_err); }
                }
            }
        }
        if self.swapchain_suboptimal { self.recreate_swapchain(vulkan_instance, physical_device_info, logical_device)?; }
        self.current_frame_index = (self.current_frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }

    pub fn recreate_swapchain(
        &mut self, vulkan_instance: &VulkanInstance, physical_device_info: &PhysicalDeviceInfo, logical_device: &LogicalDevice,
    ) -> Result<()> {
        info!("Recreating swapchain (FrameRenderer with Dynamic UBOs)...");
        unsafe { self.logical_device_raw.device_wait_idle() }?;

        for &fb in self.framebuffers.iter() { unsafe { self.logical_device_raw.destroy_framebuffer(fb, None); } }
        self.framebuffers.clear();
        unsafe { self.logical_device_raw.destroy_image_view(self.depth_image_view, None); }
        self.allocator.destroy_image(self.depth_image, &self.depth_image_allocation); 
        for i in 0..self.compute_output_images.len() {
            unsafe { self.logical_device_raw.destroy_image_view(self.compute_output_image_views[i], None); }
            self.allocator.destroy_image(self.compute_output_images[i], &self.compute_output_image_allocations[i]);
        }
        self.compute_output_image_views.clear(); self.compute_output_images.clear(); self.compute_output_image_allocations.clear();
        
        self.surface_swapchain.recreate(physical_device_info, logical_device, self.surface_swapchain.extent())?;

        let (new_depth_img, new_depth_alloc, new_depth_view, new_depth_fmt) = pipeline::create_depth_resources(
            logical_device, physical_device_info, vulkan_instance.raw(), &self.allocator, self.surface_swapchain.extent())?;
        self.depth_image = new_depth_img; self.depth_image_allocation = new_depth_alloc;
        self.depth_image_view = new_depth_view; self.depth_format = new_depth_fmt;

        self.framebuffers = create_framebuffers(logical_device, self.render_pass.raw,
            self.surface_swapchain.image_views(), self.depth_image_view, self.surface_swapchain.extent())?;

        self.compute_output_images.reserve(MAX_FRAMES_IN_FLIGHT);
        self.compute_output_image_allocations.reserve(MAX_FRAMES_IN_FLIGHT);
        self.compute_output_image_views.reserve(MAX_FRAMES_IN_FLIGHT);
        let compute_fmt = vk::Format::R8G8B8A8_SRGB;
        let compute_usage = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED;
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let (img, alloc, view) = texture::Texture::new_storage_image(&self.logical_device_raw, &self.allocator,
                self.surface_swapchain.extent().width, self.surface_swapchain.extent().height, compute_fmt, compute_usage)?;
            self.compute_output_images.push(img); self.compute_output_image_allocations.push(alloc); self.compute_output_image_views.push(view);
        }

        let ubo_item_size_for_descriptor = self.dynamic_ubo_manager.get_item_size_for_descriptor();
        let sampler_for_compute_out = self.texture.as_ref().map_or(self.default_sampler, |t| t.sampler);

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            if let Some(texture_ref) = self.texture.as_ref() {
                let input_img_info = vk::DescriptorImageInfo::builder().sampler(texture_ref.sampler).image_view(texture_ref.view).image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
                let input_write = vk::WriteDescriptorSet::builder().dst_set(self.compute_descriptor_sets[i]).dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(&[input_img_info.build()]).build();
                 unsafe { self.logical_device_raw.update_descriptor_sets(&[input_write], &[]); }
            }
            let output_storage_img_info = vk::DescriptorImageInfo::builder().image_layout(vk::ImageLayout::GENERAL).image_view(self.compute_output_image_views[i]);
            let output_write = vk::WriteDescriptorSet::builder().dst_set(self.compute_descriptor_sets[i]).dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(&[output_storage_img_info.build()]).build();
            unsafe { self.logical_device_raw.update_descriptor_sets(&[output_write], &[]); }

            let ubo_info = vk::DescriptorBufferInfo::builder()
                .buffer(self.dynamic_ubo_manager.get_buffer(i))
                .offset(0)
                .range(ubo_item_size_for_descriptor); 
            let ubo_write = vk::WriteDescriptorSet::builder().dst_set(self.graphics_descriptor_sets[i]).dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC).buffer_info(&[ubo_info.build()]).build();
            let final_img_info = vk::DescriptorImageInfo::builder().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(self.compute_output_image_views[i]).sampler(sampler_for_compute_out);
            let final_img_write = vk::WriteDescriptorSet::builder().dst_set(self.graphics_descriptor_sets[i]).dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(&[final_img_info.build()]).build();
            unsafe { self.logical_device_raw.update_descriptor_sets(&[ubo_write, final_img_write], &[]); }
        }
        self.swapchain_suboptimal = false;
        info!("Swapchain recreation complete.");
        Ok(())
    }
}

impl Drop for FrameRenderer {
    fn drop(&mut self) {
        info!("Dropping FrameRenderer...");
        unsafe { if let Err(e) = self.logical_device_raw.device_wait_idle() { error!("Device wait idle error in drop: {}", e); } }

        // Destroy the default sampler if it was created and is managed by FrameRenderer
        // (assuming self.default_sampler is not vk::Sampler::null())
        if self.default_sampler != vk::Sampler::null() {
            unsafe { self.logical_device_raw.destroy_sampler(self.default_sampler, None); }
            debug!("Default sampler destroyed.");
        }

        // Destroy the default sampler if it was created and is managed by FrameRenderer
        // (assuming self.default_sampler is not vk::Sampler::null())
        if self.default_sampler != vk::Sampler::null() {
            unsafe { self.logical_device_raw.destroy_sampler(self.default_sampler, None); }
            debug!("Default sampler destroyed.");
        }
        
        unsafe {
            if self.pipeline_cache != vk::PipelineCache::null() {
                match self.logical_device_raw.get_pipeline_cache_data(self.pipeline_cache) {
                    Ok(cache_data) => {
                        if !cache_data.is_empty() {
                            if let Err(e) = fs::write(PIPELINE_CACHE_FILENAME, &cache_data) {
                                warn!("Failed to write pipeline cache data to '{}': {}", PIPELINE_CACHE_FILENAME, e);
                            } else { info!("Pipeline cache data ({} bytes) saved to '{}'.", cache_data.len(), PIPELINE_CACHE_FILENAME); }
                        } else { info!("No data in pipeline cache to save."); }
                    }
                    Err(e) => warn!("Failed to get pipeline cache data: {}", e),
                }
                self.logical_device_raw.destroy_pipeline_cache(self.pipeline_cache, None);
            }

            if self.default_sampler != vk::Sampler::null() { self.logical_device_raw.destroy_sampler(self.default_sampler, None); }
            if self.compute_pipeline != vk::Pipeline::null() { self.logical_device_raw.destroy_pipeline(self.compute_pipeline, None); }
            if self.compute_descriptor_set_layout != vk::DescriptorSetLayout::null() { self.logical_device_raw.destroy_descriptor_set_layout(self.compute_descriptor_set_layout, None); }
        }
        
        for i in 0..self.compute_output_images.len() {
            unsafe { self.logical_device_raw.destroy_image_view(self.compute_output_image_views[i], None); }
            self.allocator.destroy_image(self.compute_output_images[i], &self.compute_output_image_allocations[i]);
        }
        
        for &fb in self.framebuffers.iter() { unsafe { self.logical_device_raw.destroy_framebuffer(fb, None); } }
        unsafe { self.logical_device_raw.destroy_image_view(self.depth_image_view, None); }
        self.allocator.destroy_image(self.depth_image, &self.depth_image_allocation);
        
        // DynamicUboManager's Drop will handle its own buffers. No explicit loop needed here.
        // The uniform_buffers, uniform_buffer_allocations, uniform_buffer_mapped_pointers fields are now part of DynamicUboManager.
        
        if self.descriptor_pool != vk::DescriptorPool::null() {
             unsafe { self.logical_device_raw.destroy_descriptor_pool(self.descriptor_pool, None); }
        }
        if self.graphics_descriptor_set_layout != vk::DescriptorSetLayout::null() {
             unsafe { self.logical_device_raw.destroy_descriptor_set_layout(self.graphics_descriptor_set_layout, None); }
        }
        
        self.allocator.destroy_buffer(self.vertex_buffer, &self.vertex_buffer_allocation);
        self.allocator.destroy_buffer(self.index_buffer, &self.index_buffer_allocation);
        
        for i in 0..self.sync_primitives.len() { 
            self.sync_primitives[i].destroy(&self.logical_device_raw);
        }
        
        unsafe { self.logical_device_raw.destroy_command_pool(self.command_pool, None); }
        info!("FrameRenderer dropped successfully.");
    }
}

// New methods for texture import for VulkanFrameRenderer

use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::wayland::shm::with_buffer_contents_data;
// Removed: use crate::client_buffer::ClientVkBuffer; 
use super::texture::VulkanRenderableTexture; // Import the new struct
use crate::compositor::renderer_interface::abstraction::RendererError;
use smithay::reexports::drm_fourcc::DrmFourcc;


impl FrameRenderer {
    /// Imports a DMABUF as a Vulkan texture directly using the `Allocator`.
    ///
    /// This method performs the low-level import of the DMABUF into Vulkan memory,
    /// creates a `vk::Image` and `vk::ImageView`, transitions the image layout to
    /// `SHADER_READ_ONLY_OPTIMAL`, and wraps the resources in a `VulkanRenderableTexture`.
    /// It bypasses `ClientVkBuffer` to avoid `ash` vs `vulkanalia` type conflicts.
    ///
    /// # Arguments
    /// - `attributes`: The DMABUF attributes from Smithay.
    /// - `instance_arc`: `Arc` of the `VulkanInstance` (provides raw `ash::Instance`).
    /// - `physical_device_info_arc`: `Arc` of `PhysicalDeviceInfo` (provides raw `vk::PhysicalDevice`).
    /// - `logical_device_arc`: `Arc` of the `LogicalDevice` (provides raw `ash::Device`).
    /// - `allocator_arc`: `Arc` of the `Allocator` (the `novade-system` `ash`-based VMA wrapper).
    ///
    /// # Returns
    /// A `Result` containing a `Box<dyn RenderableTexture>` (specifically `VulkanRenderableTexture`)
    /// or a `RendererError` on failure.
    pub fn import_dmabuf_texture(
        &mut self,
        attributes: &Dmabuf,
        instance_arc: &Arc<VulkanInstance>,
        physical_device_info_arc: &Arc<PhysicalDeviceInfo>,
        logical_device_arc: &Arc<LogicalDevice>,
        allocator_arc: &Arc<Allocator>,
    ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        let drm_fourcc = attributes.format();
        let width = attributes.width();
        let height = attributes.height();
        let plane_count = attributes.num_planes();
        // TODO: For multi-planar, fd and modifier logic would need to iterate or use specific plane indices.
        let fd = attributes.plane_fd(0).map_err(|e| {
            let err_msg = format!("DMABUF fd access error for plane 0: {}. DMABUF: format={:?}, dims={}x{}, planes={}", e, drm_fourcc, width, height, plane_count);
            tracing::error!("{}", err_msg);
            RendererError::Generic(err_msg)
        })?;
        let modifier = attributes.plane_modifier(0).ok(); // .ok() converts Result to Option

        tracing::debug!(
            "Importing DMABUF: fd={}, format={:?}, modifier={:?}, dims={}x{}, planes={}",
            fd, drm_fourcc, modifier, width, height, plane_count
        );

        let vk_fmt = match drm_fourcc {
            DrmFourcc::Argb8888 => vk::Format::B8G8R8A8_UNORM,
            DrmFourcc::Xrgb8888 => vk::Format::B8G8R8A8_UNORM,
            DrmFourcc::Abgr8888 => vk::Format::R8G8B8A8_UNORM,
            DrmFourcc::Xbgr8888 => vk::Format::R8G8B8A8_UNORM,
            _ => {
                let err_msg = format!("Unsupported DRM FourCC format for DMABUF import: {:?}. DMABUF: fd={}, modifier={:?}, dims={}x{}", drm_fourcc, fd, modifier, width, height);
                tracing::error!("{}", err_msg);
                return Err(RendererError::UnsupportedPixelFormat(err_msg));
            }
        };

        let (image, memory) = allocator_arc.import_dma_buf_as_image(
            fd,
            width,
            height,
            vk_fmt,
            modifier,
            vk::ImageUsageFlags::SAMPLED, // Basic usage, add TRANSFER_DST if needed for manual transitions not handled by import
            instance_arc.raw(), 
            physical_device_info_arc.raw(), 
            logical_device_arc.raw(), 
        ).map_err(|e| {
            let err_msg = format!("Allocator::import_dma_buf_as_image failed. DMABUF: fd={}, format={:?}, modifier={:?}, dims={}x{}. Error: {}", fd, drm_fourcc, modifier, width, height, e);
            tracing::error!("{}", err_msg);
            RendererError::Generic(err_msg) 
        })?;

        // Create an image view
        let image_view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk_fmt)
            .subresource_range(vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR) // Adjust if depth/stencil
                .base_mip_level(0).level_count(1)
                .base_array_layer(0).layer_count(1).build());
        let image_view = unsafe { logical_device_arc.raw().create_image_view(&image_view_info, None) }
            .map_err(|e| {
                tracing::error!(
                    "Failed to create ImageView for DMABUF (fd={}, format={:?}, image={:?}). Error: {:?}",
                    fd, vk_fmt, image, e
                );
                unsafe {
                    logical_device_arc.raw().destroy_image(image, None);
                    logical_device_arc.raw().free_memory(memory, None);
                }
                RendererError::Generic(format!("Failed to create ImageView for DMABUF: {}", e))
            })?;
        
        tracing::debug!("Created ImageView {:?} for DMABUF image {:?}", image_view, image);

        // Transition image layout to SHADER_READ_ONLY_OPTIMAL
        if let Err(e) = texture::record_one_time_submit_commands(
            logical_device_arc.raw(),
            self.command_pool,
            logical_device_arc.queues.graphics_queue, 
            |cmd_buffer| {
                let barrier = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::UNDEFINED) 
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR, // Adjust if not color
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .src_access_mask(vk::AccessFlags::empty()) // Or MEMORY_READ/WRITE from import
                    .dst_access_mask(vk::AccessFlags::SHADER_READ);
                unsafe {
                    logical_device_arc.raw().cmd_pipeline_barrier(
                        cmd_buffer,
                        vk::PipelineStageFlags::TOP_OF_PIPE, // Or appropriate stage after import
                        vk::PipelineStageFlags::FRAGMENT_SHADER, // Or first shader stage to use it
                        vk::DependencyFlags::empty(),
                        &[], &[], &[barrier.build()],
                    );
                }
            },
        ).map_err(|e| {
            // Cleanup all resources if layout transition fails
            unsafe {
                logical_device_arc.raw().destroy_image_view(image_view, None);
                logical_device_arc.raw().destroy_image(image, None);
                logical_device_arc.raw().free_memory(memory, None);
            }
            RendererError::Generic(format!("DMABUF layout transition failed: {}", e))
        }) {
            tracing::error!(
                "Layout transition to SHADER_READ_ONLY_OPTIMAL failed for DMABUF image {:?} (fd={}). Error: {:?}",
                image, fd, e
            );
            unsafe {
                logical_device_arc.raw().destroy_image_view(image_view, None);
                logical_device_arc.raw().destroy_image(image, None);
                logical_device_arc.raw().free_memory(memory, None);
            }
            return Err(e);
        }

        tracing::info!(
            "Successfully imported DMABUF (fd={}, format={:?}, modifier={:?}, dims={}x{}) as Vulkan texture (image: {:?}, view: {:?})",
            fd, drm_fourcc, modifier, width, height, image, image_view
        );

        Ok(Box::new(VulkanRenderableTexture::new(
            image,
            image_view,
            Some(memory), 
            None,         
            self.default_sampler,
            vk_fmt,
            width,
            height,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, 
            logical_device_arc.raw().clone(), 
            None, 
        )))
    }

    /// Imports an SHM buffer as a Vulkan texture.
    ///
    /// This involves:
    /// 1. Reading SHM buffer contents.
    /// 2. Creating a Vulkan staging buffer and copying SHM data into it.
    /// 3. Creating a GPU-local `vk::Image` (the destination texture).
    /// 4. Recording and submitting commands to:
    ///    a. Transition destination image to `TRANSFER_DST_OPTIMAL`.
    ///    b. Copy data from staging buffer to destination image.
    ///    c. Transition destination image to `SHADER_READ_ONLY_OPTIMAL`.
    /// 5. Cleaning up the staging buffer.
    /// 6. Wrapping the resources in a `VulkanRenderableTexture`.
    ///
    /// # Arguments
    /// - `buffer`: The Wayland SHM buffer (`WlBuffer`).
    /// - `allocator`: `Arc` of the `Allocator` (VMA wrapper).
    /// - `logical_device_arc`: `Arc` of the `LogicalDevice`.
    ///
    /// # Returns
    /// A `Result` containing a `Box<dyn RenderableTexture>` or a `RendererError`.
    pub fn import_shm_texture(
        &mut self,
        buffer: &WlBuffer,
        allocator: &Arc<Allocator>,
        logical_device_arc: &Arc<LogicalDevice>,
    ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        let (shm_data, width, height, wl_shm_format) = match with_buffer_contents_data(buffer) {
            Ok(data) => data,
            Err(e) => {
                let err_msg = format!("Failed to access SHM buffer contents: {}", e);
                tracing::error!("{}", err_msg);
                return Err(RendererError::Generic(err_msg));
            }
        };
        
        tracing::debug!(
            "Importing SHM buffer: format={:?}, dims={}x{}",
            wl_shm_format, width, height
        );

        let vk_format = match wl_shm_format {
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Argb8888 => vk::Format::B8G8R8A8_UNORM,
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Xrgb8888 => vk::Format::B8G8R8A8_UNORM, 
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Abgr8888 => vk::Format::R8G8B8A8_UNORM,
            smithay::reexports::wayland_server::protocol::wl_shm::Format::Xbgr8888 => vk::Format::R8G8B8A8_UNORM,
            _ => {
                let err_msg = format!("Unsupported SHM format for Vulkan import: {:?}. Dimensions: {}x{}", wl_shm_format, width, height);
                tracing::error!("{}", err_msg);
                return Err(RendererError::UnsupportedPixelFormat(err_msg));
            }
        };
        
        let buffer_size = (width * height * 4) as vk::DeviceSize;

        // 1. Create staging buffer
        let staging_buffer_create_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let staging_allocation_create_info = vk_mem::AllocationCreateInfo {
            .usage(vk_mem::MemoryUsage::CpuToGpu, 
            flags: vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            ..Default::default()
        };
        let (staging_buffer, staging_allocation, staging_alloc_info) = match allocator
            .create_buffer(&staging_buffer_create_info, &staging_allocation_create_info) {
            Ok(res) => res,
            Err(e) => {
                let err_msg = format!("Failed to create staging buffer for SHM import (format: {:?}, dims: {}x{}). Error: {}", vk_format, width, height, e);
                tracing::error!("{}", err_msg);
                return Err(RendererError::Generic(err_msg));
            }
        };
        
        tracing::debug!("Staging buffer {:?} created for SHM texture ({}x{})", staging_buffer, width, height);

        // 2. Copy SHM data to staging buffer
        unsafe {
            let mapped_data = staging_alloc_info.get_mapped_data_mut();
            std::ptr::copy_nonoverlapping(shm_data.as_ptr(), mapped_data, buffer_size as usize);
            // If memory is not HOST_COHERENT, flush is needed. VMA usually handles this if CpuToGpu is used.
            // allocator.flush_allocation(&staging_allocation, 0, buffer_size)?;
        }

        // 3. Create destination image (GPU local)
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk_format)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1).array_layers(1).samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image_allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (dest_image, dest_allocation, _dest_alloc_info) = match allocator
            .create_image(&image_create_info, &image_allocation_create_info) {
            Ok(res) => res,
            Err(e) => {
                let err_msg = format!("Failed to create destination image for SHM import (format: {:?}, dims: {}x{}). Error: {}", vk_format, width, height, e);
                tracing::error!("{}", err_msg);
                allocator.destroy_buffer(staging_buffer, &staging_allocation); // Cleanup staging buffer
                return Err(RendererError::Generic(err_msg));
            }
        };
        tracing::debug!("Destination image {:?} created for SHM texture ({}x{})", dest_image, width, height);

        // 4. Record and submit commands for layout transitions and copy
        if let Err(e) = texture::record_one_time_submit_commands(
            &logical_device_arc.raw, 
            self.command_pool,
            logical_device_arc.queues.graphics_queue,
            |cmd_buffer| {
                // Transition destination image to TRANSFER_DST_OPTIMAL
                let barrier_to_dst = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(dest_image)
                    .subresource_range(vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0).level_count(1)
                        .base_array_layer(0).layer_count(1).build())
                    .src_access_mask(vk::AccessFlags::empty())
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
                unsafe {
                    logical_device_arc.raw.cmd_pipeline_barrier(
                        cmd_buffer,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::DependencyFlags::empty(), &[], &[], &[barrier_to_dst.build()],
                    );
                }

                // Copy buffer to image
                let buffer_image_copy = vk::BufferImageCopy::builder()
                    .buffer_offset(0)
                    .buffer_row_length(0) // 0 means tightly packed
                    .buffer_image_height(0) // 0 means tightly packed
                    .image_subresource(vk::ImageSubresourceLayers {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                    .image_extent(vk::Extent3D { width, height, depth: 1 });
                unsafe {
                    logical_device_arc.raw.cmd_copy_buffer_to_image(
                        cmd_buffer,
                        staging_buffer,
                        dest_image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[buffer_image_copy.build()],
                    );
                }

                // Transition destination image to SHADER_READ_ONLY_OPTIMAL
                let barrier_to_shader_read = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(dest_image)
                    .subresource_range(vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0).level_count(1)
                        .base_array_layer(0).layer_count(1).build())
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ);
                unsafe {
                    logical_device_arc.raw.cmd_pipeline_barrier(
                        cmd_buffer,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER, // Or appropriate shader stage
                        vk::DependencyFlags::empty(), &[], &[], &[barrier_to_shader_read.build()],
                    );
                }
            },
        ) {
            tracing::error!(
                "Command submission failed for SHM texture import (format: {:?}, dims: {}x{}, image: {:?}). Error: {}",
                vk_format, width, height, dest_image, e
            );
            allocator.destroy_image(dest_image, &dest_allocation); // Cleanup dest image
            allocator.destroy_buffer(staging_buffer, &staging_allocation); // Cleanup staging buffer
            return Err(RendererError::Generic(format!("SHM texture command submission failed: {}", e)));
        }
        
        tracing::debug!("SHM data copied to image {:?}, layout transitioned.", dest_image);

        // 5. Destroy staging buffer
        allocator.destroy_buffer(staging_buffer, &staging_allocation);
        tracing::debug!("Staging buffer {:?} destroyed for SHM texture.", staging_buffer);

        // 6. Create image view
        let image_view_info = vk::ImageViewCreateInfo::builder()
            .image(dest_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk_format)
            .subresource_range(vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0).level_count(1)
                .base_array_layer(0).layer_count(1).build());
        let dest_image_view = unsafe { logical_device_arc.raw.create_image_view(&image_view_info, None) }
            .map_err(|e| {
                tracing::error!(
                    "Failed to create ImageView for SHM texture (format: {:?}, image: {:?}). Error: {}",
                    vk_format, dest_image, e
                );
                allocator.destroy_image(dest_image, &dest_allocation); // Cleanup dest image
                RendererError::Generic(format!("Failed to create image view for SHM texture: {}", e))
            })?;
        
        tracing::info!(
            "Successfully imported SHM (format={:?}, dims={}x{}) as Vulkan texture (image: {:?}, view: {:?})",
            wl_shm_format, width, height, dest_image, dest_image_view
        );

        Ok(Box::new(VulkanRenderableTexture::new(
            dest_image,
            dest_image_view,
            None, // No direct vk::DeviceMemory for VMA-allocated image
            Some(dest_allocation), // VMA allocation
            self.default_sampler,
            vk_format,
            width,
            height,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            logical_device_arc.raw().clone(), // Arc<ash::Device>
            Some(allocator.clone()), // Arc<Allocator> for VMA deallocation
        )))
    }
}
