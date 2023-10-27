#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Block {
    pub name: &'static str,
    pub dev_name: &'static str,
    pub block_type: BlockType,
    pub top_uv: glm::Vec2,
    pub side_uv: glm::Vec2,
    pub bottom_uv: glm::Vec2
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum BlockType {
    Air,
    Solid
}

pub const BLOCKS: [Block; 2] = [
    Block {
        name: "Air",
        dev_name: "air",
        block_type: BlockType::Air,
        top_uv: glm::Vec2::new(0.0, 0.0),
        side_uv: glm::Vec2::new(0.0, 0.0),
        bottom_uv: glm::Vec2::new(0.0, 0.0)
    },
    Block {
        name: "Grass Block",
        dev_name: "grass_block",
        block_type: BlockType::Solid,
        top_uv: glm::Vec2::new(0.0, 0.0),
        side_uv: glm::Vec2::new(0.1, 0.0),
        bottom_uv: glm::Vec2::new(0.2, 0.0)
    },
];

pub fn get_block(dev_name: &str) -> Option<&'static Block> {
    BLOCKS.iter().find(|b| b.dev_name == dev_name)
}

pub fn get_block_id(dev_name: &str) -> Option<usize> {
    BLOCKS.iter().position(|b| b.dev_name == dev_name)
}
