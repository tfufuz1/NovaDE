// ANCHOR [CopyTextureFragmentShader]
@group(0) @binding(0) var t_source: texture_2d<f32>;
@group(0) @binding(1) var s_source: sampler;

@fragment
fn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(t_source, s_source, tex_coords);
}
