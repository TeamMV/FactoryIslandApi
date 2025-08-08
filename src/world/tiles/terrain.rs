use crate::registry::{ObjectSource, Registerable};
use mvutils::save::Savable;
use mvutils::Savable;
use std::fmt::{Debug, Formatter, Write};
use abi_stable::traits::IntoReprC;
use crate::world::tiles::Orientation;

#[derive(Clone, Savable)]
#[repr(C)]
pub struct TerrainTile {
    pub id: usize,
    pub info: TerrainTileInfo
}

impl TerrainTile {
    pub const fn void() -> Self {
        Self {
            id: 0,
            info: TerrainTileInfo::vanilla(),
        }
    }

    pub(crate) const fn new(id: usize, info: TerrainTileInfo) -> Self {
        Self {
            id,
            info,
        }
    }
}

#[derive(Clone, Savable)]
#[repr(C)]
pub struct TerrainTileInfo {
    pub source: ObjectSource
}

impl TerrainTileInfo {
    pub const fn vanilla() -> Self {
        Self { source: ObjectSource::Vanilla }
    }
    
    pub fn from_mod(modid: &str) -> Self {
        Self { source: ObjectSource::Mod(modid.to_string().into_c()) }
    }
}

impl Registerable for TerrainTile {
    type CreateInfo = TerrainTileInfo;

    fn with_id(id: usize, info: Self::CreateInfo) -> Self {
        Self::new(id, info)
    }
}

impl Debug for TerrainTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("id: {}", self.id))
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct WorldTerrain {
    pub id: u16,
    pub orientation: Orientation
}