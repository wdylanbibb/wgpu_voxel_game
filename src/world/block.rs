#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasTile {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockTextures {
    pub top: AtlasTile,
    pub side: AtlasTile,
    pub bottom: AtlasTile,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BlockId {
    #[default]
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Sand = 4,
    Snow = 5,
}

pub struct BlockDefinition {
    pub solid: bool,
    pub opaque: bool,
    pub textures: Option<BlockTextures>,
}

impl BlockId {
    pub const fn definition(self) -> BlockDefinition {
        match self {
            Self::Air => BlockDefinition {
                solid: false,
                opaque: false,
                textures: None,
            },

            Self::Grass => BlockDefinition {
                solid: true,
                opaque: true,
                textures: Some(BlockTextures {
                    top: AtlasTile { x: 0, y: 0 },
                    side: AtlasTile { x: 1, y: 0 },
                    bottom: AtlasTile { x: 2, y: 0 },
                }),
            },

            Self::Dirt => BlockDefinition {
                solid: true,
                opaque: true,
                textures: Some(BlockTextures {
                    top: AtlasTile { x: 2, y: 0 },
                    side: AtlasTile { x: 2, y: 0 },
                    bottom: AtlasTile { x: 2, y: 0 },
                }),
            },

            Self::Stone => BlockDefinition {
                solid: true,
                opaque: true,
                textures: Some(BlockTextures {
                    top: AtlasTile { x: 3, y: 0 },
                    side: AtlasTile { x: 3, y: 0 },
                    bottom: AtlasTile { x: 3, y: 0 },
                }),
            },

            Self::Sand => BlockDefinition {
                solid: true,
                opaque: true,
                textures: Some(BlockTextures {
                    top: AtlasTile { x: 4, y: 0 },
                    side: AtlasTile { x: 4, y: 0 },
                    bottom: AtlasTile { x: 4, y: 0 },
                }),
            },

            Self::Snow => BlockDefinition {
                solid: true,
                opaque: true,
                textures: Some(BlockTextures {
                    top: AtlasTile { x: 5, y: 0 },
                    side: AtlasTile { x: 6, y: 0 },
                    bottom: AtlasTile { x: 2, y: 0 },
                }),
            },
        }
    }

    pub const fn solid(self) -> bool {
        self.definition().solid
    }

    pub const fn opaque(self) -> bool {
        self.definition().opaque
    }

    pub const fn textures(self) -> Option<BlockTextures> {
        self.definition().textures
    }
}
