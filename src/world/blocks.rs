use super::block::{Block, BlockID};

pub struct Blocks;

impl Blocks {
    pub const AIR: Block<'static> = Block::new(0, "Air", "air");
    pub const GRASS_BLOCK: Block<'static> = Block::new(1, "Grass Block", "grass_block");
    pub const DIRT: Block<'static> = Block::new(2, "Dirt", "dirt");
}