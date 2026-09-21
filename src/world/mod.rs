use std::{
    collections::{HashMap, HashSet},
    range::Range,
};

use crate::world::{
    block::BlockId,
    chunk::{CHUNK_SIZE, Chunk},
};

pub mod block;
pub mod chunk;
pub mod generation;
pub mod meshing;

pub struct World {
    chunks: HashMap<cgmath::Vector3<i32>, Chunk>,
    dirty_meshes: HashSet<cgmath::Vector3<i32>>,
}

impl World {
    pub fn new(dimensions: cgmath::Vector3<Range<i32>>) -> Self {
        let mut chunks = HashMap::new();

        let generator = generation::TerrainGenerator::new(
            1337,
            generation::TerrainConfig {
                base_height: 4,
                height_amplitude: 12,
                soil_depth: 3,
            },
        );

        for x in dimensions.x {
            for y in dimensions.y {
                for z in dimensions.z {
                    let chunk_pos = cgmath::Vector3::new(x, y, z);

                    let chunk = generator.generate_chunk(chunk_pos);

                    chunks.insert(chunk_pos, chunk);
                }
            }
        }

        World::from_chunks(chunks)
    }

    pub fn from_chunks(chunks: HashMap<cgmath::Vector3<i32>, Chunk>) -> Self {
        let dirty_meshes = chunks.keys().copied().collect();
        Self {
            chunks,
            dirty_meshes,
        }
    }

    pub fn spawn_point(&self, world_x: i32, world_z: i32) -> Option<cgmath::Vector3<f32>> {
        let size = CHUNK_SIZE as i32;
        let chunk_x = world_x.div_euclid(size);
        let chunk_z = world_z.div_euclid(size);

        let min_chunk_y = self
            .chunks
            .keys()
            .filter(|pos| pos.x == chunk_x && pos.z == chunk_z)
            .map(|pos| pos.y)
            .min()?;

        let max_chunk_y = self
            .chunks
            .keys()
            .filter(|pos| pos.x == chunk_x && pos.z == chunk_z)
            .map(|pos| pos.y)
            .max()?;

        let min_y = min_chunk_y * size;
        let max_y = (max_chunk_y + 1) * size - 1;

        for y in (min_y..=max_y).rev() {
            let ground = cgmath::Vector3::new(world_x, y, world_z);
            let feet = cgmath::Vector3::new(world_x, y + 1, world_z);
            let head = cgmath::Vector3::new(world_x, y + 2, world_z);

            if self.is_solid(ground) && !self.is_solid(feet) && !self.is_solid(head) {
                return Some(cgmath::Vector3::new(
                    world_x as f32 + 0.5,
                    y as f32 + 1.001,
                    world_z as f32 + 0.5,
                ));
            }
        }

        None
    }

    pub fn contains_chunk(&self, chunk_pos: cgmath::Vector3<i32>) -> bool {
        self.chunks.contains_key(&chunk_pos)
    }

    pub fn take_dirty_meshes(&mut self) -> Vec<cgmath::Vector3<i32>> {
        self.dirty_meshes.drain().collect()
    }

    pub fn set_block(&mut self, world_pos: cgmath::Vector3<i32>, block: BlockId) -> bool {
        let size = CHUNK_SIZE as i32;

        let chunk_pos = cgmath::Vector3::new(
            world_pos.x.div_euclid(size),
            world_pos.y.div_euclid(size),
            world_pos.z.div_euclid(size),
        );

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

    fn mark_boundary_neighbors_dirty(
        &mut self,
        chunk_pos: cgmath::Vector3<i32>,
        local: [usize; 3],
    ) {
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

    pub fn get_block(&self, world_pos: cgmath::Vector3<i32>) -> Option<BlockId> {
        let size = CHUNK_SIZE as i32;

        let chunk_pos = cgmath::Vector3::new(
            world_pos.x.div_euclid(size),
            world_pos.y.div_euclid(size),
            world_pos.z.div_euclid(size),
        );

        let local_pos = [
            world_pos.x.rem_euclid(size) as usize,
            world_pos.y.rem_euclid(size) as usize,
            world_pos.z.rem_euclid(size) as usize,
        ];

        Some(
            self.chunks
                .get(&chunk_pos)?
                .get(local_pos[0], local_pos[1], local_pos[2]),
        )
    }

    pub fn is_solid(&self, world_pos: cgmath::Vector3<i32>) -> bool {
        self.get_block(world_pos).map_or(true, BlockId::solid)
    }
}
