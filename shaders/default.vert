#version 450

layout(location = 0) in vec3 vPos;
layout(location = 1) in vec2 vUV;

layout(location = 0) out vec2 UV;

layout(binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
};

layout(binding = 2) uniform Model {
    mat4 model;
};

void main() {
    UV = vUV;
    gl_Position = projection * view * model * vec4(vPos.x, vPos.y * -1.0, vPos.z, 1.0);
}