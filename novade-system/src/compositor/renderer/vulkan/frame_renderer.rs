use crate::compositor::renderer::vulkan::{
    allocator::Allocator, // Now FrameRenderer will own Allocator for UBO cleanup in Drop
    device::LogicalDevice,
    physical_device::PhysicalDeviceInfo,
    instance::VulkanInstance,
    pipeline::{self, UniformBufferObject, PipelineLayout, GraphicsPipeline},
    render_pass::RenderPass,
    surface_swapchain::SurfaceSwapchain,
    framebuffer::create_framebuffers,
    texture::Texture, // Import Texture
};
use ash::vk;
use log::{debug, info, warn, error};
use vk_mem;
use std::ffi::c_void; // For mapped UBO pointers

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// Manages Vulkan command buffers, synchronization primitives, UBOs, and the main render loop.
#[derive(Debug)]
pub struct FrameRenderer {
    logical_device_raw: ash::Device,
    allocator: Allocator, // FrameRenderer now owns the allocator
    pub surface_swapchain: SurfaceSwapchain,
    render_pass: RenderPass,
    // GraphicsPipeline is now created and owned internally
    graphics_pipeline: GraphicsPipeline,

    // Texture (optional)
    texture: Option<Texture>,

    // Descriptor Set related resources for UBO and Texture
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    // Uniform Buffer resources
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffer_allocations: Vec<vk_mem::Allocation>,
    uniform_buffer_mapped_pointers: Vec<*mut c_void>,

    // Framebuffers - managed by FrameRenderer
    framebuffers: Vec<vk::Framebuffer>,

    // Depth Buffer resources - managed by FrameRenderer
    depth_image: vk::Image,
    depth_image_allocation: vk_mem::Allocation,
    depth_image_view: vk::ImageView,
    depth_format: vk::Format,

    // Command Execution resources
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    // Synchronization primitives
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,

    // State
    current_frame_index: usize,
    swapchain_suboptimal: bool,
}

impl FrameRenderer {
    /// Creates a new `FrameRenderer`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vulkan_instance: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice,
        allocator: Allocator, // Takes ownership of the Allocator
        surface_swapchain: SurfaceSwapchain,
        render_pass: RenderPass,
        // Shader modules are now needed to create GraphicsPipeline internally
        vertex_shader_module: vk::ShaderModule,
        fragment_shader_module: vk::ShaderModule,
    ) -> Result<Self, String> {
        info!("Creating FrameRenderer with UBO support...");
        let logical_device_raw = logical_device.raw.clone();

        // 1. Create DescriptorSetLayout for UBO and Texture Sampler
        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0) // UBO
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();

        let sampler_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1) // Texture Sampler
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();

        let dsl_bindings = [ubo_layout_binding, sampler_layout_binding];
        let dsl_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&dsl_bindings);
        let descriptor_set_layout = unsafe {
            logical_device_raw.create_descriptor_set_layout(&dsl_create_info, None)
        }.map_err(|e| format!("Failed to create combined descriptor set layout: {}", e))?;
        info!("Combined DescriptorSetLayout created: {:?}", descriptor_set_layout);

        // 2. Create PipelineLayout
        let pipeline_layout_obj = PipelineLayout::new(logical_device, &[descriptor_set_layout])?;
        info!("PipelineLayout created for FrameRenderer: {:?}", pipeline_layout_obj.raw);

        // 3. Create GraphicsPipeline using the PipelineLayout
        let graphics_pipeline = GraphicsPipeline::new(
            logical_device,
            render_pass.raw,
            surface_swapchain.extent(),
            pipeline_layout_obj, // GraphicsPipeline takes ownership of PipelineLayout
            vertex_shader_module,
            fragment_shader_module,
        )?;
        info!("GraphicsPipeline created for FrameRenderer: {:?}", graphics_pipeline.raw);


        // 4. Create Uniform Buffers (one per frame in flight)
        let mut uniform_buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut uniform_buffer_allocations = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut uniform_buffer_mapped_pointers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let ubo_size = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_create_info = vk::BufferCreateInfo::builder()
                .size(ubo_size)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE); // Keep it simple

            let allocation_create_info = vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::CpuToGpu,
                flags: vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
                ..Default::default()
            };

            let (buffer, allocation, alloc_info) = allocator
                .create_buffer(&buffer_create_info, &allocation_create_info)
                .map_err(|e| format!("Failed to create UBO {} with VMA: {}", i, e))?;
            
            uniform_buffers.push(buffer);
            uniform_buffer_allocations.push(allocation);
            uniform_buffer_mapped_pointers.push(alloc_info.get_mapped_data_mut());
            debug!("UBO {} created: {:?}, mapped pointer: {:?}", i, buffer, alloc_info.get_mapped_data_mut());
        }
        info!("{} Uniform Buffers created.", MAX_FRAMES_IN_FLIGHT);

        // 5. Create DescriptorPool
        let pool_sizes = [vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32)
            .build(),
        vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32) // One sampler per frame if texture changes per frame, or 1 if global
            .build()
        ];
        let dp_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32) // Max number of descriptor sets that can be allocated
            .pool_sizes(&pool_sizes);
        let descriptor_pool = unsafe {
            logical_device_raw.create_descriptor_pool(&dp_create_info, None)
        }.map_err(|e| format!("Failed to create descriptor pool: {}", e))?;
        info!("DescriptorPool created for UBO and Sampler: {:?}", descriptor_pool);

        // 6. Allocate DescriptorSets (initially only UBO is updated)
        let d_set_layouts: Vec<vk::DescriptorSetLayout> =
            std::iter::repeat(descriptor_set_layout).take(MAX_FRAMES_IN_FLIGHT).collect();
        let d_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&d_set_layouts);
        let descriptor_sets = unsafe {
            logical_device_raw.allocate_descriptor_sets(&d_set_alloc_info)
        }.map_err(|e| format!("Failed to allocate descriptor sets: {}", e))?;
        info!("{} DescriptorSets allocated.", descriptor_sets.len());

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info = vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i])
                .offset(0)
                .range(ubo_size)
                .build();
            let buffer_infos = [buffer_info];

            let write_set = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0) // UBO binding
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build();
            
            // Initially, only update UBO. Texture part will be updated when texture is loaded.
            // If a texture is already available here, it could be updated too.
            unsafe { logical_device_raw.update_descriptor_sets(&[ubo_write_set], &[]); }
            debug!("DescriptorSet {} updated for UBO {:?}. Texture part uninitialized.", i, uniform_buffers[i]);
        }

        // 7. Create Depth Resources
        let (depth_image, depth_image_allocation, depth_image_view, depth_format) =
            pipeline::create_depth_resources(
                logical_device,
                physical_device_info,
                vulkan_instance.raw(),
                &allocator, // Pass reference to allocator
                surface_swapchain.extent(),
            )?;
        info!("Initial depth resources created for FrameRenderer.");

        // 8. Create Framebuffers (largely same as before)
        let framebuffers = create_framebuffers(
            logical_device,
            render_pass.raw,
            surface_swapchain.image_views(),
            depth_image_view,
            surface_swapchain.extent(),
        )?;
        info!("Initial framebuffers created for FrameRenderer: {} count", framebuffers.len());


        // Create Command Pool
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(physical_device_info.queue_family_indices.graphics_family.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER); // Allow resetting individual command buffers

        let command_pool = unsafe { logical_device_raw.create_command_pool(&pool_create_info, None) }
            .map_err(|e| format!("Failed to create command pool: {}", e))?;
        info!("Command pool created: {:?}", command_pool);

        // Allocate Command Buffers
        let cmd_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);

        let command_buffers = unsafe { logical_device_raw.allocate_command_buffers(&cmd_buffer_allocate_info) }
            .map_err(|e| format!("Failed to allocate command buffers: {}", e))?;
        info!("Allocated {} command buffers.", command_buffers.len());

        // Create Synchronization Primitives
        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        let fence_create_info_signaled = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let fence_create_info_unsignaled = vk::FenceCreateInfo::builder();


        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let image_available_semaphore = unsafe { logical_device_raw.create_semaphore(&semaphore_create_info, None) }
                .map_err(|e| format!("Failed to create image_available_semaphore[{}]: {}", i, e))?;
            image_available_semaphores.push(image_available_semaphore);

            let render_finished_semaphore = unsafe { logical_device_raw.create_semaphore(&semaphore_create_info, None) }
                .map_err(|e| format!("Failed to create render_finished_semaphore[{}]: {}", i, e))?;
            render_finished_semaphores.push(render_finished_semaphore);

            // First fence is signaled to allow first wait_for_fences to pass
            let fence_ci = if i == 0 { &fence_create_info_signaled } else { &fence_create_info_unsignaled };
            let in_flight_fence = unsafe { logical_device_raw.create_fence(fence_ci, None) }
                 .map_err(|e| format!("Failed to create in_flight_fence[{}]: {}", i, e))?;
            in_flight_fences.push(in_flight_fence);
        }
        info!("Synchronization primitives created ({} sets).", MAX_FRAMES_IN_FLIGHT);


        Ok(Self {
            logical_device_raw,
            allocator,
            surface_swapchain,
            render_pass,
            graphics_pipeline,
            texture: None, // Initialize texture as None
            descriptor_set_layout,
            descriptor_pool,
            descriptor_sets,
            uniform_buffers,
            uniform_buffer_allocations,
            uniform_buffer_mapped_pointers,
            framebuffers,
            depth_image,
            depth_image_allocation,
            depth_image_view,
            depth_format,
            command_pool,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame_index: 0,
            swapchain_suboptimal: false,
        })
    }

    /// Updates the Uniform Buffer Object for the current frame.
    pub fn update_uniform_buffer(
        &mut self,
        frame_index: usize, // Should be self.current_frame_index from draw_frame context
        data: UniformBufferObject,
    ) -> Result<(), String> {
        if frame_index >= MAX_FRAMES_IN_FLIGHT {
            return Err(format!("Invalid frame_index {} for UBO update.", frame_index));
        }
        let ptr = self.uniform_buffer_mapped_pointers[frame_index];
        if ptr.is_null() {
            return Err(format!("UBO for frame_index {} is not mapped.", frame_index));
        }
        unsafe {
            std::ptr::copy_nonoverlapping(&data, ptr as *mut UniformBufferObject, 1);
        }
        Ok(())
    }

    /// Loads a texture and updates descriptor sets.
    pub fn load_texture(
        &mut self,
        image_path: &str,
        // Required context for Texture::new_from_file
        physical_device_info: &PhysicalDeviceInfo, // Needed for sampler properties
        graphics_queue: vk::Queue, // For one-time submit commands for texture loading
    ) -> Result<(), String> {
        info!("Loading texture: {}", image_path);
        if self.texture.is_some() {
            // Destroy existing texture before loading a new one.
            // This involves waiting for device idle to ensure resources are not in use.
            unsafe { self.logical_device_raw.device_wait_idle() }
                .map_err(|e| format!("Failed to wait for device idle before replacing texture: {}", e))?;
            self.texture = None; // This will trigger Texture's Drop
            info!("Previous texture dropped.");
        }

        let new_texture = Texture::new_from_file(
            &LogicalDevice { raw: self.logical_device_raw.clone(), queues: self.allocator.raw_allocator().get_device().get_queues_ref_TODO_NEEDS_QUEUES_IN_LOGICAL_DEVICE_OR_PASSING_IT() }, // HACK: Need proper LogicalDevice or pass queues
            physical_device_info,
            &self.allocator,
            self.command_pool,
            graphics_queue,
            image_path,
        )?;
        info!("Texture {} loaded successfully, view: {:?}, sampler: {:?}", image_path, new_texture.view, new_texture.sampler);
        
        // Update descriptor sets for the new texture
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let image_info = vk::DescriptorImageInfo::builder()
                .sampler(new_texture.sampler)
                .image_view(new_texture.view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .build();
            let image_infos = [image_info];

            let texture_write_set = vk::WriteDescriptorSet::builder()
                .dst_set(self.descriptor_sets[i])
                .dst_binding(1) // Texture sampler binding
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_infos)
                .build();
            
            unsafe { self.logical_device_raw.update_descriptor_sets(&[texture_write_set], &[]); }
            debug!("DescriptorSet {} updated for new texture.", i);
        }
        
        self.texture = Some(new_texture);
        Ok(())
    }


    /// Records commands into a command buffer for rendering a frame.
    fn record_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        image_index: u32, // Index of the swapchain image to render to
        // current_frame_idx is needed to select the correct descriptor set
        current_frame_idx_for_descriptors: usize,
    ) -> Result<(), String> {
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { self.logical_device_raw.begin_command_buffer(command_buffer, &begin_info) }
            .map_err(|e| format!("Failed to begin command buffer recording: {}", e))?;

        let clear_values = [
            vk::ClearValue { // Color
                color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] },
            },
            vk::ClearValue { // Depth
                depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
            },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass.raw)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.surface_swapchain.extent(),
            })
            .clear_values(&clear_values);

        unsafe {
            self.logical_device_raw.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            self.logical_device_raw.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline.raw,
            );

            // Bind Descriptor Set for UBO
            self.logical_device_raw.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline.layout.raw, // Use layout from the owned graphics_pipeline
                0, // firstSet
                &[self.descriptor_sets[current_frame_idx_for_descriptors]],
                &[], // No dynamic offsets
            );

            // Dynamic viewport
            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.surface_swapchain.extent().width as f32,
                height: self.surface_swapchain.extent().height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            self.logical_device_raw.cmd_set_viewport(command_buffer, 0, &[viewport]);

            // Dynamic scissor
            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.surface_swapchain.extent(),
            };
            self.logical_device_raw.cmd_set_scissor(command_buffer, 0, &[scissor]);

            // Draw call (simple triangle)
            self.logical_device_raw.cmd_draw(command_buffer, 3, 1, 0, 0);

            self.logical_device_raw.cmd_end_render_pass(command_buffer);
            self.logical_device_raw.end_command_buffer(command_buffer)
        }.map_err(|e| format!("Failed during command buffer recording: {}", e))?;

        Ok(())
    }

    /// Renders a single frame.
    pub fn draw_frame(
        &mut self,
        // For swapchain recreation, these are needed.
        vulkan_instance: &VulkanInstance, // Passed for recreate_swapchain
        physical_device_info: &PhysicalDeviceInfo, // Passed for recreate_swapchain
        logical_device: &LogicalDevice, // Passed for queue access in submit & present, and for recreate_swapchain
    ) -> Result<(), String> {
        // Update UBO
        let ubo_data = UniformBufferObject {
            color_multiplier: [
                (self.current_frame_index as f32 * 0.3).sin() * 0.5 + 0.5, // R
                (self.current_frame_index as f32 * 0.5).cos() * 0.5 + 0.5, // G
                0.8, // B
                1.0  // A
            ],
        };
        self.update_uniform_buffer(self.current_frame_index, ubo_data)?;


        let fence_to_wait = self.in_flight_fences[self.current_frame_index];
        unsafe { self.logical_device_raw.wait_for_fences(&[fence_to_wait], true, u64::MAX) }
            .map_err(|e| format!("Failed to wait for in-flight fence: {}", e))?;

        let image_index: u32;
        match unsafe {
            self.surface_swapchain.swapchain_loader.acquire_next_image(
                self.surface_swapchain.swapchain_khr(),
                u64::MAX,
                self.image_available_semaphores[self.current_frame_index],
                vk::Fence::null(),
            )
        } {
            Ok((idx, suboptimal)) => {
                image_index = idx;
                if suboptimal { self.swapchain_suboptimal = true; }
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.swapchain_suboptimal = true;
                // Recreate swapchain and return early.
                self.recreate_swapchain(vulkan_instance, physical_device_info, logical_device)?; // allocator is now self.allocator
                return Ok(());
            }
            Err(e) => return Err(format!("Failed to acquire swapchain image: {}", e)),
        }

        // Only reset fence if we are submitting work
        unsafe { self.logical_device_raw.reset_fences(&[fence_to_wait]) }
            .map_err(|e| format!("Failed to reset in-flight fence: {}", e))?;
        
        let current_command_buffer = self.command_buffers[self.current_frame_index];
        unsafe {
            self.logical_device_raw.reset_command_buffer(current_command_buffer, vk::CommandBufferResetFlags::empty())
        }.map_err(|e| format!("Failed to reset command buffer: {}", e))?;

        self.record_command_buffer(current_command_buffer, image_index, self.current_frame_index)?;

        let wait_semaphores = [self.image_available_semaphores[self.current_frame_index]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame_index]];
        let submit_command_buffers = [current_command_buffer];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&submit_command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build();

        unsafe {
            self.logical_device_raw.queue_submit(
                logical_device.queues.graphics_queue, // Use queue from original LogicalDevice
                &[submit_info],
                fence_to_wait,
            )
        }.map_err(|e| format!("Failed to submit command buffer: {}", e))?;

        let present_wait_semaphores = [self.render_finished_semaphores[self.current_frame_index]];
        let present_swapchains = [self.surface_swapchain.swapchain_khr()];
        let present_image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&present_wait_semaphores)
            .swapchains(&present_swapchains)
            .image_indices(&present_image_indices);

        match unsafe {
            self.surface_swapchain.swapchain_loader.queue_present(
                logical_device.queues.present_queue, // Use queue from original LogicalDevice
                &present_info
            )
        } {
            Ok(suboptimal) => {
                if suboptimal { self.swapchain_suboptimal = true; }
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.swapchain_suboptimal = true;
            }
            Err(e) => return Err(format!("Failed to present swapchain image: {}", e)),
        }
        
        // Check and handle swapchain recreation if needed
        // This check is after presenting
        if self.swapchain_suboptimal {
            self.recreate_swapchain(vulkan_instance, physical_device_info, logical_device)?; // allocator is now self.allocator
        }

        self.current_frame_index = (self.current_frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }

    /// Recreates the swapchain and dependent resources.
    /// This is typically called when the window is resized or the surface properties change.
    pub fn recreate_swapchain(
        &mut self,
        vulkan_instance: &VulkanInstance,
        physical_device_info: &PhysicalDeviceInfo,
        logical_device: &LogicalDevice, // Still needed for SurfaceSwapchain's recreate
    ) -> Result<(), String> {
        info!("Recreating swapchain (FrameRenderer)...");
        unsafe { self.logical_device_raw.device_wait_idle() }
            .map_err(|e| format!("Failed to wait for device idle before swapchain recreation: {}", e))?;

        // 1. Cleanup old framebuffers
        for &framebuffer in self.framebuffers.iter() {
            unsafe { self.logical_device_raw.destroy_framebuffer(framebuffer, None); }
        }
        self.framebuffers.clear();
        debug!("Old framebuffers destroyed.");

        // 2. Cleanup old depth resources
        unsafe { self.logical_device_raw.destroy_image_view(self.depth_image_view, None); }
        self.allocator.destroy_image(self.depth_image, &self.depth_image_allocation);
        debug!("Old depth resources destroyed.");

        // 3. Recreate swapchain
        // We need to determine the new extent. For now, assume it's handled by SurfaceSwapchain
        // or we pass the same extent. A real app would get this from windowing system.
        // Let's assume SurfaceSwapchain's recreate can determine the new extent or we pass current.
        // For this task, we pass the current extent, assuming it will be updated if necessary by SurfaceSwapchain itself.
        // A more robust way is to get the new extent from the window manager and pass it to surface_swapchain.recreate().
        let new_extent = self.surface_swapchain.extent(); // Or get from window manager
        self.surface_swapchain.recreate(physical_device_info, logical_device, new_extent)?;
        info!("Swapchain itself recreated.");

        // 4. Recreate depth resources with the new swapchain extent
        let (new_depth_image, new_depth_alloc, new_depth_view, new_depth_fmt) =
            pipeline::create_depth_resources(
                logical_device,
                physical_device_info,
                vulkan_instance.raw(),
                &self.allocator, // Pass reference to owned allocator
                self.surface_swapchain.extent(),
            )?;
        self.depth_image = new_depth_image;
        self.depth_image_allocation = new_depth_alloc;
        self.depth_image_view = new_depth_view;
        self.depth_format = new_depth_fmt; // Should be the same, but good to update
        info!("New depth resources recreated.");

        // 5. Recreate framebuffers
        self.framebuffers = create_framebuffers(
            logical_device,
            self.render_pass.raw,
            self.surface_swapchain.image_views(),
            self.depth_image_view,
            self.surface_swapchain.extent(),
        )?;
        info!("New framebuffers recreated: {} count.", self.framebuffers.len());

        self.swapchain_suboptimal = false;
        info!("Swapchain recreation complete.");
        Ok(())
    }
}

impl Drop for FrameRenderer {
    fn drop(&mut self) {
        info!("Dropping FrameRenderer...");
        // Ensure device is idle before destroying resources
        unsafe {
            if let Err(e) = self.logical_device_raw.device_wait_idle() {
                error!("Error waiting for device idle in FrameRenderer drop: {}", e);
            }
        }

        // Destroy depth resources
        unsafe { self.logical_device_raw.destroy_image_view(self.depth_image_view, None); }
        // Destroy UBOs and their allocations
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            if i < self.uniform_buffers.len() && i < self.uniform_buffer_allocations.len() {
                self.allocator.destroy_buffer(self.uniform_buffers[i], &self.uniform_buffer_allocations[i]);
            }
        }
        self.uniform_buffers.clear();
        self.uniform_buffer_allocations.clear();
        self.uniform_buffer_mapped_pointers.clear(); // Pointers become invalid
        debug!("Uniform buffers destroyed.");

        // Destroy DescriptorPool (frees DescriptorSets)
        unsafe { self.logical_device_raw.destroy_descriptor_pool(self.descriptor_pool, None); }
        debug!("DescriptorPool destroyed.");

        // Destroy DescriptorSetLayout
        unsafe { self.logical_device_raw.destroy_descriptor_set_layout(self.descriptor_set_layout, None); }
        debug!("DescriptorSetLayout destroyed.");

        // Destroy depth resources
        unsafe { self.logical_device_raw.destroy_image_view(self.depth_image_view, None); }
        self.allocator.destroy_image(self.depth_image, &self.depth_image_allocation);
        debug!("Depth resources destroyed.");

        // Destroy framebuffers
        for &framebuffer in self.framebuffers.iter() {
            unsafe { self.logical_device_raw.destroy_framebuffer(framebuffer, None); }
        }
        debug!("Framebuffers destroyed.");

        // Destroy synchronization primitives
        for semaphore in self.image_available_semaphores.drain(..) {
            unsafe { self.logical_device_raw.destroy_semaphore(semaphore, None); }
        }
        for semaphore in self.render_finished_semaphores.drain(..) {
            unsafe { self.logical_device_raw.destroy_semaphore(semaphore, None); }
        }
        for fence in self.in_flight_fences.drain(..) {
            unsafe { self.logical_device_raw.destroy_fence(fence, None); }
        }
        debug!("Synchronization primitives destroyed.");

        // Command buffers are freed when the pool is destroyed
        unsafe { self.logical_device_raw.destroy_command_pool(self.command_pool, None); }
        debug!("Command pool destroyed.");

        // Owned components (Allocator, SurfaceSwapchain, RenderPass, GraphicsPipeline, Texture)
        // will be dropped automatically, and their Drop impls will handle their resources.
        // logical_device_raw is a clone of ash::Device, its Drop is handled by ash.
        info!("FrameRenderer dropped.");
    }
}
