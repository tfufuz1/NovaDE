// ANCHOR [GammaCorrectionFragmentShader]
@group(0) @binding(0) var t_source: texture_2d<f32>;
@group(0) @binding(1) var s_source: sampler;

struct GammaUniform {
    gamma_value: f32,
};
@group(1) @binding(0) var<uniform> gamma_settings: GammaUniform;

@fragment
fn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    let source_color = textureSample(t_source, s_source, tex_coords);

    // Assuming source_color is in linear space.
    // Apply gamma correction: color_out = color_in^(1/gamma)
    // If gamma_value is 0 or very small, pow might produce undefined results or NaNs.
    // Add a safeguard for gamma_value.
    var G: f32 = gamma_settings.gamma_value;
    if (G <= 0.0) {
        G = 2.2; // Default to a common gamma if input is invalid
    }
    let inv_gamma = 1.0 / G;

    let corrected_rgb = pow(source_color.rgb, vec3<f32>(inv_gamma));

    return vec4<f32>(corrected_rgb, source_color.a);
}
