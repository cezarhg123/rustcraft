use std::{collections::HashMap, mem::size_of};
use ash::vk;
use crate::engine::{buffer::Buffer, vertex::Vertex, self};
use super::block::{Block, BlockType};

type LocalPos = glm::I8Vec3;
pub type GlobalPos = glm::IVec3;
pub type BufferOffset = u64;
pub type Size = u64;
pub type Count = u64;

#[derive(Debug, Clone)]
pub struct Chunk {
    position: glm::IVec3,
    blocks: HashMap<LocalPos, Block>,
    mesh: Option<(BufferOffset, Size, Buffer<glm::Mat4>, vk::DescriptorPool, vk::DescriptorSet, Count)>
}

impl Chunk {
    pub const SIZE: u8 = 8;

    pub fn new(position: glm::IVec3, chunk_gen: fn(GlobalPos) -> Block) -> Chunk {
        let mut blocks = HashMap::new();

        for x in 0..Chunk::SIZE {
            for y in 0..Chunk::SIZE {
                for z in 0..Chunk::SIZE {
                    let global_pos = position + glm::vec3(x as i32, y as i32, z as i32);

                    blocks.insert(glm::vec3(x as i8, y as i8, z as i8), chunk_gen(global_pos));
                }
            }
        }

        Chunk {
            position,
            blocks,
            mesh: None
        }
    }

    pub fn write_descriptor(&self, camera_buffer_info: vk::DescriptorBufferInfo, atlas_image_info: vk::DescriptorImageInfo) {
        if let Some(mesh) = &self.mesh {
            unsafe {
                engine::instance::get_device().update_descriptor_sets(&[
                    vk::WriteDescriptorSet::builder()
                        .dst_set(mesh.4)
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(&[
                            camera_buffer_info
                        ])
                        .build(),
                    vk::WriteDescriptorSet::builder()
                        .dst_set(mesh.4)
                        .dst_binding(1)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(&[
                            atlas_image_info
                        ])
                        .build(),
                    vk::WriteDescriptorSet::builder()
                        .dst_set(mesh.4)
                        .dst_binding(2)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(&[
                            vk::DescriptorBufferInfo::builder()
                                .buffer(mesh.2.buffer())
                                .range(size_of::<glm::Mat4>() as u64)
                                .offset(0)
                                .build()
                        ])
                        .build()
                ], &[]);
            }
        }
    }

    pub fn get_draw_info(&self) -> Option<(BufferOffset, Size, vk::DescriptorSet, Count)> {
        self.mesh.as_ref().map(|mesh| (mesh.0, mesh.1, mesh.4, mesh.5))
    }
}

pub fn build_mesh(chunk: *const Chunk, neighbour_chunks: [Option<*const Chunk>; 6], world_buffer: *const Buffer<Vertex>, offset: u64, size: u64) {
    let chunk = unsafe { &mut *chunk.cast_mut() };

    let neighbour_chunks = unsafe {[
        neighbour_chunks[0].map(|c| &*c),
        neighbour_chunks[1].map(|c| &*c),
        neighbour_chunks[2].map(|c| &*c),
        neighbour_chunks[3].map(|c| &*c),
        neighbour_chunks[4].map(|c| &*c),
        neighbour_chunks[5].map(|c| &*c)
    ]};

    let mut vertices = Vec::new();

    for x in 0..Chunk::SIZE {
        for y in 0..Chunk::SIZE {
            for z in 0..Chunk::SIZE {
                let local_pos = glm::vec3(x as i8, y as i8, z as i8);

                let current_block = chunk.blocks.get(&local_pos).unwrap();
                if current_block.block_type == BlockType::Solid {
                    let mut gen_west_face = || {
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32 + 1.0, z as f32), current_block.side_uv + glm::vec2(0.1, 0.1)));
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32 + 1.0, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.0, 0.1)));

                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32, z as f32), current_block.side_uv + glm::vec2(0.1, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32 + 1.0, z as f32), current_block.side_uv + glm::vec2(0.1, 0.1)));
                    };
                    if let Some(west_block) = chunk.blocks.get(&glm::vec3(x as i8 - 1, y as i8, z as i8)) {
                        if west_block.block_type == BlockType::Air {
                            gen_west_face();
                        }
                    } else {
                        if let Some(west_chunk) = neighbour_chunks[0] {
                            if west_chunk.blocks.get(&glm::vec3(Chunk::SIZE as i8 - 1, y as i8, z as i8)).unwrap().block_type == BlockType::Air {
                                gen_west_face();
                            }
                        } else {
                            gen_west_face();
                        }
                    }

                    let mut gen_east_face = || {
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32, z as f32), current_block.side_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.1, 0.1)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32), current_block.side_uv + glm::vec2(0.0, 0.1)));

                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32, z as f32), current_block.side_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.1, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.1, 0.1)));
                    };
                    if let Some(east_block) = chunk.blocks.get(&glm::vec3(x as i8 + 1, y as i8, z as i8)) {
                        if east_block.block_type == BlockType::Air {
                            gen_east_face();
                        }
                    } else {
                        if let Some(east_chunk) = neighbour_chunks[1] {
                            if east_chunk.blocks.get(&glm::vec3(0, y as i8, z as i8)).unwrap().block_type == BlockType::Air {
                                gen_east_face();
                            }
                        } else {
                            gen_east_face();
                        }
                    }

                    let mut gen_up_face = || {
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32 + 1.0, z as f32), current_block.top_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32 - 1.0), current_block.top_uv + glm::vec2(0.1, 0.1)));
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32 + 1.0, z as f32 - 1.0), current_block.top_uv + glm::vec2(0.0, 0.1)));

                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32 + 1.0, z as f32), current_block.top_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32), current_block.top_uv + glm::vec2(0.1, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32 - 1.0), current_block.top_uv + glm::vec2(0.1, 0.1)));
                    };
                    if let Some(up_block) = chunk.blocks.get(&glm::vec3(x as i8, y as i8 + 1, z as i8)) {
                        if up_block.block_type == BlockType::Air {
                            gen_up_face();
                        }
                    } else {
                        if let Some(up_chunk) = neighbour_chunks[2] {
                            if up_chunk.blocks.get(&glm::vec3(x as i8, 0, z as i8)).unwrap().block_type == BlockType::Air {
                                gen_up_face();
                            }
                        } else {
                            gen_up_face();
                        }
                    }

                    let mut gen_down_face = || {
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32, z as f32 - 1.0), current_block.bottom_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32, z as f32), current_block.bottom_uv + glm::vec2(0.1, 0.1)));
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32, z as f32), current_block.bottom_uv + glm::vec2(0.0, 0.1)));

                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32, z as f32 - 1.0), current_block.bottom_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32, z as f32 - 1.0), current_block.bottom_uv + glm::vec2(0.1, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32, z as f32), current_block.bottom_uv + glm::vec2(0.1, 0.1)));
                    };
                    if let Some(down_block) = chunk.blocks.get(&glm::vec3(x as i8, y as i8 - 1, z as i8)) {
                        if down_block.block_type == BlockType::Air {
                            gen_down_face();
                        }
                    } else {
                        if let Some(down_chunk) = neighbour_chunks[3] {
                            if down_chunk.blocks.get(&glm::vec3(x as i8, Chunk::SIZE as i8 - 1, z as i8)).unwrap().block_type == BlockType::Air {
                                gen_down_face();
                            }
                        } else {
                            gen_down_face();
                        }
                    }

                    let mut gen_north_face = || {
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32 + 1.0, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.1, 0.1)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.0, 0.1)));

                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.1, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32 + 1.0, z as f32 - 1.0), current_block.side_uv + glm::vec2(0.1, 0.1)));
                    };
                    if let Some(north_block) = chunk.blocks.get(&glm::vec3(x as i8, y as i8, z as i8 - 1)) {
                        if north_block.block_type == BlockType::Air {
                            gen_north_face();
                        }
                    } else {
                        if let Some(north_chunk) = neighbour_chunks[4] {
                            if north_chunk.blocks.get(&glm::vec3(x as i8, y as i8, Chunk::SIZE as i8 - 1)).unwrap().block_type == BlockType::Air {
                                gen_north_face();
                            }
                        } else {
                            gen_north_face();
                        }
                    }

                    let mut gen_south_face = || {
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32, z as f32), current_block.side_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32), current_block.side_uv + glm::vec2(0.1, 0.1)));
                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32 + 1.0, z as f32), current_block.side_uv + glm::vec2(0.0, 0.1)));

                        vertices.push(Vertex::new(glm::vec3(x as f32, y as f32, z as f32), current_block.side_uv + glm::vec2(0.0, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32, z as f32), current_block.side_uv + glm::vec2(0.1, 0.0)));
                        vertices.push(Vertex::new(glm::vec3(x as f32 + 1.0, y as f32 + 1.0, z as f32), current_block.side_uv + glm::vec2(0.1, 0.1)));
                    };
                    if let Some(south_block) = chunk.blocks.get(&glm::vec3(x as i8, y as i8, z as i8 + 1)) {
                        if south_block.block_type == BlockType::Air {
                            gen_south_face();
                        }
                    } else {
                        if let Some(down_chunk) = neighbour_chunks[5] {
                            if down_chunk.blocks.get(&glm::vec3(x as i8, y as i8, 0)).unwrap().block_type == BlockType::Air {
                                gen_south_face();
                            }
                        } else {
                            gen_south_face();
                        }
                    }
                }
            }
        }
    }

    let world_buffer = unsafe { &*world_buffer };
    let ptr = world_buffer.map(offset, size);
    unsafe {
        ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
    }
    world_buffer.unmap();
    
    let model = Buffer::new(&[glm::Mat4::new_translation(&glm::vec3(chunk.position.x as f32, -chunk.position.y as f32, chunk.position.z as f32))], vk::BufferUsageFlags::UNIFORM_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT).unwrap();
    
    if vertices.len() > 0 {    
        let descriptor_pool = engine::instance::create_descriptor_pool();
        let descriptor_set = unsafe {
            engine::instance::get_device().allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&[engine::instance::get_descriptor_set_layout()])
                .build()
            ).unwrap()[0]
        };
        
        chunk.mesh = Some((offset, size, model, descriptor_pool, descriptor_set, vertices.len() as u64));
    }
}
