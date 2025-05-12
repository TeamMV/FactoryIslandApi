mod noise;

use crate::registry::terrain::TERRAIN_REGISTRY;
use crate::registry::tiles::TILE_REGISTRY;
use crate::registry::GameObjects;
use crate::world::chunk::Chunk;
use crate::world::generate::noise::GeneratorNoise;
use crate::world::tiles::Orientation;
use crate::world::CHUNK_SIZE;
use mvutils::utils::{Map, MapTo};

pub trait ChunkGenerator {
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
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {                
                let orientation = rand::random_range(0..4);
                let orientation = match orientation {
                    0 => Orientation::North,
                    1 => Orientation::South,
                    2 => Orientation::West,
                    3 => Orientation::East,
                    _ => unreachable!()
                };

                if rand::random_bool(0.2) {
                    if let Some(mut tile) = TILE_REGISTRY.create_object(game_objects.tiles.wood) {
                        tile.info.orientation = orientation;
                        chunk.set_tile(x, z, tile.to_type());
                    }
                }
            }
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

                let orientation = rand::random_range(0..4);
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

