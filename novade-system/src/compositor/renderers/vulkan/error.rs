//! Defines the custom error type and `Result` alias for the Vulkan rendering subsystem.
//!
//! This module centralizes error handling for Vulkan-specific operations,
//! memory allocation errors from `vk-mem`, I/O errors (e.g., when loading shaders
//! or pipeline caches), and other custom error conditions that can occur within
//! the renderer.

use ash::vk;
use std::fmt;

/// Custom error type for the Vulkan rendering subsystem.
///
/// Encapsulates various error sources, including direct Vulkan API errors (`vk::Result`),
/// errors from the `vk-mem` memory allocator, standard I/O errors, and specific
/// application-level error conditions encountered during renderer setup or operation.
#[derive(Debug)]
pub enum VulkanError {
    /// An error originating directly from a Vulkan API call.
    /// Contains the `vk::Result` error code.
    VkResult(vk::Result),
    /// An error from the `vk-mem` memory allocator.
    VkMemError(vk_mem::Error),
    /// A standard I/O error, typically from file operations like loading shaders or pipeline caches.
    IoError(std::io::Error),
    /// An error that occurred during the general initialization phase of the renderer or a component.
    InitializationError(String),
    /// An error that occurred during the creation of a specific Vulkan resource.
    ResourceCreationError {
        /// The type of resource that failed to be created (e.g., "Buffer", "Image", "Swapchain").
        resource_type: String,
        /// A message detailing the cause of the failure.
        message: String,
    },
    /// A required Vulkan instance or device extension was not found or is not supported.
    MissingExtension(String),
    /// A required Vulkan validation layer was not found or is not supported.
    MissingLayer(String),
    /// No suitable Vulkan physical device (GPU) could be found that meets the application's requirements.
    NoSuitablePhysicalDevice,
    /// A required queue family (e.g., graphics, present, compute) could not be found on the selected physical device.
    QueueFamilyNotFound(String),
    /// The Vulkan surface (e.g., window surface) has been lost and needs to be recreated.
    /// This typically occurs due to window system events.
    SurfaceLost,
    /// The swapchain is no longer optimal or compatible with the surface (e.g., after a window resize)
    /// and needs to be recreated.
    SwapchainOutOfDate,
    /// A requested format (e.g., for an image, depth buffer, or surface) is not supported by the physical device.
    UnsupportedFormat(String),
    /// An error occurred while loading or parsing a shader module (e.g., SPIR-V file not found or corrupt).
    ShaderLoadingError(String),
    /// An error occurred during the creation of a graphics or compute pipeline.
    PipelineCreationError(String),
}

impl fmt::Display for VulkanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VulkanError::VkResult(res) => write!(f, "Vulkan API Error: {}", res), // vk::Result has Display
            VulkanError::VkMemError(err) => write!(f, "Vulkan Memory Allocator Error: {}", err), // vk_mem::Error has Display
            VulkanError::IoError(err) => write!(f, "I/O Error: {}", err),
            VulkanError::InitializationError(msg) => write!(f, "Initialization Error: {}", msg),
            VulkanError::ResourceCreationError { resource_type, message } => {
                write!(f, "Failed to create resource '{}': {}", resource_type, message)
            }
            VulkanError::MissingExtension(ext) => {
                write!(f, "Missing required Vulkan instance/device extension: {}", ext)
            }
            VulkanError::MissingLayer(layer) => write!(f, "Missing required Vulkan layer: {}", layer),
            VulkanError::NoSuitablePhysicalDevice => write!(f, "No suitable physical device found"),
            VulkanError::QueueFamilyNotFound(q_type) => {
                write!(f, "Required queue family not found: {}", q_type)
            }
            VulkanError::SurfaceLost => write!(f, "Vulkan surface lost, needs recreation."),
            VulkanError::SwapchainOutOfDate => {
                write!(f, "Vulkan swapchain is out of date or suboptimal, needs recreation.")
            }
            VulkanError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            VulkanError::ShaderLoadingError(msg) => write!(f, "Shader loading error: {}", msg),
            VulkanError::PipelineCreationError(msg) => write!(f, "Pipeline creation error: {}", msg),
        }
    }
}

impl std::error::Error for VulkanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VulkanError::VkMemError(err) => Some(err),
            VulkanError::IoError(err) => Some(err),
            // vk::Result does not implement std::error::Error directly,
            // but its Display impl provides a textual representation.
            VulkanError::VkResult(_) => None, 
            _ => None,
        }
    }
}

impl From<vk::Result> for VulkanError {
    /// Converts a raw `vk::Result` into a `VulkanError`.
    ///
    /// Special cases `vk::Result::ERROR_OUT_OF_DATE_KHR` and `vk::Result::ERROR_SURFACE_LOST_KHR`
    /// to their more specific `VulkanError` variants. Other `vk::Result` values are wrapped
    /// in `VulkanError::VkResult`.
    fn from(err: vk::Result) -> Self {
        match err {
            vk::Result::ERROR_OUT_OF_DATE_KHR => VulkanError::SwapchainOutOfDate,
            vk::Result::ERROR_SURFACE_LOST_KHR => VulkanError::SurfaceLost,
            _ => VulkanError::VkResult(err),
        }
    }
}

impl From<vk_mem::Error> for VulkanError {
    /// Converts a `vk_mem::Error` into a `VulkanError::VkMemError`.
    fn from(err: vk_mem::Error) -> Self {
        VulkanError::VkMemError(err)
    }
}

impl From<std::io::Error> for VulkanError {
    /// Converts a `std::io::Error` into a `VulkanError::IoError`.
    fn from(err: std::io::Error) -> Self {
        VulkanError::IoError(err)
    }
}

/// A `Result` type alias used throughout the Vulkan renderer module,
/// defaulting the error type to `VulkanError`.
pub type Result<T, E = VulkanError> = std::result::Result<T, E>;
