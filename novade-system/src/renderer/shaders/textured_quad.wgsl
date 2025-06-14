// novade-system/src/renderer/shaders/textured_quad.wgsl

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

// Uniforms for transformation
@group(1) @binding(0) var<uniform> transform_matrix: mat3x3<f32>;
// transform_matrix will be used to scale and translate the quad.
// It should convert model coordinates (e.g., -0.5 to 0.5, or 0 to 1 for quad)
// to clip space coordinates (-1 to 1).

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    // Apply transformation: mat3x3 * vec3f(x, y, 1.0)
    // Input model.position is vec2f. We extend it to vec3f for matrix multiplication.
    let transformed_position = transform_matrix * vec3<f32>(model.position, 1.0);

    out.clip_position = vec4<f32>(transformed_position.xy, 0.0, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
