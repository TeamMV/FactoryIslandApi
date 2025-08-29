#![feature(map_try_insert)]
#![feature(try_trait_v2)]

use crate::command::{CommandProcessor, CommandSender, COMMAND_PROCESSOR};
use crate::mods::{ModLoader, DEFAULT_MOD_DIR};
use crate::player::{Player, PlayerType};
use crate::registry::terrain::TerrainTiles;
use crate::registry::GameObjects;
use crate::server::packets::common::{ClientDataPacket, PlayerData, ServerStatePacket, TileKind};
use crate::server::packets::player::{OtherPlayerChatPacket, OtherPlayerJoinPacket, OtherPlayerLeavePacket, OtherPlayerMovePacket, PlayerMovePacket};
use crate::server::{ClientBoundPacket, ServerBoundPacket};
use crate::world::{TileSetReason, World, WorldType};
use hashbrown::HashSet;
use log::{debug, error, info};
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
use mvutils::unsafe_utils::Unsafe;
use parking_lot::RwLock;
use crate::mods::modsdk::events::Event;
use crate::mods::modsdk::events::player::{PlayerJoinEvent, PlayerLeaveEvent, PlayerMoveEvent};
use crate::mods::modsdk::{MPlayerData, MTileUnit};
use crate::registry::tiles::TILE_REGISTRY;
use crate::server::packets::world::TileSetPacket;
use crate::world::chunk::ToClientObject;
use crate::world::tiles::Orientation;
use crate::world::tiles::tiles::TileType;

pub mod world;
pub mod settings;
pub mod player;
pub mod server;
pub mod mods;
pub mod command;
pub mod registry;

lazy! {
    pub(crate) static PLAYERS: RwLock<HashMap<ClientId, PlayerType, U64IdentityHasher>> = RwLock::new(HashMap::with_hasher(U64IdentityHasher::default()));
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
        ModLoader::dispatch_event(Event::ServerTickEvent);
    }

    pub fn on_command(&mut self, command: String, player: Option<PlayerData>) {
        COMMAND_PROCESSOR.process(player.map_or(CommandSender::Console, |d| CommandSender::Player(d)), command, self);
    }

    pub fn stop(&mut self) {
        ModLoader::dispatch_event(Event::GameEndEvent);
        ModLoader::unload();
        exit(0);
    }
}

impl ServerHandler<ServerBoundPacket> for FactoryIsland {
    fn on_server_start(port: u16) -> Self {
        let args = ParsedArgs::parse(env::args());
        let mod_dir = args.try_get_as("-mods").unwrap_or(DEFAULT_MOD_DIR.clone());
        
        let terrain_tiles = registry::terrain::register_all();
        let tiles = registry::tiles::register_all();
        command::register_commands();
        
        let objects = GameObjects {
            terrain: terrain_tiles,
            tiles,
        };

        ModLoader::load(&mod_dir, objects.clone());
        ModLoader::dispatch_event(Event::GameStartEvent);

        let world = World::get_main(objects.clone());

        FactoryIsland {
            world,
            render_distance: 1,
            
            objects,
        }
    }

    fn on_client_connect(&mut self, client: Arc<ClientEndpoint>) {
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
        let mut tiles = Vec::with_capacity(TILE_REGISTRY.len());
        for id in 0..TILE_REGISTRY.len() {
            if let Some(object) = TILE_REGISTRY.create_object(id) {
                tiles.push(TileKind {
                    id,
                    source: object.info.source.clone(),
                });
            }
        }
        client.send(ClientBoundPacket::ServerState(ServerStatePacket {
            players: player_data,
            mods: ModLoader::res_mod_ids(),
            tiles,
            client_id: client.id(),
        }));
        let id = client.id();
        let player = Player::new(client, self.world.clone());
        ModLoader::dispatch_event(Event::PlayerJoinEvent(PlayerJoinEvent {
            player: id,
        }));
        players.insert(id, player);
    }

    fn on_client_disconnect(&mut self, client: Arc<ClientEndpoint>, reason: DisconnectReason) {
        let mut players = PLAYERS.write();
        if let Some(player) = players.get(&client.id()).cloned() {
            let lock = player.lock();
            let name = lock.data.name.clone().into_c();
            let pos = MTileUnit::from_normal(lock.position);
            let event = Event::PlayerLeaveEvent(PlayerLeaveEvent {
                player: client.id(),
                data: MPlayerData {
                    name,
                    pos,
                },
            });
            ModLoader::dispatch_event(event);

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
        let mut players = PLAYERS.write();
        match packet {
            ServerBoundPacket::ClientData(packet) => {
                debug!("Client data packet arrived");
                if let Some(player) = players.get(&client.id()) {
                    let mut lock = player.lock();
                    lock.apply_data(packet.clone());

                    let id = client.id();
                    debug!("starting client join message");
                    for (_, other_player) in players.iter().filter(|(p, _)| **p != id) {
                        let lock = other_player.lock();
                        if let Some(endpoint) = lock.client_endpoint() {
                            endpoint.send(ClientBoundPacket::OtherPlayerJoin(OtherPlayerJoinPacket {
                                client_id: id,
                                client_data: packet.clone()
                            }));
                        }
                    }
                    debug!("finished client join message");
                }
            }
            ServerBoundPacket::PlayerMove(packet) => {
                if let Some(player) = players.get(&client.id()) {
                    let pos = player.lock().position;
                    let position = MTileUnit::from_normal(pos);
                    let mut event = Event::PlayerMoveEvent(PlayerMoveEvent {
                        has_been_cancelled: false,
                        player: client.id(),
                        position,
                    });
                    let event = ModLoader::dispatch_event(event);

                    let event = enum_val!(Event, event, PlayerMoveEvent);
                    let pos = event.position.to_normal();
                    if !event.has_been_cancelled {
                        let mut lock = player.lock();
                        lock.move_to(pos);
                        drop(lock);
                    } else {
                        client.send(ClientBoundPacket::PlayerMove(PlayerMovePacket {
                            pos,
                        }));
                    }
                    for (_, other_player) in players.iter().filter(|(p, _)| **p != client.id()) {
                        let lock = other_player.lock();
                        if let Some(endpoint) = lock.client_endpoint() {
                            endpoint.send(ClientBoundPacket::OtherPlayerMove(OtherPlayerMovePacket {
                                client_id: client.id(),
                                pos,
                            }));
                        }
                    }
                }
            }
            ServerBoundPacket::TileSet(packet) => {
                if let Some(mut tile) = TILE_REGISTRY.create_object(packet.tile_id) {
                    if let Some((state)) = &mut tile.info.state {
                        let mut loader = ByteBuffer::from_vec_le(packet.tile_state.clone());
                        if let Err(e) = state.load_from_client(&mut loader) {
                            error!("Error when loading tile state from client: {e}");
                        }
                    }
                    let cloned = self.world.clone();
                    let mut world_lock = cloned.lock();
                    let player = players.get(&client.id()).cloned();
                    if let Some(player) = player {
                        let reason = TileSetReason::Player(player.lock().data.clone());

                        let to_client = ToClientObject {
                            id: packet.tile_id as u16,
                            source: tile.info.source.clone(),
                            orientation: packet.orientation,
                            state: packet.tile_state.clone(),
                        };

                        world_lock.set_tile_at(packet.pos.clone(), tile.to_type(), reason.clone());
                        
                        // Dont filter out the current id as the tile only gets set on the client if the server says its okay. just to avoid desync
                        for (_, other_player) in players.iter() {
                            let lock = other_player.lock();
                            if let Some(endpoint) = lock.client_endpoint() {
                                endpoint.send(ClientBoundPacket::TileSet(TileSetPacket {
                                    pos: packet.pos.clone(),
                                    tile: to_client.clone(),
                                    reason: reason.clone(),
                                }));
                            }
                        }
                    }
                } else {
                    info!("Received Invalid tile from client with id: {}", packet.tile_id);
                };
            },
            ServerBoundPacket::PlayerChat(packet) => {
                if let Some(player) = players.get(&client.id()) {
                    let lock = player.lock();
                    let client_data = lock.data.clone();
                    drop(lock);
                    let data = PlayerData {
                        client_id: client.id(),
                        data: client_data,
                    };
                    if packet.message.chars().next() == Some('/') {
                        let command = packet.message[1..].trim().to_string();
                        self.on_command(command, Some(data));
                    } else {
                        for (_, other_player) in players.iter() {
                            let lock = other_player.lock();
                            if let Some(endpoint) = lock.client_endpoint() {
                                endpoint.send(ClientBoundPacket::OtherPlayerChat(OtherPlayerChatPacket {
                                    player: data.clone(),
                                    message: packet.message.clone(),
                                }));
                            }
                        }
                    }
                }
            }
            ServerBoundPacket::RequestReload => {
                if let Some(player) = players.get(&client.id()) {
                    let mut lock = player.lock();
                    lock.loaded_chunks.clear();
                    let rdst = lock.data.render_distance;
                    lock.after_move(rdst);
                }
            }
        }
    }

    fn on_server_stop(&mut self, message: &str) {

    }
}