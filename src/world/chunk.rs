use std::{cell::{Ref, RefCell, RefMut}, sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use noise::{NoiseFn, OpenSimplex, Perlin};
use vust::{buffer::Buffer, descriptor::Descriptor, pipeline::GraphicsPipeline, write_descriptor_info::WriteDescriptorInfo, Vust};
use crate::vertex::Vertex;
use super::{block::BlockID, blocks::Blocks};

pub type BlockArray = [[[BlockID; Chunk::SIZE]; Chunk::SIZE]; Chunk::SIZE];

pub struct Chunk {
    /// position of chunk, actual global position of it is chunk_pos * Chunk::SIZE
    chunk_pos: glm::IVec3,
    blocks: BlockArray,
    vertex_count: usize,
    descriptor: Descriptor,
    vertex_buffer: Option<Buffer>,
    uniform_buffer: Buffer
}

impl Chunk {
    pub const SIZE: usize = 32;

    pub fn new(chunk_pos: glm::IVec3, pipeline: &GraphicsPipeline, vust: &Vust) -> Chunk {
        let uniform_buffer = {
            let name = format!("chunk {} {} {} uniform buffer", chunk_pos.x, chunk_pos.y, chunk_pos.z);

            Buffer::builder()
                .with_name(&name)
                .with_data(&[
                    glm::Mat4::new_translation(&(
                        glm::vec3(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32) * Chunk::SIZE as f32
                    ))
                ])
                .with_usage(vust::buffer::BufferUsageFlags::UNIFORM_BUFFER)
                .with_memory_location(vust::buffer::MemoryPropertyFlags::HOST_VISIBLE | vust::buffer::MemoryPropertyFlags::HOST_COHERENT)
                .build(vust, true)
        };
        
        Chunk {
            chunk_pos,
            blocks: [[[Blocks::AIR.block_id(); Chunk::SIZE]; Chunk::SIZE]; Chunk::SIZE],
            vertex_count: 0,
            descriptor: pipeline.create_descriptor(vust).unwrap(),
            vertex_buffer: None,
            uniform_buffer
        }
    }

    pub fn get_blocks(&self) -> &BlockArray {
        &self.blocks
    }

    pub fn get_vertex_buffer(&self) -> Option<&Buffer> {
        self.vertex_buffer.as_ref()
    }

    pub fn get_vertex_count(&self) -> usize {
        self.vertex_count
    }

    pub fn gen_terrain(&mut self, noise: &Perlin) {
        for x in 0..Chunk::SIZE {
            for z in 0..Chunk::SIZE {
                let mut global_block_pos = glm::vec3(x as i32, 0, z as i32) + self.chunk_pos * Chunk::SIZE as i32;

                let grass_level = 10.0 + noise.get([global_block_pos.x as f64 / 1_000_000.0, global_block_pos.z as f64 / 1_000_000.0]) * 1_000_000.0;
                let grass_level = grass_level.div_euclid(10.0) as i32;

                for y in 0..Chunk::SIZE {
                    global_block_pos.y = y as i32 + self.chunk_pos.y * Chunk::SIZE as i32;
                    if global_block_pos.y == grass_level {
                        self.blocks[x][y][z] = Blocks::GRASS_BLOCK.block_id();
                    } else if global_block_pos.y < grass_level {
                        self.blocks[x][y][z] = Blocks::DIRT.block_id();
                    }
                }
            }
        }
    }

    /// neighbour blocks: [-z, +z, -y, +y, -x, +x]
    pub fn gen_mesh(&mut self, vust: &Vust, neighbour_blocks: [Option<Ref<Chunk>>; 6]) {
        let mut vertices = Vec::new();

        for x in 0..Chunk::SIZE {
            for y in 0..Chunk::SIZE {
                for z in 0..Chunk::SIZE {
                    let block_pos = glm::vec3(x as f32, y as f32, z as f32);
                    let current_block = self.blocks[x][y][z];

                    // 0 = air
                    if current_block == 0 {
                        continue;
                    }

                    if z == 0 {
                        if let Some(south_chunk) = &neighbour_blocks[0] {
                            let south_chunk_blocks = south_chunk.get_blocks();

                            if south_chunk_blocks[x][y][Chunk::SIZE - 1] == 0 {
                                vertices.push([
                                    Vertex::new(block_pos, current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), current_block.get_tr_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), current_block.get_tl_uv()),
                                    Vertex::new(block_pos, current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), current_block.get_br_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), current_block.get_tr_uv()),
                                ]);
                            }
                        }
                    } else {
                        let south_block_id = self.blocks[x][y][z - 1];

                        if south_block_id == 0 {
                            vertices.push([
                                Vertex::new(block_pos, current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), current_block.get_tr_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), current_block.get_tl_uv()),
                                Vertex::new(block_pos, current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), current_block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), current_block.get_tr_uv()),
                            ]);
                        }
                    }

                    if z == Chunk::SIZE - 1 {
                        if let Some(north_chunk) = &neighbour_blocks[1] {
                            let north_chunk_blocks = north_chunk.get_blocks();

                            if north_chunk_blocks[x][y][0] == 0 {
                                vertices.push([
                                    Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), current_block.get_tr_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), current_block.get_tl_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), current_block.get_br_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), current_block.get_tr_uv()),
                                ]);
                            }
                        }
                    } else {
                        let north_block_id = self.blocks[x][y][z + 1];

                        if north_block_id == 0 {
                            vertices.push([
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), current_block.get_tr_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), current_block.get_tl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), current_block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), current_block.get_tr_uv()),
                            ]);
                        }
                    }

                    if y == 0 {
                        if let Some(bottom_chunk) = &neighbour_blocks[2] {
                            let bottom_chunk_blocks = bottom_chunk.get_blocks();

                            if bottom_chunk_blocks[x][Chunk::SIZE - 1][z] == 0 {
                                vertices.push([
                                    Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), current_block.get_tr_uv()),
                                    Vertex::new(block_pos, current_block.get_tl_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), current_block.get_br_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), current_block.get_tr_uv()),
                                ]);
                            }
                        }
                    } else {
                        let bottom_block_id = self.blocks[x][y - 1][z];

                        if bottom_block_id == 0 {
                            vertices.push([
                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), current_block.get_tr_uv()),
                                Vertex::new(block_pos, current_block.get_tl_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), current_block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), current_block.get_tr_uv()),
                            ]);
                        }
                    }

                    if y == Chunk::SIZE - 1 {
                        if let Some(above_chunk) = &neighbour_blocks[3] {
                            let above_chunk_blocks = above_chunk.get_blocks();

                            if above_chunk_blocks[x][0][z] == 0 {
                                vertices.push([
                                    Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), current_block.get_tr_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), current_block.get_tl_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), current_block.get_br_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), current_block.get_tr_uv()),
                                ]);
                            }
                        }
                    } else {
                        let top_block_id = self.blocks[x][y + 1][z];

                        if top_block_id == 0 {
                            vertices.push([
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), current_block.get_tr_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), current_block.get_tl_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), current_block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), current_block.get_tr_uv()),
                            ]);
                        }
                    }

                    if x == 0 {
                        if let Some(west_chunk) = &neighbour_blocks[4] {
                            let west_chunk_blocks = west_chunk.get_blocks();

                            if west_chunk_blocks[Chunk::SIZE - 1][y][z] == 0 {
                                vertices.push([
                                    Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), current_block.get_tr_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), current_block.get_tl_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), current_block.get_bl_uv()),
                                    Vertex::new(block_pos, current_block.get_br_uv()),
                                    Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), current_block.get_tr_uv()),
                                ]);
                            }
                        }
                    } else {
                        let west_block_id = self.blocks[x - 1][y][z];

                        if west_block_id == 0 {
                            vertices.push([
                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), current_block.get_tr_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), current_block.get_tl_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), current_block.get_bl_uv()),
                                Vertex::new(block_pos, current_block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), current_block.get_tr_uv()),
                            ]);
                        }
                    }

                    if x == Chunk::SIZE - 1 {
                        if let Some(east_chunk) = &neighbour_blocks[5] {
                            let east_chunk_blocks = east_chunk.get_blocks();

                            if east_chunk_blocks[0][y][z] == 0 {
                                vertices.push([
                                    Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), current_block.get_tr_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), current_block.get_tl_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), current_block.get_bl_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), current_block.get_br_uv()),
                                    Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), current_block.get_tr_uv()),
                                ]);
                            }
                        }
                    } else {
                        let east_block_id = self.blocks[x + 1][y][z];

                        if east_block_id == 0 {
                            vertices.push([
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), current_block.get_tr_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), current_block.get_tl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), current_block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), current_block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), current_block.get_tr_uv()),
                            ]);
                        }
                    }
                }
            }
        }

        if vertices.len() > 0 {
            self.vertex_count = vertices.len() * 6;

            self.vertex_buffer = Some(
                Buffer::builder()
                    .with_data(&vertices)
                    .with_usage(vust::buffer::BufferUsageFlags::VERTEX_BUFFER)
                    .with_memory_location(vust::buffer::MemoryPropertyFlags::HOST_VISIBLE | vust::buffer::MemoryPropertyFlags::HOST_COHERENT)
                    .build(vust, true)
            );
        }
    }

    pub fn draw(&self, vust: &Vust, pipeline: &GraphicsPipeline, camera_buffer_info: WriteDescriptorInfo, atlas_image_info: WriteDescriptorInfo) {
        if let Some(buffer) = self.vertex_buffer.as_ref() {
            vust.update_descriptor_set(&self.descriptor, &[
                camera_buffer_info,
                WriteDescriptorInfo::Buffer { buffer: self.uniform_buffer.handle(), offset: 0, range: size_of::<glm::Mat4>() as u64 },
                atlas_image_info
            ]);
            vust.bind_descriptor_set(pipeline, &self.descriptor);
            vust.bind_vertex_buffer(buffer.handle());
            vust.draw(self.vertex_count as u32);
        }
    }
}
