#[derive(Debug, Clone, Copy)]
pub struct Vertex(u32);
impl Vertex {
    pub fn new_glm(pos: glm::Vec3, uv: glm::Vec2) -> Vertex {
        // first byte is x, second byte is y, third byte is z
        // fourth byte is uv

        let mut compressed = (pos.x as u32) << 24;
        compressed |= (pos.y as u32) << 16;
        compressed |= (pos.z as u32) << 8;
        compressed |= ((uv.x * 10.0) as u32) << 4;
        compressed |= (uv.y * 10.0) as u32;
        
        Vertex(compressed)
    }

    pub fn get_binding_description() -> [ash::vk::VertexInputBindingDescription; 1] {
        [
            ash::vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<Vertex>() as u32,
                input_rate: ash::vk::VertexInputRate::VERTEX
            }
        ]
    }

    pub fn get_attribute_descriptions() -> [ash::vk::VertexInputAttributeDescription; 1] {
        [
            ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: ash::vk::Format::R32_UINT,
                offset: 0
            }
        ]
    }
}
