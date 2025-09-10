use mvutils::lazy;
use crate::multitile::{MultiTile, MultiTileCreateInfo};
use crate::registry::{GameObjects, Registry};
use crate::registry::tiles::Tiles;

lazy! {
    pub static MULTI_REGISTRY: Registry<MultiTile> = Registry::new();
}

macro_rules! define_multis {
    ($struct_name:ident, $func_name:ident, $go_ident:ident => [$($tile_name:ident = $tile_init:expr),* $(,)?]) => {
        #[derive(Clone)]
        pub struct $struct_name {
            $(pub $tile_name: usize),*
        }

        pub fn $func_name($go_ident: &Tiles) -> $struct_name {
            $struct_name {
                $(
                    $tile_name: MULTI_REGISTRY.register($tile_init),
                )*
            }
        }
    };
}

define_multis!(MultiTiles, register_all, tiles => [
    hello_multi_block = MultiTileCreateInfo::new((2, 3), &[(tiles.conveyor, 3), (tiles.wood, 2), (tiles.lamp, 1)]),
]);