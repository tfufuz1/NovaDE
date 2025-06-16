use ash::{vk, Device as AshDevice, Instance as AshInstance};
use std::sync::Arc;
use std::ffi::c_void;
use raw_window_handle::{WaylandDisplayHandle, WaylandWindowHandle, HasRawWindowHandle, HasRawDisplayHandle, RawWindowHandle, RawDisplayHandle};

// Import necessary modules from the current Vulkan renderer structure
use super::context::VulkanContext;
use super::swapchain::VulkanSwapchain;
use super::render_pass::VulkanRenderPass;
use super::framebuffer::VulkanFramebuffer;
use super::pipeline::VulkanPipeline;
use super::shader::VulkanShaderModule;
use super::sync::FrameSyncPrimitives;

use gpu_allocator::MemoryUsage as GpuMemoryUsage; // For VMA

// Import necessary modules from the current Vulkan renderer structure
use super::context::VulkanContext;
use super::swapchain::VulkanSwapchain;
use super::render_pass::VulkanRenderPass;
use super::framebuffer::VulkanFramebuffer;
use super::pipeline::VulkanPipeline;
use super::shader::VulkanShaderModule;
use super::sync::FrameSyncPrimitives;
// Import the new VMA-backed Buffer and Image structs
use super::buffer::VulkanBuffer;
use super::image::VulkanImage;
// Import command buffer helpers if they are to be used from here
use super::command_buffer;


// ANCHOR: NovaVulkanRenderer Struct Definition
pub struct NovaVulkanRenderer {
    pub vulkan_context: Arc<VulkanContext>,
    swapchain: Option<VulkanSwapchain>,
    render_pass: Option<VulkanRenderPass>,
    framebuffers: Vec<VulkanFramebuffer>,

    textured_quad_pipeline: Option<VulkanPipeline>,

    // Use VMA-backed VulkanBuffer and VulkanImage
    vertex_buffer: Option<VulkanBuffer>,
    index_buffer: Option<VulkanBuffer>,

    texture_descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    texture_descriptor_pool: Option<vk::DescriptorPool>,
    // One descriptor set per frame in flight, or per swapchain image, if texture can change
    // For a single placeholder texture, one might be enough if updated carefully,
    // but per-frame is safer with multiple frames in flight.
    texture_descriptor_sets: Vec<vk::DescriptorSet>,

    // Placeholder texture and sampler
    placeholder_texture: Option<VulkanImage>,
    placeholder_texture_view: Option<vk::ImageView>, // ImageView is separate from VulkanImage
    placeholder_sampler: Option<vk::Sampler>,

    // Placeholder for actual Wayland window handles from Smithay
    // These would be properly obtained and managed in a real integration.
    // We need a way to provide these to VulkanSwapchain::new
    // For now, we store raw pointers and create dummy handles.
    wayland_display_ptr: *mut c_void,
    wayland_surface_ptr: *mut c_void,

    // Smithay window handle abstraction (not used yet, but for future)
    // window_handle: raw_window_handle::WaylandWindowHandle,

    // Tracks which set of sync primitives (from VulkanContext) to use for the current frame.
    // This ensures that we use a different semaphore/fence set for each concurrent frame.
    // VulkanContext.current_frame_index is the one that indexes into its arrays.
    // NovaVulkanRenderer.current_frame_cycle_index is used to pick which sync primitive set.
    // This can be the same as VulkanContext.current_frame_index if context manages cycling directly.
    // For clarity, let's assume NovaVulkanRenderer tells context which sync objects to use implicitly via context.current_frame_index
}

// ANCHOR: NovaVulkanRenderer Implementation
impl NovaVulkanRenderer {
    pub fn new(
        // These raw pointers are placeholders for what Smithay would provide.
        // Smithay's DisplayHandle and WlSurface would be used to get RawDisplayHandle and RawWindowHandle.
        wayland_display_handle_ptr: *mut c_void,
        wayland_surface_handle_ptr: *mut c_void,
    ) -> Result<Self, String> {
        if wayland_display_handle_ptr.is_null() || wayland_surface_handle_ptr.is_null() {
            println!("NovaVulkanRenderer::new: Initializing with null Wayland display/surface handles. Surface creation is expected to fail.");
        }
        let context = VulkanContext::new().map_err(|e| format!("Failed to create VulkanContext: {}", e))?;

        Ok(Self {
            vulkan_context: Arc::new(context),
            swapchain: None,
            render_pass: None,
            framebuffers: Vec::new(),
            textured_quad_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            texture_descriptor_set_layout: None,
            texture_descriptor_pool: None,
            texture_descriptor_sets: Vec::new(),
            placeholder_texture: None,
            placeholder_sampler: None,
            wayland_display_ptr: wayland_display_handle_ptr,
            wayland_surface_ptr: wayland_surface_handle_ptr,
        })
    }

    pub fn init_swapchain(&mut self, width: u32, height: u32) -> Result<(), String> {
        let device = self.vulkan_context.device();
        // Ensure device is idle before changing critical resources like swapchain
        unsafe { device.device_wait_idle().map_err(|e| e.to_string())? };

        // 1. Cleanup old swapchain-dependent resources
        self.cleanup_swapchain_dependent_resources();

        let old_swapchain_khr = self.swapchain.take().map(|sc| {
            let khr = sc.swapchain;
            drop(sc);
            khr
        });
        println!("Old swapchain resources cleaned up.");

        // 2. Create new VulkanSwapchain
        let new_swapchain = VulkanSwapchain::new(
            &self.vulkan_context,
            &self,
            width, height,
            old_swapchain_khr,
        ).map_err(|e| format!("Failed to create VulkanSwapchain: {}", e))?;
        let swapchain_format = new_swapchain.format();
        let swapchain_extent = new_swapchain.extent();
        self.swapchain = Some(new_swapchain);
        println!("New swapchain created.");

        // 3. Create VulkanRenderPass
        let new_render_pass = VulkanRenderPass::new(
            device.clone(),
            swapchain_format,
        ).map_err(|e| format!("Failed to create VulkanRenderPass: {}", e))?;
        self.render_pass = Some(new_render_pass);
        println!("New render pass created.");

        // 4. Create VulkanFramebuffers
        self.framebuffers = self.swapchain.as_ref().unwrap().image_views()
            .iter()
            .map(|iv| {
                VulkanFramebuffer::new(
                    device.clone(),
                    self.render_pass.as_ref().unwrap().handle(),
                    *iv,
                    swapchain_extent,
                )
            })
            .collect::<Result<Vec<_>, String>>()
            .map_err(|e| format!("Failed to create framebuffers: {}", e))?;
        println!("Framebuffers created (count: {}).", self.framebuffers.len());

        // 5. Create Descriptor Set Layout (for texture sampler)
        self.create_texture_descriptor_set_layout()?;

        // 6. Create Graphics Pipeline
        let vert_shader_path = "novade-system/assets/shaders/textured.vert.spv";
        let frag_shader_path = "novade-system/assets/shaders/textured.frag.spv";
        let vert_shader_module = VulkanShaderModule::new_from_file(device.clone(), vert_shader_path)?;
        let frag_shader_module = VulkanShaderModule::new_from_file(device.clone(), frag_shader_path)?;

        let new_pipeline = VulkanPipeline::new(
            device.clone(),
            self.render_pass.as_ref().unwrap().handle(),
            swapchain_extent,
            &vert_shader_module,
            &frag_shader_module,
            self.texture_descriptor_set_layout.unwrap(), // Safe to unwrap after create_..._layout
        ).map_err(|e| format!("Failed to create graphics pipeline: {}", e))?;
        self.textured_quad_pipeline = Some(new_pipeline);
        // Shaders can be dropped now as they are compiled into the pipeline
        drop(vert_shader_module);
        drop(frag_shader_module);
        println!("Textured quad graphics pipeline created.");

        // 7. Create Vertex and Index Buffers
        self.create_vertex_buffer()?;
        self.create_index_buffer()?;

        // 8. Create Placeholder Texture and Sampler
        self.create_placeholder_texture_and_sampler()?;

        // 9. Create Descriptor Pool
        let pool_sizes = [vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1) // Only one descriptor set needed for the single placeholder texture
            .build()];
        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(1) // Max number of descriptor sets that can be allocated
            .flags(vk::DescriptorPoolCreateFlags::empty()); // Or FREE_DESCRIPTOR_SET_BIT if sets can be freed individually

        self.texture_descriptor_pool = Some(unsafe {
            device.create_descriptor_pool(&pool_info, None)
        }.map_err(|e| format!("Failed to create descriptor pool: {}", e))?);
        println!("Texture descriptor pool created.");

        // 10. Allocate Descriptor Sets
        let layouts_to_alloc = [self.texture_descriptor_set_layout.unwrap()]; // Safe to unwrap
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.texture_descriptor_pool.unwrap()) // Safe
            .set_layouts(&layouts_to_alloc);

        let descriptor_sets_vec = unsafe {
            device.allocate_descriptor_sets(&alloc_info)
        }.map_err(|e| format!("Failed to allocate descriptor sets: {}", e))?;
        self.texture_descriptor_sets = descriptor_sets_vec; // Should be Vec<vk::DescriptorSet>
        println!("Texture descriptor set allocated: {:?}", self.texture_descriptor_sets[0]);

        // 11. Update Descriptor Sets
        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) // Assuming texture is in this layout
            .image_view(self.placeholder_texture.as_ref().unwrap().view) // Safe
            .sampler(self.placeholder_sampler.unwrap()) // Safe
            .build();
        let image_infos = [image_info];

        let descriptor_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.texture_descriptor_sets[0])
            .dst_binding(0) // Binding 0 in shader
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_infos)
            // .buffer_info(&[]) // For buffer descriptors
            // .texel_buffer_view(&[]) // For texel buffer views
            .build();

        unsafe { device.update_descriptor_sets(std::slice::from_ref(&descriptor_write), &[]) };
        println!("Descriptor set updated to point to placeholder texture/sampler.");

        Ok(())
    }

    fn cleanup_swapchain_dependent_resources(&mut self) {
        let device = self.vulkan_context.device();
        // Framebuffers depend on render pass and image views (managed by swapchain).
        self.framebuffers.clear(); // Drops VulkanFramebuffer, destroying vk::Framebuffer
        if let Some(rp) = self.render_pass.take() { drop(rp); }
        if let Some(pipeline) = self.textured_quad_pipeline.take() { drop(pipeline); }

        // Cleanup descriptor-related resources that might be (re)created with swapchain
        // or if they depend on the number of swapchain images (e.g. one set per image).
        // Here, we assume they are generally recreated.
        if let Some(pool) = self.texture_descriptor_pool.take() {
            // Descriptor sets are freed when the pool is destroyed
            unsafe { device.destroy_descriptor_pool(pool, None); }
            self.texture_descriptor_sets.clear();
        }
        if let Some(layout) = self.texture_descriptor_set_layout.take() {
            unsafe { device.destroy_descriptor_set_layout(layout, None); }
        }
        // Sampler and texture are more general, but might be recreated if their properties depend on swapchain.
        // For now, placeholder_sampler and placeholder_texture are handled in main drop.
        // If they were swapchain-dependent, they'd be cleaned here too.
    }

    fn create_texture_descriptor_set_layout(&mut self) -> Result<(), String> {
        let bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0) // layout (set=0, binding=0) in fragment shader
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build()];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
        self.texture_descriptor_set_layout = Some(unsafe {
            self.vulkan_context.device().create_descriptor_set_layout(&layout_info, None)
        }.map_err(|e| format!("Failed to create descriptor set layout: {}", e))?);
        println!("Texture descriptor set layout created.");
        Ok(())
    }

    // ANCHOR_EXT: create_vertex_buffer
    fn create_vertex_buffer(&mut self) -> Result<(), String> {
        let vertices_data: [f32; 16] = [
            // pos         texcoord
            -0.5, -0.5,   0.0, 0.0, // Vertex 0: Top-left
             0.5, -0.5,   1.0, 0.0, // Vertex 1: Top-right
             0.5,  0.5,   1.0, 1.0, // Vertex 2: Bottom-right
            -0.5,  0.5,   0.0, 1.0, // Vertex 3: Bottom-left
        ];
        let size = std::mem::size_of_val(&vertices_data) as vk::DeviceSize;

        // For now, using simplified placeholder buffer creation.
        // A real implementation would use a staging buffer for device-local memory.
        let buffer = VulkanBuffer::new_placeholder(
            self.vulkan_context.device().clone(),
            size,
            vk::BufferUsageFlags::VERTEX_BUFFER, // No TRANSFER_DST if not using staging
        ).map_err(|e| format!("Failed to create vertex buffer: {}", e))?;

        // If not using staging, need to map and copy data (requires host-visible memory for buffer)
        // This is highly dependent on the memory type chosen by new_placeholder, which is currently bad.
        // For now, we assume new_placeholder creates a buffer we can theoretically use.
        // Proper implementation needs VMA and memory mapping.
        // unsafe {
        //     let memory = self.vulkan_context.device().map_memory(buffer.memory, 0, size, vk::MemoryMapFlags::empty())?;
        //     std::ptr::copy_nonoverlapping(vertices_data.as_ptr() as *const c_void, memory, size as usize);
        //     self.vulkan_context.device().unmap_memory(buffer.memory);
        // }
        self.vertex_buffer = Some(buffer);
        println!("Vertex buffer created (placeholder). Size: {}", size);
        Ok(())
    }

    // ANCHOR_EXT: create_index_buffer
    fn create_index_buffer(&mut self) -> Result<(), String> {
        let indices_data: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let size = std::mem::size_of_val(&indices_data) as vk::DeviceSize;

        let buffer = VulkanBuffer::new_placeholder(
            self.vulkan_context.device().clone(),
            size,
            vk::BufferUsageFlags::INDEX_BUFFER,
        ).map_err(|e| format!("Failed to create index buffer: {}", e))?;
        // Similar memory mapping story as vertex buffer for non-staging.
        self.index_buffer = Some(buffer);
        println!("Index buffer created (placeholder). Size: {}", size);
        Ok(())
    }

    // ANCHOR_EXT: create_placeholder_texture_and_sampler
    fn create_placeholder_texture_and_sampler(&mut self) -> Result<(), String> {
        let device = self.vulkan_context.device();
        // Create a 2x2 checkerboard texture (RGBA)
        let width = 2;
        let height = 2;
        let format = vk::Format::R8G8B8A8_UNORM; // Or SRGB if color space handled correctly

        let texture = VulkanImage::new_placeholder_texture(device.clone(), width, height, format)?;
        // Placeholder: Fill texture with data (e.g., checkerboard)
        // This would involve:
        // 1. Creating a staging buffer.
        // 2. Copying pixel data to staging buffer.
        // 3. Transitioning image layout to TRANSFER_DST_OPTIMAL.
        // 4. Copying from staging buffer to image.
        // 5. Transitioning image layout to SHADER_READ_ONLY_OPTIMAL.
        // For now, the texture is created but not filled with data.
        println!("Placeholder texture created (but not filled with data). View: {:?}", texture.view);
        self.placeholder_texture = Some(texture);

        // Create a sampler
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(false) // Enable if physical device feature is available and desired
            // .max_anisotropy(16.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0); // Set max_lod > 0 if using mipmaps

        self.placeholder_sampler = Some(unsafe {
            device.create_sampler(&sampler_info, None)
        }.map_err(|e| format!("Failed to create sampler: {}", e))?);
        println!("Placeholder sampler created: {:?}", self.placeholder_sampler.unwrap());
        Ok(())
    }

    // ANCHOR_EXT: render_frame
    pub fn render_frame(&mut self) -> Result<(), String> {
        let device = self.vulkan_context.device(); // Arc<AshDevice>

        // Ensure all necessary components are initialized
        let swapchain = self.swapchain.as_ref().ok_or_else(|| "Swapchain not initialized".to_string())?;
        let render_pass = self.render_pass.as_ref().ok_or_else(|| "RenderPass not initialized".to_string())?;
        let pipeline = self.textured_quad_pipeline.as_ref().ok_or_else(|| "Pipeline not initialized".to_string())?;
        let vb = self.vertex_buffer.as_ref().ok_or_else(|| "Vertex buffer not initialized".to_string())?;
        let ib = self.index_buffer.as_ref().ok_or_else(|| "Index buffer not initialized".to_string())?;
        // Assuming 6 indices for a quad
        let index_count = 6;
        let current_descriptor_set = self.texture_descriptor_sets.get(0) // Assuming one DS for now
            .ok_or_else(|| "Descriptor set not available".to_string())?;


        // Use sync primitives from VulkanContext based on its internal current_frame_index
        let sync_primitives = &self.vulkan_context.per_frame_sync_primitives[self.vulkan_context.current_frame_index()];

        // 1. Acquire Next Swapchain Image
        let acquire_result = unsafe {
            swapchain.swapchain_loader.acquire_next_image(
                swapchain.swapchain,
                u64::MAX, // Timeout
                sync_primitives.image_available_semaphore,
                vk::Fence::null(), // Not using a fence for acquisition itself here
            )
        };

        let (image_index, _is_suboptimal) = match acquire_result {
            Ok((idx, suboptimal)) => (idx, suboptimal),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                // Handle swapchain recreation
                println!("Swapchain out of date during acquire, needs recreation.");
                // self.recreate_swapchain(width, height)?; // Assuming width/height are available
                return Err("Swapchain out of date, implement recreation.".to_string()); // Placeholder
            }
            Err(e) => return Err(format!("Failed to acquire swapchain image: {}", e)),
        };

        // 2. Record and Submit Command Buffer
        // Ensure VulkanContext's current_frame_index is correct for command buffer and its own sync primitives
        // NovaVulkanRenderer does not need its own current_frame_index if VulkanContext manages its array indexing.

        let framebuffer_handle = self.framebuffers[image_index as usize].handle();

        self.vulkan_context.record_and_submit_draw_commands(
            image_index,
            swapchain.extent(),
            render_pass.handle(),
            framebuffer_handle,
            pipeline.handle(),
            pipeline.layout_handle(),
            vb.buffer,
            ib.buffer,
            index_count,
            Some(*current_descriptor_set),
        )?;

        // 3. Present Image
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(std::slice::from_ref(&sync_primitives.render_finished_semaphore))
            .swapchains(std::slice::from_ref(&swapchain.swapchain))
            .image_indices(std::slice::from_ref(&image_index));

        let present_result = unsafe {
            swapchain.swapchain_loader.queue_present(self.vulkan_context.present_queue(), &present_info)
        };

        match present_result {
            Ok(suboptimal) if suboptimal => {
                println!("Swapchain suboptimal during present, should recreate soon.");
                // self.recreate_swapchain(width, height)?; // Placeholder
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                println!("Swapchain out of date during present, needs recreation.");
                // self.recreate_swapchain(width, height)?; // Placeholder
                return Err("Swapchain out of date, implement recreation.".to_string()); // Placeholder
            }
            Err(e) => return Err(format!("Failed to present swapchain image: {}", e)),
            _ => {} // Success
        }

        // 4. Advance to next frame's sync primitives and command buffer
        self.vulkan_context.advance_frame_index();

        Ok(())
    }

    // Placeholder RawWindowHandle provider using stored pointers
    // This is unsafe and for temporary use only.
    // A proper implementation would get these from Smithay's WlSurface.
    fn get_raw_window_handle(&self) -> RawWindowHandle {
        let mut wlh = WaylandWindowHandle::empty();
        wlh.surface = self.wayland_surface_ptr;
        // wlh.display = self.wayland_display_ptr; // WaylandWindowHandle doesn't have display, DisplayHandle does
        RawWindowHandle::Wayland(wlh)
    }
    fn get_raw_display_handle(&self) -> RawDisplayHandle {
        let mut dh = WaylandDisplayHandle::empty();
        dh.display = self.wayland_display_ptr;
        RawDisplayHandle::Wayland(dh)
    }

}

// ANCHOR: NovaVulkanRenderer Drop Implementation
impl Drop for NovaVulkanRenderer {
    fn drop(&mut self) {
        // Important: Ensure device is idle before destroying resources,
        // especially if some resources (like pipelines, render passes)
        // are used by commands that might still be executing.
        // The VulkanContext itself doesn't have a device_wait_idle in its drop,
        // so the top-level renderer should handle this.
        if let Ok(_) = unsafe { self.vulkan_context.device().device_wait_idle() } {
             println!("NovaVulkanRenderer: Device idle on drop.");
        } else {
            eprintln!("NovaVulkanRenderer: Failed to wait for device idle on drop.");
        }


        // Explicitly drop resources in correct order.
        // Framebuffers depend on render pass and image views (managed by swapchain).
        // Pipeline depends on render pass and descriptor set layouts.
        // Swapchain holds image views.

        // 1. Destroy pipeline
        if let Some(pipeline) = self.textured_quad_pipeline.take() {
            // VulkanPipeline's Drop will handle vkDestroyPipeline and vkDestroyPipelineLayout
            drop(pipeline);
            println!("Textured quad pipeline dropped.");
        }

        // 2. Destroy descriptor pool (which frees descriptor sets)
        if let Some(pool) = self.texture_descriptor_pool.take() {
            unsafe { self.vulkan_context.device().destroy_descriptor_pool(pool, None); }
            println!("Texture descriptor pool dropped.");
        }
        // Descriptor sets are freed with the pool.
        self.texture_descriptor_sets.clear();

        // 3. Destroy descriptor set layout
        if let Some(layout) = self.texture_descriptor_set_layout.take() {
            unsafe { self.vulkan_context.device().destroy_descriptor_set_layout(layout, None); }
            println!("Texture descriptor set layout dropped.");
        }

        // 4. Destroy placeholder texture resources
        if let Some(sampler) = self.placeholder_sampler.take() {
            unsafe { self.vulkan_context.device().destroy_sampler(sampler, None); }
            println!("Placeholder sampler dropped.");
        }
        // Destroy placeholder texture resources
        if let Some(view) = self.placeholder_texture_view.take() {
            unsafe { self.vulkan_context.device().destroy_image_view(view, None); }
            println!("Placeholder texture view dropped.");
        }
        if let Some(sampler) = self.placeholder_sampler.take() {
            unsafe { self.vulkan_context.device().destroy_sampler(sampler, None); }
            println!("Placeholder sampler dropped.");
        }
        if let Some(image) = self.placeholder_texture.take() {
            // VulkanImage's Drop will handle cleanup via VMA
            drop(image);
            println!("Placeholder texture dropped (VMA).");
        }

        // 5. Destroy vertex and index buffers
        // VulkanBuffer's Drop will handle cleanup via VMA
        if let Some(buffer) = self.vertex_buffer.take() {
            drop(buffer);
            println!("Vertex buffer dropped (VMA).");
        }
        if let Some(buffer) = self.index_buffer.take() {
            drop(buffer);
            println!("Index buffer dropped (VMA).");
        }

        // 6. Framebuffers (Vec will drop its contents, calling VulkanFramebuffer::drop)
        // Make sure framebuffers are dropped before render_pass and swapchain (image_views)
        self.framebuffers.clear();
        println!("Framebuffers cleared and dropped.");

        // 7. RenderPass
        if let Some(rp) = self.render_pass.take() {
            drop(rp);
            println!("RenderPass dropped.");
        }

        // 8. Swapchain (VulkanSwapchain::drop will handle its image views, swapchain, surface)
        if let Some(sc) = self.swapchain.take() {
            drop(sc);
            println!("Swapchain dropped.");
        }

        // 9. VulkanContext is an Arc, it will drop when its ref count is zero.
        // Its own Drop impl handles its resources (command pool, sync primitives, device, instance etc.)
        // We might want to ensure that `vulkan_context` is the last thing to be dropped,
        // or at least that its device is alive for all the cleanup above.
        // Arc ensures this as long as we hold `vulkan_context` and pass clones of `Arc<Device>` to sub-structs.
        println!("NovaVulkanRenderer dropped. VulkanContext will be dropped if this was the last Arc ref.");
    }
}

// Implement HasRawWindowHandle and HasRawDisplayHandle for NovaVulkanRenderer using placeholders
// This is VERY UNSAFE and only for temporary testing until proper Smithay integration.
// A real solution would involve getting these handles from Smithay's WlSurface and DisplayHandle.
unsafe impl HasRawWindowHandle for NovaVulkanRenderer {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = WaylandWindowHandle::empty();
        handle.surface = self.wayland_surface_ptr;
        // handle.display = self.wayland_display_ptr; // Not part of WaylandWindowHandle
        RawWindowHandle::Wayland(handle)
    }
}

unsafe impl HasRawDisplayHandle for NovaVulkanRenderer {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        let mut handle = WaylandDisplayHandle::empty();
        handle.display = self.wayland_display_ptr;
        RawDisplayHandle::Wayland(handle)
    }
}
