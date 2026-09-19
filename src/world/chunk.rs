use crate::world::block::BlockId;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub struct Chunk {
    blocks: Box<[BlockId; CHUNK_VOLUME]>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: Box::new([BlockId::Air; CHUNK_VOLUME]),
        }
    }
    pub fn from_blocks(blocks: Box<[BlockId; CHUNK_VOLUME]>) -> Self {
        Self { blocks }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        self.blocks[index(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) -> Option<BlockId> {
        let index = index(x, y, z);
        let old_block = self.blocks[index];

        if block == old_block {
            return None;
        }

        self.blocks[index] = block;
        Some(old_block)
    }
}

fn index(x: usize, y: usize, z: usize) -> usize {
    x + CHUNK_SIZE * (z + CHUNK_SIZE * y)
}
