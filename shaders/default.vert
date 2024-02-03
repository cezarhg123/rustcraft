#version 450

layout(location = 0) in uint compressedVertex;

layout(location = 0) out vec2 UV;

layout(binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
};

layout(binding = 2) uniform Model {
    mat4 model;
};

#define FIRST_BYTE 4278190080
#define SECOND_BYTE 16711680
#define THIRD_BYTE 65280
#define FOURTH_BYTE 255

void main() {
    vec3 vPos;

    vPos.x = float((compressedVertex & FIRST_BYTE) >> 24);
    vPos.y = float((compressedVertex & SECOND_BYTE) >> 16);
    vPos.z = float((compressedVertex & THIRD_BYTE) >> 8);

    // UV is stored in a byte
    // x = 1111 0000
    // y = 0000 1111
    UV.x = float((compressedVertex & FOURTH_BYTE) >> 4) / 10.0;
    UV.y = float(compressedVertex & 15) / 10.0;

    gl_Position = projection * view * model * vec4(vPos.x, vPos.y * -1.0, vPos.z * -1.0, 1.0);
}