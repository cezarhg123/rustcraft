use crate::ptr_wrapper::PtrWrapper;
use super::{block::BlockID, chunk::Chunk};

#[derive(Debug)]
pub enum Job {
    GenerateTerrain {
        chunk: PtrWrapper<Chunk>,
        gen_func: fn(glm::IVec3) -> BlockID
    },
    GenerateMesh {
        chunk: PtrWrapper<Chunk>,
        /// * 0 = -x
        /// * 1 = +x
        /// * 2 = -y
        /// * 3 = +y
        /// * 4 = -z
        /// * 5 = +z
        neighbors: [Option<PtrWrapper<Chunk>>; 6]
    },
    /// kill yourself
    KYS
}

impl Job {
    pub fn do_job(&self) {
        match self {
            Job::GenerateTerrain {..} => self.generate_terrain(),
            Job::GenerateMesh {..} => self.generate_mesh(),
            Job::KYS => {}
        }
    }

    fn generate_terrain(&self) {
        match self {
            Job::GenerateTerrain { chunk, gen_func } => {
                chunk.as_mut().generate_terrain(gen_func);
            },
            _ => {}
        }
    }

    fn generate_mesh(&self) {
        match self {
            Job::GenerateMesh { chunk, neighbors } => {
                chunk.as_mut().generate_mesh(neighbors);
            },
            _ => {}
        }
    }
}
