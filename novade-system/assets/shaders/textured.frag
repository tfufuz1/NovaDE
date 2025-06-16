#version 450

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

// ANCHOR: Texture Sampler Binding
layout(set = 0, binding = 0) uniform sampler2D texSampler;

void main() {
    outColor = texture(texSampler, fragTexCoord);
    // For testing without a real texture, you could output fragTexCoord:
    // outColor = vec4(fragTexCoord, 0.0, 1.0);
}
