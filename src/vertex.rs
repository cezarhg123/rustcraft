#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex(u32);

impl Vertex {
    pub fn new(pos: glm::Vec3, uv: glm::Vec2) -> Vertex {
        let mut compressed_vertex = 0;

        compressed_vertex |= (pos.x as u32) << 24;
        compressed_vertex |= (pos.y as u32) << 16;
        compressed_vertex |= (pos.z as u32) << 8;
        compressed_vertex |= ((uv.x * 16.0) as u32) << 4;
        compressed_vertex |= (uv.y * 16.0) as u32;
        
        Vertex(compressed_vertex)
    }

    pub fn get_binding_info() -> [vust::VertexInputBindingDescription; 1] {
        [
            vust::VertexInputBindingDescription::builder()
                .binding(0)
                .input_rate(vust::VertexInputRate::VERTEX)
                .stride(size_of::<Vertex>() as u32)
                .build()
        ]
    }

    pub fn get_attribute_info() -> [vust::VertexInputAttributeDescription; 1] {
        [
            vust::VertexInputAttributeDescription::builder()
                .location(0)
                .binding(0)
                .format(vust::Format::R32_UINT)
                .offset(0)
                .build(),
        ]
    }
}