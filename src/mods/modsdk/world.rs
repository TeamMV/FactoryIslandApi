use abi_stable::std_types::RStr;
use abi_stable::traits::IntoReprC;
use mvengine::utils::savers::SaveArc;
use parking_lot::{Mutex, RwLock};
use crate::mods::modsdk::{store_lock, MOpt, MUniqueAccess, ModCtx};
use crate::mods::modsdk::player::MPlayer;
use crate::registry;
use crate::registry::ObjectSource;
use crate::world::chunk::Chunk;
use crate::world::tiles::tiles::{Tile, TileType};
use crate::world::{ChunkPos, TileSetReason, TileUnit, World};
use crate::world::tiles::Orientation;
use crate::world::tiles::pos::TilePos;
use crate::world::tiles::terrain::{TerrainTile, WorldTerrain};

pub type MChunk = *mut Chunk;
pub type MWorld = *mut World;
pub type MTile = *mut Tile;

#[derive(Clone, Debug)]
#[repr(C)]
pub enum MTileSetReason {
    DontCare,
    Player(MPlayer)
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Hash, Debug)]
#[repr(C)]
pub struct MTileUnit {
    pub x: f64,
    pub z: f64
}

impl MTileUnit {
    pub(crate) fn from_normal(tu: TileUnit) -> Self {
        Self {
            x: tu.0,
            z: tu.1,
        }
    }

    pub(crate) fn to_normal(&self) -> TileUnit {
        (self.x, self.z)
    }

    pub fn new(x: f64, z: f64) -> Self {
        Self {
            x,
            z,
        }
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Hash, Debug)]
#[repr(C)]
pub struct MChunkPos {
    pub x: i32,
    pub z: i32
}

impl MChunkPos {
    pub(crate) fn from_normal(tu: ChunkPos) -> Self {
        Self {
            x: tu.0,
            z: tu.1,
        }
    }

    pub(crate) fn to_normal(&self) -> ChunkPos {
        (self.x, self.z)
    }

    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z,
        }
    }
}

#[no_mangle]
pub extern "C" fn fim_world_get_chunk(mut world: MWorld, pos: MChunkPos) -> MUniqueAccess<MChunk> {
    let chunk = world.get_chunk(pos.to_normal());
    let chunk_lock = chunk.lock();
    let ptr: MChunk = &mut *chunk_lock;
    let handle = store_lock(chunk_lock);
    MUniqueAccess {
        lock_handle: handle,
        data: ptr,
    }
}

#[no_mangle]
pub extern "C" fn fim_world_unload_chunk(mut world: MWorld, pos: MChunkPos) {
    world.unload_chunk(pos.to_normal());
}

#[no_mangle]
pub extern "C" fn fim_world_name(world: MWorld) -> RStr {
    world.name().into_c()
}

#[no_mangle]
pub extern "C" fn fim_world_get_tile_at(mut world: MWorld, pos: TilePos) -> MOpt<MUniqueAccess<MTile>> {
    if let Some(tile) = world.get_tile_at(pos) {
        let mut tile_lock = tile.write();
        let ptr: MTile = &mut *tile_lock;
        let handle = store_lock(tile_lock);
        MOpt::Some(MUniqueAccess {
            lock_handle: handle,
            data: ptr,
        })
    } else {
        MOpt::None
    }
}

#[no_mangle]
pub extern "C" fn fim_world_set_terrain_at(mut world: MWorld, pos: TilePos, reason: MTileSetReason, tile: WorldTerrain) {
    world.set_terrain_at(pos, tile, TileSetReason::from_m(reason));
}

#[no_mangle]
pub extern "C" fn fim_world_get_terrain_at(mut world: MWorld, pos: TilePos) -> WorldTerrain {
    world.get_terrain_at(pos)
}

#[no_mangle]
pub extern "C" fn fim_world_set_tile_at(mut world: MWorld, pos: TilePos, reason: MTileSetReason, tile: Tile) {
    let arc = SaveArc::new(RwLock::new(tile));
    world.set_tile_at(pos, arc, TileSetReason::from_m(reason));
}

#[no_mangle]
pub extern "C" fn fim_create_tile(id: usize, orientation: Orientation) -> MOpt<Tile> {
    if let Some(mut template) = registry::tiles::TILE_REGISTRY.create_object(id) {
        template.info.orientation = orientation;
        MOpt::Some(template)
    } else {
        MOpt::None
    }
}

#[no_mangle]
pub extern "C" fn fim_create_terrain(id: usize) -> MOpt<WorldTerrain> {
    if let Some(template) = registry::terrain::TERRAIN_REGISTRY.create_object(id) {
        MOpt::Some(WorldTerrain {
            id: template.id as u16,
            orientation: Orientation::North,
        })
    } else {
        MOpt::None
    }
}

#[no_mangle]
pub extern "C" fn fim_create_terrain_unchecked(id: usize) -> WorldTerrain {
    WorldTerrain {
        id: id as u16,
        orientation: Orientation::North,
    }
}

#[no_mangle]
pub extern "C" fn fim_world_is_chunk_loaded(world: MWorld, pos: MChunkPos) -> bool {
    world.is_loaded(pos.to_normal())
}

#[no_mangle]
pub extern "C" fn fim_world_exists_file(world: MWorld, pos: MChunkPos) -> bool {
    world.exists_file(pos.to_normal())
}