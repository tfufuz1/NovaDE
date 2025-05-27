use thiserror::Error;

#[derive(Error, Debug)]
pub enum Gles2RendererError {
    #[error("EGL error: {0}")]
    EglError(String),

    #[error("EGL context creation failed: {0}")]
    ContextCreationFailed(String),

    #[error("Shader compilation failed for {shader_type}: {error_log}")]
    ShaderCompilationFailed {
        shader_type: String,
        error_log: String,
    },

    #[error("Shader program linking failed: {0}")]
    ShaderProgramLinkFailed(String),

    #[error("Texture operation error: {0}")]
    TextureError(String),

    #[error("OpenGL render call failed: {0}")]
    RenderCallFailed(String),

    #[error("Failed to get GL extension function: {0}")]
    ExtensionLoadingFailed(String),

    #[error("Uniform location not found: {0}")]
    UniformNotFound(String),
}
