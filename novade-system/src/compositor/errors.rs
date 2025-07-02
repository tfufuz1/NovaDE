// This is novade-system/src/compositor/errors.rs
// Definition of specific error types for the compositor using `thiserror`.

use thiserror::Error;
use smithay::backend::{
    drm::DrmError,
    egl::EglError,
    input::Error as InputError, // Smithay's backend input error
    renderer::gles2::Gles2Error, // Smithay's GLES2 renderer error
    // Add Vulkan related errors if Smithay or ash provides specific ones to wrap
};
use smithay::reexports::calloop;
use smithay::xwayland; // For XWayland errors

#[derive(Error, Debug)]
pub enum CompositorError {
    #[error("Wayland display error: {0}")]
    DisplayError(String),

    #[error("Event loop error: {0}")]
    EventLoopError(#[from] calloop::Error),

    #[error("Failed to initialize rendering backend: {0}")]
    BackendCreation(String),

    #[error("Renderer not initialized or failed")]
    RendererNotInitialized,

    #[error("Rendering operation failed: {0}")]
    RenderingError(String),

    #[error("OpenGL ES 2.0 Renderer error: {0}")]
    GlesError(#[from] Gles2Error),

    #[error("Vulkan Renderer error: {0}")]
    VulkanError(String), // Placeholder for specific Vulkan errors

    #[error("EGL error: {0}")]
    EglError(#[from] EglError),

    #[error("DRM error: {0}")]
    DrmError(#[from] DrmError),

    #[error("Input backend error: {0}")]
    InputBackendError(String), // General input backend error

    #[error("Smithay input error: {0}")]
    SmithayInputError(#[from] InputError),

    #[error("XWayland error: {0}")]
    XWaylandError(#[from] xwayland::XWaylandError),

    #[error("XWayland startup error: {0}")]
    XWaylandStartup(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Shader compilation/linking failed: {0}")]
    ShaderError(String),

    #[error("Buffer import failed (e.g., DMABUF, SHM): {0}")]
    BufferImportError(String),

    #[error("Wayland protocol error ({protocol}): {message}")]
    Protocol { protocol: String, message: String },

    #[error("Requested Wayland global not available: {0}")]
    GlobalMissing(String),

    #[error("Client is not alive or disconnected")]
    ClientDisconnected,

    #[error("Resource is dead or wrong type")]
    ResourceInvalid,

    #[error("Internal compositor error: {0}")]
    Internal(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to initialize XKB context or keymap: {0}")]
    XkbError(String), // For xkbcommon related errors
}

// Implement From for xkbcommon::xkb::Error if needed, though it's often simple enough to map to String
impl From<xkbcommon::xkb::Error> for CompositorError {
    fn from(err: xkbcommon::xkb::Error) -> Self {
        CompositorError::XkbError(err.to_string())
    }
}

// This alias can be used throughout the compositor module for function return types.
pub type Result<T, E = CompositorError> = std::result::Result<T, E>;
