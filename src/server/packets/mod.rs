pub mod common;
pub mod world;
pub mod player;

use mvutils::Savable;
use crate::server::packets::common::{ClientDataPacket, ServerStatePacket};
use crate::server::packets::player::{OtherPlayerJoinPacket, OtherPlayerLeavePacket, OtherPlayerMovePacket, PlayerMovePacket};
use crate::server::packets::world::{ChunkDataPacket, ChunkUnloadPacket, TileSetPacket};

#[derive(Savable, Clone)]
pub enum ClientBoundPacket {
    TileSet(TileSetPacket),
    ChunkData(ChunkDataPacket),
    PlayerMove(PlayerMovePacket),
    OtherPlayerMove(OtherPlayerMovePacket),
    OtherPlayerJoin(OtherPlayerJoinPacket),
    OtherPlayerLeave(OtherPlayerLeavePacket),
    ServerState(ServerStatePacket),
    ChunkUnload(ChunkUnloadPacket)
}

#[derive(Savable, Clone)]
pub enum ServerBoundPacket {
    ClientData(ClientDataPacket),
    PlayerMove(PlayerMovePacket)
}