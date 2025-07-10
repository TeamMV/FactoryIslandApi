use crate::server::packets::common::ClientDataPacket;
use crate::server::packets::world::{ChunkDataPacket, ChunkUnloadPacket};
use crate::server::ClientBoundPacket;
use crate::world::{ChunkPos, TileUnit, WorldType, CHUNK_SIZE};
use hashbrown::HashSet;
use mvengine::net::server::ClientEndpoint;
use mvengine::utils::savers::SaveArc;
use mvutils::save::Savable;
use mvutils::Savable;
use parking_lot::Mutex;
use std::sync::Arc;
use mvengine::event::EventBus;
use crate::event::Event;

pub type PlayerType = SaveArc<Mutex<Player>>;

const DEFAULT_NAME: &str = "unknown";
pub const UNLOAD_DISTANCE: i32 = 3;

#[derive(Savable)]
pub struct Player {
    pub data: ClientDataPacket,
    #[unsaved]
    client_endpoint: Option<Arc<ClientEndpoint>>,
    #[unsaved]
    world: Option<WorldType>,
    position: TileUnit,
    pub loaded_chunks: HashSet<ChunkPos>,
}

impl Player {
    pub fn new(endpoint: Arc<ClientEndpoint>, world: WorldType) -> SaveArc<Mutex<Self>> {
        let this = Self {
            data: ClientDataPacket {
                name: DEFAULT_NAME.to_string(),
                render_distance: 1,
            },
            client_endpoint: Some(endpoint),
            world: Some(world),
            position: (0.0, 0.0),
            loaded_chunks: HashSet::new(),
        };
        SaveArc::new(Mutex::new(this))
    }

    pub fn tick(&mut self) {

    }

    pub(crate) fn after_move(&mut self, render_distance: i32, event_bus: &mut EventBus<Event>) {
        if let Some(world) = &self.world {
            let current_chunk = self.get_current_chunk();
            let mut world_lock = world.lock();
            let mut to_unload = self.loaded_chunks.clone();
            for chunk_x in (current_chunk.0 - render_distance)..=(current_chunk.0 + render_distance) {
                for chunk_y in (current_chunk.1 - render_distance)..=(current_chunk.1 + render_distance) {
                    if !self.loaded_chunks.contains(&(chunk_x, chunk_y)) {
                        let chunk = world_lock.get_chunk((chunk_x, chunk_y), event_bus);
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
    }

    pub fn move_to(&mut self, pos: TileUnit, event_bus: &mut EventBus<Event>) {
        self.position = pos;
        self.after_move(self.data.render_distance, event_bus);
    }

    pub fn move_by(&mut self, dpos: TileUnit, event_bus: &mut EventBus<Event>) {
        self.position.0 += dpos.0;
        self.position.1 += dpos.1;
        self.after_move(self.data.render_distance, event_bus);
    }

    pub fn apply_data(&mut self, packet: ClientDataPacket, event_bus: &mut EventBus<Event>) {
        self.data = packet;
        self.after_move(self.data.render_distance, event_bus);


        if let Some(world) = &self.world {
            let mut world_lock = world.lock();
            let current_chunk = self.get_current_chunk();
            for chunk_x in (current_chunk.0 - self.data.render_distance)..=(current_chunk.0 + self.data.render_distance) {
                for chunk_y in (current_chunk.1 - self.data.render_distance)..=(current_chunk.1 + self.data.render_distance) {
                    let chunk = world_lock.get_chunk((chunk_x, chunk_y), event_bus);
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
        &self.data.name
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