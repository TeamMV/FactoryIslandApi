use crate::registry::Registerable;
use crate::world::tiles::resources::ClientTileRes;
pub use crate::world::tiles::ObjectSource;
use mvutils::save::Savable;
use mvutils::Savable;
use std::fmt::{Debug, Formatter, Write};

#[derive(Clone, Savable)]
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
pub struct TerrainTileInfo {
    pub source: ObjectSource
}

impl TerrainTileInfo {
    pub const fn vanilla() -> Self {
        Self { source: ObjectSource::Vanilla }
    }
    
    pub fn from_mod(modid: &str, res: ClientTileRes) -> Self {
        Self { source: ObjectSource::Mod(modid.to_string(), res) }
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