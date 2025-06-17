// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// --- Standard Library Imports ---
use std::ffi::CString;
use std::sync::Arc;

// --- External Crate Imports ---
use anyhow::Result;
use ash::vk;
use thiserror::Error;
use vk_mem; // Ensure this is present if using vk_mem::Error directly

// --- Workspace Crate Imports ---
use crate::config::graphics_config::GraphicsConfig; // Adjusted path
use crate::renderer::allocator::GpuAllocator; // Assuming this is your allocator wrapper
use crate::renderer::shader_module::ShaderModule;
use crate::renderer::vulkan_context::VulkanContext; // Assuming this is your Vulkan context
use crate::renderer::vulkan_swapchain::VulkanSwapchain;

#[derive(Error, Debug)]
pub enum VulkanError {
    #[error("Vulkan instance creation failed: {0}")]
    InstanceCreation(ash::vk::Result),
    #[error("Failed to enumerate physical devices: {0}")]
    PhysicalDeviceEnumeration(ash::vk::Result),
    #[error("No suitable Vulkan physical device found")]
    NoSuitableDevice,
    #[error("Vulkan logical device creation failed: {0}")]
    DeviceCreation(ash::vk::Result),
    #[error("VMA allocator creation failed: {0}")]
    AllocatorCreation(vk_mem::Error),
    #[error("Vulkan command pool creation failed: {0}")]
    CommandPoolCreation(ash::vk::Result),
    #[error("VMA image creation failed: {0}")]
    ImageCreation(vk_mem::Error),
    #[error("Vulkan image view creation failed: {0}")]
    ImageViewCreation(ash::vk::Result),
    #[error("Vulkan sampler creation failed: {0}")]
    SamplerCreation(ash::vk::Result),
    #[error("Invalid Wayland buffer provided")]
    InvalidBuffer,
    #[error("VMA buffer creation failed: {0}")]
    BufferCreation(vk_mem::Error),
    #[error("Failed to begin command buffer: {0}")]
    CommandBufferBegin(ash::vk::Result),
    #[error("Failed to end command buffer: {0}")]
    CommandBufferEnd(ash::vk::Result),
    #[error("Failed to allocate command buffer: {0}")]
    CommandBufferAllocation(ash::vk::Result),
    #[error("Queue submit failed: {0}")]
    QueueSubmit(ash::vk::Result),
    #[error("Queue wait idle failed: {0}")]
    QueueWaitIdle(ash::vk::Result),
    #[error("Render pass creation failed: {0}")]
    RenderPassCreation(ash::vk::Result),
    #[error("Shader module creation failed: {0}")]
    ShaderModuleCreation(ash::vk::Result),
    #[error("Pipeline layout creation failed: {0}")]
    PipelineLayoutCreation(ash::vk::Result),
    #[error("Graphics pipeline creation failed: {0}")]
    GraphicsPipelineCreation(ash::vk::Result),
    #[error("Framebuffer creation failed: {0}")]
    FramebufferCreation(ash::vk::Result),
    #[error("Semaphore creation failed: {0}")]
    SemaphoreCreation(ash::vk::Result),
    #[error("Fence creation failed: {0}")]
    FenceCreation(ash::vk::Result),
    #[error("Swapchain image acquisition failed: {0}")]
    SwapchainImageAcquisition(ash::vk::Result),
    #[error("Swapchain queue presentation failed: {0}")]
    SwapchainQueuePresent(ash::vk::Result),
    #[error("Failed to map VMA memory: {0}")]
    MapMemory(vk_mem::Error),
    #[error("Failed to unmap VMA memory")]
    UnmapMemory, // Consider adding source error if available from vk_mem
    #[error("Failed to flush VMA allocation: {0}")]
    FlushAllocation(vk_mem::Error),
    #[error("Command buffer reset failed: {0}")]
    CommandBufferReset(ash::vk::Result),
    #[error("Waiting for fence failed: {0}")]
    WaitForFences(ash::vk::Result),
    #[error("Resetting fence failed: {0}")]
    ResetFences(ash::vk::Result),
    #[error("Device wait idle failed: {0}")]
    DeviceWaitIdle(ash::vk::Result),
    #[error("Descriptor set layout creation failed: {0}")]
    DescriptorSetLayoutCreation(ash::vk::Result),

    #[error("General Vulkan error: {0}")]
    VkError(#[from] ash::vk::Result),
    #[error("General VMA error: {0}")]
    VmaError(#[from] vk_mem::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("CString creation error: {0}")]
    NulError(#[from] std::ffi::NulError),
}

//ANCHOR [VERTEX_STRUCT] - Vertex Data Structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}

impl Vertex {
    fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex, position) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex, tex_coord) as u32),
        ]
    }
}

//ANCHOR [UNIFORM_BUFFER_OBJECT] - UBO Structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct UniformBufferObject {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    alpha: f32,
    _padding: [u8; 12],
}

pub struct VulkanFrameRenderer {
    vulkan_context: Arc<VulkanContext>,
    allocator: Arc<GpuAllocator>, // Using the wrapper
    swapchain: VulkanSwapchain,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    vertex_shader_module: ShaderModule,
    fragment_shader_module: ShaderModule,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
    command_pool: vk::CommandPool,
    // One command buffer per swapchain image
    command_buffers: Vec<vk::CommandBuffer>,
    // Synchronization objects for each frame in flight
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
    max_frames_in_flight: usize,
    vertex_buffer: vk::Buffer,
    vertex_buffer_allocation: vk_mem::Allocation,
}

impl VulkanFrameRenderer {
    pub fn new(
        graphics_config: &GraphicsConfig,
        vulkan_context: Arc<VulkanContext>,
        allocator: Arc<GpuAllocator>, // Pass the wrapper
        initial_swapchain: Option<VulkanSwapchain>, // Allow providing an existing swapchain
    ) -> Result<Self, VulkanError> {
        let device = vulkan_context.device();
        let max_frames_in_flight = graphics_config.max_frames_in_flight.unwrap_or(2);

        let swapchain = match initial_swapchain {
            Some(sc) => sc,
            None => VulkanSwapchain::new(graphics_config, Arc::clone(&vulkan_context), None)?,
        };

        let render_pass = Self::create_render_pass(device, swapchain.surface_format().format)?;
        let framebuffers = Self::create_framebuffers(
            device,
            &swapchain.image_views(),
            swapchain.extent(),
            render_pass,
        )?;

        // --- Shader Modules ---
        // Ensure your shader paths are correct or use include_bytes! if they are bundled
        let vertex_shader_module = ShaderModule::new(
            Arc::clone(device),
            include_bytes!("../../../assets/shaders/quad.vert.spv"),
        )?;
        let fragment_shader_module = ShaderModule::new(
            Arc::clone(device),
            include_bytes!("../../../assets/shaders/quad.frag.spv"),
        )?;

        // --- Graphics Pipeline ---
        let (pipeline_layout, graphics_pipeline) = Self::create_graphics_pipeline(
            device,
            swapchain.extent(),
            render_pass,
            vertex_shader_module.module(),
            fragment_shader_module.module(),
        )?;

        // --- Command Pool & Buffers ---
        let command_pool = Self::create_command_pool(
            device,
            vulkan_context.queue_families_indices().graphics_family_index(),
        )?;
        // Create a command buffer for each framebuffer/swapchain image
        let command_buffers = Self::create_command_buffers(
            device,
            command_pool,
            framebuffers.len() as u32,
        )?;

        // --- Synchronization Primitives ---
        let mut image_available_semaphores = Vec::with_capacity(max_frames_in_flight);
        let mut render_finished_semaphores = Vec::with_capacity(max_frames_in_flight);
        let mut in_flight_fences = Vec::with_capacity(max_frames_in_flight);

        for _ in 0..max_frames_in_flight {
            image_available_semaphores.push(Self::create_semaphore(device)?);
            render_finished_semaphores.push(Self::create_semaphore(device)?);
            in_flight_fences.push(Self::create_fence(device)?);
        }

        // --- Vertex Buffer ---
        let vertices = create_quad_vertices(); // Helper function to get quad vertices
        let (vertex_buffer, vertex_buffer_allocation) = Self::create_vertex_buffer(
            &allocator, // Pass the GpuAllocator wrapper
            device,
            vulkan_context.graphics_queue(),
            command_pool,
            &vertices,
        )?;


        Ok(Self {
            vulkan_context,
            allocator,
            swapchain,
            render_pass,
            framebuffers,
            vertex_shader_module,
            fragment_shader_module,
            pipeline_layout,
            graphics_pipeline,
            command_pool,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
            max_frames_in_flight,
            vertex_buffer,
            vertex_buffer_allocation,
        })
    }

    fn create_render_pass(
        device: &ash::Device, // Direct ash::Device
        surface_format: vk::Format,
    ) -> Result<vk::RenderPass, VulkanError> {
        let color_attachment = vk::AttachmentDescription::default()
            .format(surface_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref));

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty()) // No access needed before this
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        unsafe { device.create_render_pass(&render_pass_info, None) }
            .map_err(VulkanError::RenderPassCreation)
    }

    fn create_framebuffers(
        device: &ash::Device, // Direct ash::Device
        swapchain_image_views: &[vk::ImageView],
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
    ) -> Result<Vec<vk::Framebuffer>, VulkanError> {
        swapchain_image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view];
                let framebuffer_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(swapchain_extent.width)
                    .height(swapchain_extent.height)
                    .layers(1);
                unsafe { device.create_framebuffer(&framebuffer_info, None) }
                    .map_err(VulkanError::FramebufferCreation)
            })
            .collect()
    }

    fn create_graphics_pipeline(
        device: &ash::Device, // Direct ash::Device
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        vertex_shader_module: vk::ShaderModule,
        fragment_shader_module: vk::ShaderModule,
    ) -> Result<(vk::PipelineLayout, vk::Pipeline), VulkanError> {
        let main_function_name = CString::new("main").unwrap(); // Should not fail

        let vertex_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader_module)
            .name(&main_function_name);

        let fragment_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader_module)
            .name(&main_function_name);

        let shader_stages = [vertex_shader_stage_info, fragment_shader_stage_info];

        let binding_description = Vertex::get_binding_description();
        let attribute_descriptions = Vertex::get_attribute_descriptions();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
            .vertex_attribute_descriptions(&attribute_descriptions);


        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        };
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE) // Adjusted to match quad vertices winding
            .depth_bias_enable(false);

        let multisampling_info = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false); // No blending for simple quad

        let color_blending_info = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }
            .map_err(VulkanError::PipelineLayoutCreation)?;

        let graphics_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampling_info)
            .color_blend_state(&color_blending_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0); // Index of the subpass where this pipeline will be used

        let graphics_pipeline = unsafe {
            device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&graphics_pipeline_info),
                None,
            )
        }
        .map_err(|(_, result)| VulkanError::GraphicsPipelineCreation(result))?
        [0]; // We are creating a single pipeline

        Ok((pipeline_layout, graphics_pipeline))
    }

    fn create_command_pool(
        device: &ash::Device, // Direct ash::Device
        graphics_family_index: u32,
    ) -> Result<vk::CommandPool, VulkanError> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER) // Allow resetting individual command buffers
            .queue_family_index(graphics_family_index);
        unsafe { device.create_command_pool(&pool_info, None) }
            .map_err(VulkanError::CommandPoolCreation)
    }

    fn create_command_buffers(
        device: &ash::Device, // Direct ash::Device
        command_pool: vk::CommandPool,
        count: u32,
    ) -> Result<Vec<vk::CommandBuffer>, VulkanError> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);
        unsafe { device.allocate_command_buffers(&alloc_info) }
            .map_err(VulkanError::CommandBufferAllocation)
    }

    fn create_semaphore(device: &ash::Device) -> Result<vk::Semaphore, VulkanError> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        unsafe { device.create_semaphore(&semaphore_info, None) }.map_err(VulkanError::SemaphoreCreation)
    }

    fn create_fence(device: &ash::Device) -> Result<vk::Fence, VulkanError> {
        // Create fences in signaled state so wait_for_fences doesn't block indefinitely on first frame
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        unsafe { device.create_fence(&fence_info, None) }.map_err(VulkanError::FenceCreation)
    }

    fn create_vertex_buffer(
        allocator: &GpuAllocator, // Use the wrapper
        device: &ash::Device, // Direct ash::Device
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        vertices: &[Vertex],
    ) -> Result<(vk::Buffer, vk_mem::Allocation), VulkanError> {
        let buffer_size = (std::mem::size_of::<Vertex>() * vertices.len()) as vk::DeviceSize;

        // Create staging buffer
        let (staging_buffer, staging_allocation, staging_alloc_info) = allocator.create_buffer(
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuToGpu, // Or CpuOnly if preferred for staging
        )?;

        // Map and copy data to staging buffer
        let data_ptr = allocator.map_memory(&staging_allocation)? as *mut Vertex;
        unsafe {
            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
        }
        // Flush if not coherent, though CpuToGpu often is
        if staging_alloc_info.get_memory_type_index() != vk::MAX_MEMORY_TYPES as usize { // Basic check, better to check flags
             allocator.flush_allocation(&staging_allocation, 0, buffer_size)?;
        }
        allocator.unmap_memory(&staging_allocation)?;


        // Create vertex buffer (device local)
        let (vertex_buffer, vertex_allocation, _vertex_alloc_info) = allocator.create_buffer(
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk_mem::MemoryUsage::GpuOnly,
        )?;

        // Copy from staging to vertex buffer
        let command_buffer = Self::begin_single_time_commands(device, command_pool)?;
        let copy_region = vk::BufferCopy::default().size(buffer_size);
        unsafe {
            device.cmd_copy_buffer(
                command_buffer,
                staging_buffer,
                vertex_buffer,
                std::slice::from_ref(&copy_region),
            );
        }
        Self::end_single_time_commands(device, command_pool, graphics_queue, command_buffer)?;

        // Clean up staging buffer
        allocator.destroy_buffer(staging_buffer, &staging_allocation)?;

        Ok((vertex_buffer, vertex_allocation))
    }


    fn begin_single_time_commands(
        device: &ash::Device,
        command_pool: vk::CommandPool,
    ) -> Result<vk::CommandBuffer, VulkanError> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info)? }[0];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device.begin_command_buffer(command_buffer, &begin_info)? };
        Ok(command_buffer)
    }

    fn end_single_time_commands(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), VulkanError> {
        unsafe { device.end_command_buffer(command_buffer)? };

        let submit_info = vk::SubmitInfo::default()
            .command_buffers(std::slice::from_ref(&command_buffer));

        unsafe {
            device.queue_submit(queue, std::slice::from_ref(&submit_info), vk::Fence::null())?;
            device.queue_wait_idle(queue)?; // Wait for the transfer to complete
            device.free_command_buffers(command_pool, std::slice::from_ref(&command_buffer));
        }
        Ok(())
    }


    fn record_command_buffer(
        &self,
        command_buffer_index: usize, // Index for the command buffer to use
        image_index: u32,            // Index of the swapchain image
    ) -> Result<(), VulkanError> {
        let command_buffer = self.command_buffers[command_buffer_index];
        let device = self.vulkan_context.device();

        unsafe { device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())? };

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT); // Or empty if re-recorded often
        unsafe { device.begin_command_buffer(command_buffer, &begin_info)? };

        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0], // Black
            },
        };
        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D::default(),
                extent: self.swapchain.extent(),
            })
            .clear_values(std::slice::from_ref(&clear_color));

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );

            // Bind vertex buffer
            let vertex_buffers = [self.vertex_buffer];
            let offsets = [0];
            device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);

            // Draw call for the quad (6 vertices)
            device.cmd_draw(command_buffer, create_quad_vertices().len() as u32, 1, 0, 0);

            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer)?;
        }
        Ok(())
    }

    pub fn draw_frame(&mut self) -> Result<(), VulkanError> {
        let device = self.vulkan_context.device();
        let graphics_queue = self.vulkan_context.graphics_queue();
        let present_queue = self.vulkan_context.present_queue();

        let fence_to_wait = self.in_flight_fences[self.current_frame];
        unsafe { device.wait_for_fences(std::slice::from_ref(&fence_to_wait), true, u64::MAX)? };

        // Acquire an image from the swapchain
        let image_available_semaphore = self.image_available_semaphores[self.current_frame];
        let (image_index, _is_suboptimal) = match self.swapchain.acquire_next_image(image_available_semaphore) {
            Ok(val) => val,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                // TODO: Handle swapchain recreation
                // self.recreate_swapchain()?;
                return Ok(()); // Skip rendering this frame
            }
            Err(e) => return Err(VulkanError::SwapchainImageAcquisition(e)),
        };

        // Reset fence only if we are submitting work
        unsafe { device.reset_fences(std::slice::from_ref(&fence_to_wait))? };

        // Record the command buffer for the acquired image
        // Use current_frame to pick the command buffer, as we have one CB per frame in flight
        self.record_command_buffer(self.current_frame, image_index)?;

        // Submit the command buffer
        let wait_semaphores = [image_available_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let command_buffer_to_submit = self.command_buffers[self.current_frame];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(&command_buffer_to_submit))
            .signal_semaphores(&signal_semaphores);

        unsafe {
            device.queue_submit(graphics_queue, std::slice::from_ref(&submit_info), fence_to_wait)?;
        }

        // Present the image
        let result = self.swapchain.queue_present(
            present_queue,
            image_index,
            self.render_finished_semaphores[self.current_frame],
        );

        match result {
            Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                // TODO: Handle swapchain recreation
                // self.recreate_swapchain()?;
            }
            Err(e) => return Err(VulkanError::SwapchainQueuePresent(e)),
            _ => {} // Success
        }

        // Advance to the next frame
        self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;

        Ok(())
    }

    // TODO: Implement recreate_swapchain for window resizing
    // pub fn recreate_swapchain(&mut self, new_config: &GraphicsConfig) -> Result<(), VulkanError> { ... }

    pub fn wait_device_idle(&self) -> Result<(), VulkanError> {
        unsafe { self.vulkan_context.device().device_wait_idle()? };
        Ok(())
    }
}

impl Drop for VulkanFrameRenderer {
    fn drop(&mut self) {
        // It's crucial to wait for the device to be idle before destroying resources
        if let Err(e) = self.wait_device_idle() {
            // Use your logging solution, e.g., log::error! or eprintln!
            eprintln!("Error waiting for device idle during drop: {:?}", e);
            // Depending on the error, you might still want to proceed with cleanup
        }


        let device = self.vulkan_context.device();
        unsafe {
            // Destroy vertex buffer
            if self.vertex_buffer != vk::Buffer::null() {
                 // Ensure allocator is valid and buffer/allocation were properly initialized.
                if let Err(e) = self.allocator.destroy_buffer(self.vertex_buffer, &self.vertex_buffer_allocation) {
                     eprintln!("Error destroying vertex buffer: {:?}", e);
                }
            }


            for i in 0..self.max_frames_in_flight {
                device.destroy_semaphore(self.image_available_semaphores[i], None);
                device.destroy_semaphore(self.render_finished_semaphores[i], None);
                device.destroy_fence(self.in_flight_fences[i], None);
            }

            // Command buffers are freed when the pool is destroyed
            if self.command_pool != vk::CommandPool::null() {
                device.destroy_command_pool(self.command_pool, None);
            }

            for framebuffer in self.framebuffers.drain(..) {
                if framebuffer != vk::Framebuffer::null() {
                    device.destroy_framebuffer(framebuffer, None);
                }
            }

            if self.graphics_pipeline != vk::Pipeline::null() {
                device.destroy_pipeline(self.graphics_pipeline, None);
            }
            if self.pipeline_layout != vk::PipelineLayout::null() {
                device.destroy_pipeline_layout(self.pipeline_layout, None);
            }
            if self.render_pass != vk::RenderPass::null() {
                device.destroy_render_pass(self.render_pass, None);
            }

            // Shader modules are Arc'd, their drop will handle cleanup if they own the module.
            // If ShaderModule struct itself needs explicit cleanup, call it here.

            // Swapchain is also responsible for its own cleanup via its Drop impl.
        }
        // Note: VulkanContext and Allocator are Arc'd, so they will be dropped
        // when their reference count goes to zero. Ensure their Drop impls are correct.
    }
}


// Helper to create quad vertices (adjust UVs if needed for your texture)
fn create_quad_vertices() -> [Vertex; 6] {
    [
        Vertex { pos: [-0.5, -0.5], uv: [0.0, 0.0] }, // Bottom-left
        Vertex { pos: [ 0.5, -0.5], uv: [1.0, 0.0] }, // Bottom-right
        Vertex { pos: [ 0.5,  0.5], uv: [1.0, 1.0] }, // Top-right

        Vertex { pos: [-0.5, -0.5], uv: [0.0, 0.0] }, // Bottom-left
        Vertex { pos: [ 0.5,  0.5], uv: [1.0, 1.0] }, // Top-right
        Vertex { pos: [-0.5,  0.5], uv: [0.0, 1.0] }, // Top-left
    ]
}

// Helper for offset_of macro if not using an external crate for it
#[macro_export]
macro_rules! offset_of {
    ($struct:path, $field:ident) => {{
        // Using a null pointer to calculate offset is not ideal in const context
        // but works for struct layouts in practice.
        // Consider `memoffset::offset_of` for a more robust solution.
        let dummy = std::mem::MaybeUninit::<$struct>::uninit();
        let dummy_ptr = dummy.as_ptr();
        let field_ptr = unsafe { &(*dummy_ptr).$field as *const _ };
        (field_ptr as usize) - (dummy_ptr as usize)
    }};
}
