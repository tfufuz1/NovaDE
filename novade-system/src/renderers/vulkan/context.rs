use ash::{vk, Entry, Instance};
use gpu_allocator::vulkan as vma; // Or direct VMA bindings
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;
use std::borrow::Cow; // Added for debug message strings

// Assuming sync module is at super::sync relative to this file (context.rs)
// This needs to be at the crate root or correctly pathed if context.rs is not directly in renderers::vulkan
// For now, let's assume it will be available at this path:
use super::sync::FrameSyncPrimitives;

// ANCHOR: MAX_FRAMES_IN_FLIGHT Constant
const MAX_FRAMES_IN_FLIGHT: usize = 2;

// ANCHOR: VulkanContext Struct Definition
pub struct VulkanContext {
    entry: Entry,
    instance: Arc<Instance>, // Changed to Arc<Instance>
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    debug_utils_loader: Option<ash::extensions::ext::DebugUtils>,
    physical_device: vk::PhysicalDevice,
    device: Arc<ash::Device>, // Arc for easier sharing with VMA and other parts
    graphics_queue_family_index: u32,
    present_queue_family_index: u32,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    allocator: Arc<std::sync::Mutex<vma::Allocator>>, // Mutex for interior mutability with Arc

    // ANCHOR_EXT: New fields for command buffers and sync
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    per_frame_sync_primitives: Vec<FrameSyncPrimitives>,
    current_frame_index: usize,
}

// ANCHOR: Debug Callback Function
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;
    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };
    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };
    println!(
        "{:?}:
{:?} [{} ({})] : {}
",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );
    vk::FALSE
}


// ANCHOR: VulkanContext Implementation
impl VulkanContext {
    pub fn new() -> Result<Self, String> {
        let entry = Entry::linked();

        // ANCHOR: Instance Creation
        let app_name = CString::new("NovaDE").unwrap();
        let engine_name = CString::new("NovaDE Engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_3);

        let mut instance_extensions = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::WaylandSurface::name().as_ptr(),
            ash::extensions::ext::DebugUtils::name().as_ptr(),
        ];

        let mut required_validation_layers = Vec::new();
        if cfg!(debug_assertions) {
            required_validation_layers.push(CString::new("VK_LAYER_KHRONOS_validation").unwrap());
        }

        let required_validation_layers_raw: Vec<*const c_char> = required_validation_layers
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&required_validation_layers_raw);

        let instance_ash = unsafe { // Renamed to instance_ash to avoid conflict before Arc
            entry.create_instance(&instance_create_info, None)
        }.map_err(|e| format!("Failed to create Vulkan instance: {}", e))?;
        let instance = Arc::new(instance_ash); // Wrap in Arc

        // ANCHOR: Debug Messenger Setup
        let mut debug_messenger = None;
        let mut debug_utils_loader = None;
        if cfg!(debug_assertions) {
            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO // Optional: reduce verbosity
                        // | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE // Optional: reduce verbosity
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback));

            let loader = ash::extensions::ext::DebugUtils::new(&entry, instance.as_ref()); // Use instance.as_ref()
            let messenger = unsafe {
                loader.create_debug_utils_messenger(&debug_info, None)
            }.map_err(|e| format!("Failed to create debug messenger: {}", e))?;
            debug_messenger = Some(messenger);
            debug_utils_loader = Some(loader);
            println!("Vulkan debug messenger created.");
        }

        // ANCHOR: Physical Device Selection
        let physical_devices = unsafe {
            instance.enumerate_physical_devices()
        }.map_err(|e| format!("Failed to enumerate physical devices: {}", e))?;

        if physical_devices.is_empty() {
            return Err("No Vulkan-compatible physical devices found.".to_string());
        }

        let mut chosen_device_info = None;

        for device in physical_devices {
            let properties = unsafe { instance.get_physical_device_properties(device) };
            let features = unsafe { instance.get_physical_device_features(device) };
            let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };

            // Prioritize AMD Vega 8 Integrated GPU
            let is_amd_vega_8 = properties.vendor_id == 0x1002 && properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU;
            // A more generic check could be added here if specific device name is known and properties.device_name can be checked.

            if Self::is_device_suitable(device, &instance, &queue_families, is_amd_vega_8) {
                if let Some((graphics_index, present_index)) = Self::find_queue_families(&instance, device, &queue_families) {
                    // If it's the preferred device, take it immediately
                    if is_amd_vega_8 {
                        chosen_device_info = Some((device, graphics_index, present_index, properties.device_name_as_c_str().unwrap().to_string_lossy().into_owned()));
                        break;
                    }
                    // Otherwise, store it and continue searching for a potentially better one (the preferred one)
                    if chosen_device_info.is_none() {
                         chosen_device_info = Some((device, graphics_index, present_index, properties.device_name_as_c_str().unwrap().to_string_lossy().into_owned()));
                    }
                }
            }
        }

        let (physical_device, graphics_queue_family_index, present_queue_family_index, device_name) =
            chosen_device_info.ok_or_else(|| "No suitable physical device found.".to_string())?;

        println!("Selected physical device: {} (Graphics QF: {}, Present QF: {})", device_name, graphics_queue_family_index, present_queue_family_index);

        // ANCHOR: Logical Device Creation
        let mut queue_create_infos = vec![];
        let queue_priority = 1.0f32;

        // Graphics queue
        let graphics_queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .queue_priorities(std::slice::from_ref(&queue_priority));
        queue_create_infos.push(graphics_queue_info);

        // Present queue, only if different from graphics
        if present_queue_family_index != graphics_queue_family_index {
            let present_queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(present_queue_family_index)
                .queue_priorities(std::slice::from_ref(&queue_priority));
            queue_create_infos.push(present_queue_info);
        }

        let device_extensions_raw = [
            ash::extensions::khr::Swapchain::name().as_ptr(),
            // Add other required device extensions here if any
        ];

        // Enable minimal features for now. Specific features can be enabled as needed.
        let features = vk::PhysicalDeviceFeatures::builder();
            // .sampler_anisotropy(true) // Example: if you need anisotropy
            // .build();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions_raw)
            .enabled_features(&features);

        let device: Arc<ash::Device> = unsafe {
            Arc::new(instance.create_device(physical_device, &device_create_info, None)
                .map_err(|e| format!("Failed to create logical device: {}", e))?)
        };
        println!("Logical device created.");

        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_queue_family_index, 0) };
        println!("Graphics and Present queues obtained.");

        // ANCHOR: VMA Allocator Initialization
        let allocator_create_info = vma::AllocatorCreateInfo::new(
            instance.as_ref(), // Use instance.as_ref()
            &device,
            physical_device,
        )
        // .flags(vma::AllocatorCreateFlags::EXT_MEMORY_BUDGET) // Example flag if needed
        .vulkan_api_version(app_info.api_version); // Use api_version from app_info

        let allocator = match vma::Allocator::new(allocator_create_info) {
            Ok(alloc) => alloc,
            Err(e) => return Err(format!("Failed to create VMA allocator: {:?}", e)),
        };

        let allocator_arc = Arc::new(std::sync::Mutex::new(allocator));
        println!("VMA allocator created.");

        // ANCHOR_EXT: Create Command Pool
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER); // Allow resetting individual command buffers

        let command_pool = unsafe {
            device.create_command_pool(&pool_create_info, None)
        }.map_err(|e| format!("Failed to create command pool: {}", e))?;
        println!("Command pool created.");

        // ANCHOR_EXT: Allocate Command Buffers
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);

        let command_buffers = unsafe {
            device.allocate_command_buffers(&command_buffer_allocate_info)
        }.map_err(|e| {
            unsafe { device.destroy_command_pool(command_pool, None); } // Cleanup pool if alloc fails
            format!("Failed to allocate command buffers: {}", e)
        })?;
        println!("Command buffers allocated (count: {}).", command_buffers.len());

        // ANCHOR_EXT: Create Frame Sync Primitives
        let mut per_frame_sync_primitives = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            match FrameSyncPrimitives::new(device.clone()) {
                Ok(sync_prim) => per_frame_sync_primitives.push(sync_prim),
                Err(e) => {
                    // Cleanup already created sync objects, command buffers, and pool
                    // This is verbose; a helper function or RAII guard would be better in production
                    for sp in per_frame_sync_primitives { drop(sp); } // Explicitly drop
                    unsafe {
                        // Command buffers are freed when pool is destroyed
                        device.destroy_command_pool(command_pool, None);
                    }
                    return Err(format!("Failed to create FrameSyncPrimitives for frame {}: {}", i, e));
                }
            }
        }
        println!("FrameSyncPrimitives created (count: {}).", per_frame_sync_primitives.len());

        Ok(Self {
            entry,
            instance, // This is now Arc<Instance>
            debug_messenger,
            debug_utils_loader,
            physical_device,
            device, // Actual device
            graphics_queue_family_index,
            present_queue_family_index,
            graphics_queue, // Actual queue
            present_queue,   // Actual queue
            allocator: allocator_arc, // Actual allocator
            command_pool,
            command_buffers,
            per_frame_sync_primitives,
            current_frame_index: 0,
        })
    }

    fn is_device_suitable(
        pdevice: vk::PhysicalDevice,
        instance: &Instance,
        queue_families: &[vk::QueueFamilyProperties],
        is_preferred_device: bool,
    ) -> bool {
        // Check for required extensions (e.g., swapchain)
        let required_extensions = [ash::extensions::khr::Swapchain::name()];
        let available_extensions = unsafe {
            instance.enumerate_device_extension_properties(pdevice)
        }.unwrap_or_else(|_| Vec::new());

        let mut all_extensions_supported = true;
        for req_ext_name in required_extensions.iter() {
            let found = available_extensions.iter().any(|ext| {
                let ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                ext_name == *req_ext_name
            });
            if !found {
                all_extensions_supported = false;
                break;
            }
        }
        if !all_extensions_supported {
            return false;
        }

        // Check if we have at least one graphics queue
        let has_graphics_queue = queue_families.iter().any(|qf| qf.queue_flags.contains(vk::QueueFlags::GRAPHICS));

        // For now, we only check for graphics queue and extensions.
        // Presentation support will be checked by find_queue_families.
        // More checks can be added here (e.g., features, specific limits)

        has_graphics_queue // && (is_preferred_device || true) // Allow non-preferred if it's suitable
    }

    fn find_queue_families(
        instance: &Instance, // Not strictly needed here but might be for future surface checks
        pdevice: vk::PhysicalDevice,
        queue_families: &[vk::QueueFamilyProperties],
    ) -> Option<(u32, u32)> {
        let mut graphics_family_index = None;
        // For now, assume graphics and present might be the same queue.
        // Later, Wayland surface support check will be needed for present_family_index.
        // For this initial setup, we'll simplify and assume a queue that supports graphics can also present.
        // This will need to be refined when actual surface interaction is implemented.
        let mut present_family_index = None;

        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family_index = Some(i as u32);
                // Placeholder for present check - for now, use graphics queue if it exists
                // In a real scenario, you'd check for surface support here:
                // let surface_support = unsafe {
                //     ash::extensions::khr::Surface::get_physical_device_surface_support(
                //         instance, pdevice, i as u32, surface_khr // surface_khr would be an argument
                //     ).unwrap_or(false)
                // };
                // if surface_support {
                //    present_family_index = Some(i as u32);
                // }
                present_family_index = Some(i as u32); // Simplified for now
            }

            if graphics_family_index.is_some() && present_family_index.is_some() {
                break;
            }
        }

        if graphics_family_index.is_some() && present_family_index.is_some() {
             graphics_family_index.zip(present_family_index)
        } else {
            None
        }
    }

    // ANCHOR: Accessor Methods
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    pub fn instance(&self) -> &Instance { // Returns &ash::Instance
        self.instance.as_ref()
    }

    pub fn instance_arc(&self) -> &Arc<Instance> { // Returns &Arc<ash::Instance>
        &self.instance
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn device(&self) -> &Arc<ash::Device> { // Already returns &Arc<ash::Device>
        &self.device
    }

    pub fn graphics_queue_family_index(&self) -> u32 {
        self.graphics_queue_family_index
    }

    pub fn present_queue_family_index(&self) -> u32 {
        self.present_queue_family_index
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    pub fn allocator(&self) -> Arc<std::sync::Mutex<vma::Allocator>> {
        self.allocator.clone()
    }

    // Method to get current_frame_index, useful for NovaVulkanRenderer to select sync primitives
    pub fn current_frame_index(&self) -> usize {
        self.current_frame_index
    }


    // ANCHOR_EXT: Record and Submit Draw Commands
    pub fn record_and_submit_draw_commands(
        &mut self,
        image_index: u32, // Acquired swapchain image index
        // Assuming these are passed in; VulkanContext doesn't own them directly.
        // In a fuller renderer, these might be members or retrieved differently.
        swapchain_extent: vk::Extent2D,
        render_pass_handle: vk::RenderPass,
        framebuffer_handle: vk::Framebuffer, // Specific framebuffer for the image_index
        graphics_pipeline_handle: vk::Pipeline,
        pipeline_layout_handle: vk::PipelineLayout,
        // New parameters for drawing a textured quad
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        index_count: u32,
        descriptor_set: Option<vk::DescriptorSet>, // Optional, for texture
    ) -> Result<(), String> {
        let frame_sync_primitives = &self.per_frame_sync_primitives[self.current_frame_index];
        let command_buffer = self.command_buffers[self.current_frame_index];

        unsafe {
            // ANCHOR_EXT_SUB: Wait for Fence
            // Wait for the previous frame using this set of sync primitives to finish
            self.device.wait_for_fences(
                &[frame_sync_primitives.in_flight_fence],
                true, // Wait for all fences
                u64::MAX, // Timeout
            ).map_err(|e| format!("Failed to wait for in_flight_fence: {}", e))?;

            // Reset fence once we know the command buffer it's guarding is no longer in use
            self.device.reset_fences(&[frame_sync_primitives.in_flight_fence])
                .map_err(|e| format!("Failed to reset in_flight_fence: {}", e))?;

            // ANCHOR_EXT_SUB: Reset and Begin Command Buffer
            self.device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| format!("Failed to reset command buffer: {}", e))?;

            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            self.device.begin_command_buffer(command_buffer, &begin_info)
                .map_err(|e| format!("Failed to begin command buffer: {}", e))?;

            // ANCHOR_EXT_SUB: Begin Render Pass
            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0], // Black clear color
                },
            }];
            let render_area = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain_extent,
            };
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass_handle)
                .framebuffer(framebuffer_handle)
                .render_area(render_area)
                .clear_values(&clear_values);

            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            // ANCHOR_EXT_SUB: Bind Pipeline
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline_handle,
            );

            // ANCHOR_EXT_SUB: Set Viewport & Scissor (Dynamic States)
            let viewport = vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(swapchain_extent.width as f32)
                .height(swapchain_extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .build();
            self.device.cmd_set_viewport(command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(swapchain_extent)
                .build();
            self.device.cmd_set_scissor(command_buffer, 0, &[scissor]);

            // ANCHOR_EXT_SUB: Bind Vertex and Index Buffers
            let vertex_buffers = [vertex_buffer];
            let offsets = [0_u64]; // Offset into the vertex buffer
            self.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            self.device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT16); // Assuming u16 indices

            // ANCHOR_EXT_SUB: Bind Descriptor Sets (if provided)
            if let Some(ds) = descriptor_set {
                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout_handle,
                    0, // firstSet
                    &[ds],
                    &[], // No dynamic offsets
                );
            }

            // ANCHOR_EXT_SUB: Draw Indexed Call
            self.device.cmd_draw_indexed(command_buffer, index_count, 1, 0, 0, 0);

            // ANCHOR_EXT_SUB: End Render Pass
            self.device.cmd_end_render_pass(command_buffer);

            // ANCHOR_EXT_SUB: End Command Buffer
            self.device.end_command_buffer(command_buffer)
                .map_err(|e| format!("Failed to end command buffer: {}", e))?;

            // ANCHOR_EXT_SUB: Submit to Graphics Queue
            let wait_semaphores = [frame_sync_primitives.image_available_semaphore];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [frame_sync_primitives.render_finished_semaphore];
            let submit_command_buffers = [command_buffer];

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&submit_command_buffers)
                .signal_semaphores(&signal_semaphores)
                .build();

            self.device.queue_submit(
                self.graphics_queue,
                &[submit_info],
                frame_sync_primitives.in_flight_fence, // Signal this fence when commands complete
            ).map_err(|e| format!("Failed to submit to graphics queue: {}", e))?;
        }
        Ok(())
    }

    // Call this after successful submission and presentation, before starting the next frame.
    pub fn advance_frame_index(&mut self) {
        self.current_frame_index = (self.current_frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

// ANCHOR: VulkanContext Drop Implementation
impl Drop for VulkanContext {
    fn drop(&mut self) {
        println!("Dropping VulkanContext...");

        // The resources are generally dropped in the reverse order of their declaration
        // within the struct, due to Rust's RAII rules. Arc<T> and Mutex<T> also follow this,
        // calling the Drop trait of their contained type when their reference count reaches zero
        // or they go out of scope respectively.

        // 1. `allocator: Arc<std::sync::Mutex<vma::Allocator>>`
        //    When `VulkanContext` is dropped, this `Arc`'s count might decrement. If it's the last one,
        //    the `Mutex` is dropped, and then the `vma::Allocator` is dropped.
        //    `gpu_allocator::vulkan::Allocator` has its own `Drop` implementation that releases
        //    all VMA resources. This needs the `ash::Device` and `ash::Instance` to be alive.
        //    The declaration order in `VulkanContext` (device then allocator) means allocator is dropped first.
        //    This is correct.
        //    (No explicit call needed here; RAII handles it.)
        //    println!("VMA Allocator will be dropped by RAII.");

        // 2. `device: Arc<ash::Device>`
        //    After the allocator (if any user of it is dropped), the `device` Arc may be dropped.
        //    If this `VulkanContext` holds the last `Arc<ash::Device>`, then the `ash::Device`
        //    itself is dropped. The `ash::Device`'s `Drop` trait calls `vkDestroyDevice`.
        //    This needs the `ash::Instance` to be alive.
        //    (No explicit call needed here; RAII handles it.)
        //    println!("Logical Device will be dropped by RAII via Arc.");

        // 3. `debug_messenger: Option<vk::DebugUtilsMessengerEXT>`
        //    This must be destroyed before the instance.
        //    It also needs the `debug_utils_loader` to be alive.
        if let (Some(messenger), Some(loader)) = (self.debug_messenger.take(), self.debug_utils_loader.as_ref()) {
            // `take()` the Option field to prevent potential double-free if drop was ever called multiple times
            // (Rust ensures drop is called once, but good practice with manual cleanup patterns).
            // `as_ref()` on loader is fine as it's only used for its function pointers.
            unsafe {
                loader.destroy_debug_utils_messenger(messenger, None);
            }
            println!("Vulkan debug messenger destroyed.");
        }
        // `debug_utils_loader` is Option<ash::extensions::ext::DebugUtils>. ash::extensions::ext::DebugUtils
        // is a struct of function pointers loaded from the instance, it doesn't own resources itself.
        // It will be dropped, but no specific cleanup call is needed for it.

        // 4. `instance: Arc<Instance>`
        //    The `Arc<ash::Instance>` is dropped last among these key components.
        //    If this is the last Arc, its inner `ash::Instance`'s Drop trait calls `vkDestroyInstance`.
        //    (No explicit call needed here; RAII handles it.)
        //    println!("Vulkan Instance will be dropped by RAII via Arc.");

        // `entry: Entry` is typically loaded and doesn't own Vulkan resources that need explicit cleanup.
        // `physical_device` is a handle and doesn't need to be destroyed.
        // Queues (`graphics_queue`, `present_queue`) are owned by the logical device and are cleaned up with it.

        println!("VulkanContext dropped.");
    }
}
