#version 450

layout(location = 0) in vec2 fragTexCoord;

// ANCHOR[GLSL_Fragment_UBO]
layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 mvp_matrix;       // Not typically used in fragment shader
    vec4 surface_props;    // x: alpha_multiplier, yzw: unused or for future
} ubo;

// ANCHOR[GLSL_Fragment_Sampler]
layout(set = 0, binding = 1) uniform sampler2D texSampler;

// ANCHOR[GLSL_Fragment_PushConstants]
layout(push_constant) uniform PushConstants {
    vec4 tint_color;
    vec2 offset;        // Screen-space offset of the element
    vec2 element_size;  // Screen-space size of the element
} pushConstants;

layout(location = 0) out vec4 outColor;

void main() {
    // ANCHOR[GLSL_Fragment_ColorCalc]
    vec4 texColor = texture(texSampler, fragTexCoord);

    // Apply tint color (RGB part) to texture color (RGB part)
    vec3 tintedRgb = texColor.rgb * pushConstants.tint_color.rgb;

    // Combine tinted RGB with original texture alpha, then apply overall surface alpha and tint alpha
    float combinedAlpha = texColor.a * pushConstants.tint_color.a * ubo.surface_props.x; // surface_props.x is alpha_multiplier

    outColor = vec4(tintedRgb, combinedAlpha);

    // Optional: Output pre-multiplied alpha if required by blend state
    // ANCHOR[GLSL_Fragment_PremultAlpha]
    outColor.rgb *= outColor.a;
}
