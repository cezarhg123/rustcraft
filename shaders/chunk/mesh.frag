#version 460

layout(location = 0) out vec4 out_color;

layout(location = 0) in vec2 f_uv;

layout(binding = 2) uniform sampler2D tex;

void main() {
    out_color = texture(tex, f_uv);
}