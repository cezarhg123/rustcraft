use ash::vk;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: [f32; 3],
    uv: [f32; 2]
}

impl Vertex {
    pub fn new(position: glm::Vec3, uv: glm::Vec2) -> Vertex {
        Vertex {
            position: [position.x, position.y, position.z],
            uv: [uv.x, uv.y]
        }
    }

    pub fn position(&self) -> glm::Vec3 {
        glm::vec3(self.position[0], self.position[1], self.position[2])
    }

    pub fn uv(&self) -> glm::Vec2 {
        glm::vec2(self.uv[0], self.uv[1])
    }

    pub fn get_binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [
            vk::VertexInputBindingDescription::builder()
                .binding(0)
                .stride(std::mem::size_of::<Vertex>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX)
                .build()
        ]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(12)
                .build()
        ]
    }
}
