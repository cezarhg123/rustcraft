pub mod chunk;
pub mod block;
pub mod blocks;

use std::{collections::HashMap, io::Cursor, sync::mpsc, thread};
use blocks::Blocks;
use chunk::Chunk;
use image::GenericImageView;
use vust::{pipeline::{DescriptorSetBinding, DescriptorSetLayout, GraphicsPipeline, GraphicsPipelineCreateInfo}, texture::Texture, write_descriptor_info::WriteDescriptorInfo};
use crate::{thread_pool::ThreadPool, vertex::Vertex, WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct World {
    // not the best performance cuz each element could be in random memory locations but good enough
    chunks: HashMap<glm::I64Vec3, Chunk>,
    chunk_pipeline: GraphicsPipeline,
    draw_distance: i8,
    atlas: Texture,
    thread_pool: ThreadPool
}

impl World {
    pub fn new(draw_distance: i8, vust: &mut vust::Vust) -> World {
        let chunk_pipeline = GraphicsPipeline::new(
            &vust,
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
            .build(vust)
            .unwrap();

        let mut thread_pool = ThreadPool::new(8);

        let mut chunks = HashMap::new();

        for x in -draw_distance..draw_distance {
            for y in -draw_distance..draw_distance {
                for z in -draw_distance..draw_distance {
                    chunks.insert(glm::vec3(x as i64, y as i64, z as i64), Chunk::new(glm::vec3(x as i32, y as i32, z as i32), &chunk_pipeline, vust));
                }
            }
        }

        for x in -draw_distance..draw_distance {
            for y in -draw_distance..draw_distance {
                for z in -draw_distance..draw_distance {
                    thread_pool.send_task(crate::thread_pool::task::Task::GenTerrain {
                        chunk_pos: glm::vec3(x as i32, y as i32, z as i32),
                        blocks: chunks.get_mut(&glm::vec3(x as i64, y as i64, z as i64)).unwrap().get_blocks(),
                        gen_func: |pos| {
                            if pos.y == -6 {
                                return Blocks::GRASS_BLOCK.block_id();
                            }
                            
                            if pos.x == 2 && pos.y == 2 && pos.z == 2 {
                                return Blocks::GRASS_BLOCK.block_id();
                            }
                            
                            if pos.x == -2 && pos.y == -2 && pos.z == -2 {
                                return Blocks::DIRT.block_id();
                            }

                            if pos.x >= 14 && pos.x <= 18 && pos.y == 5 && pos.z == 5 {
                                return Blocks::GRASS_BLOCK.block_id();
                            }

                            Blocks::AIR.block_id()
                        }
                    });
                }
            }
        }

        for x in -draw_distance..draw_distance {
            for y in -draw_distance..draw_distance {
                for z in -draw_distance..draw_distance {
                    let chunk = chunks.get(&glm::vec3(x as i64, y as i64, z as i64)).unwrap();
                    thread_pool.send_task(crate::thread_pool::task::Task::GenMesh {
                        chunk_pos: glm::vec3(x as i32, y as i32, z as i32),
                        blocks: chunk.get_blocks(),
                        vertex_buffer: chunk.get_vertex_buffer(),
                        vertex_count: chunk.get_vertex_count(),
                        neighbour_blocks: [
                            chunks.get(&glm::vec3(x as i64, y as i64, z as i64 - 1)).map(|c| c.get_blocks()),
                            chunks.get(&glm::vec3(x as i64, y as i64, z as i64 + 1)).map(|c| c.get_blocks()),
                            chunks.get(&glm::vec3(x as i64, y as i64 - 1, z as i64)).map(|c| c.get_blocks()),
                            chunks.get(&glm::vec3(x as i64, y as i64 + 1, z as i64)).map(|c| c.get_blocks()),
                            chunks.get(&glm::vec3(x as i64 - 1, y as i64, z as i64)).map(|c| c.get_blocks()),
                            chunks.get(&glm::vec3(x as i64 + 1, y as i64, z as i64)).map(|c| c.get_blocks())
                        ],
                        vust_device: vust.get_device(),
                        memory_allocator: vust.get_memory_allocator()
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

    pub fn draw(&self, vust: &mut vust::Vust, camera_buffer_info: WriteDescriptorInfo) {
        vust.bind_pipeline(self.chunk_pipeline.handle());
        for chunk in self.chunks.values() {
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

    pub fn cleanup(&mut self, vust: &mut vust::Vust) {
        for chunk in self.chunks.values_mut() {
            chunk.cleanup(vust);
        }

        self.atlas.destroy(vust);
    }
}
