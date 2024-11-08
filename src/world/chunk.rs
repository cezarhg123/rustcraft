use std::sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use vust::{buffer::Buffer, descriptor::Descriptor, pipeline::GraphicsPipeline, write_descriptor_info::WriteDescriptorInfo, Vust};
use crate::vertex::Vertex;
use super::{block::BlockID, blocks::Blocks};

pub type BlockArray = [[[BlockID; Chunk::SIZE]; Chunk::SIZE]; Chunk::SIZE];

pub struct Chunk {
    /// position of chunk, actual global position of it is chunk_pos * Chunk::SIZE
    chunk_pos: glm::IVec3,
    blocks: Arc<RwLock<BlockArray>>,
    vertex_count: Arc<AtomicUsize>,
    descriptor: Descriptor,
    vertex_buffer: Arc<Mutex<Option<Buffer>>>,
    uniform_buffer: Buffer
}

impl Chunk {
    pub const SIZE: usize = 16;

    pub fn new(chunk_pos: glm::IVec3, pipeline: &GraphicsPipeline, vust: &Vust) -> Chunk {
        let uniform_buffer = {
            let name = format!("chunk {} {} {} uniform buffer", chunk_pos.x, chunk_pos.y, chunk_pos.z);

            Buffer::builder()
                .with_name(&name)
                .with_data(&[
                    glm::Mat4::new_translation(&(
                        glm::vec3(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32) * Chunk::SIZE as f32
                    ))
                ])
                .with_usage(vust::buffer::BufferUsageFlags::UNIFORM_BUFFER)
                .with_memory_location(vust::buffer::MemoryPropertyFlags::HOST_VISIBLE | vust::buffer::MemoryPropertyFlags::HOST_COHERENT)
                .build(vust, true)
        };
        
        Chunk {
            chunk_pos,
            blocks: Arc::new(RwLock::new([[[Blocks::AIR.block_id(); Chunk::SIZE]; Chunk::SIZE]; Chunk::SIZE])),
            vertex_count: Arc::new(AtomicUsize::new(0)),
            descriptor: pipeline.create_descriptor(vust).unwrap(),
            vertex_buffer: Arc::new(Mutex::new(None)),
            uniform_buffer
        }
    }

    pub fn get_blocks(&self) -> Arc<RwLock<BlockArray>> {
        Arc::clone(&self.blocks)
    }

    pub fn get_vertex_buffer(&self) -> Arc<Mutex<Option<Buffer>>> {
        Arc::clone(&self.vertex_buffer)
    }

    pub fn get_vertex_count(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.vertex_count)
    }

    pub fn draw(&self, vust: &Vust, pipeline: &GraphicsPipeline, camera_buffer_info: WriteDescriptorInfo, atlas_image_info: WriteDescriptorInfo) {
        if let Ok(mutex_guard) = self.vertex_buffer.try_lock() {
            if let Some(buffer) = mutex_guard.as_ref() {
                vust.update_descriptor_set(&self.descriptor, &[
                    camera_buffer_info,
                    WriteDescriptorInfo::Buffer { buffer: self.uniform_buffer.handle(), offset: 0, range: size_of::<glm::Mat4>() as u64 },
                    atlas_image_info
                ]);
                vust.bind_descriptor_set(pipeline, &self.descriptor);
                vust.bind_vertex_buffer(buffer.handle());
                vust.draw(self.vertex_count.load(Ordering::Relaxed) as u32);
            }
        }
    }
}
