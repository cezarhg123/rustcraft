pub mod chunk;
pub mod block;

use std::{collections::HashMap, mem::size_of};
use ash::vk;
use crate::{timer::Timer, engine::{buffer::Buffer, vertex::Vertex, self}};
use self::{chunk::Chunk, block::{BlockType, Block}};

/// has position of 1, 2, 3 instead of going in intervals of `Chunk::SIZE`
type ChunkPos = glm::I8Vec3;

pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
    world_vertex_buffer: Buffer<Vertex>
}

impl World {
    pub fn new(distance: u32) -> World {
        let vertices_per_block = 36;
        // max vertices per chunk in bytes with some padding
        let max_vertices_per_chunk_bytes = ((((Chunk::SIZE as u64).pow(3) / 2) + Chunk::SIZE as u64) * vertices_per_block) * size_of::<Vertex>() as u64;
        let vertex_buffer = Buffer::new_empty(max_vertices_per_chunk_bytes * (distance as u64).pow(3), vk::BufferUsageFlags::VERTEX_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        let mut chunks = HashMap::with_capacity((Chunk::SIZE as usize).pow(3));

        let half_distance = distance as i32 / 2;

        let mut generating_terrain_timer = Timer::new();
        for x in -half_distance..half_distance {
            for y in -half_distance..half_distance {
                for z in -half_distance..half_distance {
                    let chunk_pos = ChunkPos::new(x as i8, y as i8, z as i8);

                    chunks.insert(chunk_pos, Chunk::new(glm::vec3(x as i32 * Chunk::SIZE as i32, y as i32 * Chunk::SIZE as i32, z as i32 * Chunk::SIZE as i32), |global_pos| {
                        if global_pos.y < 2 {
                            Block::new("Grass Block", "grass_block", BlockType::Solid, glm::vec2(0.0, 0.0), glm::vec2(0.1, 0.0), glm::vec2(0.2, 0.0))
                        } else {
                            Block::new("Air", "air", BlockType::Air, glm::vec2(0.9, 0.9), glm::vec2(0.9, 0.9), glm::vec2(0.9, 0.9))
                        }
                    }));
                }
            }
        }
        generating_terrain_timer.tick();
        println!("Generated terrain in {}s", generating_terrain_timer.elapsed());
        generating_terrain_timer.reset();

        let mut offset = 0;

        let mut generating_mesh_timer = Timer::new();
        for x in -half_distance..half_distance {
            for y in -half_distance..half_distance {
                for z in -half_distance..half_distance {
                    let chunk = chunks.get(&glm::vec3(x as i8, y as i8, z as i8)).unwrap() as *const Chunk;

                    let west_chunk = chunks.get(&glm::vec3(x as i8 - 1, y as i8, z as i8)).map(|c| c as *const Chunk);
                    let east_chunk = chunks.get(&glm::vec3(x as i8 + 1, y as i8, z as i8)).map(|c| c as *const Chunk);
                    let up_chunk = chunks.get(&glm::vec3(x as i8, y as i8 + 1, z as i8)).map(|c| c as *const Chunk);
                    let down_chunk = chunks.get(&glm::vec3(x as i8, y as i8 - 1, z as i8)).map(|c| c as *const Chunk);
                    let north_chunk = chunks.get(&glm::vec3(x as i8, y as i8, z as i8 - 1)).map(|c| c as *const Chunk);
                    let south_chunk = chunks.get(&glm::vec3(x as i8, y as i8, z as i8 + 1)).map(|c| c as *const Chunk);

                    chunk::build_mesh(chunk, [west_chunk, east_chunk, up_chunk, down_chunk, north_chunk, south_chunk], &vertex_buffer, offset, max_vertices_per_chunk_bytes);
                    offset += max_vertices_per_chunk_bytes;
                }
            }
        }
        generating_mesh_timer.tick();
        println!("Generated mesh in {}s", generating_mesh_timer.elapsed());
        generating_mesh_timer.reset();

        World {
            chunks,
            world_vertex_buffer: vertex_buffer
        }
    }

    pub fn draw(&self, camera_buffer_info: vk::DescriptorBufferInfo, atlas_image_info: vk::DescriptorImageInfo) {
        for chunk in self.chunks.values() {
            chunk.write_descriptor(camera_buffer_info, atlas_image_info);
            let chunk_draw_info = chunk.get_draw_info();
            if let Some(chunk_draw_info) = chunk_draw_info {
                engine::instance::draw(self.world_vertex_buffer.buffer(), chunk_draw_info.0, chunk_draw_info.2, chunk_draw_info.3);
            }
        }
    }
}
