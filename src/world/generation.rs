use cgmath::{ElementWise, Vector3};
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::world::{
    block::BlockId,
    chunk::{CHUNK_SIZE, Chunk},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Desert,
    Tundra,
}

#[derive(Debug, Clone, Copy)]
pub struct TerrainConfig {
    pub base_height: i32,
    pub height_amplitude: i32,
    pub soil_depth: i32,
}

pub struct TerrainGenerator {
    seed: u32,
    config: TerrainConfig,
    height_noise: noise::Fbm<noise::Perlin>,
    temperature_noise: noise::Fbm<noise::Perlin>,
    moisture_noise: noise::Fbm<noise::Perlin>,
}

impl TerrainGenerator {
    pub fn new(seed: u32, config: TerrainConfig) -> Self {
        let height_noise = Fbm::<Perlin>::new(seed)
            .set_octaves(5)
            .set_frequency(0.008)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        let temperature_noise = Fbm::<Perlin>::new(seed.wrapping_add(0x9E37_79B9))
            .set_octaves(3)
            .set_frequency(0.010);

        let moisture_noise = Fbm::<Perlin>::new(seed.wrapping_add(0x85EB_CA6B))
            .set_octaves(3)
            .set_frequency(0.010);

        Self {
            seed,
            config,
            height_noise,
            temperature_noise,
            moisture_noise,
        }
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn surface_height(&self, world_x: i32, world_z: i32) -> i32 {
        let noise = self
            .height_noise
            .get([world_x as f64, world_z as f64])
            .clamp(-1.0, 1.0);

        let offset = (noise * self.config.height_amplitude as f64).round() as i32;

        self.config.base_height + offset
    }

    pub fn temperature(&self, world_x: i32, world_z: i32) -> f64 {
        self.temperature_noise
            .get([world_x as f64, world_z as f64])
            .clamp(-1.0, 1.0)
    }

    pub fn moisture(&self, world_x: i32, world_z: i32) -> f64 {
        self.moisture_noise
            .get([world_x as f64, world_z as f64])
            .clamp(-1.0, 1.0)
    }

    pub fn biome_at(&self, world_x: i32, world_z: i32) -> Biome {
        let temperature = self.temperature(world_x, world_z);

        let moisture = self.moisture(world_x, world_z);

        if temperature < -0.15 {
            Biome::Tundra
        } else if moisture < 0.0 {
            Biome::Desert
        } else {
            Biome::Plains
        }
    }

    pub fn generate_chunk(&self, chunk_pos: Vector3<i32>) -> Chunk {
        let mut chunk = Chunk::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let world = chunk_pos
                        .mul_element_wise(16)
                        .add_element_wise(Vector3::new(x as i32, y as i32, z as i32));

                    let surface = self.surface_height(world.x, world.z);

                    let block = if world.y > surface {
                        BlockId::Air
                    } else if world.y == surface {
                        match self.biome_at(world.x, world.z) {
                            Biome::Plains => BlockId::Grass,
                            Biome::Desert => BlockId::Sand,
                            Biome::Tundra => BlockId::Snow,
                        }
                    } else if world.y >= self.config.soil_depth {
                        match self.biome_at(world.x, world.z) {
                            Biome::Plains | Biome::Tundra => BlockId::Dirt,
                            Biome::Desert => BlockId::Sand,
                        }
                    } else {
                        BlockId::Stone
                    };

                    chunk.set(x, y, z, block);
                }
            }
        }

        chunk
    }
}
