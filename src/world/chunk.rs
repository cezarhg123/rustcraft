use std::collections::HashMap;
use ash::vk;
use crate::{ptr_wrapper::PtrWrapper, vust::{self, buffer::Buffer, instance::DrawCall, vertex::Vertex}};
use super::{block::{Block, BlockID}, ALL_BLOCKS};

type VertexCount = usize;

/// global position of chunk = chunk position * chunk size
#[derive(Debug)]
pub struct Chunk {
    position: glm::IVec3,
    blocks: HashMap<glm::I8Vec3, BlockID>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    model_uniform: Buffer,
    chunk_mesh: Option<(Buffer, VertexCount)>,
    terrain_generated: bool
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
            model_uniform: Buffer::new(&[glm::Mat4::new_translation(&glm::vec3(position.x as f32 * Chunk::SIZE as f32, -position.y as f32 * Chunk::SIZE as f32, position.z as f32 * (Chunk::SIZE as f32 - 1.0)))], vk::BufferUsageFlags::UNIFORM_BUFFER, gpu_allocator::MemoryLocation::CpuToGpu),
            chunk_mesh: None,
            terrain_generated: false
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
                    let position = (self.position * Chunk::SIZE as i32) + glm::vec3(x as i32, y as i32, -z as i32);
                    self.blocks.insert(
                        glm::vec3(x, y, z),
                        gen_func(position)
                    );
                }
            }
        }

        self.terrain_generated = true;
    }

    pub fn generate_mesh(&mut self, neighbours: &[Option<PtrWrapper<Chunk>>; 6]) {
        while !self.terrain_generated {} // wait for terrain to be generated

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

                    // if you see '-z', its because for the chunk, +z = forward, but for renderer +z = backward

                    // west face
                    generate_face(
                        glm::vec3(x as f32, y as f32, z as f32),
                        &current_block,
                        self.blocks.get(&glm::vec3(x - 1, y, z)),
                        &mut vertices,
                        neighbours[0].as_ref(),
                        glm::vec3(Chunk::SIZE - 1, y, z),
                        west_face_vertices
                    );

                    // east face
                    generate_face(
                        glm::vec3(x as f32, y as f32, z as f32),
                        &current_block,
                        self.blocks.get(&glm::vec3(x + 1, y, z)),
                        &mut vertices,
                        neighbours[1].as_ref(),
                        glm::vec3(0, y, z),
                        east_face_vertices
                    );

                    // down face
                    generate_face(
                        glm::vec3(x as f32, y as f32, z as f32),
                        &current_block,
                        self.blocks.get(&glm::vec3(x, y - 1, z)),
                        &mut vertices,
                        neighbours[2].as_ref(),
                        glm::vec3(x, Chunk::SIZE - 1, z),
                        down_face_vertices
                    );

                    // up face
                    generate_face(
                        glm::vec3(x as f32, y as f32, z as f32),
                        &current_block,
                        self.blocks.get(&glm::vec3(x, y + 1, z)),
                        &mut vertices,
                        neighbours[3].as_ref(),
                        glm::vec3(x, 0, z),
                        up_face_vertices
                    );

                    // north face
                    generate_face(
                        glm::vec3(x as f32, y as f32, z as f32),
                        &current_block,
                        self.blocks.get(&glm::vec3(x, y, z + 1)),
                        &mut vertices,
                        neighbours[4].as_ref(),
                        glm::vec3(x, y, 0),
                        south_face_vertices
                    );

                    // south face
                    generate_face(
                        glm::vec3(x as f32, y as f32, z as f32),
                        &current_block,
                        self.blocks.get(&glm::vec3(x, y, z - 1)),
                        &mut vertices,
                        neighbours[5].as_ref(),
                        glm::vec3(x, y, Chunk::SIZE - 1),
                        north_face_vertices
                    );

                }
            }
        }
        
        if !vertices.is_empty() {
            self.chunk_mesh = Some((Buffer::new(vertices.as_slice(), vk::BufferUsageFlags::VERTEX_BUFFER, gpu_allocator::MemoryLocation::CpuToGpu), vertices.len()));
        }
    }
}

fn generate_face(
    block_pos: glm::Vec3,
    current_block: &Block,
    neighbour_block: Option<&BlockID>,
    vertices: &mut Vec<Vertex>,
    neighbour_chunk: Option<&PtrWrapper<Chunk>>,
    neighbour_block_pos: glm::I8Vec3,
    face_vertices_func: fn(glm::Vec3, glm::Vec2) -> Vec<Vertex>
) {
    if let Some(west_block) = neighbour_block {
        if *west_block == 0 {
            vertices.append(&mut face_vertices_func(block_pos, current_block.get_uv()));
        }
    } else {
        if let Some(neighbour) = neighbour_chunk {
            if neighbour.as_ref().blocks[&neighbour_block_pos] == 0 {
                vertices.append(&mut face_vertices_func(block_pos, current_block.get_uv()));
            }
        } else {
            vertices.append(&mut face_vertices_func(block_pos, current_block.get_uv()));
        }
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

fn east_face_vertices(pos: glm::Vec3, uv: glm::Vec2) -> Vec<Vertex> {
    vec![
        Vertex::new_glm(pos + glm::vec3(1.0, 0.0, 0.0), uv),
        Vertex::new_glm(pos + glm::vec3(1.0, 1.0, -1.0), uv + glm::vec2(0.1, 0.1)),
        Vertex::new_glm(pos + glm::vec3(1.0, 1.0, 0.0), uv + glm::vec2(0.0, 0.1)),

        Vertex::new_glm(pos + glm::vec3(1.0, 0.0, 0.0), uv),
        Vertex::new_glm(pos + glm::vec3(1.0, 0.0, -1.0), uv + glm::vec2(0.1, 0.0)),
        Vertex::new_glm(pos + glm::vec3(1.0, 1.0, -1.0), uv + glm::vec2(0.1, 0.1))
    ]
}

fn down_face_vertices(pos: glm::Vec3, uv: glm::Vec2) -> Vec<Vertex> {
    vec![
        Vertex::new_glm(pos + glm::vec3(0.0, 0.0, -1.0), uv),
        Vertex::new_glm(pos + glm::vec3(1.0, 0.0, 0.0), uv + glm::vec2(0.1, 0.1)),
        Vertex::new_glm(pos, uv + glm::vec2(0.0, 0.1)),

        Vertex::new_glm(pos + glm::vec3(0.0, 0.0, -1.0), uv),
        Vertex::new_glm(pos + glm::vec3(1.0, 0.0, -1.0), uv + glm::vec2(0.1, 0.0)),
        Vertex::new_glm(pos + glm::vec3(1.0, 0.0, 0.0), uv + glm::vec2(0.1, 0.1))
    ]
}

fn up_face_vertices(pos: glm::Vec3, uv: glm::Vec2) -> Vec<Vertex> {
    vec![
        Vertex::new_glm(pos + glm::vec3(0.0, 1.0, 0.0), uv),
        Vertex::new_glm(pos + glm::vec3(1.0, 1.0, -1.0), uv + glm::vec2(0.1, 0.1)),
        Vertex::new_glm(pos + glm::vec3(0.0, 1.0, -1.0), uv + glm::vec2(0.0, 0.1)),

        Vertex::new_glm(pos + glm::vec3(0.0, 1.0, 0.0), uv),
        Vertex::new_glm(pos + glm::vec3(1.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.0)),
        Vertex::new_glm(pos + glm::vec3(1.0, 1.0, -1.0), uv + glm::vec2(0.1, 0.1))
    ]
}

/// north = -z
fn north_face_vertices(pos: glm::Vec3, uv: glm::Vec2) -> Vec<Vertex> {
    vec![
        Vertex::new_glm(pos + glm::vec3(1.0, 0.0, -1.0), uv),
        Vertex::new_glm(pos + glm::vec3(0.0, 1.0, -1.0), uv + glm::vec2(0.1, 0.1)),
        Vertex::new_glm(pos + glm::vec3(1.0, 1.0, -1.0), uv + glm::vec2(0.0, 0.1)),

        Vertex::new_glm(pos + glm::vec3(1.0, 0.0, -1.0), uv),
        Vertex::new_glm(pos + glm::vec3(0.0, 0.0, -1.0), uv + glm::vec2(0.1, 0.0)),
        Vertex::new_glm(pos + glm::vec3(0.0, 1.0, -1.0), uv + glm::vec2(0.1, 0.1))
    ]
}

/// south = +z
fn south_face_vertices(pos: glm::Vec3, uv: glm::Vec2) -> Vec<Vertex> {
    vec![
        Vertex::new_glm(pos, uv),
        Vertex::new_glm(pos + glm::vec3(1.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.1)),
        Vertex::new_glm(pos + glm::vec3(0.0, 1.0, 0.0), uv + glm::vec2(0.0, 0.1)),

        Vertex::new_glm(pos, uv),
        Vertex::new_glm(pos + glm::vec3(1.0, 0.0, 0.0), uv + glm::vec2(0.1, 0.0)),
        Vertex::new_glm(pos + glm::vec3(1.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.1))
    ]
}
