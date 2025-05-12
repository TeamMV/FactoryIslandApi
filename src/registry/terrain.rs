use crate::registry::Registry;
use crate::world::tiles::terrain::{TerrainTile, TerrainTileInfo};
use mvutils::lazy;

lazy! {
    pub static TERRAIN_REGISTRY: Registry<TerrainTile> = Registry::new();
}

macro_rules! define_tiles {
    ($struct_name:ident, $func_name:ident, [$($tile_name:ident),* $(,)?]) => {
        #[derive(Clone)]
        pub struct $struct_name {
            $(pub $tile_name: usize),*
        }

        pub fn $func_name() -> $struct_name {
            $struct_name {
                $(
                    $tile_name: TERRAIN_REGISTRY.register(TerrainTileInfo::vanilla()),
                )*
            }
        }
    };
}

define_tiles!(TerrainTiles, register_all, [
    void,
    water,
    sand,
    grass,
    stone,
]);