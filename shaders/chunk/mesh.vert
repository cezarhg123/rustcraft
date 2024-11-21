#version 460

layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec2 v_uv;

layout(binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
} camera;

layout(binding = 1) uniform Chunk {
    mat4 model;
} chunk;

layout(location = 0) out vec2 f_uv;

void main() {
    f_uv = v_uv;
    gl_Position = camera.projection * camera.view * chunk.model * vec4(v_pos, 1.0);
}
