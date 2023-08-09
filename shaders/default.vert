#version 450

layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec2 v_uv;

layout(location = 0) out vec2 v_uv_out;

layout(binding = 0) uniform Camera {
    mat4 proj;
    mat4 view;
};

layout(binding = 2) uniform Model {
    mat4 model;
};

void main() {
    gl_Position = proj * view * model * vec4(v_pos.x, v_pos.y * -1, v_pos.z, 1.0);
    v_uv_out = v_uv;
}
