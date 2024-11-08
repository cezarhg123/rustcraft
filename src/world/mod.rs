pub mod chunk;
pub mod block;
pub mod blocks;

use std::{collections::HashMap, io::Cursor, sync::{mpsc, Arc, RwLock}, thread};
use block::BlockID;
use blocks::Blocks;
use chunk::Chunk;
use image::GenericImageView;
use noise::{NoiseFn, OpenSimplex, Perlin, Simplex, Worley};
use vust::{pipeline::{DescriptorSetBinding, DescriptorSetLayout, GraphicsPipeline, GraphicsPipelineCreateInfo}, texture::Texture, write_descriptor_info::WriteDescriptorInfo};
use crate::{thread_pool::ThreadPool, vertex::Vertex, WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct World {
    // not the best performance cuz each element could be in random memory locations but good enough
    chunks: HashMap<glm::IVec3, Chunk>,
    chunk_pipeline: GraphicsPipeline,
    draw_distance: i8,
    atlas: Texture,
    thread_pool: ThreadPool
}

impl World {
    // i doubt someone will travel 2 billion blocks
    // 1 billion blocks each direction
    pub const MAX_BLOCKS: usize = 2_000_000_000;

    pub fn new<'a>(draw_distance: i8, vust: Arc<RwLock<vust::Vust>>) -> World {
        let chunk_pipeline = GraphicsPipeline::new(
            &*vust.read().unwrap(),
            GraphicsPipelineCreateInfo {
                name: "pipeline".to_string(),
                vertex_bin: std::fs::read("shaders/chunk.vert.spv").unwrap(),
                fragment_bin: std::fs::read("shaders/chunk.frag.spv").unwrap(),
                vertex_binding_descriptions: Vertex::get_binding_info().to_vec(),
                vertex_attribute_descriptions: Vertex::get_attribute_info().to_vec(),
                topology: vust::pipeline::PrimitiveTopology::TRIANGLE_LIST,
                viewport: vust::pipeline::Viewport::Static { x: 0.0, y: 0.0, width: WINDOW_WIDTH as f32, height: WINDOW_HEIGHT as f32, min_depth: 0.0, max_depth: 1.0 },
                scissor: vust::pipeline::Scissor::Static { x: 0, y: 0, width: WINDOW_WIDTH, height: WINDOW_HEIGHT },
                polygon_mode: vust::pipeline::PolygonMode::FILL,
                cull_mode: vust::pipeline::CullMode::AntiClockwise,
                descriptor_set_layout: Some(DescriptorSetLayout {
                    bindings: vec![
                        DescriptorSetBinding {
                            descriptor_type: vust::pipeline::DescriptorType::UNIFORM_BUFFER,
                            stage_flags: vust::pipeline::ShaderStageFlags::VERTEX,
                        },
                        DescriptorSetBinding {
                            descriptor_type: vust::pipeline::DescriptorType::UNIFORM_BUFFER,
                            stage_flags: vust::pipeline::ShaderStageFlags::VERTEX
                        },
                        DescriptorSetBinding {
                            descriptor_type: vust::pipeline::DescriptorType::COMBINED_IMAGE_SAMPLER,
                            stage_flags: vust::pipeline::ShaderStageFlags::FRAGMENT
                        }
                    ]
                })
            }
        );

        let atlas_image = image::load(
            Cursor::new(std::fs::read("textures/atlas.png").unwrap()),
            image::ImageFormat::Png
        ).unwrap();

        let atlas_texture = Texture::builder()
            .with_name("texture atlas")
            .with_data(atlas_image.as_bytes())
            .with_dimensions(atlas_image.dimensions())
            .with_format(vust::texture::Format::R8G8B8A8_SRGB)
            .with_filter(vust::texture::Filter::NEAREST)
            .build(&*vust.read().unwrap())
            .unwrap();

        let mut thread_pool = ThreadPool::new(8);

        let mut chunks = HashMap::new();

        for x in -draw_distance..draw_distance {
            for y in -draw_distance..draw_distance {
                for z in -draw_distance..draw_distance {
                    chunks.insert(glm::vec3(x as i32, y as i32, z as i32), Chunk::new(glm::vec3(x as i32, y as i32, z as i32), &chunk_pipeline, &*vust.read().unwrap()));
                }
            }
        }

        for x in -draw_distance..draw_distance {
            for y in -draw_distance..draw_distance {
                for z in -draw_distance..draw_distance {
                    thread_pool.send_task(crate::thread_pool::task::Task::GenTerrain {
                        chunk_pos: glm::vec3(x as i32, y as i32, z as i32),
                        blocks: chunks.get_mut(&glm::vec3(x as i32, y as i32, z as i32)).unwrap().get_blocks(),
                        gen_func: |pos| {
                            let perlin_noise = OpenSimplex::new(51234);
                            let perlin_x = (pos.x.abs() as f64 % 200.0) / 200.0;
                            let perlin_z = (pos.z.abs() as f64 % 200.0) / 200.0;
                            let perlin_y = (perlin_noise.get([perlin_x, perlin_z]) * 100.0) as i32;
                            
                            if pos.y == perlin_y {
                                Blocks::GRASS_BLOCK.block_id()
                            } else {
                                Blocks::AIR.block_id()
                            }
                        }
                    });
                }
            }
        }

        for x in -draw_distance..draw_distance {
            for y in -draw_distance..draw_distance {
                for z in -draw_distance..draw_distance {
                    let chunk = chunks.get(&glm::vec3(x as i32, y as i32, z as i32)).unwrap();
                    thread_pool.send_task(crate::thread_pool::task::Task::GenMesh {
                        chunk_pos: glm::vec3(x as i32, y as i32, z as i32),
                        blocks: chunk.get_blocks(),
                        vertex_buffer: chunk.get_vertex_buffer(),
                        vertex_count: chunk.get_vertex_count(),
                        neighbour_blocks: [
                            chunks.get(&glm::vec3(x as i32, y as i32, z as i32 - 1)).map(|c| c.get_blocks()),
                            chunks.get(&glm::vec3(x as i32, y as i32, z as i32 + 1)).map(|c| c.get_blocks()),
                            chunks.get(&glm::vec3(x as i32, y as i32 - 1, z as i32)).map(|c| c.get_blocks()),
                            chunks.get(&glm::vec3(x as i32, y as i32 + 1, z as i32)).map(|c| c.get_blocks()),
                            chunks.get(&glm::vec3(x as i32 - 1, y as i32, z as i32)).map(|c| c.get_blocks()),
                            chunks.get(&glm::vec3(x as i32 + 1, y as i32, z as i32)).map(|c| c.get_blocks())
                        ],
                        vust: Arc::clone(&vust)
                    });
                }
            }
        }

        World {
            chunks,
            chunk_pipeline,
            draw_distance,
            atlas: atlas_texture,
            thread_pool
        }
    }

    pub fn update_chunks(&mut self, player_pos: glm::Vec3, vust: Arc<RwLock<vust::Vust>>) {
        // snap the player position to closest chunk
        let snapped_player_pos = snap_player_pos_to_chunk(player_pos);

        let mut new_chunks = Vec::new();

        for x in -self.draw_distance..self.draw_distance {
            for y in -self.draw_distance..self.draw_distance {
                for z in -self.draw_distance..self.draw_distance {
                    let chunk_pos = glm::vec3(x as i32, y as i32, z as i32) + snapped_player_pos;
                    if self.chunks.get(&chunk_pos).is_none() {
                        self.chunks.insert(chunk_pos, Chunk::new(chunk_pos, &self.chunk_pipeline, &*vust.read().unwrap()));
                        new_chunks.push(chunk_pos);
                    }
                }
            }
        }

        let all_chunks_positions = self.chunks.keys().cloned().collect::<Vec<glm::IVec3>>();
        for chunk_pos in all_chunks_positions {
            
            // "buffer" the edge chunks that are outside of the draw distance so that by the time the mesh memory is destroyed, it wont be while actively used in a draw call
            // lazy fix but it works
            let unrendered_chunk_offset = 1;

            let within_draw_distance_x = chunk_pos.x >= snapped_player_pos.x - (self.draw_distance as i32 + unrendered_chunk_offset) && chunk_pos.x <= snapped_player_pos.x + (self.draw_distance as i32 + unrendered_chunk_offset);
            let within_draw_distance_y = chunk_pos.y >= snapped_player_pos.y - (self.draw_distance as i32 + unrendered_chunk_offset) && chunk_pos.y <= snapped_player_pos.y + (self.draw_distance as i32 + unrendered_chunk_offset);
            let within_draw_distance_z = chunk_pos.z >= snapped_player_pos.z - (self.draw_distance as i32 + unrendered_chunk_offset) && chunk_pos.z <= snapped_player_pos.z + (self.draw_distance as i32 + unrendered_chunk_offset);

            if !(within_draw_distance_x && within_draw_distance_y && within_draw_distance_z) {
                self.chunks.remove(&chunk_pos);
            }
        }

        for chunk_pos in &new_chunks {
            self.thread_pool.send_task(crate::thread_pool::task::Task::GenTerrain {
                chunk_pos: *chunk_pos,
                blocks: self.chunks.get(chunk_pos).unwrap().get_blocks(),
                gen_func: |pos| {
                    let perlin_noise = OpenSimplex::new(51234);
                    let perlin_x = (pos.x.abs() as f64 % 200.0) / 200.0;
                    let perlin_z = (pos.z.abs() as f64 % 200.0) / 200.0;
                    let perlin_y = (perlin_noise.get([perlin_x, perlin_z]) * 100.0) as i32;
                    
                    if pos.y == perlin_y {
                        Blocks::GRASS_BLOCK.block_id()
                    } else {
                        Blocks::AIR.block_id()
                    }
                }
            });
        }

        for chunk_pos in new_chunks {
            self.thread_pool.send_task(crate::thread_pool::task::Task::GenMesh {
                chunk_pos: chunk_pos,
                blocks: self.chunks.get(&chunk_pos).unwrap().get_blocks(),
                vertex_buffer: self.chunks.get(&chunk_pos).unwrap().get_vertex_buffer(),
                vertex_count: self.chunks.get(&chunk_pos).unwrap().get_vertex_count(),
                neighbour_blocks: [
                    self.chunks.get(&glm::vec3(chunk_pos.x, chunk_pos.y, chunk_pos.z - 1)).map(|c| c.get_blocks()),
                    self.chunks.get(&glm::vec3(chunk_pos.x, chunk_pos.y, chunk_pos.z + 1)).map(|c| c.get_blocks()),
                    self.chunks.get(&glm::vec3(chunk_pos.x, chunk_pos.y - 1, chunk_pos.z)).map(|c| c.get_blocks()),
                    self.chunks.get(&glm::vec3(chunk_pos.x, chunk_pos.y + 1, chunk_pos.z)).map(|c| c.get_blocks()),
                    self.chunks.get(&glm::vec3(chunk_pos.x - 1, chunk_pos.y, chunk_pos.z)).map(|c| c.get_blocks()),
                    self.chunks.get(&glm::vec3(chunk_pos.x + 1, chunk_pos.y, chunk_pos.z)).map(|c| c.get_blocks())
                ],
                vust: Arc::clone(&vust)
            });
        }
    }

    pub fn draw(&self, vust: &vust::Vust, player_pos: glm::Vec3, camera_buffer_info: WriteDescriptorInfo) {
        vust.bind_pipeline(self.chunk_pipeline.handle());
        let snapped_player_pos = snap_player_pos_to_chunk(player_pos);

        for x in -self.draw_distance..self.draw_distance {
            for y in -self.draw_distance..self.draw_distance {
                for z in -self.draw_distance..self.draw_distance {
                    let chunk = self.chunks.get(&(glm::vec3(x as i32, y as i32, z as i32) + snapped_player_pos)).unwrap();
                    chunk.draw(
                        vust,
                        &self.chunk_pipeline,
                        camera_buffer_info,
                        WriteDescriptorInfo::Image {
                            image_view: self.atlas.view(),
                            sampler: self.atlas.sampler()
                        }
                    );
                }
            }
        }
    }
}

fn snap_player_pos_to_chunk(player_pos: glm::Vec3) -> glm::IVec3 {
    glm::vec3(
        player_pos.x.floor() as i32 / Chunk::SIZE as i32,
        player_pos.y.floor() as i32 / Chunk::SIZE as i32,
        player_pos.z.floor() as i32 / Chunk::SIZE as i32
    )
}
