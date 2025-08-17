mod noise;

use crate::registry::terrain::TERRAIN_REGISTRY;
use crate::registry::GameObjects;
use crate::world::chunk::Chunk;
use crate::world::generate::noise::GeneratorNoise;
use crate::world::tiles::Orientation;
use crate::world::CHUNK_SIZE;
use mvutils::utils::{Map, MapTo};
use parking_lot::lock_api::RwLock;
use crate::registry::tiles::TILE_REGISTRY;
use crate::world::tiles::tiles::TileType;

pub trait ChunkGenerator {
    fn gen_orientation(seed: u32, x: i32, z: i32) -> u8 {
        let mut hash = seed;
        let x = x as u32;
        let z = z as u32;

        hash = hash.wrapping_add(x);
        hash = hash.wrapping_mul(1610612741);
        hash = hash.wrapping_add(z);
        hash = hash.wrapping_mul(805306457);

        hash ^= hash >> 16;
        hash = hash.wrapping_mul(937412447);
        hash ^= hash >> 13;
        hash = hash.wrapping_mul(1385293057);
        hash ^= hash >> 16;

        (hash & 3) as u8
    }

    fn generate(&self, chunk: &mut Chunk, game_objects: &GameObjects);
    fn generate_terrain(&self, chunk: &mut Chunk, game_objects: &GameObjects);
}

pub struct GeneratePipeline {
    noise: GeneratorNoise
}

impl GeneratePipeline {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: GeneratorNoise::new(0.1, seed),
        }
    }
}

impl ChunkGenerator for GeneratePipeline {
    fn generate(&self, chunk: &mut Chunk, game_objects: &GameObjects) {
        if chunk.position == (0, 0) {
            let tile = TILE_REGISTRY.create_object(game_objects.tiles.lamp).unwrap();
            chunk.set_tile(0, 0, TileType::new(RwLock::new(tile)));
        }
    }

    fn generate_terrain(&self, chunk: &mut Chunk, game_objects: &GameObjects) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let noise_x = x + chunk.position.0 * CHUNK_SIZE;
                let noise_z = z + chunk.position.1 * CHUNK_SIZE;
                let noise_val = self.noise.get_for_tile(noise_x, noise_z);
                let mapped = noise_val.map_to(TERRAIN_REGISTRY.len() as u32 - 2);
                let id = mapped + 1;

                let orientation = Self::gen_orientation(chunk.seed, x, z);
                let orientation = match orientation {
                    0 => Orientation::North,
                    1 => Orientation::South,
                    2 => Orientation::West,
                    3 => Orientation::East,
                    _ => unreachable!()
                };
                chunk.set_terrain_in_generate(x, z, id as u16, orientation);
            }
        }
    }
}
