use crate::world::{ChunkPos, ChunkType, TileSetReason};
use mvutils::Savable;
use crate::multitile::MultiTilePlacement;
use crate::player::uuid::UUID;
use crate::world::chunk::{ToClientChunk, ToClientObject};
use crate::world::tiles::{Orientation, TileKind};
use crate::world::tiles::pos::TilePos;

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
    pub tile_id: TileKind,
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

#[derive(Savable, Clone)]
pub struct MultiTilePlacedPacket {
    pub placement: MultiTilePlacement
}

#[derive(Savable, Clone)]
pub struct MultiTileDestroyedPacket {
    pub placement_id: UUID,
    pub chunk_pos: ChunkPos,
}