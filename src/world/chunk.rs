/// TODO
/// `World` should also contain a large buffer for the model uniform of each chunk
/// and then give an offset to each chunk


use std::{collections::HashMap, mem::{size_of, ManuallyDrop}};
use ash::vk;
use crate::engine::{buffer::Buffer, vertex::{Vertex, CompressedVertex}, self};
use super::{block::{Block, BlockType, get_block, BLOCKS}, World};

type LocalPos = glm::I8Vec3;
pub type GlobalPos = glm::IVec3;

#[derive(Debug, Clone)]
pub struct Chunk {
    position: glm::IVec3,
    model: glm::Mat4,
    blocks: HashMap<LocalPos, usize>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    vertex_count: u64
}

impl Chunk {
    pub const SIZE: u8 = 20;

    pub fn new(position: glm::IVec3, chunk_gen: impl Fn(GlobalPos) -> usize) -> Chunk {
        let mut blocks = HashMap::new();

        for x in 0..Chunk::SIZE {
            for y in 0..Chunk::SIZE {
                for z in 0..Chunk::SIZE {
                    let global_pos = position + glm::vec3(x as i32, y as i32, z as i32);

                    blocks.insert(glm::vec3(x as i8, y as i8, z as i8), chunk_gen(global_pos));
                }
            }
        }

        let descriptor_pool = engine::instance::create_descriptor_pool();

        Chunk {
            position,
            model: glm::Mat4::new_translation(&position.cast()),
            blocks,
            descriptor_pool,
            descriptor_set: unsafe {
                engine::instance::get_device().allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::builder()
                        .descriptor_pool(descriptor_pool)
                        .set_layouts(&[engine::instance::get_descriptor_set_layout()])
                        .build()
                ).unwrap()[0]
            },
            vertex_count: 0
        }
    }

    pub fn write_descriptor(&self, model_uniform_buffer_ptr: *mut f32, model_buffer: vk::Buffer, model_offset: u64, camera_buffer_info: vk::DescriptorBufferInfo, atlas_image_info: vk::DescriptorImageInfo) {        
        unsafe {
            model_uniform_buffer_ptr.copy_from_nonoverlapping(self.model.as_ptr(), 16);

            engine::instance::get_device().update_descriptor_sets(&[
                vk::WriteDescriptorSet::builder()
                    .dst_set(self.descriptor_set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&[
                        camera_buffer_info
                    ])
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(self.descriptor_set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&[
                        atlas_image_info
                    ])
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(self.descriptor_set)
                    .dst_binding(2)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&[
                        vk::DescriptorBufferInfo::builder()
                            .buffer(model_buffer)
                            .offset(model_offset)
                            .range(size_of::<glm::Mat4>() as u64)
                            .build()
                    ])
                    .build()
            ], &[]);
        }
    }

    pub fn position(&self) -> glm::IVec3 {
        self.position
    }

    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }

    pub fn vertex_count(&self) -> u64 {
        self.vertex_count
    }
}

fn create_west_face(position: glm::Vec3, uv: glm::Vec2) -> [CompressedVertex; 6] {
    [
        CompressedVertex::new_raw(position, uv),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 1.0, 1.0), uv + glm::vec2(0.1, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 1.0, 0.0), uv + glm::vec2(0.0, 0.1)),
        CompressedVertex::new_raw(position, uv),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 0.0, 1.0), uv + glm::vec2(0.1, 0.0)),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 1.0, 1.0), uv + glm::vec2(0.1, 0.1))
    ]
}

fn create_east_face(position: glm::Vec3, uv: glm::Vec2) -> [CompressedVertex; 6] {
    [
        CompressedVertex::new_raw(position + glm::vec3(1.0, 0.0, 1.0), uv),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 1.0, 1.0), uv + glm::vec2(0.0, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 0.0, 1.0), uv),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 0.0, 0.0), uv + glm::vec2(0.1, 0.0)),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.1))
    ]
}

fn create_up_face(position: glm::Vec3, uv: glm::Vec2) -> [CompressedVertex; 6] {
    [
        CompressedVertex::new_raw(position + glm::vec3(0.0, 1.0, 1.0), uv),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 1.0, 0.0), uv + glm::vec2(0.0, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 1.0, 1.0), uv),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 1.0, 1.0), uv + glm::vec2(0.1, 0.0)),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.1))
    ]
}

fn create_down_face(position: glm::Vec3, uv: glm::Vec2) -> [CompressedVertex; 6] {
    [
        CompressedVertex::new_raw(position + glm::vec3(0.0, 0.0, 1.0), uv),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 0.0, 0.0), uv + glm::vec2(0.0, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 0.0, 0.0), uv + glm::vec2(0.1, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 0.0, 1.0), uv),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 0.0, 0.0), uv + glm::vec2(0.1, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 0.0, 1.0), uv + glm::vec2(0.1, 0.0)),
    ]
}

fn create_north_face(position: glm::Vec3, uv: glm::Vec2) -> [CompressedVertex; 6] {
    [
        CompressedVertex::new_raw(position + glm::vec3(1.0, 0.0, 0.0), uv),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 1.0, 0.0), uv + glm::vec2(0.0, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 0.0, 0.0), uv),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 0.0, 0.0), uv + glm::vec2(0.1, 0.0)),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 1.0, 0.0), uv + glm::vec2(0.1, 0.1))
    ]
}

fn create_south_face(position: glm::Vec3, uv: glm::Vec2) -> [CompressedVertex; 6] {
    [
        CompressedVertex::new_raw(position + glm::vec3(0.0, 0.0, 1.0), uv),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 1.0, 1.0), uv + glm::vec2(0.1, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 1.0, 1.0), uv + glm::vec2(0.0, 0.1)),
        CompressedVertex::new_raw(position + glm::vec3(0.0, 0.0, 1.0), uv),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 0.0, 1.0), uv + glm::vec2(0.1, 0.0)),
        CompressedVertex::new_raw(position + glm::vec3(1.0, 1.0, 1.0), uv + glm::vec2(0.1, 0.1))
    ]
}

pub fn build_centre_mesh(chunk: *const Chunk) -> Box<[CompressedVertex; World::MAX_CENTRE_VERTICES_PER_CHUNK as usize]> {
    let mut buffer = Box::new([CompressedVertex(0); World::MAX_CENTRE_VERTICES_PER_CHUNK as usize]);
    let chunk = unsafe { &*chunk };

    fn write_to_world_buffer(buffer: &mut Box<[CompressedVertex; World::MAX_CENTRE_VERTICES_PER_CHUNK as usize]>, vertex: [CompressedVertex; 6], index: usize) -> usize {
        for i in 0..6 {
            buffer[index+i] = vertex[i];
        }
        index + 6
    }

    fn update_vertex_count(chunk: *const Chunk) {
        unsafe {
            let chunk = chunk.cast_mut();

            (*chunk).vertex_count += 6;
        }
    }

    let mut index = 0;

    for x in 1..Chunk::SIZE - 1 {
        for y in 1..Chunk::SIZE - 1 {
            for z in 1..Chunk::SIZE - 1 {
                let local_pos = glm::vec3(x as i8, y as i8, z as i8);

                let block_id = *chunk.blocks.get(&local_pos).unwrap();

                if block_id != 0 {
                    let block = BLOCKS[block_id];
                    if *chunk.blocks.get(&(local_pos + glm::vec3(-1, 0, 0))).unwrap() == 0 {
                        index = write_to_world_buffer(&mut buffer, create_west_face(glm::vec3(x as f32, y as f32, z as f32), block.side_uv), index);
                        update_vertex_count(chunk);
                    }
                    if *chunk.blocks.get(&(local_pos + glm::vec3(1, 0, 0))).unwrap() == 0 {
                        index = write_to_world_buffer(&mut buffer, create_east_face(glm::vec3(x as f32, y as f32, z as f32), block.side_uv), index);
                        update_vertex_count(chunk);
                    }
                    if *chunk.blocks.get(&(local_pos + glm::vec3(0, 1, 0))).unwrap() == 0 {
                        index = write_to_world_buffer(&mut buffer, create_up_face(glm::vec3(x as f32, y as f32, z as f32), block.top_uv), index);
                        update_vertex_count(chunk);   
                    }
                    if *chunk.blocks.get(&(local_pos + glm::vec3(0, -1, 0))).unwrap() == 0 {
                        index = write_to_world_buffer(&mut buffer, create_down_face(glm::vec3(x as f32, y as f32, z as f32), block.bottom_uv), index);
                        update_vertex_count(chunk);
                    }
                    if *chunk.blocks.get(&(local_pos + glm::vec3(0, 0, -1))).unwrap() == 0 {
                        index = write_to_world_buffer(&mut buffer, create_north_face(glm::vec3(x as f32, y as f32, z as f32), block.side_uv), index);
                        update_vertex_count(chunk);
                    }
                    if *chunk.blocks.get(&(local_pos + glm::vec3(0, 0, 1))).unwrap() == 0 {
                        index = write_to_world_buffer(&mut buffer, create_south_face(glm::vec3(x as f32, y as f32, z as f32), block.side_uv), index);
                        update_vertex_count(chunk);
                    }
                }
            }
        }
    }

    buffer
}
