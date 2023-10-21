#version 450

layout(location = 0) in uint compressed_vertex;

layout(location = 0) out vec2 v_uv_out;

layout(binding = 0) uniform Camera {
    mat4 proj;
    mat4 view;
};

layout(binding = 2) uniform Model {
    mat4 model;
};

void main() {
    vec3 v_pos;
    // 11111111_00000000_00000000_00000000
    v_pos.x = float((compressed_vertex & 4278190080) >> 24);
    // 00000000_11111111_00000000_00000000
    v_pos.y = float((compressed_vertex & 16711680) >> 16);
    // 00000000_00000000_11111111_00000000
    v_pos.z = float((compressed_vertex & 65280) >> 8);

    // 00000000_00000000_00000000_11111111
    uint compressed_uv = compressed_vertex & 255;
    v_uv_out.x = float((compressed_uv & 240) >> 4) / 10.0;
    v_uv_out.y = float(compressed_uv & 15) / 10.0;

    gl_Position = proj * view * model * vec4(v_pos.x, v_pos.y * -1, v_pos.z, 1.0);
}
