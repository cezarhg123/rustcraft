#version 460

layout(location = 0) in uint compressed_vertex;

layout(binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
} camera;

layout(binding = 1) uniform Chunk {
    mat4 model;
} chunk;

layout(location = 0) out vec2 f_uv;

void main() {
    vec3 v_pos = vec3(
        float((compressed_vertex & 0xFF000000) >> 24),
        float((compressed_vertex & 0x00FF0000) >> 16),
        float((compressed_vertex & 0x0000FF00) >> 8)
    );

    uint uv_byte = compressed_vertex & 0x000000FF;
    f_uv = vec2(
        // max 16 x 16 textures on atlas
        float((uv_byte & 0x000000F0) >> 4) / 16.0,
        float(uv_byte & 0x0000000F) / 16.0
    );

    gl_Position = camera.projection * camera.view * chunk.model * vec4(v_pos, 1.0);
}
