// novade-system/src/renderers/vulkan/mod.rs
// Main module for the Vulkan rendering backend.

pub mod instance;
pub mod device;
pub mod surface;
pub mod swapchain;
pub mod memory;
pub mod dmabuf;
pub mod texture_pipeline; // New
pub mod command_buffer;   // New

use crate::renderer_interface::RendererInterface;
use novade_core::types::geometry::Size2D;
use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use gpu_allocator::vulkan::Allocator;
use texture_pipeline::{TextureId, GpuTexture};
use std::collections::HashMap;
use std::os::unix::io::RawFd; // For DMA-BUF FD
use super::dmabuf::{self, DmaBufImportOptions}; // dmabuf module and options struct


pub struct VulkanRenderer {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_utils: Option<ash::extensions::ext::DebugUtils>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

    surface_loader: ash::extensions::khr::Surface, // New
    surface: vk::SurfaceKHR,                       // New

    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain_loader: ash::extensions::khr::Swapchain, // New
    swapchain: vk::SwapchainKHR,                       // New
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,

    allocator: Allocator,

    command_pool: vk::CommandPool, // New: For transfer operations primarily
    textures: HashMap<TextureId, GpuTexture>, // New
}

impl VulkanRenderer {
    // Updated signature to accept window handles
    pub fn new(
        raw_display_handle_provider: &impl HasRawDisplayHandle,
        raw_window_handle_provider: &impl HasRawWindowHandle,
        window_width: u32,
        window_height: u32
    ) -> Result<Self, String> {
        let entry = instance::create_entry()?;
        let (instance, debug_utils, debug_messenger) = instance::create_instance(&entry)?;

        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
        let surface = surface::create_surface(&entry, &instance, raw_display_handle_provider, raw_window_handle_provider)?;

        let physical_device = device::pick_physical_device(&instance, &surface_loader, surface)?;
        let q_indices = device::find_queue_families(&instance, physical_device, &surface_loader, surface);
        if !q_indices.is_complete() {
            return Err("Failed to find all required queue families (graphics and present).".to_string());
        }

        let (logical_device, graphics_queue, present_queue) =
            device::create_logical_device(&instance, physical_device, &q_indices)?;

        let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &logical_device);
        let (swapchain, swapchain_format, swapchain_extent, swapchain_images) =
            swapchain::create_swapchain(
                &instance, &logical_device, physical_device, &surface_loader, surface, &q_indices,
                window_width, window_height, None
            )?;
        let swapchain_image_views = swapchain::create_image_views(&logical_device, &swapchain_images, swapchain_format)?;
        let allocator = memory::create_allocator(&instance, physical_device, &logical_device)?;

        // Create Command Pool
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(q_indices.graphics_family.unwrap());
        let command_pool = unsafe {
            logical_device.create_command_pool(&pool_create_info, None)
                .map_err(|e| format!("Failed to create command pool: {}",e))?
        };

        println!("Vulkan Renderer initialized with Texture Pipeline capabilities.");
        Ok(VulkanRenderer {
            entry, instance, debug_utils, debug_messenger,
            surface_loader, surface, physical_device, device: logical_device,
            graphics_queue, present_queue, swapchain_loader, swapchain,
            swapchain_images, swapchain_image_views, swapchain_format, swapchain_extent,
            allocator,
            command_pool,
            textures: HashMap::new(),
        })
    }

    pub fn upload_texture_from_data(
        &mut self,
        pixel_data: &[u8],
        width: u32,
        height: u32,
        format: vk::Format,
        generate_mipmaps: bool, // New parameter
    ) -> Result<TextureId, String> {
        let gpu_texture = texture_pipeline::upload_shm_buffer_to_texture(
            &self.device,
            &mut self.allocator,
            self.graphics_queue,
            self.command_pool,
            pixel_data,
            width,
            height,
            format,
            generate_mipmaps, // Pass it through
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
            // Pass self.device and self.allocator by reference
            texture.destroy(&self.device, &mut self.allocator);
            Ok(())
        } else {
            Err(format!("TextureId {:?} not found for destruction.", id))
        }
    }

    pub fn import_texture_from_dmabuf(
        &mut self,
        fd: RawFd, // From wayland client (zwp_linux_buffer_params_v1)
        width: u32,
        height: u32,
        format: vk::Format, // Vulkan format
        drm_modifier: Option<u64>,
        allocation_size: vk::DeviceSize, // Actual size of the DMA-BUF
    ) -> Result<TextureId, String> {
        // 1. Determine memory_type_index (using placeholder for now)
        let memory_type_index = dmabuf::get_memory_type_index_for_dmabuf_placeholder(
            &self.instance,
            self.physical_device,
        )?;

        let import_options = DmaBufImportOptions {
            fd,
            width,
            height,
            format,
            drm_format_modifier,
            allocation_size,
            memory_type_index,
        };

        let (image, device_memory, image_view) =
            dmabuf::import_dmabuf_as_image(&mut self.allocator, &self.device, &import_options)?;

        let texture_id = TextureId::new();
        let gpu_texture = GpuTexture {
            id: texture_id,
            image,
            image_view,
            allocation: None, // Not allocated by gpu-allocator
            memory: Some(device_memory), // Store the imported vk::DeviceMemory
            format,
            extent: vk::Extent2D { width, height },
            layout: vk::ImageLayout::UNDEFINED,
        };

        self.textures.insert(texture_id, gpu_texture);

        // Perform initial layout transition UNDEFINED -> SHADER_READ_ONLY_OPTIMAL
        let temp_cmd_buffer = command_buffer::begin_single_time_commands(&self.device, self.command_pool)?;
        texture_pipeline::transition_image_layout(
            &self.device,
            temp_cmd_buffer,
            image, // this is the vk::Image handle from import_dmabuf_as_image
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
        command_buffer::end_single_time_commands(&self.device, self.command_pool, temp_cmd_buffer, self.graphics_queue)?;

        // Update the texture's layout in our map
        if let Some(tex) = self.textures.get_mut(&texture_id) {
            tex.layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        }

        Ok(texture_id)
    }

    pub fn update_texture_region_from_data(
        &mut self,
        texture_id: TextureId,
        region: vk::Rect2D,
        pixel_data: &[u8],
    ) -> Result<(), String> {
        let gpu_texture = self.textures.get_mut(&texture_id)
            .ok_or_else(|| format!("TextureId {:?} not found for update.", texture_id))?;

        // Basic validation (more can be added)
        if gpu_texture.mip_levels > 1 {
            // Log a warning or handle as an error if mipmaps should be regenerated
            println!("Warning: Updating region of a mipmapped texture (TextureId: {:?}). Only base level (0) is updated. Mipmaps will be stale.", texture_id);
        }

        texture_pipeline::update_texture_region(
            &self.device,
            &mut self.allocator,
            self.graphics_queue,
            self.command_pool,
            gpu_texture,
            region,
            pixel_data,
        )
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            // Wait for device to be idle before destroying resources
            if self.device.device_wait_idle().is_err() {
                eprintln!("Error waiting for device idle in VulkanRenderer drop");
            }

            let texture_ids: Vec<TextureId> = self.textures.keys().cloned().collect();
            for id in texture_ids {
                if let Some(mut texture) = self.textures.remove(&id) {
                    texture.destroy(&self.device, &mut self.allocator);
                }
            }

            self.device.destroy_command_pool(self.command_pool, None);

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
            println!("Vulkan Renderer dropped (with texture and command pool cleanup).");
        }
    }
}

// Implement RendererInterface for VulkanRenderer (placeholders for now)
impl RendererInterface for VulkanRenderer {
    fn begin_frame(&mut self) { println!("VulkanRenderer: Beginning frame"); }
    fn submit_frame(&mut self) { println!("VulkanRenderer: Submitting frame"); }
    fn present(&mut self) { println!("VulkanRenderer: Presenting frame"); }
    fn resize(&mut self, _new_size: Size2D) { println!("VulkanRenderer: Resizing"); }
}
