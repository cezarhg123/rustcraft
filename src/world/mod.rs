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
    pub fn new() -> World {
        let mut chunks = HashMap::new();
        chunks.insert(glm::zero(), Chunk::new(glm::zero()));

        let mut multithread = Multithread::new(4);
        multithread.add_job(job::Job::GenerateTerrain {
            chunk: PtrWrapper::new(chunks.get(&glm::zero()).unwrap()),
            gen_func: |position| {
                if position.y < 4 {
                    1
                } else {
                    0
                }
            }
        });
        
        multithread.add_job(job::Job::GenerateMesh {
            chunk: PtrWrapper::new(chunks.get(&glm::zero()).unwrap()),
            neighbors: [None, None, None, None, None, None]
        });

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