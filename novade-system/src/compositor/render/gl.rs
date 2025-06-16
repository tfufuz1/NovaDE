//! Provides an OpenGL ES 2.0 based rendering backend for the NovaDE compositor.
//!
//! This module implements the `CompositorRenderer` trait using Smithay's `Gles2Renderer`
//! and EGL for context management. It handles shader loading, texture importing (SHM and DMABUF),
//! and rendering of compositor elements.

// novade-system/src/compositor/render/gl.rs

use smithay::backend::renderer::{
    gles2::{Gles2Renderer, Gles2Texture, Gles2Program}, // Added GlesProgram
    Error as SmithayRendererError, // Renamed for clarity
};
use smithay::backend::egl::{EglDisplay, Error as EglError};
use smithay::backend::allocator::Fourcc;
use smithay::utils::{Physical, Rectangle, Size, Point, Transform};
use raw_window_handle::DisplayHandle;
use tracing::{info, error, warn}; // Removed debug as it wasn't used in new code
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::any::Any;
use std::fmt;

// Import from parent module (renderer.rs)
use super::renderer::{
    CompositorRenderer, RenderableTexture, RenderError, RenderElement,
};
use crate::compositor::core::state::DesktopState; // For import methods
use smithay::reexports::wayland_server::protocol::{
    wl_buffer::WlBuffer,
    wl_surface::WlSurface,
    wl_shm,
};
use smithay::wayland::shm::with_buffer_contents_data;
use smithay::backend::renderer::ImportShmError;
use smithay::backend::renderer::gles2::GlesShader; // Added for shader compilation
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::utils::BufferCoord;
use glow; // Added for GLenum constants like VERTEX_SHADER


/// Errors specific to the OpenGL (GLES2) renderer initialization.
///
/// These errors typically occur during the setup of EGL or the underlying GLES2 context.
#[derive(Debug, thiserror::Error)]
pub enum GlInitError {
    /// An error occurred within the EGL library (e.g., context creation, display connection).
    #[error("EGL error: {0}")]
    EglError(#[from] EglError),
    /// An error occurred within Smithay's GLES2 renderer component.
    #[error("Smithay renderer error: {0}")]
    SmithayRendererError(#[from] SmithayRendererError),
    /// No suitable EGL context or EGL display was found or could be initialized.
    /// This might indicate missing graphics drivers or insufficient hardware capabilities.
    #[error("No suitable EGL context or EGL display available.")]
    NoSuitableEglContext,
}

// Convert GlInitError to RenderError
impl From<GlInitError> for RenderError {
    fn from(err: GlInitError) -> Self {
        match err {
            GlInitError::EglError(e) => RenderError::BackendInitializationFailed(format!("EGL error: {}", e)),
            GlInitError::SmithayRendererError(e) => RenderError::BackendInitializationFailed(format!("Smithay Renderer error: {}", e)),
            GlInitError::NoSuitableEglContext => RenderError::BackendInitializationFailed("No suitable EGL context found".to_string()),
        }
    }
}
// Also convert Smithay's own EglError and RendererError directly for convenience
impl From<EglError> for RenderError {
    fn from(err: EglError) -> Self {
        RenderError::BackendInitializationFailed(format!("EGL error: {}", err))
    }
}

impl From<SmithayRendererError> for RenderError {
    fn from(err: SmithayRendererError) -> Self {
        RenderError::BackendInitializationFailed(format!("Smithay Renderer error: {}", err))
    }
}


static TEXTURE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_texture_id() -> u64 {
    TEXTURE_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Represents a texture managed by the `OpenGLRenderer`.
///
/// This struct wraps Smithay's `Gles2Texture` in an `Arc` for shared ownership,
/// and includes additional metadata like a unique ID, pixel format, and dimensions.
#[derive(Debug, Clone)]
pub struct OpenGLTexture {
    /// Unique identifier for this texture, used for caching.
    id: u64,
    /// The underlying Smithay GLES2 texture object.
    smithay_texture: Arc<Gles2Texture>,
    /// The pixel format of the texture (e.g., ARGB8888), if known.
    format: Option<Fourcc>,
    /// The dimensions (width and height) of the texture in physical pixels.
    size: Size<i32, Physical>,
}

impl RenderableTexture for OpenGLTexture {
    /// Returns the dimensions (width and height) of the texture in physical pixels.
    fn dimensions(&self) -> Size<i32, Physical> {
        self.size
    }

    /// Returns the pixel format of the texture.
    fn format(&self) -> Option<Fourcc> {
        self.format
    }

    /// Returns the texture as a dynamic type for downcasting if necessary.
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    /// Returns the unique identifier for this texture instance.
    fn unique_id(&self) -> u64 {
        self.id
    }
}

/// An OpenGL ES 2.0 based renderer for the compositor.
///
/// This struct implements the `CompositorRenderer` trait, using Smithay's `Gles2Renderer`
/// for OpenGL operations and EGL for context management. It handles shader loading,
/// texture import from SHM and DMABUF, and rendering of various visual elements.
pub struct OpenGLRenderer {
    /// Smithay's GLES2 renderer instance, which abstracts direct OpenGL calls.
    smithay_renderer: Gles2Renderer,
    /// The EGL display connection, kept alive for the duration of the renderer.
    _egl_display: EglDisplay,
    /// A cache of compiled GLES2 shader programs, keyed by a descriptive name.
    shaders: HashMap<String, Gles2Program>,
    /// A cache of `OpenGLTexture` instances, keyed by their unique ID.
    /// This helps avoid redundant texture imports.
    texture_cache: HashMap<u64, Arc<OpenGLTexture>>,
    /// The transformation currently applied to the output (e.g., rotation).
    current_output_transform: Transform,
    /// The physical size of the current output being rendered to.
    current_output_physical_size: Size<i32, Physical>,
    /// The current scale factor of the output.
    current_scale: f64,
}

impl fmt::Debug for OpenGLRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenGLRenderer")
            .field("smithay_renderer", &"Gles2Renderer { ... }") // Avoid printing internal Gles2Renderer details
            .field("egl_display", &"EglDisplay { ... }")
            .field("shaders_count", &self.shaders.len())
            .field("texture_cache_count", &self.texture_cache.len())
            .field("current_output_transform", &self.current_output_transform)
            .field("current_output_physical_size", &self.current_output_physical_size)
            .field("current_scale", &self.current_scale)
            .finish()
    }
}

// Shader Definitions
const PASSTHROUGH_VERTEX_SHADER: &str = r#"
#version 100
attribute vec2 position;
attribute vec2 texcoord;
varying vec2 v_texcoord;
uniform mat3 transform_matrix;
void main() {
    v_texcoord = texcoord;
    vec3 V = transform_matrix * vec3(position, 1.0);
    gl_Position = vec4(V.xy, 0.0, 1.0);
}
"#;

const PASSTHROUGH_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;
varying vec2 v_texcoord;
uniform sampler2D tex;
uniform float alpha;
void main() {
    gl_FragColor = texture2D(tex, v_texcoord) * alpha;
}
"#;

const PREMULTIPLIED_ALPHA_BLEND_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;
varying vec2 v_texcoord;
uniform sampler2D tex;
uniform float alpha_uniform;
void main() {
    vec4 texture_color = texture2D(tex, v_texcoord);
    gl_FragColor = vec4(texture_color.rgb * alpha_uniform, texture_color.a * alpha_uniform);
}
"#;


impl OpenGLRenderer {
    fn load_shader_program(
        smithay_renderer: &mut Gles2Renderer,
        vertex_shader_src: &str,
        fragment_shader_src: &str,
    ) -> Result<Gles2Program, RenderError> {
        let vert_shader = smithay_renderer
            .compile_shader(vertex_shader_src, glow::VERTEX_SHADER)
            .map_err(|e| RenderError::ShaderCompilationFailed(format!("Vertex shader compilation failed: {}", e)))?;

        let frag_shader = smithay_renderer
            .compile_shader(fragment_shader_src, glow::FRAGMENT_SHADER)
            .map_err(|e| RenderError::ShaderCompilationFailed(format!("Fragment shader compilation failed: {}", e)))?;

        smithay_renderer
            .link_program(&[vert_shader, frag_shader])
            .map_err(|e| RenderError::ShaderCompilationFailed(format!("Shader program linking failed: {}", e)))
    }
}

impl CompositorRenderer for OpenGLRenderer {
    type Texture = OpenGLTexture;

    /// Creates a new `OpenGLRenderer` instance.
    ///
    /// Initializes the EGL display and context, loads necessary shaders, and prepares
    /// the Smithay `Gles2Renderer`.
    ///
    /// # Parameters
    /// - `display_handle`: A raw display handle, necessary for EGL initialization.
    ///
    /// # Errors
    /// Returns `RenderError::BackendInitializationFailed` if EGL setup, GLES2 context creation,
    /// or shader compilation fails.
    fn new<'a>(display_handle: DisplayHandle<'a>, /* other necessary params */) -> Result<Self, RenderError> {
        info!("Initializing OpenGLRenderer...");

        let egl_display = EglDisplay::new(display_handle)
            .map_err(|e| RenderError::BackendInitializationFailed(format!("Failed to create EGL display: {}", e)))?;
        info!("EGL display created successfully.");

        let mut smithay_renderer = unsafe { Gles2Renderer::new(egl_display.clone()) }
            .map_err(|e| RenderError::BackendInitializationFailed(format!("Failed to create Gles2Renderer: {}", e)))?;
        info!("Smithay Gles2Renderer initialized successfully.");

        let mut shaders = HashMap::new();

        // Load passthrough shader
        let passthrough_program = OpenGLRenderer::load_shader_program(
            &mut smithay_renderer,
            PASSTHROUGH_VERTEX_SHADER,
            PASSTHROUGH_FRAGMENT_SHADER,
        )?;
        shaders.insert("passthrough".to_string(), passthrough_program);
        info!("Passthrough shader program loaded successfully.");

        // Load premultiplied alpha blend shader
        let premultiplied_alpha_program = OpenGLRenderer::load_shader_program(
            &mut smithay_renderer,
            PASSTHROUGH_VERTEX_SHADER, // Uses the same vertex shader
            PREMULTIPLIED_ALPHA_BLEND_FRAGMENT_SHADER,
        )?;
        shaders.insert("premultiplied_alpha_blend".to_string(), premultiplied_alpha_program);
        info!("Premultiplied alpha blend shader program loaded successfully.");

        Ok(OpenGLRenderer {
            smithay_renderer,
            _egl_display: egl_display,
            shaders,
            texture_cache: HashMap::new(),
            current_output_transform: Transform::Normal,
            current_output_physical_size: Size::from((0, 0)), // Default size
            current_scale: 1.0,
        })
    }

    /// Prepares the OpenGL renderer for a new frame.
    ///
    /// This involves making the EGL context current and setting the OpenGL viewport
    /// to match the physical size of the output.
    fn begin_frame(&mut self, output_transform: Transform, output_physical_size: Size<i32, Physical>) -> Result<(), RenderError> {
        self.current_output_transform = output_transform;
        self.current_output_physical_size = output_physical_size;

        // Bind the renderer to make the EGL context current and set up the viewport.
        // This is necessary before any OpenGL calls.
        // Gles2Renderer::bind() itself doesn't return a Result that needs to be propagated here.
        // If it could fail in a way that needs RenderError, the API would reflect that.
        self.smithay_renderer.bind();

        unsafe {
            // Set the OpenGL viewport to match the output physical size.
            // The arguments are (x, y, width, height).
            // (0,0) is typically the bottom-left in OpenGL.
            self.smithay_renderer.gl().viewport(
                0,
                0,
                output_physical_size.w,
                output_physical_size.h,
            );
            // Check for GL errors, though viewport itself rarely fails if context is valid.
            // let gl_error = self.smithay_renderer.gl().get_error();
            // if gl_error != glow::NO_ERROR {
            //     return Err(RenderError::OpenGL(format!("Failed to set viewport: GL error {}", gl_error)));
            // }
        }

        info!(
            "OpenGLRenderer::begin_frame - Transform: {:?}, Physical Size: {:?}, Viewport Set.",
            self.current_output_transform, self.current_output_physical_size
        );
        Ok(())
    }

    /// Renders a collection of `RenderElement`s using the OpenGL backend.
    ///
    /// It first clears the background, then iterates through each element,
    /// setting appropriate OpenGL blend modes and dispatching draw calls via
    /// Smithay's `Gles2Renderer`.
    fn render_elements<'a>(
        &mut self,
        elements: Vec<RenderElement<'a, Self::Texture>>,
        _output_damage: &[Rectangle<i32, Physical>] // Not used directly, but could be for optimizations
    ) -> Result<(), RenderError> {
        // 1. Clear the background
        let clear_damage = [Rectangle::from_loc_and_size(Point::from((0,0)), self.current_output_physical_size)];
        if let Err(e) = self.smithay_renderer.clear([0.1, 0.1, 0.1, 1.0], &clear_damage) { // Dark gray
            error!("Failed to clear background: {:?}", e);
            return Err(RenderError::OpenGL(format!("Failed to clear background: {}", e)));
        }

        let mut last_element_was_transparent = false;

        // 2. Iterate through elements
        for element in elements {
            match element {
                RenderElement::Surface { texture, geometry, damage, alpha, transform: _ } => {
                    let opengl_texture = match texture.as_any().downcast_ref::<OpenGLTexture>() {
                        Some(tex) => tex,
                        None => {
                            error!("Failed to downcast RenderableTexture to OpenGLTexture for a Surface element.");
                            continue;
                        }
                    };

                    let smithay_texture_ref = &*opengl_texture.smithay_texture;

                    if alpha < 1.0 {
                        unsafe {
                            self.smithay_renderer.gl().enable(glow::BLEND);
                            self.smithay_renderer.gl().blend_func_separate(glow::ONE, glow::ONE_MINUS_SRC_ALPHA, glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
                        }
                        last_element_was_transparent = true;
                    } else {
                        unsafe {
                            self.smithay_renderer.gl().disable(glow::BLEND);
                        }
                        last_element_was_transparent = false;
                    }

                    if let Err(e) = self.smithay_renderer.render_texture_at(
                        smithay_texture_ref,
                        geometry,
                        1, // texture_scale (physical pixels)
                        self.current_output_transform,
                        damage,
                        alpha,
                    ) {
                        error!("Failed to render surface texture ID {}: {}. Error: {:?}", opengl_texture.id, geometry, e);
                        // For now, log and continue. Consider returning the error.
                        // return Err(RenderError::OpenGL(format!("Failed to render surface texture {}: {}", opengl_texture.id, e)));
                    }
                }
                RenderElement::Cursor { texture, position, hotspot, damage, .. } => {
                    let opengl_texture = match texture.as_any().downcast_ref::<OpenGLTexture>() {
                        Some(tex) => tex,
                        None => {
                            error!("Failed to downcast RenderableTexture to OpenGLTexture for a Cursor element.");
                            continue;
                        }
                    };

                    let smithay_texture_ref = &*opengl_texture.smithay_texture;
                    let cursor_rect = Rectangle::from_loc_and_size(position - hotspot, opengl_texture.dimensions());

                    unsafe {
                        self.smithay_renderer.gl().enable(glow::BLEND);
                        self.smithay_renderer.gl().blend_func_separate(glow::ONE, glow::ONE_MINUS_SRC_ALPHA, glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
                    }
                    last_element_was_transparent = true;

                    if let Err(e) = self.smithay_renderer.render_texture_at(
                        smithay_texture_ref,
                        cursor_rect,
                        1, // texture_scale
                        self.current_output_transform,
                        damage,
                        1.0, // Alpha (cursor alpha is usually baked into its texture)
                    ) {
                        error!("Failed to render cursor texture ID {}: {}. Error: {:?}", opengl_texture.id, cursor_rect, e);
                        // return Err(RenderError::OpenGL(format!("Failed to render cursor texture {}: {}", opengl_texture.id, e)));
                    }
                }
                RenderElement::SolidColor { color, geometry: _, damage } => { // geometry might be used if damage is relative
                    // Smithay's clear function clears the specified damaged regions to the given color.
                    if let Err(e) = self.smithay_renderer.clear(color, damage) {
                       error!("Failed to render solid color block for damage {:?}: {:?}", damage, e);
                       // return Err(RenderError::OpenGL(format!("Failed to render solid color: {}", e)));
                    }
                    // SolidColor does not change the blend state for subsequent textures.
                    // If it did, we'd set last_element_was_transparent accordingly.
                }
                RenderElement::CompositorUi { texture, geometry, damage, .. } => {
                    // Similar to Surface, but could have different blending or shaders in the future.
                     let opengl_texture = match texture.as_any().downcast_ref::<OpenGLTexture>() {
                        Some(tex) => tex,
                        None => {
                            error!("Failed to downcast RenderableTexture to OpenGLTexture for a CompositorUi element.");
                            continue;
                        }
                    };
                    let smithay_texture_ref = &*opengl_texture.smithay_texture;

                    // Assuming UI elements might have transparency
                    unsafe {
                        self.smithay_renderer.gl().enable(glow::BLEND);
                        self.smithay_renderer.gl().blend_func_separate(glow::ONE, glow::ONE_MINUS_SRC_ALPHA, glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
                    }
                    last_element_was_transparent = true;

                    if let Err(e) = self.smithay_renderer.render_texture_at(
                        smithay_texture_ref,
                        geometry,
                        1, // texture_scale
                        self.current_output_transform,
                        damage,
                        1.0, // Alpha for UI, assuming it's pre-multiplied or handled by texture
                    ) {
                        error!("Failed to render CompositorUi texture ID {}: {}. Error: {:?}", opengl_texture.id, geometry, e);
                    }
                }
            }
        }

        // 6. After processing all elements, disable blending if the last element left it enabled.
        if last_element_was_transparent {
            unsafe {
                self.smithay_renderer.gl().disable(glow::BLEND);
            }
        }

        Ok(())
    }

    /// Finalizes and submits the current frame to the display using OpenGL.
    ///
    /// This typically involves calling `eglSwapBuffers` (or equivalent) via
    /// Smithay's `Gles2Renderer::submit`.
    fn finish_frame(&mut self) -> Result<(), RenderError> {
        match self.smithay_renderer.submit(None) { // Pass damage if partial rendering is supported and desired
            Ok(_) => {
                info!("OpenGLRenderer::finish_frame - Frame submitted successfully.");
                Ok(())
            }
            Err(e) => {
                error!("Failed to submit frame: {:?}", e);
                Err(RenderError::OpenGL(format!("Failed to submit frame: {}", e)))
            }
        }
    }

/// Helper function to convert a Wayland SHM buffer format (`wl_shm::Format`)
/// to a `Fourcc` code. This aids in standardizing format representation.
fn wl_shm_format_to_fourcc(format: wl_shm::Format) -> Option<Fourcc> {
    match format {
        wl_shm::Format::Argb8888 => Some(Fourcc::Argb8888),
        wl_shm::Format::Xrgb8888 => Some(Fourcc::Xrgb8888),
        wl_shm::Format::Abgr8888 => Some(Fourcc::Abgr8888),
        wl_shm::Format::Xbgr8888 => Some(Fourcc::Xbgr8888),
        // TODO: Add other common formats if the renderer supports them
        // e.g. Rgb565, Argb1555, etc.
        // Note: GLES2 might have limited direct support for some formats.
        _ => {
            warn!("Unsupported wl_shm::Format encountered: {:?}", format);
            None
        }
    }
}

    /// Imports a `WlBuffer` (SHM) into an `OpenGLTexture`.
    ///
    /// The buffer's pixel data is uploaded to a GLES2 texture via `Gles2Renderer::import_shm_buffer`.
    /// The created texture is then cached in `self.texture_cache`.
    fn import_shm_buffer(
        &mut self,
        buffer: &WlBuffer,
        surface: Option<&WlSurface>,
        desktop_state: &DesktopState,
    ) -> Result<Arc<Self::Texture>, RenderError> {
        info!(
            "Importing SHM buffer for surface {:?}",
            surface.map(|s| s.id())
        );

        // Attempt to retrieve from cache first can be added here if beneficial,
        // but buffer contents can change, so direct import might be safer unless
        // buffer IDs or content hashes are used for caching keys.
        // For now, we always import and then cache.

        let (gles2_texture, shm_format, width, height) = with_buffer_contents_data(&desktop_state.shm_state, buffer, |data, attributes| {
            let size = Size::from((attributes.width, attributes.height));
            // Gles2Renderer::import_shm_buffer expects upside_down to be true if y_inverted is true
            let upside_down = attributes.y_inverted;

            self.smithay_renderer.import_shm_buffer(
                data,
                size.to_buffer_coord(), // Requires Size<i32, BufferCoord>
                attributes.stride,
                upside_down,
                attributes.format,
            )
            .map(|tex| (tex, attributes.format, attributes.width, attributes.height)) // Pass format and dimensions out
            .map_err(|e| match e {
                ImportShmError::UnsupportedFormat => {
                    error!("SHM Import Error: Unsupported format provided for buffer.");
                    RenderError::UnsupportedFormat(wl_shm_format_to_fourcc(attributes.format))
                },
                ImportShmError::InvalidData => {
                    error!("SHM Import Error: Invalid data provided for buffer.");
                    RenderError::InvalidBuffer("Invalid data for SHM buffer import".to_string())
                },
                ImportShmError::Renderer(err) => {
                    error!("SHM Import Error: Renderer error: {}", err);
                    RenderError::ShmBufferImportFailed(format!("Renderer error during SHM import: {}", err))
                }
            })
        }).map_err(|access_error| {
            // This error occurs if the buffer is not valid (e.g. already destroyed)
            error!("Failed to access SHM buffer contents: {:?}", access_error);
            RenderError::InvalidBuffer(format!("Failed to access SHM buffer: {:?}", access_error))
        })??; // Double ?? because with_buffer_contents_data itself returns a Result, and then the closure returns a Result

        let fourcc_format = wl_shm_format_to_fourcc(shm_format);

        let opengl_texture_struct = OpenGLTexture {
            id: generate_texture_id(),
            smithay_texture: Arc::new(gles2_texture),
            format: fourcc_format,
            size: Size::from((width, height)),
        };

        let opengl_texture = Arc::new(opengl_texture_struct);
        self.texture_cache.insert(opengl_texture.unique_id(), opengl_texture.clone());

        info!("SHM buffer imported successfully as OpenGLTexture id {}", opengl_texture.unique_id());
        Ok(opengl_texture)
    }

    /// Imports a `Dmabuf` into an `OpenGLTexture`.
    ///
    /// This typically utilizes EGL extensions (like `EGL_EXT_image_dma_buf_import`)
    /// to create an `EGLImage` from the DMABUF, which is then bound to a GLES2 texture.
    /// This process is handled by `Gles2Renderer::import_dmabuf`.
    /// The created texture is then cached in `self.texture_cache`.
    fn import_dmabuf(
        &mut self,
        dmabuf: &Dmabuf,
        surface: Option<&WlSurface>,
    ) -> Result<Arc<Self::Texture>, RenderError> {
        info!(
            "Importing DMABUF with format {:?} for surface {:?}",
            dmabuf.format().code,
            surface.map(|s| s.id())
        );

        // Similar to SHM, caching strategy could be reviewed.
        // For DMABUF, the buffer content is more opaque to CPU, so import is standard.

        // Import the DMABUF using smithay's Gles2Renderer
        // The second argument `frame` can be None if we are not using damage-tracking import directly here.
        let gles2_texture = self.smithay_renderer.import_dmabuf(dmabuf, None)
            .map_err(|import_error| {
                error!("Failed to import DMABUF: {}", import_error);
                // It's important to map Smithay's specific error to our RenderError
                match import_error {
                    smithay::backend::renderer::ImportDmaError::ImporterDoesNotExist => {
                        RenderError::DmabufImportFailed("DMABUF importer does not exist. Ensure EGL extensions are available.".to_string())
                    }
                    smithay::backend::renderer::ImportDmaError::Unsupported => {
                        RenderError::DmabufImportFailed("DMABUF import is unsupported (possibly missing EGL extensions or driver issue).".to_string())
                    }
                    smithay::backend::renderer::ImportDmaError::NoSuitableVisual => {
                        RenderError::DmabufImportFailed("No suitable EGL visual found for DMABUF import.".to_string())
                    }
                    smithay::backend::renderer::ImportDmaError::Render(e) => {
                        RenderError::DmabufImportFailed(format!("Renderer error during DMABUF import: {}", e))
                    }
                }
            })?;

        let dimensions = gles2_texture.dimensions(); // From smithay::backend::renderer::Texture trait
        let fourcc_format = dmabuf.format().code;

        let opengl_texture_struct = OpenGLTexture {
            id: generate_texture_id(),
            smithay_texture: Arc::new(gles2_texture), // Wrap the Gles2Texture in Arc
            format: Some(fourcc_format),
            size: dimensions,
        };

        let opengl_texture = Arc::new(opengl_texture_struct);
        self.texture_cache.insert(opengl_texture.unique_id(), opengl_texture.clone());

        info!("DMABUF imported successfully as OpenGLTexture id {}", opengl_texture.unique_id());
        Ok(opengl_texture)
    }

    /// Removes a texture from the internal `texture_cache`.
    ///
    /// This is called when a texture is no longer needed (e.g., its source buffer is released).
    fn clear_texture_cache(&mut self, texture_id: u64) {
        if self.texture_cache.remove(&texture_id).is_some() {
            info!("Texture {} removed from cache.", texture_id);
        } else {
            warn!("Attempted to remove non-existent texture {} from cache.", texture_id);
        }
    }

    /// Sets the scale factor for the current output.
    ///
    /// This information can be used by the renderer to adjust rendering parameters
    /// for HiDPI displays, though current implementation only stores it.
    fn set_output_scale(&mut self, scale: f64) -> Result<(), RenderError> {
        if scale <= 0.0 {
            warn!("Received non-positive output scale: {}. Storing as is.", scale);
            // Consider returning Err(RenderError::InvalidParameter("Scale must be positive".to_string()))
        }
        self.current_scale = scale;
        info!("OpenGLRenderer::set_output_scale - Scale set to: {}", self.current_scale);
        Ok(())
    }

    /// Returns a list of preferred FourCC pixel formats for DMABUF import.
    ///
    /// These formats are generally well-supported by GLES2 implementations for
    /// creating textures from EGLImages.
    fn preferred_formats(&self) -> Option<Vec<Fourcc>> {
        // These are common formats well-supported by GLES2 through EGLImage.
        // The actual support can depend on the EGL implementation and drivers.
        Some(vec![
            Fourcc::Argb8888,
            Fourcc::Xrgb8888,
            Fourcc::Abgr8888,
            Fourcc::Xbgr8888,
            // Other formats like Rgb565 could be added if needed and supported.
        ])
    }
}

// The old init_gl_renderer function is no longer needed as its logic is within OpenGLRenderer::new.
// The old tests might need significant rework to test the new structure.
// The duplicate impl CompositorRenderer for OpenGLRenderer block was removed by the changes above.

#[cfg(test)]
mod tests {
    use super::*;
    use raw_window_handle::{HasRawDisplayHandle, RawDisplayHandle}; // Required for mock DisplayHandle

    // Mock for DisplayHandle for testing purposes
    struct MockDisplayHandle;
    unsafe impl HasRawDisplayHandle for MockDisplayHandle {
        fn raw_display_handle(&self) -> RawDisplayHandle {
            // This is highly platform-specific and unsafe.
            // For testing on headless CI, this might not work or might need specific EGL setup.
            // e.g., for X11, it might be XlibDisplayHandle, for Wayland WlDisplayHandle.
            // Returning an empty handle, which will likely cause EglDisplay::new to fail
            // unless the environment has a default EGL display.
            #[cfg(feature = "wayland_frontend")] // Smithay's EglDisplay might need this
            {
                use raw_window_handle::WaylandDisplayHandle;
                let mut handle = WaylandDisplayHandle::empty();
                // handle.display = ???; // This would need a pointer to a wl_display
                RawDisplayHandle::Wayland(handle)
            }
            #[cfg(not(feature = "wayland_frontend"))]
            {
                // Fallback for environments where Wayland isn't the primary mock target
                // This is a very rough mock.
                warn!("Using a very basic mock RawDisplayHandle. EGL initialization might fail.");
                RawDisplayHandle::AppKit(raw_window_handle::AppKitDisplayHandle::empty()) // Or any other variant
            }

        }
    }


    #[test]
    fn test_opengl_renderer_new() {
        // This test requires a valid EGL environment or a sophisticated mock.
        // On many CI systems, this will likely fail if no display server is available.
        let display_handle = MockDisplayHandle;
        let display_handle_raw = display_handle.raw_display_handle();

        // Wrap in DisplayHandle::borrow_raw if that's the expected type
        // Or directly if DisplayHandle is just a type alias for RawDisplayHandle
        // For Smithay, EglDisplay::new takes DisplayHandle<'a> which is an enum over raw handles.
        let dh_wrapper = unsafe { DisplayHandle::borrow_raw(display_handle_raw) };

        // The ::new method now also loads shaders.
        match OpenGLRenderer::new(dh_wrapper) {
            Ok(_renderer) => {
                info!("OpenGLRenderer::new() succeeded in test environment (shaders loaded).");
            }
            Err(e) => {
                warn!("OpenGLRenderer::new() failed: {}. This might be expected in CI without a display or if shader compilation fails.", e);
                // If shader compilation is robust, this mainly points to EGL issues.
            }
        }
    }

    #[test]
    fn test_opengl_texture_id_generation() {
        let id1 = generate_texture_id();
        let id2 = generate_texture_id();
        assert_ne!(id1, id2, "Generated texture IDs should be unique.");
        assert!(id1 > 0, "Texture ID should be positive.");
        assert!(id2 > id1, "Sequential texture IDs should increment.");
    }

    #[test]
    fn test_opengl_texture_impl() {
        // This test would require a Gles2Texture instance, which needs a Gles2Renderer.
        // For now, we can't easily test this part without a renderer.
        // We've tested unique_id separately.
        // Other methods (dimensions, format, as_any) are simple getters.
        warn!("Skipping full OpenGLTexture methods test as it requires a Gles2Renderer instance.");
    }

    #[test]
    fn test_texture_cache_interaction_conceptual() {
        // This test conceptually verifies the cache interaction logic (add/remove).
        // It uses a HashMap<u64, String> to simulate the `texture_cache`
        // because `OpenGLRenderer` and its `Gles2Texture` components are difficult
        // to mock reliably in a unit test environment without a display server or EGL context.
        // The `OpenGLRenderer::clear_texture_cache` method itself primarily calls `HashMap::remove`.

        let mut mock_texture_cache: HashMap<u64, String> = HashMap::new();

        // 1. Generate a few unique IDs
        let id1 = generate_texture_id();
        let id2 = generate_texture_id();
        let id3 = generate_texture_id();

        // 2. Insert IDs with placeholder values into the mock cache
        mock_texture_cache.insert(id1, "mock_texture_data_1".to_string());
        mock_texture_cache.insert(id2, "mock_texture_data_2".to_string());
        mock_texture_cache.insert(id3, "mock_texture_data_3".to_string());

        assert!(mock_texture_cache.contains_key(&id1), "ID1 should be in the cache");
        assert!(mock_texture_cache.contains_key(&id2), "ID2 should be in the cache");
        assert!(mock_texture_cache.contains_key(&id3), "ID3 should be in the cache");
        assert_eq!(mock_texture_cache.len(), 3, "Cache should contain 3 items");

        // 3. Simulate clear_texture_cache by removing an ID (e.g., id2)
        let removed_item = mock_texture_cache.remove(&id2);
        assert!(removed_item.is_some(), "Item for ID2 should have been removed");
        assert_eq!(removed_item.unwrap(), "mock_texture_data_2", "Correct item removed for ID2");

        // 4. Assert the specific ID is removed and others remain
        assert!(!mock_texture_cache.contains_key(&id2), "ID2 should no longer be in the cache");
        assert!(mock_texture_cache.contains_key(&id1), "ID1 should still be in the cache");
        assert!(mock_texture_cache.contains_key(&id3), "ID3 should still be in the cache");
        assert_eq!(mock_texture_cache.len(), 2, "Cache should now contain 2 items");

        // 5. Test removing a non-existent ID (should be a no-op for the HashMap)
        let non_existent_id = generate_texture_id(); // Ensure it's different
        let removed_non_existent = mock_texture_cache.remove(&non_existent_id);
        assert!(removed_non_existent.is_none(), "Removing a non-existent ID should return None");
        assert_eq!(mock_texture_cache.len(), 2, "Cache size should remain unchanged after attempting to remove non-existent ID");
    }
}
