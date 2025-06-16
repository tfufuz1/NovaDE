// novade-system/src/compositor/render/mod.rs
pub mod dmabuf_importer;
pub mod renderer;
pub mod gl;

// Re-export key components if needed by other parts of the compositor
pub use dmabuf_importer::DmabufImporter;
pub use renderer::CompositorRenderer;
// Assuming CompositorError will be defined, possibly in novade-system/src/compositor/core/errors.rs
// or a new novade-system/src/compositor/render/error.rs
// For now, let's assume it might be moved or defined here.
// pub mod error;
// pub use error::RenderError; // Or CompositorError if it's made more general
