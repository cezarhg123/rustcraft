#version 450

layout(location = 0) in vec3 vPos;

layout(binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
};

void main() {
    gl_Position = projection * view * vec4(vPos, 1.0);
}