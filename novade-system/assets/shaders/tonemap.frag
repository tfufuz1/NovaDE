// ANCHOR [ToneMappingFragmentShader]
@group(0) @binding(0) var t_source: texture_2d<f32>;
@group(0) @binding(1) var s_source: sampler;

struct ToneMapUniforms {
    // max_luminance: f32, // Could be used for more advanced tone mappers
    exposure: f32,
};
@group(1) @binding(0) var<uniform> settings: ToneMapUniforms;

// Simple Reinhard tone mapping: C_out = C_in / (C_in + 1.0)
// Exposure is applied before tone mapping: C_exposed = exposure * C_in
fn reinhard(color: vec3<f32>) -> vec3<f32> {
    return color / (color + vec3<f32>(1.0));
}

@fragment
fn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(t_source, s_source, tex_coords);

    // Apply exposure
    let exposed_color = hdr_color.rgb * settings.exposure;

    // Apply Reinhard tone mapping
    let sdr_color = reinhard(exposed_color);

    // It's generally better to clamp here to ensure output is strictly [0,1] if
    // subsequent passes or the display expect it.
    let clamped_sdr_color = clamp(sdr_color, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(clamped_sdr_color, hdr_color.a);
}
