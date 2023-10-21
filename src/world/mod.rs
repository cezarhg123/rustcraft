pub mod chunk;
pub mod block;

use std::{collections::HashMap, mem::size_of};
use ash::vk;
use noise::{Perlin, NoiseFn};
use crate::{timer::Timer, engine::{buffer::Buffer, vertex::{Vertex, CompressedVertex}, self}};
use self::{chunk::Chunk, block::{BlockType, Block}};

/// has position of 1, 2, 3 instead of going in intervals of `Chunk::SIZE`
type ChunkPos = glm::I8Vec3;

pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
    world_vertex_buffer: Buffer<CompressedVertex>,
    half_distance: i32
}

impl World {
    pub const VERTICES_PER_BLOCK: u64 = 36;
    pub const MAX_VERTICES_PER_CHUNK_BYTES: u64 = ((((Chunk::SIZE as u64).pow(3) / 2) + Chunk::SIZE as u64) * World::VERTICES_PER_BLOCK) * size_of::<Vertex>() as u64;

    pub fn new(distance: u32) -> World {
        // max vertices per chunk in bytes with some padding
        let vertex_buffer = Buffer::new_empty(World::MAX_VERTICES_PER_CHUNK_BYTES * (distance as u64).pow(3), vk::BufferUsageFlags::VERTEX_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        let mut chunks = HashMap::with_capacity((Chunk::SIZE as usize).pow(3));

        let half_distance = distance as i32 / 2;

        let perlin = Perlin::new(123);

        let mut generating_terrain_timer = Timer::new();
        for x in -half_distance..half_distance {
            for y in -half_distance..half_distance {
                for z in -half_distance..half_distance {
                    chunks.insert(glm::vec3(x as i8, y as i8, z as i8), Chunk::new(glm::vec3(x as i32 * Chunk::SIZE as i32, y as i32 * Chunk::SIZE as i32, z as i32 * Chunk::SIZE as i32), |global_pos| {
                        let perlin_y = perlin.get([global_pos.x as f64 / 200_000_000.0, global_pos.z as f64 / 200_000_000.0]) * 200_000_000.0;
                        let perlin_y = (perlin_y as i32).div_euclid(10);
                        
                        if global_pos.y < perlin_y {
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
                    let x = x as i8;
                    let y = y as i8;
                    let z = z as i8;

                    let chunk = chunks.get(&glm::vec3(x, y, z)).unwrap() as *const Chunk;

                    let west_chunk = chunks.get(&glm::vec3(x - 1, y, z)).map(|c| c as *const Chunk);
                    let east_chunk = chunks.get(&glm::vec3(x + 1, y, z)).map(|c| c as *const Chunk);
                    let up_chunk = chunks.get(&glm::vec3(x, y + 1, z)).map(|c| c as *const Chunk);
                    let down_chunk = chunks.get(&glm::vec3(x, y - 1, z)).map(|c| c as *const Chunk);
                    let north_chunk = chunks.get(&glm::vec3(x, y, z - 1)).map(|c| c as *const Chunk);
                    let south_chunk = chunks.get(&glm::vec3(x, y, z + 1)).map(|c| c as *const Chunk);

                    chunk::build_mesh(chunk, [west_chunk, east_chunk, up_chunk, down_chunk, north_chunk, south_chunk], &vertex_buffer, offset);
                    offset += World::MAX_VERTICES_PER_CHUNK_BYTES;
                }
            }
        }
        generating_mesh_timer.tick();
        println!("Generated mesh in {}s", generating_mesh_timer.elapsed());
        generating_mesh_timer.reset();

        World {
            chunks,
            world_vertex_buffer: vertex_buffer,
            half_distance
        }
    }

    pub fn update_world(&mut self, player_position: glm::Vec3) {
        let player_position = glm::vec3(
            player_position.x,
            player_position.y * -1.0,
            player_position.z
        );
        let player_position = glm::vec3(
            (player_position.x - (player_position.x % Chunk::SIZE as f32)) as i32,
            (player_position.y - (player_position.y % Chunk::SIZE as f32)) as i32,
            (player_position.z - (player_position.z % Chunk::SIZE as f32)) as i32
        );
        println!("Player position: {:?}", player_position);

        let perlin = Perlin::new(123);

        for x in -self.half_distance..self.half_distance {
            for y in -self.half_distance..self.half_distance {
                for z in -self.half_distance..self.half_distance {
                    let chunk = self.chunks.get(&glm::vec3(x as i8, y as i8, z as i8)).unwrap();

                    if chunk.position() != player_position + glm::vec3(x as i32 * Chunk::SIZE as i32, y as i32 * Chunk::SIZE as i32, z as i32 * Chunk::SIZE as i32) {
                        let old_chunk = self.chunks.insert(
                            glm::vec3(x as i8, y as i8, z as i8),
                            Chunk::new(
                                player_position + glm::vec3(x as i32 * Chunk::SIZE as i32, y as i32 * Chunk::SIZE as i32, z as i32 * Chunk::SIZE as i32),
                                |global_pos| {
                                    let perlin_y = perlin.get([global_pos.x as f64 / 200_000_000.0, global_pos.z as f64 / 200_000_000.0]) * 200_000_000.0;
                                    let perlin_y = (perlin_y as i32).div_euclid(10);
                        
                                    if global_pos.y < perlin_y {
                                        Block::new("Grass Block", "grass_block", BlockType::Solid, glm::vec2(0.0, 0.0), glm::vec2(0.1, 0.0), glm::vec2(0.2, 0.0))
                                    } else {
                                        Block::new("Air", "air", BlockType::Air, glm::vec2(0.9, 0.9), glm::vec2(0.9, 0.9), glm::vec2(0.9, 0.9))
                                    }
                                }
                            )
                        ).unwrap();

                        let x = x as i8;
                        let y = y as i8;
                        let z = z as i8;

                        let west_chunk = self.chunks.get(&glm::vec3(x - 1, y, z)).map(|c| c as *const Chunk);
                        let east_chunk = self.chunks.get(&glm::vec3(x + 1, y, z)).map(|c| c as *const Chunk);
                        let up_chunk = self.chunks.get(&glm::vec3(x, y + 1, z)).map(|c| c as *const Chunk);
                        let down_chunk = self.chunks.get(&glm::vec3(x, y - 1, z)).map(|c| c as *const Chunk);
                        let north_chunk = self.chunks.get(&glm::vec3(x, y, z - 1)).map(|c| c as *const Chunk);
                        let south_chunk = self.chunks.get(&glm::vec3(x, y, z + 1)).map(|c| c as *const Chunk);

                        chunk::build_mesh(
                            self.chunks.get(&glm::vec3(x as i8, y as i8, z as i8)).unwrap(),
                            [west_chunk, east_chunk, up_chunk, down_chunk, north_chunk, south_chunk],
                            &self.world_vertex_buffer,
                            old_chunk.buffer_offset(),

                        )
                    }
                }
            }
        }
    }

    pub fn draw(&self, camera_buffer_info: vk::DescriptorBufferInfo, atlas_image_info: vk::DescriptorImageInfo) {
        for chunk in self.chunks.values() {
            chunk.write_descriptor(camera_buffer_info, atlas_image_info);
            let chunk_draw_info = chunk.get_draw_info();
            if let Some(chunk_draw_info) = chunk_draw_info {
                engine::instance::draw(self.world_vertex_buffer.buffer(), chunk_draw_info.0, chunk_draw_info.1, chunk_draw_info.2);
            }
        }
    }
}
