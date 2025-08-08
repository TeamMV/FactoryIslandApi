use crate::mods::LOADED_MODS;
use crate::mods::modsdk::{ModCtx, ModData};
use crate::mods::modsdk::events::player::{PlayerJoinEvent, PlayerLeaveEvent, PlayerMoveEvent};
use crate::mods::modsdk::events::world::{BeforeChunkGenerateEvent, BeforeChunkGenerateTerrainEvent, TerrainSetEvent, TileSetEvent};

pub mod common;
pub mod world;
pub mod player;

#[repr(C)]
#[derive(Clone, Debug)]
pub enum Event {
    GameStartEvent,
    GameEndEvent,
    PlayerJoinEvent(PlayerJoinEvent),
    PlayerLeaveEvent(PlayerLeaveEvent),
    ServerTickEvent,
    TileSetEvent(TileSetEvent),
    TerrainSetEvent(TerrainSetEvent),
    PlayerMoveEvent(PlayerMoveEvent),
    BeforeChunkGenerateEvent(BeforeChunkGenerateEvent),
    BeforeChunkGenerateTerrainEvent(BeforeChunkGenerateTerrainEvent)
}

#[repr(C)]
pub enum EventResponse {
    None,
    Changed(Event)
}

pub type EventHandler = fn(Event, ModData) -> EventResponse;

#[no_mangle]
pub extern "C" fn fim_register_event_handler(ctx: ModCtx, handler: EventHandler) {
    let mut reg = LOADED_MODS.write();
    if let Some(loaded) = reg.get_mut(&(&ctx).id) {
        loaded.event_listeners.push(handler);
    }
}

