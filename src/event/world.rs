use crate::world::chunk::Chunk;
use crate::world::tiles::pos::TilePos;
use crate::world::tiles::tiles::TileType;
use crate::world::{ChunkPos, TileSetReason, World};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct TileSetEvent {
    pub world: Arc<Mutex<World>>,
    pub has_been_cancelled: bool,
    pub tile: TileType,
    pub pos: TilePos,
    pub reason: TileSetReason
}

/// Fired before a new chunk is generated and after the BeforeChunkGenerateTerrainEvent. When cancelled, the chunk can be populated manually from the mod without being overridden.
pub struct BeforeChunkGenerateEvent {
    pub has_been_cancelled: bool,
    pub pos: ChunkPos,
    pub chunk: Chunk
}

/// Fired before a chunk terrain is generated. This happens either when a new chunk is created or the chunk is loaded from memory. When cancelled, the chunk can be populated manually from the mod without being overridden.
pub struct BeforeChunkGenerateTerrainEvent {
    pub has_been_cancelled: bool,
    pub pos: ChunkPos,
    pub chunk: Chunk
}