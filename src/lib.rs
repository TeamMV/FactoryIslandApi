use crate::command::{CommandProcessor, CommandSender, COMMAND_PROCESSOR};
use crate::event::common::{ServerCommandEvent, ServerTickEvent};
use crate::event::player::PlayerMoveEvent;
use crate::event::{Event, GameStartEvent};
use crate::mods::{ModLoader, DEFAULT_MOD_DIR};
use crate::player::{Player, PlayerType};
use crate::registry::terrain::TerrainTiles;
use crate::registry::GameObjects;
use crate::server::packets::common::{ClientDataPacket, PlayerData, ServerStatePacket, TileKind};
use crate::server::packets::player::{OtherPlayerChatPacket, OtherPlayerJoinPacket, OtherPlayerLeavePacket, OtherPlayerMovePacket, PlayerMovePacket};
use crate::server::{ClientBoundPacket, ServerBoundPacket};
use crate::world::{TileSetReason, World, WorldType};
use event::player::{PlayerJoinEvent, PlayerLeaveEvent};
use hashbrown::HashSet;
use log::{debug, error, info};
use mvengine::event::EventBus;
use mvengine::net::server::{ClientEndpoint, ClientId, ServerHandler};
use mvengine::net::DisconnectReason;
use mvengine::utils::args::ParsedArgs;
use mvengine::utils::savers::save_to_vec;
use mvutils::enum_val;
use mvutils::hashers::U64IdentityHasher;
use std::collections::HashMap;
use std::ops::Deref;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use std::{env, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use mvutils::clock::Clock;
use mvutils::unsafe_utils::Unsafe;
use parking_lot::lock_api::RwLock;
use crate::registry::tiles::TILE_REGISTRY;
use crate::server::packets::world::TileSetPacket;
use crate::world::chunk::ToClientObject;
use crate::world::tiles::implementations::power::lamp::LampState;
use crate::world::tiles::Orientation;
use crate::world::tiles::tiles::TileType;

pub mod event;
pub mod world;
pub mod settings;
pub mod player;
pub mod server;
pub mod mods;
pub mod command;
pub mod registry;

pub struct FactoryIsland {
    pub(crate) world: WorldType,
    pub(crate) players: HashMap<ClientId, PlayerType, U64IdentityHasher>,
    pub(crate) event_bus: EventBus<Event>,
    pub(crate) mod_loader: ModLoader,
    pub(crate) render_distance: i32,
    
    pub objects: GameObjects,
}

impl FactoryIsland {
    pub fn tick(&mut self) {
        let this = unsafe { Unsafe::cast_lifetime(self) };
        let mut loaded_by_player = HashSet::new();
        for player in self.players.values() {
            let mut lock = player.lock();
            lock.tick();
            for pos in &lock.loaded_chunks {
                loaded_by_player.insert(*pos);
            }
        }
        let mut world = self.world.lock();
        world.check_unload(loaded_by_player);
        world.tick(&mut self.event_bus, this);
        
        drop(world);

        self.event_bus.dispatch(&mut Event::ServerTickEvent(ServerTickEvent));
    }

    pub fn on_command(&mut self, command: String, player: Option<PlayerData>) {
        COMMAND_PROCESSOR.process(player.map_or(CommandSender::Console, |d| CommandSender::Player(d)), command, self);
    }
}

impl ServerHandler<ServerBoundPacket> for FactoryIsland {
    fn on_server_start(port: u16) -> Self {
        let args = ParsedArgs::parse(env::args());
        let mod_dir = args.try_get_as("-mods").unwrap_or(DEFAULT_MOD_DIR.clone());

        let mut event_bus = EventBus::new();
        
        let terrain_tiles = registry::terrain::register_all();
        let tiles = registry::tiles::register_all();
        command::register_commands();
        
        let objects = GameObjects {
            terrain: terrain_tiles,
            tiles,
        };
        
        let mut mod_loader = ModLoader::new();
        mod_loader.load(&mod_dir, &mut event_bus, objects.clone());
        
        event_bus.dispatch(&mut Event::GameStartEvent(GameStartEvent));

        let world = World::get_main(objects.clone(), &mut event_bus);

        FactoryIsland {
            world,
            players: HashMap::with_hasher(U64IdentityHasher::default()),
            event_bus,
            mod_loader,
            render_distance: 1,
            
            objects,
        }
    }

    fn on_client_connect(&mut self, client: Arc<ClientEndpoint>) {
        let mut player_data = Vec::new();
        for (client_id, player) in &self.players {
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
                    source: object.info.source,
                });
            }
        }
        client.send(ClientBoundPacket::ServerState(ServerStatePacket {
            players: player_data,
            mods: self.mod_loader.res_mod_ids(),
            tiles,
        }));

        let id = client.id();
        let player = Player::new(client, self.world.clone());
        let mut event = Event::PlayerJoinEvent(PlayerJoinEvent {
            player: player.clone(),
        });
        self.event_bus.dispatch(&mut event);
        self.players.insert(id, player);
    }

    fn on_client_disconnect(&mut self, client: Arc<ClientEndpoint>, reason: DisconnectReason) {
        if let Some(player) = self.players.get(&client.id()) {
            let mut event = Event::PlayerLeaveEvent(PlayerLeaveEvent {
                player: player.clone(),
                reason,
            });
            self.event_bus.dispatch(&mut event);

            let mut lock = player.lock();
            lock.on_disconnect();
            drop(lock);
            self.players.remove(&client.id());

            let id = client.id();
            for (_, other_player) in self.players.iter().filter(|(p, _)| **p != id) {
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
        match packet {
            ServerBoundPacket::ClientData(packet) => {
                debug!("Client data packet arrived");
                if let Some(player) = self.players.get(&client.id()) {
                    let mut lock = player.lock();
                    lock.apply_data(packet.clone(), &mut self.event_bus);

                    let id = client.id();
                    debug!("starting client join message");
                    for (_, other_player) in self.players.iter().filter(|(p, _)| **p != id) {
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
                if let Some(player) = self.players.get(&client.id()) {
                    let mut event = Event::PlayerMoveEvent(PlayerMoveEvent {
                        has_been_cancelled: false,
                        pos: packet.pos,
                        player: player.clone(),
                    });
                    self.event_bus.dispatch(&mut event);

                    let event = enum_val!(Event, event, PlayerMoveEvent);
                    if !event.has_been_cancelled {
                        let mut lock = player.lock();
                        lock.move_to(event.pos, &mut self.event_bus);
                        drop(lock);
                    } else {
                        client.send(ClientBoundPacket::PlayerMove(PlayerMovePacket {
                            pos: event.pos,
                        }));
                    }
                    for (_, other_player) in self.players.iter().filter(|(p, _)| **p != client.id()) {
                        let lock = other_player.lock();
                        if let Some(endpoint) = lock.client_endpoint() {
                            endpoint.send(ClientBoundPacket::OtherPlayerMove(OtherPlayerMovePacket {
                                client_id: client.id(),
                                pos: event.pos,
                            }));
                        }
                    }
                }
            }
            ServerBoundPacket::TileSet(packet) => {
                if let Some(mut tile) = TILE_REGISTRY.create_object(packet.tile_id) {
                    if let Some(state) = &mut tile.info.state {
                        state.apply_client_state(packet.tile_state);
                    }
                    let cloned = self.world.clone();
                    let mut world_lock = cloned.lock();
                    let player = self.players.get(&client.id()).cloned();
                    if let Some(player) = player {
                        let reason = TileSetReason::Player(player.lock().data.clone());
                        
                        let to_client = ToClientObject {
                            id: packet.tile_id as u16,
                            source: tile.info.source.clone(),
                            orientation: packet.orientation,
                            state: packet.tile_state,
                        };

                        world_lock.set_tile_at(packet.pos.clone(), tile.to_type(), reason.clone(), self);
                        
                        // Dont filter out the current id as the tile only gets set on the client if the server says its okay. just to avoid desync
                        for (_, other_player) in self.players.iter() {
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
                if let Some(player) = self.players.get(&client.id()) {
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
                        for (_, other_player) in self.players.iter() {
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
                if let Some(player) = self.players.get(&client.id()) {
                    let mut lock = player.lock();
                    lock.loaded_chunks.clear();
                    let rdst = lock.data.render_distance;
                    lock.after_move(rdst, &mut self.event_bus);
                }
            }
        }
    }

    fn on_server_stop(&mut self, message: &str) {

    }
}