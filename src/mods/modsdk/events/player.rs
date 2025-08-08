use crate::mods::modsdk::player::{MPlayer, MPlayerData};
use crate::mods::modsdk::world::MTileUnit;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PlayerJoinEvent {
    pub player: MPlayer
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PlayerLeaveEvent {
    pub player: MPlayer,
    pub data: MPlayerData
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PlayerMoveEvent {
    pub player: MPlayer,
    pub position: MTileUnit
}