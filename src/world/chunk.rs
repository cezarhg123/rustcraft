use vust::{buffer::Buffer, descriptor::Descriptor, pipeline::GraphicsPipeline, write_descriptor_info::WriteDescriptorInfo, Vust};
use crate::vertex::Vertex;
use super::{block::BlockID, blocks::Blocks};

pub struct Chunk {
    /// position of chunk, actual global position of it is chunk_pos * Chunk::SIZE
    chunk_pos: glm::IVec3,
    blocks: [[[BlockID; Chunk::SIZE]; Chunk::SIZE]; Chunk::SIZE],
    vertex_count: usize,
    descriptor: Descriptor,
    vertex_buffer: Option<Buffer>,
    uniform_buffer: Buffer
}

impl Chunk {
    pub const SIZE: usize = 16;

    pub fn new(chunk_pos: glm::IVec3, pipeline: &GraphicsPipeline, vust: &mut Vust) -> Chunk {
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
            blocks: [[[Blocks::AIR.block_id(); Chunk::SIZE]; Chunk::SIZE]; Chunk::SIZE],
            vertex_count: 0,
            descriptor: pipeline.create_descriptor(vust).unwrap(),
            vertex_buffer: None,
            uniform_buffer
        }
    }

    pub fn gen_terrain(&mut self, gen_func: fn(pos: glm::Vec3) -> BlockID) {
        for x in 0..Chunk::SIZE {
            for y in 0..Chunk::SIZE {
                for z in 0..Chunk::SIZE {
                    self.blocks[x][y][z] = gen_func(
                        glm::vec3(x as f32, y as f32, z as f32) +
                        (glm::vec3(self.chunk_pos.x as f32, self.chunk_pos.y as f32, self.chunk_pos.z as f32) * Chunk::SIZE as f32)
                    );
                }
            }
        }
    }

    pub fn gen_mesh(&mut self, vust: &mut Vust) {
        let mut vertices = Vec::new();

        for x in 0..Chunk::SIZE {
            for y in 0..Chunk::SIZE {
                for z in 0..Chunk::SIZE {
                    if self.blocks[x][y][z] == Blocks::AIR.block_id() {
                        continue;
                    }

                    let block_pos = glm::vec3(x as f32, y as f32, z as f32);
                    let block = self.blocks[x][y][z];

                    if z == 0 { // south
                        // check neighbour chunk
                    } else if z == Chunk::SIZE - 1 { // north
                        // check neighbour chunk
                    } else {
                        if self.blocks[x][y][z - 1] == 0 {
                            vertices.push([
                                Vertex::new(block_pos, block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), block.get_tr_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_tl_uv()),
                                
                                Vertex::new(block_pos, block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), block.get_tr_uv())
                            ]);
                        }

                        if self.blocks[x][y][z + 1] == 0 {
                            vertices.push([
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), block.get_tr_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), block.get_tl_uv()),
                                
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_tl_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), block.get_tr_uv())
                            ]);
                        }
                    }

                    if y == 0 { // bottom
                        // check neighbour chunk
                    } else if y == Chunk::SIZE - 1 { // top
                        // check neighbour chunk
                    } else {
                        if self.blocks[x][y - 1][z] == 0 {
                            vertices.push([
                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_tr_uv()),
                                Vertex::new(block_pos, block.get_tl_uv()),
                                
                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_tr_uv())
                            ]);
                        }

                        if self.blocks[x][y + 1][z] == 0 {
                            vertices.push([
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), block.get_tr_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), block.get_tl_uv()),
                                
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), block.get_tr_uv())
                            ]);
                        }
                    }

                    if x == 0 { // left
                        // check neighbour chunk
                    } else if x == Chunk::SIZE - 1 { // right
                        // check neighbour chunk
                    } else {
                        if self.blocks[x - 1][y][z] == 0 {
                            vertices.push([
                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_tr_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), block.get_tl_uv()),
                                
                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                Vertex::new(block_pos, block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_tr_uv())
                            ]);
                        }

                        if self.blocks[x + 1][y][z] == 0 {
                            vertices.push([
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), block.get_tr_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), block.get_tl_uv()),
                                
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_bl_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), block.get_br_uv()),
                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), block.get_tr_uv())
                            ]);
                        }
                    }
                }
            }
        }

        if !vertices.is_empty() {
            self.vertex_buffer = {
                let name = format!("chunk {} {} {} vertex buffer", self.chunk_pos.x, self.chunk_pos.y, self.chunk_pos.z);
                Some(
                    Buffer::builder()
                        .with_name(&name)
                        .with_data(vertices.as_slice())
                        .with_usage(vust::buffer::BufferUsageFlags::VERTEX_BUFFER)
                        .with_memory_location(vust::buffer::MemoryPropertyFlags::HOST_VISIBLE | vust::buffer::MemoryPropertyFlags::HOST_COHERENT)
                        .build(vust, true)
                )
            };
            self.vertex_count = vertices.len() * 6;
        }
    }

    pub fn draw(&self, vust: &mut Vust, pipeline: &GraphicsPipeline, camera_buffer_info: WriteDescriptorInfo, atlas_image_info: WriteDescriptorInfo) {
        if let Some(buffer) = &self.vertex_buffer {
            vust.update_descriptor_set(&self.descriptor, &[
                camera_buffer_info,
                WriteDescriptorInfo::Buffer { buffer: self.uniform_buffer.handle(), offset: 0, range: size_of::<glm::Mat4>() as u64 },
                atlas_image_info
            ]);
            vust.bind_descriptor_set(pipeline, &self.descriptor);
            vust.bind_vertex_buffer(buffer.handle());
            vust.draw(self.vertex_count as u32);
        }
    }

    pub fn cleanup(&mut self, vust: &mut Vust) {
        if let Some(buffer) = &mut self.vertex_buffer {
            buffer.destroy(vust);
        }
        self.uniform_buffer.destroy(vust);
    }
}
