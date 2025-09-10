pub mod chunk;
pub mod manager;
pub mod tiles;
pub mod generate;

use crate::player::PlayerType;
use crate::world::chunk::{Chunk, ToClientObject};
use crate::world::generate::{ChunkGenerator, GeneratePipeline};
use crate::world::manager::ChunkManager;
use crate::world::tiles::pos::TilePos;
use mvengine::event::EventBus;
use mvengine::rendering::RenderContext;
use mvengine::ui::context::UiResources;
use mvengine::ui::geometry::SimpleRect;
use mvengine::utils::savers::SaveArc;
use mvutils::once::CreateOnce;
use mvutils::save::{Loader, Savable, Saver};
use parking_lot::Mutex;
use rand::{rng, RngCore};
use std::collections::HashMap;
use std::{env, fs};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use abi_stable::pmr::IsAccessible::No;
use bytebuffer::ByteBuffer;
use hashbrown::HashSet;
use log::{debug, error, info, warn};
use mvengine::game::fs::smartdir::SmartDir;
use mvengine::net::server::ClientEndpoint;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::{enum_val, Savable};
use mvutils::unsafe_utils::Unsafe;
use crate::{broadcast_all_players, registry, FactoryIsland, PLAYERS};
use crate::multitile::MultiTilePlacement;
use crate::registry::GameObjects;
use crate::registry::multitiles::MULTI_REGISTRY;
use crate::registry::tiles::TILE_REGISTRY;
use crate::server::{ClientBoundPacket, ServerBoundPacket};
use crate::server::packets::common::{ClientDataPacket, PlayerData};
use crate::server::packets::player::{OtherPlayerChatPacket, OtherPlayerJoinPacket, OtherPlayerMovePacket};
use crate::server::packets::world::{MultiTileDestroyedPacket, MultiTilePlacedPacket, TerrainSetPacket, TileSetPacket};
use crate::world::tiles::terrain::{WorldTerrain, TerrainTile};
use crate::world::tiles::tiles::{InnerTile, TileType};

pub const CHUNK_SIZE: i32 = 64;

pub type ChunkPos = (i32, i32);
pub type ChunkType = SaveArc<Mutex<Chunk>>;
pub type WorldType = SaveArc<Mutex<World>>;

pub const META_FILENAME: &str = "meta.sav";
pub const PLAYERS_DIR : &str = "players";
pub const CHUNKS_DIR : &str = "chunks";
pub const MULTITILES_FILENAME: &str = "multitiles.sav";

pub const START_FORCE_ALLOWED: u16 = 9;

#[derive(Savable)]
pub struct WorldMeta {
    name: String,
    seed: u32,
    max_forced_chunks: u16,
    forced_chunks: HashSet<ChunkPos>,
}

impl WorldMeta {
    pub fn new(name: &str, seed: u32) -> Self {
        let mut forced_chunks = HashSet::new();
        forced_chunks.insert((0, 0));
        Self {
            name: name.to_string(),
            seed,
            max_forced_chunks: START_FORCE_ALLOWED,
            forced_chunks,
        }
    }
}

pub struct World {
    meta: WorldMeta,
    directory: SmartDir,
    chunk_directory: SmartDir,
    players_directory: SmartDir,
    loaded_chunks: HashMap<ChunkPos, ChunkType>,
    chunk_manager: ChunkManager,
    generator_pipeline: GeneratePipeline,
    objects: GameObjects,
    
    arc: Weak<Mutex<World>>
}

impl World {
    pub fn get_main(game_objects: GameObjects) -> WorldType {
        if let Some(world) = World::load("main", game_objects.clone()) {
            world
        } else {
            let rng_seed = rand::rng().next_u32();
            World::new("main", rng_seed, game_objects)
        }
    }
}

impl World {
    pub fn load(name: &str, game_objects: GameObjects) -> Option<WorldType> {
        let dir_name = name.replace(' ', "_");
        let appdata = env::var("APPDATA").expect("Failed to get APPDATA environment variable");
        let mut full = PathBuf::from(appdata);
        full.push(manager::PATH);
        full.push(dir_name);

        let directory = SmartDir::new(full);
        if !directory.exists_yet() {
            None
        } else {
            let chunk_directory = directory.join(CHUNKS_DIR);
            let players_directory = directory.join(PLAYERS_DIR);
            if !directory.exists_file(META_FILENAME) {
                let new_seed = rng().next_u32();
                warn!("Meta file is not available, recreating it with a new seed: {}", new_seed);
                let meta = WorldMeta::new(name, new_seed);
                directory.save_object(&meta, META_FILENAME);
            }
            if let Some(meta) = directory.read_object::<WorldMeta>(META_FILENAME) {
                let seed = meta.seed;

                let mut this = Self {
                    meta,
                    directory,
                    chunk_directory,
                    players_directory,
                    loaded_chunks: HashMap::new(),
                    chunk_manager: ChunkManager,
                    generator_pipeline: GeneratePipeline::new(seed),
                    objects: game_objects,
                    arc: Weak::new(),
                };

                for chunk_pos in this.meta.forced_chunks.clone() {
                    let _ = this.get_chunk(chunk_pos);
                }

                let arc = Arc::new_cyclic(|weak| {
                    this.arc = weak.clone();
                    Mutex::new(this)
                }).into();

                Some(arc)
            } else {
                None
            }
        }
    }

    pub fn new(name: &str, seed: u32, game_objects: GameObjects) -> WorldType {
        let dir_name = name.replace(' ', "_");
        let appdata = env::var("APPDATA").expect("Failed to get APPDATA environment variable");
        let mut full = PathBuf::from(appdata);
        full.push(manager::PATH);
        full.push(dir_name);

        let directory = SmartDir::new(full);

        let chunk_directory = directory.join(CHUNKS_DIR);
        let players_directory = directory.join(PLAYERS_DIR);

        Arc::new_cyclic(|weak| {
            Mutex::new(Self {
                meta: WorldMeta::new(name, seed),
                directory,
                chunk_directory,
                players_directory,
                loaded_chunks: HashMap::new(),
                chunk_manager: ChunkManager {},
                generator_pipeline: GeneratePipeline::new(seed),
                objects: game_objects,
                arc: weak.clone(),
            })
        }).into()
    }

    pub fn save(&mut self) {
        self.directory.save_object(&self.meta, META_FILENAME);
        for chunk in self.loaded_chunks.values() {
            let c = chunk.lock();
            self.chunk_manager.try_save_chunk(self, &*c);
        }
    }

    pub fn get_chunk(&mut self, chunk_pos: ChunkPos) -> ChunkType {
        if let Some(chunk) = self.loaded_chunks.get(&chunk_pos) {
            return chunk.clone();
        }

        let loaded = self.chunk_manager.try_load_chunk(self, chunk_pos);
        if let Some(chunk) = loaded {
            //chunk loaded
            self.loaded_chunks.insert(chunk_pos, chunk.clone());
            chunk
        } else {
            let chunk = Chunk::new(chunk_pos, self.meta.seed);
            let chunk = SaveArc::new(Mutex::new(chunk));
            //generate new chunk if loading fails
            debug!("Generating new chunk at {chunk_pos:?}");
            let mut chunk_lock = chunk.lock();
            chunk_lock.generate_terrain(&self.generator_pipeline, &self.objects);
            chunk_lock.generate(&self.generator_pipeline, &self.objects);
            let pos = chunk_lock.position;
            drop(chunk_lock);
            self.loaded_chunks.insert(pos, chunk.clone());
            chunk
        }
    }

    pub fn is_loaded(&self, pos: ChunkPos) -> bool {
        self.loaded_chunks.contains_key(&pos)
    }

    pub fn exists_file(&self, pos: ChunkPos) -> bool {
        let filename = format!("c{}_{}.chunk", pos.0, pos.1);
        self.chunk_directory.exists_file(&filename)
    }

    pub fn unload_chunk(&mut self, pos: ChunkPos) {
        if let Some(chunk) = self.loaded_chunks.get(&pos) {
            let c = chunk.lock();
            self.chunk_manager.try_save_chunk(self, &*c);
        }
        self.loaded_chunks.remove(&pos);
    }

    pub fn check_unload(&mut self, mut keep: HashSet<ChunkPos>) {
        keep.extend(&self.meta.forced_chunks);
        let mut to_unload = Vec::new();
        for (pos, _) in self.loaded_chunks.iter().filter(|(c, _)| !keep.contains(*c)) {
            to_unload.push(*pos);
        }
        for pos in to_unload {
            self.unload_chunk(pos);
        }
    }

    pub fn set_chunk(&mut self, pos: ChunkPos, chunk: ChunkType) {
        self.loaded_chunks.insert(pos, chunk);
    }

    pub fn is_multitile_at(&mut self, pos: &TilePos) -> bool {
        for chunk_pos in pos.multitile_chunk_maybe_positions() {
            let chunk = self.get_chunk(chunk_pos);
            let lock = chunk.lock();
            if lock.multitiles.iter().any(|mt| mt.includes(pos)) {
                return true;
            }
        }
        false
    }

    pub fn get_tile_at(&mut self, pos: TilePos) -> Option<TileType> {
        let chunk = self.get_chunk((pos.world_chunk_x, pos.world_chunk_z));
        let mut lock = chunk.lock();
        lock.tiles[Chunk::get_index(&pos)].clone()
    }

    pub fn get_tile_id_at(&mut self, pos: TilePos) -> Option<u16> {
        let lock = self.get_tile_at(pos)?;
        let x = { let x = Some(lock.read().id as u16); x }; x
    }

    pub fn get_terrain_at(&mut self, pos: TilePos) -> WorldTerrain {
        let chunk = self.get_chunk((pos.world_chunk_x, pos.world_chunk_z));
        let mut lock = chunk.lock();
        let index = Chunk::get_index(&pos);
        let id = lock.terrain.terrain[index];
        let orientation = lock.terrain.orientation[index];
        WorldTerrain {
            id,
            orientation,
        }
    }

    pub fn set_tile_at(&mut self, pos: TilePos, tile: TileType, reason: TileSetReason) {
        let chunk = self.get_chunk(pos.chunk_pos);
        let mut lock = chunk.lock();
        lock.tiles[Chunk::get_index(&pos)] = Some(tile.clone());

        let client_tile = tiles::tile_to_client(&tile);

        broadcast_all_players(ClientBoundPacket::TileSet(TileSetPacket {
            pos: pos.clone(),
            tile: client_tile,
            reason,
        }));
        drop(lock);

        let rw = tile.read();

        let mut remove = None;
        for chunk_pos in pos.multitile_chunk_maybe_positions() {
            let chunk = self.get_chunk(chunk_pos);
            let mut lock = chunk.lock();
            for i in 0..lock.multitiles.len() {
                if lock.multitiles[i].includes(&pos) {
                    remove = Some(i);
                    break;
                }
            }
            if let Some(i) = remove {
                let multiblock = lock.multitiles.remove(i);
                let packet = ClientBoundPacket::MultiTileDestroyedPacket(MultiTileDestroyedPacket {
                    placement_id: multiblock.uuid,
                    chunk_pos: multiblock.pos.chunk_pos,
                });
                broadcast_all_players(packet);
                break;
            }
        }

        //check multiblocks
        let mut placement = None;
        for mt_id in 0..MULTI_REGISTRY.len() {
            if let Some(multi) = MULTI_REGISTRY.reference_object(mt_id) {
                if let Some(p) = multi.check_completion(self, pos.clone(), rw.id as u16) {
                    placement = Some(p);
                    break;
                }
            }
        }

        if let Some(placement) = placement {
            let chunk = self.get_chunk(placement.pos.chunk_pos);
            let mut lock = chunk.lock();
            lock.multitiles.push(placement.clone());
            drop(lock);
            let packet = ClientBoundPacket::MultiTilePlacedPacket(MultiTilePlacedPacket {
                placement,
            });
            broadcast_all_players(packet);
        }


        let state = if let Some(s) = &rw.info.state {
            let mut buf = ByteBuffer::new_le();
            s.save_for_client(&mut buf);
            buf.into_vec()
        } else {
            vec![]
        };
        let client_obj = ToClientObject {
            id: rw.id as u16,
            orientation: rw.info.orientation,
            source: rw.info.source.clone(),
            state
        };

        broadcast_all_players(ClientBoundPacket::TileSet(TileSetPacket {
            pos: pos.clone(),
            tile: client_obj.clone(),
            reason: TileSetReason::DontCare,
        }));
    }

    pub fn set_terrain_at(&mut self, pos: TilePos, terrain: WorldTerrain, reason: TileSetReason) {
        if let Some(template) = registry::terrain::TERRAIN_REGISTRY.create_object(terrain.id as usize) {
            let chunk = self.get_chunk(pos.chunk_pos);
            let mut chunk_lock = chunk.lock();
            let index = Chunk::get_index(&pos);
            let id = terrain.id;
            let orientation = terrain.orientation;
            chunk_lock.terrain.terrain[index] = id;
            chunk_lock.terrain.orientation[index] = orientation;

            let client_obj = ToClientObject {
                id,
                orientation,
                source: template.info.source,
                state: vec![]
            };

            broadcast_all_players(ClientBoundPacket::TerrainSet(TerrainSetPacket {
                pos: pos.clone(),
                tile: client_obj.clone(),
                reason: TileSetReason::DontCare,
            }));
        }
    }
    
    pub fn send_update(&mut self, pos: TilePos) {
        let chunk_pos = pos.chunk_pos;
        if self.is_loaded(chunk_pos) {
            let chunk = self.get_chunk(chunk_pos);
            let lock = chunk.lock();
            let index = Chunk::get_index(&pos);
            if let Some(tile) = &lock.tiles[index] {
                let mut tile_lock = tile.write();
                if let InnerTile::Update(updatable) = &mut tile_lock.info.inner {
                    updatable.send_update(pos, self);
                }
            }
        }
    }

    pub fn sync_tilestate(&mut self, at: TilePos) {
        let tile = self.get_tile_at(at.clone());
        if let Some(tile) = tile {
            let rw = tile.read();
            let state = if let Some(s) = &rw.info.state {
                let mut buf = ByteBuffer::new_le();
                s.save_for_client(&mut buf);
                buf.into_vec()
            } else {
                vec![]
            };
            let client_obj = ToClientObject {
                id: rw.id as u16,
                orientation: rw.info.orientation,
                source: rw.info.source.clone(),
                state
            };

            broadcast_all_players(ClientBoundPacket::TileSet(TileSetPacket {
                pos: at.clone(),
                tile: client_obj.clone(),
                reason: TileSetReason::DontCare,
            }));
        }
    }

    pub fn tick(&mut self) {
        let mut to_tick = Vec::new();
        let mut tiles = Vec::new();
        for chunk in self.loaded_chunks.values() {
            let mut lock = chunk.lock();
            for (tile, pos) in lock.iter_tiles() {
                let tile_lock = tile.read();
                if let InnerTile::Update(_) = &tile_lock.info.inner {
                    tiles.push(tile.clone());
                }
                if tile_lock.info.should_tick {
                    drop(tile_lock);
                    to_tick.push(pos);
                }
            }
        }
        for pos in to_tick {
            self.send_update(pos);
        }
        
        for tile in tiles {
            let mut lock = tile.write();
            match &mut lock.info.inner {
                InnerTile::Static => {}
                InnerTile::Update(update) => {
                    update.end_tick();
                }
            }
        }
    }

    pub fn check_packet(&mut self, packet: ServerBoundPacket, client: &Arc<ClientEndpoint>) -> Option<ServerBoundPacket> {
        let mut players = PLAYERS.write();
        match packet {
            ServerBoundPacket::TileSet(packet) => {
                if let Some(mut tile) = TILE_REGISTRY.create_object(packet.tile_id) {
                    tile.info.orientation = packet.orientation;
                    if let Some((state)) = &mut tile.info.state {
                        if !packet.tile_state.is_empty() {
                            let mut loader = ByteBuffer::from_vec_le(packet.tile_state.clone());
                            if let Err(e) = state.load_from_client(&mut loader) {
                                warn!("Error when loading tile state from client: {e}");
                            }
                        }
                    }
                    let player = players.get(&client.id()).cloned();
                    if let Some(player) = player {
                        let data = {
                            let l = player.lock();
                            l.data.clone()
                        };
                        let reason = TileSetReason::Player(data);

                        drop(players);
                        self.set_tile_at(packet.pos.clone(), tile.to_type(), reason.clone());
                    }
                } else {
                    info!("Received Invalid tile from client with id: {}", packet.tile_id);
                };
            },

            other => return Some(other),
        };
        None
    }

    pub fn name(&self) -> &str {
        &self.meta.name
    }

    pub fn directory(&self) -> &SmartDir {
        &self.directory
    }

    pub fn chunk_directory(&self) -> &SmartDir {
        &self.chunk_directory
    }

    pub fn players_directory(&self) -> &SmartDir {
        &self.players_directory
    }

    pub fn generator(&self) -> &GeneratePipeline {
        &self.generator_pipeline
    }

    pub fn objects(&self) -> &GameObjects {
        &self.objects
    }
}

#[derive(Clone, Savable)]
pub enum TileSetReason {
    DontCare,
    Player(ClientDataPacket),
}

impl Debug for TileSetReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TileSetReason::DontCare => f.write_str("DontCare"),
            TileSetReason::Player(_) => f.write_str("Player"),
        }
    }
}

pub type SingleTileUnit = f64;
pub type TileUnit = (SingleTileUnit, SingleTileUnit); //These are in TILES because we dont know the tilesize on server
pub type PixelUnit = (i32, i32);
pub type TileExtent = (i32, i32);

pub fn resolve_unit(value: TileUnit, tile_size: i32) -> PixelUnit {
    (
        (value.0 * tile_size as f64).floor() as i32,
        (value.1 * tile_size as f64).floor() as i32,
    )
}