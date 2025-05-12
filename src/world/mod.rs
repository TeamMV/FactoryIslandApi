pub mod chunk;
pub mod manager;
pub mod tiles;
pub mod generate;

use crate::event::world::{BeforeChunkGenerateEvent, TileSetEvent};
use crate::event::Event;
use crate::player::PlayerType;
use crate::world::chunk::Chunk;
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
use rand::RngCore;
use std::collections::HashMap;
use std::{env, fs};
use std::fmt::{Debug, Formatter, Write};
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use bytebuffer::ByteBuffer;
use hashbrown::HashSet;
use log::{debug, error};
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::{enum_val, Savable};
use crate::registry::GameObjects;
use crate::world::tiles::tiles::{InnerTile, TileType};

pub const CHUNK_SIZE: i32 = 64;

pub type ChunkPos = (i32, i32);
pub type ChunkType = SaveArc<Mutex<Chunk>>;
pub type WorldType = SaveArc<Mutex<World>>;

pub const META_FILENAME: &str = "meta.sav";

#[derive(Savable)]
pub struct WorldMeta {
    name: String,
    seed: u32
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

        if fs::exists(&full).unwrap_or(false) {
            let world_dir = full.clone();
            let mut chunk_dir = full.clone();
            chunk_dir.push("chunks");
            fs::create_dir_all(&chunk_dir).expect("Failed to create world directory");

            full.push(META_FILENAME);
            if let Ok(mut file) = File::open(&full) {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).ok()?;
                let mut buffer = ByteBuffer::from_vec_le(buffer);
                let meta = WorldMeta::load(&mut buffer).ok()?;
                let seed = meta.seed;

                let this = Self {
                    meta,
                    directory: world_dir,
                    chunk_directory: chunk_dir,
                    loaded_chunks: HashMap::new(),
                    chunk_manager: ChunkManager,
                    generator_pipeline: GeneratePipeline::new(seed),
                    objects: game_objects,
                    arc: Weak::new(),
                };

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
            meta: WorldMeta {
                name: name.to_string(),
                seed,
            },
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

    pub fn check_unload(&mut self, keep: HashSet<ChunkPos>) {
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

    pub fn set_tile_at(&mut self, pos: TilePos, tile: TileType, event_bus: &mut EventBus<Event>, reason: TileSetReason) {
        let event = TileSetEvent {
            has_been_cancelled: false,
            world: self.arc.upgrade().unwrap().clone(),
            tile: tile.clone(),
            pos: pos.clone(),
            reason,
        };
        let mut event = Event::TileSetEvent(event);
        event_bus.dispatch(&mut event);

        let chunk = self.get_chunk(pos.chunk_pos, event_bus);
        let mut lock = chunk.lock();
        lock.tiles[Chunk::get_index(&pos)] = Some(tile);
    }
    
    pub fn send_update(&mut self, pos: TilePos, event_bus: &mut EventBus<Event>) {
        if self.is_loaded(pos.chunk_pos) { 
            let chunk = self.get_chunk(pos.chunk_pos, event_bus);
            let lock = chunk.lock();
            let index = Chunk::get_index(&pos);
            if let Some(tile) = &lock.tiles[index] {
                let mut tile_lock = tile.write();
                if let InnerTile::Update(updatable) = &mut tile_lock.info.inner {
                    updatable.send_update(pos, self, event_bus);
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
    Player(PlayerType),
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