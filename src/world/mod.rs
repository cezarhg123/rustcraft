pub mod chunk;
pub mod block;
pub mod multithread;
pub mod job;

use std::{collections::HashMap, io::Cursor};
use ash::vk;
use crate::{ptr_wrapper::PtrWrapper, vust::texture::Texture};
use self::{block::Block, chunk::Chunk, multithread::Multithread};

pub struct World {
    multithread: Multithread,
    texture_atlas: Texture,
    chunks: HashMap<glm::IVec3, Chunk>,
}

impl World {
    const DISTANCE: i32 = 10;

    pub fn new() -> World {
        let mut chunks = HashMap::new();
        
        for x in -World::DISTANCE..World::DISTANCE {
            for y in -World::DISTANCE..World::DISTANCE {
                for z in -World::DISTANCE..World::DISTANCE {
                    chunks.insert(glm::vec3(x, y, z), Chunk::new(glm::vec3(x, y, z)));
                }
            }
        }


        let mut multithread = Multithread::new(4);
        for x in -World::DISTANCE..World::DISTANCE {
            for y in -World::DISTANCE..World::DISTANCE {
                for z in -World::DISTANCE..World::DISTANCE {

                    multithread.add_job(job::Job::GenerateTerrain {
                        chunk: PtrWrapper::new(chunks.get(&glm::vec3(x, y, z)).unwrap()),
                        gen_func: |position| {
                            if position.y < 0 {
                                1
                            } else {
                                0
                            }
                        }
                    });

                    // std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }

        multithread.wait_for_idle();

        for x in -World::DISTANCE..World::DISTANCE {
            for y in -World::DISTANCE..World::DISTANCE {
                for z in -World::DISTANCE..World::DISTANCE {
                    multithread.add_job(job::Job::GenerateMesh {
                        chunk: PtrWrapper::new(chunks.get(&glm::vec3(x, y, z)).unwrap()),
                        neighbors: [
                            chunks.get(&glm::vec3(x - 1, y, z)).map(|c| PtrWrapper::new(c)),
                            chunks.get(&glm::vec3(x + 1, y, z)).map(|c| PtrWrapper::new(c)),
                            chunks.get(&glm::vec3(x, y - 1, z)).map(|c| PtrWrapper::new(c)),
                            chunks.get(&glm::vec3(x, y + 1, z)).map(|c| PtrWrapper::new(c)),
                            chunks.get(&glm::vec3(x, y, z - 1)).map(|c| PtrWrapper::new(c)),
                            chunks.get(&glm::vec3(x, y, z + 1)).map(|c| PtrWrapper::new(c))
                        ]
                    });
                }
            }
        }

        World {
            texture_atlas: Texture::new(
                image::load(
                    Cursor::new(include_bytes!("../../textures/atlas.png")),
                    image::ImageFormat::Png
                ).unwrap()
            ),
            chunks,
            multithread
        }
    }

    pub fn draw(&self, camera_buffer_info: vk::DescriptorBufferInfo) {
        for chunk in self.chunks.values() {
            chunk.draw(camera_buffer_info, self.texture_atlas.descirptor_image_info());
        }
    }
}

pub const ALL_BLOCKS: [Block; 2] = [
    Block::Air,
    Block::new_solid("Grass Block", "grass_block", glm::Vec2::new(0.0, 0.0)),
];

pub fn get_global_block(name: &str) -> Option<&'static Block> {
    ALL_BLOCKS
        .iter()
        .find(
            |block| block.get_dev_name() == name
        )
}