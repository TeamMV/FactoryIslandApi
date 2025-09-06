pub mod uuid;
pub mod profile;

use std::ops::{Deref, DerefMut};
use crate::server::packets::common::ClientDataPacket;
use crate::server::packets::world::{ChunkDataPacket, ChunkUnloadPacket};
use crate::server::ClientBoundPacket;
use crate::world::{ChunkPos, SingleTileUnit, TileUnit, WorldType, CHUNK_SIZE};
use hashbrown::HashSet;
use mvengine::net::server::ClientEndpoint;
use mvengine::utils::savers::SaveArc;
use mvutils::save::{Loader, Savable, Saver};
use mvutils::Savable;
use parking_lot::Mutex;
use std::sync::Arc;
use mvutils::bytebuffer::ByteBufferExtras;
use uuid::UUID;
use crate::player::profile::PlayerProfile;
use crate::server::packets::player::PlayerDataPacket;

pub type PlayerType = SaveArc<Mutex<Player>>;

pub const UNLOAD_DISTANCE: i32 = 3;

impl Default for ClientDataPacket {
    fn default() -> Self {
        Self {
            profile: PlayerProfile::new(),
            render_distance: 1,
            client_id: 0
        }
    }
}

#[derive(Savable)]
pub struct Player {
    #[unsaved]
    pub data: ClientDataPacket,
    #[unsaved]
    client_endpoint: Option<Arc<ClientEndpoint>>,
    #[unsaved]
    world: Option<WorldType>,
    pub position: TileUnit,
    #[unsaved]
    pub loaded_chunks: HashSet<ChunkPos>,
    pub reach: SingleTileUnit
}

impl Player {
    pub fn new(endpoint: Arc<ClientEndpoint>, world: WorldType) -> SaveArc<Mutex<Self>> {
        let this = Self {
            data: ClientDataPacket::default(),
            client_endpoint: Some(endpoint),
            world: Some(world),
            position: (0.0, 0.0),
            loaded_chunks: HashSet::new(),
            reach: 7.0,
        };
        SaveArc::new(Mutex::new(this))
    }

    pub fn tick(&mut self) {

    }

    pub(crate) fn after_move(&mut self, render_distance: i32) {
        if let Some(world) = &self.world {
            let current_chunk = self.get_current_chunk();
            let mut world_lock = world.lock();
            let mut to_unload = self.loaded_chunks.clone();
            for chunk_x in (current_chunk.0 - render_distance)..=(current_chunk.0 + render_distance) {
                for chunk_y in (current_chunk.1 - render_distance)..=(current_chunk.1 + render_distance) {
                    if !self.loaded_chunks.contains(&(chunk_x, chunk_y)) {
                        let chunk = world_lock.get_chunk((chunk_x, chunk_y));
                        let chunk = chunk.lock();
                        let data_packet = ChunkDataPacket {
                            pos: (chunk_x, chunk_y),
                            data: chunk.to_client(),
                        };
                        if let Some(client_endpoint) = &self.client_endpoint {
                            client_endpoint.send(ClientBoundPacket::ChunkData(data_packet));
                        }
                        self.loaded_chunks.insert((chunk_x, chunk_y));
                    }
                    to_unload.remove(&(chunk_x, chunk_y));
                }
            }
            for pos in to_unload {
                if (current_chunk.0 - pos.0).abs() < render_distance + UNLOAD_DISTANCE {
                    if (current_chunk.1 - pos.1).abs() < render_distance + UNLOAD_DISTANCE {
                        continue;
                    }
                }
                self.loaded_chunks.remove(&pos);
                if let Some(client) = &self.client_endpoint {
                    client.send(ClientBoundPacket::ChunkUnload(ChunkUnloadPacket {
                        pos,
                    }));
                }
            }
        }
    }

    pub(crate) fn on_disconnect(&mut self) {
        self.loaded_chunks.clear();
        //save player to file
        if let Some(world) = &self.world {
            let lock = world.lock();
            let dir = lock.players_directory();
            let filename = format!("{:?}.sav", self.data.profile.uuid);
            dir.save_object(self, &filename);
        }
    }

    pub fn move_to(&mut self, pos: TileUnit) {
        self.position = pos;
        self.after_move(self.data.render_distance);
    }

    pub fn move_by(&mut self, dpos: TileUnit) {
        self.position.0 += dpos.0;
        self.position.1 += dpos.1;
        self.after_move(self.data.render_distance);
    }

    pub fn apply_data(&mut self, packet: ClientDataPacket) {
        self.data = packet;
        if let Some(world) = &self.world {
            let lock = world.lock();
            let filename = format!("{:?}.sav", self.data.profile.uuid);
            let players_dir = lock.players_directory();
            if let Some(t) = players_dir.read_object::<Player>(&filename) {
                self.position = t.position;
                self.reach = t.reach;
            }
        }

        self.after_move(self.data.render_distance);

        if let Some(endpoint) = &self.client_endpoint {
            endpoint.send(ClientBoundPacket::PlayerDataPacket(PlayerDataPacket {
                pos: self.position,
                reach: self.reach,
            }));
        }

        if let Some(world) = &self.world {
            let mut world_lock = world.lock();
            let current_chunk = self.get_current_chunk();
            for chunk_x in (current_chunk.0 - self.data.render_distance)..=(current_chunk.0 + self.data.render_distance) {
                for chunk_y in (current_chunk.1 - self.data.render_distance)..=(current_chunk.1 + self.data.render_distance) {
                    let chunk = world_lock.get_chunk((chunk_x, chunk_y));
                    let chunk = chunk.lock();
                    let data_packet = ChunkDataPacket {
                        pos: (chunk_x, chunk_y),
                        data: chunk.to_client(),
                    };
                    if let Some(client_endpoint) = &self.client_endpoint {
                        client_endpoint.send(ClientBoundPacket::ChunkData(data_packet));
                    }
                    self.loaded_chunks.insert((chunk_x, chunk_y));
                }
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.data.profile.name
    }

    pub fn client_endpoint(&self) -> Option<&Arc<ClientEndpoint>> {
        self.client_endpoint.as_ref()
    }

    pub fn world(&self) -> Option<&WorldType> {
        self.world.as_ref()
    }

    pub fn get_current_chunk(&self) -> ChunkPos {
        (
            (self.position.0 / CHUNK_SIZE as f64).floor() as i32,
            (self.position.1 / CHUNK_SIZE as f64).floor() as i32
        )
    }
}