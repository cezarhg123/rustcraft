#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub uv_x: f32,
    pub uv_y: f32
}
impl Vertex {
    pub fn new_glm(pos: glm::Vec3, uv: glm::Vec2) -> Vertex {
        Vertex {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            uv_x: uv.x,
            uv_y: uv.y
        }
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

    pub fn get_attribute_descriptions() -> [ash::vk::VertexInputAttributeDescription; 2] {
        [
            ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: ash::vk::Format::R32G32B32_SFLOAT,
                offset: 0
            },
            ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: ash::vk::Format::R32G32_SFLOAT,
                offset: 3 * std::mem::size_of::<f32>() as u32
            }
        ]
    }
}
