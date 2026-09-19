use std::{
    collections::{HashMap, HashSet},
};


use crate::world::{block::BlockId, chunk::{CHUNK_SIZE, Chunk}};

pub mod block;
pub mod chunk;
pub mod meshing;
pub mod generation;

pub struct World {
    chunks: HashMap<cgmath::Vector3<i32>, Chunk>,
    dirty_meshes: HashSet<cgmath::Vector3<i32>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            dirty_meshes: HashSet::new(),
        }
    }

    pub fn from_chunks(chunks: HashMap<cgmath::Vector3<i32>, Chunk>) -> Self {
        let dirty_meshes = chunks.keys().copied().collect();
        Self {
            chunks,
            dirty_meshes,
        }
    }

    pub fn contains_chunk(&self, chunk_pos: cgmath::Vector3<i32>) -> bool {
        self.chunks.contains_key(&chunk_pos)
    }

    pub fn take_dirty_meshes(&mut self) -> Vec<cgmath::Vector3<i32>> {
        self.dirty_meshes.drain().collect()
    }

    pub fn set_block(&mut self, world_pos: cgmath::Vector3<i32>, block: BlockId) -> bool {
        let size = CHUNK_SIZE as i32;

        let chunk_pos = cgmath::Vector3::new(world_pos.x.div_euclid(size), world_pos.y.div_euclid(size), world_pos.z.div_euclid(size));

        let local_pos = [
            world_pos.x.rem_euclid(size) as usize,
            world_pos.y.rem_euclid(size) as usize,
            world_pos.z.rem_euclid(size) as usize,
        ];

        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
            return false;
        };

        let Some(old_block) = chunk.set(local_pos[0], local_pos[1], local_pos[2], block) else {
            return false;
        };

        self.dirty_meshes.insert(chunk_pos);

        if old_block.definition().opaque != block.definition().opaque {
            self.mark_boundary_neighbors_dirty(chunk_pos, local_pos);
        }

        true
    }

    fn mark_boundary_neighbors_dirty(&mut self, chunk_pos: cgmath::Vector3<i32>, local: [usize; 3]) {
        let last = CHUNK_SIZE - 1;

        let neighbors = [
            (local[0] == 0, cgmath::Vector3::new(-1, 0, 0)),
            (local[0] == last, cgmath::Vector3::new(1, 0, 0)),
            (local[1] == 0, cgmath::Vector3::new(0, -1, 0)),
            (local[1] == last, cgmath::Vector3::new(0, 1, 0)),
            (local[2] == 0, cgmath::Vector3::new(0, 0, -1)),
            (local[2] == last, cgmath::Vector3::new(0, 0, 1)),
        ];

        for (on_boundary, offset) in neighbors {
            let neighbor = chunk_pos + offset;

            if on_boundary && self.chunks.contains_key(&neighbor) {
                self.dirty_meshes.insert(neighbor);
            }
        }
    }
}
