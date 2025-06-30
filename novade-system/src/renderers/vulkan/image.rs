use ash::{vk, Device as AshDevice};
use gpu_allocator::vulkan as vma;
use gpu_allocator::MemoryUsage as GpuMemoryUsage;
use std::sync::{Arc, Mutex};

// ANCHOR: VulkanImage Struct Definition
/// Represents a Vulkan image (`vk::Image`) and its associated memory allocation
/// managed by `gpu-allocator`.
///
/// This struct encapsulates the `vk::Image` handle, its `gpu_allocator::vulkan::Allocation`,
/// and essential properties like extent and format. It handles the creation of the image
/// and its memory, and ensures proper destruction and deallocation when dropped.
///
/// It aligns with `Rendering Vulkan.md` (Spec 9.1 for image creation principles).
/// The `gpu-allocator` crate is used for memory management, which is a Rust alternative
/// to the VMA C++ library mentioned in the specification.
pub struct VulkanImage {
    /// Arc-wrapped logical device handle, shared for image operations.
    device: Arc<AshDevice>,
    /// Arc-wrapped mutex-guarded allocator, shared for memory management.
    allocator: Arc<Mutex<vma::Allocator>>,
    /// The underlying Vulkan image handle. Public for direct use where needed (e.g., render pass attachments).
    pub image: vk::Image,
    /// The memory allocation for this image, managed by `gpu-allocator`.
    /// Stored as an `Option` to allow it to be taken during `Drop`.
    allocation: Option<vma::Allocation>,
    /// The dimensions (width, height, depth) of the image. Public for informational purposes.
    pub extent: vk::Extent3D,
    /// The Vulkan format of the image data. Public for informational purposes (e.g., when creating image views).
    pub format: vk::Format,
    // current_layout: vk::ImageLayout, // Could be added if image tracks its own layout state.
}

// ANCHOR: VulkanImage Implementation
impl VulkanImage {
    /// Creates a new `VulkanImage` with allocated device memory.
    ///
    /// This constructor takes all necessary information to create a `vk::Image`
    /// and allocate its memory using `gpu-allocator`.
    ///
    /// # Arguments
    /// * `device`: An `Arc` reference to the logical `ash::Device`.
    /// * `allocator`: An `Arc<Mutex<...>>` reference to the `gpu_allocator::vulkan::Allocator`.
    /// * `image_create_info`: A reference to `vk::ImageCreateInfo` defining the image properties.
    ///   Important fields like `extent`, `mip_levels`, `array_layers`, `format`, `tiling`, `usage`,
    ///   and `initial_layout` must be correctly set by the caller as per Spec 9.1.
    /// * `memory_usage`: `gpu_allocator::MemoryUsage` (e.g., `GpuOnly`, `CpuToGpu`) indicating
    ///   the desired memory properties for the allocation. For typical image resources like textures
    ///   or attachments, `GpuMemoryUsage::GpuOnly` is common.
    ///
    /// # Returns
    /// A `Result` containing the new `VulkanImage` or an error string on failure.
    pub fn new(
        device: Arc<AshDevice>,
        allocator: Arc<Mutex<vma::Allocator>>,
        image_create_info: &vk::ImageCreateInfo, // Pass by reference as it can be large
        memory_usage: GpuMemoryUsage,        // e.g., GpuOnly, CpuToGpu
    ) -> Result<Self, String> {
        // Basic validation based on Vulkan specification requirements for ImageCreateInfo
        if image_create_info.extent.width == 0 || image_create_info.extent.height == 0 || image_create_info.extent.depth == 0 {
            return Err("Image extent dimensions (width, height, depth) cannot be zero.".to_string());
        }
        if image_create_info.mip_levels == 0 {
            return Err("Image mip_levels cannot be zero.".to_string());
        }
        if image_create_info.array_layers == 0 {
            return Err("Image array_layers cannot be zero.".to_string());
        }

        // Prepare allocation info for gpu-allocator
        let allocation_create_info = vma::AllocationCreateInfo {
            usage: memory_usage,
            // flags, required_flags, preferred_flags, pool can be specified for more control
            ..Default::default() // Sensible defaults for other fields
        };

        // Create the image and allocate memory using gpu-allocator
        // The allocator's create_image method handles both vkCreateImage and memory allocation/binding.
        let (image, allocation) = unsafe {
            allocator
                .lock()
                .map_err(|_| "Failed to lock allocator mutex for image creation".to_string())?
                .create_image(image_create_info, &allocation_create_info)
                .map_err(|e| format!("gpu-allocator failed to create image (format: {:?}, extent: {:?}): {:?}",
                                     image_create_info.format, image_create_info.extent, e))?
        };

        Ok(Self {
            device,
            allocator,
            image,
            allocation: Some(allocation), // Store the allocation to be freed on Drop
            extent: image_create_info.extent,
            format: image_create_info.format,
            // current_layout: image_create_info.initial_layout, // Optionally store initial layout
        })
    }

    // ANCHOR_EXT: create_image_view (Associated function or method)
    // Making this an associated function for now, but could be a method on VulkanImage.
    // ANCHOR_EXT: create_image_view (Associated function or method)
    /// Creates a `vk::ImageView` for a given `vk::Image`.
    ///
    /// An image view describes how to access the image and which parts of the image to access.
    /// It's required for using images in descriptor sets or as framebuffer attachments.
    /// This function aligns with `Rendering Vulkan.md` (Spec 9.2).
    ///
    /// # Arguments
    /// * `device`: A reference to the logical `ash::Device`.
    /// * `image`: The `vk::Image` for which to create the view.
    /// * `view_type`: The `vk::ImageViewType` (e.g., `TYPE_2D`, `TYPE_CUBE`, `TYPE_2D_ARRAY`).
    /// * `format`: The `vk::Format` of the image view (must be compatible with the image's format).
    /// * `aspect_flags`: `vk::ImageAspectFlags` specifying which aspects of the image are included in the view
    ///   (e.g., `COLOR`, `DEPTH`, `STENCIL`).
    /// * `base_mip_level`: The first mipmap level accessible to the view.
    /// * `level_count`: The number of mipmap levels (starting from `base_mip_level`) accessible to the view.
    /// * `base_array_layer`: The first array layer accessible to the view.
    /// * `layer_count`: The number of array layers (starting from `base_array_layer`) accessible to the view.
    ///
    /// # Returns
    /// A `Result` containing the created `vk::ImageView` or an error string on failure.
    pub fn create_image_view(
        device: &AshDevice,
        image: vk::Image,
        view_type: vk::ImageViewType,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
        base_mip_level: u32,
        level_count: u32,
        base_array_layer: u32,
        layer_count: u32,
    ) -> Result<vk::ImageView, String> {
        if level_count == 0 {
            return Err("level_count for an image view cannot be zero.".to_string());
        }
        if layer_count == 0 {
            return Err("layer_count for an image view cannot be zero.".to_string());
        }

        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(view_type)
            .format(format)
            .components(vk::ComponentMapping { // Default: No component swizzling
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: aspect_flags,
                base_mip_level,
                level_count,
                base_array_layer,
                layer_count,
            });

        unsafe {
            device.create_image_view(&image_view_create_info, None)
        }.map_err(|e| format!("Failed to create image view (format: {:?}, aspect: {:?} view_type: {:?} levels: {}-{}, layers: {}-{}): {}",
            format, aspect_flags, view_type, base_mip_level, base_mip_level + level_count -1, base_array_layer, base_array_layer + layer_count -1, e))
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
