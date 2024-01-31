pub type BlockID = usize;

#[derive(Debug, Clone, Copy)]
pub enum Block<'a> {
    Air,
    Solid {
        name: &'a str,
        dev_name: &'a str,
        uv: glm::Vec2
    }
}

impl<'a> Block<'a> {
    pub const fn new_air() -> Block<'a> {
        Block::Air
    }

    pub const fn new_solid(name: &'a str, dev_name: &'a str, uv: glm::Vec2) -> Block<'a> {
        Block::Solid { name, dev_name, uv }
    }

    pub const fn is_solid(&self) -> bool {
        match self {
            Block::Air => false,
            Block::Solid { .. } => true
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            Block::Air => "Air",
            Block::Solid { name, .. } => name
        }
    }

    pub fn get_dev_name(&self) -> &str {
        match self {
            Block::Air => "air",
            Block::Solid { dev_name, .. } => dev_name
        }
    }

    pub fn get_uv(&self) -> glm::Vec2 {
        match self {
            Block::Air => glm::zero(),
            Block::Solid { uv, .. } => uv.clone()
        }
    }
}
