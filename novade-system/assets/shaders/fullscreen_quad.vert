// ANCHOR [FullscreenQuadVertexShader]
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Generate a full-screen triangle strip using 6 vertices
    // Clip: (-1,-1), (1,-1), (-1,1), (1,-1), (1,1), (-1,1)
    // UVs (texture origin top-left (0,0), bottom-right (1,1)):
    // (0,1) BL, (1,1) BR, (0,0) TL,
    // (1,1) BR, (1,0) TR, (0,0) TL
    let positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), // Bottom-left
        vec2<f32>( 1.0, -1.0), // Bottom-right
        vec2<f32>(-1.0,  1.0), // Top-left
        vec2<f32>( 1.0, -1.0), // Bottom-right - Re-use for second triangle
        vec2<f32>( 1.0,  1.0), // Top-right
        vec2<f32>(-1.0,  1.0)  // Top-left - Re-use for second triangle
    );

    let uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0), // Corresponds to (-1.0, -1.0) -> Bottom-left UV
        vec2<f32>(1.0, 1.0), // Corresponds to ( 1.0, -1.0) -> Bottom-right UV
        vec2<f32>(0.0, 0.0), // Corresponds to (-1.0,  1.0) -> Top-left UV
        vec2<f32>(1.0, 1.0), // Corresponds to ( 1.0, -1.0) -> Bottom-right UV
        vec2<f32>(1.0, 0.0), // Corresponds to ( 1.0,  1.0) -> Top-right UV
        vec2<f32>(0.0, 0.0)  // Corresponds to (-1.0,  1.0) -> Top-left UV
    );

    out.clip_position = vec4<f32>(positions[in_vertex_index], 0.0, 1.0);
    out.tex_coords = uvs[in_vertex_index];
    return out;
}
