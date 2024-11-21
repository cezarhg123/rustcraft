pub mod chunk;
pub mod block;
pub mod blocks;

use std::{cell::RefCell, collections::HashMap, io::Cursor, sync::{mpsc, Arc, RwLock}, thread, time::Instant};
use block::BlockID;
use blocks::Blocks;
use chunk::Chunk;
use image::GenericImageView;
use noise::{NoiseFn, OpenSimplex, Perlin, Simplex, Worley};
use vust::{pipeline::{DescriptorSetBinding, DescriptorSetLayout, GraphicsPipeline, GraphicsPipelineCreateInfo}, texture::Texture, write_descriptor_info::WriteDescriptorInfo};
use crate::{vertex::Vertex, WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct World {
    // not the best performance cuz each element could be in random memory locations but good enough
    chunks: HashMap<glm::IVec3, RefCell<Chunk>>,
    chunk_pipeline: GraphicsPipeline,
    draw_distance: i8,
    atlas: Texture,
    noise: Perlin
}

impl World {
    // i doubt someone will travel 2 billion blocks
    // 1 billion blocks each direction
    pub const MAX_BLOCKS: usize = 2_000_000_000;

    pub fn new<'a>(draw_distance: i8, vust: &vust::Vust) -> World {
        let chunk_pipeline = GraphicsPipeline::new(
            &*vust,
            GraphicsPipelineCreateInfo {
                name: "pipeline".to_string(),
                vertex_bin: std::fs::read("shaders/chunk/mesh.vert.spv").unwrap(),
                fragment_bin: std::fs::read("shaders/chunk/mesh.frag.spv").unwrap(),
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

        let noise = Perlin::new(121312);

        let mut chunks = HashMap::new();

        let world_gen_time = Instant::now();

        for x in -draw_distance..draw_distance {
            for y in -draw_distance..draw_distance {
                for z in -draw_distance..draw_distance {
                    chunks.insert(glm::vec3(x as i32, y as i32, z as i32), RefCell::new(Chunk::new(glm::vec3(x as i32, y as i32, z as i32), &chunk_pipeline, &*vust)));
                }
            }
        }
        let chunk_init_time = world_gen_time.elapsed();
        println!("chunk hashmap init time: {}ms", chunk_init_time.as_millis());

        for chunk in chunks.values_mut() {
            chunk.borrow_mut().gen_terrain(&noise);
        }
        let terrain_gen_time = world_gen_time.elapsed() - chunk_init_time;
        println!("terrain generation time: {}ms", terrain_gen_time.as_millis());

        for x in -draw_distance..draw_distance {
            for y in -draw_distance..draw_distance {
                for z in -draw_distance..draw_distance {
                    chunks.get(&glm::vec3(x as i32, y as i32, z as i32)).unwrap().borrow_mut().gen_mesh(
                        vust,
                        [
                            chunks.get(&glm::vec3(x as i32, y as i32, z as i32 - 1)).map(|chunk| chunk.borrow()),
                            chunks.get(&glm::vec3(x as i32, y as i32, z as i32 + 1)).map(|chunk| chunk.borrow()),
                            chunks.get(&glm::vec3(x as i32, y as i32 - 1, z as i32)).map(|chunk| chunk.borrow()),
                            chunks.get(&glm::vec3(x as i32, y as i32 + 1, z as i32)).map(|chunk| chunk.borrow()),
                            chunks.get(&glm::vec3(x as i32 - 1, y as i32, z as i32)).map(|chunk| chunk.borrow()),
                            chunks.get(&glm::vec3(x as i32 + 1, y as i32, z as i32)).map(|chunk| chunk.borrow())
                        ]
                    );
                }
            }
        }
        let mesh_gen_time = world_gen_time.elapsed() - chunk_init_time - terrain_gen_time;
        println!("mesh generation time: {}ms", mesh_gen_time.as_millis());

        println!("world gen {}ms", world_gen_time.elapsed().as_millis());


        World {
            chunks,
            chunk_pipeline,
            draw_distance,
            atlas: atlas_texture,
            noise
        }
    }

    pub fn update_chunks(&mut self, player_pos: glm::Vec3, vust: &vust::Vust) {
        // snap the player position to closest chunk
        let snapped_player_pos = snap_player_pos_to_chunk(player_pos);

        let mut new_chunks = Vec::new();

        for x in -self.draw_distance..self.draw_distance {
            for y in -self.draw_distance..self.draw_distance {
                for z in -self.draw_distance..self.draw_distance {
                    let chunk_pos = glm::vec3(x as i32, y as i32, z as i32) + snapped_player_pos;
                    if self.chunks.get(&chunk_pos).is_none() {
                        self.chunks.insert(chunk_pos, RefCell::new(Chunk::new(chunk_pos, &self.chunk_pipeline, vust)));
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
    }

    pub fn draw(&self, vust: &vust::Vust, player_pos: glm::Vec3, camera_buffer_info: WriteDescriptorInfo) {
        vust.bind_pipeline(self.chunk_pipeline.handle());
        let snapped_player_pos = snap_player_pos_to_chunk(player_pos);

        for x in -self.draw_distance..self.draw_distance {
            for y in -self.draw_distance..self.draw_distance {
                for z in -self.draw_distance..self.draw_distance {
                    let chunk = self.chunks.get(&(glm::vec3(x as i32, y as i32, z as i32) + snapped_player_pos)).unwrap();
                    chunk.borrow().draw(
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
