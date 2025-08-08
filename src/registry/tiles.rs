use crate::registry::Registry;
use crate::world::tiles::tiles::{ObjControl, Tile, TileInfo, TileState};
use mvutils::lazy;
use crate::{leak, this};
use crate::world::tiles::implementations::lamp::LampTile;
use crate::world::tiles::update::UpdateTile;

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
    lamp = TileInfo::vanilla_update_state(LampTile::new(), LampTile::create_update_trait(), LampTile::create_state_trait(), LampTile::create_oc_trait()),
]);