use std::error::Error;
use std::fmt;
use std::io;

use image;
use vk_mem_rs;
use vulkanalia;

/// A specialized `Result` type for Vulkan operations.
pub type Result<T> = std::result::Result<T, VulkanError>;

/// The error type for Vulkan operations.
#[derive(Debug)]
pub enum VulkanError {
    VkResult(vulkanalia::vk::Result),
    VmaResult(vk_mem_rs::Error),
    IoError(io::Error),
    ImageError(image::ImageError),
    Message(String),
    PhysicalDeviceSelectionFailed,
    QueueFamilyNotFound,
    SwapchainCreationError(String),
    PipelineLayoutCreationError(String),
    PipelineCreationError(String),
    ShaderModuleCreationError(String),
    RenderPassCreationError(String),
    FramebufferCreationError(String),
    CommandPoolCreationError(String),
    CommandBufferAllocationError(String),
    DescriptorSetLayoutCreationError(String),
    DescriptorPoolCreationError(String),
    DescriptorSetAllocationError(String),
    ImageViewCreationError(String),
    SamplerCreationError(String),
    FenceCreationError(String),
    SemaphoreCreationError(String),
    SurfaceCreationError(String),
    DeviceLost,
    SubmitError(String),
    AcquireNextImageError(vulkanalia::vk::Result),
    PresentError(vulkanalia::vk::Result),
}

impl fmt::Display for VulkanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VulkanError::VkResult(ref err) => write!(f, "Vulkan error: {}", err),
            VulkanError::VmaResult(ref err) => write!(f, "VMA error: {}", err),
            VulkanError::IoError(ref err) => write!(f, "IO error: {}", err),
            VulkanError::ImageError(ref err) => write!(f, "Image error: {}", err),
            VulkanError::Message(ref msg) => write!(f, "{}", msg),
            VulkanError::PhysicalDeviceSelectionFailed => write!(f, "Physical device selection failed"),
            VulkanError::QueueFamilyNotFound => write!(f, "Queue family not found"),
            VulkanError::SwapchainCreationError(ref msg) => write!(f, "Swapchain creation error: {}", msg),
            VulkanError::PipelineLayoutCreationError(ref msg) => write!(f, "Pipeline layout creation error: {}", msg),
            VulkanError::PipelineCreationError(ref msg) => write!(f, "Pipeline creation error: {}", msg),
            VulkanError::ShaderModuleCreationError(ref msg) => write!(f, "Shader module creation error: {}", msg),
            VulkanError::RenderPassCreationError(ref msg) => write!(f, "Render pass creation error: {}", msg),
            VulkanError::FramebufferCreationError(ref msg) => write!(f, "Framebuffer creation error: {}", msg),
            VulkanError::CommandPoolCreationError(ref msg) => write!(f, "Command pool creation error: {}", msg),
            VulkanError::CommandBufferAllocationError(ref msg) => write!(f, "Command buffer allocation error: {}", msg),
            VulkanError::DescriptorSetLayoutCreationError(ref msg) => write!(f, "Descriptor set layout creation error: {}", msg),
            VulkanError::DescriptorPoolCreationError(ref msg) => write!(f, "Descriptor pool creation error: {}", msg),
            VulkanError::DescriptorSetAllocationError(ref msg) => write!(f, "Descriptor set allocation error: {}", msg),
            VulkanError::ImageViewCreationError(ref msg) => write!(f, "Image view creation error: {}", msg),
            VulkanError::SamplerCreationError(ref msg) => write!(f, "Sampler creation error: {}", msg),
            VulkanError::FenceCreationError(ref msg) => write!(f, "Fence creation error: {}", msg),
            VulkanError::SemaphoreCreationError(ref msg) => write!(f, "Semaphore creation error: {}", msg),
            VulkanError::SurfaceCreationError(ref msg) => write!(f, "Surface creation error: {}", msg),
            VulkanError::DeviceLost => write!(f, "Device lost"),
            VulkanError::SubmitError(ref msg) => write!(f, "Submit error: {}", msg),
            VulkanError::AcquireNextImageError(ref err) => write!(f, "Acquire next image error: {}", err),
            VulkanError::PresentError(ref err) => write!(f, "Present error: {}", err),
        }
    }
}

impl Error for VulkanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            VulkanError::VkResult(ref err) => Some(err),
            VulkanError::VmaResult(ref err) => Some(err),
            VulkanError::IoError(ref err) => Some(err),
            VulkanError::ImageError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<vulkanalia::vk::Result> for VulkanError {
    fn from(err: vulkanalia::vk::Result) -> VulkanError {
        VulkanError::VkResult(err)
    }
}

impl From<vk_mem_rs::Error> for VulkanError {
    fn from(err: vk_mem_rs::Error) -> VulkanError {
        VulkanError::VmaResult(err)
    }
}

impl From<io::Error> for VulkanError {
    fn from(err: io::Error) -> VulkanError {
        VulkanError::IoError(err)
    }
}

impl From<image::ImageError> for VulkanError {
    fn from(err: image::ImageError) -> VulkanError {
        VulkanError::ImageError(err)
    }
}

impl From<String> for VulkanError {
    fn from(msg: String) -> VulkanError {
        VulkanError::Message(msg)
    }
}

impl From<&str> for VulkanError {
    fn from(msg: &str) -> VulkanError {
        VulkanError::Message(msg.to_owned())
    }
}
