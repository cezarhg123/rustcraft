#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32
}
impl Vertex {
    pub fn new_glm(pos: glm::Vec3) -> Vertex {
        Vertex {
            x: pos.x,
            y: pos.y,
            z: pos.z
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

    pub fn get_attribute_descriptions() -> [ash::vk::VertexInputAttributeDescription; 1] {
        [
            ash::vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: ash::vk::Format::R32G32B32_SFLOAT,
                offset: 0
            }
        ]
    }
}
