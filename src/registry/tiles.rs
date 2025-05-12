use crate::registry::Registry;
use crate::world::tiles::tiles::{Tile, TileInfo};
use mvutils::lazy;
use crate::world::tiles::implementations::TestUpdateTile;

lazy! {
    pub static TILE_REGISTRY: Registry<Tile> = Registry::new();
}

macro_rules! define_tiles {
    ($struct_name:ident, $func_name:ident, [$($tile_name:ident = $tile_init:expr),* $(,)?]) => {
        #[derive(Clone)]
        pub struct $struct_name {
            $(pub $tile_name: usize),*
        }

        pub fn $func_name() -> $struct_name {
            $struct_name {
                $(
                    $tile_name: TILE_REGISTRY.register($tile_init),
                )*
            }
        }
    };
}

define_tiles!(Tiles, register_all, [
    air = TileInfo::vanilla_static(),
    wood = TileInfo::vanilla_static(),
    test = TileInfo::vanilla_update(TestUpdateTile::new())
]);