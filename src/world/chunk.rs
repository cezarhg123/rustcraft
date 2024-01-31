use std::collections::HashMap;
use ash::vk;
use crate::{ptr_wrapper::PtrWrapper, vust::{self, buffer::Buffer, instance::DrawCall, vertex::Vertex}};
use super::{block::BlockID, ALL_BLOCKS};

type VertexCount = usize;

pub struct Chunk {
    position: glm::IVec3,
    blocks: HashMap<glm::I8Vec3, BlockID>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    model_uniform: Buffer,
    chunk_mesh: Option<(Buffer, VertexCount)>
}

impl Chunk {
    pub const SIZE: i8 = 20;

    pub fn new(position: glm::IVec3) -> Chunk {
        let descriptor_pool = vust::instance::create_descriptor_pool();

        Chunk {
            position,
            blocks: HashMap::new(),
            descriptor_pool,
            descriptor_set: unsafe {
                vust::instance::get_device().allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::builder()
                        .descriptor_pool(descriptor_pool)
                        .set_layouts(&[*vust::instance::get_descriptor_set_layout()])
                        .build()
                ).unwrap()[0]
            },
            model_uniform: Buffer::new(&[glm::Mat4::new_scaling(1.0)], vk::BufferUsageFlags::UNIFORM_BUFFER, gpu_allocator::MemoryLocation::CpuToGpu),
            chunk_mesh: None
        }
    }

    pub fn draw(&self, camera_buffer_info: vk::DescriptorBufferInfo, texture_atlas_info: vk::DescriptorImageInfo) {
        if let Some((mesh, vertex_count)) = &self.chunk_mesh {
            unsafe {
                vust::instance::get_device().update_descriptor_sets(
                    &[
                        vk::WriteDescriptorSet::builder()
                            .dst_set(self.descriptor_set)
                            .dst_binding(0)
                            .dst_array_element(0)
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                            .buffer_info(&[camera_buffer_info])
                            .build(),
                        vk::WriteDescriptorSet::builder()
                            .dst_set(self.descriptor_set)
                            .dst_binding(1)
                            .dst_array_element(0)
                            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .image_info(&[texture_atlas_info])
                            .build(),
                        vk::WriteDescriptorSet::builder()
                            .dst_set(self.descriptor_set)
                            .dst_binding(2)
                            .dst_array_element(0)
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                            .buffer_info(&[
                                vk::DescriptorBufferInfo::builder()
                                    .buffer(self.model_uniform.buffer())
                                    .offset(0)
                                    .range(std::mem::size_of::<glm::Mat4>() as u64)
                                    .build()
                            ])
                            .build()

                    ],
                    &[]
                );
            }

            vust::instance::draw(
                DrawCall {
                    buffer: mesh.buffer(),
                    descriptor_set: self.descriptor_set,
                    vertex_count: *vertex_count as u32
                }
            );
        }
    }

    /// gen_func takes global position of block
    pub fn generate_terrain<T: Fn(glm::IVec3) -> BlockID>(&mut self, gen_func: T) {
        for x in 0..Chunk::SIZE {
            for y in 0..Chunk::SIZE {
                for z in 0..Chunk::SIZE {
                    let position = self.position + glm::vec3(x as i32, y as i32, -z as i32);
                    self.blocks.insert(
                        glm::vec3(x, y, z),
                        gen_func(position)
                    );
                }
            }
        }
    }

    pub fn generate_mesh(&mut self, neighbours: &[Option<PtrWrapper<Chunk>>; 6]) {
        let mut vertices = Vec::new();
        
        for x in 0..Chunk::SIZE {
            for y in 0..Chunk::SIZE {
                for z in 0..Chunk::SIZE {
                    // 0 is air
                    // why do a shit ton of checks if its air
                    if self.blocks[&glm::vec3(x, y, z)] == 0 {
                        continue;
                    }

                    let current_block = ALL_BLOCKS[self.blocks[&glm::vec3(x, y, z)]];

                    if let Some(west_block) = self.blocks.get(&glm::vec3(x - 1, y, z)) {
                        if *west_block == 0 {
                            vertices.append(&mut west_face_vertices(glm::vec3(x as f32, y as f32, -z as f32), current_block.get_uv()));
                        }
                    } else {
                        if let Some(neighbour) = neighbours[0].as_ref() {
                            if neighbour.as_ref().blocks[&glm::vec3(Chunk::SIZE - 1, y, z)] == 0 {
                                vertices.append(&mut west_face_vertices(glm::vec3(x as f32, y as f32, -z as f32), current_block.get_uv()));
                            }
                        } else {
                            vertices.append(&mut west_face_vertices(glm::vec3(x as f32, y as f32, -z as f32), current_block.get_uv()));
                        }
                    }
                }
            }
        }

        self.chunk_mesh = Some((Buffer::new(&vertices, vk::BufferUsageFlags::VERTEX_BUFFER, gpu_allocator::MemoryLocation::CpuToGpu), vertices.len()));
    }
}

fn west_face_vertices(pos: glm::Vec3, uv: glm::Vec2) -> Vec<Vertex> {
    vec![
        Vertex::new_glm(pos + glm::vec3(0.0, 0.0, -1.0), uv),
        Vertex::new_glm(pos + glm::vec3(0.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.1)),
        Vertex::new_glm(pos + glm::vec3(0.0, 1.0, -1.0), uv + glm::vec2(0.0, 0.1)),

        Vertex::new_glm(pos + glm::vec3(0.0, 0.0, -1.0), uv),
        Vertex::new_glm(pos, uv + glm::vec2(0.1, 0.0)),
        Vertex::new_glm(pos + glm::vec3(0.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.1))
    ]
}
