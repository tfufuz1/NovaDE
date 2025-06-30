// novade-system/src/renderers/vulkan/mod.rs
// Main module for the Vulkan rendering backend.

pub mod instance;
pub mod device;
pub mod surface;
pub mod swapchain;
pub mod memory;
pub mod dmabuf;
pub mod texture_pipeline;
pub mod command_buffer;
pub mod render_pass;
pub mod framebuffer;
pub mod shader;
pub mod pipeline;
pub mod sync;
pub mod renderer;
pub mod buffer;
pub mod image;

pub use self::context::VulkanContext;
pub use self::swapchain::VulkanSwapchain;
pub use self::render_pass::VulkanRenderPass;
pub use self::framebuffer::VulkanFramebuffer;
pub use self::shader::VulkanShaderModule;
pub use self::pipeline::VulkanPipeline;
pub use self::sync::FrameSyncPrimitives;
pub use self.renderer::NovaVulkanRenderer;
pub use self::buffer::VulkanBuffer;
pub use self::image::VulkanImage;

use crate::renderer_interface::RendererInterface;
use novade_core::types::geometry::Size2D;
use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use gpu_allocator::vulkan::Allocator;
use gpu_allocator::MemoryLocation;
use texture_pipeline::{TextureId, GpuTexture};
use std::collections::HashMap;
use std::sync::Arc;
use std::os::unix::io::RawFd;
use super::dmabuf::{self, DmaBufImportOptions};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct UniformBufferObject {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

impl Default for UniformBufferObject {
    fn default() -> Self {
        Self {
            model: glam::Mat4::IDENTITY.to_cols_array_2d(),
            view: glam::Mat4::IDENTITY.to_cols_array_2d(),
            proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

pub struct VulkanRenderer {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_utils: Option<ash::extensions::ext::DebugUtils>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,

    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    compute_queue: Option<vk::Queue>,
    physical_device_properties: vk::PhysicalDeviceProperties, // Storing for timestamp period

    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,

    allocator: Allocator,

    render_command_pool: vk::CommandPool,
    render_command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame_index: usize,

    render_pass: vk::RenderPass,

    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,

    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: Option<gpu_allocator::vulkan::Allocation>,
    index_buffer: vk::Buffer,
    index_buffer_memory: Option<gpu_allocator::vulkan::Allocation>,
    index_count: u32,

    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<Option<gpu_allocator::vulkan::Allocation>>,
    descriptor_sets: Vec<vk::DescriptorSet>,

    textures: HashMap<TextureId, GpuTexture>,
    default_texture_id: Option<TextureId>,

    compute_command_pool: Option<vk::CommandPool>,
    compute_command_buffer: Option<vk::CommandBuffer>,
    compute_descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    compute_pipeline_layout: Option<vk::PipelineLayout>,
    compute_pipeline: Option<vk::Pipeline>,
    compute_descriptor_set: Option<vk::DescriptorSet>,
    compute_storage_image: Option<GpuTexture>,
    compute_fence: Option<vk::Fence>,

    // --- WP-06 Timestamp Queries ---
    timestamp_query_pool: Option<vk::QueryPool>,
    // We'll use 2 queries per frame in flight for simple duration: start and end.
    // So, query_count = MAX_FRAMES_IN_FLIGHT * 2
    // --- End WP-06 ---
}

const MAX_FRAMES_IN_FLIGHT: usize = 2;
const TIMESTAMP_QUERY_COUNT: u32 = 2; // For start and end of frame rendering

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SimpleVertex {
    pos: [f32; 2],
    #[allow(dead_code)]
    color: [f32; 3],
    tex_coord: [f32; 2],
}

impl SimpleVertex {
    fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription::builder()
                .binding(0).location(0) .format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(SimpleVertex, pos) as u32).build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0).location(1).format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(SimpleVertex, tex_coord) as u32).build(),
        ]
    }
}


impl VulkanRenderer {
    pub fn new(
        raw_display_handle_provider: &impl HasRawDisplayHandle,
        raw_window_handle_provider: &impl HasRawWindowHandle,
        window_width: u32,
        window_height: u32
    ) -> Result<Self, String> {
        let entry = instance::create_entry()?;
        let (instance_ash, debug_utils, debug_messenger) = instance::create_instance(&entry)?;

        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance_ash);
        let surface = surface::create_surface(&entry, &instance_ash, raw_display_handle_provider, raw_window_handle_provider)?;

        let chosen_physical_device = device::pick_physical_device(&instance_ash, &surface_loader, surface)?;
        let physical_device_properties = chosen_physical_device.properties.clone(); // Store for timestamp period

        let (logical_device_ash, queues) = device::create_logical_device(&instance_ash, &chosen_physical_device)?;
        let graphics_queue = queues.graphics_queue;
        let present_queue = queues.present_queue;
        let compute_queue = queues.compute_queue;

        let (swapchain_loader, swapchain, swapchain_format, swapchain_extent, swapchain_images, swapchain_image_views) =
            swapchain::create_swapchain_direct(
                &instance_ash, &logical_device_ash, chosen_physical_device.physical_device, &surface_loader, surface,
                chosen_physical_device.queue_family_indices.graphics_family.ok_or("Graphics family index missing")?,
                chosen_physical_device.queue_family_indices.present_family.ok_or("Present family index missing")?,
                window_width, window_height, None
            )?;

        let mut allocator = memory::create_allocator(&instance_ash, chosen_physical_device.physical_device, &logical_device_ash)?;

        let vertices = [
            SimpleVertex { pos: [-0.5, -0.5], color: [1.0,0.0,0.0], tex_coord: [1.0, 0.0] },
            SimpleVertex { pos: [ 0.5, -0.5], color: [0.0,1.0,0.0], tex_coord: [0.0, 0.0] },
            SimpleVertex { pos: [ 0.5,  0.5], color: [0.0,0.0,1.0], tex_coord: [0.0, 1.0] },
            SimpleVertex { pos: [-0.5,  0.5], color: [1.0,1.0,0.0], tex_coord: [1.0, 1.0] },
        ];
        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let index_count = indices.len() as u32;

        let vertex_buffer_size = (std::mem::size_of_val(&vertices)) as vk::DeviceSize;
        let (vertex_buffer, mut vertex_buffer_memory_alloc) = memory::create_buffer(
            &mut allocator, &logical_device_ash, vertex_buffer_size,
            vk::BufferUsageFlags::VERTEX_BUFFER, MemoryLocation::CpuToGpu,
        )?;
        if let Some(slice) = vertex_buffer_memory_alloc.mapped_slice_mut() {
            let data_ptr = vertices.as_ptr() as *const u8;
            let data_slice = unsafe { std::slice::from_raw_parts(data_ptr, vertex_buffer_size as usize) };
            slice[..vertex_buffer_size as usize].copy_from_slice(data_slice);
        } else {
             unsafe { logical_device_ash.destroy_buffer(vertex_buffer, None); }
            return Err("Failed to map vertex buffer memory".to_string());
        }

        let index_buffer_size = (std::mem::size_of_val(&indices)) as vk::DeviceSize;
        let (index_buffer, mut index_buffer_memory_alloc) = memory::create_buffer(
            &mut allocator, &logical_device_ash, index_buffer_size,
            vk::BufferUsageFlags::INDEX_BUFFER, MemoryLocation::CpuToGpu,
        )?;
        if let Some(slice) = index_buffer_memory_alloc.mapped_slice_mut() {
            let data_ptr = indices.as_ptr() as *const u8;
            let data_slice = unsafe { std::slice::from_raw_parts(data_ptr, index_buffer_size as usize) };
            slice[..index_buffer_size as usize].copy_from_slice(data_slice);
        } else {
            unsafe {
                logical_device_ash.destroy_buffer(vertex_buffer, None); allocator.free(vertex_buffer_memory_alloc).ok();
                logical_device_ash.destroy_buffer(index_buffer, None);
            }
            return Err("Failed to map index buffer memory".to_string());
        }

        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);
        let sampler_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let bindings = [ubo_layout_binding.build(), sampler_layout_binding.build()];
        let dsl_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
        let descriptor_set_layout = unsafe { logical_device_ash.create_descriptor_set_layout(&dsl_create_info, None) }?;

        let render_pass = render_pass::create_vulkan_render_pass(&logical_device_ash, swapchain_format)?;
        let swapchain_framebuffers = swapchain_image_views.iter().map(|&view| {
            framebuffer::create_vulkan_framebuffer(&logical_device_ash, render_pass, view, swapchain_extent)
        }).collect::<Result<Vec<_>,_>>()?;

        let set_layouts_for_pipeline = [descriptor_set_layout];
        let pipeline_layout = pipeline::create_vulkan_pipeline_layout(&logical_device_ash, &set_layouts_for_pipeline, &[])?;

        let logical_device_arc_for_shaders = Arc::new(logical_device_ash.clone());
        let vert_shader_module = shader::VulkanShaderModule::new_from_file(
            logical_device_arc_for_shaders.clone(), "assets/shaders/textured_geometry.vert.spv"
        )?;
        let frag_shader_module = shader::VulkanShaderModule::new_from_file(
            logical_device_arc_for_shaders.clone(), "assets/shaders/textured_geometry.frag.spv"
        )?;

        let logical_device_arc_for_pipeline = Arc::new(logical_device_ash.clone());
        let binding_description_array = [SimpleVertex::get_binding_description()];
        let attribute_descriptions_array = SimpleVertex::get_attribute_descriptions();

        let graphics_pipeline_struct = pipeline::VulkanPipeline::new_with_vertex_input(
            logical_device_arc_for_pipeline.clone(), render_pass, swapchain_extent,
            &vert_shader_module, &frag_shader_module, pipeline_layout,
            &binding_description_array, &attribute_descriptions_array,
        )?;
        let graphics_pipeline = graphics_pipeline_struct.pipeline;

        let mut uniform_buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut uniform_buffers_memory = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let (buffer, alloc) = memory::create_buffer(
                &mut allocator, &logical_device_ash, std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize,
                vk::BufferUsageFlags::UNIFORM_BUFFER, MemoryLocation::CpuToGpu,
            )?;
            uniform_buffers.push(buffer);
            uniform_buffers_memory.push(Some(alloc));
        }

        let pool_sizes = [
            vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32).build(),
            vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32).build(),
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder().pool_sizes(&pool_sizes).max_sets(MAX_FRAMES_IN_FLIGHT as u32).flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
        let descriptor_pool = unsafe { logical_device_ash.create_descriptor_pool(&descriptor_pool_info, None) }?;

        let d_set_layouts_vec = vec![descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&d_set_layouts_vec);
        let descriptor_sets = unsafe { logical_device_ash.allocate_descriptor_sets(&descriptor_set_alloc_info) }?;

        // Command Pool must be created before texture upload if it uses it.
        let q_family_graphics = chosen_physical_device.queue_family_indices.graphics_family.ok_or("Graphics family index missing for command pool")?;
        let pool_create_info = vk::CommandPoolCreateInfo::builder().flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER).queue_family_index(q_family_graphics);
        let render_command_pool = unsafe { logical_device_ash.create_command_pool(&pool_create_info, None) }?;

        let mut textures_map = HashMap::new();
        let default_texture_id;
        let default_pixel_data: [u8; 4] = [255, 0, 255, 255];
        let default_gpu_texture = texture_pipeline::upload_shm_buffer_to_texture(
            &logical_device_ash, &mut allocator, graphics_queue,
            render_command_pool, // Now render_command_pool is initialized
            &default_pixel_data, 1, 1, vk::Format::R8G8B8A8_UNORM, false,
            &pdevice_properties, true,
        ).map_err(|e| format!("Failed to create default 1x1 texture: {}", e))?;
        default_texture_id = Some(default_gpu_texture.id);
        textures_map.insert(default_gpu_texture.id, default_gpu_texture);

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let ubo_buffer_info = vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i]).offset(0).range(std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize);
            let bound_texture = textures_map.get(&default_texture_id.unwrap()).ok_or("Default texture not found")?;
            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(bound_texture.image_view).sampler(bound_texture.sampler);
            let descriptor_writes = [
                vk::WriteDescriptorSet::builder().dst_set(descriptor_sets[i]).dst_binding(0).dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).buffer_info(std::slice::from_ref(&ubo_buffer_info)).build(),
                vk::WriteDescriptorSet::builder().dst_set(descriptor_sets[i]).dst_binding(1).dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&image_info)).build(),
            ];
            unsafe { logical_device_ash.update_descriptor_sets(&descriptor_writes, &[]) };
        }

        let cmd_buffer_alloc_info = vk::CommandBufferAllocateInfo::builder().command_pool(render_command_pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
        let render_command_buffers = unsafe { logical_device_ash.allocate_command_buffers(&cmd_buffer_alloc_info) }?;

        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores.push(unsafe { logical_device_ash.create_semaphore(&semaphore_info, None)? });
            render_finished_semaphores.push(unsafe { logical_device_ash.create_semaphore(&semaphore_info, None)? });
            in_flight_fences.push(unsafe { logical_device_ash.create_fence(&fence_info, None)? });
        }

        // --- WP-06 Timestamp Query Pool ---
        let timestamp_query_pool = if physical_device_properties.limits.timestamp_compute_and_graphics {
            let query_pool_info = vk::QueryPoolCreateInfo::builder()
                .query_type(vk::QueryType::TIMESTAMP)
                .query_count(TIMESTAMP_QUERY_COUNT * MAX_FRAMES_IN_FLIGHT as u32); // 2 timestamps per frame in flight
            Some(unsafe { logical_device_ash.create_query_pool(&query_pool_info, None)? })
        } else {
            eprintln!("Warning: Timestamps not supported on graphics/compute queues by this device.");
            None
        };
        // --- End WP-06 ---

        let mut compute_setup = VulkanRenderer::setup_compute_specific_resources(
            &logical_device_ash, &mut allocator, &pdevice_properties,
            chosen_physical_device.queue_family_indices.compute_family.unwrap_or(q_family_graphics),
            descriptor_pool // Pass the main descriptor pool for compute sets for now
        )?;

        println!("Vulkan Renderer initialized with vertex/index buffers and UBOs for textured geometry rendering.");
        Ok(VulkanRenderer {
            entry, instance: instance_ash, debug_utils, debug_messenger,
            surface_loader, surface, physical_device: chosen_physical_device.physical_device,
            device: logical_device_ash, graphics_queue, present_queue, compute_queue,
            physical_device_properties, // Stored
            swapchain_loader, swapchain, swapchain_images, swapchain_image_views,
            swapchain_format, swapchain_extent, allocator,
            render_command_pool, render_command_buffers,
            image_available_semaphores, render_finished_semaphores, in_flight_fences,
            current_frame_index: 0, render_pass,
            descriptor_set_layout, descriptor_pool,
            pipeline_layout, graphics_pipeline, swapchain_framebuffers,
            vertex_buffer, vertex_buffer_memory: Some(vertex_buffer_memory_alloc),
            index_buffer, index_buffer_memory: Some(index_buffer_memory_alloc),
            index_count,
            uniform_buffers, uniform_buffers_memory, descriptor_sets,
            textures: textures_map, default_texture_id,
            compute_command_pool: compute_setup.command_pool,
            compute_command_buffer: compute_setup.command_buffer,
            compute_descriptor_set_layout: Some(compute_setup.descriptor_set_layout),
            compute_pipeline_layout: Some(compute_setup.pipeline_layout),
            compute_pipeline: Some(compute_setup.pipeline),
            compute_descriptor_set: Some(compute_setup.descriptor_set),
            compute_storage_image: compute_setup.storage_image,
            compute_fence: Some(compute_setup.fence),
            timestamp_query_pool,
        })
    }

    struct ComputeResources { // Renamed from ComputeVulkanRendererResources
        command_pool: Option<vk::CommandPool>,
        command_buffer: Option<vk::CommandBuffer>,
        descriptor_set_layout: vk::DescriptorSetLayout,
        pipeline_layout: vk::PipelineLayout,
        pipeline: vk::Pipeline,
        descriptor_set: vk::DescriptorSet,
        storage_image: Option<GpuTexture>,
        fence: vk::Fence,
    }

    fn setup_compute_specific_resources(
        device: &ash::Device,
        allocator: &mut Allocator,
        pdevice_properties: &vk::PhysicalDeviceProperties,
        compute_q_family_idx: u32,
        graphics_descriptor_pool: vk::DescriptorPool, // To allocate compute descriptor set
    ) -> Result<ComputeResources, String> {
        let compute_pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(compute_q_family_idx);
        let compute_command_pool = unsafe { device.create_command_pool(&compute_pool_info, None)? };

        let compute_cb_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(compute_command_pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(1);
        let compute_command_buffer = unsafe { device.allocate_command_buffers(&compute_cb_alloc_info)?[0] };

        let storage_image_extent = vk::Extent2D { width: 256, height: 256 };
        let storage_image_format = vk::Format::R8G8B8A8_UNORM;
        let storage_image_usage = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST; // Added TRANSFER_DST for initial clear/write

        // Create a dummy GpuQueue for texture_pipeline function
        // This is a simplification. Ideally, texture_pipeline wouldn't need a full GpuQueue.
        let dummy_compute_queue_for_texture_upload = unsafe { device.get_device_queue(compute_q_family_idx, 0) };

        let compute_storage_image_tex = texture_pipeline::upload_shm_buffer_to_texture(
            device, allocator, dummy_compute_queue_for_texture_upload,
            compute_command_pool, &vec![0u8; (storage_image_extent.width * storage_image_extent.height * 4) as usize],
            storage_image_extent.width, storage_image_extent.height, storage_image_format, false,
            pdevice_properties, false
        ).map_err(|e| format!("Failed to create compute storage image: {}", e))?;

        let storage_image_dsl_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0).descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE);
        let compute_dsl_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(std::slice::from_ref(&storage_image_dsl_binding));
        let compute_dsl = unsafe { device.create_descriptor_set_layout(&compute_dsl_info, None)? };

        let compute_pipeline_layout = pipeline::create_vulkan_pipeline_layout(device, &[compute_dsl], &[])?;

        let compute_shader_module = shader::VulkanShaderModule::new_from_file(
            Arc::new(device.clone()), "assets/shaders/compute_shader.comp.spv"
        )?;

        let compute_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE).module(compute_shader_module.handle())
            .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap());
        let compute_pipeline_info = vk::ComputePipelineCreateInfo::builder()
            .stage(compute_stage_info.build()).layout(compute_pipeline_layout);
        let compute_pipeline = unsafe {
            device.create_compute_pipelines(vk::PipelineCache::null(), &[compute_pipeline_info.build()], None)
                .map_err(|(_, err)| format!("Failed to create compute pipeline: {:?}", err))?[0]
        };

        let compute_d_set_layouts = [compute_dsl];
        let compute_d_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(graphics_descriptor_pool) // Use the main graphics pool for this single set
            .set_layouts(&compute_d_set_layouts);
        let compute_descriptor_set = unsafe { device.allocate_descriptor_sets(&compute_d_set_alloc_info)?[0] };

        let storage_image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(compute_storage_image_tex.image_view);
        let compute_write = vk::WriteDescriptorSet::builder()
            .dst_set(compute_descriptor_set).dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(std::slice::from_ref(&storage_image_info));
        unsafe { device.update_descriptor_sets(&[compute_write.build()], &[]) };

        let compute_fence_info = vk::FenceCreateInfo::builder();
        let compute_fence = unsafe { device.create_fence(&compute_fence_info, None)? };

        Ok(ComputeResources {
            command_pool: Some(compute_command_pool),
            command_buffer: Some(compute_command_buffer),
            descriptor_set_layout: compute_dsl,
            pipeline_layout: compute_pipeline_layout,
            pipeline: compute_pipeline,
            descriptor_set: compute_descriptor_set,
            storage_image: Some(compute_storage_image_tex),
            fence: compute_fence,
        })
    }


    fn record_main_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        clear_color_value: [f32; 4],
        current_frame_idx: usize,
    ) -> Result<(), String> {
        unsafe {
            self.device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?;
            let begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            self.device.begin_command_buffer(command_buffer, &begin_info)?;

            // --- WP-06 Timestamp Start ---
            if let Some(pool) = self.timestamp_query_pool {
                 // Reset queries for the current frame in flight. Query indices are (current_frame_idx * 2) and (current_frame_idx * 2 + 1)
                self.device.cmd_reset_query_pool(command_buffer, pool, (current_frame_idx * TIMESTAMP_QUERY_COUNT as usize) as u32, TIMESTAMP_QUERY_COUNT);
                self.device.cmd_write_timestamp(command_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, pool, (current_frame_idx * TIMESTAMP_QUERY_COUNT as usize) as u32);
            }
            // --- End WP-06 ---

            let clear_values = [vk::ClearValue { color: vk::ClearColorValue { float32: clear_color_value } }];
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.render_pass).framebuffer(framebuffer)
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain_extent, })
                .clear_values(&clear_values);

            self.device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline);

            let viewport = vk::Viewport {
                x: 0.0, y: 0.0, width: self.swapchain_extent.width as f32, height: self.swapchain_extent.height as f32,
                min_depth: 0.0, max_depth: 1.0,
            };
            self.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            let scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain_extent };
            self.device.cmd_set_scissor(command_buffer, 0, &[scissor]);

            let vertex_buffers = [self.vertex_buffer];
            let offsets = [0_u64];
            self.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            self.device.cmd_bind_index_buffer(command_buffer, self.index_buffer, 0, vk::IndexType::UINT16);
            self.device.cmd_bind_descriptor_sets(
                command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0,
                &[self.descriptor_sets[current_frame_idx]], &[],
            );
            self.device.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0);
            self.device.cmd_end_render_pass(command_buffer);

            // --- WP-06 Timestamp End ---
            if let Some(pool) = self.timestamp_query_pool {
                self.device.cmd_write_timestamp(command_buffer, vk::PipelineStageFlags::BOTTOM_OF_PIPE, pool, (current_frame_idx * TIMESTAMP_QUERY_COUNT as usize + 1) as u32);
            }
            // --- End WP-06 ---

            self.device.end_command_buffer(command_buffer)?;
        }
        Ok(())
    }

    fn update_uniform_buffer(&mut self, current_image_index: usize) -> Result<(), String> {
        let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f32();
        let model = glam::Mat4::from_rotation_z(time * std::f32::consts::FRAC_PI_2 * 0.2);
        let view = glam::Mat4::look_at_rh(glam::vec3(0.0, 0.0, 1.5), glam::Vec3::ZERO, glam::Vec3::Y);
        let mut proj = glam::Mat4::perspective_rh_gl(
            60.0_f32.to_radians(), self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32, 0.1, 10.0
        );
        proj.y_axis.y *= -1.0;
        let ubo = UniformBufferObject { model: model.to_cols_array_2d(), view: view.to_cols_array_2d(), proj: proj.to_cols_array_2d() };

        if let Some(Some(alloc)) = self.uniform_buffers_memory.get_mut(current_image_index) {
            if let Some(mapped_slice) = alloc.mapped_slice_mut() {
                let ubo_size = std::mem::size_of::<UniformBufferObject>();
                let ubo_ptr = &ubo as *const UniformBufferObject as *const u8;
                let ubo_data_slice = unsafe { std::slice::from_raw_parts(ubo_ptr, ubo_size) };
                mapped_slice[..ubo_size].copy_from_slice(ubo_data_slice);
            } else { return Err(format!("Failed to map uniform buffer memory for frame index {}", current_image_index)); }
        } else { return Err(format!("Uniform buffer memory not found for frame index {}", current_image_index)); }
        Ok(())
    }

    pub fn draw_frame(&mut self, clear_color: [f32; 4]) -> Result<bool, String> {
        let current_frame_sync_idx = self.current_frame_index;
        let fence = self.in_flight_fences[current_frame_sync_idx];
        let image_available_semaphore = self.image_available_semaphores[current_frame_sync_idx];
        let render_finished_semaphore = self.render_finished_semaphores[current_frame_sync_idx];
        let command_buffer = self.render_command_buffers[current_frame_sync_idx];

        unsafe { self.device.wait_for_fences(&[fence], true, u64::MAX)? };
        let acquire_result = unsafe { self.swapchain_loader.acquire_next_image(self.swapchain, u64::MAX, image_available_semaphore, vk::Fence::null()) };
        let (image_index_u32, mut needs_recreate) = match acquire_result {
            Ok((index, suboptimal)) => (index, suboptimal),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return Ok(true),
            Err(e) => return Err(format!("Failed to acquire swapchain image: {}", e)),
        };
        let image_index = image_index_u32 as usize;
        self.update_uniform_buffer(current_frame_sync_idx)?;
        unsafe { self.device.reset_fences(&[fence])? };
        self.record_main_command_buffer(command_buffer, self.swapchain_framebuffers[image_index], clear_color, current_frame_sync_idx)?;

        let wait_semaphores = [image_available_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [render_finished_semaphore];
        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores).wait_dst_stage_mask(&wait_stages)
            .command_buffers(&[command_buffer]).signal_semaphores(&signal_semaphores).build()];
        unsafe { self.device.queue_submit(self.graphics_queue, &submit_infos, fence)? };

        let swapchains = [self.swapchain];
        let image_indices_present = [image_index_u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores).swapchains(&swapchains).image_indices(&image_indices_present);
        let present_result = unsafe { self.swapchain_loader.queue_present(self.present_queue, &present_info) };
        match present_result {
            Ok(suboptimal) => if suboptimal { needs_recreate = true; },
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => needs_recreate = true,
            Err(e) => return Err(format!("Failed to present swapchain image: {}", e)),
        }
        self.current_frame_index = (self.current_frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(needs_recreate)
    }

    pub fn dispatch_compute_shader(&mut self) -> Result<(), String> {
        let (cmd_pool, cmd_buffer, fence, pipeline, layout, d_set, storage_image_ref) =
        match (
            self.compute_command_pool, self.compute_command_buffer, self.compute_fence,
            self.compute_pipeline, self.compute_pipeline_layout, self.compute_descriptor_set,
            self.compute_storage_image.as_ref() // Borrow storage image
        ) {
            (Some(pool), Some(cb), Some(f), Some(p), Some(pl), Some(ds), Some(si)) => (pool, cb, f, p, pl, ds, si),
            _ => return Err("Compute resources not properly initialized for dispatch.".to_string()),
        };

        let compute_q = self.compute_queue.unwrap_or(self.graphics_queue);

        unsafe {
            self.device.wait_for_fences(&[fence], true, u64::MAX)?;
            self.device.reset_fences(&[fence])?;
            self.device.reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::empty())?;

            let begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            self.device.begin_command_buffer(cmd_buffer, &begin_info)?;

            // --- Transition storage image to GENERAL for compute write ---
            // This assumes the image was created and is in some initial layout (e.g., UNDEFINED or SHADER_READ_ONLY_OPTIMAL)
            // A more robust system would track current layout.
            let pre_compute_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(storage_image_ref.layout) // Use GpuTexture's current layout
                .new_layout(vk::ImageLayout::GENERAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(storage_image_ref.image)
                .subresource_range(vk::ImageSubresourceRange{ aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1})
                .src_access_mask(vk::AccessFlags::empty()) // Or specific if coming from another operation
                .dst_access_mask(vk::AccessFlags::SHADER_WRITE);

            self.device.cmd_pipeline_barrier(cmd_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE, // Or previous stage
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(), &[], &[], &[pre_compute_barrier.build()]);
            // --- End Transition ---


            self.device.cmd_bind_pipeline(cmd_buffer, vk::PipelineBindPoint::COMPUTE, pipeline);
            self.device.cmd_bind_descriptor_sets(cmd_buffer, vk::PipelineBindPoint::COMPUTE, layout, 0, &[d_set], &[]);

            let group_x = (storage_image_ref.extent.width + 15) / 16;
            let group_y = (storage_image_ref.extent.height + 15) / 16;
            self.device.cmd_dispatch(cmd_buffer, group_x, group_y, 1);

            // --- Barrier after compute write, before potential read ---
            let post_compute_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) // Example: prepare for sampling
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(storage_image_ref.image)
                .subresource_range(vk::ImageSubresourceRange{ aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1})
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ); // Example for reading in fragment shader

            self.device.cmd_pipeline_barrier(cmd_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER, // Example stage
                vk::DependencyFlags::empty(), &[], &[], &[post_compute_barrier.build()]);
            // Update the GpuTexture's layout state if it's tracked
            // self.compute_storage_image.as_mut().unwrap().layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            // --- End Barrier ---


            self.device.end_command_buffer(cmd_buffer)?;

            let submit_info = vk::SubmitInfo::builder().command_buffers(&[cmd_buffer]);
            self.device.queue_submit(compute_q, &[submit_info.build()], fence)?;
        }
        // To make the result visible or usable by CPU/other GPU operations,
        // one would typically wait on the `compute_fence` here or elsewhere.
        // For example:
        // unsafe { self.device.wait_for_fences(&[fence], true, u64::MAX)?; }
        // And then potentially read back data from compute_storage_image if needed.
        Ok(())
    }


    pub fn upload_texture_from_data(
        &mut self, pixel_data: &[u8], width: u32, height: u32,
        format: vk::Format, generate_mipmaps: bool,
    ) -> Result<TextureId, String> {
        let pdevice_properties = unsafe { self.instance.get_physical_device_properties(self.physical_device) };
        let enable_anisotropy = true;
        let gpu_texture = texture_pipeline::upload_shm_buffer_to_texture(
            &self.device, &mut self.allocator, self.graphics_queue,
            self.render_command_pool, pixel_data, width, height, format, generate_mipmaps,
            &pdevice_properties, enable_anisotropy,
        )?;
        let id = gpu_texture.id;
        self.textures.insert(id, gpu_texture);
        Ok(id)
    }

    pub fn get_gpu_texture(&self, id: TextureId) -> Option<&GpuTexture> {
        self.textures.get(&id)
    }

    pub fn destroy_texture(&mut self, id: TextureId) -> Result<(), String>{
        if let Some(mut texture) = self.textures.remove(&id) {
            texture.destroy(&self.device, &mut self.allocator);
            Ok(())
        } else { Err(format!("TextureId {:?} not found for destruction.", id)) }
    }

    pub fn import_texture_from_dmabuf(
        &mut self, fd: RawFd, width: u32, height: u32,
        format: vk::Format, drm_modifier: Option<u64>, allocation_size: vk::DeviceSize,
    ) -> Result<TextureId, String> {
        let memory_type_index = dmabuf::get_memory_type_index_for_dmabuf_placeholder(&self.instance, self.physical_device)?;
        let import_options = DmaBufImportOptions {fd, width, height, format, drm_modifier, allocation_size, memory_type_index};
        let (image, device_memory, image_view) = dmabuf::import_dmabuf_as_image(&mut self.allocator, &self.device, &import_options)?;
        let pdevice_properties = unsafe { self.instance.get_physical_device_properties(self.physical_device) };
        let sampler = texture_pipeline::create_texture_sampler(&self.device, &pdevice_properties, 1, true)?;
        let texture_id = TextureId::new();
        let gpu_texture = GpuTexture {
            id: texture_id, image, image_view, allocation: None, memory: Some(device_memory),
            format, extent: vk::Extent2D { width, height }, layout: vk::ImageLayout::UNDEFINED,
            mip_levels: 1, sampler,
        };
        self.textures.insert(texture_id, gpu_texture);
        let temp_cmd_buffer = command_buffer::begin_single_time_commands(&self.device, self.render_command_pool)?;
        texture_pipeline::transition_image_layout(
            &self.device, temp_cmd_buffer, image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, 0, 1,
        );
        command_buffer::end_single_time_commands(&self.device, self.render_command_pool, temp_cmd_buffer, self.graphics_queue)?;
        if let Some(tex) = self.textures.get_mut(&texture_id) { tex.layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL; }
        Ok(texture_id)
    }

    pub fn update_texture_region_from_data(
        &mut self, texture_id: TextureId, region: vk::Rect2D, pixel_data: &[u8],
    ) -> Result<(), String> {
        let gpu_texture = self.textures.get_mut(&texture_id).ok_or_else(|| format!("TextureId {:?} not found for update.", texture_id))?;
        if gpu_texture.mip_levels > 1 {
            println!("Warning: Updating region of a mipmapped texture (TextureId: {:?}). Only base level (0) is updated. Mipmaps will be stale.", texture_id);
        }
        texture_pipeline::update_texture_region(
            &self.device, &mut self.allocator, self.graphics_queue, self.render_command_pool, gpu_texture, region, pixel_data,
        )
    }

    // --- WP-06: Timestamp Query Result ---
    /// Attempts to get the duration of the last fully rendered frame using timestamp queries.
    /// Returns `Ok(None)` if results are not yet available or timestamps are not supported.
    /// Returns `Ok(Some(duration_ms))` if successful.
    pub fn get_last_frame_gpu_duration_ms(&self) -> Result<Option<f32>, String> {
        if let Some(pool) = self.timestamp_query_pool {
            // Determine which set of queries to fetch based on current_frame_index.
            // The queries for the *last completed frame* would be for (current_frame_index - 1 + MAX_FRAMES_IN_FLIGHT) % MAX_FRAMES_IN_FLIGHT.
            // For simplicity, let's assume we are querying for the set of timestamps that *should* be ready.
            // A robust implementation would track which query sets are pending/available.
            // We query for the *previous* frame's set of timestamps, as the current one is likely still in flight.
            let query_idx_for_last_completed_frame = (self.current_frame_index.wrapping_sub(1) + MAX_FRAMES_IN_FLIGHT) % MAX_FRAMES_IN_FLIGHT;
            let first_query = (query_idx_for_last_completed_frame * TIMESTAMP_QUERY_COUNT as usize) as u32;

            let mut query_results = vec![0u64; TIMESTAMP_QUERY_COUNT as usize];
            let status = unsafe {
                self.device.get_query_pool_results(
                    pool,
                    first_query, // first_query
                    TIMESTAMP_QUERY_COUNT, // query_count
                    &mut query_results,
                    vk::QueryResultFlags::BITS_64 | vk::QueryResultFlags::WAIT, // Wait for results
                )
            };

            match status {
                Ok(()) => { // VK_SUCCESS means results are available
                    let start_time = query_results[0];
                    let end_time = query_results[1];
                    if start_time == 0 || end_time == 0 || end_time < start_time {
                        // Timestamps might not have been written correctly or device doesn't support them properly.
                        // Or queries were reset and not written yet for this slot.
                        return Ok(None);
                    }
                    let duration_ticks = end_time - start_time;
                    let nanoseconds_per_tick = self.physical_device_properties.limits.timestamp_period;
                    let duration_ns = duration_ticks as f64 * nanoseconds_per_tick as f64;
                    Ok(Some((duration_ns / 1_000_000.0) as f32)) // Convert to milliseconds
                }
                Err(vk::Result::NOT_READY) => Ok(None), // Results not yet available
                Err(e) => Err(format!("Failed to get query pool results: {}", e)),
            }
        } else {
            Ok(None) // Timestamp queries not enabled/supported
        }
    }
    // --- End WP-06 ---
}


// Helper struct for returning compute-specific resources from setup function.
// This avoids making VulkanRenderer::new too complex if compute setup is extensive.
struct ComputeResources { // Renamed from ComputeVulkanRendererResources
    command_pool: Option<vk::CommandPool>,
    command_buffer: Option<vk::CommandBuffer>,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_set: vk::DescriptorSet,
    storage_image: Option<GpuTexture>,
    fence: vk::Fence,
}


impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            if self.device.device_wait_idle().is_err() {
                eprintln!("Error waiting for device idle in VulkanRenderer drop");
            }

            self.device.destroy_buffer(self.vertex_buffer, None);
            if let Some(alloc) = self.vertex_buffer_memory.take() {
                self.allocator.free(alloc).unwrap_or_else(|e| eprintln!("Failed to free vertex buffer memory: {:?}", e));
            }
            self.device.destroy_buffer(self.index_buffer, None);
            if let Some(alloc) = self.index_buffer_memory.take() {
                self.allocator.free(alloc).unwrap_or_else(|e| eprintln!("Failed to free index buffer memory: {:?}", e));
            }

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device.destroy_buffer(self.uniform_buffers[i], None);
                if let Some(alloc) = self.uniform_buffers_memory[i].take() {
                     self.allocator.free(alloc).unwrap_or_else(|e| eprintln!("Failed to free UBO memory for frame {}: {:?}", i, e));
                }
            }
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            if let Some(pipeline) = self.compute_pipeline.take() { self.device.destroy_pipeline(pipeline, None); }
            if let Some(layout) = self.compute_pipeline_layout.take() { self.device.destroy_pipeline_layout(layout, None); }
            if let Some(dsl) = self.compute_descriptor_set_layout.take() { self.device.destroy_descriptor_set_layout(dsl, None); }
            if let Some(mut image) = self.compute_storage_image.take() { image.destroy(&self.device, &mut self.allocator); } // compute_descriptor_set freed with pool
            if let Some(fence) = self.compute_fence.take() { self.device.destroy_fence(fence, None); }
            if let Some(cb) = self.compute_command_buffer.take() {
                if let Some(pool) = self.compute_command_pool {
                    self.device.free_command_buffers(pool, &[cb]);
                }
            }
            if let Some(pool) = self.compute_command_pool.take() { self.device.destroy_command_pool(pool, None); }


            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            // --- WP-06 Timestamp Query Pool Cleanup ---
            if let Some(pool) = self.timestamp_query_pool.take() {
                self.device.destroy_query_pool(pool, None);
            }
            // --- End WP-06 ---

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device.destroy_semaphore(self.image_available_semaphores[i], None);
                self.device.destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.device.free_command_buffers(self.render_command_pool, &self.render_command_buffers);
            self.device.destroy_command_pool(self.render_command_pool, None);

            let texture_ids: Vec<TextureId> = self.textures.keys().cloned().collect();
            for id in texture_ids {
                if let Some(mut texture) = self.textures.remove(&id) {
                    texture.destroy(&self.device, &mut self.allocator);
                }
            }

            for &image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            if let (Some(loader), Some(messenger)) = (&self.debug_utils, self.debug_messenger) {
                loader.destroy_debug_utils_messenger(messenger, None);
            }
            self.instance.destroy_instance(None);
            println!("Vulkan Renderer dropped comprehensively.");
        }
    }
}

impl RendererInterface for VulkanRenderer {
    fn begin_frame(&mut self) {}
    fn submit_frame(&mut self) {}
    fn present(&mut self) {
        let clear_color = [0.0, 0.0, 0.0, 1.0];
        match self.draw_frame(clear_color) {
            Ok(needs_recreate) => {
                if needs_recreate {
                    // Handle swapchain recreation (e.g., by calling self.resize with current dimensions)
                    println!("Swapchain needs recreation after present.");
                    // self.resize(Size2D { width: self.swapchain_extent.width, height: self.swapchain_extent.height });
                }
            }
            Err(e) => eprintln!("Error during present (draw_frame): {}", e),
        }
    }
    fn resize(&mut self, _new_size: Size2D) {
        println!("VulkanRenderer: Resizing (TODO: Implement swapchain recreation)");
    }
}
