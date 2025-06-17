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

use std::sync::Arc;

use ash::vk;
use thiserror::Error;
use vk_mem;
use gpu_allocator::vulkan::Allocator;

use crate::graphics_config::GraphicsConfig;
use crate::renderer::allocator::GpuAllocator;
use crate::renderer::shader_module::ShaderModule;
use crate::renderer::vulkan_context::VulkanContext;
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
    ImageCreation(vk_mem::Error), // Used by VulkanTexture
    #[error("Vulkan image view creation failed: {0}")]
    ImageViewCreation(ash::vk::Result), // Used by VulkanTexture
    #[error("Vulkan sampler creation failed: {0}")]
    SamplerCreation(ash::vk::Result), // Used by VulkanTexture
    #[error("Invalid Wayland buffer provided")]
    InvalidBuffer, // Used by VulkanTexture
    #[error("VMA buffer creation failed: {0}")]
    BufferCreation(vk_mem::Error), // Used by VulkanTexture & FrameRenderer
    #[error("Failed to begin command buffer: {0}")]
    CommandBufferBegin(ash::vk::Result), // Used by VulkanTexture
    #[error("Failed to end command buffer: {0}")]
    CommandBufferEnd(ash::vk::Result), // Used by VulkanTexture
    #[error("Failed to allocate command buffer: {0}")]
    CommandBufferAllocation(ash::vk::Result), // Used by VulkanTexture & FrameRenderer
    #[error("Queue submit failed: {0}")]
    QueueSubmit(ash::vk::Result), // Used by VulkanTexture
    #[error("Queue wait idle failed: {0}")]
    QueueWaitIdle(ash::vk::Result), // Used by VulkanTexture
    #[error("Render pass creation failed: {0}")]
    RenderPassCreation(ash::vk::Result), // Used by FrameRenderer
    #[error("Shader module creation failed: {0}")]
    ShaderModuleCreation(ash::vk::Result), // Implied by FrameRenderer
    #[error("Pipeline layout creation failed: {0}")]
    PipelineLayoutCreation(ash::vk::Result), // Implied by FrameRenderer
    #[error("Graphics pipeline creation failed: {0}")]
    GraphicsPipelineCreation(ash::vk::Result), // Implied by FrameRenderer
    #[error("Descriptor pool creation failed: {0}")]
    DescriptorPoolCreation(ash::vk::Result), // Implied by FrameRenderer
    #[error("General Vulkan error: {0}")]
    VkError(#[from] ash::vk::Result),
    #[error("General VMA error: {0}")]
    VmaError(#[from] vk_mem::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error), // For shader loading via include_bytes!, though less common to map this way
    #[error("CString creation error: {0}")]
    NulError(#[from] std::ffi::NulError), // For CString::new errors
}

pub struct VulkanFrameRenderer {
    vulkan_context: Arc<VulkanContext>,
    allocator: Arc<GpuAllocator>,
    swapchain: VulkanSwapchain,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    vertex_shader_module: ShaderModule,
    fragment_shader_module: ShaderModule,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
}

impl VulkanFrameRenderer {
    pub fn new(
        graphics_config: &GraphicsConfig,
        vulkan_context: Arc<VulkanContext>,
        allocator: Arc<GpuAllocator>,
    ) -> anyhow::Result<Self> {
        let swapchain = VulkanSwapchain::new(
            graphics_config,
            Arc::clone(&vulkan_context),
            Option::None,
        )?;
        let render_pass = Self::create_render_pass(vulkan_context.device(), swapchain.surface_format().format)?;
        let framebuffers = Self::create_framebuffers(
            vulkan_context.device(),
            &swapchain.image_views(),
            swapchain.extent(),
            render_pass,
        )?;

        let vertex_shader_module = ShaderModule::new(
            Arc::clone(&vulkan_context.device()),
            &include_bytes!("../../../assets/shaders/quad.vert.spv")[..],
        )?;
        let fragment_shader_module = ShaderModule::new(
            Arc::clone(&vulkan_context.device()),
            &include_bytes!("../../../assets/shaders/quad.frag.spv")[..],
        )?;
        let (pipeline_layout, graphics_pipeline) = Self::create_graphics_pipeline(
            vulkan_context.device(),
            swapchain.extent(),
            render_pass,
            vertex_shader_module.module(),
            fragment_shader_module.module(),
        )?;

        let command_pool = Self::create_command_pool(
            vulkan_context.device(),
            vulkan_context.queue_families_indices().graphics_family_index(),
        )?;
        let command_buffers = Self::create_command_buffers(
            vulkan_context.device(),
            command_pool,
            framebuffers.len() as u32,
        )?;

        let image_available_semaphore = Self::create_semaphore(vulkan_context.device())?;
        let render_finished_semaphore = Self::create_semaphore(vulkan_context.device())?;
        let in_flight_fence = Self::create_fence(vulkan_context.device())?;

        let renderer = Self {
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
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        };
        Ok(renderer)
    }

    fn create_render_pass(
        device: &Arc<ash::Device>,
        surface_format: vk::Format,
    ) -> anyhow::Result<vk::RenderPass> {
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
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));
        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None)? };
        Ok(render_pass)
    }

    fn create_framebuffers(
        device: &Arc<ash::Device>,
        swapchain_image_views: &[vk::ImageView],
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
    ) -> anyhow::Result<Vec<vk::Framebuffer>> {
        let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());
        for image_view in swapchain_image_views {
            let attachments = [*image_view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1);
            let framebuffer = unsafe { device.create_framebuffer(&framebuffer_info, None)? };
            framebuffers.push(framebuffer);
        }
        Ok(framebuffers)
    }

    fn create_graphics_pipeline(
        device: &Arc<ash::Device>,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        vertex_shader_module: vk::ShaderModule,
        fragment_shader_module: vk::ShaderModule,
    ) -> anyhow::Result<(vk::PipelineLayout, vk::Pipeline)> {
        let main_function_name = std::ffi::CString::new("main").unwrap();
        let vertex_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader_module)
            .name(&main_function_name);
        let fragment_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader_module)
            .name(&main_function_name);
        let shader_stages = [vertex_shader_stage_info, fragment_shader_stage_info];

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&[])
            .vertex_attribute_descriptions(&[]);
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);
        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D::default())
            .extent(swapchain_extent);
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling_info = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);
        let color_blending_info = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(std::slice::from_ref(&color_blend_attachment))
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        let graphics_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampling_info)
            .color_blend_state(&color_blending_info)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);
        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&graphics_pipeline_info),
                    None,
                )
                .map_err(|(_pipelines, result)| result)?
        }[0];

        Ok((pipeline_layout, graphics_pipeline))
    }

    fn create_command_pool(
        device: &Arc<ash::Device>,
        graphics_family_index: u32,
    ) -> anyhow::Result<vk::CommandPool> {
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_family_index);
        let command_pool = unsafe { device.create_command_pool(&command_pool_info, None)? };
        Ok(command_pool)
    }

    fn create_command_buffers(
        device: &Arc<ash::Device>,
        command_pool: vk::CommandPool,
        frame_count: u32,
    ) -> anyhow::Result<Vec<vk::CommandBuffer>> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(frame_count);
        let command_buffers =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)? };
        Ok(command_buffers)
    }

    fn create_semaphore(device: &Arc<ash::Device>) -> anyhow::Result<vk::Semaphore> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let semaphore = unsafe { device.create_semaphore(&semaphore_info, None)? };
        Ok(semaphore)
    }

    fn create_fence(device: &Arc<ash::Device>) -> anyhow::Result<vk::Fence> {
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.create_fence(&fence_info, None)? };
        Ok(fence)
    }

    fn record_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        image_index: u32,
    ) -> anyhow::Result<()> {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            self.vulkan_context
                .device()
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)?;
        }

        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };
        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent(),
            })
            .clear_values(std::slice::from_ref(&clear_color));
        unsafe {
            self.vulkan_context.device().cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            self.vulkan_context
                .device()
                .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline);
            self.vulkan_context.device().cmd_draw(command_buffer, 3, 1, 0, 0);
            self.vulkan_context.device().cmd_end_render_pass(command_buffer);
            self.vulkan_context.device().end_command_buffer(command_buffer)?;
        }
        Ok(())
    }

    pub fn draw_frame(&mut self) -> anyhow::Result<()> {
        let device = self.vulkan_context.device();
        let graphics_queue = self.vulkan_context.graphics_queue();
        let present_queue = self.vulkan_context.present_queue();
        unsafe {
            device.wait_for_fences(std::slice::from_ref(&self.in_flight_fence), true, u64::MAX)?;
            device.reset_fences(std::slice::from_ref(&self.in_flight_fence))?;
        }
        let result = self.swapchain.acquire_next_image(self.image_available_semaphore);
        let image_index = match result {
            Ok((image_index, _)) => image_index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                // TODO: recreate swapchain
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };

        let command_buffer = self.command_buffers[image_index as usize];
        unsafe {
            device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?;
        }
        self.record_command_buffer(command_buffer, image_index)?;

        let wait_semaphores = [self.image_available_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphore];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(&command_buffer))
            .signal_semaphores(&signal_semaphores);
        unsafe {
            device.queue_submit(
                graphics_queue,
                std::slice::from_ref(&submit_info),
                self.in_flight_fence,
            )?;
        }

        let result = self.swapchain.queue_present(present_queue, image_index, self.render_finished_semaphore);
        match result {
            Ok(suboptimal) if suboptimal => {
                // TODO: recreate swapchain
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                // TODO: recreate swapchain
                return Ok(());
            }
            Err(e) => return Err(e.into()),
            _ => {}
        }

        Ok(())
    }
}

impl Drop for VulkanFrameRenderer {
    fn drop(&mut self) {
        let device = self.vulkan_context.device();
        unsafe {
            device.device_wait_idle().expect("Failed to wait device idle");
            device.destroy_semaphore(self.image_available_semaphore, None);
            device.destroy_semaphore(self.render_finished_semaphore, None);
            device.destroy_fence(self.in_flight_fence, None);
            device.destroy_command_pool(self.command_pool, None);
            for framebuffer in self.framebuffers.drain(..) {
                device.destroy_framebuffer(framebuffer, None);
            }
            device.destroy_pipeline(self.graphics_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_render_pass(self.render_pass, None);
        }
    }
}
