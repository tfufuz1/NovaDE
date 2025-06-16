#version 450
// ANCHOR[GammaFrag_Shader]

layout(location = 0) in vec2 fragTexCoord;

layout(set = 0, binding = 0) uniform sampler2D inputTexture;

layout(push_constant) uniform GammaPushConstants {
    float gamma_value;
} pc;

layout(location = 0) out vec4 outColor;

void main() {
    vec4 texColor = texture(inputTexture, fragTexCoord);
    // Apply gamma correction to RGB, pass alpha through.
    // Assumes input texColor.rgb is linear (non-PMA).
    vec3 correctedRgb = pow(texColor.rgb, vec3(1.0 / pc.gamma_value));
    outColor = vec4(correctedRgb, texColor.a);

    // PMA considerations:
    // If the rendering pipeline expects PMA for blending on the swapchain,
    // the final blit shader (blit.frag.spv) would be responsible for converting
    // its input (which is the output of this gamma shader) to PMA.
    // This gamma shader outputs non-PMA color values if the input was non-PMA.
}
