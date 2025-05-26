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
//! - **`texture`**: Handles loading, creation, and management of 2D textures, including
//!   mipmap generation and storage images.
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
//! 5. Create a `SurfaceSwapchain` for the target window/surface.
//! 6. Define `RenderPass`es and `PipelineLayout`s.
//! 7. Compile shaders and create `GraphicsPipeline`s and `ComputePipeline`s.
//! 8. Set up `FrameRenderer` with these components to manage the render loop.
//!
//! The module aims for robust error handling using the custom `VulkanError` type
//! and detailed logging for diagnostics.

// Publicly export main error types and commonly used structs/functions if desired,
// or require users to path into submodules. For now, exporting key errors and Result.
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
