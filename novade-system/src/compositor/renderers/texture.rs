use glow;
use image;
use std::path::Path;
use std::rc::Rc;

use super::egl_context::OpenGLError; // Assuming OpenGLError is accessible

// Shader sources for textured rendering
pub const TEXTURED_VERTEX_SHADER_SRC: &str = r#"#version 300 es
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoords;
out vec2 TexCoords;
void main() {
    gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);
    TexCoords = aTexCoords;
}"#;

pub const TEXTURED_FRAGMENT_SHADER_SRC: &str = r#"#version 300 es
precision mediump float;
out vec4 FragColor;
in vec2 TexCoords;
uniform sampler2D textureSampler;
void main() {
    FragColor = texture(textureSampler, TexCoords);
}"#;

#[derive(Debug)]
pub enum TextureError {
    ImageError(image::ImageError),
    OpenGLCall(String), // Changed from OpenGLError to String for more flexibility from glow calls
    UnsupportedFormat(String),
    DimensionTooLarge(String),
    InternalError(String), // For unexpected issues like create_texture failing
}

impl From<image::ImageError> for TextureError {
    fn from(e: image::ImageError) -> Self {
        TextureError::ImageError(e)
    }
}

// If OpenGLError needs to be converted, ensure it provides a suitable string message
impl From<OpenGLError> for TextureError {
    fn from(e: OpenGLError) -> Self {
        TextureError::OpenGLCall(format!("{:?}", e)) // Or more specific mapping
    }
}

pub struct Texture {
    gl: Rc<glow::Context>,
    id: glow::Texture,
    width: u32,
    height: u32,
    format: u32, // OpenGL format enum, e.g., glow::RGBA
}

impl Texture {
    /// Creates a new texture from raw image data.
    pub fn new_from_data(
        gl: Rc<glow::Context>,
        width: u32,
        height: u32,
        data: Option<&[u8]>, // Changed to Option<&[u8]>
        format: u32,        // Format of the pixel data provided (e.g., glow::RGBA)
        internal_format: i32, // Internal format to store in OpenGL (e.g., glow::RGBA8)
    ) -> Result<Self, TextureError> {
        if width == 0 || height == 0 {
            return Err(TextureError::DimensionTooLarge("Width or height cannot be 0".to_string()));
        }
        // Consider adding a max texture size check against gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE)

        let texture_id = unsafe { gl.create_texture() }
            .map_err(|e| TextureError::InternalError(format!("glCreateTexture failed: {}", e)))?;

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));

            // Set texture parameters
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            // Load texture data
            // The `border` parameter (0) is legacy and should be 0.
            // `type_` is glow::UNSIGNED_BYTE for u8 data.
            gl.tex_image_2d(
                glow::TEXTURE_2D,      // target
                0,                     // level (mipmap level)
                internal_format,       // internal_format (how GL stores it)
                width as i32,          // width
                height as i32,         // height
                0,                     // border (must be 0)
                format,                // format (of the data provided)
                glow::UNSIGNED_BYTE,   // type (of the data provided)
                data,                  // data (now an Option<&[u8]>)
            );

            // Optional: Generate mipmaps if min_filter uses mipmaps
            // if min_filter == glow::LINEAR_MIPMAP_LINEAR { gl.generate_mipmap(glow::TEXTURE_2D); }

            gl.bind_texture(glow::TEXTURE_2D, None); // Unbind
        }
        // Check for GL errors after critical operations
        // let error_code = unsafe { gl.get_error() };
        // if error_code != glow::NO_ERROR {
        //     unsafe { gl.delete_texture(texture_id) }; // Clean up allocated texture
        //     return Err(TextureError::OpenGLCall(format!("OpenGL error after texture creation: {}", error_code)));
        // }


        Ok(Self {
            gl,
            id: texture_id,
            width,
            height,
            format,
        })
    }
    
    /// Creates a new texture by loading an image from a file path.
    /// Converts the image to RGBA8 format.
    pub fn new_from_file(gl: Rc<glow::Context>, path: &Path) -> Result<Self, TextureError> {
        let img = image::open(path)?.to_rgba8(); // Convert to RGBA8 for simplicity
        let (width, height) = img.dimensions();
        let data = img.into_raw(); // This gives Vec<u8>

        // For RGBA8 data:
        // format is glow::RGBA
        // internal_format is glow::RGBA8 (or just glow::RGBA if GL should decide sized format)
        Self::new_from_data(gl, width, height, Some(&data), glow::RGBA, glow::RGBA8)
    }


    /// Binds the texture to a specific texture unit.
    pub fn bind(&self, texture_unit: u32) {
        // texture_unit is 0 for TEXTURE0, 1 for TEXTURE1, etc.
        unsafe {
            self.gl.active_texture(glow::TEXTURE0 + texture_unit);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.id));
        }
    }

    /// Unbinds the texture from a specific texture unit.
    pub fn unbind(&self, texture_unit: u32) {
        unsafe {
            self.gl.active_texture(glow::TEXTURE0 + texture_unit);
            self.gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    pub fn id(&self) -> glow::Texture { self.id }
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn format(&self) -> u32 { self.format } // GLenum for format
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.id);
        }
    }
}

// Example: Create a dummy 1x1 white texture if no file is available
pub fn create_dummy_texture(gl: Rc<glow::Context>) -> Result<Texture, TextureError> {
    let data: [u8; 4] = [255, 255, 255, 255]; // White pixel
    Texture::new_from_data(gl, 1, 1, Some(&data), glow::RGBA, glow::RGBA8)
}
