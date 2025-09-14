use crate::registry::{Registerable};
use mvutils::save::Savable;
use mvutils::Savable;
use std::fmt::{Debug, Formatter, Write};
use abi_stable::traits::IntoReprC;
use crate::world::tiles::Orientation;

#[derive(Clone, Savable)]
#[repr(C)]
pub struct TerrainTile {
    pub id: usize,
}

impl TerrainTile {
    pub const fn void() -> Self {
        Self {
            id: 0,
        }
    }

    pub(crate) const fn new(id: usize) -> Self {
        Self {
            id,
        }
    }
}

impl Registerable for TerrainTile {
    type CreateInfo = ();

    fn with_id(id: usize, _: Self::CreateInfo) -> Self {
        Self::new(id)
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