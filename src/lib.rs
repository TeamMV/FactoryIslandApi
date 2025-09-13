#![feature(map_try_insert)]
#![feature(try_trait_v2)]

use crate::command::{CommandProcessor, CommandSender, COMMAND_PROCESSOR};
use crate::player::{Player, PlayerType};
use crate::registry::terrain::TerrainTiles;
use crate::registry::GameObjects;
use crate::server::packets::common::{ClientDataPacket, PlayerData, ServerStatePacket, TileKind};
use crate::server::packets::player::{OtherPlayerChatPacket, OtherPlayerJoinPacket, OtherPlayerLeavePacket, OtherPlayerMovePacket, PlayerMovePacket};
use crate::server::{ClientBoundPacket, ServerBoundPacket};
use crate::world::{TileSetReason, World, WorldType};
use hashbrown::HashSet;
use log::{debug, error, info, warn};
use mvengine::event::EventBus;
use mvengine::net::server::{ClientEndpoint, ClientId, ServerHandler};
use mvengine::net::DisconnectReason;
use mvengine::utils::args::ParsedArgs;
use mvengine::utils::savers::save_to_vec;
use mvutils::{enum_val, lazy};
use mvutils::hashers::U64IdentityHasher;
use std::collections::HashMap;
use std::ops::Deref;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use std::{env, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use abi_stable::traits::IntoReprC;
use bytebuffer::ByteBuffer;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::clock::Clock;
use mvutils::save::Savable;
use mvutils::unsafe_utils::Unsafe;
use parking_lot::RwLock;
use crate::ingredients::{IngredientKind, IngredientStack};
use crate::packethandler::PacketHandler;
use crate::registry::ingredients::INGREDIENT_REGISTRY;
use crate::registry::tiles::TILE_REGISTRY;
use crate::server::packets::world::TileSetPacket;
use crate::world::chunk::ToClientObject;
use crate::world::tiles::Orientation;
use crate::world::tiles::tiles::TileType;

pub mod world;
pub mod settings;
pub mod player;
pub mod server;
pub mod command;
pub mod registry;
pub mod ingredients;
pub mod multitile;
pub mod packethandler;
pub mod inventory;
pub mod unit;
mod utils;

lazy! {
    pub(crate) static PLAYERS: RwLock<HashMap<ClientId, PlayerType, U64IdentityHasher>> = RwLock::new(HashMap::with_hasher(U64IdentityHasher::default()));
}

pub fn broadcast_all_players(packet: ClientBoundPacket) {
    let players = PLAYERS.read();
    for player in players.values() {
        let lock = player.lock();
        if let Some(endpoint) = lock.client_endpoint() {
            endpoint.send(packet.clone());
        }
    }
}

pub struct FactoryIsland {
    pub(crate) world: WorldType,
    pub(crate) render_distance: i32,
    
    pub objects: GameObjects,
}

impl FactoryIsland {
    pub fn tick(&mut self) {
        let mut loaded_by_player = HashSet::new();
        let mut player_lock = PLAYERS.write();
        for player in player_lock.values() {
            let mut lock = player.lock();
            lock.tick();
            for pos in &lock.loaded_chunks {
                loaded_by_player.insert(*pos);
            }
        }
        drop(player_lock);
        let mut world = self.world.lock();
        world.check_unload(loaded_by_player);
        world.tick();
        
        drop(world);
    }

    pub fn on_command(&mut self, command: String, player: Option<PlayerData>) {
        COMMAND_PROCESSOR.process(player.map_or(CommandSender::Console, |d| CommandSender::Player(d)), command, self);
    }

    pub fn stop(&mut self) {
        exit(0);
    }
    
    pub fn save(&self) {
        let mut world = self.world.lock();
        world.save();
        drop(world);
        let players = PLAYERS.read();
        for player in players.values() {
            let mut locked = player.lock();
            locked.on_disconnect();
        }
    }
}

impl ServerHandler<ServerBoundPacket> for FactoryIsland {
    fn on_server_start(port: u16) -> Self {
        let args = ParsedArgs::parse(env::args());
        
        let terrain_tiles = registry::terrain::register_all();
        let tiles = registry::tiles::register_all();
        let ingredients = registry::ingredients::register_all();
        let multitiles = registry::multitiles::register_all(&tiles);
        command::register_commands();
        
        let objects = GameObjects {
            terrain: terrain_tiles,
            tiles,
            ingredients,
            multitiles,
        };

        let stack = IngredientStack::new(objects.ingredients.stone, 1);
        println!("stone stack: {stack:#?}");

        let world = World::get_main(objects.clone());

        FactoryIsland {
            world,
            render_distance: 1,
            
            objects,
        }
    }

    fn on_client_connect(&mut self, client: Arc<ClientEndpoint>) {
        debug!("client connect call");
        let mut player_data = Vec::new();
        let mut players = PLAYERS.write();
        for (client_id, player) in players.iter() {
            let lock = player.lock();
            let data = PlayerData {
                client_id: *client_id,
                data: lock.data.clone(),
            };
            player_data.push(data);
        }
        let mut tiles = Vec::with_capacity(TILE_REGISTRY.len() - 1);
        for id in 1..TILE_REGISTRY.len() {
            if let Some(object) = TILE_REGISTRY.create_object(id) {
                tiles.push(TileKind {
                    id,
                    source: object.info.source.clone(),
                });
            }
        }
        let mut ingredients = Vec::with_capacity(INGREDIENT_REGISTRY.len() - 1);
        for id in 1..INGREDIENT_REGISTRY.len() {
            if let Some(object) = INGREDIENT_REGISTRY.create_object(id) {
                ingredients.push(id);
            }
        }
        client.send(ClientBoundPacket::ServerState(ServerStatePacket {
            players: player_data,
            tiles,
            ingredients,
            client_id: client.id(),
        }));
        let id = client.id();
        let player = Player::new(client, self.world.clone());
        players.insert(id, player);
    }

    fn on_client_disconnect(&mut self, client: Arc<ClientEndpoint>, reason: DisconnectReason) {
        let mut players = PLAYERS.write();
        if let Some(player) = players.get(&client.id()).cloned() {
            let mut lock = player.lock();

            lock.on_disconnect();
            drop(lock);
            players.remove(&client.id());

            let id = client.id();
            for (_, other_player) in players.iter().filter(|(p, _)| **p != id) {
                let lock = other_player.lock();
                if let Some(endpoint) = lock.client_endpoint() {
                    endpoint.send(ClientBoundPacket::OtherPlayerLeave(OtherPlayerLeavePacket {
                        client_id: id,
                    }));
                }
            }
        }
    }

    fn on_packet(&mut self, client: Arc<ClientEndpoint>, packet: ServerBoundPacket) {
        let mut world_lock = self.world.lock();
        if let Some(packet) = world_lock.check_packet(packet, &client) {
            drop(world_lock);
            if let Some(_) = PacketHandler::check_packet(packet, &client, self) {
                warn!("Couldnt handle packet!");
            }
        }
    }

    fn on_server_stop(&mut self, message: &str) {

    }
}