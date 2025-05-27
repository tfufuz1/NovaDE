use crate::allocator::Allocator;
use crate::device::LogicalDevice;
use crate::error::{Result, VulkanError};
use crate::framebuffer; // For create_framebuffers function
use crate::instance::VulkanInstance;
use crate::physical_device::PhysicalDeviceInfo;
use crate::pipeline::{GraphicsPipeline, PipelineLayout}; // GraphicsPipeline::create_depth_resources_internal
use crate::render_pass; // For RenderPass::find_depth_format
use crate::render_pass::RenderPass;
use crate::surface_swapchain::{SurfaceSwapchain, WaylandDisplayPtr, WaylandSurfacePtr};
use crate::sync_primitives::{FrameSyncPrimitives, create_sync_objects_list};
use crate::pipeline::{Vertex, UniformBufferObject}; // Added UBO
use crate::buffer_utils;
use crate::allocator::{Allocation, Allocator};
use crate::texture::Texture;
use crate::pipeline::ComputePipeline; // Added ComputePipeline
use std::sync::Arc;
use vulkanalia::prelude::v1_0::*;
use glam::{Mat4, Vec3}; // Added glam
use vulkanalia::window as vk_window; // For the temporary surface creation

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct FrameRenderer {
    // Keep Vulkan core components alive
    _instance: VulkanInstance,
    logical_device: Arc<LogicalDevice>,
    allocator: Arc<Allocator>, // Changed to Arc<Allocator>

    surface_swapchain: SurfaceSwapchain,
    render_pass: RenderPass,
    pipeline_layout: PipelineLayout,
    graphics_pipeline: GraphicsPipeline, // Manages its own depth buffer view

    framebuffers: Vec<vk::Framebuffer>,
    
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    // Vertex and Index Buffers
    vertex_buffer: vk::Buffer,
    vertex_buffer_allocation: Allocation,
    index_buffer: vk::Buffer,
    index_buffer_allocation: Allocation,
    index_count: u32,
    
    sync_primitives: Vec<FrameSyncPrimitives>,
    current_frame: usize,
    
    // For recreating swapchain if window is resized
    window_resized_flag: bool,
    preferred_width: u32,
    preferred_height: u32,

    // New fields for texture and UBOs
    loaded_texture: Texture, // Will serve as input to compute pass
    output_compute_texture: Texture, // Output of compute, input to graphics
    
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffer_allocations: Vec<Allocation>,
    uniform_buffer_mapped_ptrs: Vec<*mut std::ffi::c_void>,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>, // For graphics pass

    // Compute-specific resources
    compute_pipeline: ComputePipeline,
    compute_descriptor_sets: Vec<vk::DescriptorSet>,
}

impl FrameRenderer {
    pub fn new(
        window_title: &str,
        enable_validation_layers: bool,
        wayland_display: WaylandDisplayPtr,
        wayland_surface_ptr: WaylandSurfacePtr,
        initial_width: u32,
        initial_height: u32,
    ) -> Result<Self> {
        let instance = VulkanInstance::new(window_title, enable_validation_layers)?;
        
        // Order: Instance -> Surface (via SurfaceSwapchain) -> PhysicalDevice -> LogicalDevice -> Allocator -> Rest
        log::debug!("FrameRenderer::new - Creating temporary surface for physical device selection.");
        let temp_surface_for_phys_dev_check = unsafe {
            vk_window::create_surface(
                instance.raw().instance(), // Get &vulkanalia::Instance
                wayland_display,
                wayland_surface_ptr,
                None,
            )
        }.map_err(VulkanError::VkResult)?;
        log::debug!("FrameRenderer::new - Temporary surface created: {:?}", temp_surface_for_phys_dev_check);

        let physical_device_info = PhysicalDeviceInfo::new(
            &instance, 
            temp_surface_for_phys_dev_check, // Use the temp surface
            wayland_display
        )?;
        log::debug!("FrameRenderer::new - PhysicalDeviceInfo created.");

        let logical_device = Arc::new(LogicalDevice::new(&instance, &physical_device_info)?);
        log::debug!("FrameRenderer::new - LogicalDevice created.");
        let allocator = Arc::new(Allocator::new(&instance, &physical_device_info, &logical_device)?);
        log::debug!("FrameRenderer::new - Allocator created.");
        
        let command_pool = Self::create_command_pool(&logical_device, &physical_device_info)?;
        log::debug!("FrameRenderer::new - CommandPool created (early).");

        // Create Vertex and Index Buffers (Quad)
        let vertices = vec![
           Vertex { pos: [-0.5, -0.5, 0.0], color: [1.0, 1.0, 1.0], tex_coord: [0.0, 0.0] },
           Vertex { pos: [0.5, -0.5, 0.0], color: [1.0, 1.0, 1.0], tex_coord: [1.0, 0.0] },
           Vertex { pos: [-0.5, 0.5, 0.0], color: [1.0, 1.0, 1.0], tex_coord: [0.0, 1.0] },
           Vertex { pos: [0.5, 0.5, 0.0], color: [1.0, 1.0, 1.0], tex_coord: [1.0, 1.0] },
        ];
        let indices = vec![0u32, 1, 2, 2, 1, 3];
        let index_count = indices.len() as u32;

        let (vertex_buffer, vertex_buffer_allocation) =
            buffer_utils::create_gpu_buffer_with_data(
                &logical_device,
                &allocator,
                command_pool,
                logical_device.queues().graphics_queue,
                &vertices,
                vk::BufferUsageFlags::VERTEX_BUFFER,
            )?;
        log::debug!("FrameRenderer::new - Vertex buffer created.");

        let (index_buffer, index_buffer_allocation) =
            buffer_utils::create_gpu_buffer_with_data(
                &logical_device,
                &allocator,
                command_pool,
                logical_device.queues().graphics_queue,
                &indices,
                vk::BufferUsageFlags::INDEX_BUFFER,
            )?;
        log::debug!("FrameRenderer::new - Index buffer created.");

        // Load Texture (this will be input to compute)
        let texture_path = std::path::Path::new("textures/texture.png");
        let loaded_texture = Texture::from_file(
            &logical_device,
            allocator.clone(),
            command_pool,
            logical_device.queues().graphics_queue,
            physical_device_info.properties(),
            physical_device_info.features(),
            texture_path,
        )?;
        log::debug!("FrameRenderer::new - Input texture loaded.");

        // Create Output Texture for Compute
        let output_compute_texture = Texture::new_compute_target(
            &logical_device,
            allocator.clone(),
            loaded_texture.width, // Same dimensions as input
            loaded_texture.height,
            loaded_texture.image_format_vulkan(), // Or a specific format like R8G8B8A8_UNORM
        )?;
        log::debug!("FrameRenderer::new - Output compute texture created.");

        // Create Compute Pipeline
        let compute_shader_path = std::path::Path::new("shaders/compute_shader.comp.spv");
        let compute_pipeline = ComputePipeline::new(&logical_device, compute_shader_path)?;
        log::debug!("FrameRenderer::new - Compute pipeline created.");
        
        // Create UBO Buffers
        let mut uniform_buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut uniform_buffer_allocations = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut uniform_buffer_mapped_ptrs = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let ubo_size = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let (buffer, mut allocation) = allocator.create_buffer(
                &vk::BufferCreateInfo::builder()
                    .size(ubo_size)
                    .usage(vk::BufferUsageFlags::UNIFORM_BUFFER),
                vk_mem_rs::MemoryUsage::CpuToGpu,
                Some(vk_mem_rs::AllocationCreateFlags::MAPPED | vk_mem_rs::AllocationCreateFlags::HOST_ACCESS_RANDOM),
            )?;
            let mapped_ptr = allocator.map_memory(&mut allocation)?;
            uniform_buffers.push(buffer);
            uniform_buffer_allocations.push(allocation);
            uniform_buffer_mapped_ptrs.push(mapped_ptr as *mut std::ffi::c_void);
        }
        log::debug!("FrameRenderer::new - UBOs created and mapped.");
        
        log::debug!("FrameRenderer::new - Creating main SurfaceSwapchain.");
        let surface_swapchain = SurfaceSwapchain::new(
            &instance,
            &physical_device_info,
            &logical_device,
            wayland_display,
            wayland_surface_ptr,
            initial_width,
            initial_height,
        )?;
        log::debug!("FrameRenderer::new - SurfaceSwapchain created. Destroying temporary surface.");
        unsafe { instance.raw().destroy_surface_khr(temp_surface_for_phys_dev_check, None); }
        log::debug!("FrameRenderer::new - Temporary surface destroyed.");


        let render_pass = RenderPass::new(
            &instance, 
            &physical_device_info, 
            &logical_device,
            surface_swapchain.format(),
        )?;
        log::debug!("FrameRenderer::new - RenderPass created.");

        let pipeline_layout = PipelineLayout::new(&logical_device)?;
        log::debug!("FrameRenderer::new - PipelineLayout created.");

        let vert_shader_path = std::path::Path::new("shaders/triangle.vert.spv");
        let frag_shader_path = std::path::Path::new("shaders/triangle.frag.spv");

        // Need to get depth format. RenderPass's `find_depth_format` is private.
        // We'll re-call the static method from render_pass module.
        let depth_format = render_pass::RenderPass::find_depth_format(
            instance.raw().instance(), // Pass &vulkanalia::Instance
            physical_device_info.raw()
        )?;
        log::debug!("FrameRenderer::new - Depth format found: {:?}", depth_format);

        let graphics_pipeline = GraphicsPipeline::new(
            &logical_device,
            &allocator,
            surface_swapchain.extent(),
            &render_pass,
            &pipeline_layout,
            depth_format, // Use the obtained depth format
            vert_shader_path,
            frag_shader_path,
        )?;
        log::debug!("FrameRenderer::new - GraphicsPipeline created.");

        // Create Descriptor Pool (Updated sizes and max_sets)
        let pool_sizes = &[
            vk::DescriptorPoolSize::builder()._type(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32), // Graphics UBO
            vk::DescriptorPoolSize::builder()._type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32), // Graphics Sampler (for compute output)
            vk::DescriptorPoolSize::builder()._type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32), // Compute Input Sampler
            vk::DescriptorPoolSize::builder()._type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32), // Compute Output Storage Image
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets((MAX_FRAMES_IN_FLIGHT * 2) as u32); // *2 for graphics and compute sets
        let descriptor_pool = unsafe { logical_device.raw().create_descriptor_pool(&pool_info, None) }
            .map_err(VulkanError::VkResult)?;
        log::debug!("FrameRenderer::new - DescriptorPool created (updated for compute).");

        // Allocate Graphics Descriptor Sets
        let graphics_dsl = pipeline_layout.descriptor_set_layout;
        let graphics_layouts = vec![graphics_dsl; MAX_FRAMES_IN_FLIGHT];
        let graphics_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&graphics_layouts);
        let descriptor_sets = unsafe { logical_device.raw().allocate_descriptor_sets(&graphics_alloc_info) }
            .map_err(VulkanError::VkResult)?;
        log::debug!("FrameRenderer::new - Graphics DescriptorSets allocated.");

        // Update Graphics Descriptor Sets (to sample from output_compute_texture)
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let ubo_info = vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i])
                .offset(0)
                .range(ubo_size);

            let image_info_for_graphics = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(output_compute_texture.image_view) // Graphics samples compute output
                .sampler(output_compute_texture.sampler);

            let ubo_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0).dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&ubo_info));

            let sampler_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(1).dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&image_info_for_graphics));

            unsafe { logical_device.raw().update_descriptor_sets(&[ubo_write.build(), sampler_write.build()], &[]) };
        }
        log::debug!("FrameRenderer::new - Graphics DescriptorSets updated (sampling compute output).");
        
        // Allocate and Update Compute Descriptor Sets
        let compute_dsl = compute_pipeline.descriptor_set_layout;
        let compute_layouts = vec![compute_dsl; MAX_FRAMES_IN_FLIGHT];
        let compute_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&compute_layouts);
        let compute_descriptor_sets = unsafe { logical_device.raw().allocate_descriptor_sets(&compute_alloc_info) }?;
        log::debug!("FrameRenderer::new - Compute DescriptorSets allocated.");

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let input_image_info_for_compute = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(loaded_texture.image_view)
                .sampler(loaded_texture.sampler);

            let output_image_info_for_compute = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::GENERAL) // Storage image general layout
                .image_view(output_compute_texture.image_view);
                // No sampler for storage image write

            let input_sampler_write = vk::WriteDescriptorSet::builder()
                .dst_set(compute_descriptor_sets[i])
                .dst_binding(0) // Compute input sampler binding
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&input_image_info_for_compute));

            let output_storage_write = vk::WriteDescriptorSet::builder()
                .dst_set(compute_descriptor_sets[i])
                .dst_binding(1) // Compute output storage image binding
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&output_image_info_for_compute));
            
            unsafe { logical_device.raw().update_descriptor_sets(&[input_sampler_write.build(), output_storage_write.build()], &[]) };
        }
        log::debug!("FrameRenderer::new - Compute DescriptorSets updated.");


        let framebuffers = framebuffer::create_framebuffers(
            &logical_device,
            surface_swapchain.image_views(),
            graphics_pipeline.depth_image_view, 
            render_pass.raw(),
            surface_swapchain.extent(),
        )?;
        log::debug!("FrameRenderer::new - Framebuffers created: {} count", framebuffers.len());
        
        let command_buffers = Self::allocate_command_buffers(&logical_device, command_pool, MAX_FRAMES_IN_FLIGHT)?;
        log::debug!("FrameRenderer::new - CommandBuffers allocated: {} count", command_buffers.len());
        
        let sync_primitives = create_sync_objects_list(&logical_device, MAX_FRAMES_IN_FLIGHT)?;
        log::debug!("FrameRenderer::new - SyncPrimitives created: {} sets", sync_primitives.len());

        Ok(Self {
            _instance: instance,
            logical_device,
            allocator,
            surface_swapchain,
            render_pass,
            pipeline_layout,
            graphics_pipeline,
            framebuffers,
            command_pool, // Stays, as it's owned
            command_buffers,
            vertex_buffer,
            vertex_buffer_allocation,
            index_buffer,
            index_buffer_allocation,
            index_count,
            loaded_texture, 
            output_compute_texture, // Added
            uniform_buffers,
            uniform_buffer_allocations,
            uniform_buffer_mapped_ptrs,
            descriptor_pool,
            descriptor_sets, // Graphics descriptor sets
            compute_pipeline, // Added
            compute_descriptor_sets, // Added
            sync_primitives,
            current_frame: 0,
            window_resized_flag: false,
            preferred_width: initial_width,
            preferred_height: initial_height,
        })
    }

    fn create_command_pool(
        logical_device: &LogicalDevice,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> Result<vk::CommandPool> {
        let queue_family_indices = physical_device_info.queue_family_indices();
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER) 
            .queue_family_index(queue_family_indices.graphics_family.ok_or(VulkanError::QueueFamilyNotFound)?);

        unsafe { logical_device.raw().create_command_pool(&pool_info, None) }
            .map_err(VulkanError::VkResult)
    }

    fn allocate_command_buffers(
        logical_device: &LogicalDevice,
        command_pool: vk::CommandPool,
        count: usize,
    ) -> Result<Vec<vk::CommandBuffer>> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count as u32);

        unsafe { logical_device.raw().allocate_command_buffers(&alloc_info) }
            .map_err(VulkanError::VkResult)
    }
    
    pub fn draw_frame(&mut self) -> Result<()> {
        let sync_objects = &self.sync_primitives[self.current_frame];
        let device_arc = self.logical_device.raw(); // Get Arc<Device>
        let device = device_arc.as_ref(); // Get &Device for calls

        unsafe {
            device.wait_for_fences(&[sync_objects.in_flight_fence], true, u64::MAX)
        }.map_err(VulkanError::VkResult)?;
        
        let result_acquire = unsafe {
            device.acquire_next_image_khr(
                self.surface_swapchain.swapchain(),
                u64::MAX,
                sync_objects.image_available_semaphore,
                vk::Fence::null(), 
            )
        };

        let image_index = match result_acquire {
            Ok((index, _suboptimal)) => index,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                log::info!("Swapchain out of date or suboptimal. Recreating.");
                return self.recreate_swapchain_and_dependents();
            }
            Err(e) => return Err(VulkanError::AcquireNextImageError(e)),
        };
        
        unsafe { device.reset_fences(&[sync_objects.in_flight_fence]) }.map_err(VulkanError::VkResult)?;

        // Update UBO
        let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f32();
        let ubo = UniformBufferObject { // ubo is not mut, its content is copied
            model: Mat4::from_rotation_z(time * 0.1 * std::f32::consts::PI), // Simple rotation
            view: Mat4::look_at_rh(
                Vec3::new(0.0, 0.0, 2.0), // Camera position
                Vec3::new(0.0, 0.0, 0.0), // Target position
                Vec3::new(0.0, 1.0, 0.0), // Up vector
            ),
            proj: Mat4::perspective_rh_gl(
                45.0f32.to_radians(), // FOV
                self.surface_swapchain.extent().width as f32 / self.surface_swapchain.extent().height as f32, // Aspect ratio
                0.1,  // Near plane
                10.0, // Far plane
            ),
        };
        // glam's perspective_rh_gl already handles Vulkan's coordinate system adjustments.

        let ubo_ptr = self.uniform_buffer_mapped_ptrs[self.current_frame];
        unsafe { std::ptr::copy_nonoverlapping(&ubo, ubo_ptr as *mut UniformBufferObject, 1); }
        // No explicit flush needed for UBO if VMA picked HOST_COHERENT for CpuToGpu with MAPPED.
        // If not, self.allocator.flush_allocation(&self.uniform_buffer_allocations[self.current_frame], 0, vk::WHOLE_SIZE)?;


        let command_buffer = self.command_buffers[self.current_frame];
        unsafe { device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty()) }
            .map_err(VulkanError::VkResult)?;

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device.begin_command_buffer(command_buffer, &begin_info) }
            .map_err(VulkanError::VkResult)?;

        // --- COMPUTE PASS ---
        // Barrier 1: Transition output_compute_texture to GENERAL for compute shader writes
        // Input (loaded_texture) is assumed to be SHADER_READ_ONLY_OPTIMAL from its creation.
        let pre_compute_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::UNDEFINED) // Or SHADER_READ_ONLY_OPTIMAL if previously sampled
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty()) // Or SHADER_READ
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .image(self.output_compute_texture.image)
            .subresource_range(vk::ImageSubresourceRange::builder().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1).build());
        
        unsafe { device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE, // Or FRAGMENT_SHADER if output_compute_texture was read by graphics
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::DependencyFlags::empty(), &[], &[], &[pre_compute_barrier.build()]
        )};

        unsafe {
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, self.compute_pipeline.pipeline);
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.compute_pipeline.layout,
                0, // firstSet
                &[self.compute_descriptor_sets[self.current_frame]],
                &[], // dynamic offsets
            );
            let group_count_x = (self.output_compute_texture.width + 15) / 16;
            let group_count_y = (self.output_compute_texture.height + 15) / 16;
            device.cmd_dispatch(command_buffer, group_count_x, group_count_y, 1);
        }

        // Barrier 2: Transition output_compute_texture to SHADER_READ_ONLY_OPTIMAL for graphics pass sampling
        let post_compute_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .image(self.output_compute_texture.image)
            .subresource_range(vk::ImageSubresourceRange::builder().aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1).base_array_layer(0).layer_count(1).build());
        
        unsafe { device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::FRAGMENT_SHADER, // For graphics pass sampling
            vk::DependencyFlags::empty(), &[], &[], &[post_compute_barrier.build()]
        )};
        
        // --- GRAPHICS PASS ---
        let clear_color_value = vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0] },
        };
        let clear_depth_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
        };
        let clear_values = &[clear_color_value, clear_depth_value];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass.raw())
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D::default(),
                extent: self.surface_swapchain.extent(),
            })
            .clear_values(clear_values);

        unsafe {
            device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline.raw());

            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout.layout, // Use the public `layout` field
                0, // firstSet
                &[self.descriptor_sets[self.current_frame]],
                &[], // dynamic offsets
            );

            let viewport = vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(self.surface_swapchain.extent().width as f32)
                .height(self.surface_swapchain.extent().height as f32)
                .min_depth(0.0)
                .max_depth(1.0);
            device.cmd_set_viewport(command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D::builder()
                .offset(vk::Offset2D { x:0, y:0 })
                .extent(self.surface_swapchain.extent());
            device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer], &[0]);
            device.cmd_bind_index_buffer(command_buffer, self.index_buffer, 0, vk::IndexType::UINT32);
            device.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0);

            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer)
        }.map_err(VulkanError::VkResult)?;

        let wait_semaphores = &[sync_objects.image_available_semaphore];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = &[sync_objects.render_finished_semaphore];
        
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(&[command_buffer])
            .signal_semaphores(signal_semaphores);
            
        unsafe { device.queue_submit(self.logical_device.queues().graphics_queue, &[submit_info], sync_objects.in_flight_fence) }
            .map_err(VulkanError::VkResult)?;

        let swapchains = &[self.surface_swapchain.swapchain()];
        let image_indices = &[image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores) 
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result_present = unsafe { device.queue_present_khr(self.logical_device.queues().present_queue, &present_info) };
        
        let mut needs_recreation = false;
        match result_present {
            Ok(vk::Success::SUBOPTIMAL_KHR) => {
                log::info!("Swapchain suboptimal. Flagging for recreation.");
                needs_recreation = true;
            }
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                log::info!("Swapchain out of date. Flagging for recreation.");
                needs_recreation = true;
            }
            Err(e) => return Err(VulkanError::PresentError(e)),
            _ => {} 
        }
        
        if self.window_resized_flag {
            log::info!("Window resize detected. Flagging for recreation.");
            needs_recreation = true;
            self.window_resized_flag = false; // Reset flag
        }

        if needs_recreation {
            self.recreate_swapchain_and_dependents()?;
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }
    
    fn recreate_swapchain_and_dependents(&mut self) -> Result<()> {
        unsafe { self.logical_device.raw().device_wait_idle() }.map_err(VulkanError::VkResult)?;
        log::info!("Recreating swapchain and dependent resources.");

        // Destroy old framebuffers
        for framebuffer in self.framebuffers.drain(..) {
            unsafe { self.logical_device.raw().destroy_framebuffer(framebuffer, None) };
        }
        log::debug!("Old framebuffers destroyed.");

        // Destroy old depth buffer resources (view first, then image via allocator)
        unsafe { self.logical_device.raw().destroy_image_view(self.graphics_pipeline.depth_image_view, None) };
        if let Some(allocation) = self.graphics_pipeline.depth_image_allocation.take() {
             self.allocator.destroy_image(self.graphics_pipeline.depth_image, allocation)?;
        } else {
            // This case should ideally not happen if depth_image_allocation was properly Some.
            // If it's None, it means it was already taken or never there.
            log::warn!("Depth image allocation was None before explicit destruction in recreate_swapchain.");
        }
        log::debug!("Old depth buffer resources destroyed.");

        self.surface_swapchain.recreate_swapchain(self.preferred_width, self.preferred_height)?;
        log::debug!("SurfaceSwapchain recreated.");
        
        let depth_format = render_pass::RenderPass::find_depth_format(
            self._instance.raw().instance(),
            self.surface_swapchain.physical_device_handle(), 
        )?;
        log::debug!("Depth format re-queried: {:?}", depth_format);

        let (new_depth_image, new_depth_image_view, new_depth_allocation) = 
            GraphicsPipeline::create_depth_resources_internal( 
                &self.logical_device,
                &self.allocator,
                self.surface_swapchain.extent(),
                depth_format,
            )?;
        log::debug!("New depth resources created.");
        
        // Update graphics pipeline with new depth resources
        // The old depth_image and depth_image_view fields in graphics_pipeline are now stale.
        // The new ones are assigned here.
        self.graphics_pipeline.depth_image = new_depth_image;
        self.graphics_pipeline.depth_image_view = new_depth_image_view;
        self.graphics_pipeline.depth_image_allocation = Some(new_depth_allocation);

        self.framebuffers = framebuffer::create_framebuffers(
            &self.logical_device,
            self.surface_swapchain.image_views(),
            self.graphics_pipeline.depth_image_view, // Use the new depth image view
            self.render_pass.raw(),
            self.surface_swapchain.extent(),
        )?;
        log::info!("Framebuffers recreated.");
        
        Ok(())
    }
    
    pub fn window_resized(&mut self, width: u32, height: u32) {
        self.window_resized_flag = true;
        self.preferred_width = width;
        self.preferred_height = height;
        log::info!("Window resize event received: new dimensions {}x{}. Flag set.", width, height);
    }
}

impl Drop for FrameRenderer {
    fn drop(&mut self) {
        log::debug!("Starting FrameRenderer drop sequence.");
        unsafe {
            if let Err(e) = self.logical_device.raw().device_wait_idle() {
                log::error!("Error during device_wait_idle in FrameRenderer drop: {:?}", e);
            } else {
                log::debug!("Device idle achieved in FrameRenderer drop.");
            }
        }
        
        // Destroy graphics pipeline's VMA-allocated depth image BEFORE graphics_pipeline struct is dropped
        if let Some(allocation) = self.graphics_pipeline.depth_image_allocation.take() {
            log::debug!("Destroying depth image {:?} with allocation {:?} via allocator.", self.graphics_pipeline.depth_image, allocation);
            if let Err(e) = self.allocator.destroy_image(self.graphics_pipeline.depth_image, allocation) {
                 log::error!("Error destroying depth image via allocator: {:?}", e);
            } else {
                 log::debug!("Depth image and allocation destroyed successfully.");
            }
        } else {
            log::warn!("Depth image allocation was already None before explicit destruction in Drop.");
        }
        // GraphicsPipeline struct itself is dropped after this, calling its own Drop
        // for the vk::Pipeline and vk::ImageView.

        // Destroy framebuffers before render_pass or swapchain image views are gone
        for framebuffer in self.framebuffers.drain(..) { // drain consumes the vec
            unsafe { self.logical_device.raw().destroy_framebuffer(framebuffer, None) };
        }
        log::debug!("Framebuffers destroyed.");
        
        // Command pool destroys command buffers implicitly
        unsafe {
            self.logical_device.raw().destroy_command_pool(self.command_pool, None);
            log::debug!("Command pool destroyed.");
        }
        // Sync primitives are dropped one by one when Vec `sync_primitives` is dropped.
        // Each FrameSyncPrimitives has its own Drop.
        
        // Destroy vertex and index buffers
        // Need to ensure these are VMA allocations.
        // The create_gpu_buffer_with_data returns (vk::Buffer, Allocation)
        // So, these fields should be `vertex_buffer_allocation: Allocation` etc.
        // Assuming they are correctly typed and stored.
        if let Err(e) = self.allocator.destroy_buffer(self.vertex_buffer, self.vertex_buffer_allocation) {
            log::error!("Error destroying vertex buffer: {:?}", e);
        } else {
            log::debug!("Vertex buffer destroyed.");
        }
        if let Err(e) = self.allocator.destroy_buffer(self.index_buffer, self.index_buffer_allocation) {
            log::error!("Error destroying index buffer: {:?}", e);
        } else {
            log::debug!("Index buffer destroyed.");
        }

        // Destroy UBOs
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            // VMA handles unmapping on destroy for MAPPED allocations if they were created with MAPPED flag.
            if let Err(e) = self.allocator.destroy_buffer(self.uniform_buffers[i], self.uniform_buffer_allocations[i]) {
               log::error!("Error destroying UBO buffer {}: {:?}", i, e);
            }
        }
        log::debug!("UBOs destroyed.");

        unsafe { self.logical_device.raw().destroy_descriptor_pool(self.descriptor_pool, None); }
        log::debug!("Descriptor pool destroyed (frees descriptor sets implicitly).");
        
        // self.loaded_texture (Texture) is dropped here, its Drop impl handles its resources.
        // self.pipeline_layout (PipelineLayout) is dropped here, its Drop impl handles its resources.
        // self.graphics_pipeline (GraphicsPipeline) is dropped here, its Drop impl handles its resources (except depth image VMA allocation).
        // self.render_pass (RenderPass) is dropped here, its Drop impl handles its resources.
        // self.surface_swapchain (SurfaceSwapchain) is dropped here, its Drop impl handles its resources.
        // self.allocator (Arc<Allocator>) rc decreases, VMA allocator is dropped if rc is 0.
        // self.logical_device (Arc<LogicalDevice>) rc decreases, device is dropped if rc is 0.
        // self._instance (VulkanInstance) is dropped here, its Drop impl handles its resources.
        // - graphics_pipeline (its Drop runs)
        // - pipeline_layout (its Drop runs)
        // - render_pass (its Drop runs)
        // - surface_swapchain (its Drop runs, cleaning up swapchain, image views, surface)
        // - allocator (its Drop runs, VMA cleanup)
        // - logical_device (Arc drops, its Drop runs, destroying device)
        // - _instance (its Drop runs, destroying instance and debug messenger)
        log::info!("FrameRenderer destroyed.");
    }
}

```
