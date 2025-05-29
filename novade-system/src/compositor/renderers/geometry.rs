use glow;
use std::rc::Rc;
use std::mem;
use bytemuck::{Pod, Zeroable};

// Assuming egl_context, shader, egl_surface, texture, and framebuffer are in the same parent module (super)
use super::egl_context::{GlContext, OpenGLError};
use super::shader::{ShaderProgram, ShaderError, load_textured_shader_program}; 
use super::egl_surface::{EGLSurfaceWrapper, EGLSurfaceError};
use super::texture::{Texture, TextureError, TEXTURED_VERTEX_SHADER_SRC, TEXTURED_FRAGMENT_SHADER_SRC, create_dummy_texture};
use super::framebuffer::{Framebuffer, FramebufferError}; 
use super::client_buffer::{ClientTexture, ClientBufferError}; // Import ClientTexture

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2], // Color is now replaced by texture coordinates
}

// --- VertexBuffer ---
pub struct VertexBuffer {
    gl: Rc<glow::Context>,
    id: glow::Buffer,
    target: u32, // e.g., glow::ARRAY_BUFFER or glow::ELEMENT_ARRAY_BUFFER
    // usage: u32, // e.g., glow::STATIC_DRAW - Stored if needed for re-buffering, otherwise not strictly necessary
}

impl VertexBuffer {
    pub fn new<T: Pod>(
        gl: Rc<glow::Context>,
        data: &[T],
        target: u32,
        usage: u32,
    ) -> Result<Self, OpenGLError> {
        let id = unsafe { gl.create_buffer() }
            .map_err(|e| OpenGLError::Other(format!("Failed to create buffer: {}", e)))?; // Use a more specific error if available in OpenGLError

        unsafe {
            gl.bind_buffer(target, Some(id));
            gl.buffer_data_u8_slice(target, bytemuck::cast_slice(data), usage);
            // Unbind after configuration is good practice, though not strictly necessary
            // if the next operation will bind another buffer or this one again.
            gl.bind_buffer(target, None); 
        }
        // Check for GL errors if necessary, though glow calls often panic on error directly
        // or return Result where applicable. buffer_data_u8_slice does not return Result.
        // let err = unsafe { gl.get_error() };
        // if err != glow::NO_ERROR { return Err(OpenGLError::Other(format!("OpenGL error after buffer data: {}", err))); }

        Ok(Self { gl, id, target })
    }

    pub fn bind(&self) {
        unsafe { self.gl.bind_buffer(self.target, Some(self.id)) };
    }

    pub fn unbind(&self) {
        unsafe { self.gl.bind_buffer(self.target, None) };
    }
    
    pub fn id(&self) -> glow::Buffer {
        self.id
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe { self.gl.delete_buffer(self.id) };
    }
}

// --- VertexArray (VAO) ---
pub struct VertexArray {
    gl: Rc<glow::Context>,
    id: glow::VertexArray,
}

impl VertexArray {
    pub fn new(gl: Rc<glow::Context>) -> Result<Self, OpenGLError> {
        let id = unsafe { gl.create_vertex_array() }
            .map_err(|e| OpenGLError::Other(format!("Failed to create vertex array: {}", e)))?;
        Ok(Self { gl, id })
    }

    pub fn bind(&self) {
        unsafe { self.gl.bind_vertex_array(Some(self.id)) };
    }

    pub fn unbind(&self) {
        unsafe { self.gl.bind_vertex_array(None) };
    }

    /// Configures a vertex attribute pointer for this VAO.
    /// Assumes the VAO is already bound externally if optimizing, or binds internally.
    /// `vbo` is the VertexBuffer to associate with this attribute.
    /// `index` is the attribute location (e.g., `layout (location = 0)`).
    /// `size` is the number of components per vertex attribute (e.g., 2 for vec2).
    /// `data_type` is the type of data (e.g., `glow::FLOAT`).
    /// `normalized` specifies if fixed-point data values should be normalized.
    /// `stride` is the byte offset between consecutive generic vertex attributes.
    /// `offset` is the byte offset of the first component of the first attribute.
    pub fn configure_vertex_attribute(
        &self,
        vbo: &VertexBuffer, // Pass VBO to bind it for this attribute configuration
        index: u32,
        size: i32,
        data_type: u32,
        normalized: bool,
        stride: i32,
        offset: i32,
    ) {
        self.bind(); // Bind this VAO
        vbo.bind();   // Bind the VBO
        unsafe {
            self.gl.vertex_attrib_pointer_f32(index, size, data_type, normalized, stride, offset);
            self.gl.enable_vertex_attrib_array(index);
        }
        // Unbind VBO after configuration, VAO can remain bound if further configuration follows.
        vbo.unbind(); 
        // self.unbind(); // Optional: unbind VAO if no more attributes for it now
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe { self.gl.delete_vertex_array(self.id) };
    }
}

// --- SimpleRenderer ---
// Combined error type for SimpleRenderer::new
#[derive(Debug)]
pub enum RendererSetupError {
    ShaderError(ShaderError),
    OpenGLError(OpenGLError),
    TextureError(TextureError),
    FramebufferError(FramebufferError), // Added FramebufferError variant
}
impl From<ShaderError> for RendererSetupError { fn from(e: ShaderError) -> Self { RendererSetupError::ShaderError(e) } }
impl From<OpenGLError> for RendererSetupError { fn from(e: OpenGLError) -> Self { RendererSetupError::OpenGLError(e) } }
impl From<TextureError> for RendererSetupError { fn from(e: TextureError) -> Self { RendererSetupError::TextureError(e) } }
impl From<FramebufferError> for RendererSetupError { fn from(e: FramebufferError) -> Self { RendererSetupError::FramebufferError(e) } }


impl std::fmt::Display for RendererSetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RendererSetupError::ShaderError(e) => write!(f, "Shader error: {:?}", e),
            RendererSetupError::OpenGLError(e) => write!(f, "OpenGL error: {:?}", e),
            RendererSetupError::TextureError(e) => write!(f, "Texture error: {:?}", e),
            RendererSetupError::FramebufferError(e) => write!(f, "Framebuffer error: {:?}", e),
        }
    }
}
impl std::error::Error for RendererSetupError {}


pub struct SimpleRenderer {
    gl: Rc<glow::Context>,
    shader_program: ShaderProgram,
    texture: Texture, // Added texture field
    vao: VertexArray,
    _vbo: VertexBuffer, 
    vertex_count: i32,
}

impl SimpleRenderer {
    pub fn new(gl: Rc<glow::Context>) -> Result<Self, RendererSetupError> {
        // Load textured shader program
        let shader_program = load_textured_shader_program(
            Rc::clone(&gl),
            TEXTURED_VERTEX_SHADER_SRC,
            TEXTURED_FRAGMENT_SHADER_SRC,
        )?;

        // Load a texture (using dummy texture for now, replace with file loading if needed)
        // To use Texture::new_from_file, a path to an image is needed.
        // For simplicity in this step, using a dummy texture.
        // let texture = Texture::new_from_file(Rc::clone(&gl), Path::new("assets/test_texture.png"))?;
        let texture = create_dummy_texture(Rc::clone(&gl))?;


        // Vertex data with texture coordinates
        let vertices: [Vertex; 6] = [
            Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0] }, // Bottom-left
            Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 0.0] }, // Bottom-right
            Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 1.0] }, // Top-left
            
            Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 0.0] }, // Bottom-right
            Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 1.0] }, // Top-right
            Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 1.0] }, // Top-left
        ];
        let vertex_count = vertices.len() as i32;

        let vbo = VertexBuffer::new(Rc::clone(&gl), &vertices, glow::ARRAY_BUFFER, glow::STATIC_DRAW)?;
        let vao = VertexArray::new(Rc::clone(&gl))?;

        vao.bind();
        vbo.bind();

        let stride = mem::size_of::<Vertex>() as i32;
        // Position attribute (location = 0)
        unsafe {
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(0);
        }
        // Texture coordinates attribute (location = 1)
        let tex_coords_offset = mem::size_of::<[f32; 2]>() as i32; // Offset of tex_coords field
        unsafe {
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, tex_coords_offset);
            gl.enable_vertex_attrib_array(1);
        }
        
        vao.unbind();
        vbo.unbind();

        Ok(Self {
            gl,
            shader_program,
            texture, // Store the texture
            vao,
            _vbo: vbo,
            vertex_count,
        })
    }

    pub fn draw(&self, surface: &EGLSurfaceWrapper) -> Result<(), EGLSurfaceError> {
        surface.make_current()?;

        unsafe {
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0); // Clear window background
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        self.shader_program.use_program();
        self.texture.bind(0); // Bind the original texture loaded by SimpleRenderer
        if let Err(e) = self.shader_program.set_uniform_1i("textureSampler", 0) {
            eprintln!("Failed to set textureSampler uniform: {:?}", e);
            // Potentially return an error
        }

        self.vao.bind();
        unsafe {
            self.gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count);
        }
        self.vao.unbind();
        
        self.texture.unbind(0);
        self.shader_program.unuse_program();

        surface.swap_buffers()?;
        surface.release_current()?;
        Ok(())
    }

    /// Renders the stored texture (e.g. a pre-rendered scene from an FBO) to the given EGLSurfaceWrapper.
    /// This method assumes the VAO/VBO describe a quad that can map the texture.
    pub fn draw_texture_to_surface(
        &self,
        surface: &EGLSurfaceWrapper,
        texture_to_draw: &Texture, // The texture to draw (e.g., FBO's color attachment)
    ) -> Result<(), EGLSurfaceError> {
        surface.make_current()?;
        unsafe {
            // Set viewport to surface dimensions
            self.gl.viewport(0, 0, surface.width(), surface.height());
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0); // Black background for screen
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        self.shader_program.use_program();
        texture_to_draw.bind(0); // Bind the provided texture
        if let Err(e) = self.shader_program.set_uniform_1i("textureSampler", 0) {
            eprintln!("Failed to set textureSampler uniform: {:?}", e);
            // Potentially return an error
        }

        self.vao.bind(); // Use the renderer's quad VAO
        unsafe {
            self.gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count);
        }
        self.vao.unbind();

        texture_to_draw.unbind(0);
        self.shader_program.unuse_program();

        surface.swap_buffers()?;
        surface.release_current()?;
        Ok(())
    }


    /// Renders the SimpleRenderer's own texture to the specified Framebuffer.
    pub fn draw_to_fbo(&self, fbo: &Framebuffer) -> Result<(), RendererSetupError> {
        fbo.bind(); // Bind FBO and sets viewport

        unsafe {
            self.gl.clear_color(0.5, 0.2, 0.2, 1.0); // Different clear color for FBO
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        self.shader_program.use_program();
        self.texture.bind(0); // Bind the SimpleRenderer's own internal texture
        
        self.shader_program.set_uniform_1i("textureSampler", 0)?;
        
        self.vao.bind();
        unsafe {
            self.gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count);
        }
        self.vao.unbind();

        self.texture.unbind(0);
        self.shader_program.unuse_program();

        fbo.unbind(); // Unbind FBO, viewport should be reset by next target
        Ok(())
    }

    /// Renders a ClientTexture (e.g., from an SHM or DMA-BUF) to the specified EGLSurfaceWrapper.
    /// Assumes the ClientTexture contains the content to be drawn.
    /// The viewport is set to the EGLSurfaceWrapper's dimensions.
    pub fn render_client_texture_to_surface(
        &self,
        surface: &EGLSurfaceWrapper,
        client_texture: &ClientTexture,
        // TODO: Add transformation matrix if needed
    ) -> Result<(), RendererSetupError> { // Changed error type for consistency
        surface.make_current().map_err(RendererSetupError::from)?;
        unsafe {
            self.gl.viewport(0, 0, surface.width(), surface.height());
            self.gl.clear_color(0.0, 0.0, 0.2, 1.0); // Dark blue background for client content
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        self.shader_program.use_program();
        client_texture.bind(0); // Bind the client's texture

        self.shader_program.set_uniform_1i("textureSampler", 0)?;
        
        self.vao.bind(); // Use the renderer's quad VAO
        unsafe {
            self.gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count);
        }
        self.vao.unbind();

        client_texture.unbind(0);
        self.shader_program.unuse_program();

        surface.swap_buffers().map_err(RendererSetupError::from)?;
        surface.release_current().map_err(RendererSetupError::from)?;
        Ok(())
    }

    // --- Scissor Test Methods ---
    pub fn enable_scissor_test(&self) {
        unsafe { self.gl.enable(glow::SCISSOR_TEST) };
    }

    pub fn disable_scissor_test(&self) {
        unsafe { self.gl.disable(glow::SCISSOR_TEST) };
    }

    /// Sets the scissor box.
    /// IMPORTANT: y is specified from the bottom of the viewport in OpenGL,
    /// while Wayland/Smithay typically use y from top. Conversion is needed.
    pub fn set_scissor_box(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { self.gl.scissor(x, y, width, height) };
    }

    /// Example method demonstrating rendering a texture with damage regions using scissor test.
    /// `damage_rects` are expected in surface coordinates (y-down from top).
    /// `surface_height` is the total height of the rendering surface (EGLSurfaceWrapper).
    pub fn draw_texture_to_surface_with_damage(
        &self,
        surface: &EGLSurfaceWrapper,
        texture_to_draw: &Texture,
        damage_rects: &[smithay::utils::Rectangle<i32, smithay::utils::Physical>], // Smithay type
        surface_height: i32, 
    ) -> Result<(), RendererSetupError> {
        surface.make_current().map_err(RendererSetupError::from)?;
        
        unsafe {
            self.gl.viewport(0, 0, surface.width(), surface.height()); // Ensure viewport is set
            // Clear the whole surface once (optional, could clear per scissor rect if preferred)
            // self.gl.clear_color(0.0, 0.0, 0.0, 1.0); 
            // self.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        self.shader_program.use_program();
        texture_to_draw.bind(0);
        self.shader_program.set_uniform_1i("textureSampler", 0)?;
        self.vao.bind();

        if damage_rects.is_empty() {
            // No specific damage, draw the whole texture without scissor test.
            // This might be an initial full draw or if damage tracking is not used.
            self.disable_scissor_test();
            unsafe { self.gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count); }
        } else {
            self.enable_scissor_test();
            for rect in damage_rects {
                let gl_x = rect.loc.x;
                // Convert Y: Smithay y is from top, GL y is from bottom.
                let gl_y = surface_height - (rect.loc.y + rect.size.h);
                let gl_w = rect.size.w;
                let gl_h = rect.size.h;

                // Basic clamping to ensure scissor box is within surface bounds.
                // More sophisticated clipping might be needed depending on coordinate systems.
                let final_x = gl_x.max(0);
                let final_y = gl_y.max(0);
                let final_w = gl_w.min(surface.width() - final_x);
                let final_h = gl_h.min(surface.height() - final_y);

                if final_w > 0 && final_h > 0 {
                    self.set_scissor_box(final_x, final_y, final_w, final_h);
                    unsafe { self.gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count); }
                }
            }
            self.disable_scissor_test();
        }

        self.vao.unbind();
        texture_to_draw.unbind(0);
        self.shader_program.unuse_program();

        surface.swap_buffers().map_err(RendererSetupError::from)?;
        surface.release_current().map_err(RendererSetupError::from)?;
        Ok(())
    }
}

// Note: The `SimpleRenderer::new` method returns `Result<Self, RendererSetupError>`
// in the prompt, but here it's `Result<Self, RendererSetupError>`.
// `RendererSetupError` can be converted to `Box<dyn std::error::Error>` if needed by the caller:
// `SimpleRenderer::new(gl).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)`
// Using a specific error enum is generally better practice within the module.

// The `vao.configure_vertex_attribute` helper was not used in `SimpleRenderer::new`
// as the setup was done directly for clarity there. It can be used as:
// vao.configure_vertex_attribute(&vbo, 0, 2, glow::FLOAT, false, stride, 0);
// vao.configure_vertex_attribute(&vbo, 1, 3, glow::FLOAT, false, stride, color_offset);
// This would encapsulate the bind/unbind logic within configure_vertex_attribute if preferred.
// The current direct usage in SimpleRenderer::new is also fine and common.
