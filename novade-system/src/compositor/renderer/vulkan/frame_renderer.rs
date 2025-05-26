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
        &self, command_buffer: vk::CommandBuffer, image_index: u32, 
        current_frame_idx_for_descriptors: usize, object_index_for_ubo: usize
    ) -> Result<()> {
        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } },
        ];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass.raw).framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D { offset: vk::Offset2D::default(), extent: self.surface_swapchain.extent() })
            .clear_values(&clear_values);

        unsafe {
            self.logical_device_raw.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            self.logical_device_raw.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline.raw);
            
            let dynamic_offset = object_index_for_ubo as u32 * self.dynamic_ubo_manager.get_aligned_item_size() as u32;
            self.logical_device_raw.cmd_bind_descriptor_sets(
                command_buffer, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline.layout.raw,
                0, &[self.graphics_descriptor_sets[current_frame_idx_for_descriptors]], &[dynamic_offset],
            );

            let push_constant_data = GraphicsPushConstants {
                tint_color: [ (self.current_frame_index as f32 * 0.1 + object_index_for_ubo as f32 * 0.5).sin() * 0.5 + 0.5, 
                              0.7, 
                              (self.current_frame_index as f32 * 0.05 + object_index_for_ubo as f32 * 0.3).cos() * 0.5 + 0.5, 
                              1.0 ],
                scale: 0.4 + ((self.current_frame_index as f32 * 0.02 + object_index_for_ubo as f32 * 0.1).sin() * 0.1),
            };
            self.logical_device_raw.cmd_push_constants( command_buffer, self.graphics_pipeline.layout.raw,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT, 0, bytemuck::bytes_of(&push_constant_data),
            );

            self.logical_device_raw.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer], &[0]);
            self.logical_device_raw.cmd_bind_index_buffer(command_buffer, self.index_buffer, 0, vk::IndexType::UINT16);
            let viewport = vk::Viewport { x: 0.0, y: 0.0, width: self.surface_swapchain.extent().width as f32, height: self.surface_swapchain.extent().height as f32, min_depth: 0.0, max_depth: 1.0 };
            self.logical_device_raw.cmd_set_viewport(command_buffer, 0, &[viewport]);
            let scissor = vk::Rect2D { offset: vk::Offset2D::default(), extent: self.surface_swapchain.extent() };
            self.logical_device_raw.cmd_set_scissor(command_buffer, 0, &[scissor]);
            self.logical_device_raw.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0); 
            self.logical_device_raw.cmd_end_render_pass(command_buffer);
        }
        Ok(())
    }

    pub fn draw_frame(
        &mut self, vulkan_instance: &VulkanInstance, physical_device_info: &PhysicalDeviceInfo, logical_device: &LogicalDevice,
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
        
        // Update UBO for the object(s) to be drawn in the graphics pass
        // For simplicity, draw one object using dynamic UBO object_index 0
        let ubo_data_for_object0 = UniformBufferObject { color_multiplier: [1.0, 1.0, 1.0, 1.0] }; 
        self.update_uniform_buffer(self.current_frame_index, 0, ubo_data_for_object0)?;
        // If drawing more objects, loop update_uniform_buffer and record_graphics_pass (or its contents) multiple times.
        // For this task, we'll just draw one object with dynamic offset.
        self.record_graphics_pass(current_command_buffer, image_index, self.current_frame_index, 0)?;
        
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
