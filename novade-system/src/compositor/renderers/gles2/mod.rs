// This file declares the submodules within the gles2 renderer module.

pub mod errors;
pub mod shaders;
pub mod texture;
pub mod renderer;

// Re-export key components for easier access from the parent `renderers` module, if needed.
// For example, if Gles2Renderer is the primary or only GLES2 implementation offered.
pub use self::renderer::Gles2Renderer;
pub use self::texture::Gles2Texture;
pub use self::errors::Gles2RendererError;
