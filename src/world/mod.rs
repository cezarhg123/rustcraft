pub mod chunk;
pub mod block;

use std::{collections::HashMap, mem::size_of, sync::mpsc};
use ash::vk;
use noise::{Perlin, NoiseFn};
use crate::{timer::Timer, engine::{buffer::Buffer, vertex::{Vertex, CompressedVertex}, self, instance::DEBUG}, world::block::{get_block, get_block_id}};
use self::{chunk::{Chunk, build_centre_mesh}, block::{BlockType, Block}};

/// has position of 1, 2, 3 instead of going in intervals of `Chunk::SIZE`
type ChunkPos = glm::IVec3;
type BufferOffset = u64;
type ModelOffset = u64;

pub struct World {
    chunks: HashMap<ChunkPos, (Chunk, BufferOffset, ModelOffset)>,
    world_vertex_buffer: Buffer<CompressedVertex>,
    model_uniform_buffer: Buffer<f32>,
    half_distance: i32
}

// HOW?
// first the chunk blocks are generated
// then the mesh is generated in parallel with worker threads, they send a Box to a 'buffer write' thread which writes to the world vertex buffer


impl World {
    pub const MAX_BLOCKS: u64 = (Chunk::SIZE as u64).pow(3);
    pub const VERTICES_PER_BLOCK: u64 = 36;
    pub const MAX_VERTICES_PER_CHUNK: u64 = (World::MAX_BLOCKS / 2) * World::VERTICES_PER_BLOCK;
    pub const MAX_VERTICES_PER_CHUNK_BYTES: u64 = World::MAX_VERTICES_PER_CHUNK * size_of::<CompressedVertex>() as u64;

    pub const MAX_CENTRE_VERTICES_PER_CHUNK: u64 = ((Chunk::SIZE as u64 - 2).pow(3) / 2) * World::VERTICES_PER_BLOCK;
    pub const MAX_CENTRE_VERTICES_PER_CHUNK_BYTES: u64 = World::MAX_CENTRE_VERTICES_PER_CHUNK * size_of::<CompressedVertex>() as u64;

    pub fn new(distance: u32) -> World {
        let vertex_buffer = Buffer::new_empty(World::MAX_VERTICES_PER_CHUNK_BYTES * (distance as u64).pow(3), vk::BufferUsageFlags::VERTEX_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        let model_uniform_buffer = Buffer::new_empty(64 * (distance as u64).pow(3), vk::BufferUsageFlags::UNIFORM_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        let mut chunks = HashMap::with_capacity((distance as usize).pow(3));

        let half_distance = distance as i32 / 2;

        let perlin = Perlin::new(123);

        let mut offset = 0;
        let mut model_offset = 0;

        let (
            worker_sender,
            buffer_writer_receiver
        ) = mpsc::channel::<
            (Box<[CompressedVertex; World::MAX_CENTRE_VERTICES_PER_CHUNK as usize]>, BufferOffset)
        >();

        let vertex_buffer_memory = vertex_buffer.memory();

        let buffer_writer_thread = std::thread::spawn(move || {
            let iterations = distance.pow(3);

            for i in 0..iterations {
                let buffer = buffer_writer_receiver.recv().unwrap();
                unsafe {
                    let data_ptr = engine::instance::get_device().map_memory(vertex_buffer_memory, buffer.1, World::MAX_CENTRE_VERTICES_PER_CHUNK_BYTES, vk::MemoryMapFlags::empty()).unwrap() as *mut CompressedVertex;
                    data_ptr.copy_from_nonoverlapping(buffer.0.as_ptr(), buffer.0.len());
                    engine::instance::get_device().unmap_memory(vertex_buffer_memory);
                }
            }
        });
        let mut worker_threads = Vec::new();

        let mut timer = Timer::new();

        for x in -half_distance..half_distance {
            for y in -half_distance..half_distance {
                for z in -half_distance..half_distance {
                    let global_pos = glm::vec3(x * Chunk::SIZE as i32, y * Chunk::SIZE as i32, z * Chunk::SIZE as i32);

                    let chunk = Chunk::new(global_pos, |global_pos| {
                        if global_pos.y < 3 {
                            1
                        } else if global_pos.x == 1 && global_pos.y == 5 && global_pos.z == -4 {
                            1
                        } else { 0 }
                    });
                    
                    chunks.insert(global_pos, (chunk, offset, model_offset));

                    let chunk_ptr: *const Chunk = &chunks.get(&global_pos).unwrap().0;
                    let chunk_ptr_num = chunk_ptr as usize;
                    
                    let worker_sender = worker_sender.clone();
                    worker_threads.push(std::thread::spawn(move || {
                        let chunk_ptr = chunk_ptr_num as *const Chunk;
                        let data = build_centre_mesh(chunk_ptr);
                        worker_sender.send((data, offset)).unwrap();
                    }));

                    offset += World::MAX_VERTICES_PER_CHUNK_BYTES;
                    model_offset += size_of::<glm::Mat4>() as u64;
                }
            }
        }

        for thread_handle in worker_threads {
            thread_handle.join().unwrap();
        }
        buffer_writer_thread.join().unwrap();

        timer.tick();
        let elapsed = timer.elapsed_millis();
        if DEBUG {
            println!("Loaded {} chunks in {}ms", chunks.len(), elapsed);
        }

        World {
            chunks,
            world_vertex_buffer: vertex_buffer,
            model_uniform_buffer,
            half_distance
        }
    }

    pub fn draw(&self, camera_buffer_info: vk::DescriptorBufferInfo, atlas_image_info: vk::DescriptorImageInfo) {
        for chunk in self.chunks.values() {
            chunk.0.write_descriptor(
                self.model_uniform_buffer.map(chunk.2, size_of::<glm::Mat4>() as u64),
                self.model_uniform_buffer.buffer(),
                chunk.2,
                camera_buffer_info,
                atlas_image_info
            );
            self.model_uniform_buffer.unmap();

            engine::instance::draw(self.world_vertex_buffer.buffer(), chunk.1, chunk.0.descriptor_set(), chunk.0.vertex_count());
        }
    }
}
