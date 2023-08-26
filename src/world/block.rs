#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Block {
    pub name: String,
    pub dev_name: String,
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

impl Block {
    pub fn new(name: impl ToString, dev_name: impl ToString, block_type: BlockType, top_uv: glm::Vec2, side_uv: glm::Vec2, bottom_uv: glm::Vec2) -> Block {
        Block {
            name: name.to_string(),
            dev_name: dev_name.to_string(),
            block_type,
            top_uv,
            side_uv,
            bottom_uv
        }
    }
}
