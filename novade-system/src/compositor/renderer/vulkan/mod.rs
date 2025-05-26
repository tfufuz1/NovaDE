//! The Vulkan rendering subsystem for the NovaDE compositor.
//!
//! This module encapsulates all components required for setting up and using Vulkan
//! for rendering. It provides a structured approach to managing Vulkan resources,
//! from instance creation and device selection to memory allocation, pipeline setup,
//! texture management, and per-frame rendering logic.
//!
//! ## Core Components:
//!
//! - **`error`**: Defines custom error types (`VulkanError`) and a `Result` alias for
//!   consistent error handling throughout the Vulkan module.
//! - **`instance`**: Manages the Vulkan instance (`VkInstance`), including validation layers
//!   and debug messaging.
//! - **`physical_device`**: Handles selection and querying of physical devices (GPUs),
//!   evaluating their capabilities and queue families.
//! - **`device`**: Manages the Vulkan logical device (`VkDevice`) and its associated command queues.
//! - **`allocator`**: Wraps the Vulkan Memory Allocator (VMA) library for efficient
//!   memory management of buffers and images.
//! - **`sync_primitives`**: Defines structures for managing per-frame synchronization
//!   objects like semaphores and fences.
//! - **`buffer_utils`**: Provides utilities for creating and managing Vulkan buffers,
//!   such as staging buffer transfers.
//! - **`dynamic_uniform_buffer`**: Manages large uniform buffers that can hold multiple
//!   UBO instances, accessed via dynamic offsets, for efficient per-object data updates.
//! - **`texture`**: Handles loading, creation, and management of 2D textures, including
//!   mipmap generation and storage images.
//! - **`texture_atlas`**: Manages texture atlases, combining multiple small images into a single
//!   larger texture for efficient rendering.
//! - **`vertex_input`**: Defines vertex data structures and their input descriptions
//!   for graphics pipelines.
//! - **`render_pass`**: Manages `VkRenderPass` objects, describing the attachments and
//!   subpasses for rendering operations.
//! - **`framebuffer`**: Utilities for creating `VkFramebuffer` objects, which link
//!   render pass attachments to specific image views.
//! - **`pipeline`**: Responsible for creating Vulkan pipeline layouts (`VkPipelineLayout`)
//!   and both graphics and compute pipelines (`VkPipeline`). Also defines common
//!   shader data structures like UBOs and PushConstants.
//! - **`surface_swapchain`**: Manages Vulkan surfaces (`VkSurfaceKHR`) for window system
//!   integration (specifically Wayland) and swapchains (`VkSwapchainKHR`) for presenting
//!   rendered images.
//! - **`frame_renderer`**: Orchestrates the per-frame rendering loop, combining all other
//!   components to execute compute and graphics passes, handle synchronization,
//!   and manage resources for frames in flight.
//!
//! ## Usage:
//!
//! Typically, a higher-level renderer component would initialize these modules in sequence:
//! 1. Create a `VulkanInstance`.
//! 2. Select a `PhysicalDeviceInfo` using the instance and a surface.
//! 3. Create a `LogicalDevice` from the physical device.
//! 4. Initialize the `Allocator`.
//! 5. Initialize `DynamicUboManager` if dynamic UBOs are used.
//! 6. Create a `SurfaceSwapchain` for the target window/surface.
//! 7. Define `RenderPass`es and `PipelineLayout`s.
//! 8. Compile shaders and create `GraphicsPipeline`s and `ComputePipeline`s.
//! 9. Set up `FrameRenderer` with these components to manage the render loop.
//!
//! The module aims for robust error handling using the custom `VulkanError` type
//! and detailed logging for diagnostics.

pub mod error;
pub use error::{VulkanError, Result};

pub mod instance;
pub mod physical_device;
pub mod device;
pub mod allocator;
pub mod surface_swapchain;
pub mod pipeline;
pub mod render_pass;
pub mod framebuffer;
pub mod frame_renderer;
pub mod texture;
pub mod vertex_input;
pub mod buffer_utils;
pub mod sync_primitives;
pub mod dynamic_uniform_buffer; // Added new module
pub mod texture_atlas; // Added previously but ensure it's here
