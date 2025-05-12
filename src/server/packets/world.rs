use crate::world::{ChunkPos, ChunkType, TileSetReason};
use mvutils::Savable;
use crate::world::chunk::ToClientChunk;
use crate::world::tiles::tiles::TileType;

#[derive(Savable, Clone)]
pub struct TileSetPacket {
    pub tile: TileType,
    pub reason: TileSetReason
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