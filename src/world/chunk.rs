use crate::multitile::MultiTilePlacement;
use crate::registry::terrain::TERRAIN_REGISTRY;
use crate::registry::{GameObjects, ObjectSource};
use crate::world::generate::ChunkGenerator;
use crate::world::tiles::pos::TilePos;
use crate::world::tiles::tiles::TileType;
use crate::world::tiles::Orientation;
use crate::world::{tiles, ChunkPos, CHUNK_SIZE};
use abi_stable::std_types::{RHashMap, Tuple2};
use mvutils::save::custom::ignore_save;
use mvutils::save::{Loader, Savable};
use mvutils::Savable;

pub const CHUNK_TILES: usize = CHUNK_SIZE as usize * CHUNK_SIZE as usize;

#[derive(Clone, Savable)]
#[repr(C)]
pub struct Chunk {
    pub seed: u32,
    pub position: ChunkPos,
    pub tiles: Box<[Option<TileType>; CHUNK_TILES]>,
    pub terrain: TerrainLayer,
    pub multitiles: Vec<MultiTilePlacement>,
}

impl Chunk {
    pub fn new(pos: ChunkPos, seed: u32) -> Self {
        Self {
            seed,
            position: pos,
            tiles: Box::new([0; CHUNK_TILES].map(|_| None)),
            terrain: TerrainLayer::new(),
            multitiles: vec![],
        }
    }

    pub fn get_index(pos: &TilePos) -> usize {
        pos.in_chunk_x + pos.in_chunk_z * CHUNK_SIZE as usize
    }

    pub fn get_index_economy_edition(x: i32, z: i32) -> usize {
        (x + z * CHUNK_SIZE) as usize
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

    pub fn iter_tiles(&self) -> impl Iterator<Item=(TileType, TilePos)> + use<'_> {
        pub struct Iter<'a> {
            index: usize,
            tiles: &'a[Option<TileType>],
            pos: ChunkPos
        }

        impl<'a> Iterator for Iter<'a> {
            type Item = (TileType, TilePos);

            fn next(&mut self) -> Option<Self::Item> {
                if self.index >= CHUNK_TILES {
                    None
                } else {
                    loop {
                        if self.index >= CHUNK_TILES {
                            return None;
                        }
                        if let Some(tile) = self.tiles[self.index].clone() {
                            let pos = Chunk::position_from_index(&self.pos, self.index);
                            self.index += 1;
                            return Some((tile, pos));
                        }
                        self.index += 1;
                    }
                }
            }
        }

        Iter {
            index: 0,
            tiles: self.tiles.as_slice(),
            pos: self.position
        }
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
                        state: vec![]
                    }
                } else {
                    ToClientObject {
                        id: 0,
                        orientation: o,
                        source: ObjectSource::Vanilla,
                        state: vec![]
                    }
                }
            })
            .collect::<Vec<_>>();

        let tiles = (0..CHUNK_TILES)
            .map(|i| {
                self.tiles[i].as_ref().map(|t| {
                    tiles::tile_to_client(t)
                })
            })
            .collect::<Vec<_>>();

        ToClientChunk {
            terrain,
            tiles,
            multitiles: self.multitiles.clone(),
        }
    }
    
    pub fn generate(&mut self, generator: &impl ChunkGenerator, objects: &GameObjects) {
        generator.generate(self, objects);
    }

    pub fn generate_terrain(&mut self, generator: &impl ChunkGenerator, objects: &GameObjects) {
        generator.generate_terrain(self, objects);
    }
}

#[derive(Clone, Savable)]
#[repr(C)]
pub struct TerrainLayer {
    #[custom(save = ignore_save, load = empty_terrain)]
    pub terrain: Box<[u16; CHUNK_TILES]>,
    #[custom(save = ignore_save, load = empty_orientation)]
    pub orientation: Box<[Orientation; CHUNK_TILES]>,
    pub mods: RHashMap<u16, u16>,
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
            mods: RHashMap::new(),
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
        for Tuple2(pos, material) in self.mods.iter() {
            let (x, z) = Self::unmap(*pos);
            self.terrain[x as usize + z as usize * CHUNK_SIZE as usize] = material.clone();
        }
    }
}

#[derive(Clone, Savable)]
pub struct ToClientChunk {
    pub terrain: Vec<ToClientObject>,
    pub tiles: Vec<Option<ToClientObject>>,
    pub multitiles: Vec<MultiTilePlacement>,
}

#[derive(Clone, Savable)]
pub struct ToClientObject {
    pub id: u16,
    pub source: ObjectSource,
    pub orientation: Orientation,
    pub state: Vec<u8>
}