use crate::registry::terrain::TERRAIN_REGISTRY;
use crate::world::tiles::pos::TilePos;
use crate::world::tiles::tiles::TileType;
use crate::world::tiles::{ObjectSource, Orientation};
use crate::world::{ChunkPos, CHUNK_SIZE};
use mvutils::save::custom::ignore_save;
use mvutils::save::{Loader, Savable};
use mvutils::{enum_val, Savable};
use std::collections::HashMap;
use mvengine::event::EventBus;
use crate::event::Event;
use crate::event::world::{BeforeChunkGenerateEvent, BeforeChunkGenerateTerrainEvent};
use crate::registry::GameObjects;
use crate::world::generate::{ChunkGenerator, GeneratePipeline};

pub const CHUNK_TILES: usize = CHUNK_SIZE as usize * CHUNK_SIZE as usize;

#[derive(Clone, Savable)]
pub struct Chunk {
    pub seed: u32,
    pub position: ChunkPos,
    pub tiles: Box<[Option<TileType>; CHUNK_TILES]>,
    pub terrain: TerrainLayer,
}

impl Chunk {
    pub fn new(pos: ChunkPos, seed: u32) -> Self {
        Self {
            seed,
            position: pos,
            tiles: Box::new([0; CHUNK_TILES].map(|_| None)),
            terrain: TerrainLayer::new(),
        }
    }

    pub fn get_index(pos: &TilePos) -> usize {
        pos.in_chunk_x + pos.in_chunk_z * CHUNK_SIZE as usize
    }

    pub fn position_from_index(chunk_pos: &ChunkPos, index: usize) -> TilePos {
        let x = index as i32 % CHUNK_SIZE;
        let y = index as i32 / CHUNK_SIZE;
        (chunk_pos.0 * CHUNK_SIZE + x, chunk_pos.1 * CHUNK_SIZE + y).into()
    }

    pub fn set_terrain(&mut self, x: i32, z: i32, tile: u16, orientation: Orientation) {
        self.terrain.set_terrain(x, z, tile, orientation);
    }

    pub fn set_terrain_in_generate(&mut self, x: i32, z: i32, tile: u16, orientation: Orientation) {
        self.terrain.generate_terrain(x, z, tile, orientation);
    }

    pub fn set_tile(&mut self, x: i32, z: i32, tile: TileType) {
        let idx = x + z * CHUNK_SIZE;
        self.tiles[idx as usize] = Some(tile);
    }

    pub(crate) fn to_client(&self) -> ToClientChunk {
        let terrain = (0..CHUNK_TILES)
            .map(|i| {
                let id = self.terrain.terrain[i];
                let o = self.terrain.orientation[i];
                if let Some(tile) = TERRAIN_REGISTRY.create_object(id as usize) {
                    ToClientObject {
                        id,
                        orientation: o,
                        source: tile.info.source.clone(),
                    }
                } else {
                    ToClientObject {
                        id: 0,
                        orientation: o,
                        source: ObjectSource::Vanilla,
                    }
                }
            })
            .collect::<Vec<_>>();

        let tiles = (0..CHUNK_TILES)
            .map(|i| {
                self.tiles[i].as_ref().map(|t| {
                    let rw = t.read();
                    ToClientObject {
                        id: rw.id as u16,
                        orientation: rw.info.orientation,
                        source: rw.info.source.clone(),
                    }
                })
            })
            .collect::<Vec<_>>();

        ToClientChunk {
            terrain,
            tiles,
        }
    }
    
    pub fn generate(mut self, generator: &impl ChunkGenerator, event_bus: &mut EventBus<Event>, objects: &GameObjects) -> Self {
        let event = BeforeChunkGenerateEvent {
            has_been_cancelled: false,
            pos: self.position,
            chunk: self,
        };
        let mut event = Event::BeforeChunkGenerateEvent(event);
        event_bus.dispatch(&mut event);

        let mut event = enum_val!(Event, event, BeforeChunkGenerateEvent);
        if !event.has_been_cancelled {
            generator.generate(&mut event.chunk, objects);
        }
        
        event.chunk
    }

    pub fn generate_terrain(mut self, generator: &impl ChunkGenerator, event_bus: &mut EventBus<Event>, objects: &GameObjects) -> Self {
        let event = BeforeChunkGenerateTerrainEvent {
            has_been_cancelled: false,
            pos: self.position,
            chunk: self,
        };
        let mut event = Event::BeforeChunkGenerateTerrainEvent(event);
        event_bus.dispatch(&mut event);

        let mut event = enum_val!(Event, event, BeforeChunkGenerateTerrainEvent);
        if !event.has_been_cancelled {
            generator.generate_terrain(&mut event.chunk, objects);
        }

        event.chunk
    }
}

#[derive(Clone, Savable)]
pub struct TerrainLayer {
    #[custom(save = ignore_save, load = empty_terrain)]
    pub terrain: Box<[u16; CHUNK_TILES]>,
    #[custom(save = ignore_save, load = empty_orientation)]
    pub orientation: Box<[Orientation; CHUNK_TILES]>,
    pub mods: HashMap<u16, u16>,
}

fn empty_terrain(_: &mut impl Loader) -> Result<Box<[u16; CHUNK_TILES]>, String> {
    let array: [u16; CHUNK_TILES] = [0; CHUNK_TILES];
    Ok(Box::new(array))
}

fn empty_orientation(_: &mut impl Loader) -> Result<Box<[Orientation; CHUNK_TILES]>, String> {
    let array: [Orientation; CHUNK_TILES] = [Orientation::North; CHUNK_TILES];
    Ok(Box::new(array))
}

impl TerrainLayer {
    pub fn new() -> Self {
        let terrain_array: [u16; CHUNK_TILES] = [0; CHUNK_TILES];
        let orientation_array: [Orientation; CHUNK_TILES] = [Orientation::North; CHUNK_TILES];

        Self {
            terrain: Box::new(terrain_array),
            orientation: Box::new(orientation_array),
            mods: HashMap::new(),
        }
    }

    fn map(x: u8, z: u8) -> u16 {
        u16::from_be_bytes([x, z])
    }

    fn unmap(id: u16) -> (u8, u8) {
        let bytes = id.to_be_bytes();
        (bytes[0], bytes[1])
    }

    pub fn is_original(&self, x: u8, z: u8) -> bool {
        !self.mods.contains_key(&Self::map(x, z))
    }

    pub fn get_tile_at(&self, x: u8, z: u8) -> u16 {
        self.terrain[x as usize + z as usize * CHUNK_SIZE as usize]
    }

    pub fn set_terrain(&mut self, x: i32, z: i32, tile: u16, orientation: Orientation) {
        let idx = (x + z * CHUNK_SIZE) as usize;
        self.orientation[idx] = orientation;
        self.terrain[idx] = tile;
        self.mods.insert(Self::map(x as u8, z as u8), tile);
    }

    pub fn generate_terrain(&mut self, x: i32, z: i32, tile: u16, orientation: Orientation) {
        let idx = (x + z * CHUNK_SIZE) as usize;
        self.orientation[idx] = orientation;
        self.terrain[idx] = tile;
    }

    pub fn apply_modifications(&mut self) {
        for (pos, material) in &self.mods {
            let (x, z) = Self::unmap(*pos);
            self.terrain[x as usize + z as usize * CHUNK_SIZE as usize] = material.clone();
        }
    }
}

#[derive(Clone, Savable)]
pub struct ToClientChunk {
    pub terrain: Vec<ToClientObject>,
    pub tiles: Vec<Option<ToClientObject>>,
}

#[derive(Clone, Savable)]
pub struct ToClientObject {
    pub id: u16,
    pub source: ObjectSource,
    pub orientation: Orientation
}