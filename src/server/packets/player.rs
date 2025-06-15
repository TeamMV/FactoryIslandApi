use mvengine::net::server::ClientId;
use mvutils::Savable;
use crate::server::packets::common::{ClientDataPacket, PlayerData};
use crate::world::TileUnit;

#[derive(Clone, Savable)]
pub struct PlayerMovePacket {
    pub pos: TileUnit
}

#[derive(Clone, Savable)]
pub struct OtherPlayerMovePacket {
    pub client_id: ClientId,
    pub pos: TileUnit
}

#[derive(Clone, Savable)]
pub struct OtherPlayerJoinPacket {
    pub client_id: ClientId,
    pub client_data: ClientDataPacket
}

#[derive(Clone, Savable)]
pub struct OtherPlayerLeavePacket {
    pub client_id: ClientId,
}

#[derive(Clone, Savable)]
pub struct PlayerChatPacket {
    pub message: String,
}

#[derive(Clone, Savable)]
pub struct OtherPlayerChatPacket {
    pub player: PlayerData,
    pub message: String,
}