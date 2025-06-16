use ash::{vk, Device as AshDevice};
use gpu_allocator::vulkan as vma;
use gpu_allocator::MemoryUsage as GpuMemoryUsage;
use std::sync::{Arc, Mutex};

// ANCHOR: VulkanImage Struct Definition
pub struct VulkanImage {
    device: Arc<AshDevice>,
    allocator: Arc<Mutex<vma::Allocator>>,
    pub image: vk::Image, // Public for direct use in render pass attachments, descriptor updates, etc.
    allocation: Option<vma::Allocation>, // Option to allow taking it in drop
    pub extent: vk::Extent3D, // Public for info
    pub format: vk::Format,   // Public for info (e.g., creating image views)
    // current_layout: vk::ImageLayout, // Could be added if image tracks its own layout
}

// ANCHOR: VulkanImage Implementation
impl VulkanImage {
    pub fn new(
        device: Arc<AshDevice>,
        allocator: Arc<Mutex<vma::Allocator>>,
        image_create_info: &vk::ImageCreateInfo, // Pass by reference as it can be large
        memory_usage: GpuMemoryUsage,        // e.g., GpuOnly, CpuToGpu
    ) -> Result<Self, String> {
        if image_create_info.extent.width == 0 || image_create_info.extent.height == 0 || image_create_info.extent.depth == 0 {
            return Err("Image extent dimensions cannot be zero.".to_string());
        }
        if image_create_info.mip_levels == 0 {
            return Err("Image mip_levels cannot be zero.".to_string());
        }
        if image_create_info.array_layers == 0 {
            return Err("Image array_layers cannot be zero.".to_string());
        }

        let allocation_create_info = vma::AllocationCreateInfo {
            usage: memory_usage,
            // flags, required_flags, preferred_flags, pool can be specified if needed
            ..Default::default()
        };

        let (image, allocation) = unsafe {
            allocator
                .lock()
                .map_err(|_| "Failed to lock allocator mutex for image creation".to_string())?
                .create_image(image_create_info, &allocation_create_info)
                .map_err(|e| format!("VMA failed to create image: {:?}", e))?
        };

        Ok(Self {
            device,
            allocator,
            image,
            allocation: Some(allocation),
            extent: image_create_info.extent,
            format: image_create_info.format,
            // current_layout: image_create_info.initial_layout, // Store initial layout
        })
    }

    // ANCHOR_EXT: create_image_view (Associated function or method)
    // Making this an associated function for now, but could be a method on VulkanImage.
    pub fn create_image_view(
        device: &AshDevice, // Can take &Arc<AshDevice> too
        image: vk::Image,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
        mip_levels: u32, // Pass mip_levels for the view
    ) -> Result<vk::ImageView, String> {
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D) // Assuming 2D, make flexible if needed
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: aspect_flags,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1, // Assuming 1 layer, make flexible if needed (e.g. for cube maps)
            });

        unsafe {
            device.create_image_view(&image_view_create_info, None)
        }.map_err(|e| format!("Failed to create image view: {}", e))
    }

    // ANCHOR: Accessors
    pub fn handle(&self) -> vk::Image {
        self.image
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn extent(&self) -> vk::Extent3D {
        self.extent
    }

    // pub fn current_layout(&self) -> vk::ImageLayout {
    //     self.current_layout
    // }

    // pub fn set_current_layout(&mut self, new_layout: vk::ImageLayout) {
    //     self.current_layout = new_layout;
    // }

    pub fn allocation_info(&self) -> Option<&vma::AllocationInfo> {
        self.allocation.as_ref().map(|a| a.info())
    }
}

// ANCHOR: VulkanImage Drop Implementation
impl Drop for VulkanImage {
    fn drop(&mut self) {
        if let Some(allocation) = self.allocation.take() {
            //println!("Dropping VulkanImage: {:?}, Allocation: {:?}", self.image, allocation.info());
            if let Ok(mut allocator_guard) = self.allocator.lock() {
                unsafe {
                    allocator_guard.destroy_image(self.image, allocation)
                        .unwrap_or_else(|e| eprintln!("VMA failed to destroy image: {:?}", e));
                }
            } else {
                eprintln!("VulkanImage::drop: Failed to lock allocator mutex. Image and allocation may leak.");
            }
        }
    }
}
