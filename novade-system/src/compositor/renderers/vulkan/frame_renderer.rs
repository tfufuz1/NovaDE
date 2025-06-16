use crate::compositor::renderer::vulkan::{
    allocator::Allocator,
    device::LogicalDevice,
    physical_device::PhysicalDeviceInfo,
    instance::VulkanInstance,
    pipeline::{self, UniformBufferObject, PipelineLayout, GraphicsPipeline, create_compute_pipeline, create_compute_pipeline_layout, GraphicsPushConstants, GammaPushConstants},
    render_pass::{RenderPass, self as render_pass_module},
    surface_swapchain::SurfaceSwapchain,
    framebuffer::{create_framebuffers as create_swapchain_framebuffers_ext, self as framebuffer_module},
    texture::{self, Texture},
    vertex_input::Vertex,
    buffer_utils::create_and_fill_gpu_buffer,
    sync_primitives::FrameSyncPrimitives,
    error::{Result, VulkanError},
    dynamic_uniform_buffer::DynamicUboManager,
};
use crate::compositor::renderer_interface::abstraction::{
    FrameRenderer as AbstractionFrameRenderer,
    RenderableTexture as AbstractionRenderableTexture,
    RenderElement,
    RendererError as AbstractionRendererError,
};
use ash::vk;
use bytemuck;
use log::{debug, info, warn, error};
use smithay::reexports::drm_fourcc::DrmFourcc;
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::utils::{Physical, Logical, Size as SmithaySize, Rectangle as SmithayRectangle, Point as SmithayPoint};
use std::ffi::CString;
use std::path::Path;
use std::fs;
use std::sync::Arc;
use uuid::Uuid;
use vk_mem;
use std::time::{Duration, Instant};

// ... (fn orthographic_projection, fn matrix_multiply, constants - same as before)
fn orthographic_projection(left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let r_l = right-left; let t_b = top-bottom; let f_n = far-near;
    [[2.0/r_l,0.0,0.0,-(right+left)/r_l],[0.0,2.0/t_b,0.0,-(top+bottom)/t_b],[0.0,0.0,1.0/f_n,-near/f_n],[0.0,0.0,0.0,1.0]]
}
fn matrix_multiply(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0f32;4];4];
    for i in 0..4 { for j in 0..4 { for k in 0..4 { result[i][j] += a[i][k]*b[k][j]; }}} result
}
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const MAX_DYNAMIC_OBJECTS: usize = 64;
const PIPELINE_CACHE_FILENAME: &str = "novade_pipeline.cache";


#[derive(Debug)]
enum RenderElementProcessed {
    Texture { texture_dyn: Arc<dyn AbstractionRenderableTexture>, physical_rect: SmithayRectangle<f32, Physical>, tint_color: [f32; 4], },
    SolidColor { color: [f32; 4], geometry: SmithayRectangle<i32, Logical>, }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PostProcessEffectType {
    GammaCorrection,
}

struct PostProcessPassConfig {
    effect_type: PostProcessEffectType,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
}

#[derive(Debug)]
pub struct FrameRenderer {
    // ... (all fields from previous version - same)
    vulkan_instance: Arc<VulkanInstance>,
    physical_device_info: Arc<PhysicalDeviceInfo>,
    logical_device: Arc<LogicalDevice>,
    logical_device_raw: ash::Device, 
    allocator: Arc<Allocator>,
    pub surface_swapchain: SurfaceSwapchain,
    swapchain_render_pass: RenderPass,
    composition_graphics_pipeline: GraphicsPipeline,
    texture: Option<Texture>,
    default_sampler: vk::Sampler,
    composition_descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    composition_descriptor_sets: Vec<vk::DescriptorSet>,
    compute_output_images: Vec<vk::Image>, compute_output_image_allocations: Vec<vk_mem::Allocation>, compute_output_image_views: Vec<vk::ImageView>,
    compute_descriptor_set_layout: vk::DescriptorSetLayout, compute_pipeline_layout: PipelineLayout, compute_pipeline: vk::Pipeline, compute_descriptor_sets: Vec<vk::DescriptorSet>,
    dynamic_ubo_manager: DynamicUboManager<UniformBufferObject>,
    vertex_buffer: vk::Buffer, vertex_buffer_allocation: vk_mem::Allocation,
    index_buffer: vk::Buffer, index_buffer_allocation: vk_mem::Allocation, index_count: u32,
    swapchain_framebuffers: Vec<vk::Framebuffer>,
    depth_image: vk::Image, depth_image_allocation: vk_mem::Allocation, depth_image_view: vk::ImageView, depth_format: vk::Format,
    command_pool: vk::CommandPool, command_buffers: Vec<vk::CommandBuffer>,
    sync_primitives: Vec<FrameSyncPrimitives>,
    current_frame_index: usize, swapchain_suboptimal: bool, pipeline_cache: vk::PipelineCache,
    internal_id: Uuid, last_acquired_image_index: Option<u32>,

    scene_image: vk::Image, scene_image_allocation: vk_mem::Allocation, scene_image_view: vk::ImageView,
    scene_image_format: vk::Format, scene_image_extent: vk::Extent2D,
    scene_render_pass: RenderPass,
    scene_framebuffer: vk::Framebuffer,

    blit_descriptor_set_layout: vk::DescriptorSetLayout,
    blit_pipeline_layout: PipelineLayout, blit_pipeline: vk::Pipeline,
    blit_descriptor_sets: Vec<vk::DescriptorSet>,

    post_process_image_a: vk::Image, post_process_image_a_allocation: vk_mem::Allocation, post_process_image_a_view: vk::ImageView,
    post_process_image_b: vk::Image, post_process_image_b_allocation: vk_mem::Allocation, post_process_image_b_view: vk::ImageView,
    post_process_image_format: vk::Format, post_process_image_extent: vk::Extent2D,
    post_process_render_pass: RenderPass,
    post_process_fb_a: vk::Framebuffer, post_process_fb_b: vk::Framebuffer,
    post_process_descriptor_set_layout: vk::DescriptorSetLayout,
    gamma_correction_pipeline_layout: PipelineLayout,
    gamma_correction_pipeline: vk::Pipeline,
    post_process_input_descriptor_sets: Vec<vk::DescriptorSet>,
    post_process_passes: Vec<PostProcessPassConfig>,

    last_present_time: Option<Instant>,
    target_frame_duration: Duration,
    timeline_value: u64,
}

impl FrameRenderer {
    pub fn create_render_target_texture_resources(logical_device_raw: &ash::Device, allocator: &Allocator, extent: vk::Extent2D, format: vk::Format, usage: vk::ImageUsageFlags) -> Result<(vk::Image, vk_mem::Allocation, vk::ImageView), VulkanError> {
        let image_create_info = vk::ImageCreateInfo::builder().image_type(vk::ImageType::TYPE_2D).format(format).extent(vk::Extent3D{width:extent.width,height:extent.height,depth:1}).mip_levels(1).array_layers(1).samples(vk::SampleCountFlags::TYPE_1).tiling(vk::ImageTiling::OPTIMAL).usage(usage).initial_layout(vk::ImageLayout::UNDEFINED);
        let allocation_create_info = vk_mem::AllocationCreateInfo{usage: vk_mem::MemoryUsage::GpuOnly, ..Default::default()};
        let (image, allocation, _alloc_info) = allocator.create_image(&image_create_info, &allocation_create_info)?;
        let view_create_info = vk::ImageViewCreateInfo::builder().image(image).view_type(vk::ImageViewType::TYPE_2D).format(format).subresource_range(vk::ImageSubresourceRange{aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level:0, level_count:1, base_array_layer:0, layer_count:1});
        let image_view = unsafe { logical_device_raw.create_image_view(&view_create_info, None) }?; Ok((image,allocation,image_view))
    }

    fn create_scene_render_pass_internal(logical_device: Arc<LogicalDevice>, scene_format: vk::Format, depth_format: vk::Format) -> Result<RenderPass, VulkanError> {
        let color_attachment = vk::AttachmentDescription::builder().format(scene_format).samples(vk::SampleCountFlags::TYPE_1).load_op(vk::AttachmentLoadOp::CLEAR).store_op(vk::AttachmentStoreOp::STORE).stencil_load_op(vk::AttachmentLoadOp::DONT_CARE).stencil_store_op(vk::AttachmentStoreOp::DONT_CARE).initial_layout(vk::ImageLayout::UNDEFINED).final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        let depth_attachment = vk::AttachmentDescription::builder().format(depth_format).samples(vk::SampleCountFlags::TYPE_1).load_op(vk::AttachmentLoadOp::CLEAR).store_op(vk::AttachmentStoreOp::DONT_CARE).stencil_load_op(vk::AttachmentLoadOp::DONT_CARE).stencil_store_op(vk::AttachmentStoreOp::DONT_CARE).initial_layout(vk::ImageLayout::UNDEFINED).final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        let color_attachment_ref = vk::AttachmentReference::builder().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let depth_attachment_ref = vk::AttachmentReference::builder().attachment(1).layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        let subpass = vk::SubpassDescription::builder().pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS).color_attachments(std::slice::from_ref(&color_attachment_ref)).depth_stencil_attachment(&depth_attachment_ref);
        let attachments = [color_attachment.build(), depth_attachment.build()]; let subpasses = [subpass.build()];
        let dep_external_to_subpass = vk::SubpassDependency::builder().src_subpass(vk::SUBPASS_EXTERNAL).dst_subpass(0).src_stage_mask(vk::PipelineStageFlags::TOP_OF_PIPE).dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS).src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);
        let dep_subpass_to_external = vk::SubpassDependency::builder().src_subpass(0).dst_subpass(vk::SUBPASS_EXTERNAL).src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT).dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER).src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ);
        let dependencies = [dep_external_to_subpass.build(), dep_subpass_to_external.build()];
        let render_pass_info = vk::RenderPassCreateInfo::builder().attachments(&attachments).subpasses(&subpasses).dependencies(&dependencies);
        let raw_pass = unsafe { logical_device.raw.create_render_pass(&render_pass_info, None) }.map_err(VulkanError::from)?;
        Ok(RenderPass { raw: raw_pass, logical_device_raw: logical_device.raw.clone() })
    }

    fn create_framebuffer_with_depth(logical_device_raw: &ash::Device, render_pass: vk::RenderPass, color_image_view: vk::ImageView, depth_image_view: vk::ImageView, extent: vk::Extent2D) -> Result<vk::Framebuffer, VulkanError> {
        let attachments = [color_image_view, depth_image_view];
        let framebuffer_info = vk::FramebufferCreateInfo::builder().render_pass(render_pass).attachments(&attachments).width(extent.width).height(extent.height).layers(1);
        unsafe { logical_device_raw.create_framebuffer(&framebuffer_info, None) }.map_err(VulkanError::from)
    }

    fn create_color_only_framebuffer(logical_device_raw: &ash::Device, render_pass: vk::RenderPass, color_image_view: vk::ImageView, extent: vk::Extent2D) -> Result<vk::Framebuffer, VulkanError> {
        let attachments = [color_image_view];
        let framebuffer_info = vk::FramebufferCreateInfo::builder().render_pass(render_pass).attachments(&attachments).width(extent.width).height(extent.height).layers(1);
        unsafe { logical_device_raw.create_framebuffer(&framebuffer_info, None) }.map_err(VulkanError::from)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(vulkan_instance: Arc<VulkanInstance>, physical_device_info: Arc<PhysicalDeviceInfo>, logical_device: Arc<LogicalDevice>, allocator_owned: Allocator, surface_swapchain: SurfaceSwapchain, _old_swapchain_render_pass_arg: RenderPass, vertex_shader_module: vk::ShaderModule, fragment_shader_module: vk::ShaderModule) -> Result<Self, VulkanError> {
        let logical_device_raw = logical_device.raw.clone();
        let allocator = Arc::new(allocator_owned);
        let swapchain_render_pass = RenderPass::new_color_only(logical_device.as_ref(), surface_swapchain.format(), vk::ImageLayout::PRESENT_SRC_KHR)?;
        let initial_cache_data = match fs::read(PIPELINE_CACHE_FILENAME) { Ok(data) => data, Err(_) => Vec::new() };
        let pipeline_cache_create_info = vk::PipelineCacheCreateInfo::builder().initial_data(&initial_cache_data);
        let pipeline_cache = unsafe { logical_device_raw.create_pipeline_cache(&pipeline_cache_create_info, None) }?;
        let sampler_create_info = vk::SamplerCreateInfo::builder().mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR).address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE).address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE).address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE).mipmap_mode(vk::SamplerMipmapMode::LINEAR).min_lod(0.0).max_lod(1.0).anisotropy_enable(false).border_color(vk::BorderColor::FLOAT_OPAQUE_BLACK);
        let default_sampler = unsafe { logical_device_raw.create_sampler(&sampler_create_info, None) }?;
        let comp_ubo_binding = vk::DescriptorSetLayoutBinding::builder().binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC).descriptor_count(1).stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);
        let comp_sampler_binding = vk::DescriptorSetLayoutBinding::builder().binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let composition_dsl_bindings = [comp_ubo_binding.build(), comp_sampler_binding.build()];
        let composition_dsl_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&composition_dsl_bindings);
        let composition_descriptor_set_layout = unsafe { logical_device_raw.create_descriptor_set_layout(&composition_dsl_create_info, None) }?;
        let comp_push_constant_range = vk::PushConstantRange::builder().stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT).offset(0).size(std::mem::size_of::<GraphicsPushConstants>() as u32);
        
        let scene_image_extent = surface_swapchain.extent();
        let scene_image_format = vk::Format::R8G8B8A8_UNORM;
        let scene_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST;
        let (scene_image, scene_image_allocation, scene_image_view) =
            Self::create_render_target_texture_resources(&logical_device_raw, allocator.as_ref(), scene_image_extent, scene_image_format, scene_usage)?;
        let (depth_image, depth_image_allocation, depth_image_view, depth_format) =
            pipeline::create_depth_resources(logical_device.as_ref(), physical_device_info.as_ref(), vulkan_instance.raw(), allocator.as_ref(), scene_image_extent)?;
        let scene_render_pass = Self::create_scene_render_pass_internal(logical_device.clone(), scene_image_format, depth_format)?;
        let scene_framebuffer = Self::create_framebuffer_with_depth(&logical_device_raw, scene_render_pass.raw, scene_image_view, depth_image_view, scene_image_extent)?;

        let composition_pipeline_layout_obj = PipelineLayout::new(logical_device.as_ref(), &[composition_descriptor_set_layout], &[comp_push_constant_range.build()])?;
        let composition_graphics_pipeline = GraphicsPipeline::new(logical_device.as_ref(), scene_render_pass.raw, scene_image_extent, composition_pipeline_layout_obj, vertex_shader_module, fragment_shader_module, pipeline_cache, true, true)?;
        
        let blit_sampler_binding = vk::DescriptorSetLayoutBinding::builder().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let blit_dsl_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(std::slice::from_ref(&blit_sampler_binding));
        let blit_descriptor_set_layout = unsafe { logical_device_raw.create_descriptor_set_layout(&blit_dsl_create_info, None) }?;
        let blit_pipeline_layout_obj = PipelineLayout::new(logical_device.as_ref(), &[blit_descriptor_set_layout], &[])?;
        let blit_vert_shader_module = vertex_shader_module;
        let blit_frag_shader_code = pipeline::load_spirv_file("assets/shaders/blit.frag.spv")?;
        let blit_frag_shader_module = pipeline::create_shader_module(&logical_device_raw, &blit_frag_shader_code)?;
        let blit_pipeline = GraphicsPipeline::new(logical_device.as_ref(), swapchain_render_pass.raw, surface_swapchain.extent(), blit_pipeline_layout_obj, blit_vert_shader_module, blit_frag_shader_module, pipeline_cache, false, false)?;
        unsafe { logical_device_raw.destroy_shader_module(blit_frag_shader_module, None); }

        let post_process_image_extent = scene_image_extent;
        let post_process_image_format = scene_image_format;
        let pp_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED;
        let (post_process_image_a, post_process_image_a_allocation, post_process_image_a_view) =
            Self::create_render_target_texture_resources(&logical_device_raw, allocator.as_ref(), post_process_image_extent, post_process_image_format, pp_usage)?;
        let (post_process_image_b, post_process_image_b_allocation, post_process_image_b_view) =
            Self::create_render_target_texture_resources(&logical_device_raw, allocator.as_ref(), post_process_image_extent, post_process_image_format, pp_usage)?;
        let post_process_render_pass = RenderPass::new_color_only(logical_device.as_ref(), post_process_image_format, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)?;
        let post_process_fb_a = Self::create_color_only_framebuffer(&logical_device_raw, post_process_render_pass.raw, post_process_image_a_view, post_process_image_extent)?;
        let post_process_fb_b = Self::create_color_only_framebuffer(&logical_device_raw, post_process_render_pass.raw, post_process_image_b_view, post_process_image_extent)?;

        let pp_sampler_binding = vk::DescriptorSetLayoutBinding::builder().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let post_process_dsl_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(std::slice::from_ref(&pp_sampler_binding));
        let post_process_descriptor_set_layout = unsafe { logical_device_raw.create_descriptor_set_layout(&post_process_dsl_create_info, None) }?;

        let pp_push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<GammaPushConstants>() as u32);
        let gamma_correction_pipeline_layout = PipelineLayout::new(logical_device.as_ref(), &[post_process_descriptor_set_layout], &[pp_push_constant_range.build()])?;

        let gamma_frag_shader_code = pipeline::load_spirv_file("assets/shaders/gamma_correction.frag.spv")?;
        let gamma_frag_shader_module = pipeline::create_shader_module(&logical_device_raw, &gamma_frag_shader_code)?;
        let gamma_correction_pipeline = GraphicsPipeline::new(logical_device.as_ref(), post_process_render_pass.raw, post_process_image_extent, gamma_correction_pipeline_layout.clone(), vertex_shader_module, gamma_frag_shader_module, pipeline_cache, false, false)?;
        unsafe { logical_device_raw.destroy_shader_module(gamma_frag_shader_module, None); }

        let mut post_process_passes = Vec::new();
        post_process_passes.push(PostProcessPassConfig {
            effect_type: PostProcessEffectType::GammaCorrection,
            pipeline: gamma_correction_pipeline.raw,
            pipeline_layout: gamma_correction_pipeline_layout.raw,
        });

        let mut compute_output_images = Vec::new(); let mut compute_output_image_allocations = Vec::new(); let mut compute_output_image_views = Vec::new();
        let compute_image_format = vk::Format::R8G8B8A8_SRGB; let compute_image_usage = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED;
        for _ in 0..MAX_FRAMES_IN_FLIGHT { let (img,alloc,view) = texture::Texture::new_storage_image(&logical_device_raw,allocator.as_ref(),surface_swapchain.extent().width,surface_swapchain.extent().height,compute_image_format,compute_image_usage,)?; compute_output_images.push(img);compute_output_image_allocations.push(alloc);compute_output_image_views.push(view); }
        let compute_input_sampler_binding = vk::DescriptorSetLayoutBinding::builder().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE);
        let compute_output_storage_binding = vk::DescriptorSetLayoutBinding::builder().binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE);
        let compute_dsl_bindings = [compute_input_sampler_binding.build(),compute_output_storage_binding.build()];
        let compute_dsl_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&compute_dsl_bindings);
        let compute_descriptor_set_layout = unsafe { logical_device_raw.create_descriptor_set_layout(&compute_dsl_create_info, None) }?;
        let compute_pipeline_layout = create_compute_pipeline_layout(logical_device.as_ref(),&[compute_descriptor_set_layout])?;
        let compute_shader_spirv = pipeline::load_spirv_file("assets/shaders/invert.comp.spv")?;
        let compute_shader_module = pipeline::create_shader_module(&logical_device_raw, &compute_shader_spirv)?;
        let compute_pipeline = create_compute_pipeline(logical_device.as_ref(), compute_pipeline_layout.raw, compute_shader_module, pipeline_cache)?;
        unsafe { logical_device_raw.destroy_shader_module(compute_shader_module, None); }
        let dynamic_ubo_manager = DynamicUboManager::<UniformBufferObject>::new(allocator.as_ref(), logical_device.as_ref(), &physical_device_info.properties, MAX_DYNAMIC_OBJECTS)?;
        let pool_sizes = [ vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32).build(), vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32 * 4).build(), vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(MAX_FRAMES_IN_FLIGHT as u32).build(), ];
        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder().max_sets(MAX_FRAMES_IN_FLIGHT as u32 * 4).pool_sizes(&pool_sizes);
        let descriptor_pool = unsafe { logical_device_raw.create_descriptor_pool(&descriptor_pool_create_info, None) }?;
        let composition_dsl_vec=vec![composition_descriptor_set_layout;MAX_FRAMES_IN_FLIGHT]; let composition_d_set_alloc_info=vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&composition_dsl_vec); let composition_descriptor_sets=unsafe{logical_device_raw.allocate_descriptor_sets(&composition_d_set_alloc_info)}?;
        for i in 0..MAX_FRAMES_IN_FLIGHT { let ubo_info=vk::DescriptorBufferInfo::builder().buffer(dynamic_ubo_manager.get_buffer(i)).offset(0).range(dynamic_ubo_manager.get_item_size_for_descriptor()); let ubo_write=vk::WriteDescriptorSet::builder().dst_set(composition_descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC).buffer_info(&[ubo_info.build()]).build(); let dummy_sampler_info=vk::DescriptorImageInfo::builder().sampler(default_sampler).image_view(compute_output_image_views[i]).image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL); let dummy_sampler_write=vk::WriteDescriptorSet::builder().dst_set(composition_descriptor_sets[i]).dst_binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&dummy_sampler_info)).build(); unsafe{logical_device_raw.update_descriptor_sets(&[ubo_write,dummy_sampler_write],&[])};}
        let blit_dsl_vec=vec![blit_descriptor_set_layout;MAX_FRAMES_IN_FLIGHT]; let blit_d_set_alloc_info=vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&blit_dsl_vec); let blit_descriptor_sets=unsafe{logical_device_raw.allocate_descriptor_sets(&blit_d_set_alloc_info)}?;
        for i in 0..MAX_FRAMES_IN_FLIGHT { let scene_image_sampler_info=vk::DescriptorImageInfo::builder().sampler(default_sampler).image_view(scene_image_view).image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL); let blit_write=vk::WriteDescriptorSet::builder().dst_set(blit_descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&scene_image_sampler_info)).build(); unsafe{logical_device_raw.update_descriptor_sets(&[blit_write],&[])};}
        let compute_dsl_vec=vec![compute_descriptor_set_layout;MAX_FRAMES_IN_FLIGHT]; let compute_d_set_alloc_info=vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&compute_dsl_vec); let compute_descriptor_sets=unsafe{logical_device_raw.allocate_descriptor_sets(&compute_d_set_alloc_info)}?;
        for i in 0..MAX_FRAMES_IN_FLIGHT { let storage_image_info=vk::DescriptorImageInfo::builder().image_layout(vk::ImageLayout::GENERAL).image_view(compute_output_image_views[i]); let output_write=vk::WriteDescriptorSet::builder().dst_set(compute_descriptor_sets[i]).dst_binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(&[storage_image_info.build()]).build(); unsafe{logical_device_raw.update_descriptor_sets(&[output_write],&[])};}
        let pp_input_dsl_vec = vec![post_process_descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];
        let pp_input_d_set_alloc_info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&pp_input_dsl_vec);
        let post_process_input_descriptor_sets = unsafe { logical_device_raw.allocate_descriptor_sets(&pp_input_d_set_alloc_info) }?;
        for i in 0..MAX_FRAMES_IN_FLIGHT { let pp_input_sampler_info = vk::DescriptorImageInfo::builder().sampler(default_sampler).image_view(scene_image_view).image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL); let pp_input_write = vk::WriteDescriptorSet::builder().dst_set(post_process_input_descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&pp_input_sampler_info)).build(); unsafe { logical_device_raw.update_descriptor_sets(&[pp_input_write], &[]); } }
        let vertices = [ Vertex { pos: [-0.5,-0.5], tex_coord: [0.0,0.0] }, Vertex { pos: [0.5,-0.5], tex_coord: [1.0,0.0] }, Vertex { pos: [0.5,0.5], tex_coord: [1.0,1.0] }, Vertex { pos: [-0.5,0.5], tex_coord: [0.0,1.0] }, ];
        let indices: [u16;6] = [0,1,2,2,3,0]; let index_count = indices.len() as u32;
        let pool_create_info = vk::CommandPoolCreateInfo::builder().queue_family_index(physical_device_info.queue_family_indices.graphics_family.unwrap()).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe { logical_device_raw.create_command_pool(&pool_create_info, None) }?;
        let (vertex_buffer,vertex_buffer_allocation) = create_and_fill_gpu_buffer(allocator.as_ref(),logical_device.as_ref(),command_pool,logical_device.queues.graphics_queue,&vertices,vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let (index_buffer,index_buffer_allocation) = create_and_fill_gpu_buffer(allocator.as_ref(),logical_device.as_ref(),command_pool,logical_device.queues.graphics_queue,&indices,vk::BufferUsageFlags::INDEX_BUFFER)?;
        let swapchain_framebuffers = create_swapchain_framebuffers_ext(logical_device.as_ref(), swapchain_render_pass.raw, surface_swapchain.image_views(), None, surface_swapchain.extent())?;
        let cmd_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder().command_pool(command_pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
        let command_buffers = unsafe { logical_device_raw.allocate_command_buffers(&cmd_buffer_allocate_info) }?;
        let mut sync_primitives = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for i in 0..MAX_FRAMES_IN_FLIGHT { sync_primitives.push(FrameSyncPrimitives::new(logical_device.as_ref(), i==0)?); }

        Ok(Self {
            vulkan_instance, physical_device_info, logical_device, logical_device_raw, allocator,
            surface_swapchain, swapchain_render_pass,
            composition_graphics_pipeline, texture, default_sampler,
            composition_descriptor_set_layout, descriptor_pool, composition_descriptor_sets,
            compute_output_images, compute_output_image_allocations, compute_output_image_views,
            compute_descriptor_set_layout, compute_pipeline_layout, compute_pipeline, compute_descriptor_sets,
            dynamic_ubo_manager,
            vertex_buffer, vertex_buffer_allocation, index_buffer, index_buffer_allocation, index_count,
            swapchain_framebuffers,
            depth_image, depth_image_allocation, depth_image_view, depth_format,
            command_pool, command_buffers, sync_primitives,
            current_frame_index: 0, swapchain_suboptimal: false, pipeline_cache,
            internal_id: Uuid::new_v4(), last_acquired_image_index: None,
            scene_image, scene_image_allocation, scene_image_view,
            scene_image_format, scene_image_extent,
            scene_render_pass,
            scene_framebuffer,
            blit_descriptor_set_layout, blit_pipeline_layout: blit_pipeline_layout_obj, blit_pipeline, blit_descriptor_sets,
            post_process_image_a, post_process_image_a_allocation, post_process_image_a_view,
            post_process_image_b, post_process_image_b_allocation, post_process_image_b_view,
            post_process_image_format, post_process_image_extent,
            post_process_render_pass,
            post_process_fb_a, post_process_fb_b,
            post_process_descriptor_set_layout,
            gamma_correction_pipeline_layout,
            gamma_correction_pipeline,
            post_process_input_descriptor_sets,
            post_process_passes,
            last_present_time: None,
            target_frame_duration: Duration::from_secs_f64(1.0 / 60.0),
            timeline_value: 0,
        })
    }

    fn record_composition_pass( &self, command_buffer: vk::CommandBuffer, current_frame_idx_for_descriptors: usize, elements_to_render: &[RenderElementProcessed], ) -> Result<(), AbstractionRendererError> { /* ... */ Ok(()) }
    fn record_blit_to_swapchain_pass( &self, command_buffer: vk::CommandBuffer, swapchain_image_index: u32, current_frame_idx_for_blit_ds: usize, _input_image_view_for_blit: vk::ImageView ) -> Result<(), AbstractionRendererError> { /* ... */ Ok(()) }

    pub fn recreate_swapchain(&mut self) -> Result<(), VulkanError> {
        unsafe { self.logical_device_raw.device_wait_idle() }?;
        for &fb in self.swapchain_framebuffers.iter() { unsafe { self.logical_device_raw.destroy_framebuffer(fb,None);}} self.swapchain_framebuffers.clear();
        unsafe { self.logical_device_raw.destroy_framebuffer(self.scene_framebuffer,None); }
        unsafe { self.logical_device_raw.destroy_framebuffer(self.post_process_fb_a, None); }
        unsafe { self.logical_device_raw.destroy_framebuffer(self.post_process_fb_b, None); }
        unsafe { self.logical_device_raw.destroy_image_view(self.depth_image_view,None); }
        self.allocator.destroy_image(self.depth_image, &self.depth_image_allocation);
        unsafe { self.logical_device_raw.destroy_image_view(self.scene_image_view,None); }
        self.allocator.destroy_image(self.scene_image, &self.scene_image_allocation);
        unsafe { self.logical_device_raw.destroy_image_view(self.post_process_image_a_view, None); }
        self.allocator.destroy_image(self.post_process_image_a, &self.post_process_image_a_allocation);
        unsafe { self.logical_device_raw.destroy_image_view(self.post_process_image_b_view, None); }
        self.allocator.destroy_image(self.post_process_image_b, &self.post_process_image_b_allocation);
        for i in 0..self.compute_output_images.len() { unsafe {self.logical_device_raw.destroy_image_view(self.compute_output_image_views[i],None);} self.allocator.destroy_image(self.compute_output_images[i], &self.compute_output_image_allocations[i]); }
        self.compute_output_image_views.clear(); self.compute_output_images.clear(); self.compute_output_image_allocations.clear();
        self.surface_swapchain.recreate(&self.physical_device_info, &self.logical_device, self.surface_swapchain.extent())?;
        let new_swapchain_extent = self.surface_swapchain.extent();
        self.scene_image_extent = new_swapchain_extent;
        self.post_process_image_extent = new_swapchain_extent;
        let scene_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST;
        let (si,sia,siv) = Self::create_render_target_texture_resources(&self.logical_device_raw, &self.allocator, self.scene_image_extent, self.scene_image_format, scene_usage)?;
        self.scene_image = si; self.scene_image_allocation = sia; self.scene_image_view = siv;
        let pp_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED;
        let (ppa_i,ppa_a,ppa_v) = Self::create_render_target_texture_resources(&self.logical_device_raw, &self.allocator, self.post_process_image_extent, self.post_process_image_format, pp_usage)?;
        self.post_process_image_a = ppa_i; self.post_process_image_a_allocation = ppa_a; self.post_process_image_a_view = ppa_v;
        let (ppb_i,ppb_a,ppb_v) = Self::create_render_target_texture_resources(&self.logical_device_raw, &self.allocator, self.post_process_image_extent, self.post_process_image_format, pp_usage)?;
        self.post_process_image_b = ppb_i; self.post_process_image_b_allocation = ppb_a; self.post_process_image_b_view = ppb_v;
        let (ndi,ndia,ndiv,ndf) = pipeline::create_depth_resources(&self.logical_device, &self.physical_device_info, self.vulkan_instance.raw(), &self.allocator, self.scene_image_extent)?;
        self.depth_image = ndi; self.depth_image_allocation = ndia; self.depth_image_view = ndiv; self.depth_format = ndf;
        self.scene_framebuffer = Self::create_framebuffer_with_depth(&self.logical_device_raw, self.scene_render_pass.raw, self.scene_image_view, self.depth_image_view, self.scene_image_extent)?;
        self.post_process_fb_a = Self::create_color_only_framebuffer(&self.logical_device_raw, self.post_process_render_pass.raw, self.post_process_image_a_view, self.post_process_image_extent)?;
        self.post_process_fb_b = Self::create_color_only_framebuffer(&self.logical_device_raw, self.post_process_render_pass.raw, self.post_process_image_b_view, self.post_process_image_extent)?;
        self.swapchain_framebuffers = create_swapchain_framebuffers_ext(&self.logical_device, self.swapchain_render_pass.raw, self.surface_swapchain.image_views(), None, new_swapchain_extent)?;
        self.compute_output_images.reserve(MAX_FRAMES_IN_FLIGHT); self.compute_output_image_allocations.reserve(MAX_FRAMES_IN_FLIGHT); self.compute_output_image_views.reserve(MAX_FRAMES_IN_FLIGHT);
        let compute_fmt = vk::Format::R8G8B8A8_SRGB; let compute_usage = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED;
        for _ in 0..MAX_FRAMES_IN_FLIGHT { let (img,alloc,view) = texture::Texture::new_storage_image(&self.logical_device_raw, &self.allocator, new_swapchain_extent.width, new_swapchain_extent.height, compute_fmt, compute_usage)?; self.compute_output_images.push(img); self.compute_output_image_allocations.push(alloc); self.compute_output_image_views.push(view); }
        let ubo_item_size_for_descriptor = self.dynamic_ubo_manager.get_item_size_for_descriptor();
        let sampler_for_compute_or_demo_texture = self.texture.as_ref().map_or(self.default_sampler, |t|t.sampler);
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            if let Some(texture_ref) = self.texture.as_ref() { let input_img_info = vk::DescriptorImageInfo::builder().sampler(texture_ref.sampler).image_view(texture_ref.view).image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL); let input_write = vk::WriteDescriptorSet::builder().dst_set(self.compute_descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(&[input_img_info.build()]).build(); unsafe { self.logical_device_raw.update_descriptor_sets(&[input_write], &[]); } }
            let output_storage_img_info = vk::DescriptorImageInfo::builder().image_layout(vk::ImageLayout::GENERAL).image_view(self.compute_output_image_views[i]);
            let output_write = vk::WriteDescriptorSet::builder().dst_set(self.compute_descriptor_sets[i]).dst_binding(1).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(&[output_storage_img_info.build()]).build();
            unsafe { self.logical_device_raw.update_descriptor_sets(&[output_write], &[]); }
            let ubo_info = vk::DescriptorBufferInfo::builder().buffer(self.dynamic_ubo_manager.get_buffer(i)).offset(0).range(ubo_item_size_for_descriptor);
            let ubo_write = vk::WriteDescriptorSet::builder().dst_set(self.composition_descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC).buffer_info(&[ubo_info.build()]).build();
            let default_sampled_image_info = vk::DescriptorImageInfo::builder().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(self.compute_output_image_views[i]).sampler(sampler_for_compute_or_demo_texture);
            let default_sampled_write = vk::WriteDescriptorSet::builder().dst_set(self.composition_descriptor_sets[i]).dst_binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(&[default_sampled_image_info.build()]).build();
            unsafe { self.logical_device_raw.update_descriptor_sets(&[ubo_write, default_sampled_write], &[]); }
            let scene_image_sampler_info = vk::DescriptorImageInfo::builder().sampler(self.default_sampler).image_view(self.scene_image_view).image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            let blit_write = vk::WriteDescriptorSet::builder().dst_set(self.blit_descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&scene_image_sampler_info)).build();
            unsafe { self.logical_device_raw.update_descriptor_sets(&[blit_write], &[]); }
            let pp_input_sampler_info = vk::DescriptorImageInfo::builder().sampler(self.default_sampler).image_view(self.scene_image_view).image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            let pp_input_write = vk::WriteDescriptorSet::builder().dst_set(self.post_process_input_descriptor_sets[i]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&pp_input_sampler_info)).build();
            unsafe { self.logical_device_raw.update_descriptor_sets(&[pp_input_write], &[]); }
        }
        self.swapchain_suboptimal = false;
        info!("Swapchain and all render target resources recreation complete.");
        Ok(())
    }
    impl Drop for FrameRenderer {
        fn drop(&mut self) {
            unsafe { if let Err(e) = self.logical_device_raw.device_wait_idle() { error!("Device wait idle error in drop: {}", e); } }
            unsafe {
                // self.gamma_correction_pipeline_layout is PipelineLayout struct, drops itself
                self.logical_device_raw.destroy_pipeline(self.gamma_correction_pipeline, None);
                self.logical_device_raw.destroy_descriptor_set_layout(self.post_process_descriptor_set_layout, None);
                self.logical_device_raw.destroy_framebuffer(self.post_process_fb_a, None);
                self.logical_device_raw.destroy_framebuffer(self.post_process_fb_b, None);
                // post_process_render_pass is RenderPass struct, drops itself
                self.logical_device_raw.destroy_image_view(self.post_process_image_a_view, None);
                self.logical_device_raw.destroy_image_view(self.post_process_image_b_view, None);
            }
            self.allocator.destroy_image(self.post_process_image_a, &self.post_process_image_a_allocation);
            self.allocator.destroy_image(self.post_process_image_b, &self.post_process_image_b_allocation);
            unsafe {
                self.logical_device_raw.destroy_descriptor_set_layout(self.blit_descriptor_set_layout, None);
                self.logical_device_raw.destroy_pipeline(self.blit_pipeline, None);
                self.logical_device_raw.destroy_framebuffer(self.scene_framebuffer, None);
                // scene_render_pass is RenderPass struct, drops itself
                self.logical_device_raw.destroy_image_view(self.scene_image_view, None);
            }
            self.allocator.destroy_image(self.scene_image, &self.scene_image_allocation);
            if self.default_sampler != vk::Sampler::null() { unsafe { self.logical_device_raw.destroy_sampler(self.default_sampler, None); } }
            unsafe { if self.pipeline_cache != vk::PipelineCache::null() { self.logical_device_raw.destroy_pipeline_cache(self.pipeline_cache, None); } }
            if self.compute_pipeline != vk::Pipeline::null() { unsafe { self.logical_device_raw.destroy_pipeline(self.compute_pipeline, None); } }
            // compute_pipeline_layout is PipelineLayout struct, drops itself
            if self.compute_descriptor_set_layout != vk::DescriptorSetLayout::null() { unsafe { self.logical_device_raw.destroy_descriptor_set_layout(self.compute_descriptor_set_layout, None); } }
            for i in 0..self.compute_output_images.len() { unsafe { self.logical_device_raw.destroy_image_view(self.compute_output_image_views[i], None); } self.allocator.destroy_image(self.compute_output_images[i], &self.compute_output_image_allocations[i]); }
            for &fb in self.swapchain_framebuffers.iter() { unsafe { self.logical_device_raw.destroy_framebuffer(fb, None); } }
            // composition_graphics_pipeline is GraphicsPipeline struct, drops itself (and its layout)
            // swapchain_render_pass is RenderPass struct, drops itself
            unsafe { self.logical_device_raw.destroy_image_view(self.depth_image_view, None); }
            self.allocator.destroy_image(self.depth_image, &self.depth_image_allocation);
            if self.descriptor_pool != vk::DescriptorPool::null() { unsafe { self.logical_device_raw.destroy_descriptor_pool(self.descriptor_pool, None); } }
            if self.composition_descriptor_set_layout != vk::DescriptorSetLayout::null() { unsafe { self.logical_device_raw.destroy_descriptor_set_layout(self.composition_descriptor_set_layout, None); } }
            self.allocator.destroy_buffer(self.vertex_buffer, &self.vertex_buffer_allocation);
            self.allocator.destroy_buffer(self.index_buffer, &self.index_buffer_allocation);
            for prim in self.sync_primitives.iter_mut() { prim.destroy(&self.logical_device_raw); }
            unsafe { self.logical_device_raw.destroy_command_pool(self.command_pool, None); }
            info!("FrameRenderer dropped successfully.");
        }
    }
}
impl From<VulkanError> for AbstractionRendererError {
    fn from(err: VulkanError) -> Self {
        match err {
            VulkanError::VkResult(vk_err) => { if vk_err == vk::Result::ERROR_OUT_OF_DATE_KHR || vk_err == vk::Result::SUBOPTIMAL_KHR { AbstractionRendererError::BufferSwapFailed(format!("Swapchain out of date/suboptimal: {:?}", vk_err)) } else { AbstractionRendererError::Generic(format!("Vulkan API error: {:?}", vk_err)) } }
            VulkanError::VkMemError(mem_err) => AbstractionRendererError::Generic(format!("VMA error: {:?}", mem_err)),
            _ => AbstractionRendererError::Generic(format!("Unknown or unmapped Vulkan error: {:?}", err)),
        }
    }
}
impl AbstractionFrameRenderer for FrameRenderer {
    fn id(&self) -> Uuid { self.internal_id }
    fn render_frame<'a>( &mut self, elements: impl IntoIterator<Item = RenderElement<'a>>, _output_geometry: SmithayRectangle<i32, Physical>, _output_scale: f64, ) -> Result<(), AbstractionRendererError> {
        // ANCHOR[TimelineSubmit_FrameRendererRS]
        let current_sync_primitives = &self.sync_primitives[self.current_frame_index];

        unsafe { self.logical_device_raw.wait_for_fences(&[current_sync_primitives.in_flight_fence], true, u64::MAX) }?;

        let image_available_target_value = self.timeline_value + 1;
        let render_finished_target_value = image_available_target_value;

        let image_index_result = unsafe {
            self.surface_swapchain.swapchain_loader.acquire_next_image(
                self.surface_swapchain.swapchain_khr(),
                u64::MAX,
                current_sync_primitives.image_available_semaphore,
                vk::Fence::null(),
            )
        };

        let swapchain_image_index = match image_index_result {
            Ok((idx, suboptimal)) => { if suboptimal { self.swapchain_suboptimal = true; } idx }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                warn!("Swapchain suboptimal/OOD during acquire. Triggering deferred recreation.");
                self.swapchain_suboptimal = true; self.last_acquired_image_index = None; return Ok(());
            }
            Err(e) => return Err(VulkanError::from(e).into()),
        };
        self.last_acquired_image_index = Some(swapchain_image_index);

        unsafe { self.logical_device_raw.reset_fences(&[current_sync_primitives.in_flight_fence]) }?;

        let current_command_buffer = self.command_buffers[self.current_frame_index];
        unsafe { self.logical_device_raw.reset_command_buffer(current_command_buffer, vk::CommandBufferResetFlags::empty()) }?;
        let cmd_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { self.logical_device_raw.begin_command_buffer(current_command_buffer, &cmd_begin_info) }?;

        let mut processed_elements: Vec<RenderElementProcessed> = Vec::new();
        for element in elements { match element { RenderElement::WaylandSurface { surface_data_mutex_arc, physical_clipped_rect, alpha, .. } => { let surface_data = surface_data_mutex_arc.lock().unwrap(); if let Some(texture_handle_dyn) = &surface_data.texture_handle { processed_elements.push(RenderElementProcessed::Texture { texture_dyn: texture_handle_dyn.clone(), physical_rect: physical_clipped_rect, tint_color: [1.0, 1.0, 1.0, alpha.unwrap_or(1.0)], }); } } RenderElement::SolidColor { color, geometry } => { processed_elements.push(RenderElementProcessed::SolidColor { color, geometry }); } RenderElement::Cursor { texture_arc, position_logical, hotspot_logical, .. } => { let px=(position_logical.x-hotspot_logical.x)as f32; let py=(position_logical.y-hotspot_logical.y)as f32; let pw=texture_arc.width_px()as f32; let ph=texture_arc.height_px()as f32; processed_elements.push(RenderElementProcessed::Texture { texture_dyn:texture_arc.clone(), physical_rect:SmithayRectangle::from_loc_and_size(SmithayPoint::from((px,py)),SmithaySize::from((pw,ph))), tint_color:[1.0,1.0,1.0,1.0],}); } } }

        self.record_composition_pass(current_command_buffer, self.current_frame_index, &processed_elements)?;

        let mut current_input_view = self.scene_image_view;
        let mut final_pp_output_view = self.scene_image_view;

        if !self.post_process_passes.is_empty() {
            for (i, pass_config) in self.post_process_passes.iter().enumerate() {
                let target_pp_fb; let result_view_of_this_pass;
                if i % 2 == 0 { target_pp_fb = self.post_process_fb_a; result_view_of_this_pass = self.post_process_image_a_view; }
                else { target_pp_fb = self.post_process_fb_b; result_view_of_this_pass = self.post_process_image_b_view; }
                let pp_input_sampler_info = vk::DescriptorImageInfo::builder().sampler(self.default_sampler).image_view(current_input_view).image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
                let pp_input_write = vk::WriteDescriptorSet::builder().dst_set(self.post_process_input_descriptor_sets[self.current_frame_index]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&pp_input_sampler_info));
                unsafe { self.logical_device_raw.update_descriptor_sets(&[pp_input_write.build()], &[]); }

                let pp_clear_values = [vk::ClearValue { color: vk::ClearColorValue { float32: [0.0,0.0,0.0,0.0] } }];
                let pp_rp_begin_info = vk::RenderPassBeginInfo::builder().render_pass(self.post_process_render_pass.raw).framebuffer(target_pp_fb).render_area(vk::Rect2D { offset: vk::Offset2D::default(), extent: self.post_process_image_extent }).clear_values(&pp_clear_values);
                unsafe {
                    self.logical_device_raw.cmd_begin_render_pass(current_command_buffer, &pp_rp_begin_info, vk::SubpassContents::INLINE);
                    self.logical_device_raw.cmd_bind_pipeline(current_command_buffer, vk::PipelineBindPoint::GRAPHICS, pass_config.pipeline);
                    self.logical_device_raw.cmd_bind_vertex_buffers(current_command_buffer, 0, &[self.vertex_buffer], &[0]);
                    self.logical_device_raw.cmd_bind_index_buffer(current_command_buffer, self.index_buffer, 0, vk::IndexType::UINT16);
                    self.logical_device_raw.cmd_bind_descriptor_sets(current_command_buffer, vk::PipelineBindPoint::GRAPHICS, pass_config.pipeline_layout, 0, &[self.post_process_input_descriptor_sets[self.current_frame_index]], &[]);
                    match pass_config.effect_type { PostProcessEffectType::GammaCorrection => { let gamma_values = pipeline::GammaPushConstants { gamma_value: 2.2 }; self.logical_device_raw.cmd_push_constants(current_command_buffer,pass_config.pipeline_layout, vk::ShaderStageFlags::FRAGMENT,0,bytemuck::bytes_of(&gamma_values),); } }
                    let viewport = vk::Viewport {x:0.0,y:0.0,width:self.post_process_image_extent.width as f32,height:self.post_process_image_extent.height as f32,min_depth:0.0,max_depth:1.0};
                    self.logical_device_raw.cmd_set_viewport(current_command_buffer,0,&[viewport]);
                    let scissor = vk::Rect2D{offset:vk::Offset2D::default(),extent:self.post_process_image_extent};
                    self.logical_device_raw.cmd_set_scissor(current_command_buffer,0,&[scissor]);
                    self.logical_device_raw.cmd_draw_indexed(current_command_buffer, self.index_count, 1, 0, 0, 0);
                    self.logical_device_raw.cmd_end_render_pass(current_command_buffer);
                }
                current_input_view = result_view_of_this_pass;
            }
            final_pp_output_view = current_input_view;
        }
        
        let blit_input_sampler_info = vk::DescriptorImageInfo::builder().sampler(self.default_sampler).image_view(final_pp_output_view).image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        let blit_ds_update_write = vk::WriteDescriptorSet::builder().dst_set(self.blit_descriptor_sets[self.current_frame_index]).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&blit_input_sampler_info));
        unsafe { self.logical_device_raw.update_descriptor_sets(&[blit_ds_update_write.build()], &[]); }
        self.record_blit_to_swapchain_pass(current_command_buffer, swapchain_image_index, self.current_frame_index, final_pp_output_view)?;

        unsafe { self.logical_device_raw.end_command_buffer(current_command_buffer) }?;

        let wait_semaphores_raw = [current_sync_primitives.image_available_semaphore];
        let wait_semaphore_values = [image_available_target_value];
        let signal_semaphores_raw = [current_sync_primitives.render_finished_semaphore];
        let signal_semaphore_values = [render_finished_target_value];

        let mut timeline_submit_info = vk::TimelineSemaphoreSubmitInfo::builder()
            .wait_semaphore_values(&wait_semaphore_values)
            .signal_semaphore_values(&signal_semaphore_values);

        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores_raw)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&[current_command_buffer])
            .signal_semaphores(&signal_semaphores_raw)
            .p_next(&mut timeline_submit_info as *mut _ as *mut std::ffi::c_void);
        
        let graphics_queue = self.logical_device.queues.graphics_queue;
        unsafe { self.logical_device_raw.queue_submit(graphics_queue, &[submit_info.build()], current_sync_primitives.in_flight_fence) }?;
        Ok(())
    }

    fn present_frame(&mut self) -> Result<(), AbstractionRendererError> {
        if let Some(last_time) = self.last_present_time {
            let elapsed_since_last_present = last_time.elapsed();
            if elapsed_since_last_present < self.target_frame_duration {
                let sleep_duration = self.target_frame_duration - elapsed_since_last_present;
                std::thread::sleep(sleep_duration);
                debug!("Frame pacing: slept for {:?} to meet target rate.", sleep_duration);
            }
        }

        if self.swapchain_suboptimal || self.last_acquired_image_index.is_none() {
            warn!("Swapchain suboptimal or no image acquired prior to present. Attempting recreation.");
            let recreate_res = self.recreate_swapchain().map_err(VulkanError::into);
            self.last_acquired_image_index = None; 
            self.last_present_time = Some(Instant::now());
            return recreate_res;
        }
        let image_index = self.last_acquired_image_index.unwrap();
        let current_sync_primitives = &self.sync_primitives[self.current_frame_index];

        let wait_semaphores_raw = [current_sync_primitives.render_finished_semaphore];
        let render_finished_target_value_for_this_frame = self.timeline_value + 1;
        let wait_timeline_values = [render_finished_target_value_for_this_frame];
        let mut timeline_present_info = vk::TimelineSemaphoreSubmitInfo::builder()
            .wait_semaphore_values(&wait_timeline_values);

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores_raw)
            .swapchains(&[self.surface_swapchain.swapchain_khr()])
            .image_indices(&[image_index])
            .p_next(&mut timeline_present_info as *mut _ as *mut std::ffi::c_void);
        
        let present_queue = self.logical_device.queues.present_queue;
        let present_result = unsafe { self.surface_swapchain.swapchain_loader.queue_present(present_queue, &present_info) };

        self.last_present_time = Some(Instant::now());

        let mut needs_recreation = false;
        match present_result {
            Ok(suboptimal) if suboptimal || self.swapchain_suboptimal => { needs_recreation = true; warn!("Swapchain suboptimal after present."); }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => { needs_recreation = true; warn!("Swapchain out of date/suboptimal after present."); }
            Err(e) => return Err(VulkanError::from(e).into()), _ => {}
        }

        self.timeline_value = render_finished_target_value_for_this_frame;

        if needs_recreation {
            self.swapchain_suboptimal = true;
        } else {
            self.swapchain_suboptimal = false;
        }
        self.current_frame_index = (self.current_frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
        self.last_acquired_image_index = None; 
        Ok(())
    }
    fn create_texture_from_shm(&mut self, buffer: &WlBuffer) -> Result<Box<dyn AbstractionRenderableTexture>, AbstractionRendererError> { self.import_shm_texture(buffer) }
    fn create_texture_from_dmabuf(&mut self, dmabuf_attributes: &Dmabuf) -> Result<Box<dyn AbstractionRenderableTexture>, AbstractionRendererError> { self.import_dmabuf_texture(dmabuf_attributes) }
    fn screen_size(&self) -> SmithaySize<i32, Physical> { let extent = self.surface_swapchain.extent(); SmithaySize::from((extent.width as i32, extent.height as i32)) }
}

impl FrameRenderer {
    pub fn import_dmabuf_texture( &mut self, attributes: &Dmabuf ) -> Result<Box<dyn AbstractionRenderableTexture>, AbstractionRendererError> { Ok(Box::new(texture::VulkanRenderableTexture::new( vk::Image::null(), vk::ImageView::null(), None, None, self.default_sampler, vk::Format::UNDEFINED, 0,0, vk::ImageLayout::UNDEFINED, self.logical_device.raw.clone(), None, ))) }
    pub fn import_shm_texture( &mut self, buffer: &WlBuffer, ) -> Result<Box<dyn AbstractionRenderableTexture>, AbstractionRendererError> { Ok(Box::new(texture::VulkanRenderableTexture::new( vk::Image::null(), vk::ImageView::null(), None, None, self.default_sampler, vk::Format::UNDEFINED, 0,0, vk::ImageLayout::UNDEFINED, self.logical_device.raw.clone(), Some(self.allocator.clone()), ))) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::renderer::vulkan::allocator::Allocator;
    use crate::compositor::renderer::vulkan::device::LogicalDevice; // Already present
    use crate::compositor::renderer::vulkan::instance::VulkanInstance; // Already present
    use crate::compositor::renderer::vulkan::physical_device::{PhysicalDeviceInfo, QueueFamilyIndices}; // Already present
    use crate::compositor::renderer::vulkan::render_pass::RenderPass; // Already present
    // No SurfaceSwapchain mock needed for this focused test.
    use crate::compositor::renderer_interface::abstraction::{RenderableTexture, RenderElement}; // RenderElement might not be needed
    use crate::compositor::surface_management::{SurfaceData, SurfaceAttributes, TextureHandle};
    use crate::compositor::roles::RoleData;
    use crate::compositor::renderer_interface::RendererSurfaceState;


    use ash::{vk, Entry};
    use smithay::reexports::wayland_server::{
        protocol::wl_surface,
        Display, DisplayHandle, Client, Global, Main,
        Resource, UserDataMap,
    };
    use smithay::utils::{Point as SmithayPoint, Size as SmithaySize, Rectangle as SmithayRectangle, Physical, Logical, Buffer, Transform as SmithayTransform};
    use std::any::Any;
    use std::sync::{Arc, Mutex};
    // Uuid already imported from parent
    // HashMap already imported from parent
    // CString already imported from parent
    use std::ptr;


    #[derive(Debug)]
    struct MockVulkanRenderableTexture {
        id: Uuid,
        image: vk::Image,
        image_view: vk::ImageView,
        memory: Option<vk_mem::Allocation>,
        sampler: vk::Sampler,
        device_raw: ash::Device,
        allocator_arc: Option<Arc<Allocator>>,
        width: u32,
        height: u32,
    }

    impl MockVulkanRenderableTexture {
        fn new(device_raw: ash::Device, allocator: Option<Arc<Allocator>>, default_sampler: vk::Sampler, width: u32, height: u32) -> Self {
            let mut dummy_image = vk::Image::null();
            let mut dummy_image_view = vk::ImageView::null();
            let mut dummy_memory_alloc: Option<vk_mem::Allocation> = None;

            if let Some(alloc_ref) = allocator.as_ref() { // Use as_ref for Arc
                let image_create_info = vk::ImageCreateInfo::builder()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(vk::Format::R8G8B8A8_UNORM)
                    .extent(vk::Extent3D { width: 1, height: 1, depth: 1 }) // Minimal 1x1 image
                    .mip_levels(1).array_layers(1).samples(vk::SampleCountFlags::TYPE_1)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::COLOR_ATTACHMENT) // Sampled as input, CA if it were a render target itself
                    .initial_layout(vk::ImageLayout::UNDEFINED);
                let allocation_info = vk_mem::AllocationCreateInfo {
                    usage: vk_mem::MemoryUsage::GpuOnly, ..Default::default()
                };
                match alloc_ref.create_image(&image_create_info, &allocation_info) {
                    Ok((img, alloc_handle, _)) => {
                        dummy_image = img;
                        dummy_memory_alloc = Some(alloc_handle);
                        let view_info = vk::ImageViewCreateInfo::builder()
                            .image(dummy_image).view_type(vk::ImageViewType::TYPE_2D)
                            .format(vk::Format::R8G8B8A8_UNORM)
                            .subresource_range(vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR).base_mip_level(0).level_count(1)
                                .base_array_layer(0).layer_count(1).build());
                        dummy_image_view = unsafe { device_raw.create_image_view(&view_info, None) }.unwrap_or(vk::ImageView::null());
                    }
                    Err(e) => { println!("MockVulkanRenderableTexture: Failed to create dummy image: {:?}", e); }
                }
            }

            Self {
                id: Uuid::new_v4(), image: dummy_image, image_view: dummy_image_view, memory: dummy_memory_alloc,
                sampler: default_sampler, device_raw: device_raw.clone(), allocator_arc: allocator, width, height,
            }
        }
    }
    impl RenderableTexture for MockVulkanRenderableTexture {
        fn id(&self) -> Uuid { self.id }
        fn descriptor_image_info(&self) -> vk::DescriptorImageInfo {
            vk::DescriptorImageInfo { sampler: self.sampler, image_view: self.image_view, image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, }
        }
        fn width_px(&self) -> u32 { self.width }
        fn height_px(&self) -> u32 { self.height }
        fn estimated_gpu_memory_size(&self) -> u64 { if self.image != vk::Image::null() { 1 * 1 * 4 } else { 0 } + (self.width * self.height * 4) as u64 }
        fn as_any(&self) -> &dyn Any { self }
    }
    impl Drop for MockVulkanRenderableTexture {
        fn drop(&mut self) {
            if self.image_view != vk::ImageView::null() { unsafe { self.device_raw.destroy_image_view(self.image_view, None); } }
            if self.image != vk::Image::null() { if let (Some(allocator), Some(mem_alloc)) = (self.allocator_arc.as_ref(), self.memory.take()) { allocator.destroy_image(self.image, &mem_alloc); } }
        }
    }

    // ANCHOR[HeadlessVulkanTestSetup_FrameRendererTests]
    fn setup_headless_vulkan_for_test() -> Result<(Arc<VulkanInstance>, Arc<PhysicalDeviceInfo>, Arc<LogicalDevice>, Arc<Allocator>, vk::CommandPool, vk::Queue), String> {
        let entry = Entry::linked();
        let instance = Arc::new(VulkanInstance::new_headless(&entry).map_err(|e| format!("Instance creation failed: {}", e))?);
        let pdevices = PhysicalDeviceInfo::enumerate(instance.raw()).map_err(|e| format!("PDevice enumeration failed: {}", e))?;
        let physical_device_info = Arc::new(pdevices.into_iter()
            .find(|p| p.is_suitable_for_graphics_and_compute()) // Ensure compute for some shaders if needed by FrameRenderer internals
            .ok_or_else(|| "No suitable physical device found".to_string())?);

        // Ensure timeline semaphore feature is requested (as FrameRenderer::new enables it)
        let mut timeline_semaphore_features = vk::PhysicalDeviceTimelineSemaphoreFeatures::builder().timeline_semaphore(true);
        let mut features2 = vk::PhysicalDeviceFeatures2::builder().p_next(&mut timeline_semaphore_features as *mut _ as *mut _);
        // Query features to ensure they are supported (though PhysicalDeviceInfo should ideally do this)
        unsafe { instance.raw().get_physical_device_features2(physical_device_info.physical_device, &mut features2) };
        if timeline_semaphore_features.timeline_semaphore != vk::TRUE {
            return Err("Timeline semaphore feature not supported by physical device".to_string());
        }

        let queue_family_indices = QueueFamilyIndices::find(instance.raw(), physical_device_info.physical_device, None)
            .ok_or_else(|| "Required queue families not found".to_string())?;
        if queue_family_indices.graphics_family.is_none() {
             return Err("Graphics queue family missing".to_string());
        }
        let logical_device = Arc::new(LogicalDevice::new(instance.raw(), physical_device_info.clone(), queue_family_indices)
            .map_err(|e| format!("LogicalDevice creation failed: {}", e))?); // This new should enable timelineSemaphore feature

        let allocator = Arc::new(Allocator::new(instance.raw(), physical_device_info.raw(), logical_device.raw.clone())
            .map_err(|e| format!("Allocator creation failed: {}", e))?);

        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_indices.graphics_family.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe { logical_device.raw.create_command_pool(&pool_create_info, None) }
            .map_err(|e| format!("CommandPool creation failed: {}", e))?;

        let graphics_queue = unsafe { logical_device.raw.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0) };

        Ok((instance, physical_device_info, logical_device, allocator, command_pool, graphics_queue))
    }

    #[test]
    // ANCHOR[GammaPass_CommandRecordingTest_FrameRenderer]
    fn test_gamma_correction_pass_command_recording() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let (instance, _pdevice_info, device, allocator, command_pool, graphics_queue) =
            setup_headless_vulkan_for_test().expect("Vulkan headless setup failed for test");

        let device_raw = &device.raw;
        let test_extent = vk::Extent2D { width: 1, height: 1 };
        let test_format = vk::Format::R8G8B8A8_UNORM;

        // 1. Render Pass (color-only for post-processing)
        let pp_render_pass = RenderPass::new_color_only(&device, test_format, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .expect("Failed to create post-process render pass");

        // 2. Dummy Target Image & Framebuffer
        let target_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED; // Sampled if it were input to another pass
        let (gamma_target_image, gamma_target_alloc, gamma_target_image_view) =
            FrameRenderer::create_render_target_texture_resources(device_raw, &allocator, test_extent, test_format, target_usage)
            .expect("Failed to create gamma target image");
        let gamma_pass_fb = FrameRenderer::create_color_only_framebuffer(device_raw, pp_render_pass.raw, gamma_target_image_view, test_extent)
            .expect("Failed to create gamma pass framebuffer");

        // 3. Descriptor Set Layout (for input texture)
        let pp_sampler_binding = vk::DescriptorSetLayoutBinding::builder().binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let pp_dsl_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(std::slice::from_ref(&pp_sampler_binding));
        let pp_dsl = unsafe { device_raw.create_descriptor_set_layout(&pp_dsl_create_info, None) }.expect("Failed to create PP DSL");

        // 4. Pipeline Layout (DSL + Push Constants for Gamma)
        let pp_push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT).offset(0).size(std::mem::size_of::<GammaPushConstants>() as u32);
        let gamma_pipeline_layout_obj = PipelineLayout::new(device.as_ref(), &[pp_dsl], &[pp_push_constant_range.build()])
            .expect("Failed to create gamma pipeline layout");

        // 5. Shaders & Pipeline
        let vert_shader_code = pipeline::load_spirv_file("assets/shaders/triangle.vert.spv")?; // Assuming this is a fullscreen quad vert shader
        let vert_shader_module = pipeline::create_shader_module(device_raw, &vert_shader_code)?;
        let gamma_frag_code = pipeline::load_spirv_file("assets/shaders/gamma_correction.frag.spv")?;
        let gamma_frag_module = pipeline::create_shader_module(device_raw, &gamma_frag_code)?;

        let gamma_pipeline = GraphicsPipeline::new(device.as_ref(), pp_render_pass.raw, test_extent, gamma_pipeline_layout_obj.clone(), vert_shader_module, gamma_frag_module, vk::PipelineCache::null(), false, false)
            .expect("Failed to create gamma pipeline");

        unsafe { device_raw.destroy_shader_module(vert_shader_module, None); device_raw.destroy_shader_module(gamma_frag_module, None); }

        // 6. Descriptor Pool & Set
        let pool_sizes = [vk::DescriptorPoolSize { ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, descriptor_count: 1 }];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder().pool_sizes(&pool_sizes).max_sets(1);
        let descriptor_pool = unsafe { device_raw.create_descriptor_pool(&descriptor_pool_info, None) }.expect("Failed to create descriptor pool");
        let desc_set_alloc_info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(std::slice::from_ref(&pp_dsl));
        let input_texture_ds = unsafe { device_raw.allocate_descriptor_sets(&desc_set_alloc_info) }.expect("Failed to allocate descriptor set")[0];

        // 7. Mock Input Texture & Sampler
        let default_sampler_info = vk::SamplerCreateInfo::builder().mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR);
        let default_sampler = unsafe { device_raw.create_sampler(&default_sampler_info, None) }.expect("Failed to create sampler");
        let mock_input_texture = MockVulkanRenderableTexture::new(device_raw.clone(), Some(allocator.clone()), default_sampler, 1, 1);

        let input_sampler_info = vk::DescriptorImageInfo::builder().sampler(default_sampler).image_view(mock_input_texture.image_view).image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        let input_write = vk::WriteDescriptorSet::builder().dst_set(input_texture_ds).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&input_sampler_info));
        unsafe { device_raw.update_descriptor_sets(&[input_write.build()], &[]); }

        // 8. Vertex/Index Buffers for Fullscreen Quad
        let quad_vertices = [ Vertex { pos: [-1.0, -1.0], tex_coord: [0.0, 0.0] }, Vertex { pos: [1.0, -1.0], tex_coord: [1.0, 0.0] }, Vertex { pos: [1.0, 1.0], tex_coord: [1.0, 1.0] }, Vertex { pos: [-1.0, 1.0], tex_coord: [0.0, 1.0] }, ];
        let quad_indices: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let (vertex_buffer, vertex_buffer_alloc) = buffer_utils::create_and_fill_gpu_buffer(&allocator, &device, command_pool, graphics_queue, &quad_vertices, vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let (index_buffer, index_buffer_alloc) = buffer_utils::create_and_fill_gpu_buffer(&allocator, &device, command_pool, graphics_queue, &quad_indices, vk::BufferUsageFlags::INDEX_BUFFER)?;


        // 9. Command Buffer Recording
        let cmd_alloc_info = vk::CommandBufferAllocateInfo::builder().command_pool(command_pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(1);
        let cmd_buffer = unsafe { device_raw.allocate_command_buffers(&cmd_alloc_info) }.expect("Failed to allocate cmd buffer")[0];
        let cmd_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device_raw.begin_command_buffer(cmd_buffer, &cmd_begin_info) }.expect("Begin CB failed");

        let clear_values = [vk::ClearValue { color: vk::ClearColorValue { float32: [0.0,0.0,0.0,0.0] } }];
        let rp_begin_info = vk::RenderPassBeginInfo::builder().render_pass(pp_render_pass.raw).framebuffer(gamma_pass_fb)
            .render_area(vk::Rect2D { offset: vk::Offset2D::default(), extent: test_extent }).clear_values(&clear_values);
        unsafe {
            device_raw.cmd_begin_render_pass(cmd_buffer, &rp_begin_info, vk::SubpassContents::INLINE);
            device_raw.cmd_bind_pipeline(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, gamma_pipeline.raw);
            device_raw.cmd_bind_vertex_buffers(cmd_buffer, 0, &[vertex_buffer], &[0]);
            device_raw.cmd_bind_index_buffer(cmd_buffer, index_buffer, 0, vk::IndexType::UINT16);
            device_raw.cmd_bind_descriptor_sets(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, gamma_pipeline_layout_obj.raw, 0, &[input_texture_ds], &[]);

            let gamma_values = GammaPushConstants { gamma_value: 2.2 };
            device_raw.cmd_push_constants(cmd_buffer, gamma_pipeline_layout_obj.raw, vk::ShaderStageFlags::FRAGMENT, 0, bytemuck::bytes_of(&gamma_values));

            let viewport = vk::Viewport {x:0.0,y:0.0,width:test_extent.width as f32,height:test_extent.height as f32,min_depth:0.0,max_depth:1.0};
            device_raw.cmd_set_viewport(cmd_buffer,0,&[viewport]);
            let scissor = vk::Rect2D{offset:vk::Offset2D::default(),extent:test_extent};
            device_raw.cmd_set_scissor(cmd_buffer,0,&[scissor]);
            device_raw.cmd_draw_indexed(cmd_buffer, quad_indices.len() as u32, 1, 0, 0, 0);
            device_raw.cmd_end_render_pass(cmd_buffer);
        }
        unsafe { device_raw.end_command_buffer(cmd_buffer) }.expect("End CB failed");

        // 10. Submission (Optional but good for validation)
        let submit_info = vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&cmd_buffer));
        unsafe { device_raw.queue_submit(graphics_queue, &[submit_info.build()], vk::Fence::null()) }.expect("Queue submit failed");
        unsafe { device_raw.queue_wait_idle(graphics_queue) }.expect("Queue wait idle failed");

        // 11. Cleanup
        unsafe {
            device_raw.free_command_buffers(command_pool, &[cmd_buffer]);
            allocator.destroy_buffer(vertex_buffer, &vertex_buffer_alloc);
            allocator.destroy_buffer(index_buffer, &index_buffer_alloc);
            // MockVulkanRenderableTexture's Drop handles its image/view/memory
            device_raw.destroy_sampler(default_sampler, None);
            device_raw.destroy_descriptor_pool(descriptor_pool, None);
            // gamma_pipeline is GraphicsPipeline struct, its Drop impl handles pipeline and layout (gamma_pipeline_layout_obj)
            // pp_render_pass is RenderPass struct, its Drop impl handles render pass
            device_raw.destroy_descriptor_set_layout(pp_dsl, None);
            device_raw.destroy_framebuffer(gamma_pass_fb, None);
            device_raw.destroy_image_view(gamma_target_image_view, None);
            allocator.destroy_image(gamma_target_image, &gamma_target_alloc);
            device_raw.destroy_command_pool(command_pool, None);
            // Device, Allocator, Instance are Arcs, will drop when out of scope.
        }
        Ok(())
    }

    #[test]
    #[ignore = "Full FrameRenderer setup, especially SurfaceSwapchain, is too complex for this test iteration. Test focuses on logic after setup."]
    fn test_record_graphics_pass_with_mock_elements() { /* ... (same as before) ... */ }
}
