use crate::world::{ChunkPos, ChunkType, TileSetReason};
use mvutils::Savable;
use crate::world::chunk::{ToClientChunk, ToClientObject};
use crate::world::tiles::Orientation;
use crate::world::tiles::pos::TilePos;
use crate::world::tiles::tiles::TileType;

#[derive(Savable, Clone)]
pub struct TileSetPacket {
    pub pos: TilePos,
    pub tile: ToClientObject,
    pub reason: TileSetReason
}

#[derive(Savable, Clone)]
pub struct TerrainSetPacket {
    pub pos: TilePos,
    pub tile: ToClientObject,
    pub reason: TileSetReason
}


#[derive(Clone, Savable)]
pub struct TileSetFromClientPacket {
    pub pos: TilePos,
    pub tile_id: usize,
    pub tile_state: Vec<u8>,
    pub orientation: Orientation
}

#[derive(Savable, Clone)]
pub struct ChunkDataPacket {
    pub pos: ChunkPos,
    pub data: ToClientChunk
}

#[derive(Savable, Clone)]
pub struct ChunkUnloadPacket {
    pub pos: ChunkPos,
}