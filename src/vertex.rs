#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pos_x: f32, 
    pos_y: f32,
    pos_z: f32,

    uv_x: f32,
    uv_y: f32
}

impl Vertex {
    pub fn new(pos: glm::Vec3, uv: glm::Vec2) -> Vertex {
        Vertex {
            pos_x: pos.x,
            pos_y: pos.y,
            pos_z: pos.z,
            uv_x: uv.x,
            uv_y: uv.y
        }
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

    pub fn get_attribute_info() -> [vust::VertexInputAttributeDescription; 2] {
        [
            vust::VertexInputAttributeDescription::builder()
                .location(0)
                .binding(0)
                .format(vust::Format::R32G32B32_SFLOAT)
                .offset(0)
                .build(),
            vust::VertexInputAttributeDescription::builder()
                .location(1)
                .binding(0)
                .format(vust::Format::R32G32_SFLOAT)
                .offset(size_of::<f32>() as u32 * 3)
                .build()
        ]
    }
}