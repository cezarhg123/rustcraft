use std::sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex, RwLock};
use vust::{buffer::Buffer, Vust};
use crate::{vertex::Vertex, world::{block::BlockID, blocks::Blocks, chunk::{BlockArray, Chunk}}};

pub enum Task {
    GenTerrain {
        chunk_pos: glm::IVec3,
        blocks: Arc<RwLock<BlockArray>>,
        gen_func: fn(glm::IVec3) -> BlockID
    },
    GenMesh {
        chunk_pos: glm::IVec3,
        blocks: Arc<RwLock<BlockArray>>,
        vertex_buffer: Arc<Mutex<Option<Buffer>>>,
        vertex_count: Arc<AtomicUsize>,
        /// * 0 = -z
        /// * 1 = +z
        /// * 2 = -y
        /// * 3 = +y
        /// * 4 = -x
        /// * 5 = +x
        neighbour_blocks: [Option<Arc<RwLock<BlockArray>>>; 6],
        vust: Arc<RwLock<Vust>>
    },
    /// Kill Yourself - kill this thread
    KYS
}

impl Task {
    pub fn kys(&self) -> bool {
        matches!(self, Self::KYS)
    }

    pub fn run(&self) {
        match self {
            Self::GenTerrain {
                chunk_pos,
                blocks,
                gen_func
            } => {
                for x in 0..Chunk::SIZE {
                    for y in 0..Chunk::SIZE {
                        for z in 0..Chunk::SIZE {
                            blocks.write().unwrap()[x][y][z] = gen_func((*chunk_pos * Chunk::SIZE as i32) + glm::vec3(x as i32, y as i32, z as i32));
                        }
                    }
                }
            },

            Self::GenMesh {
                chunk_pos,
                blocks,
                vertex_buffer,
                vertex_count,
                neighbour_blocks,
                vust
            } => {
                let mut vertices = Vec::new();

                for x in 0..Chunk::SIZE {
                    for y in 0..Chunk::SIZE {
                        for z in 0..Chunk::SIZE {
                            let blocks = blocks.read().unwrap();
                            if blocks[x][y][z] == 0 {
                                continue;
                            }
                        
                            let block_pos = glm::vec3(x as f32, y as f32, z as f32);
                            let block = blocks[x][y][z];
                            
                            if z == 0 {
                                match &neighbour_blocks[0] {
                                    Some(neighbour_blocks) => {
                                        let neighbour_blocks = neighbour_blocks.read().unwrap();
                                        if neighbour_blocks[x][y][Chunk::SIZE - 1] == 0 {
                                            vertices.push([
                                                Vertex::new(block_pos, block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), block.get_tr_uv()),
                                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_tl_uv()),

                                                Vertex::new(block_pos, block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_br_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), block.get_tr_uv())
                                            ]);
                                        }
                                    },
                                    None => {} // neighbour chunk doesn't exist so dont create face
                                }
                            } else {
                                if blocks[x][y][z - 1] == 0 {
                                    vertices.push([
                                        Vertex::new(block_pos, block.get_bl_uv()),
                                        Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), block.get_tr_uv()),
                                        Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_tl_uv()),

                                        Vertex::new(block_pos, block.get_bl_uv()),
                                        Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_br_uv()),
                                        Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), block.get_tr_uv())
                                    ]);
                                }
                            }

                            if z == Chunk::SIZE - 1 {
                                match &neighbour_blocks[1] {
                                    Some(neighbour_blocks) => {
                                        let neighbour_blocks = neighbour_blocks.read().unwrap();
                                        if neighbour_blocks[x][y][0] == 0 {
                                            vertices.push([
                                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), block.get_tr_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), block.get_tl_uv()),
        
                                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_tl_uv()),
                                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), block.get_tr_uv())
                                            ]);
                                        }
                                    },
                                    None => {} // neighbour chunk doesn't exist so dont create face
                                }
                            } else {
                                if blocks[x][y][z + 1] == 0 {
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

                            if y == 0 {
                                match &neighbour_blocks[2] {
                                    Some(neighbour_blocks) => {
                                        let neighbour_blocks = neighbour_blocks.read().unwrap();
                                        if neighbour_blocks[x][Chunk::SIZE - 1][z] == 0 {
                                            vertices.push([
                                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_tr_uv()),
                                                Vertex::new(block_pos, block.get_tl_uv()),
        
                                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), block.get_br_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_tr_uv())
                                            ]);
                                        }
                                    },
                                    None => {} // neighbour chunk doesn't exist so dont create face
                                }
                            } else {
                                if blocks[x][y - 1][z] == 0 {
                                    vertices.push([
                                        Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                        Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_tr_uv()),
                                        Vertex::new(block_pos, block.get_tl_uv()),

                                        Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                        Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), block.get_br_uv()),
                                        Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_tr_uv())
                                    ]);
                                }
                            }

                            if y == Chunk::SIZE - 1 {
                                match &neighbour_blocks[3] {
                                    Some(neighbour_blocks) => {
                                        let neighbour_blocks = neighbour_blocks.read().unwrap();
                                        if neighbour_blocks[x][0][z] == 0 {
                                            vertices.push([
                                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), block.get_tr_uv()),
                                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), block.get_tl_uv()),
        
                                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), block.get_br_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), block.get_tr_uv())
                                            ]);
                                        }
                                    },
                                    None => {} // neighbour chunk doesn't exist so dont create face
                                }
                            } else {
                                if blocks[x][y + 1][z] == 0 {
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
                            
                            if x == 0 {
                                match &neighbour_blocks[4] {
                                    Some(neighbour_blocks) => {
                                        let neighbour_blocks = neighbour_blocks.read().unwrap();
                                        if neighbour_blocks[Chunk::SIZE - 1][y][z] == 0 {
                                            vertices.push([
                                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_tr_uv()),
                                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), block.get_tl_uv()),
        
                                                Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                                Vertex::new(block_pos, block.get_br_uv()),
                                                Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_tr_uv())
                                            ]);
                                        }
                                    },
                                    None => {} // neighbour chunk doesn't exist so dont create face
                                }
                            } else {
                                if blocks[x - 1][y][z] == 0 {
                                    vertices.push([
                                        Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                        Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_tr_uv()),
                                        Vertex::new(block_pos + glm::vec3(0.0, 1.0, 1.0), block.get_tl_uv()),

                                        Vertex::new(block_pos + glm::vec3(0.0, 0.0, 1.0), block.get_bl_uv()),
                                        Vertex::new(block_pos, block.get_br_uv()),
                                        Vertex::new(block_pos + glm::vec3(0.0, 1.0, 0.0), block.get_tr_uv())
                                    ]);
                                }
                            }

                            if x == Chunk::SIZE - 1 {
                                match &neighbour_blocks[5] {
                                    Some(neighbour_blocks) => {
                                        let neighbour_blocks = neighbour_blocks.read().unwrap();
                                        if neighbour_blocks[0][y][z] == 0 {
                                            vertices.push([
                                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), block.get_tr_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 0.0), block.get_tl_uv()),
        
                                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 0.0), block.get_bl_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 0.0, 1.0), block.get_br_uv()),
                                                Vertex::new(block_pos + glm::vec3(1.0, 1.0, 1.0), block.get_tr_uv())
                                            ]);
                                        }
                                    },
                                    None => {} // neighbour chunk doesn't exist so dont create face
                                }
                            } else {
                                if blocks[x + 1][y][z] == 0 {
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
                    let name = format!("chunk {} {} {} vertex buffer", chunk_pos.x, chunk_pos.y, chunk_pos.z);
                    let buffer = Buffer::builder()
                        .with_name(&name)
                        .with_data(vertices.as_slice())
                        .with_usage(vust::buffer::BufferUsageFlags::VERTEX_BUFFER)
                        .with_memory_location(vust::buffer::MemoryPropertyFlags::HOST_VISIBLE | vust::buffer::MemoryPropertyFlags::HOST_COHERENT)
                        .build(&*vust.read().unwrap(), true);

                    *vertex_buffer.lock().unwrap() = Some(buffer);
                    vertex_count.store(vertices.len() * 6, Ordering::Relaxed);
                }
            }
            Self::KYS => {}
        }
    }
}