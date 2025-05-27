use glow::{Context, HasContext, Shader, Program};
use super::errors::Gles2RendererError; // Assuming errors.rs is in the same gles2 module

pub const VS_TEXTURE_SRC: &str = r#"
    #version 100
    attribute vec2 position;
    attribute vec2 tex_coord;
    varying vec2 v_tex_coord;
    uniform mat3 mvp; // Model-View-Projection matrix
    void main() {
        gl_Position = vec4(mvp * vec3(position, 1.0), 1.0);
        v_tex_coord = tex_coord;
    }
"#;

pub const FS_TEXTURE_SRC: &str = r#"
    #version 100
    precision mediump float;
    varying vec2 v_tex_coord;
    uniform sampler2D u_texture;
    void main() {
        gl_FragColor = texture2D(u_texture, v_tex_coord);
    }
"#;

pub const VS_SOLID_SRC: &str = r#"
    #version 100
    attribute vec2 position;
    uniform mat3 mvp;
    void main() {
        gl_Position = vec4(mvp * vec3(position, 1.0), 1.0);
    }
"#;

pub const FS_SOLID_SRC: &str = r#"
    #version 100
    precision mediump float;
    uniform vec4 u_color;
    void main() {
        gl_FragColor = u_color;
    }
"#;

pub fn compile_shader(
    gl: &Context,
    shader_type: u32, // e.g., glow::VERTEX_SHADER or glow::FRAGMENT_SHADER
    source: &str,
) -> Result<Shader, Gles2RendererError> {
    unsafe {
        let shader = gl.create_shader(shader_type).map_err(|e| {
            Gles2RendererError::ShaderCompilationFailed {
                shader_type: format!("Type {}", shader_type),
                error_log: format!("Failed to create shader object: {}", e),
            }
        })?;
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let error_log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader); // Avoid leaking shader
            Err(Gles2RendererError::ShaderCompilationFailed {
                shader_type: format!("Type {}", shader_type),
                error_log,
            })
        } else {
            Ok(shader)
        }
    }
}

pub fn link_program(
    gl: &Context,
    vs: Shader,
    fs: Shader,
) -> Result<Program, Gles2RendererError> {
    unsafe {
        let program = gl.create_program().map_err(|e| {
            Gles2RendererError::ShaderProgramLinkFailed(format!("Failed to create program object: {}", e))
        })?;
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            let error_log = gl.get_program_info_log(program);
            // Detach and delete shaders if program link failed to allow them to be cleaned up
            gl.detach_shader(program, vs);
            gl.detach_shader(program, fs);
            // Note: shaders are typically deleted by the caller after successful linking if they are no longer needed.
            // If linking fails, they should also be cleaned up if they were meant to be temporary.
            // For simplicity here, we assume the caller will handle shader deletion if link_program is part of a larger setup.
            gl.delete_program(program); // Avoid leaking program
            Err(Gles2RendererError::ShaderProgramLinkFailed(error_log))
        } else {
            // Shaders can be detached and deleted after successful linking
            // gl.detach_shader(program, vs);
            // gl.detach_shader(program, fs);
            // gl.delete_shader(vs);
            // gl.delete_shader(fs);
            Ok(program)
        }
    }
}
