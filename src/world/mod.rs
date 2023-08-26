pub mod chunk;
pub mod block;

use std::collections::HashMap;
use ash::vk;

use crate::timer::Timer;

use self::{chunk::Chunk, block::{BlockType, Block}};

/// has position of 1, 2, 3 instead of going in intervals of `Chunk::SIZE`
type ChunkPos = glm::I8Vec3;

pub struct World {
    chunks: HashMap<ChunkPos, Chunk>
}

impl World {
    pub fn new(distance: u32) -> World {
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

                    chunk::build_mesh(chunk, [west_chunk, east_chunk, up_chunk, down_chunk, north_chunk, south_chunk]);
                }
            }
        }
        generating_mesh_timer.tick();
        println!("Generated mesh in {}s", generating_mesh_timer.elapsed());
        generating_mesh_timer.reset();

        World {
            chunks
        }
    }

    pub fn draw(&self, camera_buffer_info: vk::DescriptorBufferInfo, atlas_image_info: vk::DescriptorImageInfo) {
        for chunk in self.chunks.values() {
            chunk.draw(camera_buffer_info, atlas_image_info);
        }
    }
}
