#version 450

// ANCHOR[GLSL_Vertex_Input]
layout(location = 0) in vec2 inPosition;   // Unit quad vertices, e.g., (-0.5,-0.5) to (0.5,0.5)
layout(location = 1) in vec2 inTexCoord;   // Texture coordinates, e.g., (0,0) to (1,1)

// ANCHOR[GLSL_Vertex_UBO]
layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 mvp_matrix;       // Combined Model-View-Projection matrix
    vec4 surface_props;    // x: alpha_multiplier, yzw: unused or for future
} ubo;

// ANCHOR[GLSL_Vertex_PushConstants]
layout(push_constant) uniform PushConstants {
    vec4 tint_color;       // Not typically used in vertex shader for this setup
    vec2 offset;           // Screen-space offset (can be used for minor adjustments if needed)
    vec2 element_size;     // Screen-space size (can be used for minor adjustments if needed)
} pushConstants;

layout(location = 0) out vec2 fragTexCoord;

void main() {
    // ANCHOR[GLSL_Vertex_PositionCalc]
    gl_Position = ubo.mvp_matrix * vec4(inPosition, 0.0, 1.0); // Z can be 0 for 2D
    fragTexCoord = inTexCoord;
}
