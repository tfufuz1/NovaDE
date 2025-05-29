use glow;
use std::rc::Rc;
use std::io; // For ShaderError::IOError

// Default GLSL shader sources
pub const DEFAULT_VERTEX_SHADER_SRC: &str = r#"#version 300 es
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec3 aColor;
out vec3 fColor;
void main() {
    gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);
    fColor = aColor;
}"#;

pub const DEFAULT_FRAGMENT_SHADER_SRC: &str = r#"#version 300 es
precision mediump float;
in vec3 fColor;
out vec4 FragColor;
void main() {
    FragColor = vec4(fColor, 1.0);
}"#;

#[derive(Debug)]
pub enum ShaderError {
    CompileError { shader_type: String, log: String },
    LinkError { log: String },
    InvalidShaderType(String), // For when a program gets mismatched shader types
    IOError(io::Error), // If loading shaders from files
    InternalError(String), // For unexpected issues
    UniformNotFound(String), // For when a uniform location is not found
}

impl From<io::Error> for ShaderError {
    fn from(e: io::Error) -> Self {
        ShaderError::IOError(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    fnto_glow_type(self) -> u32 {
        match self {
            ShaderType::Vertex => glow::VERTEX_SHADER,
            ShaderType::Fragment => glow::FRAGMENT_SHADER,
        }
    }

    fnto_string(self) -> String {
        match self {
            ShaderType::Vertex => "Vertex Shader".to_string(),
            ShaderType::Fragment => "Fragment Shader".to_string(),
        }
    }
}

pub struct Shader {
    gl: Rc<glow::Context>,
    id: glow::Shader,
    shader_type: ShaderType, // To keep track for debugging or potential reuse
}

impl Shader {
    pub fn new(gl: Rc<glow::Context>, source: &str, shader_type: ShaderType) -> Result<Self, ShaderError> {
        let shader_id = unsafe { gl.create_shader(shader_type.to_glow_type()) }
            .map_err(|e| ShaderError::InternalError(format!("Failed to create shader object: {}", e)))?;

        unsafe {
            gl.shader_source(shader_id, source);
            gl.compile_shader(shader_id);
        }

        if !unsafe { gl.get_shader_compile_status(shader_id) } {
            let log = unsafe { gl.get_shader_info_log(shader_id) };
            // It's good practice to delete the shader if compilation failed
            unsafe { gl.delete_shader(shader_id) };
            return Err(ShaderError::CompileError {
                shader_type: shader_type.to_string(),
                log,
            });
        }

        Ok(Self {
            gl,
            id: shader_id,
            shader_type,
        })
    }

    pub fn id(&self) -> glow::Shader {
        self.id
    }

    pub fn shader_type(&self) -> ShaderType {
        self.shader_type
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_shader(self.id);
        }
    }
}

pub struct ShaderProgram {
    gl: Rc<glow::Context>,
    id: glow::Program,
}

impl ShaderProgram {
    pub fn new(gl: Rc<glow::Context>, vertex_shader: &Shader, fragment_shader: &Shader) -> Result<Self, ShaderError> {
        if vertex_shader.shader_type() != ShaderType::Vertex {
            return Err(ShaderError::InvalidShaderType("Vertex shader provided is not of ShaderType::Vertex".to_string()));
        }
        if fragment_shader.shader_type() != ShaderType::Fragment {
            return Err(ShaderError::InvalidShaderType("Fragment shader provided is not of ShaderType::Fragment".to_string()));
        }

        let program_id = unsafe { gl.create_program() }
            .map_err(|e| ShaderError::InternalError(format!("Failed to create program object: {}", e)))?;

        unsafe {
            gl.attach_shader(program_id, vertex_shader.id());
            gl.attach_shader(program_id, fragment_shader.id());
            gl.link_program(program_id);
        }

        if !unsafe { gl.get_program_link_status(program_id) } {
            let log = unsafe { gl.get_program_info_log(program_id) };
            // It's good practice to delete the program if linking failed
            unsafe { gl.delete_program(program_id) };
            return Err(ShaderError::LinkError { log });
        }

        // Detach shaders after successful linking as they are no longer needed by the program object.
        // The Shader struct's Drop impl will delete them when they go out of scope.
        unsafe {
            gl.detach_shader(program_id, vertex_shader.id());
            gl.detach_shader(program_id, fragment_shader.id());
        }

        Ok(Self {
            gl,
            id: program_id,
        })
    }

    pub fn id(&self) -> glow::Program {
        self.id
    }

    /// Activates this shader program for use in subsequent rendering commands.
    pub fn use_program(&self) {
        unsafe {
            self.gl.use_program(Some(self.id));
        }
    }
    
    /// Deactivates any currently active shader program.
    pub fn unuse_program(&self) {
        unsafe {
            self.gl.use_program(None);
        }
    }


    pub fn get_attrib_location(&self, name: &str) -> Option<u32> {
        unsafe { self.gl.get_attrib_location(self.id, name) }
    }

    pub fn get_uniform_location(&self, name: &str) -> Option<glow::UniformLocation> {
        unsafe { self.gl.get_uniform_location(self.id, name) }
    }

    /// Sets an integer uniform value for this shader program.
    /// The program must be active (`use_program()`) before calling this.
    pub fn set_uniform_1i(&self, name: &str, value: i32) -> Result<(), ShaderError> {
        // Consider if use_program should be called here or be a precondition.
        // self.use_program(); // Alternatively, ensure program is used before this call.
        match unsafe { self.gl.get_uniform_location(self.id, name) } {
            Some(location) => {
                unsafe { self.gl.uniform_1_i32(Some(&location), value) };
                Ok(())
            }
            None => Err(ShaderError::UniformNotFound(name.to_string())),
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            // Ensure no program is active before deleting, or that this isn't the active one in a critical path.
            // Current Glow behavior might not require this, but some GL state machines can be tricky.
            // self.gl.use_program(None); // Optional: unbind before delete if issues arise.
            self.gl.delete_program(self.id);
        }
    }
}

/// Helper function to load, compile, and link shaders from source strings.
pub fn load_shader_program_from_source(
    gl: Rc<glow::Context>,
    vs_src: &str,
    fs_src: &str,
) -> Result<ShaderProgram, ShaderError> {
    let vertex_shader = Shader::new(Rc::clone(&gl), vs_src, ShaderType::Vertex)?;
    let fragment_shader = Shader::new(Rc::clone(&gl), fs_src, ShaderType::Fragment)?;
    
    // Shader objects can be dropped here if ShaderProgram::new detaches them and
    // they are not needed further. If new() doesn't detach, they must live as long as ShaderProgram.
    // Current impl of ShaderProgram::new detaches them, so this is fine.
    ShaderProgram::new(gl, &vertex_shader, &fragment_shader)
}

// Example of loading default shaders
pub fn load_default_shader_program(gl: Rc<glow::Context>) -> Result<ShaderProgram, ShaderError> {
    load_shader_program_from_source(gl, DEFAULT_VERTEX_SHADER_SRC, DEFAULT_FRAGMENT_SHADER_SRC)
}

/// Loads the textured shader program using the shader sources defined in texture.rs (or this file).
/// This function assumes TEXTURED_VERTEX_SHADER_SRC and TEXTURED_FRAGMENT_SHADER_SRC are accessible.
/// If they are in texture.rs, this function might better reside there or take sources as args.
/// For now, let's assume they are accessible via super::texture or similar if moved.
/// Or, if they are defined in this file (e.g. if texture.rs also defines them, pick one source of truth).
/// For this step, let's use placeholder names and assume they will be available.
pub fn load_textured_shader_program(
    gl: Rc<glow::Context>,
    textured_vs_src: &str, // Pass source directly
    textured_fs_src: &str, // Pass source directly
) -> Result<ShaderProgram, ShaderError> {
    load_shader_program_from_source(gl, textured_vs_src, textured_fs_src)
}

// Note: The GlContext (from egl_context.rs) is needed to get the Rc<glow::Context>.
// The user of this shader module will typically get the Rc<glow::Context> from their GlContext instance
// (e.g., `gl_context.gl()`) and pass it into these shader functions.
