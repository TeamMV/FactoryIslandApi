use player::{PlayerJoinEvent, PlayerLeaveEvent};
use crate::event::common::{ServerCommandEvent, ServerTickEvent};
use crate::event::player::PlayerMoveEvent;
use crate::event::world::{BeforeChunkGenerateEvent, BeforeChunkGenerateTerrainEvent, TileSetEvent};

pub mod world;
pub mod common;
pub mod player;

pub enum Event {
    GameStartEvent(GameStartEvent),
    GameEndEvent(GameEndEvent),
    PlayerJoinEvent(PlayerJoinEvent),
    PlayerLeaveEvent(PlayerLeaveEvent),
    ServerTickEvent(ServerTickEvent),
    TileSetEvent(TileSetEvent),
    PlayerMoveEvent(PlayerMoveEvent),
    ServerCommandEvent(ServerCommandEvent),
    BeforeChunkGenerateEvent(BeforeChunkGenerateEvent),
    BeforeChunkGenerateTerrainEvent(BeforeChunkGenerateTerrainEvent)
}

pub struct GameStartEvent;
pub struct GameEndEvent;