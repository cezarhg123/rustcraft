#[derive(Debug, Clone, Copy)]
pub struct BlockID(u8);

const SIXTEENTH: f32 = 1.0 / 16.0;
impl BlockID {
    /// Bottom Left
    pub fn get_bl_uv(&self) -> glm::Vec2 {
        glm::vec2(
            (((self.0 - 1) % 16) % 16) as f32 / 16.0,
            (((self.0 - 1) / 16) / 16) as f32 / 16.0 
        )
    }

    /// Bottom Right
    pub fn get_br_uv(&self) -> glm::Vec2 {
        glm::vec2(
            ((((self.0 - 1) % 16) % 16) as f32 / 16.0) + SIXTEENTH,
            (((self.0 - 1) / 16) / 16) as f32 / 16.0 
        )
    }

    /// Top Left
    pub fn get_tl_uv(&self) -> glm::Vec2 {
        glm::vec2(
            (((self.0 - 1) % 16) % 16) as f32 / 16.0,
            ((((self.0 - 1) / 16) / 16) as f32 / 16.0) + SIXTEENTH
        )
    }

    /// Top Right
    pub fn get_tr_uv(&self) -> glm::Vec2 {
        glm::vec2(
            ((((self.0 - 1) % 16) % 16) as f32 / 16.0) + SIXTEENTH,
            ((((self.0 - 1) / 16) / 16) as f32 / 16.0) + SIXTEENTH
        )
    }
}

impl PartialEq<BlockID> for BlockID {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<u8> for BlockID {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block<'a> {
    id: BlockID,
    pub name: &'a str,
    pub dev_name: &'a str,
}

impl<'a> Block<'a> {
    pub const fn new(id: u8, name: &'a str, dev_name: &'a str) -> Block<'a> {
        Block {
            id: BlockID(id),
            name,
            dev_name
        }
    }

    pub const fn block_id(&self) -> BlockID {
        self.id
    }
}
