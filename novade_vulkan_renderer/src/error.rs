use std::fmt;

// Import necessary vulkano error types
use vulkano::instance::InstanceCreationError as VulkanoInstanceCreationError;
use vulkano::device::DeviceCreationError as VulkanoDeviceCreationError;
use vulkano::device::physical::PhysicalDeviceError as VulkanoPhysicalDeviceError;
// Add other vulkano error types as they become relevant (e.g. for swapchain, memory)
// use vulkano::swapchain::SwapchainCreationError as VulkanoSwapchainCreationError;
// use vulkano::memory::DeviceMemoryAllocError as VulkanoDeviceMemoryAllocError;

#[derive(Debug)]
pub enum VulkanError {
    // Vulkano specific errors
    VulkanoInstance(VulkanoInstanceCreationError),
    VulkanoDevice(VulkanoDeviceCreationError),
    VulkanoPhysicalDevice(VulkanoPhysicalDeviceError),
    // VulkanoSwapchain(VulkanoSwapchainCreationError), // Example for later phase
    // VulkanoMemory(VulkanoDeviceMemoryAllocError),    // Example for later phase

    // Custom errors for logic within NovaDE renderer
    NoSuitablePhysicalDevice,
    QueueFamilyIdentificationError(String),
    MissingExtension(String),
    WaylandSurfaceError(String),      // Placeholder for Phase 2
    UnsupportedFormat(String),        // Placeholder for Phase 2 (Swapchain format)
    GenericVulkanError(String),       // For general Vulkan related issues not covered by specific types
}

impl fmt::Display for VulkanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VulkanError::VulkanoInstance(err) => write!(f, "Vulkano Instance Error: {}", err),
            VulkanError::VulkanoDevice(err) => write!(f, "Vulkano Device Error: {}", err),
            VulkanError::VulkanoPhysicalDevice(err) => write!(f, "Vulkano Physical Device Error: {}", err),
            // VulkanError::VulkanoSwapchain(err) => write!(f, "Vulkano Swapchain Error: {}", err),
            // VulkanError::VulkanoMemory(err) => write!(f, "Vulkano Memory Allocation Error: {}", err),
            VulkanError::NoSuitablePhysicalDevice => write!(f, "No suitable Vulkan Physical Device found"),
            VulkanError::QueueFamilyIdentificationError(msg) => write!(f, "Vulkan Queue Family Identification Error: {}", msg),
            VulkanError::MissingExtension(ext) => write!(f, "Missing Vulkan Extension: {}", ext),
            VulkanError::WaylandSurfaceError(msg) => write!(f, "Wayland Surface Error: {}", msg),
            VulkanError::UnsupportedFormat(msg) => write!(f, "Unsupported Format: {}", msg),
            VulkanError::GenericVulkanError(msg) => write!(f, "Generic Vulkan Error: {}", msg),
        }
    }
}

impl std::error::Error for VulkanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VulkanError::VulkanoInstance(err) => Some(err),
            VulkanError::VulkanoDevice(err) => Some(err),
            VulkanError::VulkanoPhysicalDevice(err) => Some(err),
            // VulkanError::VulkanoSwapchain(err) => Some(err),
            // VulkanError::VulkanoMemory(err) => Some(err),
            _ => None,
        }
    }
}

// From implementations for vulkano errors
impl From<VulkanoInstanceCreationError> for VulkanError {
    fn from(err: VulkanoInstanceCreationError) -> Self {
        VulkanError::VulkanoInstance(err)
    }
}

impl From<VulkanoDeviceCreationError> for VulkanError {
    fn from(err: VulkanoDeviceCreationError) -> Self {
        VulkanError::VulkanoDevice(err)
    }
}

impl From<VulkanoPhysicalDeviceError> for VulkanError {
    fn from(err: VulkanoPhysicalDeviceError) -> Self {
        VulkanError::VulkanoPhysicalDevice(err)
    }
}

/*
// Example for later phase:
impl From<VulkanoSwapchainCreationError> for VulkanError {
    fn from(err: VulkanoSwapchainCreationError) -> Self {
        VulkanError::VulkanoSwapchain(err)
    }
}

impl From<VulkanoDeviceMemoryAllocError> for VulkanError {
    fn from(err: VulkanoDeviceMemoryAllocError) -> Self {
        VulkanError::VulkanoMemory(err)
    }
}
*/

// Define a type alias for Result for convenience throughout the crate
pub type Result<T> = std::result::Result<T, VulkanError>;
