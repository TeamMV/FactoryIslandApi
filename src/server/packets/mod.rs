pub mod common;
pub mod world;
pub mod player;

use mvutils::Savable;
use crate::server::packets::common::{ClientDataPacket, ServerStatePacket};
use crate::server::packets::player::{OtherPlayerChatPacket, OtherPlayerJoinPacket, OtherPlayerLeavePacket, OtherPlayerMovePacket, PlayerChatPacket, PlayerDataPacket, PlayerMovePacket};
use crate::server::packets::world::{ChunkDataPacket, ChunkUnloadPacket, MultiTileDestroyedPacket, MultiTilePlacedPacket, TerrainSetPacket, TileSetFromClientPacket, TileSetPacket};

#[derive(Savable, Clone)]
pub enum ClientBoundPacket {
    TileSet(TileSetPacket),
    TerrainSet(TerrainSetPacket),
    ChunkData(ChunkDataPacket),
    PlayerMove(PlayerMovePacket),
    OtherPlayerMove(OtherPlayerMovePacket),
    OtherPlayerJoin(OtherPlayerJoinPacket),
    OtherPlayerLeave(OtherPlayerLeavePacket),
    ServerState(ServerStatePacket),
    ChunkUnload(ChunkUnloadPacket),
    OtherPlayerChat(OtherPlayerChatPacket),
    PlayerDataPacket(PlayerDataPacket),
    MultiTilePlacedPacket(MultiTilePlacedPacket),
    MultiTileDestroyedPacket(MultiTileDestroyedPacket)
}

#[derive(Savable, Clone)]
pub enum ServerBoundPacket {
    ClientData(ClientDataPacket),
    PlayerMove(PlayerMovePacket),
    TileSet(TileSetFromClientPacket),
    PlayerChat(PlayerChatPacket),
    RequestReload,
}