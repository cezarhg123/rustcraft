#version 450

layout(location = 0) out vec4 fragColor;

layout(location = 0) in vec2 UV;

layout(binding = 1) uniform sampler2D tex;

void main() {
    fragColor = texture(tex, UV);
}