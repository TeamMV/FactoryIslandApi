use crate::mods::modsdk::world::{MChunk, MTile, MTileSetReason, MWorld};
use crate::world::tiles::pos::TilePos;
use crate::world::ChunkPos;
use crate::world::tiles::terrain::WorldTerrain;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct BeforeChunkGenerateEvent {
    pub has_been_cancelled: bool,
    pub pos: ChunkPos,
    pub chunk: MChunk
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct BeforeChunkGenerateTerrainEvent {
    pub has_been_cancelled: bool,
    pub pos: ChunkPos,
    pub chunk: MChunk
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct TileSetEvent {
    pub has_been_cancelled: bool,
    pub pos: TilePos,
    pub world: MWorld,
    pub tile: MTile,
    pub reason: MTileSetReason
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct TerrainSetEvent {
    pub has_been_cancelled: bool,
    pub pos: TilePos,
    pub world: MWorld,
    pub terrain: WorldTerrain,
    pub reason: MTileSetReason
}
