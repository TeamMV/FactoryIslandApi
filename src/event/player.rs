use mvengine::net::DisconnectReason;
use crate::player::PlayerType;
use crate::world::TileUnit;

pub struct PlayerJoinEvent {
    pub player: PlayerType
}

pub struct PlayerLeaveEvent {
    pub player: PlayerType,
    pub reason: DisconnectReason
}

pub struct PlayerMoveEvent {
    pub has_been_cancelled: bool,
    pub pos: TileUnit,
    pub player: PlayerType
}