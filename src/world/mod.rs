pub mod chunk;
pub mod manager;
pub mod tiles;
pub mod generate;

use crate::event::world::{BeforeChunkGenerateEvent, TileSetEvent};
use crate::event::Event;
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
use bytebuffer::ByteBuffer;
use hashbrown::HashSet;
use log::{debug, error, info, warn};
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::{enum_val, Savable};
use mvutils::unsafe_utils::Unsafe;
use crate::FactoryIsland;
use crate::registry::GameObjects;
use crate::server::ClientBoundPacket;
use crate::server::packets::common::ClientDataPacket;
use crate::server::packets::world::TileSetPacket;
use crate::world::tiles::tiles::{InnerTile, TileType};

pub const CHUNK_SIZE: i32 = 64;

pub type ChunkPos = (i32, i32);
pub type ChunkType = SaveArc<Mutex<Chunk>>;
pub type WorldType = SaveArc<Mutex<World>>;

pub const META_FILENAME: &str = "meta.sav";

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
    directory: PathBuf,
    chunk_directory: PathBuf,
    loaded_chunks: HashMap<ChunkPos, ChunkType>,
    chunk_manager: ChunkManager,
    generator_pipeline: GeneratePipeline,
    objects: GameObjects,
    
    arc: Weak<Mutex<World>>
}

impl World {
    pub fn get_main(game_objects: GameObjects, event_bus: &mut EventBus<Event>) -> WorldType {
        if let Some(world) = World::load("main", game_objects.clone(), event_bus) {
            world
        } else {
            let rng_seed = rand::rng().next_u32();
            World::new("main", rng_seed, game_objects)
        }
    }
}

impl World {
    pub fn load(name: &str, game_objects: GameObjects, event_bus: &mut EventBus<Event>) -> Option<WorldType> {
        let dir_name = name.replace(' ', "_");
        let appdata = env::var("APPDATA").expect("Failed to get APPDATA environment variable");
        let mut full = PathBuf::from(appdata);
        full.push(manager::PATH);
        full.push(dir_name);

        if fs::exists(&full).unwrap_or(false) {
            let world_dir = full.clone();
            let mut chunk_dir = full.clone();
            chunk_dir.push("chunks");
            fs::create_dir_all(&chunk_dir).expect("Failed to create world directory");

            full.push(META_FILENAME);
            if !full.exists() {
                warn!("Meta file is not available, recreating it with a new seed");
                if let Ok(mut file) = File::create(&full) {
                    let meta = WorldMeta::new(name, rng().next_u32());
                    let mut buffer = ByteBuffer::new_le();
                    meta.save(&mut buffer);
                    if let Err(e) = file.write_all(buffer.as_bytes()) {
                        error!("Error when recreating meta file: {e:?}");
                    }
                }
            }
            if let Ok(mut file) = File::open(&full) {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).ok()?;
                let mut buffer = ByteBuffer::from_vec_le(buffer);
                let meta = WorldMeta::load(&mut buffer).ok()?;
                let seed = meta.seed;

                let mut this = Self {
                    meta,
                    directory: world_dir,
                    chunk_directory: chunk_dir,
                    loaded_chunks: HashMap::new(),
                    chunk_manager: ChunkManager,
                    generator_pipeline: GeneratePipeline::new(seed),
                    objects: game_objects,
                    arc: Weak::new(),
                };

                for chunk_pos in this.meta.forced_chunks.clone() {
                    let _ = this.get_chunk(chunk_pos, event_bus);
                }

                let arc = SaveArc::new(Mutex::new(this));
                let clone = arc.clone();
                let mut lock = arc.lock();
                lock.arc = Arc::downgrade(clone.arc());
                drop(lock);
                Some(arc)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn save(&mut self) {
        let mut full = self.directory.clone();
        full.push(META_FILENAME);
        let mut buffer = ByteBuffer::new_le();
        self.meta.save(&mut buffer);
        if let Err(e) = fs::write(full, buffer.as_bytes()) {
            error!("Error writing meta file: {e:?}");
        }
        for chunk in self.loaded_chunks.values() {
            let c = chunk.lock();
            self.chunk_manager.try_save_chunk(self, &*c);
        }
    }

    pub fn new(name: &str, seed: u32, game_objects: GameObjects) -> WorldType {
        let dir_name = name.replace(' ', "_");
        let appdata = env::var("APPDATA").expect("Failed to get APPDATA environment variable");
        let mut full = PathBuf::from(appdata);
        full.push(manager::PATH);
        full.push(dir_name);
        let mut chunk_dir = full.clone();
        chunk_dir.push("chunks");
        fs::create_dir_all(&chunk_dir).expect("Failed to create world directory");
        let this = Self {
            meta: WorldMeta::new(name, seed),
            directory: full,
            chunk_directory: chunk_dir,
            loaded_chunks: HashMap::new(),
            chunk_manager: ChunkManager {},
            generator_pipeline: GeneratePipeline::new(seed),
            objects: game_objects,
            arc: Weak::new(),
        };
        let arc = SaveArc::new(Mutex::new(this));
        let clone = arc.clone();
        let mut lock = arc.lock();
        lock.arc = Arc::downgrade(clone.arc());
        drop(lock);
        arc
    }

    pub fn get_chunk(&mut self, chunk_pos: ChunkPos, event_bus: &mut EventBus<Event>) -> ChunkType {
        if let Some(chunk) = self.loaded_chunks.get(&chunk_pos) {
            return chunk.clone();
        }

        let loaded = self.chunk_manager.try_load_chunk(self, chunk_pos, event_bus);
        if let Some(chunk) = loaded {
            //chunk loaded
            let arc = SaveArc::new(Mutex::new(chunk));
            self.loaded_chunks.insert(chunk_pos, arc.clone());
            arc
        } else {
            let mut chunk = Chunk::new(chunk_pos, self.meta.seed);
            //generate new chunk if loading fails
            debug!("Generating new chunk at {chunk_pos:?}");
            let mut chunk = chunk.generate_terrain(&self.generator_pipeline, event_bus, &self.objects);
            let mut chunk = chunk.generate(&self.generator_pipeline, event_bus, &self.objects);
            let pos = chunk.position;
            let arc = SaveArc::new(Mutex::new(chunk));
            self.loaded_chunks.insert(pos, arc.clone());
            arc
        }
    }

    pub fn is_loaded(&self, pos: ChunkPos) -> bool {
        self.loaded_chunks.contains_key(&pos)
    }

    pub fn exists_file(&self, pos: ChunkPos) -> bool {
        let mut path = self.chunk_directory.clone();
        let filename = format!("c{}_{}.chunk", pos.0, pos.1);
        path.push(&filename);
        fs::exists(path).unwrap_or(false)
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

    pub fn get_tile_at(&mut self, pos: TilePos, event_bus: &mut EventBus<Event>) -> Option<TileType> {
        let chunk = self.get_chunk((pos.world_chunk_x, pos.world_chunk_z), event_bus);
        let mut lock = chunk.lock();
        lock.tiles[Chunk::get_index(&pos)].clone()
    }

    pub fn set_tile_at(&mut self, pos: TilePos, tile: TileType, reason: TileSetReason, fi: &mut FactoryIsland) {
        let event = TileSetEvent {
            has_been_cancelled: false,
            world: self.arc.upgrade().unwrap().clone(),
            tile: tile.clone(),
            pos: pos.clone(),
            reason: reason.clone(),
        };
        let mut event = Event::TileSetEvent(event);
        fi.event_bus.dispatch(&mut event);

        let chunk = self.get_chunk(pos.chunk_pos, &mut fi.event_bus);
        let mut lock = chunk.lock();
        lock.tiles[Chunk::get_index(&pos)] = Some(tile.clone());
        drop(lock);

        let rw = tile.read();
        let client_obj = ToClientObject {
            id: rw.id as u16,
            orientation: rw.info.orientation,
            source: rw.info.source.clone(),
            state: rw.info.state.as_ref().map(|s| s.client_state()).unwrap_or_default()
        };

        for player in fi.players.values() {
            let lock = player.lock();
            if let Some(endpoint) = lock.client_endpoint() {
                endpoint.send(ClientBoundPacket::TileSet(TileSetPacket {
                    pos: pos.clone(),
                    tile: client_obj.clone(),
                    reason: TileSetReason::DontCare,
                }));
            }
        }
    }
    
    pub fn send_update(&mut self, pos: TilePos, event_bus: &mut EventBus<Event>, fi: &FactoryIsland) {
        if self.is_loaded(pos.chunk_pos) { 
            let chunk = self.get_chunk(pos.chunk_pos, event_bus);
            let lock = chunk.lock();
            let index = Chunk::get_index(&pos);
            if let Some(tile) = &lock.tiles[index] {
                let mut tile_lock = tile.write();
                if let InnerTile::Update(updatable) = &mut tile_lock.info.inner {
                    let updatable = unsafe { Unsafe::cast_mut_static(updatable) };
                    drop(tile_lock);
                    drop(lock);
                    updatable.send_update(pos, self, event_bus, fi);
                }
            }
        }
    }

    pub fn sync_tilestate(&mut self, at: TilePos, event_bus: &mut EventBus<Event>, fi: &FactoryIsland) {
        let tile = self.get_tile_at(at.clone(), event_bus);
        if let Some(tile) = tile {
            let rw = tile.read();
            let client_obj = ToClientObject {
                id: rw.id as u16,
                orientation: rw.info.orientation,
                source: rw.info.source.clone(),
                state: rw.info.state.as_ref().map(|s| s.client_state()).unwrap_or_default()
            };

            for player in fi.players.values() {
                let lock = player.lock();
                if let Some(endpoint) = lock.client_endpoint() {
                    endpoint.send(ClientBoundPacket::TileSet(TileSetPacket {
                        pos: at.clone(),
                        tile: client_obj.clone(),
                        reason: TileSetReason::DontCare,
                    }));
                }
            }
        }
    }

    pub fn tick(&mut self, event_bus: &mut EventBus<Event>, fi: &FactoryIsland) {
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
            self.send_update(pos, event_bus, fi);
        }
        
        for tile in tiles {
            let mut lock = tile.write();
            match &mut lock.info.inner {
                InnerTile::Static => {}
                InnerTile::Update(update) => {
                    update.end_tick();
                }
                InnerTile::PowerGenerator(generator) => {
                    generator.end_tick();
                }
                InnerTile::PowerTransformer(transformer) => {
                    transformer.end_tick();
                }
                InnerTile::PowerConsumer(consumer) => {
                    consumer.end_tick();
                }
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.meta.name
    }

    pub fn directory(&self) -> &PathBuf {
        &self.directory
    }

    pub fn chunk_directory(&self) -> &PathBuf {
        &self.chunk_directory
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

pub type TileUnit = (f64, f64); //These are in TILES because we dont know the tilesize on server
pub type PixelUnit = (i32, i32);

pub fn resolve_unit(value: TileUnit, tile_size: i32) -> PixelUnit {
    (
        (value.0 * tile_size as f64).floor() as i32,
        (value.1 * tile_size as f64).floor() as i32,
    )
}